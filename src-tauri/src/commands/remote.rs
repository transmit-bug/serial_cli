// Copyright 2024 Serial CLI Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Remote device management commands
//!
//! Lets the GUI operate a serial Daemon on another machine over the LAN.
//! A device registry (name / host / port) is persisted in the app data dir;
//! every operation goes through [`RemoteRpcClient`] with a fresh connection,
//! so a rebooting target device needs no reconnect logic.

use crate::state::app_state::AppState;
use serde::{Deserialize, Serialize};
use serial_cli::server::client::{
    RemoteConnectionInfo, RemoteOpenResult, RemotePortInfo, RemoteRpcClient, RemoteServerStats,
};
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

/// A saved remote device (LAN daemon endpoint)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteDevice {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub created_at: i64,
}

// ── Device registry (JSON file in the app data dir) ──────────────────────

fn registry_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Cannot resolve app data dir: {}", e))?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("Cannot create app data dir: {}", e))?;
    Ok(dir.join("remote_devices.json"))
}

fn load_devices(app: &AppHandle) -> Result<Vec<RemoteDevice>, String> {
    let path = registry_path(app)?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Cannot read device registry: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Corrupt device registry ({}): {}", path.display(), e))
}

fn save_devices(app: &AppHandle, devices: &[RemoteDevice]) -> Result<(), String> {
    let path = registry_path(app)?;
    let json = serde_json::to_string_pretty(devices)
        .map_err(|e| format!("Cannot serialize device registry: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("Cannot write device registry: {}", e))
}

fn find_device(app: &AppHandle, id: &str) -> Result<RemoteDevice, String> {
    load_devices(app)?
        .into_iter()
        .find(|d| d.id == id)
        .ok_or_else(|| format!("Remote device not found: {}", id))
}

fn client_for(device: &RemoteDevice) -> Result<RemoteRpcClient, String> {
    RemoteRpcClient::new(&device.host, device.port).map_err(|e| e.to_string())
}

// ── Device registry commands ─────────────────────────────────────────────

/// List all saved remote devices
#[tauri::command]
pub async fn get_remote_devices(app: AppHandle) -> Result<Vec<RemoteDevice>, String> {
    load_devices(&app)
}

/// Add a remote device. Returns the full updated list.
#[tauri::command]
pub async fn add_remote_device(
    app: AppHandle,
    name: String,
    host: String,
    port: u16,
) -> Result<Vec<RemoteDevice>, String> {
    let name = name.trim().to_string();
    let host = host.trim().to_string();
    if name.is_empty() || host.is_empty() {
        return Err("Name and host are required".to_string());
    }
    if port == 0 {
        return Err("Port must be between 1 and 65535".to_string());
    }

    let mut devices = load_devices(&app)?;
    if devices.iter().any(|d| d.name == name) {
        return Err(format!("Device '{}' already exists", name));
    }

    devices.push(RemoteDevice {
        id: Uuid::new_v4().to_string(),
        name,
        host,
        port,
        created_at: chrono::Utc::now().timestamp(),
    });
    save_devices(&app, &devices)?;
    Ok(devices)
}

/// Update a remote device. Returns the full updated list.
#[tauri::command]
pub async fn update_remote_device(
    app: AppHandle,
    id: String,
    name: String,
    host: String,
    port: u16,
) -> Result<Vec<RemoteDevice>, String> {
    let mut devices = load_devices(&app)?;
    let device = devices
        .iter_mut()
        .find(|d| d.id == id)
        .ok_or_else(|| format!("Remote device not found: {}", id))?;
    device.name = name.trim().to_string();
    device.host = host.trim().to_string();
    device.port = port;
    save_devices(&app, &devices)?;
    Ok(devices)
}

/// Delete a remote device. Returns the full updated list.
#[tauri::command]
pub async fn delete_remote_device(app: AppHandle, id: String) -> Result<Vec<RemoteDevice>, String> {
    let mut devices = load_devices(&app)?;
    devices.retain(|d| d.id != id);
    save_devices(&app, &devices)?;
    Ok(devices)
}

// ── Remote RPC commands ──────────────────────────────────────────────────

/// Ping a remote device and return its daemon stats (used for connect tests)
#[tauri::command]
pub async fn test_remote_device(
    app: AppHandle,
    device_id: String,
) -> Result<RemoteServerStats, String> {
    let device = find_device(&app, &device_id)?;
    let client = client_for(&device)?;
    client.server_stats().await.map_err(|e| e.to_string())
}

/// List serial ports on a remote device
#[tauri::command]
pub async fn remote_port_list(
    _state: State<'_, AppState>,
    app: AppHandle,
    device_id: String,
) -> Result<Vec<RemotePortInfo>, String> {
    let device = find_device(&app, &device_id)?;
    let client = client_for(&device)?;
    client.port_list().await.map_err(|e| e.to_string())
}

/// Open a serial port on a remote device
#[tauri::command]
pub async fn remote_open_port(
    _state: State<'_, AppState>,
    app: AppHandle,
    device_id: String,
    port: String,
    baudrate: Option<u32>,
) -> Result<RemoteOpenResult, String> {
    let device = find_device(&app, &device_id)?;
    let client = client_for(&device)?;
    client
        .port_open(&port, baudrate.unwrap_or(115_200), None)
        .await
        .map_err(|e| e.to_string())
}

/// Close a remote connection
#[tauri::command]
pub async fn remote_close_connection(
    _state: State<'_, AppState>,
    app: AppHandle,
    device_id: String,
    connection_id: String,
) -> Result<(), String> {
    let device = find_device(&app, &device_id)?;
    let client = client_for(&device)?;
    client
        .port_close(&connection_id)
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Send bytes to a remote connection
#[tauri::command]
pub async fn remote_send_data(
    _state: State<'_, AppState>,
    app: AppHandle,
    device_id: String,
    connection_id: String,
    data: Vec<u8>,
) -> Result<usize, String> {
    let device = find_device(&app, &device_id)?;
    let client = client_for(&device)?;
    client
        .port_send(&connection_id, &data)
        .await
        .map_err(|e| e.to_string())
}

/// Receive data from a remote connection (blocking read with timeout)
#[tauri::command]
pub async fn remote_recv_data(
    _state: State<'_, AppState>,
    app: AppHandle,
    device_id: String,
    connection_id: String,
    timeout_ms: Option<u64>,
) -> Result<serial_cli::server::client::RemoteRecvResult, String> {
    let device = find_device(&app, &device_id)?;
    let client = client_for(&device)?;
    client
        .port_recv(&connection_id, timeout_ms.unwrap_or(1000))
        .await
        .map_err(|e| e.to_string())
}

/// List active connections on a remote daemon
#[tauri::command]
pub async fn remote_connection_list(
    _state: State<'_, AppState>,
    app: AppHandle,
    device_id: String,
) -> Result<Vec<RemoteConnectionInfo>, String> {
    let device = find_device(&app, &device_id)?;
    let client = client_for(&device)?;
    client.connection_list().await.map_err(|e| e.to_string())
}
