//! Benchmark module for performance testing and optimization
//!
//! This module provides benchmarking utilities for measuring
//! and optimizing serial CLI performance.

pub mod reporter;
pub mod runner;

pub use reporter::{BenchmarkReport, ComparisonResult};
pub use runner::BenchmarkRunner;

/// Benchmark categories, each grouping related performance tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BenchmarkCategory {
    /// Serial I/O throughput — buffer copy, protocol encode/decode round-trips.
    SerialIo,
    /// Virtual port creation and bridging performance.
    VirtualPort,
    /// Protocol parsing speed (Modbus RTU/ASCII, CRC, LRC).
    Protocol,
    /// Application startup time (cold/warm start, Lua engine init).
    Startup,
    /// Memory allocation overhead and footprint measurements.
    Memory,
    /// Concurrent task execution overhead.
    Concurrency,
}

impl BenchmarkCategory {
    pub fn all() -> Vec<Self> {
        vec![
            Self::SerialIo,
            Self::VirtualPort,
            Self::Protocol,
            Self::Startup,
            Self::Memory,
            Self::Concurrency,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::SerialIo => "serial-io",
            Self::VirtualPort => "virtual-port",
            Self::Protocol => "protocol",
            Self::Startup => "startup",
            Self::Memory => "memory",
            Self::Concurrency => "concurrency",
        }
    }
}

/// Result of a single benchmark run, capturing timing and optional throughput data.
///
/// Can be serialized to JSON for persistence and later comparison via
/// [`compare_benchmarks`](crate::benchmark::reporter::compare_benchmarks).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkResult {
    /// Human-readable benchmark name (e.g., `"modbus_rtu_parse"`).
    pub name: String,
    /// The category this benchmark belongs to.
    pub category: BenchmarkCategory,
    /// Number of iterations executed during measurement.
    pub iterations: u64,
    /// Total elapsed time across all iterations, in nanoseconds.
    pub elapsed_ns: u64,
    /// Total bytes processed across all iterations, if applicable.
    /// When `Some`, [`throughput_bytes_per_sec`](Self::throughput_bytes_per_sec)
    /// returns a meaningful value.
    pub bytes_processed: Option<u64>,
}

impl BenchmarkResult {
    /// Calculate throughput in bytes per second. Returns `None` if this
    /// benchmark did not track byte counts.
    pub fn throughput_bytes_per_sec(&self) -> Option<f64> {
        self.bytes_processed.map(|bytes| {
            let elapsed_sec = self.elapsed_ns as f64 / 1_000_000_000.0;
            bytes as f64 / elapsed_sec
        })
    }

    /// Calculate the average time per iteration in nanoseconds.
    pub fn avg_ns_per_iteration(&self) -> f64 {
        self.elapsed_ns as f64 / self.iterations as f64
    }
}
