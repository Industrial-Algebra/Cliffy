# Cliffy

[![npm](https://img.shields.io/npm/v/@industrialalgebra/cliffy-core)](https://www.npmjs.com/package/@industrialalgebra/cliffy-core)
[![Netlify Status](https://api.netlify.com/api/v1/badges/211f9fc6-dbfb-4837-a0b0-11aaf4c04573/deploy-status)](https://app.netlify.com/projects/cliffy-ga/deploys)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-nightly--2026--08--14-orange.svg)](rust-toolchain.toml)
[![WASM](https://img.shields.io/badge/WebAssembly-ready-blueviolet.svg)](https://webassembly.org/)

A WASM-first reactive framework with classical FRP semantics, powered by geometric algebra.

**[Live Examples](https://cliffy-ga.netlify.app/)** | [Documentation](docs/) | [API Reference](docs/api-reference.md)

Build collaborative applications at scale where **distributed systems problems become geometric algebra problems** — with a synchronization layer whose convergence is trivial by construction.

## Install

```bash
npm install @industrialalgebra/cliffy-core
```

> **Migrating from `@cliffy-ga/core`?** The `@cliffy-ga` org is retired as of
> 0.4.0. The new packages are `@industrialalgebra/cliffy-core` (WASM) and
> `@industrialalgebra/cliffy-tsukoshi` (pure TypeScript). See the
> [0.4.0 changelog](CHANGELOG.md) for breaking changes.

## Quick Start

```bash
# Create a new Cliffy project
npx create-cliffy my-app

# Or specify a template
npx create-cliffy my-app --template typescript-vite  # TypeScript + Vite (default)
npx create-cliffy my-app --template bun              # Bun runtime
npx create-cliffy my-app --template purescript       # PureScript + type-safe DSL

cd my-app
npm install
npm run dev
```

## Features

- **Classical FRP** - `Behavior<T>` (continuous) and `Event<T>` (discrete) following Conal Elliott's original semantics
- **Algebraic TSX** - Declarative UI with `html` tagged templates that auto-update when Behaviors change
- **WASM-First** - Core logic in Rust, runs anywhere via WebAssembly
- **Distributed State** - `ObservationSet`, a grow-only set CRDT (merge = union, a true semilattice) with **deterministic geometric projections**: Markley rotor consensus, scalar and vector means
- **Geometric Foundation** - State transformations as geometric operations (hidden from users)
- **Multi-Language** - TypeScript, JavaScript, and PureScript bindings
- **No WASM? No problem** - [cliffy-tsukoshi](cliffy-tsukoshi/) provides pure TypeScript geometric state plus a React component library for mobile and constrained environments

## Algebraic TSX

Cliffy uses **Algebraic TSX** for rendering - a declarative approach where Behaviors automatically update the DOM:

### TypeScript (html tagged template)

```typescript
import init, { behavior, combine } from '@industrialalgebra/cliffy-core';
import { html, mount } from '@industrialalgebra/cliffy-core/html';

async function main() {
  await init();

  // Create reactive state
  const count = behavior(0);
  const doubled = count.map(n => n * 2);

  // Event handlers
  const increment = () => count.update(n => n + 1);
  const decrement = () => count.update(n => n - 1);

  // Behaviors in templates automatically update the DOM
  const app = html`
    <div class="counter">
      <h1>Count: ${count}</h1>
      <button onclick=${decrement}>-</button>
      <button onclick=${increment}>+</button>
      <p>Doubled: ${doubled}</p>
    </div>
  `;

  mount(app, '#app');
}

main();
```

### PureScript (type-safe Html DSL)

```purescript
import Cliffy (behavior, update)
import Cliffy.Html (div, h1_, button, text, behaviorText, mount)
import Cliffy.Html.Attributes (className)
import Cliffy.Html.Events (onClick)

counter :: Effect Html
counter = do
  count <- behavior 0

  pure $ div [ className "counter" ]
    [ h1_ [ text "Clicks: ", behaviorText count ]
    , button [ onClick \_ -> update (_ + 1) count ] [ text "+" ]
    , button [ onClick \_ -> update (_ - 1) count ] [ text "-" ]
    ]
```

### Vanilla JavaScript

```javascript
import init, { behavior, event } from '@industrialalgebra/cliffy-core';

await init();

const count = behavior(0);
count.subscribe(n => {
  document.getElementById('count').textContent = n;
});

document.getElementById('increment').onclick = () => {
  count.update(n => n + 1);
};
```

## Core Concepts

### Behaviors (Time-Varying Values)

A `Behavior<T>` represents a value that changes over time - like a spreadsheet cell that updates automatically.

```typescript
const count = behavior(0);
count.sample();              // Get current value: 0
count.set(10);               // Set directly
count.update(n => n + 1);    // Transform: 11

// Derived behaviors update automatically
const doubled = count.map(n => n * 2);  // 22

// Combine multiple behaviors
const sum = combine(a, b, (x, y) => x + y);
```

### Events (Discrete Occurrences)

An `Event<T>` represents discrete occurrences over time - like button clicks or network responses.

```typescript
const clicks = event();

clicks.subscribe(e => console.log('Clicked at', e.clientX));
clicks.emit(mouseEvent);

// Transform events
const xPositions = clicks.map(e => e.clientX);
const leftClicks = clicks.filter(e => e.button === 0);

// Accumulate into Behavior
const clickCount = clicks.fold(0, (acc, _) => acc + 1);
```

### Combinators

```typescript
// Combine multiple behaviors
const width = behavior(10);
const height = behavior(5);
const area = combine(width, height, (w, h) => w * h);

// Conditional rendering
const show = behavior(true);
const message = when(show, () => 'Visible!');

// if/else for behaviors
const text = ifElse(isLoading, () => "Loading...", () => "Ready");
```

## Distributed State (0.4.0)

The distributed layer is built on one sound idea: **the CRDT is the set; the
geometry is a deterministic projection of it.**

```rust
use cliffy_protocols::{scalar_mean, Observation, ObservationSet};
use uuid::Uuid;

let sensor1 = Uuid::new_v4();
let mut a = ObservationSet::new();
a.insert(Observation::new_scalar(sensor1, 1, 5.0));
a.insert(Observation::new_scalar(sensor1, 2, 10.0));

let mut b = a.clone();
b.insert(Observation::new_scalar(Uuid::new_v4(), 1, 7.5));

// Merge is set union — associative, commutative, idempotent.
// Convergence is trivial by construction; it cannot annihilate data.
a.merge(&b);

// Geometry happens at read time, deterministically:
assert_eq!(scalar_mean(&a), Some(7.5));
```

Rotor observations get principled consensus via the Markley eigen-mean (an
in-house Jacobi eigensolve — no external LAPACK dependency), exposed over WASM
as `ObservationSet.observeRotor` / `rotorConsensus`. See the
[Distributed State Guide](docs/distributed-state-guide.md).

## Examples

| Example | Description | Run |
|---------|-------------|-----|
| [tsx-counter](examples/tsx-counter) | Basic counter with derived state | `npm run dev -w tsx-counter` |
| [tsx-todo](examples/tsx-todo) | Todo list with filtering | `npm run dev -w tsx-todo` |
| [tsx-forms](examples/tsx-forms) | Form validation patterns | `npm run dev -w tsx-forms` |
| [whiteboard](examples/whiteboard) | Collaborative drawing canvas | `npm run dev -w whiteboard` |
| [design-tool](examples/design-tool) | Shape manipulation with rotors | `npm run dev -w design-tool` |
| [multiplayer-game](examples/multiplayer-game) | Entity interpolation with latency sim | `npm run dev -w multiplayer-game` |
| [document-editor](examples/document-editor) | ObservationSet-based collaborative editing | `npm run dev -w document-editor` |
| [p2p-sync](examples/p2p-sync) | P2P sync with network partitions | `npm run dev -w p2p-sync` |
| [crdt-playground](examples/crdt-playground) | Interactive ObservationSet + projection probes | `npm run dev -w crdt-playground` |
| [geometric-transforms](examples/geometric-transforms) | Rotor rotations visualized | `npm run dev -w geometric-transforms` |
| [gpu-benchmark](examples/gpu-benchmark) | WebGPU vs CPU performance | `npm run dev -w gpu-benchmark` |
| [testing-showcase](examples/testing-showcase) | Algebraic testing patterns | `npm run dev -w testing-showcase` |
| [alive-button](examples/alive-button) | Living UI (experimental, cliffy-alive) | See example README |
| [alive-garden](examples/alive-garden) | Cellular automata garden (experimental) | See example README |
| [purescript-counter](examples/purescript-counter) | Counter in PureScript | See example README |
| [purescript-todo](examples/purescript-todo) | Todo list in PureScript | See example README |

## Project Structure

```
cliffy/
├── cliffy-core/           # Rust FRP implementation
│   └── src/
│       ├── behavior.rs    # Behavior<T> - continuous signals
│       ├── event.rs       # Event<T> - discrete occurrences
│       ├── combinators.rs # when, ifElse, combine
│       ├── component.rs   # Component model
│       ├── dataflow.rs    # Dataflow graph IR
│       └── geometric.rs   # GA conversion (internal)
├── cliffy-wasm/           # WASM bindings (@industrialalgebra/cliffy-core on npm)
│   ├── src/
│   │   ├── lib.rs         # WASM exports
│   │   └── protocols.rs   # ObservationSet bindings
│   └── pkg/
│       ├── html.ts        # Algebraic TSX implementation
│       └── cliffy_wasm.js
├── cliffy-tsukoshi/       # Pure TypeScript geometric state + React components
│   └── src/
│       ├── ga3.ts         # GA3 multivector operations
│       ├── rotor.ts       # Rotations with SLERP
│       ├── transform.ts   # Rotation + translation
│       └── state.ts       # GeometricState + ReactiveState
├── cliffy-purescript/     # PureScript bindings
│   └── src/
│       ├── Cliffy.purs    # FRP primitives (Behavior, Event)
│       └── Cliffy/
│           ├── Html.purs  # Type-safe Html DSL
│           └── Foreign.js # FFI bridge
├── cliffy-protocols/      # ObservationSet CRDT + geometric projections
│   └── src/
│       ├── observation.rs # ObservationSet (merge = union)
│       ├── projection.rs  # scalar/vector means, rotor consensus
│       └── eigen.rs       # Deterministic Jacobi eigensolve
├── cliffy-gpu/            # WebGPU/SIMD acceleration
├── cliffy-test/           # Algebraic testing framework
├── cliffy-loadtest/       # Scale testing simulator
├── cliffy-alive/          # Living UI / cellular automata (highly experimental)
├── tools/
│   └── create-cliffy/     # Project scaffolding CLI
├── examples/              # Example applications (see table above)
└── docs/                  # Documentation
```

## cliffy-tsukoshi

For environments without WASM support (mobile apps, edge functions, etc.), **cliffy-tsukoshi** provides the geometric state management core as pure TypeScript, plus a React component library:

```typescript
import { GeometricState, Rotor, ReactiveState } from '@industrialalgebra/cliffy-tsukoshi';

// Smooth interpolation between states
const current = GeometricState.fromVector(0, 0, 0);
const target = GeometricState.fromVector(100, 50, 0);
const midway = current.blend(target, 0.5);  // (50, 25, 0)

// Rotations via rotors
const rotate90 = Rotor.fromAxisAngle('xy', Math.PI / 2);
const rotated = current.applyRotor(rotate90);

// Reactive wrapper with subscriptions
const state = new ReactiveState(current);
state.subscribe(s => updateUI(s));
state.blendTo(target, 0.3);
```

Zero dependencies, 113 tests. See [cliffy-tsukoshi/README.md](cliffy-tsukoshi/README.md) for full documentation.

## Building from Source

### Prerequisites

- Rust via [rustup](https://rustup.rs/) — the repo pins `nightly-2026-08-14` in
  `rust-toolchain.toml` (installed automatically; a floating-nightly and a
  stable lane also run in CI)
- wasm-pack (`cargo install wasm-pack`)
- Node.js 20+
- Or use the Nix devShell: `nix develop` (see [flake.nix](flake.nix))

### Build

```bash
npm run build          # Build WASM + post-process
npm run build:release  # Optimized release build
npm run dev            # Watch mode for development
```

### Test

```bash
cargo nextest run --workspace   # 214 tests
cargo test --doc --workspace    # 31 doctests

# Run specific crate tests
cargo nextest run -p cliffy-core
cargo nextest run -p cliffy-protocols

# TypeScript
cd cliffy-tsukoshi && npm test  # 113 tests
```

### Development Server

```bash
# Run an example (from examples/)
cd examples
npm run dev -w tsx-counter
```

## Why Geometric Algebra?

Cliffy uses [Clifford Algebra](https://en.wikipedia.org/wiki/Clifford_algebra) (GA3 = Cl(3,0)) internally to represent state. This provides:

- **Unified representation**: Scalars, vectors, and higher-grade elements in one structure
- **Natural transformations**: Rotations, translations, scaling as algebraic operations
- **Principled consensus**: Rotor averaging via the Markley eigen-mean, computed deterministically from a set of observations
- **Mathematical elegance**: Clean composition of transformations

**You never need to know this.** The geometric algebra is purely an implementation detail. The public API exposes familiar FRP primitives — and the distributed layer's merge is deliberately boring (set union); the geometry only appears at projection time, where it cannot break convergence.

## Documentation

- [Getting Started](docs/getting-started.md) - Installation and first app
- [API Reference](docs/api-reference.md) - Complete API documentation
- [FRP Guide](docs/frp-guide.md) - Behavior, Event, and combinators in depth
- [Algebraic TSX Guide](docs/algebraic-tsx-guide.md) - Declarative UI patterns
- [Distributed State Guide](docs/distributed-state-guide.md) - ObservationSet and geometric projections
- [Testing Guide](docs/testing-guide.md) - Algebraic testing patterns
- [Migration Guide](docs/migration-guide.md) - Coming from React/Vue
- [PureScript FFI Patterns](docs/purescript-ffi-patterns.md) - PureScript integration
- [Architecture Decision Records](docs/architecture/) - Design rationale

## Roadmap

See [ROADMAP.md](ROADMAP.md) for the full development plan.

| Version | Focus | Status |
|---------|-------|--------|
| 0.3.x | Production readiness (Algebraic TSX, PureScript, GPU) | Released |
| **0.4.0** | **Sound distributed state — ObservationSet + deterministic projections, `@industrialalgebra` npm org** | **This release** |
| 0.5.0 | Production polish — memory leaks, type holes, performance | Planned |
| 0.6.0 | API coherence — unified naming, docs, E2E tests | Planned |
| 1.0.0 | Stable release — semver commitment, crates.io + npm | Planned |

## License

Apache-2.0. See [LICENSE](LICENSE). Industrial Algebra is the copyright holder;
contributions are made under the [CLA](https://github.com/Industrial-Algebra/.github/blob/main/CLA.md).
