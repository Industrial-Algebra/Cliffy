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
//! - [`GeometricCRDT`]: Operation-based CRDT with geometric transforms
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
//! ```rust
//! use cliffy_protocols::{GeometricCRDT, OperationType};
//! use cliffy_core::GA3;
//! use uuid::Uuid;
//!
//! let node_id = Uuid::new_v4();
//! let mut crdt = GeometricCRDT::new(node_id, GA3::scalar(0.0));
//!
//! // Apply a geometric transformation
//! let op = crdt.create_operation(GA3::scalar(5.0), OperationType::Addition);
//! crdt.apply_operation(op);
//! ```

use cliffy_core::GA3;

// Phase 2: Core CRDT and consensus
pub mod consensus;
pub mod crdt;
pub mod lattice;
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
pub use lattice::{ComponentLattice, GA3Lattice, GeometricLattice};
pub use storage::{GeometricStore, MemoryStore, Snapshot, StorageStats};
pub use sync::{
    PeerCapabilities, PeerConnectionState, PeerInfo, PeerState, SyncConfig, SyncMessage,
    SyncPayload, SyncState,
};
pub use vector_clock::*;

/// Type alias for the default multivector type used in protocols
pub type ProtocolMultivector = GA3;
