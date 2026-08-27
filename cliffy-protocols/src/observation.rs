// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! `ObservationSet` — the sound CRDT floor (salvage plan Phase 1).
//!
//! The CRDT **is** the set: a grow-only set of attributed observations whose
//! merge is plain union. Union is associative, commutative, and idempotent by
//! construction — convergence is trivial, never aspirational.
//!
//! The geometry is a **deterministic projection** over the set (see
//! [`crate::projection`]): equal sets ⇒ bit-identical consensus values on
//! every replica. Consensus is never merge state.
//!
//! # Determinism contract
//!
//! - Observations iterate in canonical key order (`participant_id`, `seq`)
//!   (`BTreeMap` — no `HashMap` anywhere in the set).
//! - Projections fold sequentially; float summation order is part of the
//!   wire contract.
//!
//! # Example
//!
//! ```
//! use cliffy_protocols::{Observation, ObservationSet};
//! use uuid::Uuid;
//!
//! let a = Uuid::new_v4();
//! let b = Uuid::new_v4();
//!
//! let mut replica_a = ObservationSet::new();
//! replica_a.insert(Observation::new_scalar(a, 0, 5.0));
//!
//! let mut replica_b = ObservationSet::new();
//! replica_b.insert(Observation::new_scalar(b, 0, 10.0));
//! replica_a.merge(&replica_b);
//!
//! // Both first-observations coexist — participant-scoped keys never collide.
//! assert_eq!(replica_a.len(), 2);
//! ```

use amari_core::Rotor;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::vector_clock::VectorClock;

/// Capability grant reference. Opaque in Phase 1; Phase 2 (Schubert gating)
/// gives it capability semantics — observations carry it now so the wire
/// format does not change later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct GrantRef(pub Uuid);

/// Orientation observation payload: a unit quaternion — the Cl⁺(3,0,0)
/// rotor, componentwise.
///
/// Convention: standard Hamilton quaternion `(w, x, y, z)`, right-handed.
/// Plain `f64` fields keep the merge/projection domain deterministic and
/// serde-native (amari 0.23's `Rotor` has neither `Serialize` nor a
/// coefficient constructor — see the Phase 1 plan, "design adjustment").
/// Convert for *applying* the rotation with [`RotorObservation::to_amari_rotor`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RotorObservation {
    /// Scalar (cosine) part.
    pub w: f64,
    /// Rotation-axis × sin(θ/2), x component.
    pub x: f64,
    /// Rotation-axis × sin(θ/2), y component.
    pub y: f64,
    /// Rotation-axis × sin(θ/2), z component.
    pub z: f64,
}

impl RotorObservation {
    /// The identity rotor (no rotation).
    #[must_use]
    pub const fn identity() -> Self {
        Self {
            w: 1.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Euclidean (chordal) inner product with another rotor observation.
    ///
    /// Negative dot means the two represent the *same* rotation on opposite
    /// hemispheres of the double cover — projections canonicalize this away
    /// before any aggregation.
    #[must_use]
    pub fn dot(&self, other: &Self) -> f64 {
        self.w * other.w + self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Euclidean norm.
    #[must_use]
    pub fn norm(&self) -> f64 {
        self.dot(self).sqrt()
    }

    /// Convert to an amari `Rotor<3,0,0>` for applying rotations.
    ///
    /// Goes through axis–angle and amari's own `from_axis_angle`, so the
    /// result may differ from the exact coefficients by a few ulps — a
    /// deterministic function of the input, which is all the projection
    /// contract requires. Near-identity observations (|sin(θ/2)| below
    /// 1e-12) map to the identity rotor.
    #[must_use]
    pub fn to_amari_rotor(&self) -> Rotor<3, 0, 0> {
        let w = self.w.clamp(-1.0, 1.0);
        let half_angle = w.acos();
        let sin_half = half_angle.sin();
        if sin_half < 1e-12 {
            return Rotor::identity();
        }
        let s = self.x.hypot(self.y).hypot(self.z);
        if s < 1e-12 {
            return Rotor::identity();
        }
        let scale = sin_half / s;
        Rotor::from_axis_angle(
            &amari_core::Vector::from_components(self.x * scale, self.y * scale, self.z * scale),
            2.0 * half_angle,
        )
    }
}

/// Position/displacement observation payload: a 3-vector, componentwise.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VectorObservation {
    /// x component.
    pub x: f64,
    /// y component.
    pub y: f64,
    /// z component.
    pub z: f64,
}

impl VectorObservation {
    /// Convert to an amari typed vector.
    #[must_use]
    pub fn to_amari_vector(&self) -> amari_core::Vector<3, 0, 0> {
        amari_core::Vector::from_components(self.x, self.y, self.z)
    }
}

/// The value a participant observed. Payloads are plain data: the merge
/// never interprets them, projections do.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ObservationPayload {
    /// Scalar state (counters, dials, levels) — componentwise floor.
    Scalar(f64),
    /// Position/displacement — componentwise floor.
    Vector(VectorObservation),
    /// Orientation — manifold (Markley eigen-mean) projection.
    Rotor(RotorObservation),
}

/// One attributed observation, canonically identified by
/// `(participant_id, seq)` — the participant-scoped identity whose absence
/// (len()-minted u64 ids) caused the old design's cross-replica collisions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    /// Observing participant — scopes the sequence number.
    pub participant_id: Uuid,
    /// Per-participant monotonic sequence (the participant assigns it).
    pub seq: u64,
    /// Causal context at observation time (the surviving `VectorClock`).
    pub clock: VectorClock,
    /// What was observed.
    pub payload: ObservationPayload,
    /// Provenance grant (Phase 2 capability gating); `None` = ungated.
    pub grant_ref: Option<GrantRef>,
}

impl Observation {
    /// A scalar observation with an empty clock.
    #[must_use]
    pub fn new_scalar(participant_id: Uuid, seq: u64, value: f64) -> Self {
        Self {
            participant_id,
            seq,
            clock: VectorClock::new(),
            payload: ObservationPayload::Scalar(value),
            grant_ref: None,
        }
    }

    /// A rotor observation with an empty clock.
    #[must_use]
    pub fn new_rotor(participant_id: Uuid, seq: u64, rotor: RotorObservation) -> Self {
        Self {
            participant_id,
            seq,
            clock: VectorClock::new(),
            payload: ObservationPayload::Rotor(rotor),
            grant_ref: None,
        }
    }

    /// A vector observation with an empty clock.
    #[must_use]
    pub fn new_vector(participant_id: Uuid, seq: u64, vector: VectorObservation) -> Self {
        Self {
            participant_id,
            seq,
            clock: VectorClock::new(),
            payload: ObservationPayload::Vector(vector),
            grant_ref: None,
        }
    }

    /// Attach a grant reference (builder-style).
    #[must_use]
    pub fn with_grant(mut self, grant: GrantRef) -> Self {
        self.grant_ref = Some(grant);
        self
    }

    /// This observation's canonical key.
    #[must_use]
    pub fn key(&self) -> ObservationKey {
        (self.participant_id, self.seq)
    }
}

/// Canonical identity of an observation: participant-scoped, globally unique
/// without coordination.
pub type ObservationKey = (Uuid, u64);

/// Grow-only set of attributed observations (G-Set semantics).
///
/// Merge is plain set union — associative, commutative, idempotent by
/// construction. The consensus value is a deterministic projection over the
/// set ([`crate::projection`]), computed on demand; it is never merge state.
///
/// # Domain invariant
///
/// `(participant_id, seq)` **uniquely determines** an observation: a
/// participant assigns seq monotonically, so two observations sharing a key
/// are re-deliveries of the same event and must be identical. The property
/// laws (commutativity, associativity) hold for well-formed sets; a
/// same-key-different-payload input is a malformed/Byzantine delivery
/// outside the contract (merge keeps the first-seen payload — deterministic,
/// but no ordering law is promised for that input class).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ObservationSet {
    observations: BTreeMap<ObservationKey, Observation>,
}

// Wire format: a JSON array of observations in canonical key order. (A
// BTreeMap<(Uuid, u64), _> cannot serialize as a JSON object — tuple keys
// are not strings — and the array keeps iteration order canonical on the
// wire. Deserialization reinserts into the map, re-sorting canonically;
// duplicate keys resolve last-wins, a deterministic function of the array.)
impl Serialize for ObservationSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde::Serialize::serialize(&self.observations.values().collect::<Vec<_>>(), serializer)
    }
}

impl<'de> Deserialize<'de> for ObservationSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let observations = Vec::<Observation>::deserialize(deserializer)?;
        let mut set = Self::new();
        for observation in observations {
            set.observations.insert(observation.key(), observation);
        }
        Ok(set)
    }
}

impl ObservationSet {
    /// The empty set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            observations: BTreeMap::new(),
        }
    }

    /// Insert an observation. Returns `false` if the key already exists
    /// (insertion is idempotent — a re-delivered observation merges as a
    /// no-op).
    pub fn insert(&mut self, observation: Observation) -> bool {
        self.observations
            .insert(observation.key(), observation)
            .is_none()
    }

    /// Union-merge: absorb every observation whose key is absent.
    /// Returns `true` if anything changed.
    ///
    /// This is the entire CRDT. Union is a true join-semilattice — unlike
    /// the old geometric merge, nothing here can annihilate, reorder, or
    /// diverge.
    pub fn merge(&mut self, other: &Self) -> bool {
        let before = self.observations.len();
        for (key, observation) in &other.observations {
            self.observations
                .entry(*key)
                .or_insert_with(|| observation.clone());
        }
        self.observations.len() != before
    }

    /// Number of observations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.observations.len()
    }

    /// Whether the set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.observations.is_empty()
    }

    /// Whether the key is present.
    #[must_use]
    pub fn contains(&self, key: &ObservationKey) -> bool {
        self.observations.contains_key(key)
    }

    /// Iterate in canonical key order — the deterministic iteration the
    /// projections rely on.
    pub fn iter(&self) -> impl Iterator<Item = &Observation> {
        self.observations.values()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    // Exactly-representable constants: bit equality is the point.

    use super::*;

    #[test]
    fn insert_is_idempotent_per_key() {
        let id = Uuid::new_v4();
        let mut set = ObservationSet::new();
        assert!(set.insert(Observation::new_scalar(id, 0, 1.0)));
        assert!(!set.insert(Observation::new_scalar(id, 0, 2.0)));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn merge_unions_disjoint_and_duplicate_keys() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let mut left = ObservationSet::new();
        left.insert(Observation::new_scalar(a, 0, 1.0));
        let mut right = ObservationSet::new();
        right.insert(Observation::new_scalar(b, 0, 2.0));
        right.insert(Observation::new_scalar(a, 0, 1.0)); // duplicate key

        assert!(left.merge(&right));
        assert_eq!(left.len(), 2);
        assert!(!left.merge(&right)); // idempotent
    }

    #[test]
    fn iteration_is_canonical_key_order() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let (first, second) = if a < b { (a, b) } else { (b, a) };
        let mut set = ObservationSet::new();
        set.insert(Observation::new_scalar(second, 1, 1.0));
        set.insert(Observation::new_scalar(first, 9, 2.0)); // low uuid, high seq
        set.insert(Observation::new_scalar(first, 0, 3.0));
        let keys: Vec<ObservationKey> = set.iter().map(Observation::key).collect();
        assert_eq!(keys, vec![(first, 0), (first, 9), (second, 1)]);
    }

    #[test]
    fn identity_rotor_observation_converts_to_identity_rotor() {
        let rotor = RotorObservation::identity().to_amari_rotor();
        let m = rotor.to_rotation_matrix();
        let identity = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        for i in 0..3 {
            for j in 0..3 {
                assert!((m[i][j] - identity[i][j]).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn serde_roundtrips_the_set() {
        let a = Uuid::new_v4();
        let mut set = ObservationSet::new();
        set.insert(Observation::new_scalar(a, 0, 1.5));
        set.insert(Observation::new_rotor(
            a,
            1,
            RotorObservation {
                w: 0.5,
                x: 0.5,
                y: 0.5,
                z: 0.5,
            },
        ));
        let json = serde_json::to_string(&set).expect("serialize");
        let back: ObservationSet = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(set, back);
    }
}
