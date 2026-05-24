// Copyright 2024 Serial CLI Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::state::app_state::AppState;
use serial_cli::lua::ui_actions::UiAction;
use serial_cli::lua::LuaBindings;
use tauri::State;

/// List UI actions from a standalone script (not attached to a port)
#[tauri::command]
pub async fn list_standalone_script_actions(
    script_source: String,
    _state: State<'_, AppState>,
) -> Result<Vec<UiAction>, String> {
    let bindings = LuaBindings::new().map_err(|e| format!("Failed to create Lua engine: {}", e))?;

    bindings
        .register_all_apis()
        .map_err(|e| format!("Failed to register APIs: {}", e))?;

    bindings
        .execute_script(&script_source)
        .map_err(|e| format!("Failed to load script: {}", e))?;

    let actions = bindings.discover_actions().map_err(|e| e.to_string())?;

    Ok(actions)
}

/// Execute a function in a standalone script
#[tauri::command]
pub async fn call_standalone_script_function(
    script_source: String,
    function_name: String,
    _state: State<'_, AppState>,
) -> Result<String, String> {
    let bindings = LuaBindings::new().map_err(|e| format!("Failed to create Lua engine: {}", e))?;

    bindings
        .register_all_apis()
        .map_err(|e| format!("Failed to register APIs: {}", e))?;

    bindings
        .execute_script(&script_source)
        .map_err(|e| format!("Failed to load script: {}", e))?;

    let result = bindings
        .execute_action(&function_name)
        .map_err(|e| e.to_string())?;

    Ok(result)
}
