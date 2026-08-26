// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Behavior - Time-varying values backed by geometric algebra
//!
//! A `Behavior<T>` represents a value that can change over time.
//! Internally, all values are stored as GA3 multivectors, enabling
//! geometric transformations as the foundation for state updates.
//!
//! This is inspired by Conal Elliott's classical FRP semantics where
//! `Behavior a = Time -> a` - a function from time to a value.

use crate::geometric::{FromGeometric, IntoGeometric, GA3};
use std::cell::RefCell;
use std::rc::Rc;

/// A subscription handle that can be used to unsubscribe
#[allow(clippy::type_complexity)]
pub struct Subscription {
    #[allow(dead_code)] // Reserved for debugging
    id: usize,
    unsubscribe: Rc<RefCell<Option<Box<dyn Fn()>>>>,
}

impl Subscription {
    /// Unsubscribe from updates
    pub fn unsubscribe(self) {
        if let Some(unsub) = self.unsubscribe.borrow_mut().take() {
            unsub();
        }
    }
}

/// A time-varying value backed by geometric algebra
///
/// `Behavior<T>` wraps any value type `T` and stores it internally as a
/// GA3 multivector. This enables:
///
/// - Reactive updates with subscriber notification
/// - Derived behaviors via `map` and `combine`
/// - Geometric transformations under the hood
///
/// # Example
///
/// ```rust
/// use cliffy_core::{behavior, Behavior};
///
/// let count = behavior(0);
/// assert_eq!(count.sample(), 0);
///
/// count.update(|n| n + 1);
/// assert_eq!(count.sample(), 1);
/// ```
#[allow(clippy::type_complexity)]
pub struct Behavior<T> {
    /// Internal geometric state
    state: Rc<RefCell<GA3>>,

    /// Cached value (for types that can't be reconstructed from GA3)
    cache: Rc<RefCell<T>>,

    /// Subscribers to notify on update
    subscribers: Rc<RefCell<Vec<(usize, Box<dyn Fn(&T)>)>>>,

    /// Next subscriber ID
    next_id: Rc<RefCell<usize>>,
}

impl<T> Clone for Behavior<T> {
    fn clone(&self) -> Self {
        Self {
            state: Rc::clone(&self.state),
            cache: Rc::clone(&self.cache),
            subscribers: Rc::clone(&self.subscribers),
            next_id: Rc::clone(&self.next_id),
        }
    }
}

impl<T: IntoGeometric + FromGeometric + Clone + 'static> Behavior<T> {
    /// Create a new behavior with an initial value
    pub fn new(initial: T) -> Self {
        let mv = initial.clone().into_geometric();
        Self {
            state: Rc::new(RefCell::new(mv)),
            cache: Rc::new(RefCell::new(initial)),
            subscribers: Rc::new(RefCell::new(Vec::new())),
            next_id: Rc::new(RefCell::new(0)),
        }
    }

    /// Sample the current value
    pub fn sample(&self) -> T {
        self.cache.borrow().clone()
    }

    /// Update the value using a transformation function
    ///
    /// The function receives the current value and returns the new value.
    /// All subscribers are notified after the update.
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(T) -> T,
    {
        let current = self.cache.borrow().clone();
        let new_value = f(current);
        let new_mv = new_value.clone().into_geometric();

        *self.state.borrow_mut() = new_mv;
        *self.cache.borrow_mut() = new_value;

        // Notify subscribers
        let cache = self.cache.borrow();
        for (_, callback) in self.subscribers.borrow().iter() {
            callback(&cache);
        }
    }

    /// Set the value directly
    pub fn set(&self, value: T) {
        self.update(|_| value);
    }

    /// Subscribe to value changes
    ///
    /// Returns a `Subscription` that can be used to unsubscribe.
    pub fn subscribe<F>(&self, callback: F) -> Subscription
    where
        F: Fn(&T) + 'static,
    {
        let id = {
            let mut next = self.next_id.borrow_mut();
            let id = *next;
            *next += 1;
            id
        };

        self.subscribers.borrow_mut().push((id, Box::new(callback)));

        let subscribers = Rc::clone(&self.subscribers);
        let unsubscribe = Rc::new(RefCell::new(Some(Box::new(move || {
            subscribers.borrow_mut().retain(|(i, _)| *i != id);
        }) as Box<dyn Fn()>)));

        Subscription { id, unsubscribe }
    }

    /// Create a derived behavior by mapping a function over this behavior
    ///
    /// The derived behavior will automatically update when this behavior changes.
    pub fn map<U, F>(&self, f: F) -> Behavior<U>
    where
        U: IntoGeometric + FromGeometric + Clone + 'static,
        F: Fn(T) -> U + 'static,
    {
        let initial = f(self.sample());
        let derived = Behavior::new(initial);

        // Subscribe to changes and update derived
        let derived_clone = derived.clone();
        self.subscribe(move |value| {
            let new_value = f(value.clone());
            derived_clone.set(new_value);
        });

        derived
    }

    /// Combine two behaviors into a new behavior
    pub fn combine<U, V, F>(&self, other: &Behavior<U>, f: F) -> Behavior<V>
    where
        U: IntoGeometric + FromGeometric + Clone + 'static,
        V: IntoGeometric + FromGeometric + Clone + 'static,
        F: Fn(T, U) -> V + Clone + 'static,
    {
        let initial = f(self.sample(), other.sample());
        let combined = Behavior::new(initial);

        // Subscribe to self
        let combined_clone = combined.clone();
        let other_clone = other.clone();
        let f_clone = f.clone();
        self.subscribe(move |a| {
            let b = other_clone.sample();
            combined_clone.set(f_clone(a.clone(), b));
        });

        // Subscribe to other
        let combined_clone = combined.clone();
        let self_clone = self.clone();
        other.subscribe(move |b| {
            let a = self_clone.sample();
            combined_clone.set(f(a, b.clone()));
        });

        combined
    }

    /// Get the internal geometric state (for advanced users)
    pub fn geometric_state(&self) -> GA3 {
        self.state.borrow().clone()
    }

    /// Apply a geometric transformation directly (for advanced users)
    pub fn apply_geometric<F>(&self, transform: F)
    where
        F: FnOnce(&GA3) -> GA3,
    {
        let current = self.state.borrow().clone();
        let new_mv = transform(&current);
        *self.state.borrow_mut() = new_mv.clone();
        *self.cache.borrow_mut() = T::from_geometric(&new_mv);

        // Notify subscribers
        let cache = self.cache.borrow();
        for (_, callback) in self.subscribers.borrow().iter() {
            callback(&cache);
        }
    }
}

/// Convenience function to create a behavior
///
/// # Example
///
/// ```rust
/// use cliffy_core::behavior;
///
/// let count = behavior(0);
/// let name = behavior("Alice".to_string());
/// let active = behavior(true);
/// ```
pub fn behavior<T: IntoGeometric + FromGeometric + Clone + 'static>(initial: T) -> Behavior<T> {
    Behavior::new(initial)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn test_behavior_new_and_sample() {
        let b = behavior(42i32);
        assert_eq!(b.sample(), 42);
    }

    #[test]
    fn test_behavior_update() {
        let b = behavior(0i32);
        b.update(|n| n + 1);
        assert_eq!(b.sample(), 1);

        b.update(|n| n * 2);
        assert_eq!(b.sample(), 2);
    }

    #[test]
    fn test_behavior_set() {
        let b = behavior(0i32);
        b.set(100);
        assert_eq!(b.sample(), 100);
    }

    #[test]
    fn test_behavior_subscribe() {
        let b = behavior(0i32);
        let called = Rc::new(Cell::new(0));
        let called_clone = Rc::clone(&called);

        let _sub = b.subscribe(move |_value| {
            called_clone.set(called_clone.get() + 1);
        });

        b.update(|n| n + 1);
        assert_eq!(called.get(), 1);

        b.update(|n| n + 1);
        assert_eq!(called.get(), 2);
    }

    #[test]
    fn test_behavior_unsubscribe() {
        let b = behavior(0i32);
        let called = Rc::new(Cell::new(0));
        let called_clone = Rc::clone(&called);

        let sub = b.subscribe(move |_value| {
            called_clone.set(called_clone.get() + 1);
        });

        b.update(|n| n + 1);
        assert_eq!(called.get(), 1);

        sub.unsubscribe();

        b.update(|n| n + 1);
        assert_eq!(called.get(), 1); // Should not increase
    }

    #[test]
    fn test_behavior_map() {
        let count = behavior(5i32);
        let doubled = count.map(|n| n * 2);

        assert_eq!(doubled.sample(), 10);

        count.update(|n| n + 1);
        assert_eq!(doubled.sample(), 12);
    }

    #[test]
    fn test_behavior_combine() {
        let a = behavior(10i32);
        let b = behavior(20i32);
        let sum = a.combine(&b, |x, y| x + y);

        assert_eq!(sum.sample(), 30);

        a.update(|n| n + 5);
        assert_eq!(sum.sample(), 35);

        b.update(|n| n + 10);
        assert_eq!(sum.sample(), 45);
    }

    #[test]
    fn test_behavior_with_string() {
        let name = behavior("Alice".to_string());
        assert_eq!(name.sample(), "Alice");

        name.set("Bob".to_string());
        assert_eq!(name.sample(), "Bob");
    }

    #[test]
    fn test_behavior_with_bool() {
        let active = behavior(false);
        assert!(!active.sample());

        active.update(|_| true);
        assert!(active.sample());
    }

    #[test]
    fn test_behavior_clone_shares_state() {
        let a = behavior(0i32);
        let b = a.clone();

        a.update(|n| n + 1);
        assert_eq!(b.sample(), 1);
    }
}
