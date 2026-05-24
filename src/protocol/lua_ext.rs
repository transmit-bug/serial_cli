//! Lua protocol extension interface
//!
//! This module allows users to define custom protocols in Lua.
//! Custom protocol scripts get access to all tool functions via [`ScriptRuntime`](crate::lua::runtime::ScriptRuntime).

use crate::error::{ProtocolError, Result, SerialError};
use crate::lua::runtime::ScriptRuntime;
use crate::protocol::{Protocol, ProtocolStats};
use mlua::{Function, Lua, Value};

/// Custom protocol defined in Lua.
///
/// The Lua VM is lazily initialized on first use and reused across subsequent
/// `parse`/`encode` calls, allowing scripts to maintain state (counters,
/// buffers, etc.). Clones start with a fresh VM.
pub struct LuaProtocol {
    name: String,
    script: Option<String>,
    /// Cached Lua VM — lazily initialized.
    lua: Option<Lua>,
    stats: ProtocolStats,
}

impl Clone for LuaProtocol {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            script: self.script.clone(),
            lua: None,
            stats: self.stats.clone(),
        }
    }
}

// Safety: LuaProtocol is only mutated through &mut self (parse/encode),
// so the Lua VM is never accessed concurrently.
unsafe impl Send for LuaProtocol {}
unsafe impl Sync for LuaProtocol {}

impl LuaProtocol {
    /// Create a new Lua protocol from a Lua script
    pub fn from_script(name: String, script: &str) -> Result<Self> {
        // Validate the script syntax by trying to load it
        let lua = Lua::new();
        if let Err(e) = lua.load(script).exec() {
            return Err(SerialError::Protocol(ProtocolError::InvalidFrame(format!(
                "Invalid Lua script: {}",
                e
            ))));
        }

        Ok(Self {
            name,
            script: Some(script.to_string()),
            lua: None,
            stats: ProtocolStats::default(),
        })
    }

    /// Create a new empty Lua protocol
    pub fn new(name: String) -> Result<Self> {
        Ok(Self {
            name,
            script: None,
            lua: None,
            stats: ProtocolStats::default(),
        })
    }

    /// Get the script content
    pub fn script(&self) -> Option<&String> {
        self.script.as_ref()
    }

    /// Ensure the Lua VM is initialized with the script.
    fn ensure_lua(&mut self) -> Result<()> {
        if self.lua.is_none() {
            if let Some(ref script) = self.script {
                let lua = Lua::new();
                let _ = ScriptRuntime::register_all(&lua);
                lua.load(script).exec().map_err(|e| {
                    SerialError::Protocol(ProtocolError::InvalidFrame(format!(
                        "Failed to load Lua script: {}",
                        e
                    )))
                })?;
                self.lua = Some(lua);
            }
        }
        Ok(())
    }

    /// Execute Lua callback on the cached VM.
    fn execute_callback(&mut self, callback_name: &str, data: &[u8]) -> Result<Vec<u8>> {
        self.ensure_lua()?;

        let lua = match &self.lua {
            Some(l) => l,
            None => return Ok(data.to_vec()),
        };

        let globals = lua.globals();

        // Set global data variable
        let data_table = lua.create_table()?;
        for (i, &byte) in data.iter().enumerate() {
            data_table.set(i + 1, byte)?;
        }
        globals.set("data", data_table.clone())?;

        // Get the callback function
        let callback: Function = globals.get(callback_name).map_err(|_| {
            SerialError::Protocol(ProtocolError::InvalidFrame(format!(
                "Callback function '{}' not found",
                callback_name
            )))
        })?;

        // Call the callback function
        let result = callback.call::<_, Value>(data_table);

        match result {
            Ok(Value::Table(table)) => {
                let mut bytes = Vec::new();
                let len = table.len().unwrap_or(0);
                for i in 1..=len {
                    let byte: u8 = table.get(i).unwrap_or(0);
                    bytes.push(byte);
                }
                Ok(bytes)
            }
            Ok(Value::String(s)) => Ok(s.to_str().unwrap_or("").as_bytes().to_vec()),
            Ok(Value::Integer(n)) => Ok(vec![n as u8]),
            Ok(Value::Number(n)) => Ok(vec![n as u8]),
            Ok(_) | Err(_) => Ok(data.to_vec()),
        }
    }
}

impl Protocol for LuaProtocol {
    fn name(&self) -> &str {
        &self.name
    }

    fn parse(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        self.stats.frames_parsed += 1;
        self.execute_callback("on_frame", data)
            .or_else(|_| Ok(data.to_vec()))
    }

    fn encode(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        self.stats.frames_encoded += 1;
        self.execute_callback("on_encode", data)
            .or_else(|_| Ok(data.to_vec()))
    }

    fn reset(&mut self) -> Result<()> {
        // Discard cached VM — next call will reinitialize
        self.lua = None;
        Ok(())
    }

    fn stats(&self) -> ProtocolStats {
        self.stats.clone()
    }
}

/// Helper to create a Lua protocol from a string
pub fn create_lua_protocol(name: String, script: &str) -> Result<Box<dyn Protocol>> {
    let protocol = LuaProtocol::from_script(name, script)?;
    Ok(Box::new(protocol))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_protocol_basic() {
        let protocol = LuaProtocol::new("test".to_string()).unwrap();
        assert_eq!(protocol.name(), "test");
        assert!(protocol.script().is_none());

        // Test default behavior (passthrough when no script)
        let mut protocol = protocol;
        let data = vec![0x01, 0x02, 0x03];
        let encoded = protocol.encode(&data).unwrap();
        assert_eq!(encoded, data);

        let parsed = protocol.parse(&data).unwrap();
        assert_eq!(parsed, data);
    }

    #[test]
    fn test_lua_protocol_clone() {
        let protocol1 = LuaProtocol::new("clone_test".to_string()).unwrap();
        let protocol2 = protocol1.clone();
        assert_eq!(protocol1.name(), protocol2.name());
    }

    #[test]
    fn test_lua_protocol_from_script() {
        let script = r#"
            function on_frame(data)
                return data
            end

            function on_encode(data)
                return data
            end
        "#;

        let protocol = LuaProtocol::from_script("custom".to_string(), script).unwrap();
        assert_eq!(protocol.name(), "custom");
        assert!(protocol.script().is_some());
        assert_eq!(protocol.script().unwrap(), script);
    }

    #[test]
    fn test_lua_protocol_stats() {
        let mut protocol = LuaProtocol::new("stats_test".to_string()).unwrap();
        let data = vec![0x01, 0x02];

        protocol.encode(&data).unwrap();
        protocol.parse(&data).unwrap();

        let stats = protocol.stats();
        assert_eq!(stats.frames_encoded, 1);
        assert_eq!(stats.frames_parsed, 1);
    }

    #[test]
    fn test_lua_protocol_with_simple_callback() {
        // Test with a simple Lua callback that modifies data
        let script = r#"
            function on_frame(data)
                -- Return first 2 bytes only
                local result = {}
                for i = 1, math.min(2, #data) do
                    table.insert(result, data[i])
                end
                return result
            end

            function on_encode(data)
                return data
            end
        "#;

        let mut protocol = LuaProtocol::from_script("callback_test".to_string(), script).unwrap();

        // Test parsing - should return only first 2 bytes
        let input = vec![0x10, 0x20, 0x30, 0x40];
        let parsed = protocol.parse(&input).unwrap();
        assert_eq!(parsed, vec![0x10, 0x20]);
    }

    #[test]
    fn test_lua_protocol_with_string_result() {
        // Test callback that returns a string
        let script = r#"
            function on_frame(data)
                return "processed"
            end

            function on_encode(data)
                return data
            end
        "#;

        let mut protocol = LuaProtocol::from_script("string_test".to_string(), script).unwrap();

        let data = vec![0x01, 0x02, 0x03];
        let parsed = protocol.parse(&data).unwrap();
        // Should return the string as bytes
        assert_eq!(parsed, b"processed");
    }

    #[test]
    fn test_lua_protocol_reset() {
        let script = r#"
            local reset_called = false

            function on_frame(data)
                return data
            end

            function on_encode(data)
                return data
            end

            function on_reset()
                reset_called = true
            end
        "#;

        let mut protocol = LuaProtocol::from_script("reset_test".to_string(), script).unwrap();
        protocol.reset().unwrap();
        // Should not error even with reset callback
    }

    #[test]
    fn test_lua_protocol_error_handling() {
        // Test that errors in callbacks don't crash
        let script = r#"
            function on_frame(data)
                error("Intentional error")
            end

            function on_encode(data)
                return data
            end
        "#;

        let mut protocol = LuaProtocol::from_script("error_test".to_string(), script).unwrap();

        let data = vec![0x01, 0x02, 0x03];
        // Should fallback to passthrough on error
        let parsed = protocol.parse(&data).unwrap();
        assert_eq!(parsed, data);
    }
}
