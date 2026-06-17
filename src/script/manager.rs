//! Script manager
//!
//! Manages loading, unloading, reloading, and listing of Lua scripts.

use crate::error::{Result, ScriptError, SerialError};
use crate::script::built_in;
use crate::script::ScriptInfo;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

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
    #[allow(dead_code)]
    hot_reload_enabled: Arc<Mutex<bool>>,
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

        Self {
            scripts,
            hot_reload_enabled: Arc::new(Mutex::new(false)),
        }
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

        // Validate syntax
        let lua = mlua::Lua::new();
        lua.load(&source).exec().map_err(|e| {
            SerialError::Script(ScriptError::Syntax {
                script: path.to_path_buf(),
                line: 0,
                message: e.to_string(),
            })
        })?;

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
        let lua = mlua::Lua::new();
        lua.load(source)
            .exec()
            .map_err(|e| e.to_string())
    }

    /// Create a SerialScriptEngine from a named script.
    ///
    /// This is the bridge between ScriptManager and the port-level
    /// script attachment system.
    pub fn create_engine(&self, name: &str) -> Result<crate::serial_core::serial_script::SerialScriptEngine> {
        let source = self.get_source(name)?;
        crate::serial_core::serial_script::SerialScriptEngine::new(&source)
    }
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
}
