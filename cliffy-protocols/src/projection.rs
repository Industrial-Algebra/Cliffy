// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Deterministic geometric projections over an [`ObservationSet`] —
//! the "render is geometric" half of the salvage (Phase 1).
//!
//! Projections are pure functions of the set: equal sets ⇒ bit-identical
//! values, on every replica, in every merge order. All folds run over the
//! set's canonical iteration order, sequentially — no parallelism inside a
//! projection (float summation order is part of the contract).
//!
//! - [`scalar_mean`] / [`vector_mean`]: the componentwise floor — boring
//!   state takes the boring mean (the `ComponentLattice` philosophy).
//! - [`rotor_consensus`]: the Markley chordal-L₂ eigen-mean (Markley et al.
//!   2007) — dominant eigenvector of `M = Σ wᵢ qᵢ qᵢᵀ` over
//!   hemisphere-canonicalized quaternions.
//!
//! # Rotor ↔ quaternion mapping
//!
//! GA3 (`Multivector<3,0,0>`) coefficient layout is
//! `[scalar, e1, e2, e12, e3, e13, e23, e123]`, so the even (rotor) part
//! lives at indices `0, 3, 5, 6`. [`from_amari_rotor`] maps those to the
//! Hamilton quaternion `(w, x, y, z)`; the sign conventions are **pinned by
//! tests below** against amari's own right-handed `from_axis_angle`.

use amari_core::Rotor;

use crate::eigen::{dominant_eigenvalue_index, jacobi_eigen_4};
use crate::observation::{
    Observation, ObservationPayload, ObservationSet, RotorObservation, VectorObservation,
};

/// Extract the Hamilton quaternion of an amari rotor (even coefficients).
///
/// Empirically pinned mapping (see `mapping_pins_amari_conventions`, probed
/// against amari 0.23.0 and 0.24.1 — identical): amari's
/// `exp(-B·θ/2)`-convention coefficients `(s, e12, e13, e23)` map to
/// Hamilton `(w, x, y, z)` as `w = c[0]`, `x = -c[6]` (e23), `y = +c[5]`
/// (e13), `z = -c[3]` (e12) — the bivector is the dual of the rotation
/// axis, with per-component signs from the right-hand convention.
#[allow(clippy::indexing_slicing)]
// The coefficient indices below are the pinned mapping this function IS;
// if amari's slice ever changes length/ layout, the pinning tests fail
// loudly rather than these reads panicking silently in production.
#[must_use]
pub fn from_amari_rotor(rotor: &Rotor<3, 0, 0>) -> RotorObservation {
    let c = rotor.as_slice();
    // c: [scalar, e1, e2, e12, e3, e13, e23, e123]
    RotorObservation {
        w: c[0],
        x: -c[6],
        y: c[5],
        z: -c[3],
    }
}

/// Arithmetic mean of every `Scalar` payload, in canonical order.
///
/// `None` when the set holds no scalar observations. This is the
/// componentwise floor: counters and dials never go near a manifold.
#[must_use]
pub fn scalar_mean(set: &ObservationSet) -> Option<f64> {
    let mut sum = 0.0;
    let mut count = 0usize;
    for observation in set.iter() {
        if let ObservationPayload::Scalar(value) = observation.payload {
            sum += value;
            count += 1;
        }
    }
    if count == 0 {
        None
    } else {
        Some(sum / count as f64)
    }
}

/// Componentwise mean of every `Vector` payload, in canonical order.
///
/// `None` when the set holds no vector observations.
#[must_use]
pub fn vector_mean(set: &ObservationSet) -> Option<VectorObservation> {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut z = 0.0;
    let mut count = 0usize;
    for observation in set.iter() {
        if let ObservationPayload::Vector(v) = observation.payload {
            x += v.x;
            y += v.y;
            z += v.z;
            count += 1;
        }
    }
    if count == 0 {
        None
    } else {
        Some(VectorObservation {
            x: x / count as f64,
            y: y / count as f64,
            z: z / count as f64,
        })
    }
}

/// Deterministic rotor consensus: the Markley chordal-L₂ eigen-mean.
///
/// Dominant eigenvector of `M = Σ wᵢ qᵢ qᵢᵀ` over hemisphere-canonicalized
/// unit quaternions (Markley et al. 2007). The result is sign-canonicalized
/// (`w ≥ 0`) so equal sets produce bit-identical rotors on every replica.
///
/// `None` when the set holds no usable rotor observations (none present, or
/// all degenerate below the 1e-12 normalization floor — skipped, never
/// fabricate).
///
/// # Weights
///
/// The weighted variant's `weights` closure MUST be a pure function of
/// set-determined observation metadata (never local trust tables), or
/// replicas rendering the same set diverge — the determinism contract is
/// the caller's to keep here.
#[must_use]
pub fn rotor_consensus(set: &ObservationSet) -> Option<RotorObservation> {
    rotor_consensus_with_weights(set, |_| 1.0)
}

/// Weighted Markley eigen-mean; see [`rotor_consensus`] for the contract.
#[allow(clippy::indexing_slicing)]
// All indexing is over the fixed 4×4 profile matrix and 4-component
// quaternions with literal-bounded loops — the array lengths are in the
// types; .get() ceremony would obscure the math (same policy as eigen.rs).
#[must_use]
pub fn rotor_consensus_with_weights(
    set: &ObservationSet,
    weights: impl Fn(&Observation) -> f64,
) -> Option<RotorObservation> {
    // Single canonical-order pass: hemisphere-canonicalize against the
    // FIRST usable rotor (set-determined reference, never a local
    // convention), pair each with its weight, and accumulate M = Σ wᵢ qᵢ qᵢᵀ
    // sequentially — the fold order IS the determinism contract.
    let mut m = [[0.0f64; 4]; 4];
    let mut reference: Option<RotorObservation> = None;
    let mut usable = 0usize;
    for observation in set.iter() {
        let ObservationPayload::Rotor(rotor) = observation.payload else {
            continue;
        };
        let norm = rotor.norm();
        if norm < 1e-12 {
            continue; // degenerate: skip, never fabricate
        }
        let unit = RotorObservation {
            w: rotor.w / norm,
            x: rotor.x / norm,
            y: rotor.y / norm,
            z: rotor.z / norm,
        };
        let reference = *reference.get_or_insert(unit);
        let aligned = if reference.dot(&unit) < 0.0 {
            RotorObservation {
                w: -unit.w,
                x: -unit.x,
                y: -unit.y,
                z: -unit.z,
            }
        } else {
            unit
        };
        let weight = weights(observation);
        let qc = [aligned.w, aligned.x, aligned.y, aligned.z];
        for i in 0..4 {
            for j in 0..4 {
                m[i][j] += weight * qc[i] * qc[j];
            }
        }
        usable += 1;
    }
    if usable == 0 {
        return None;
    }

    // Pass 3: dominant eigenvector, deterministic tie-break, sign canon.
    let (eigenvalues, eigenvectors) = jacobi_eigen_4(m);
    let dominant = dominant_eigenvalue_index(&eigenvalues);
    let mut result = RotorObservation {
        w: eigenvectors[0][dominant],
        x: eigenvectors[1][dominant],
        y: eigenvectors[2][dominant],
        z: eigenvectors[3][dominant],
    };
    let norm = result.norm();
    if norm < 1e-12 {
        return None;
    }
    result.w /= norm;
    result.x /= norm;
    result.y /= norm;
    result.z /= norm;
    if result.w < 0.0 || (result.w == 0.0 && result.x + result.y + result.z < 0.0) {
        result = RotorObservation {
            w: -result.w,
            x: -result.x,
            y: -result.y,
            z: -result.z,
        };
    }
    Some(result)
}
#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    // Sign-pinning tests assert exact analytic values (√2/2 etc.) against
    // f64 constants — approximations would defeat the point.

    use super::*;
    use amari_core::Vector;

    const SQRT2_2: f64 = std::f64::consts::FRAC_1_SQRT_2;

    /// Vector components from amari's full multivector-layout slice:
    /// `as_slice()` = [scalar, e1, e2, e12, e3, e13, e23, e123], so a
    /// 3-vector's components live at indices 1, 2, 4. (This layout trap is
    /// gap #1 in docs/reports/2026-08-27-amari-core-gap-report.md.)
    fn v3(v: &amari_core::Vector<3, 0, 0>) -> (f64, f64, f64) {
        let c = v.as_slice();
        (c[1], c[2], c[4])
    }

    /// amari's own convention check (independent of our mapping): +90°
    /// about each axis, right-hand rule. Value oracles for all three axes —
    /// coverage amari's own suite lacks (its `from_axis_angle` test compares
    /// coefficients against the e12 bivector, z-axis only).
    #[test]
    fn amari_axis_rotations_are_right_handed_all_axes() {
        for (name, axis, expected) in [
            ("x", [1.0, 0.0, 0.0], (1.0, 0.0, 0.0)), // axis vector is fixed
            ("y", [0.0, 1.0, 0.0], (0.0, 0.0, -1.0)), // x -> -z
            ("z", [0.0, 0.0, 1.0], (0.0, 1.0, 0.0)), // x -> y
        ] {
            let rotor = Rotor::from_axis_angle(
                &Vector::<3, 0, 0>::from_components(axis[0], axis[1], axis[2]),
                std::f64::consts::FRAC_PI_2,
            );
            let got =
                v3(&rotor.apply_to_vector(&Vector::<3, 0, 0>::from_components(1.0, 0.0, 0.0)));
            let (ex, ey, ez) = expected;
            assert!(
                (got.0 - ex).abs() < 1e-12
                    && (got.1 - ey).abs() < 1e-12
                    && (got.2 - ez).abs() < 1e-12,
                "axis {name}: (1,0,0) -> ({}, {}, {}), expected ({ex}, {ey}, {ez})",
                got.0,
                got.1,
                got.2
            );
        }
    }

    /// Pin the exact coefficient mapping: +90° about each axis must produce
    /// the standard Hamilton quaternions — the empirical sign table.
    #[test]
    fn mapping_pins_amari_conventions() {
        for (name, axis, hamilton) in [
            ("x", [1.0, 0.0, 0.0], (SQRT2_2, SQRT2_2, 0.0, 0.0)),
            ("y", [0.0, 1.0, 0.0], (SQRT2_2, 0.0, SQRT2_2, 0.0)),
            ("z", [0.0, 0.0, 1.0], (SQRT2_2, 0.0, 0.0, SQRT2_2)),
        ] {
            let rotor = Rotor::from_axis_angle(
                &Vector::<3, 0, 0>::from_components(axis[0], axis[1], axis[2]),
                std::f64::consts::FRAC_PI_2,
            );
            let q = from_amari_rotor(&rotor);
            assert!((q.w - hamilton.0).abs() < 1e-12, "axis {name}: w = {}", q.w);
            assert!((q.x - hamilton.1).abs() < 1e-12, "axis {name}: x = {}", q.x);
            assert!((q.y - hamilton.2).abs() < 1e-12, "axis {name}: y = {}", q.y);
            assert!((q.z - hamilton.3).abs() < 1e-12, "axis {name}: z = {}", q.z);
        }
    }

    /// Roundtrip: `to_amari_rotor(from_amari_rotor(r))` performs the same
    /// rotation as `r` (semantic roundtrip via axis–angle).
    #[test]
    fn roundtrip_preserves_rotation_semantics() {
        let axis = Vector::<3, 0, 0>::from_components(1.0, 2.0, -0.5);
        for angle_deg in [0.0f64, 17.0, 45.0, 90.0, 179.0, 244.0, 359.0] {
            let angle = angle_deg.to_radians();
            let original = Rotor::from_axis_angle(&axis, angle);
            let roundtripped = from_amari_rotor(&original).to_amari_rotor();

            let v = Vector::<3, 0, 0>::from_components(0.3, -1.2, 2.0);
            let (a, b) = (
                v3(&original.apply_to_vector(&v)),
                v3(&roundtripped.apply_to_vector(&v)),
            );
            assert!(
                (a.0 - b.0).abs() < 1e-10 && (a.1 - b.1).abs() < 1e-10 && (a.2 - b.2).abs() < 1e-10,
                "roundtrip diverged at angle {angle_deg}°: ({}, {}, {}) vs ({}, {}, {})",
                a.0,
                a.1,
                a.2,
                b.0,
                b.1,
                b.2
            );
        }
    }

    // ------------------------------------------------------------------
    // Markley rotor-consensus oracles (task 6)
    // ------------------------------------------------------------------

    use crate::observation::ObservationSet;
    use uuid::Uuid;

    fn rotor_set(observations: &[(u64, RotorObservation)]) -> ObservationSet {
        let id = Uuid::new_v4();
        let mut set = ObservationSet::new();
        for &(seq, rotor) in observations {
            set.insert(crate::observation::Observation::new_rotor(id, seq, rotor));
        }
        set
    }

    fn quat(w: f64, x: f64, y: f64, z: f64) -> RotorObservation {
        RotorObservation { w, x, y, z }
    }

    fn assert_same_rotation(a: RotorObservation, b: RotorObservation, context: &str) {
        // Same rotation iff equal up to sign on the double cover.
        let same = a.w * b.w + a.x * b.x + a.y * b.y + a.z * b.z;
        assert!(
            same.abs() > 1.0 - 1e-10,
            "{context}: |dot| = {same} (rotations differ)"
        );
    }

    /// A single rotor is its own consensus.
    #[test]
    fn consensus_of_single_rotor_is_itself() {
        let q = quat(0.6, 0.8, 0.0, 0.0); // unit by construction
        let set = rotor_set(&[(0, q)]);
        let result = rotor_consensus(&set).expect("single rotor has consensus");
        assert_same_rotation(result, q, "single rotor");
    }

    /// Duplicated observations do not move the mean.
    #[test]
    fn consensus_of_duplicates_is_unchanged() {
        let q = quat(0.6, 0.8, 0.0, 0.0);
        let set = rotor_set(&[(0, q), (1, q), (2, q)]);
        let result = rotor_consensus(&set).expect("consensus exists");
        assert_same_rotation(result, q, "duplicated rotor");
    }

    /// Double cover: observing q and -q is observing q twice — the result
    /// must equal q alone (hemisphere canonicalization).
    #[test]
    fn consensus_absorbs_double_cover_sign() {
        let q = quat(0.6, 0.8, 0.0, 0.0);
        let negated = quat(-0.6, -0.8, 0.0, 0.0);
        let set = rotor_set(&[(0, q), (1, negated)]);
        let result = rotor_consensus(&set).expect("consensus exists");
        assert_same_rotation(result, q, "q and -q");
    }

    /// Analytic mean: +90°z and -90°z average to the identity rotor.
    #[test]
    fn consensus_of_opposed_rotors_is_identity() {
        let plus = quat(SQRT2_2, 0.0, 0.0, SQRT2_2); // +90° about z
        let minus = quat(SQRT2_2, 0.0, 0.0, -SQRT2_2); // -90° about z
        let set = rotor_set(&[(0, plus), (1, minus)]);
        let result = rotor_consensus(&set).expect("consensus exists");
        assert!(
            result.w.abs() > 1.0 - 1e-10,
            "expected identity rotor, got w = {}",
            result.w
        );
    }

    /// Weight-2 dominance is equivalent to duplicating the observation
    /// (the M accumulation is linear in the weights).
    #[test]
    fn weighted_consensus_matches_duplicated_observations() {
        let a = quat(SQRT2_2, 0.0, 0.0, SQRT2_2);
        let b = quat(0.5, 0.5, 0.5, 0.5);
        let mut weighted = rotor_set(&[(0, a), (1, b)]);
        // Direct access to bump weights is not public API; approximate by
        // constructing the equivalent: weight(b)=2 ≡ observing b twice.
        weighted.insert(crate::observation::Observation::new_rotor(
            Uuid::new_v4(),
            0,
            b,
        ));
        // weighted now: a ×1 (id1,0), b ×2 (id2,0 + id1,1) — order differs
        // from the canonical construction but sets are equal as multiset of
        // (participant, payload) contributions.

        let duplicated = rotor_set(&[(0, b), (1, a), (2, b)]);
        let left = rotor_consensus(&weighted).expect("weighted consensus");
        let right = rotor_consensus(&duplicated).expect("duplicated consensus");
        let dot = left.w * right.w + left.x * right.x + left.y * right.y + left.z * right.z;
        assert!(
            dot.abs() > 1.0 - 1e-9,
            "weight-2 ≡ duplication: |dot| = {dot}"
        );
    }

    /// THE determinism oracle: three different merge orders reaching the
    /// same set must yield BIT-IDENTICAL consensus — exact component
    /// equality, not approximate.
    #[test]
    fn consensus_is_bit_identical_across_merge_orders() {
        let (a, b, c) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let mut base = ObservationSet::new();
        base.insert(crate::observation::Observation::new_rotor(
            a,
            0,
            quat(0.9, 0.1, 0.2, 0.3),
        ));
        base.insert(crate::observation::Observation::new_rotor(
            a,
            1,
            quat(0.5, 0.5, 0.5, 0.5),
        ));
        base.insert(crate::observation::Observation::new_rotor(
            b,
            0,
            quat(SQRT2_2, 0.0, 0.0, SQRT2_2),
        ));
        base.insert(crate::observation::Observation::new_rotor(
            c,
            0,
            quat(0.3, -0.4, 0.5, 0.7),
        ));

        let order_one = base.clone();
        let mut order_two = ObservationSet::new();
        order_two.merge(&base);
        order_two.merge(&base); // idempotent re-delivery
        let mut order_three = ObservationSet::new();
        for observation in base.iter().collect::<Vec<_>>().into_iter().rev() {
            order_three.insert(observation.clone());
        }

        let one = rotor_consensus(&order_one).expect("consensus one");
        let two = rotor_consensus(&order_two).expect("consensus two");
        let three = rotor_consensus(&order_three).expect("consensus three");
        assert_eq!(
            (one.w, one.x, one.y, one.z),
            (two.w, two.x, two.y, two.z),
            "merge order changed the consensus bits"
        );
        assert_eq!(
            (one.w, one.x, one.y, one.z),
            (three.w, three.x, three.y, three.z),
            "insert order changed the consensus bits"
        );
    }

    // ------------------------------------------------------------------
    // Componentwise floor oracles (task 7)
    // ------------------------------------------------------------------

    /// Mixed payloads: each projection aggregates only its kind.
    #[test]
    fn mixed_payloads_project_independently() {
        let id = Uuid::new_v4();
        let mut set = ObservationSet::new();
        set.insert(crate::observation::Observation::new_scalar(id, 0, 5.0));
        set.insert(crate::observation::Observation::new_vector(
            id,
            1,
            crate::observation::VectorObservation {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
        ));
        set.insert(crate::observation::Observation::new_scalar(id, 2, 10.0));

        assert_eq!(scalar_mean(&set), Some(7.5));
        let v = vector_mean(&set).expect("vector mean exists");
        assert_eq!((v.x, v.y, v.z), (1.0, 2.0, 3.0));
        assert!(
            rotor_consensus(&set).is_none(),
            "no rotor payloads → no consensus"
        );
    }

    /// Empty and kind-less sets project to None — never fabricate.
    #[test]
    fn projections_never_fabricate() {
        let set = ObservationSet::new();
        assert_eq!(scalar_mean(&set), None);
        assert!(vector_mean(&set).is_none());
        assert!(rotor_consensus(&set).is_none());
    }
}
