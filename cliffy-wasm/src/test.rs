//! WASM bindings for cliffy-test - Algebraic Testing Framework
//!
//! Exposes the test framework to JavaScript/TypeScript for testing
//! geometric algebra operations in the browser.

use js_sys::Function;
use wasm_bindgen::prelude::*;

// Re-export from cliffy-test
use cliffy_test::error::GeometricError as CoreGeometricError;
use cliffy_test::result::{
    InvariantCategory as CoreInvariantCategory, TestResult as CoreTestResult,
};
use cliffy_test::GA3;

/// Test result from a geometric test
#[wasm_bindgen]
pub struct TestResult {
    inner: CoreTestResult,
}

#[wasm_bindgen]
impl TestResult {
    /// Create a passing test result
    #[wasm_bindgen]
    pub fn pass() -> TestResult {
        TestResult {
            inner: CoreTestResult::Pass,
        }
    }

    /// Create a failing test result with distance and description
    #[wasm_bindgen(js_name = failWithDistance)]
    pub fn fail_with_distance(distance: f64, description: &str) -> TestResult {
        TestResult {
            inner: CoreTestResult::fail_with_distance(distance, description),
        }
    }

    /// Create a skipped test result
    #[wasm_bindgen]
    pub fn skipped(reason: &str) -> TestResult {
        TestResult {
            inner: CoreTestResult::skipped(reason),
        }
    }

    /// Check if test passed
    #[wasm_bindgen(js_name = isPass)]
    pub fn is_pass(&self) -> bool {
        self.inner.is_pass()
    }

    /// Check if test failed
    #[wasm_bindgen(js_name = isFail)]
    pub fn is_fail(&self) -> bool {
        self.inner.is_fail()
    }

    /// Check if test was skipped
    #[wasm_bindgen(js_name = isSkipped)]
    pub fn is_skipped(&self) -> bool {
        matches!(self.inner, CoreTestResult::Skipped(_))
    }

    /// Get the geometric error if test failed
    #[wasm_bindgen]
    pub fn error(&self) -> Option<GeometricError> {
        self.inner
            .error()
            .map(|e| GeometricError { inner: e.clone() })
    }

    /// Get the skip reason if test was skipped
    #[wasm_bindgen(js_name = skipReason)]
    pub fn skip_reason(&self) -> Option<String> {
        match &self.inner {
            CoreTestResult::Skipped(reason) => Some(reason.clone()),
            _ => None,
        }
    }

    /// Convert to string representation
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string_js(&self) -> String {
        match &self.inner {
            CoreTestResult::Pass => "Pass".to_string(),
            CoreTestResult::Fail(e) => {
                format!("Fail: {} (distance: {})", e.description, e.distance)
            }
            CoreTestResult::Skipped(reason) => format!("Skipped: {}", reason),
        }
    }
}

/// Geometric error information for test failures
#[wasm_bindgen]
pub struct GeometricError {
    inner: CoreGeometricError,
}

#[wasm_bindgen]
impl GeometricError {
    /// Create a new geometric error
    #[wasm_bindgen(constructor)]
    pub fn new(distance: f64, description: &str) -> GeometricError {
        GeometricError {
            inner: CoreGeometricError::new(distance, description),
        }
    }

    /// Get the distance from expected manifold
    #[wasm_bindgen(getter)]
    pub fn distance(&self) -> f64 {
        self.inner.distance
    }

    /// Get the error description
    #[wasm_bindgen(getter)]
    pub fn description(&self) -> String {
        self.inner.description.clone()
    }

    /// Get the gradient as [x, y, z]
    #[wasm_bindgen(getter)]
    pub fn gradient(&self) -> Vec<f64> {
        self.inner.gradient.to_vec()
    }

    /// Get the correction as [s, e1, e2, e3, e12, e13, e23, e123]
    #[wasm_bindgen(getter)]
    pub fn correction(&self) -> Vec<f64> {
        self.inner.correction.to_vec()
    }

    /// Check if error is within tolerance
    #[wasm_bindgen(js_name = isWithinTolerance)]
    pub fn is_within_tolerance(&self, epsilon: f64) -> bool {
        self.inner.is_within_tolerance(epsilon)
    }
}

/// Category of invariant
#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum InvariantCategory {
    /// Impossible (P = 0) - must never fail
    Impossible = 0,
    /// Rare (0 < P << 1) - bounded failure probability
    Rare = 1,
    /// Emergent (P > 0) - valid but unpredicted
    Emergent = 2,
}

impl From<CoreInvariantCategory> for InvariantCategory {
    fn from(cat: CoreInvariantCategory) -> Self {
        match cat {
            CoreInvariantCategory::Impossible => InvariantCategory::Impossible,
            CoreInvariantCategory::Rare => InvariantCategory::Rare,
            CoreInvariantCategory::Emergent => InvariantCategory::Emergent,
        }
    }
}

/// Result of running an invariant test suite
#[wasm_bindgen]
pub struct InvariantTestReport {
    /// Name of the invariant
    name: String,
    /// Category
    category: InvariantCategory,
    /// Number of samples tested
    samples: usize,
    /// Number of failures
    failures: usize,
    /// Failure rate
    failure_rate: f64,
    /// Whether verified
    verified: bool,
}

#[wasm_bindgen]
impl InvariantTestReport {
    /// Get the invariant name
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Get the category
    #[wasm_bindgen(getter)]
    pub fn category(&self) -> InvariantCategory {
        self.category
    }

    /// Get number of samples tested
    #[wasm_bindgen(getter)]
    pub fn samples(&self) -> usize {
        self.samples
    }

    /// Get number of failures
    #[wasm_bindgen(getter)]
    pub fn failures(&self) -> usize {
        self.failures
    }

    /// Get the failure rate
    #[wasm_bindgen(getter, js_name = failureRate)]
    pub fn failure_rate(&self) -> f64 {
        self.failure_rate
    }

    /// Check if invariant was verified
    #[wasm_bindgen(getter)]
    pub fn verified(&self) -> bool {
        self.verified
    }
}

/// A geometric manifold for testing state validity
#[wasm_bindgen]
pub struct Manifold {
    name: String,
    constraints: Vec<ConstraintType>,
    tolerance: f64,
}

#[derive(Clone)]
enum ConstraintType {
    Magnitude(f64),
    Scalar(f64),
    PureVector,
    PureBivector,
}

#[wasm_bindgen]
impl Manifold {
    /// Create a new empty manifold
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str) -> Manifold {
        Manifold {
            name: name.to_string(),
            constraints: Vec::new(),
            tolerance: 1e-10,
        }
    }

    /// Set the tolerance for membership tests
    #[wasm_bindgen(js_name = withTolerance)]
    pub fn with_tolerance(mut self, tolerance: f64) -> Manifold {
        self.tolerance = tolerance;
        self
    }

    /// Add a magnitude constraint
    #[wasm_bindgen(js_name = withMagnitude)]
    pub fn with_magnitude(mut self, target: f64) -> Manifold {
        self.constraints.push(ConstraintType::Magnitude(target));
        self
    }

    /// Add a unit magnitude constraint
    #[wasm_bindgen(js_name = withUnitMagnitude)]
    pub fn with_unit_magnitude(mut self) -> Manifold {
        self.constraints.push(ConstraintType::Magnitude(1.0));
        self
    }

    /// Add a scalar constraint
    #[wasm_bindgen(js_name = withScalar)]
    pub fn with_scalar(mut self, target: f64) -> Manifold {
        self.constraints.push(ConstraintType::Scalar(target));
        self
    }

    /// Add a pure vector constraint (only grade 1 components)
    #[wasm_bindgen(js_name = withPureVector)]
    pub fn with_pure_vector(mut self) -> Manifold {
        self.constraints.push(ConstraintType::PureVector);
        self
    }

    /// Add a pure bivector constraint (only grade 2 components)
    #[wasm_bindgen(js_name = withPureBivector)]
    pub fn with_pure_bivector(mut self) -> Manifold {
        self.constraints.push(ConstraintType::PureBivector);
        self
    }

    /// Check if a point (as [s, e1, e2, e3, e12, e13, e23, e123]) lies on the manifold
    #[wasm_bindgen]
    pub fn contains(&self, coeffs: &[f64]) -> bool {
        if coeffs.len() != 8 {
            return false;
        }
        let point = GA3::from_coefficients(coeffs.to_vec());
        self.distance_internal(&point) < self.tolerance
    }

    /// Calculate distance from the manifold
    #[wasm_bindgen]
    pub fn distance(&self, coeffs: &[f64]) -> f64 {
        if coeffs.len() != 8 {
            return f64::INFINITY;
        }
        let point = GA3::from_coefficients(coeffs.to_vec());
        self.distance_internal(&point)
    }

    /// Project a point onto the manifold
    #[wasm_bindgen]
    pub fn project(&self, coeffs: &[f64]) -> Vec<f64> {
        if coeffs.len() != 8 {
            return coeffs.to_vec();
        }
        let point = GA3::from_coefficients(coeffs.to_vec());
        let projected = self.project_internal(&point);
        (0..8).map(|i| projected.get(i)).collect()
    }

    /// Verify that a point lies on the manifold
    #[wasm_bindgen]
    pub fn verify(&self, coeffs: &[f64]) -> TestResult {
        if coeffs.len() != 8 {
            return TestResult::fail_with_distance(
                f64::INFINITY,
                "Invalid coefficients: expected 8 elements",
            );
        }
        let point = GA3::from_coefficients(coeffs.to_vec());
        let dist = self.distance_internal(&point);

        if dist < self.tolerance {
            TestResult::pass()
        } else {
            TestResult::fail_with_distance(dist, &format!("Point not on manifold '{}'", self.name))
        }
    }

    fn distance_internal(&self, point: &GA3) -> f64 {
        self.constraints
            .iter()
            .map(|c| self.constraint_distance(c, point))
            .fold(0.0, f64::max)
    }

    fn constraint_distance(&self, constraint: &ConstraintType, point: &GA3) -> f64 {
        match constraint {
            ConstraintType::Magnitude(target) => (point.magnitude() - target).abs(),
            ConstraintType::Scalar(target) => (point.get(0) - target).abs(),
            ConstraintType::PureVector => {
                let s = point.get(0);
                let b12 = point.get(3);
                let b13 = point.get(5);
                let b23 = point.get(6);
                let tri = point.get(7);
                (s * s + b12 * b12 + b13 * b13 + b23 * b23 + tri * tri).sqrt()
            }
            ConstraintType::PureBivector => {
                let s = point.get(0);
                let v1 = point.get(1);
                let v2 = point.get(2);
                let v3 = point.get(4);
                let tri = point.get(7);
                (s * s + v1 * v1 + v2 * v2 + v3 * v3 + tri * tri).sqrt()
            }
        }
    }

    fn project_internal(&self, point: &GA3) -> GA3 {
        let mut result = point.clone();
        for _ in 0..10 {
            let mut changed = false;
            for constraint in &self.constraints {
                if self.constraint_distance(constraint, &result) >= self.tolerance {
                    result = self.project_constraint(constraint, &result);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        result
    }

    fn project_constraint(&self, constraint: &ConstraintType, point: &GA3) -> GA3 {
        match constraint {
            ConstraintType::Magnitude(target) => {
                let mag = point.magnitude();
                if mag > 1e-10 {
                    point * (*target / mag)
                } else {
                    point.clone()
                }
            }
            ConstraintType::Scalar(target) => {
                let mut coeffs: Vec<f64> = (0..8).map(|i| point.get(i)).collect();
                coeffs[0] = *target;
                GA3::from_coefficients(coeffs)
            }
            ConstraintType::PureVector => {
                // Keep only vector components (e1, e2, e3 at indices 1, 2, 4)
                GA3::from_coefficients(vec![
                    0.0,
                    point.get(1),
                    point.get(2),
                    0.0,
                    point.get(4),
                    0.0,
                    0.0,
                    0.0,
                ])
            }
            ConstraintType::PureBivector => {
                // Keep only bivector components (e12, e13, e23 at indices 3, 5, 6)
                GA3::from_coefficients(vec![
                    0.0,
                    0.0,
                    0.0,
                    point.get(3),
                    0.0,
                    point.get(5),
                    point.get(6),
                    0.0,
                ])
            }
        }
    }
}

/// Create the unit sphere manifold (pure vectors with magnitude 1)
#[wasm_bindgen(js_name = unitSphere)]
pub fn unit_sphere() -> Manifold {
    Manifold::new("Unit Sphere")
        .with_pure_vector()
        .with_unit_magnitude()
}

/// Create the rotor manifold (unit magnitude elements)
#[wasm_bindgen(js_name = rotorManifold)]
pub fn rotor_manifold() -> Manifold {
    Manifold::new("Rotor Manifold").with_unit_magnitude()
}

/// Run an impossible invariant test
///
/// The check function should return a boolean (true = pass, false = fail).
/// This will run the check `samples` times and fail if ANY invocation fails.
#[wasm_bindgen(js_name = testImpossible)]
pub fn test_impossible(
    name: &str,
    check: &Function,
    samples: usize,
) -> Result<InvariantTestReport, JsValue> {
    let mut failures = 0;

    for _ in 0..samples {
        let result = check.call0(&JsValue::null())?;

        // Check if result is falsy (failure)
        if !result.is_truthy() {
            failures += 1;
        }
    }

    let failure_rate = failures as f64 / samples as f64;

    Ok(InvariantTestReport {
        name: name.to_string(),
        category: InvariantCategory::Impossible,
        samples,
        failures,
        failure_rate,
        verified: failures == 0,
    })
}

/// Run a rare invariant test with probability bound
///
/// The check function should return a boolean (true = pass, false = fail).
/// This will run the check `samples` times and fail if failure rate exceeds bound.
#[wasm_bindgen(js_name = testRare)]
pub fn test_rare(
    name: &str,
    probability_bound: f64,
    check: &Function,
    samples: usize,
) -> Result<InvariantTestReport, JsValue> {
    let mut failures = 0;

    for _ in 0..samples {
        let result = check.call0(&JsValue::null())?;

        // Check if result is falsy (failure)
        if !result.is_truthy() {
            failures += 1;
        }
    }

    let failure_rate = failures as f64 / samples as f64;

    Ok(InvariantTestReport {
        name: name.to_string(),
        category: InvariantCategory::Rare,
        samples,
        failures,
        failure_rate,
        verified: failure_rate <= probability_bound,
    })
}

/// Generate a random GA3 multivector for property testing
#[wasm_bindgen(js_name = randomGA3)]
pub fn random_ga3() -> Vec<f64> {
    use rand::RngExt;
    let mut rng = rand::rng();
    (0..8).map(|_| rng.random_range(-10.0..10.0)).collect()
}

/// Generate a random unit vector for property testing
#[wasm_bindgen(js_name = randomUnitVector)]
pub fn random_unit_vector() -> Vec<f64> {
    use rand::RngExt;
    let mut rng = rand::rng();
    let x: f64 = rng.random_range(-1.0..1.0);
    let y: f64 = rng.random_range(-1.0..1.0);
    let z: f64 = rng.random_range(-1.0..1.0);
    let mag = (x * x + y * y + z * z).sqrt();
    if mag > 1e-10 {
        vec![0.0, x / mag, y / mag, 0.0, z / mag, 0.0, 0.0, 0.0]
    } else {
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0] // fallback to e1
    }
}

/// Generate a random rotor for property testing
#[wasm_bindgen(js_name = randomRotor)]
pub fn random_rotor() -> Vec<f64> {
    use rand::RngExt;
    let mut rng = rand::rng();

    // Generate random angle
    let angle: f64 = rng.random_range(0.0..std::f64::consts::TAU);
    let half_angle = angle / 2.0;

    // Generate random axis
    let x: f64 = rng.random_range(-1.0..1.0);
    let y: f64 = rng.random_range(-1.0..1.0);
    let z: f64 = rng.random_range(-1.0..1.0);
    let mag = (x * x + y * y + z * z).sqrt();

    if mag < 1e-10 {
        // Identity rotor
        return vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    }

    let nx = x / mag;
    let ny = y / mag;
    let nz = z / mag;

    // Rotor = cos(θ/2) + sin(θ/2)(n_x e23 - n_y e13 + n_z e12)
    let cos_half = half_angle.cos();
    let sin_half = half_angle.sin();

    vec![
        cos_half,       // scalar
        0.0,            // e1
        0.0,            // e2
        sin_half * nz,  // e12
        0.0,            // e3
        -sin_half * ny, // e13
        sin_half * nx,  // e23
        0.0,            // e123
    ]
}

/// Assertion helpers for JavaScript tests
#[wasm_bindgen(js_name = assertGA3Equal)]
pub fn assert_ga3_equal(a: &[f64], b: &[f64], tolerance: f64) -> TestResult {
    if a.len() != 8 || b.len() != 8 {
        return TestResult::fail_with_distance(
            f64::INFINITY,
            "Invalid GA3: expected 8 coefficients",
        );
    }

    let mut max_diff: f64 = 0.0;
    for i in 0..8 {
        let diff = (a[i] - b[i]).abs();
        if diff > max_diff {
            max_diff = diff;
        }
    }

    if max_diff < tolerance {
        TestResult::pass()
    } else {
        TestResult::fail_with_distance(max_diff, "GA3 values not equal within tolerance")
    }
}

/// Assert magnitude equals expected value
#[wasm_bindgen(js_name = assertMagnitude)]
pub fn assert_magnitude(coeffs: &[f64], expected: f64, tolerance: f64) -> TestResult {
    if coeffs.len() != 8 {
        return TestResult::fail_with_distance(
            f64::INFINITY,
            "Invalid GA3: expected 8 coefficients",
        );
    }

    let point = GA3::from_coefficients(coeffs.to_vec());
    let actual = point.magnitude();
    let diff = (actual - expected).abs();

    if diff < tolerance {
        TestResult::pass()
    } else {
        TestResult::fail_with_distance(
            diff,
            &format!("Magnitude mismatch: expected {}, got {}", expected, actual),
        )
    }
}

/// Assert the value is approximately zero
#[wasm_bindgen(js_name = assertZero)]
pub fn assert_zero(coeffs: &[f64], tolerance: f64) -> TestResult {
    if coeffs.len() != 8 {
        return TestResult::fail_with_distance(
            f64::INFINITY,
            "Invalid GA3: expected 8 coefficients",
        );
    }

    let point = GA3::from_coefficients(coeffs.to_vec());
    let mag = point.magnitude();

    if mag < tolerance {
        TestResult::pass()
    } else {
        TestResult::fail_with_distance(mag, "Value should be zero")
    }
}
