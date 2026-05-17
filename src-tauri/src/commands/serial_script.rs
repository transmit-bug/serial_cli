// Copyright 2024 Serial CLI Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::state::app_state::AppState;
use serial_cli::lua::ui_actions::UiAction;
use serial_cli::serial_core::serial_script::SerialScriptEngine;
use tauri::State;

/// Attach a Lua script engine to an open serial port.
/// The script's lifecycle hooks (on_open, on_send, on_recv, on_timer, on_close)
/// will be called automatically during port operations.
#[tauri::command]
pub async fn attach_script(
    port_id: String,
    script_source: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let engine =
        SerialScriptEngine::new(&script_source).map_err(|e| format!("Invalid script: {}", e))?;

    let manager = state.port_manager.lock().await;
    manager
        .attach_script(&port_id, engine)
        .await
        .map_err(|e| e.to_string())
}

/// Detach the Lua script engine from an open serial port.
/// Calls `on_close` and stops the timer before detaching.
#[tauri::command]
pub async fn detach_script(port_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let manager = state.port_manager.lock().await;
    manager
        .detach_script(&port_id)
        .await
        .map_err(|e| e.to_string())
}

/// Check whether a serial port has a script engine attached.
#[tauri::command]
pub async fn has_script(port_id: String, state: State<'_, AppState>) -> Result<bool, String> {
    let manager = state.port_manager.lock().await;
    manager
        .has_script(&port_id)
        .await
        .map_err(|e| e.to_string())
}

/// Get the script status for a serial port.
/// Returns `{ has_script, timer_interval_ms }`.
#[tauri::command]
pub async fn get_script_status(
    port_id: String,
    state: State<'_, AppState>,
) -> Result<ScriptStatus, String> {
    let manager = state.port_manager.lock().await;
    let (has, timer_ms) = manager
        .get_script_status(&port_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ScriptStatus {
        has_script: has,
        timer_interval_ms: timer_ms,
    })
}

/// List all UI actions (`action_*` functions) exposed by the script attached to a port.
#[tauri::command]
pub async fn list_script_actions(
    port_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<UiAction>, String> {
    let manager = state.port_manager.lock().await;
    manager
        .list_script_actions(&port_id)
        .await
        .map_err(|e| e.to_string())
}

/// Execute a UI action function on the port's attached script engine.
#[tauri::command]
pub async fn call_script_function(
    port_id: String,
    function_name: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let manager = state.port_manager.lock().await;
    manager
        .call_script_action(&port_id, &function_name)
        .await
        .map_err(|e| e.to_string())
}

/// Script status response
#[derive(serde::Serialize)]
pub struct ScriptStatus {
    pub has_script: bool,
    pub timer_interval_ms: u64,
}
