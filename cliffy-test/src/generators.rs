// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Generators for property-based testing with QuickCheck
//!
//! These generators create random geometric algebra elements for testing:
//! - Arbitrary GA3 multivectors
//! - Unit vectors
//! - Rotors (unit versors)
//! - Bivectors

use crate::{bivector, vector, GA3};
use quickcheck::{Arbitrary, Gen};

/// Generate a random f64 in a range using QuickCheck's Gen
fn gen_f64_range(g: &mut Gen, min: f64, max: f64) -> f64 {
    // Use u64::arbitrary and normalize to the desired range
    // This avoids NaN/infinity issues from f64::arbitrary
    let val: u64 = u64::arbitrary(g);
    // Normalize to [0, 1)
    let normalized = (val as f64) / (u64::MAX as f64);
    min + normalized * (max - min)
}

/// Generate an arbitrary GA3 multivector
pub fn arbitrary_ga3(g: &mut Gen) -> GA3 {
    let coeffs: Vec<f64> = vec![
        gen_f64_range(g, -10.0, 10.0), // scalar
        gen_f64_range(g, -10.0, 10.0), // e1
        gen_f64_range(g, -10.0, 10.0), // e2
        gen_f64_range(g, -10.0, 10.0), // e12
        gen_f64_range(g, -10.0, 10.0), // e3
        gen_f64_range(g, -10.0, 10.0), // e13
        gen_f64_range(g, -10.0, 10.0), // e23
        gen_f64_range(g, -10.0, 10.0), // e123
    ];
    GA3::from_coefficients(coeffs)
}

/// Generate an arbitrary vector (grade 1 element)
pub fn arbitrary_vector(g: &mut Gen) -> GA3 {
    vector(
        gen_f64_range(g, -10.0, 10.0),
        gen_f64_range(g, -10.0, 10.0),
        gen_f64_range(g, -10.0, 10.0),
    )
}

/// Generate an arbitrary unit vector
pub fn arbitrary_unit_vector(g: &mut Gen) -> GA3 {
    loop {
        let v = vector(
            gen_f64_range(g, -1.0, 1.0),
            gen_f64_range(g, -1.0, 1.0),
            gen_f64_range(g, -1.0, 1.0),
        );
        let mag = v.magnitude();
        if mag > 0.1 {
            // Avoid division by very small numbers
            return v.normalize().unwrap_or_else(GA3::zero);
        }
    }
}

/// Generate an arbitrary rotor (unit versor)
///
/// A rotor is constructed as exp(B/2) where B is a bivector representing
/// the rotation plane and angle.
pub fn arbitrary_rotor(g: &mut Gen) -> GA3 {
    // Generate random bivector for rotation plane
    let angle: f64 = gen_f64_range(g, 0.0, std::f64::consts::TAU);
    let half_angle = angle / 2.0;

    // Generate a random unit bivector
    let bx: f64 = gen_f64_range(g, -1.0, 1.0);
    let by: f64 = gen_f64_range(g, -1.0, 1.0);
    let bz: f64 = gen_f64_range(g, -1.0, 1.0);
    let b = bivector(bx, by, bz);
    let b_mag = b.magnitude();

    if b_mag > 0.1 {
        let b_unit = b.normalize().unwrap_or_else(GA3::zero);

        // Rotor = cos(θ/2) + sin(θ/2) * B
        let scalar_part = GA3::scalar(half_angle.cos());
        let bivector_part = &b_unit * half_angle.sin();
        &scalar_part + &bivector_part
    } else {
        // Degenerate case - return identity rotor
        GA3::scalar(1.0)
    }
}

/// Generate an arbitrary bivector (grade 2 element)
pub fn arbitrary_bivector(g: &mut Gen) -> GA3 {
    bivector(
        gen_f64_range(g, -10.0, 10.0), // e12
        gen_f64_range(g, -10.0, 10.0), // e13
        gen_f64_range(g, -10.0, 10.0), // e23
    )
}

/// Wrapper for QuickCheck Arbitrary trait
#[derive(Clone, Debug)]
pub struct ArbitraryGA3(pub GA3);

impl Arbitrary for ArbitraryGA3 {
    fn arbitrary(g: &mut Gen) -> Self {
        ArbitraryGA3(arbitrary_ga3(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        // Shrink by reducing magnitude
        let mv = self.0.clone();
        let mag = mv.magnitude();
        if mag < 0.01 {
            Box::new(std::iter::empty())
        } else {
            let shrunk = &mv * 0.5;
            Box::new(std::iter::once(ArbitraryGA3(shrunk)))
        }
    }
}

/// Wrapper for arbitrary vectors
#[derive(Clone, Debug)]
pub struct ArbitraryVector(pub GA3);

impl Arbitrary for ArbitraryVector {
    fn arbitrary(g: &mut Gen) -> Self {
        ArbitraryVector(arbitrary_vector(g))
    }
}

/// Wrapper for arbitrary unit vectors
#[derive(Clone, Debug)]
pub struct ArbitraryUnitVector(pub GA3);

impl Arbitrary for ArbitraryUnitVector {
    fn arbitrary(g: &mut Gen) -> Self {
        ArbitraryUnitVector(arbitrary_unit_vector(g))
    }
}

/// Wrapper for arbitrary rotors
#[derive(Clone, Debug)]
pub struct ArbitraryRotor(pub GA3);

impl Arbitrary for ArbitraryRotor {
    fn arbitrary(g: &mut Gen) -> Self {
        ArbitraryRotor(arbitrary_rotor(g))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arbitrary_vector() {
        let mut gen = Gen::new(10);
        let v = arbitrary_vector(&mut gen);
        // Vector should only have grade 1 components
        assert!(v.get(0).abs() < 1e-10); // No scalar
        assert!(v.get(3).abs() < 1e-10); // No e12
        assert!(v.get(5).abs() < 1e-10); // No e13
        assert!(v.get(6).abs() < 1e-10); // No e23
        assert!(v.get(7).abs() < 1e-10); // No e123
    }

    #[test]
    fn test_arbitrary_unit_vector() {
        let mut gen = Gen::new(10);
        for _ in 0..10 {
            let v = arbitrary_unit_vector(&mut gen);
            let mag = v.magnitude();
            assert!((mag - 1.0).abs() < 1e-10, "Unit vector magnitude: {}", mag);
        }
    }

    #[test]
    fn test_arbitrary_rotor_is_unit() {
        let mut gen = Gen::new(10);
        for _ in 0..10 {
            let r = arbitrary_rotor(&mut gen);
            // A rotor should have magnitude close to 1
            let norm_sq = r.geometric_product(&r.reverse()).get(0);
            assert!(
                (norm_sq - 1.0).abs() < 0.1,
                "Rotor norm squared: {}",
                norm_sq
            );
        }
    }

    #[test]
    fn test_quickcheck_arbitrary() {
        fn prop_magnitude_positive(v: ArbitraryVector) -> bool {
            v.0.magnitude() >= 0.0
        }

        quickcheck::quickcheck(prop_magnitude_positive as fn(ArbitraryVector) -> bool);
    }
}
