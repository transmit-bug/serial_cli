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

    if !path_buf.exists() {
        return Err(format!("File not found: {}", path));
    }

    let mut manager = state.protocol_manager.lock().await;
    manager
        .load_protocol(&path_buf)
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

    if !path_buf.exists() {
        return Err(format!("File not found: {}", path));
    }

    serial_cli::protocol::ProtocolManager::validate_protocol(&path_buf)
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
