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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_stats_default() {
        let stats = PortStats::default();
        assert_eq!(stats.bytes_sent, 0);
        assert_eq!(stats.bytes_received, 0);
        assert_eq!(stats.packets_sent, 0);
        assert_eq!(stats.packets_received, 0);
        assert!(stats.last_activity.is_none());
    }

    #[test]
    fn test_port_stats_serialization() {
        let stats = PortStats {
            bytes_sent: 1024,
            bytes_received: 2048,
            packets_sent: 10,
            packets_received: 20,
            last_activity: Some(1234567890),
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("1024"));
        assert!(json.contains("2048"));
        assert!(json.contains("1234567890"));
    }

    #[test]
    fn test_port_stats_deserialization() {
        let json = r#"{
            "bytes_sent": 1024,
            "bytes_received": 2048,
            "packets_sent": 10,
            "packets_received": 20,
            "last_activity": 1234567890
        }"#;
        let stats: PortStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.bytes_sent, 1024);
        assert_eq!(stats.bytes_received, 2048);
        assert_eq!(stats.packets_sent, 10);
        assert_eq!(stats.packets_received, 20);
        assert_eq!(stats.last_activity, Some(1234567890));
    }

    #[test]
    fn test_port_stats_deserialization_missing_last_activity() {
        let json = r#"{"bytes_sent": 0, "bytes_received": 0, "packets_sent": 0, "packets_received": 0}"#;
        let stats: PortStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.last_activity, None);
    }

    #[test]
    fn test_serial_config_serialization() {
        let config = SerialConfig {
            baudrate: 115200,
            databits: 8,
            stopbits: 1,
            parity: "None".to_string(),
            timeout_ms: 1000,
            flow_control: "None".to_string(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SerialConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.baudrate, 115200);
        assert_eq!(deserialized.databits, 8);
    }

    #[test]
    fn test_port_status_serialization() {
        let status = PortStatus {
            id: "port-1".to_string(),
            port_name: "/dev/ttyUSB0".to_string(),
            is_open: true,
            config: Some(SerialConfig {
                baudrate: 9600,
                databits: 8,
                stopbits: 1,
                parity: "None".to_string(),
                timeout_ms: 500,
                flow_control: "None".to_string(),
            }),
            stats: PortStats::default(),
        };
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: PortStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "port-1");
        assert_eq!(deserialized.port_name, "/dev/ttyUSB0");
        assert!(deserialized.is_open);
        assert!(deserialized.config.is_some());
    }
}
