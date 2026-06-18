//! Configuration management
//!
//! This module handles loading and managing configuration from TOML files.

use crate::error::{Result, SerialError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Global configuration manager
#[derive(Clone)]
pub struct ConfigManager {
    config: Arc<RwLock<Config>>,
    config_path: Option<PathBuf>,
}

impl ConfigManager {
    /// Create a new config manager with default configuration
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(Config::default())),
            config_path: None,
        }
    }

    /// Load configuration from a specific path
    pub fn load_from_path(path: &Path) -> Result<Self> {
        let config = load_config(path)?;
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path: Some(path.to_path_buf()),
        })
    }

    /// Load configuration with fallback
    pub fn load_with_fallback() -> Self {
        let config = load_config_with_fallback();
        Self {
            config: Arc::new(RwLock::new(config)),
            config_path: get_config_file_path(),
        }
    }

    /// Get the current configuration
    pub fn get(&self) -> Config {
        self.config.read().unwrap().clone()
    }

    /// Update a configuration value by key
    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        let mut config = self.config.write().unwrap();

        // Parse the key (e.g., "serial.baudrate", "logging.level")
        let parts: Vec<&str> = key.split('.').collect();

        match parts.as_slice() {
            ["serial", "baudrate"] => {
                let baudrate = value
                    .parse::<u32>()
                    .map_err(|_| SerialError::Config(format!("Invalid baudrate: {}", value)))?;
                config.serial.baudrate = baudrate;
            }
            ["serial", "databits"] => {
                let databits = value
                    .parse::<u8>()
                    .map_err(|_| SerialError::Config(format!("Invalid databits: {}", value)))?;
                config.serial.databits = databits;
            }
            ["serial", "stopbits"] => {
                let stopbits = value
                    .parse::<u8>()
                    .map_err(|_| SerialError::Config(format!("Invalid stopbits: {}", value)))?;
                config.serial.stopbits = stopbits;
            }
            ["serial", "parity"] => {
                config.serial.parity = value.to_string();
            }
            ["serial", "timeout_ms"] => {
                let timeout = value
                    .parse::<u64>()
                    .map_err(|_| SerialError::Config(format!("Invalid timeout: {}", value)))?;
                config.serial.timeout_ms = timeout;
            }
            ["logging", "level"] => {
                config.logging.level = value.to_string();
            }
            ["logging", "format"] => {
                config.logging.format = value.to_string();
            }
            ["logging", "file"] => {
                config.logging.file = value.to_string();
            }
            ["lua", "memory_limit_mb"] => {
                let limit = value
                    .parse::<usize>()
                    .map_err(|_| SerialError::Config(format!("Invalid memory limit: {}", value)))?;
                config.lua.memory_limit_mb = limit;
            }
            ["lua", "timeout_seconds"] => {
                let timeout = value
                    .parse::<u64>()
                    .map_err(|_| SerialError::Config(format!("Invalid timeout: {}", value)))?;
                config.lua.timeout_seconds = timeout;
            }
            ["lua", "enable_sandbox"] => {
                let enable = value
                    .parse::<bool>()
                    .map_err(|_| SerialError::Config(format!("Invalid boolean: {}", value)))?;
                config.lua.enable_sandbox = enable;
            }
            ["output", "json_pretty"] => {
                let pretty = value
                    .parse::<bool>()
                    .map_err(|_| SerialError::Config(format!("Invalid boolean: {}", value)))?;
                config.output.json_pretty = pretty;
            }
            ["output", "show_timestamp"] => {
                let show = value
                    .parse::<bool>()
                    .map_err(|_| SerialError::Config(format!("Invalid boolean: {}", value)))?;
                config.output.show_timestamp = show;
            }
            ["virtual", "backend"] => {
                config.virtual_ports.backend = value.to_string();
            }
            ["virtual", "monitor"] => {
                let monitor = value
                    .parse::<bool>()
                    .map_err(|_| SerialError::Config(format!("Invalid boolean: {}", value)))?;
                config.virtual_ports.monitor = monitor;
            }
            ["virtual", "monitor_format"] => {
                config.virtual_ports.monitor_format = value.to_string();
            }
            ["virtual", "auto_cleanup"] => {
                let cleanup = value
                    .parse::<bool>()
                    .map_err(|_| SerialError::Config(format!("Invalid boolean: {}", value)))?;
                config.virtual_ports.auto_cleanup = cleanup;
            }
            ["virtual", "max_packets"] => {
                let max = value
                    .parse::<usize>()
                    .map_err(|_| SerialError::Config(format!("Invalid max packets: {}", value)))?;
                config.virtual_ports.max_packets = max;
            }
            ["virtual", "bridge_buffer_size"] => {
                let size = value
                    .parse::<usize>()
                    .map_err(|_| SerialError::Config(format!("Invalid buffer size: {}", value)))?;
                config.virtual_ports.bridge_buffer_size = size;
            }
            ["virtual", "bridge_poll_interval_ms"] => {
                let interval = value.parse::<u64>().map_err(|_| {
                    SerialError::Config(format!("Invalid poll interval: {}", value))
                })?;
                config.virtual_ports.bridge_poll_interval_ms = interval;
            }
            _ => {
                return Err(SerialError::Config(format!(
                    "Unknown configuration key: {}",
                    key
                )));
            }
        }

        Ok(())
    }

    /// Get the virtual backend type from config
    pub fn get_virtual_backend_type(&self) -> crate::serial_core::backends::BackendType {
        self.config
            .read()
            .unwrap()
            .virtual_ports
            .backend
            .parse()
            .unwrap_or(crate::serial_core::backends::BackendType::Auto)
    }

    /// Add a custom protocol to the configuration
    ///
    /// Returns error if a protocol with the same name already exists.
    pub fn add_custom_protocol(&self, name: String, path: PathBuf) -> Result<()> {
        let mut config = self.config.write().unwrap();
        if config.protocols.custom.contains_key(&name) {
            return Err(SerialError::Config(format!(
                "Protocol '{}' already exists. Use 'protocol reload' to update, or 'protocol unload' first.",
                name
            )));
        }
        config.protocols.custom.insert(
            name.clone(),
            CustomProtocolConfig {
                name,
                path,
                version: 1,
                loaded_at: Some(chrono::Local::now().to_rfc3339()),
            },
        );
        Ok(())
    }

    /// Update an existing custom protocol (for reload). Atomic single-lock operation.
    pub fn update_custom_protocol(&self, name: String, path: PathBuf) -> Result<()> {
        let mut config = self.config.write().unwrap();
        let entry =
            config.protocols.custom.get_mut(&name).ok_or_else(|| {
                SerialError::Config(format!("Custom protocol not found: {}", name))
            })?;
        entry.path = path;
        entry.version += 1;
        entry.loaded_at = Some(chrono::Local::now().to_rfc3339());
        Ok(())
    }

    /// Remove a custom protocol from the configuration
    pub fn remove_custom_protocol(&self, name: &str) -> Result<()> {
        let mut config = self.config.write().unwrap();
        if config.protocols.custom.remove(name).is_none() {
            return Err(SerialError::Config(format!(
                "Custom protocol not found: {}",
                name
            )));
        }
        Ok(())
    }

    /// Get a custom protocol by name
    pub fn get_custom_protocol(&self, name: &str) -> Option<CustomProtocolConfig> {
        let config = self.config.read().unwrap();
        config.protocols.custom.get(name).cloned()
    }

    /// Check if protocol hot-reload is enabled
    pub fn is_hot_reload_enabled(&self) -> bool {
        let config = self.config.read().unwrap();
        config.protocols.hot_reload
    }

    /// Save configuration to file
    pub fn save(&self, path: Option<&Path>) -> Result<()> {
        let config = self.config.read().unwrap();
        let default_path = PathBuf::from(".serial-cli.toml");
        let save_path = path.unwrap_or_else(|| self.config_path.as_ref().unwrap_or(&default_path));

        // Create parent directories if they don't exist
        if let Some(parent) = save_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    SerialError::Config(format!("Failed to create config directory: {}", e))
                })?;
            }
        }

        // Serialize to TOML
        let toml_str = toml::to_string_pretty(&*config)
            .map_err(|e| SerialError::Config(format!("Failed to serialize config: {}", e)))?;

        // Write to file
        fs::write(save_path, toml_str)
            .map_err(|e| SerialError::Config(format!("Failed to write config file: {}", e)))?;

        tracing::info!("Configuration saved to: {}", save_path.display());
        Ok(())
    }

    /// Reset configuration to defaults
    pub fn reset(&self) -> Result<()> {
        let mut config = self.config.write().unwrap();
        *config = Config::default();
        tracing::info!("Configuration reset to defaults");
        Ok(())
    }

    /// Validate the current configuration
    pub fn validate(&self) -> Result<()> {
        let config = self.config.read().unwrap();

        // Validate serial configuration
        if config.serial.baudrate == 0 {
            return Err(SerialError::Config("Baudrate cannot be zero".to_string()));
        }

        if config.serial.databits < 5 || config.serial.databits > 8 {
            return Err(SerialError::Config(
                "Databits must be between 5 and 8".to_string(),
            ));
        }

        if config.serial.stopbits < 1 || config.serial.stopbits > 2 {
            return Err(SerialError::Config("Stopbits must be 1 or 2".to_string()));
        }

        // Validate parity
        match config.serial.parity.to_lowercase().as_str() {
            "none" | "odd" | "even" => {}
            _ => {
                return Err(SerialError::Config(
                    "Parity must be 'none', 'odd', or 'even'".to_string(),
                ))
            }
        }

        // Validate logging level
        match config.logging.level.to_lowercase().as_str() {
            "error" | "warn" | "info" | "debug" | "trace" => {}
            _ => return Err(SerialError::Config("Invalid logging level".to_string())),
        }

        Ok(())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the actual config file path that would be used
fn get_config_file_path() -> Option<PathBuf> {
    if Path::new(".serial-cli.toml").exists() {
        return Some(PathBuf::from(".serial-cli.toml"));
    }

    let global_path = get_global_config_path()?;
    if global_path.exists() {
        Some(global_path)
    } else {
        None
    }
}

/// Connection preset for quick serial port configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPreset {
    pub name: String,
    pub port_name: String,
    pub baudrate: u32,
    pub databits: u8,
    pub stopbits: u8,
    pub parity: String,
    pub flow_control: String,
    pub timeout_ms: u64,
}

/// Configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Serial port configuration
    pub serial: SerialConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Lua runtime configuration
    pub lua: LuaConfig,
    /// Output configuration
    pub output: OutputConfig,
    /// Protocol configuration
    #[serde(default)]
    pub protocols: ProtocolsConfig,
    /// Virtual serial port configuration
    #[serde(default)]
    pub virtual_ports: VirtualPortsConfig,
    /// Display / UI configuration
    #[serde(default)]
    pub display: DisplayConfig,
    /// Saved connection presets
    #[serde(default)]
    pub connection_presets: Vec<ConnectionPreset>,
}

/// Serial port configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialConfig {
    /// Default baud rate
    pub baudrate: u32,
    /// Default data bits
    pub databits: u8,
    /// Default stop bits
    pub stopbits: u8,
    /// Default parity
    pub parity: String,
    /// Default timeout in milliseconds
    pub timeout_ms: u64,
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self {
            baudrate: 115200,
            databits: 8,
            stopbits: 1,
            parity: "none".to_string(),
            timeout_ms: 1000,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (error, warn, info, debug, trace)
    pub level: String,
    /// Log format (text or json)
    pub format: String,
    /// Log file path (empty for stdout)
    pub file: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "text".to_string(),
            file: String::new(),
        }
    }
}

/// Lua runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LuaConfig {
    /// Memory limit in MB
    pub memory_limit_mb: usize,
    /// Timeout in seconds
    pub timeout_seconds: u64,
    /// Enable sandbox mode
    pub enable_sandbox: bool,
}

impl Default for LuaConfig {
    fn default() -> Self {
        Self {
            memory_limit_mb: 128,
            timeout_seconds: 300,
            enable_sandbox: true,
        }
    }
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Pretty print JSON
    pub json_pretty: bool,
    /// Show timestamps
    pub show_timestamp: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            json_pretty: true,
            show_timestamp: true,
        }
    }
}

/// Custom protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomProtocolConfig {
    /// Protocol name
    pub name: String,
    /// Path to Lua script file
    pub path: PathBuf,
    /// Protocol version
    #[serde(default)]
    pub version: u64,
    /// Load timestamp
    #[serde(default)]
    pub loaded_at: Option<String>,
}

/// Protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProtocolsConfig {
    /// Custom protocols
    #[serde(default)]
    pub custom: HashMap<String, CustomProtocolConfig>,

    /// Enable hot-reload for protocol scripts
    #[serde(default)]
    pub hot_reload: bool,
}

/// Virtual serial port configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualPortsConfig {
    /// Default backend type (pty/socat/namedpipe)
    pub backend: String,

    /// Enable monitoring by default
    pub monitor: bool,

    /// Monitoring output format (hex/raw)
    pub monitor_format: String,

    /// Auto-cleanup on exit
    pub auto_cleanup: bool,

    /// Maximum packets to capture (0 = unlimited)
    pub max_packets: usize,

    /// Buffer size for bridge (bytes)
    pub bridge_buffer_size: usize,

    /// Bridge poll interval (milliseconds)
    pub bridge_poll_interval_ms: u64,
}

impl Default for VirtualPortsConfig {
    fn default() -> Self {
        Self {
            backend: "pty".to_string(),
            monitor: false,
            monitor_format: "hex".to_string(),
            auto_cleanup: true,
            max_packets: 0,
            bridge_buffer_size: 8192,
            bridge_poll_interval_ms: 10,
        }
    }
}

/// Display / UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// UI theme (dark / light)
    pub theme: String,
    /// Default data display format (hex / ascii / mixed)
    pub format: String,
    /// Maximum packets to keep in display buffer (0 = unlimited)
    pub max_packets: usize,
    /// Show timestamps in data view
    pub show_timestamp: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            format: "mixed".to_string(),
            max_packets: 10000,
            show_timestamp: true,
        }
    }
}

/// Load configuration from a file
pub fn load_config(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| SerialError::Config(format!("Failed to read config file: {}", e)))?;

    let config: Config = toml::from_str(&content)
        .map_err(|e| SerialError::Config(format!("Failed to parse config: {}", e)))?;

    Ok(config)
}

/// Get global config directory path
pub fn get_global_config_dir() -> Option<PathBuf> {
    let config_dir = if cfg!(windows) {
        std::env::var("APPDATA")
            .ok()
            .map(|p| PathBuf::from(p).join("serial-cli"))
    } else {
        directories::BaseDirs::new().map(|dirs| PathBuf::from(dirs.config_dir()).join("serial-cli"))
    };

    config_dir
}

/// Get global config file path
pub fn get_global_config_path() -> Option<PathBuf> {
    get_global_config_dir().map(|dir| dir.join("config.toml"))
}

/// Load configuration with fallback to defaults
pub fn load_config_with_fallback() -> Config {
    // Try project-level config first
    if let Ok(config) = load_config(Path::new(".serial-cli.toml")) {
        return config;
    }

    // Try global config
    if let Some(path) = get_global_config_path() {
        if path.exists() {
            if let Ok(config) = load_config(&path) {
                return config;
            }
        }
    }

    // Return default config
    Config::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.serial.baudrate, 115200);
        assert_eq!(config.serial.databits, 8);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_serialize_config() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("baudrate"));
    }

    #[test]
    fn test_deserialize_config() {
        let toml_str = r#"
            [serial]
            baudrate = 9600
            databits = 7
            stopbits = 2
            parity = "even"
            timeout_ms = 2000

            [logging]
            level = "debug"
            format = "json"
            file = "/tmp/test.log"

            [lua]
            memory_limit_mb = 256
            timeout_seconds = 60
            enable_sandbox = false

            [output]
            json_pretty = false
            show_timestamp = false
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.serial.baudrate, 9600);
        assert_eq!(config.serial.databits, 7);
        assert_eq!(config.serial.stopbits, 2);
        assert_eq!(config.serial.parity, "even");
        assert_eq!(config.logging.level, "debug");
        assert!(!config.lua.enable_sandbox);
    }

    #[test]
    fn test_config_from_invalid_toml() {
        let result = load_config(Path::new("nonexistent.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_config_malformed_content() {
        let dir = std::env::temp_dir();
        let path = dir.join("malformed_config.toml");
        std::fs::write(&path, "this is not [[valid toml {{{").unwrap();
        let result = load_config(&path);
        assert!(result.is_err());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_config_set_valid_keys() {
        let manager = ConfigManager::new();
        manager.set("serial.baudrate", "9600").unwrap();
        manager.set("serial.databits", "7").unwrap();
        manager.set("serial.parity", "even").unwrap();
        manager.set("logging.level", "debug").unwrap();
        manager.set("lua.memory_limit_mb", "256").unwrap();
        manager.set("output.json_pretty", "false").unwrap();

        let config = manager.get();
        assert_eq!(config.serial.baudrate, 9600);
        assert_eq!(config.serial.parity, "even");
        assert_eq!(config.logging.level, "debug");
        assert_eq!(config.lua.memory_limit_mb, 256);
        assert!(!config.output.json_pretty);
    }

    #[test]
    fn test_config_set_invalid_values() {
        let manager = ConfigManager::new();
        assert!(manager.set("serial.baudrate", "not_a_number").is_err());
        assert!(manager.set("lua.memory_limit_mb", "abc").is_err());
        assert!(manager.set("output.json_pretty", "yes").is_err());
    }

    #[test]
    fn test_config_set_invalid_databits_out_of_range() {
        let manager = ConfigManager::new();
        // 99 is a valid u8, so set() succeeds — validation catches it
        manager.set("serial.databits", "99").unwrap();
        assert!(manager.validate().is_err());
    }

    #[test]
    fn test_config_set_unknown_key() {
        let manager = ConfigManager::new();
        assert!(manager.set("unknown.key", "value").is_err());
    }

    #[test]
    fn test_config_reset() {
        let manager = ConfigManager::new();
        manager.set("serial.baudrate", "9600").unwrap();
        manager.reset().unwrap();
        let config = manager.get();
        assert_eq!(config.serial.baudrate, 115200);
    }

    #[test]
    fn test_config_validate_success() {
        let manager = ConfigManager::new();
        assert!(manager.validate().is_ok());
    }

    #[test]
    fn test_config_validate_zero_baudrate() {
        let manager = ConfigManager::new();
        manager.set("serial.baudrate", "0").unwrap();
        assert!(manager.validate().is_err());
    }

    #[test]
    fn test_config_validate_invalid_databits() {
        let manager = ConfigManager::new();
        manager.set("serial.databits", "4").unwrap();
        assert!(manager.validate().is_err());
        manager.set("serial.databits", "9").unwrap();
        assert!(manager.validate().is_err());
    }

    #[test]
    fn test_config_validate_invalid_stopbits() {
        let manager = ConfigManager::new();
        manager.set("serial.stopbits", "0").unwrap();
        assert!(manager.validate().is_err());
        manager.set("serial.stopbits", "3").unwrap();
        assert!(manager.validate().is_err());
    }

    #[test]
    fn test_config_validate_invalid_parity() {
        let manager = ConfigManager::new();
        manager.set("serial.parity", "mark").unwrap();
        assert!(manager.validate().is_err());
    }

    #[test]
    fn test_config_validate_invalid_log_level() {
        let manager = ConfigManager::new();
        manager.set("logging.level", "verbose").unwrap();
        assert!(manager.validate().is_err());
    }
}
