// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Cliffy WASM - WebAssembly bindings for Cliffy
//!
//! This crate exposes Cliffy's reactive primitives to JavaScript/TypeScript
//! via WebAssembly.
//!
//! # Example (JavaScript)
//!
//! ```javascript
//! import init, { Behavior, Event, when } from '@cliffy/core';
//!
//! await init();
//!
//! const count = new Behavior(0);
//! count.subscribe(n => console.log('Count:', n));
//!
//! count.update(n => n + 1);  // Logs: Count: 1
//! ```
//!
//! # Distributed State (CRDT)
//!
//! ```javascript
//! import { GeometricCRDT, VectorClock, generateNodeId } from '@cliffy-ga/core';
//!
//! const nodeId = generateNodeId();
//! const crdt = new GeometricCRDT(nodeId, 0.0);
//!
//! crdt.add(5.0);
//! console.log(crdt.state()); // 5.0
//! ```

pub mod dom;
pub mod protocols;
pub mod test;

// Re-export protocols types at top level
pub use protocols::{
    generate_node_id, GeometricCRDT, GeometricOperation, OperationType, VectorClock,
};

// Re-export test types at top level
pub use test::{
    assert_ga3_equal, assert_magnitude, assert_zero, random_ga3, random_rotor, random_unit_vector,
    rotor_manifold, test_impossible, test_rare, unit_sphere, GeometricError, InvariantCategory,
    InvariantTestReport, Manifold, TestResult,
};

use js_sys::Function;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

/// Initialize the Cliffy WASM module.
///
/// Call this once before using any other Cliffy functions.
#[wasm_bindgen]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Get the Cliffy version.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// Type alias for subscriber callbacks
type SubscriberFn = Rc<RefCell<Vec<(usize, Function)>>>;

/// A reactive value that can change over time.
///
/// `Behavior` represents a value that can be updated and observed.
/// Subscribers are notified whenever the value changes.
///
/// # JavaScript Example
///
/// ```javascript
/// const count = new Behavior(0);
///
/// // Subscribe to changes
/// const unsubscribe = count.subscribe(value => {
///     console.log('New value:', value);
/// });
///
/// // Update the value
/// count.update(n => n + 1);
///
/// // Get current value
/// console.log(count.sample()); // 1
///
/// // Clean up
/// unsubscribe();
/// ```
#[wasm_bindgen]
pub struct Behavior {
    value: Rc<RefCell<JsValue>>,
    subscribers: SubscriberFn,
    next_id: Rc<RefCell<usize>>,
}

#[wasm_bindgen]
impl Behavior {
    /// Create a new Behavior with an initial value.
    #[wasm_bindgen(constructor)]
    pub fn new(initial: JsValue) -> Behavior {
        Behavior {
            value: Rc::new(RefCell::new(initial)),
            subscribers: Rc::new(RefCell::new(Vec::new())),
            next_id: Rc::new(RefCell::new(0)),
        }
    }

    /// Get the current value.
    #[wasm_bindgen]
    pub fn sample(&self) -> JsValue {
        self.value.borrow().clone()
    }

    /// Set the value directly.
    #[wasm_bindgen]
    pub fn set(&self, value: JsValue) {
        *self.value.borrow_mut() = value;
        self.notify_subscribers();
    }

    /// Update the value using a transformation function.
    ///
    /// The function receives the current value and should return the new value.
    #[wasm_bindgen]
    pub fn update(&self, f: &Function) -> Result<(), JsValue> {
        let current = self.value.borrow().clone();
        let this = JsValue::null();
        let new_value = f.call1(&this, &current)?;
        *self.value.borrow_mut() = new_value;
        self.notify_subscribers();
        Ok(())
    }

    /// Subscribe to value changes.
    ///
    /// Returns a Subscription that can be used to unsubscribe.
    #[wasm_bindgen]
    pub fn subscribe(&self, callback: Function) -> Subscription {
        let id = {
            let mut next = self.next_id.borrow_mut();
            let id = *next;
            *next += 1;
            id
        };

        self.subscribers.borrow_mut().push((id, callback));

        Subscription {
            id,
            subscribers: Rc::clone(&self.subscribers),
        }
    }

    /// Create a derived Behavior by mapping a function over this one.
    ///
    /// The derived Behavior will automatically update when this Behavior changes.
    #[wasm_bindgen]
    pub fn map(&self, f: &Function) -> Result<Behavior, JsValue> {
        let current = self.value.borrow().clone();
        let this_js = JsValue::null();
        let initial = f.call1(&this_js, &current)?;

        let derived = Behavior::new(initial);

        // Subscribe to changes
        let derived_value = Rc::clone(&derived.value);
        let derived_subscribers = Rc::clone(&derived.subscribers);
        let f_clone = f.clone();

        let callback = Closure::wrap(Box::new(move |value: JsValue| {
            let this = JsValue::null();
            if let Ok(new_value) = f_clone.call1(&this, &value) {
                *derived_value.borrow_mut() = new_value;
                // Notify derived's subscribers
                for (_, cb) in derived_subscribers.borrow().iter() {
                    let _ = cb.call1(&this, &derived_value.borrow());
                }
            }
        }) as Box<dyn Fn(JsValue)>);

        // Get the Function from the closure
        let js_fn = callback.as_ref().unchecked_ref::<Function>().clone();
        callback.forget(); // Prevent the closure from being dropped

        self.subscribers.borrow_mut().push((
            {
                let mut next = self.next_id.borrow_mut();
                let id = *next;
                *next += 1;
                id
            },
            js_fn,
        ));

        Ok(derived)
    }

    /// Combine this Behavior with another using a function.
    #[wasm_bindgen]
    pub fn combine(&self, other: &Behavior, f: &Function) -> Result<Behavior, JsValue> {
        let a = self.value.borrow().clone();
        let b = other.value.borrow().clone();
        let this_js = JsValue::null();
        let initial = f.call2(&this_js, &a, &b)?;

        let combined = Behavior::new(initial);

        // Subscribe to self
        let combined_value = Rc::clone(&combined.value);
        let combined_subscribers = Rc::clone(&combined.subscribers);
        let other_value = Rc::clone(&other.value);
        let f_clone = f.clone();

        let callback_a = Closure::wrap(Box::new(move |a_val: JsValue| {
            let this = JsValue::null();
            let b_val = other_value.borrow().clone();
            if let Ok(new_value) = f_clone.call2(&this, &a_val, &b_val) {
                *combined_value.borrow_mut() = new_value;
                for (_, cb) in combined_subscribers.borrow().iter() {
                    let _ = cb.call1(&this, &combined_value.borrow());
                }
            }
        }) as Box<dyn Fn(JsValue)>);

        let js_fn_a = callback_a.as_ref().unchecked_ref::<Function>().clone();
        callback_a.forget();

        self.subscribers.borrow_mut().push((
            {
                let mut next = self.next_id.borrow_mut();
                let id = *next;
                *next += 1;
                id
            },
            js_fn_a,
        ));

        // Subscribe to other
        let combined_value = Rc::clone(&combined.value);
        let combined_subscribers = Rc::clone(&combined.subscribers);
        let self_value = Rc::clone(&self.value);
        let f_clone = f.clone();

        let callback_b = Closure::wrap(Box::new(move |b_val: JsValue| {
            let this = JsValue::null();
            let a_val = self_value.borrow().clone();
            if let Ok(new_value) = f_clone.call2(&this, &a_val, &b_val) {
                *combined_value.borrow_mut() = new_value;
                for (_, cb) in combined_subscribers.borrow().iter() {
                    let _ = cb.call1(&this, &combined_value.borrow());
                }
            }
        }) as Box<dyn Fn(JsValue)>);

        let js_fn_b = callback_b.as_ref().unchecked_ref::<Function>().clone();
        callback_b.forget();

        other.subscribers.borrow_mut().push((
            {
                let mut next = other.next_id.borrow_mut();
                let id = *next;
                *next += 1;
                id
            },
            js_fn_b,
        ));

        Ok(combined)
    }

    /// Project a value through this Behavior (used when this holds a boolean).
    ///
    /// When the condition is true, returns the value from value_fn.
    /// When false, returns null.
    ///
    /// This is the GA-inspired projection operator: projecting a value
    /// through a condition subspace.
    ///
    /// # JavaScript Example
    ///
    /// ```javascript
    /// const showMessage = behavior(true);
    /// const message = showMessage.project(() => "Hello!");
    ///
    /// console.log(message.sample()); // "Hello!"
    ///
    /// showMessage.set(false);
    /// console.log(message.sample()); // null
    /// ```
    #[wasm_bindgen]
    pub fn project(&self, value_fn: &Function) -> Result<Behavior, JsValue> {
        let cond_value = self.value.borrow().clone();
        let this_js = JsValue::null();

        let initial = if cond_value.is_truthy() {
            value_fn.call0(&this_js)?
        } else {
            JsValue::null()
        };

        let result = Behavior::new(initial);

        let result_value = Rc::clone(&result.value);
        let result_subscribers = Rc::clone(&result.subscribers);
        let value_fn_clone = value_fn.clone();

        let callback = Closure::wrap(Box::new(move |cond: JsValue| {
            let this = JsValue::null();
            let new_value = if cond.is_truthy() {
                value_fn_clone.call0(&this).unwrap_or(JsValue::null())
            } else {
                JsValue::null()
            };
            *result_value.borrow_mut() = new_value;
            for (_, cb) in result_subscribers.borrow().iter() {
                let _ = cb.call1(&this, &result_value.borrow());
            }
        }) as Box<dyn Fn(JsValue)>);

        let js_fn = callback.as_ref().unchecked_ref::<Function>().clone();
        callback.forget();

        self.subscribers.borrow_mut().push((
            {
                let mut next = self.next_id.borrow_mut();
                let id = *next;
                *next += 1;
                id
            },
            js_fn,
        ));

        Ok(result)
    }

    /// Select between two values based on this Behavior (used when this holds a boolean).
    ///
    /// When the condition is true, returns the value from then_fn.
    /// When false, returns the value from else_fn.
    ///
    /// This is the GA-inspired selection operator: selecting which subspace
    /// to project onto based on a condition.
    ///
    /// # JavaScript Example
    ///
    /// ```javascript
    /// const isDarkMode = behavior(false);
    /// const theme = isDarkMode.select(() => "dark", () => "light");
    ///
    /// console.log(theme.sample()); // "light"
    ///
    /// isDarkMode.set(true);
    /// console.log(theme.sample()); // "dark"
    /// ```
    #[wasm_bindgen]
    pub fn select(&self, then_fn: &Function, else_fn: &Function) -> Result<Behavior, JsValue> {
        let cond_value = self.value.borrow().clone();
        let this_js = JsValue::null();

        let initial = if cond_value.is_truthy() {
            then_fn.call0(&this_js)?
        } else {
            else_fn.call0(&this_js)?
        };

        let result = Behavior::new(initial);

        let result_value = Rc::clone(&result.value);
        let result_subscribers = Rc::clone(&result.subscribers);
        let then_fn_clone = then_fn.clone();
        let else_fn_clone = else_fn.clone();

        let callback = Closure::wrap(Box::new(move |cond: JsValue| {
            let this = JsValue::null();
            let new_value = if cond.is_truthy() {
                then_fn_clone.call0(&this).unwrap_or(JsValue::null())
            } else {
                else_fn_clone.call0(&this).unwrap_or(JsValue::null())
            };
            *result_value.borrow_mut() = new_value;
            for (_, cb) in result_subscribers.borrow().iter() {
                let _ = cb.call1(&this, &result_value.borrow());
            }
        }) as Box<dyn Fn(JsValue)>);

        let js_fn = callback.as_ref().unchecked_ref::<Function>().clone();
        callback.forget();

        self.subscribers.borrow_mut().push((
            {
                let mut next = self.next_id.borrow_mut();
                let id = *next;
                *next += 1;
                id
            },
            js_fn,
        ));

        Ok(result)
    }

    /// Notify all subscribers of the current value.
    fn notify_subscribers(&self) {
        let value = self.value.borrow().clone();
        let this = JsValue::null();
        for (_, callback) in self.subscribers.borrow().iter() {
            let _ = callback.call1(&this, &value);
        }
    }
}

/// A subscription handle that can be used to unsubscribe.
#[wasm_bindgen]
pub struct Subscription {
    id: usize,
    subscribers: SubscriberFn,
}

#[wasm_bindgen]
impl Subscription {
    /// Unsubscribe from the Behavior.
    #[wasm_bindgen]
    pub fn unsubscribe(&self) {
        self.subscribers
            .borrow_mut()
            .retain(|(id, _)| *id != self.id);
    }
}

/// A stream of discrete events.
///
/// `Event` represents a stream of values that occur at discrete moments.
/// Unlike Behavior, Event does not have a "current value" - it only
/// emits values when they occur.
///
/// # JavaScript Example
///
/// ```javascript
/// const clicks = new Event();
///
/// clicks.subscribe(event => {
///     console.log('Clicked!', event);
/// });
///
/// // Emit an event
/// clicks.emit({ x: 100, y: 200 });
/// ```
#[wasm_bindgen]
pub struct Event {
    subscribers: SubscriberFn,
    next_id: Rc<RefCell<usize>>,
}

#[wasm_bindgen]
impl Event {
    /// Create a new Event stream.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Event {
        Event {
            subscribers: Rc::new(RefCell::new(Vec::new())),
            next_id: Rc::new(RefCell::new(0)),
        }
    }

    /// Emit a value to all subscribers.
    #[wasm_bindgen]
    pub fn emit(&self, value: JsValue) {
        let this = JsValue::null();
        for (_, callback) in self.subscribers.borrow().iter() {
            let _ = callback.call1(&this, &value);
        }
    }

    /// Subscribe to this event stream.
    #[wasm_bindgen]
    pub fn subscribe(&self, callback: Function) -> Subscription {
        let id = {
            let mut next = self.next_id.borrow_mut();
            let id = *next;
            *next += 1;
            id
        };

        self.subscribers.borrow_mut().push((id, callback));

        Subscription {
            id,
            subscribers: Rc::clone(&self.subscribers),
        }
    }

    /// Map a function over this event stream.
    #[wasm_bindgen]
    pub fn map(&self, f: &Function) -> Event {
        let mapped = Event::new();

        let mapped_subscribers = Rc::clone(&mapped.subscribers);
        let f_clone = f.clone();

        let callback = Closure::wrap(Box::new(move |value: JsValue| {
            let this = JsValue::null();
            if let Ok(mapped_value) = f_clone.call1(&this, &value) {
                for (_, cb) in mapped_subscribers.borrow().iter() {
                    let _ = cb.call1(&this, &mapped_value);
                }
            }
        }) as Box<dyn Fn(JsValue)>);

        let js_fn = callback.as_ref().unchecked_ref::<Function>().clone();
        callback.forget();

        self.subscribers.borrow_mut().push((
            {
                let mut next = self.next_id.borrow_mut();
                let id = *next;
                *next += 1;
                id
            },
            js_fn,
        ));

        mapped
    }

    /// Filter events based on a predicate.
    #[wasm_bindgen]
    pub fn filter(&self, predicate: &Function) -> Event {
        let filtered = Event::new();

        let filtered_subscribers = Rc::clone(&filtered.subscribers);
        let predicate_clone = predicate.clone();

        let callback = Closure::wrap(Box::new(move |value: JsValue| {
            let this = JsValue::null();
            if let Ok(result) = predicate_clone.call1(&this, &value) {
                if result.is_truthy() {
                    for (_, cb) in filtered_subscribers.borrow().iter() {
                        let _ = cb.call1(&this, &value);
                    }
                }
            }
        }) as Box<dyn Fn(JsValue)>);

        let js_fn = callback.as_ref().unchecked_ref::<Function>().clone();
        callback.forget();

        self.subscribers.borrow_mut().push((
            {
                let mut next = self.next_id.borrow_mut();
                let id = *next;
                *next += 1;
                id
            },
            js_fn,
        ));

        filtered
    }

    /// Merge two event streams into one.
    #[wasm_bindgen]
    pub fn merge(&self, other: &Event) -> Event {
        let merged = Event::new();

        // Subscribe to self
        let merged_subscribers_1 = Rc::clone(&merged.subscribers);
        let callback_1 = Closure::wrap(Box::new(move |value: JsValue| {
            let this = JsValue::null();
            for (_, cb) in merged_subscribers_1.borrow().iter() {
                let _ = cb.call1(&this, &value);
            }
        }) as Box<dyn Fn(JsValue)>);

        let js_fn_1 = callback_1.as_ref().unchecked_ref::<Function>().clone();
        callback_1.forget();

        self.subscribers.borrow_mut().push((
            {
                let mut next = self.next_id.borrow_mut();
                let id = *next;
                *next += 1;
                id
            },
            js_fn_1,
        ));

        // Subscribe to other
        let merged_subscribers_2 = Rc::clone(&merged.subscribers);
        let callback_2 = Closure::wrap(Box::new(move |value: JsValue| {
            let this = JsValue::null();
            for (_, cb) in merged_subscribers_2.borrow().iter() {
                let _ = cb.call1(&this, &value);
            }
        }) as Box<dyn Fn(JsValue)>);

        let js_fn_2 = callback_2.as_ref().unchecked_ref::<Function>().clone();
        callback_2.forget();

        other.subscribers.borrow_mut().push((
            {
                let mut next = other.next_id.borrow_mut();
                let id = *next;
                *next += 1;
                id
            },
            js_fn_2,
        ));

        merged
    }

    /// Fold events into a Behavior, accumulating values.
    ///
    /// # Arguments
    ///
    /// * `initial` - The initial accumulated value
    /// * `f` - A function that takes (accumulator, event) and returns new accumulator
    #[wasm_bindgen]
    pub fn fold(&self, initial: JsValue, f: &Function) -> Result<Behavior, JsValue> {
        let behavior = Behavior::new(initial);

        let behavior_value = Rc::clone(&behavior.value);
        let behavior_subscribers = Rc::clone(&behavior.subscribers);
        let f_clone = f.clone();

        let callback = Closure::wrap(Box::new(move |event_value: JsValue| {
            let this = JsValue::null();
            let current = behavior_value.borrow().clone();
            if let Ok(new_value) = f_clone.call2(&this, &current, &event_value) {
                *behavior_value.borrow_mut() = new_value;
                for (_, cb) in behavior_subscribers.borrow().iter() {
                    let _ = cb.call1(&this, &behavior_value.borrow());
                }
            }
        }) as Box<dyn Fn(JsValue)>);

        let js_fn = callback.as_ref().unchecked_ref::<Function>().clone();
        callback.forget();

        self.subscribers.borrow_mut().push((
            {
                let mut next = self.next_id.borrow_mut();
                let id = *next;
                *next += 1;
                id
            },
            js_fn,
        ));

        Ok(behavior)
    }
}

impl Default for Event {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Combinators
// ============================================================================

/// **Deprecated**: Use `condition.project(value_fn)` instead.
///
/// Create a Behavior that holds a value only when a condition is true.
///
/// # JavaScript Example
///
/// ```javascript
/// // Old style (deprecated):
/// const message = when(showMessage, () => "Hello!");
///
/// // New style (preferred):
/// const message = showMessage.project(() => "Hello!");
/// ```
#[wasm_bindgen]
pub fn when(condition: &Behavior, then_fn: &Function) -> Result<Behavior, JsValue> {
    let cond_value = condition.value.borrow().clone();
    let this_js = JsValue::null();

    let initial = if cond_value.is_truthy() {
        then_fn.call0(&this_js)?
    } else {
        JsValue::null()
    };

    let result = Behavior::new(initial);

    // Subscribe to condition changes
    let result_value = Rc::clone(&result.value);
    let result_subscribers = Rc::clone(&result.subscribers);
    let then_fn_clone = then_fn.clone();

    let callback = Closure::wrap(Box::new(move |cond: JsValue| {
        let this = JsValue::null();
        let new_value = if cond.is_truthy() {
            then_fn_clone.call0(&this).unwrap_or(JsValue::null())
        } else {
            JsValue::null()
        };
        *result_value.borrow_mut() = new_value;
        for (_, cb) in result_subscribers.borrow().iter() {
            let _ = cb.call1(&this, &result_value.borrow());
        }
    }) as Box<dyn Fn(JsValue)>);

    let js_fn = callback.as_ref().unchecked_ref::<Function>().clone();
    callback.forget();

    condition.subscribers.borrow_mut().push((
        {
            let mut next = condition.next_id.borrow_mut();
            let id = *next;
            *next += 1;
            id
        },
        js_fn,
    ));

    Ok(result)
}

/// **Deprecated**: Use `condition.select(then_fn, else_fn)` instead.
///
/// Create a Behavior that selects between two values based on a condition.
///
/// # JavaScript Example
///
/// ```javascript
/// // Old style (deprecated):
/// const theme = ifElse(isDarkMode, () => "dark", () => "light");
///
/// // New style (preferred):
/// const theme = isDarkMode.select(() => "dark", () => "light");
/// ```
#[wasm_bindgen(js_name = ifElse)]
pub fn if_else(
    condition: &Behavior,
    then_fn: &Function,
    else_fn: &Function,
) -> Result<Behavior, JsValue> {
    let cond_value = condition.value.borrow().clone();
    let this_js = JsValue::null();

    let initial = if cond_value.is_truthy() {
        then_fn.call0(&this_js)?
    } else {
        else_fn.call0(&this_js)?
    };

    let result = Behavior::new(initial);

    // Subscribe to condition changes
    let result_value = Rc::clone(&result.value);
    let result_subscribers = Rc::clone(&result.subscribers);
    let then_fn_clone = then_fn.clone();
    let else_fn_clone = else_fn.clone();

    let callback = Closure::wrap(Box::new(move |cond: JsValue| {
        let this = JsValue::null();
        let new_value = if cond.is_truthy() {
            then_fn_clone.call0(&this).unwrap_or(JsValue::null())
        } else {
            else_fn_clone.call0(&this).unwrap_or(JsValue::null())
        };
        *result_value.borrow_mut() = new_value;
        for (_, cb) in result_subscribers.borrow().iter() {
            let _ = cb.call1(&this, &result_value.borrow());
        }
    }) as Box<dyn Fn(JsValue)>);

    let js_fn = callback.as_ref().unchecked_ref::<Function>().clone();
    callback.forget();

    condition.subscribers.borrow_mut().push((
        {
            let mut next = condition.next_id.borrow_mut();
            let id = *next;
            *next += 1;
            id
        },
        js_fn,
    ));

    Ok(result)
}

/// Combine two Behaviors into one using a function.
///
/// # Arguments
///
/// * `a` - First Behavior
/// * `b` - Second Behavior
/// * `f` - A function that takes both values and returns the combined value
///
/// # JavaScript Example
///
/// ```javascript
/// const width = new Behavior(100);
/// const height = new Behavior(50);
/// const area = combine(width, height, (w, h) => w * h);
///
/// console.log(area.sample()); // 5000
/// ```
#[wasm_bindgen]
pub fn combine(a: &Behavior, b: &Behavior, f: &Function) -> Result<Behavior, JsValue> {
    a.combine(b, f)
}

// For combining 3+ behaviors, use the wedge() function from TypeScript:
//
//   import { wedge } from '@cliffy-ga/core';
//
//   // Combine any number of behaviors with wedge().map()
//   const volume = wedge(width, height, depth).map((w, h, d) => w * h * d);
//   const isValid = wedge(a, b, c, d).map((a, b, c, d) => a && b && c && d);
//
// The wedge product (∧) from Clifford algebra creates a "Blade" - a higher-grade
// multivector that holds multiple values. The .map() method then projects the
// blade back to a scalar (a new Behavior).
//
// This pattern elegantly mirrors GA: vectors ∧ to form blades, then project to scalars.

// Note: `constant` was removed - use `behavior(value)` instead.
// A "constant" is just a Behavior you don't update.

/// Create a new Behavior (convenience function).
///
/// # JavaScript Example
///
/// ```javascript
/// const count = behavior(0);
/// count.update(n => n + 1);
/// ```
#[wasm_bindgen]
pub fn behavior(initial: JsValue) -> Behavior {
    Behavior::new(initial)
}

/// Create a new Event stream (convenience function).
///
/// # JavaScript Example
///
/// ```javascript
/// const clicks = event();
/// clicks.subscribe(e => console.log('Clicked!', e));
/// ```
#[wasm_bindgen]
pub fn event() -> Event {
    Event::new()
}

// ============================================================================
// Geometric Types
// ============================================================================

use cliffy_core::{
    GeometricState as CoreGeometricState, Rotor as CoreRotor, Transform as CoreTransform,
    Translation as CoreTranslation, Versor as CoreVersor,
};

/// A rotor represents a rotation in 3D space.
///
/// Use rotors for smooth, gimbal-lock-free rotations.
///
/// # JavaScript Example
///
/// ```javascript
/// // Create a 90-degree rotation in the XY plane (around Z axis)
/// const rot = Rotor.xy(Math.PI / 2);
///
/// // Apply to a geometric state
/// const pos = new GeometricState(1, 0, 0);
/// const rotated = pos.applyRotor(rot);
/// ```
#[wasm_bindgen]
pub struct Rotor {
    inner: CoreRotor,
}

#[wasm_bindgen]
impl Rotor {
    /// Create the identity rotor (no rotation).
    #[wasm_bindgen]
    pub fn identity() -> Rotor {
        Rotor {
            inner: CoreRotor::identity(),
        }
    }

    /// Create a rotation in the XY plane (around Z axis).
    ///
    /// # Arguments
    /// * `angle` - Rotation angle in radians
    #[wasm_bindgen]
    pub fn xy(angle: f64) -> Rotor {
        Rotor {
            inner: CoreRotor::xy(angle),
        }
    }

    /// Create a rotation in the XZ plane (around Y axis).
    ///
    /// # Arguments
    /// * `angle` - Rotation angle in radians
    #[wasm_bindgen]
    pub fn xz(angle: f64) -> Rotor {
        Rotor {
            inner: CoreRotor::xz(angle),
        }
    }

    /// Create a rotation in the YZ plane (around X axis).
    ///
    /// # Arguments
    /// * `angle` - Rotation angle in radians
    #[wasm_bindgen]
    pub fn yz(angle: f64) -> Rotor {
        Rotor {
            inner: CoreRotor::yz(angle),
        }
    }

    /// Create a rotation around an arbitrary axis.
    ///
    /// # Arguments
    /// * `x`, `y`, `z` - Axis vector components (doesn't need to be normalized)
    /// * `angle` - Rotation angle in radians
    #[wasm_bindgen(js_name = fromAxisAngle)]
    pub fn from_axis_angle(x: f64, y: f64, z: f64, angle: f64) -> Rotor {
        Rotor {
            inner: CoreRotor::from_axis_angle(x, y, z, angle),
        }
    }

    /// Get the rotation angle in radians.
    #[wasm_bindgen]
    pub fn angle(&self) -> f64 {
        self.inner.angle()
    }

    /// Compose this rotor with another (apply self, then other).
    #[wasm_bindgen]
    pub fn then(&self, other: &Rotor) -> Rotor {
        Rotor {
            inner: self.inner.then(&other.inner),
        }
    }

    /// Get the inverse rotor (reverse rotation).
    #[wasm_bindgen]
    pub fn inverse(&self) -> Rotor {
        Rotor {
            inner: self.inner.inverse(),
        }
    }

    /// Spherical linear interpolation from identity to this rotor.
    ///
    /// # Arguments
    /// * `t` - Interpolation factor (0 = identity, 1 = this rotor)
    #[wasm_bindgen]
    pub fn slerp(&self, t: f64) -> Rotor {
        Rotor {
            inner: self.inner.slerp(t),
        }
    }

    /// Spherical linear interpolation to another rotor.
    ///
    /// # Arguments
    /// * `other` - Target rotor
    /// * `t` - Interpolation factor (0 = this, 1 = other)
    #[wasm_bindgen(js_name = slerpTo)]
    pub fn slerp_to(&self, other: &Rotor, t: f64) -> Rotor {
        Rotor {
            inner: self.inner.slerp_to(&other.inner, t),
        }
    }

    /// Blend (interpolate) to another rotor.
    ///
    /// This is the unified interpolation method for all geometric types.
    /// For rotors, it uses spherical linear interpolation (slerp) to ensure
    /// smooth rotation along the shortest path.
    ///
    /// # Arguments
    /// * `other` - Target rotor
    /// * `t` - Interpolation factor (0 = this, 1 = other)
    ///
    /// # JavaScript Example
    ///
    /// ```javascript
    /// const start = Rotor.identity();
    /// const end = Rotor.xy(Math.PI / 2);
    /// const mid = start.blend(end, 0.5);  // 45-degree rotation
    /// ```
    #[wasm_bindgen]
    pub fn blend(&self, other: &Rotor, t: f64) -> Rotor {
        self.slerp_to(other, t)
    }
}

/// A versor represents a general geometric transformation (rotation, reflection, or composition).
///
/// Versors generalize rotors to include reflections. An even versor (product of an even number
/// of vectors) is a rotor. An odd versor includes a reflection component.
///
/// # JavaScript Example
///
/// ```javascript
/// // Create a reflection through the XY plane
/// const reflect = Versor.reflection(0, 0, 1);
///
/// // Convert a rotor to a versor
/// const rot = Rotor.xy(Math.PI / 2);
/// const versor = Versor.fromRotor(rot);
///
/// // Check if it's a pure rotation
/// console.log(versor.isRotor()); // true
/// ```
#[wasm_bindgen]
pub struct Versor {
    inner: CoreVersor,
}

#[wasm_bindgen]
impl Versor {
    /// Create the identity versor (no transformation).
    #[wasm_bindgen]
    pub fn identity() -> Versor {
        Versor {
            inner: CoreVersor::identity(),
        }
    }

    /// Create a reflection through a plane with the given normal vector.
    ///
    /// The plane passes through the origin with normal (x, y, z).
    /// The normal doesn't need to be normalized.
    ///
    /// # Arguments
    /// * `x`, `y`, `z` - Normal vector components
    #[wasm_bindgen]
    pub fn reflection(x: f64, y: f64, z: f64) -> Versor {
        Versor {
            inner: CoreVersor::reflection(x, y, z),
        }
    }

    /// Create a versor from a rotor.
    ///
    /// This converts a pure rotation into a versor representation.
    #[wasm_bindgen(js_name = fromRotor)]
    pub fn from_rotor(rotor: &Rotor) -> Versor {
        Versor {
            inner: CoreVersor::from_rotor(rotor.inner.clone()),
        }
    }

    /// Compose this versor with another (apply self, then other).
    #[wasm_bindgen]
    pub fn then(&self, other: &Versor) -> Versor {
        Versor {
            inner: self.inner.then(&other.inner),
        }
    }

    /// Check if this is an even versor (a rotor, i.e., pure rotation).
    #[wasm_bindgen(js_name = isRotor)]
    pub fn is_rotor(&self) -> bool {
        self.inner.is_rotor()
    }

    /// Try to convert to a Rotor.
    ///
    /// Returns null if this is an odd versor (includes reflection).
    #[wasm_bindgen(js_name = toRotor)]
    pub fn to_rotor(&self) -> Option<Rotor> {
        self.inner.to_rotor().map(|r| Rotor { inner: r })
    }
}

/// A translation in 3D space.
///
/// # JavaScript Example
///
/// ```javascript
/// const trans = new Translation(10, 0, 0);
/// const pos = new GeometricState(0, 0, 0);
/// const moved = pos.applyTranslation(trans);
/// ```
#[wasm_bindgen]
pub struct Translation {
    inner: CoreTranslation,
}

#[wasm_bindgen]
impl Translation {
    /// Create a new translation.
    #[wasm_bindgen(constructor)]
    pub fn new(x: f64, y: f64, z: f64) -> Translation {
        Translation {
            inner: CoreTranslation::new(x, y, z),
        }
    }

    /// Create a translation along the X axis.
    #[wasm_bindgen]
    pub fn x(amount: f64) -> Translation {
        Translation {
            inner: CoreTranslation::x(amount),
        }
    }

    /// Create a translation along the Y axis.
    #[wasm_bindgen]
    pub fn y(amount: f64) -> Translation {
        Translation {
            inner: CoreTranslation::y(amount),
        }
    }

    /// Create a translation along the Z axis.
    #[wasm_bindgen]
    pub fn z(amount: f64) -> Translation {
        Translation {
            inner: CoreTranslation::z(amount),
        }
    }

    /// Compose this translation with another.
    #[wasm_bindgen]
    pub fn then(&self, other: &Translation) -> Translation {
        Translation {
            inner: self.inner.then(&other.inner),
        }
    }

    /// Get the inverse translation.
    #[wasm_bindgen]
    pub fn inverse(&self) -> Translation {
        Translation {
            inner: self.inner.inverse(),
        }
    }

    /// Linear interpolation from zero to this translation.
    #[wasm_bindgen]
    pub fn lerp(&self, t: f64) -> Translation {
        Translation {
            inner: self.inner.lerp(t),
        }
    }

    /// Linear interpolation to another translation.
    #[wasm_bindgen(js_name = lerpTo)]
    pub fn lerp_to(&self, other: &Translation, t: f64) -> Translation {
        Translation {
            inner: self.inner.lerp_to(&other.inner, t),
        }
    }

    /// Blend (interpolate) to another translation.
    ///
    /// This is the unified interpolation method for all geometric types.
    /// For translations, it uses linear interpolation.
    ///
    /// # Arguments
    /// * `other` - Target translation
    /// * `t` - Interpolation factor (0 = this, 1 = other)
    ///
    /// # JavaScript Example
    ///
    /// ```javascript
    /// const start = new Translation(0, 0, 0);
    /// const end = new Translation(10, 0, 0);
    /// const mid = start.blend(end, 0.5);  // (5, 0, 0)
    /// ```
    #[wasm_bindgen]
    pub fn blend(&self, other: &Translation, t: f64) -> Translation {
        self.lerp_to(other, t)
    }
}

/// A combined rotation and translation transform.
///
/// # JavaScript Example
///
/// ```javascript
/// const rot = Rotor.xy(Math.PI / 4);
/// const trans = new Translation(10, 0, 0);
/// const transform = Transform.fromRotorAndTranslation(rot, trans);
///
/// const pos = new GeometricState(1, 0, 0);
/// const result = pos.applyTransform(transform);
/// ```
#[wasm_bindgen]
pub struct Transform {
    inner: CoreTransform,
}

#[wasm_bindgen]
impl Transform {
    /// Create an identity transform (no rotation or translation).
    #[wasm_bindgen]
    pub fn identity() -> Transform {
        Transform {
            inner: CoreTransform::identity(),
        }
    }

    /// Create a transform from a rotor and translation.
    #[wasm_bindgen(js_name = fromRotorAndTranslation)]
    pub fn from_rotor_and_translation(rotor: &Rotor, translation: &Translation) -> Transform {
        Transform {
            inner: CoreTransform::new(rotor.inner.clone(), translation.inner.clone()),
        }
    }

    /// Create a pure rotation transform.
    #[wasm_bindgen(js_name = fromRotor)]
    pub fn from_rotor(rotor: &Rotor) -> Transform {
        Transform {
            inner: CoreTransform::rotation(rotor.inner.clone()),
        }
    }

    /// Create a pure translation transform.
    #[wasm_bindgen(js_name = fromTranslation)]
    pub fn from_translation(translation: &Translation) -> Transform {
        Transform {
            inner: CoreTransform::translation(translation.inner.clone()),
        }
    }

    /// Compose this transform with another (apply self, then other).
    #[wasm_bindgen]
    pub fn then(&self, other: &Transform) -> Transform {
        Transform {
            inner: self.inner.then(&other.inner),
        }
    }

    /// Get the inverse transform.
    #[wasm_bindgen]
    pub fn inverse(&self) -> Transform {
        Transform {
            inner: self.inner.inverse(),
        }
    }

    /// Interpolate from identity to this transform.
    #[wasm_bindgen]
    pub fn interpolate(&self, t: f64) -> Transform {
        Transform {
            inner: self.inner.interpolate(t),
        }
    }

    /// Interpolate to another transform.
    #[wasm_bindgen(js_name = interpolateTo)]
    pub fn interpolate_to(&self, other: &Transform, t: f64) -> Transform {
        Transform {
            inner: self.inner.interpolate_to(&other.inner, t),
        }
    }

    /// Blend (interpolate) to another transform.
    ///
    /// This is the unified interpolation method for all geometric types.
    /// For transforms, it uses slerp for rotation and lerp for translation.
    ///
    /// # Arguments
    /// * `other` - Target transform
    /// * `t` - Interpolation factor (0 = this, 1 = other)
    ///
    /// # JavaScript Example
    ///
    /// ```javascript
    /// const start = Transform.identity();
    /// const end = Transform.fromRotorAndTranslation(rot, trans);
    /// const mid = start.blend(end, 0.5);
    /// ```
    #[wasm_bindgen]
    pub fn blend(&self, other: &Transform, t: f64) -> Transform {
        self.interpolate_to(other, t)
    }
}

/// Geometric state with explicit transformation support.
///
/// Unlike regular Behavior which hides the geometric algebra,
/// GeometricState exposes it for animation, physics, and advanced use cases.
///
/// # JavaScript Example
///
/// ```javascript
/// // Create a position
/// const pos = GeometricState.fromVector(1, 0, 0);
///
/// // Apply transformations
/// const rot = Rotor.xy(Math.PI / 2);
/// const rotated = pos.applyRotor(rot);
///
/// // Get the result
/// const [x, y, z] = rotated.asVector();
/// console.log(x, y, z); // ~0, ~1, ~0
///
/// // Subscribe to changes
/// pos.subscribe(mv => console.log('State changed'));
/// ```
#[wasm_bindgen]
pub struct GeometricState {
    inner: CoreGeometricState,
}

#[wasm_bindgen]
impl GeometricState {
    /// Create a geometric state from a scalar value.
    #[wasm_bindgen(js_name = fromScalar)]
    pub fn from_scalar(value: f64) -> GeometricState {
        GeometricState {
            inner: CoreGeometricState::from_scalar(value),
        }
    }

    /// Create a geometric state from a 3D vector.
    #[wasm_bindgen(js_name = fromVector)]
    pub fn from_vector(x: f64, y: f64, z: f64) -> GeometricState {
        GeometricState {
            inner: CoreGeometricState::from_vector(x, y, z),
        }
    }

    /// Create the zero state.
    #[wasm_bindgen]
    pub fn zero() -> GeometricState {
        GeometricState {
            inner: CoreGeometricState::zero(),
        }
    }

    /// Get the scalar component.
    #[wasm_bindgen]
    pub fn scalar(&self) -> f64 {
        self.inner.scalar()
    }

    /// Get the vector components as an array [x, y, z].
    #[wasm_bindgen(js_name = asVector)]
    pub fn as_vector(&self) -> Vec<f64> {
        let (x, y, z) = self.inner.as_vector();
        vec![x, y, z]
    }

    /// Get the magnitude (length) of the state.
    #[wasm_bindgen]
    pub fn magnitude(&self) -> f64 {
        self.inner.magnitude()
    }

    /// Set the state to a scalar value.
    #[wasm_bindgen(js_name = setScalar)]
    pub fn set_scalar(&self, value: f64) {
        self.inner.set_scalar(value);
    }

    /// Set the state to a vector value.
    #[wasm_bindgen(js_name = setVector)]
    pub fn set_vector(&self, x: f64, y: f64, z: f64) {
        self.inner.set_vector(x, y, z);
    }

    /// Apply a rotor transformation (returns new state).
    #[wasm_bindgen(js_name = applyRotor)]
    pub fn apply_rotor(&self, rotor: &Rotor) -> GeometricState {
        GeometricState {
            inner: self.inner.apply_rotor(&rotor.inner),
        }
    }

    /// Apply a translation (returns new state).
    #[wasm_bindgen(js_name = applyTranslation)]
    pub fn apply_translation(&self, translation: &Translation) -> GeometricState {
        GeometricState {
            inner: self.inner.apply_translation(&translation.inner),
        }
    }

    /// Apply a transform (returns new state).
    #[wasm_bindgen(js_name = applyTransform)]
    pub fn apply_transform(&self, transform: &Transform) -> GeometricState {
        GeometricState {
            inner: self.inner.apply_transform(&transform.inner),
        }
    }

    /// Scale the state by a factor (returns new state).
    #[wasm_bindgen]
    pub fn scale(&self, factor: f64) -> GeometricState {
        GeometricState {
            inner: self.inner.scale(factor),
        }
    }

    /// Normalize to unit magnitude (returns new state).
    #[wasm_bindgen]
    pub fn normalize(&self) -> Option<GeometricState> {
        self.inner.normalize().map(|inner| GeometricState { inner })
    }

    /// Linear interpolation to another state.
    #[wasm_bindgen]
    pub fn lerp(&self, other: &GeometricState, t: f64) -> GeometricState {
        GeometricState {
            inner: self.inner.lerp(&other.inner, t),
        }
    }

    /// Spherical linear interpolation (for rotor-like states).
    #[wasm_bindgen]
    pub fn slerp(&self, other: &GeometricState, t: f64) -> GeometricState {
        GeometricState {
            inner: self.inner.slerp(&other.inner, t),
        }
    }

    /// Blend (interpolate) to another geometric state.
    ///
    /// This is the unified interpolation method for all geometric types.
    /// For GeometricState, it uses linear interpolation (lerp).
    /// Use slerp() explicitly for rotor-like states that need spherical interpolation.
    ///
    /// # Arguments
    /// * `other` - Target state
    /// * `t` - Interpolation factor (0 = this, 1 = other)
    ///
    /// # JavaScript Example
    ///
    /// ```javascript
    /// const start = GeometricState.fromVector(0, 0, 0);
    /// const end = GeometricState.fromVector(10, 0, 0);
    /// const mid = start.blend(end, 0.5);  // (5, 0, 0)
    /// ```
    #[wasm_bindgen]
    pub fn blend(&self, other: &GeometricState, t: f64) -> GeometricState {
        self.lerp(other, t)
    }

    /// Get state as coefficients array for JS interop.
    ///
    /// Returns a Float64Array with 8 coefficients representing the multivector.
    #[wasm_bindgen(js_name = toArray)]
    pub fn to_array(&self) -> js_sys::Float64Array {
        let mv = self.inner.multivector();
        let coeffs: Vec<f64> = (0..8).map(|i| mv.get(i)).collect();
        js_sys::Float64Array::from(coeffs.as_slice())
    }
}

#[cfg(test)]
mod tests {
    // WASM tests go in tests/wasm_tests.rs
}
