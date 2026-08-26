//! # Cliffy GPU
//!
//! WebGPU compute shaders and SIMD-optimized CPU operations for geometric algebra.
//!
//! This crate provides hardware-accelerated geometric algebra operations using:
//! - **WebGPU**: Massive parallel computation on GPU
//! - **SIMD**: CPU-optimized operations using portable SIMD intrinsics
//!
//! ## Overview
//!
//! Every browser becomes a compute node with cliffy-gpu:
//!
//! ```ignore
//! use cliffy_gpu::{GpuContext, GpuMultivector, AutoDispatcher};
//! use cliffy_core::GA3;
//!
//! // Initialize auto dispatcher (chooses GPU or SIMD-CPU based on batch size)
//! let dispatcher = AutoDispatcher::new().await;
//!
//! // Batch geometric products - automatically dispatched
//! let a_batch: Vec<GA3> = vec![...];
//! let b_batch: Vec<GA3> = vec![...];
//! let results = dispatcher.geometric_product(&a_batch, &b_batch).await?;
//! ```
//!
//! ## Features
//!
//! - **Batch Operations**: Process thousands of multivectors in parallel
//! - **Auto Dispatch**: Automatic CPU/GPU selection based on batch size
//! - **SIMD Fallback**: Optimized CPU operations when GPU unavailable
//! - **WASM Support**: Works in browsers with WebGPU
//! - **Compute Shaders**: WGSL shaders for geometric product, sandwich, exp, slerp

pub mod simd;

#[cfg(feature = "wasm")]
pub mod wasm;

use bytemuck::{Pod, Zeroable};
use std::borrow::Cow;
use std::sync::Arc;
use thiserror::Error;
use wgpu::util::DeviceExt;

use cliffy_core::GA3;
pub use simd::{addition_simd, geometric_product_simd, sandwich_simd, SimdBatch};

#[cfg(feature = "wasm")]
pub use wasm::*;

/// Errors that can occur during GPU operations.
#[derive(Error, Debug)]
pub enum GpuError {
    #[error("Failed to request GPU adapter")]
    AdapterNotFound,

    #[error("Failed to request GPU device: {0}")]
    DeviceRequestFailed(#[from] wgpu::RequestDeviceError),

    #[error("Buffer size mismatch: expected {expected}, got {actual}")]
    BufferSizeMismatch { expected: usize, actual: usize },

    #[error("GPU computation failed: {0}")]
    ComputeFailed(String),

    #[error("WebGPU not available")]
    WebGpuNotAvailable,
}

/// GPU-compatible multivector representation.
///
/// Uses 8 f32 coefficients for Cl(3,0) geometric algebra:
/// - coeffs[0]: scalar (1)
/// - coeffs[1]: e1
/// - coeffs[2]: e2
/// - coeffs[3]: e12
/// - coeffs[4]: e3
/// - coeffs[5]: e13
/// - coeffs[6]: e23
/// - coeffs[7]: e123 (pseudoscalar)
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable, Default)]
pub struct GpuMultivector {
    pub coeffs: [f32; 8],
}

impl GpuMultivector {
    /// Create a new GPU multivector with all zeros.
    pub fn zero() -> Self {
        Self { coeffs: [0.0; 8] }
    }

    /// Create a scalar multivector.
    pub fn scalar(s: f32) -> Self {
        let mut mv = Self::zero();
        mv.coeffs[0] = s;
        mv
    }

    /// Create a vector multivector (e1, e2, e3 components).
    pub fn vector(x: f32, y: f32, z: f32) -> Self {
        let mut mv = Self::zero();
        mv.coeffs[1] = x;
        mv.coeffs[2] = y;
        mv.coeffs[4] = z;
        mv
    }

    /// Get the scalar component.
    pub fn get_scalar(&self) -> f32 {
        self.coeffs[0]
    }

    /// Get the vector components (e1, e2, e3).
    pub fn get_vector(&self) -> (f32, f32, f32) {
        (self.coeffs[1], self.coeffs[2], self.coeffs[4])
    }
}

impl From<&GA3> for GpuMultivector {
    fn from(mv: &GA3) -> Self {
        let mut coeffs = [0.0f32; 8];
        // GA3 = Multivector<3,0,0> has 8 components
        // Map from amari-core's storage to our layout
        let slice = mv.as_slice();
        for (i, &c) in slice.iter().enumerate() {
            if i < 8 {
                coeffs[i] = c as f32;
            }
        }
        Self { coeffs }
    }
}

impl From<GpuMultivector> for GA3 {
    fn from(gpu_mv: GpuMultivector) -> Self {
        let coeffs: Vec<f64> = gpu_mv.coeffs.iter().map(|&c| c as f64).collect();
        GA3::from_slice(&coeffs)
    }
}

/// Threshold for automatic GPU dispatch.
/// Below this count, CPU is often faster due to GPU overhead.
pub const GPU_DISPATCH_THRESHOLD: usize = 256;

/// GPU compute context for geometric algebra operations.
///
/// Manages WebGPU device, queue, and compute pipelines for
/// parallel geometric algebra computation.
pub struct GpuContext {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    geometric_product_pipeline: wgpu::ComputePipeline,
    addition_pipeline: wgpu::ComputePipeline,
    sandwich_pipeline: wgpu::ComputePipeline,
    exp_pipeline: wgpu::ComputePipeline,
    rotor_slerp_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl GpuContext {
    /// Create a new GPU context.
    ///
    /// This initializes WebGPU and creates all compute pipelines.
    pub async fn new() -> Result<Self, GpuError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .map_err(|_| GpuError::AdapterNotFound)?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Cliffy GPU Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                experimental_features: wgpu::ExperimentalFeatures::default(),
                trace: wgpu::Trace::default(),
            })
            .await?;

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let shader_source = include_str!("../shaders/geometric.wgsl");
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Geometric Algebra Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_source)),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Geometric Compute Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Geometric Compute Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let geometric_product_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Geometric Product Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: Some("geometric_product_kernel"),
                compilation_options: Default::default(),
                cache: None,
            });

        let addition_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Addition Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("addition_kernel"),
            compilation_options: Default::default(),
            cache: None,
        });

        let sandwich_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Sandwich Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("sandwich_kernel"),
            compilation_options: Default::default(),
            cache: None,
        });

        let exp_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Exponential Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("exp_kernel"),
            compilation_options: Default::default(),
            cache: None,
        });

        let rotor_slerp_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Rotor Slerp Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: Some("rotor_slerp_kernel"),
                compilation_options: Default::default(),
                cache: None,
            });

        Ok(Self {
            device,
            queue,
            geometric_product_pipeline,
            addition_pipeline,
            sandwich_pipeline,
            exp_pipeline,
            rotor_slerp_pipeline,
            bind_group_layout,
        })
    }

    /// Batch geometric product: a[i] * b[i] for all i.
    ///
    /// Computes the geometric product of corresponding elements
    /// from two input arrays in parallel on the GPU.
    pub async fn batch_geometric_product(
        &self,
        a: &[GA3],
        b: &[GA3],
    ) -> Result<Vec<GA3>, GpuError> {
        if a.len() != b.len() {
            return Err(GpuError::BufferSizeMismatch {
                expected: a.len(),
                actual: b.len(),
            });
        }

        // Convert to GPU format
        let a_gpu: Vec<GpuMultivector> = a.iter().map(|mv| mv.into()).collect();
        let b_gpu: Vec<GpuMultivector> = b.iter().map(|mv| mv.into()).collect();

        let result = self.run_binary_kernel(&self.geometric_product_pipeline, &a_gpu, &b_gpu)?;

        Ok(result.into_iter().map(Into::into).collect())
    }

    /// Batch addition: a[i] + b[i] for all i.
    pub async fn batch_addition(&self, a: &[GA3], b: &[GA3]) -> Result<Vec<GA3>, GpuError> {
        if a.len() != b.len() {
            return Err(GpuError::BufferSizeMismatch {
                expected: a.len(),
                actual: b.len(),
            });
        }

        let a_gpu: Vec<GpuMultivector> = a.iter().map(|mv| mv.into()).collect();
        let b_gpu: Vec<GpuMultivector> = b.iter().map(|mv| mv.into()).collect();

        let result = self.run_binary_kernel(&self.addition_pipeline, &a_gpu, &b_gpu)?;

        Ok(result.into_iter().map(Into::into).collect())
    }

    /// Batch sandwich product: rotor[i] * x[i] * ~rotor[i] for all i.
    ///
    /// The sandwich product applies a rotation to each element.
    pub async fn batch_sandwich(
        &self,
        rotors: &[GA3],
        vectors: &[GA3],
    ) -> Result<Vec<GA3>, GpuError> {
        if rotors.len() != vectors.len() {
            return Err(GpuError::BufferSizeMismatch {
                expected: rotors.len(),
                actual: vectors.len(),
            });
        }

        let rotors_gpu: Vec<GpuMultivector> = rotors.iter().map(|mv| mv.into()).collect();
        let vectors_gpu: Vec<GpuMultivector> = vectors.iter().map(|mv| mv.into()).collect();

        let result = self.run_binary_kernel(&self.sandwich_pipeline, &rotors_gpu, &vectors_gpu)?;

        Ok(result.into_iter().map(Into::into).collect())
    }

    /// Batch exponential: exp(a[i]) for all i.
    ///
    /// The exponential map converts bivectors to rotors.
    pub async fn batch_exp(&self, a: &[GA3]) -> Result<Vec<GA3>, GpuError> {
        let a_gpu: Vec<GpuMultivector> = a.iter().map(|mv| mv.into()).collect();

        // For unary operations, use same input for both buffers
        let result = self.run_binary_kernel(&self.exp_pipeline, &a_gpu, &a_gpu)?;

        Ok(result.into_iter().map(Into::into).collect())
    }

    /// Batch rotor SLERP: interpolate from a[i] to b[i] by t.
    ///
    /// Spherical linear interpolation for smooth rotation blending.
    pub async fn batch_rotor_slerp(
        &self,
        a: &[GA3],
        b: &[GA3],
        t: f32,
    ) -> Result<Vec<GA3>, GpuError> {
        if a.len() != b.len() {
            return Err(GpuError::BufferSizeMismatch {
                expected: a.len(),
                actual: b.len(),
            });
        }

        let a_gpu: Vec<GpuMultivector> = a.iter().map(|mv| mv.into()).collect();
        // Encode t in the first coefficient of b
        let b_gpu: Vec<GpuMultivector> = b
            .iter()
            .map(|mv| {
                let mut gpu_mv: GpuMultivector = mv.into();
                gpu_mv.coeffs[0] = t;
                gpu_mv
            })
            .collect();

        let result = self.run_binary_kernel(&self.rotor_slerp_pipeline, &a_gpu, &b_gpu)?;

        Ok(result.into_iter().map(Into::into).collect())
    }

    /// Run a binary compute kernel (two input buffers, one output).
    fn run_binary_kernel(
        &self,
        pipeline: &wgpu::ComputePipeline,
        a: &[GpuMultivector],
        b: &[GpuMultivector],
    ) -> Result<Vec<GpuMultivector>, GpuError> {
        let count = a.len();
        if count == 0 {
            return Ok(Vec::new());
        }

        // Create input buffers
        let a_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Input A Buffer"),
                contents: bytemuck::cast_slice(a),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let b_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Input B Buffer"),
                contents: bytemuck::cast_slice(b),
                usage: wgpu::BufferUsages::STORAGE,
            });

        // Create output buffer
        let output_size = std::mem::size_of_val(a) as u64;
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: output_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Create staging buffer for reading results
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: output_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: a_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: b_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: output_buffer.as_entire_binding(),
                },
            ],
        });

        // Encode and submit
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);

            // Dispatch workgroups (64 threads per group)
            let workgroup_count = count.div_ceil(64) as u32;
            compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
        }

        // Copy output to staging buffer
        encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, output_size);

        self.queue.submit(std::iter::once(encoder.finish()));

        // Read results
        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });

        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .ok();

        receiver
            .recv()
            .map_err(|e| GpuError::ComputeFailed(e.to_string()))?
            .map_err(|e| GpuError::ComputeFailed(format!("{:?}", e)))?;

        let data = buffer_slice.get_mapped_range();
        let result: Vec<GpuMultivector> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        staging_buffer.unmap();

        Ok(result)
    }

    /// Check if GPU dispatch is recommended for the given batch size.
    pub fn should_use_gpu(&self, batch_size: usize) -> bool {
        batch_size >= GPU_DISPATCH_THRESHOLD
    }

    /// Get the device info for debugging.
    pub fn device_info(&self) -> String {
        "Cliffy GPU Context (wgpu)".to_string()
    }
}

/// Automatic dispatcher that chooses CPU (SIMD) or GPU based on batch size.
///
/// For small batches (< threshold), SIMD-optimized CPU operations are used.
/// For large batches (>= threshold), GPU compute shaders are used.
pub struct AutoDispatcher {
    gpu_ctx: Option<GpuContext>,
    threshold: usize,
}

impl AutoDispatcher {
    /// Create a new auto dispatcher, attempting to initialize GPU.
    pub async fn new() -> Self {
        let gpu_ctx = GpuContext::new().await.ok();
        Self {
            gpu_ctx,
            threshold: GPU_DISPATCH_THRESHOLD,
        }
    }

    /// Create with a custom threshold.
    pub async fn with_threshold(threshold: usize) -> Self {
        let gpu_ctx = GpuContext::new().await.ok();
        Self { gpu_ctx, threshold }
    }

    /// Create a CPU-only dispatcher (no GPU).
    pub fn cpu_only() -> Self {
        Self {
            gpu_ctx: None,
            threshold: GPU_DISPATCH_THRESHOLD,
        }
    }

    /// Check if GPU is available.
    pub fn has_gpu(&self) -> bool {
        self.gpu_ctx.is_some()
    }

    /// Get the current dispatch threshold.
    pub fn threshold(&self) -> usize {
        self.threshold
    }

    /// Batch geometric product with automatic dispatch.
    ///
    /// Uses GPU for large batches, SIMD-optimized CPU for small batches.
    pub async fn geometric_product(&self, a: &[GA3], b: &[GA3]) -> Result<Vec<GA3>, GpuError> {
        if let Some(ref ctx) = self.gpu_ctx {
            if a.len() >= self.threshold {
                return ctx.batch_geometric_product(a, b).await;
            }
        }

        // SIMD-optimized CPU fallback
        if a.len() != b.len() {
            return Err(GpuError::BufferSizeMismatch {
                expected: a.len(),
                actual: b.len(),
            });
        }

        let a_gpu = SimdBatch::from_ga3(a);
        let b_gpu = SimdBatch::from_ga3(b);
        let result = SimdBatch::geometric_product(&a_gpu, &b_gpu);
        Ok(SimdBatch::to_ga3(&result))
    }

    /// Batch addition with automatic dispatch.
    ///
    /// Uses GPU for large batches, SIMD-optimized CPU for small batches.
    pub async fn addition(&self, a: &[GA3], b: &[GA3]) -> Result<Vec<GA3>, GpuError> {
        if let Some(ref ctx) = self.gpu_ctx {
            if a.len() >= self.threshold {
                return ctx.batch_addition(a, b).await;
            }
        }

        // SIMD-optimized CPU fallback
        if a.len() != b.len() {
            return Err(GpuError::BufferSizeMismatch {
                expected: a.len(),
                actual: b.len(),
            });
        }

        let a_gpu = SimdBatch::from_ga3(a);
        let b_gpu = SimdBatch::from_ga3(b);
        let result = SimdBatch::addition(&a_gpu, &b_gpu);
        Ok(SimdBatch::to_ga3(&result))
    }

    /// Batch sandwich product with automatic dispatch.
    ///
    /// Uses GPU for large batches, SIMD-optimized CPU for small batches.
    pub async fn sandwich(&self, rotors: &[GA3], vectors: &[GA3]) -> Result<Vec<GA3>, GpuError> {
        if let Some(ref ctx) = self.gpu_ctx {
            if rotors.len() >= self.threshold {
                return ctx.batch_sandwich(rotors, vectors).await;
            }
        }

        // SIMD-optimized CPU fallback
        if rotors.len() != vectors.len() {
            return Err(GpuError::BufferSizeMismatch {
                expected: rotors.len(),
                actual: vectors.len(),
            });
        }

        let rotors_gpu = SimdBatch::from_ga3(rotors);
        let vectors_gpu = SimdBatch::from_ga3(vectors);
        let result = SimdBatch::sandwich(&rotors_gpu, &vectors_gpu);
        Ok(SimdBatch::to_ga3(&result))
    }

    /// Batch exponential with automatic dispatch.
    ///
    /// Uses GPU for large batches, SIMD-optimized CPU for small batches.
    pub async fn exp(&self, a: &[GA3]) -> Result<Vec<GA3>, GpuError> {
        if let Some(ref ctx) = self.gpu_ctx {
            if a.len() >= self.threshold {
                return ctx.batch_exp(a).await;
            }
        }

        // SIMD-optimized CPU fallback
        let a_gpu = SimdBatch::from_ga3(a);
        let result = SimdBatch::exp(&a_gpu);
        Ok(SimdBatch::to_ga3(&result))
    }

    /// Batch rotor SLERP with automatic dispatch.
    ///
    /// Uses GPU for large batches, SIMD-optimized CPU for small batches.
    pub async fn rotor_slerp(&self, a: &[GA3], b: &[GA3], t: f32) -> Result<Vec<GA3>, GpuError> {
        if let Some(ref ctx) = self.gpu_ctx {
            if a.len() >= self.threshold {
                return ctx.batch_rotor_slerp(a, b, t).await;
            }
        }

        // SIMD-optimized CPU fallback
        if a.len() != b.len() {
            return Err(GpuError::BufferSizeMismatch {
                expected: a.len(),
                actual: b.len(),
            });
        }

        let a_gpu = SimdBatch::from_ga3(a);
        let b_gpu = SimdBatch::from_ga3(b);
        let result = SimdBatch::rotor_slerp(&a_gpu, &b_gpu, t);
        Ok(SimdBatch::to_ga3(&result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_multivector_zero() {
        let mv = GpuMultivector::zero();
        assert!(mv.coeffs.iter().all(|&c| c == 0.0));
    }

    #[test]
    fn test_gpu_multivector_scalar() {
        let mv = GpuMultivector::scalar(5.0);
        assert_eq!(mv.get_scalar(), 5.0);
    }

    #[test]
    fn test_gpu_multivector_vector() {
        let mv = GpuMultivector::vector(1.0, 2.0, 3.0);
        assert_eq!(mv.get_vector(), (1.0, 2.0, 3.0));
    }

    #[test]
    fn test_ga3_roundtrip() {
        use amari_core::Vector;
        let vec = Vector::<3, 0, 0>::from_components(1.0, 2.0, 3.0);
        let original = GA3::from_vector(&vec);
        let gpu: GpuMultivector = (&original).into();
        let back: GA3 = gpu.into();

        // Check vector components are preserved (indices 1, 2, 4 for e1, e2, e3)
        let x = back.get(1);
        let y = back.get(2);
        let z = back.get(4);
        assert!((x - 1.0).abs() < 1e-5);
        assert!((y - 2.0).abs() < 1e-5);
        assert!((z - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_dispatch_threshold() {
        // Test AutoDispatcher CPU-only mode
        let dispatcher = AutoDispatcher::cpu_only();
        assert!(!dispatcher.has_gpu());
        assert_eq!(dispatcher.threshold(), GPU_DISPATCH_THRESHOLD);
    }
}
