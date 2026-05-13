// Copyright 2024 Serial CLI Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::state::app_state::AppState;
use serial_cli::lua::LuaBindings;
use std::fs;
use std::path::PathBuf;
use tauri::State;

/// Execute a Lua script
#[tauri::command]
pub async fn execute_script(script: String, _state: State<'_, AppState>) -> Result<String, String> {
    // Create Lua bindings
    let bindings = LuaBindings::new().map_err(|e| format!("Failed to create Lua engine: {}", e))?;

    // Register all APIs at once
    bindings
        .register_all_apis()
        .map_err(|e| format!("Failed to register APIs: {}", e))?;

    // Execute the script
    bindings
        .execute_script(&script)
        .map(|_| "Script executed successfully".to_string())
        .map_err(|e| format!("Script execution error: {}", e))
}

/// Validate a Lua script
#[tauri::command]
pub async fn validate_script(
    script: String,
    _state: State<'_, AppState>,
) -> Result<Vec<ValidationError>, String> {
    // Create Lua bindings
    let bindings = LuaBindings::new().map_err(|e| format!("Failed to create Lua engine: {}", e))?;

    // Try to load the script (without executing)
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

/// List available scripts
#[tauri::command]
pub async fn list_scripts(_state: State<'_, AppState>) -> Result<Vec<ScriptInfo>, String> {
    let scripts_dir = get_scripts_dir()?;
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

                scripts.push(ScriptInfo {
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

/// Load a script
#[tauri::command]
pub async fn load_script(name: String, _state: State<'_, AppState>) -> Result<String, String> {
    let scripts_dir = get_scripts_dir()?;
    let script_path = scripts_dir.join(format!("{}.lua", name));

    if !script_path.exists() {
        return Err(format!("Script not found: {}", name));
    }

    fs::read_to_string(&script_path).map_err(|e| format!("Failed to read script: {}", e))
}

/// Save a script
#[tauri::command]
pub async fn save_script(
    name: String,
    content: String,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    let scripts_dir = get_scripts_dir()?;

    // Create scripts directory if it doesn't exist
    if !scripts_dir.exists() {
        fs::create_dir_all(&scripts_dir)
            .map_err(|e| format!("Failed to create scripts directory: {}", e))?;
    }

    let script_path = scripts_dir.join(format!("{}.lua", name));

    fs::write(&script_path, content).map_err(|e| format!("Failed to write script: {}", e))
}

/// Delete a script
#[tauri::command]
pub async fn delete_script(name: String, _state: State<'_, AppState>) -> Result<(), String> {
    let scripts_dir = get_scripts_dir()?;
    let script_path = scripts_dir.join(format!("{}.lua", name));

    if !script_path.exists() {
        return Err(format!("Script not found: {}", name));
    }

    fs::remove_file(&script_path).map_err(|e| format!("Failed to delete script: {}", e))
}

/// Get scripts directory
fn get_scripts_dir() -> Result<PathBuf, String> {
    let mut base_dir = dirs::home_dir().ok_or("Failed to get home directory")?;

    base_dir.push(".serial-cli");
    base_dir.push("scripts");

    Ok(base_dir)
}

/// Parse Lua error to extract line and column information
fn parse_lua_error(error_msg: &str) -> Vec<ValidationError> {
    // Simple parsing - in production, you'd want more robust error parsing
    let mut errors = Vec::new();

    // Try to extract line number from error message
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
pub struct ScriptInfo {
    pub name: String,
    pub path: String,
    pub size: usize,
    pub modified: i64,
}
