//! UI Actions - Lua function discovery and execution
//!
//! This module provides functionality to discover and execute Lua functions
//! that are exposed as UI actions through the `action_*` naming convention.
//!
//! # Naming Convention
//!
//! Functions prefixed with `action_` are automatically discovered and exposed:
//!
//! ```lua
//! function action_send_at()
//!     serial_send(port_id, "AT\r\n")
//! end
//!
//! function action_reset_device()
//!     serial_send(port_id, "ATZ\r\n")
//! end
//! ```
//!
//! # Metadata
//!
//! Optional metadata can be provided via the `_actions` global table:
//!
//! ```lua
//! _actions = {
//!     send_at = { label = "📡 Send AT", group = "Basic Commands" },
//!     reset_device = { label = "🔄 Reset", group = "Device Control", confirm = true },
//! }
//! ```

use crate::error::{Result, SerialError};
use mlua::{Lua, Table, Value};
use serde::Serialize;

/// Action parameter descriptor
#[derive(Debug, Clone, Serialize)]
pub struct ActionParam {
    /// Parameter name (matches Lua function argument name)
    pub name: String,
    /// Parameter type hint: "number", "string", "hex"
    #[serde(rename = "type")]
    pub param_type: String,
    /// Display label for the UI
    pub label: Option<String>,
    /// Default value (if present, the param is optional)
    pub default: Option<String>,
}

/// UI Action metadata
#[derive(Debug, Clone, Serialize)]
pub struct UiAction {
    /// Function name (e.g., "action_send_at")
    pub function_name: String,

    /// Display label (fallback to humanized function name)
    pub label: String,

    /// Icon name (lucide-react)
    pub icon: Option<String>,

    /// Group name for organization
    pub group: Option<String>,

    /// Whether to show confirmation dialog
    pub confirm: bool,

    /// Parameter descriptors (empty = no params needed)
    pub params: Vec<ActionParam>,
}

impl UiAction {
    /// Create a new UiAction with minimal metadata
    fn new(function_name: String) -> Self {
        let label = humanize_function_name(&function_name);
        Self {
            function_name,
            label,
            icon: None,
            group: None,
            confirm: false,
            params: Vec::new(),
        }
    }
}

/// Humanize a function name for display
///
/// Converts "action_send_at_command" to "Send At Command"
fn humanize_function_name(name: &str) -> String {
    name.strip_prefix("action_")
        .unwrap_or(name)
        .replace('_', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Discover all `action_*` functions in the Lua globals
///
/// Scans the Lua global namespace for functions with the `action_` prefix
/// and enriches them with metadata from the optional `_actions` table.
pub fn discover_actions(lua: &Lua) -> Result<Vec<UiAction>> {
    let globals = lua.globals();

    // Try to get the optional _actions metadata table
    let _actions: Option<Table> = globals.get("_actions").ok();

    let mut actions = Vec::new();

    // Iterate over all globals to find action_* functions
    let pairs: std::result::Result<Vec<(String, Value)>, mlua::Error> = globals.pairs().collect();

    let pairs = pairs.map_err(SerialError::Lua)?;

    for (name, value) in pairs {
        // Only consider functions with the action_ prefix
        if name.starts_with("action_") {
            // Check if it's a function
            if matches!(value, Value::Function(_)) {
                let key = name.strip_prefix("action_").unwrap_or(&name);

                // Try to get metadata from _actions table
                let meta: Option<Table> = _actions.as_ref().and_then(|t| t.get(key).ok());

                let mut action = UiAction::new(name.clone());

                // Apply metadata if available
                if let Some(meta_table) = meta {
                    // Label (override fallback)
                    if let Ok(label) = meta_table.get::<_, String>("label") {
                        action.label = label;
                    }

                    // Icon
                    if let Ok(icon) = meta_table.get::<_, String>("icon") {
                        action.icon = Some(icon);
                    }

                    // Group
                    if let Ok(group) = meta_table.get::<_, String>("group") {
                        action.group = Some(group);
                    }

                    // Confirm flag
                    if let Ok(confirm) = meta_table.get::<_, bool>("confirm") {
                        action.confirm = confirm;
                    }

                    // Params table
                    if let Ok(params_table) = meta_table.get::<_, Table>("params") {
                        for i in 1..=params_table.len().unwrap_or(0) {
                            if let Ok(p) = params_table.get::<_, Table>(i) {
                                let name: String =
                                    p.get("name").unwrap_or_else(|_| format!("arg{}", i));
                                let param_type: String =
                                    p.get("type").unwrap_or_else(|_| "string".to_string());
                                let label: Option<String> = p.get("label").ok();
                                let default: Option<String> = p.get("default").ok();
                                action.params.push(ActionParam {
                                    name,
                                    param_type,
                                    label,
                                    default,
                                });
                            }
                        }
                    }
                }

                actions.push(action);
            }
        }
    }

    actions.sort_by(|a, b| {
        // Sort by group first, then by label
        match (&a.group, &b.group) {
            (Some(g1), Some(g2)) if g1 != g2 => g1.cmp(g2),
            _ => a.label.cmp(&b.label),
        }
    });

    Ok(actions)
}

/// Execute an action function by name
///
/// Calls the specified Lua function with optional arguments.
pub fn execute_action(lua: &Lua, function_name: &str) -> Result<()> {
    let globals = lua.globals();
    let func = globals
        .get::<_, mlua::Function>(function_name)
        .map_err(SerialError::Lua)?;

    func.call::<_, ()>(()).map_err(SerialError::Lua)?;
    Ok(())
}

/// Execute an action function and return its result as a string.
///
/// This is the Tauri-compatible entry point — returns a plain `String`
/// that can be sent across the FFI boundary.
pub fn execute_action_string(lua: &Lua, function_name: &str) -> Result<String> {
    let globals = lua.globals();
    let func = globals
        .get::<_, mlua::Function>(function_name)
        .map_err(SerialError::Lua)?;

    let result = func.call::<_, Value>(()).map_err(SerialError::Lua)?;

    Ok(lua_value_to_string(&result))
}

/// Execute an action function with JSON-encoded arguments.
///
/// Parses the `args_json` string as a JSON array and converts each element
/// to the corresponding Lua value before calling the function. Supports
/// numbers, strings, booleans, and hex-encoded byte arrays.
pub fn execute_action_with_args(lua: &Lua, function_name: &str, args_json: &str) -> Result<String> {
    let globals = lua.globals();
    let func = globals
        .get::<_, mlua::Function>(function_name)
        .map_err(SerialError::Lua)?;

    let args: serde_json::Value = serde_json::from_str(args_json)
        .map_err(|e| SerialError::Config(format!("Invalid args JSON: {}", e)))?;

    let args_array = match args {
        serde_json::Value::Array(arr) => arr,
        _ => return Err(SerialError::Config("args must be a JSON array".to_string())),
    };

    // Build a Lua table to hold the args, then unpack in a wrapper call
    // This avoids lifetime issues with MultiValue and borrowed strings.
    let args_table = lua.create_table().map_err(SerialError::Lua)?;
    for (i, arg) in args_array.iter().enumerate() {
        match arg {
            serde_json::Value::Null => {
                args_table
                    .set(i + 1, mlua::Value::Nil)
                    .map_err(SerialError::Lua)?;
            }
            serde_json::Value::Bool(b) => {
                args_table.set(i + 1, *b).map_err(SerialError::Lua)?;
            }
            serde_json::Value::Number(n) => {
                if let Some(v) = n.as_i64() {
                    args_table.set(i + 1, v).map_err(SerialError::Lua)?;
                } else if let Some(v) = n.as_f64() {
                    args_table.set(i + 1, v).map_err(SerialError::Lua)?;
                }
            }
            serde_json::Value::String(s) => {
                args_table
                    .set(i + 1, s.as_str())
                    .map_err(SerialError::Lua)?;
            }
            _ => {
                // Objects/arrays: serialize as JSON string
                let json_str =
                    serde_json::to_string(arg).map_err(|e| SerialError::Config(e.to_string()))?;
                args_table.set(i + 1, json_str).map_err(SerialError::Lua)?;
            }
        }
    }

    // Use a Lua wrapper to spread table values as function arguments
    let wrapper = lua
        .create_function(
            move |lua, (func, args_table): (mlua::Function, mlua::Table)| {
                // Build a call string: return function(unpack(args))
                let globals = lua.globals();
                let unpack_fn: mlua::Function = globals.get("unpack").or_else(|_| {
                    globals
                        .get("table")
                        .and_then(|t: mlua::Table| t.get("unpack"))
                })?;
                let unpacked = unpack_fn.call::<_, mlua::MultiValue>(args_table)?;
                func.call::<_, Value>(unpacked)
            },
        )
        .map_err(SerialError::Lua)?;

    let result = wrapper
        .call::<_, Value>((func, args_table))
        .map_err(SerialError::Lua)?;

    Ok(lua_value_to_string(&result))
}

/// Convert a Lua return value to a display string.
fn lua_value_to_string(value: &Value) -> String {
    match value {
        Value::Nil => "ok".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.to_str().map(|v| v.to_string()).unwrap_or_default(),
        _ => "ok".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_humanize_function_name() {
        assert_eq!(humanize_function_name("action_send_at"), "Send At");
        assert_eq!(
            humanize_function_name("action_reset_device"),
            "Reset Device"
        );
        assert_eq!(
            humanize_function_name("action_query_signal_strength"),
            "Query Signal Strength"
        );
        assert_eq!(humanize_function_name("send_at"), "Send At"); // no prefix
    }

    #[test]
    fn test_discover_actions_empty_lua() {
        let lua = Lua::new();
        let actions = discover_actions(&lua).unwrap();
        assert_eq!(actions.len(), 0);
    }

    #[test]
    fn test_discover_actions_with_functions() {
        let lua = Lua::new();

        // Define some action functions
        lua.load(
            r#"
            function action_send_at()
                return "sent"
            end

            function action_reset()
                return "reset"
            end
        "#,
        )
        .exec()
        .unwrap();

        let actions = discover_actions(&lua).unwrap();
        assert_eq!(actions.len(), 2);

        // Check that actions are sorted by label
        // "Reset" < "Send At" alphabetically
        assert_eq!(actions[0].function_name, "action_reset");
        assert_eq!(actions[0].label, "Reset");

        assert_eq!(actions[1].function_name, "action_send_at");
        assert_eq!(actions[1].label, "Send At");
    }

    #[test]
    fn test_discover_actions_with_metadata() {
        let lua = Lua::new();

        lua.load(
            r#"
            function action_send_at()
                return "sent"
            end

            _actions = {
                send_at = { label = "📡 Send AT Command", group = "Basic" }
            }
        "#,
        )
        .exec()
        .unwrap();

        let actions = discover_actions(&lua).unwrap();
        assert_eq!(actions.len(), 1);

        let action = &actions[0];
        assert_eq!(action.function_name, "action_send_at");
        assert_eq!(action.label, "📡 Send AT Command");
        assert_eq!(action.group, Some("Basic".to_string()));
    }

    #[test]
    fn test_execute_action() {
        let lua = Lua::new();

        lua.load(
            r#"
            function action_test()
                return 42
            end
        "#,
        )
        .exec()
        .unwrap();

        // execute_action returns Result<()>
        assert!(execute_action(&lua, "action_test").is_ok());

        // execute_action_string returns the result as "42"
        let result = execute_action_string(&lua, "action_test").unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_discover_actions_with_params() {
        let lua = Lua::new();

        lua.load(
            r#"
            function action_read_coils(slave, addr, count)
                return "read"
            end

            _actions = {
                read_coils = {
                    label = "📡 读线圈",
                    group = "Modbus",
                    params = {
                        { name = "slave", type = "number", default = 1, label = "从站地址" },
                        { name = "addr",  type = "number", default = 0, label = "起始地址" },
                        { name = "count", type = "number", label = "数量" },
                    },
                },
            }
        "#,
        )
        .exec()
        .unwrap();

        let actions = discover_actions(&lua).unwrap();
        assert_eq!(actions.len(), 1);

        let action = &actions[0];
        assert_eq!(action.params.len(), 3);
        assert_eq!(action.params[0].name, "slave");
        assert_eq!(action.params[0].param_type, "number");
        assert_eq!(action.params[0].default, Some("1".to_string()));
        assert_eq!(action.params[1].default, Some("0".to_string()));
        assert_eq!(action.params[2].default, None); // required param
    }

    #[test]
    fn test_execute_action_with_args() {
        let lua = Lua::new();

        lua.load(
            r#"
            function action_add(a, b)
                return a + b
            end
        "#,
        )
        .exec()
        .unwrap();

        let result = execute_action_with_args(&lua, "action_add", "[3, 7]").unwrap();
        assert_eq!(result, "10");
    }

    #[test]
    fn test_execute_action_with_string_args() {
        let lua = Lua::new();

        lua.load(
            r#"
            function action_greet(name)
                return "Hello " .. name
            end
        "#,
        )
        .exec()
        .unwrap();

        let result = execute_action_with_args(&lua, "action_greet", "[\"World\"]").unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_execute_action_with_empty_args() {
        let lua = Lua::new();

        lua.load(
            r#"
            function action_noop()
                return 42
            end
        "#,
        )
        .exec()
        .unwrap();

        let result = execute_action_with_args(&lua, "action_noop", "[]").unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_execute_action_nil_return() {
        let lua = Lua::new();

        lua.load(
            r#"
            function action_do_nothing()
                -- no return
            end
        "#,
        )
        .exec()
        .unwrap();

        let result = execute_action_string(&lua, "action_do_nothing").unwrap();
        assert_eq!(result, "ok");
    }

    #[test]
    fn test_execute_action_not_found() {
        let lua = Lua::new();
        let result = execute_action(&lua, "action_nonexistent");
        assert!(result.is_err());
    }
}
