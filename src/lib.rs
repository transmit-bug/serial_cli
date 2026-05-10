//! Serial CLI - A universal serial port tool optimized for AI interaction
//!
//! This library provides the core functionality for the serial-cli tool,
//! including serial port management, protocol handling, and Lua scripting.

pub mod benchmark;
pub mod cli;
pub mod config;
pub mod error;
pub mod error_handling;
pub mod logging;
pub mod lua;
pub mod monitoring;
pub mod protocol;
pub mod serial_core;
pub mod server;
pub mod task;
pub mod utils;

// Re-exports for convenience
pub use error::{Result, SerialError};
