// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! # Cliffy Core
//!
//! Reactive UI framework with geometric algebra state management.
//!
//! Cliffy uses Clifford/Geometric Algebra as its mathematical foundation,
//! providing FRP-style reactive primitives where all state transformations
//! are geometric operations under the hood.
//!
//! ## Quick Start
//!
//! ```rust
//! use cliffy_core::{Behavior, behavior};
//!
//! // Create a reactive behavior (internally a GA3 multivector)
//! let count = behavior(0i32);
//!
//! // Sample the current value
//! assert_eq!(count.sample(), 0);
//!
//! // Update via transformation
//! count.update(|n| n + 1);
//! assert_eq!(count.sample(), 1);
//!
//! // Map to derived behavior
//! let doubled = count.map(|n| n * 2);
//! assert_eq!(doubled.sample(), 2);
//! ```
//!
//! ## Explicit Geometric Control
//!
//! For applications that need explicit geometric transformations:
//!
//! ```rust
//! use cliffy_core::{GeometricState, Rotor, Translation};
//! use std::f64::consts::PI;
//!
//! // Create state representing a 3D position
//! let pos = GeometricState::from_vector(1.0, 0.0, 0.0);
//!
//! // Apply explicit geometric transformations
//! let rotated = pos.apply_rotor(&Rotor::xy(PI / 2.0));  // 90° rotation
//! let translated = rotated.apply_translation(&Translation::new(1.0, 0.0, 0.0));
//!
//! // Get the result as a vector
//! let (x, y, z) = translated.as_vector();
//! assert!((x - 1.0).abs() < 1e-10);  // ~1.0
//! assert!((y - 1.0).abs() < 1e-10);  // ~1.0
//! ```
//!
//! ## Architecture
//!
//! - **`Behavior<T>`**: Time-varying values backed by geometric algebra
//! - **`Event<T>`**: Discrete occurrences with geometric transformations
//! - **GeometricState**: Explicit geometric operations (rotations, translations)
//! - **Projection**: Extract user types from geometric state
//! - **Combinators**: `when`, `combine` for composition
//!
//! The geometric algebra foundation can be hidden from users (they work with
//! familiar types like `i32`, `String`, `Vec<T>`) or exposed explicitly via
//! `GeometricState` for advanced use cases.

pub mod behavior;
pub mod combinators;
pub mod component;
pub mod dataflow;
pub mod event;
pub mod geometric;
pub mod projection;
pub mod state;
pub mod transforms;

// Re-export main types - basic FRP
pub use behavior::{behavior, Behavior, Subscription};
pub use combinators::{combine, when};
pub use event::{event, Event};
pub use geometric::{FromGeometric, IntoGeometric, GA3};

// Re-export geometric state types
pub use projection::{
    BivectorProjection, BoolProjection, ColorAlphaProjection, ColorProjection, CustomProjection,
    IntProjection, MagnitudeProjection, MappedProjection, Position2DProjection,
    Position3DProjection, Projection, RotorAngleProjection, ScalarProjection,
    TypedBivectorProjection, TypedVectorProjection, VectorProjection,
};
pub use state::{GeometricState, GeometricSubscription};
pub use transforms::{Rotor, Transform, Translation, Versor};

// Re-export component types
pub use component::{
    component, compose, Component, ComposedComponent, Element, ElementKind, FnComponent, PropValue,
    Props, StateSplit,
};

// Re-export dataflow types
pub use dataflow::{
    CombinerType, DataflowGraph, GraphBuilder, Node, NodeId, NodeKind, ProjectionSpec,
    RotationPlane, SinkSpec, TransformType,
};

// Re-export Amari types for advanced users
pub use amari_core::Multivector;
