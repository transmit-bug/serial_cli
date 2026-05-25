// Copyright 2024 Serial CLI Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::state::app_state::{AppState, DataSniffer, PortStatsTracker};
use log::{debug, error, info};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use tauri::{AppHandle, State};

/// Send data to a serial port
#[tauri::command]
pub async fn send_data(
    port_id: String,
    data: Vec<u8>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let manager = state.port_manager.lock().await;
    let port_handle = manager
        .get_port(&port_id)
        .await
        .map_err(|e: serial_cli::error::SerialError| e.to_string())?;
    let mut handle = port_handle.lock().await;

    let bytes_written = handle
        .write(&data)
        .map_err(|e: serial_cli::error::SerialError| e.to_string())?;

    // Track bytes sent
    let port_stats = state.port_stats.lock().await;
    if let Some(stats) = port_stats.get(&port_id) {
        stats
            .bytes_sent
            .fetch_add(bytes_written as u64, std::sync::atomic::Ordering::Relaxed);
        stats
            .packets_sent
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        stats.last_activity.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
    }
    drop(port_stats);

    // Emit data-sent event
    if let Err(e) = crate::events::emitter::emit_data_sent(app, port_id, data).await {
        error!("Failed to emit data-sent event: {}", e);
    }

    Ok(bytes_written)
}

/// Read data from a serial port
#[tauri::command]
pub async fn read_data(
    port_id: String,
    max_bytes: usize,
    state: State<'_, AppState>,
) -> Result<Vec<u8>, String> {
    let manager = state.port_manager.lock().await;
    let port_handle = manager
        .get_port(&port_id)
        .await
        .map_err(|e: serial_cli::error::SerialError| e.to_string())?;
    let mut handle = port_handle.lock().await;
    let mut buffer = vec![0u8; max_bytes];
    let bytes_read = handle
        .read(&mut buffer)
        .map_err(|e: serial_cli::error::SerialError| e.to_string())?;
    buffer.truncate(bytes_read);
    Ok(buffer)
}

/// Start sniffing data on a port.
///
/// Uses `spawn_blocking` for the serial port read loop (the underlying
/// `serialport` API is synchronous) and an `mpsc` channel to forward data
/// to the async event loop that emits Tauri events. This eliminates the
/// previous 50ms polling — data is now processed immediately on arrival.
#[tauri::command]
pub async fn start_sniffing(
    port_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Starting data sniffing for port: {}", port_id);

    // Check if already sniffing
    let sniffers = state.active_sniffers.lock().await;
    if sniffers.contains_key(&port_id) {
        return Err(format!("Already sniffing port: {}", port_id));
    }

    // Get the port handle Arc once — no need to re-lock PortManager during reads
    let port_handle = {
        let manager = state.port_manager.lock().await;
        manager
            .get_port(&port_id)
            .await
            .map_err(|e: serial_cli::error::SerialError| e.to_string())?
    };
    drop(sniffers); // release early

    // Create or get port stats
    let mut port_stats = state.port_stats.lock().await;
    let stats = port_stats
        .entry(port_id.clone())
        .or_insert_with(|| Arc::new(PortStatsTracker::new()))
        .clone();
    drop(port_stats);

    // Data channel: blocking read → async event loop
    let (data_tx, mut data_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(256);

    // Shared stop flag
    let stop_flag = Arc::new(AtomicBool::new(false));

    // --- Blocking read task ---
    let read_stop = stop_flag.clone();
    let read_port_id = port_id.clone();
    let read_task_handle = tokio::task::spawn_blocking(move || {
        info!("Sniffer blocking-read task started for port: {}", read_port_id);
        let mut buffer = vec![0u8; 4096];

        loop {
            // Check stop signal between reads
            if read_stop.load(std::sync::atomic::Ordering::Relaxed) {
                info!("Stop signal received for port: {}", read_port_id);
                break;
            }

            // Lock the port handle, do one read, release lock
            // Lock is released after each read so writes can interleave
            {
                let mut handle = port_handle.blocking_lock();
                match handle.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        if data_tx.blocking_send(buffer[..n].to_vec()).is_err() {
                            // Channel closed — async side shut down
                            break;
                        }
                    }
                    Ok(_) => {} // 0 bytes (serialport timeout)
                    Err(e) => {
                        let msg = e.to_string();
                        if msg.contains("timed out") || msg.contains("timeout") {
                            continue;
                        }
                        if msg.contains("Broken pipe") || msg.contains("disconnected") {
                            debug!("Port {} disconnected", read_port_id);
                            let _ = data_tx.blocking_send(vec![]); // sentinel to signal disconnect
                            break;
                        }
                        debug!("Read error on port {}: {}", read_port_id, msg);
                        // Brief pause before retrying to avoid tight error loop
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
            }
        }

        info!("Sniffer blocking-read task stopped for port: {}", read_port_id);
    });

    // --- Async event loop: receive data, update stats, emit Tauri events ---
    let event_port_id = port_id.clone();
    let event_app = app.clone();
    let event_stats = stats.clone();
    let event_stop = stop_flag.clone();
    let task_handle = tokio::spawn(async move {
        while let Some(data) = data_rx.recv().await {
            if event_stop.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }

            // Empty data = disconnect sentinel from read task
            if data.is_empty() {
                let disconnect_msg = format!("Port {} disconnected", event_port_id);
                tracing::warn!("{}", disconnect_msg);
                let _ =
                    crate::events::emitter::emit_error(event_app.clone(), disconnect_msg).await;
                break;
            }

            let len = data.len() as u64;
            debug!("Received {} bytes from port {}", len, event_port_id);

            // Update stats
            event_stats
                .bytes_received
                .fetch_add(len, std::sync::atomic::Ordering::Relaxed);
            event_stats
                .packets_received
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            event_stats.last_activity.store(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                std::sync::atomic::Ordering::Relaxed,
            );

            // Emit data-received event
            if let Err(e) = crate::events::emitter::emit_data_received(
                event_app.clone(),
                event_port_id.clone(),
                data,
            )
            .await
            {
                error!("Failed to emit data-received event: {}", e);
            }
        }
    });

    // Store the sniffer
    let mut sniffers = state.active_sniffers.lock().await;
    sniffers.insert(
        port_id.clone(),
        DataSniffer {
            task_handle,
            read_task_handle,
            stop_flag,
            stats,
        },
    );

    info!("Started sniffing for port: {}", port_id);
    Ok(())
}

/// Stop sniffing data on a port
#[tauri::command]
pub async fn stop_sniffing(port_id: String, state: State<'_, AppState>) -> Result<(), String> {
    info!("Stopping data sniffing for port: {}", port_id);

    let mut sniffers = state.active_sniffers.lock().await;

    if let Some(sniffer) = sniffers.remove(&port_id) {
        // Signal both tasks to stop
        sniffer
            .stop_flag
            .store(true, std::sync::atomic::Ordering::Relaxed);

        // Wait for async event loop
        match tokio::time::timeout(Duration::from_secs(2), sniffer.task_handle).await {
            Ok(Ok(())) => {
                info!("Sniffer event task stopped for port: {}", port_id);
            }
            Ok(Err(e)) => {
                error!("Sniffer event task error for port {}: {:?}", port_id, e);
            }
            Err(_) => {
                error!("Timeout waiting for sniffer event task for port: {}", port_id);
            }
        }

        // Wait for blocking read task
        match tokio::time::timeout(Duration::from_secs(3), sniffer.read_task_handle).await {
            Ok(Ok(())) => {
                info!("Sniffer read task stopped for port: {}", port_id);
            }
            Ok(Err(e)) => {
                error!("Sniffer read task error for port {}: {:?}", port_id, e);
            }
            Err(_) => {
                error!("Timeout waiting for sniffer read task for port: {}", port_id);
            }
        }
    } else {
        return Err(format!("No active sniffer for port: {}", port_id));
    }

    info!("Stopped sniffing for port: {}", port_id);
    Ok(())
}
