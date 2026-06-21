//! Serial CLI — entry point
//!
//! This binary provides the `serial-cli` command-line interface for serial port
//! communication. It parses CLI arguments, initializes logging and configuration,
//! then dispatches to the appropriate command handler. When no subcommand is
//! provided, it falls back to interactive shell mode.
//!
//! # Startup sequence
//!
//! 1. Parse CLI arguments via [`clap`]
//! 2. Initialize logging (JSON or human-readable based on `--json` flag)
//! 3. Load and validate TOML configuration (falls back to defaults on failure)
//! 4. Dispatch to the matching command handler
//!
//! # Default behavior
//!
//! If no subcommand is given, the application enters interactive shell mode
//! (equivalent to `serial-cli interactive`).

use std::path::PathBuf;

use clap::Parser;

use serial_cli::cli::args::{Cli, Commands};
use serial_cli::cli::commands::{
    config as config_cmd, port as port_cmd, script as script_cmd, server as server_cmd,
    sniff as sniff_cmd, virtual_port,
};
use serial_cli::cli::interactive::InteractiveShell;
use serial_cli::cli::sniff_session;
use serial_cli::error::Result;
use serial_cli::script::ScriptManager;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Application entry point. Parses arguments, initializes subsystems, and
/// dispatches to the requested command handler.
///
/// # Exit codes
///
/// Returns `Ok(())` on success. Any error is propagated through [`Result`]
/// and printed by the runtime.
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging - supports RUST_LOG, LOG_FORMAT, LOG_FILE env vars
    if cli.json {
        serial_cli::logging::init_json(cli.verbose);
    } else {
        serial_cli::logging::init_cli(cli.verbose);
    }

    // Load configuration
    let config_manager = serial_cli::config::ConfigManager::load_with_fallback();
    let _config = config_manager.get();

    // Create ScriptManager
    let script_manager = Arc::new(Mutex::new(ScriptManager::new()));

    // Validate configuration
    if let Err(e) = config_manager.validate() {
        tracing::info!("Warning: Configuration validation failed: {}", e);
    }

    // Execute command
    let json_output = cli.json;
    match cli.command {
        Some(Commands::Port { port_command }) => {
            port_cmd::handle_port_command(port_command, json_output).await?;
        }
        Some(Commands::Interactive) => {
            let mut shell = InteractiveShell::new();
            shell.run().await?;
        }
        Some(Commands::Run { script, args }) => {
            script_cmd::run_lua_script(PathBuf::from(script), args, script_manager).await?;
        }
        Some(Commands::Script { script_command }) => {
            script_cmd::handle_script_command(script_command, json_output, script_manager).await?;
        }
        Some(Commands::Sniff { sniff_command }) => {
            sniff_cmd::handle_sniff_command(sniff_command, json_output).await?;
        }
        Some(Commands::Config { config_command }) => {
            config_cmd::handle_config_command(config_command, json_output)?;
        }
        Some(Commands::Virtual { virtual_command }) => {
            virtual_port::handle_virtual_command(virtual_command, json_output).await?;
        }
        Some(Commands::Server { server_command }) => {
            server_cmd::handle_server_command(server_command, json_output).await?;
        }
        Some(Commands::SniffDaemon {
            port,
            output,
            max_packets,
            hex,
        }) => {
            sniff_session::run_sniff_daemon(
                &port,
                output.as_deref().map(std::path::Path::new),
                max_packets,
                hex,
            )
            .await?;
        }
        None => {
            // No command specified, default to interactive mode
            let mut shell = InteractiveShell::new();
            shell.run().await?;
        }
    }

    Ok(())
}
