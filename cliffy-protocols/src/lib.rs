// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Distributed consensus protocols using geometric algebra
//!
//! This crate provides CRDT, consensus, and synchronization implementations
//! using Clifford algebra for coordination-free distributed systems.
//!
//! # Key Components
//!
//! ## State Management
//! - [`GeometricCRDT`]: Operation-based CRDT with geometric transforms — **deprecated** (merge is unsound; see the [salvage plan](docs/plans/2026-08-26-geometric-crdt-salvage.md))
//! - [`GeometricLattice`]: Trait for lattice-based conflict resolution
//! - [`VectorClock`]: Causal ordering for distributed operations
//!
//! ## Consensus
//! - [`GeometricConsensus`]: Consensus protocol using geometric mean
//!
//! ## Synchronization (Phase 3)
//! - [`delta`]: State delta computation for efficient sync
//! - [`sync`]: P2P synchronization protocol
//! - [`storage`]: Persistence layer with snapshots and operation logs
//!
//! # Example
//!
//! The geometric CRDT below is deprecated (unsound merge — see the
//! [salvage plan](docs/plans/2026-08-26-geometric-crdt-salvage.md)); this
//! example shows the sound pieces that survive: vector clocks and the
//! componentwise lattice floor.
//!
//! ```rust
//! use cliffy_protocols::{ComponentLattice, GeometricLattice, VectorClock};
//! use uuid::Uuid;
//!
//! let node = Uuid::new_v4();
//! let mut clock = VectorClock::new();
//! clock.tick(node);
//!
//! // Componentwise max/min: a genuine join-semilattice
//! let a = ComponentLattice::from_scalar(5.0);
//! let b = ComponentLattice::from_scalar(3.0);
//! let joined = a.join(&b);
//! assert!((joined.as_multivector().scalar_part() - 5.0).abs() < 1e-10);
//! ```

use cliffy_core::GA3;

// Phase 2: Core CRDT and consensus
pub mod consensus;
pub mod crdt;
pub mod lattice;
pub mod observation;
pub mod serde_ga3;
pub mod vector_clock;

// Phase 3: Synchronization layer
pub mod delta;
pub mod storage;
pub mod sync;

// Re-exports
pub use consensus::*;
pub use crdt::*;
pub use delta::{
    apply_additive_delta, apply_delta, compute_delta, DeltaBatch, DeltaEncoding, StateDelta,
};
#[allow(deprecated)] // re-exports GA3Lattice for one more cycle; removed with Phase 1
pub use lattice::{ComponentLattice, GA3Lattice, GeometricLattice};
pub use observation::{
    GrantRef, Observation, ObservationKey, ObservationPayload, ObservationSet, RotorObservation,
    VectorObservation,
};
pub use storage::{GeometricStore, MemoryStore, Snapshot, StorageStats};
pub use sync::{
    PeerCapabilities, PeerConnectionState, PeerInfo, PeerState, SyncConfig, SyncMessage,
    SyncPayload, SyncState,
};
pub use vector_clock::*;

/// Type alias for the default multivector type used in protocols
pub type ProtocolMultivector = GA3;
