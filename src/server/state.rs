//! Server state management
//!
//! Provides the global server state shared across all RPC connections.

use crate::protocol::{ProtocolManager, ProtocolRegistry};
use crate::serial_core::PortManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{Mutex, RwLock};

/// Global server state (similar to Tauri's AppState)
#[derive(Clone)]
pub struct ServerState {
    /// Serial port manager (shared with CLI)
    pub port_manager: Arc<Mutex<PortManager>>,

    /// Protocol registry (shared with CLI)
    pub protocol_registry: Arc<Mutex<ProtocolRegistry>>,

    /// Protocol manager for custom protocols (shared with CLI)
    pub protocol_manager: Arc<Mutex<ProtocolManager>>,

    /// Active connections (connection_id -> ConnectionContext)
    pub connections: Arc<RwLock<HashMap<String, ConnectionContext>>>,

    /// Server configuration
    pub config: ServerConfig,
}

/// Server configuration
#[derive(Clone, Debug)]
pub struct ServerConfig {
    /// Unix socket path
    pub socket_path: Option<PathBuf>,

    /// TCP port (alternative to Unix socket)
    pub tcp_port: Option<u16>,

    /// Max concurrent connections
    pub max_connections: usize,

    /// Log file path
    pub log_path: PathBuf,

    /// Connection idle timeout (seconds)
    pub idle_timeout_secs: u64,
}

/// Connection context (per-connection state)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionContext {
    /// Unique connection ID
    pub connection_id: String,

    /// Port ID (if port is open)
    pub port_id: Option<String>,

    /// Protocol name (if protocol is set)
    pub protocol_name: Option<String>,

    /// Connection creation timestamp
    pub created_at: SystemTime,

    /// Last activity timestamp
    pub last_activity: SystemTime,
}

impl ServerState {
    /// Create a new server state
    pub async fn new(config: ServerConfig) -> Self {
        let protocol_registry = Arc::new(Mutex::new(ProtocolRegistry::new()));
        let protocol_manager =
            Arc::new(Mutex::new(ProtocolManager::new(protocol_registry.clone())));

        Self {
            port_manager: Arc::new(Mutex::new(PortManager::new())),
            protocol_registry,
            protocol_manager,
            connections: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Get connection stats
    pub async fn connection_stats(&self) -> ConnectionStats {
        let connections = self.connections.read().await;
        ConnectionStats {
            active: connections.len(),
            max: self.config.max_connections,
        }
    }

    /// Check if max connections reached
    pub async fn is_max_connections_reached(&self) -> bool {
        let connections = self.connections.read().await;
        connections.len() >= self.config.max_connections
    }

    /// Add a new connection
    pub async fn add_connection(&self, ctx: ConnectionContext) -> Result<(), crate::error::SerialError> {
        if self.is_max_connections_reached().await {
            return Err(crate::error::SerialError::Io(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Max connections reached",
            )));
        }

        let mut connections = self.connections.write().await;
        connections.insert(ctx.connection_id.clone(), ctx);
        Ok(())
    }

    /// Remove a connection
    pub async fn remove_connection(&self, connection_id: &str) -> Option<ConnectionContext> {
        let mut connections = self.connections.write().await;
        connections.remove(connection_id)
    }

    /// Update connection activity
    pub async fn update_activity(&self, connection_id: &str) {
        let mut connections = self.connections.write().await;
        if let Some(ctx) = connections.get_mut(connection_id) {
            ctx.last_activity = SystemTime::now();
        }
    }

    /// Cleanup idle connections
    pub async fn cleanup_idle_connections(&self) -> Vec<String> {
        let now = SystemTime::now();
        let timeout = std::time::Duration::from_secs(self.config.idle_timeout_secs);

        let mut connections = self.connections.write().await;
        let mut to_remove = Vec::new();

        for (id, ctx) in connections.iter() {
            if let Ok(elapsed) = now.duration_since(ctx.last_activity) {
                if elapsed > timeout {
                    to_remove.push(id.clone());
                }
            }
        }

        for id in &to_remove {
            connections.remove(id);
        }

        to_remove
    }
}

/// Connection statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionStats {
    pub active: usize,
    pub max: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            socket_path: Some(default_socket_path()),
            tcp_port: None,
            max_connections: 10,
            log_path: default_log_path(),
            idle_timeout_secs: 300, // 5 minutes
        }
    }
}

/// Get default Unix socket path
pub fn default_socket_path() -> PathBuf {
    PathBuf::from("/tmp/serial-cli.sock")
}

/// Get default log file path
pub fn default_log_path() -> PathBuf {
    if let Some(base_dirs) = directories::BaseDirs::new() {
        base_dirs
            .cache_dir()
            .join("serial_cli")
            .join("server.log")
    } else {
        PathBuf::from("/tmp/serial-cli-server.log")
    }
}
