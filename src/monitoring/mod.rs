//! Performance monitoring and profiling utilities
//!
//! This module provides performance monitoring capabilities for serial operations,
//! including platform-specific implementations for Windows and Unix systems.
//!
//! **Experimental** — not yet integrated into any CLI command.
//! Planned for v0.6.0 integration with a unified session management abstraction.

#![allow(dead_code)]

pub mod windows;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Operation name
    pub operation: String,
    /// Total operations performed
    pub total_operations: usize,
    /// Successful operations
    pub successful_operations: usize,
    /// Failed operations
    pub failed_operations: usize,
    /// Total duration
    pub total_duration: Duration,
    /// Average operation time
    pub average_operation_time: Duration,
    /// Min operation time
    pub min_operation_time: Duration,
    /// Max operation time
    pub max_operation_time: Duration,
    /// Operations per second
    pub operations_per_second: f64,
    /// Data throughput (bytes/second)
    pub throughput_bytes_per_second: f64,
}

impl PerformanceMetrics {
    /// Create new metrics
    pub fn new(operation: String) -> Self {
        Self {
            operation,
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            total_duration: Duration::ZERO,
            average_operation_time: Duration::ZERO,
            min_operation_time: Duration::MAX,
            max_operation_time: Duration::ZERO,
            operations_per_second: 0.0,
            throughput_bytes_per_second: 0.0,
        }
    }

    /// Update metrics with new operation
    pub fn update(&mut self, duration: Duration, success: bool, bytes_processed: usize) {
        self.total_operations += 1;
        self.total_duration += duration;

        if success {
            self.successful_operations += 1;
        } else {
            self.failed_operations += 1;
        }

        // Update min/max times
        self.min_operation_time = self.min_operation_time.min(duration);
        self.max_operation_time = self.max_operation_time.max(duration);

        // Update average
        if self.total_operations > 0 {
            self.average_operation_time = self.total_duration / self.total_operations as u32;

            // Calculate ops per second
            let total_secs = self.total_duration.as_secs_f64();
            if total_secs > 0.0 {
                self.operations_per_second = self.total_operations as f64 / total_secs;
                self.throughput_bytes_per_second = bytes_processed as f64 / total_secs;
            }
        }
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            return 1.0;
        }
        self.successful_operations as f64 / self.total_operations as f64
    }

    /// Format as string
    pub fn format(&self) -> String {
        format!(
            "{}: {} ops ({} successful, {} failed) | {:.2} ops/s | {:.2} MB/s | avg: {:?} | min: {:?} | max: {:?}",
            self.operation,
            self.total_operations,
            self.successful_operations,
            self.failed_operations,
            self.operations_per_second,
            self.throughput_bytes_per_second / 1_048_576.0, // Convert to MB/s
            self.average_operation_time,
            if self.min_operation_time == Duration::MAX {
                Duration::ZERO
            } else {
                self.min_operation_time
            },
            self.max_operation_time
        )
    }
}

/// Performance monitor
pub struct PerformanceMonitor {
    metrics: Arc<Mutex<HashMap<String, PerformanceMetrics>>>,
    start_time: Instant,
}

impl PerformanceMonitor {
    /// Create new performance monitor
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    /// Record an operation
    pub async fn record_operation(
        &self,
        operation: &str,
        duration: Duration,
        success: bool,
        bytes_processed: usize,
    ) {
        let mut metrics = self.metrics.lock().await;

        if !metrics.contains_key(operation) {
            metrics.insert(
                operation.to_string(),
                PerformanceMetrics::new(operation.to_string()),
            );
        }

        metrics
            .get_mut(operation)
            .unwrap()
            .update(duration, success, bytes_processed);
    }

    /// Get metrics for an operation
    pub async fn get_metrics(&self, operation: &str) -> Option<PerformanceMetrics> {
        let metrics = self.metrics.lock().await;
        metrics.get(operation).cloned()
    }

    /// Get all metrics
    pub async fn get_all_metrics(&self) -> Vec<PerformanceMetrics> {
        let metrics = self.metrics.lock().await;
        metrics.values().cloned().collect()
    }

    /// Print performance report
    pub async fn print_report(&self) {
        let metrics = self.get_all_metrics().await;

        if metrics.is_empty() {
            tracing::info!("No performance metrics available");
            return;
        }

        tracing::info!("\n=== Performance Report ===");
        tracing::info!("Uptime: {:.2}s", self.start_time.elapsed().as_secs_f64());
        tracing::info!("");

        for metric in metrics {
            tracing::info!("{}", metric.format());
        }

        tracing::info!("");
    }

    /// Reset all metrics
    pub async fn reset(&mut self) {
        let mut metrics = self.metrics.lock().await;
        metrics.clear();
        self.start_time = Instant::now();
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Operation timer
pub struct OperationTimer {
    operation: String,
    start_time: Instant,
    monitor: Arc<PerformanceMonitor>,
    bytes_processed: usize,
}

impl OperationTimer {
    /// Create new operation timer
    pub fn new(operation: String, monitor: Arc<PerformanceMonitor>) -> Self {
        Self {
            operation,
            start_time: Instant::now(),
            monitor,
            bytes_processed: 0,
        }
    }

    /// Set bytes processed
    pub fn set_bytes_processed(&mut self, bytes: usize) {
        self.bytes_processed = bytes;
    }

    /// Complete the operation (success)
    pub async fn complete(self) {
        let duration = self.start_time.elapsed();
        self.monitor
            .record_operation(&self.operation, duration, true, self.bytes_processed)
            .await;
    }

    /// Complete the operation with failure
    pub async fn complete_failure(self) {
        let duration = self.start_time.elapsed();
        self.monitor
            .record_operation(&self.operation, duration, false, self.bytes_processed)
            .await;
    }
}

/// Resource usage monitor
pub struct ResourceMonitor {
    /// Memory usage in bytes
    pub memory_usage: usize,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Open file descriptors
    pub open_fds: usize,
    /// Thread count
    pub thread_count: usize,
}

impl ResourceMonitor {
    /// Create new resource monitor
    pub fn new() -> Self {
        Self {
            memory_usage: 0,
            cpu_usage: 0.0,
            open_fds: 0,
            thread_count: 0,
        }
    }

    /// Update resource usage (platform-specific)
    pub fn update(&mut self) {
        #[cfg(unix)]
        {
            // Get memory usage on Unix systems
            self.memory_usage = Self::get_memory_usage_unix();
            self.open_fds = Self::get_open_fds_unix();
        }

        #[cfg(windows)]
        {
            // Use Windows-specific monitoring
            match windows::WindowsPerformanceMonitor::new() {
                Ok(mut win_monitor) => {
                    match win_monitor.update_metrics() {
                        Ok(metrics) => {
                            self.memory_usage = metrics.working_set_size;
                            self.cpu_usage = metrics.cpu_usage;
                        }
                        Err(e) => {
                            tracing::warn!("Failed to update Windows metrics: {:?}", e);
                            // Fallback to basic memory estimation
                            self.memory_usage = 0;
                            self.cpu_usage = 0.0;
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to create Windows performance monitor: {:?}", e);
                    // Fallback values
                    self.memory_usage = 0;
                    self.cpu_usage = 0.0;
                }
            }
            // On Windows, file descriptors are not directly accessible
            self.open_fds = 0;
        }

        self.thread_count = Self::get_thread_count();
    }

    #[cfg(unix)]
    fn get_memory_usage_unix() -> usize {
        use std::fs;
        // Try to read from /proc/self/status
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<usize>() {
                            return kb * 1024; // Convert to bytes
                        }
                    }
                }
            }
        }

        // Fallback: estimate based on heap size
        0
    }

    #[cfg(unix)]
    fn get_open_fds_unix() -> usize {
        use std::fs;
        // Count entries in /proc/self/fd
        if let Ok(entries) = fs::read_dir("/proc/self/fd") {
            return entries.count();
        }
        0
    }

    fn get_thread_count() -> usize {
        // Estimate thread count using available parallelism
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    }

    /// Format as string
    pub fn format(&self) -> String {
        format!(
            "Memory: {:.2} MB | CPU: {:.1}% | FDs: {} | Threads: {}",
            self.memory_usage as f64 / 1_048_576.0,
            self.cpu_usage,
            self.open_fds,
            self.thread_count
        )
    }
}

impl Default for ResourceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new();

        // Record some operations
        monitor
            .record_operation("test_op", Duration::from_millis(100), true, 1024)
            .await;
        monitor
            .record_operation("test_op", Duration::from_millis(200), true, 2048)
            .await;
        monitor
            .record_operation("test_op", Duration::from_millis(150), false, 0)
            .await;

        let metrics = monitor.get_metrics("test_op").await;
        assert!(metrics.is_some());

        let metrics = metrics.unwrap();
        assert_eq!(metrics.total_operations, 3);
        assert_eq!(metrics.successful_operations, 2);
        assert_eq!(metrics.failed_operations, 1);
    }

    #[tokio::test]
    async fn test_operation_timer() {
        let monitor = Arc::new(PerformanceMonitor::new());
        let timer = OperationTimer::new("test_timer".to_string(), monitor.clone());

        // Simulate some work
        tokio::time::sleep(Duration::from_millis(50)).await;

        timer.complete().await;

        let metrics = monitor.get_metrics("test_timer").await;
        assert!(metrics.is_some());
        assert_eq!(metrics.unwrap().total_operations, 1);
    }

    #[test]
    fn test_resource_monitor() {
        let mut monitor = ResourceMonitor::new();
        monitor.update();

        // Basic sanity checks
        assert!(monitor.thread_count > 0);
    }
}
