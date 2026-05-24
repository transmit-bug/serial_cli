// Copyright 2024 Serial CLI Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::state::app_state::AppState;
use crate::state::port_state::{PortStats, PortStatus, SerialConfig};
use serial_cli::serial_core::{FlowControl, Parity, PortManager, SerialConfig as CoreSerialConfig};
use tauri::State;

/// List available serial ports (includes hardware + virtual ports)
#[tauri::command]
pub async fn list_ports(state: State<'_, AppState>) -> Result<Vec<PortInfo>, String> {
    use tokio::sync::MutexGuard;

    // Get hardware ports
    let manager: MutexGuard<serial_cli::serial_core::PortManager> = state.port_manager.lock().await;
    let hw_ports = manager
        .list_ports()
        .map(|ports| {
            ports
                .into_iter()
                .filter(|p| {
                    // Filter out debug consoles and pseudo-terminals that cause ENOTTY errors
                    !p.port_name.contains("debug-console")
                        && !p.port_name.contains("pty.")
                        && !p.port_name.contains("ttys")
                })
                .map(|p| PortInfo {
                    port_name: p.port_name,
                    port_type: format!("{:?}", p.port_type),
                    is_virtual: false,
                    virtual_id: None,
                })
                .collect::<Vec<_>>()
        })
        .map_err(|e| e.to_string())?;

    // Get virtual ports from registry (sorted by port_name for stability)
    let registry = state.virtual_port_registry.read().await;
    let mut virt_ports: Vec<PortInfo> = {
        let mut ports = Vec::new();
        for (id, pair) in registry.iter() {
            ports.push(PortInfo {
                port_name: pair.port_a.clone(),
                port_type: format!("Virtual ({:?})", pair.backend_type),
                is_virtual: true,
                virtual_id: Some(id.clone()),
            });
            ports.push(PortInfo {
                port_name: pair.port_b.clone(),
                port_type: format!("Virtual ({:?})", pair.backend_type),
                is_virtual: true,
                virtual_id: Some(id.clone()),
            });
        }
        ports
    };
    virt_ports.sort_by(|a, b| a.port_name.cmp(&b.port_name));
    drop(registry);

    // Merge: hardware ports first, then virtual ports
    let mut all_ports = hw_ports;
    all_ports.extend(virt_ports);

    Ok(all_ports)
}

/// Open a serial port
#[tauri::command]
pub async fn open_port(
    port_name: String,
    config: SerialConfig,
    is_virtual: Option<bool>,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    use tokio::sync::MutexGuard;

    let manager: MutexGuard<PortManager> = state.port_manager.lock().await;

    let core_config = CoreSerialConfig {
        baudrate: config.baudrate,
        databits: config.databits,
        stopbits: config.stopbits,
        parity: parse_parity(&config.parity),
        timeout_ms: config.timeout_ms,
        flow_control: parse_flow_control(&config.flow_control),
        dtr_enable: false,
        rts_enable: false,
    };

    let port_id = manager
        .open_port_virtual(&port_name, core_config, is_virtual.unwrap_or(false))
        .await
        .map_err(|e: serial_cli::error::SerialError| e.to_string())?;

    drop(manager);

    if let Err(e) = crate::events::emitter::emit_port_status_changed(
        app,
        port_id.clone(),
        serde_json::json!({"status": "opened", "port_name": port_name}),
    )
    .await
    {
        log::warn!("Failed to emit port-status-changed event: {}", e);
    }

    Ok(port_id)
}

/// Close a serial port
#[tauri::command]
pub async fn close_port(
    port_id: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let manager = state.port_manager.lock().await;
    manager
        .close_port(&port_id)
        .await
        .map_err(|e| e.to_string())?;
    drop(manager);

    if let Err(e) = crate::events::emitter::emit_port_status_changed(
        app,
        port_id.clone(),
        serde_json::json!({"status": "closed"}),
    )
    .await
    {
        log::warn!("Failed to emit port-status-changed event: {}", e);
    }

    Ok(())
}

/// Get port status
#[tauri::command]
pub async fn get_port_status(
    port_id: String,
    state: State<'_, AppState>,
) -> Result<PortStatus, String> {
    use tokio::sync::MutexGuard;

    let manager: MutexGuard<PortManager> = state.port_manager.lock().await;
    let port_handle = manager
        .get_port(&port_id)
        .await
        .map_err(|e: serial_cli::error::SerialError| e.to_string())?;
    let handle = port_handle.lock().await;

    // Get tracked stats
    let stats_snapshot = {
        let port_stats = state.port_stats.lock().await;
        port_stats
            .get(&port_id)
            .map(|s| {
                let (br, bs, pr, ps, la) = s.snapshot();
                PortStats {
                    bytes_received: br,
                    bytes_sent: bs,
                    packets_received: pr,
                    packets_sent: ps,
                    last_activity: if la > 0 { Some(la) } else { None },
                }
            })
            .unwrap_or_default()
    };

    Ok(PortStatus {
        id: port_id,
        port_name: handle.name().to_string(),
        is_open: true,
        config: Some(SerialConfig {
            baudrate: handle.config().baudrate,
            databits: handle.config().databits,
            stopbits: handle.config().stopbits,
            parity: format!("{:?}", handle.config().parity),
            timeout_ms: handle.config().timeout_ms,
            flow_control: format!("{:?}", handle.config().flow_control),
        }),
        stats: stats_snapshot,
    })
}

/// Get all active ports status
#[tauri::command]
pub async fn get_all_ports_status(state: State<'_, AppState>) -> Result<Vec<PortStatus>, String> {
    let open_ports = {
        let manager = state.port_manager.lock().await;
        manager.list_open_ports().await
    };

    let mut statuses = Vec::with_capacity(open_ports.len());

    for (port_id, port_name) in open_ports {
        let stats_snapshot = {
            let port_stats = state.port_stats.lock().await;
            port_stats
                .get(&port_id)
                .map(|s| {
                    let (br, bs, pr, ps, la) = s.snapshot();
                    PortStats {
                        bytes_received: br,
                        bytes_sent: bs,
                        packets_received: pr,
                        packets_sent: ps,
                        last_activity: if la > 0 { Some(la) } else { None },
                    }
                })
                .unwrap_or_default()
        };

        let config = {
            let manager = state.port_manager.lock().await;
            if let Ok(port_handle) = manager.get_port(&port_id).await {
                let handle = port_handle.lock().await;
                Some(SerialConfig {
                    baudrate: handle.config().baudrate,
                    databits: handle.config().databits,
                    stopbits: handle.config().stopbits,
                    parity: format!("{:?}", handle.config().parity),
                    timeout_ms: handle.config().timeout_ms,
                    flow_control: format!("{:?}", handle.config().flow_control),
                })
            } else {
                None
            }
        };

        statuses.push(PortStatus {
            id: port_id,
            port_name,
            is_open: true,
            config,
            stats: stats_snapshot,
        });
    }

    Ok(statuses)
}

/// Check if a specific port is still connected and responsive
#[tauri::command]
pub async fn check_port_health(
    port_id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    use tokio::sync::MutexGuard;

    let manager: MutexGuard<PortManager> = state.port_manager.lock().await;

    // Try to get the port handle
    match manager.get_port(&port_id).await {
        Ok(_) => Ok(true),   // Port is still accessible
        Err(_) => Ok(false), // Port is closed or error
    }
}

/// Port information
#[derive(serde::Serialize)]
pub struct PortInfo {
    pub port_name: String,
    pub port_type: String,
    /// Whether this is a virtual port (PTY/named-pipe/socat)
    #[serde(default)]
    pub is_virtual: bool,
    /// Virtual port pair ID (only set when is_virtual == true)
    #[serde(default)]
    pub virtual_id: Option<String>,
}

/// Parse parity from string
fn parse_parity(parity: &str) -> Parity {
    match parity.to_lowercase().as_str() {
        "odd" => Parity::Odd,
        "even" => Parity::Even,
        _ => Parity::None,
    }
}

/// Parse flow control from string
fn parse_flow_control(flow: &str) -> FlowControl {
    match flow.to_lowercase().as_str() {
        "software" => FlowControl::Software,
        "hardware" => FlowControl::Hardware,
        _ => FlowControl::None,
    }
}
