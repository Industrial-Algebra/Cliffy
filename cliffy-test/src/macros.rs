//! Declarative macros for algebraic testing
//!
//! These macros provide ergonomic syntax for defining invariants:
//!
//! - `invariant_impossible!` - Must NEVER fail (P = 0)
//! - `invariant_rare!` - Bounded failure probability (0 < P << 1)
//! - `emergent!` - Track valid but unexpected behaviors

/// Define an impossible invariant - must NEVER fail (P = 0)
///
/// Use this for properties that are mathematically guaranteed,
/// like "rotors preserve magnitude" or "geometric product is associative".
///
/// # Syntax
///
/// ```rust
/// use cliffy_test::invariant_impossible;
/// use cliffy_test::prelude::*;
///
/// let _inv = invariant_impossible! {
///     name: "Example invariant",
///     check: || {
///         // Return TestResult::Pass or TestResult::Fail
///         TestResult::Pass
///     }
/// };
/// ```
///
/// # Example
///
/// ```rust
/// use cliffy_test::invariant_impossible;
/// use cliffy_test::prelude::*;
///
/// let inv = invariant_impossible! {
///     name: "Rotor preserves magnitude",
///     check: || {
///         use cliffy_test::generators::*;
///         let mut gen = quickcheck::Gen::new(100);
///
///         let v = arbitrary_vector(&mut gen);
///         let r = arbitrary_rotor(&mut gen);
///         let rotated = cliffy_test::sandwich(&r, &v);
///
///         let diff = (v.magnitude() - rotated.magnitude()).abs();
///         if diff < 1e-6 {
///             TestResult::Pass
///         } else {
///             TestResult::fail_with_distance(diff, "Magnitude changed under rotation")
///         }
///     }
/// };
/// // Use inv.verify(100) to run the invariant
/// ```
#[macro_export]
macro_rules! invariant_impossible {
    (
        name: $name:expr,
        check: $check:expr
    ) => {{
        $crate::invariants::ImpossibleInvariant::new($name, $check)
    }};
}

/// Define a rare invariant - bounded failure probability (0 < P << 1)
///
/// Use this for properties that should rarely fail, like
/// "convergence within N iterations" or "floating point precision bounds".
///
/// # Syntax
///
/// ```rust
/// use cliffy_test::invariant_rare;
/// use cliffy_test::prelude::*;
///
/// let _inv = invariant_rare! {
///     name: "Example rare invariant",
///     probability_bound: 0.001,
///     check: || {
///         // Return TestResult::Pass or TestResult::Fail
///         TestResult::Pass
///     }
/// };
/// ```
///
/// # Example
///
/// ```rust
/// use cliffy_test::invariant_rare;
/// use cliffy_test::prelude::*;
/// use rand::RngExt;
///
/// let inv = invariant_rare! {
///     name: "Random value usually in range",
///     probability_bound: 0.1,  // Allow up to 10% failure rate
///     check: || {
///         let mut rng = rand::rng();
///         let value: f64 = rng.random_range(0.0..1.0);
///
///         if value < 0.95 {  // 95% of values should be < 0.95
///             TestResult::Pass
///         } else {
///             TestResult::fail_with_distance(value - 0.95, "Value too high")
///         }
///     }
/// };
/// // Use inv.verify(100) to run the invariant
/// ```
#[macro_export]
macro_rules! invariant_rare {
    (
        name: $name:expr,
        probability_bound: $bound:expr,
        check: $check:expr
    ) => {{
        $crate::invariants::RareInvariant::new($name, $bound, $check)
    }};
}

/// Track emergent behavior - valid but unpredicted (P > 0)
///
/// Use this to track behaviors that aren't failures but are interesting
/// or unexpected. Like the ISOs in Tron, emergent behaviors are opportunities
/// for discovery.
///
/// # Syntax
///
/// ```rust
/// use cliffy_test::emergent;
///
/// let _tracker = emergent! {
///     name: "Novel pattern",
///     description: "An unexpected but valid behavior",
///     on_observe: |data: &String| {
///         println!("Observed: {}", data);
///     }
/// };
/// ```
///
/// # Example
///
/// ```rust
/// use cliffy_test::emergent;
///
/// let tracker = emergent! {
///     name: "Novel merge trajectories",
///     description: "Merge paths we didn't anticipate but are valid",
///     on_observe: |trajectory: &Vec<i32>| {
///         eprintln!("Emergent merge trajectory: {:?}", trajectory);
///     }
/// };
///
/// // Later, when observing the behavior:
/// // tracker.observe(&trajectory);
/// ```
#[macro_export]
macro_rules! emergent {
    (
        name: $name:expr,
        description: $desc:expr,
        on_observe: $callback:expr
    ) => {{
        $crate::invariants::EmergentBehavior::new($name, $desc, $callback)
    }};
}

/// Run a test with geometric invariant checking
///
/// This macro wraps a test function to provide geometric error reporting.
///
/// # Example
///
/// ```rust
/// use cliffy_test::geometric_test;
/// use cliffy_test::prelude::*;
///
/// geometric_test! {
///     name: "Vector operations preserve structure",
///     fn test_vector_ops() -> TestResult {
///         let v1 = GA3::vector([1.0, 0.0, 0.0]);
///         let v2 = GA3::vector([0.0, 1.0, 0.0]);
///         let sum = &v1 + &v2;
///
///         // Verify the sum is still a vector (grade 1)
///         let manifold = cliffy_test::manifold::Manifold::new("Vector space")
///             .with_constraint(cliffy_test::manifold::PureVectorConstraint);
///
///         manifold.verify(&sum)
///     }
/// }
/// ```
#[macro_export]
macro_rules! geometric_test {
    (
        name: $name:expr,
        fn $fn_name:ident() -> TestResult $body:block
    ) => {
        #[test]
        fn $fn_name() {
            use $crate::result::TestResult;

            fn inner() -> TestResult $body

            let result = inner();
            match result {
                TestResult::Pass => {}
                TestResult::Fail(error) => {
                    panic!(
                        "Geometric test '{}' failed: {}\n  Distance: {:.6}\n  Gradient: {:?}",
                        $name, error.description, error.distance, error.gradient
                    );
                }
                TestResult::Skipped(reason) => {
                    println!("Skipped '{}': {}", $name, reason);
                }
            }
        }
    };
}

/// Verify an invariant and panic with geometric error info on failure
///
/// # Example
///
/// ```rust
/// use cliffy_test::{verify_invariant, invariant_impossible};
/// use cliffy_test::prelude::*;
///
/// let inv = invariant_impossible! {
///     name: "Always passes",
///     check: || TestResult::Pass
/// };
///
/// verify_invariant!(inv, samples: 100);
/// ```
#[macro_export]
macro_rules! verify_invariant {
    ($invariant:expr, samples: $samples:expr) => {{
        use $crate::invariants::Invariant;

        let report = $invariant.verify($samples);
        if report.is_violated() {
            panic!(
                "Invariant '{}' violated!\n  Category: {:?}\n  Failures: {}/{}\n  Failure rate: {:.4}%\n  Sample errors: {:?}",
                report.name,
                report.category,
                report.failures,
                report.samples,
                report.failure_rate * 100.0,
                report.sample_errors.iter().map(|e| &e.description).collect::<Vec<_>>()
            );
        }
    }};
}

#[cfg(test)]
mod tests {
    use crate::invariants::Invariant;
    use crate::result::TestResult;

    #[test]
    fn test_invariant_impossible_macro() {
        let inv = invariant_impossible! {
            name: "Always passes",
            check: || TestResult::Pass
        };

        let report = inv.verify(10);
        assert!(report.verified);
        assert_eq!(report.failures, 0);
    }

    #[test]
    fn test_invariant_rare_macro() {
        let inv = invariant_rare! {
            name: "Always passes rare",
            probability_bound: 0.1,
            check: || TestResult::Pass
        };

        let report = inv.verify(100);
        assert!(report.verified);
    }

    #[test]
    fn test_emergent_macro() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = count.clone();

        let tracker = emergent! {
            name: "Test emergent",
            description: "Tracking test observations",
            on_observe: move |_: &i32| {
                count_clone.fetch_add(1, Ordering::Relaxed);
            }
        };

        tracker.observe(&42);
        tracker.observe(&43);

        assert_eq!(count.load(Ordering::Relaxed), 2);
    }
}
