//! Port management command handler
//!
//! Handlers for `serial-cli port list` and `serial-cli port send`.

use crate::cli::commands::parsers;
use crate::cli::types::PortCommand;
use crate::error::Result;
use crate::serial_core::{PortManager, SerialConfig};

/// Dispatch a [`PortCommand`] to list ports or send data.
///
/// # Errors
///
/// Propagates errors from port enumeration or serial communication.
pub async fn handle_port_command(cmd: PortCommand, json_output: bool) -> Result<()> {
    match cmd {
        PortCommand::List => {
            list_ports(json_output)?;
        }
        PortCommand::Send {
            port,
            data,
            hex,
            base64,
        } => {
            send_data(&port, &data, json_output, hex, base64).await?;
        }
    }
    Ok(())
}

/// List all available serial ports and print them as JSON or human-readable text.
///
/// Uses the system's serial port enumeration to discover ports like
/// `/dev/ttyUSB0` (Linux/macOS) or `COM1` (Windows).
///
/// # Errors
///
/// Returns an error if port enumeration fails (e.g., permission issues
/// on the platform).
pub fn list_ports(json_output: bool) -> Result<()> {
    use serde_json::json;

    let manager = PortManager::new();
    let ports = manager.list_ports()?;

    if json_output {
        let output: Vec<serde_json::Value> = ports
            .iter()
            .map(|p| {
                json!({
                    "port_name": p.port_name,
                    "port_type": format!("{:?}", p.port_type),
                })
            })
            .collect();

        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        if ports.is_empty() {
            println!("No serial ports found.");
        } else {
            println!("Available serial ports:");
            for port in ports {
                println!("  - {} ({})", port.port_name, port.port_type);
            }
        }
    }

    Ok(())
}

/// Open a serial port, send the provided data, and attempt to read a response.
///
/// Uses [`SerialConfig::default()`] for port settings (115200 baud, 8N1).
/// After writing, waits 100ms then reads up to 1024 bytes of response data.
/// The port is closed after the operation regardless of success or failure.
///
/// # Arguments
///
/// * `port` - Port name (e.g., `/dev/ttyUSB0`, `COM1`)
/// * `data` - Data to send (plain text, hex, or base64 depending on flags)
/// * `json_output` - Whether to output results as JSON
/// * `hex` - Interpret data as hex-encoded bytes
/// * `base64` - Interpret data as base64-encoded bytes
///
/// # Errors
///
/// Returns an error when:
/// - The port does not exist or cannot be opened
/// - Permission is denied
/// - The port is busy
/// - Write or read fails at the OS level
/// - Hex or base64 decoding fails
pub async fn send_data(
    port: &str,
    data: &str,
    json_output: bool,
    hex: bool,
    base64: bool,
) -> Result<()> {
    use std::thread;
    use std::time::Duration;

    // Create port manager
    let manager = PortManager::new();

    // Use default configuration
    let config = SerialConfig::default();

    // Open the port
    let port_id = manager.open_port(port, config).await?;

    if !json_output {
        println!("Opening port: {}", port);
        println!("Port opened successfully: {}", port_id);
        println!("Sending data: {}", data);
    }

    // Get the port handle
    let port_handle = manager.get_port(&port_id).await?;
    let mut handle = port_handle.lock().await;

    // Convert data to bytes
    let bytes: Vec<u8> = if hex {
        parsers::parse_hex_string(data)?
    } else if base64 {
        parsers::base64_decode(data)?
    } else {
        data.as_bytes().to_vec()
    };

    // Send data
    let bytes_written = handle.write(&bytes)?;

    // Wait a bit for response
    thread::sleep(Duration::from_millis(100));

    // Try to read response
    let mut buffer = [0u8; 1024];
    let (bytes_read, response) = match handle.read(&mut buffer) {
        Ok(br) => {
            if br > 0 {
                let resp = String::from_utf8_lossy(&buffer[..br]).to_string();
                (br, Some(resp))
            } else {
                (0, None)
            }
        }
        Err(e) => {
            if !json_output {
                eprintln!("Note: Could not read response: {}", e);
            }
            (0, None)
        }
    };

    // Close the port
    drop(handle);
    manager.close_port(&port_id).await?;

    if json_output {
        use serde_json::json;
        let result = json!({
            "port": port,
            "port_id": port_id,
            "data_sent": data,
            "bytes_written": bytes_written,
            "bytes_read": bytes_read,
            "response": response
        });
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else {
        println!("Sent {} bytes", bytes_written);
        if bytes_read > 0 {
            println!(
                "Received response ({} bytes): {}",
                bytes_read,
                response.as_ref().unwrap()
            );
        } else {
            println!("No response received");
        }
        println!("Port closed");
    }

    Ok(())
}
