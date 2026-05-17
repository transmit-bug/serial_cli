//! Lua script execution command
//!
//! Handles `serial-cli run <script.lua> [args...]`.

use std::path::PathBuf;

use crate::error::Result;
use crate::lua::executor::ScriptEngine;
use crate::lua::ScriptRuntime;

/// Load and execute a Lua script with optional command-line arguments.
///
/// The script is executed in the following order:
/// 1. Create a `ScriptEngine` and register all built-in APIs
/// 2. Register standard library utilities (`json`, `http`, `fs`, etc.)
/// 3. Read the script file from disk
/// 4. Execute the script, passing `args` as a Lua table (if any)
///
/// # Arguments
///
/// * `path` - Path to the `.lua` script file
/// * `args` - Arguments forwarded to the script as a Lua table
///
/// # Errors
///
/// Returns an `Io` error if the script file cannot be read.
/// Returns a `Lua` error if the script fails to compile or execute.
/// Returns a `Script` error for sandbox violations or resource limits.
pub async fn run_lua_script(path: PathBuf, args: Vec<String>) -> Result<()> {
    // 1. Create script engine
    let engine = ScriptEngine::new()?;

    // 2. Register all available APIs
    engine.bindings.register_all_apis()?;

    // 3. Register runtime utilities (log, json, hex, string/bytes, time)
    let lua = engine.bindings.lua();
    ScriptRuntime::register_all(lua)?;

    // 4. Read script file
    let script_content = std::fs::read_to_string(&path).map_err(crate::error::SerialError::Io)?;

    // 5. Execute script with arguments
    if args.is_empty() {
        engine.execute_file(&path)?;
    } else {
        tracing::info!("Executing script with arguments: {:?}", args);
        engine.execute_with_args(&script_content, args)?;
    }

    Ok(())
}
