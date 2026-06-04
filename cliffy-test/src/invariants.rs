//! Invariant types for algebraic testing
//!
//! Invariants are properties that should hold across state transformations.
//! They are categorized into three types following the Flynn philosophy:
//!
//! - **Impossible** (P = 0): Must NEVER fail
//! - **Rare** (0 < P << 1): Bounded failure probability
//! - **Emergent** (P > 0): Valid but unpredicted behaviors

use crate::error::GeometricError;
use crate::result::{InvariantCategory, InvariantTestReport, TestResult};
use amari_flynn::backend::monte_carlo::MonteCarloVerifier;
use amari_flynn::backend::smt::{
    hoeffding_obligation, precondition_obligation, SmtProofObligation,
};
use amari_flynn::contracts::VerificationResult;
use amari_flynn::statistical::bounds::hoeffding_bound;
use amari_flynn::statistical::estimators::confidence_interval;
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Trait for invariants that can be verified
pub trait Invariant {
    /// Name of the invariant
    fn name(&self) -> &str;

    /// Category of the invariant
    fn category(&self) -> InvariantCategory;

    /// Verify the invariant with a single sample
    fn verify_once(&self) -> TestResult;

    /// Verify the invariant with multiple samples
    fn verify(&self, samples: usize) -> InvariantTestReport
    where
        Self: Sync,
    {
        let failures = AtomicUsize::new(0);
        let sample_errors: std::sync::Mutex<Vec<GeometricError>> =
            std::sync::Mutex::new(Vec::new());

        // Run samples in parallel
        (0..samples).into_par_iter().for_each(|_| {
            if let TestResult::Fail(error) = self.verify_once() {
                failures.fetch_add(1, Ordering::Relaxed);
                let mut errors = sample_errors.lock().unwrap();
                if errors.len() < 10 {
                    // Keep at most 10 sample errors
                    errors.push(error);
                }
            }
        });

        let failure_count = failures.load(Ordering::Relaxed);
        let failure_rate = failure_count as f64 / samples as f64;

        InvariantTestReport {
            name: self.name().to_string(),
            category: self.category(),
            samples,
            failures: failure_count,
            failure_rate,
            probability_bound: None,
            verified: failure_count == 0 || self.category() == InvariantCategory::Emergent,
            sample_errors: sample_errors.into_inner().unwrap(),
            confidence_interval: None,
            confidence_level: None,
        }
    }
}

/// An impossible invariant - must NEVER fail (P = 0)
///
/// Use this for properties that are mathematically guaranteed,
/// like "rotors preserve magnitude" or "geometric product is associative".
pub struct ImpossibleInvariant<F>
where
    F: Fn() -> TestResult + Send + Sync,
{
    /// Name of the invariant
    pub name: String,
    /// Verification function
    pub check: F,
}

impl<F> ImpossibleInvariant<F>
where
    F: Fn() -> TestResult + Send + Sync,
{
    /// Create a new impossible invariant
    pub fn new(name: impl Into<String>, check: F) -> Self {
        Self {
            name: name.into(),
            check,
        }
    }

    /// Export this invariant as an SMT-LIB2 proof obligation
    ///
    /// The obligation asserts that the precondition holds with probability 1.0,
    /// suitable for verification by Z3, CVC5, or other SMT solvers.
    pub fn export_smt(&self) -> SmtProofObligation {
        precondition_obligation(
            &self.name,
            format!("Impossible invariant: {} (P = 0 failure)", self.name),
            1.0,
        )
    }

    /// Verify and cross-check against SMT obligation via Monte Carlo
    ///
    /// Runs the invariant check, then verifies the generated SMT obligation
    /// statistically for consistency.
    pub fn verify_with_smt(&self, samples: usize) -> (InvariantTestReport, VerificationResult) {
        let report = self.verify(samples);
        let obligation = self.export_smt();
        let smt_result = obligation.verify_with_monte_carlo(samples);
        (report, smt_result)
    }
}

impl<F> Invariant for ImpossibleInvariant<F>
where
    F: Fn() -> TestResult + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn category(&self) -> InvariantCategory {
        InvariantCategory::Impossible
    }

    fn verify_once(&self) -> TestResult {
        (self.check)()
    }
}

/// A rare invariant - bounded failure probability (0 < P << 1)
///
/// Use this for properties that should rarely fail, like
/// "convergence within N iterations" or "floating point precision bounds".
pub struct RareInvariant<F>
where
    F: Fn() -> TestResult + Send + Sync,
{
    /// Name of the invariant
    pub name: String,
    /// Probability bound (maximum acceptable failure rate)
    pub probability_bound: f64,
    /// Verification function
    pub check: F,
}

impl<F> RareInvariant<F>
where
    F: Fn() -> TestResult + Send + Sync,
{
    /// Create a new rare invariant
    pub fn new(name: impl Into<String>, probability_bound: f64, check: F) -> Self {
        assert!(
            probability_bound > 0.0 && probability_bound < 1.0,
            "Probability bound must be in (0, 1)"
        );
        Self {
            name: name.into(),
            probability_bound,
            check,
        }
    }

    /// Export this invariant as an SMT-LIB2 proof obligation
    ///
    /// The obligation encodes a Hoeffding bound verification: given n samples
    /// and the probability bound, it asserts the concentration inequality holds.
    pub fn export_smt(&self, samples: usize) -> SmtProofObligation {
        hoeffding_obligation(
            &self.name,
            samples,
            self.probability_bound,
            0.05, // 95% confidence (delta = 0.05)
        )
    }

    /// Verify and cross-check against SMT obligation via Monte Carlo
    ///
    /// Runs probabilistic verification, then verifies the generated SMT obligation
    /// statistically for consistency.
    pub fn verify_with_smt(&self, samples: usize) -> (InvariantTestReport, VerificationResult) {
        let report = self.verify_probabilistic(samples);
        let obligation = self.export_smt(samples);
        let smt_result = obligation.verify_with_monte_carlo(samples);
        (report, smt_result)
    }

    /// Verify using Monte Carlo with Hoeffding bounds
    pub fn verify_probabilistic(&self, samples: usize) -> InvariantTestReport {
        let verifier = MonteCarloVerifier::new(samples);

        let failures = AtomicUsize::new(0);
        let sample_errors: std::sync::Mutex<Vec<GeometricError>> =
            std::sync::Mutex::new(Vec::new());

        // Use Monte Carlo verification
        let result = verifier.verify_probability_bound(
            || {
                let test_result = (self.check)();
                if let TestResult::Fail(error) = test_result {
                    failures.fetch_add(1, Ordering::Relaxed);
                    let mut errors = sample_errors.lock().unwrap();
                    if errors.len() < 10 {
                        errors.push(error);
                    }
                    true // Predicate returns true for failures
                } else {
                    false
                }
            },
            self.probability_bound,
        );

        let failure_count = failures.load(Ordering::Relaxed);
        let failure_rate = failure_count as f64 / samples as f64;

        // Verified if Monte Carlo says so, OR if empirical failure rate is below bound
        let verified =
            result == VerificationResult::Verified || failure_rate < self.probability_bound;

        // Compute confidence interval for the failure rate estimate
        let ci = confidence_interval(failure_count, samples, 0.95);

        // Compute confidence level from Hoeffding bound
        let epsilon = (self.probability_bound - failure_rate).abs();
        let hoeffding = hoeffding_bound(samples, epsilon);
        // Hoeffding gives P(|X̄ - μ| ≥ ε) ≤ bound, so confidence = 1 - bound
        let conf_level = (1.0 - hoeffding).clamp(0.0, 1.0);

        InvariantTestReport {
            name: self.name.clone(),
            category: InvariantCategory::Rare,
            samples,
            failures: failure_count,
            failure_rate,
            probability_bound: Some(self.probability_bound),
            verified,
            sample_errors: sample_errors.into_inner().unwrap(),
            confidence_interval: Some(ci),
            confidence_level: Some(conf_level),
        }
    }
}

impl<F> Invariant for RareInvariant<F>
where
    F: Fn() -> TestResult + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn category(&self) -> InvariantCategory {
        InvariantCategory::Rare
    }

    fn verify_once(&self) -> TestResult {
        (self.check)()
    }

    fn verify(&self, samples: usize) -> InvariantTestReport {
        self.verify_probabilistic(samples)
    }
}

/// An emergent behavior tracker - valid but unpredicted (P > 0)
///
/// Use this to track behaviors that aren't failures but are interesting
/// or unexpected. Like the ISOs in Tron, emergent behaviors are opportunities
/// for discovery.
pub struct EmergentBehavior<T, F>
where
    F: Fn(&T) + Send + Sync,
{
    /// Name of the emergent behavior
    pub name: String,
    /// Description of what this tracks
    pub description: String,
    /// Callback when behavior is observed
    pub on_observe: F,
    /// Phantom data for the type
    _phantom: std::marker::PhantomData<T>,
}

impl<T, F> EmergentBehavior<T, F>
where
    F: Fn(&T) + Send + Sync,
{
    /// Create a new emergent behavior tracker
    pub fn new(name: impl Into<String>, description: impl Into<String>, on_observe: F) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            on_observe,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Record an observation of this emergent behavior
    pub fn observe(&self, data: &T) {
        (self.on_observe)(data);
    }
}

/// Registry for tracking emergent behaviors
pub struct EmergentRegistry {
    observations: std::sync::Mutex<Vec<EmergentObservation>>,
}

/// A recorded emergent observation
#[derive(Clone, Debug)]
pub struct EmergentObservation {
    /// Name of the emergent behavior
    pub name: String,
    /// Description
    pub description: String,
    /// Timestamp of observation
    pub timestamp: std::time::Instant,
}

impl EmergentRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            observations: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Record an observation
    pub fn record(&self, name: impl Into<String>, description: impl Into<String>) {
        let mut obs = self.observations.lock().unwrap();
        obs.push(EmergentObservation {
            name: name.into(),
            description: description.into(),
            timestamp: std::time::Instant::now(),
        });
    }

    /// Get all observations
    pub fn observations(&self) -> Vec<EmergentObservation> {
        self.observations.lock().unwrap().clone()
    }

    /// Get count of observations
    pub fn count(&self) -> usize {
        self.observations.lock().unwrap().len()
    }
}

impl Default for EmergentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_impossible_invariant_pass() {
        let inv = ImpossibleInvariant::new("Always passes", || TestResult::Pass);
        let report = inv.verify(100);
        assert!(report.verified);
        assert_eq!(report.failures, 0);
    }

    #[test]
    fn test_impossible_invariant_fail() {
        let inv = ImpossibleInvariant::new("Always fails", || {
            TestResult::fail_with_distance(1.0, "Intentional failure")
        });
        let report = inv.verify(10);
        assert!(!report.verified);
        assert_eq!(report.failures, 10);
    }

    #[test]
    fn test_rare_invariant() {
        use rand::RngExt;

        // Invariant that fails ~10% of the time
        let inv = RareInvariant::new(
            "Rarely fails",
            0.2, // Allow up to 20% failure rate
            || {
                let mut rng = rand::rng();
                if rng.random::<f64>() < 0.1 {
                    TestResult::fail_with_distance(0.1, "Random failure")
                } else {
                    TestResult::Pass
                }
            },
        );

        let report = inv.verify(1000);
        // Should be verified since actual rate (~10%) < bound (20%)
        assert!(report.verified);
    }

    #[test]
    fn test_emergent_registry() {
        let registry = EmergentRegistry::new();
        registry.record("test", "A test observation");
        registry.record("test2", "Another observation");

        assert_eq!(registry.count(), 2);
        let obs = registry.observations();
        assert_eq!(obs[0].name, "test");
        assert_eq!(obs[1].name, "test2");
    }

    #[test]
    fn test_impossible_export_smt() {
        let inv = ImpossibleInvariant::new("Magnitude non-negative", || TestResult::Pass);
        let obligation = inv.export_smt();
        let smt_output = obligation.to_smtlib2();

        assert!(smt_output.contains("Magnitude non-negative"));
        assert!(!smt_output.is_empty());
    }

    #[test]
    fn test_impossible_verify_with_smt() {
        let inv = ImpossibleInvariant::new("Always passes", || TestResult::Pass);
        let (report, smt_result) = inv.verify_with_smt(100);

        assert!(report.verified);
        assert_eq!(report.failures, 0);
        // SMT cross-verification should not report a violation
        assert_ne!(smt_result, VerificationResult::Violated);
    }

    #[test]
    fn test_rare_export_smt() {
        let inv = RareInvariant::new("Rarely fails", 0.1, || TestResult::Pass);
        let obligation = inv.export_smt(1000);
        let smt_output = obligation.to_smtlib2();

        assert!(smt_output.contains("Rarely fails"));
        assert!(!smt_output.is_empty());
    }

    #[test]
    fn test_rare_verify_with_smt() {
        use rand::RngExt;

        let inv = RareInvariant::new("Bounded failure", 0.2, || {
            let mut rng = rand::rng();
            if rng.random::<f64>() < 0.05 {
                TestResult::fail_with_distance(0.1, "Random failure")
            } else {
                TestResult::Pass
            }
        });

        let (report, _smt_result) = inv.verify_with_smt(500);
        assert!(report.verified);
    }

    #[test]
    fn test_rare_confidence_interval() {
        let inv = RareInvariant::new("Test CI", 0.5, || TestResult::Pass);
        let report = inv.verify_probabilistic(1000);

        // Should have confidence interval populated
        assert!(report.confidence_interval.is_some());
        let (lower, upper) = report.confidence_interval.unwrap();
        assert!(lower <= upper);
        // Since no failures, lower bound should be near 0
        assert!(lower >= 0.0);

        // Should have confidence level populated
        assert!(report.confidence_level.is_some());
        let conf = report.confidence_level.unwrap();
        assert!((0.0..=1.0).contains(&conf));
    }
}
