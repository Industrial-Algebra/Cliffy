// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! DOM projection module for Algebraic TSX
//!
//! This module provides direct DOM projection from geometric state,
//! bypassing the virtual DOM entirely.
//!
//! # Key Concepts
//!
//! - **`DOMProjection`**: Connects geometric state to a DOM property
//! - **`DOMUpdate`**: Types of updates that can be applied to DOM
//! - **`ProjectionScheduler`**: Batches updates to animation frames
//!
//! # Example (JavaScript)
//!
//! ```javascript
//! import { DOMProjection, ProjectionScheduler } from '@cliffy/core';
//!
//! const scheduler = new ProjectionScheduler();
//!
//! // Create a projection from state to element text
//! const textProj = DOMProjection.text(element, state => `Count: ${state.scalar()}`);
//!
//! // Register with scheduler for batched updates
//! scheduler.register(textProj);
//!
//! // Update state - changes will be batched to next animation frame
//! state.setScalar(42);
//! ```

use js_sys::Function;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};

/// A DOM update operation.
#[derive(Debug, Clone)]
pub enum DOMUpdateKind {
    /// Update text content
    Text(String),
    /// Update a style property
    Style { property: String, value: String },
    /// Update an attribute
    Attribute { name: String, value: String },
    /// Remove an attribute
    RemoveAttribute { name: String },
    /// Update a CSS class
    Class { name: String, add: bool },
    /// Update a data attribute
    Data { key: String, value: String },
}

/// A projection from geometric state to DOM.
///
/// `DOMProjection` directly updates DOM properties without virtual DOM
/// reconciliation, providing efficient reactive updates.
#[wasm_bindgen]
pub struct DOMProjection {
    element: Element,
    projection_fn: Rc<RefCell<Option<Function>>>,
    update_type: DOMProjectionType,
}

/// Type of DOM projection.
#[derive(Clone)]
enum DOMProjectionType {
    Text,
    Style(String),
    Attribute(String),
    Class(String),
    Data(String),
    /// Reserved for user-defined custom projections
    #[allow(dead_code)]
    Custom,
}

#[wasm_bindgen]
impl DOMProjection {
    /// Create a text content projection.
    ///
    /// The projection function should return a string to set as textContent.
    ///
    /// # Arguments
    /// * `element` - The DOM element to update
    /// * `project_fn` - Function that takes state and returns string
    #[wasm_bindgen]
    #[must_use]
    pub fn text(element: Element, project_fn: Function) -> Self {
        Self {
            element,
            projection_fn: Rc::new(RefCell::new(Some(project_fn))),
            update_type: DOMProjectionType::Text,
        }
    }

    /// Create a style property projection.
    ///
    /// # Arguments
    /// * `element` - The DOM element to update
    /// * `property` - CSS property name (e.g., "color", "transform")
    /// * `project_fn` - Function that takes state and returns CSS value string
    #[wasm_bindgen]
    #[must_use]
    pub fn style(element: Element, property: String, project_fn: Function) -> Self {
        Self {
            element,
            projection_fn: Rc::new(RefCell::new(Some(project_fn))),
            update_type: DOMProjectionType::Style(property),
        }
    }

    /// Create an attribute projection.
    ///
    /// # Arguments
    /// * `element` - The DOM element to update
    /// * `attribute` - Attribute name
    /// * `project_fn` - Function that takes state and returns attribute value
    #[wasm_bindgen]
    #[must_use]
    pub fn attribute(element: Element, attribute: String, project_fn: Function) -> Self {
        Self {
            element,
            projection_fn: Rc::new(RefCell::new(Some(project_fn))),
            update_type: DOMProjectionType::Attribute(attribute),
        }
    }

    /// Create a CSS class toggle projection.
    ///
    /// # Arguments
    /// * `element` - The DOM element to update
    /// * `class_name` - CSS class name
    /// * `project_fn` - Function that takes state and returns boolean (add/remove)
    #[wasm_bindgen(js_name = classToggle)]
    #[must_use]
    pub fn class_toggle(element: Element, class_name: String, project_fn: Function) -> Self {
        Self {
            element,
            projection_fn: Rc::new(RefCell::new(Some(project_fn))),
            update_type: DOMProjectionType::Class(class_name),
        }
    }

    /// Create a data attribute projection.
    ///
    /// # Arguments
    /// * `element` - The DOM element to update
    /// * `data_key` - Data attribute key (without "data-" prefix)
    /// * `project_fn` - Function that takes state and returns value
    #[wasm_bindgen]
    #[must_use]
    pub fn data(element: Element, data_key: String, project_fn: Function) -> Self {
        Self {
            element,
            projection_fn: Rc::new(RefCell::new(Some(project_fn))),
            update_type: DOMProjectionType::Data(data_key),
        }
    }

    /// Update the DOM element with a new state value.
    ///
    /// Call this with the result of your projection function.
    /// # Errors
    ///
    /// Returns a `JsValue` when a DOM update or the registered projection
    /// callback fails.
    #[wasm_bindgen]
    pub fn update(&self, value: &JsValue) -> Result<(), JsValue> {
        match &self.update_type {
            DOMProjectionType::Text => {
                let text = value.as_string().unwrap_or_default();
                self.element.set_text_content(Some(&text));
            }
            DOMProjectionType::Style(property) => {
                if let Some(html_elem) = self.element.dyn_ref::<HtmlElement>() {
                    let style = html_elem.style();
                    let value_str = value.as_string().unwrap_or_default();
                    style.set_property(property, &value_str)?;
                }
            }
            DOMProjectionType::Attribute(name) => {
                let value_str = value.as_string().unwrap_or_default();
                self.element.set_attribute(name, &value_str)?;
            }
            DOMProjectionType::Class(class_name) => {
                let add = value.is_truthy();
                let class_list = self.element.class_list();
                if add {
                    class_list.add_1(class_name)?;
                } else {
                    class_list.remove_1(class_name)?;
                }
            }
            DOMProjectionType::Data(key) => {
                if let Some(html_elem) = self.element.dyn_ref::<HtmlElement>() {
                    let dataset = html_elem.dataset();
                    let value_str = value.as_string().unwrap_or_default();
                    dataset.set(key, &value_str)?;
                }
            }
            DOMProjectionType::Custom => {
                // Custom projections handle their own updates
            }
        }
        Ok(())
    }

    /// Apply the projection function and update the DOM.
    ///
    /// # Arguments
    /// * `state` - The geometric state to project
    /// # Errors
    ///
    /// Returns a `JsValue` when the invoked JavaScript callback throws or
    /// returns an unexpected type.
    #[wasm_bindgen(js_name = applyWithState)]
    pub fn apply_with_state(&self, state: &crate::GeometricState) -> Result<(), JsValue> {
        let fn_ref = self.projection_fn.borrow();
        if let Some(ref project_fn) = *fn_ref {
            let this = JsValue::null();
            // Convert GeometricState to JsValue for the callback
            let state_array = state.to_array();
            let result = project_fn.call1(&this, &state_array)?;
            self.update(&result)?;
        }
        Ok(())
    }

    /// Get the element this projection targets.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn element(&self) -> Element {
        self.element.clone()
    }
}

/// Scheduler for batching DOM updates to animation frames.
///
/// Groups multiple projection updates together and applies them
/// in a single animation frame for better performance.
#[wasm_bindgen]
pub struct ProjectionScheduler {
    pending_updates: Rc<RefCell<Vec<PendingUpdate>>>,
    frame_requested: Rc<RefCell<bool>>,
}

struct PendingUpdate {
    projection: DOMProjection,
    value: JsValue,
}

#[wasm_bindgen]
impl ProjectionScheduler {
    /// Create a new projection scheduler.
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            pending_updates: Rc::new(RefCell::new(Vec::new())),
            frame_requested: Rc::new(RefCell::new(false)),
        }
    }

    /// Schedule an update for the next animation frame.
    ///
    /// # Arguments
    /// * `projection` - The DOM projection to update
    /// * `value` - The value to apply
    /// # Errors
    ///
    /// Returns a `JsValue` when the animation-frame request fails.
    #[wasm_bindgen]
    pub fn schedule(&self, projection: DOMProjection, value: JsValue) -> Result<(), JsValue> {
        self.pending_updates
            .borrow_mut()
            .push(PendingUpdate { projection, value });

        // Request animation frame if not already requested
        if !*self.frame_requested.borrow() {
            *self.frame_requested.borrow_mut() = true;
            self.request_frame()?;
        }

        Ok(())
    }

    /// Flush all pending updates immediately (don't wait for animation frame).
    /// # Errors
    ///
    /// Returns a `JsValue` when any pending projection update fails.
    #[wasm_bindgen]
    pub fn flush(&self) -> Result<(), JsValue> {
        let updates: Vec<_> = self.pending_updates.borrow_mut().drain(..).collect();

        for update in updates {
            update.projection.update(&update.value)?;
        }

        *self.frame_requested.borrow_mut() = false;
        Ok(())
    }

    /// Get the number of pending updates.
    #[wasm_bindgen(getter, js_name = pendingCount)]
    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.pending_updates.borrow().len()
    }

    fn request_frame(&self) -> Result<(), JsValue> {
        let pending = Rc::clone(&self.pending_updates);
        let frame_requested = Rc::clone(&self.frame_requested);

        let callback = Closure::once(Box::new(move || {
            let updates: Vec<_> = pending.borrow_mut().drain(..).collect();

            for update in updates {
                let _ = update.projection.update(&update.value);
            }

            *frame_requested.borrow_mut() = false;
        }) as Box<dyn FnOnce()>);

        let window = web_sys::window().ok_or("No window")?;
        window.request_animation_frame(callback.as_ref().unchecked_ref())?;
        callback.forget();

        Ok(())
    }
}

impl Default for ProjectionScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Create multiple DOM projections from a single element.
///
/// This is a convenience function for setting up multiple projections
/// on the same element efficiently.
#[wasm_bindgen]
pub struct ElementProjections {
    element: Element,
    projections: Vec<DOMProjection>,
}

#[wasm_bindgen]
impl ElementProjections {
    /// Create a new `ElementProjections` builder.
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new(element: Element) -> Self {
        Self {
            element,
            projections: Vec::new(),
        }
    }

    /// Add a text content projection.
    #[wasm_bindgen]
    #[must_use]
    pub fn text(mut self, project_fn: Function) -> Self {
        let proj = DOMProjection::text(self.element.clone(), project_fn);
        self.projections.push(proj);
        self
    }

    /// Add a style projection.
    #[wasm_bindgen]
    #[must_use]
    pub fn style(mut self, property: String, project_fn: Function) -> Self {
        let proj = DOMProjection::style(self.element.clone(), property, project_fn);
        self.projections.push(proj);
        self
    }

    /// Add an attribute projection.
    #[wasm_bindgen]
    #[must_use]
    pub fn attribute(mut self, name: String, project_fn: Function) -> Self {
        let proj = DOMProjection::attribute(self.element.clone(), name, project_fn);
        self.projections.push(proj);
        self
    }

    /// Add a class toggle projection.
    #[wasm_bindgen(js_name = classToggle)]
    #[must_use]
    pub fn class_toggle(mut self, class_name: String, project_fn: Function) -> Self {
        let proj = DOMProjection::class_toggle(self.element.clone(), class_name, project_fn);
        self.projections.push(proj);
        self
    }

    /// Get the number of projections.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn count(&self) -> usize {
        self.projections.len()
    }

    /// Apply all projections with the given state.
    /// # Errors
    ///
    /// Returns a `JsValue` when the invoked JavaScript callback throws or
    /// returns an unexpected type.
    #[wasm_bindgen(js_name = applyAll)]
    pub fn apply_all(&self, state: &crate::GeometricState) -> Result<(), JsValue> {
        for proj in &self.projections {
            proj.apply_with_state(state)?;
        }
        Ok(())
    }
}

/// Helper to create a transform string from geometric state.
///
/// Useful for CSS transform properties.
#[wasm_bindgen(js_name = stateToTransform)]
#[must_use]
pub fn state_to_transform(x: f64, y: f64, z: f64, rotate: f64, scale: f64) -> String {
    format!("translate3d({x}px, {y}px, {z}px) rotate({rotate}deg) scale({scale})")
}

/// Helper to create an RGBA color string from geometric state.
///
/// Maps 4 components to RGBA values.
#[wasm_bindgen(js_name = stateToColor)]
#[must_use]
pub fn state_to_color(r: f64, g: f64, b: f64, a: f64) -> String {
    let r = (r.clamp(0.0, 1.0) * 255.0) as u8;
    let g = (g.clamp(0.0, 1.0) * 255.0) as u8;
    let b = (b.clamp(0.0, 1.0) * 255.0) as u8;
    let a = a.clamp(0.0, 1.0);
    format!("rgba({r}, {g}, {b}, {a})")
}
