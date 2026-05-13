// Copyright 2024 Serial CLI Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::state::app_state::AppState;
use serial_cli::config::Config;
use std::fs;
use std::path::PathBuf;
use tauri::State;

/// Get current configuration
#[tauri::command]
pub async fn get_config(_state: State<'_, AppState>) -> Result<ConfigData, String> {
    let config = serial_cli::config::load_config_with_fallback();
    Ok(ConfigData {
        serial: SerialConfigData {
            baudrate: config.serial.baudrate,
            databits: config.serial.databits,
            stopbits: config.serial.stopbits,
            parity: config.serial.parity,
            timeout_ms: config.serial.timeout_ms,
        },
        logging: LoggingConfigData {
            level: config.logging.level,
            format: config.logging.format,
            file: config.logging.file,
        },
        lua: LuaConfigData {
            memory_limit_mb: config.lua.memory_limit_mb,
            timeout_seconds: config.lua.timeout_seconds,
            enable_sandbox: config.lua.enable_sandbox,
        },
        task: TaskConfigData {
            max_concurrent: config.task.max_concurrent,
            default_timeout_seconds: config.task.default_timeout_seconds,
        },
        output: OutputConfigData {
            json_pretty: config.output.json_pretty,
            show_timestamp: config.output.show_timestamp,
        },
    })
}

/// Get configuration file content as TOML string
#[allow(dead_code)] // GUI API stub — awaiting frontend integration
#[tauri::command]
pub async fn get_config_raw() -> Result<String, String> {
    let config_path = get_config_path()?;
    if !config_path.exists() {
        // Return default config as TOML
        let default_config = Config::default();
        let toml_string = toml::to_string_pretty(&default_config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        return Ok(toml_string);
    }

    fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config file: {}", e))
}

/// Update configuration
#[tauri::command]
pub async fn update_config(config: ConfigData, _state: State<'_, AppState>) -> Result<(), String> {
    let config_path = get_config_path()?;

    // Create config directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
    }

    // Load existing config or create default
    let mut existing_config = if config_path.exists() {
        serial_cli::config::load_config(&config_path)
            .map_err(|e| format!("Failed to load existing config: {}", e))?
    } else {
        Config::default()
    };

    // Update fields
    existing_config.serial.baudrate = config.serial.baudrate;
    existing_config.serial.databits = config.serial.databits;
    existing_config.serial.stopbits = config.serial.stopbits;
    existing_config.serial.parity = config.serial.parity;
    existing_config.serial.timeout_ms = config.serial.timeout_ms;

    existing_config.logging.level = config.logging.level;
    existing_config.logging.format = config.logging.format;
    existing_config.logging.file = config.logging.file;

    existing_config.lua.memory_limit_mb = config.lua.memory_limit_mb;
    existing_config.lua.timeout_seconds = config.lua.timeout_seconds;
    existing_config.lua.enable_sandbox = config.lua.enable_sandbox;

    existing_config.task.max_concurrent = config.task.max_concurrent;
    existing_config.task.default_timeout_seconds = config.task.default_timeout_seconds;

    existing_config.output.json_pretty = config.output.json_pretty;
    existing_config.output.show_timestamp = config.output.show_timestamp;

    // Serialize to TOML
    let toml_string = toml::to_string_pretty(&existing_config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    // Write to file
    fs::write(&config_path, toml_string).map_err(|e| format!("Failed to write config file: {}", e))
}

/// Save configuration from raw TOML string
#[allow(dead_code)] // GUI API stub — awaiting frontend integration
#[tauri::command]
pub async fn save_config_raw(content: String) -> Result<(), String> {
    let config_path = get_config_path()?;

    // Validate TOML syntax
    toml::from_str::<Config>(&content).map_err(|e| format!("Invalid TOML syntax: {}", e))?;

    // Create config directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
    }

    // Write to file
    fs::write(&config_path, content).map_err(|e| e.to_string())
}

/// Reset configuration to defaults
#[tauri::command]
pub async fn reset_config() -> Result<(), String> {
    let config_path = get_config_path()?;

    let default_config = Config::default();
    let toml_string = toml::to_string_pretty(&default_config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, toml_string).map_err(|e| format!("Failed to write config file: {}", e))
}

/// Get configuration file path
#[allow(dead_code)] // GUI API stub — awaiting frontend integration
#[tauri::command]
pub async fn get_config_file_path() -> Result<String, String> {
    let path = get_config_path()?;
    Ok(path.to_string_lossy().to_string())
}

/// Get configuration path helper
fn get_config_path() -> Result<PathBuf, String> {
    if let Some(path) = serial_cli::config::get_global_config_path() {
        Ok(path)
    } else {
        // Fallback to project-local config
        Ok(PathBuf::from(".serial-cli.toml"))
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ConfigData {
    pub serial: SerialConfigData,
    pub logging: LoggingConfigData,
    pub lua: LuaConfigData,
    pub task: TaskConfigData,
    pub output: OutputConfigData,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SerialConfigData {
    pub baudrate: u32,
    pub databits: u8,
    pub stopbits: u8,
    pub parity: String,
    pub timeout_ms: u64,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoggingConfigData {
    pub level: String,
    pub format: String,
    pub file: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LuaConfigData {
    pub memory_limit_mb: usize,
    pub timeout_seconds: u64,
    pub enable_sandbox: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TaskConfigData {
    pub max_concurrent: usize,
    pub default_timeout_seconds: u64,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct OutputConfigData {
    pub json_pretty: bool,
    pub show_timestamp: bool,
}
