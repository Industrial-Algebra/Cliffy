/**
 * Cliffy GPU Benchmark Example
 *
 * Demonstrates:
 * - WebGPU detection and capability checking
 * - SIMD/WASM feature detection
 * - Benchmarking geometric algebra operations
 * - Performance comparison visualization
 * - Fallback behavior verification
 */

import init, {
  behavior,
  Rotor,
  Transform,
  Translation,
  GeometricState,
} from '@industrialalgebra/cliffy-core';
import { html, mount } from '@industrialalgebra/cliffy-core/html';

// ============================================================================
// Types
// ============================================================================

interface Capabilities {
  webgpu: boolean;
  webgpuAdapter: string | null;
  simd: boolean;
  threads: boolean;
  wasmVersion: string;
}

interface BenchmarkResult {
  name: string;
  operations: number;
  cpuTime: number;
  gpuTime: number | null;
  simdTime: number | null;
  opsPerSecCpu: number;
  opsPerSecGpu: number | null;
  opsPerSecSimd: number | null;
  speedup: number | null;
}

type LogLevel = 'info' | 'success' | 'warning' | 'error';

// ============================================================================
// Feature Detection
// ============================================================================

async function detectCapabilities(): Promise<Capabilities> {
  const caps: Capabilities = {
    webgpu: false,
    webgpuAdapter: null,
    simd: false,
    threads: false,
    wasmVersion: '1.0',
  };

  // WebGPU detection
  if ('gpu' in navigator) {
    try {
      const adapter = await (navigator as any).gpu.requestAdapter();
      if (adapter) {
        caps.webgpu = true;
        const info = await adapter.requestAdapterInfo?.();
        caps.webgpuAdapter = info?.description || info?.vendor || 'Unknown GPU';
      }
    } catch (e) {
      console.log('WebGPU adapter request failed:', e);
    }
  }

  // SIMD detection (check if WASM SIMD is supported)
  try {
    // This is a simple SIMD detection using a minimal WASM module
    const simdTest = new Uint8Array([
      0, 97, 115, 109, 1, 0, 0, 0, 1, 5, 1, 96, 0, 1, 123, 3, 2, 1, 0, 10, 10, 1, 8, 0, 65, 0, 253,
      15, 253, 98, 11,
    ]);
    await WebAssembly.compile(simdTest);
    caps.simd = true;
  } catch (e) {
    caps.simd = false;
  }

  // SharedArrayBuffer / threads detection
  caps.threads = typeof SharedArrayBuffer !== 'undefined';

  return caps;
}

// ============================================================================
// Benchmark Functions
// ============================================================================

function benchmarkRotorCreation(iterations: number): number {
  const start = performance.now();
  for (let i = 0; i < iterations; i++) {
    const angle = (i / iterations) * Math.PI * 2;
    Rotor.xy(angle);
    Rotor.xz(angle);
    Rotor.yz(angle);
  }
  return performance.now() - start;
}

function benchmarkRotorComposition(iterations: number): number {
  const rotors: Rotor[] = [];
  for (let i = 0; i < 100; i++) {
    rotors.push(Rotor.xy((i / 100) * Math.PI));
  }

  const start = performance.now();
  for (let i = 0; i < iterations; i++) {
    const a = rotors[i % rotors.length];
    const b = rotors[(i + 50) % rotors.length];
    a.then(b);
  }
  return performance.now() - start;
}

function benchmarkRotorBlend(iterations: number): number {
  const rotorA = Rotor.xy(0);
  const rotorB = Rotor.xy(Math.PI);

  const start = performance.now();
  for (let i = 0; i < iterations; i++) {
    const t = (i % 100) / 100;
    rotorA.blend(rotorB, t);
  }
  return performance.now() - start;
}

function benchmarkTransformCreation(iterations: number): number {
  const start = performance.now();
  for (let i = 0; i < iterations; i++) {
    const angle = (i / iterations) * Math.PI * 2;
    const rotor = Rotor.xy(angle);
    const translation = new Translation(i % 100, (i % 50) - 25, 0);
    Transform.fromRotorAndTranslation(rotor, translation);
  }
  return performance.now() - start;
}

function benchmarkGeometricStateTransform(iterations: number): number {
  const states: GeometricState[] = [];
  const rotors: Rotor[] = [];

  for (let i = 0; i < 100; i++) {
    states.push(GeometricState.fromVector(i, i * 2, 0));
    rotors.push(Rotor.xy((i / 100) * Math.PI));
  }

  const start = performance.now();
  for (let i = 0; i < iterations; i++) {
    const state = states[i % states.length];
    const rotor = rotors[(i + 25) % rotors.length];
    state.applyRotor(rotor);
  }
  return performance.now() - start;
}

function benchmarkVectorOperations(iterations: number): number {
  const states: GeometricState[] = [];
  for (let i = 0; i < 100; i++) {
    states.push(GeometricState.fromVector(Math.random() * 100, Math.random() * 100, Math.random() * 100));
  }

  const start = performance.now();
  for (let i = 0; i < iterations; i++) {
    const s = states[i % states.length];
    s.asVector();
    s.magnitude();
  }
  return performance.now() - start;
}

// ============================================================================
// Main
// ============================================================================

async function main() {
  await init();

  // ==========================================================================
  // State
  // ==========================================================================

  const capabilities = behavior(null);
  const isRunning = behavior(false);
  const results = behavior([]);
  const logs = behavior([]);
  const iterations = behavior(10000);
  const selectedBenchmark = behavior('all');

  // ==========================================================================
  // Logging
  // ==========================================================================

  function log(level: LogLevel, message: string) {
    const currentLogs = logs.sample() as { level: LogLevel; message: string; time: string }[];
    const time = new Date().toLocaleTimeString();
    logs.set([...currentLogs, { level, message, time }].slice(-100));
  }

  // ==========================================================================
  // Detect Capabilities
  // ==========================================================================

  log('info', 'Detecting system capabilities...');

  detectCapabilities().then((caps) => {
    capabilities.set(caps);
    log('success', `WebGPU: ${caps.webgpu ? 'Available' : 'Not available'}`);
    if (caps.webgpuAdapter) {
      log('info', `GPU: ${caps.webgpuAdapter}`);
    }
    log('success', `SIMD: ${caps.simd ? 'Available' : 'Not available'}`);
    log('success', `Threads: ${caps.threads ? 'Available' : 'Not available'}`);
  });

  // ==========================================================================
  // Run Benchmarks
  // ==========================================================================

  async function runBenchmarks() {
    isRunning.set(true);
    results.set([]);

    const iters = iterations.sample() as number;
    const selected = selectedBenchmark.sample() as string;

    log('info', `Starting benchmarks with ${iters.toLocaleString()} iterations...`);

    const benchmarks: { name: string; fn: (n: number) => number }[] = [
      { name: 'Rotor Creation', fn: benchmarkRotorCreation },
      { name: 'Rotor Composition', fn: benchmarkRotorComposition },
      { name: 'Rotor Blend (SLERP)', fn: benchmarkRotorBlend },
      { name: 'Transform Creation', fn: benchmarkTransformCreation },
      { name: 'GeometricState Transform', fn: benchmarkGeometricStateTransform },
      { name: 'Vector Operations', fn: benchmarkVectorOperations },
    ];

    const filteredBenchmarks =
      selected === 'all' ? benchmarks : benchmarks.filter((b) => b.name === selected);

    const newResults: BenchmarkResult[] = [];

    for (const benchmark of filteredBenchmarks) {
      log('info', `Running: ${benchmark.name}`);

      // Warm up
      benchmark.fn(100);

      // Actual benchmark
      const cpuTime = benchmark.fn(iters);
      const opsPerSecCpu = (iters / cpuTime) * 1000;

      const result: BenchmarkResult = {
        name: benchmark.name,
        operations: iters,
        cpuTime,
        gpuTime: null, // GPU benchmarks would be separate
        simdTime: null, // SIMD is used internally by WASM
        opsPerSecCpu,
        opsPerSecGpu: null,
        opsPerSecSimd: null,
        speedup: null,
      };

      newResults.push(result);
      results.set([...newResults]);

      log('success', `${benchmark.name}: ${cpuTime.toFixed(2)}ms (${Math.round(opsPerSecCpu).toLocaleString()} ops/sec)`);

      // Small delay to allow UI to update
      await new Promise((r) => setTimeout(r, 10));
    }

    log('success', 'Benchmarks complete!');
    isRunning.set(false);
  }

  function clearResults() {
    results.set([]);
    logs.set([]);
    log('info', 'Results cleared');
  }

  // ==========================================================================
  // UI
  // ==========================================================================

  const app = html`
    <div class="container">
      <header>
        <h1>Cliffy GPU Benchmark</h1>
        <p class="subtitle">
          Performance testing for geometric algebra operations
        </p>
      </header>

      <!-- Capability Status -->
      <div class="status-grid">
        <div class="status-card">
          <h3>WebGPU</h3>
          ${capabilities.map((caps: Capabilities | null) =>
            caps === null
              ? html`<div class="status-value pending">Detecting...</div>`
              : caps.webgpu
              ? html`
                  <div class="status-value available">Available</div>
                  <div class="status-detail">${caps.webgpuAdapter}</div>
                `
              : html`
                  <div class="status-value unavailable">Not Available</div>
                  <div class="status-detail">Using CPU fallback</div>
                `
          )}
        </div>

        <div class="status-card">
          <h3>WASM SIMD</h3>
          ${capabilities.map((caps: Capabilities | null) =>
            caps === null
              ? html`<div class="status-value pending">Detecting...</div>`
              : caps.simd
              ? html`
                  <div class="status-value available">Available</div>
                  <div class="status-detail">128-bit vector operations</div>
                `
              : html`
                  <div class="status-value unavailable">Not Available</div>
                  <div class="status-detail">Using scalar fallback</div>
                `
          )}
        </div>

        <div class="status-card">
          <h3>Shared Memory</h3>
          ${capabilities.map((caps: Capabilities | null) =>
            caps === null
              ? html`<div class="status-value pending">Detecting...</div>`
              : caps.threads
              ? html`
                  <div class="status-value available">Available</div>
                  <div class="status-detail">Multi-threaded WASM ready</div>
                `
              : html`
                  <div class="status-value unavailable">Not Available</div>
                  <div class="status-detail">Single-threaded mode</div>
                `
          )}
        </div>

        <div class="status-card">
          <h3>Backend</h3>
          <div class="status-value available">WASM</div>
          <div class="status-detail">Rust-compiled WebAssembly</div>
        </div>
      </div>

      <!-- Benchmark Section -->
      <div class="benchmark-section">
        <h2>Geometric Algebra Benchmarks</h2>

        <div class="benchmark-controls">
          <div class="control-group">
            <label>Iterations:</label>
            <select
              onchange=${(e: Event) =>
                iterations.set(Number((e.target as HTMLSelectElement).value))}
            >
              <option value="1000">1,000</option>
              <option value="10000" selected>10,000</option>
              <option value="50000">50,000</option>
              <option value="100000">100,000</option>
            </select>
          </div>

          <div class="control-group">
            <label>Benchmark:</label>
            <select
              onchange=${(e: Event) =>
                selectedBenchmark.set((e.target as HTMLSelectElement).value)}
            >
              <option value="all">All Benchmarks</option>
              <option value="Rotor Creation">Rotor Creation</option>
              <option value="Rotor Composition">Rotor Composition</option>
              <option value="Rotor Blend (SLERP)">Rotor Blend (SLERP)</option>
              <option value="Transform Creation">Transform Creation</option>
              <option value="GeometricState Transform">GeometricState Transform</option>
              <option value="Vector Operations">Vector Operations</option>
            </select>
          </div>

          <button
            class=${isRunning.map((r: boolean) => `btn btn-primary ${r ? 'running' : ''}`)}
            onclick=${runBenchmarks}
            disabled=${isRunning}
          >
            ${isRunning.map((r: boolean) => (r ? 'Running...' : 'Run Benchmarks'))}
          </button>

          <button class="btn btn-secondary" onclick=${clearResults}>
            Clear
          </button>
        </div>

        <!-- Results Table -->
        <table class="results-table">
          <thead>
            <tr>
              <th>Benchmark</th>
              <th>Time (ms)</th>
              <th>Ops/sec</th>
              <th class="bar-cell">Performance</th>
            </tr>
          </thead>
          <tbody>
            ${results.map((res: BenchmarkResult[]) =>
              res.length === 0
                ? html`
                    <tr>
                      <td colspan="4" style="text-align: center; color: #666;">
                        No results yet. Click "Run Benchmarks" to start.
                      </td>
                    </tr>
                  `
                : res.map((r) => {
                    const maxOps = Math.max(...res.map((x) => x.opsPerSecCpu));
                    const barWidth = (r.opsPerSecCpu / maxOps) * 100;
                    return html`
                      <tr>
                        <td>${r.name}</td>
                        <td>${r.cpuTime.toFixed(2)}</td>
                        <td>${Math.round(r.opsPerSecCpu).toLocaleString()}</td>
                        <td class="bar-cell">
                          <div class="bar-container">
                            <div class="bar cpu" style="width: ${barWidth}%"></div>
                          </div>
                        </td>
                      </tr>
                    `;
                  })
            )}
          </tbody>
        </table>

        <div class="legend">
          <div class="legend-item">
            <div class="legend-color cpu"></div>
            <span>WASM (CPU/SIMD)</span>
          </div>
          <div class="legend-item">
            <div class="legend-color gpu"></div>
            <span>WebGPU (Future)</span>
          </div>
        </div>
      </div>

      <!-- Log Output -->
      <div class="benchmark-section">
        <h2>Output Log</h2>
        <div class="log-output">
          ${logs.map((entries: { level: LogLevel; message: string; time: string }[]) =>
            entries.length === 0
              ? html`<div class="log-entry">Waiting for benchmark...</div>`
              : entries.map(
                  (entry) =>
                    html`<div class="log-entry ${entry.level}">
                      [${entry.time}] ${entry.message}
                    </div>`
                )
          )}
        </div>
      </div>
    </div>
  `;

  mount(app, '#app');

  console.log('Cliffy GPU Benchmark initialized');
}

main().catch(console.error);
