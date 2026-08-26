// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! # Cliffy Test - Algebraic Testing Framework
//!
//! Tests are geometric invariants. Failures are geometric distances.
//! Test composition uses geometric product.
//!
//! ## Philosophy
//!
//! Traditional testing asks: "Does this assertion pass?"
//! Cliffy testing asks: "Does this state lie on the expected manifold?"
//!
//! By integrating with `amari-flynn`, we categorize test events into three types:
//!
//! | Category | Probability | Meaning |
//! |----------|-------------|---------|
//! | **Impossible** | P = 0 | Formally proven to never occur |
//! | **Rare** | 0 < P << 1 | Statistically bounded failure rate |
//! | **Emergent** | P > 0 | Valid but unpredicted behaviors |
//!
//! ## Quick Start
//!
//! ```rust
//! use cliffy_test::prelude::*;
//!
//! // Define an impossible invariant - must NEVER fail
//! let inv = invariant_impossible! {
//!     name: "Magnitude is non-negative",
//!     check: || {
//!         let v = vector(1.0, 2.0, 3.0);
//!         if v.magnitude() >= 0.0 {
//!             TestResult::Pass
//!         } else {
//!             TestResult::fail_with_distance(v.magnitude(), "Negative magnitude")
//!         }
//!     }
//! };
//! ```

#![warn(missing_docs)]

pub mod error;
pub mod generators;
pub mod invariants;
pub mod macros;
pub mod manifold;
pub mod result;

// Re-export amari types
pub use amari_core::{Bivector, Multivector, Vector};

// Re-export amari-flynn for probabilistic contracts
pub use amari_flynn;

/// Type alias for 3D Euclidean geometric algebra
pub type GA3 = Multivector<3, 0, 0>;

/// Type alias for 3D Vector
pub type Vec3 = Vector<3, 0, 0>;

/// Type alias for 3D Bivector
pub type Biv3 = Bivector<3, 0, 0>;

/// Convenience function to create a GA3 vector from components
#[must_use]
pub fn vector(x: f64, y: f64, z: f64) -> GA3 {
    GA3::from_vector(&Vec3::from_components(x, y, z))
}

/// Convenience function to create a GA3 bivector from components
#[must_use]
pub fn bivector(xy: f64, xz: f64, yz: f64) -> GA3 {
    GA3::from_bivector(&Biv3::from_components(xy, xz, yz))
}

/// Convenience function to create a GA3 from coefficient array
#[must_use]
pub fn from_coeffs(coeffs: [f64; 8]) -> GA3 {
    GA3::from_coefficients(coeffs.to_vec())
}

/// Sandwich product: r * v * `r.reverse()`
///
/// This is the fundamental operation for geometric transformations.
/// When r is a rotor (unit versor), this preserves magnitude.
#[must_use]
pub fn sandwich(rotor: &GA3, value: &GA3) -> GA3 {
    rotor
        .geometric_product(value)
        .geometric_product(&rotor.reverse())
}

/// Prelude module for common imports
pub mod prelude {
    // Core types
    pub use crate::error::GeometricError;
    pub use crate::result::TestResult;

    // Invariant types
    pub use crate::invariants::{EmergentBehavior, ImpossibleInvariant, Invariant, RareInvariant};

    // Manifold testing
    pub use crate::manifold::{Manifold, ManifoldConstraint};

    // Generators for property testing
    pub use crate::generators::{
        arbitrary_ga3, arbitrary_rotor, arbitrary_unit_vector, arbitrary_vector,
    };

    // Convenience constructors
    pub use crate::{bivector, from_coeffs, sandwich, vector, Biv3, Vec3, GA3};

    // Macros
    pub use crate::{emergent, invariant_impossible, invariant_rare};

    // Re-export amari types
    pub use amari_core::{Bivector, Multivector, Vector};

    // Re-export amari-flynn types
    pub use amari_flynn::prelude::*;

    // SMT proof obligations for formal verification
    pub use amari_flynn::backend::smt::SmtProofObligation;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_creation() {
        let v = vector(1.0, 2.0, 3.0);
        assert!((v.magnitude() - 14.0_f64.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn test_sandwich_identity() {
        let v = vector(1.0, 2.0, 3.0);
        let identity = GA3::scalar(1.0);
        let result = sandwich(&identity, &v);

        // Identity sandwich should return the same vector
        assert!((v.magnitude() - result.magnitude()).abs() < 1e-10);
    }

    #[test]
    fn test_bivector_creation() {
        let b = bivector(1.0, 0.0, 0.0);
        assert!(b.magnitude() > 0.0);
    }
}
