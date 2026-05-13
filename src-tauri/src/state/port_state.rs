// Copyright 2024 Serial CLI Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Port-specific state
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortStatus {
    /// Port UUID
    pub id: String,
    /// Port name (e.g., COM1, /dev/ttyUSB0)
    pub port_name: String,
    /// Whether the port is currently open
    pub is_open: bool,
    /// Current configuration
    pub config: Option<SerialConfig>,
    /// Statistics
    pub stats: PortStats,
}

/// Serial port configuration
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialConfig {
    pub baudrate: u32,
    pub databits: u8,
    pub stopbits: u8,
    pub parity: String,
    pub timeout_ms: u64,
    pub flow_control: String,
}

/// Port statistics
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PortStats {
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Packets sent
    pub packets_sent: u64,
    /// Packets received
    pub packets_received: u64,
    /// Last activity timestamp
    pub last_activity: Option<u64>,
}

/// Port state manager
/// GUI API stub — awaiting frontend integration
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PortStateManager {
    ports: HashMap<String, PortStatus>,
}

impl PortStateManager {
    /// Create a new port state manager
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            ports: HashMap::new(),
        }
    }

    /// Add or update a port
    #[allow(dead_code)]
    pub fn upsert_port(&mut self, port: PortStatus) {
        self.ports.insert(port.id.clone(), port);
    }

    /// Remove a port
    #[allow(dead_code)]
    pub fn remove_port(&mut self, id: &str) {
        self.ports.remove(id);
    }

    /// Get a port by ID
    #[allow(dead_code)]
    pub fn get_port(&self, id: &str) -> Option<&PortStatus> {
        self.ports.get(id)
    }

    /// Get all ports
    #[allow(dead_code)]
    pub fn get_all_ports(&self) -> Vec<&PortStatus> {
        self.ports.values().collect()
    }
}

impl Default for PortStateManager {
    fn default() -> Self {
        Self::new()
    }
}
