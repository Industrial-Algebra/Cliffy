/**
 * Cliffy Testing Showcase
 *
 * Demonstrates:
 * - Geometric invariant testing concepts
 * - Manifold constraint verification
 * - Probabilistic test patterns (rare/impossible events)
 * - Visual test result debugging
 *
 * This example visualizes the concepts from cliffy-test in JavaScript,
 * showing how geometric algebra enables powerful testing patterns.
 */

import init, {
  behavior,
  GeometricState,
  Rotor,
} from '@industrialalgebra/cliffy-core';

// =============================================================================
// Types
// =============================================================================

type TestStatus = 'pending' | 'running' | 'pass' | 'fail';
type TestType = 'invariant' | 'manifold' | 'probabilistic';

interface TestCase {
  id: string;
  name: string;
  type: TestType;
  description: string;
  code: string;
  status: TestStatus;
  duration: number;
  error?: string;
  samples?: Array<{ x: number; y: number; pass: boolean }>;
}

interface TestState {
  tests: TestCase[];
  selectedTestId: string | null;
  running: boolean;
  progress: number;
  totalPassed: number;
  totalFailed: number;
}

// =============================================================================
// Test Definitions
// =============================================================================

function createTestCases(): TestCase[] {
  return [
    {
      id: 'rotor-norm',
      name: 'Rotor Normalization Invariant',
      type: 'invariant',
      description: 'Rotors must have unit norm to represent valid rotations. This invariant verifies |R| = 1 for all generated rotors.',
      code: `#[invariant]
fn rotor_is_normalized(r: Rotor) -> bool {
    (r.norm() - 1.0).abs() < 1e-10
}`,
      status: 'pending',
      duration: 0,
      samples: [],
    },
    {
      id: 'sandwich-preserves-norm',
      name: 'Sandwich Product Preserves Magnitude',
      type: 'invariant',
      description: 'The sandwich product R v R† preserves the magnitude of vectors. This is fundamental to rotations being isometries.',
      code: `#[invariant]
fn sandwich_preserves_magnitude(
    r: Rotor,
    v: Vector
) -> bool {
    let rotated = r.sandwich(v);
    (rotated.norm() - v.norm()).abs() < 1e-9
}`,
      status: 'pending',
      duration: 0,
      samples: [],
    },
    {
      id: 'rotor-inverse',
      name: 'Rotor Inverse Property',
      type: 'invariant',
      description: 'For unit rotors, the reverse equals the inverse: R R† = 1. This enables efficient inverse computation.',
      code: `#[invariant]
fn rotor_reverse_is_inverse(r: Rotor) -> bool {
    let product = r * r.reverse();
    (product.scalar() - 1.0).abs() < 1e-10
        && product.bivector().norm() < 1e-10
}`,
      status: 'pending',
      duration: 0,
      samples: [],
    },
    {
      id: 'unit-sphere-manifold',
      name: 'Unit Sphere Manifold',
      type: 'manifold',
      description: 'Points must lie on the unit sphere S². The manifold constraint projects points back to the sphere surface.',
      code: `let sphere = Manifold::new(
    |p| p.norm() - 1.0,      // constraint
    |p| p.normalize()         // projection
);

sphere.verify(point)?;`,
      status: 'pending',
      duration: 0,
      samples: [],
    },
    {
      id: 'positive-definite-manifold',
      name: 'Positive Definite Manifold',
      type: 'manifold',
      description: 'Scalar values must be positive. This manifold ensures geometric states represent valid positive quantities.',
      code: `let positive = Manifold::new(
    |s| if s > 0.0 { 0.0 } else { -s },
    |s| s.abs()
);

positive.contains(value)?;`,
      status: 'pending',
      duration: 0,
      samples: [],
    },
    {
      id: 'rare-collision',
      name: 'Rare Event: Hash Collision',
      type: 'probabilistic',
      description: 'Hash collisions should be rare (probability < 1e-6). This test verifies the hash function quality using amari-flynn patterns.',
      code: `#[invariant_rare(1e-6)]
fn hash_collision_is_rare(
    a: NodeId,
    b: NodeId
) -> bool {
    a != b && hash(a) == hash(b)
}`,
      status: 'pending',
      duration: 0,
      samples: [],
    },
    {
      id: 'impossible-divergence',
      name: 'Impossible: CRDT Divergence',
      type: 'probabilistic',
      description: 'CRDTs must never diverge after merging the same observations. This is an impossible event that should never occur.',
      code: `#[invariant_impossible]
fn crdt_diverges_after_sync(
    a: ObservationSet,
    b: ObservationSet,
) -> bool {
    // Union merge bidirectionally
    a.merge(&b);
    b.merge(&a);
    // Should never diverge — union is a true join-semilattice
    a != b
}`,
      status: 'pending',
      duration: 0,
      samples: [],
    },
    {
      id: 'blend-continuity',
      name: 'Blend Interpolation Continuity',
      type: 'invariant',
      description: 'The .blend() function must produce continuous results. Small changes in t should produce small changes in output.',
      code: `#[invariant]
fn blend_is_continuous(
    a: GeometricState,
    b: GeometricState,
    t: f64  // 0..1
) -> bool {
    let epsilon = 0.001;
    let v1 = a.blend(&b, t);
    let v2 = a.blend(&b, t + epsilon);
    v1.distance(&v2) < epsilon * 10.0
}`,
      status: 'pending',
      duration: 0,
      samples: [],
    },
    {
      id: 'geometric-mean-convergence',
      name: 'Geometric Mean Convergence',
      type: 'invariant',
      description: 'The geometric mean of states must converge to a unique value regardless of merge order (commutativity).',
      code: `#[invariant]
fn geometric_mean_is_commutative(
    states: Vec<GeometricState>
) -> bool {
    let mean1 = geometric_mean(&states);
    let mean2 = geometric_mean(
        &states.iter().rev().collect()
    );
    mean1.distance(&mean2) < 1e-9
}`,
      status: 'pending',
      duration: 0,
      samples: [],
    },
    {
      id: 'emergent-stability',
      name: 'Emergent: System Stability',
      type: 'probabilistic',
      description: 'Under heavy load, the system should remain stable. This emergent property test runs many iterations to verify stability.',
      code: `#[emergent(10000)]
fn system_remains_stable(
    load: u32
) -> bool {
    let system = simulate_load(load);
    system.is_healthy()
        && system.memory_bounded()
        && system.no_deadlocks()
}`,
      status: 'pending',
      duration: 0,
      samples: [],
    },
  ];
}

// =============================================================================
// State
// =============================================================================

const state: TestState = {
  tests: createTestCases(),
  selectedTestId: null,
  running: false,
  progress: 0,
  totalPassed: 0,
  totalFailed: 0,
};

// =============================================================================
// Test Execution
// =============================================================================

async function runTest(test: TestCase): Promise<void> {
  test.status = 'running';
  test.samples = [];

  const startTime = performance.now();

  // Simulate test execution with sample generation
  const numSamples = test.type === 'probabilistic' ? 50 : 30;

  for (let i = 0; i < numSamples; i++) {
    await new Promise(resolve => setTimeout(resolve, 10));

    // Generate sample point
    const x = Math.random() * 100;
    const y = Math.random() * 100;

    // Determine pass/fail based on test type
    let pass = true;

    if (test.type === 'manifold') {
      // Manifold tests: points near the manifold pass
      const distFromManifold = Math.abs(Math.sqrt(x * x + y * y) - 50);
      pass = distFromManifold < 15;
    } else if (test.type === 'probabilistic') {
      // Probabilistic: mostly pass, rare failures for "rare" tests
      if (test.id === 'rare-collision') {
        pass = Math.random() > 0.02; // 2% failure rate
      } else if (test.id === 'impossible-divergence') {
        pass = true; // Should always pass
      } else {
        pass = Math.random() > 0.05;
      }
    } else {
      // Invariant tests: high pass rate
      pass = Math.random() > 0.03;
    }

    test.samples.push({ x, y, pass });

    // Update progress
    state.progress = (i + 1) / numSamples;
  }

  test.duration = performance.now() - startTime;

  // Determine overall test result
  const failedSamples = test.samples.filter(s => !s.pass).length;

  if (test.type === 'probabilistic' && test.id.includes('rare')) {
    // Rare events: some failures expected
    test.status = failedSamples < numSamples * 0.1 ? 'pass' : 'fail';
  } else if (test.type === 'probabilistic' && test.id.includes('impossible')) {
    // Impossible events: no failures allowed
    test.status = failedSamples === 0 ? 'pass' : 'fail';
  } else {
    // Regular tests: very few failures allowed
    test.status = failedSamples < 2 ? 'pass' : 'fail';
  }

  if (test.status === 'fail') {
    test.error = `${failedSamples} of ${numSamples} samples failed`;
  }
}

async function runAllTests(): Promise<void> {
  state.running = true;
  state.totalPassed = 0;
  state.totalFailed = 0;

  // Reset all tests
  for (const test of state.tests) {
    test.status = 'pending';
    test.duration = 0;
    test.error = undefined;
    test.samples = [];
  }

  // Run each test
  for (let i = 0; i < state.tests.length; i++) {
    state.progress = i / state.tests.length;
    await runTest(state.tests[i]);

    if (state.tests[i].status === 'pass') {
      state.totalPassed++;
    } else {
      state.totalFailed++;
    }

    renderApp();
  }

  state.progress = 1;
  state.running = false;
  renderApp();
}

// =============================================================================
// DOM Helpers
// =============================================================================

function createElement(
  tag: string,
  attrs: Record<string, string> = {},
  children: (Node | string)[] = []
): HTMLElement {
  const el = document.createElement(tag);
  for (const [key, value] of Object.entries(attrs)) {
    el.setAttribute(key, value);
  }
  for (const child of children) {
    if (typeof child === 'string') {
      el.appendChild(document.createTextNode(child));
    } else {
      el.appendChild(child);
    }
  }
  return el;
}

// =============================================================================
// Rendering
// =============================================================================

function renderTestList(): HTMLElement {
  const list = createElement('div', { class: 'test-list' });

  for (const test of state.tests) {
    const item = createElement('div', {
      class: `test-item ${state.selectedTestId === test.id ? 'selected' : ''}`,
    });

    item.onclick = () => {
      state.selectedTestId = test.id;
      renderApp();
    };

    // Status icon
    const statusIcon = createElement('div', { class: `test-status ${test.status}` });
    if (test.status === 'pass') statusIcon.textContent = '✓';
    else if (test.status === 'fail') statusIcon.textContent = '✗';
    else if (test.status === 'running') statusIcon.textContent = '●';
    else statusIcon.textContent = '○';
    item.appendChild(statusIcon);

    // Test info
    const info = createElement('div', { class: 'test-info' });
    info.appendChild(createElement('div', { class: 'test-name' }, [test.name]));
    info.appendChild(createElement('div', { class: `test-type ${test.type}` }, [test.type]));
    item.appendChild(info);

    // Duration
    if (test.duration > 0) {
      item.appendChild(
        createElement('div', { class: 'test-duration' }, [`${test.duration.toFixed(0)}ms`])
      );
    }

    list.appendChild(item);
  }

  return list;
}

function renderTestDetail(): HTMLElement {
  const container = createElement('div', { class: 'detail-panel' });

  const selectedTest = state.tests.find(t => t.id === state.selectedTestId);

  if (!selectedTest) {
    container.appendChild(
      createElement('div', { style: 'color: var(--text-dim); text-align: center;' }, [
        'Select a test to view details',
      ])
    );
    return container;
  }

  // Description
  const descSection = createElement('div', { class: 'detail-section' });
  descSection.appendChild(createElement('div', { class: 'detail-title' }, ['Description']));
  descSection.appendChild(createElement('div', { class: 'detail-content' }, [selectedTest.description]));
  container.appendChild(descSection);

  // Code
  const codeSection = createElement('div', { class: 'detail-section' });
  codeSection.appendChild(createElement('div', { class: 'detail-title' }, ['Test Code (Rust)']));
  const codeBlock = createElement('div', { class: 'code-block' });
  codeBlock.textContent = selectedTest.code;
  codeSection.appendChild(codeBlock);
  container.appendChild(codeSection);

  // Result
  if (selectedTest.status !== 'pending') {
    const resultSection = createElement('div', { class: 'detail-section' });
    resultSection.appendChild(createElement('div', { class: 'detail-title' }, ['Result']));

    const resultContent = createElement('div', { class: 'detail-content' });
    if (selectedTest.status === 'pass') {
      resultContent.style.color = 'var(--success)';
      resultContent.textContent = `PASSED - ${selectedTest.samples?.length || 0} samples verified in ${selectedTest.duration.toFixed(0)}ms`;
    } else if (selectedTest.status === 'fail') {
      resultContent.style.color = 'var(--danger)';
      resultContent.textContent = `FAILED - ${selectedTest.error}`;
    } else {
      resultContent.style.color = 'var(--warning)';
      resultContent.textContent = 'Running...';
    }
    resultSection.appendChild(resultContent);
    container.appendChild(resultSection);
  }

  return container;
}

function renderVisualization(): HTMLElement {
  const viz = createElement('div', { class: 'visualization' });

  const selectedTest = state.tests.find(t => t.id === state.selectedTestId);

  if (!selectedTest || !selectedTest.samples || selectedTest.samples.length === 0) {
    viz.appendChild(
      createElement('div', { style: 'color: var(--text-dim); text-align: center; padding-top: 80px;' }, [
        'Run tests to see sample visualization',
      ])
    );
    return viz;
  }

  // Draw manifold line for manifold tests
  if (selectedTest.type === 'manifold') {
    // Draw a circle representing the manifold
    for (let angle = 0; angle < Math.PI * 2; angle += 0.1) {
      const line = createElement('div', { class: 'manifold-line' });
      const x = 50 + Math.cos(angle) * 50;
      const y = 50 + Math.sin(angle) * 50;
      line.style.left = `${x}%`;
      line.style.top = `${y}%`;
      line.style.width = '4px';
      line.style.transform = `rotate(${angle + Math.PI / 2}rad)`;
      viz.appendChild(line);
    }
  }

  // Draw sample points
  for (const sample of selectedTest.samples) {
    const point = createElement('div', { class: `viz-point ${sample.pass ? 'pass' : 'fail'}` });
    point.style.left = `${sample.x}%`;
    point.style.top = `${sample.y}%`;
    viz.appendChild(point);
  }

  return viz;
}

function renderApp(): void {
  const app = document.getElementById('app');
  if (!app) return;

  app.textContent = '';

  const container = createElement('div', { class: 'app-container' });

  // Test runner header
  const runner = createElement('div', { class: 'test-runner' });

  // Stats
  const stats = createElement('div', { class: 'stats-grid' });

  const addStat = (value: string, label: string, cls: string = '') => {
    const item = createElement('div', { class: 'stat-item' });
    const valueEl = createElement('div', { class: `stat-value ${cls}` }, [value]);
    item.appendChild(valueEl);
    item.appendChild(createElement('div', { class: 'stat-label' }, [label]));
    stats.appendChild(item);
  };

  addStat(String(state.tests.length), 'Total Tests');
  addStat(String(state.totalPassed), 'Passed', 'pass');
  addStat(String(state.totalFailed), 'Failed', 'fail');
  addStat(
    state.running ? 'Running' : state.totalPassed + state.totalFailed > 0 ? 'Complete' : 'Ready',
    'Status',
    state.running ? 'running' : ''
  );

  runner.appendChild(stats);

  // Controls
  const controls = createElement('div', { class: 'controls' });

  const runBtn = createElement('button', { class: 'primary' }, ['Run All Tests']);
  runBtn.onclick = () => runAllTests();
  if (state.running) {
    runBtn.setAttribute('disabled', 'true');
  }
  controls.appendChild(runBtn);

  const resetBtn = createElement('button', {}, ['Reset']);
  resetBtn.onclick = () => {
    state.tests = createTestCases();
    state.selectedTestId = null;
    state.totalPassed = 0;
    state.totalFailed = 0;
    state.progress = 0;
    renderApp();
  };
  controls.appendChild(resetBtn);

  // Progress bar
  const progressBar = createElement('div', { class: 'progress-bar' });
  const progressFill = createElement('div', {
    class: `progress-fill ${state.running ? 'running' : ''}`,
  });
  progressFill.style.width = `${state.progress * 100}%`;
  progressBar.appendChild(progressFill);
  controls.appendChild(progressBar);

  // Status text
  const statusText = createElement('div', { class: 'status-text' });
  if (state.running) {
    statusText.textContent = `${Math.round(state.progress * 100)}%`;
  } else if (state.totalPassed + state.totalFailed > 0) {
    statusText.textContent = `${state.totalPassed}/${state.tests.length} passed`;
  }
  controls.appendChild(statusText);

  runner.appendChild(controls);
  container.appendChild(runner);

  // Panels
  const panels = createElement('div', { class: 'panels' });

  // Test list panel
  const listPanel = createElement('div', { class: 'panel' });
  listPanel.appendChild(createElement('h2', {}, ['Test Cases']));
  listPanel.appendChild(renderTestList());
  panels.appendChild(listPanel);

  // Detail panel
  const detailPanel = createElement('div', { class: 'panel' });
  detailPanel.appendChild(createElement('h2', {}, ['Test Details']));
  detailPanel.appendChild(renderTestDetail());

  // Visualization
  detailPanel.appendChild(createElement('h2', { style: 'margin-top: 15px;' }, ['Sample Visualization']));
  detailPanel.appendChild(renderVisualization());

  panels.appendChild(detailPanel);

  // Concept panel
  const conceptPanel = createElement('div', { class: 'panel' });
  const conceptBox = createElement('div', { class: 'concept-box' });
  conceptBox.appendChild(createElement('h3', {}, ['Geometric Testing Patterns']));
  const conceptP = createElement('p', {});
  conceptP.textContent =
    'cliffy-test provides three powerful testing patterns: (1) Invariants verify properties that must always hold, ' +
    'like rotor normalization. (2) Manifolds define geometric constraints with automatic projection. ' +
    '(3) Probabilistic tests (rare/impossible) catch edge cases using amari-flynn probability patterns. ' +
    'The visualization shows sample points - green for passing, red for failing.';
  conceptBox.appendChild(conceptP);
  conceptPanel.appendChild(conceptBox);
  panels.appendChild(conceptPanel);

  container.appendChild(panels);
  app.appendChild(container);
}

// =============================================================================
// Initialize
// =============================================================================

async function main() {
  await init();

  console.log('Cliffy Testing Showcase initialized');
  console.log('Demonstrating geometric invariants, manifolds, and probabilistic tests');

  // Select first test by default
  state.selectedTestId = state.tests[0].id;

  renderApp();
}

main().catch(console.error);
