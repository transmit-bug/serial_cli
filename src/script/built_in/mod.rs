//! Built-in scripts
//!
//! Lua scripts bundled with the binary that implement common protocols.

/// Built-in script entry: (name, description, lua_source).
pub struct BuiltInScript {
    pub name: &'static str,
    pub description: &'static str,
    pub source: &'static str,
}

/// Return all built-in scripts.
pub fn all_built_in() -> Vec<BuiltInScript> {
    vec![
        BuiltInScript {
            name: "line",
            description: "Line-based protocol (text-based communication)",
            source: include_str!("line.lua"),
        },
        BuiltInScript {
            name: "at_command",
            description: "AT Command protocol (modem control)",
            source: include_str!("at_command.lua"),
        },
        BuiltInScript {
            name: "modbus_rtu",
            description: "Modbus RTU protocol (industrial communication)",
            source: include_str!("modbus_rtu.lua"),
        },
    ]
}
