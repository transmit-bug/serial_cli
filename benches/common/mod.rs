//! Common utilities for benchmarks

pub mod data_generator;

// Re-export commonly used functions for convenience
pub use data_generator::{generate_at_command, generate_at_response, generate_modbus_request, generate_pattern_data};
