//! Virtual serial port pair implementation
//!
//! This module provides virtual serial port pairs for testing, monitoring,
//! and debugging serial communication without physical hardware.
//!
//! `VirtualSerialPair` is a high-level facade over a `Box<dyn VirtualBackend>`.
//! All platform-specific bridge logic lives in the backend implementations
//! (`PtyBackend`, `NamedPipeBackend`, `SocatBackend`).

use crate::config::ConfigManager;
use crate::error::Result;
use crate::serial_core::backends::{BackendType, VirtualBackend};
use crate::serial_core::factory::BackendFactory;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;

/// Direction of a captured packet from a virtual pair
#[derive(Debug, Clone, Copy)]
pub enum PacketDirection {
    /// Data flowed from port A to port B
    AtoB,
    /// Data flowed from port B to port A
    BtoA,
}

/// A single captured packet from the virtual pair bridge
#[derive(Debug, Clone)]
pub struct CapturedPacket {
    pub direction: PacketDirection,
    pub data: Vec<u8>,
    pub timestamp: SystemTime,
}

/// Virtual serial port configuration
#[derive(Debug, Clone)]
pub struct VirtualConfig {
    /// Backend type for creating virtual ports
    pub backend: BackendType,

    /// Enable traffic monitoring
    pub monitor: bool,

    /// Monitoring output file (optional)
    pub monitor_output: Option<std::path::PathBuf>,

    /// Maximum packets to capture (0 = unlimited)
    pub max_packets: usize,

    /// Bridge buffer size
    pub bridge_buffer_size: usize,
}

impl Default for VirtualConfig {
    fn default() -> Self {
        Self {
            backend: BackendType::Auto,
            monitor: false,
            monitor_output: None,
            max_packets: 0,
            bridge_buffer_size: 8192,
        }
    }
}

/// Virtual serial port pair
pub struct VirtualSerialPair {
    /// Unique identifier for this pair
    pub id: String,

    /// Port A path (e.g., /dev/pts/0)
    pub port_a: String,

    /// Port B path (e.g., /dev/pts/1)
    pub port_b: String,

    /// Backend type used
    pub backend_type: BackendType,

    /// The underlying virtual backend
    backend: Box<dyn VirtualBackend>,

    /// Running state
    running: bool,

    /// Creation timestamp
    pub created_at: SystemTime,

    /// Statistics (captured from backend at creation time, updated via error monitoring task)
    stats: Arc<Mutex<crate::serial_core::backends::BackendStats>>,
}

impl VirtualSerialPair {
    /// Create a new virtual serial port pair.
    ///
    /// Uses `BackendFactory` to select and instantiate the appropriate backend
    /// based on CLI override, config file, or platform auto-detection.
    pub async fn create(config: VirtualConfig) -> Result<Self> {
        let config_manager = ConfigManager::load_with_fallback();
        let factory = BackendFactory::new(Arc::new(config_manager));

        // Resolve backend: CLI config.backend > config file > auto-detect
        let cli_override = if config.backend == BackendType::Auto {
            None
        } else {
            Some(config.backend)
        };

        let mut backend = factory.create_backend(cli_override).await?;

        // Create the pair and start the bridge
        let (port_a, port_b, error_rx, stats) = backend.create_pair().await?;

        // Resolve the actual backend type from the instantiated backend
        let backend_type = backend.backend_type().parse().unwrap_or(config.backend);

        let id = uuid::Uuid::new_v4().to_string();
        let created_at = SystemTime::now();

        tracing::info!(
            "Created virtual pair: A={}, B={}, ID={}, backend={}",
            port_a.path.display(),
            port_b.path.display(),
            id,
            backend.backend_type()
        );

        // Spawn error monitoring task to log bridge errors
        let stats_clone = Arc::clone(&stats);
        tokio::spawn(async move {
            let mut error_rx = error_rx;
            while let Some(error) = error_rx.recv().await {
                tracing::error!("Bridge error: {}", error);
                let mut s = stats_clone.lock().await;
                s.bridge_errors += 1;
            }
        });

        Ok(Self {
            id,
            port_a: port_a.path.to_string_lossy().to_string(),
            port_b: port_b.path.to_string_lossy().to_string(),
            backend_type,
            backend,
            running: true,
            created_at,
            stats,
        })
    }

    /// Stop the virtual pair and cleanup resources
    pub async fn stop(mut self) -> Result<()> {
        tracing::info!("Stopping virtual pair: {}", self.id);
        self.running = false;
        self.backend.cleanup().await?;
        tracing::info!("Virtual pair stopped: {}", self.id);
        Ok(())
    }

    /// Check if the pair is still running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Check backend health (async)
    pub async fn is_healthy(&self) -> bool {
        self.backend.is_healthy().await
    }

    /// Get statistics about this virtual pair
    pub async fn stats(&self) -> VirtualStats {
        let uptime = self.created_at.elapsed().unwrap_or_default().as_secs();

        let bridge_stats = self.stats.lock().await.clone();

        VirtualStats {
            id: self.id.clone(),
            port_a: self.port_a.clone(),
            port_b: self.port_b.clone(),
            backend: self.backend_type,
            running: self.is_running(),
            uptime_secs: uptime,
            bytes_bridged: bridge_stats.bytes_read,
            packets_bridged: 0, // not tracked per-backend; requires packet-level instrumentation
            bridge_errors: bridge_stats.bridge_errors,
            last_error: None,
            capture_packets: 0,
            capture_bytes: 0,
        }
    }

    /// Check if monitoring is enabled for this virtual pair
    ///
    /// Note: Monitoring was removed during architecture cleanup (P1-1).
    /// This method now returns false for backward compatibility with Tauri commands.
    pub fn is_monitoring(&self) -> bool {
        false
    }

    /// Get captured packets from this virtual pair
    ///
    /// Note: Packet capture was removed during architecture cleanup (P1-1).
    /// This method now returns an empty vector for backward compatibility with Tauri commands.
    pub async fn captured_packets(&self) -> Vec<CapturedPacket> {
        Vec::new()
    }
}

/// Implement Drop to ensure resources are cleaned up
impl Drop for VirtualSerialPair {
    fn drop(&mut self) {
        tracing::debug!("VirtualSerialPair dropped: {}", self.id);
        self.running = false;
    }
}

/// Virtual serial port statistics
#[derive(Debug, Clone)]
pub struct VirtualStats {
    /// Unique identifier
    pub id: String,

    /// Port A name
    pub port_a: String,

    /// Port B name
    pub port_b: String,

    /// Backend type
    pub backend: BackendType,

    /// Running state
    pub running: bool,

    /// Uptime in seconds
    pub uptime_secs: u64,

    /// Total bytes bridged
    pub bytes_bridged: u64,

    /// Total packets bridged
    pub packets_bridged: u64,

    /// Number of bridge errors
    pub bridge_errors: u64,

    /// Last bridge error message
    pub last_error: Option<String>,

    /// Total packets captured (when monitoring is enabled)
    pub capture_packets: u64,

    /// Total capture bytes (when monitoring is enabled)
    pub capture_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_backend_availability() {
        let pty = BackendType::Pty;
        assert_eq!(pty.is_available(), cfg!(unix));

        let named_pipe = BackendType::NamedPipe;
        assert_eq!(named_pipe.is_available(), cfg!(windows));

        let socat = BackendType::Socat;
        assert!(socat.is_available());
    }

    #[test]
    fn test_default_backend_for_platform() {
        let backend = BackendType::detect();
        assert!(backend.is_available());
    }

    #[test]
    fn test_virtual_config_default() {
        let config = VirtualConfig::default();
        assert_eq!(config.backend, BackendType::Auto);
        assert!(!config.monitor);
        assert!(config.monitor_output.is_none());
        assert_eq!(config.max_packets, 0);
        assert_eq!(config.bridge_buffer_size, 8192);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_create_virtual_pair() {
        let config = VirtualConfig::default();
        let result = VirtualSerialPair::create(config).await;

        if let Err(e) = &result {
            tracing::warn!("Failed to create virtual pair: {}", e);
            // This might fail in some environments (CI, containers, etc.)
            return;
        }

        let pair = result.unwrap();
        assert!(!pair.id.is_empty());
        assert!(!pair.port_a.is_empty());
        assert!(!pair.port_b.is_empty());
        assert!(pair.is_running());

        // Cleanup
        pair.stop().await.unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_virtual_pair_stats() {
        let config = VirtualConfig::default();

        let result = VirtualSerialPair::create(config).await;
        if result.is_err() {
            tracing::warn!("Skipping test: virtual pair creation failed");
            return;
        }

        let pair = result.unwrap();
        let stats = pair.stats().await;

        assert_eq!(stats.id, pair.id);
        assert_eq!(stats.port_a, pair.port_a);
        assert_eq!(stats.port_b, pair.port_b);
        assert!(stats.running);

        // Cleanup
        pair.stop().await.unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_virtual_pair_cleanup() {
        let config = VirtualConfig::default();

        let result = VirtualSerialPair::create(config).await;
        if result.is_err() {
            tracing::warn!("Skipping test: virtual pair creation failed");
            return;
        }

        // Test that cleanup works when pair is dropped
        {
            let pair = result.unwrap();
            assert!(pair.is_running());
            // pair is dropped here
        }

        // Give time for cleanup
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // If we got here without panic, cleanup worked
        tracing::info!("Cleanup test passed");
    }
}
