// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Metrics collection for scale testing
//!
//! Tracks convergence time, throughput, latency, and other performance indicators.

use std::time::{Duration, Instant};

/// Metrics for measuring convergence behavior
#[derive(Debug, Clone, Default)]
pub struct ConvergenceMetrics {
    /// Time to reach convergence (all states equal)
    pub convergence_time: Option<Duration>,
    /// Whether convergence was achieved
    pub converged: bool,
    /// Maximum state divergence observed during simulation
    pub max_divergence: f64,
    /// Final divergence value
    pub final_divergence: f64,
    /// Number of convergence checks performed
    pub check_count: usize,
    /// Number of times divergence increased
    pub divergence_spikes: usize,
}

impl ConvergenceMetrics {
    /// Create new metrics
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a divergence measurement
    pub fn record_divergence(&mut self, divergence: f64) {
        if divergence > self.max_divergence {
            self.max_divergence = divergence;
        }
        if divergence > self.final_divergence {
            self.divergence_spikes += 1;
        }
        self.final_divergence = divergence;
        self.check_count += 1;
    }

    /// Mark convergence achieved
    pub const fn mark_converged(&mut self, time: Duration) {
        self.converged = true;
        self.convergence_time = Some(time);
    }
}

/// Metrics for measuring throughput
#[derive(Debug, Clone, Default)]
pub struct ThroughputMetrics {
    /// Total operations processed
    pub total_operations: usize,
    /// Total messages sent
    pub total_messages: usize,
    /// Messages dropped due to packet loss
    pub dropped_messages: usize,
    /// Operations per second (average)
    pub ops_per_second: f64,
    /// Messages per second (average)
    pub messages_per_second: f64,
    /// Duration of measurement period
    pub duration: Duration,
    /// Latency percentiles
    pub latency_p50_ms: f64,
    pub latency_p95_ms: f64,
    pub latency_p99_ms: f64,
}

impl ThroughputMetrics {
    /// Create new metrics
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an operation
    pub const fn record_operation(&mut self) {
        self.total_operations += 1;
    }

    /// Record a message
    pub const fn record_message(&mut self, dropped: bool) {
        self.total_messages += 1;
        if dropped {
            self.dropped_messages += 1;
        }
    }

    /// Calculate rates based on duration
    pub fn finalize(&mut self, duration: Duration) {
        self.duration = duration;
        let secs = duration.as_secs_f64();
        if secs > 0.0 {
            self.ops_per_second = self.total_operations as f64 / secs;
            self.messages_per_second = self.total_messages as f64 / secs;
        }
    }

    /// Calculate latency percentiles from samples
    pub fn calculate_latency_percentiles(&mut self, latencies: &mut [f64]) {
        if latencies.is_empty() {
            return;
        }

        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let len = latencies.len();
        // len*k/100 < len for len >= 1 (empty case returned above), so these
        // are provably in-bounds; .get() keeps the panic-freedom auditable.
        self.latency_p50_ms = latencies.get(len * 50 / 100).copied().unwrap_or_default();
        self.latency_p95_ms = latencies.get(len * 95 / 100).copied().unwrap_or_default();
        self.latency_p99_ms = latencies.get(len * 99 / 100).copied().unwrap_or_default();
    }

    /// Get drop rate as a percentage
    #[must_use]
    pub fn drop_rate(&self) -> f64 {
        if self.total_messages == 0 {
            return 0.0;
        }
        self.dropped_messages as f64 / self.total_messages as f64 * 100.0
    }
}

/// Stopwatch for timing operations
#[derive(Debug)]
pub struct Stopwatch {
    start: Instant,
    laps: Vec<Duration>,
}

impl Stopwatch {
    /// Create and start a new stopwatch
    #[must_use]
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
            laps: Vec::new(),
        }
    }

    /// Record a lap time
    pub fn lap(&mut self) -> Duration {
        let elapsed = self.start.elapsed();
        self.laps.push(elapsed);
        elapsed
    }

    /// Get total elapsed time
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get all lap times
    #[must_use]
    pub fn laps(&self) -> &[Duration] {
        &self.laps
    }

    /// Reset the stopwatch
    pub fn reset(&mut self) {
        self.start = Instant::now();
        self.laps.clear();
    }
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self::start()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    // Exactly-representable constants: bit equality is the point.
    // (Stable clippy lacks the test-context exemption newer nightly has.)
    use super::*;

    #[test]
    fn test_convergence_metrics() {
        let mut metrics = ConvergenceMetrics::new();

        metrics.record_divergence(1.0);
        metrics.record_divergence(0.5);
        metrics.record_divergence(0.1);

        assert_eq!(metrics.max_divergence, 1.0);
        assert_eq!(metrics.final_divergence, 0.1);
        assert_eq!(metrics.check_count, 3);
    }

    #[test]
    fn test_throughput_metrics() {
        let mut metrics = ThroughputMetrics::new();

        for _ in 0..100 {
            metrics.record_operation();
        }

        for i in 0..50 {
            metrics.record_message(i % 10 == 0);
        }

        metrics.finalize(Duration::from_secs(1));

        assert_eq!(metrics.total_operations, 100);
        assert_eq!(metrics.ops_per_second, 100.0);
        assert_eq!(metrics.total_messages, 50);
        assert_eq!(metrics.dropped_messages, 5);
    }

    #[test]
    fn test_latency_percentiles() {
        let mut metrics = ThroughputMetrics::new();
        let mut latencies: Vec<f64> = (1..=100).map(f64::from).collect();

        metrics.calculate_latency_percentiles(&mut latencies);

        // Percentile calculation uses ceil, so p50 of 1-100 gives index 50 = value 51
        assert_eq!(metrics.latency_p50_ms, 51.0);
        assert_eq!(metrics.latency_p95_ms, 96.0);
        assert_eq!(metrics.latency_p99_ms, 100.0);
    }
}
