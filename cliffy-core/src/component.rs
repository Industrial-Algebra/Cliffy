// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Component model for Algebraic TSX
//!
//! Components are geometric morphisms from state to renderable elements.
//! They compose naturally via geometric product of their state spaces.
//!
//! # Key Concepts
//!
//! - **Component**: A function from geometric state to Element
//! - **Element**: A node in the virtual element tree (not virtual DOM - dataflow graph)
//! - **Composition**: Components compose via geometric product
//! - **Props**: Constraints on a component's state space
//!
//! # Example
//!
//! ```rust
//! use cliffy_core::component::{Component, Element, ElementKind, Props};
//! use cliffy_core::GA3;
//!
//! // Define a simple counter component
//! struct Counter {
//!     initial: i32,
//! }
//!
//! impl Component for Counter {
//!     fn render(&self, state: &GA3) -> Element {
//!         let count = state.get(0) as i32;
//!         Element::new(ElementKind::Text(format!("Count: {}", count)))
//!     }
//! }
//!
//! let counter = Counter { initial: 0 };
//! let element = counter.render(&GA3::scalar(42.0));
//! ```

use crate::GA3;
use std::collections::HashMap;
use std::sync::Arc;

/// A component is a geometric morphism from state to renderable output.
///
/// Components transform geometric state into Element trees. The state
/// is always a GA3 multivector, but components can interpret it in
/// any way they choose (scalar counter, 3D position, etc.).
pub trait Component: Send + Sync {
    /// Render the component given the current geometric state.
    ///
    /// The state represents the component's local state. Components
    /// should be pure functions of their state.
    fn render(&self, state: &GA3) -> Element;

    /// Get the initial state for this component.
    ///
    /// Returns the geometric state to use when the component mounts.
    fn initial_state(&self) -> GA3 {
        GA3::zero()
    }

    /// Get the component's type name for debugging.
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// A renderable element in the algebraic element tree.
///
/// Unlike virtual DOM nodes, Elements are nodes in a geometric dataflow graph.
/// They represent projections from geometric state to DOM operations.
#[derive(Debug, Clone)]
pub struct Element {
    /// The kind of element (tag, text, component reference)
    pub kind: ElementKind,
    /// Props/attributes for this element
    pub props: Props,
    /// Child elements
    pub children: Vec<Self>,
    /// Unique key for reconciliation (optional)
    pub key: Option<String>,
}

impl Element {
    /// Create a new element with the given kind.
    #[must_use]
    pub fn new(kind: ElementKind) -> Self {
        Self {
            kind,
            props: Props::new(),
            children: Vec::new(),
            key: None,
        }
    }

    /// Create a text element.
    pub fn text(content: impl Into<String>) -> Self {
        Self::new(ElementKind::Text(content.into()))
    }

    /// Create an element with a tag name.
    pub fn tag(name: impl Into<String>) -> Self {
        Self::new(ElementKind::Tag(name.into()))
    }

    /// Create a fragment (multiple elements without a wrapper).
    #[must_use]
    pub fn fragment(children: Vec<Self>) -> Self {
        Self {
            kind: ElementKind::Fragment,
            props: Props::new(),
            children,
            key: None,
        }
    }

    /// Create an empty element (renders nothing).
    #[must_use]
    pub fn empty() -> Self {
        Self::new(ElementKind::Empty)
    }

    /// Add a child element.
    #[must_use]
    pub fn child(mut self, child: Self) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple children.
    #[must_use]
    pub fn children(mut self, children: impl IntoIterator<Item = Self>) -> Self {
        self.children.extend(children);
        self
    }

    /// Set a prop/attribute.
    #[must_use]
    pub fn prop(mut self, key: impl Into<String>, value: PropValue) -> Self {
        self.props.set(key, value);
        self
    }

    /// Set a string prop.
    #[must_use]
    pub fn attr(self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.prop(key, PropValue::String(value.into()))
    }

    /// Set a numeric prop.
    #[must_use]
    pub fn num(self, key: impl Into<String>, value: f64) -> Self {
        self.prop(key, PropValue::Number(value))
    }

    /// Set a boolean prop.
    #[must_use]
    pub fn bool(self, key: impl Into<String>, value: bool) -> Self {
        self.prop(key, PropValue::Bool(value))
    }

    /// Set the element's key.
    #[must_use]
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Check if this is an empty element.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        matches!(self.kind, ElementKind::Empty)
    }

    /// Count total nodes in this element tree.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.children
            .iter()
            .map(Self::node_count)
            .sum::<usize>()
            .saturating_add(1)
    }
}

/// The kind of element.
#[derive(Debug, Clone, PartialEq)]
pub enum ElementKind {
    /// An HTML/DOM tag (div, span, button, etc.)
    Tag(String),
    /// Plain text content
    Text(String),
    /// A fragment (multiple elements without wrapper)
    Fragment,
    /// A component reference (for nested components)
    ComponentRef(ComponentRef),
    /// An empty element (renders nothing)
    Empty,
}

/// A reference to a child component.
#[derive(Clone)]
pub struct ComponentRef {
    /// Component type name for debugging
    pub type_name: String,
    /// The component instance (wrapped for Clone)
    pub component: Arc<dyn Component>,
    /// Props passed to the component
    pub props: Props,
}

impl std::fmt::Debug for ComponentRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentRef")
            .field("type_name", &self.type_name)
            .field("props", &self.props)
            .finish_non_exhaustive()
    }
}

impl PartialEq for ComponentRef {
    fn eq(&self, other: &Self) -> bool {
        self.type_name == other.type_name
    }
}

/// Props (properties/attributes) for an element.
#[derive(Debug, Clone, Default)]
pub struct Props {
    values: HashMap<String, PropValue>,
}

impl Props {
    /// Create empty props.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a prop value.
    pub fn set(&mut self, key: impl Into<String>, value: PropValue) {
        self.values.insert(key.into(), value);
    }

    /// Get a prop value.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&PropValue> {
        self.values.get(key)
    }

    /// Check if a prop exists.
    #[must_use]
    pub fn has(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    /// Get all prop keys.
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }

    /// Iterate over all props.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &PropValue)> {
        self.values.iter()
    }

    /// Merge with another props, other takes precedence.
    pub fn merge(&mut self, other: Self) {
        for (k, v) in other.values {
            self.values.insert(k, v);
        }
    }
}

/// A property value.
#[derive(Debug, Clone, PartialEq)]
pub enum PropValue {
    /// String value
    String(String),
    /// Numeric value
    Number(f64),
    /// Boolean value
    Bool(bool),
    /// Array of values
    Array(Vec<Self>),
    /// Nested object
    Object(HashMap<String, Self>),
    /// Null/undefined
    Null,
}

impl PropValue {
    /// Get as string, if it is one.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as f64, if it is a number.
    #[must_use]
    pub const fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get as bool, if it is one.
    #[must_use]
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

impl From<&str> for PropValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<String> for PropValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<f64> for PropValue {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

impl From<i32> for PropValue {
    fn from(n: i32) -> Self {
        Self::Number(f64::from(n))
    }
}

impl From<bool> for PropValue {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

/// A composed component that combines two components.
///
/// The state space is the geometric product of the child states.
pub struct ComposedComponent<A, B>
where
    A: Component,
    B: Component,
{
    /// First component
    pub a: A,
    /// Second component
    pub b: B,
    /// How to split state between components
    pub split: StateSplit,
}

/// How to split geometric state between composed components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateSplit {
    /// Both components share the same state
    Shared,
    /// Split by grade: A gets scalar+vector, B gets bivector+pseudoscalar
    ByGrade,
    /// A gets first 4 coefficients, B gets last 4
    ByCoefficient,
}

impl<A, B> ComposedComponent<A, B>
where
    A: Component,
    B: Component,
{
    /// Create a new composed component with shared state.
    pub const fn new(a: A, b: B) -> Self {
        Self {
            a,
            b,
            split: StateSplit::Shared,
        }
    }

    /// Create with specified state split.
    pub const fn with_split(a: A, b: B, split: StateSplit) -> Self {
        Self { a, b, split }
    }
}

impl<A, B> Component for ComposedComponent<A, B>
where
    A: Component,
    B: Component,
{
    fn render(&self, state: &GA3) -> Element {
        let (state_a, state_b) = match self.split {
            StateSplit::Shared => (state.clone(), state.clone()),
            StateSplit::ByGrade => {
                // A gets grades 0,1 (scalar + vector)
                // B gets grades 2,3 (bivector + pseudoscalar)
                let coeffs = state.as_slice();
                let a_coeffs: Vec<f64> = coeffs
                    .iter()
                    .take(4)
                    .copied()
                    .chain([0.0, 0.0, 0.0, 0.0])
                    .collect();
                let b_coeffs: Vec<f64> = [0.0, 0.0, 0.0, 0.0]
                    .into_iter()
                    .chain(coeffs.iter().skip(4).copied())
                    .collect();
                (GA3::from_slice(&a_coeffs), GA3::from_slice(&b_coeffs))
            }
            StateSplit::ByCoefficient => {
                let coeffs = state.as_slice();
                let a_coeffs: Vec<f64> = coeffs
                    .iter()
                    .take(4)
                    .copied()
                    .chain([0.0, 0.0, 0.0, 0.0])
                    .collect();
                let b_coeffs: Vec<f64> = coeffs
                    .iter()
                    .skip(4)
                    .take(4)
                    .copied()
                    .chain([0.0, 0.0, 0.0, 0.0])
                    .collect();
                (GA3::from_slice(&a_coeffs), GA3::from_slice(&b_coeffs))
            }
        };

        let elem_a = self.a.render(&state_a);
        let elem_b = self.b.render(&state_b);

        Element::fragment(vec![elem_a, elem_b])
    }

    fn initial_state(&self) -> GA3 {
        // Combine initial states
        let a_init = self.a.initial_state();
        let b_init = self.b.initial_state();

        match self.split {
            StateSplit::Shared => a_init, // Use A's initial state
            StateSplit::ByGrade | StateSplit::ByCoefficient => {
                // Combine: A in first 4, B in last 4
                let a_coeffs = a_init.as_slice();
                let b_coeffs = b_init.as_slice();
                let combined: Vec<f64> = a_coeffs
                    .iter()
                    .take(4)
                    .copied()
                    .chain(b_coeffs.iter().take(4).copied())
                    .collect();
                GA3::from_slice(&combined)
            }
        }
    }
}

/// Compose two components with shared state.
pub const fn compose<A, B>(a: A, b: B) -> ComposedComponent<A, B>
where
    A: Component,
    B: Component,
{
    ComposedComponent::new(a, b)
}

/// A function component (simple render function).
pub struct FnComponent<F>
where
    F: Fn(&GA3) -> Element + Send + Sync,
{
    render_fn: F,
    initial: GA3,
}

impl<F> FnComponent<F>
where
    F: Fn(&GA3) -> Element + Send + Sync,
{
    /// Create a new function component.
    pub fn new(render_fn: F) -> Self {
        Self {
            render_fn,
            initial: GA3::zero(),
        }
    }

    /// Create with initial state.
    pub const fn with_initial(render_fn: F, initial: GA3) -> Self {
        Self { render_fn, initial }
    }
}

impl<F> Component for FnComponent<F>
where
    F: Fn(&GA3) -> Element + Send + Sync,
{
    fn render(&self, state: &GA3) -> Element {
        (self.render_fn)(state)
    }

    fn initial_state(&self) -> GA3 {
        self.initial.clone()
    }
}

/// Create a function component from a closure.
pub fn component<F>(render_fn: F) -> FnComponent<F>
where
    F: Fn(&GA3) -> Element + Send + Sync,
{
    FnComponent::new(render_fn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_creation() {
        let elem = Element::tag("div")
            .attr("class", "container")
            .child(Element::text("Hello"));

        assert!(matches!(elem.kind, ElementKind::Tag(ref t) if t == "div"));
        assert_eq!(elem.children.len(), 1);
        assert!(elem.props.has("class"));
    }

    #[test]
    fn test_element_text() {
        let elem = Element::text("Hello, World!");

        if let ElementKind::Text(content) = &elem.kind {
            assert_eq!(content, "Hello, World!");
        } else {
            panic!("Expected text element");
        }
    }

    #[test]
    fn test_element_fragment() {
        let frag = Element::fragment(vec![Element::text("A"), Element::text("B")]);

        assert!(matches!(frag.kind, ElementKind::Fragment));
        assert_eq!(frag.children.len(), 2);
    }

    #[test]
    fn test_element_node_count() {
        let elem = Element::tag("div")
            .child(Element::tag("span").child(Element::text("Hello")))
            .child(Element::text("World"));

        // div(1) + span(1) + text(1) + text(1) = 4
        assert_eq!(elem.node_count(), 4);
    }

    #[test]
    fn test_props() {
        let mut props = Props::new();
        props.set("name", PropValue::String("test".into()));
        props.set("count", PropValue::Number(42.0));
        props.set("enabled", PropValue::Bool(true));

        assert_eq!(props.get("name").unwrap().as_str(), Some("test"));
        assert_eq!(props.get("count").unwrap().as_f64(), Some(42.0));
        assert_eq!(props.get("enabled").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_fn_component() {
        let counter = component(|state: &GA3| {
            let count = state.get(0) as i32;
            Element::text(format!("Count: {count}"))
        });

        let elem = counter.render(&GA3::scalar(5.0));

        if let ElementKind::Text(content) = &elem.kind {
            assert_eq!(content, "Count: 5");
        } else {
            panic!("Expected text element");
        }
    }

    #[test]
    fn test_composed_component() {
        let a = component(|state: &GA3| Element::text(format!("A: {}", state.get(0) as i32)));

        let b = component(|state: &GA3| Element::text(format!("B: {}", state.get(0) as i32)));

        let composed = compose(a, b);
        let elem = composed.render(&GA3::scalar(10.0));

        assert!(matches!(elem.kind, ElementKind::Fragment));
        assert_eq!(elem.children.len(), 2);
    }

    #[test]
    fn test_prop_value_conversions() {
        let s: PropValue = "hello".into();
        assert!(matches!(s, PropValue::String(_)));

        let n: PropValue = 42.0_f64.into();
        assert!(matches!(n, PropValue::Number(_)));

        let b: PropValue = true.into();
        assert!(matches!(b, PropValue::Bool(_)));

        let i: PropValue = 123_i32.into();
        assert!(matches!(i, PropValue::Number(_)));
    }

    #[test]
    fn test_element_empty() {
        let elem = Element::empty();
        assert!(elem.is_empty());
    }

    #[test]
    fn test_element_with_key() {
        let elem = Element::tag("li").with_key("item-1");
        assert_eq!(elem.key, Some("item-1".to_string()));
    }

    #[test]
    fn test_props_merge() {
        let mut props1 = Props::new();
        props1.set("a", PropValue::Number(1.0));
        props1.set("b", PropValue::Number(2.0));

        let mut props2 = Props::new();
        props2.set("b", PropValue::Number(3.0));
        props2.set("c", PropValue::Number(4.0));

        props1.merge(props2);

        assert_eq!(props1.get("a").unwrap().as_f64(), Some(1.0));
        assert_eq!(props1.get("b").unwrap().as_f64(), Some(3.0)); // Overwritten
        assert_eq!(props1.get("c").unwrap().as_f64(), Some(4.0));
    }
}
