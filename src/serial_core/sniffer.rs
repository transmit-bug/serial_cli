//! Serial port sniffer
//!
//! This module provides serial port monitoring/sniffing capabilities.

use crate::error::{Result, SerialError};
use crate::serial_core::{PortManager, SerialConfig};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

/// Serial sniffer configuration
#[derive(Debug, Clone)]
pub struct SnifferConfig {
    /// Enable packet capture
    pub capture_packets: bool,
    /// Save captured packets to file
    pub save_to_file: bool,
    /// Output directory for captures
    pub output_dir: PathBuf,
    /// Maximum packets to capture (0 = unlimited)
    pub max_packets: usize,
    /// Include timestamps in output
    pub include_timestamps: bool,
    /// Display as hex
    pub hex_display: bool,
}

impl Default for SnifferConfig {
    fn default() -> Self {
        Self {
            capture_packets: true,
            save_to_file: false,
            output_dir: PathBuf::from("."),
            max_packets: 0,
            include_timestamps: true,
            hex_display: false,
        }
    }
}

/// Captured packet information
#[derive(Debug, Clone)]
pub struct CapturedPacket {
    /// Timestamp (UNIX epoch)
    pub timestamp: u64,
    /// Packet direction (TX/RX)
    pub direction: PacketDirection,
    /// Raw packet data
    pub data: Vec<u8>,
    /// Packet length
    pub length: usize,
}

/// Packet direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PacketDirection {
    /// Transmit (TX)
    Tx,
    /// Receive (RX)
    Rx,
}

/// Serial port sniffer
pub struct SerialSniffer {
    config: SnifferConfig,
    packets: Arc<Mutex<Vec<CapturedPacket>>>,
}

impl SerialSniffer {
    /// Create a new serial sniffer
    pub fn new(config: SnifferConfig) -> Self {
        Self {
            config,
            packets: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Start sniffing on an already-open port.
    ///
    /// The caller is responsible for opening the port via PortManager.
    /// The sniffer just monitors data flowing through the port.
    pub async fn start_sniffing(&self, port_id: &str, port_name: &str) -> Result<SnifferSession> {
        Ok(SnifferSession::new(
            port_id.to_string(),
            port_name.to_string(),
            self.packets.clone(),
            self.config.clone(),
            Arc::new(Mutex::new(true)),
        ))
    }

    /// Get all captured packets
    pub async fn get_packets(&self) -> Vec<CapturedPacket> {
        let packets = self.packets.lock().await;
        packets.clone()
    }

    /// Clear all captured packets
    pub async fn clear_packets(&self) {
        let mut packets = self.packets.lock().await;
        packets.clear();
    }

    /// Get packet count
    pub async fn packet_count(&self) -> usize {
        let packets = self.packets.lock().await;
        packets.len()
    }

    /// Save captured packets to file
    pub async fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let packets = self.packets.lock().await;

        let mut file = std::fs::File::create(path).map_err(SerialError::Io)?;

        writeln!(
            file,
            "Serial Port Capture - {}",
            chrono::Utc::now().to_rfc3339()
        )
        .map_err(SerialError::Io)?;

        for packet in packets.iter() {
            self.write_packet(&mut file, packet)?;
        }

        Ok(())
    }

    /// Write a packet to the output
    fn write_packet(&self, file: &mut std::fs::File, packet: &CapturedPacket) -> Result<()> {
        if self.config.include_timestamps {
            write!(file, "[{}] ", packet.timestamp).map_err(SerialError::Io)?;
        }

        let direction = match packet.direction {
            PacketDirection::Tx => "TX",
            PacketDirection::Rx => "RX",
        };

        writeln!(file, "{} ({} bytes)", direction, packet.length).map_err(SerialError::Io)?;

        if self.config.hex_display {
            // Display as hex
            for (i, chunk) in packet.data.chunks(16).enumerate() {
                let offset = i * 16;
                write!(file, "  {:04X}: ", offset).map_err(SerialError::Io)?;

                for (j, &byte) in chunk.iter().enumerate() {
                    write!(file, "{:02X} ", byte).map_err(SerialError::Io)?;

                    if j == 7 {
                        write!(file, " ").map_err(SerialError::Io)?;
                    }
                }

                // Pad with spaces if needed
                for _ in chunk.len()..16 {
                    write!(file, "   ").map_err(SerialError::Io)?;
                }

                // ASCII representation
                write!(file, "  |").map_err(SerialError::Io)?;

                for &byte in chunk.iter() {
                    if byte.is_ascii_graphic() || byte == b' ' {
                        write!(file, "{}", byte as char).map_err(SerialError::Io)?;
                    } else {
                        write!(file, ".").map_err(SerialError::Io)?;
                    }
                }

                writeln!(file, "|").map_err(SerialError::Io)?;
            }
        } else {
            // Display as raw bytes
            let hex: String = packet.data.iter().map(|b| format!("{:02X} ", b)).collect();
            writeln!(file, "  {}", hex).map_err(SerialError::Io)?;
        }

        writeln!(file).map_err(SerialError::Io)?;

        Ok(())
    }
}

/// Active sniffing session
#[derive(Clone)]
pub struct SnifferSession {
    #[allow(dead_code)]
    port_id: String,
    port_name: String,
    packets: Arc<Mutex<Vec<CapturedPacket>>>,
    config: SnifferConfig,
    running: Arc<Mutex<bool>>,
}

impl SnifferSession {
    /// Create a new sniffer session
    pub fn new(
        port_id: String,
        port_name: String,
        packets: Arc<Mutex<Vec<CapturedPacket>>>,
        config: SnifferConfig,
        running: Arc<Mutex<bool>>,
    ) -> Self {
        Self {
            port_id,
            port_name,
            packets,
            config,
            running,
        }
    }

    /// Capture a transmitted packet
    pub async fn capture_tx(&self, data: &[u8]) -> Result<()> {
        self.capture_packet(data, PacketDirection::Tx).await
    }

    /// Capture a received packet
    pub async fn capture_rx(&self, data: &[u8]) -> Result<()> {
        self.capture_packet(data, PacketDirection::Rx).await
    }

    /// Capture a packet
    async fn capture_packet(&self, data: &[u8], direction: PacketDirection) -> Result<()> {
        // Check if still running
        if !*self.running.lock().await {
            return Ok(());
        }

        // Check max packet limit
        if self.config.max_packets > 0 {
            let count = self.packets.lock().await.len();
            if count >= self.config.max_packets {
                return Ok(());
            }
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let packet = CapturedPacket {
            timestamp,
            direction,
            data: data.to_vec(),
            length: data.len(),
        };

        let mut packets = self.packets.lock().await;
        packets.push(packet);

        Ok(())
    }

    /// Stop sniffing
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.lock().await;
        *running = false;
        Ok(())
    }

    /// Check if still running
    pub async fn is_running(&self) -> bool {
        *self.running.lock().await
    }

    /// Get session statistics
    pub async fn stats(&self) -> SnifferStats {
        let packets = self.packets.lock().await;
        let tx_count = packets
            .iter()
            .filter(|p| p.direction == PacketDirection::Tx)
            .count();
        let rx_count = packets
            .iter()
            .filter(|p| p.direction == PacketDirection::Rx)
            .count();
        let total_bytes: usize = packets.iter().map(|p| p.length).sum();

        SnifferStats {
            port_name: self.port_name.clone(),
            total_packets: packets.len(),
            tx_packets: tx_count,
            rx_packets: rx_count,
            total_bytes,
        }
    }
}

/// Sniffer statistics
#[derive(Debug, Clone)]
pub struct SnifferStats {
    pub port_name: String,
    pub total_packets: usize,
    pub tx_packets: usize,
    pub rx_packets: usize,
    pub total_bytes: usize,
}

impl Default for SerialSniffer {
    fn default() -> Self {
        Self::new(SnifferConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sniffer_creation() {
        let config = SnifferConfig::default();
        let sniffer = SerialSniffer::new(config);

        // Note: packet_count is async, so we can't test it in a sync test
        // This is just to ensure the sniffer can be created
        assert_eq!(sniffer.config.max_packets, 0);
    }

    #[tokio::test]
    async fn test_packet_capture() {
        let config = SnifferConfig::default();
        let sniffer = SerialSniffer::new(config);

        // Create a session without actually opening a port
        // (just for testing the capture functionality)
        let session = SnifferSession::new(
            "test-id".to_string(),
            "/dev/ttyUSB0".to_string(),
            sniffer.packets.clone(),
            sniffer.config.clone(),
            Arc::new(Mutex::new(true)),
        );

        // Simulate capturing some packets
        session.capture_tx(&[0x01, 0x02, 0x03]).await.unwrap();
        session.capture_rx(&[0x04, 0x05]).await.unwrap();

        let packets = sniffer.get_packets().await;
        assert_eq!(packets.len(), 2);
        assert_eq!(packets[0].direction, PacketDirection::Tx);
        assert_eq!(packets[1].direction, PacketDirection::Rx);
    }

    #[tokio::test]
    async fn test_max_packets_limit() {
        let config = SnifferConfig {
            max_packets: 2,
            ..Default::default()
        };

        let sniffer = SerialSniffer::new(config);

        // Create a test session
        let session = SnifferSession::new(
            "test-id".to_string(),
            "/dev/ttyUSB0".to_string(),
            sniffer.packets.clone(),
            sniffer.config.clone(),
            Arc::new(Mutex::new(true)),
        );

        // Capture more than max_packets
        session.capture_tx(&[0x01]).await.unwrap();
        session.capture_tx(&[0x02]).await.unwrap();
        session.capture_tx(&[0x03]).await.unwrap(); // Should be ignored

        let packets = sniffer.get_packets().await;
        assert_eq!(packets.len(), 2); // Only 2 packets should be captured
    }

    #[tokio::test]
    async fn test_clear_packets() {
        let sniffer = SerialSniffer::new(SnifferConfig::default());

        // Create a test session
        let session = SnifferSession::new(
            "test-id".to_string(),
            "/dev/ttyUSB0".to_string(),
            sniffer.packets.clone(),
            sniffer.config.clone(),
            Arc::new(Mutex::new(true)),
        );

        session.capture_tx(&[0x01]).await.unwrap();
        assert_eq!(sniffer.packet_count().await, 1);

        sniffer.clear_packets().await;
        assert_eq!(sniffer.packet_count().await, 0);
    }

    #[tokio::test]
    async fn test_session_stats() {
        let sniffer = SerialSniffer::new(SnifferConfig::default());

        // Create a test session
        let session = SnifferSession::new(
            "test-id".to_string(),
            "/dev/ttyUSB0".to_string(),
            sniffer.packets.clone(),
            sniffer.config.clone(),
            Arc::new(Mutex::new(true)),
        );

        session.capture_tx(&[0x01, 0x02, 0x03]).await.unwrap();
        session.capture_rx(&[0x04, 0x05]).await.unwrap();

        let stats = session.stats().await;
        assert_eq!(stats.total_packets, 2);
        assert_eq!(stats.tx_packets, 1);
        assert_eq!(stats.rx_packets, 1);
        assert_eq!(stats.total_bytes, 5);
    }
}
