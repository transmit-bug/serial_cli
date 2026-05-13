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

/// Data sniffer for monitoring serial port data
pub struct DataSniffer {
    /// Join handle for the sniffer task
    pub task_handle: JoinHandle<()>,
    /// Channel to stop the sniffer
    pub stop_tx: tokio::sync::oneshot::Sender<()>,
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
            virtual_port_registry: Arc::new(RwLock::new(HashMap::new())),
            protocols_dir,
        }
    }
}
