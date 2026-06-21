//! Script command handler
//!
//! Handles listing, loading, unloading, validating, and hot-reloading
//! custom Lua scripts. Also handles `serial-cli run <script.lua> [args...]`.
//! Uses ScriptManager (unified script system).

use crate::cli::types::ScriptCommand;
use crate::error::{Result, ScriptError, SerialError};
use crate::lua::executor::ScriptEngine;
use crate::lua::ScriptRuntime;
use crate::script::ScriptManager;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Load and execute a Lua script with optional command-line arguments.
///
/// The script is executed in the following order:
/// 1. Create a `ScriptEngine` with shared ScriptManager
/// 2. Register all built-in APIs
/// 3. Register standard library utilities (`json`, `http`, `fs`, etc.)
/// 4. Read the script file from disk
/// 5. Execute the script, passing `args` as a Lua table (if any)
///
/// # Arguments
///
/// * `path` - Path to the `.lua` script file
/// * `args` - Arguments forwarded to the script as a Lua table
/// * `script_manager` - Shared ScriptManager for script discovery and state
///
/// # Errors
///
/// Returns an `Io` error if the script file cannot be read.
/// Returns a `Lua` error if the script fails to compile or execute.
/// Returns a `Script` error for sandbox violations or resource limits.
pub async fn run_lua_script(
    path: PathBuf,
    args: Vec<String>,
    script_manager: Arc<Mutex<ScriptManager>>,
) -> Result<()> {
    // 1. Create script engine with shared ScriptManager
    let engine = ScriptEngine::new(script_manager)?;

    // 2. Register all available APIs
    engine.bindings.register_all_apis()?;

    // 3. Register runtime utilities (log, json, hex, string/bytes, time)
    let lua = engine.bindings.lua();
    ScriptRuntime::register_all(lua)?;

    // 4. Read script file
    let script_content = std::fs::read_to_string(&path).map_err(SerialError::Io)?;

    // 5. Execute script with arguments
    if args.is_empty() {
        engine.execute_file(&path)?;
    } else {
        engine.execute_with_args(&script_content, args)?;
    }

    Ok(())
}

/// Dispatch a [`ScriptCommand`] to the appropriate handler.
///
/// # Errors
///
/// Propagates validation errors, config errors, and I/O errors from
/// the underlying script operations.
pub async fn handle_script_command(
    cmd: ScriptCommand,
    json_output: bool,
    script_manager: Arc<Mutex<ScriptManager>>,
) -> Result<()> {
    match cmd {
        ScriptCommand::List { detailed } => {
            list_scripts(detailed, json_output, script_manager).await
        }
        ScriptCommand::Info { name } => show_script_info(&name, json_output, script_manager).await,
        ScriptCommand::Validate { path } => validate_script(&path, json_output),
        ScriptCommand::Load { path, .. } => load_script(&path, json_output, script_manager).await,
        ScriptCommand::Unload { name } => unload_script(&name, json_output, script_manager).await,
        ScriptCommand::Reload { name } => reload_script(&name, json_output, script_manager).await,
        ScriptCommand::HotReload { action } => handle_hot_reload(&action, json_output),
    }
}

async fn list_scripts(
    detailed: bool,
    json_output: bool,
    script_manager: Arc<Mutex<ScriptManager>>,
) -> Result<()> {
    use serde_json::json;

    let manager = script_manager.lock().await;
    let scripts = manager.list();

    if json_output {
        let items: Vec<serde_json::Value> = scripts
            .iter()
            .map(|s| {
                json!({
                    "name": s.name,
                    "description": s.description,
                    "type": if s.built_in { "built-in" } else { "custom" }
                })
            })
            .collect();

        let result = json!({
            "scripts": items,
            "count": items.len()
        });

        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else {
        println!("Available scripts:");
        println!();

        let built_in: Vec<_> = scripts.iter().filter(|s| s.built_in).collect();
        let custom: Vec<_> = scripts.iter().filter(|s| !s.built_in).collect();

        if detailed {
            println!("Built-in scripts:");
            for s in &built_in {
                println!("  {:15} - {}", s.name, s.description);
            }
        } else {
            for s in &built_in {
                println!("  {}", s.name);
            }
        }

        if !custom.is_empty() {
            println!();
            println!("Custom scripts:");
            for s in &custom {
                if detailed {
                    println!("  {:15} - {}", s.name, s.description);
                } else {
                    println!("  {}", s.name);
                }
            }
        } else if !detailed {
            println!();
            println!("Custom scripts: (none loaded)");
            println!("Use 'serial-cli script load <script.lua>' to add custom scripts");
        }
    }

    Ok(())
}

async fn show_script_info(
    name: &str,
    _json_output: bool,
    script_manager: Arc<Mutex<ScriptManager>>,
) -> Result<()> {
    let manager = script_manager.lock().await;
    let meta = manager.get_meta(name)?;

    println!("Script: {}", meta.name);
    println!(
        "Type: {}",
        if meta.built_in { "Built-in" } else { "Custom" }
    );
    println!("Description: {}", meta.description);
    if let Some(ref path) = meta.path {
        println!("Path: {}", path.display());
    }
    println!("Version: {}", meta.version);

    Ok(())
}

fn validate_script(path: &Path, _json_output: bool) -> Result<()> {
    println!("Validating script: {}", path.display());

    // Read the file
    let source = std::fs::read_to_string(path).map_err(SerialError::Io)?;

    // Validate Lua syntax
    let lua = mlua::Lua::new();
    match lua.load(&source).exec() {
        Ok(_) => {
            println!("\u{2713} Script is valid");
            Ok(())
        }
        Err(e) => {
            println!("\u{2717} Validation failed: {}", e);
            Err(SerialError::Script(ScriptError::Syntax {
                script: path.to_path_buf(),
                line: 0,
                message: e.to_string(),
            }))
        }
    }
}

async fn load_script(
    path: &Path,
    _json_output: bool,
    script_manager: Arc<Mutex<ScriptManager>>,
) -> Result<()> {
    let mut manager = script_manager.lock().await;

    // Validate syntax first
    validate_script(path, false)?;

    let info = manager.load(path)?;

    println!("\u{2713} Script loaded: {}", info.name);
    println!("  Path: {}", path.display());
    Ok(())
}

async fn unload_script(
    name: &str,
    _json_output: bool,
    script_manager: Arc<Mutex<ScriptManager>>,
) -> Result<()> {
    let mut manager = script_manager.lock().await;
    manager.unload(name)?;

    println!("\u{2713} Script unloaded: {}", name);
    Ok(())
}

async fn reload_script(
    name: &str,
    _json_output: bool,
    script_manager: Arc<Mutex<ScriptManager>>,
) -> Result<()> {
    let mut manager = script_manager.lock().await;
    manager.reload(name)?;

    println!("\u{2713} Script reloaded: {}", name);
    Ok(())
}

fn handle_hot_reload(action: &str, _json_output: bool) -> Result<()> {
    // Hot-reload config is still managed via ConfigManager for now
    let config_manager = crate::config::ConfigManager::load_with_fallback();

    match action {
        "enable" => {
            println!("Enabling script hot-reload...");
            config_manager.set("scripts.hot_reload", "true")?;
            config_manager.save(None)?;
            println!("\u{2713} Script hot-reload enabled");
        }
        "disable" => {
            println!("Disabling script hot-reload...");
            config_manager.set("scripts.hot_reload", "false")?;
            config_manager.save(None)?;
            println!("\u{2713} Script hot-reload disabled");
        }
        "status" => {
            let enabled = config_manager.is_hot_reload_enabled();
            println!(
                "Script hot-reload status: {}",
                if enabled { "Enabled" } else { "Disabled" }
            );
        }
        _ => {
            return Err(SerialError::InvalidInput(format!(
                "Unknown action: {}. Valid actions: enable, disable, status",
                action
            )));
        }
    }

    Ok(())
}
