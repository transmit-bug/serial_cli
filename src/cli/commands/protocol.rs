//! Protocol command handler
//!
//! Handles listing, loading, unloading, validating, and hot-reloading
//! custom protocol scripts. Uses ScriptManager (unified script system).

use crate::cli::types::ProtocolCommand;
use crate::error::{Result, ScriptError, SerialError};
use crate::script::ScriptManager;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Dispatch a [`ProtocolCommand`] to the appropriate handler.
///
/// # Errors
///
/// Propagates validation errors, config errors, and I/O errors from
/// the underlying script operations.
pub async fn handle_protocol_command(
    cmd: ProtocolCommand,
    json_output: bool,
    script_manager: Arc<Mutex<ScriptManager>>,
) -> Result<()> {
    match cmd {
        ProtocolCommand::List { detailed } => list_scripts(detailed, json_output, script_manager).await,
        ProtocolCommand::Info { name } => show_script_info(&name, json_output, script_manager).await,
        ProtocolCommand::Validate { path } => validate_script(&path, json_output),
        ProtocolCommand::Load { path, .. } => load_script(&path, json_output, script_manager).await,
        ProtocolCommand::Unload { name } => unload_script(&name, json_output, script_manager).await,
        ProtocolCommand::Reload { name } => reload_script(&name, json_output, script_manager).await,
        ProtocolCommand::HotReload { action } => handle_hot_reload(&action, json_output),
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
            println!("Use 'serial-cli protocol load <script.lua>' to add custom scripts");
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
    println!("Type: {}", if meta.built_in { "Built-in" } else { "Custom" });
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
            config_manager.set("protocols.hot_reload", "true")?;
            config_manager.save(None)?;
            println!("\u{2713} Script hot-reload enabled");
        }
        "disable" => {
            println!("Disabling script hot-reload...");
            config_manager.set("protocols.hot_reload", "false")?;
            config_manager.save(None)?;
            println!("\u{2713} Script hot-reload disabled");
        }
        "status" => {
            let enabled = config_manager.is_hot_reload_enabled();
            println!("Script hot-reload status: {}", if enabled { "Enabled" } else { "Disabled" });
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
