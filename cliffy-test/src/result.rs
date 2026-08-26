// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Test result types with geometric information
//!
//! Test results are not just pass/fail - they carry geometric error information
//! that helps diagnose and fix failures.

use crate::error::GeometricError;
use serde::{Deserialize, Serialize};

/// Result of a geometric test
///
/// Unlike boolean tests, geometric tests provide rich error information
/// when they fail, including distance from expected manifold and
/// correction vectors.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TestResult {
    /// Test passed - state lies on expected manifold
    Pass,

    /// Test failed with geometric error information
    Fail(GeometricError),

    /// Test was skipped (e.g., precondition not met)
    Skipped(String),
}

impl TestResult {
    /// Check if test passed
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        matches!(self, Self::Pass)
    }

    /// Check if test failed
    #[must_use]
    pub const fn is_fail(&self) -> bool {
        matches!(self, Self::Fail(_))
    }

    /// Get the error if test failed
    #[must_use]
    pub const fn error(&self) -> Option<&GeometricError> {
        match self {
            Self::Fail(e) => Some(e),
            _ => None,
        }
    }

    /// Create a pass result
    #[must_use]
    pub const fn pass() -> Self {
        Self::Pass
    }

    /// Create a fail result from geometric error
    #[must_use]
    pub const fn fail(error: GeometricError) -> Self {
        Self::Fail(error)
    }

    /// Create a fail result from distance and description
    pub fn fail_with_distance(distance: f64, description: impl Into<String>) -> Self {
        Self::Fail(GeometricError::new(distance, description))
    }

    /// Create a skipped result
    pub fn skipped(reason: impl Into<String>) -> Self {
        Self::Skipped(reason.into())
    }

    /// Convert to standard Result type for use with ?
    ///
    /// # Errors
    ///
    /// Returns the inner `GeometricError` for `Fail`; a `Skipped` result
    /// converts to an error carrying its skip reason.
    pub fn into_result(self) -> Result<(), GeometricError> {
        match self {
            Self::Pass => Ok(()),
            Self::Fail(e) => Err(e),
            Self::Skipped(reason) => Err(GeometricError::new(0.0, reason)),
        }
    }

    /// Combine multiple test results
    ///
    /// Returns Pass only if all results are Pass.
    /// Returns the first Fail if any failed.
    pub fn combine(results: impl IntoIterator<Item = Self>) -> Self {
        for result in results {
            match result {
                Self::Pass => {}
                Self::Fail(e) => return Self::Fail(e),
                Self::Skipped(reason) => return Self::Skipped(reason),
            }
        }
        Self::Pass
    }
}

impl From<bool> for TestResult {
    fn from(passed: bool) -> Self {
        if passed {
            Self::Pass
        } else {
            Self::Fail(GeometricError::new(1.0, "Boolean test failed"))
        }
    }
}

impl From<TestResult> for bool {
    fn from(result: TestResult) -> Self {
        result.is_pass()
    }
}

/// Result of running an invariant test suite
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvariantTestReport {
    /// Name of the invariant
    pub name: String,

    /// Category (impossible, rare, emergent)
    pub category: InvariantCategory,

    /// Number of samples tested
    pub samples: usize,

    /// Number of failures
    pub failures: usize,

    /// Observed failure rate (failures / samples)
    pub failure_rate: f64,

    /// Probability bound (for rare invariants)
    pub probability_bound: Option<f64>,

    /// Whether the invariant was verified
    pub verified: bool,

    /// Sample of failure errors (if any)
    pub sample_errors: Vec<GeometricError>,

    /// Confidence interval for the failure rate estimate (lower, upper)
    ///
    /// Computed using Hoeffding-based bounds from amari-flynn.
    /// Only populated for Rare invariants verified with Monte Carlo.
    pub confidence_interval: Option<(f64, f64)>,

    /// Confidence level of the statistical verification (0.0 to 1.0)
    ///
    /// Computed from the Hoeffding bound given sample count and epsilon.
    pub confidence_level: Option<f64>,
}

/// Category of invariant
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvariantCategory {
    /// Impossible (P = 0) - must never fail
    Impossible,

    /// Rare (0 < P << 1) - bounded failure probability
    Rare,

    /// Emergent (P > 0) - valid but unpredicted
    Emergent,
}

impl InvariantTestReport {
    /// Check if the invariant was violated
    #[must_use]
    pub fn is_violated(&self) -> bool {
        match self.category {
            InvariantCategory::Impossible => self.failures > 0,
            InvariantCategory::Rare => {
                if let Some(bound) = self.probability_bound {
                    self.failure_rate > bound
                } else {
                    self.failures > 0
                }
            }
            InvariantCategory::Emergent => false, // Emergent behaviors are never violations
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    // Exactly-representable constants: bit equality is the point.
    // (Stable clippy lacks the test-context exemption newer nightly has.)
    use super::*;

    #[test]
    fn test_result_pass() {
        let result = TestResult::pass();
        assert!(result.is_pass());
        assert!(!result.is_fail());
    }

    #[test]
    fn test_result_fail() {
        let result = TestResult::fail_with_distance(0.5, "Too far");
        assert!(!result.is_pass());
        assert!(result.is_fail());
        assert_eq!(result.error().unwrap().distance, 0.5);
    }

    #[test]
    fn test_result_combine() {
        let results = vec![TestResult::Pass, TestResult::Pass, TestResult::Pass];
        assert!(TestResult::combine(results).is_pass());

        let results_with_fail = vec![
            TestResult::Pass,
            TestResult::fail_with_distance(0.1, "Error"),
            TestResult::Pass,
        ];
        assert!(TestResult::combine(results_with_fail).is_fail());
    }

    #[test]
    fn test_from_bool() {
        let pass: TestResult = true.into();
        assert!(pass.is_pass());

        let fail: TestResult = false.into();
        assert!(fail.is_fail());
    }
}
