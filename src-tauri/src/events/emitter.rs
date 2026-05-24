// Copyright 2024 Serial CLI Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use log::{debug, error};
use tauri::{AppHandle, Emitter, Listener};

/// Setup the event system for real-time updates
pub fn setup_event_system(app: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // Listen for data received events
    app.listen("data-received", move |event| {
        debug!("Data received event: {:?}", event);
        // Event will be handled by frontend listeners
    });

    // Listen for data sent events
    app.listen("data-sent", move |event| {
        debug!("Data sent event: {:?}", event);
        // Event will be handled by frontend listeners
    });

    // Listen for port status changes
    app.listen("port-status-changed", move |event| {
        debug!("Port status changed: {:?}", event);
    });

    // Listen for virtual port created events
    app.listen("virtual-port-created", move |event| {
        debug!("Virtual port created: {:?}", event);
    });

    // Listen for virtual port stopped events
    app.listen("virtual-port-stopped", move |event| {
        debug!("Virtual port stopped: {:?}", event);
    });

    // Listen for virtual port stats updated events
    app.listen("virtual-port-stats-updated", move |event| {
        debug!("Virtual port stats updated: {:?}", event);
    });

    // Listen for errors
    app.listen("error-occurred", move |event| {
        error!("Error occurred: {:?}", event);
    });

    Ok(())
}

/// Emit a data received event
pub async fn emit_data_received(
    app: AppHandle,
    port_id: String,
    data: Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    let payload = serde_json::json!({
        "port_id": port_id,
        "data": data,
        "timestamp": chrono::Utc::now().timestamp_millis(),
        "direction": "rx"
    });

    app.emit("data-received", payload)?;
    Ok(())
}

/// Emit a data sent event
pub async fn emit_data_sent(
    app: AppHandle,
    port_id: String,
    data: Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    let payload = serde_json::json!({
        "port_id": port_id,
        "data": data,
        "timestamp": chrono::Utc::now().timestamp_millis(),
        "direction": "tx"
    });

    app.emit("data-sent", payload)?;
    Ok(())
}

/// Emit a port status change event
pub async fn emit_port_status_changed(
    app: AppHandle,
    port_id: String,
    status: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let payload = serde_json::json!({
        "port_id": port_id,
        "status": status,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    });

    app.emit("port-status-changed", payload)?;
    Ok(())
}

/// Emit an error event
pub async fn emit_error(app: AppHandle, error: String) -> Result<(), Box<dyn std::error::Error>> {
    let payload = serde_json::json!({
        "error": error,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    });

    app.emit("error-occurred", payload)?;
    Ok(())
}

/// Emit a virtual port created event
pub async fn emit_virtual_port_created(
    app: AppHandle,
    port_id: String,
    port_info: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let payload = serde_json::json!({
        "port_id": port_id,
        "port_info": port_info,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    });

    app.emit("virtual-port-created", payload)?;
    Ok(())
}

/// Emit a virtual port stopped event
pub async fn emit_virtual_port_stopped(
    app: AppHandle,
    port_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let payload = serde_json::json!({
        "port_id": port_id,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    });

    app.emit("virtual-port-stopped", payload)?;
    Ok(())
}

/// Emit a virtual port stats updated event
pub async fn emit_virtual_port_stats_updated(
    app: AppHandle,
    port_id: String,
    stats: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let payload = serde_json::json!({
        "port_id": port_id,
        "stats": stats,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    });

    app.emit("virtual-port-stats-updated", payload)?;
    Ok(())
}
