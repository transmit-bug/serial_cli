// Copyright 2024 Serial CLI Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use serial_cli::protocol::{ProtocolManager, ProtocolRegistry};
use serial_cli::serial_core::{PortManager, VirtualSerialPair};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;

use std::sync::atomic::{AtomicU64, Ordering};

/// Per-port statistics tracked by the backend
pub struct PortStatsTracker {
    /// Bytes received (RX)
    pub bytes_received: Arc<AtomicU64>,
    /// Bytes sent (TX)
    pub bytes_sent: Arc<AtomicU64>,
    /// Packets received (RX event count)
    pub packets_received: Arc<AtomicU64>,
    /// Packets sent (TX event count)
    pub packets_sent: Arc<AtomicU64>,
    /// Last activity timestamp (Unix millis)
    pub last_activity: Arc<AtomicU64>,
}

impl PortStatsTracker {
    pub fn new() -> Self {
        Self {
            bytes_received: Arc::new(AtomicU64::new(0)),
            bytes_sent: Arc::new(AtomicU64::new(0)),
            packets_received: Arc::new(AtomicU64::new(0)),
            packets_sent: Arc::new(AtomicU64::new(0)),
            last_activity: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn snapshot(&self) -> (u64, u64, u64, u64, u64) {
        (
            self.bytes_received.load(Ordering::Relaxed),
            self.bytes_sent.load(Ordering::Relaxed),
            self.packets_received.load(Ordering::Relaxed),
            self.packets_sent.load(Ordering::Relaxed),
            self.last_activity.load(Ordering::Relaxed),
        )
    }
}

impl Default for PortStatsTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Data sniffer for monitoring serial port data
pub struct DataSniffer {
    /// Join handle for the sniffer task
    pub task_handle: JoinHandle<()>,
    /// Channel to stop the sniffer
    pub stop_tx: tokio::sync::oneshot::Sender<()>,
    /// Statistics for this port
    pub stats: Arc<PortStatsTracker>,
}

/// Global application state shared across all Tauri commands
#[derive(Clone)]
pub struct AppState {
    /// Serial port manager
    pub port_manager: Arc<Mutex<PortManager>>,
    /// Protocol registry
    pub protocol_registry: Arc<Mutex<ProtocolRegistry>>,
    /// Protocol manager for custom protocols
    pub protocol_manager: Arc<Mutex<ProtocolManager>>,
    /// Active data sniffers per port (port_id -> DataSniffer)
    pub active_sniffers: Arc<Mutex<HashMap<String, DataSniffer>>>,
    /// Port statistics (port_id -> PortStatsTracker) — survives sniffer stop
    pub port_stats: Arc<Mutex<HashMap<String, PortStatsTracker>>>,
    /// Virtual port registry (id -> VirtualSerialPair)
    pub virtual_port_registry: Arc<RwLock<HashMap<String, VirtualSerialPair>>>,
    /// Directory for storing custom protocol files
    pub protocols_dir: Option<PathBuf>,
}

impl AppState {
    /// Create a new application state
    pub async fn new() -> Self {
        let protocol_registry = Arc::new(Mutex::new(ProtocolRegistry::new()));
        let protocol_manager =
            Arc::new(Mutex::new(ProtocolManager::new(protocol_registry.clone())));

        // Set up protocols directory
        let protocols_dir = dirs::data_local_dir().map(|mut p| {
            p.push("serial-cli");
            p.push("protocols");
            p
        });

        Self {
            port_manager: Arc::new(Mutex::new(PortManager::new())),
            protocol_registry,
            protocol_manager,
            active_sniffers: Arc::new(Mutex::new(HashMap::new())),
            port_stats: Arc::new(Mutex::new(HashMap::new())),
            virtual_port_registry: Arc::new(RwLock::new(HashMap::new())),
            protocols_dir,
        }
    }
}
