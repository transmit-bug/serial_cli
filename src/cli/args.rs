//! CLI argument definitions
//!
//! Top-level CLI parser ([`Cli`]) and command routing ([`Commands`]).
//! All subcommand-specific types live in [`super::types`].

use clap::{Parser, Subcommand};

use super::types::{
    BatchCommand, ConfigCommand, PortCommand, ProtocolCommand, ServerCommand, SniffCommand,
    VirtualCommand,
};

/// Top-level CLI arguments for the serial-cli application.
///
/// Provides global flags (`--json`, `--verbose`) and a required subcommand.
/// When no subcommand is specified, the application defaults to interactive shell mode.
#[derive(Parser)]
#[command(name = "serial-cli")]
#[command(about = "A universal serial port CLI tool optimized for AI interaction", long_about = None)]
pub struct Cli {
    /// Enable JSON output for all commands.
    ///
    /// When set, command results are printed as formatted JSON
    /// instead of human-readable text.
    #[arg(long, global = true)]
    pub json: bool,

    /// Enable verbose logging output (maps to `DEBUG` level).
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// The subcommand to execute. Defaults to [`Commands::Interactive`] if `None`.
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// All available subcommands for the serial-cli application.
///
/// Each variant maps to a distinct CLI action. Most subcommands delegate to
/// a handler function in `src/cli/commands/`.
#[derive(Subcommand)]
pub enum Commands {
    /// Serial port management (list, send).
    Port {
        #[command(subcommand)]
        port_command: PortCommand,
    },

    /// Start an interactive REPL shell for serial communication.
    Interactive,

    /// Execute a Lua script with optional arguments.
    Run {
        /// Path to the `.lua` script file.
        script: String,

        /// Arguments passed to the Lua script.
        #[arg(value_name = "ARGS", trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Protocol management (list, load, unload, validate protocols).
    Protocol {
        #[command(subcommand)]
        protocol_command: ProtocolCommand,
    },

    /// Sniff and monitor serial port traffic.
    Sniff {
        #[command(subcommand)]
        sniff_command: SniffCommand,
    },

    /// Batch execution of scripts or batch files.
    Batch {
        #[command(subcommand)]
        batch_command: BatchCommand,
    },

    /// Configuration management (show, set, save, reset).
    Config {
        #[command(subcommand)]
        config_command: ConfigCommand,
    },

    /// Virtual serial port management (create, list, stop pairs).
    Virtual {
        #[command(subcommand)]
        virtual_command: VirtualCommand,
    },

    /// Server mode (daemon for AI/automation workflows).
    Server {
        #[command(subcommand)]
        server_command: ServerCommand,
    },

    /// (Internal) Background sniff daemon — not for direct user invocation.
    #[command(hide = true, name = "__sniff_daemon__")]
    SniffDaemon {
        #[arg(long)]
        port: String,

        #[arg(long)]
        output: Option<String>,

        #[arg(long, default_value = "0")]
        max_packets: usize,

        #[arg(long, default_value = "false")]
        hex: bool,
    },
}
