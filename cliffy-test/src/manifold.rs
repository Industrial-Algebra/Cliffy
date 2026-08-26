// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Manifold testing - verify states lie on expected geometric manifolds
//!
//! Valid states form geometric manifolds. Testing verifies that state
//! transformations keep states on the manifold.

use crate::error::GeometricError;
use crate::result::TestResult;
use crate::{bivector, vector, GA3};

/// A constraint on a geometric manifold
pub trait ManifoldConstraint: Send + Sync {
    /// Check if a point satisfies this constraint
    fn satisfied(&self, point: &GA3) -> bool;

    /// Calculate distance from this constraint
    fn distance(&self, point: &GA3) -> f64;

    /// Project a point onto the constraint surface
    fn project(&self, point: &GA3) -> GA3;

    /// Description of this constraint
    fn description(&self) -> &str;
}

/// A geometric manifold defined by constraints
pub struct Manifold {
    /// Name of the manifold
    pub name: String,
    /// Constraints defining the manifold
    constraints: Vec<Box<dyn ManifoldConstraint>>,
    /// Tolerance for membership tests
    pub tolerance: f64,
}

impl Manifold {
    /// Create a new empty manifold
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            constraints: Vec::new(),
            tolerance: 1e-10,
        }
    }

    /// Set the tolerance for membership tests
    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance;
        self
    }

    /// Add a constraint to the manifold
    pub fn with_constraint(mut self, constraint: impl ManifoldConstraint + 'static) -> Self {
        self.constraints.push(Box::new(constraint));
        self
    }

    /// Check if a point lies on the manifold
    pub fn contains(&self, point: &GA3) -> bool {
        self.constraints
            .iter()
            .all(|c| c.distance(point) < self.tolerance)
    }

    /// Calculate distance from the manifold
    pub fn distance(&self, point: &GA3) -> f64 {
        self.constraints
            .iter()
            .map(|c| c.distance(point))
            .fold(0.0, f64::max)
    }

    /// Project a point onto the manifold
    ///
    /// Iteratively projects through all constraints.
    /// Note: This is an approximation for non-convex manifolds.
    pub fn project(&self, point: &GA3) -> GA3 {
        let mut result = point.clone();
        for _ in 0..10 {
            // Max iterations
            let mut changed = false;
            for constraint in &self.constraints {
                if !constraint.satisfied(&result) {
                    result = constraint.project(&result);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        result
    }

    /// Verify that a point lies on the manifold
    pub fn verify(&self, point: &GA3) -> TestResult {
        let dist = self.distance(point);
        if dist < self.tolerance {
            TestResult::Pass
        } else {
            let projected = self.project(point);
            let error_mv = &projected - point;
            let mut error = GeometricError::from_multivector(
                &error_mv,
                format!("Point not on manifold '{}'", self.name),
            );
            error.distance = dist;
            TestResult::Fail(error)
        }
    }
}

/// Constraint: magnitude equals a specific value
pub struct MagnitudeConstraint {
    target: f64,
}

impl MagnitudeConstraint {
    /// Create a magnitude constraint
    pub fn new(target: f64) -> Self {
        Self { target }
    }

    /// Create a unit magnitude constraint (magnitude = 1)
    pub fn unit() -> Self {
        Self { target: 1.0 }
    }
}

impl ManifoldConstraint for MagnitudeConstraint {
    fn satisfied(&self, point: &GA3) -> bool {
        (point.magnitude() - self.target).abs() < 1e-10
    }

    fn distance(&self, point: &GA3) -> f64 {
        (point.magnitude() - self.target).abs()
    }

    fn project(&self, point: &GA3) -> GA3 {
        let mag = point.magnitude();
        if mag > 1e-10 {
            point * (self.target / mag)
        } else {
            // Degenerate - can't normalize zero
            point.clone()
        }
    }

    fn description(&self) -> &str {
        "Magnitude constraint"
    }
}

/// Constraint: scalar part equals a specific value
pub struct ScalarConstraint {
    target: f64,
}

impl ScalarConstraint {
    /// Create a scalar constraint
    pub fn new(target: f64) -> Self {
        Self { target }
    }

    /// Create a zero scalar constraint
    pub fn zero() -> Self {
        Self { target: 0.0 }
    }
}

impl ManifoldConstraint for ScalarConstraint {
    fn satisfied(&self, point: &GA3) -> bool {
        (point.get(0) - self.target).abs() < 1e-10
    }

    fn distance(&self, point: &GA3) -> f64 {
        (point.get(0) - self.target).abs()
    }

    fn project(&self, point: &GA3) -> GA3 {
        // Create new multivector with modified scalar
        let mut coeffs: Vec<f64> = (0..8).map(|i| point.get(i)).collect();
        coeffs[0] = self.target; // Set scalar to target
        GA3::from_coefficients(coeffs)
    }

    fn description(&self) -> &str {
        "Scalar constraint"
    }
}

/// Constraint: point is a pure vector (only grade 1 components)
///
/// In GA3 basis indices:
/// - 0 = scalar, 1 = e1, 2 = e2, 3 = e12, 4 = e3, 5 = e13, 6 = e23, 7 = e123
/// - Vector components are at indices 1, 2, 4 (e1, e2, e3)
pub struct PureVectorConstraint;

impl ManifoldConstraint for PureVectorConstraint {
    fn satisfied(&self, point: &GA3) -> bool {
        let s = point.get(0).abs(); // scalar
        let b12 = point.get(3).abs(); // e12
        let b13 = point.get(5).abs(); // e13
        let b23 = point.get(6).abs(); // e23
        let tri = point.get(7).abs(); // e123
        s < 1e-10 && b12 < 1e-10 && b13 < 1e-10 && b23 < 1e-10 && tri < 1e-10
    }

    fn distance(&self, point: &GA3) -> f64 {
        let s = point.get(0);
        let b12 = point.get(3);
        let b13 = point.get(5);
        let b23 = point.get(6);
        let tri = point.get(7);
        (s * s + b12 * b12 + b13 * b13 + b23 * b23 + tri * tri).sqrt()
    }

    fn project(&self, point: &GA3) -> GA3 {
        // Extract only vector components (e1, e2, e3 at indices 1, 2, 4)
        vector(point.get(1), point.get(2), point.get(4))
    }

    fn description(&self) -> &str {
        "Pure vector constraint"
    }
}

/// Constraint: point is a pure bivector (only grade 2 components)
///
/// In GA3 basis indices:
/// - Bivector components are at indices 3, 5, 6 (e12, e13, e23)
pub struct PureBivectorConstraint;

impl ManifoldConstraint for PureBivectorConstraint {
    fn satisfied(&self, point: &GA3) -> bool {
        let s = point.get(0).abs(); // scalar
        let v1 = point.get(1).abs(); // e1
        let v2 = point.get(2).abs(); // e2
        let v3 = point.get(4).abs(); // e3
        let tri = point.get(7).abs(); // e123
        s < 1e-10 && v1 < 1e-10 && v2 < 1e-10 && v3 < 1e-10 && tri < 1e-10
    }

    fn distance(&self, point: &GA3) -> f64 {
        let s = point.get(0);
        let v1 = point.get(1);
        let v2 = point.get(2);
        let v3 = point.get(4);
        let tri = point.get(7);
        (s * s + v1 * v1 + v2 * v2 + v3 * v3 + tri * tri).sqrt()
    }

    fn project(&self, point: &GA3) -> GA3 {
        // Extract only bivector components (e12, e13, e23 at indices 3, 5, 6)
        bivector(point.get(3), point.get(5), point.get(6))
    }

    fn description(&self) -> &str {
        "Pure bivector constraint"
    }
}

/// Create the unit sphere manifold (vectors with magnitude 1)
pub fn unit_sphere() -> Manifold {
    Manifold::new("Unit Sphere")
        .with_constraint(PureVectorConstraint)
        .with_constraint(MagnitudeConstraint::unit())
}

/// Create the rotor manifold (even-grade elements with unit magnitude)
pub fn rotor_manifold() -> Manifold {
    Manifold::new("Rotor Manifold").with_constraint(MagnitudeConstraint::unit())
    // Note: Rotors are even-grade (scalar + bivector), but verifying
    // the even-grade constraint is more complex in practice
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magnitude_constraint() {
        let constraint = MagnitudeConstraint::unit();
        let unit_vec = vector(1.0, 0.0, 0.0);
        assert!(constraint.satisfied(&unit_vec));

        let non_unit = vector(2.0, 0.0, 0.0);
        assert!(!constraint.satisfied(&non_unit));

        let projected = constraint.project(&non_unit);
        assert!(constraint.satisfied(&projected));
    }

    #[test]
    fn test_manifold_contains() {
        let manifold = unit_sphere();

        let on_sphere = vector(1.0, 0.0, 0.0);
        assert!(manifold.contains(&on_sphere));

        let off_sphere = vector(2.0, 0.0, 0.0);
        assert!(!manifold.contains(&off_sphere));
    }

    #[test]
    fn test_manifold_project() {
        let manifold = unit_sphere();

        let off_sphere = vector(3.0, 4.0, 0.0);
        let projected = manifold.project(&off_sphere);

        assert!(
            manifold.contains(&projected),
            "Projected point should be on sphere"
        );
        assert!(
            (projected.magnitude() - 1.0).abs() < 1e-10,
            "Projected magnitude should be 1"
        );
    }

    #[test]
    fn test_manifold_verify() {
        let manifold = unit_sphere();

        let on = vector(0.0, 1.0, 0.0);
        assert!(manifold.verify(&on).is_pass());

        let off = vector(0.0, 2.0, 0.0);
        let result = manifold.verify(&off);
        assert!(result.is_fail());

        if let TestResult::Fail(error) = result {
            assert!(error.distance > 0.0);
        }
    }
}
