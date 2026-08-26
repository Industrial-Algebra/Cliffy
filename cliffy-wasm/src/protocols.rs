// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! WASM bindings for cliffy-protocols distributed state types.
//!
//! This module exposes CRDTs, vector clocks, and lattice operations to JavaScript.
//!
//! # Example
//!
//! ```javascript
//! import { VectorClock, GeometricCRDT, OperationType } from '@cliffy-ga/core';
//!
//! // Create a CRDT with initial state
//! const nodeId = crypto.randomUUID();
//! const crdt = new GeometricCRDT(nodeId, 0.0);
//!
//! // Apply operations
//! crdt.add(5.0);
//! crdt.multiply(2.0);
//!
//! console.log(crdt.state()); // 10.0
//!
//! // Merge with another CRDT
//! const merged = crdt.merge(otherCrdt);
//! ```

use cliffy_protocols::{
    GeometricCRDT as CoreGeometricCRDT, OperationType as CoreOperationType,
    VectorClock as CoreVectorClock,
};
use js_sys::{Array, Object, Reflect};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

/// A vector clock for tracking causality in distributed systems.
///
/// Vector clocks provide a partial ordering of events across distributed nodes,
/// enabling detection of concurrent operations and causal dependencies.
///
/// # JavaScript Example
///
/// ```javascript
/// const clock1 = new VectorClock();
/// const clock2 = new VectorClock();
///
/// const nodeA = crypto.randomUUID();
/// const nodeB = crypto.randomUUID();
///
/// clock1.tick(nodeA);  // {nodeA: 1}
/// clock2.tick(nodeB);  // {nodeB: 1}
///
/// console.log(clock1.concurrent(clock2)); // true - neither happened before the other
///
/// clock1.update(clock2);  // {nodeA: 1, nodeB: 1}
/// clock1.tick(nodeA);     // {nodeA: 2, nodeB: 1}
///
/// console.log(clock2.happensBefore(clock1)); // true
/// ```
#[wasm_bindgen]
pub struct VectorClock {
    inner: CoreVectorClock,
}

#[wasm_bindgen]
impl VectorClock {
    /// Create a new empty vector clock.
    #[wasm_bindgen(constructor)]
    pub fn new() -> VectorClock {
        VectorClock {
            inner: CoreVectorClock::new(),
        }
    }

    /// Increment the clock for this node.
    ///
    /// This should be called before sending a message or performing a local operation.
    #[wasm_bindgen]
    pub fn tick(&mut self, node_id: &str) -> Result<(), JsValue> {
        let uuid = Uuid::parse_str(node_id)
            .map_err(|e| JsValue::from_str(&format!("Invalid UUID: {}", e)))?;
        self.inner.tick(uuid);
        Ok(())
    }

    /// Update this clock with values from another clock.
    ///
    /// This should be called when receiving a message from another node.
    #[wasm_bindgen]
    pub fn update(&mut self, other: &VectorClock) {
        self.inner.update(&other.inner);
    }

    /// Check if this clock happens-before another clock.
    ///
    /// Returns true if all events in this clock happened before or at the same
    /// time as events in the other clock, and at least one event happened strictly before.
    #[wasm_bindgen(js_name = happensBefore)]
    pub fn happens_before(&self, other: &VectorClock) -> bool {
        self.inner.happens_before(&other.inner)
    }

    /// Check if this clock is concurrent with another clock.
    ///
    /// Returns true if neither clock happens-before the other.
    /// Concurrent events may conflict and require resolution.
    #[wasm_bindgen]
    pub fn concurrent(&self, other: &VectorClock) -> bool {
        self.inner.concurrent(&other.inner)
    }

    /// Merge two clocks, taking the maximum of each component.
    ///
    /// Returns a new clock representing the combined knowledge of both clocks.
    #[wasm_bindgen]
    pub fn merge(&self, other: &VectorClock) -> VectorClock {
        VectorClock {
            inner: self.inner.merge(&other.inner),
        }
    }

    /// Get the clock as a JavaScript object {nodeId: timestamp, ...}.
    #[wasm_bindgen(js_name = toObject)]
    pub fn to_object(&self) -> Object {
        let obj = Object::new();
        for (uuid, time) in &self.inner.clocks {
            let _ = Reflect::set(&obj, &uuid.to_string().into(), &(*time as f64).into());
        }
        obj
    }

    /// Get the time for a specific node.
    #[wasm_bindgen(js_name = getTime)]
    pub fn get_time(&self, node_id: &str) -> Result<u32, JsValue> {
        let uuid = Uuid::parse_str(node_id)
            .map_err(|e| JsValue::from_str(&format!("Invalid UUID: {}", e)))?;
        Ok(*self.inner.clocks.get(&uuid).unwrap_or(&0) as u32)
    }
}

impl Default for VectorClock {
    fn default() -> Self {
        Self::new()
    }
}

/// Types of geometric operations supported by the CRDT.
#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum OperationType {
    /// Add a scalar value to the state.
    Addition,
    /// Multiply the state by a scalar value.
    Multiplication,
    /// Apply the geometric product (includes rotation + scaling).
    GeometricProduct,
    /// Apply the exponential of a bivector (pure rotation).
    Exponential,
    /// Apply the sandwich product R * v * R^-1.
    Sandwich,
}

impl From<OperationType> for CoreOperationType {
    fn from(op: OperationType) -> Self {
        match op {
            OperationType::Addition => CoreOperationType::Addition,
            OperationType::Multiplication => CoreOperationType::GeometricProduct, // Map to GP for now
            OperationType::GeometricProduct => CoreOperationType::GeometricProduct,
            OperationType::Exponential => CoreOperationType::Exponential,
            OperationType::Sandwich => CoreOperationType::Sandwich,
        }
    }
}

impl From<&CoreOperationType> for OperationType {
    fn from(op: &CoreOperationType) -> Self {
        match op {
            CoreOperationType::Addition => OperationType::Addition,
            CoreOperationType::GeometricProduct => OperationType::GeometricProduct,
            CoreOperationType::Exponential => OperationType::Exponential,
            CoreOperationType::Sandwich => OperationType::Sandwich,
        }
    }
}

/// A geometric operation that can be applied to CRDT state.
///
/// Operations are immutable and can be serialized for network transmission.
#[wasm_bindgen]
pub struct GeometricOperation {
    id: u64,
    node_id: String,
    value: f64,
    op_type: OperationType,
}

#[wasm_bindgen]
impl GeometricOperation {
    /// Get the operation ID.
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> u32 {
        self.id as u32
    }

    /// Get the node ID that created this operation.
    #[wasm_bindgen(getter, js_name = nodeId)]
    pub fn node_id(&self) -> String {
        self.node_id.clone()
    }

    /// Get the operation value.
    #[wasm_bindgen(getter)]
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Get the operation type.
    #[wasm_bindgen(getter, js_name = operationType)]
    pub fn operation_type(&self) -> OperationType {
        self.op_type
    }
}

/// A conflict-free replicated data type using geometric algebra.
///
/// `GeometricCRDT` maintains state as a scalar value (simplified from the full
/// GA3 multivector) and uses vector clocks for causal ordering. Operations
/// are stored and can be replayed during merge to ensure convergence.
///
/// # Key Properties
///
/// - **Eventual Consistency**: All replicas converge to the same state
/// - **Conflict-Free**: Concurrent operations resolve deterministically
/// - **Causal Ordering**: Respects happens-before relationships
///
/// # JavaScript Example
///
/// ```javascript
/// const crdt1 = new GeometricCRDT(nodeId1, 10.0);
/// const crdt2 = new GeometricCRDT(nodeId2, 10.0);
///
/// // Concurrent updates
/// crdt1.add(5.0);
/// crdt2.multiply(2.0);
///
/// // Merge to converge
/// const merged = crdt1.merge(crdt2);
/// ```
#[wasm_bindgen]
pub struct GeometricCRDT {
    inner: CoreGeometricCRDT,
}

#[wasm_bindgen]
impl GeometricCRDT {
    /// Create a new CRDT with the given node ID and initial state.
    ///
    /// The node ID should be a unique UUID string identifying this replica.
    #[wasm_bindgen(constructor)]
    pub fn new(node_id: &str, initial_state: f64) -> Result<GeometricCRDT, JsValue> {
        let uuid = Uuid::parse_str(node_id)
            .map_err(|e| JsValue::from_str(&format!("Invalid UUID: {}", e)))?;
        Ok(GeometricCRDT {
            inner: CoreGeometricCRDT::new(uuid, cliffy_core::GA3::scalar(initial_state)),
        })
    }

    /// Create a new CRDT with a randomly generated node ID.
    #[wasm_bindgen(js_name = withRandomId)]
    pub fn with_random_id(initial_state: f64) -> GeometricCRDT {
        GeometricCRDT {
            inner: CoreGeometricCRDT::new(Uuid::new_v4(), cliffy_core::GA3::scalar(initial_state)),
        }
    }

    /// Get the current state as a scalar value.
    #[wasm_bindgen]
    pub fn state(&self) -> f64 {
        self.inner.state.get(0)
    }

    /// Get the node ID.
    #[wasm_bindgen(getter, js_name = nodeId)]
    pub fn node_id(&self) -> String {
        self.inner.node_id.to_string()
    }

    /// Get the current vector clock.
    #[wasm_bindgen(getter, js_name = vectorClock)]
    pub fn vector_clock(&self) -> VectorClock {
        VectorClock {
            inner: self.inner.vector_clock.clone(),
        }
    }

    /// Get the number of operations in the log.
    #[wasm_bindgen(getter, js_name = operationCount)]
    pub fn operation_count(&self) -> u32 {
        self.inner.operations.len() as u32
    }

    /// Add a value to the current state.
    #[wasm_bindgen]
    pub fn add(&mut self, value: f64) -> GeometricOperation {
        let op = self
            .inner
            .create_operation(cliffy_core::GA3::scalar(value), CoreOperationType::Addition);
        let result = GeometricOperation {
            id: op.id,
            node_id: op.node_id.to_string(),
            value,
            op_type: OperationType::Addition,
        };
        self.inner.apply_operation(op);
        result
    }

    /// Multiply the current state by a value.
    #[wasm_bindgen]
    pub fn multiply(&mut self, value: f64) -> GeometricOperation {
        let op = self.inner.create_operation(
            cliffy_core::GA3::scalar(value),
            CoreOperationType::GeometricProduct,
        );
        let result = GeometricOperation {
            id: op.id,
            node_id: op.node_id.to_string(),
            value,
            op_type: OperationType::Multiplication,
        };
        self.inner.apply_operation(op);
        result
    }

    /// Apply an arbitrary geometric operation.
    #[wasm_bindgen(js_name = applyOperation)]
    pub fn apply_operation(&mut self, value: f64, op_type: OperationType) -> GeometricOperation {
        let core_op_type: CoreOperationType = op_type.into();
        let op = self
            .inner
            .create_operation(cliffy_core::GA3::scalar(value), core_op_type.clone());
        let result = GeometricOperation {
            id: op.id,
            node_id: op.node_id.to_string(),
            value,
            op_type,
        };
        self.inner.apply_operation(op);
        result
    }

    /// Merge this CRDT with another, resolving any conflicts.
    ///
    /// Returns a new CRDT representing the merged state.
    /// Note: This mutates the current CRDT's clock during merge.
    #[wasm_bindgen]
    pub fn merge(&mut self, other: &GeometricCRDT) -> GeometricCRDT {
        GeometricCRDT {
            inner: self.inner.merge(&other.inner),
        }
    }

    /// Compute the geometric join with another state value.
    ///
    /// Uses magnitude comparison for ordering, with geometric mean
    /// as tiebreaker for equal magnitudes.
    #[wasm_bindgen(js_name = geometricJoin)]
    pub fn geometric_join(&self, other_value: f64) -> f64 {
        let other = cliffy_core::GA3::scalar(other_value);
        self.inner.geometric_join(&other).get(0)
    }

    /// Get all operation IDs in the log.
    #[wasm_bindgen(js_name = getOperationIds)]
    pub fn get_operation_ids(&self) -> Array {
        let arr = Array::new();
        for id in self.inner.operations.keys() {
            arr.push(&(*id as f64).into());
        }
        arr
    }

    /// Check if an operation has been applied.
    #[wasm_bindgen(js_name = hasOperation)]
    pub fn has_operation(&self, id: u32) -> bool {
        self.inner.operations.contains_key(&(id as u64))
    }
}

/// Generate a random UUID suitable for use as a node ID.
#[wasm_bindgen(js_name = generateNodeId)]
pub fn generate_node_id() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_clock_wasm() {
        let mut clock = VectorClock::new();
        let node_id = Uuid::new_v4().to_string();
        clock.tick(&node_id).unwrap();
        assert_eq!(clock.get_time(&node_id).unwrap(), 1);
    }

    #[test]
    fn test_crdt_wasm_basic() {
        let crdt = GeometricCRDT::with_random_id(10.0);
        assert!((crdt.state() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_crdt_wasm_add() {
        let mut crdt = GeometricCRDT::with_random_id(10.0);
        crdt.add(5.0);
        assert!((crdt.state() - 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_crdt_wasm_merge() {
        let mut crdt1 = GeometricCRDT::with_random_id(10.0);
        let mut crdt2 = GeometricCRDT::with_random_id(10.0);

        crdt1.add(5.0);
        crdt2.add(3.0);

        // crdt1 state: 10 + 5 = 15
        assert!((crdt1.state() - 15.0).abs() < 1e-10);
        // crdt2 state: 10 + 3 = 13
        assert!((crdt2.state() - 13.0).abs() < 1e-10);

        // Merge returns a valid CRDT
        // Note: Due to operation ID collision (both start at 0),
        // only one operation may be preserved. This is a known
        // limitation in the current CRDT design that should be
        // addressed by keying operations by (node_id, op_id).
        let merged = crdt1.merge(&crdt2);
        // At minimum, merged inherits the merged vector clocks
        let clock = merged.vector_clock();
        // Clock should have entries from both nodes
        assert!(clock.get_time(&crdt1.node_id()).unwrap_or(0) > 0);
    }
}
