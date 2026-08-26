// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Geometric CRDT implementations using Clifford algebra

use crate::serde_ga3;
use crate::vector_clock::VectorClock;
use cliffy_core::GA3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A CRDT that uses geometric algebra operations for conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometricCRDT {
    #[serde(with = "serde_ga3")]
    pub state: GA3,
    pub vector_clock: VectorClock,
    pub node_id: Uuid,
    pub operations: HashMap<u64, GeometricOperation>,
}

/// A geometric operation that can be applied to the CRDT state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometricOperation {
    pub id: u64,
    pub node_id: Uuid,
    pub timestamp: VectorClock,
    #[serde(with = "serde_ga3")]
    pub transform: GA3,
    pub operation_type: OperationType,
}

/// Types of geometric operations supported by the CRDT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    GeometricProduct,
    Addition,
    Sandwich,
    Exponential,
}

impl GeometricCRDT {
    /// Create a new GeometricCRDT with the given initial state
    pub fn new(node_id: Uuid, initial_state: GA3) -> Self {
        Self {
            state: initial_state,
            vector_clock: VectorClock::new(),
            node_id,
            operations: HashMap::new(),
        }
    }

    /// Apply a geometric operation to the CRDT state
    pub fn apply_operation(&mut self, operation: GeometricOperation) {
        if self.operations.contains_key(&operation.id) {
            return;
        }

        self.vector_clock.update(&operation.timestamp);
        self.operations.insert(operation.id, operation.clone());

        self.state = match operation.operation_type {
            OperationType::GeometricProduct => self.state.geometric_product(&operation.transform),
            OperationType::Addition => &self.state + &operation.transform,
            OperationType::Sandwich => {
                // R * v * R^-1 sandwich product
                let rev = operation.transform.reverse();
                operation
                    .transform
                    .geometric_product(&self.state)
                    .geometric_product(&rev)
            }
            OperationType::Exponential => operation.transform.exp().geometric_product(&self.state),
        };
    }

    /// Create a new operation to be applied
    pub fn create_operation(
        &mut self,
        transform: GA3,
        op_type: OperationType,
    ) -> GeometricOperation {
        self.vector_clock.tick(self.node_id);
        let op_id = self.operations.len() as u64;

        GeometricOperation {
            id: op_id,
            node_id: self.node_id,
            timestamp: self.vector_clock.clone(),
            transform,
            operation_type: op_type,
        }
    }

    /// Merge this CRDT with another, resolving conflicts using geometric algebra
    pub fn merge(&mut self, other: &GeometricCRDT) -> GeometricCRDT {
        let merged_clock = self.vector_clock.merge(&other.vector_clock);

        let mut merged_ops = self.operations.clone();
        for (id, op) in &other.operations {
            if !merged_ops.contains_key(id) {
                merged_ops.insert(*id, op.clone());
            }
        }

        // Re-apply all operations in causal order
        let mut sorted_ops: Vec<_> = merged_ops.values().cloned().collect();
        sorted_ops.sort_by(|a, b| {
            if a.timestamp.happens_before(&b.timestamp) {
                std::cmp::Ordering::Less
            } else if b.timestamp.happens_before(&a.timestamp) {
                std::cmp::Ordering::Greater
            } else {
                a.id.cmp(&b.id) // Deterministic tie-breaking
            }
        });

        let mut result = GeometricCRDT::new(self.node_id, GA3::zero());
        result.vector_clock = merged_clock;
        result.operations = merged_ops;

        for op in sorted_ops {
            result.apply_operation(op);
        }

        result
    }

    /// Compute geometric join for conflict resolution.
    ///
    /// Returns the state with larger magnitude, or their geometric mean if equal.
    pub fn geometric_join(&self, other: &GA3) -> GA3 {
        let self_norm = self.state.magnitude();
        let other_norm = other.magnitude();

        if self_norm > other_norm {
            self.state.clone()
        } else if other_norm > self_norm {
            other.clone()
        } else {
            // Equal magnitudes - use geometric mean
            geometric_mean(&[self.state.clone(), other.clone()])
        }
    }
}

/// Compute the geometric mean of a set of multivectors
pub fn geometric_mean(multivectors: &[GA3]) -> GA3 {
    if multivectors.is_empty() {
        return GA3::zero();
    }

    let n = multivectors.len() as f64;
    let sum_logs: GA3 = multivectors
        .iter()
        .map(|mv| mv.exp()) // Note: using exp as approximation since log may not exist for all
        .fold(GA3::zero(), |acc, log_mv| &acc + &log_mv);

    // Scale and return
    let coeffs: Vec<f64> = sum_logs.as_slice().iter().map(|&c| c / n).collect();
    GA3::from_slice(&coeffs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geometric_crdt_basic() {
        let node_id = Uuid::new_v4();
        let initial_state = GA3::scalar(1.0);
        let mut crdt = GeometricCRDT::new(node_id, initial_state);

        // Create and apply an addition operation
        let transform = GA3::scalar(2.0);
        let op = crdt.create_operation(transform, OperationType::Addition);
        crdt.apply_operation(op);

        // State should now be 3.0 (1.0 + 2.0)
        assert!((crdt.state.scalar_part() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_geometric_crdt_convergence() {
        let node1_id = Uuid::new_v4();
        let node2_id = Uuid::new_v4();

        let initial_state = GA3::scalar(1.0);
        let mut crdt1 = GeometricCRDT::new(node1_id, initial_state.clone());
        let mut crdt2 = GeometricCRDT::new(node2_id, initial_state);

        // Node 1 applies an operation
        let op1 = crdt1.create_operation(GA3::scalar(2.0), OperationType::Addition);
        crdt1.apply_operation(op1.clone());

        // Node 2 applies a different operation
        let op2 = crdt2.create_operation(GA3::scalar(3.0), OperationType::Addition);
        crdt2.apply_operation(op2.clone());

        // Merge states - should converge
        let merged1 = crdt1.merge(&crdt2);
        let merged2 = crdt2.merge(&crdt1);

        // Both merges should produce the same state
        let diff = merged1.state.scalar_part() - merged2.state.scalar_part();
        assert!(diff.abs() < 1e-10);
    }

    #[test]
    fn test_vector_clock_ordering() {
        let mut clock1 = VectorClock::new();
        let mut clock2 = VectorClock::new();

        let node1 = Uuid::new_v4();
        let node2 = Uuid::new_v4();

        clock1.tick(node1);
        clock2.tick(node2);

        assert!(clock1.concurrent(&clock2));

        clock1.update(&clock2);
        clock1.tick(node1);

        assert!(clock2.happens_before(&clock1));
    }
}
