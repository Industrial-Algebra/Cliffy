// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Deterministic geometric projections over an [`ObservationSet`] —
//! the "render is geometric" half of the salvage (Phase 1).
//!
//! Projections are pure functions of the set: equal sets ⇒ bit-identical
//! values, on every replica, in every merge order. Scalar and vector means
//! (the componentwise floor) land with task 7; the rotor consensus
//! (Markley chordal-L₂ eigen-mean) with task 6.
//!
//! # Rotor ↔ quaternion mapping
//!
//! GA3 (`Multivector<3,0,0>`) coefficient layout is
//! `[scalar, e1, e2, e12, e3, e13, e23, e123]`, so the even (rotor) part
//! lives at indices `0, 3, 5, 6`. [`from_amari_rotor`] maps those to the
//! Hamilton quaternion `(w, x, y, z)`; the sign conventions are **pinned by
//! tests below** against amari's own right-handed `from_axis_angle`.

use amari_core::Rotor;

use crate::observation::RotorObservation;

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
}
