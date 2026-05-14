//! Interactive shell
//!
//! This module provides an interactive REPL shell for serial communication.

use crate::error::{Result, SerialError};
use crate::serial_core::{PortManager, SerialConfig};
use std::io::{self, Write};

/// Interactive shell
pub struct InteractiveShell {
    running: bool,
    manager: PortManager,
    current_port_id: Option<String>,
}

impl InteractiveShell {
    /// Create a new interactive shell
    pub fn new() -> Self {
        Self {
            running: false,
            manager: PortManager::new(),
            current_port_id: None,
        }
    }

    /// Set the current port ID
    pub fn set_current_port(&mut self, port_id: String) {
        self.current_port_id = Some(port_id);
    }

    /// Run the interactive shell
    pub async fn run(&mut self) -> Result<()> {
        self.running = true;
        tracing::info!("Starting interactive shell");
        println!("Serial CLI Interactive Shell");
        println!("Type 'help' for available commands, 'quit' to exit");
        println!();

        while self.running {
            tracing::trace!("serial> ");
            io::stdout().flush().map_err(SerialError::Io)?;

            let mut line = String::new();
            io::stdin().read_line(&mut line).map_err(SerialError::Io)?;

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Err(e) = self.execute_command(line).await {
                eprintln!("Error: {}", e);
            }
        }

        Ok(())
    }

    /// Execute a command
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
            "protocol" => self.cmd_protocol(&parts[1..]).await?,
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

    /// Help command
    fn cmd_help(&self) {
        println!("Available commands:");
        println!("  help              - Show this help message");
        println!("  list              - List available serial ports");
        println!("  open <port>       - Open a serial port");
        println!("  close [port_id]   - Close a serial port (closes current if no ID given)");
        println!("  send <data>       - Send data to the current port");
        println!("  recv [n]          - Receive data from the current port (default: 64 bytes)");
        println!("  status            - Show port status");
        println!();
        println!("Protocol commands:");
        println!("  protocol          - Show current protocol and available protocols");
        println!("  protocol list     - List all available protocols");
        println!("  protocol set <name>  - Set protocol for current port");
        println!("  protocol clear    - Clear protocol from current port");
        println!("  protocol show     - Show protocol status");
        println!();
        println!("Hardware control commands:");
        println!("  dtr [on|off]      - Get or set DTR signal state");
        println!("  rts [on|off]      - Get or set RTS signal state");
        println!();
        println!("  quit/exit         - Exit the shell");
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

        tracing::info!("Opening serial port {}", port_name);

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
            println!("Usage: send <data>");
            return Ok(());
        }

        if self.current_port_id.is_none() {
            println!("No port is currently open");
            println!("Use 'open <port>' first");
            return Ok(());
        }

        let data = args.join(" ");
        let port_id = self.current_port_id.as_ref().unwrap();

        tracing::info!("Sending data to port {}", port_id);

        // Get the port handle
        let port_handle = self.manager.get_port(port_id).await?;
        let mut handle = port_handle.lock().await;

        // Send data
        match handle.write(data.as_bytes()) {
            Ok(n) => {
                println!("Sent {} bytes", n);
            }
            Err(e) => {
                eprintln!("Failed to send data: {}", e);
            }
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

                    // Show protocol information
                    match handle.protocol_name() {
                        Some(protocol) => println!("  Protocol: {}", protocol),
                        None => println!("  Protocol: (none - raw mode)"),
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

    /// Protocol command
    async fn cmd_protocol(&mut self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            // Show current protocol and available protocols
            self.show_protocol_status().await?;
        } else {
            match args[0] {
                "list" | "ls" => {
                    // List all available protocols
                    self.list_protocols().await?;
                }
                "set" => {
                    // Set protocol for current port
                    if args.len() < 2 {
                        println!("Usage: protocol set <protocol_name>");
                        println!("Available protocols:");
                        self.list_protocols().await?;
                    } else {
                        self.set_port_protocol(args[1]).await?;
                    }
                }
                "show" | "status" => {
                    // Show current protocol details
                    self.show_protocol_status().await?;
                }
                "clear" | "none" => {
                    // Clear protocol from current port
                    self.clear_port_protocol().await?;
                }
                _ => {
                    // Try to set protocol directly (shorthand)
                    self.set_port_protocol(args[0]).await?;
                }
            }
        }

        Ok(())
    }

    /// Show protocol status for current port
    async fn show_protocol_status(&self) -> Result<()> {
        if let Some(ref port_id) = self.current_port_id {
            match self.manager.get_port_protocol_name(port_id).await {
                Ok(Some(protocol)) => {
                    println!("Current protocol: {}", protocol);
                    println!();
                    println!("Protocol commands:");
                    println!("  protocol list          - List all available protocols");
                    println!("  protocol set <name>    - Set protocol for current port");
                    println!("  protocol clear         - Clear protocol from current port");
                    println!("  protocol show          - Show protocol status");
                }
                Ok(None) => {
                    println!("Current protocol: (none)");
                    println!();
                    println!("Available protocols:");
                    self.list_protocols().await?;
                    println!();
                    println!("Use 'protocol set <name>' to attach a protocol to this port");
                }
                Err(e) => {
                    println!("Error getting protocol: {}", e);
                }
            }
        } else {
            println!("No port is currently open");
            println!("Use 'open <port>' first");
        }

        Ok(())
    }

    /// List all available protocols
    async fn list_protocols(&self) -> Result<()> {
        println!("Built-in protocols:");
        println!("  - modbus_rtu      - Modbus RTU protocol");
        println!("  - modbus_ascii    - Modbus ASCII protocol");
        println!("  - at_command      - AT Command protocol");
        println!("  - line            - Line-based protocol");
        println!();
        println!("Custom protocols can be loaded with 'protocol_load' in Lua scripts");

        Ok(())
    }

    /// Set protocol for current port
    async fn set_port_protocol(&mut self, protocol_name: &str) -> Result<()> {
        if self.current_port_id.is_none() {
            println!("No port is currently open");
            println!("Use 'open <port>' first");
            return Ok(());
        }

        // Validate and create protocol instance
        let protocol = match crate::protocol::built_in::create_builtin_protocol(protocol_name) {
            Some(p) => p,
            None => {
                println!("Unknown protocol: {}", protocol_name);
                println!();
                println!("Available protocols:");
                self.list_protocols().await?;
                return Ok(());
            }
        };

        let port_id = self.current_port_id.as_ref().unwrap();
        match self
            .manager
            .set_port_protocol_instance(port_id, Some(protocol))
            .await
        {
            Ok(_) => {
                println!("Protocol '{}' set for port", protocol_name);
                println!(
                    "Data will now be processed using the {} protocol",
                    protocol_name
                );
            }
            Err(e) => {
                println!("Failed to set protocol: {}", e);
            }
        }

        Ok(())
    }

    /// Clear protocol from current port
    async fn clear_port_protocol(&mut self) -> Result<()> {
        if self.current_port_id.is_none() {
            println!("No port is currently open");
            println!("Use 'open <port>' first");
            return Ok(());
        }

        let port_id = self.current_port_id.as_ref().unwrap();
        match self.manager.set_port_protocol_instance(port_id, None).await {
            Ok(_) => {
                println!("Protocol cleared from port");
                println!("Data will be processed as raw bytes");
            }
            Err(e) => {
                println!("Failed to clear protocol: {}", e);
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

    #[test]
    fn test_shell_creation() {
        let shell = InteractiveShell::new();
        assert!(!shell.running);
        assert!(shell.current_port_id.is_none());
    }
}
