// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! SIMD-optimized CPU operations for geometric algebra.
//!
//! This module provides CPU-based SIMD acceleration for geometric algebra
//! operations when GPU is unavailable or for small batch sizes where GPU
//! overhead would be too high.
//!
//! Uses the `wide` crate for portable SIMD that works on x86, ARM, and WASM.

use wide::f32x8;

use crate::GpuMultivector;
use cliffy_core::GA3;

/// SIMD-optimized geometric product for Cl(3,0).
///
/// Computes the geometric product of two multivectors using SIMD
/// operations where possible. The Cl(3,0) geometric product follows
/// the multiplication table for basis elements {1, e1, e2, e12, e3, e13, e23, e123}.
#[must_use]
pub fn geometric_product_simd(a: &GpuMultivector, b: &GpuMultivector) -> GpuMultivector {
    // For Cl(3,0), the geometric product expands to:
    // result[i] = sum over j,k where basis[j] * basis[k] = ±basis[i]
    //
    // The sign table for Cl(3,0) is:
    // e1*e1 = 1, e2*e2 = 1, e3*e3 = 1
    // e1*e2 = e12, e2*e1 = -e12
    // e1*e3 = e13, e3*e1 = -e13
    // e2*e3 = e23, e3*e2 = -e23
    // etc.

    // Scalar component: 1*1 + e1*e1 + e2*e2 + e12*e12 + e3*e3 + e13*e13 + e23*e23 + e123*e123
    // Note: e12*e12 = e1*e2*e1*e2 = -e1*e1*e2*e2 = -1
    //       e123*e123 = e1*e2*e3*e1*e2*e3 = -1
    let c0 = a.coeffs[0] * b.coeffs[0]   // 1*1
           + a.coeffs[1] * b.coeffs[1]   // e1*e1 = 1
           + a.coeffs[2] * b.coeffs[2]   // e2*e2 = 1
           - a.coeffs[3] * b.coeffs[3]   // e12*e12 = -1
           + a.coeffs[4] * b.coeffs[4]   // e3*e3 = 1
           - a.coeffs[5] * b.coeffs[5]   // e13*e13 = -1
           - a.coeffs[6] * b.coeffs[6]   // e23*e23 = -1
           - a.coeffs[7] * b.coeffs[7]; // e123*e123 = -1

    // e1 component
    let c1 = a.coeffs[0] * b.coeffs[1]   // 1*e1
           + a.coeffs[1] * b.coeffs[0]   // e1*1
           - a.coeffs[2] * b.coeffs[3]   // e2*e12 = -e1
           + a.coeffs[3] * b.coeffs[2]   // e12*e2 = e1
           - a.coeffs[4] * b.coeffs[5]   // e3*e13 = -e1
           + a.coeffs[5] * b.coeffs[4]   // e13*e3 = e1
           + a.coeffs[6] * b.coeffs[7]   // e23*e123 = e1
           - a.coeffs[7] * b.coeffs[6]; // e123*e23 = -e1

    // e2 component
    let c2 = a.coeffs[0] * b.coeffs[2]   // 1*e2
           + a.coeffs[1] * b.coeffs[3]   // e1*e12 = e2
           + a.coeffs[2] * b.coeffs[0]   // e2*1
           - a.coeffs[3] * b.coeffs[1]   // e12*e1 = -e2
           - a.coeffs[4] * b.coeffs[6]   // e3*e23 = -e2
           - a.coeffs[5] * b.coeffs[7]   // e13*e123 = -e2
           + a.coeffs[6] * b.coeffs[4]   // e23*e3 = e2
           + a.coeffs[7] * b.coeffs[5]; // e123*e13 = e2

    // e12 component
    let c3 = a.coeffs[0] * b.coeffs[3]   // 1*e12
           + a.coeffs[1] * b.coeffs[2]   // e1*e2 = e12
           - a.coeffs[2] * b.coeffs[1]   // e2*e1 = -e12
           + a.coeffs[3] * b.coeffs[0]   // e12*1
           + a.coeffs[4] * b.coeffs[7]   // e3*e123 = e12
           + a.coeffs[5] * b.coeffs[6]   // e13*e23 = e12
           - a.coeffs[6] * b.coeffs[5]   // e23*e13 = -e12
           + a.coeffs[7] * b.coeffs[4]; // e123*e3 = e12

    // e3 component
    let c4 = a.coeffs[0] * b.coeffs[4]   // 1*e3
           + a.coeffs[1] * b.coeffs[5]   // e1*e13 = e3
           + a.coeffs[2] * b.coeffs[6]   // e2*e23 = e3
           + a.coeffs[3] * b.coeffs[7]   // e12*e123 = e3
           + a.coeffs[4] * b.coeffs[0]   // e3*1
           - a.coeffs[5] * b.coeffs[1]   // e13*e1 = -e3
           - a.coeffs[6] * b.coeffs[2]   // e23*e2 = -e3
           - a.coeffs[7] * b.coeffs[3]; // e123*e12 = -e3

    // e13 component
    let c5 = a.coeffs[0] * b.coeffs[5]   // 1*e13
           + a.coeffs[1] * b.coeffs[4]   // e1*e3 = e13
           - a.coeffs[2] * b.coeffs[7]   // e2*e123 = -e13
           - a.coeffs[3] * b.coeffs[6]   // e12*e23 = -e13
           - a.coeffs[4] * b.coeffs[1]   // e3*e1 = -e13
           + a.coeffs[5] * b.coeffs[0]   // e13*1
           + a.coeffs[6] * b.coeffs[3]   // e23*e12 = e13
           + a.coeffs[7] * b.coeffs[2]; // e123*e2 = e13

    // e23 component
    let c6 = a.coeffs[0] * b.coeffs[6]   // 1*e23
           + a.coeffs[1] * b.coeffs[7]   // e1*e123 = e23
           + a.coeffs[2] * b.coeffs[4]   // e2*e3 = e23
           + a.coeffs[3] * b.coeffs[5]   // e12*e13 = e23
           - a.coeffs[4] * b.coeffs[2]   // e3*e2 = -e23
           - a.coeffs[5] * b.coeffs[3]   // e13*e12 = -e23
           + a.coeffs[6] * b.coeffs[0]   // e23*1
           - a.coeffs[7] * b.coeffs[1]; // e123*e1 = -e23

    // e123 component
    let c7 = a.coeffs[0] * b.coeffs[7]   // 1*e123
           + a.coeffs[1] * b.coeffs[6]   // e1*e23 = e123
           - a.coeffs[2] * b.coeffs[5]   // e2*e13 = -e123
           + a.coeffs[3] * b.coeffs[4]   // e12*e3 = e123
           + a.coeffs[4] * b.coeffs[3]   // e3*e12 = e123
           - a.coeffs[5] * b.coeffs[2]   // e13*e2 = -e123
           + a.coeffs[6] * b.coeffs[1]   // e23*e1 = e123
           + a.coeffs[7] * b.coeffs[0]; // e123*1

    GpuMultivector {
        coeffs: [c0, c1, c2, c3, c4, c5, c6, c7],
    }
}

/// SIMD-optimized addition of two multivectors.
#[inline]
#[must_use]
pub fn addition_simd(a: &GpuMultivector, b: &GpuMultivector) -> GpuMultivector {
    let a_vec = f32x8::from(a.coeffs);
    let b_vec = f32x8::from(b.coeffs);
    let result = a_vec + b_vec;
    GpuMultivector {
        coeffs: result.to_array(),
    }
}

/// SIMD-optimized subtraction of two multivectors.
#[inline]
#[must_use]
pub fn subtraction_simd(a: &GpuMultivector, b: &GpuMultivector) -> GpuMultivector {
    let a_vec = f32x8::from(a.coeffs);
    let b_vec = f32x8::from(b.coeffs);
    let result = a_vec - b_vec;
    GpuMultivector {
        coeffs: result.to_array(),
    }
}

/// SIMD-optimized scalar multiplication.
#[inline]
#[must_use]
pub fn scalar_mul_simd(a: &GpuMultivector, scalar: f32) -> GpuMultivector {
    let a_vec = f32x8::from(a.coeffs);
    let s_vec = f32x8::splat(scalar);
    let result = a_vec * s_vec;
    GpuMultivector {
        coeffs: result.to_array(),
    }
}

/// SIMD-optimized reverse (reversion) of a multivector.
///
/// For Cl(3,0), the reverse negates bivector and pseudoscalar components:
/// ~(a + b*e12 + c*e13 + d*e23 + e*e123) = a - b*e12 - c*e13 - d*e23 - e*e123
#[inline]
#[must_use]
pub fn reverse_simd(a: &GpuMultivector) -> GpuMultivector {
    // Reverse: negate grades 2 and 3
    // Grade 0 (scalar): coeffs[0] - keep
    // Grade 1 (vectors): coeffs[1,2,4] - keep
    // Grade 2 (bivectors): coeffs[3,5,6] - negate
    // Grade 3 (pseudoscalar): coeffs[7] - negate
    let signs = f32x8::from([1.0, 1.0, 1.0, -1.0, 1.0, -1.0, -1.0, -1.0]);
    let a_vec = f32x8::from(a.coeffs);
    let result = a_vec * signs;
    GpuMultivector {
        coeffs: result.to_array(),
    }
}

/// SIMD-optimized grade involution (main involution).
///
/// Negates odd-grade components.
#[inline]
#[must_use]
pub fn grade_involution_simd(a: &GpuMultivector) -> GpuMultivector {
    // Grade involution: negate odd grades
    // Grade 0: keep, Grade 1: negate, Grade 2: keep, Grade 3: negate
    let signs = f32x8::from([1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0]);
    let a_vec = f32x8::from(a.coeffs);
    let result = a_vec * signs;
    GpuMultivector {
        coeffs: result.to_array(),
    }
}

/// SIMD-optimized conjugate (Clifford conjugate).
///
/// Combines reversion and grade involution.
#[inline]
#[must_use]
pub fn conjugate_simd(a: &GpuMultivector) -> GpuMultivector {
    // Conjugate = reverse ∘ grade_involution
    // Grade 0: keep, Grade 1: negate, Grade 2: negate, Grade 3: keep
    let signs = f32x8::from([1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, 1.0]);
    let a_vec = f32x8::from(a.coeffs);
    let result = a_vec * signs;
    GpuMultivector {
        coeffs: result.to_array(),
    }
}

/// SIMD-optimized squared magnitude (norm squared).
#[inline]
#[must_use]
pub fn norm_squared_simd(a: &GpuMultivector) -> f32 {
    // ||a||² = a * ~a (scalar part only)
    let rev = reverse_simd(a);
    let prod = geometric_product_simd(a, &rev);
    prod.coeffs[0]
}

/// SIMD-optimized norm (magnitude).
#[inline]
#[must_use]
pub fn norm_simd(a: &GpuMultivector) -> f32 {
    norm_squared_simd(a).sqrt()
}

/// SIMD-optimized normalization.
#[inline]
#[must_use]
pub fn normalize_simd(a: &GpuMultivector) -> GpuMultivector {
    let n = norm_simd(a);
    if n > 1e-10 {
        scalar_mul_simd(a, 1.0 / n)
    } else {
        GpuMultivector::zero()
    }
}

/// SIMD-optimized sandwich product: b * a * ~b
///
/// Used for rotations and reflections.
#[inline]
#[must_use]
pub fn sandwich_simd(a: &GpuMultivector, b: &GpuMultivector) -> GpuMultivector {
    let b_rev = reverse_simd(b);
    let temp = geometric_product_simd(b, a);
    geometric_product_simd(&temp, &b_rev)
}

/// SIMD-optimized dot product (inner product).
#[inline]
#[must_use]
pub fn dot_simd(a: &GpuMultivector, b: &GpuMultivector) -> f32 {
    let a_vec = f32x8::from(a.coeffs);
    let b_vec = f32x8::from(b.coeffs);
    let prod = a_vec * b_vec;
    // Sum all elements
    prod.reduce_add()
}

/// SIMD-optimized exponential for bivectors (creates rotors).
///
/// For a bivector B, exp(B) = cos(|B|) + sin(|B|) * B/|B|
#[must_use]
pub fn exp_bivector_simd(b: &GpuMultivector) -> GpuMultivector {
    // Extract bivector components
    let bx = b.coeffs[3]; // e12
    let by = b.coeffs[5]; // e13
    let bz = b.coeffs[6]; // e23

    // Compute magnitude of bivector
    let mag_sq = bx * bx + by * by + bz * bz;
    let mag = mag_sq.sqrt();

    if mag < 1e-10 {
        // For small bivector, exp(B) ≈ 1 + B
        let mut result = GpuMultivector::scalar(1.0);
        result.coeffs[3] = bx;
        result.coeffs[5] = by;
        result.coeffs[6] = bz;
        return result;
    }

    let cos_mag = mag.cos();
    let sin_mag_over_mag = mag.sin() / mag;

    GpuMultivector {
        coeffs: [
            cos_mag,
            0.0,
            0.0,
            bx * sin_mag_over_mag,
            0.0,
            by * sin_mag_over_mag,
            bz * sin_mag_over_mag,
            0.0,
        ],
    }
}

/// SIMD-optimized linear interpolation.
#[inline]
#[must_use]
pub fn lerp_simd(a: &GpuMultivector, b: &GpuMultivector, t: f32) -> GpuMultivector {
    let a_vec = f32x8::from(a.coeffs);
    let b_vec = f32x8::from(b.coeffs);
    let t_vec = f32x8::splat(t);
    let one_minus_t = f32x8::splat(1.0 - t);
    let result = a_vec * one_minus_t + b_vec * t_vec;
    GpuMultivector {
        coeffs: result.to_array(),
    }
}

/// SIMD-optimized rotor spherical linear interpolation (SLERP).
///
/// Interpolates smoothly between two rotors.
#[must_use]
pub fn rotor_slerp_simd(a: &GpuMultivector, b: &GpuMultivector, t: f32) -> GpuMultivector {
    // Compute dot product to find angle between rotors
    let dot = dot_simd(a, b);

    // Clamp dot product to valid range
    let dot = dot.clamp(-1.0, 1.0);

    // If rotors are very close, use linear interpolation
    if dot.abs() > 0.9995 {
        let result = lerp_simd(a, b, t);
        return normalize_simd(&result);
    }

    // Compute interpolation using geometric algebra
    // slerp(a, b, t) = a * exp(t * log(~a * b))
    let a_rev = reverse_simd(a);
    let ratio = geometric_product_simd(&a_rev, b);

    // Extract bivector part for log
    // For a rotor r = cos(θ) + sin(θ)B, log(r) = θB
    let scalar = ratio.coeffs[0];
    let theta = scalar.clamp(-1.0, 1.0).acos();

    if theta.abs() < 1e-10 {
        return *a;
    }

    // Scale the bivector by t*theta/sin(theta)
    let scale = t * theta / theta.sin();
    let mut log_ratio = GpuMultivector::zero();
    log_ratio.coeffs[3] = ratio.coeffs[3] * scale;
    log_ratio.coeffs[5] = ratio.coeffs[5] * scale;
    log_ratio.coeffs[6] = ratio.coeffs[6] * scale;

    let exp_log = exp_bivector_simd(&log_ratio);
    geometric_product_simd(a, &exp_log)
}

/// Batch processing context for SIMD operations.
///
/// Provides efficient batch processing of geometric algebra operations
/// using SIMD acceleration.
pub struct SimdBatch;

impl SimdBatch {
    /// Batch geometric product using SIMD.
    #[must_use]
    pub fn geometric_product(a: &[GpuMultivector], b: &[GpuMultivector]) -> Vec<GpuMultivector> {
        a.iter()
            .zip(b.iter())
            .map(|(a, b)| geometric_product_simd(a, b))
            .collect()
    }

    /// Batch addition using SIMD.
    #[must_use]
    pub fn addition(a: &[GpuMultivector], b: &[GpuMultivector]) -> Vec<GpuMultivector> {
        a.iter()
            .zip(b.iter())
            .map(|(a, b)| addition_simd(a, b))
            .collect()
    }

    /// Batch sandwich product using SIMD.
    #[must_use]
    pub fn sandwich(rotors: &[GpuMultivector], vectors: &[GpuMultivector]) -> Vec<GpuMultivector> {
        rotors
            .iter()
            .zip(vectors.iter())
            .map(|(r, v)| sandwich_simd(v, r))
            .collect()
    }

    /// Batch exponential using SIMD.
    pub fn exp(bivectors: &[GpuMultivector]) -> Vec<GpuMultivector> {
        bivectors.iter().map(exp_bivector_simd).collect()
    }

    /// Batch rotor SLERP using SIMD.
    #[must_use]
    pub fn rotor_slerp(a: &[GpuMultivector], b: &[GpuMultivector], t: f32) -> Vec<GpuMultivector> {
        a.iter()
            .zip(b.iter())
            .map(|(a, b)| rotor_slerp_simd(a, b, t))
            .collect()
    }

    /// Batch normalize using SIMD.
    pub fn normalize(mvs: &[GpuMultivector]) -> Vec<GpuMultivector> {
        mvs.iter().map(normalize_simd).collect()
    }

    /// Convert from GA3 to GPU format.
    #[must_use]
    pub fn from_ga3(mvs: &[GA3]) -> Vec<GpuMultivector> {
        mvs.iter().map(std::convert::Into::into).collect()
    }

    /// Convert from GPU format to GA3.
    #[must_use]
    pub fn to_ga3(mvs: &[GpuMultivector]) -> Vec<GA3> {
        mvs.iter().map(|mv| (*mv).into()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition_simd() {
        let a = GpuMultivector::vector(1.0, 2.0, 3.0);
        let b = GpuMultivector::vector(4.0, 5.0, 6.0);
        let result = addition_simd(&a, &b);
        assert_eq!(result.get_vector(), (5.0, 7.0, 9.0));
    }

    #[test]
    fn test_subtraction_simd() {
        let a = GpuMultivector::vector(5.0, 7.0, 9.0);
        let b = GpuMultivector::vector(1.0, 2.0, 3.0);
        let result = subtraction_simd(&a, &b);
        assert_eq!(result.get_vector(), (4.0, 5.0, 6.0));
    }

    #[test]
    fn test_scalar_mul_simd() {
        let a = GpuMultivector::vector(1.0, 2.0, 3.0);
        let result = scalar_mul_simd(&a, 2.0);
        assert_eq!(result.get_vector(), (2.0, 4.0, 6.0));
    }

    #[test]
    fn test_geometric_product_scalars() {
        let a = GpuMultivector::scalar(3.0);
        let b = GpuMultivector::scalar(4.0);
        let result = geometric_product_simd(&a, &b);
        assert!((result.get_scalar() - 12.0).abs() < 1e-6);
    }

    #[test]
    fn test_geometric_product_vectors() {
        // e1 * e1 = 1
        let e1 = GpuMultivector {
            coeffs: [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        };
        let result = geometric_product_simd(&e1, &e1);
        assert!((result.get_scalar() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_geometric_product_bivector() {
        // e1 * e2 = e12
        let e1 = GpuMultivector {
            coeffs: [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        };
        let e2 = GpuMultivector {
            coeffs: [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        };
        let result = geometric_product_simd(&e1, &e2);
        assert!((result.coeffs[3] - 1.0).abs() < 1e-6); // e12 component
    }

    #[test]
    fn test_reverse_simd() {
        let mut a = GpuMultivector::zero();
        a.coeffs[0] = 1.0; // scalar
        a.coeffs[1] = 2.0; // e1
        a.coeffs[3] = 3.0; // e12 (bivector)
        a.coeffs[7] = 4.0; // e123 (pseudoscalar)

        let rev = reverse_simd(&a);
        assert_eq!(rev.coeffs[0], 1.0); // scalar unchanged
        assert_eq!(rev.coeffs[1], 2.0); // vector unchanged
        assert_eq!(rev.coeffs[3], -3.0); // bivector negated
        assert_eq!(rev.coeffs[7], -4.0); // pseudoscalar negated
    }

    #[test]
    fn test_norm_simd() {
        let v = GpuMultivector::vector(3.0, 4.0, 0.0);
        let n = norm_simd(&v);
        assert!((n - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_simd() {
        let v = GpuMultivector::vector(3.0, 4.0, 0.0);
        let normalized = normalize_simd(&v);
        let (x, y, z) = normalized.get_vector();
        assert!((x - 0.6).abs() < 1e-6);
        assert!((y - 0.8).abs() < 1e-6);
        assert!(z.abs() < 1e-6);
    }

    #[test]
    fn test_lerp_simd() {
        let a = GpuMultivector::scalar(0.0);
        let b = GpuMultivector::scalar(10.0);
        let mid = lerp_simd(&a, &b, 0.5);
        assert!((mid.get_scalar() - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_exp_bivector_small() {
        // Small bivector: exp(B) ≈ 1 + B
        let mut b = GpuMultivector::zero();
        b.coeffs[3] = 0.001; // small e12 component
        let result = exp_bivector_simd(&b);
        assert!((result.coeffs[0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_sandwich_identity() {
        // Sandwich with identity rotor should preserve the vector
        let rotor = GpuMultivector::scalar(1.0);
        let v = GpuMultivector::vector(1.0, 2.0, 3.0);
        let result = sandwich_simd(&v, &rotor);
        let (x, y, z) = result.get_vector();
        assert!((x - 1.0).abs() < 1e-6);
        assert!((y - 2.0).abs() < 1e-6);
        assert!((z - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_batch_operations() {
        let a = vec![GpuMultivector::scalar(1.0), GpuMultivector::scalar(2.0)];
        let b = vec![GpuMultivector::scalar(3.0), GpuMultivector::scalar(4.0)];

        let results = SimdBatch::addition(&a, &b);
        assert!((results[0].get_scalar() - 4.0).abs() < 1e-6);
        assert!((results[1].get_scalar() - 6.0).abs() < 1e-6);
    }
}
