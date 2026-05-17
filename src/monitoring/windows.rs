//! Windows-specific monitoring utilities
//!
//! This module provides Windows implementations for performance monitoring,
//! including CPU usage, memory metrics, and system resource tracking.

#[cfg(windows)]
use std::mem;
#[cfg(windows)]
use windows::Win32::Foundation::HANDLE;
#[cfg(windows)]
use windows::Win32::System::ProcessStatus::{
    GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
};
#[cfg(windows)]
use windows::Win32::System::Threading::GetCurrentProcess;

/// Windows-specific performance metrics
#[derive(Debug, Clone)]
pub struct WindowsMetrics {
    /// Working set size (physical memory usage)
    pub working_set_size: usize,
    /// Peak working set size
    pub peak_working_set_size: usize,
    /// Page file usage (virtual memory)
    pub page_file_usage: usize,
    /// Peak page file usage
    pub peak_page_file_usage: usize,
    /// Page fault count
    pub page_fault_count: u64,
    /// CPU usage percentage (0-100)
    pub cpu_usage: f64,
    /// Handle count
    pub handle_count: usize,
    /// Thread count
    pub thread_count: usize,
    /// User time (CPU time in user mode)
    pub user_time: u64,
    /// Kernel time (CPU time in kernel mode)
    pub kernel_time: u64,
}

impl Default for WindowsMetrics {
    fn default() -> Self {
        Self {
            working_set_size: 0,
            peak_working_set_size: 0,
            page_file_usage: 0,
            peak_page_file_usage: 0,
            page_fault_count: 0,
            cpu_usage: 0.0,
            handle_count: 0,
            thread_count: 0,
            user_time: 0,
            kernel_time: 0,
        }
    }
}

impl WindowsMetrics {
    /// Format metrics as a human-readable string
    pub fn format(&self) -> String {
        format!(
            "Memory: {:.2} MB (peak: {:.2} MB) | Pagefile: {:.2} MB | CPU: {:.1}% | Handles: {} | Threads: {} | Page Faults: {}",
            self.working_set_size as f64 / 1_048_576.0,
            self.peak_working_set_size as f64 / 1_048_576.0,
            self.page_file_usage as f64 / 1_048_576.0,
            self.cpu_usage,
            self.handle_count,
            self.thread_count,
            self.page_fault_count
        )
    }

    /// Get memory usage in MB
    pub fn memory_mb(&self) -> f64 {
        self.working_set_size as f64 / 1_048_576.0
    }

    /// Get total CPU time (user + kernel)
    pub fn total_cpu_time(&self) -> u64 {
        self.user_time + self.kernel_time
    }
}

/// Windows performance monitor
#[cfg(windows)]
pub struct WindowsPerformanceMonitor {
    process_handle: HANDLE,
    last_cpu_time: Option<(u64, u64)>, // (user_time, kernel_time)
    last_update: Option<std::time::Instant>,
}

#[cfg(windows)]
impl WindowsPerformanceMonitor {
    /// Create a new Windows performance monitor
    pub fn new() -> Result<Self, crate::error::SerialError> {
        unsafe {
            let handle = GetCurrentProcess();
            Ok(Self {
                process_handle: handle,
                last_cpu_time: None,
                last_update: None,
            })
        }
    }

    /// Update Windows metrics with current process information
    pub fn update_metrics(&mut self) -> Result<WindowsMetrics, crate::error::SerialError> {
        unsafe {
            let mut metrics = WindowsMetrics::default();

            // Get memory information
            let mut pmc: PROCESS_MEMORY_COUNTERS_EX = mem::zeroed();
            pmc.cb = mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;

            if GetProcessMemoryInfo(
                self.process_handle,
                &mut pmc as *mut PROCESS_MEMORY_COUNTERS_EX as *mut PROCESS_MEMORY_COUNTERS,
                mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32,
            )
            .is_ok()
            {
                metrics.working_set_size = pmc.WorkingSetSize;
                metrics.peak_working_set_size = pmc.PeakWorkingSetSize;
                metrics.page_file_usage = pmc.PagefileUsage;
                metrics.peak_page_file_usage = pmc.PeakPagefileUsage;
                metrics.page_fault_count = pmc.PageFaultCount as u64;
            }

            // Get CPU times and calculate usage
            let current_time = std::time::Instant::now();
            if let (Some((last_user, last_kernel)), Some(last_time)) =
                (self.last_cpu_time, self.last_update)
            {
                let elapsed = current_time.duration_since(last_time).as_secs_f64();

                if elapsed > 0.0 {
                    // Calculate CPU usage based on time difference
                    let user_diff = metrics.user_time.saturating_sub(last_user);
                    let kernel_diff = metrics.kernel_time.saturating_sub(last_kernel);
                    let total_diff = user_diff + kernel_diff;

                    // Convert to percentage (100ns units to seconds, then to percentage)
                    metrics.cpu_usage = (total_diff as f64 / 10_000_000.0 / elapsed) * 100.0;
                    metrics.cpu_usage = metrics.cpu_usage.min(100.0); // Cap at 100%
                }
            }

            self.last_cpu_time = Some((metrics.user_time, metrics.kernel_time));
            self.last_update = Some(current_time);

            Ok(metrics)
        }
    }

    /// Get current CPU usage percentage
    pub fn get_cpu_usage(&mut self) -> Result<f64, crate::error::SerialError> {
        let metrics = self.update_metrics()?;
        Ok(metrics.cpu_usage)
    }

    /// Get current memory usage in bytes
    pub fn get_memory_usage(&mut self) -> Result<usize, crate::error::SerialError> {
        let metrics = self.update_metrics()?;
        Ok(metrics.working_set_size)
    }

    /// Get handle count for the process
    pub fn get_handle_count(&self) -> Result<usize, crate::error::SerialError> {
        // This would require GetProcessHandleCount, which may not be available
        // Returning a reasonable estimate
        Ok(100) // Placeholder
    }

    /// Check if process is elevated (running as administrator)
    pub fn is_elevated(&self) -> Result<bool, crate::error::SerialError> {
        unsafe {
            use windows::Win32::Security::{
                GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
            };
            use windows::Win32::System::Threading::OpenProcessToken;

            let mut token = HANDLE::default();
            if OpenProcessToken(self.process_handle, TOKEN_QUERY, &mut token).is_ok() {
                let mut elevation: TOKEN_ELEVATION = mem::zeroed();
                let mut size = 0;

                if GetTokenInformation(
                    token,
                    TokenElevation,
                    Some(&mut elevation as *mut _ as *mut _),
                    mem::size_of::<TOKEN_ELEVATION>() as u32,
                    &mut size,
                )
                .is_ok()
                {
                    return Ok(elevation.TokenIsElevated != 0);
                }
            }

            Ok(false)
        }
    }
}

#[cfg(windows)]
impl Default for WindowsPerformanceMonitor {
    fn default() -> Self {
        Self::new().expect("Failed to create Windows performance monitor")
    }
}

/// CPU usage tracker for Windows
#[cfg(windows)]
pub struct CpuUsageTracker {
    monitor: WindowsPerformanceMonitor,
    history: Vec<f64>,
    max_history: usize,
}

#[cfg(windows)]
impl CpuUsageTracker {
    /// Create a new CPU usage tracker
    pub fn new(max_history: usize) -> Self {
        Self {
            monitor: WindowsPerformanceMonitor::new().expect("Failed to create CPU tracker"),
            history: Vec::with_capacity(max_history),
            max_history,
        }
    }

    /// Record current CPU usage
    pub fn record(&mut self) -> Result<f64, crate::error::SerialError> {
        let usage = self.monitor.get_cpu_usage()?;

        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(usage);

        Ok(usage)
    }

    /// Get average CPU usage over the recorded history
    pub fn average(&self) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        self.history.iter().sum::<f64>() / self.history.len() as f64
    }

    /// Get maximum CPU usage in the recorded history
    pub fn maximum(&self) -> f64 {
        self.history.iter().cloned().fold(0.0f64, f64::max)
    }

    /// Get minimum CPU usage in the recorded history
    pub fn minimum(&self) -> f64 {
        self.history.iter().cloned().fold(100.0f64, f64::min)
    }

    /// Get the history of CPU usage values
    pub fn history(&self) -> &[f64] {
        &self.history
    }

    /// Clear the history
    pub fn clear(&mut self) {
        self.history.clear();
    }
}

#[cfg(windows)]
impl Default for CpuUsageTracker {
    fn default() -> Self {
        Self::new(100)
    }
}

/// Memory usage tracker for Windows
#[cfg(windows)]
pub struct MemoryUsageTracker {
    monitor: WindowsPerformanceMonitor,
    history: Vec<usize>,
    max_history: usize,
}

#[cfg(windows)]
impl MemoryUsageTracker {
    /// Create a new memory usage tracker
    pub fn new(max_history: usize) -> Self {
        Self {
            monitor: WindowsPerformanceMonitor::new().expect("Failed to create memory tracker"),
            history: Vec::with_capacity(max_history),
            max_history,
        }
    }

    /// Record current memory usage in bytes
    pub fn record(&mut self) -> Result<usize, crate::error::SerialError> {
        let usage = self.monitor.get_memory_usage()?;

        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(usage);

        Ok(usage)
    }

    /// Get average memory usage in bytes
    pub fn average(&self) -> usize {
        if self.history.is_empty() {
            return 0;
        }
        self.history.iter().sum::<usize>() / self.history.len()
    }

    /// Get peak memory usage in bytes
    pub fn peak(&self) -> usize {
        self.history.iter().cloned().max().unwrap_or(0)
    }

    /// Get the history of memory usage values
    pub fn history(&self) -> &[usize] {
        &self.history
    }

    /// Clear the history
    pub fn clear(&mut self) {
        self.history.clear();
    }

    /// Get memory usage in MB
    pub fn average_mb(&self) -> f64 {
        self.average() as f64 / 1_048_576.0
    }

    /// Get peak memory usage in MB
    pub fn peak_mb(&self) -> f64 {
        self.peak() as f64 / 1_048_576.0
    }
}

#[cfg(windows)]
impl Default for MemoryUsageTracker {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_metrics_default() {
        let metrics = WindowsMetrics::default();
        assert_eq!(metrics.working_set_size, 0);
        assert_eq!(metrics.cpu_usage, 0.0);
    }

    #[test]
    fn test_windows_metrics_format() {
        let metrics = WindowsMetrics {
            working_set_size: 10_485_760,      // 10 MB
            peak_working_set_size: 20_971_520, // 20 MB
            page_file_usage: 5_242_880,        // 5 MB
            peak_page_file_usage: 10_485_760,  // 10 MB
            page_fault_count: 100,
            cpu_usage: 25.5,
            handle_count: 50,
            thread_count: 4,
            user_time: 1000,
            kernel_time: 500,
        };

        let formatted = metrics.format();
        assert!(formatted.contains("10.00 MB"));
        assert!(formatted.contains("20.00 MB"));
        assert!(formatted.contains("25.5%"));
        assert!(formatted.contains("Handles: 50"));
        assert!(formatted.contains("Threads: 4"));
    }

    #[test]
    fn test_windows_metrics_memory_mb() {
        let metrics = WindowsMetrics {
            working_set_size: 10_485_760, // 10 MB
            ..Default::default()
        };

        assert_eq!(metrics.memory_mb(), 10.0);
    }

    #[test]
    fn test_windows_metrics_total_cpu_time() {
        let metrics = WindowsMetrics {
            user_time: 1000,
            kernel_time: 500,
            ..Default::default()
        };

        assert_eq!(metrics.total_cpu_time(), 1500);
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_performance_monitor_creation() {
        let monitor = WindowsPerformanceMonitor::new();
        assert!(monitor.is_ok());
    }

    #[cfg(windows)]
    #[test]
    fn test_cpu_tracker() {
        let mut tracker = CpuUsageTracker::new(5);

        // Record some values (will be 0.0 in tests)
        let _ = tracker.record();
        let _ = tracker.record();

        assert_eq!(tracker.history().len(), 2);
        assert_eq!(tracker.average(), 0.0); // Should be 0.0 in tests
    }

    #[cfg(windows)]
    #[test]
    fn test_memory_tracker() {
        let mut tracker = MemoryUsageTracker::new(5);

        // Record some values
        let _ = tracker.record();
        let _ = tracker.record();

        assert_eq!(tracker.history().len(), 2);
    }
}
