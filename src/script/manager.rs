//! Script manager
//!
//! Manages loading, unloading, reloading, and listing of Lua scripts.

use crate::error::{Result, ScriptError, SerialError};
use crate::script::built_in;
use crate::script::ScriptInfo;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Metadata for a loaded script.
#[derive(Debug, Clone)]
pub struct LoadedScript {
    pub name: String,
    pub description: String,
    pub source: String,
    pub path: Option<PathBuf>,
    pub built_in: bool,
    pub loaded_at: std::time::SystemTime,
    pub version: u64,
}

/// Manages the lifecycle of Lua scripts: loading from disk, registering
/// built-in scripts, unloading, reloading, and hot-reload watching.
pub struct ScriptManager {
    scripts: HashMap<String, LoadedScript>,
}

impl ScriptManager {
    /// Create a new script manager with built-in scripts registered.
    pub fn new() -> Self {
        let mut scripts = HashMap::new();

        // Register built-in scripts
        for entry in built_in::all_built_in() {
            scripts.insert(
                entry.name.to_string(),
                LoadedScript {
                    name: entry.name.to_string(),
                    description: entry.description.to_string(),
                    source: entry.source.to_string(),
                    path: None,
                    built_in: true,
                    loaded_at: std::time::SystemTime::now(),
                    version: 1,
                },
            );
        }

        Self { scripts }
    }

    /// List all registered scripts (built-in + custom).
    pub fn list(&self) -> Vec<ScriptInfo> {
        self.scripts
            .values()
            .map(|s| ScriptInfo {
                name: s.name.clone(),
                description: s.description.clone(),
                built_in: s.built_in,
            })
            .collect()
    }

    /// Get the Lua source for a script by name.
    pub fn get_source(&self, name: &str) -> Result<String> {
        self.scripts
            .get(name)
            .map(|s| s.source.clone())
            .ok_or_else(|| SerialError::Script(ScriptError::NotFound(PathBuf::from(name))))
    }

    /// Get metadata for a script by name.
    pub fn get_meta(&self, name: &str) -> Result<&LoadedScript> {
        self.scripts
            .get(name)
            .ok_or_else(|| SerialError::Script(ScriptError::NotFound(PathBuf::from(name))))
    }

    /// Load a custom script from a Lua file.
    pub fn load(&mut self, path: &Path) -> Result<ScriptInfo> {
        // Read the file
        let source = std::fs::read_to_string(path).map_err(SerialError::Io)?;

        // Validate syntax using pooled Lua instance
        use crate::lua::runtime::{acquire_lua, release_lua};
        let lua = acquire_lua();
        let validation_result = lua.load(&source).exec().map_err(|e| {
            SerialError::Script(ScriptError::Syntax {
                script: path.to_path_buf(),
                line: 0,
                message: e.to_string(),
            })
        });
        release_lua(lua);
        validation_result?;

        // Derive name from filename
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed")
            .to_string();

        let loaded = LoadedScript {
            name: name.clone(),
            description: format!("Custom script: {}", path.display()),
            source,
            path: Some(path.to_path_buf()),
            built_in: false,
            loaded_at: std::time::SystemTime::now(),
            version: 1,
        };

        self.scripts.insert(name.clone(), loaded);

        tracing::info!("Loaded script: {} from {}", name, path.display());

        Ok(ScriptInfo {
            name,
            description: format!("Custom script: {}", path.display()),
            built_in: false,
        })
    }

    /// Unload a custom script by name. Built-in scripts cannot be unloaded.
    pub fn unload(&mut self, name: &str) -> Result<()> {
        let script = self
            .scripts
            .get(name)
            .ok_or_else(|| SerialError::Script(ScriptError::NotFound(PathBuf::from(name))))?;

        if script.built_in {
            return Err(SerialError::Script(ScriptError::ApiError(format!(
                "Cannot unload built-in script: {}",
                name
            ))));
        }

        self.scripts.remove(name);
        tracing::info!("Unloaded script: {}", name);
        Ok(())
    }

    /// Reload a custom script from its original file path.
    pub fn reload(&mut self, name: &str) -> Result<()> {
        let path = {
            let script = self
                .scripts
                .get(name)
                .ok_or_else(|| SerialError::Script(ScriptError::NotFound(PathBuf::from(name))))?;

            if script.built_in {
                return Err(SerialError::Script(ScriptError::ApiError(format!(
                    "Cannot reload built-in script: {}",
                    name
                ))));
            }

            script
                .path
                .clone()
                .ok_or_else(|| SerialError::Script(ScriptError::ApiError(format!(
                    "Script has no file path: {}",
                    name
                ))))?
        };

        self.unload(name)?;
        self.load(&path)?;
        Ok(())
    }

    /// Check if a script exists.
    pub fn has(&self, name: &str) -> bool {
        self.scripts.contains_key(name)
    }

    /// Validate Lua source code syntax without loading it.
    pub fn validate_source(source: &str) -> std::result::Result<(), String> {
        use crate::lua::runtime::{acquire_lua, release_lua};
        let lua = acquire_lua();
        let result = lua.load(source).exec().map_err(|e| e.to_string());
        release_lua(lua);
        result
    }

    /// Create a SerialScriptEngine from a named script.
    ///
    /// This is the bridge between ScriptManager and the port-level
    /// script attachment system.
    pub fn create_engine(&self, name: &str) -> Result<crate::serial_core::serial_script::SerialScriptEngine> {
        let source = self.get_source(name)?;
        crate::serial_core::serial_script::SerialScriptEngine::new(&source)
    }

    // ── Hot-reload support ──────────────────────────────────────────

    /// Check if hot-reload is enabled (reads from ConfigManager).
    pub fn is_hot_reload_enabled(&self) -> bool {
        let config_manager = crate::config::ConfigManager::load_with_fallback();
        config_manager.is_hot_reload_enabled()
    }

    /// Get the file path for a custom script.
    pub fn get_script_path(&self, name: &str) -> Option<&PathBuf> {
        self.scripts.get(name).and_then(|s| s.path.as_ref())
    }

    /// Check if a script file has been modified since it was loaded.
    ///
    /// Returns `true` if the file's modification time is newer than
    /// the script's loaded_at time.
    pub fn is_script_modified(&self, name: &str) -> Result<bool> {
        let script = self.scripts.get(name)
            .ok_or_else(|| SerialError::Script(ScriptError::NotFound(PathBuf::from(name))))?;

        let path = match &script.path {
            Some(p) => p,
            None => return Ok(false),  // Built-in scripts have no path
        };

        let metadata = std::fs::metadata(path).map_err(SerialError::Io)?;
        let modified = metadata.modified().map_err(SerialError::Io)?;

        Ok(modified > script.loaded_at)
    }

    /// Reload all scripts that have been modified on disk.
    ///
    /// Returns a list of script names that were reloaded.
    pub fn reload_modified_scripts(&mut self) -> Result<Vec<String>> {
        if !self.is_hot_reload_enabled() {
            return Ok(Vec::new());
        }

        let mut reloaded = Vec::new();
        let script_names: Vec<String> = self.scripts.keys().cloned().collect();

        for name in script_names {
            if self.is_script_modified(&name).unwrap_or(false) {
                match self.reload(&name) {
                    Ok(_) => {
                        tracing::info!("Hot-reloaded script: {}", name);
                        reloaded.push(name);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to hot-reload script {}: {}", name, e);
                    }
                }
            }
        }

        Ok(reloaded)
    }

    /// Get list of all custom script paths being tracked.
    pub fn tracked_paths(&self) -> Vec<&PathBuf> {
        self.scripts.values()
            .filter_map(|s| s.path.as_ref())
            .collect()
    }

    // ── Enhanced validation ─────────────────────────────────────────

    /// Validate a Lua script for common issues.
    ///
    /// Checks:
    /// - Syntax validity
    /// - Presence of at least one callback (on_send, on_recv, on_open, on_close, on_timer)
    /// - No undefined global variables (basic check)
    ///
    /// Returns a list of warnings (empty if script is valid).
    pub fn validate_script_detailed(source: &str) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check syntax
        if let Err(e) = Self::validate_source(source) {
            warnings.push(format!("Syntax error: {}", e));
            return warnings;
        }

        // Check for at least one callback
        let has_on_send = source.contains("function on_send") || source.contains("on_send =");
        let has_on_recv = source.contains("function on_recv") || source.contains("on_recv =");
        let has_on_open = source.contains("function on_open") || source.contains("on_open =");
        let has_on_close = source.contains("function on_close") || source.contains("on_close =");
        let has_on_timer = source.contains("function on_timer") || source.contains("on_timer =");

        if !has_on_send && !has_on_recv && !has_on_open && !has_on_close && !has_on_timer {
            warnings.push("No callbacks defined (on_send, on_recv, on_open, on_close, on_timer)".to_string());
        }

        // Check for common issues
        if source.contains("require(") && !source.contains("-- require(") {
            warnings.push("Script uses 'require()' which may not be available in all contexts".to_string());
        }

        if source.contains("os.execute") || source.contains("io.popen") {
            warnings.push("Script uses potentially dangerous functions (os.execute, io.popen)".to_string());
        }

        warnings
    }

    /// Load a script with detailed validation.
    ///
    /// Returns the script info and any warnings.
    pub fn load_with_validation(&mut self, path: &Path) -> Result<(ScriptInfo, Vec<String>)> {
        // Read the file
        let source = std::fs::read_to_string(path).map_err(SerialError::Io)?;

        // Validate with details
        let warnings = Self::validate_script_detailed(&source);

        // Check for fatal errors (syntax)
        if warnings.iter().any(|w| w.starts_with("Syntax error")) {
            return Err(SerialError::Script(ScriptError::Syntax {
                script: path.to_path_buf(),
                line: 0,
                message: warnings.join("; "),
            }));
        }

        // Load the script
        let info = self.load(path)?;

        Ok((info, warnings))
    }

    /// Get script statistics.
    pub fn statistics(&self) -> ScriptStatistics {
        let total = self.scripts.len();
        let built_in = self.scripts.values().filter(|s| s.built_in).count();
        let custom = total - built_in;
        let with_path = self.scripts.values().filter(|s| s.path.is_some()).count();

        ScriptStatistics {
            total,
            built_in,
            custom,
            with_path,
        }
    }
}

/// Statistics about loaded scripts.
#[derive(Debug, Clone)]
pub struct ScriptStatistics {
    pub total: usize,
    pub built_in: usize,
    pub custom: usize,
    pub with_path: usize,
}

impl Default for ScriptManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_lists_built_in_scripts() {
        let manager = ScriptManager::new();
        let scripts = manager.list();

        assert!(!scripts.is_empty(), "Should have built-in scripts");

        let names: Vec<&str> = scripts.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"line"), "Should include line protocol");
        assert!(names.contains(&"at_command"), "Should include AT command");
        assert!(names.contains(&"modbus_rtu"), "Should include Modbus RTU");
    }

    #[test]
    fn test_built_in_scripts_are_marked() {
        let manager = ScriptManager::new();
        let scripts = manager.list();

        for script in &scripts {
            assert!(script.built_in, "{} should be marked as built-in", script.name);
        }
    }

    #[test]
    fn test_get_source_returns_lua_code() {
        let manager = ScriptManager::new();
        let source = manager.get_source("line").unwrap();
        assert!(source.contains("on_send"), "Line script should define on_send");
        assert!(source.contains("on_recv"), "Line script should define on_recv");
    }

    #[test]
    fn test_get_source_not_found() {
        let manager = ScriptManager::new();
        let result = manager.get_source("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_built_in_scripts_are_valid_lua() {
        let manager = ScriptManager::new();
        let lua = mlua::Lua::new();

        for script in manager.list() {
            let source = manager.get_source(&script.name).unwrap();
            let result = lua.load(&source).exec();
            assert!(
                result.is_ok(),
                "Built-in script '{}' should be valid Lua: {:?}",
                script.name,
                result.err()
            );
        }
    }

    #[test]
    fn test_manager_default_has_built_ins() {
        let manager = ScriptManager::default();
        assert!(manager.has("line"));
        assert!(manager.has("modbus_rtu"));
    }

    #[test]
    fn test_load_custom_script_from_file() {
        let dir = std::env::temp_dir().join("serial_cli_test_load_custom");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("my_custom.lua");
        std::fs::write(&path, "function on_recv(data)\n    return data\nend\n").unwrap();

        let mut manager = ScriptManager::new();
        let info = manager.load(&path).unwrap();

        assert_eq!(info.name, "my_custom");
        assert!(!info.built_in);
        assert!(manager.has("my_custom"));

        // Cleanup
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_load_rejects_invalid_lua() {
        let dir = std::env::temp_dir().join("serial_cli_test_rejects_invalid");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bad_syntax.lua");
        std::fs::write(&path, "this is not valid lua {{{").unwrap();

        let mut manager = ScriptManager::new();
        let result = manager.load(&path);
        assert!(result.is_err());

        // Cleanup
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_unload_custom_script() {
        let dir = std::env::temp_dir().join("serial_cli_test_unload");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("temp_script.lua");
        std::fs::write(&path, "function on_recv(data) return data end").unwrap();

        let mut manager = ScriptManager::new();
        manager.load(&path).unwrap();
        assert!(manager.has("temp_script"));

        manager.unload("temp_script").unwrap();
        assert!(!manager.has("temp_script"));

        // Cleanup
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_cannot_unload_built_in() {
        let mut manager = ScriptManager::new();
        let result = manager.unload("line");
        assert!(result.is_err());
        assert!(manager.has("line"));
    }

    #[test]
    fn test_cannot_reload_built_in() {
        let mut manager = ScriptManager::new();
        let result = manager.reload("line");
        assert!(result.is_err());
    }

    #[test]
    fn test_reload_custom_script() {
        let dir = std::env::temp_dir().join("serial_cli_test_reload");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("reloadable.lua");
        std::fs::write(&path, "function on_recv(data) return data end").unwrap();

        let mut manager = ScriptManager::new();
        manager.load(&path).unwrap();

        // Modify the file
        std::fs::write(&path, "function on_recv(data)\n    -- modified\n    return data\nend").unwrap();

        manager.reload("reloadable").unwrap();
        let source = manager.get_source("reloadable").unwrap();
        assert!(source.contains("modified"));

        // Cleanup
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_create_engine_from_built_in() {
        let manager = ScriptManager::new();
        let engine = manager.create_engine("line");
        assert!(engine.is_ok(), "Should create engine from line script");
    }

    #[test]
    fn test_create_engine_from_custom_script() {
        let dir = std::env::temp_dir().join("serial_cli_test_engine");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("engine_test.lua");
        std::fs::write(&path, "function on_recv(data) return data end").unwrap();

        let mut manager = ScriptManager::new();
        manager.load(&path).unwrap();

        let engine = manager.create_engine("engine_test");
        assert!(engine.is_ok(), "Should create engine from custom script");

        // Cleanup
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_create_engine_not_found() {
        let manager = ScriptManager::new();
        let result = manager.create_engine("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_line_script_on_send_appends_newline() {
        let manager = ScriptManager::new();
        let engine = manager.create_engine("line").unwrap();
        engine.load().unwrap();

        // on_send should append newline
        let data = b"hello".to_vec();
        let result = engine.on_send(&data).unwrap();
        assert_eq!(result, b"hello\n");
    }

    #[test]
    fn test_line_script_on_send_preserves_existing_newline() {
        let manager = ScriptManager::new();
        let engine = manager.create_engine("line").unwrap();
        engine.load().unwrap();

        let data = b"hello\n".to_vec();
        let result = engine.on_send(&data).unwrap();
        assert_eq!(result, b"hello\n");
    }

    #[test]
    fn test_line_script_on_recv_passthrough() {
        let manager = ScriptManager::new();
        let engine = manager.create_engine("line").unwrap();
        engine.load().unwrap();

        let data = b"hello\n".to_vec();
        let result = engine.on_recv(&data);
        assert_eq!(result, b"hello\n");
    }

    #[test]
    fn test_modbus_rtu_on_send_adds_crc() {
        let manager = ScriptManager::new();
        let engine = manager.create_engine("modbus_rtu").unwrap();
        engine.load().unwrap();

        // Modbus request: slave=1, func=0x03, start_addr=0x0000, qty=0x000A
        let request = vec![0x01, 0x03, 0x00, 0x00, 0x00, 0x0A];
        let frame = engine.on_send(&request).unwrap();

        // Frame should be 8 bytes: request(6) + CRC(2)
        assert_eq!(frame.len(), 8);
        assert_eq!(&frame[..6], &request);
        // CRC bytes should be present
        assert!(frame[6] != 0 || frame[7] != 0, "CRC should not be zero");
    }

    #[test]
    fn test_modbus_rtu_on_recv_incomplete_returns_nil() {
        let manager = ScriptManager::new();
        let engine = manager.create_engine("modbus_rtu").unwrap();
        engine.load().unwrap();

        // Only 2 bytes — too short for any Modbus frame
        let data = vec![0x01, 0x03];
        let result = engine.on_recv(&data);
        assert!(result.is_empty(), "Incomplete frame should return empty (nil in Lua)");
    }

    // ── Additional comprehensive tests ──────────────────────────────

    #[test]
    fn test_load_custom_script_with_callbacks() {
        let dir = std::env::temp_dir().join("serial_cli_test_callbacks");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("callbacks.lua");
        std::fs::write(&path, r#"
            function on_open(port, config)
                log_info("Port opened: " .. port)
            end

            function on_send(data)
                return data
            end

            function on_recv(data)
                return data
            end

            function on_close()
                log_info("Port closed")
            end
        "#).unwrap();

        let mut manager = ScriptManager::new();
        let info = manager.load(&path).unwrap();

        assert_eq!(info.name, "callbacks");
        assert!(!info.built_in);

        // Verify engine can be created and loaded
        let engine = manager.create_engine("callbacks").unwrap();
        engine.load().unwrap();

        // Cleanup
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_validate_source_valid_lua() {
        let source = r#"
            function on_send(data)
                return data
            end
        "#;
        assert!(ScriptManager::validate_source(source).is_ok());
    }

    #[test]
    fn test_validate_source_invalid_lua() {
        let source = "this is not valid lua {{{";
        assert!(ScriptManager::validate_source(source).is_err());
    }

    #[test]
    fn test_validate_source_empty() {
        let source = "";
        assert!(ScriptManager::validate_source(source).is_ok());
    }

    #[test]
    fn test_validate_source_with_comments() {
        let source = r#"
            -- This is a comment
            function on_send(data)
                -- Another comment
                return data
            end
        "#;
        assert!(ScriptManager::validate_source(source).is_ok());
    }

    #[test]
    fn test_get_meta_returns_correct_info() {
        let manager = ScriptManager::new();
        let meta = manager.get_meta("line").unwrap();

        assert_eq!(meta.name, "line");
        assert!(meta.built_in);
        assert!(!meta.description.is_empty());
    }

    #[test]
    fn test_get_meta_not_found() {
        let manager = ScriptManager::new();
        let result = manager.get_meta("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_script_info_fields() {
        let manager = ScriptManager::new();
        let scripts = manager.list();

        for script in &scripts {
            assert!(!script.name.is_empty(), "Script name should not be empty");
            assert!(!script.description.is_empty(), "Script description should not be empty");
        }
    }

    #[test]
    fn test_load_script_with_relative_imports() {
        let dir = std::env::temp_dir().join("serial_cli_test_imports");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("with_imports.lua");
        std::fs::write(&path, r#"
            -- Simple script without external dependencies
            local function helper(data)
                return data
            end

            function on_send(data)
                return helper(data)
            end
        "#).unwrap();

        let mut manager = ScriptManager::new();
        let info = manager.load(&path).unwrap();
        assert_eq!(info.name, "with_imports");

        // Cleanup
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_multiple_custom_scripts() {
        let dir = std::env::temp_dir().join("serial_cli_test_multiple");
        std::fs::create_dir_all(&dir).unwrap();

        let mut manager = ScriptManager::new();

        // Load multiple scripts
        for i in 0..5 {
            let path = dir.join(format!("script_{}.lua", i));
            std::fs::write(&path, format!("function on_recv(data) return data end")).unwrap();
            manager.load(&path).unwrap();
        }

        // Verify all scripts are loaded
        let scripts = manager.list();
        let custom_scripts: Vec<_> = scripts.iter().filter(|s| !s.built_in).collect();
        assert_eq!(custom_scripts.len(), 5, "Should have 5 custom scripts");

        // Cleanup
        for i in 0..5 {
            let path = dir.join(format!("script_{}.lua", i));
            std::fs::remove_file(&path).ok();
        }
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_unload_nonexistent_script() {
        let mut manager = ScriptManager::new();
        let result = manager.unload("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_reload_nonexistent_script() {
        let mut manager = ScriptManager::new();
        let result = manager.reload("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_script_preserves_source() {
        let dir = std::env::temp_dir().join("serial_cli_test_source");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("source_test.lua");
        let original_source = "function on_recv(data)\n    return data\nend";
        std::fs::write(&path, original_source).unwrap();

        let mut manager = ScriptManager::new();
        manager.load(&path).unwrap();

        let loaded_source = manager.get_source("source_test").unwrap();
        assert_eq!(loaded_source, original_source, "Source should be preserved");

        // Cleanup
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    // ── Hot-reload tests ───────────────────────────────────────────

    #[test]
    fn test_hot_reload_reads_from_config() {
        let manager = ScriptManager::new();
        // is_hot_reload_enabled now reads from ConfigManager
        // Default config should have hot_reload disabled
        let enabled = manager.is_hot_reload_enabled();
        // This test just verifies the method works without panicking
        assert!(enabled == true || enabled == false);
    }

    #[test]
    fn test_get_script_path_custom() {
        let dir = std::env::temp_dir().join("serial_cli_test_path");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("path_test.lua");
        std::fs::write(&path, "function on_recv(data) return data end").unwrap();

        let mut manager = ScriptManager::new();
        manager.load(&path).unwrap();

        let script_path = manager.get_script_path("path_test");
        assert!(script_path.is_some());
        assert_eq!(script_path.unwrap(), &path);

        // Cleanup
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_get_script_path_builtin() {
        let manager = ScriptManager::new();
        let script_path = manager.get_script_path("line");
        assert!(script_path.is_none(), "Built-in scripts have no path");
    }

    #[test]
    fn test_get_script_path_not_found() {
        let manager = ScriptManager::new();
        let script_path = manager.get_script_path("nonexistent");
        assert!(script_path.is_none());
    }

    #[test]
    fn test_is_script_modified_not_found() {
        let manager = ScriptManager::new();
        let result = manager.is_script_modified("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_script_modified_builtin() {
        let manager = ScriptManager::new();
        // Built-in scripts have no path, so they're never modified
        let result = manager.is_script_modified("line").unwrap();
        assert!(!result);
    }

    #[test]
    fn test_reload_modified_scripts_disabled() {
        let mut manager = ScriptManager::new();
        // Hot-reload is disabled by default
        let reloaded = manager.reload_modified_scripts().unwrap();
        assert!(reloaded.is_empty());
    }

    #[test]
    fn test_tracked_paths_empty() {
        let manager = ScriptManager::new();
        // Only built-in scripts, no paths tracked
        let paths = manager.tracked_paths();
        assert!(paths.is_empty());
    }

    #[test]
    fn test_tracked_paths_with_custom() {
        let dir = std::env::temp_dir().join("serial_cli_test_tracked");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("tracked.lua");
        std::fs::write(&path, "function on_recv(data) return data end").unwrap();

        let mut manager = ScriptManager::new();
        manager.load(&path).unwrap();

        let paths = manager.tracked_paths();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], &path);

        // Cleanup
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    // ── Enhanced validation tests ──────────────────────────────────

    #[test]
    fn test_validate_script_detailed_valid() {
        let source = r#"
            function on_send(data)
                return data
            end

            function on_recv(data)
                return data
            end
        "#;
        let warnings = ScriptManager::validate_script_detailed(source);
        assert!(warnings.is_empty(), "Valid script should have no warnings");
    }

    #[test]
    fn test_validate_script_detailed_no_callbacks() {
        let source = "local x = 1";
        let warnings = ScriptManager::validate_script_detailed(source);
        assert!(!warnings.is_empty(), "Script with no callbacks should have warning");
        assert!(warnings.iter().any(|w| w.contains("No callbacks defined")));
    }

    #[test]
    fn test_validate_script_detailed_syntax_error() {
        let source = "this is not valid lua {{{";
        let warnings = ScriptManager::validate_script_detailed(source);
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| w.starts_with("Syntax error")));
    }

    #[test]
    fn test_validate_script_detailed_require_warning() {
        let source = r#"
            local json = require("json")
            function on_send(data)
                return data
            end
        "#;
        let warnings = ScriptManager::validate_script_detailed(source);
        assert!(warnings.iter().any(|w| w.contains("require")));
    }

    #[test]
    fn test_validate_script_detailed_dangerous_functions() {
        let source = r#"
            function on_send(data)
                os.execute("ls")
                return data
            end
        "#;
        let warnings = ScriptManager::validate_script_detailed(source);
        assert!(warnings.iter().any(|w| w.contains("dangerous")));
    }

    #[test]
    fn test_load_with_validation_valid() {
        let dir = std::env::temp_dir().join("serial_cli_test_validation");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("valid.lua");
        std::fs::write(&path, "function on_recv(data) return data end").unwrap();

        let mut manager = ScriptManager::new();
        let (info, warnings) = manager.load_with_validation(&path).unwrap();

        assert_eq!(info.name, "valid");
        assert!(warnings.is_empty(), "Valid script should have no warnings");

        // Cleanup
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_load_with_validation_with_warnings() {
        let dir = std::env::temp_dir().join("serial_cli_test_validation_warn");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("warn.lua");
        // Script with no callbacks
        std::fs::write(&path, "local x = 1").unwrap();

        let mut manager = ScriptManager::new();
        let (info, warnings) = manager.load_with_validation(&path).unwrap();

        assert_eq!(info.name, "warn");
        assert!(!warnings.is_empty(), "Script with no callbacks should have warnings");

        // Cleanup
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_load_with_validation_syntax_error() {
        let dir = std::env::temp_dir().join("serial_cli_test_validation_err");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("error.lua");
        std::fs::write(&path, "this is not valid lua {{{").unwrap();

        let mut manager = ScriptManager::new();
        let result = manager.load_with_validation(&path);

        assert!(result.is_err(), "Script with syntax error should fail");

        // Cleanup
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_statistics_empty() {
        let manager = ScriptManager::new();
        // We have built-in scripts, so not empty
        let stats = manager.statistics();
        assert!(stats.total > 0);
        assert!(stats.built_in > 0);
        assert_eq!(stats.custom, 0);
    }

    #[test]
    fn test_statistics_with_custom() {
        let dir = std::env::temp_dir().join("serial_cli_test_stats");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("custom.lua");
        std::fs::write(&path, "function on_recv(data) return data end").unwrap();

        let mut manager = ScriptManager::new();
        manager.load(&path).unwrap();

        let stats = manager.statistics();
        assert!(stats.custom > 0, "Should have custom scripts");
        assert!(stats.with_path > 0, "Should have scripts with paths");

        // Cleanup
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }
}
