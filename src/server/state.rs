//! Server state management
//!
//! Provides the global server state shared across all RPC connections.

use crate::script::ScriptManager;
use crate::serial_core::PortManager;
use crate::state_factory::CoreManagers;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{broadcast, Mutex, RwLock};

/// Data push event for subscribed clients
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataPushEvent {
    /// Connection ID this event belongs to
    pub connection_id: String,
    /// Raw data received (hex encoded)
    pub data_hex: String,
    /// Number of bytes received
    pub bytes_read: usize,
    /// Timestamp when data was received
    pub timestamp: u64,
}

/// Global server state (similar to Tauri's AppState)
#[derive(Clone)]
pub struct ServerState {
    /// Serial port manager (shared with CLI)
    pub port_manager: Arc<Mutex<PortManager>>,

    /// Unified script manager (replaces protocol lifecycle)
    pub script_manager: Arc<Mutex<ScriptManager>>,

    /// Active connections (connection_id -> ConnectionContext)
    pub connections: Arc<RwLock<HashMap<String, ConnectionContext>>>,

    /// Server configuration
    pub config: ServerConfig,

    /// Total RPC requests processed
    pub total_requests: Arc<AtomicU64>,
    /// Total RPC errors
    pub total_errors: Arc<AtomicU64>,

    /// Broadcast channel for data push notifications
    pub data_push_tx: broadcast::Sender<DataPushEvent>,
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

    /// Whether this connection is subscribed to data push notifications
    #[serde(default)]
    pub subscribed: bool,
}

impl ServerState {
    /// Create a new server state
    pub async fn new(config: ServerConfig) -> Self {
        let core = CoreManagers::new();
        let (data_push_tx, _) = broadcast::channel::<DataPushEvent>(100);

        Self {
            port_manager: core.port_manager,
            script_manager: core.script_manager,
            connections: Arc::new(RwLock::new(HashMap::new())),
            config,
            total_requests: Arc::new(AtomicU64::new(0)),
            total_errors: Arc::new(AtomicU64::new(0)),
            data_push_tx,
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
    pub async fn add_connection(
        &self,
        ctx: ConnectionContext,
    ) -> Result<(), crate::error::SerialError> {
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
        base_dirs.cache_dir().join("serial_cli").join("server.log")
    } else {
        PathBuf::from("/tmp/serial-cli-server.log")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn create_test_connection(id: &str) -> ConnectionContext {
        ConnectionContext {
            connection_id: id.to_string(),
            port_id: None,
            protocol_name: None,
            created_at: SystemTime::now(),
            last_activity: SystemTime::now(),
            subscribed: false,
        }
    }

    #[tokio::test]
    async fn test_max_connections_limit() {
        let config = ServerConfig {
            max_connections: 2,
            ..Default::default()
        };
        let state = ServerState::new(config).await;

        // First connection should succeed
        let ctx1 = create_test_connection("conn1");
        assert!(state.add_connection(ctx1).await.is_ok());

        // Second connection should succeed
        let ctx2 = create_test_connection("conn2");
        assert!(state.add_connection(ctx2).await.is_ok());

        // Third connection should fail (max reached)
        let ctx3 = create_test_connection("conn3");
        assert!(state.add_connection(ctx3).await.is_err());

        // Verify we have exactly 2 connections
        let stats = state.connection_stats().await;
        assert_eq!(stats.active, 2);
        assert_eq!(stats.max, 2);
    }

    #[tokio::test]
    async fn test_connection_add_remove() {
        let config = ServerConfig::default();
        let state = ServerState::new(config).await;

        let ctx = create_test_connection("test_conn");

        // Add connection
        assert!(state.add_connection(ctx.clone()).await.is_ok());

        // Verify it exists
        let connections = state.connections.read().await;
        assert!(connections.contains_key("test_conn"));
        drop(connections);

        // Remove connection
        let removed = state.remove_connection("test_conn").await;
        assert!(removed.is_some());

        // Verify it's gone
        let connections = state.connections.read().await;
        assert!(!connections.contains_key("test_conn"));
    }

    #[tokio::test]
    async fn test_idle_connection_cleanup() {
        let config = ServerConfig {
            max_connections: 10,
            idle_timeout_secs: 1, // 1 second timeout for testing
            ..Default::default()
        };
        let state = ServerState::new(config).await;

        // Add an old connection (2 seconds ago)
        let mut old_ctx = create_test_connection("old_conn");
        old_ctx.last_activity = SystemTime::now()
            .checked_sub(std::time::Duration::from_secs(2))
            .unwrap();

        // Add a recent connection
        let recent_ctx = create_test_connection("recent_conn");

        assert!(state.add_connection(old_ctx).await.is_ok());
        assert!(state.add_connection(recent_ctx).await.is_ok());

        // Run cleanup
        let removed = state.cleanup_idle_connections().await;

        // Only the old connection should be removed
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0], "old_conn");

        // Verify only recent connection remains
        let connections = state.connections.read().await;
        assert_eq!(connections.len(), 1);
        assert!(connections.contains_key("recent_conn"));
    }

    #[tokio::test]
    async fn test_update_activity() {
        let config = ServerConfig::default();
        let state = ServerState::new(config).await;

        let ctx = create_test_connection("test_conn");
        let original_activity = ctx.last_activity;

        assert!(state.add_connection(ctx).await.is_ok());

        // Wait a bit
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Update activity
        state.update_activity("test_conn").await;

        // Verify activity was updated
        let connections = state.connections.read().await;
        let updated_ctx = connections.get("test_conn").unwrap();
        assert!(updated_ctx.last_activity > original_activity);
    }
}
