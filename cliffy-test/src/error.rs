// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Geometric error types for test failures
//!
//! When tests fail, they provide geometric information about the failure:
//! - Distance from expected manifold
//! - Gradient pointing toward valid states
//! - Projected correction to nearest valid state

use crate::{vector, GA3};
use serde::{Deserialize, Serialize};

/// Geometric error information for test failures
///
/// Unlike boolean test failures, geometric errors tell you:
/// 1. How far the state is from being valid (distance)
/// 2. Which direction to move to become valid (gradient)
/// 3. What the nearest valid state would be (correction)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeometricError {
    /// Distance from expected manifold (scalar)
    pub distance: f64,

    /// Gradient pointing toward valid states
    ///
    /// This is the direction of steepest descent toward the manifold.
    /// Stored as coefficients [e1, e2, e3] for the vector part.
    pub gradient: [f64; 3],

    /// Projected correction to nearest valid state
    ///
    /// If you add this to the invalid state, you get the nearest valid state.
    /// Stored as full multivector coefficients.
    pub correction: [f64; 8],

    /// Human-readable description of the error
    pub description: String,
}

impl GeometricError {
    /// Create a new geometric error
    pub fn new(distance: f64, description: impl Into<String>) -> Self {
        Self {
            distance,
            gradient: [0.0; 3],
            correction: [0.0; 8],
            description: description.into(),
        }
    }

    /// Create error with gradient information
    pub fn with_gradient(mut self, gradient: [f64; 3]) -> Self {
        self.gradient = gradient;
        self
    }

    /// Create error with correction information
    pub fn with_correction(mut self, correction: [f64; 8]) -> Self {
        self.correction = correction;
        self
    }

    /// Create error from a GA3 multivector representing the error
    ///
    /// Note: In GA3, basis blade indices use binary representation:
    /// - 0 = scalar, 1 = e1, 2 = e2, 3 = e12, 4 = e3, 5 = e13, 6 = e23, 7 = e123
    pub fn from_multivector(error_mv: &GA3, description: impl Into<String>) -> Self {
        Self {
            distance: error_mv.magnitude(),
            gradient: [
                error_mv.get(1), // e1 (0b001)
                error_mv.get(2), // e2 (0b010)
                error_mv.get(4), // e3 (0b100)
            ],
            correction: [
                error_mv.get(0), // scalar
                error_mv.get(1), // e1
                error_mv.get(2), // e2
                error_mv.get(4), // e3
                error_mv.get(3), // e12
                error_mv.get(5), // e13
                error_mv.get(6), // e23
                error_mv.get(7), // e123
            ],
            description: description.into(),
        }
    }

    /// Check if this error is within tolerance
    pub fn is_within_tolerance(&self, epsilon: f64) -> bool {
        self.distance < epsilon
    }

    /// Get the gradient as a GA3 vector
    pub fn gradient_as_ga3(&self) -> GA3 {
        vector(self.gradient[0], self.gradient[1], self.gradient[2])
    }

    /// Get the correction as a GA3 multivector
    pub fn correction_as_ga3(&self) -> GA3 {
        GA3::from_coefficients(self.correction.to_vec())
    }
}

impl std::fmt::Display for GeometricError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GeometricError {{ distance: {:.6}, description: \"{}\" }}",
            self.distance, self.description
        )
    }
}

impl std::error::Error for GeometricError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = GeometricError::new(0.5, "Test error");
        assert_eq!(error.distance, 0.5);
        assert_eq!(error.description, "Test error");
    }

    #[test]
    fn test_error_from_multivector() {
        let mv = vector(1.0, 2.0, 3.0);
        let error = GeometricError::from_multivector(&mv, "Vector error");

        assert!((error.distance - mv.magnitude()).abs() < 1e-10);
        assert_eq!(error.gradient[0], 1.0);
        assert_eq!(error.gradient[1], 2.0);
        assert_eq!(error.gradient[2], 3.0);
    }

    #[test]
    fn test_within_tolerance() {
        let error = GeometricError::new(0.001, "Small error");
        assert!(error.is_within_tolerance(0.01));
        assert!(!error.is_within_tolerance(0.0001));
    }
}
