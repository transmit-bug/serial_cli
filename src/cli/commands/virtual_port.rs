//! Virtual serial port command handler
//!
//! Handles `serial-cli virtual create|list|stop|stats`.
//! Manages a global registry of active virtual port pairs.

use std::collections::HashMap;
use std::sync::Arc;

use once_cell::sync::Lazy;
use tokio::sync::RwLock;

use crate::cli::types::VirtualCommand;
use crate::error::{Result, SerialError};
use crate::serial_core::{BackendType, VirtualConfig, VirtualSerialPair};

/// In-memory registry of active virtual port pairs, keyed by pair ID.
static VIRTUAL_REGISTRY: Lazy<Arc<RwLock<HashMap<String, VirtualSerialPair>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Dispatch a [`VirtualCommand`] to create, list, stop, or show stats for
/// virtual serial port pairs.
///
/// # Errors
///
/// Returns [`SerialError::VirtualPort`] if the requested pair is not found
/// or if the backend is unavailable on the current platform.
/// Returns [`SerialError::UnsupportedBackend`] for invalid backend type strings.
pub async fn handle_virtual_command(cmd: VirtualCommand) -> Result<()> {
    match cmd {
        VirtualCommand::Create {
            backend,
            monitor,
            output,
            max_packets,
        } => {
            tracing::info!("Creating virtual serial port pair...");

            // Load configuration for defaults
            let config_manager = crate::config::ConfigManager::load_with_fallback();

            // Parse backend type from CLI argument
            // Priority: CLI arg > config file > auto-detect
            let backend_type = if backend == "auto" || backend.is_empty() {
                // Use config or auto-detect
                config_manager.get_virtual_backend_type()
            } else {
                // Parse CLI argument
                match backend.parse::<BackendType>() {
                    Ok(backend) => backend,
                    Err(e) => {
                        tracing::error!("Invalid backend type: {}", e);
                        tracing::info!("Available backends: auto, pty, namedpipe, socat");
                        return Err(e);
                    }
                }
            };

            tracing::info!("Using backend type: {:?}", backend_type);

            // Check if backend is available on this platform
            if !backend_type.is_available() {
                return Err(SerialError::VirtualPort(format!(
                    "Backend {:?} is not available on this platform",
                    backend_type
                )));
            }

            // Get app config for other settings
            let app_config = config_manager.get();

            // Use monitor from config if not explicitly set
            let monitor_enabled = if !monitor {
                app_config.virtual_ports.monitor
            } else {
                monitor
            };

            // Use max_packets from config if not explicitly set
            let max_packets_config = if max_packets == 0 {
                app_config.virtual_ports.max_packets
            } else {
                max_packets
            };

            // Create virtual config
            let config = VirtualConfig {
                backend: backend_type,
                monitor: monitor_enabled,
                monitor_output: output,
                max_packets: max_packets_config,
                bridge_buffer_size: app_config.virtual_ports.bridge_buffer_size,
            };

            // Create the virtual pair
            let pair = VirtualSerialPair::create(config).await?;

            // Clone the values we need before moving pair
            let id = pair.id.clone();
            let port_a = pair.port_a.clone();
            let port_b = pair.port_b.clone();

            // Store in registry
            let mut registry = VIRTUAL_REGISTRY.write().await;
            registry.insert(id.clone(), pair);

            tracing::info!("Virtual port pair created successfully");
            println!("✓ Virtual port pair created");
            println!("  ID: {}", id);
            println!("  Port A: {}", port_a);
            println!("  Port B: {}", port_b);
            println!("  Backend: {:?}", backend_type);
            if monitor_enabled {
                println!("  Monitoring: enabled (max {} packets)", max_packets_config);
            }
        }

        VirtualCommand::List => {
            let registry = VIRTUAL_REGISTRY.read().await;

            if registry.is_empty() {
                println!("No active virtual port pairs");
                println!();
                println!("Create a virtual pair with:");
                println!("  serial-cli virtual create");
            } else {
                println!("Active virtual port pairs:");
                println!();
                for (id, pair) in registry.iter() {
                    let stats = pair.stats().await;
                    println!("  ID: {}", id);
                    println!("    Port A: {}", stats.port_a);
                    println!("    Port B: {}", stats.port_b);
                    println!("    Backend: {:?}", stats.backend);
                    println!("    Uptime: {}s", stats.uptime_secs);
                    println!(
                        "    Status: {}",
                        if stats.running { "Running" } else { "Stopped" }
                    );
                    println!("    Bytes bridged: {}", stats.bytes_bridged);
                    println!("    Packets bridged: {}", stats.packets_bridged);
                    if stats.bridge_errors > 0 {
                        println!("    Bridge errors: {}", stats.bridge_errors);
                    }
                    println!();
                }
            }
        }

        VirtualCommand::Stop { id } => {
            let mut registry = VIRTUAL_REGISTRY.write().await;

            if let Some(pair) = registry.remove(&id) {
                tracing::info!("Stopping virtual pair: {}", id);
                match pair.stop().await {
                    Ok(_) => tracing::info!("\u{2713} Virtual pair stopped"),
                    Err(e) => {
                        tracing::error!("\u{26A0} Error stopping virtual pair: {}", e);
                        return Err(e);
                    }
                }
            } else {
                tracing::error!("\u{2717} Virtual pair not found: {}", id);
                tracing::info!("Use 'serial-cli virtual list' to see active pairs");
                return Err(SerialError::VirtualPort(format!(
                    "Virtual pair not found: {}",
                    id
                )));
            }
        }

        VirtualCommand::Stats { id } => {
            let registry = VIRTUAL_REGISTRY.read().await;

            if let Some(pair) = registry.get(&id) {
                let stats = pair.stats().await;
                println!("Virtual pair statistics:");
                println!("  ID: {}", stats.id);
                println!("  Port A: {}", stats.port_a);
                println!("  Port B: {}", stats.port_b);
                println!("  Backend: {:?}", stats.backend);
                println!(
                    "  Status: {}",
                    if stats.running { "Running" } else { "Stopped" }
                );
                println!("  Uptime: {}s", stats.uptime_secs);
                println!("  Bytes bridged: {}", stats.bytes_bridged);
                println!("  Packets bridged: {}", stats.packets_bridged);
                println!("  Bridge errors: {}", stats.bridge_errors);

                if let Some(ref error) = stats.last_error {
                    println!("  Last error: {}", error);
                }
            } else {
                eprintln!("\u{2717} Virtual pair not found: {}", id);
                println!("Use 'serial-cli virtual list' to see active pairs");
                return Err(SerialError::VirtualPort(format!(
                    "Virtual pair not found: {}",
                    id
                )));
            }
        }
    }

    Ok(())
}
