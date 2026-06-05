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
    #[allow(dead_code)]
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
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<VirtualPortStats, String> {
    let registry = state.virtual_port_registry.read().await;

    if let Some(pair) = registry.get(&id) {
        let stats = pair.stats().await;
        let result = VirtualPortStats {
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
        };

        let stats_json = serde_json::json!({
            "bytes_bridged": result.bytes_bridged,
            "packets_bridged": result.packets_bridged,
            "running": result.running,
        });

        if let Err(e) =
            crate::events::emitter::emit_virtual_port_stats_updated(app, id.clone(), stats_json)
                .await
        {
            log::warn!("Failed to emit virtual-port-stats-updated event: {}", e);
        }

        Ok(result)
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

/// Send data to one end of a virtual serial port pair.
///
/// Opens the target device path as a regular serial port, writes the data,
/// and closes it immediately. This is a one-shot write suitable for testing.
#[tauri::command]
pub async fn send_to_virtual_port(
    id: String,
    port_end: String,
    data: Vec<u8>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let registry = state.virtual_port_registry.read().await;
    let pair = registry
        .get(&id)
        .ok_or_else(|| format!("Virtual port not found: {}", id))?;

    let device_path = if port_end == "b" || port_end == "port_b" {
        pair.port_b.clone()
    } else if port_end == "a" || port_end == "port_a" {
        pair.port_a.clone()
    } else {
        return Err(format!(
            "Invalid port_end: {}. Must be 'a' or 'b'",
            port_end
        ));
    };
    drop(registry);

    // Open the virtual port device through the PortManager, write data, and close it
    let config = serial_cli::serial_core::SerialConfig {
        baudrate: 115200,
        databits: 8,
        stopbits: 1,
        parity: serial_cli::serial_core::Parity::None,
        timeout_ms: 1000,
        flow_control: serial_cli::serial_core::FlowControl::None,
        dtr_enable: false,
        rts_enable: false,
    };

    let manager = state.port_manager.lock().await;
    let port_id = manager
        .open_port_virtual(&device_path, config, true)
        .await
        .map_err(|e| e.to_string())?;

    let port_handle = manager
        .get_port(&port_id)
        .await
        .map_err(|e: serial_cli::error::SerialError| e.to_string())?;
    let mut handle = port_handle.lock().await;

    let bytes_written = handle
        .write(&data)
        .map_err(|e: serial_cli::error::SerialError| e.to_string())?;
    drop(handle);

    // Clean up the temporary port
    let _ = manager.close_port(&port_id).await;
    drop(manager);

    // Emit data-sent event for the virtual port
    if let Err(e) = crate::events::emitter::emit_data_sent(app, id.clone(), data.clone()).await {
        log::warn!("Failed to emit data-sent event for virtual port: {}", e);
    }

    Ok(bytes_written)
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
                serial_cli::serial_core::VirtualPacketDirection::AtoB => "AtoB".to_string(),
                serial_cli::serial_core::VirtualPacketDirection::BtoA => "BtoA".to_string(),
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

fn parse_backend_type(backend: &str) -> Option<BackendType> {
    match backend {
        "pty" => Some(BackendType::Pty),
        "namedpipe" => Some(BackendType::NamedPipe),
        "socat" => Some(BackendType::Socat),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_backend_type_pty() {
        assert_eq!(parse_backend_type("pty"), Some(BackendType::Pty));
    }

    #[test]
    fn test_parse_backend_type_namedpipe() {
        assert_eq!(parse_backend_type("namedpipe"), Some(BackendType::NamedPipe));
    }

    #[test]
    fn test_parse_backend_type_socat() {
        assert_eq!(parse_backend_type("socat"), Some(BackendType::Socat));
    }

    #[test]
    fn test_parse_backend_type_invalid() {
        assert_eq!(parse_backend_type("invalid"), None);
        assert_eq!(parse_backend_type(""), None);
        assert_eq!(parse_backend_type("PTY"), None); // case-sensitive
    }

    #[test]
    fn test_captured_packet_dto_serialization() {
        let dto = CapturedPacketDto {
            direction: "AtoB".to_string(),
            data: vec![0x01, 0x02, 0x03],
            timestamp_millis: 1234567890,
        };
        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("AtoB"));
        assert!(json.contains("1234567890"));
    }

    #[test]
    fn test_virtual_port_stats_serialization() {
        let stats = VirtualPortStats {
            id: "vp-1".to_string(),
            port_a: "/dev/pty0".to_string(),
            port_b: "/dev/pty1".to_string(),
            backend: "Pty".to_string(),
            running: true,
            uptime_secs: 300,
            bytes_bridged: 10240,
            packets_bridged: 100,
            bridge_errors: 0,
            last_error: None,
            capture_packets: 50,
            capture_bytes: 5120,
            monitoring: true,
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("vp-1"));
        assert!(json.contains("running"));
        assert!(json.contains("10240"));
        assert!(json.contains("Pty"));
    }

    #[test]
    fn test_create_virtual_port_config_deserialization() {
        let json = r#"{
            "name": "test-pair",
            "backend": "pty",
            "buffer_size": 4096,
            "monitor": true
        }"#;
        let config: CreateVirtualPortConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.backend, "pty");
        assert_eq!(config.buffer_size, Some(4096));
        assert_eq!(config.monitor, Some(true));
    }

    #[test]
    fn test_create_virtual_port_config_optional_fields() {
        let json = r#"{"backend": "socat"}"#;
        let config: CreateVirtualPortConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.backend, "socat");
        assert!(config.name.is_none());
        assert!(config.buffer_size.is_none());
        assert!(config.monitor.is_none());
    }
}
