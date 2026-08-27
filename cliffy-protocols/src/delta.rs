// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! State delta computation for efficient synchronization
//!
//! This module provides geometric delta computation between states,
//! enabling bandwidth-efficient synchronization in distributed systems.
//!
//! # Key Concepts
//!
//! - **Delta**: The minimal transformation to go from one state to another
//! - **Compression**: Represent deltas in log space for smaller payloads
//! - **Batching**: Combine multiple deltas into single compound transformations
//!
//! # Example
//!
//! ```rust
//! use cliffy_protocols::delta::{compute_delta, apply_additive_delta};
//! use cliffy_core::GA3;
//!
//! let from = GA3::scalar(1.0);
//! let to = GA3::scalar(5.0);
//!
//! // Compute the delta between states
//! let delta = compute_delta(&from, &to);
//!
//! // Apply delta to reconstruct target state
//! let mut state = from.clone();
//! apply_additive_delta(&mut state, &delta);
//!
//! assert!((state.scalar_part() - to.scalar_part()).abs() < 1e-10);
//! ```

use crate::serde_ga3;
use crate::VectorClock;
use cliffy_core::GA3;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A state delta representing the transformation between two states.
///
/// Deltas can be represented in different forms for efficiency:
/// - `Additive`: Simple difference (to - from)
/// - `Multiplicative`: Versor/rotor transformation
/// - `Compressed`: Log-space representation for smaller payloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDelta {
    /// The delta transformation
    #[serde(with = "serde_ga3")]
    pub transform: GA3,
    /// Type of delta encoding
    pub encoding: DeltaEncoding,
    /// Source state clock (for causal ordering)
    pub from_clock: VectorClock,
    /// Target state clock
    pub to_clock: VectorClock,
    /// Node that computed this delta
    pub source_node: Uuid,
}

/// How the delta is encoded
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeltaEncoding {
    /// Simple additive delta: result = state + delta
    Additive,
    /// Multiplicative (sandwich) delta: result = delta * state * reverse(delta)
    Multiplicative,
    /// Compressed log-space: result = exp(delta) * state
    Compressed,
}

impl StateDelta {
    /// Create a new additive delta.
    #[must_use]
    pub const fn additive(
        transform: GA3,
        from_clock: VectorClock,
        to_clock: VectorClock,
        source_node: Uuid,
    ) -> Self {
        Self {
            transform,
            encoding: DeltaEncoding::Additive,
            from_clock,
            to_clock,
            source_node,
        }
    }

    /// Create a new multiplicative (versor) delta.
    #[must_use]
    pub const fn multiplicative(
        transform: GA3,
        from_clock: VectorClock,
        to_clock: VectorClock,
        source_node: Uuid,
    ) -> Self {
        Self {
            transform,
            encoding: DeltaEncoding::Multiplicative,
            from_clock,
            to_clock,
            source_node,
        }
    }

    /// Create a compressed (log-space) delta.
    #[must_use]
    pub const fn compressed(
        transform: GA3,
        from_clock: VectorClock,
        to_clock: VectorClock,
        source_node: Uuid,
    ) -> Self {
        Self {
            transform,
            encoding: DeltaEncoding::Compressed,
            from_clock,
            to_clock,
            source_node,
        }
    }

    /// Get the approximate size of this delta in bytes (for bandwidth estimation).
    #[must_use]
    pub const fn estimated_size(&self) -> usize {
        // 8 coefficients * 8 bytes each + overhead
        8 * 8 + 32
    }

    /// Check if this delta is causally applicable to a state with the given clock.
    #[must_use]
    pub fn is_applicable_to(&self, state_clock: &VectorClock) -> bool {
        // Delta is applicable if state_clock >= from_clock
        self.from_clock.happens_before(state_clock) || self.from_clock == *state_clock
    }
}

/// Compute the delta between two states.
///
/// Returns an additive delta by default. Use `compute_delta_compressed`
/// for log-space representation.
#[must_use]
pub fn compute_delta(from: &GA3, to: &GA3) -> GA3 {
    to - from
}

/// Compute a compressed (log-space) delta.
///
/// This representation is more compact for states that differ by
/// multiplicative factors rather than additive differences.
#[must_use]
pub fn compute_delta_compressed(from: &GA3, to: &GA3) -> GA3 {
    // For compressed representation, we compute log(to) - log(from)
    // which can be applied as exp(delta) * from
    //
    // For now, we use a simplified version that works well for
    // scalar-dominated multivectors
    let from_mag = from.magnitude();
    let to_mag = to.magnitude();

    if from_mag < 1e-10 {
        // Can't compute log of zero - fall back to additive
        return to - from;
    }

    // Compute the ratio and take log
    let ratio = to_mag / from_mag;
    let log_ratio = ratio.ln();

    // Return as scalar multivector (simplified)
    GA3::scalar(log_ratio)
}

/// Apply a delta to a state, modifying it in place.
pub fn apply_delta(state: &mut GA3, delta: &StateDelta) {
    match delta.encoding {
        DeltaEncoding::Additive => {
            *state = &*state + &delta.transform;
        }
        DeltaEncoding::Multiplicative => {
            // Sandwich product: delta * state * reverse(delta)
            let rev = delta.transform.reverse();
            *state = delta
                .transform
                .geometric_product(state)
                .geometric_product(&rev);
        }
        DeltaEncoding::Compressed => {
            // Exponential application: exp(delta) * state
            let exp_delta = delta.transform.exp();
            *state = exp_delta.geometric_product(state);
        }
    }
}

/// Apply a raw additive delta to a state.
pub fn apply_additive_delta(state: &mut GA3, delta: &GA3) {
    *state = &*state + delta;
}

/// A batch of deltas that can be applied together.
///
/// Batching reduces network overhead when multiple deltas need to
/// be transmitted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaBatch {
    /// The deltas in this batch, in causal order
    pub deltas: Vec<StateDelta>,
    /// Combined clock covering all deltas
    pub combined_clock: VectorClock,
}

impl DeltaBatch {
    /// Create a new empty batch.
    #[must_use]
    pub fn new() -> Self {
        Self {
            deltas: Vec::new(),
            combined_clock: VectorClock::new(),
        }
    }

    /// Add a delta to the batch.
    pub fn push(&mut self, delta: StateDelta) {
        self.combined_clock.update(&delta.to_clock);
        self.deltas.push(delta);
    }

    /// Check if the batch is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.deltas.is_empty()
    }

    /// Get the number of deltas in the batch.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.deltas.len()
    }

    /// Combine all additive deltas into a single delta.
    ///
    /// This only works for additive deltas; mixed batches are not combined.
    #[must_use]
    pub fn combine_additive(&self) -> Option<GA3> {
        if self.deltas.is_empty() {
            return None;
        }

        // Check all deltas are additive
        if !self
            .deltas
            .iter()
            .all(|d| d.encoding == DeltaEncoding::Additive)
        {
            return None;
        }

        // Sum all transforms
        let combined = self
            .deltas
            .iter()
            .fold(GA3::zero(), |acc, d| &acc + &d.transform);

        Some(combined)
    }

    /// Apply all deltas in the batch to a state.
    pub fn apply_to(&self, state: &mut GA3) {
        for delta in &self.deltas {
            apply_delta(state, delta);
        }
    }

    /// Get the estimated total size in bytes.
    #[must_use]
    pub fn estimated_size(&self) -> usize {
        self.deltas.iter().map(StateDelta::estimated_size).sum()
    }
}

impl Default for DeltaBatch {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute the size savings from using deltas vs full state sync.
///
/// Returns (`delta_size`, `full_size`, `savings_ratio`).
#[must_use]
pub fn compute_savings(delta: &StateDelta, _full_state: &GA3) -> (usize, usize, f64) {
    let delta_size = delta.estimated_size();
    let full_size = 8 * 8; // 8 coefficients * 8 bytes

    let savings = if full_size > 0 {
        1.0 - (delta_size as f64 / full_size as f64)
    } else {
        0.0
    };

    (delta_size, full_size, savings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_additive_delta() {
        let from = GA3::scalar(3.0);
        let to = GA3::scalar(7.0);

        let delta = compute_delta(&from, &to);

        // Delta should be 4.0
        assert!((delta.scalar_part() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_apply_additive_delta() {
        let from = GA3::scalar(3.0);
        let to = GA3::scalar(7.0);

        let delta_transform = compute_delta(&from, &to);
        let delta = StateDelta::additive(
            delta_transform,
            VectorClock::new(),
            VectorClock::new(),
            Uuid::new_v4(),
        );

        let mut state = from;
        apply_delta(&mut state, &delta);

        assert!((state.scalar_part() - 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_apply_multiplicative_delta() {
        // Create a rotor-like delta (simplified: scalar 2.0 for doubling)
        let delta = StateDelta::multiplicative(
            GA3::scalar(2.0),
            VectorClock::new(),
            VectorClock::new(),
            Uuid::new_v4(),
        );

        let mut state = GA3::scalar(3.0);
        apply_delta(&mut state, &delta);

        // sandwich(2, 3, 2) = 2 * 3 * 2 = 12
        assert!((state.scalar_part() - 12.0).abs() < 1e-10);
    }

    #[test]
    fn test_apply_compressed_delta() {
        let delta = StateDelta::compressed(
            GA3::scalar(1.0), // e^1 ≈ 2.718
            VectorClock::new(),
            VectorClock::new(),
            Uuid::new_v4(),
        );

        let mut state = GA3::scalar(1.0);
        apply_delta(&mut state, &delta);

        // exp(1) * 1 ≈ 2.718
        let expected = std::f64::consts::E;
        assert!((state.scalar_part() - expected).abs() < 1e-10);
    }

    #[test]
    fn test_delta_batch_combine() {
        let mut batch = DeltaBatch::new();

        batch.push(StateDelta::additive(
            GA3::scalar(1.0),
            VectorClock::new(),
            VectorClock::new(),
            Uuid::new_v4(),
        ));

        batch.push(StateDelta::additive(
            GA3::scalar(2.0),
            VectorClock::new(),
            VectorClock::new(),
            Uuid::new_v4(),
        ));

        batch.push(StateDelta::additive(
            GA3::scalar(3.0),
            VectorClock::new(),
            VectorClock::new(),
            Uuid::new_v4(),
        ));

        let combined = batch.combine_additive().unwrap();

        // 1 + 2 + 3 = 6
        assert!((combined.scalar_part() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_delta_batch_apply() {
        let mut batch = DeltaBatch::new();

        batch.push(StateDelta::additive(
            GA3::scalar(1.0),
            VectorClock::new(),
            VectorClock::new(),
            Uuid::new_v4(),
        ));

        batch.push(StateDelta::additive(
            GA3::scalar(2.0),
            VectorClock::new(),
            VectorClock::new(),
            Uuid::new_v4(),
        ));

        let mut state = GA3::scalar(10.0);
        batch.apply_to(&mut state);

        // 10 + 1 + 2 = 13
        assert!((state.scalar_part() - 13.0).abs() < 1e-10);
    }

    #[test]
    fn test_delta_applicability() {
        let mut from_clock = VectorClock::new();
        let node = Uuid::new_v4();
        from_clock.tick(node);

        let mut to_clock = from_clock.clone();
        to_clock.tick(node);

        let delta = StateDelta::additive(GA3::scalar(1.0), from_clock.clone(), to_clock, node);

        // Delta should be applicable to state with from_clock
        assert!(delta.is_applicable_to(&from_clock));

        // Delta should be applicable to state with higher clock
        let mut higher_clock = from_clock.clone();
        higher_clock.tick(node);
        assert!(delta.is_applicable_to(&higher_clock));
    }

    #[test]
    fn test_compute_delta_compressed() {
        let from = GA3::scalar(2.0);
        let to = GA3::scalar(8.0);

        let delta = compute_delta_compressed(&from, &to);

        // ln(8/2) = ln(4) ≈ 1.386
        let expected = (8.0_f64 / 2.0_f64).ln();
        assert!((delta.scalar_part() - expected).abs() < 1e-10);
    }

    #[test]
    fn test_delta_encoding_equality() {
        assert_eq!(DeltaEncoding::Additive, DeltaEncoding::Additive);
        assert_ne!(DeltaEncoding::Additive, DeltaEncoding::Multiplicative);
    }
}
