//! Logging and tracing configuration
//!
//! This module provides centralized logging setup with support for:
//! - Environment variable configuration (RUST_LOG)
//! - JSON and pretty-print output formats
//! - Performance tracing with spans
//! - Log filtering by target/module

use tracing_subscriber::{fmt, EnvFilter};

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Log level filter (e.g., "debug", "info", "warn", "error")
    pub level: String,
    /// Output format: "pretty", "compact", "json"
    pub format: String,
    /// Enable ANSI colors
    pub with_colors: bool,
    /// Show thread names
    pub with_threads: bool,
    /// Show target/module names
    pub with_targets: bool,
    /// Show timestamps
    pub with_timestamps: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(),
            with_colors: true,
            with_threads: false,
            with_targets: true,
            with_timestamps: true,
        }
    }
}

impl LoggingConfig {
    /// Create from environment and CLI args
    pub fn from_env(verbose: bool) -> Self {
        use std::env;

        let level = env::var("RUST_LOG")
            .or_else(|_| env::var("LOG_LEVEL"))
            .unwrap_or_else(|_| {
                if verbose {
                    "debug".to_string()
                } else {
                    "info".to_string()
                }
            });

        let format = env::var("LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string());

        Self {
            level,
            format,
            with_colors: env::var("NO_COLOR").is_err(),
            with_threads: env::var("LOG_THREADS").is_ok(),
            with_targets: env::var("LOG_TARGETS").is_ok(),
            with_timestamps: env::var("LOG_NO_TIME").is_err(),
        }
    }

    /// Initialize logging with this configuration
    pub fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        let filter = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new(&self.level))
            .unwrap_or_else(|_| EnvFilter::new("info"));

        match self.format.as_str() {
            "json" => {
                fmt()
                    .with_env_filter(filter)
                    .json()
                    .with_ansi(false)
                    .with_target(true)
                    .with_thread_names(true)
                    .init();
            }
            "compact" => {
                let builder = fmt()
                    .with_env_filter(filter)
                    .compact()
                    .with_ansi(self.with_colors)
                    .with_target(self.with_targets)
                    .with_thread_names(self.with_threads);
                if !self.with_timestamps {
                    builder.without_time().init();
                } else {
                    builder.init();
                }
            }
            _ => {
                // pretty format (default)
                let builder = fmt()
                    .with_env_filter(filter)
                    .pretty()
                    .with_ansi(self.with_colors)
                    .with_target(self.with_targets)
                    .with_thread_names(self.with_threads);
                if !self.with_timestamps {
                    builder.without_time().init();
                } else {
                    builder.init();
                }
            }
        }

        Ok(())
    }
}

/// Initialize logging for CLI mode
/// - Human-readable output to stdout
/// - Respects RUST_LOG environment variable
/// - Verbose flag overrides to debug level
pub fn init_cli(verbose: bool) {
    let config = LoggingConfig::from_env(verbose);
    if let Err(e) = config.init() {
        eprintln!("Warning: Failed to initialize logging: {}", e);
    }
}

/// Initialize logging for JSON output mode
/// - Machine-readable JSON lines
/// - Includes all metadata (timestamps, targets, threads)
pub fn init_json(verbose: bool) {
    let mut config = LoggingConfig::from_env(verbose);
    config.format = "json".to_string();
    if let Err(e) = config.init() {
        eprintln!("Warning: Failed to initialize JSON logging: {}", e);
    }
}

/// Log a serial data transfer with hex dump
pub fn log_data_transfer(direction: &str, data: &[u8], port: &str) {
    let hex = data
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ");
    tracing::debug!(
        target: "serial_cli::data",
        port = %port,
        direction = %direction,
        bytes = data.len(),
        hex = %hex,
        "Data transfer"
    );
}

/// Log a protocol message with structured fields
pub fn log_protocol_message(protocol: &str, message: &str, port: &str) {
    tracing::info!(
        target: "serial_cli::protocol",
        port = %port,
        protocol = %protocol,
        "{}", message
    );
}

/// Performance tracing helper - record an operation's duration
pub fn record_operation_duration(operation: &str, start: std::time::Instant) {
    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
    tracing::debug!(
        target: "serial_cli::perf",
        operation = %operation,
        duration_ms = duration_ms,
        "Operation completed"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // Serialize tests that modify RUST_LOG to avoid parallel test pollution
    static RUST_LOG_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert_eq!(config.format, "pretty");
        assert!(config.with_colors);
        assert!(config.with_timestamps);
    }

    #[test]
    fn test_config_from_env_with_verbose() {
        let _lock = RUST_LOG_MUTEX.lock().unwrap();
        std::env::set_var("RUST_LOG", "debug");
        let config = LoggingConfig::from_env(true);
        assert_eq!(config.level, "debug");
        std::env::remove_var("RUST_LOG");
    }

    #[test]
    fn test_config_from_env_without_verbose() {
        let _lock = RUST_LOG_MUTEX.lock().unwrap();
        std::env::set_var("RUST_LOG", "warn");
        let config = LoggingConfig::from_env(false);
        assert_eq!(config.level, "warn");
        std::env::remove_var("RUST_LOG");
    }

    #[test]
    fn test_config_from_env_falls_back_without_verbose() {
        let _lock = RUST_LOG_MUTEX.lock().unwrap();
        // Without env var set, should fall back to "info"
        std::env::remove_var("RUST_LOG");
        std::env::remove_var("LOG_LEVEL");
        let config = LoggingConfig::from_env(false);
        assert_eq!(config.level, "info");
    }
}
