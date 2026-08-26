// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Event - Discrete occurrences with geometric transformations
//!
//! An `Event<T>` represents a stream of discrete occurrences over time.
//! Each occurrence carries a value of type `T` and can be transformed
//! or combined with other events.
//!
//! This is inspired by Conal Elliott's classical FRP semantics where
//! `Event a = [(Time, a)]` - a list of time-value pairs.

use crate::geometric::{FromGeometric, IntoGeometric, GA3};
use std::cell::RefCell;
use std::rc::Rc;

/// An event occurrence with a value
#[derive(Debug, Clone)]
pub struct Occurrence<T> {
    /// The value of this occurrence
    pub value: T,
    /// Timestamp (relative to behavior creation)
    pub timestamp: f64,
}

/// A stream of discrete occurrences
///
/// `Event<T>` allows you to react to discrete happenings like clicks,
/// keypresses, or timer ticks. Events can be mapped, filtered, and
/// merged with other events.
///
/// # Example
///
/// ```rust
/// use cliffy_core::{event, Event};
///
/// let clicks = event::<()>();
///
/// clicks.subscribe(|_| {
///     println!("Clicked!");
/// });
///
/// clicks.emit(());
/// ```
#[allow(clippy::type_complexity)]
pub struct Event<T> {
    /// Subscribers to notify on occurrence
    subscribers: Rc<RefCell<Vec<(usize, Box<dyn Fn(&Occurrence<T>)>)>>>,

    /// Next subscriber ID
    next_id: Rc<RefCell<usize>>,

    /// Start time for timestamp calculation
    start_time: std::time::Instant,

    /// Optional geometric transform applied to each event
    transform: Option<GA3>,
}

impl<T> Clone for Event<T> {
    fn clone(&self) -> Self {
        Self {
            subscribers: Rc::clone(&self.subscribers),
            next_id: Rc::clone(&self.next_id),
            start_time: self.start_time,
            transform: self.transform.clone(),
        }
    }
}

impl<T: Clone + 'static> Event<T> {
    /// Create a new event stream
    #[must_use]
    pub fn new() -> Self {
        Self {
            subscribers: Rc::new(RefCell::new(Vec::new())),
            next_id: Rc::new(RefCell::new(0)),
            start_time: std::time::Instant::now(),
            transform: None,
        }
    }

    /// Emit a value to all subscribers
    pub fn emit(&self, value: T) {
        let occurrence = Occurrence {
            value,
            timestamp: self.start_time.elapsed().as_secs_f64(),
        };

        for (_, callback) in self.subscribers.borrow().iter() {
            callback(&occurrence);
        }
    }

    /// Subscribe to this event stream
    pub fn subscribe<F>(&self, callback: F) -> EventSubscription
    where
        F: Fn(&T) + 'static,
    {
        let id = {
            let mut next = self.next_id.borrow_mut();
            let id = *next;
            *next = next.saturating_add(1);
            id
        };

        self.subscribers
            .borrow_mut()
            .push((id, Box::new(move |occ| callback(&occ.value))));

        let subscribers = Rc::clone(&self.subscribers);
        EventSubscription {
            id,
            unsubscribe: Rc::new(RefCell::new(Some(Box::new(move || {
                subscribers.borrow_mut().retain(|(i, _)| *i != id);
            })))),
        }
    }

    /// Map a function over this event stream
    pub fn map<U, F>(&self, f: F) -> Event<U>
    where
        U: Clone + 'static,
        F: Fn(T) -> U + 'static,
    {
        let mapped = Event::<U>::new();

        let mapped_clone = mapped.clone();
        self.subscribe(move |value| {
            mapped_clone.emit(f(value.clone()));
        });

        mapped
    }

    /// Filter events based on a predicate
    #[must_use]
    pub fn filter<F>(&self, predicate: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        let filtered = Self::new();

        let filtered_clone = filtered.clone();
        self.subscribe(move |value| {
            if predicate(value) {
                filtered_clone.emit(value.clone());
            }
        });

        filtered
    }

    /// Merge two event streams
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        let merged = Self::new();

        let merged_clone = merged.clone();
        self.subscribe(move |value| {
            merged_clone.emit(value.clone());
        });

        let merged_clone = merged.clone();
        other.subscribe(move |value| {
            merged_clone.emit(value.clone());
        });

        merged
    }

    /// Fold events into a behavior, accumulating values
    pub fn fold<S, F>(&self, initial: S, f: F) -> crate::Behavior<S>
    where
        S: IntoGeometric + FromGeometric + Clone + 'static,
        F: Fn(S, T) -> S + 'static,
    {
        let behavior = crate::behavior(initial);

        let behavior_clone = behavior.clone();
        self.subscribe(move |value| {
            behavior_clone.update(|state| f(state, value.clone()));
        });

        behavior
    }
}

impl<T: Clone + 'static> Default for Event<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// A subscription handle for events
#[allow(clippy::type_complexity)]
pub struct EventSubscription {
    #[allow(dead_code)] // Reserved for debugging
    id: usize,
    unsubscribe: Rc<RefCell<Option<Box<dyn Fn()>>>>,
}

impl EventSubscription {
    /// Unsubscribe from the event stream
    pub fn unsubscribe(self) {
        if let Some(unsub) = self.unsubscribe.borrow_mut().take() {
            unsub();
        }
    }
}

/// Convenience function to create an event stream
///
/// # Example
///
/// ```rust
/// use cliffy_core::event;
///
/// let clicks = event::<()>();
/// let key_presses = event::<char>();
/// let values = event::<i32>();
/// ```
#[must_use]
pub fn event<T: Clone + 'static>() -> Event<T> {
    Event::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn test_event_new_and_emit() {
        let evt = event::<i32>();
        let received = Rc::new(Cell::new(0));
        let received_clone = Rc::clone(&received);

        evt.subscribe(move |value| {
            received_clone.set(*value);
        });

        evt.emit(42);
        assert_eq!(received.get(), 42);
    }

    #[test]
    fn test_event_multiple_subscribers() {
        let evt = event::<i32>();
        let count = Rc::new(Cell::new(0));

        let count1 = Rc::clone(&count);
        evt.subscribe(move |_| count1.set(count1.get() + 1));

        let count2 = Rc::clone(&count);
        evt.subscribe(move |_| count2.set(count2.get() + 1));

        evt.emit(1);
        assert_eq!(count.get(), 2);
    }

    #[test]
    fn test_event_unsubscribe() {
        let evt = event::<i32>();
        let count = Rc::new(Cell::new(0));
        let count_clone = Rc::clone(&count);

        let sub = evt.subscribe(move |_| {
            count_clone.set(count_clone.get() + 1);
        });

        evt.emit(1);
        assert_eq!(count.get(), 1);

        sub.unsubscribe();

        evt.emit(2);
        assert_eq!(count.get(), 1); // Should not increase
    }

    #[test]
    fn test_event_map() {
        let evt = event::<i32>();
        let doubled = evt.map(|n| n * 2);

        let received = Rc::new(Cell::new(0));
        let received_clone = Rc::clone(&received);

        doubled.subscribe(move |value| {
            received_clone.set(*value);
        });

        evt.emit(5);
        assert_eq!(received.get(), 10);
    }

    #[test]
    fn test_event_filter() {
        let evt = event::<i32>();
        let evens = evt.filter(|n| n % 2 == 0);

        let received = Rc::new(RefCell::new(Vec::new()));
        let received_clone = Rc::clone(&received);

        evens.subscribe(move |value| {
            received_clone.borrow_mut().push(*value);
        });

        evt.emit(1);
        evt.emit(2);
        evt.emit(3);
        evt.emit(4);

        assert_eq!(*received.borrow(), vec![2, 4]);
    }

    #[test]
    fn test_event_merge() {
        let evt1 = event::<i32>();
        let evt2 = event::<i32>();
        let merged = evt1.merge(&evt2);

        let received = Rc::new(RefCell::new(Vec::new()));
        let received_clone = Rc::clone(&received);

        merged.subscribe(move |value| {
            received_clone.borrow_mut().push(*value);
        });

        evt1.emit(1);
        evt2.emit(2);
        evt1.emit(3);

        assert_eq!(*received.borrow(), vec![1, 2, 3]);
    }

    #[test]
    fn test_event_fold() {
        let clicks = event::<()>();
        let count = clicks.fold(0i32, |n, ()| n + 1);

        assert_eq!(count.sample(), 0);

        clicks.emit(());
        assert_eq!(count.sample(), 1);

        clicks.emit(());
        clicks.emit(());
        assert_eq!(count.sample(), 3);
    }
}
