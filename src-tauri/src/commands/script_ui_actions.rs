// Copyright 2024 Serial CLI Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::state::app_state::AppState;
use serial_cli::lua::ui_actions;
use serial_cli::lua::LuaBindings;
use tauri::State;

/// List all available UI actions from a script
///
/// Returns a list of actions that can be exposed as UI buttons.
/// Actions are discovered by scanning for functions with the `action_` prefix.
#[tauri::command]
pub async fn list_script_actions(
    port_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ui_actions::UiAction>, String> {
    // Get the port manager
    let port_manager = state.port_manager.lock().await;

    // Check if the port has a script attached
    let has_script = port_manager
        .has_script(&port_id)
        .await
        .map_err(|e| e.to_string())?;

    if !has_script {
        return Err(format!("No script attached to port: {}", port_id));
    }

    // Get the script engine from the port
    let script_engine = port_manager
        .get_script_engine(&port_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Script engine not found for port: {}", port_id))?;

    // Discover actions from the script's Lua state
    let actions = script_engine
        .discover_actions()
        .await
        .map_err(|e| e.to_string())?;

    Ok(actions)
}

/// Call a script function by name
///
/// Executes a Lua function (typically an `action_*` function) in the context
/// of an attached script. Returns the result as a string.
#[tauri::command]
pub async fn call_script_function(
    port_id: String,
    function_name: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Get the port manager
    let port_manager = state.port_manager.lock().await;

    // Check if the port has a script attached
    let has_script = port_manager
        .has_script(&port_id)
        .await
        .map_err(|e| e.to_string())?;

    if !has_script {
        return Err(format!("No script attached to port: {}", port_id));
    }

    // Get the script engine from the port
    let script_engine = port_manager
        .get_script_engine(&port_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Script engine not found for port: {}", port_id))?;

    // Execute the action function
    let result = script_engine
        .execute_action(&function_name)
        .await
        .map_err(|e| e.to_string())?;

    // Return the result (already converted to string)
    Ok(result.unwrap_or_else(|| "OK".to_string()))
}

/// List UI actions from a standalone script (not attached to a port)
///
/// This is used for scripts that are executed independently (not in hook mode).
#[tauri::command]
pub async fn list_standalone_script_actions(
    script_source: String,
    _state: State<'_, AppState>,
) -> Result<Vec<ui_actions::UiAction>, String> {
    // Create a new Lua bindings instance
    let bindings = LuaBindings::new().map_err(|e| format!("Failed to create Lua engine: {}", e))?;

    // Register all APIs
    bindings
        .register_all_apis()
        .map_err(|e| format!("Failed to register APIs: {}", e))?;

    // Load the script to discover actions
    bindings
        .execute_script(&script_source)
        .map_err(|e| format!("Failed to load script: {}", e))?;

    // Discover actions from the Lua state
    let actions = bindings.discover_actions().map_err(|e| e.to_string())?;

    Ok(actions)
}

/// Execute a function in a standalone script
///
/// This is used for scripts that are executed independently (not in hook mode).
#[tauri::command]
pub async fn call_standalone_script_function(
    script_source: String,
    function_name: String,
    _state: State<'_, AppState>,
) -> Result<String, String> {
    // Create a new Lua bindings instance
    let bindings = LuaBindings::new().map_err(|e| format!("Failed to create Lua engine: {}", e))?;

    // Register all APIs
    bindings
        .register_all_apis()
        .map_err(|e| format!("Failed to register APIs: {}", e))?;

    // Load the script
    bindings
        .execute_script(&script_source)
        .map_err(|e| format!("Failed to load script: {}", e))?;

    // Execute the function
    let result = bindings
        .execute_action(&function_name)
        .map_err(|e| e.to_string())?;

    // Return the result (already converted to string)
    Ok(result.unwrap_or_else(|| "OK".to_string()))
}
