// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Dataflow graph intermediate representation for Algebraic TSX
//!
//! This module provides the dataflow graph model that connects geometric
//! state to DOM operations. Unlike virtual DOM, this represents a static
//! graph of transformations that can be optimized at compile time.
//!
//! # Key Concepts
//!
//! - **Node**: A computation unit (source, transform, projection, sink)
//! - **Edge**: Data dependency between nodes
//! - **Graph**: The complete dataflow network
//!
//! # Dataflow Semantics
//!
//! ```text
//! GeometricState ──┬── Projection ──► DOM Text
//!                  │
//!                  ├── Transform ───► Projection ──► DOM Style
//!                  │
//!                  └── Sink ────────► Event Handler
//! ```
//!
//! # Example
//!
//! ```rust
//! use cliffy_core::dataflow::{DataflowGraph, Node, NodeKind};
//!
//! let mut graph = DataflowGraph::new();
//!
//! // Add a source node (geometric state)
//! let state_id = graph.add_node(Node::source("counter_state"));
//!
//! // Add a projection to extract the count
//! let proj_id = graph.add_node(Node::projection("count_text", "scalar_to_string"));
//!
//! // Connect source to projection
//! graph.connect(state_id, proj_id);
//!
//! // Add a sink (DOM text content)
//! let sink_id = graph.add_node(Node::sink("span_text", "textContent"));
//! graph.connect(proj_id, sink_id);
//! ```

use crate::GA3;
use std::collections::{HashMap, HashSet, VecDeque};

/// A unique identifier for a node in the dataflow graph.
pub type NodeId = usize;

/// A dataflow graph representing the transformation pipeline.
#[derive(Debug, Clone)]
pub struct DataflowGraph {
    /// All nodes in the graph
    nodes: Vec<Node>,
    /// Adjacency list: node -> outgoing edges
    edges: HashMap<NodeId, Vec<NodeId>>,
    /// Reverse adjacency: node -> incoming edges
    reverse_edges: HashMap<NodeId, Vec<NodeId>>,
    /// Named node lookup
    node_names: HashMap<String, NodeId>,
}

impl DataflowGraph {
    /// Create a new empty dataflow graph.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: HashMap::new(),
            reverse_edges: HashMap::new(),
            node_names: HashMap::new(),
        }
    }

    /// Add a node to the graph.
    ///
    /// Returns the node's ID for connecting edges.
    pub fn add_node(&mut self, node: Node) -> NodeId {
        let id = self.nodes.len();
        if let Some(ref name) = node.name {
            self.node_names.insert(name.clone(), id);
        }
        self.nodes.push(node);
        self.edges.insert(id, Vec::new());
        self.reverse_edges.insert(id, Vec::new());
        id
    }

    /// Connect two nodes with an edge (from -> to).
    ///
    /// Returns false if either node doesn't exist.
    pub fn connect(&mut self, from: NodeId, to: NodeId) -> bool {
        if from >= self.nodes.len() || to >= self.nodes.len() {
            return false;
        }

        self.edges.get_mut(&from).unwrap().push(to);
        self.reverse_edges.get_mut(&to).unwrap().push(from);
        true
    }

    /// Get a node by ID.
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// Get a mutable node by ID.
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }

    /// Get a node by name.
    pub fn get_node_by_name(&self, name: &str) -> Option<&Node> {
        self.node_names.get(name).and_then(|id| self.nodes.get(*id))
    }

    /// Get node ID by name.
    pub fn get_id_by_name(&self, name: &str) -> Option<NodeId> {
        self.node_names.get(name).copied()
    }

    /// Get all outgoing edges from a node.
    pub fn outgoing(&self, id: NodeId) -> &[NodeId] {
        self.edges.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get all incoming edges to a node.
    pub fn incoming(&self, id: NodeId) -> &[NodeId] {
        self.reverse_edges
            .get(&id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get the number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of edges.
    pub fn edge_count(&self) -> usize {
        self.edges.values().map(|v| v.len()).sum()
    }

    /// Get all source nodes (no incoming edges).
    pub fn sources(&self) -> Vec<NodeId> {
        (0..self.nodes.len())
            .filter(|&id| self.incoming(id).is_empty())
            .collect()
    }

    /// Get all sink nodes (no outgoing edges).
    pub fn sinks(&self) -> Vec<NodeId> {
        (0..self.nodes.len())
            .filter(|&id| self.outgoing(id).is_empty())
            .collect()
    }

    /// Perform topological sort of the graph.
    ///
    /// Returns None if the graph has cycles.
    pub fn topological_sort(&self) -> Option<Vec<NodeId>> {
        let mut in_degree: HashMap<NodeId, usize> = HashMap::new();
        let mut result = Vec::new();
        let mut queue = VecDeque::new();

        // Initialize in-degrees
        for id in 0..self.nodes.len() {
            in_degree.insert(id, self.incoming(id).len());
        }

        // Start with nodes that have no dependencies
        for (&id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(id);
            }
        }

        while let Some(id) = queue.pop_front() {
            result.push(id);

            for &neighbor in self.outgoing(id) {
                let degree = in_degree.get_mut(&neighbor).unwrap();
                *degree -= 1;
                if *degree == 0 {
                    queue.push_back(neighbor);
                }
            }
        }

        if result.len() == self.nodes.len() {
            Some(result)
        } else {
            None // Cycle detected
        }
    }

    /// Check if the graph has any cycles.
    pub fn has_cycles(&self) -> bool {
        self.topological_sort().is_none()
    }

    /// Find all nodes reachable from a given node.
    pub fn reachable_from(&self, start: NodeId) -> HashSet<NodeId> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(start);
        visited.insert(start);

        while let Some(id) = queue.pop_front() {
            for &neighbor in self.outgoing(id) {
                if visited.insert(neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }

        visited
    }

    /// Iterate over all nodes.
    pub fn nodes(&self) -> impl Iterator<Item = (NodeId, &Node)> {
        self.nodes.iter().enumerate()
    }
}

impl Default for DataflowGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// A node in the dataflow graph.
#[derive(Debug, Clone)]
pub struct Node {
    /// Node name (for debugging and lookup)
    pub name: Option<String>,
    /// The kind of computation this node performs
    pub kind: NodeKind,
    /// Optional constant value (for constant folding)
    pub constant_value: Option<GA3>,
}

impl Node {
    /// Create a new node with the given kind.
    pub fn new(kind: NodeKind) -> Self {
        Self {
            name: None,
            kind,
            constant_value: None,
        }
    }

    /// Create a source node (geometric state input).
    pub fn source(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            kind: NodeKind::Source,
            constant_value: None,
        }
    }

    /// Create a projection node (state -> value).
    pub fn projection(name: impl Into<String>, projection_type: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            kind: NodeKind::Projection(ProjectionSpec {
                projection_type: projection_type.into(),
            }),
            constant_value: None,
        }
    }

    /// Create a transform node (geometric operation).
    pub fn transform(name: impl Into<String>, transform_type: TransformType) -> Self {
        Self {
            name: Some(name.into()),
            kind: NodeKind::Transform(transform_type),
            constant_value: None,
        }
    }

    /// Create a sink node (output to DOM).
    pub fn sink(name: impl Into<String>, target_property: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            kind: NodeKind::Sink(SinkSpec {
                target_property: target_property.into(),
            }),
            constant_value: None,
        }
    }

    /// Create a combine node (multiple inputs).
    pub fn combine(name: impl Into<String>, combiner: CombinerType) -> Self {
        Self {
            name: Some(name.into()),
            kind: NodeKind::Combine(combiner),
            constant_value: None,
        }
    }

    /// Create a conditional node.
    pub fn conditional(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            kind: NodeKind::Conditional,
            constant_value: None,
        }
    }

    /// Create a constant node.
    pub fn constant(name: impl Into<String>, value: GA3) -> Self {
        Self {
            name: Some(name.into()),
            kind: NodeKind::Constant,
            constant_value: Some(value),
        }
    }

    /// Check if this is a source node.
    pub fn is_source(&self) -> bool {
        matches!(self.kind, NodeKind::Source)
    }

    /// Check if this is a sink node.
    pub fn is_sink(&self) -> bool {
        matches!(self.kind, NodeKind::Sink(_))
    }
}

/// The kind of computation a node performs.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    /// Input source (geometric state)
    Source,
    /// Projection from geometric state to value
    Projection(ProjectionSpec),
    /// Geometric transformation
    Transform(TransformType),
    /// Output sink (DOM update)
    Sink(SinkSpec),
    /// Combine multiple inputs
    Combine(CombinerType),
    /// Conditional (switch between inputs)
    Conditional,
    /// Constant value
    Constant,
}

/// Specification for a projection node.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectionSpec {
    /// Type of projection (e.g., "scalar", "vector", "magnitude")
    pub projection_type: String,
}

/// Type of geometric transformation.
#[derive(Debug, Clone, PartialEq)]
pub enum TransformType {
    /// Translation by a vector
    Translation { x: f64, y: f64, z: f64 },
    /// Rotation in a plane
    Rotation { angle: f64, plane: RotationPlane },
    /// Uniform scaling
    Scale { factor: f64 },
    /// Arbitrary rotor transformation
    Rotor { coefficients: [f64; 8] },
    /// Linear interpolation
    Lerp { t: f64 },
    /// Custom named transform
    Custom { name: String },
}

/// Plane of rotation in 3D.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationPlane {
    XY,
    YZ,
    ZX,
}

/// Specification for a sink node.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkSpec {
    /// DOM property to update (e.g., "textContent", "style.color")
    pub target_property: String,
}

/// Type of combiner for multiple inputs.
#[derive(Debug, Clone, PartialEq)]
pub enum CombinerType {
    /// Add all inputs
    Sum,
    /// Multiply all inputs (geometric product)
    Product,
    /// Take minimum
    Min,
    /// Take maximum
    Max,
    /// Average (geometric mean)
    Average,
    /// Custom combiner
    Custom(String),
}

/// Builder for constructing dataflow graphs fluently.
pub struct GraphBuilder {
    graph: DataflowGraph,
    last_node: Option<NodeId>,
}

impl GraphBuilder {
    /// Create a new graph builder.
    pub fn new() -> Self {
        Self {
            graph: DataflowGraph::new(),
            last_node: None,
        }
    }

    /// Add a source node.
    pub fn source(mut self, name: impl Into<String>) -> Self {
        let id = self.graph.add_node(Node::source(name));
        self.last_node = Some(id);
        self
    }

    /// Add a projection node, connecting from the last node.
    pub fn project(mut self, name: impl Into<String>, projection_type: impl Into<String>) -> Self {
        let id = self.graph.add_node(Node::projection(name, projection_type));
        if let Some(last) = self.last_node {
            self.graph.connect(last, id);
        }
        self.last_node = Some(id);
        self
    }

    /// Add a transform node, connecting from the last node.
    pub fn transform(mut self, name: impl Into<String>, transform_type: TransformType) -> Self {
        let id = self.graph.add_node(Node::transform(name, transform_type));
        if let Some(last) = self.last_node {
            self.graph.connect(last, id);
        }
        self.last_node = Some(id);
        self
    }

    /// Add a sink node, connecting from the last node.
    pub fn sink(mut self, name: impl Into<String>, target_property: impl Into<String>) -> Self {
        let id = self.graph.add_node(Node::sink(name, target_property));
        if let Some(last) = self.last_node {
            self.graph.connect(last, id);
        }
        self.last_node = Some(id);
        self
    }

    /// Branch from a named node (for multiple paths).
    pub fn from(mut self, name: &str) -> Self {
        self.last_node = self.graph.get_id_by_name(name);
        self
    }

    /// Build and return the graph.
    pub fn build(self) -> DataflowGraph {
        self.graph
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_creation() {
        let mut graph = DataflowGraph::new();

        let source = graph.add_node(Node::source("state"));
        let sink = graph.add_node(Node::sink("output", "textContent"));

        graph.connect(source, sink);

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = DataflowGraph::new();

        let a = graph.add_node(Node::source("a"));
        let b = graph.add_node(Node::projection("b", "scalar"));
        let c = graph.add_node(Node::sink("c", "text"));

        graph.connect(a, b);
        graph.connect(b, c);

        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted, vec![a, b, c]);
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = DataflowGraph::new();

        let a = graph.add_node(Node::source("a"));
        let b = graph.add_node(Node::transform("b", TransformType::Scale { factor: 2.0 }));
        let c = graph.add_node(Node::sink("c", "text"));

        graph.connect(a, b);
        graph.connect(b, c);
        graph.connect(c, a); // Creates cycle

        assert!(graph.has_cycles());
    }

    #[test]
    fn test_sources_and_sinks() {
        let mut graph = DataflowGraph::new();

        let source = graph.add_node(Node::source("input"));
        let transform = graph.add_node(Node::transform(
            "scale",
            TransformType::Scale { factor: 2.0 },
        ));
        let sink = graph.add_node(Node::sink("output", "value"));

        graph.connect(source, transform);
        graph.connect(transform, sink);

        assert_eq!(graph.sources(), vec![source]);
        assert_eq!(graph.sinks(), vec![sink]);
    }

    #[test]
    fn test_node_lookup_by_name() {
        let mut graph = DataflowGraph::new();

        graph.add_node(Node::source("counter_state"));

        let node = graph.get_node_by_name("counter_state").unwrap();
        assert!(node.is_source());
    }

    #[test]
    fn test_reachable_from() {
        let mut graph = DataflowGraph::new();

        let a = graph.add_node(Node::source("a"));
        let b = graph.add_node(Node::projection("b", "scalar"));
        let c = graph.add_node(Node::sink("c", "text"));
        let d = graph.add_node(Node::source("d")); // Disconnected

        graph.connect(a, b);
        graph.connect(b, c);

        let reachable = graph.reachable_from(a);
        assert!(reachable.contains(&a));
        assert!(reachable.contains(&b));
        assert!(reachable.contains(&c));
        assert!(!reachable.contains(&d));
    }

    #[test]
    fn test_graph_builder() {
        let graph = GraphBuilder::new()
            .source("state")
            .project("count", "scalar")
            .sink("display", "textContent")
            .build();

        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 2);

        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted.len(), 3);
    }

    #[test]
    fn test_graph_builder_branching() {
        let graph = GraphBuilder::new()
            .source("state")
            .project("text_proj", "to_string")
            .sink("text_out", "textContent")
            .from("state")
            .project("color_proj", "to_color")
            .sink("style_out", "style.color")
            .build();

        assert_eq!(graph.node_count(), 5);
        // state -> text_proj -> text_out
        // state -> color_proj -> style_out
        assert_eq!(graph.edge_count(), 4);
    }

    #[test]
    fn test_transform_types() {
        let t1 = TransformType::Translation {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let t2 = TransformType::Rotation {
            angle: 1.57,
            plane: RotationPlane::XY,
        };
        let t3 = TransformType::Scale { factor: 2.0 };

        let node1 = Node::transform("t1", t1);
        let node2 = Node::transform("t2", t2);
        let node3 = Node::transform("t3", t3);

        assert!(matches!(node1.kind, NodeKind::Transform(_)));
        assert!(matches!(node2.kind, NodeKind::Transform(_)));
        assert!(matches!(node3.kind, NodeKind::Transform(_)));
    }

    #[test]
    fn test_constant_node() {
        let const_node = Node::constant("pi", GA3::scalar(std::f64::consts::PI));

        assert!(matches!(const_node.kind, NodeKind::Constant));
        assert!(const_node.constant_value.is_some());

        let value = const_node.constant_value.unwrap();
        assert!((value.get(0) - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_combine_node() {
        let sum_node = Node::combine("sum", CombinerType::Sum);
        let product_node = Node::combine("product", CombinerType::Product);

        assert!(matches!(
            sum_node.kind,
            NodeKind::Combine(CombinerType::Sum)
        ));
        assert!(matches!(
            product_node.kind,
            NodeKind::Combine(CombinerType::Product)
        ));
    }
}
