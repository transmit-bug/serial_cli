// Copyright 2024 Serial CLI Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::state::app_state::AppState;
use serial_cli::serial_core::backends::BackendType;
use serial_cli::serial_core::{VirtualConfig, VirtualSerialPair};
use tauri::{AppHandle, State};

/// Virtual port information for Tauri frontend
#[derive(serde::Serialize)]
pub struct VirtualPortInfo {
    pub id: String,
    pub port_a: String,
    pub port_b: String,
    pub backend: String,
    pub created_at: String,
    pub uptime_secs: u64,
    pub running: bool,
}

/// Virtual port statistics for Tauri frontend
#[derive(serde::Serialize)]
pub struct VirtualPortStats {
    pub id: String,
    pub port_a: String,
    pub port_b: String,
    pub backend: String,
    pub running: bool,
    pub uptime_secs: u64,
    pub bytes_bridged: u64,
    pub packets_bridged: u64,
    pub bridge_errors: u64,
    pub last_error: Option<String>,
    pub capture_packets: u64,
    pub capture_bytes: u64,
    pub monitoring: bool,
}

/// A single captured packet for the frontend
#[derive(serde::Serialize)]
pub struct CapturedPacketDto {
    pub direction: String,
    pub data: Vec<u8>,
    pub timestamp_millis: u64,
}

/// Virtual port configuration from Tauri frontend
#[derive(serde::Deserialize)]
pub struct CreateVirtualPortConfig {
    #[allow(dead_code)] // Reserved for future GUI use
    pub name: Option<String>,
    pub backend: String,
    pub buffer_size: Option<usize>,
    pub monitor: Option<bool>,
}

/// Create a virtual serial port pair
#[tauri::command]
pub async fn create_virtual_port(
    config: CreateVirtualPortConfig,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Parse backend type
    let backend_type = match config.backend.as_str() {
        "pty" => BackendType::Pty,
        "namedpipe" => BackendType::NamedPipe,
        "socat" => BackendType::Socat,
        _ => {
            return Err(format!(
                "Unknown backend: {}. Available: pty, namedpipe, socat",
                config.backend
            ))
        }
    };

    // Check if backend is available
    if !backend_type.is_available() {
        return Err(format!(
            "Backend {:?} is not available on this platform",
            backend_type
        ));
    }

    // Create virtual config
    let virtual_config = VirtualConfig {
        backend: backend_type,
        monitor: config.monitor.unwrap_or(false),
        monitor_output: None,
        max_packets: 0,
        bridge_buffer_size: config.buffer_size.unwrap_or(8192),
    };

    // Create the virtual pair
    let pair = VirtualSerialPair::create(virtual_config)
        .await
        .map_err(|e| e.to_string())?;

    let id = pair.id.clone();

    // Get port info for event
    let stats = pair.stats().await;
    let port_info = serde_json::json!({
        "id": id,
        "port_a": stats.port_a,
        "port_b": stats.port_b,
        "backend": format!("{:?}", stats.backend),
        "created_at": format!("{:?}", pair.created_at),
    });

    // Add to registry
    let mut registry = state.virtual_port_registry.write().await;
    registry.insert(id.clone(), pair);
    drop(registry);

    // Emit event
    if let Err(e) =
        crate::events::emitter::emit_virtual_port_created(app.clone(), id.clone(), port_info).await
    {
        eprintln!("Failed to emit virtual-port-created event: {}", e);
    }

    Ok(id)
}

/// List all active virtual port pairs
#[tauri::command]
pub async fn list_virtual_ports(
    state: State<'_, AppState>,
) -> Result<Vec<VirtualPortInfo>, String> {
    let registry = state.virtual_port_registry.read().await;

    let mut ports = Vec::new();
    for (id, pair) in registry.iter() {
        let stats = pair.stats().await;
        ports.push(VirtualPortInfo {
            id: id.clone(),
            port_a: stats.port_a.clone(),
            port_b: stats.port_b.clone(),
            backend: format!("{:?}", stats.backend),
            created_at: format!("{:?}", pair.created_at),
            uptime_secs: stats.uptime_secs,
            running: stats.running,
        });
    }

    Ok(ports)
}

/// Stop a virtual port pair
#[tauri::command]
pub async fn stop_virtual_port(
    id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut registry = state.virtual_port_registry.write().await;

    if let Some(pair) = registry.remove(&id) {
        let id_clone = id.clone();
        pair.stop().await.map_err(|e| e.to_string())?;
        drop(registry);

        // Emit event
        if let Err(e) = crate::events::emitter::emit_virtual_port_stopped(app, id_clone).await {
            eprintln!("Failed to emit virtual-port-stopped event: {}", e);
        }

        Ok(())
    } else {
        Err(format!("Virtual port not found: {}", id))
    }
}

/// Get statistics for a virtual port pair
#[tauri::command]
pub async fn get_virtual_port_stats(
    id: String,
    state: State<'_, AppState>,
) -> Result<VirtualPortStats, String> {
    let registry = state.virtual_port_registry.read().await;

    if let Some(pair) = registry.get(&id) {
        let stats = pair.stats().await;
        Ok(VirtualPortStats {
            id: stats.id.clone(),
            port_a: stats.port_a.clone(),
            port_b: stats.port_b.clone(),
            backend: format!("{:?}", stats.backend),
            running: stats.running,
            uptime_secs: stats.uptime_secs,
            bytes_bridged: stats.bytes_bridged,
            packets_bridged: stats.packets_bridged,
            bridge_errors: stats.bridge_errors,
            last_error: stats.last_error.clone(),
            capture_packets: stats.capture_packets,
            capture_bytes: stats.capture_bytes,
            monitoring: pair.is_monitoring(),
        })
    } else {
        Err(format!("Virtual port not found: {}", id))
    }
}

/// Check if a virtual port is still running
#[tauri::command]
pub async fn check_virtual_port_health(
    id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let registry = state.virtual_port_registry.read().await;

    if let Some(pair) = registry.get(&id) {
        let stats = pair.stats().await;
        Ok(stats.running)
    } else {
        Ok(false)
    }
}

/// Get captured packets for a monitored virtual port
#[tauri::command]
pub async fn get_captured_packets(
    id: String,
    state: State<'_, AppState>,
) -> Result<Vec<CapturedPacketDto>, String> {
    let registry = state.virtual_port_registry.read().await;

    let pair = registry
        .get(&id)
        .ok_or_else(|| format!("Virtual port not found: {}", id))?;

    if !pair.is_monitoring() {
        return Ok(Vec::new());
    }

    let packets = pair.captured_packets().await;
    let dtos = packets
        .into_iter()
        .map(|p| {
            let dir = match p.direction {
                serial_cli::serial_core::VirtualPacketDirection::AtoB => "A→B".to_string(),
                serial_cli::serial_core::VirtualPacketDirection::BtoA => "B→A".to_string(),
            };
            let ts = p
                .timestamp
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            CapturedPacketDto {
                direction: dir,
                data: p.data.clone(),
                timestamp_millis: ts,
            }
        })
        .collect();

    Ok(dtos)
}
