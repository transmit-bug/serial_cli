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
/// Calls the specified Lua function with no arguments.
/// Returns the Lua value returned by the function (if any).
pub fn execute_action(lua: &Lua, function_name: &str) -> Result<()> {
    let globals = lua.globals();
    let func = globals
        .get::<_, mlua::Function>(function_name)
        .map_err(|e| SerialError::Lua(e))?;

    // Call with no arguments
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
        .map_err(|e| SerialError::Lua(e))?;

    let result = func.call::<_, Value>(()).map_err(SerialError::Lua)?;

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
