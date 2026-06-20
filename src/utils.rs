//! Utility functions for serial CLI operations
//!
//! This module provides helper functions for common operations.

use crate::error::{Result, SerialError};
use crate::serial_core::PortManager;
use std::io::Write;
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub mod hex;
pub mod lua_conversion;

/// Auto-reconnect configuration
#[derive(Debug, Clone)]
pub struct AutoReconnectConfig {
    /// Enable auto-reconnect
    pub enabled: bool,
    /// Maximum retry attempts
    pub max_attempts: usize,
    /// Delay between attempts in milliseconds
    pub delay_ms: u64,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for AutoReconnectConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 5,
            delay_ms: 1000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Port statistics
#[derive(Debug, Clone, Default)]
pub struct PortStats {
    /// Bytes sent
    pub bytes_sent: usize,
    /// Bytes received
    pub bytes_received: usize,
    /// Packets sent
    pub packets_sent: usize,
    /// Packets received
    pub packets_received: usize,
    /// Errors count
    pub errors: usize,
    /// Connection time
    pub connected_at: Option<Instant>,
    /// Last activity time
    pub last_activity: Option<Instant>,
}

impl PortStats {
    /// Create new statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Update connection time
    pub fn mark_connected(&mut self) {
        self.connected_at = Some(Instant::now());
        self.last_activity = Some(Instant::now());
    }

    /// Update activity time
    pub fn mark_activity(&mut self) {
        self.last_activity = Some(Instant::now());
    }

    /// Record sent data
    pub fn record_sent(&mut self, bytes: usize) {
        self.bytes_sent += bytes;
        self.packets_sent += 1;
        self.mark_activity();
    }

    /// Record received data
    pub fn record_received(&mut self, bytes: usize) {
        self.bytes_received += bytes;
        self.packets_received += 1;
        self.mark_activity();
    }

    /// Record error
    pub fn record_error(&mut self) {
        self.errors += 1;
    }

    /// Get uptime duration
    pub fn uptime(&self) -> Option<Duration> {
        self.connected_at.map(|t| t.elapsed())
    }

    /// Get idle time duration
    pub fn idle_time(&self) -> Option<Duration> {
        self.last_activity.map(|t| t.elapsed())
    }

    /// Get total bytes transferred
    pub fn total_bytes(&self) -> usize {
        self.bytes_sent + self.bytes_received
    }

    /// Get total packets transferred
    pub fn total_packets(&self) -> usize {
        self.packets_sent + self.packets_received
    }

    /// Get success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        let total = self.total_packets();
        if total == 0 {
            return 1.0;
        }
        let errors = self.errors;
        1.0 - (errors as f64 / total as f64)
    }
}

/// Auto-reconnect utility
pub struct AutoReconnect {
    config: AutoReconnectConfig,
    manager: PortManager,
}

impl AutoReconnect {
    /// Create new auto-reconnect utility
    pub fn new(config: AutoReconnectConfig) -> Self {
        Self {
            config,
            manager: PortManager::new(),
        }
    }

    /// Try to open port with auto-reconnect
    pub async fn open_with_retry(
        &self,
        port_name: &str,
        config: crate::serial_core::SerialConfig,
    ) -> Result<String> {
        if !self.config.enabled {
            return self.manager.open_port(port_name, config).await;
        }

        let mut attempt = 0;
        let mut delay = self.config.delay_ms;

        loop {
            attempt += 1;

            match self.manager.open_port(port_name, config.clone()).await {
                Ok(port_id) => {
                    if attempt > 1 {
                        tracing::info!(
                            "Successfully connected to {} after {} attempts",
                            port_name,
                            attempt
                        );
                    }
                    return Ok(port_id);
                }
                Err(e) => {
                    if attempt >= self.config.max_attempts {
                        return Err(SerialError::Serial(crate::error::SerialPortError::IoError(
                            format!("Failed to connect after {} attempts: {}", attempt, e),
                        )));
                    }

                    tracing::warn!(
                        "Connection attempt {}/{} failed: {}. Retrying in {}ms...",
                        attempt,
                        self.config.max_attempts,
                        e,
                        delay
                    );

                    sleep(Duration::from_millis(delay)).await;

                    // Exponential backoff
                    delay = (delay as f64 * self.config.backoff_multiplier) as u64;
                    delay = delay.min(30000); // Cap at 30 seconds
                }
            }
        }
    }
}

/// Data format utilities
pub struct DataFormat;

impl DataFormat {
    /// Format bytes as hex string
    pub fn bytes_to_hex(data: &[u8], separator: &str) -> String {
        data.iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(separator)
    }

    /// Format bytes as hex dump
    pub fn hex_dump(data: &[u8]) -> String {
        let mut output = String::new();

        for (i, chunk) in data.chunks(16).enumerate() {
            let offset = i * 16;

            // Offset
            output.push_str(&format!("{:04X}: ", offset));

            // Hex bytes
            for (j, &byte) in chunk.iter().enumerate() {
                output.push_str(&format!("{:02X} ", byte));
                if j == 7 {
                    output.push(' ');
                }
            }

            // Padding
            for _ in chunk.len()..16 {
                output.push_str("   ");
                if chunk.len() <= 8 {
                    output.push(' ');
                }
            }

            // ASCII representation
            output.push_str("  |");
            for &byte in chunk.iter() {
                if byte.is_ascii_graphic() || byte == b' ' {
                    output.push(byte as char);
                } else {
                    output.push('.');
                }
            }
            output.push_str("|\n");
        }

        output
    }

    /// Parse hex string to bytes
    pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>> {
        let hex = hex.trim().replace([' ', '\n'], "");

        if !hex.len().is_multiple_of(2) {
            return Err(SerialError::Config(
                "Hex string must have even length".to_string(),
            ));
        }

        let mut bytes = Vec::new();
        for i in (0..hex.len()).step_by(2) {
            let byte_str = &hex[i..i + 2];
            let byte = u8::from_str_radix(byte_str, 16)
                .map_err(|_| SerialError::Config(format!("Invalid hex string: {}", byte_str)))?;
            bytes.push(byte);
        }

        Ok(bytes)
    }

    /// Escape special characters for display
    pub fn escape_bytes(data: &[u8]) -> String {
        let mut result = String::new();

        for &byte in data.iter() {
            match byte {
                b'\n' => result.push_str("\\n"),
                b'\r' => result.push_str("\\r"),
                b'\t' => result.push_str("\\t"),
                b'\\' => result.push_str("\\\\"),
                b'\"' => result.push_str("\\\""),
                _ if byte.is_ascii_graphic() || byte == b' ' => {
                    result.push(byte as char);
                }
                _ => result.push_str(&format!("\\x{:02X}", byte)),
            }
        }

        result
    }
}

/// Progress reporter for long operations
pub struct ProgressReporter {
    name: String,
    total: usize,
    current: usize,
    start_time: Instant,
    last_report: Instant,
}

impl ProgressReporter {
    /// Create new progress reporter
    pub fn new(name: String, total: usize) -> Self {
        Self {
            name,
            total,
            current: 0,
            start_time: Instant::now(),
            last_report: Instant::now(),
        }
    }

    /// Update progress
    pub fn update(&mut self, increment: usize) {
        self.current += increment;

        // Only report every 100ms or if complete
        if self.current == self.total || self.last_report.elapsed() > Duration::from_millis(100) {
            self.report();
            self.last_report = Instant::now();
        }
    }

    /// Report progress
    fn report(&self) {
        let percentage = if self.total > 0 {
            (self.current as f64 / self.total as f64 * 100.0) as u32
        } else {
            100
        };

        let elapsed = self.start_time.elapsed();
        let rate = if elapsed.as_secs() > 0 {
            self.current as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        let eta = if rate > 0.0 && self.current < self.total {
            let remaining = self.total - self.current;
            Some(Duration::from_secs_f64(remaining as f64 / rate))
        } else {
            None
        };

        let eta_str = eta
            .map(|d| format!(" in {:.1}s", d.as_secs_f64()))
            .unwrap_or_default();

        tracing::trace!(
            "\r{}: {}/{} ({}%) {:.2} ops/s{}",
            self.name,
            self.current,
            self.total,
            percentage,
            rate,
            eta_str
        );

        if self.current >= self.total {
            tracing::info!(""); // New line when complete
        }

        let _ = std::io::stdout().flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_stats() {
        let mut stats = PortStats::new();
        stats.mark_connected();
        stats.record_sent(100);
        stats.record_received(50);
        stats.record_sent(25);

        assert_eq!(stats.bytes_sent, 125);
        assert_eq!(stats.bytes_received, 50);
        assert_eq!(stats.total_bytes(), 175);
        assert_eq!(stats.total_packets(), 3); // 2 sent + 1 received = 3 total
        assert_eq!(stats.success_rate(), 1.0);
    }

    #[test]
    fn test_data_format_hex() {
        let data = vec![0x01, 0x02, 0x03, 0xFF];
        assert_eq!(DataFormat::bytes_to_hex(&data, ":"), "01:02:03:FF");
        assert_eq!(DataFormat::bytes_to_hex(&data, " "), "01 02 03 FF");
    }

    #[test]
    fn test_data_format_hex_dump() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x41, 0x42];
        let dump = DataFormat::hex_dump(&data);
        assert!(dump.contains("0000:"));
        assert!(dump.contains("01 02 03 04"));
        assert!(dump.contains("AB"));
    }

    #[test]
    fn test_hex_to_bytes() {
        let bytes = DataFormat::hex_to_bytes("01 02 03 FF").unwrap();
        assert_eq!(bytes, vec![0x01, 0x02, 0x03, 0xFF]);

        let bytes = DataFormat::hex_to_bytes("010203FF").unwrap();
        assert_eq!(bytes, vec![0x01, 0x02, 0x03, 0xFF]);
    }

    #[test]
    fn test_hex_to_bytes_empty() {
        let bytes = DataFormat::hex_to_bytes("").unwrap();
        assert!(bytes.is_empty());
    }

    #[test]
    fn test_hex_to_bytes_odd_length() {
        let result = DataFormat::hex_to_bytes("0102F");
        assert!(result.is_err());
    }

    #[test]
    fn test_hex_to_bytes_invalid_chars() {
        let result = DataFormat::hex_to_bytes("01GG03");
        assert!(result.is_err());
    }

    #[test]
    fn test_hex_to_bytes_with_mixed_case() {
        let bytes = DataFormat::hex_to_bytes("aAbB01").unwrap();
        assert_eq!(bytes, vec![0xAA, 0xBB, 0x01]);
    }

    #[test]
    fn test_escape_bytes() {
        let data = b"Hello\nWorld\t\x01";
        let escaped = DataFormat::escape_bytes(data);
        assert!(escaped.contains("Hello"));
        assert!(escaped.contains("\\n"));
        assert!(escaped.contains("\\t"));
        assert!(escaped.contains("\\x01"));
    }
}
