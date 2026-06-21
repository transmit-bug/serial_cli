// Copyright 2024 Serial CLI Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Unified script management commands
//!
//! This module handles all script-related operations:
//! - Script lifecycle (load/unload/reload/list)
//! - Script execution and validation
//! - Port-script binding
//! - Data encoding/decoding

use crate::state::app_state::AppState;
use serial_cli::lua::LuaBindings;
use serial_cli::script::ScriptInfo;
use std::fs;
use std::path::PathBuf;
use tauri::State;

// ── Script Lifecycle (ScriptManager-backed) ─────────────────────────────────

/// List all registered scripts (built-in + custom)
#[tauri::command]
pub async fn list_scripts(state: State<'_, AppState>) -> Result<Vec<ScriptInfo>, String> {
    let manager = state.script_manager.lock().await;
    Ok(manager.list())
}

/// Load a custom script from a file path
#[tauri::command]
pub async fn load_script(path: String, state: State<'_, AppState>) -> Result<ScriptInfo, String> {
    let path_buf = PathBuf::from(&path);
    let canonical = path_buf
        .canonicalize()
        .map_err(|e| format!("Cannot resolve path: {}", e))?;

    let scripts_dir = state
        .scripts_dir
        .as_ref()
        .ok_or("Scripts directory not configured")?;
    let canonical_dir = scripts_dir
        .canonicalize()
        .map_err(|e| format!("Cannot resolve scripts directory: {}", e))?;

    if !canonical.starts_with(&canonical_dir) {
        return Err("Path must be within the scripts directory".to_string());
    }

    let mut manager = state.script_manager.lock().await;
    manager
        .load(&canonical)
        .map_err(|e| format!("Failed to load script: {}", e))
}

/// Unload a custom script by name
#[tauri::command]
pub async fn unload_script(name: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut manager = state.script_manager.lock().await;
    manager
        .unload(&name)
        .map_err(|e| format!("Failed to unload script: {}", e))
}

/// Reload a custom script from its original file path
#[tauri::command]
pub async fn reload_script(name: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut manager = state.script_manager.lock().await;
    manager
        .reload(&name)
        .map_err(|e| format!("Failed to reload script: {}", e))
}

/// Get script metadata
#[tauri::command]
pub async fn get_script_info(
    name: String,
    state: State<'_, AppState>,
) -> Result<ScriptInfo, String> {
    let manager = state.script_manager.lock().await;
    let meta = manager
        .get_meta(&name)
        .map_err(|e| format!("Script not found: {}", e))?;

    Ok(ScriptInfo {
        name: meta.name.clone(),
        description: meta.description.clone(),
        built_in: meta.built_in,
    })
}

// ── Script Execution & Validation ────────────────────────────────────────────

/// Execute a Lua script (runs in spawn_blocking to avoid blocking async runtime)
#[tauri::command]
pub async fn execute_script(script: String, _state: State<'_, AppState>) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let bindings =
            LuaBindings::new().map_err(|e| format!("Failed to create Lua engine: {}", e))?;

        bindings
            .register_all_apis()
            .map_err(|e| format!("Failed to register APIs: {}", e))?;

        bindings
            .execute_script(&script)
            .map(|_| "Script executed successfully".to_string())
            .map_err(|e| format!("Script execution error: {}", e))
    })
    .await
    .map_err(|e| format!("Script task failed: {}", e))?
}

/// Validate Lua source code syntax
#[tauri::command]
pub async fn validate_script(
    script: String,
    _state: State<'_, AppState>,
) -> Result<Vec<ValidationError>, String> {
    let bindings = LuaBindings::new().map_err(|e| format!("Failed to create Lua engine: {}", e))?;

    match bindings.lua().load(script).exec() {
        Ok(_) => Ok(vec![]),
        Err(e) => {
            let mut errors = parse_lua_error(&e.to_string());
            if errors.is_empty() {
                errors.push(ValidationError {
                    line: 0,
                    column: 0,
                    message: e.to_string(),
                });
            }
            Ok(errors)
        }
    }
}

/// Validate a script file at the given path
#[tauri::command]
pub async fn validate_script_file(path: String) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);
    let canonical = path_buf
        .canonicalize()
        .map_err(|e| format!("Cannot resolve path: {}", e))?;

    let file_name = canonical
        .file_name()
        .ok_or("Invalid script file path")?
        .to_str()
        .ok_or("Non-UTF8 path")?;
    if !file_name.ends_with(".lua") {
        return Err("Script files must have .lua extension".to_string());
    }

    let source =
        std::fs::read_to_string(&canonical).map_err(|e| format!("Failed to read script: {}", e))?;

    serial_cli::script::ScriptManager::validate_source(&source)
        .map_err(|e| format!("Script validation failed: {}", e))
}

// ── User Script File Management ──────────────────────────────────────────────

/// List user scripts saved in ~/.serial-cli/scripts/
#[tauri::command]
pub async fn list_user_scripts(_state: State<'_, AppState>) -> Result<Vec<UserScriptInfo>, String> {
    let scripts_dir = get_user_scripts_dir()?;
    let mut scripts = Vec::new();

    if scripts_dir.exists() {
        for entry in fs::read_dir(scripts_dir)
            .map_err(|e| format!("Failed to read scripts directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("lua") {
                let metadata = fs::metadata(&path)
                    .map_err(|e| format!("Failed to read file metadata: {}", e))?;

                scripts.push(UserScriptInfo {
                    name: path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    path: path.to_string_lossy().to_string(),
                    size: metadata.len() as usize,
                    modified: metadata
                        .modified()
                        .map_err(|e| format!("Failed to get modification time: {}", e))?
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_err(|e| format!("Failed to convert time: {}", e))?
                        .as_secs() as i64,
                });
            }
        }
    }

    scripts.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(scripts)
}

/// Save a user script to ~/.serial-cli/scripts/
#[tauri::command]
pub async fn save_user_script(
    name: String,
    content: String,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    let scripts_dir = get_user_scripts_dir()?;

    if !scripts_dir.exists() {
        fs::create_dir_all(&scripts_dir)
            .map_err(|e| format!("Failed to create scripts directory: {}", e))?;
    }

    let script_path = scripts_dir.join(format!("{}.lua", name));

    fs::write(&script_path, content).map_err(|e| format!("Failed to write script: {}", e))
}

/// Delete a user script from ~/.serial-cli/scripts/
#[tauri::command]
pub async fn delete_user_script(name: String, _state: State<'_, AppState>) -> Result<(), String> {
    let scripts_dir = get_user_scripts_dir()?;
    let script_path = scripts_dir.join(format!("{}.lua", name));

    if !script_path.exists() {
        return Err(format!("Script not found: {}", name));
    }

    fs::remove_file(&script_path).map_err(|e| format!("Failed to delete script: {}", e))
}

// ── Port-Script Binding ──────────────────────────────────────────────────────

/// Attach a script to an open port (by name, via ScriptManager)
#[tauri::command]
pub async fn bind_script(
    port_id: String,
    script_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let script_mgr = state.script_manager.lock().await;
    let port_mgr = state.port_manager.lock().await;

    port_mgr
        .attach_script_by_name(&port_id, &script_mgr, &script_name)
        .await
        .map_err(|e| e.to_string())
}

// ── Data Encoding/Decoding ───────────────────────────────────────────────────

/// Encode data using a script (by name)
#[tauri::command]
pub async fn script_encode(
    script: String,
    data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<Vec<u8>, String> {
    let manager = state.script_manager.lock().await;
    let engine = manager
        .create_engine(&script)
        .map_err(|e| format!("Failed to create engine: {}", e))?;
    engine
        .load()
        .map_err(|e| format!("Failed to load script: {}", e))?;
    engine
        .on_send(&data)
        .map_err(|e| format!("Encode failed: {}", e))
}

/// Decode data using a script (by name)
#[tauri::command]
pub async fn script_decode(
    script: String,
    data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<Vec<u8>, String> {
    let manager = state.script_manager.lock().await;
    let engine = manager
        .create_engine(&script)
        .map_err(|e| format!("Failed to create engine: {}", e))?;
    engine
        .load()
        .map_err(|e| format!("Failed to load script: {}", e))?;
    Ok(engine.on_recv(&data))
}

// ── File Operations ──────────────────────────────────────────────────────────

/// Save a script file from frontend content and return its filesystem path
#[tauri::command]
pub async fn save_script_file(
    name: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let safe_name = sanitize_name(&name)?;

    let file_name = if safe_name.ends_with(".lua") {
        safe_name
    } else {
        format!("{safe_name}.lua")
    };

    let scripts_dir = state
        .scripts_dir
        .clone()
        .ok_or("Scripts directory not configured")?;

    std::fs::create_dir_all(&scripts_dir)
        .map_err(|e| format!("Failed to create scripts directory: {}", e))?;

    let file_path = scripts_dir.join(&file_name);
    std::fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write script file: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

// ── Hot Reload & Validation ──────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct ScriptValidationResult {
    pub warnings: Vec<String>,
}

/// Get hot-reload status
#[tauri::command]
pub async fn get_hot_reload_status(_state: State<'_, AppState>) -> Result<bool, String> {
    let config_manager = serial_cli::config::ConfigManager::load_with_fallback();
    Ok(config_manager.is_hot_reload_enabled())
}

/// Enable or disable hot-reload
#[tauri::command]
pub async fn set_hot_reload_enabled(
    enabled: bool,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    let config_manager = serial_cli::config::ConfigManager::load_with_fallback();
    config_manager
        .set("protocols.hot_reload", &enabled.to_string())
        .map_err(|e| e.to_string())?;
    config_manager.save(None).map_err(|e| e.to_string())?;
    Ok(())
}

/// Validate script with detailed warnings (callbacks, dangerous functions, etc.)
#[tauri::command]
pub async fn validate_script_detailed(
    script: String,
    _state: State<'_, AppState>,
) -> Result<ScriptValidationResult, String> {
    let warnings = serial_cli::script::ScriptManager::validate_script_detailed(&script);

    Ok(ScriptValidationResult { warnings })
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Reject paths containing traversal components or absolute separators
fn sanitize_name(name: &str) -> Result<String, String> {
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err("Invalid script name: path components not allowed".to_string());
    }
    Ok(name.to_string())
}

/// Get user scripts directory (~/.serial-cli/scripts/)
fn get_user_scripts_dir() -> Result<PathBuf, String> {
    let mut base_dir = dirs::home_dir().ok_or("Failed to get home directory")?;
    base_dir.push(".serial-cli");
    base_dir.push("scripts");
    Ok(base_dir)
}

/// Parse Lua error to extract line and column information
fn parse_lua_error(error_msg: &str) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    if let Some(line_start) = error_msg.find("line ") {
        let line_part = &error_msg[line_start + 5..];
        if let Some(line_end) = line_part.find(',') {
            if let Ok(line_num) = line_part[..line_end].parse::<usize>() {
                errors.push(ValidationError {
                    line: line_num,
                    column: 0,
                    message: error_msg.to_string(),
                });
            }
        }
    }

    if errors.is_empty() {
        errors.push(ValidationError {
            line: 0,
            column: 0,
            message: error_msg.to_string(),
        });
    }

    errors
}

#[derive(serde::Serialize)]
pub struct ValidationError {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

#[derive(serde::Serialize)]
pub struct UserScriptInfo {
    pub name: String,
    pub path: String,
    pub size: usize,
    pub modified: i64,
}
