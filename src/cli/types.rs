//! CLI command type definitions
//!
//! This module contains all command and subcommand enum definitions
//! used by clap for argument parsing.

use std::path::PathBuf;

/// Serial port management subcommands
#[derive(clap::Subcommand)]
pub enum PortCommand {
    /// List available serial ports on the system
    List,

    /// Send raw data to a serial port and optionally read the response
    Send {
        /// Port name (e.g., `COM1`, `/dev/ttyUSB0`)
        #[arg(short, long)]
        port: String,

        /// Interpret data as hex-encoded bytes (e.g., `AABBCC` or `0xAABBCC`)
        #[arg(long)]
        hex: bool,

        /// Interpret data as base64-encoded bytes
        #[arg(long)]
        base64: bool,

        /// Data to send (plain text by default; use --hex or --base64 for binary)
        data: String,
    },
}

/// Virtual serial port subcommands
#[derive(clap::Subcommand)]
pub enum VirtualCommand {
    /// Create a virtual serial port pair
    Create {
        /// Backend type (auto/pty/socat/namedpipe)
        ///
        /// auto: Automatically detect best backend for platform (default)
        /// pty: POSIX pseudo-terminal (Unix/macOS only)
        /// socat: Socat-based virtual ports (requires socat binary)
        /// namedpipe: Windows named pipes (Windows only)
        #[arg(long, default_value = "auto")]
        backend: String,

        /// Enable traffic monitoring
        #[arg(long)]
        monitor: bool,

        /// Output monitoring to file
        #[arg(long)]
        output: Option<PathBuf>,

        /// Maximum packets to capture (0 = unlimited)
        #[arg(long, default_value = "0")]
        max_packets: usize,
    },

    /// List active virtual port pairs
    List,

    /// Stop a virtual port pair
    Stop {
        /// Virtual port pair ID
        id: String,
    },

    /// Show statistics for a virtual port pair
    Stats {
        /// Virtual port pair ID
        id: String,
    },
}

/// Script management subcommands
#[derive(clap::Subcommand)]
pub enum ScriptCommand {
    /// List all available scripts
    List {
        /// Show verbose information including descriptions
        #[arg(long)]
        detailed: bool,
    },

    /// Show script information
    Info {
        /// Script name
        name: String,
    },

    /// Load a custom script from Lua file
    Load {
        /// Path to script file
        path: PathBuf,

        /// Custom script name (default: filename without extension)
        #[arg(long)]
        name: Option<String>,
    },

    /// Unload a custom script
    Unload {
        /// Script name
        name: String,
    },

    /// Reload a custom script from disk
    Reload {
        /// Script name
        name: String,
    },

    /// Validate a script without loading
    Validate {
        /// Path to script file
        path: PathBuf,
    },

    /// Hot-reload management
    HotReload {
        /// Action: enable, disable, or status
        action: String,
    },
}

/// Sniff subcommands
#[derive(clap::Subcommand)]
pub enum SniffCommand {
    /// Start sniffing on a port
    Start {
        /// Port name
        #[arg(short, long)]
        port: String,

        /// Output file path (optional, auto-generated if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Maximum packets to capture (0 = unlimited)
        #[arg(short, long, default_value = "0")]
        max_packets: usize,

        /// Enable real-time display
        #[arg(long, default_value = "true")]
        display: bool,

        /// Display format (raw or hex)
        #[arg(long, default_value = "raw")]
        format: String,
    },

    /// Stop sniffing
    Stop,

    /// Show sniffing statistics
    Stats,

    /// Save captured packets to file
    Save {
        /// Output file path
        #[arg(short, long)]
        path: PathBuf,
    },
}

/// Config subcommands
#[derive(clap::Subcommand)]
pub enum ConfigCommand {
    /// Show configuration
    Show {
        /// Show as JSON
        #[arg(long)]
        json: bool,
    },

    /// Set a configuration value
    Set {
        /// Configuration key (e.g., serial.baudrate, logging.level)
        key: String,

        /// Configuration value
        value: String,
    },

    /// Save configuration to file
    Save {
        /// Output file path (optional, uses default if not specified)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Reset configuration to defaults
    Reset,
}

/// Server subcommands
#[derive(clap::Subcommand)]
pub enum ServerCommand {
    /// Start server daemon
    Start {
        /// Unix socket path (default: /tmp/serial-cli.sock)
        #[arg(long)]
        socket_path: Option<String>,

        /// TCP port for LAN remote access (default: 23333)
        #[arg(long)]
        port: Option<u16>,

        /// Bind address for the TCP listener (default: 0.0.0.0)
        #[arg(long)]
        bind: Option<String>,

        /// Disable the TCP listener (local access only)
        #[arg(long)]
        no_tcp: bool,

        /// Log file path
        #[arg(long)]
        log: Option<String>,

        /// Max concurrent connections
        #[arg(long, default_value = "10")]
        max_connections: usize,
    },

    /// Stop server daemon
    Stop,

    /// Server status
    Status,

    /// Call RPC method (for AI/automation)
    Call {
        /// RPC method name (e.g., port_open, port_send)
        method: String,

        /// JSON arguments (e.g., '{"port": "/dev/ttyUSB0"}')
        #[arg(value_name = "JSON_ARGS")]
        args: String,

        /// Use stdin for args (useful for piping)
        #[arg(long)]
        stdin: bool,

        /// Remote daemon address (e.g., 192.168.1.50:23333).
        /// When omitted, targets the local daemon session.
        #[arg(long)]
        remote: Option<String>,
    },

    /// (Internal) Run daemon in the foreground.
    /// Used by e2e tests and the Windows detached-spawn path — not for
    /// direct user invocation.
    #[command(hide = true, name = "daemon")]
    Daemon {
        /// Unix socket path
        #[arg(long)]
        socket_path: Option<String>,

        /// TCP port (off unless explicitly provided)
        #[arg(long)]
        port: Option<u16>,

        /// Bind address for the TCP listener (default: 0.0.0.0)
        #[arg(long)]
        bind: Option<String>,

        /// Log file path
        #[arg(long)]
        log: Option<String>,
    },
}
