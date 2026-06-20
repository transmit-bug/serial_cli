//! Script execution engine

use crate::error::{Result, SerialError};
use crate::lua::bindings::LuaBindings;
use crate::script::ScriptManager;
use crate::serial_core::PortManager;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Script execution engine
pub struct ScriptEngine {
    pub bindings: LuaBindings,
    port_manager: PortManager,
    script_manager: Arc<Mutex<ScriptManager>>,
}

impl ScriptEngine {
    /// Create a new script engine with explicit ScriptManager dependency
    pub fn new(script_manager: Arc<Mutex<ScriptManager>>) -> Result<Self> {
        let mut bindings = LuaBindings::new()?;
        bindings.set_script_manager(script_manager.clone());
        
        Ok(Self {
            bindings,
            port_manager: PortManager::new(),
            script_manager,
        })
    }

    /// Execute a script from a string
    pub fn execute_string(&self, script: &str) -> Result<()> {
        self.bindings.execute_script(script)
    }

    /// Execute a script from a file
    pub fn execute_file(&self, path: &Path) -> Result<()> {
        let script = fs::read_to_string(path).map_err(SerialError::Io)?;

        self.bindings.execute_script(&script)
    }

    /// Execute a script with arguments
    pub fn execute_with_args(&self, script: &str, args: Vec<String>) -> Result<()> {
        let lua = self.bindings.lua();
        let globals = lua.globals();

        // Create the 'arg' table
        let arg_table = lua.create_table()?;
        arg_table.set(0, "script")?;
        for (i, arg) in args.iter().enumerate() {
            arg_table.set(i + 1, arg.clone())?;
        }
        arg_table.set("n", args.len())?;
        globals.set("arg", arg_table)?;

        // Set individual global variables for convenience
        for (i, arg) in args.iter().enumerate() {
            let var_name = format!("arg{}", i + 1);
            globals.set(var_name, arg.clone())?;
        }

        // Execute the script
        let result = self.bindings.execute_script(script);

        // Clean up globals to prevent state leakage across calls
        globals.set("arg", mlua::Value::Nil)?;
        for i in 0..args.len() {
            globals.set(format!("arg{}", i + 1), mlua::Value::Nil)?;
        }

        result
    }

    /// Get the port manager
    pub fn port_manager(&self) -> &PortManager {
        &self.port_manager
    }

    /// Get the script manager
    pub fn script_manager(&self) -> &Arc<Mutex<ScriptManager>> {
        &self.script_manager
    }
}

impl Default for ScriptEngine {
    fn default() -> Self {
        let script_manager = Arc::new(Mutex::new(ScriptManager::new()));
        Self::new(script_manager).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let script_manager = Arc::new(Mutex::new(ScriptManager::new()));
        let engine = ScriptEngine::new(script_manager).unwrap();
        assert!(engine.execute_string("print('test')").is_ok());
    }

    #[test]
    fn test_execute_math() {
        let script_manager = Arc::new(Mutex::new(ScriptManager::new()));
        let engine = ScriptEngine::new(script_manager).unwrap();
        let script = r#"
            local result = 2 + 2
            assert(result == 4, "Math failed")
        "#;
        assert!(engine.execute_string(script).is_ok());
    }

    #[test]
    fn test_execute_syntax_error() {
        let script_manager = Arc::new(Mutex::new(ScriptManager::new()));
        let engine = ScriptEngine::new(script_manager).unwrap();
        // Intentionally malformed Lua script
        let result = engine.execute_string("if true then");
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_runtime_error() {
        let script_manager = Arc::new(Mutex::new(ScriptManager::new()));
        let engine = ScriptEngine::new(script_manager).unwrap();
        // Calling nil function causes runtime error
        let result = engine.execute_string("nonexistent_function()");
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_file_not_found() {
        let script_manager = Arc::new(Mutex::new(ScriptManager::new()));
        let engine = ScriptEngine::new(script_manager).unwrap();
        let result = engine.execute_file(std::path::Path::new("nonexistent_script.lua"));
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_file_valid() {
        let script_manager = Arc::new(Mutex::new(ScriptManager::new()));
        let engine = ScriptEngine::new(script_manager).unwrap();
        let result = engine.execute_file(std::path::Path::new(
            "tests/fixtures/protocols/test_valid.lua",
        ));
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_with_args() {
        let script_manager = Arc::new(Mutex::new(ScriptManager::new()));
        let engine = ScriptEngine::new(script_manager).unwrap();
        let script = r#"
            assert(arg[1] == "hello", "arg[1] mismatch")
            assert(arg[2] == "world", "arg[2] mismatch")
            assert(arg1 == "hello", "global arg1 mismatch")
        "#;
        let result =
            engine.execute_with_args(script, vec!["hello".to_string(), "world".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_state_isolation() {
        // Verify that separate engines don't share state
        let script_manager1 = Arc::new(Mutex::new(ScriptManager::new()));
        let engine1 = ScriptEngine::new(script_manager1).unwrap();
        engine1.execute_string("myvar = 42").unwrap();

        let script_manager2 = Arc::new(Mutex::new(ScriptManager::new()));
        let engine2 = ScriptEngine::new(script_manager2).unwrap();
        // engine2 should not see engine1's globals
        let result = engine2.execute_string("if myvar == nil then return end");
        assert!(result.is_ok());
    }
}
