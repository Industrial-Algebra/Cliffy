// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Combinators for composing behaviors
//!
//! These combinators provide algebraic ways to compose reactive values,
//! inspired by classical FRP semantics. They replace imperative control
//! flow with declarative compositions.

use crate::behavior::{behavior, Behavior};
use crate::geometric::{FromGeometric, IntoGeometric};

/// Conditional combinator - select between values based on a condition
///
/// Creates a behavior that follows the value of `then_value` when `condition`
/// is true, otherwise it holds `None`.
///
/// # Example
///
/// ```rust
/// use cliffy_core::{behavior, when};
///
/// let show_message = behavior(true);
/// let message = when(&show_message, || "Hello!".to_string());
///
/// assert_eq!(message.sample(), Some("Hello!".to_string()));
///
/// show_message.set(false);
/// assert_eq!(message.sample(), None);
/// ```
pub fn when<T, F>(condition: &Behavior<bool>, then_value: F) -> Behavior<Option<T>>
where
    T: IntoGeometric + FromGeometric + Clone + Default + 'static,
    F: Fn() -> T + 'static,
{
    let initial = if condition.sample() {
        Some(then_value())
    } else {
        None
    };

    let result = behavior(initial);
    let result_clone = result.clone();

    condition.subscribe(move |&cond| {
        if cond {
            result_clone.set(Some(then_value()));
        } else {
            result_clone.set(None);
        }
    });

    result
}

/// Combine two behaviors into one
///
/// Creates a behavior whose value is computed from both input behaviors
/// using the provided function.
///
/// # Example
///
/// ```rust
/// use cliffy_core::{behavior, combine};
///
/// let a = behavior(10i32);
/// let b = behavior(20i32);
/// let sum = combine(&a, &b, |x, y| x + y);
///
/// assert_eq!(sum.sample(), 30);
/// ```
pub fn combine<A, B, C, F>(a: &Behavior<A>, b: &Behavior<B>, f: F) -> Behavior<C>
where
    A: IntoGeometric + FromGeometric + Clone + 'static,
    B: IntoGeometric + FromGeometric + Clone + 'static,
    C: IntoGeometric + FromGeometric + Clone + 'static,
    F: Fn(A, B) -> C + Clone + 'static,
{
    a.combine(b, f)
}

/// Combine three behaviors into one
///
/// # Example
///
/// ```rust
/// use cliffy_core::{behavior, combinators::combine3};
///
/// let a = behavior(1i32);
/// let b = behavior(2i32);
/// let c = behavior(3i32);
/// let sum = combine3(&a, &b, &c, |x, y, z| x + y + z);
///
/// assert_eq!(sum.sample(), 6);
/// ```
pub fn combine3<A, B, C, D, F>(
    a: &Behavior<A>,
    b: &Behavior<B>,
    c: &Behavior<C>,
    f: F,
) -> Behavior<D>
where
    A: IntoGeometric + FromGeometric + Clone + 'static,
    B: IntoGeometric + FromGeometric + Clone + 'static,
    C: IntoGeometric + FromGeometric + Clone + 'static,
    D: IntoGeometric + FromGeometric + Clone + 'static,
    F: Fn(A, B, C) -> D + Clone + 'static,
{
    let ab = a.combine(b, |x, y| (x, y));
    ab.combine(c, move |(x, y), z| f(x, y, z))
}

/// If-then-else combinator
///
/// Creates a behavior that follows `then_value` when `condition` is true,
/// otherwise follows `else_value`.
///
/// # Example
///
/// ```rust
/// use cliffy_core::{behavior, combinators::if_else};
///
/// let is_admin = behavior(false);
/// let greeting = if_else(
///     &is_admin,
///     || "Welcome, Admin!".to_string(),
///     || "Welcome, User!".to_string(),
/// );
///
/// assert_eq!(greeting.sample(), "Welcome, User!");
///
/// is_admin.set(true);
/// assert_eq!(greeting.sample(), "Welcome, Admin!");
/// ```
pub fn if_else<T, TF, EF>(condition: &Behavior<bool>, then_value: TF, else_value: EF) -> Behavior<T>
where
    T: IntoGeometric + FromGeometric + Clone + 'static,
    TF: Fn() -> T + 'static,
    EF: Fn() -> T + 'static,
{
    let initial = if condition.sample() {
        then_value()
    } else {
        else_value()
    };

    let result = behavior(initial);
    let result_clone = result.clone();

    condition.subscribe(move |&cond| {
        if cond {
            result_clone.set(then_value());
        } else {
            result_clone.set(else_value());
        }
    });

    result
}

/// Constant behavior that never changes
///
/// # Example
///
/// ```rust
/// use cliffy_core::combinators::constant;
///
/// let pi = constant(3.14159);
/// assert_eq!(pi.sample(), 3.14159);
/// ```
pub fn constant<T>(value: T) -> Behavior<T>
where
    T: IntoGeometric + FromGeometric + Clone + 'static,
{
    behavior(value)
}

/// Map multiple behaviors with a function
///
/// This is useful when you need to transform multiple behaviors at once.
pub fn map2<A, B, C, F>(a: &Behavior<A>, b: &Behavior<B>, f: F) -> Behavior<C>
where
    A: IntoGeometric + FromGeometric + Clone + 'static,
    B: IntoGeometric + FromGeometric + Clone + 'static,
    C: IntoGeometric + FromGeometric + Clone + 'static,
    F: Fn(A, B) -> C + Clone + 'static,
{
    combine(a, b, f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_when_true() {
        let condition = behavior(true);
        let result = when(&condition, || 42i32);
        assert_eq!(result.sample(), Some(42));
    }

    #[test]
    fn test_when_false() {
        let condition = behavior(false);
        let result = when(&condition, || 42i32);
        assert_eq!(result.sample(), None);
    }

    #[test]
    fn test_when_reactive() {
        let condition = behavior(true);
        let result = when(&condition, || "visible".to_string());

        assert_eq!(result.sample(), Some("visible".to_string()));

        condition.set(false);
        assert_eq!(result.sample(), None);

        condition.set(true);
        assert_eq!(result.sample(), Some("visible".to_string()));
    }

    #[test]
    fn test_combine() {
        let a = behavior(10i32);
        let b = behavior(5i32);
        let sum = combine(&a, &b, |x, y| x + y);

        assert_eq!(sum.sample(), 15);

        a.set(20);
        assert_eq!(sum.sample(), 25);
    }

    #[test]
    fn test_combine3() {
        let a = behavior(1i32);
        let b = behavior(2i32);
        let c = behavior(3i32);
        let sum = combine3(&a, &b, &c, |x, y, z| x + y + z);

        assert_eq!(sum.sample(), 6);
    }

    #[test]
    fn test_if_else() {
        let is_dark_mode = behavior(false);
        let theme = if_else(&is_dark_mode, || "dark".to_string(), || "light".to_string());

        assert_eq!(theme.sample(), "light");

        is_dark_mode.set(true);
        assert_eq!(theme.sample(), "dark");
    }

    #[test]
    fn test_constant() {
        let c = constant(42i32);
        assert_eq!(c.sample(), 42);
    }
}
