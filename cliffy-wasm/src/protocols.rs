// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! WASM bindings for cliffy-protocols distributed state types.
//!
//! This module exposes the Phase 1 sound floor to JavaScript: the
//! [`ObservationSet`] G-Set CRDT (merge is set union), its deterministic
//! projections (`scalarMean`, `vectorMean`, `rotorConsensus` — the Markley
//! eigen-mean), and vector clocks.
//!
//! # Example
//!
//! ```javascript
//! import { ObservationSet, generateNodeId } from '@industrialalgebra/cliffy-core';
//!
//! const nodeA = generateNodeId();
//! const nodeB = generateNodeId();
//!
//! const left = new ObservationSet();
//! left.observeScalar(nodeA, 0, 5.0);
//!
//! const right = new ObservationSet();
//! right.observeScalar(nodeB, 0, 10.0);
//!
//! left.merge(right);            // union — both observations survive
//! console.log(left.scalarMean()); // 7.5 — the exact mean, never annihilated
//! ```

use cliffy_protocols::{
    Observation as CoreObservation, ObservationSet as CoreObservationSet,
    RotorObservation as CoreRotorObservation, VectorClock as CoreVectorClock,
    VectorObservation as CoreVectorObservation,
};
use js_sys::{Array, Object, Reflect};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

/// A vector clock for tracking causality in distributed systems.
///
/// Vector clocks provide a partial ordering of events across distributed nodes,
/// enabling detection of concurrent operations and causal dependencies.
///
/// # JavaScript Example
///
/// ```javascript
/// const clock1 = new VectorClock();
/// const clock2 = new VectorClock();
///
/// const nodeA = crypto.randomUUID();
/// const nodeB = crypto.randomUUID();
///
/// clock1.tick(nodeA);  // {nodeA: 1}
/// clock2.tick(nodeB);  // {nodeB: 1}
///
/// console.log(clock1.concurrent(clock2)); // true - neither happened before the other
///
/// clock1.update(clock2);  // {nodeA: 1, nodeB: 1}
/// clock1.tick(nodeA);     // {nodeA: 2, nodeB: 1}
///
/// console.log(clock2.happensBefore(clock1)); // true
/// ```
#[wasm_bindgen]
pub struct VectorClock {
    inner: CoreVectorClock,
}

#[wasm_bindgen]
impl VectorClock {
    /// Create a new empty vector clock.
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: CoreVectorClock::new(),
        }
    }

    /// Increment the clock for this node.
    ///
    /// This should be called before sending a message or performing a local operation.
    /// # Errors
    ///
    /// Returns a `JsValue` when the node ID is not a valid UUID.
    #[wasm_bindgen]
    pub fn tick(&mut self, node_id: &str) -> Result<(), JsValue> {
        let uuid = Uuid::parse_str(node_id)
            .map_err(|e| JsValue::from_str(&format!("Invalid UUID: {e}")))?;
        self.inner.tick(uuid);
        Ok(())
    }

    /// Update this clock with values from another clock.
    ///
    /// This should be called when receiving a message from another node.
    #[wasm_bindgen]
    pub fn update(&mut self, other: &Self) {
        self.inner.update(&other.inner);
    }

    /// Check if this clock happens-before another clock.
    ///
    /// Returns true if all events in this clock happened before or at the same
    /// time as events in the other clock, and at least one event happened strictly before.
    #[wasm_bindgen(js_name = happensBefore)]
    #[must_use]
    pub fn happens_before(&self, other: &Self) -> bool {
        self.inner.happens_before(&other.inner)
    }

    /// Check if this clock is concurrent with another clock.
    ///
    /// Returns true if neither clock happens-before the other.
    /// Concurrent events may conflict and require resolution.
    #[wasm_bindgen]
    #[must_use]
    pub fn concurrent(&self, other: &Self) -> bool {
        self.inner.concurrent(&other.inner)
    }

    /// Merge two clocks, taking the maximum of each component.
    ///
    /// Returns a new clock representing the combined knowledge of both clocks.
    #[wasm_bindgen]
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            inner: self.inner.merge(&other.inner),
        }
    }

    /// Get the clock as a JavaScript object {nodeId: timestamp, ...}.
    #[wasm_bindgen(js_name = toObject)]
    #[must_use]
    pub fn to_object(&self) -> Object {
        let obj = Object::new();
        for (uuid, time) in &self.inner.clocks {
            let _ = Reflect::set(&obj, &uuid.to_string().into(), &(*time as f64).into());
        }
        obj
    }

    /// Get the time for a specific node.
    /// # Errors
    ///
    /// Returns a `JsValue` when the node ID is not a valid UUID.
    #[wasm_bindgen(js_name = getTime)]
    pub fn get_time(&self, node_id: &str) -> Result<u32, JsValue> {
        let uuid = Uuid::parse_str(node_id)
            .map_err(|e| JsValue::from_str(&format!("Invalid UUID: {e}")))?;
        Ok(*self.inner.clocks.get(&uuid).unwrap_or(&0) as u32)
    }
}

impl Default for VectorClock {
    fn default() -> Self {
        Self::new()
    }
}

/// A grow-only set of attributed observations — the sound CRDT floor.
///
/// Merge is plain set union (associative, commutative, idempotent);
/// consensus values are deterministic projections over the set: equal
/// sets produce identical values on every replica.
///
/// Sequence numbers are per-participant (`(nodeId, seq)` is the canonical
/// key) — two nodes' first observations coexist instead of colliding.
#[wasm_bindgen]
pub struct ObservationSet {
    inner: CoreObservationSet,
}

#[wasm_bindgen]
impl ObservationSet {
    /// Create an empty observation set.
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: CoreObservationSet::new(),
        }
    }

    /// Observe a scalar value as the given participant.
    ///
    /// # Errors
    ///
    /// Returns a `JsValue` when `node_id` is not a valid UUID or `seq` is
    /// not a non-negative integer.
    #[wasm_bindgen(js_name = observeScalar)]
    pub fn observe_scalar(&mut self, node_id: &str, seq: f64, value: f64) -> Result<(), JsValue> {
        let observation =
            CoreObservation::new_scalar(parse_uuid(node_id)?, f64_to_seq(seq)?, value);
        self.inner.insert(observation);
        Ok(())
    }

    /// Observe a 3D position as the given participant.
    ///
    /// # Errors
    ///
    /// Returns a `JsValue` when `node_id` is not a valid UUID or `seq` is
    /// not a non-negative integer.
    #[wasm_bindgen(js_name = observeVector)]
    pub fn observe_vector(
        &mut self,
        node_id: &str,
        seq: f64,
        x: f64,
        y: f64,
        z: f64,
    ) -> Result<(), JsValue> {
        let observation = CoreObservation::new_vector(
            parse_uuid(node_id)?,
            f64_to_seq(seq)?,
            CoreVectorObservation { x, y, z },
        );
        self.inner.insert(observation);
        Ok(())
    }

    /// Observe an orientation (quaternion `w, x, y, z`; normalized
    /// internally) as the given participant.
    ///
    /// # Errors
    ///
    /// Returns a `JsValue` when `node_id` is not a valid UUID, `seq` is
    /// not a non-negative integer, or the quaternion is degenerate
    /// (near-zero norm).
    #[wasm_bindgen(js_name = observeRotor)]
    pub fn observe_rotor(
        &mut self,
        node_id: &str,
        seq: f64,
        w: f64,
        x: f64,
        y: f64,
        z: f64,
    ) -> Result<(), JsValue> {
        let rotor = CoreRotorObservation { w, x, y, z };
        let norm = rotor.norm();
        if norm < 1e-12 {
            return Err(JsValue::from_str("degenerate rotor: near-zero norm"));
        }
        let unit = CoreRotorObservation {
            w: rotor.w / norm,
            x: rotor.x / norm,
            y: rotor.y / norm,
            z: rotor.z / norm,
        };
        let observation = CoreObservation::new_rotor(parse_uuid(node_id)?, f64_to_seq(seq)?, unit);
        self.inner.insert(observation);
        Ok(())
    }

    /// Union-merge another set into this one. Returns `true` if anything
    /// changed. This is the entire CRDT — nothing can annihilate.
    #[wasm_bindgen]
    pub fn merge(&mut self, other: &Self) -> bool {
        self.inner.merge(&other.inner)
    }

    /// Number of observations.
    #[wasm_bindgen]
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether the set is empty.
    #[wasm_bindgen(js_name = isEmpty)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Deterministic scalar consensus: the arithmetic mean of every scalar
    /// observation. `null` when the set holds no scalars.
    #[wasm_bindgen(js_name = scalarMean)]
    #[must_use]
    pub fn scalar_mean(&self) -> Option<f64> {
        cliffy_protocols::scalar_mean(&self.inner)
    }

    /// Deterministic vector consensus: componentwise mean. `null` when the
    /// set holds no vectors.
    #[wasm_bindgen(js_name = vectorMean)]
    #[must_use]
    pub fn vector_mean(&self) -> Option<Array> {
        cliffy_protocols::vector_mean(&self.inner).map(|v| {
            let arr = Array::new();
            arr.push(&v.x.into());
            arr.push(&v.y.into());
            arr.push(&v.z.into());
            arr
        })
    }

    /// Deterministic orientation consensus (Markley chordal-L2 eigen-mean)
    /// as `[w, x, y, z]`. `null` when the set holds no usable rotors.
    #[wasm_bindgen(js_name = rotorConsensus)]
    #[must_use]
    pub fn rotor_consensus(&self) -> Option<Array> {
        cliffy_protocols::rotor_consensus(&self.inner).map(|r| {
            let arr = Array::new();
            arr.push(&r.w.into());
            arr.push(&r.x.into());
            arr.push(&r.y.into());
            arr.push(&r.z.into());
            arr
        })
    }
}

impl Default for ObservationSet {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_uuid(node_id: &str) -> Result<Uuid, JsValue> {
    Uuid::parse_str(node_id).map_err(|e| JsValue::from_str(&format!("Invalid UUID: {e}")))
}

/// f64 → u64 seq with integral-value validation (JS numbers only);
/// `None` = invalid. Pure — testable on non-wasm hosts.
fn validate_seq(seq: f64) -> Option<u64> {
    if !(0.0..=9.007_199_254_740_992e15).contains(&seq) || seq.fract() != 0.0 {
        return None;
    }
    Some(seq as u64)
}

fn f64_to_seq(seq: f64) -> Result<u64, JsValue> {
    validate_seq(seq)
        .ok_or_else(|| JsValue::from_str(&format!("seq must be a non-negative integer, got {seq}")))
}

/// Generate a random UUID suitable for use as a node ID.
#[wasm_bindgen(js_name = generateNodeId)]
#[must_use]
pub fn generate_node_id() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_clock_wasm() {
        let mut clock = VectorClock::new();
        let node_id = Uuid::new_v4().to_string();
        clock.tick(&node_id).unwrap();
        assert_eq!(clock.get_time(&node_id).unwrap(), 1);
    }

    #[test]
    fn test_observation_set_no_annihilation() {
        // The wasm-level oracle for the February failure: two first
        // observations merge; the mean is the exact arithmetic mean —
        // where the old CRDT annihilated to zero and collided ids at 0.
        let a = Uuid::new_v4().to_string();
        let b = Uuid::new_v4().to_string();

        let mut left = ObservationSet::new();
        left.observe_scalar(&a, 0.0, 5.0).unwrap();
        let mut right = ObservationSet::new();
        right.observe_scalar(&b, 0.0, 10.0).unwrap();

        assert!(left.merge(&right));
        assert_eq!(left.len(), 2, "both first observations coexist");
        assert!((left.scalar_mean().unwrap() - 7.5).abs() < 1e-12);
    }

    #[cfg(target_arch = "wasm32")] // js_sys::Array — runs under wasm-pack test
    #[test]
    fn test_observation_set_rotor_consensus() {
        let a = Uuid::new_v4().to_string();
        let mut set = ObservationSet::new();
        set.observe_rotor(&a, 0.0, 1.0, 0.0, 0.0, 0.0).unwrap(); // identity
        set.observe_rotor(&a, 1.0, 0.0, 1.0, 0.0, 0.0).unwrap(); // 180° x
        let consensus = set.rotor_consensus().expect("consensus exists");
        // Mean of identity and 180°x: 90°x → w = x = √2/2 (up to sign).
        let w: f64 = consensus.get(0).as_f64().unwrap();
        let x: f64 = consensus.get(1).as_f64().unwrap();
        assert!((w.abs() - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-9);
        assert!((x.abs() - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-9);
    }

    #[test]
    fn test_seq_validation_is_pure() {
        // (JsValue errors cannot be constructed off-wasm; the validators
        // are pure and tested directly — the wasm-pack suite exercises the
        // Result boundary in-browser.)
        assert_eq!(validate_seq(0.0), Some(0));
        assert_eq!(validate_seq(42.0), Some(42));
        assert_eq!(validate_seq(-1.0), None);
        assert_eq!(validate_seq(0.5), None);
        assert!(Uuid::parse_str("not-a-uuid").is_err());
    }
}
