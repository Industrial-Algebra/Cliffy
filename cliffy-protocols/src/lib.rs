// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Distributed consensus protocols using geometric algebra
//!
//! This crate provides CRDT, consensus, and synchronization implementations
//! using Clifford algebra for coordination-free distributed systems.
//!
//! # The Phase 1 architecture (salvage)
//!
//! The CRDT **is** an [`ObservationSet`] — a grow-only set of attributed
//! observations whose merge is plain union (a true join-semilattice:
//! convergence by construction). The geometry is a **deterministic
//! projection** over the set ([`scalar_mean`], [`vector_mean`],
//! [`rotor_consensus`] — the Markley eigen-mean): equal sets ⇒ bit-identical
//! consensus on every replica. *The merge is boring; the render is
//! geometric.* See the [salvage plan](docs/plans/2026-08-26-geometric-crdt-salvage.md).
//!
//! ## State Management (Phase 1)
//! - [`ObservationSet`]: G-Set CRDT — merge is set union
//! - [`Observation`]: Attributed observation, key = `(participant_id, seq)`
//! - [`rotor_consensus`]: Markley chordal-L₂ eigen-mean for orientations
//! - [`scalar_mean`] / [`vector_mean`]: componentwise floor
//! - [`VectorClock`]: Causal ordering for distributed operations
//!
//! ## Lattice floor
//! - [`ComponentLattice`]: componentwise join-semilattice — the sound floor
//!   for per-grade state (via [`GeometricLattice`])
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
//! use cliffy_protocols::{scalar_mean, Observation, ObservationSet};
//! use uuid::Uuid;
//!
//! // Two replicas, concurrent observations, no coordination.
//! let (a, b) = (Uuid::new_v4(), Uuid::new_v4());
//! let mut left = ObservationSet::new();
//! left.insert(Observation::new_scalar(a, 0, 5.0));
//! let mut right = ObservationSet::new();
//! right.insert(Observation::new_scalar(b, 0, 10.0));
//!
//! // Union merge — both observations survive; consensus is the exact mean.
//! left.merge(&right);
//! right.merge(&left);
//! assert_eq!(left, right);
//! assert_eq!(scalar_mean(&left), Some(7.5));
//! ```
//!
//! The sound floor (componentwise lattice), pre-dating Phase 1:
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

// Phase 1: the sound CRDT floor
pub mod eigen;
pub mod lattice;
pub mod observation;
pub mod projection;
pub mod serde_ga3;
pub mod vector_clock;

// Phase 3: Synchronization layer
pub mod delta;
pub mod storage;
pub mod sync;

// Re-exports
pub use delta::{
    apply_additive_delta, apply_delta, compute_delta, DeltaBatch, DeltaEncoding, StateDelta,
};
pub use lattice::{ComponentLattice, GeometricLattice};
pub use observation::{
    GrantRef, Observation, ObservationKey, ObservationPayload, ObservationSet, RotorObservation,
    VectorObservation,
};
pub use projection::{
    from_amari_rotor, rotor_consensus, rotor_consensus_with_weights, scalar_mean, vector_mean,
};
pub use storage::{GeometricStore, MemoryStore, Snapshot, StorageStats};
pub use sync::{
    PeerCapabilities, PeerConnectionState, PeerInfo, PeerState, SyncConfig, SyncMessage,
    SyncPayload, SyncState,
};
pub use vector_clock::*;

/// Type alias for the default multivector type used in protocols
pub type ProtocolMultivector = GA3;
