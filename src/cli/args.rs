//! CLI argument definitions
//!
//! Top-level CLI parser ([`Cli`]) and command routing ([`Commands`]).
//! All subcommand-specific types live in [`super::types`].

use clap::{Parser, Subcommand};

use super::types::{
    ConfigCommand, PortCommand, ScriptCommand, ServerCommand, SniffCommand, VirtualCommand,
};

/// Top-level CLI arguments for the serial-cli application.
///
/// Provides global flags (`--json`, `--verbose`) and a required subcommand.
/// When no subcommand is specified, the application defaults to interactive shell mode.
#[derive(Parser)]
#[command(name = "serial-cli")]
#[command(about = "A universal serial port CLI tool optimized for AI interaction", long_about = None)]
#[command(version)]
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

    /// Script management (list, load, unload, validate scripts).
    Script {
        #[command(subcommand)]
        script_command: ScriptCommand,
    },

    /// Sniff and monitor serial port traffic.
    Sniff {
        #[command(subcommand)]
        sniff_command: SniffCommand,
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    fn test_cli_port_list() {
        let cli = Cli::try_parse_from(["serial-cli", "port", "list"]).unwrap();
        assert!(!cli.json);
        assert!(!cli.verbose);
        match cli.command.unwrap() {
            Commands::Port { port_command } => {
                assert!(matches!(port_command, PortCommand::List));
            }
            _ => panic!("Expected Port command"),
        }
    }

    #[test]
    fn test_cli_port_send_basic() {
        let cli = Cli::try_parse_from([
            "serial-cli",
            "port",
            "send",
            "--port",
            "/dev/ttyUSB0",
            "Hello",
        ])
        .unwrap();
        match cli.command.unwrap() {
            Commands::Port { port_command } => match port_command {
                PortCommand::Send {
                    port,
                    hex,
                    base64,
                    data,
                } => {
                    assert_eq!(port, "/dev/ttyUSB0");
                    assert!(!hex);
                    assert!(!base64);
                    assert_eq!(data, "Hello");
                }
                _ => panic!("Expected Send"),
            },
            _ => panic!("Expected Port command"),
        }
    }

    #[test]
    fn test_cli_port_send_hex() {
        let cli = Cli::try_parse_from([
            "serial-cli",
            "port",
            "send",
            "--port",
            "/dev/ttyUSB0",
            "--hex",
            "AABBCC",
        ])
        .unwrap();
        match cli.command.unwrap() {
            Commands::Port { port_command } => match port_command {
                PortCommand::Send { hex, data, .. } => {
                    assert!(hex);
                    assert_eq!(data, "AABBCC");
                }
                _ => panic!("Expected Send"),
            },
            _ => panic!("Expected Port command"),
        }
    }

    #[test]
    fn test_cli_interactive() {
        let cli = Cli::try_parse_from(["serial-cli", "interactive"]).unwrap();
        assert!(matches!(cli.command.unwrap(), Commands::Interactive));
    }

    #[test]
    fn test_cli_run_script() {
        let cli = Cli::try_parse_from(["serial-cli", "run", "test.lua", "arg1", "arg2"]).unwrap();
        match cli.command.unwrap() {
            Commands::Run { script, args } => {
                assert_eq!(script, "test.lua");
                assert_eq!(args, vec!["arg1".to_string(), "arg2".to_string()]);
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_cli_run_script_no_args() {
        let cli = Cli::try_parse_from(["serial-cli", "run", "test.lua"]).unwrap();
        match cli.command.unwrap() {
            Commands::Run { script, args } => {
                assert_eq!(script, "test.lua");
                assert!(args.is_empty());
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_cli_script_list() {
        let cli = Cli::try_parse_from(["serial-cli", "script", "list"]).unwrap();
        match cli.command.unwrap() {
            Commands::Script { script_command } => {
                assert!(matches!(
                    script_command,
                    ScriptCommand::List { detailed: false }
                ));
            }
            _ => panic!("Expected Script command"),
        }
    }

    #[test]
    fn test_cli_script_list_detailed() {
        let cli = Cli::try_parse_from(["serial-cli", "script", "list", "--detailed"]).unwrap();
        match cli.command.unwrap() {
            Commands::Script { script_command } => match script_command {
                ScriptCommand::List { detailed } => assert!(detailed),
                _ => panic!("Expected List"),
            },
            _ => panic!("Expected Script command"),
        }
    }

    #[test]
    fn test_cli_script_load() {
        let cli = Cli::try_parse_from(["serial-cli", "script", "load", "my_script.lua"]).unwrap();
        match cli.command.unwrap() {
            Commands::Script { script_command } => match script_command {
                ScriptCommand::Load { path, name } => {
                    assert_eq!(path, PathBuf::from("my_script.lua"));
                    assert!(name.is_none());
                }
                _ => panic!("Expected Load"),
            },
            _ => panic!("Expected Script command"),
        }
    }

    #[test]
    fn test_cli_config_show() {
        let cli = Cli::try_parse_from(["serial-cli", "config", "show"]).unwrap();
        match cli.command.unwrap() {
            Commands::Config { config_command } => {
                assert!(matches!(
                    config_command,
                    ConfigCommand::Show { json: false }
                ));
            }
            _ => panic!("Expected Config command"),
        }
    }

    #[test]
    fn test_cli_config_set() {
        let cli = Cli::try_parse_from(["serial-cli", "config", "set", "serial.baudrate", "9600"])
            .unwrap();
        match cli.command.unwrap() {
            Commands::Config { config_command } => match config_command {
                ConfigCommand::Set { key, value } => {
                    assert_eq!(key, "serial.baudrate");
                    assert_eq!(value, "9600");
                }
                _ => panic!("Expected Set"),
            },
            _ => panic!("Expected Config command"),
        }
    }

    #[test]
    fn test_cli_server_start_defaults() {
        let cli = Cli::try_parse_from(["serial-cli", "server", "start"]).unwrap();
        match cli.command.unwrap() {
            Commands::Server { server_command } => match server_command {
                ServerCommand::Start {
                    socket_path,
                    port,
                    bind,
                    no_tcp,
                    log,
                    max_connections,
                    ..
                } => {
                    assert!(socket_path.is_none());
                    assert!(port.is_none());
                    assert!(bind.is_none());
                    assert!(!no_tcp);
                    assert!(log.is_none());
                    assert_eq!(max_connections, 10);
                }
                _ => panic!("Expected Start"),
            },
            _ => panic!("Expected Server command"),
        }
    }

    #[test]
    fn test_cli_server_start_custom() {
        let cli = Cli::try_parse_from([
            "serial-cli",
            "server",
            "start",
            "--socket-path",
            "/tmp/custom.sock",
            "--max-connections",
            "5",
        ])
        .unwrap();
        match cli.command.unwrap() {
            Commands::Server { server_command } => match server_command {
                ServerCommand::Start {
                    socket_path,
                    max_connections,
                    ..
                } => {
                    assert_eq!(socket_path.unwrap(), "/tmp/custom.sock");
                    assert_eq!(max_connections, 5);
                }
                _ => panic!("Expected Start"),
            },
            _ => panic!("Expected Server command"),
        }
    }

    #[test]
    fn test_cli_server_call() {
        let cli = Cli::try_parse_from([
            "serial-cli",
            "server",
            "call",
            "port_open",
            "{\"port\": \"/dev/ttyUSB0\"}",
        ])
        .unwrap();
        match cli.command.unwrap() {
            Commands::Server { server_command } => match server_command {
                ServerCommand::Call {
                    method,
                    args,
                    stdin,
                    remote,
                    ..
                } => {
                    assert_eq!(method, "port_open");
                    assert!(args.contains("ttyUSB0"));
                    assert!(!stdin);
                    assert!(remote.is_none());
                }
                _ => panic!("Expected Call"),
            },
            _ => panic!("Expected Server command"),
        }
    }

    #[test]
    fn test_cli_virtual_create() {
        let cli =
            Cli::try_parse_from(["serial-cli", "virtual", "create", "--backend", "pty"]).unwrap();
        match cli.command.unwrap() {
            Commands::Virtual { virtual_command } => match virtual_command {
                VirtualCommand::Create {
                    backend, monitor, ..
                } => {
                    assert_eq!(backend, "pty");
                    assert!(!monitor);
                }
                _ => panic!("Expected Create"),
            },
            _ => panic!("Expected Virtual command"),
        }
    }

    #[test]
    fn test_cli_global_flags() {
        let cli =
            Cli::try_parse_from(["serial-cli", "--json", "--verbose", "port", "list"]).unwrap();
        assert!(cli.json);
        assert!(cli.verbose);
    }

    #[test]
    fn test_cli_no_command_defaults_to_interactive() {
        let cli = Cli::try_parse_from(["serial-cli"]).unwrap();
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_cli_sniff_start() {
        let cli = Cli::try_parse_from([
            "serial-cli",
            "sniff",
            "start",
            "--port",
            "/dev/ttyUSB0",
            "--max-packets",
            "100",
        ])
        .unwrap();
        match cli.command.unwrap() {
            Commands::Sniff { sniff_command } => match sniff_command {
                SniffCommand::Start {
                    port, max_packets, ..
                } => {
                    assert_eq!(port, "/dev/ttyUSB0");
                    assert_eq!(max_packets, 100);
                }
                _ => panic!("Expected Start"),
            },
            _ => panic!("Expected Sniff command"),
        }
    }

    #[test]
    fn test_cli_port_send_base64() {
        let cli = Cli::try_parse_from([
            "serial-cli",
            "port",
            "send",
            "--port",
            "/dev/ttyUSB0",
            "--base64",
            "SGVsbG8=",
        ])
        .unwrap();
        match cli.command.unwrap() {
            Commands::Port { port_command } => match port_command {
                PortCommand::Send { base64, data, .. } => {
                    assert!(base64);
                    assert_eq!(data, "SGVsbG8=");
                }
                _ => panic!("Expected Send"),
            },
            _ => panic!("Expected Port command"),
        }
    }
}
