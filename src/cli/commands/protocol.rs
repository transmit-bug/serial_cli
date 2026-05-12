//! Protocol command handler
//!
//! Handles listing, loading, unloading, validating, and hot-reloading
//! custom protocol scripts.

use crate::cli::types::ProtocolCommand;
use crate::config::ConfigManager;
use crate::error::{Result, SerialError};
use std::path::Path;

/// Built-in protocol name → description pairs.
const BUILT_IN_PROTOCOLS: &[(&str, &str)] = &[
    (
        "modbus_rtu",
        "Modbus RTU protocol (Industrial communication)",
    ),
    (
        "modbus_ascii",
        "Modbus ASCII protocol (Industrial communication)",
    ),
    ("at_command", "AT Command protocol (Modem control)"),
    ("line", "Line-based protocol (Text-based communication)"),
];

/// Dispatch a [`ProtocolCommand`] to the appropriate handler.
///
/// # Errors
///
/// Propagates validation errors, config errors, and I/O errors from
/// the underlying protocol operations.
pub fn handle_protocol_command(cmd: ProtocolCommand, json_output: bool) -> Result<()> {
    match cmd {
        ProtocolCommand::List { detailed } => list_protocols(detailed, json_output),
        ProtocolCommand::Info { name } => show_protocol_info(&name, json_output),
        ProtocolCommand::Validate { path } => validate_protocol(&path, json_output),
        ProtocolCommand::Load { path, name } => load_protocol(&path, name, json_output),
        ProtocolCommand::Unload { name } => unload_protocol(&name, json_output),
        ProtocolCommand::Reload { name } => reload_protocol(&name, json_output),
        ProtocolCommand::HotReload { action } => handle_hot_reload(&action, json_output),
    }
}

fn list_protocols(detailed: bool, json_output: bool) -> Result<()> {
    use serde_json::json;

    let config_manager = ConfigManager::load_with_fallback();
    let config = config_manager.get();
    let custom = &config.protocols.custom;

    if json_output {
        let mut protocols = Vec::new();

        // Built-in protocols
        for (name, desc) in BUILT_IN_PROTOCOLS {
            protocols.push(json!({
                "name": name,
                "description": desc,
                "type": "built-in"
            }));
        }

        // Custom protocols
        for proto in custom.values() {
            protocols.push(json!({
                "name": proto.name,
                "description": format!("{:?}", proto.path),
                "type": "custom",
                "loaded_at": proto.loaded_at
            }));
        }

        let result = json!({
            "protocols": protocols,
            "count": protocols.len()
        });

        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else {
        println!("Available protocols:");
        println!();

        // Built-in protocols
        if detailed {
            println!("Built-in protocols:");
            for (name, desc) in BUILT_IN_PROTOCOLS {
                println!("  {:15} - {}", name, desc);
            }
        } else {
            for (name, _) in BUILT_IN_PROTOCOLS {
                println!("  {}", name);
            }
        }

        // Custom protocols from config
        if !custom.is_empty() {
            println!();
            println!("Custom protocols:");
            if detailed {
                for proto in custom.values() {
                    println!(
                        "  {:15} - {} ({})",
                        proto.name,
                        proto.path.display(),
                        proto.loaded_at.as_deref().unwrap_or("unknown")
                    );
                }
            } else {
                for proto in custom.values() {
                    println!("  {}", proto.name);
                }
            }
        } else if !detailed {
            println!();
            println!("Custom protocols: (none loaded)");
            println!("Use 'serial-cli protocol load <script.lua>' to add custom protocols");
        }
    }

    Ok(())
}

fn show_protocol_info(name: &str, _json_output: bool) -> Result<()> {
    println!("Protocol: {}", name);

    // Check built-in
    if let Some((_, desc)) = BUILT_IN_PROTOCOLS.iter().find(|(n, _)| *n == name) {
        println!("Type: Built-in");
        println!("Description: {}", desc);
        return Ok(());
    }

    // Check custom protocols in config
    let config_manager = ConfigManager::load_with_fallback();
    if let Some(proto) = config_manager.get_custom_protocol(name) {
        println!("Type: Custom");
        println!("Script: {}", proto.path.display());
        println!("Version: {}", proto.version);
        println!(
            "Loaded: {}",
            proto.loaded_at.as_deref().unwrap_or("unknown")
        );
        return Ok(());
    }

    Err(SerialError::Config(format!(
        "Protocol '{}' not found in built-in or custom protocols",
        name
    )))
}

fn validate_protocol(path: &Path, _json_output: bool) -> Result<()> {
    use crate::protocol::ProtocolValidator;

    println!("Validating protocol script: {}", path.display());
    match ProtocolValidator::validate_script(path) {
        Ok(_) => println!("\u{2713} Protocol script is valid"),
        Err(e) => {
            println!("\u{2717} Validation failed: {}", e);
            return Err(e);
        }
    }
    Ok(())
}

fn load_protocol(path: &Path, name: Option<String>, _json_output: bool) -> Result<()> {
    use crate::protocol::ProtocolValidator;

    // Validate script first
    if let Err(e) = ProtocolValidator::validate_script(path) {
        println!("\u{2717} Script validation failed: {}", e);
        return Err(e);
    }

    // Determine protocol name
    let proto_name = name.unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    });

    // Check it's not a built-in name
    if BUILT_IN_PROTOCOLS.iter().any(|(n, _)| *n == proto_name) {
        return Err(SerialError::Config(format!(
            "Cannot load: '{}' is a reserved built-in protocol name",
            proto_name
        )));
    }

    // Resolve to absolute path so reload works from any directory
    let abs_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    // Save to config (add_custom_protocol returns error if already exists)
    let config_manager = ConfigManager::load_with_fallback();
    config_manager.add_custom_protocol(proto_name.clone(), abs_path)?;
    config_manager.save(None)?;

    println!("\u{2713} Protocol loaded: {}", proto_name);
    println!("  Script: {}", path.display());
    println!("  Saved to configuration");
    println!();
    println!("The protocol will be available in interactive mode after restart.");
    Ok(())
}

fn unload_protocol(name: &str, _json_output: bool) -> Result<()> {
    // Check it's not a built-in
    if BUILT_IN_PROTOCOLS.iter().any(|(n, _)| *n == name) {
        return Err(SerialError::Config(format!(
            "Cannot unload built-in protocol: {}",
            name
        )));
    }

    let config_manager = ConfigManager::load_with_fallback();

    // remove_custom_protocol returns error if not found
    config_manager.remove_custom_protocol(name)?;
    config_manager.save(None)?;

    println!("\u{2713} Protocol unloaded: {}", name);
    println!("  Removed from configuration");
    Ok(())
}

fn reload_protocol(name: &str, _json_output: bool) -> Result<()> {
    use crate::protocol::ProtocolValidator;

    let config_manager = ConfigManager::load_with_fallback();

    // Get existing protocol path
    let existing = config_manager.get_custom_protocol(name).ok_or_else(|| {
        SerialError::Config(format!(
            "Custom protocol not found: {}. Use 'protocol list' to see loaded protocols.",
            name
        ))
    })?;

    let script_path = existing.path.clone();

    // Validate the script still exists and is valid
    if !script_path.exists() {
        return Err(SerialError::Config(format!(
            "Script file not found: {}. The protocol may have been moved or deleted.",
            script_path.display()
        )));
    }

    if let Err(e) = ProtocolValidator::validate_script(&script_path) {
        println!("\u{2717} Script validation failed: {}", e);
        return Err(e);
    }

    // Update in single atomic operation (avoids gap between remove+add)
    config_manager.update_custom_protocol(name.to_string(), script_path.clone())?;
    config_manager.save(None)?;

    println!("\u{2713} Protocol reloaded: {}", name);
    println!("  Script: {}", script_path.display());
    Ok(())
}

fn handle_hot_reload(action: &str, _json_output: bool) -> Result<()> {
    let config_manager = ConfigManager::load_with_fallback();

    match action {
        "enable" => {
            println!("Enabling protocol hot-reload...");
            config_manager.set("protocols.hot_reload", "true")?;
            config_manager.save(None)?;
            println!("\u{2713} Protocol hot-reload enabled");
            println!();
            println!("Custom protocols will now be automatically reloaded when modified.");
        }
        "disable" => {
            println!("Disabling protocol hot-reload...");
            config_manager.set("protocols.hot_reload", "false")?;
            config_manager.save(None)?;
            println!("\u{2713} Protocol hot-reload disabled");
        }
        "status" => {
            let enabled = config_manager.is_hot_reload_enabled();
            println!("Protocol hot-reload status:");
            println!();
            if enabled {
                println!("  Status: \u{1F7E2} Enabled");
                println!("  Custom protocols will be automatically reloaded when modified.");
            } else {
                println!("  Status: \u{1F6AB} Disabled");
                println!("  Use 'protocol hot-reload enable' to enable automatic reloading.");
            }
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
