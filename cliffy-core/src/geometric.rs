// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Geometric algebra helpers
//!
//! This module provides the bridge between user-facing types (i32, f64, String, Vec<T>)
//! and the internal geometric algebra representation (GA3 multivectors).
//!
//! Users don't need to understand geometric algebra to use Cliffy - these conversions
//! happen automatically behind the scenes.

use amari_core::Multivector;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// 3D Euclidean geometric algebra Cl(3,0)
///
/// This is the default algebra used for UI state in Cliffy.
/// It has 8 basis elements: 1, e1, e2, e3, e12, e13, e23, e123
pub type GA3 = Multivector<3, 0, 0>;

/// Convert a value into its geometric representation
///
/// This trait is implemented for common types, allowing them to be
/// stored as GA3 multivectors internally while presenting a familiar
/// API to users.
pub trait IntoGeometric {
    /// Convert self into a GA3 multivector
    fn into_geometric(self) -> GA3;
}

/// Convert from a geometric representation back to a value
///
/// This is the inverse of `IntoGeometric`.
pub trait FromGeometric: Sized {
    /// Attempt to convert a GA3 multivector back to Self
    fn from_geometric(mv: &GA3) -> Self;
}

// ============================================================================
// Numeric types - stored as scalar (grade 0)
// ============================================================================

impl IntoGeometric for i32 {
    fn into_geometric(self) -> GA3 {
        GA3::scalar(self as f64)
    }
}

impl FromGeometric for i32 {
    fn from_geometric(mv: &GA3) -> Self {
        mv.get(0) as i32
    }
}

impl IntoGeometric for i64 {
    fn into_geometric(self) -> GA3 {
        GA3::scalar(self as f64)
    }
}

impl FromGeometric for i64 {
    fn from_geometric(mv: &GA3) -> Self {
        mv.get(0) as i64
    }
}

impl IntoGeometric for f32 {
    fn into_geometric(self) -> GA3 {
        GA3::scalar(self as f64)
    }
}

impl FromGeometric for f32 {
    fn from_geometric(mv: &GA3) -> Self {
        mv.get(0) as f32
    }
}

impl IntoGeometric for f64 {
    fn into_geometric(self) -> GA3 {
        GA3::scalar(self)
    }
}

impl FromGeometric for f64 {
    fn from_geometric(mv: &GA3) -> Self {
        mv.get(0)
    }
}

impl IntoGeometric for usize {
    fn into_geometric(self) -> GA3 {
        GA3::scalar(self as f64)
    }
}

impl FromGeometric for usize {
    fn from_geometric(mv: &GA3) -> Self {
        mv.get(0) as usize
    }
}

// ============================================================================
// Boolean - stored as scalar (0.0 = false, 1.0 = true)
// ============================================================================

impl IntoGeometric for bool {
    fn into_geometric(self) -> GA3 {
        GA3::scalar(if self { 1.0 } else { 0.0 })
    }
}

impl FromGeometric for bool {
    fn from_geometric(mv: &GA3) -> Self {
        mv.get(0) > 0.5
    }
}

// ============================================================================
// String - stored as hash in scalar + length in e1
// ============================================================================

impl IntoGeometric for String {
    fn into_geometric(self) -> GA3 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let hash = hasher.finish();

        // Store hash as scalar, length as e1 component
        let coeffs: Vec<f64> = (0..8)
            .map(|i| {
                if i == 0 {
                    (hash as f64) / (u64::MAX as f64)
                } else if i == 1 {
                    self.len() as f64
                } else {
                    0.0
                }
            })
            .collect();
        GA3::from_coefficients(coeffs)
    }
}

impl FromGeometric for String {
    fn from_geometric(mv: &GA3) -> Self {
        // We can't reconstruct the string from its hash
        // Return a placeholder - in practice, the original string is cached
        format!("[hash:{:.4}]", mv.get(0))
    }
}

// ============================================================================
// Option<T> - None is zero, Some(x) is x + 1.0 in e3 component
// ============================================================================

impl<T: IntoGeometric> IntoGeometric for Option<T> {
    fn into_geometric(self) -> GA3 {
        match self {
            None => GA3::zero(),
            Some(value) => {
                let inner = value.into_geometric();
                // Add 1.0 to e3 component (index 4) to mark as Some
                let mut coeffs: Vec<f64> = inner.as_slice().to_vec();
                if coeffs.len() > 4 {
                    coeffs[4] = 1.0;
                }
                GA3::from_coefficients(coeffs)
            }
        }
    }
}

impl<T: FromGeometric + Default> FromGeometric for Option<T> {
    fn from_geometric(mv: &GA3) -> Self {
        // Check e3 component (index 4) for Some marker
        if mv.get(4) > 0.5 {
            Some(T::from_geometric(mv))
        } else if mv.magnitude() < 1e-10 {
            None
        } else {
            Some(T::from_geometric(mv))
        }
    }
}

// ============================================================================
// Vec<T> - stored as length in scalar, hash in e1
// ============================================================================

impl<T: Hash> IntoGeometric for Vec<T> {
    fn into_geometric(self) -> GA3 {
        let len = self.len();
        let mut hasher = DefaultHasher::new();
        for item in &self {
            item.hash(&mut hasher);
        }
        let hash = hasher.finish();

        let coeffs: Vec<f64> = (0..8)
            .map(|i| {
                if i == 0 {
                    len as f64
                } else if i == 1 {
                    (hash as f64) / (u64::MAX as f64)
                } else {
                    0.0
                }
            })
            .collect();
        GA3::from_coefficients(coeffs)
    }
}

// Note: Vec<T> can't be reconstructed from geometric form without caching
// The cache is managed by the Behavior type

// ============================================================================
// Unit type
// ============================================================================

impl IntoGeometric for () {
    fn into_geometric(self) -> GA3 {
        GA3::zero()
    }
}

impl FromGeometric for () {
    fn from_geometric(_mv: &GA3) -> Self {}
}

// ============================================================================
// Tuples - stored by combining component geometric representations
// ============================================================================

impl<A: IntoGeometric, B: IntoGeometric> IntoGeometric for (A, B) {
    fn into_geometric(self) -> GA3 {
        // Combine both components into a single multivector
        // Use different grade components to avoid collision
        let a_mv = self.0.into_geometric();
        let b_mv = self.1.into_geometric();

        // Store A's scalar in grade 0, B's scalar in e1 component
        // This is a simplified encoding - works for scalar-like types
        let mut coeffs = vec![0.0; 8];
        coeffs[0] = a_mv.get(0); // A in scalar
        coeffs[1] = b_mv.get(0); // B in e1
                                 // Preserve some additional info in higher grades
        coeffs[2] = a_mv.get(1); // A's e1 in e2
        coeffs[3] = b_mv.get(1); // B's e1 in e3

        GA3::from_coefficients(coeffs)
    }
}

impl<A: FromGeometric, B: FromGeometric> FromGeometric for (A, B) {
    fn from_geometric(mv: &GA3) -> Self {
        // Reconstruct A from scalar + e2
        let a_coeffs: Vec<f64> = vec![mv.get(0), mv.get(2), 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let a_mv = GA3::from_coefficients(a_coeffs);

        // Reconstruct B from e1 + e3
        let b_coeffs: Vec<f64> = vec![mv.get(1), mv.get(3), 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let b_mv = GA3::from_coefficients(b_coeffs);

        (A::from_geometric(&a_mv), B::from_geometric(&b_mv))
    }
}

// ============================================================================
// GA3 itself (identity conversion)
// ============================================================================

impl IntoGeometric for GA3 {
    fn into_geometric(self) -> GA3 {
        self
    }
}

impl FromGeometric for GA3 {
    fn from_geometric(mv: &GA3) -> Self {
        mv.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i32_roundtrip() {
        let value = 42i32;
        let mv = value.into_geometric();
        let back = i32::from_geometric(&mv);
        assert_eq!(value, back);
    }

    #[test]
    fn test_f64_roundtrip() {
        let value = std::f64::consts::PI;
        let mv = value.into_geometric();
        let back = f64::from_geometric(&mv);
        assert!((value - back).abs() < 1e-10);
    }

    #[test]
    fn test_bool_roundtrip() {
        assert!(bool::from_geometric(&true.into_geometric()));
        assert!(!bool::from_geometric(&false.into_geometric()));
    }

    #[test]
    fn test_option_some() {
        let value = Some(42i32);
        let mv = value.into_geometric();
        let back: Option<i32> = Option::from_geometric(&mv);
        assert_eq!(back, Some(42));
    }

    #[test]
    fn test_option_none() {
        let value: Option<i32> = None;
        let mv = value.into_geometric();
        let back: Option<i32> = Option::from_geometric(&mv);
        assert_eq!(back, None);
    }
}
