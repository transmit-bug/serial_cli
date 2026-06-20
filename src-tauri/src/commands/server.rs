// Copyright 2024 Serial CLI Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Embedded server commands for GUI control panel

use crate::state::app_state::{AppState, RunningEmbeddedServer};
use serial_cli::server::ServerConfig;
use std::sync::Arc;
use tauri::State;
use tokio_util::sync::CancellationToken;

/// Server status response
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct ServerStatus {
    pub running: bool,
    pub socket_path: String,
    pub started_at: i64,
    #[serde(default)]
    pub active_connections: usize,
    #[serde(default)]
    pub total_requests: u64,
    #[serde(default)]
    pub total_errors: u64,
    #[serde(default)]
    pub connections: Vec<ConnectionInfo>,
}

/// Connection information
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectionInfo {
    pub connection_id: String,
    pub port_id: Option<String>,
    pub protocol: Option<String>,
    pub created_at: u64,
    pub subscribed: bool,
}

/// Start the embedded JSON-RPC server
#[tauri::command]
pub async fn start_server(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<ServerStatus, String> {
    // Check if already running
    {
        let server_state = state.embedded_server.lock().await;
        if let Some(ref running) = *server_state {
            return Ok(ServerStatus {
                running: true,
                socket_path: running.socket_path.clone(),
                started_at: running.started_at,
                ..Default::default()
            });
        }
    }

    // Create server configuration
    let config = ServerConfig::default();
    let socket_path = config
        .socket_path
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    // Create server state sharing the same PortManager and ScriptManager
    let server_state = {
        let port_manager = state.port_manager.clone();
        let script_manager = state.script_manager.clone();
        let (data_push_tx, _) = tokio::sync::broadcast::channel(100);

        serial_cli::server::ServerState {
            port_manager,
            script_manager,
            connections: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            config: config.clone(),
            total_requests: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            total_errors: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            data_push_tx,
        }
    };

    // Create cancellation token for graceful shutdown
    let cancel_token = CancellationToken::new();

    // Spawn the server listener task
    let socket_path_clone = config
        .socket_path
        .clone()
        .ok_or_else(|| "Socket path not configured".to_string())?;
    let listener_state = server_state.clone();
    let listener_token = cancel_token.clone();

    let listener_handle = tokio::spawn(async move {
        if let Err(e) = serial_cli::server::listener::run_socket_server(
            listener_state,
            socket_path_clone,
            listener_token,
        )
        .await
        {
            tracing::error!("Server listener error: {}", e);
        }
    });

    // Spawn idle cleanup task
    let cleanup_state = server_state.clone();
    let cleanup_token = cancel_token.clone();
    serial_cli::server::listener::spawn_idle_cleanup_task(cleanup_state, cleanup_token);

    let started_at = chrono::Utc::now().timestamp();

    // Store running state
    {
        let mut server_state_lock = state.embedded_server.lock().await;
        *server_state_lock = Some(RunningEmbeddedServer {
            socket_path: socket_path.clone(),
            started_at,
            cancel_token,
            listener_handle,
            server_state,
        });
    }

    // Emit event
    if let Err(e) =
        crate::events::emitter::emit_server_status_changed(app, true, socket_path.clone()).await
    {
        tracing::warn!("Failed to emit server-status-changed event: {}", e);
    }

    Ok(ServerStatus {
        running: true,
        socket_path,
        started_at,
        ..Default::default()
    })
}

/// Stop the embedded server
#[tauri::command]
pub async fn stop_server(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut server_state = state.embedded_server.lock().await;

    if let Some(running) = server_state.take() {
        // Cancel the server task
        running.cancel_token.cancel();

        // Wait for listener to finish (with timeout)
        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            running.listener_handle,
        )
        .await;

        // Emit event
        if let Err(e) =
            crate::events::emitter::emit_server_status_changed(app, false, String::new()).await
        {
            tracing::warn!("Failed to emit server-status-changed event: {}", e);
        }

        tracing::info!("Server stopped");
    }

    Ok(())
}

/// Get server status
#[tauri::command]
pub async fn get_server_status(state: State<'_, AppState>) -> Result<ServerStatus, String> {
    let server_state = state.embedded_server.lock().await;

    if let Some(ref running) = *server_state {
        // Get connection stats
        let connections = running.server_state.connections.read().await;
        let active_connections = connections.len();
        let connection_list: Vec<ConnectionInfo> = connections
            .values()
            .map(|ctx| ConnectionInfo {
                connection_id: ctx.connection_id.clone(),
                port_id: ctx.port_id.clone(),
                protocol: ctx.protocol_name.clone(),
                created_at: ctx
                    .created_at
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                subscribed: ctx.subscribed,
            })
            .collect();
        drop(connections);

        let total_requests = running
            .server_state
            .total_requests
            .load(std::sync::atomic::Ordering::Relaxed);
        let total_errors = running
            .server_state
            .total_errors
            .load(std::sync::atomic::Ordering::Relaxed);

        Ok(ServerStatus {
            running: true,
            socket_path: running.socket_path.clone(),
            started_at: running.started_at,
            active_connections,
            total_requests,
            total_errors,
            connections: connection_list,
        })
    } else {
        Ok(ServerStatus {
            running: false,
            ..Default::default()
        })
    }
}
