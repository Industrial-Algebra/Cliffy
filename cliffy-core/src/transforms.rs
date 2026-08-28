// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Geometric transformations for state manipulation
//!
//! This module provides explicit geometric transformation types:
//! - `Rotor`: Represents rotations in geometric algebra
//! - `Versor`: General geometric transformations (rotations + reflections)
//! - `Motor`: Translations combined with rotations (for CGA)
//!
//! These allow users to apply explicit geometric operations to state,
//! rather than hiding all geometry behind scalar updates.

use crate::geometric::GA3;
use amari_core::{Bivector, Rotor as AmariRotor, Vector};

/// A rotor represents a rotation in geometric algebra.
///
/// Rotors are even-grade multivectors with unit magnitude that
/// implement rotations via the sandwich product: R v R†
///
/// In 3D, a rotor encodes rotation by angle θ around an axis n as:
/// R = cos(θ/2) + sin(θ/2) * B
/// where B is the unit bivector representing the rotation plane.
#[derive(Clone, Debug)]
pub struct Rotor {
    /// The internal multivector representation
    inner: GA3,
}

impl Rotor {
    /// Create a rotor from a multivector (assumes it's already a valid rotor)
    #[must_use]
    pub const fn from_multivector(mv: GA3) -> Self {
        Self { inner: mv }
    }

    /// Create the identity rotor (no rotation)
    #[must_use]
    pub fn identity() -> Self {
        Self {
            inner: GA3::scalar(1.0),
        }
    }

    /// Create a rotor for rotation by angle (in radians) in the XY plane
    ///
    /// This is equivalent to rotation around the Z axis.
    #[must_use]
    pub fn xy(angle: f64) -> Self {
        Self::from_bivector_angle(angle, 1.0, 0.0, 0.0)
    }

    /// Create a rotor for rotation by angle (in radians) in the XZ plane
    ///
    /// This is equivalent to rotation around the Y axis.
    #[must_use]
    pub fn xz(angle: f64) -> Self {
        Self::from_bivector_angle(angle, 0.0, 1.0, 0.0)
    }

    /// Create a rotor for rotation by angle (in radians) in the YZ plane
    ///
    /// This is equivalent to rotation around the X axis.
    #[must_use]
    pub fn yz(angle: f64) -> Self {
        Self::from_bivector_angle(angle, 0.0, 0.0, 1.0)
    }

    /// Create a rotor from an angle and bivector components
    ///
    /// The bivector (xy, xz, yz) defines the rotation plane.
    /// Components are normalized internally.
    #[must_use]
    #[allow(clippy::arithmetic_side_effects)] // GA domain arithmetic: custom Multivector operators, pure and panic-free
    pub fn from_bivector_angle(angle: f64, xy: f64, xz: f64, yz: f64) -> Self {
        let half_angle = angle / 2.0;

        // Create unit bivector
        // Note: Negate to match standard right-hand rotation convention
        let biv = Bivector::<3, 0, 0>::from_components(-xy, -xz, -yz);
        let biv_mv = GA3::from_bivector(&biv);
        let mag = biv_mv.magnitude();

        if mag < 1e-10 {
            // Degenerate case - return identity
            return Self::identity();
        }

        let biv_unit = &biv_mv * (1.0 / mag);

        // R = cos(θ/2) + sin(θ/2) * B
        let cos_part = GA3::scalar(half_angle.cos());
        let sin_part = &biv_unit * half_angle.sin();

        Self {
            inner: &cos_part + &sin_part,
        }
    }

    /// Create a rotor for rotation around an axis vector by an angle
    ///
    /// The axis does not need to be normalized.
    #[must_use]
    pub fn from_axis_angle(axis_x: f64, axis_y: f64, axis_z: f64, angle: f64) -> Self {
        // The bivector for rotation around axis (x,y,z) is proportional to
        // x*(e2^e3) + y*(e3^e1) + z*(e1^e2)
        // which in our basis order (e12, e13, e23) is:
        // xy = z, xz = -y, yz = x
        Self::from_bivector_angle(angle, axis_z, -axis_y, axis_x)
    }

    /// Get the internal multivector
    #[must_use]
    pub const fn as_multivector(&self) -> &GA3 {
        &self.inner
    }

    /// Apply this rotor to transform a multivector (sandwich product)
    ///
    /// Returns R * v * R†
    #[allow(clippy::arithmetic_side_effects)] // GA domain arithmetic: custom Multivector operators, pure and panic-free
    #[must_use]
    pub fn transform(&self, v: &GA3) -> GA3 {
        let rev = self.inner.reverse();
        self.inner.geometric_product(v).geometric_product(&rev)
    }

    /// Compose two rotors: self followed by other
    ///
    /// The result applies self first, then other.
    #[must_use]
    pub fn then(&self, other: &Self) -> Self {
        Self {
            inner: other.inner.geometric_product(&self.inner),
        }
    }

    /// Get the inverse rotor (reverse rotation)
    #[must_use]
    pub fn inverse(&self) -> Self {
        // For unit rotors, inverse = reverse
        Self {
            inner: self.inner.reverse(),
        }
    }

    /// Normalize the rotor to unit magnitude
    #[must_use]
    pub fn normalize(&self) -> Self {
        self.inner
            .normalize()
            .map_or_else(Self::identity, |normalized| Self { inner: normalized })
    }

    /// Get the rotation angle (in radians)
    #[must_use]
    pub fn angle(&self) -> f64 {
        // For a rotor R = cos(θ/2) + sin(θ/2)*B
        // The scalar part is cos(θ/2)
        let scalar = self.inner.get(0);
        2.0 * scalar.clamp(-1.0, 1.0).acos()
    }

    /// Convert to an amari-core `Rotor<3,0,0>`
    ///
    /// This enables interop with amari-core's typed rotor operations
    /// (slerp, compose, logarithm, power, etc.)
    #[must_use]
    pub fn to_amari_rotor(&self) -> AmariRotor<3, 0, 0> {
        // Extract and normalize the bivector plane
        let e12 = self.inner.get(3);
        let e13 = self.inner.get(5);
        let e23 = self.inner.get(6);
        let biv_mag = e23.mul_add(e23, e13.mul_add(e13, e12 * e12)).sqrt();

        if biv_mag < 1e-10 {
            // Identity or near-identity rotor
            return AmariRotor::identity();
        }

        // Create unit bivector (negate to match amari convention;
        // cliffy negates bivector components in from_bivector_angle)
        let biv =
            Bivector::<3, 0, 0>::from_components(-e12 / biv_mag, -e13 / biv_mag, -e23 / biv_mag);
        AmariRotor::from_bivector(&biv, self.angle())
    }

    /// Create from an amari-core `Rotor<3,0,0>`
    #[must_use]
    pub fn from_amari_rotor(rotor: &AmariRotor<3, 0, 0>) -> Self {
        Self {
            inner: rotor.as_multivector().clone(),
        }
    }

    /// Spherical linear interpolation between identity and this rotor
    ///
    /// t=0 gives identity, t=1 gives self
    #[must_use]
    pub fn slerp(&self, t: f64) -> Self {
        let angle = self.angle();
        let new_angle = angle * t;

        // Extract the bivector part and renormalize
        // Note: Internal representation has negated bivector, so negate when extracting
        let biv_xy = -self.inner.get(3); // e12
        let biv_xz = -self.inner.get(5); // e13
        let biv_yz = -self.inner.get(6); // e23
        let biv_mag = biv_yz
            .mul_add(biv_yz, biv_xz.mul_add(biv_xz, biv_xy * biv_xy))
            .sqrt();

        if biv_mag < 1e-10 {
            // No rotation - return identity
            Self::identity()
        } else {
            Self::from_bivector_angle(
                new_angle,
                biv_xy / biv_mag,
                biv_xz / biv_mag,
                biv_yz / biv_mag,
            )
        }
    }

    /// Spherical linear interpolation between two rotors
    #[must_use]
    pub fn slerp_to(&self, other: &Self, t: f64) -> Self {
        // Compute relative rotation: other = relative * self
        // So relative = other * self.inverse()
        let relative = other.then(&self.inverse());
        let interpolated = relative.slerp(t);
        interpolated.then(self)
    }
}

/// A versor is a general geometric transformation (rotation, reflection, or their composition).
///
/// Every versor can be written as a product of vectors.
/// Versors with an even number of vectors are rotors.
/// Versors with an odd number of vectors include reflections.
#[derive(Clone, Debug)]
pub struct Versor {
    /// The internal multivector representation
    inner: GA3,
    /// Whether this is an even versor (rotor) or odd versor (includes reflection)
    is_even: bool,
}

impl Versor {
    /// Create a versor from a multivector
    #[must_use]
    pub const fn from_multivector(mv: GA3, is_even: bool) -> Self {
        Self { inner: mv, is_even }
    }

    /// Create the identity versor
    #[must_use]
    pub fn identity() -> Self {
        Self {
            inner: GA3::scalar(1.0),
            is_even: true,
        }
    }

    /// Create a reflection through a plane with normal vector (x, y, z)
    ///
    /// Reflects points through the plane passing through origin with the given normal.
    #[must_use]
    pub fn reflection(normal_x: f64, normal_y: f64, normal_z: f64) -> Self {
        // A reflection is represented by the unit vector normal to the plane
        let vec = Vector::<3, 0, 0>::from_components(normal_x, normal_y, normal_z);
        let mv = GA3::from_vector(&vec);
        let normalized = mv
            .normalize()
            .unwrap_or_else(|| GA3::from_vector(&Vector::from_components(1.0, 0.0, 0.0)));

        Self {
            inner: normalized,
            is_even: false,
        }
    }

    /// Convert from a Rotor
    #[must_use]
    pub fn from_rotor(rotor: Rotor) -> Self {
        Self {
            inner: rotor.inner,
            is_even: true,
        }
    }

    /// Get the internal multivector
    #[must_use]
    pub const fn as_multivector(&self) -> &GA3 {
        &self.inner
    }

    /// Apply this versor to transform a multivector
    ///
    /// For even versors: V * v * V†
    /// For odd versors: V * v * V† (with grade involution handling)
    #[allow(clippy::arithmetic_side_effects)] // GA domain arithmetic: custom Multivector operators, pure and panic-free
    #[must_use]
    pub fn transform(&self, v: &GA3) -> GA3 {
        let rev = self.inner.reverse();
        if self.is_even {
            self.inner.geometric_product(v).geometric_product(&rev)
        } else {
            // For odd versors, we need to handle the sign change
            // This is simplified - full implementation would use grade involution
            let result = self.inner.geometric_product(v).geometric_product(&rev);
            &result * -1.0
        }
    }

    /// Compose two versors
    #[must_use]
    pub fn then(&self, other: &Self) -> Self {
        Self {
            inner: other.inner.geometric_product(&self.inner),
            is_even: self.is_even == other.is_even, // Even * Even = Even, Odd * Odd = Even
        }
    }

    /// Check if this is an even versor (rotor)
    #[must_use]
    pub const fn is_rotor(&self) -> bool {
        self.is_even
    }

    /// Try to convert to a Rotor (only succeeds for even versors)
    #[must_use]
    pub fn to_rotor(&self) -> Option<Rotor> {
        if self.is_even {
            Some(Rotor {
                inner: self.inner.clone(),
            })
        } else {
            None
        }
    }
}

/// A translation represented geometrically
///
/// In standard GA3 (Euclidean), translations are not directly representable
/// as versors. This type provides a convenient API that internally uses
/// vector addition.
#[derive(Clone, Debug)]
pub struct Translation {
    /// Translation vector components
    x: f64,
    y: f64,
    z: f64,
}

impl Translation {
    /// Create a new translation
    #[must_use]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Create a translation along the X axis
    #[must_use]
    pub const fn x(amount: f64) -> Self {
        Self::new(amount, 0.0, 0.0)
    }

    /// Create a translation along the Y axis
    #[must_use]
    pub const fn y(amount: f64) -> Self {
        Self::new(0.0, amount, 0.0)
    }

    /// Create a translation along the Z axis
    #[must_use]
    pub const fn z(amount: f64) -> Self {
        Self::new(0.0, 0.0, amount)
    }

    /// Apply this translation to a multivector
    ///
    /// For vectors, this adds the translation. For other grades,
    /// behavior depends on the geometric interpretation.
    #[allow(clippy::arithmetic_side_effects)] // GA domain arithmetic: custom Multivector operators, pure and panic-free
    #[must_use]
    pub fn transform(&self, v: &GA3) -> GA3 {
        let trans_vec = Vector::<3, 0, 0>::from_components(self.x, self.y, self.z);
        let trans_mv = GA3::from_vector(&trans_vec);
        v + &trans_mv
    }

    /// Compose two translations
    #[must_use]
    pub fn then(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    /// Get the inverse translation
    #[must_use]
    pub fn inverse(&self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }

    /// Linear interpolation of translation
    #[must_use]
    pub fn lerp(&self, t: f64) -> Self {
        Self {
            x: self.x * t,
            y: self.y * t,
            z: self.z * t,
        }
    }

    /// Linear interpolation to another translation
    #[must_use]
    pub fn lerp_to(&self, other: &Self, t: f64) -> Self {
        Self {
            x: (other.x - self.x).mul_add(t, self.x),
            y: (other.y - self.y).mul_add(t, self.y),
            z: (other.z - self.z).mul_add(t, self.z),
        }
    }
}

/// A general geometric transformation combining rotation and translation
#[derive(Clone, Debug)]
pub struct Transform {
    /// Rotation component
    rotor: Rotor,
    /// Translation component (applied after rotation)
    translation: Translation,
}

impl Transform {
    /// Create a new transform with rotation and translation
    #[must_use]
    pub const fn new(rotor: Rotor, translation: Translation) -> Self {
        Self { rotor, translation }
    }

    /// Create identity transform
    #[must_use]
    pub fn identity() -> Self {
        Self {
            rotor: Rotor::identity(),
            translation: Translation::new(0.0, 0.0, 0.0),
        }
    }

    /// Create a pure rotation transform
    #[must_use]
    pub const fn rotation(rotor: Rotor) -> Self {
        Self {
            rotor,
            translation: Translation::new(0.0, 0.0, 0.0),
        }
    }

    /// Create a pure translation transform
    #[must_use]
    pub fn translation(translation: Translation) -> Self {
        Self {
            rotor: Rotor::identity(),
            translation,
        }
    }

    /// Apply this transform to a multivector
    #[must_use]
    pub fn transform(&self, v: &GA3) -> GA3 {
        let rotated = self.rotor.transform(v);
        self.translation.transform(&rotated)
    }

    /// Compose two transforms: self followed by other
    #[must_use]
    pub fn then(&self, other: &Self) -> Self {
        // First apply self's rotation, then self's translation
        // Then apply other's rotation, then other's translation
        //
        // Combined rotation: other.rotor * self.rotor
        // Translation is more complex due to rotation interaction
        let combined_rotor = self.rotor.then(&other.rotor);

        // Transform self's translation by other's rotation, then add other's translation
        let self_trans_vec = Vector::<3, 0, 0>::from_components(
            self.translation.x,
            self.translation.y,
            self.translation.z,
        );
        let self_trans_mv = GA3::from_vector(&self_trans_vec);
        let rotated_trans = other.rotor.transform(&self_trans_mv);

        let combined_translation = Translation::new(
            rotated_trans.get(1) + other.translation.x,
            rotated_trans.get(2) + other.translation.y,
            rotated_trans.get(4) + other.translation.z,
        );

        Self {
            rotor: combined_rotor,
            translation: combined_translation,
        }
    }

    /// Get the inverse transform
    #[must_use]
    pub fn inverse(&self) -> Self {
        let inv_rotor = self.rotor.inverse();
        let inv_trans_base = self.translation.inverse();

        // Rotate the inverse translation by the inverse rotation
        let trans_vec = Vector::<3, 0, 0>::from_components(
            inv_trans_base.x,
            inv_trans_base.y,
            inv_trans_base.z,
        );
        let trans_mv = GA3::from_vector(&trans_vec);
        let rotated = inv_rotor.transform(&trans_mv);

        Self {
            rotor: inv_rotor,
            translation: Translation::new(rotated.get(1), rotated.get(2), rotated.get(4)),
        }
    }

    /// Interpolate between identity and this transform
    #[must_use]
    pub fn interpolate(&self, t: f64) -> Self {
        Self {
            rotor: self.rotor.slerp(t),
            translation: self.translation.lerp(t),
        }
    }

    /// Interpolate to another transform
    #[must_use]
    pub fn interpolate_to(&self, other: &Self, t: f64) -> Self {
        Self {
            rotor: self.rotor.slerp_to(&other.rotor, t),
            translation: self.translation.lerp_to(&other.translation, t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_rotor_identity() {
        let r = Rotor::identity();
        let v = Vector::<3, 0, 0>::from_components(1.0, 2.0, 3.0);
        let mv = GA3::from_vector(&v);

        let rotated = r.transform(&mv);

        // Identity should not change the vector
        assert!((rotated.get(1) - 1.0).abs() < 1e-10);
        assert!((rotated.get(2) - 2.0).abs() < 1e-10);
        assert!((rotated.get(4) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotor_xy_90() {
        // 90 degree rotation in XY plane
        let r = Rotor::xy(PI / 2.0);

        // Rotate (1, 0, 0) should give (0, 1, 0)
        let v = GA3::from_vector(&Vector::from_components(1.0, 0.0, 0.0));
        let rotated = r.transform(&v);

        assert!((rotated.get(1) - 0.0).abs() < 1e-10); // x -> 0
        assert!((rotated.get(2) - 1.0).abs() < 1e-10); // y -> 1
        assert!((rotated.get(4) - 0.0).abs() < 1e-10); // z -> 0
    }

    #[test]
    fn test_rotor_preserves_magnitude() {
        let r = Rotor::from_bivector_angle(1.23, 1.0, 2.0, 3.0);
        let v = GA3::from_vector(&Vector::from_components(3.0, 4.0, 5.0));

        let original_mag = v.magnitude();
        let rotated = r.transform(&v);
        let rotated_mag = rotated.magnitude();

        assert!(
            (original_mag - rotated_mag).abs() < 1e-10,
            "Magnitude changed: {original_mag} -> {rotated_mag}"
        );
    }

    #[test]
    fn test_rotor_composition() {
        // Two 90-degree rotations should equal one 180-degree rotation
        let r1 = Rotor::xy(PI / 2.0);
        let r2 = Rotor::xy(PI / 2.0);
        let r_combined = r1.then(&r2);

        let v = GA3::from_vector(&Vector::from_components(1.0, 0.0, 0.0));
        let rotated = r_combined.transform(&v);

        // (1, 0, 0) rotated 180 degrees should give (-1, 0, 0)
        assert!((rotated.get(1) + 1.0).abs() < 1e-10);
        assert!((rotated.get(2) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotor_inverse() {
        let r = Rotor::from_bivector_angle(0.7, 1.0, 1.0, 1.0);
        let v = GA3::from_vector(&Vector::from_components(1.0, 2.0, 3.0));

        let rotated = r.transform(&v);
        let back = r.inverse().transform(&rotated);

        assert!((back.get(1) - 1.0).abs() < 1e-10);
        assert!((back.get(2) - 2.0).abs() < 1e-10);
        assert!((back.get(4) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotor_slerp() {
        let r = Rotor::xy(PI);

        // Halfway interpolation should give PI/2 rotation
        let half = r.slerp(0.5);

        let v = GA3::from_vector(&Vector::from_components(1.0, 0.0, 0.0));
        let rotated = half.transform(&v);

        // (1, 0, 0) rotated 90 degrees should give (0, 1, 0)
        assert!((rotated.get(1) - 0.0).abs() < 1e-10);
        assert!((rotated.get(2) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_translation() {
        let t = Translation::new(1.0, 2.0, 3.0);
        let v = GA3::from_vector(&Vector::from_components(0.0, 0.0, 0.0));

        let translated = t.transform(&v);

        assert!((translated.get(1) - 1.0).abs() < 1e-10);
        assert!((translated.get(2) - 2.0).abs() < 1e-10);
        assert!((translated.get(4) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_transform_composition() {
        let rot = Rotor::xy(PI / 2.0);
        let trans = Translation::new(1.0, 0.0, 0.0);
        let transform = Transform::new(rot, trans);

        let v = GA3::from_vector(&Vector::from_components(1.0, 0.0, 0.0));
        let result = transform.transform(&v);

        // First rotate (1,0,0) by 90deg -> (0,1,0)
        // Then translate by (1,0,0) -> (1,1,0)
        assert!((result.get(1) - 1.0).abs() < 1e-10);
        assert!((result.get(2) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_amari_rotor_roundtrip() {
        let r = Rotor::xy(PI / 3.0);
        let amari_r = r.to_amari_rotor();

        // The amari rotor should have the same angle
        let scalar = amari_r.as_multivector().get(0);
        let angle = scalar.clamp(-1.0, 1.0).acos() * 2.0;
        assert!((angle - PI / 3.0).abs() < 1e-10);

        // Roundtrip: convert back
        let r2 = Rotor::from_amari_rotor(&amari_r);
        let v = GA3::from_vector(&Vector::from_components(1.0, 0.0, 0.0));

        let result1 = r.transform(&v);
        let result2 = r2.transform(&v);

        assert!((result1.get(1) - result2.get(1)).abs() < 1e-10);
        assert!((result1.get(2) - result2.get(2)).abs() < 1e-10);
        assert!((result1.get(4) - result2.get(4)).abs() < 1e-10);
    }
}
