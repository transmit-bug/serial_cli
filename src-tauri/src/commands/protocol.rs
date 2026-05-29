// Copyright 2024 Serial CLI Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::state::app_state::AppState;
use serial_cli::protocol::ProtocolInfo;
use std::path::PathBuf;
use tauri::State;

/// Reject paths containing traversal components or absolute separators.
fn sanitize_name(name: &str) -> Result<String, String> {
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(format!("Invalid protocol name: path components not allowed"));
    }
    Ok(name.to_string())
}

/// List available protocols
#[tauri::command]
pub async fn list_protocols(state: State<'_, AppState>) -> Result<Vec<ProtocolInfo>, String> {
    let manager = state.protocol_manager.lock().await;
    Ok(manager.list_protocols().await)
}

/// Load a custom protocol
#[tauri::command]
pub async fn load_protocol(
    path: String,
    state: State<'_, AppState>,
) -> Result<ProtocolInfo, String> {
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

    let mut manager = state.protocol_manager.lock().await;
    manager
        .load_protocol(&canonical)
        .await
        .map_err(|e| format!("Failed to load protocol: {}", e))
}

/// Unload a custom protocol
#[tauri::command]
pub async fn unload_protocol(name: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut manager = state.protocol_manager.lock().await;
    manager
        .unload_protocol(&name)
        .await
        .map_err(|e| format!("Failed to unload protocol: {}", e))
}

/// Reload a custom protocol
#[tauri::command]
pub async fn reload_protocol(name: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut manager = state.protocol_manager.lock().await;
    manager
        .reload_protocol(&name)
        .await
        .map_err(|e| format!("Failed to reload protocol: {}", e))
}

/// Get protocol information
#[tauri::command]
pub async fn get_protocol_info(
    name: String,
    state: State<'_, AppState>,
) -> Result<ProtocolInfo, String> {
    let manager = state.protocol_manager.lock().await;

    // Check if it's a custom protocol
    if let Some(_custom) = manager.get_custom_protocol(&name) {
        return Ok(ProtocolInfo {
            name: name.clone(),
            description: "Custom Lua protocol".to_string(),
        });
    }

    // Check built-in protocols
    let registry = state.protocol_registry.lock().await;
    let protocols = registry.list_protocols().await;

    protocols
        .into_iter()
        .find(|p| p.name == name)
        .ok_or_else(|| format!("Protocol not found: {}", name))
}

/// Validate a protocol script without loading
#[tauri::command]
pub async fn validate_protocol(path: String) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);
    let canonical = path_buf
        .canonicalize()
        .map_err(|e| format!("Cannot resolve path: {}", e))?;

    // Reject paths outside expected directories (e.g., /etc/passwd)
    let file_name = canonical
        .file_name()
        .ok_or("Invalid protocol file path")?
        .to_str()
        .ok_or("Non-UTF8 path")?;
    if !file_name.ends_with(".lua") {
        return Err("Protocol files must have .lua extension".to_string());
    }

    serial_cli::protocol::ProtocolManager::validate_protocol(&canonical)
        .map_err(|e| format!("Protocol validation failed: {}", e))
}

/// Encode data using protocol
#[tauri::command]
pub async fn protocol_encode(
    protocol: String,
    data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<Vec<u8>, String> {
    let registry = state.protocol_registry.lock().await;

    // Get protocol instance
    let mut protocol_instance = registry
        .get_protocol(&protocol)
        .await
        .map_err(|e| format!("Failed to get protocol: {}", e))?;

    // Encode data
    protocol_instance
        .encode(&data)
        .map_err(|e| format!("Encode failed: {}", e))
}

/// Decode data using protocol
#[tauri::command]
pub async fn protocol_decode(
    protocol: String,
    data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<Vec<u8>, String> {
    let registry = state.protocol_registry.lock().await;

    // Get protocol instance
    let mut protocol_instance = registry
        .get_protocol(&protocol)
        .await
        .map_err(|e| format!("Failed to get protocol: {}", e))?;

    // Decode data
    protocol_instance
        .parse(&data)
        .map_err(|e| format!("Decode failed: {}", e))
}

/// Set the active protocol for an open port.
/// Resolves the protocol name via the registry and attaches a fresh
/// protocol instance to the port handle for encode/parse in the I/O path.
#[tauri::command]
pub async fn set_port_protocol(
    port_id: String,
    protocol_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // First verify the protocol exists in the registry
    {
        let registry = state.protocol_registry.lock().await;
        registry
            .get_protocol(&protocol_name)
            .await
            .map_err(|e| format!("Protocol not found '{}': {}", protocol_name, e))?;
    }

    // Set the protocol on the port (needs its own registry reference)
    let registry = state.protocol_registry.lock().await;
    let manager = state.port_manager.lock().await;
    manager
        .set_port_protocol_by_name(&port_id, &registry, &protocol_name)
        .await
        .map_err(|e| e.to_string())
}

/// Save a protocol file from frontend content and return its filesystem path.
///
/// The file is saved to the app data directory under `protocols/`.
/// Returns the absolute path suitable for passing to `load_protocol`/`validate_protocol`.
#[tauri::command]
pub async fn save_protocol_file(
    name: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Sanitize: reject path components
    let safe_name = sanitize_name(&name)?;

    // Ensure name ends with .lua
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
        .map_err(|e| format!("Failed to create protocols directory: {}", e))?;

    let file_path = protocol_dir.join(&file_name);
    std::fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write protocol file: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}
