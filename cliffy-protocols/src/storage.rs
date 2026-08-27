// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Persistence layer for geometric state and operation history
//!
//! This module provides traits and implementations for persisting
//! geometric state with operation history, enabling recovery and
//! offline support.
//!
//! # Storage Strategy
//!
//! The storage layer uses a hybrid approach:
//! - **Snapshots**: Full state at specific points in time
//! - **Operations**: Log of operations since last snapshot
//!
//! Recovery: Load latest snapshot, then replay operations.
//!
//! # Example
//!
//! ```rust
//! use cliffy_protocols::storage::{GeometricStore, MemoryStore, Snapshot};
//! use cliffy_protocols::VectorClock;
//! use cliffy_core::GA3;
//!
//! // Create in-memory store
//! let mut store = MemoryStore::new();
//!
//! // Save a snapshot
//! let state = GA3::scalar(42.0);
//! let clock = VectorClock::new();
//! store.save_snapshot(&state, &clock);
//!
//! // Load it back
//! let snapshot = store.load_latest_snapshot();
//! assert!(snapshot.is_some());
//! ```

use crate::delta::StateDelta;
use crate::serde_ga3;
use crate::VectorClock;
use cliffy_core::GA3;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// A snapshot of the geometric state at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// The state at this snapshot
    #[serde(with = "serde_ga3")]
    pub state: GA3,
    /// Vector clock at this snapshot
    pub clock: VectorClock,
    /// Snapshot ID (monotonically increasing)
    pub id: u64,
    /// Timestamp when snapshot was taken (ms since epoch)
    pub timestamp: u64,
}

/// A stored operation for replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredOperation {
    /// The operation (as a delta)
    pub delta: StateDelta,
    /// Sequence number (for ordering)
    pub sequence: u64,
    /// Whether this operation has been compacted
    pub compacted: bool,
}

/// Trait for geometric state persistence.
///
/// Implementations may target different backends:
/// - `MemoryStore`: In-memory (for testing)
/// - `IndexedDbStore`: Browser `IndexedDB` (for WASM)
/// - `FileStore`: File system (for native)
pub trait GeometricStore {
    /// Save a state snapshot.
    fn save_snapshot(&mut self, state: &GA3, clock: &VectorClock);

    /// Load the latest snapshot.
    fn load_latest_snapshot(&self) -> Option<Snapshot>;

    /// Load a specific snapshot by ID.
    fn load_snapshot(&self, id: u64) -> Option<Snapshot>;

    /// Append an operation to the log.
    fn append_operation(&mut self, delta: StateDelta);

    /// Get all operations since a given clock.
    fn operations_since(&self, clock: &VectorClock) -> Vec<StoredOperation>;

    /// Get all operations since a given sequence number.
    fn operations_since_sequence(&self, sequence: u64) -> Vec<StoredOperation>;

    /// Compact the operation log by creating a new snapshot.
    ///
    /// Operations before the snapshot can be discarded.
    fn compact(&mut self) -> Option<Snapshot>;

    /// Get storage statistics.
    fn stats(&self) -> StorageStats;

    /// Clear all stored data.
    fn clear(&mut self);
}

/// Statistics about stored data.
#[derive(Debug, Clone, Default)]
pub struct StorageStats {
    /// Number of snapshots stored
    pub snapshot_count: usize,
    /// Number of operations in the log
    pub operation_count: usize,
    /// Approximate total size in bytes
    pub total_size_bytes: usize,
    /// Operations since last snapshot
    pub pending_operations: usize,
}

/// In-memory implementation of `GeometricStore`.
///
/// Useful for testing and as a reference implementation.
#[derive(Debug, Default)]
pub struct MemoryStore {
    snapshots: Vec<Snapshot>,
    operations: VecDeque<StoredOperation>,
    next_snapshot_id: u64,
    next_sequence: u64,
    /// Current state (for compaction)
    current_state: Option<GA3>,
    /// Current clock
    current_clock: VectorClock,
    /// Configuration
    config: MemoryStoreConfig,
}

/// Configuration for `MemoryStore`.
#[derive(Debug, Clone)]
pub struct MemoryStoreConfig {
    /// Maximum number of snapshots to keep
    pub max_snapshots: usize,
    /// Maximum operations before auto-compact
    pub max_operations_before_compact: usize,
    /// Whether to auto-compact
    pub auto_compact: bool,
}

impl Default for MemoryStoreConfig {
    fn default() -> Self {
        Self {
            max_snapshots: 10,
            max_operations_before_compact: 1000,
            auto_compact: true,
        }
    }
}

impl MemoryStore {
    /// Create a new in-memory store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom configuration.
    #[must_use]
    pub fn with_config(config: MemoryStoreConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Get the current state by replaying from snapshot.
    #[must_use]
    pub fn get_current_state(&self) -> Option<GA3> {
        let snapshot = self.load_latest_snapshot()?;
        let mut state = snapshot.state;

        for op in self.operations_since(&snapshot.clock) {
            crate::delta::apply_delta(&mut state, &op.delta);
        }

        Some(state)
    }

    /// Prune old snapshots beyond `max_snapshots`.
    fn prune_snapshots(&mut self) {
        while self.snapshots.len() > self.config.max_snapshots {
            self.snapshots.remove(0);
        }
    }

    /// Check if compaction is needed.
    fn should_compact(&self) -> bool {
        self.config.auto_compact
            && self.operations.len() >= self.config.max_operations_before_compact
    }
}

impl GeometricStore for MemoryStore {
    fn save_snapshot(&mut self, state: &GA3, clock: &VectorClock) {
        let snapshot = Snapshot {
            state: state.clone(),
            clock: clock.clone(),
            id: self.next_snapshot_id,
            timestamp: current_timestamp_ms(),
        };
        self.next_snapshot_id += 1;
        self.snapshots.push(snapshot);
        self.current_state = Some(state.clone());
        self.current_clock = clock.clone();
        self.prune_snapshots();
    }

    fn load_latest_snapshot(&self) -> Option<Snapshot> {
        self.snapshots.last().cloned()
    }

    fn load_snapshot(&self, id: u64) -> Option<Snapshot> {
        self.snapshots.iter().find(|s| s.id == id).cloned()
    }

    fn append_operation(&mut self, delta: StateDelta) {
        // Update current state
        if let Some(ref mut state) = self.current_state {
            crate::delta::apply_delta(state, &delta);
        }
        self.current_clock.update(&delta.to_clock);

        let op = StoredOperation {
            delta,
            sequence: self.next_sequence,
            compacted: false,
        };
        self.next_sequence += 1;
        self.operations.push_back(op);

        // Auto-compact if needed
        if self.should_compact() {
            self.compact();
        }
    }

    fn operations_since(&self, clock: &VectorClock) -> Vec<StoredOperation> {
        self.operations
            .iter()
            .filter(|op| clock.happens_before(&op.delta.to_clock))
            .cloned()
            .collect()
    }

    fn operations_since_sequence(&self, sequence: u64) -> Vec<StoredOperation> {
        self.operations
            .iter()
            .filter(|op| op.sequence >= sequence)
            .cloned()
            .collect()
    }

    fn compact(&mut self) -> Option<Snapshot> {
        let state = self.current_state.clone()?;

        // Create new snapshot
        self.save_snapshot(&state, &self.current_clock.clone());

        // Clear operations (they're now in the snapshot)
        self.operations.clear();

        self.load_latest_snapshot()
    }

    fn stats(&self) -> StorageStats {
        let pending = if let Some(snapshot) = self.snapshots.last() {
            self.operations_since(&snapshot.clock).len()
        } else {
            self.operations.len()
        };

        StorageStats {
            snapshot_count: self.snapshots.len(),
            operation_count: self.operations.len(),
            total_size_bytes: self.estimate_size(),
            pending_operations: pending,
        }
    }

    fn clear(&mut self) {
        self.snapshots.clear();
        self.operations.clear();
        self.current_state = None;
        self.current_clock = VectorClock::new();
        self.next_snapshot_id = 0;
        self.next_sequence = 0;
    }
}

impl MemoryStore {
    fn estimate_size(&self) -> usize {
        // Rough estimate: 64 bytes per snapshot, 96 bytes per operation
        self.snapshots.len() * 64 + self.operations.len() * 96
    }
}

/// Get current timestamp in milliseconds since epoch.
fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

/// A recovery result from loading stored state.
#[derive(Debug)]
pub struct RecoveryResult {
    /// The recovered state
    pub state: GA3,
    /// The recovered clock
    pub clock: VectorClock,
    /// Number of operations replayed
    pub operations_replayed: usize,
    /// The snapshot ID used as base
    pub base_snapshot_id: Option<u64>,
}

/// Recover state from a store.
///
/// Loads the latest snapshot and replays any subsequent operations.
pub fn recover_state(store: &impl GeometricStore) -> Option<RecoveryResult> {
    let snapshot = store.load_latest_snapshot()?;
    let mut state = snapshot.state.clone();
    let mut clock = snapshot.clock.clone();

    let ops = store.operations_since(&snapshot.clock);
    let ops_count = ops.len();

    for op in ops {
        crate::delta::apply_delta(&mut state, &op.delta);
        clock.update(&op.delta.to_clock);
    }

    Some(RecoveryResult {
        state,
        clock,
        operations_replayed: ops_count,
        base_snapshot_id: Some(snapshot.id),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_memory_store_snapshot() {
        let mut store = MemoryStore::new();

        let state = GA3::scalar(42.0);
        let clock = VectorClock::new();

        store.save_snapshot(&state, &clock);

        let loaded = store.load_latest_snapshot().unwrap();
        assert!((loaded.state.scalar_part() - 42.0).abs() < 1e-10);
    }

    #[test]
    fn test_memory_store_operations() {
        let mut store = MemoryStore::new();
        let node_id = Uuid::new_v4();

        // Save initial snapshot
        let state = GA3::scalar(10.0);
        let mut clock = VectorClock::new();
        store.save_snapshot(&state, &clock);

        // Append operations
        clock.tick(node_id);
        let delta =
            StateDelta::additive(GA3::scalar(5.0), VectorClock::new(), clock.clone(), node_id);
        store.append_operation(delta);

        // Check operations
        let ops = store.operations_since(&VectorClock::new());
        assert_eq!(ops.len(), 1);
    }

    #[test]
    fn test_recovery() {
        let mut store = MemoryStore::new();
        let node_id = Uuid::new_v4();

        // Save initial snapshot
        let state = GA3::scalar(10.0);
        let mut clock = VectorClock::new();
        store.save_snapshot(&state, &clock);

        // Append some operations
        clock.tick(node_id);
        store.append_operation(StateDelta::additive(
            GA3::scalar(5.0),
            VectorClock::new(),
            clock.clone(),
            node_id,
        ));

        clock.tick(node_id);
        store.append_operation(StateDelta::additive(
            GA3::scalar(3.0),
            VectorClock::new(),
            clock.clone(),
            node_id,
        ));

        // Recover
        let result = recover_state(&store).unwrap();

        // Should be 10 + 5 + 3 = 18
        assert!((result.state.scalar_part() - 18.0).abs() < 1e-10);
        assert_eq!(result.operations_replayed, 2);
    }

    #[test]
    fn test_compaction() {
        let config = MemoryStoreConfig {
            max_snapshots: 5,
            max_operations_before_compact: 3,
            auto_compact: true,
        };
        let mut store = MemoryStore::with_config(config);
        let node_id = Uuid::new_v4();

        // Save initial state
        let state = GA3::scalar(0.0);
        let mut clock = VectorClock::new();
        store.save_snapshot(&state, &clock);

        // Add operations until auto-compact triggers
        for i in 1..=5 {
            clock.tick(node_id);
            store.append_operation(StateDelta::additive(
                GA3::scalar(f64::from(i)),
                VectorClock::new(),
                clock.clone(),
                node_id,
            ));
        }

        // After auto-compact, operations should be cleared
        // and a new snapshot created
        assert!(store.operations.len() < 5);
        assert!(store.snapshots.len() >= 2);
    }

    #[test]
    fn test_stats() {
        let mut store = MemoryStore::new();
        let node_id = Uuid::new_v4();

        let state = GA3::scalar(0.0);
        let mut clock = VectorClock::new();
        store.save_snapshot(&state, &clock);

        clock.tick(node_id);
        store.append_operation(StateDelta::additive(
            GA3::scalar(1.0),
            VectorClock::new(),
            clock.clone(),
            node_id,
        ));

        let stats = store.stats();
        assert_eq!(stats.snapshot_count, 1);
        assert_eq!(stats.operation_count, 1);
    }

    #[test]
    fn test_clear() {
        let mut store = MemoryStore::new();

        store.save_snapshot(&GA3::scalar(1.0), &VectorClock::new());
        assert!(store.load_latest_snapshot().is_some());

        store.clear();
        assert!(store.load_latest_snapshot().is_none());
    }

    #[test]
    fn test_snapshot_pruning() {
        let config = MemoryStoreConfig {
            max_snapshots: 3,
            max_operations_before_compact: 1000,
            auto_compact: false,
        };
        let mut store = MemoryStore::with_config(config);

        // Save more snapshots than max
        for i in 0..5 {
            store.save_snapshot(&GA3::scalar(f64::from(i)), &VectorClock::new());
        }

        // Should only keep last 3
        assert_eq!(store.snapshots.len(), 3);
    }

    #[test]
    fn test_get_current_state() {
        let mut store = MemoryStore::new();
        let node_id = Uuid::new_v4();

        // Save initial state
        let state = GA3::scalar(10.0);
        let mut clock = VectorClock::new();
        store.save_snapshot(&state, &clock);

        // Add operation
        clock.tick(node_id);
        store.append_operation(StateDelta::additive(
            GA3::scalar(5.0),
            VectorClock::new(),
            clock.clone(),
            node_id,
        ));

        let current = store.get_current_state().unwrap();
        assert!((current.scalar_part() - 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_operations_since_sequence() {
        let mut store = MemoryStore::new();
        let node_id = Uuid::new_v4();

        store.save_snapshot(&GA3::scalar(0.0), &VectorClock::new());

        let mut clock = VectorClock::new();
        for _ in 0..5 {
            clock.tick(node_id);
            store.append_operation(StateDelta::additive(
                GA3::scalar(1.0),
                VectorClock::new(),
                clock.clone(),
                node_id,
            ));
        }

        let ops = store.operations_since_sequence(3);
        assert_eq!(ops.len(), 2); // sequences 3 and 4
    }
}
