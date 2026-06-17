//! Interactive shell
//!
//! This module provides an interactive REPL shell for serial communication
//! with line editing, command history, and tab completion via rustyline.

use crate::error::Result;
use crate::serial_core::{PortManager, SerialConfig};
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use rustyline::{Editor, Helper, Result as RlResult};
use std::borrow::Cow::{self, Owned};

/// Command names available in the interactive shell.
const COMMANDS: &[&str] = &[
    "help", "list", "open", "close", "send", "recv", "status", "script", "dtr", "rts", "quit",
    "exit",
];

/// Sub-command completions for multi-word commands.
const SCRIPT_SUBCOMMANDS: &[&str] = &["list", "set", "clear", "show"];
const SIGNAL_VALUES: &[&str] = &["on", "off"];
const SEND_FLAGS: &[&str] = &["--hex", "--base64"];

/// Rustyline helper providing tab completion for serial-cli commands.
struct SerialCompleter;

impl Helper for SerialCompleter {}

impl Completer for SerialCompleter {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> RlResult<(usize, Vec<String>)> {
        let input = &line[..pos];
        let parts: Vec<&str> = input.split_whitespace().collect();

        // Determine if cursor is in the middle of a word
        let trailing_space = input.ends_with(' ');

        if parts.is_empty() || (parts.len() == 1 && !trailing_space) {
            // Complete first command
            let partial = parts.first().copied().unwrap_or("");
            let matches: Vec<String> = COMMANDS
                .iter()
                .filter(|cmd| cmd.starts_with(partial))
                .map(|s| s.to_string())
                .collect();
            return Ok((0, matches));
        }

        // Multi-word: context-aware completion
        let cmd = parts[0];
        let start = input.len() - input.trim_end().len();

        match cmd {
            "script" => {
                if parts.len() == 1 || (parts.len() == 2 && !trailing_space) {
                    let partial = parts.get(1).copied().unwrap_or("");
                    let matches: Vec<String> = SCRIPT_SUBCOMMANDS
                        .iter()
                        .filter(|s| s.starts_with(partial))
                        .map(|s| s.to_string())
                        .collect();
                    return Ok((start, matches));
                }
                if parts[1] == "set" && (trailing_space || parts.len() == 3) {
                    let scripts = ["at_command", "line", "modbus_ascii", "modbus_rtu"];
                    let partial = if trailing_space {
                        ""
                    } else {
                        parts.get(2).copied().unwrap_or("")
                    };
                    let matches: Vec<String> = scripts
                        .iter()
                        .filter(|s| s.starts_with(partial))
                        .map(|s| s.to_string())
                        .collect();
                    return Ok((start, matches));
                }
            }
            "send" => {
                let partial = if trailing_space {
                    ""
                } else {
                    parts.last().copied().unwrap_or("")
                };
                let matches: Vec<String> = SEND_FLAGS
                    .iter()
                    .filter(|f| f.starts_with(partial))
                    .map(|s| s.to_string())
                    .collect();
                if !matches.is_empty() {
                    return Ok((start, matches));
                }
            }
            "dtr" | "rts" => {
                if parts.len() == 1 || (parts.len() == 2 && !trailing_space) {
                    let partial = parts.get(1).copied().unwrap_or("");
                    let matches: Vec<String> = SIGNAL_VALUES
                        .iter()
                        .filter(|s| s.starts_with(partial))
                        .map(|s| s.to_string())
                        .collect();
                    return Ok((start, matches));
                }
            }
            _ => {}
        }

        Ok((pos, vec![]))
    }
}

impl Hinter for SerialCompleter {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for SerialCompleter {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        Owned(format!("\x1b[1;36m{}\x1b[0m", prompt))
    }
}

impl Validator for SerialCompleter {}

/// Interactive shell backed by rustyline.
pub struct InteractiveShell {
    running: bool,
    manager: PortManager,
    script_manager: crate::script::ScriptManager,
    current_port_id: Option<String>,
    editor: Editor<SerialCompleter, DefaultHistory>,
}

impl InteractiveShell {
    /// Create a new interactive shell.
    pub fn new() -> Self {
        let mut editor = Editor::new().expect("Failed to create line editor");

        // Load history from previous sessions
        if let Some(dir) = Self::history_dir() {
            let history_path = dir.join("serial_cli_history.txt");
            if editor.load_history(&history_path).is_err() {
                // No previous history - this is normal for first run
            }
        }

        Self {
            running: false,
            manager: PortManager::new(),
            script_manager: crate::script::ScriptManager::new(),
            current_port_id: None,
            editor,
        }
    }

    /// Set the current port ID.
    pub fn set_current_port(&mut self, port_id: String) {
        self.current_port_id = Some(port_id);
    }

    /// Run the interactive shell REPL.
    pub async fn run(&mut self) -> Result<()> {
        self.running = true;
        println!("Serial CLI Interactive Shell");
        println!("Type 'help' for available commands, 'quit' to exit");
        println!();

        while self.running {
            let readline = self.editor.readline("serial> ");
            match readline {
                Ok(line) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let _ = self.editor.add_history_entry(trimmed);
                    if let Err(e) = self.execute_command(trimmed).await {
                        eprintln!("Error: {}", e);
                    }
                }
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    // Ctrl-C: just redisplay the prompt
                    println!();
                }
                Err(rustyline::error::ReadlineError::Eof) => {
                    // Ctrl-D: exit
                    println!("Goodbye!");
                    self.running = false;
                }
                Err(e) => {
                    eprintln!("Input error: {}", e);
                    self.running = false;
                }
            }
        }

        // Save history for next session
        self.save_history();

        Ok(())
    }

    /// Execute a single command line.
    pub async fn execute_command(&mut self, line: &str) -> Result<()> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        match parts[0] {
            "help" => self.cmd_help(),
            "list" => self.cmd_list().await?,
            "open" => self.cmd_open(&parts[1..]).await?,
            "close" => self.cmd_close(&parts[1..]).await?,
            "send" => self.cmd_send(&parts[1..]).await?,
            "recv" => self.cmd_recv(&parts[1..]).await?,
            "status" => self.cmd_status().await?,
            "script" => self.cmd_script(&parts[1..]).await?,
            "dtr" => self.cmd_dtr(&parts[1..]).await?,
            "rts" => self.cmd_rts(&parts[1..]).await?,
            "quit" | "exit" => {
                println!("Goodbye!");
                self.running = false;
            }
            _ => println!(
                "Unknown command: '{}'. Type 'help' for available commands.",
                parts[0]
            ),
        }

        Ok(())
    }

    fn save_history(&mut self) {
        if let Some(dir) = Self::history_dir() {
            let _ = std::fs::create_dir_all(&dir);
            let history_path = dir.join("serial_cli_history.txt");
            if let Err(e) = self.editor.save_history(&history_path) {
                eprintln!("Warning: Failed to save history: {}", e);
            }
        }
    }

    fn history_dir() -> Option<std::path::PathBuf> {
        directories::ProjectDirs::from("", "", "serial-cli")
            .map(|dirs| dirs.data_dir().to_path_buf())
    }

    // ── Command handlers ───────────────────────────────────────────

    /// Help command
    fn cmd_help(&self) {
        println!("Available commands:");
        println!("  help              - Show this help message");
        println!("  list              - List available serial ports");
        println!("  open <port>       - Open a serial port");
        println!("  close [port_id]   - Close a serial port (closes current if no ID given)");
        println!("  send [--hex|--base64] <data> - Send data to the current port");
        println!("  recv [n]          - Receive data from the current port (default: 64 bytes)");
        println!("  status            - Show port status");
        println!();
        println!("Script commands:");
        println!("  script            - Show current script and available scripts");
        println!("  script list       - List all available scripts");
        println!("  script set <name> - Set script for current port");
        println!("  script clear      - Clear script from current port");
        println!("  script show       - Show script status");
        println!();
        println!("Hardware control commands:");
        println!("  dtr [on|off]      - Get or set DTR signal state");
        println!("  rts [on|off]      - Get or set RTS signal state");
        println!();
        println!("  quit/exit         - Exit the shell");
        println!();
        println!("Keyboard shortcuts:");
        println!("  Tab               - Auto-complete commands and arguments");
        println!("  Up/Down           - Navigate command history");
        println!("  Ctrl+C            - Cancel current input");
        println!("  Ctrl+D            - Exit the shell");
    }

    /// List ports command
    async fn cmd_list(&self) -> Result<()> {
        let ports = self.manager.list_ports()?;

        if ports.is_empty() {
            println!("No serial ports found.");
        } else {
            println!("Available serial ports:");
            for port in ports {
                println!("  - {} ({})", port.port_name, port.port_type);
            }
        }

        Ok(())
    }

    /// Open port command
    async fn cmd_open(&mut self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            println!("Usage: open <port>");
            return Ok(());
        }

        let port_name = args[0];

        // Close current port if open
        if let Some(ref port_id) = self.current_port_id {
            println!("Closing current port...");
            self.manager.close_port(port_id).await?;
            self.current_port_id = None;
        }

        // Use default configuration
        let config = SerialConfig::default();

        // Open the port
        match self.manager.open_port(port_name, config).await {
            Ok(port_id) => {
                println!("Port opened successfully");
                println!("Port ID: {}", port_id);
                self.current_port_id = Some(port_id);
            }
            Err(e) => {
                eprintln!("Failed to open port: {}", e);
            }
        }

        Ok(())
    }

    /// Close port command
    async fn cmd_close(&mut self, args: &[&str]) -> Result<()> {
        let port_id = if args.is_empty() {
            // Use current port
            if let Some(ref id) = self.current_port_id {
                id.clone()
            } else {
                println!("No port is currently open");
                println!("Usage: close <port_id>");
                return Ok(());
            }
        } else {
            // Use specified port
            args[0].to_string()
        };

        match self.manager.close_port(&port_id).await {
            Ok(_) => {
                println!("Port closed successfully");
                if self.current_port_id.as_ref() == Some(&port_id) {
                    self.current_port_id = None;
                }
            }
            Err(e) => {
                eprintln!("Failed to close port: {}", e);
            }
        }

        Ok(())
    }

    /// Send command
    async fn cmd_send(&mut self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            println!("Usage: send [--hex|--base64] <data>");
            println!("  send hello          - Send plain text");
            println!("  send --hex AABBCC   - Send hex-encoded bytes");
            println!("  send --hex 0xAABBCC - Send hex-encoded bytes (0x prefix)");
            println!("  send --base64 SGVsbG8= - Send base64-encoded bytes");
            return Ok(());
        }

        if self.current_port_id.is_none() {
            println!("No port is currently open");
            println!("Use 'open <port>' first");
            return Ok(());
        }

        // Parse flags
        let (hex_mode, base64_mode, data_args): (bool, bool, Vec<&str>) = {
            let mut h = false;
            let mut b = false;
            let mut rest = Vec::new();
            for arg in args {
                match *arg {
                    "--hex" | "-x" => h = true,
                    "--base64" | "-b" => b = true,
                    other => rest.push(other),
                }
            }
            (h, b, rest)
        };

        if hex_mode && base64_mode {
            eprintln!("Error: --hex and --base64 are mutually exclusive");
            return Ok(());
        }

        let data_str = data_args.join(" ");
        let bytes: Vec<u8> = if hex_mode {
            crate::cli::commands::parsers::parse_hex_string(&data_str)?
        } else if base64_mode {
            crate::cli::commands::parsers::base64_decode(&data_str)?
        } else {
            data_str.as_bytes().to_vec()
        };

        let port_id = self.current_port_id.as_ref().unwrap();

        // Get the port handle
        let port_handle = self.manager.get_port(port_id).await?;
        let mut handle = port_handle.lock().await;

        // Send data
        let n = handle.write(&bytes)?;
        if hex_mode {
            println!("Sent {} bytes (hex)", n);
        } else if base64_mode {
            println!("Sent {} bytes (base64)", n);
        } else {
            println!("Sent {} bytes", n);
        }

        Ok(())
    }

    /// Receive command
    async fn cmd_recv(&mut self, args: &[&str]) -> Result<()> {
        let n: usize = if args.is_empty() {
            64
        } else {
            args[0].parse().unwrap_or(64)
        };

        if self.current_port_id.is_none() {
            println!("No port is currently open");
            println!("Use 'open <port>' first");
            return Ok(());
        }

        println!("Reading up to {} bytes...", n);

        let port_id = self.current_port_id.as_ref().unwrap();

        // Get the port handle
        let port_handle = self.manager.get_port(port_id).await?;
        let mut handle = port_handle.lock().await;

        // Read data
        let mut buffer = vec![0u8; n];
        match handle.read(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read > 0 {
                    buffer.truncate(bytes_read);

                    // Try to display as string
                    if let Ok(text) = String::from_utf8(buffer.clone()) {
                        println!("Received ({} bytes as text): {}", bytes_read, text);
                    } else {
                        // Display as hex
                        let hex: String = buffer.iter().map(|b| format!("{:02x} ", b)).collect();
                        println!("Received ({} bytes as hex): {}", bytes_read, hex);
                    }
                } else {
                    println!("No data available");
                }
            }
            Err(e) => {
                eprintln!("Failed to read data: {}", e);
            }
        }

        Ok(())
    }

    /// Status command
    async fn cmd_status(&self) -> Result<()> {
        if let Some(ref port_id) = self.current_port_id {
            println!("Current port ID: {}", port_id);

            // Try to get port info
            match self.manager.get_port(port_id).await {
                Ok(port_handle) => {
                    let handle = port_handle.lock().await;
                    println!("Port name: {}", handle.name());
                    println!("Configuration:");
                    println!("  Baud rate: {}", handle.config().baudrate);
                    println!("  Data bits: {}", handle.config().databits);
                    println!("  Stop bits: {}", handle.config().stopbits);
                    println!("  Parity: {:?}", handle.config().parity);
                    println!("  Flow control: {:?}", handle.config().flow_control);

                    // Show script information
                    if handle.has_script() {
                        println!("  Script: attached");
                    } else {
                        println!("  Script: (none - raw mode)");
                    }
                }
                Err(_) => {
                    println!("Port handle not available");
                }
            }
        } else {
            println!("No port is currently open");
            println!("Use 'open <port>' to open a port");
        }

        Ok(())
    }

    /// Script command
    async fn cmd_script(&mut self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            // Show current script and available scripts
            self.show_script_status().await?;
        } else {
            match args[0] {
                "list" | "ls" => {
                    // List all available scripts
                    self.list_scripts().await?;
                }
                "set" => {
                    // Set script for current port
                    if args.len() < 2 {
                        println!("Usage: script set <script_name>");
                        println!("Available scripts:");
                        self.list_scripts().await?;
                    } else {
                        self.set_port_script(args[1]).await?;
                    }
                }
                "show" | "status" => {
                    // Show current script details
                    self.show_script_status().await?;
                }
                "clear" | "none" => {
                    // Clear script from current port
                    self.clear_port_script().await?;
                }
                _ => {
                    // Try to set script directly (shorthand)
                    self.set_port_script(args[0]).await?;
                }
            }
        }

        Ok(())
    }

    /// Show script status for current port
    async fn show_script_status(&self) -> Result<()> {
        if let Some(ref port_id) = self.current_port_id {
            match self.manager.has_script(port_id).await {
                Ok(true) => {
                    println!("Current script: attached");
                    println!();
                    println!("Script commands:");
                    println!("  script list          - List all available scripts");
                    println!("  script set <name>    - Set script for current port");
                    println!("  script clear         - Clear script from current port");
                    println!("  script show          - Show script status");
                }
                Ok(false) => {
                    println!("Current script: (none)");
                    println!();
                    println!("Available scripts:");
                    self.list_scripts().await?;
                    println!();
                    println!("Use 'script set <name>' to attach a script to this port");
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        } else {
            println!("No port is currently open");
            println!("Use 'open <port>' first");
        }

        Ok(())
    }

    /// List all available scripts
    async fn list_scripts(&self) -> Result<()> {
        let scripts = self.script_manager.list();
        println!("Available scripts:");
        for s in &scripts {
            let tag = if s.built_in { " (built-in)" } else { "" };
            println!("  - {:15} - {}{}", s.name, s.description, tag);
        }
        Ok(())
    }

    /// Set script for current port
    async fn set_port_script(&mut self, script_name: &str) -> Result<()> {
        if self.current_port_id.is_none() {
            println!("No port is currently open");
            println!("Use 'open <port>' first");
            return Ok(());
        }

        if !self.script_manager.has(script_name) {
            println!("Unknown script: {}", script_name);
            println!();
            println!("Available scripts:");
            self.list_scripts().await?;
            return Ok(());
        }

        let port_id = self.current_port_id.as_ref().unwrap();
        match self.manager.attach_script_by_name(port_id, &self.script_manager, script_name).await {
            Ok(_) => {
                println!("Script '{}' set for port", script_name);
            }
            Err(e) => {
                println!("Failed to set script: {}", e);
            }
        }

        Ok(())
    }

    /// Clear script from current port
    async fn clear_port_script(&mut self) -> Result<()> {
        if self.current_port_id.is_none() {
            println!("No port is currently open");
            println!("Use 'open <port>' first");
            return Ok(());
        }

        let port_id = self.current_port_id.as_ref().unwrap();
        match self.manager.detach_script(port_id).await {
            Ok(_) => {
                println!("Script cleared from port");
            }
            Err(e) => {
                println!("Failed to clear script: {}", e);
            }
        }

        Ok(())
    }

    /// DTR command
    async fn cmd_dtr(&mut self, args: &[&str]) -> Result<()> {
        if self.current_port_id.is_none() {
            println!("No port is currently open");
            println!("Use 'open <port>' first");
            return Ok(());
        }

        if args.is_empty() {
            // Show current DTR state
            let port_id = self.current_port_id.as_ref().unwrap();
            match self.manager.get_dtr(port_id).await {
                Ok(state) => println!("DTR signal: {}", if state { "ON" } else { "OFF" }),
                Err(e) => println!("Error getting DTR state: {}", e),
            }
            println!();
            println!("Usage: dtr on|off");
            return Ok(());
        }

        let enable = match args[0].to_lowercase().as_str() {
            "on" | "true" | "1" | "enable" => true,
            "off" | "false" | "0" | "disable" => false,
            _ => {
                println!("Invalid argument: {}", args[0]);
                println!("Usage: dtr on|off");
                return Ok(());
            }
        };

        let port_id = self.current_port_id.as_ref().unwrap();
        match self.manager.set_dtr(port_id, enable).await {
            Ok(_) => {
                println!("DTR signal set to: {}", if enable { "ON" } else { "OFF" });
                println!("Note: Full platform-specific DTR control implementation pending");
            }
            Err(e) => {
                println!("Failed to set DTR: {}", e);
            }
        }

        Ok(())
    }

    /// RTS command
    async fn cmd_rts(&mut self, args: &[&str]) -> Result<()> {
        if self.current_port_id.is_none() {
            println!("No port is currently open");
            println!("Use 'open <port>' first");
            return Ok(());
        }

        if args.is_empty() {
            // Show current RTS state
            let port_id = self.current_port_id.as_ref().unwrap();
            match self.manager.get_rts(port_id).await {
                Ok(state) => println!("RTS signal: {}", if state { "ON" } else { "OFF" }),
                Err(e) => println!("Error getting RTS state: {}", e),
            }
            println!();
            println!("Usage: rts on|off");
            return Ok(());
        }

        let enable = match args[0].to_lowercase().as_str() {
            "on" | "true" | "1" | "enable" => true,
            "off" | "false" | "0" | "disable" => false,
            _ => {
                println!("Invalid argument: {}", args[0]);
                println!("Usage: rts on|off");
                return Ok(());
            }
        };

        let port_id = self.current_port_id.as_ref().unwrap();
        match self.manager.set_rts(port_id, enable).await {
            Ok(_) => {
                println!("RTS signal set to: {}", if enable { "ON" } else { "OFF" });
                println!("Note: Full platform-specific RTS control implementation pending");
            }
            Err(e) => {
                println!("Failed to set RTS: {}", e);
            }
        }

        Ok(())
    }
}

impl Default for InteractiveShell {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustyline::history::DefaultHistory;

    fn test_ctx() -> rustyline::Context<'static> {
        static HISTORY: std::sync::OnceLock<DefaultHistory> = std::sync::OnceLock::new();
        let history = HISTORY.get_or_init(DefaultHistory::new);
        // SAFETY: the returned Context borrows `history` which lives for 'static
        unsafe { std::mem::transmute(rustyline::Context::new(history)) }
    }

    #[test]
    fn test_shell_creation() {
        let shell = InteractiveShell::new();
        assert!(!shell.running);
        assert!(shell.current_port_id.is_none());
    }

    #[test]
    fn test_completer_top_level_commands() {
        let completer = SerialCompleter;
        let (pos, matches) = completer.complete("se", 2, &test_ctx()).unwrap();
        assert_eq!(pos, 0);
        assert!(matches.contains(&"send".to_string()));
    }

    #[test]
    fn test_completer_script_subcommands() {
        let completer = SerialCompleter;
        let (_pos, matches) = completer.complete("script se", 9, &test_ctx()).unwrap();
        assert!(matches.contains(&"set".to_string()));
    }

    #[test]
    fn test_completer_signal_values() {
        let completer = SerialCompleter;
        let (_, matches) = completer.complete("dtr o", 5, &test_ctx()).unwrap();
        assert!(matches.contains(&"on".to_string()));
        assert!(matches.contains(&"off".to_string()));
    }

    #[test]
    fn test_completer_send_flags() {
        let completer = SerialCompleter;
        let (_, matches) = completer.complete("send --h", 8, &test_ctx()).unwrap();
        assert!(matches.contains(&"--hex".to_string()));
    }

    #[test]
    fn test_completer_script_set_name() {
        let completer = SerialCompleter;
        let (_, matches) = completer
            .complete("script set mod", 14, &test_ctx())
            .unwrap();
        assert!(matches.contains(&"modbus_rtu".to_string()));
        assert!(matches.contains(&"modbus_ascii".to_string()));
    }
}
