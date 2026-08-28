// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Lattice-based conflict resolution using geometric algebra
//!
//! This module provides the `GeometricLattice` trait for join-semilattice operations
//! that enable coordination-free conflict resolution in distributed systems.
//!
//! # Key Properties
//!
//! A join-semilattice must satisfy:
//! - **Idempotent**: `a ⊔ a = a`
//! - **Commutative**: `a ⊔ b = b ⊔ a`
//! - **Associative**: `(a ⊔ b) ⊔ c = a ⊔ (b ⊔ c)`
//!
//! These properties ensure that replicas always converge regardless of message
//! ordering or network partitions.
//!
//! # Example
//!
//! ```rust
//! use cliffy_protocols::lattice::{ComponentLattice, GeometricLattice};
//! use cliffy_core::GA3;
//!
//! // Create two conflicting states
//! let state_a = ComponentLattice::from_scalar(1.0);
//! let state_b = ComponentLattice::from_scalar(2.0);
//!
//! // Join always produces a consistent result
//! let joined = state_a.join(&state_b);
//! assert!(joined.dominates(&state_a));
//! assert!(joined.dominates(&state_b));
//! ```

use cliffy_core::GA3;

/// A join-semilattice with geometric algebra operations.
///
/// This trait provides the mathematical foundation for CRDTs:
/// - `join` computes the least upper bound (always converges)
/// - `dominates` checks causal ordering
/// - `divergence` measures conflict severity
pub trait GeometricLattice: Clone {
    /// Lattice join (least upper bound) - always converges, no coordination needed.
    ///
    /// The join operation must be idempotent, commutative, and associative.
    #[must_use]
    fn join(&self, other: &Self) -> Self;

    /// Check if this state dominates (is greater than or equal to) another.
    ///
    /// Returns true if `other ⊔ self = self`.
    fn dominates(&self, other: &Self) -> bool;

    /// Compute the geometric distance/divergence from another state.
    ///
    /// This measures how "far apart" two states are, useful for:
    /// - Detecting conflicts
    /// - Measuring convergence progress
    /// - Prioritizing sync operations
    fn divergence(&self, other: &Self) -> f64;

    /// Check if two states are equal in the lattice ordering.
    fn lattice_eq(&self, other: &Self) -> bool {
        self.dominates(other) && other.dominates(self)
    }

    /// Compute the lattice meet (greatest lower bound) if it exists.
    ///
    /// Not all semilattices have meets, so this returns Option.
    fn meet(&self, other: &Self) -> Option<Self>;
}

/// The componentwise lattice floor: a genuine join-semilattice providing
/// coefficient-by-coefficient join/meet operations.
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentLattice {
    inner: GA3,
}

impl ComponentLattice {
    /// Create a new component lattice element.
    #[must_use]
    pub const fn new(mv: GA3) -> Self {
        Self { inner: mv }
    }

    /// Create from a scalar value.
    #[must_use]
    pub fn from_scalar(value: f64) -> Self {
        Self::new(GA3::scalar(value))
    }

    /// Get the underlying multivector.
    #[must_use]
    pub const fn as_multivector(&self) -> &GA3 {
        &self.inner
    }

    /// Consume and return the underlying multivector.
    #[must_use]
    pub fn into_multivector(self) -> GA3 {
        self.inner
    }
}

impl GeometricLattice for ComponentLattice {
    fn join(&self, other: &Self) -> Self {
        // Component-wise maximum
        let mut coeffs = Vec::with_capacity(8);
        for i in 0..8 {
            coeffs.push(self.inner.get(i).max(other.inner.get(i)));
        }
        Self::new(GA3::from_coefficients(coeffs))
    }

    fn dominates(&self, other: &Self) -> bool {
        // Dominates if every component is >= the corresponding component
        (0..8).all(|i| self.inner.get(i) >= other.inner.get(i) - 1e-10)
    }

    fn divergence(&self, other: &Self) -> f64 {
        // L-infinity norm (max component difference)
        (0..8)
            .map(|i| (self.inner.get(i) - other.inner.get(i)).abs())
            .fold(0.0_f64, f64::max)
    }

    fn meet(&self, other: &Self) -> Option<Self> {
        // Component-wise minimum
        let mut coeffs = Vec::with_capacity(8);
        for i in 0..8 {
            coeffs.push(self.inner.get(i).min(other.inner.get(i)));
        }
        Some(Self::new(GA3::from_coefficients(coeffs)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_lattice_idempotent() {
        let a = ComponentLattice::from_scalar(5.0);
        let joined = a.join(&a);
        assert!(a.lattice_eq(&joined));
    }

    #[test]
    fn test_component_lattice_commutative() {
        let a = ComponentLattice::new(GA3::from_coefficients(vec![
            1.0, 2.0, 3.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ]));
        let b = ComponentLattice::new(GA3::from_coefficients(vec![
            2.0, 1.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ]));

        let ab = a.join(&b);
        let ba = b.join(&a);

        assert!(ab.lattice_eq(&ba));
    }

    #[test]
    fn test_component_lattice_join_max() {
        let a = ComponentLattice::new(GA3::from_coefficients(vec![
            1.0, 2.0, 3.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ]));
        let b = ComponentLattice::new(GA3::from_coefficients(vec![
            2.0, 1.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ]));

        let joined = a.join(&b);

        assert!((joined.as_multivector().scalar_part() - 2.0).abs() < 1e-10);
        assert!((joined.as_multivector().get(1) - 2.0).abs() < 1e-10);
        assert!((joined.as_multivector().get(2) - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_component_lattice_meet_min() {
        let a = ComponentLattice::new(GA3::from_coefficients(vec![
            1.0, 2.0, 3.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ]));
        let b = ComponentLattice::new(GA3::from_coefficients(vec![
            2.0, 1.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ]));

        let met = a.meet(&b).unwrap();

        assert!((met.as_multivector().scalar_part() - 1.0).abs() < 1e-10);
        assert!((met.as_multivector().get(1) - 1.0).abs() < 1e-10);
        assert!((met.as_multivector().get(2) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_lattice_convergence() {
        // Simulate distributed updates converging
        let initial = ComponentLattice::from_scalar(1.0);

        // Two nodes make concurrent updates
        let node1_update = ComponentLattice::from_scalar(5.0);
        let node2_update = ComponentLattice::from_scalar(3.0);

        // Both nodes join with initial
        let node1_state = initial.join(&node1_update);
        let node2_state = initial.join(&node2_update);

        // Cross-sync: both should converge to the same state
        let node1_final = node1_state.join(&node2_state);
        let node2_final = node2_state.join(&node1_state);

        assert!(node1_final.lattice_eq(&node2_final));
    }
}
