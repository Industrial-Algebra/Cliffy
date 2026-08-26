// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Projections from geometric space to user types
//!
//! Projections extract meaningful values from multivector state.
//! They are the bridge between the geometric representation and
//! the user-facing types.
//!
//! # Built-in Projections
//!
//! - `ScalarProjection`: Extract the scalar (grade 0) component
//! - `VectorProjection`: Extract the vector (grade 1) components
//! - `PositionProjection`: Extract 3D position from vector components
//! - `ColorProjection`: Interpret components as RGB color

use crate::geometric::GA3;
use amari_core::{Bivector, Vector};

/// A projection extracts a user-facing type from a multivector.
///
/// Projections are the observer side of geometric state.
/// While state lives in geometric space, projections define
/// how that state appears to the user.
pub trait Projection: Send + Sync {
    /// The output type of this projection
    type Output;

    /// Project a multivector to the output type
    fn project(&self, mv: &GA3) -> Self::Output;

    /// Name of this projection (for debugging)
    fn name(&self) -> &str;
}

/// Project the scalar (grade 0) component
#[derive(Clone, Debug)]
pub struct ScalarProjection;

impl Projection for ScalarProjection {
    type Output = f64;

    fn project(&self, mv: &GA3) -> f64 {
        mv.get(0)
    }

    fn name(&self) -> &str {
        "scalar"
    }
}

/// Project the scalar component as an integer
#[derive(Clone, Debug)]
pub struct IntProjection;

impl Projection for IntProjection {
    type Output = i32;

    fn project(&self, mv: &GA3) -> i32 {
        mv.get(0) as i32
    }

    fn name(&self) -> &str {
        "int"
    }
}

/// Project the scalar component as a boolean (> 0.5 = true)
#[derive(Clone, Debug)]
pub struct BoolProjection;

impl Projection for BoolProjection {
    type Output = bool;

    fn project(&self, mv: &GA3) -> bool {
        mv.get(0) > 0.5
    }

    fn name(&self) -> &str {
        "bool"
    }
}

/// Project the vector (grade 1) components as a tuple
#[derive(Clone, Debug)]
pub struct VectorProjection;

impl Projection for VectorProjection {
    type Output = (f64, f64, f64);

    fn project(&self, mv: &GA3) -> (f64, f64, f64) {
        // GA3 indices: 1=e1, 2=e2, 4=e3
        (mv.get(1), mv.get(2), mv.get(4))
    }

    fn name(&self) -> &str {
        "vector"
    }
}

/// Project as 2D position (using e1 and e2 components)
#[derive(Clone, Debug)]
pub struct Position2DProjection;

impl Projection for Position2DProjection {
    type Output = (f64, f64);

    fn project(&self, mv: &GA3) -> (f64, f64) {
        (mv.get(1), mv.get(2))
    }

    fn name(&self) -> &str {
        "position2d"
    }
}

/// Project as 3D position (using e1, e2, e3 components)
#[derive(Clone, Debug)]
pub struct Position3DProjection;

impl Projection for Position3DProjection {
    type Output = (f64, f64, f64);

    fn project(&self, mv: &GA3) -> (f64, f64, f64) {
        (mv.get(1), mv.get(2), mv.get(4))
    }

    fn name(&self) -> &str {
        "position3d"
    }
}

/// Project bivector (grade 2) components
#[derive(Clone, Debug)]
pub struct BivectorProjection;

impl Projection for BivectorProjection {
    type Output = (f64, f64, f64);

    fn project(&self, mv: &GA3) -> (f64, f64, f64) {
        // GA3 indices: 3=e12, 5=e13, 6=e23
        (mv.get(3), mv.get(5), mv.get(6))
    }

    fn name(&self) -> &str {
        "bivector"
    }
}

/// Project the vector (grade 1) components as a typed `Vector<3,0,0>`
#[derive(Clone, Debug)]
pub struct TypedVectorProjection;

impl Projection for TypedVectorProjection {
    type Output = Vector<3, 0, 0>;

    fn project(&self, mv: &GA3) -> Vector<3, 0, 0> {
        Vector::from_components(mv.get(1), mv.get(2), mv.get(4))
    }

    fn name(&self) -> &str {
        "typed_vector"
    }
}

/// Project bivector (grade 2) components as a typed `Bivector<3,0,0>`
#[derive(Clone, Debug)]
pub struct TypedBivectorProjection;

impl Projection for TypedBivectorProjection {
    type Output = Bivector<3, 0, 0>;

    fn project(&self, mv: &GA3) -> Bivector<3, 0, 0> {
        Bivector::from_components(mv.get(3), mv.get(5), mv.get(6))
    }

    fn name(&self) -> &str {
        "typed_bivector"
    }
}

/// Project components as RGB color (clamped to 0-255)
///
/// Uses scalar for red, e1 for green, e2 for blue
#[derive(Clone, Debug)]
pub struct ColorProjection;

impl Projection for ColorProjection {
    type Output = (u8, u8, u8);

    fn project(&self, mv: &GA3) -> (u8, u8, u8) {
        let clamp = |v: f64| (v.clamp(0.0, 255.0)) as u8;
        (clamp(mv.get(0)), clamp(mv.get(1)), clamp(mv.get(2)))
    }

    fn name(&self) -> &str {
        "color"
    }
}

/// Project components as RGBA color (clamped to 0-255)
///
/// Uses scalar for red, e1 for green, e2 for blue, e3 for alpha
#[derive(Clone, Debug)]
pub struct ColorAlphaProjection;

impl Projection for ColorAlphaProjection {
    type Output = (u8, u8, u8, u8);

    fn project(&self, mv: &GA3) -> (u8, u8, u8, u8) {
        let clamp = |v: f64| (v.clamp(0.0, 255.0)) as u8;
        (
            clamp(mv.get(0)),
            clamp(mv.get(1)),
            clamp(mv.get(2)),
            clamp(mv.get(4)),
        )
    }

    fn name(&self) -> &str {
        "color_alpha"
    }
}

/// Project the magnitude (norm) of the multivector
#[derive(Clone, Debug)]
pub struct MagnitudeProjection;

impl Projection for MagnitudeProjection {
    type Output = f64;

    fn project(&self, mv: &GA3) -> f64 {
        mv.magnitude()
    }

    fn name(&self) -> &str {
        "magnitude"
    }
}

/// Project the angle represented by a rotor (scalar + bivector)
///
/// Assumes the multivector is a unit rotor: cos(θ/2) + sin(θ/2)B
#[derive(Clone, Debug)]
pub struct RotorAngleProjection;

impl Projection for RotorAngleProjection {
    type Output = f64;

    fn project(&self, mv: &GA3) -> f64 {
        // For a rotor R = cos(θ/2) + sin(θ/2)*B
        // The scalar part is cos(θ/2)
        let scalar = mv.get(0);
        2.0 * scalar.clamp(-1.0, 1.0).acos()
    }

    fn name(&self) -> &str {
        "rotor_angle"
    }
}

/// A mapping projection that applies a function to another projection's output
pub struct MappedProjection<P, F, U>
where
    P: Projection,
    F: Fn(P::Output) -> U + Send + Sync,
{
    inner: P,
    map_fn: F,
    name: String,
}

impl<P, F, U> MappedProjection<P, F, U>
where
    P: Projection,
    F: Fn(P::Output) -> U + Send + Sync,
{
    /// Create a new mapped projection
    pub fn new(inner: P, map_fn: F, name: impl Into<String>) -> Self {
        Self {
            inner,
            map_fn,
            name: name.into(),
        }
    }
}

impl<P, F, U> Projection for MappedProjection<P, F, U>
where
    P: Projection,
    F: Fn(P::Output) -> U + Send + Sync,
    U: Send + Sync,
{
    type Output = U;

    fn project(&self, mv: &GA3) -> U {
        (self.map_fn)(self.inner.project(mv))
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// A custom projection defined by a closure
pub struct CustomProjection<F, T>
where
    F: Fn(&GA3) -> T + Send + Sync,
{
    project_fn: F,
    name: String,
}

impl<F, T> CustomProjection<F, T>
where
    F: Fn(&GA3) -> T + Send + Sync,
{
    /// Create a new custom projection
    pub fn new(project_fn: F, name: impl Into<String>) -> Self {
        Self {
            project_fn,
            name: name.into(),
        }
    }
}

impl<F, T> Projection for CustomProjection<F, T>
where
    F: Fn(&GA3) -> T + Send + Sync,
    T: Send + Sync,
{
    type Output = T;

    fn project(&self, mv: &GA3) -> T {
        (self.project_fn)(mv)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amari_core::Vector;

    #[test]
    fn test_scalar_projection() {
        let mv = GA3::scalar(42.0);
        let proj = ScalarProjection;
        assert!((proj.project(&mv) - 42.0).abs() < 1e-10);
    }

    #[test]
    fn test_int_projection() {
        let mv = GA3::scalar(42.7);
        let proj = IntProjection;
        assert_eq!(proj.project(&mv), 42);
    }

    #[test]
    fn test_bool_projection() {
        let proj = BoolProjection;
        assert!(!proj.project(&GA3::scalar(0.0)));
        assert!(!proj.project(&GA3::scalar(0.4)));
        assert!(proj.project(&GA3::scalar(0.6)));
        assert!(proj.project(&GA3::scalar(1.0)));
    }

    #[test]
    fn test_vector_projection() {
        let v = Vector::<3, 0, 0>::from_components(1.0, 2.0, 3.0);
        let mv = GA3::from_vector(&v);
        let proj = VectorProjection;
        let (x, y, z) = proj.project(&mv);
        assert!((x - 1.0).abs() < 1e-10);
        assert!((y - 2.0).abs() < 1e-10);
        assert!((z - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_color_projection() {
        // Create a multivector with color values
        let mut coeffs = vec![0.0; 8];
        coeffs[0] = 128.0; // R
        coeffs[1] = 64.0; // G
        coeffs[2] = 192.0; // B
        let mv = GA3::from_coefficients(coeffs);

        let proj = ColorProjection;
        let (r, g, b) = proj.project(&mv);
        assert_eq!(r, 128);
        assert_eq!(g, 64);
        assert_eq!(b, 192);
    }

    #[test]
    fn test_color_clamping() {
        let mut coeffs = vec![0.0; 8];
        coeffs[0] = 300.0; // Over 255
        coeffs[1] = -50.0; // Under 0
        coeffs[2] = 100.0; // Normal
        let mv = GA3::from_coefficients(coeffs);

        let proj = ColorProjection;
        let (r, g, b) = proj.project(&mv);
        assert_eq!(r, 255); // Clamped
        assert_eq!(g, 0); // Clamped
        assert_eq!(b, 100);
    }

    #[test]
    fn test_magnitude_projection() {
        let v = Vector::<3, 0, 0>::from_components(3.0, 4.0, 0.0);
        let mv = GA3::from_vector(&v);
        let proj = MagnitudeProjection;
        assert!((proj.project(&mv) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_custom_projection() {
        let proj = CustomProjection::new(|mv: &GA3| mv.get(0) * 2.0, "doubled");
        let mv = GA3::scalar(21.0);
        assert!((proj.project(&mv) - 42.0).abs() < 1e-10);
        assert_eq!(proj.name(), "doubled");
    }

    #[test]
    fn test_typed_vector_projection() {
        let v = Vector::<3, 0, 0>::from_components(1.0, 2.0, 3.0);
        let mv = GA3::from_vector(&v);
        let proj = TypedVectorProjection;
        let result = proj.project(&mv);
        // GA3 vector indices: 1=e1, 2=e2, 4=e3
        assert!((result.mv.get(1) - 1.0).abs() < 1e-10);
        assert!((result.mv.get(2) - 2.0).abs() < 1e-10);
        assert!((result.mv.get(4) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_typed_bivector_projection() {
        let b = Bivector::<3, 0, 0>::from_components(0.5, 0.3, 0.1);
        let mv = GA3::from_bivector(&b);
        let proj = TypedBivectorProjection;
        let result = proj.project(&mv);
        assert!((result.get(0) - 0.5).abs() < 1e-10);
        assert!((result.get(1) - 0.3).abs() < 1e-10);
        assert!((result.get(2) - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_mapped_projection() {
        let proj = MappedProjection::new(ScalarProjection, |x| x as i32 * 2, "doubled_int");
        let mv = GA3::scalar(21.0);
        assert_eq!(proj.project(&mv), 42);
    }
}
