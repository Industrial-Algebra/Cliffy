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
//! use cliffy_protocols::lattice::{GeometricLattice, GA3Lattice};
//! use cliffy_core::GA3;
//!
//! // Create two conflicting states
//! let state_a = GA3Lattice::from_scalar(1.0);
//! let state_b = GA3Lattice::from_scalar(2.0);
//!
//! // Join always produces a consistent result
//! let joined = state_a.join(&state_b);
//! assert!(joined.dominates(&state_a));
//! assert!(joined.dominates(&state_b));
//! ```

#[allow(deprecated)]
// imports the deprecated geometric_mean for the GA3Lattice tie-break; dies with Phase 1
use crate::geometric_mean;
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

/// A lattice over raw `GA3` multivectors using a magnitude-dominance join
/// (dominance by magnitude; the exp-based geometric mean as tie-break).
///
/// # Deprecated — the join is not a join-semilattice
///
/// The magnitude-dominance rule with the exp-based tie-break violates the
/// hull property: `join(+1, -1)` returns `cosh(1) ≈ 1.543`, outside the
/// hull of its arguments, and the tie-break mean is non-associative — so
/// this cannot form a join-semilattice and cannot back strong eventual
/// consistency. See `docs/plans/2026-08-26-geometric-crdt-salvage.md`.
/// Use `ComponentLattice` (componentwise max/min) for per-grade state —
/// the boring, correct floor.
#[derive(Debug, Clone, PartialEq)]
#[deprecated(
    since = "0.3.2",
    note = "magnitude-dominance join violates the join-semilattice hull property; use ComponentLattice — see docs/plans/2026-08-26-geometric-crdt-salvage.md"
)]
pub struct GA3Lattice {
    inner: GA3,
}

#[allow(deprecated)] // inherent constructors reference the deprecated Self; dies with Phase 1
impl GA3Lattice {
    /// Create a new lattice element from a multivector.
    pub fn new(mv: GA3) -> Self {
        Self { inner: mv }
    }

    /// Create a lattice element from a scalar.
    pub fn from_scalar(value: f64) -> Self {
        Self::new(GA3::scalar(value))
    }

    /// Create a lattice element from vector components.
    pub fn from_vector(x: f64, y: f64, z: f64) -> Self {
        use amari_core::Vector;
        let v = Vector::<3, 0, 0>::from_components(x, y, z);
        Self::new(GA3::from_vector(&v))
    }

    /// Create the zero element (bottom of the lattice).
    pub fn zero() -> Self {
        Self::new(GA3::zero())
    }

    /// Get the underlying multivector.
    pub fn as_multivector(&self) -> &GA3 {
        &self.inner
    }

    /// Consume and return the underlying multivector.
    pub fn into_multivector(self) -> GA3 {
        self.inner
    }

    /// Get the magnitude of this lattice element.
    pub fn magnitude(&self) -> f64 {
        self.inner.magnitude()
    }

    /// Get a coefficient at the given index.
    pub fn get(&self, index: usize) -> f64 {
        self.inner.get(index)
    }
}

/// # Deprecated
///
/// See the struct-level deprecation: the magnitude-dominance rule with the
/// exp-based tie-break is not a join-semilattice (`join(+1,-1) = cosh(1)`).
#[allow(deprecated)] // all trait methods construct/read the deprecated Self; dies with Phase 1
impl GeometricLattice for GA3Lattice {
    fn join(&self, other: &Self) -> Self {
        // Check for structural equality first (idempotence optimization)
        if self.divergence(other) < 1e-10 {
            return self.clone();
        }

        let self_mag = self.inner.magnitude();
        let other_mag = other.inner.magnitude();

        // Dominance by magnitude
        if self_mag > other_mag + 1e-10 {
            self.clone()
        } else if other_mag > self_mag + 1e-10 {
            other.clone()
        } else {
            // Equal magnitudes but different states - use geometric mean
            Self::new(geometric_mean(&[self.inner.clone(), other.inner.clone()]))
        }
    }

    fn dominates(&self, other: &Self) -> bool {
        // A dominates B if magnitude(A) >= magnitude(B)
        // This creates a total order for non-zero states
        self.inner.magnitude() >= other.inner.magnitude() - 1e-10
    }

    fn divergence(&self, other: &Self) -> f64 {
        (&self.inner - &other.inner).magnitude()
    }

    fn meet(&self, other: &Self) -> Option<Self> {
        let self_mag = self.inner.magnitude();
        let other_mag = other.inner.magnitude();

        // Meet is the element with smaller magnitude
        if self_mag < other_mag + 1e-10 {
            Some(self.clone())
        } else if other_mag < self_mag + 1e-10 {
            Some(other.clone())
        } else {
            // Equal magnitudes - meet exists and equals both
            Some(self.clone())
        }
    }
}

/// Component-wise lattice operations for multivectors.
///
/// Unlike `GA3Lattice` which uses magnitude ordering, this provides
/// coefficient-by-coefficient join/meet operations.
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentLattice {
    inner: GA3,
}

impl ComponentLattice {
    /// Create a new component lattice element.
    pub fn new(mv: GA3) -> Self {
        Self { inner: mv }
    }

    /// Create from a scalar value.
    pub fn from_scalar(value: f64) -> Self {
        Self::new(GA3::scalar(value))
    }

    /// Get the underlying multivector.
    pub fn as_multivector(&self) -> &GA3 {
        &self.inner
    }

    /// Consume and return the underlying multivector.
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
            .fold(0.0_f64, |acc, d| acc.max(d))
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
    #![allow(deprecated)]
    // GA3Lattice::join tests pin the deprecated magnitude-dominance behavior
    // until Phase 1 of the salvage plan; ComponentLattice tests below are the
    // sound floor and unaffected.
    use super::*;

    #[test]
    fn test_ga3_lattice_idempotent() {
        let a = GA3Lattice::from_scalar(5.0);
        let joined = a.join(&a);
        assert!(a.lattice_eq(&joined));
    }

    #[test]
    fn test_ga3_lattice_commutative() {
        let a = GA3Lattice::from_scalar(3.0);
        let b = GA3Lattice::from_scalar(7.0);

        let ab = a.join(&b);
        let ba = b.join(&a);

        assert!(ab.lattice_eq(&ba));
    }

    #[test]
    fn test_ga3_lattice_associative() {
        let a = GA3Lattice::from_scalar(2.0);
        let b = GA3Lattice::from_scalar(5.0);
        let c = GA3Lattice::from_scalar(3.0);

        let ab_c = a.join(&b).join(&c);
        let a_bc = a.join(&b.join(&c));

        assert!(ab_c.lattice_eq(&a_bc));
    }

    #[test]
    fn test_ga3_lattice_dominance() {
        let small = GA3Lattice::from_scalar(2.0);
        let large = GA3Lattice::from_scalar(5.0);

        assert!(large.dominates(&small));
        assert!(!small.dominates(&large));
    }

    #[test]
    fn test_ga3_lattice_divergence() {
        let a = GA3Lattice::from_scalar(3.0);
        let b = GA3Lattice::from_scalar(7.0);

        let div = a.divergence(&b);
        assert!((div - 4.0).abs() < 1e-10);
    }

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
        let initial = GA3Lattice::from_scalar(1.0);

        // Two nodes make concurrent updates
        let node1_update = GA3Lattice::from_scalar(5.0);
        let node2_update = GA3Lattice::from_scalar(3.0);

        // Both nodes join with initial
        let node1_state = initial.join(&node1_update);
        let node2_state = initial.join(&node2_update);

        // Cross-sync: both should converge to the same state
        let node1_final = node1_state.join(&node2_state);
        let node2_final = node2_state.join(&node1_state);

        assert!(node1_final.lattice_eq(&node2_final));
    }
}
