//! Virtual serial port command handler
//!
//! Handles `serial-cli virtual create|list|stop|stats`.
//!
//! Two kinds of pairs are tracked:
//!
//! - **In-process pairs** (pty backend): live in the [`VIRTUAL_REGISTRY`] and
//!   die with the CLI process. `shutdown_all` stops them on exit.
//! - **Detached pairs** (socat backend, `virtual create --backend socat`):
//!   spawned as a session-detached socat process and tracked in a state file
//!   (`src/cli/virtual_port_session.rs`), so they survive the creating CLI
//!   process and can be listed/stopped from later invocations (#70).

use std::collections::HashMap;
use std::sync::Arc;

use once_cell::sync::Lazy;
use tokio::sync::RwLock;

use crate::cli::types::VirtualCommand;
use crate::error::{Result, SerialError};
use crate::serial_core::{BackendType, VirtualConfig, VirtualSerialPair};
use serde_json::json;

/// In-memory registry of in-process virtual port pairs, keyed by pair ID.
static VIRTUAL_REGISTRY: Lazy<Arc<RwLock<HashMap<String, VirtualSerialPair>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Stop and clean up all in-process registered virtual port pairs.
///
/// In-process pairs are process-scoped: the in-memory registry dies with the
/// process. Backends that spawn external processes (socat) or create symlinks
/// must not leave those behind when the CLI exits, so `main` invokes this
/// before returning. Detached (persistent) socat pairs tracked in the state
/// file are intentionally NOT here — they must survive the CLI exit (#70).
pub async fn shutdown_all() {
    let mut registry = VIRTUAL_REGISTRY.write().await;
    for (_, pair) in registry.drain() {
        if let Err(e) = pair.stop().await {
            eprintln!("Warning: failed to stop virtual pair on exit: {}", e);
        }
    }
}

/// Dispatch a [`VirtualCommand`] to create, list, stop, or show stats for
/// virtual serial port pairs.
///
/// # Errors
///
/// Returns [`SerialError::VirtualPort`] if the requested pair is not found
/// or if the backend is unavailable on the current platform.
/// Returns [`SerialError::UnsupportedBackend`] for invalid backend type strings.
pub async fn handle_virtual_command(cmd: VirtualCommand, json_output: bool) -> Result<()> {
    match cmd {
        VirtualCommand::Create {
            backend,
            monitor,
            output,
            max_packets,
        } => {
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
                        eprintln!("Error: Invalid backend type: {}", e);
                        eprintln!("Available backends: auto, pty, namedpipe, socat");
                        return Err(e);
                    }
                }
            };

            // Check if backend is available on this platform
            if !backend_type.is_available() {
                return Err(SerialError::VirtualPort(format!(
                    "Backend {:?} is not available on this platform",
                    backend_type
                )));
            }

            // Get app config for other settings
            let app_config = config_manager.get();

            // Use max_packets from config if not explicitly set
            let max_packets_config = if max_packets == 0 {
                app_config.virtual_ports.max_packets
            } else {
                max_packets
            };

            // ── Detached (persistent) socat path (#70) ───────────────
            // A socat pair is spawned as a session-detached process and
            // tracked in a state file, so it survives this CLI process.
            // It is NOT inserted into the in-memory VIRTUAL_REGISTRY (whose
            // Drop/shutdown_all semantics would kill it on exit).
            if backend_type == BackendType::Socat {
                let entry = crate::cli::virtual_port_session::create_socat_pair()?;

                if json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "id": entry.id,
                            "portA": entry.port_a,
                            "portB": entry.port_b,
                            "backend": "socat",
                            "monitor": false,
                            "maxPackets": max_packets_config,
                            "detached": true,
                            "socatPid": entry.socat_pid,
                        }))
                        .unwrap()
                    );
                } else {
                    println!("✓ Virtual port pair created (detached socat backend)");
                    println!("  ID: {}", entry.id);
                    println!("  Port A: {}", entry.port_a);
                    println!("  Port B: {}", entry.port_b);
                    println!("  Backend: socat (persistent — survives this CLI process)");
                    println!("  Socat PID: {}", entry.socat_pid);
                    println!();
                    println!("  Use 'virtual list' to see active pairs");
                    println!("  Use 'virtual stop {}' to stop it", entry.id);
                }
                return Ok(());
            }

            // Use monitor from config if not explicitly set
            let monitor_enabled = if !monitor {
                app_config.virtual_ports.monitor
            } else {
                monitor
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

            if json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "id": id,
                        "portA": port_a,
                        "portB": port_b,
                        "backend": backend_type.to_string(),
                        "monitor": monitor_enabled,
                        "maxPackets": max_packets_config,
                    }))
                    .unwrap()
                );
            } else {
                println!("✓ Virtual port pair created");
                println!("  ID: {}", id);
                println!("  Port A: {}", port_a);
                println!("  Port B: {}", port_b);
                println!("  Backend: {:?}", backend_type);
                if monitor_enabled {
                    println!("  Monitoring: enabled (max {} packets)", max_packets_config);
                }
            }
        }

        VirtualCommand::List => {
            // Detached (persistent) pairs from the state file, pruned of
            // entries whose socat process died (#70).
            let detached = crate::cli::virtual_port_session::load_and_prune()?;
            let registry = VIRTUAL_REGISTRY.read().await;

            if json_output {
                let mut items: Vec<serde_json::Value> = Vec::new();
                for entry in &detached {
                    let stats = crate::cli::virtual_port_session::entry_stats(entry);
                    items.push(json!({
                        "id": stats.id,
                        "portA": stats.port_a,
                        "portB": stats.port_b,
                        "backend": stats.backend,
                        "running": stats.running,
                        "uptimeSecs": stats.uptime_secs,
                        "bytesBridged": stats.bytes_bridged,
                        "packetsBridged": stats.packets_bridged,
                        "bridgeErrors": stats.bridge_errors,
                    }));
                }
                for (id, pair) in registry.iter() {
                    let stats = pair.stats().await;
                    items.push(json!({
                        "id": id,
                        "portA": stats.port_a,
                        "portB": stats.port_b,
                        "backend": stats.backend.to_string(),
                        "running": stats.running,
                        "uptimeSecs": stats.uptime_secs,
                        "bytesBridged": stats.bytes_bridged,
                        "packetsBridged": stats.packets_bridged,
                        "bridgeErrors": stats.bridge_errors,
                    }));
                }
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "pairs": items,
                        "count": items.len()
                    }))
                    .unwrap()
                );
            } else if detached.is_empty() && registry.is_empty() {
                println!("No active virtual port pairs");
                println!();
                println!("Create a virtual pair with:");
                println!("  serial-cli virtual create");
            } else {
                println!("Active virtual port pairs:");
                println!();
                for entry in &detached {
                    let stats = crate::cli::virtual_port_session::entry_stats(entry);
                    println!("  ID: {}", stats.id);
                    println!("    Port A: {}", stats.port_a);
                    println!("    Port B: {}", stats.port_b);
                    println!("    Backend: {}", stats.backend);
                    println!("    Uptime: {}s", stats.uptime_secs);
                    println!(
                        "    Status: {}",
                        if stats.running { "Running" } else { "Stopped" }
                    );
                    println!("    Socat PID: {}", entry.socat_pid);
                    println!();
                }
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
            // 1. Detached (persistent) pair — works across processes (#70).
            match crate::cli::virtual_port_session::stop_socat_pair(&id)? {
                Some((_entry, was_running)) => {
                    if json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&json!({
                                "ok": true,
                                "id": id,
                                "stale": !was_running,
                            }))
                            .unwrap()
                        );
                    } else if was_running {
                        println!("✓ Virtual pair stopped");
                    } else {
                        println!("✓ Virtual pair stopped (stale — socat was no longer running)");
                        println!("  Cleaned up symlinks and state entry for {}", id);
                    }
                }
                None => {
                    // 2. Fall back to the in-process registry (pty/session pairs).
                    let mut registry = VIRTUAL_REGISTRY.write().await;

                    if let Some(pair) = registry.remove(&id) {
                        match pair.stop().await {
                            Ok(_) => {
                                if json_output {
                                    println!(
                                        "{}",
                                        serde_json::to_string_pretty(&json!({
                                            "ok": true,
                                            "id": id
                                        }))
                                        .unwrap()
                                    );
                                } else {
                                    println!("✓ Virtual pair stopped");
                                }
                            }
                            Err(e) => {
                                eprintln!("⚠ Error stopping virtual pair: {}", e);
                                return Err(e);
                            }
                        }
                    } else {
                        eprintln!("✗ Virtual pair not found: {}", id);
                        eprintln!("Use 'serial-cli virtual list' to see active pairs");
                        return Err(SerialError::VirtualPort(format!(
                            "Virtual pair not found: {}",
                            id
                        )));
                    }
                }
            }
        }

        VirtualCommand::Stats { id } => {
            // 1. Detached (persistent) pair — works across processes (#70).
            // `load_and_prune` also removes stale entries (crash recovery).
            let detached = crate::cli::virtual_port_session::load_and_prune()?;
            if let Some(entry) = detached.iter().find(|e| e.id == id) {
                let stats = crate::cli::virtual_port_session::entry_stats(entry);
                if json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "id": stats.id,
                            "portA": stats.port_a,
                            "portB": stats.port_b,
                            "backend": stats.backend,
                            "running": stats.running,
                            "uptimeSecs": stats.uptime_secs,
                            "bytesBridged": stats.bytes_bridged,
                            "packetsBridged": stats.packets_bridged,
                            "bridgeErrors": stats.bridge_errors,
                            "lastError": stats.last_error,
                        }))
                        .unwrap()
                    );
                } else {
                    println!("Virtual pair statistics:");
                    println!("  ID: {}", stats.id);
                    println!("  Port A: {}", stats.port_a);
                    println!("  Port B: {}", stats.port_b);
                    println!("  Backend: {}", stats.backend);
                    println!(
                        "  Status: {}",
                        if stats.running { "Running" } else { "Stopped" }
                    );
                    println!("  Uptime: {}s", stats.uptime_secs);
                }
                return Ok(());
            }

            // 2. In-process registry (pty/session pairs).
            let registry = VIRTUAL_REGISTRY.read().await;

            if let Some(pair) = registry.get(&id) {
                let stats = pair.stats().await;
                if json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "id": stats.id,
                            "portA": stats.port_a,
                            "portB": stats.port_b,
                            "backend": stats.backend.to_string(),
                            "running": stats.running,
                            "uptimeSecs": stats.uptime_secs,
                            "bytesBridged": stats.bytes_bridged,
                            "packetsBridged": stats.packets_bridged,
                            "bridgeErrors": stats.bridge_errors,
                            "lastError": stats.last_error,
                        }))
                        .unwrap()
                    );
                } else {
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
                }
            } else {
                eprintln!("✗ Virtual pair not found: {}", id);
                eprintln!("Use 'serial-cli virtual list' to see active pairs");
                return Err(SerialError::VirtualPort(format!(
                    "Virtual pair not found: {}",
                    id
                )));
            }
        }
    }

    Ok(())
}
