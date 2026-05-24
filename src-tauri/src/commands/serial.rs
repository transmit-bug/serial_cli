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

/// Start sniffing data on a port
#[tauri::command]
pub async fn start_sniffing(
    port_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Starting data sniffing for port: {}", port_id);

    // Check if already sniffing
    let mut sniffers = state.active_sniffers.lock().await;
    if sniffers.contains_key(&port_id) {
        return Err(format!("Already sniffing port: {}", port_id));
    }

    // Create or get port stats
    let mut port_stats = state.port_stats.lock().await;
    let stats = port_stats
        .entry(port_id.clone())
        .or_insert_with(|| Arc::new(PortStatsTracker::new()))
        .clone();
    drop(port_stats);

    // Create a channel to stop the sniffer
    let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel::<()>();

    // Clone the necessary data for the task
    let port_manager = state.port_manager.clone();
    let port_id_clone = port_id.clone();
    let app_clone = app.clone();
    let stats_for_task = stats.clone();

    // Spawn the sniffer task
    let task_handle = tokio::spawn(async move {
        info!("Sniffer task started for port: {}", port_id_clone);

        let mut buffer = vec![0u8; 4096];
        let mut last_activity = std::time::Instant::now();

        loop {
            // Check for stop signal
            match stop_rx.try_recv() {
                Ok(_) | Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
                    info!("Received stop signal for port: {}", port_id_clone);
                    break;
                }
                Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {}
            }

            // Try to read data from the port
            {
                let manager = port_manager.lock().await;
                if let Ok(port_handle) = manager.get_port(&port_id_clone).await {
                    let mut handle = port_handle.lock().await;

                    match handle.read(&mut buffer) {
                        Ok(bytes_read) => {
                            if bytes_read > 0 {
                                let data = buffer[..bytes_read].to_vec();
                                debug!("Received {} bytes from port {}", bytes_read, port_id_clone);
                                last_activity = std::time::Instant::now();

                                // Update stats
                                stats_for_task.bytes_received.fetch_add(
                                    bytes_read as u64,
                                    std::sync::atomic::Ordering::Relaxed,
                                );
                                stats_for_task
                                    .packets_received
                                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                stats_for_task.last_activity.store(
                                    std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_millis() as u64,
                                    std::sync::atomic::Ordering::Relaxed,
                                );

                                // Emit data-received event
                                if let Err(e) = crate::events::emitter::emit_data_received(
                                    app_clone.clone(),
                                    port_id_clone.clone(),
                                    data,
                                )
                                .await
                                {
                                    error!("Failed to emit data-received event: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            // Log error but continue trying
                            debug!("Read error on port {}: {}", port_id_clone, e);
                        }
                    }
                } else {
                    // Port might be closed, stop sniffing
                    error!("Port {} not found, stopping sniffer", port_id_clone);
                    let _ = crate::events::emitter::emit_error(
                        app_clone.clone(),
                        format!("Port {} disconnected, sniffer stopped", port_id_clone),
                    )
                    .await;
                    break;
                }
            }

            // Small delay to prevent busy-waiting
            tokio::time::sleep(Duration::from_millis(50)).await;

            // Check for timeout (no activity for 5 seconds)
            if last_activity.elapsed() > Duration::from_secs(5) {
                debug!("No activity on port {} for 5 seconds", port_id_clone);
            }
        }

        info!("Sniffer task stopped for port: {}", port_id_clone);
    });

    // Store the sniffer
    sniffers.insert(
        port_id.clone(),
        DataSniffer {
            task_handle,
            stop_tx,
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
        // Send stop signal
        let _ = sniffer.stop_tx.send(());

        // Wait for task to finish (with timeout)
        match tokio::time::timeout(Duration::from_secs(2), sniffer.task_handle).await {
            Ok(Ok(())) => {
                info!("Sniffer task stopped successfully for port: {}", port_id);
            }
            Ok(Err(e)) => {
                error!("Sniffer task error for port {}: {:?}", port_id, e);
            }
            Err(_) => {
                error!(
                    "Timeout waiting for sniffer task to stop for port: {}",
                    port_id
                );
            }
        }
    } else {
        return Err(format!("No active sniffer for port: {}", port_id));
    }

    info!("Stopped sniffing for port: {}", port_id);
    Ok(())
}
