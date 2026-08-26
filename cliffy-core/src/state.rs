// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Geometric state with explicit transformations
//!
//! `GeometricState` exposes the geometric algebra operations that are
//! hidden in regular `Behavior`. Use this when you want explicit control
//! over geometric transformations.
//!
//! # Example
//!
//! ```rust
//! use cliffy_core::{GeometricState, Rotor, Translation};
//! use std::f64::consts::PI;
//!
//! // Create state from initial position
//! let state = GeometricState::from_vector(1.0, 0.0, 0.0);
//!
//! // Apply explicit geometric transformations
//! let rotated = state.apply_rotor(&Rotor::xy(PI / 2.0));
//! let translated = rotated.apply_translation(&Translation::new(1.0, 0.0, 0.0));
//!
//! // Project to user types
//! let (x, y, z) = translated.as_vector();
//! ```

use crate::geometric::GA3;
use crate::projection::Projection;
use crate::transforms::{Rotor, Transform, Translation, Versor};
use amari_core::{Bivector, Vector};
use parking_lot::Mutex;
use std::sync::Arc;

/// Type alias for subscriber callbacks to avoid `clippy::type_complexity` warning
type SubscriberList = Arc<Mutex<Vec<Box<dyn Fn(&GA3) + Send + Sync>>>>;

/// State that lives in geometric space with explicit transformation support.
///
/// Unlike `Behavior<T>`, which hides the geometric representation,
/// `GeometricState` exposes it directly. This is useful for:
/// - Animation with geometric interpolation (SLERP)
/// - Physics simulations
/// - Explicit rotation/translation control
/// - Advanced users who understand geometric algebra
#[derive(Clone)]
pub struct GeometricState {
    /// The underlying multivector
    inner: Arc<Mutex<GA3>>,
    /// Subscribers for reactive updates
    subscribers: SubscriberList,
}

impl GeometricState {
    /// Create a new geometric state from a multivector
    #[must_use]
    pub fn new(mv: GA3) -> Self {
        Self {
            inner: Arc::new(Mutex::new(mv)),
            subscribers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create state representing a scalar value
    #[must_use]
    pub fn from_scalar(value: f64) -> Self {
        Self::new(GA3::scalar(value))
    }

    /// Create state representing a 3D vector/position
    #[must_use]
    pub fn from_vector(x: f64, y: f64, z: f64) -> Self {
        let v = Vector::<3, 0, 0>::from_components(x, y, z);
        Self::new(GA3::from_vector(&v))
    }

    /// Create state representing a bivector (rotation plane)
    #[must_use]
    pub fn from_bivector(xy: f64, xz: f64, yz: f64) -> Self {
        let b = Bivector::<3, 0, 0>::from_components(xy, xz, yz);
        Self::new(GA3::from_bivector(&b))
    }

    /// Create state from raw coefficients
    #[must_use]
    pub fn from_coefficients(coeffs: Vec<f64>) -> Self {
        Self::new(GA3::from_coefficients(coeffs))
    }

    /// Create the zero state
    #[must_use]
    pub fn zero() -> Self {
        Self::new(GA3::zero())
    }

    /// Create the identity state (scalar 1)
    #[must_use]
    pub fn identity() -> Self {
        Self::new(GA3::scalar(1.0))
    }

    /// Get the underlying multivector (cloned)
    #[must_use]
    pub fn multivector(&self) -> GA3 {
        self.inner.lock().clone()
    }

    /// Get the raw coefficient at a given index
    #[must_use]
    pub fn get(&self, index: usize) -> f64 {
        self.inner.lock().get(index)
    }

    /// Get the scalar component
    #[must_use]
    pub fn scalar(&self) -> f64 {
        self.get(0)
    }

    /// Get the vector components (e1, e2, e3) as a tuple
    #[must_use]
    pub fn as_vector(&self) -> (f64, f64, f64) {
        let mv = self.inner.lock();
        (mv.get(1), mv.get(2), mv.get(4))
    }

    /// Get the vector components as a typed `Vector<3,0,0>`
    #[must_use]
    pub fn as_typed_vector(&self) -> Vector<3, 0, 0> {
        let mv = self.inner.lock();
        Vector::from_components(mv.get(1), mv.get(2), mv.get(4))
    }

    /// Get the bivector components (e12, e13, e23) as a tuple
    #[must_use]
    pub fn as_bivector(&self) -> (f64, f64, f64) {
        let mv = self.inner.lock();
        (mv.get(3), mv.get(5), mv.get(6))
    }

    /// Get the bivector components as a typed `Bivector<3,0,0>`
    #[must_use]
    pub fn as_typed_bivector(&self) -> Bivector<3, 0, 0> {
        let mv = self.inner.lock();
        Bivector::from_components(mv.get(3), mv.get(5), mv.get(6))
    }

    /// Get the magnitude (norm) of the state
    #[must_use]
    pub fn magnitude(&self) -> f64 {
        self.inner.lock().magnitude()
    }

    /// Project using a projection type
    pub fn project<P: Projection>(&self, projection: &P) -> P::Output {
        let mv = self.inner.lock();
        projection.project(&mv)
    }

    /// Set the state to a new multivector
    pub fn set(&self, mv: GA3) {
        {
            let mut inner = self.inner.lock();
            *inner = mv;
        }
        self.notify_subscribers();
    }

    /// Set the scalar value
    pub fn set_scalar(&self, value: f64) {
        self.set(GA3::scalar(value));
    }

    /// Set the vector value from components
    pub fn set_vector(&self, x: f64, y: f64, z: f64) {
        let v = Vector::<3, 0, 0>::from_components(x, y, z);
        self.set(GA3::from_vector(&v));
    }

    /// Set the vector value from a typed `Vector<3,0,0>`
    pub fn set_typed_vector(&self, v: &Vector<3, 0, 0>) {
        self.set(GA3::from_vector(v));
    }

    /// Set the bivector value from a typed `Bivector<3,0,0>`
    pub fn set_typed_bivector(&self, b: &Bivector<3, 0, 0>) {
        self.set(GA3::from_bivector(b));
    }

    /// Update the state by applying a function
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&GA3) -> GA3,
    {
        {
            let mut inner = self.inner.lock();
            *inner = f(&inner);
        }
        self.notify_subscribers();
    }

    /// Apply a rotor transformation (rotation)
    ///
    /// This creates a new state; the original is unchanged.
    #[must_use]
    pub fn apply_rotor(&self, rotor: &Rotor) -> Self {
        let mv = self.inner.lock();
        let transformed = rotor.transform(&mv);
        Self::new(transformed)
    }

    /// Apply a rotor transformation in-place
    pub fn apply_rotor_mut(&self, rotor: &Rotor) {
        self.update(|mv| rotor.transform(mv));
    }

    /// Apply a translation
    ///
    /// This creates a new state; the original is unchanged.
    #[must_use]
    pub fn apply_translation(&self, translation: &Translation) -> Self {
        let mv = self.inner.lock();
        let transformed = translation.transform(&mv);
        Self::new(transformed)
    }

    /// Apply a translation in-place
    pub fn apply_translation_mut(&self, translation: &Translation) {
        self.update(|mv| translation.transform(mv));
    }

    /// Apply a versor transformation
    ///
    /// This creates a new state; the original is unchanged.
    #[must_use]
    pub fn apply_versor(&self, versor: &Versor) -> Self {
        let mv = self.inner.lock();
        let transformed = versor.transform(&mv);
        Self::new(transformed)
    }

    /// Apply a versor transformation in-place
    pub fn apply_versor_mut(&self, versor: &Versor) {
        self.update(|mv| versor.transform(mv));
    }

    /// Apply a general transform (rotation + translation)
    ///
    /// This creates a new state; the original is unchanged.
    #[must_use]
    pub fn apply_transform(&self, transform: &Transform) -> Self {
        let mv = self.inner.lock();
        let transformed = transform.transform(&mv);
        Self::new(transformed)
    }

    /// Apply a general transform in-place
    pub fn apply_transform_mut(&self, transform: &Transform) {
        self.update(|mv| transform.transform(mv));
    }

    /// Add another state (geometric addition)
    #[must_use]
    #[allow(clippy::arithmetic_side_effects)] // GA domain arithmetic: custom Multivector operators, pure and panic-free
    pub fn add(&self, other: &Self) -> Self {
        let a = self.inner.lock();
        let b = other.inner.lock();
        Self::new(&*a + &*b)
    }

    /// Subtract another state
    #[must_use]
    #[allow(clippy::arithmetic_side_effects)] // GA domain arithmetic: custom Multivector operators, pure and panic-free
    pub fn sub(&self, other: &Self) -> Self {
        let a = self.inner.lock();
        let b = other.inner.lock();
        Self::new(&*a - &*b)
    }

    /// Scale by a scalar value
    #[must_use]
    #[allow(clippy::arithmetic_side_effects)] // GA domain arithmetic: custom Multivector operators, pure and panic-free
    pub fn scale(&self, factor: f64) -> Self {
        let mv = self.inner.lock();
        Self::new(&*mv * factor)
    }

    /// Geometric product with another state
    #[must_use]
    pub fn geometric_product(&self, other: &Self) -> Self {
        let a = self.inner.lock();
        let b = other.inner.lock();
        Self::new(a.geometric_product(&b))
    }

    /// Normalize the state to unit magnitude
    pub fn normalize(&self) -> Option<Self> {
        let mv = self.inner.lock();
        mv.normalize().map(Self::new)
    }

    /// Normalize in-place
    #[must_use]
    pub fn normalize_mut(&self) -> bool {
        let mut inner = self.inner.lock();
        if let Some(normalized) = inner.normalize() {
            *inner = normalized;
            drop(inner);
            self.notify_subscribers();
            true
        } else {
            false
        }
    }

    /// Get the reverse (reversion) of this state
    #[must_use]
    pub fn reverse(&self) -> Self {
        let mv = self.inner.lock();
        Self::new(mv.reverse())
    }

    /// Linear interpolation to another state
    #[must_use]
    #[allow(clippy::arithmetic_side_effects)] // GA domain arithmetic: custom Multivector operators, pure and panic-free
    pub fn lerp(&self, other: &Self, t: f64) -> Self {
        let a = self.inner.lock();
        let b = other.inner.lock();

        // lerp = a + t * (b - a) = (1-t)*a + t*b
        let diff = &*b - &*a;
        let interpolated = &*a + &(&diff * t);
        Self::new(interpolated)
    }

    /// Spherical linear interpolation (for rotor-like states)
    ///
    /// This assumes both states are unit rotors.
    #[must_use]
    pub fn slerp(&self, other: &Self, t: f64) -> Self {
        // Convert to rotors and use rotor SLERP
        let a_mv = self.inner.lock();
        let b_mv = other.inner.lock();

        let rotor_a = Rotor::from_multivector(a_mv.clone());
        let rotor_b = Rotor::from_multivector(b_mv.clone());

        let interpolated = rotor_a.slerp_to(&rotor_b, t);
        Self::new(interpolated.as_multivector().clone())
    }

    /// Subscribe to state changes
    pub fn subscribe<F>(&self, callback: F) -> GeometricSubscription
    where
        F: Fn(&GA3) + Send + Sync + 'static,
    {
        let mut subs = self.subscribers.lock();
        let id = subs.len();
        subs.push(Box::new(callback));

        GeometricSubscription {
            id,
            subscribers: self.subscribers.clone(),
        }
    }

    /// Notify all subscribers of a state change
    fn notify_subscribers(&self) {
        let mv = self.inner.lock();
        let subs = self.subscribers.lock();
        for callback in subs.iter() {
            callback(&mv);
        }
    }
}

impl std::fmt::Debug for GeometricState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mv = self.inner.lock();
        write!(f, "GeometricState({mv:?})")
    }
}

/// A subscription handle for geometric state changes
pub struct GeometricSubscription {
    id: usize,
    subscribers: SubscriberList,
}

impl GeometricSubscription {
    /// Unsubscribe from updates
    ///
    /// Note: This doesn't actually remove the callback (to maintain indices),
    /// but marks it as inactive. In practice, subscriptions typically last
    /// for the lifetime of components.
    pub fn unsubscribe(self) {
        // In a more sophisticated implementation, we'd mark the slot as inactive
        // For now, we just drop the handle
        let _ = (self.id, self.subscribers);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projection::{IntProjection, ScalarProjection, VectorProjection};
    use std::f64::consts::PI;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_from_scalar() {
        let state = GeometricState::from_scalar(42.0);
        assert!((state.scalar() - 42.0).abs() < 1e-10);
    }

    #[test]
    fn test_from_vector() {
        let state = GeometricState::from_vector(1.0, 2.0, 3.0);
        let (x, y, z) = state.as_vector();
        assert!((x - 1.0).abs() < 1e-10);
        assert!((y - 2.0).abs() < 1e-10);
        assert!((z - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_apply_rotor() {
        let state = GeometricState::from_vector(1.0, 0.0, 0.0);
        let rotor = Rotor::xy(PI / 2.0);
        let rotated = state.apply_rotor(&rotor);

        let (x, y, z) = rotated.as_vector();
        assert!(x.abs() < 1e-10, "x should be ~0, got {x}");
        assert!((y - 1.0).abs() < 1e-10, "y should be ~1, got {y}");
        assert!(z.abs() < 1e-10, "z should be ~0, got {z}");
    }

    #[test]
    fn test_apply_translation() {
        let state = GeometricState::from_vector(0.0, 0.0, 0.0);
        let trans = Translation::new(1.0, 2.0, 3.0);
        let translated = state.apply_translation(&trans);

        let (x, y, z) = translated.as_vector();
        assert!((x - 1.0).abs() < 1e-10);
        assert!((y - 2.0).abs() < 1e-10);
        assert!((z - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_apply_transform() {
        let state = GeometricState::from_vector(1.0, 0.0, 0.0);
        let transform = Transform::new(Rotor::xy(PI / 2.0), Translation::new(1.0, 0.0, 0.0));

        let result = state.apply_transform(&transform);
        let (x, y, z) = result.as_vector();

        // Rotate (1,0,0) by 90deg -> (0,1,0), then translate -> (1,1,0)
        assert!((x - 1.0).abs() < 1e-10, "x should be ~1, got {x}");
        assert!((y - 1.0).abs() < 1e-10, "y should be ~1, got {y}");
        assert!(z.abs() < 1e-10);
    }

    #[test]
    fn test_lerp() {
        let a = GeometricState::from_scalar(0.0);
        let b = GeometricState::from_scalar(10.0);

        let half = a.lerp(&b, 0.5);
        assert!((half.scalar() - 5.0).abs() < 1e-10);

        let quarter = a.lerp(&b, 0.25);
        assert!((quarter.scalar() - 2.5).abs() < 1e-10);
    }

    #[test]
    fn test_projection() {
        let state = GeometricState::from_scalar(42.7);

        let scalar = state.project(&ScalarProjection);
        assert!((scalar - 42.7).abs() < 1e-10);

        let int = state.project(&IntProjection);
        assert_eq!(int, 42);
    }

    #[test]
    fn test_vector_projection() {
        let state = GeometricState::from_vector(1.0, 2.0, 3.0);
        let (x, y, z) = state.project(&VectorProjection);
        assert!((x - 1.0).abs() < 1e-10);
        assert!((y - 2.0).abs() < 1e-10);
        assert!((z - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_subscribe() {
        let state = GeometricState::from_scalar(0.0);
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        let _sub = state.subscribe(move |_mv| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        state.set_scalar(1.0);
        state.set_scalar(2.0);

        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_typed_vector_accessors() {
        let state = GeometricState::from_vector(1.0, 2.0, 3.0);
        let v = state.as_typed_vector();
        // GA3 vector indices: 1=e1, 2=e2, 4=e3
        assert!((v.mv.get(1) - 1.0).abs() < 1e-10);
        assert!((v.mv.get(2) - 2.0).abs() < 1e-10);
        assert!((v.mv.get(4) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_typed_bivector_accessors() {
        let state = GeometricState::from_bivector(0.5, 0.3, 0.1);
        let b = state.as_typed_bivector();
        assert!((b.get(0) - 0.5).abs() < 1e-10);
        assert!((b.get(1) - 0.3).abs() < 1e-10);
        assert!((b.get(2) - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_set_typed_vector() {
        let state = GeometricState::zero();
        let v = Vector::<3, 0, 0>::from_components(4.0, 5.0, 6.0);
        state.set_typed_vector(&v);
        let result = state.as_typed_vector();
        // GA3 vector indices: 1=e1, 2=e2, 4=e3
        assert!((result.mv.get(1) - 4.0).abs() < 1e-10);
        assert!((result.mv.get(2) - 5.0).abs() < 1e-10);
        assert!((result.mv.get(4) - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_scale() {
        let state = GeometricState::from_vector(1.0, 2.0, 3.0);
        let scaled = state.scale(2.0);

        let (x, y, z) = scaled.as_vector();
        assert!((x - 2.0).abs() < 1e-10);
        assert!((y - 4.0).abs() < 1e-10);
        assert!((z - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_normalize() {
        let state = GeometricState::from_vector(3.0, 4.0, 0.0);
        let normalized = state.normalize().unwrap();

        let mag = normalized.magnitude();
        assert!((mag - 1.0).abs() < 1e-10);

        let (x, y, _) = normalized.as_vector();
        assert!((x - 0.6).abs() < 1e-10);
        assert!((y - 0.8).abs() < 1e-10);
    }
}
