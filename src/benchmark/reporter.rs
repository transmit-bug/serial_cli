//! Benchmark reporting and comparison
//!
//! Utilities for reporting benchmark results and comparing runs.

use super::{BenchmarkCategory, BenchmarkResult};
use std::collections::HashMap;

/// Benchmark report
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkReport {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub results: Vec<BenchmarkResult>,
}

impl BenchmarkReport {
    /// Create a new report from results
    pub fn new(results: Vec<BenchmarkResult>) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            results,
        }
    }

    /// Get results by category
    pub fn by_category(&self, category: BenchmarkCategory) -> Vec<&BenchmarkResult> {
        self.results
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    /// Print summary
    pub fn print_summary(&self) {
        println!("\n=== Benchmark Summary ===");
        println!(
            "Timestamp: {}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        );
        println!("Total benchmarks: {}\n", self.results.len());

        for category in BenchmarkCategory::all() {
            let results = self.by_category(category);
            if !results.is_empty() {
                println!("{}:", category.name());
                for result in results {
                    if let Some(throughput) = result.throughput_bytes_per_sec() {
                        println!("  {}: {:.2} MB/s", result.name, throughput / 1_000_000.0);
                    } else {
                        println!(
                            "  {}: {:.2} ns/iter",
                            result.name,
                            result.avg_ns_per_iteration()
                        );
                    }
                }
                println!();
            }
        }
    }
}

/// Comparison result between two benchmark runs
#[derive(Debug, Clone)]
pub struct ComparisonResult {
    pub name: String,
    pub category: BenchmarkCategory,
    pub baseline_ns: f64,
    pub current_ns: f64,
    pub change_percent: f64,
    pub is_regression: bool,
    pub is_improvement: bool,
}

impl ComparisonResult {
    /// Print comparison
    pub fn print(&self) {
        let status = if self.is_regression {
            "[REGRESSION]"
        } else if self.is_improvement {
            "[IMPROVEMENT]"
        } else {
            "[NO CHANGE]"
        };

        println!("  {}: {} ({:+.1}%)", self.name, status, self.change_percent);
        println!(
            "    Baseline: {:.2} ns/iter -> Current: {:.2} ns/iter",
            self.baseline_ns, self.current_ns
        );
    }
}

/// Compare two benchmark reports
pub fn compare_benchmarks(
    baseline: &BenchmarkReport,
    current: &BenchmarkReport,
) -> Vec<ComparisonResult> {
    let mut comparisons = Vec::new();

    // Use &str as the key type to avoid reference issues
    let baseline_map: HashMap<&str, _> = baseline
        .results
        .iter()
        .map(|r| (r.name.as_str(), r))
        .collect();

    let current_map: HashMap<&str, _> = current
        .results
        .iter()
        .map(|r| (r.name.as_str(), r))
        .collect();

    // Find all unique benchmark names
    let all_names: std::collections::HashSet<_> = baseline_map
        .keys()
        .chain(current_map.keys())
        .cloned()
        .collect();

    const REGRESSION_THRESHOLD: f64 = 5.0; // 5% regression threshold
    const IMPROVEMENT_THRESHOLD: f64 = 5.0; // 5% improvement threshold

    for name in all_names {
        if let (Some(baseline), Some(current)) = (baseline_map.get(&name), current_map.get(&name)) {
            let baseline_ns = baseline.avg_ns_per_iteration();
            let current_ns = current.avg_ns_per_iteration();
            let change_percent = ((current_ns - baseline_ns) / baseline_ns) * 100.0;

            let is_regression = change_percent > REGRESSION_THRESHOLD;
            let is_improvement = change_percent < -IMPROVEMENT_THRESHOLD;

            comparisons.push(ComparisonResult {
                name: name.to_string(),
                category: baseline.category,
                baseline_ns,
                current_ns,
                change_percent,
                is_regression,
                is_improvement,
            });
        }
    }

    // Sort by regression severity
    comparisons.sort_by(|a, b| {
        b.change_percent
            .partial_cmp(&a.change_percent)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    comparisons
}
