//! Task monitor
//!
//! This module provides monitoring and statistics for task execution.

use crate::error::Result;
use crate::task::executor::{TaskCompletion, TaskExecutor};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Task statistics
#[derive(Debug, Clone, Default)]
pub struct TaskStats {
    pub total_submitted: u64,
    pub total_completed: u64,
    pub total_failed: u64,
    pub total_duration_ms: u64,
    pub average_duration_ms: u64,
}

/// Task monitor
pub struct TaskMonitor {
    executor: Arc<TaskExecutor>,
    stats: Arc<RwLock<TaskStats>>,
    start_time: Instant,
}

impl TaskMonitor {
    /// Create a new task monitor
    pub fn new(executor: Arc<TaskExecutor>) -> Self {
        Self {
            executor,
            stats: Arc::new(RwLock::new(TaskStats::default())),
            start_time: Instant::now(),
        }
    }

    /// Get current statistics
    pub async fn stats(&self) -> TaskStats {
        self.stats.read().await.clone()
    }

    /// Update statistics from completed tasks
    pub async fn update_stats(&self) -> Result<()> {
        let completions: Vec<TaskCompletion> = self.executor.get_completed().await;

        let mut stats = self.stats.write().await;

        let total_completed = completions.len() as u64;
        let total_failed = completions
            .iter()
            .filter(|c| matches!(c.result, crate::task::TaskResult::Error(_)))
            .count() as u64;

        let total_duration_ms: u64 = completions
            .iter()
            .map(|c| c.duration.as_millis() as u64)
            .sum();

        let average_duration_ms = if total_completed > 0 {
            total_duration_ms / total_completed
        } else {
            0
        };

        stats.total_completed = total_completed;
        stats.total_failed = total_failed;
        stats.total_duration_ms = total_duration_ms;
        stats.average_duration_ms = average_duration_ms;

        Ok(())
    }

    /// Get pending task count
    pub async fn pending_count(&self) -> usize {
        self.executor.pending_count().await
    }

    /// Get running task count
    pub async fn running_count(&self) -> usize {
        self.executor.running_count().await
    }

    /// Get uptime
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Print status report
    pub async fn print_report(&self) {
        let stats = self.stats().await;
        let pending = self.pending_count().await;
        let running = self.running_count().await;
        let uptime = self.uptime();

        tracing::info!("=== Task Monitor Report ===");
        tracing::info!("Uptime: {:.2}s", uptime.as_secs_f64());
        tracing::info!("Pending tasks: {}", pending);
        tracing::info!("Running tasks: {}", running);
        tracing::info!("Completed tasks: {}", stats.total_completed);
        tracing::info!("Failed tasks: {}", stats.total_failed);
        tracing::info!("Total duration: {}ms", stats.total_duration_ms);
        tracing::info!("Average duration: {}ms", stats.average_duration_ms);
        tracing::info!("========================");
    }

    /// Start monitoring loop
    pub async fn start_monitoring(&self, interval_secs: u64) -> Result<()> {
        let stats = self.stats.clone();
        let executor = self.executor.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));

            loop {
                interval.tick().await;

                // Update statistics
                let completions: Vec<TaskCompletion> = executor.get_completed().await;

                if !completions.is_empty() {
                    let mut stats_guard = stats.write().await;

                    let total_completed = completions.len() as u64;
                    let total_failed = completions
                        .iter()
                        .filter(|c| matches!(c.result, crate::task::TaskResult::Error(_)))
                        .count() as u64;

                    let total_duration_ms: u64 = completions
                        .iter()
                        .map(|c| c.duration.as_millis() as u64)
                        .sum();

                    let average_duration_ms =
                        total_duration_ms.checked_div(total_completed).unwrap_or(0);

                    stats_guard.total_completed = total_completed;
                    stats_guard.total_failed = total_failed;
                    stats_guard.total_duration_ms = total_duration_ms;
                    stats_guard.average_duration_ms = average_duration_ms;
                }
            }
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitor_counts() {
        let executor = Arc::new(TaskExecutor::new(10));
        let monitor = TaskMonitor::new(executor.clone());

        let stats = monitor.stats().await;
        assert_eq!(stats.total_completed, 0);
        assert_eq!(monitor.pending_count().await, 0);
        assert_eq!(monitor.running_count().await, 0);
    }
}
