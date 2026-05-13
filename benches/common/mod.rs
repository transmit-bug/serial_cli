//! Common utilities for benchmarks

pub mod data_generator;

// Re-export commonly used functions for convenience
#[allow(unused_imports)]
pub use data_generator::{
    generate_at_command, generate_at_response, generate_modbus_request, generate_pattern_data,
};
