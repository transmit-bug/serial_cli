//! Lua engine

use crate::error::Result;
use mlua::Lua;

/// Lua engine wrapper.
/// 
/// For production use, prefer `SerialScriptEngine` (serial_script.rs) or `LuaBindings` (bindings.rs).
#[allow(dead_code)]
pub struct LuaEngine {
    lua: Lua,
}

impl LuaEngine {
    /// Create a new Lua engine
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        Ok(Self { lua })
    }

    /// Get the underlying Lua instance
    pub fn lua(&self) -> &Lua {
        &self.lua
    }
}

impl Default for LuaEngine {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
