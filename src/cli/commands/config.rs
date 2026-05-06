//! Config command handler
//!
//! Handles `serial-cli config show|set|save|reset`.

use crate::cli::types::ConfigCommand;
use crate::config::ConfigManager;
use crate::error::Result;

/// Dispatch a [`ConfigCommand`] to show, set, save, or reset configuration.
///
/// # Errors
///
/// Propagates errors from [`ConfigManager`] operations (validation failures,
/// I/O errors, invalid key/value pairs).
pub fn handle_config_command(cmd: ConfigCommand) -> Result<()> {
    let config_manager = ConfigManager::load_with_fallback();

    match cmd {
        ConfigCommand::Show { json } => {
            let config = config_manager.get();
            if json {
                println!("{}", serde_json::to_string_pretty(&config).unwrap());
            } else {
                println!("Current configuration:");
                println!();
                println!("[serial]");
                println!("  baudrate = {}", config.serial.baudrate);
                println!("  databits = {}", config.serial.databits);
                println!("  stopbits = {}", config.serial.stopbits);
                println!("  parity = \"{}\"", config.serial.parity);
                println!("  timeout_ms = {}", config.serial.timeout_ms);
                println!();
                println!("[logging]");
                println!("  level = \"{}\"", config.logging.level);
                println!("  format = \"{}\"", config.logging.format);
                println!("  file = \"{}\"", config.logging.file);
                println!();
                println!("[lua]");
                println!("  memory_limit_mb = {}", config.lua.memory_limit_mb);
                println!("  timeout_seconds = {}", config.lua.timeout_seconds);
                println!("  enable_sandbox = {}", config.lua.enable_sandbox);
                println!();
                println!("[task]");
                println!("  max_concurrent = {}", config.task.max_concurrent);
                println!(
                    "  default_timeout_seconds = {}",
                    config.task.default_timeout_seconds
                );
                println!();
                println!("[output]");
                println!("  json_pretty = {}", config.output.json_pretty);
                println!("  show_timestamp = {}", config.output.show_timestamp);
                println!();
                println!("[virtual]");
                println!("  backend = \"{}\"", config.virtual_ports.backend);
                println!("  monitor = {}", config.virtual_ports.monitor);
                println!(
                    "  monitor_format = \"{}\"",
                    config.virtual_ports.monitor_format
                );
                println!("  auto_cleanup = {}", config.virtual_ports.auto_cleanup);
                println!("  max_packets = {}", config.virtual_ports.max_packets);
                println!(
                    "  bridge_buffer_size = {}",
                    config.virtual_ports.bridge_buffer_size
                );
                println!(
                    "  bridge_poll_interval_ms = {}",
                    config.virtual_ports.bridge_poll_interval_ms
                );
                println!();
                println!("Use 'config set <key> <value>' to modify configuration");
                println!("Use 'config save [path]' to save configuration to file");
                println!("Use 'config reset' to reset to defaults");
            }
        }
        ConfigCommand::Set { key, value } => {
            match config_manager.set(&key, &value) {
                Ok(_) => {
                    println!("\u{2713} Configuration updated successfully");

                    if let Err(e) = config_manager.validate() {
                        println!("\u{26A0} Warning: Configuration may be invalid: {}", e);
                    }

                    println!("Note: Use 'config save' to persist changes");
                }
                Err(e) => {
                    println!("\u{2717} Failed to set configuration: {}", e);
                    println!();
                    println!("Valid configuration keys:");
                    println!("  serial.baudrate              - Baud rate (e.g., 115200)");
                    println!("  serial.databits              - Data bits (5-8)");
                    println!("  serial.stopbits              - Stop bits (1-2)");
                    println!("  serial.parity                - Parity (none/odd/even)");
                    println!("  serial.timeout_ms            - Timeout in milliseconds");
                    println!(
                        "  logging.level                - Log level (error/warn/info/debug/trace)"
                    );
                    println!("  logging.format               - Log format (text/json)");
                    println!("  logging.file                 - Log file path");
                    println!("  lua.memory_limit_mb          - Lua memory limit");
                    println!("  lua.timeout_seconds          - Lua timeout");
                    println!("  lua.enable_sandbox           - Enable Lua sandbox");
                    println!("  task.max_concurrent          - Max concurrent tasks");
                    println!("  task.default_timeout_seconds - Default task timeout");
                    println!("  output.json_pretty           - Pretty print JSON");
                    println!("  output.show_timestamp        - Show timestamps");
                    println!("  virtual.backend              - Virtual port backend (pty/socat/namedpipe)");
                    println!("  virtual.monitor              - Enable monitoring by default");
                    println!("  virtual.monitor_format       - Monitor format (hex/raw)");
                    println!("  virtual.auto_cleanup         - Auto-cleanup on exit");
                    println!("  virtual.max_packets          - Max packets to capture");
                    println!("  virtual.bridge_buffer_size   - Bridge buffer size");
                    println!("  virtual.bridge_poll_interval_ms - Bridge poll interval");
                    return Err(e);
                }
            }
        }
        ConfigCommand::Save { path } => {
            let output_path = path.unwrap_or_else(|| {
                if let Some(global_path) = crate::config::get_global_config_path() {
                    global_path
                } else {
                    std::path::PathBuf::from(".serial-cli.toml")
                }
            });

            match config_manager.save(Some(&output_path)) {
                Ok(_) => {
                    println!("\u{2713} Configuration saved successfully");
                }
                Err(e) => {
                    eprintln!("\u{2717} Failed to save configuration: {}", e);
                    return Err(e);
                }
            }
        }
        ConfigCommand::Reset => {
            match config_manager.reset() {
                Ok(_) => {
                    println!("\u{2713} Configuration reset to defaults");
                    println!("Note: Use 'config save' to persist changes");
                }
                Err(e) => {
                    eprintln!("\u{2717} Failed to reset configuration: {}", e);
                    return Err(e);
                }
            }
        }
    }
    Ok(())
}
