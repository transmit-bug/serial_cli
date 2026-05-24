// Copyright 2024 Serial CLI Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use serde::{Deserialize, Serialize};

/// Port-specific state
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
