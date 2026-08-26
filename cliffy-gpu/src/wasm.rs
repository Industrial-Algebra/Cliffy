// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! WASM bindings for browser WebGPU operations.
//!
//! This module provides JavaScript-accessible bindings for the GPU
//! and SIMD-accelerated geometric algebra operations.
//!
//! ## Usage from JavaScript
//!
//! ```javascript
//! import init, { WasmGpuContext, WasmMultivector } from '@cliffy/gpu';
//!
//! await init();
//!
//! // Create GPU context
//! const ctx = await WasmGpuContext.new();
//!
//! // Create multivectors
//! const a = WasmMultivector.vector(1.0, 2.0, 3.0);
//! const b = WasmMultivector.vector(4.0, 5.0, 6.0);
//!
//! // Compute geometric product
//! const result = ctx.geometric_product(a, b);
//! ```

use js_sys::Float32Array;
use wasm_bindgen::prelude::*;

use crate::{
    simd::{
        addition_simd, exp_bivector_simd, geometric_product_simd, lerp_simd, norm_simd,
        normalize_simd, reverse_simd, rotor_slerp_simd, sandwich_simd, scalar_mul_simd,
        subtraction_simd, SimdBatch,
    },
    GpuMultivector,
};

/// JavaScript-accessible multivector wrapper.
#[wasm_bindgen]
pub struct WasmMultivector {
    inner: GpuMultivector,
}

#[wasm_bindgen]
impl WasmMultivector {
    /// Create a zero multivector.
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmMultivector {
        WasmMultivector {
            inner: GpuMultivector::zero(),
        }
    }

    /// Create a scalar multivector.
    #[wasm_bindgen]
    pub fn scalar(s: f32) -> WasmMultivector {
        WasmMultivector {
            inner: GpuMultivector::scalar(s),
        }
    }

    /// Create a vector multivector (e1, e2, e3 components).
    #[wasm_bindgen]
    pub fn vector(x: f32, y: f32, z: f32) -> WasmMultivector {
        WasmMultivector {
            inner: GpuMultivector::vector(x, y, z),
        }
    }

    /// Create from raw coefficients array.
    #[wasm_bindgen(js_name = "fromCoeffs")]
    pub fn from_coeffs(coeffs: &[f32]) -> WasmMultivector {
        let mut inner = GpuMultivector::zero();
        for (i, &c) in coeffs.iter().enumerate() {
            if i < 8 {
                inner.coeffs[i] = c;
            }
        }
        WasmMultivector { inner }
    }

    /// Get the scalar component.
    #[wasm_bindgen(js_name = "getScalar")]
    pub fn get_scalar(&self) -> f32 {
        self.inner.get_scalar()
    }

    /// Get vector components as array [x, y, z].
    #[wasm_bindgen(js_name = "getVector")]
    pub fn get_vector(&self) -> Float32Array {
        let (x, y, z) = self.inner.get_vector();
        Float32Array::from(&[x, y, z][..])
    }

    /// Get all coefficients as array.
    #[wasm_bindgen(js_name = "getCoeffs")]
    pub fn get_coeffs(&self) -> Float32Array {
        Float32Array::from(&self.inner.coeffs[..])
    }

    /// Add two multivectors.
    #[wasm_bindgen]
    pub fn add(&self, other: &WasmMultivector) -> WasmMultivector {
        WasmMultivector {
            inner: addition_simd(&self.inner, &other.inner),
        }
    }

    /// Subtract two multivectors.
    #[wasm_bindgen]
    pub fn sub(&self, other: &WasmMultivector) -> WasmMultivector {
        WasmMultivector {
            inner: subtraction_simd(&self.inner, &other.inner),
        }
    }

    /// Multiply by scalar.
    #[wasm_bindgen]
    pub fn scale(&self, s: f32) -> WasmMultivector {
        WasmMultivector {
            inner: scalar_mul_simd(&self.inner, s),
        }
    }

    /// Geometric product of two multivectors.
    #[wasm_bindgen(js_name = "geometricProduct")]
    pub fn geometric_product(&self, other: &WasmMultivector) -> WasmMultivector {
        WasmMultivector {
            inner: geometric_product_simd(&self.inner, &other.inner),
        }
    }

    /// Reverse (reversion) of multivector.
    #[wasm_bindgen]
    pub fn reverse(&self) -> WasmMultivector {
        WasmMultivector {
            inner: reverse_simd(&self.inner),
        }
    }

    /// Normalize the multivector.
    #[wasm_bindgen]
    pub fn normalize(&self) -> WasmMultivector {
        WasmMultivector {
            inner: normalize_simd(&self.inner),
        }
    }

    /// Get the norm (magnitude) of the multivector.
    #[wasm_bindgen]
    pub fn norm(&self) -> f32 {
        norm_simd(&self.inner)
    }

    /// Sandwich product: self * other * ~self
    #[wasm_bindgen]
    pub fn sandwich(&self, other: &WasmMultivector) -> WasmMultivector {
        WasmMultivector {
            inner: sandwich_simd(&other.inner, &self.inner),
        }
    }

    /// Exponential (for bivectors, creates rotors).
    #[wasm_bindgen]
    pub fn exp(&self) -> WasmMultivector {
        WasmMultivector {
            inner: exp_bivector_simd(&self.inner),
        }
    }

    /// Linear interpolation.
    #[wasm_bindgen]
    pub fn lerp(&self, other: &WasmMultivector, t: f32) -> WasmMultivector {
        WasmMultivector {
            inner: lerp_simd(&self.inner, &other.inner, t),
        }
    }

    /// Spherical linear interpolation (for rotors).
    #[wasm_bindgen]
    pub fn slerp(&self, other: &WasmMultivector, t: f32) -> WasmMultivector {
        WasmMultivector {
            inner: rotor_slerp_simd(&self.inner, &other.inner, t),
        }
    }

    /// Clone the multivector.
    #[wasm_bindgen(js_name = "clone")]
    pub fn clone_mv(&self) -> WasmMultivector {
        WasmMultivector { inner: self.inner }
    }
}

impl Default for WasmMultivector {
    fn default() -> Self {
        Self::new()
    }
}

/// JavaScript-accessible batch operations using SIMD.
///
/// Uses Float32Array for efficient bulk data transfer.
/// Each multivector is 8 consecutive f32 values.
#[wasm_bindgen]
pub struct WasmBatch;

/// Helper to convert Float32Array to Vec<GpuMultivector>.
fn float32_to_mvs(data: &Float32Array) -> Vec<GpuMultivector> {
    let vec: Vec<f32> = data.to_vec();
    // A trailing partial multivector (len % 8 != 0) is dropped — same
    // semantics as the previous `chunks_exact` iterator.
    vec.as_chunks::<8>()
        .0
        .iter()
        .map(|coeffs| GpuMultivector { coeffs: *coeffs })
        .collect()
}

/// Helper to convert Vec<GpuMultivector> to Float32Array.
fn mvs_to_float32(mvs: &[GpuMultivector]) -> Float32Array {
    let flat: Vec<f32> = mvs.iter().flat_map(|mv| mv.coeffs).collect();
    Float32Array::from(&flat[..])
}

#[wasm_bindgen]
impl WasmBatch {
    /// Batch geometric product.
    ///
    /// Input: Two Float32Arrays where each 8 consecutive values form a multivector.
    /// Output: Float32Array with results (8 values per multivector).
    #[wasm_bindgen(js_name = "geometricProduct")]
    pub fn geometric_product(a: &Float32Array, b: &Float32Array) -> Float32Array {
        let a_mvs = float32_to_mvs(a);
        let b_mvs = float32_to_mvs(b);
        let results = SimdBatch::geometric_product(&a_mvs, &b_mvs);
        mvs_to_float32(&results)
    }

    /// Batch addition.
    #[wasm_bindgen]
    pub fn addition(a: &Float32Array, b: &Float32Array) -> Float32Array {
        let a_mvs = float32_to_mvs(a);
        let b_mvs = float32_to_mvs(b);
        let results = SimdBatch::addition(&a_mvs, &b_mvs);
        mvs_to_float32(&results)
    }

    /// Batch sandwich product.
    #[wasm_bindgen]
    pub fn sandwich(rotors: &Float32Array, vectors: &Float32Array) -> Float32Array {
        let rotor_mvs = float32_to_mvs(rotors);
        let vector_mvs = float32_to_mvs(vectors);
        let results = SimdBatch::sandwich(&rotor_mvs, &vector_mvs);
        mvs_to_float32(&results)
    }

    /// Batch exponential.
    #[wasm_bindgen]
    pub fn exp(bivectors: &Float32Array) -> Float32Array {
        let mvs = float32_to_mvs(bivectors);
        let results = SimdBatch::exp(&mvs);
        mvs_to_float32(&results)
    }

    /// Batch normalize.
    #[wasm_bindgen]
    pub fn normalize(mvs: &Float32Array) -> Float32Array {
        let mv_vec = float32_to_mvs(mvs);
        let results = SimdBatch::normalize(&mv_vec);
        mvs_to_float32(&results)
    }

    /// Batch rotor SLERP.
    #[wasm_bindgen(js_name = "rotorSlerp")]
    pub fn rotor_slerp(a: &Float32Array, b: &Float32Array, t: f32) -> Float32Array {
        let a_mvs = float32_to_mvs(a);
        let b_mvs = float32_to_mvs(b);
        let results = SimdBatch::rotor_slerp(&a_mvs, &b_mvs, t);
        mvs_to_float32(&results)
    }
}

/// Create a rotor for rotation in the e1-e2 plane (XY rotation).
#[wasm_bindgen(js_name = "createRotorXY")]
pub fn create_rotor_xy(angle: f32) -> WasmMultivector {
    // Rotor = cos(θ/2) + sin(θ/2) * e12
    let half = angle / 2.0;
    let mut inner = GpuMultivector::zero();
    inner.coeffs[0] = half.cos();
    inner.coeffs[3] = half.sin(); // e12 component
    WasmMultivector { inner }
}

/// Create a rotor for rotation in the e1-e3 plane (XZ rotation).
#[wasm_bindgen(js_name = "createRotorXZ")]
pub fn create_rotor_xz(angle: f32) -> WasmMultivector {
    let half = angle / 2.0;
    let mut inner = GpuMultivector::zero();
    inner.coeffs[0] = half.cos();
    inner.coeffs[5] = half.sin(); // e13 component
    WasmMultivector { inner }
}

/// Create a rotor for rotation in the e2-e3 plane (YZ rotation).
#[wasm_bindgen(js_name = "createRotorYZ")]
pub fn create_rotor_yz(angle: f32) -> WasmMultivector {
    let half = angle / 2.0;
    let mut inner = GpuMultivector::zero();
    inner.coeffs[0] = half.cos();
    inner.coeffs[6] = half.sin(); // e23 component
    WasmMultivector { inner }
}

/// Create a rotor from axis-angle representation.
#[wasm_bindgen(js_name = "createRotorAxisAngle")]
pub fn create_rotor_axis_angle(ax: f32, ay: f32, az: f32, angle: f32) -> WasmMultivector {
    // Normalize axis
    let len = (ax * ax + ay * ay + az * az).sqrt();
    if len < 1e-10 {
        return WasmMultivector::scalar(1.0);
    }

    let ax = ax / len;
    let ay = ay / len;
    let az = az / len;

    // Bivector for this axis: B = ax*e23 + ay*e13 + az*e12
    // (Note: cross product of axis with basis vectors gives bivector)
    let half = angle / 2.0;
    let s = half.sin();

    let mut inner = GpuMultivector::zero();
    inner.coeffs[0] = half.cos();
    inner.coeffs[3] = az * s; // e12
    inner.coeffs[5] = -ay * s; // e13 (note sign convention)
    inner.coeffs[6] = ax * s; // e23

    WasmMultivector { inner }
}

/// Get the dispatch threshold for GPU vs CPU selection.
#[wasm_bindgen(js_name = "getDispatchThreshold")]
pub fn get_dispatch_threshold() -> usize {
    crate::GPU_DISPATCH_THRESHOLD
}
