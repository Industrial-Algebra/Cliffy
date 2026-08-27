// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Report generation for scale tests
//!
//! Produces human-readable and machine-parseable reports.

use crate::simulator::SimulationResult;
use serde::{Deserialize, Serialize};

/// Output format for reports
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    /// Plain text
    Text,
    /// JSON format
    Json,
    /// Markdown format
    Markdown,
}

/// Scale test report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleTestReport {
    /// Scenario name
    pub scenario: String,

    /// Test results summary
    pub summary: TestSummary,

    /// Convergence details
    pub convergence: ConvergenceDetails,

    /// Throughput details
    pub throughput: ThroughputDetails,

    /// Test metadata
    pub metadata: TestMetadata,
}

/// Summary of test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary {
    /// Whether the test passed (converged within timeout)
    pub passed: bool,
    /// Number of users
    pub user_count: usize,
    /// Total duration
    pub duration_ms: u64,
    /// Total operations processed
    pub total_operations: usize,
}

/// Convergence-related details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceDetails {
    /// Whether convergence was achieved
    pub converged: bool,
    /// Time to convergence in milliseconds
    pub convergence_time_ms: Option<u64>,
    /// Maximum divergence observed
    pub max_divergence: f64,
    /// Final divergence
    pub final_divergence: f64,
    /// Number of divergence spikes
    pub divergence_spikes: usize,
}

/// Throughput-related details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputDetails {
    /// Operations per second
    pub ops_per_second: f64,
    /// Messages per second
    pub messages_per_second: f64,
    /// Message drop rate (percentage)
    pub drop_rate_percent: f64,
    /// Latency percentiles in milliseconds
    pub latency_p50_ms: f64,
    pub latency_p95_ms: f64,
    pub latency_p99_ms: f64,
}

/// Test metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMetadata {
    /// Timestamp when test was run
    pub timestamp: String,
    /// Rust version
    pub rust_version: String,
    /// Platform
    pub platform: String,
}

impl ScaleTestReport {
    /// Create a report from simulation results
    pub fn from_result(scenario: impl Into<String>, result: &SimulationResult) -> Self {
        Self {
            scenario: scenario.into(),
            summary: TestSummary {
                passed: result.converged,
                user_count: result.user_count,
                duration_ms: result.total_duration.as_millis() as u64,
                total_operations: result.throughput_metrics.total_operations,
            },
            convergence: ConvergenceDetails {
                converged: result.converged,
                convergence_time_ms: result.convergence_time.map(|d| d.as_millis() as u64),
                max_divergence: result.convergence_metrics.max_divergence,
                final_divergence: result.convergence_metrics.final_divergence,
                divergence_spikes: result.convergence_metrics.divergence_spikes,
            },
            throughput: ThroughputDetails {
                ops_per_second: result.throughput_metrics.ops_per_second,
                messages_per_second: result.throughput_metrics.messages_per_second,
                drop_rate_percent: result.throughput_metrics.drop_rate(),
                latency_p50_ms: result.throughput_metrics.latency_p50_ms,
                latency_p95_ms: result.throughput_metrics.latency_p95_ms,
                latency_p99_ms: result.throughput_metrics.latency_p99_ms,
            },
            metadata: TestMetadata {
                timestamp: chrono_lite_timestamp(),
                rust_version: String::new(),
                platform: std::env::consts::OS.to_string(),
            },
        }
    }

    /// Format the report
    #[must_use]
    pub fn format(&self, format: ReportFormat) -> String {
        match format {
            ReportFormat::Text => self.format_text(),
            ReportFormat::Json => self.format_json(),
            ReportFormat::Markdown => self.format_markdown(),
        }
    }

    fn format_text(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("=== Scale Test Report: {} ===\n\n", self.scenario));

        output.push_str("SUMMARY\n");
        output.push_str(&format!(
            "  Status: {}\n",
            if self.summary.passed {
                "PASSED"
            } else {
                "FAILED"
            }
        ));
        output.push_str(&format!("  Users: {}\n", self.summary.user_count));
        output.push_str(&format!("  Duration: {}ms\n", self.summary.duration_ms));
        output.push_str(&format!(
            "  Operations: {}\n\n",
            self.summary.total_operations
        ));

        output.push_str("CONVERGENCE\n");
        output.push_str(&format!("  Converged: {}\n", self.convergence.converged));
        if let Some(time) = self.convergence.convergence_time_ms {
            output.push_str(&format!("  Time: {time}ms\n"));
        }
        output.push_str(&format!(
            "  Max Divergence: {:.6}\n",
            self.convergence.max_divergence
        ));
        output.push_str(&format!(
            "  Final Divergence: {:.6}\n\n",
            self.convergence.final_divergence
        ));

        output.push_str("THROUGHPUT\n");
        output.push_str(&format!(
            "  Ops/sec: {:.2}\n",
            self.throughput.ops_per_second
        ));
        output.push_str(&format!(
            "  Messages/sec: {:.2}\n",
            self.throughput.messages_per_second
        ));
        output.push_str(&format!(
            "  Drop Rate: {:.2}%\n",
            self.throughput.drop_rate_percent
        ));

        output
    }

    fn format_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    fn format_markdown(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("# Scale Test Report: {}\n\n", self.scenario));

        output.push_str("## Summary\n\n");
        output.push_str("| Metric | Value |\n");
        output.push_str("|--------|-------|\n");
        output.push_str(&format!(
            "| Status | {} |\n",
            if self.summary.passed {
                "PASSED"
            } else {
                "FAILED"
            }
        ));
        output.push_str(&format!("| Users | {} |\n", self.summary.user_count));
        output.push_str(&format!("| Duration | {}ms |\n", self.summary.duration_ms));
        output.push_str(&format!(
            "| Operations | {} |\n\n",
            self.summary.total_operations
        ));

        output.push_str("## Convergence\n\n");
        output.push_str("| Metric | Value |\n");
        output.push_str("|--------|-------|\n");
        output.push_str(&format!("| Converged | {} |\n", self.convergence.converged));
        if let Some(time) = self.convergence.convergence_time_ms {
            output.push_str(&format!("| Time | {time}ms |\n"));
        }
        output.push_str(&format!(
            "| Max Divergence | {:.6} |\n",
            self.convergence.max_divergence
        ));
        output.push_str(&format!(
            "| Final Divergence | {:.6} |\n\n",
            self.convergence.final_divergence
        ));

        output.push_str("## Throughput\n\n");
        output.push_str("| Metric | Value |\n");
        output.push_str("|--------|-------|\n");
        output.push_str(&format!(
            "| Ops/sec | {:.2} |\n",
            self.throughput.ops_per_second
        ));
        output.push_str(&format!(
            "| Messages/sec | {:.2} |\n",
            self.throughput.messages_per_second
        ));
        output.push_str(&format!(
            "| Drop Rate | {:.2}% |\n",
            self.throughput.drop_rate_percent
        ));

        output
    }
}

/// Simple timestamp without chrono dependency
fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenarios::ScaleTestScenario;

    #[test]
    fn test_report_generation() {
        let scenario = ScaleTestScenario::Counter {
            users: 5,
            ops_per_user: 2,
        };
        let report = scenario.run_with_report();

        assert_eq!(report.summary.user_count, 5);
    }

    #[test]
    fn test_report_formats() {
        let scenario = ScaleTestScenario::Counter {
            users: 3,
            ops_per_user: 1,
        };
        let report = scenario.run_with_report();

        let text = report.format(ReportFormat::Text);
        assert!(text.contains("Scale Test Report"));

        let json = report.format(ReportFormat::Json);
        assert!(json.contains("scenario"));

        let md = report.format(ReportFormat::Markdown);
        assert!(md.contains("# Scale Test Report"));
    }
}
