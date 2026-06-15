// Copyright 2024 Serial CLI Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::state::app_state::AppState;
use serial_cli::script::ScriptInfo;
use std::path::PathBuf;
use tauri::State;

/// Reject paths containing traversal components or absolute separators.
fn sanitize_name(name: &str) -> Result<String, String> {
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(format!(
            "Invalid protocol name: path components not allowed"
        ));
    }
    Ok(name.to_string())
}

/// List available scripts (replaces list_protocols)
#[tauri::command]
pub async fn list_protocols(state: State<'_, AppState>) -> Result<Vec<ScriptInfo>, String> {
    let manager = state.script_manager.lock().await;
    Ok(manager.list())
}

/// Load a custom script (replaces load_protocol)
#[tauri::command]
pub async fn load_protocol(
    path: String,
    state: State<'_, AppState>,
) -> Result<ScriptInfo, String> {
    let path_buf = PathBuf::from(&path);
    let canonical = path_buf
        .canonicalize()
        .map_err(|e| format!("Cannot resolve path: {}", e))?;

    let protocols_dir = state
        .protocols_dir
        .as_ref()
        .ok_or("Protocols directory not configured")?;
    let canonical_dir = protocols_dir
        .canonicalize()
        .map_err(|e| format!("Cannot resolve protocols directory: {}", e))?;

    if !canonical.starts_with(&canonical_dir) {
        return Err("Path must be within the protocols directory".to_string());
    }

    let mut manager = state.script_manager.lock().await;
    manager
        .load(&canonical)
        .map_err(|e| format!("Failed to load script: {}", e))
}

/// Unload a custom script
#[tauri::command]
pub async fn unload_protocol(name: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut manager = state.script_manager.lock().await;
    manager
        .unload(&name)
        .map_err(|e| format!("Failed to unload script: {}", e))
}

/// Reload a custom script
#[tauri::command]
pub async fn reload_protocol(name: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut manager = state.script_manager.lock().await;
    manager
        .reload(&name)
        .map_err(|e| format!("Failed to reload script: {}", e))
}

/// Get script information
#[tauri::command]
pub async fn get_protocol_info(
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

/// Validate a script without loading
#[tauri::command]
pub async fn validate_protocol(path: String) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);
    let canonical = path_buf
        .canonicalize()
        .map_err(|e| format!("Cannot resolve path: {}", e))?;

    let file_name = canonical
        .file_name()
        .ok_or("Invalid protocol file path")?
        .to_str()
        .ok_or("Non-UTF8 path")?;
    if !file_name.ends_with(".lua") {
        return Err("Script files must have .lua extension".to_string());
    }

    let source = std::fs::read_to_string(&canonical)
        .map_err(|e| format!("Failed to read script: {}", e))?;

    serial_cli::script::ScriptManager::validate_source(&source)
        .map_err(|e| format!("Script validation failed: {}", e))
}

/// Attach a script to an open port (replaces set_port_protocol)
#[tauri::command]
pub async fn set_port_protocol(
    port_id: String,
    protocol_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let script_mgr = state.script_manager.lock().await;
    let port_mgr = state.port_manager.lock().await;

    port_mgr
        .attach_script_by_name(&port_id, &script_mgr, &protocol_name)
        .await
        .map_err(|e| e.to_string())
}

/// Encode data using a script (by name)
#[tauri::command]
pub async fn protocol_encode(
    protocol: String,
    data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<Vec<u8>, String> {
    let manager = state.script_manager.lock().await;
    let engine = manager
        .create_engine(&protocol)
        .map_err(|e| format!("Failed to create engine: {}", e))?;
    engine.load().map_err(|e| format!("Failed to load script: {}", e))?;
    engine
        .on_send(&data)
        .map_err(|e| format!("Encode failed: {}", e))
}

/// Decode data using a script (by name)
#[tauri::command]
pub async fn protocol_decode(
    protocol: String,
    data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<Vec<u8>, String> {
    let manager = state.script_manager.lock().await;
    let engine = manager
        .create_engine(&protocol)
        .map_err(|e| format!("Failed to create engine: {}", e))?;
    engine.load().map_err(|e| format!("Failed to load script: {}", e))?;
    Ok(engine.on_recv(&data))
}

/// Save a script file from frontend content and return its filesystem path.
#[tauri::command]
pub async fn save_protocol_file(
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

    let protocol_dir = state
        .protocols_dir
        .clone()
        .ok_or("Protocols directory not configured")?;

    std::fs::create_dir_all(&protocol_dir)
        .map_err(|e| format!("Failed to create scripts directory: {}", e))?;

    let file_path = protocol_dir.join(&file_name);
    std::fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write script file: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}
