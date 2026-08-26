# Cliffy

[![npm](https://img.shields.io/npm/v/@cliffy-ga/core)](https://www.npmjs.com/package/@cliffy-ga/core)
[![Netlify Status](https://api.netlify.com/api/v1/badges/211f9fc6-dbfb-4837-a0b0-11aaf4c04573/deploy-status)](https://app.netlify.com/projects/cliffy-ga/deploys)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg)](https://www.rust-lang.org/)
[![WASM](https://img.shields.io/badge/WebAssembly-ready-blueviolet.svg)](https://webassembly.org/)

A WASM-first reactive framework with classical FRP semantics, powered by geometric algebra.

**[Live Examples](https://cliffy-ga.netlify.app/)** | [Documentation](docs/) | [API Reference](docs/api-reference.md)

Build collaborative applications at scale where **distributed systems problems become geometric algebra problems** with closed-form solutions and guaranteed convergence.

## Install

```bash
npm install @cliffy-ga/core
```

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
- **Distributed State** - Geometric CRDTs with lattice join, vector clocks, and coordination-free merge
- **Geometric Foundation** - State transformations as geometric operations (hidden from users)
- **Multi-Language** - TypeScript, JavaScript, and PureScript bindings
- **No WASM? No problem** - [cliffy-tsukoshi](cliffy-tsukoshi/) provides pure TypeScript geometric state for mobile and constrained environments

## Algebraic TSX

Cliffy uses **Algebraic TSX** for rendering - a declarative approach where Behaviors automatically update the DOM:

### TypeScript (html tagged template)

```typescript
import init, { behavior, combine } from '@cliffy-ga/core';
import { html, mount } from '@cliffy-ga/core/html';

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
    [ h1_ [ text "Count: ", behaviorText count ]
    , button [ onClick \_ -> update (_ + 1) count ] [ text "+" ]
    , button [ onClick \_ -> update (_ - 1) count ] [ text "-" ]
    ]
```

### Vanilla JavaScript

```javascript
import init, { behavior, event } from '@cliffy-ga/core';

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
const clicks = event<MouseEvent>();

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
const volume = combine3(w, h, d, (w, h, d) => w * h * d);

// Conditional rendering
const show = behavior(true);
const message = when(show, () => 'Visible!');

// if/else for behaviors
const text = ifElse(isLoading, () => "Loading...", () => "Ready");
```

## Examples

| Example | Description | Run |
|---------|-------------|-----|
| [tsx-counter](examples/tsx-counter) | Basic counter with derived state | `npm run dev -w tsx-counter` |
| [tsx-todo](examples/tsx-todo) | Todo list with filtering | `npm run dev -w tsx-todo` |
| [tsx-forms](examples/tsx-forms) | Form validation patterns | `npm run dev -w tsx-forms` |
| [whiteboard](examples/whiteboard) | Collaborative drawing canvas | `npm run dev -w whiteboard` |
| [design-tool](examples/design-tool) | Shape manipulation with rotors | `npm run dev -w design-tool` |
| [multiplayer-game](examples/multiplayer-game) | Entity interpolation with latency sim | `npm run dev -w multiplayer-game` |
| [document-editor](examples/document-editor) | CRDT-based collaborative editing | `npm run dev -w document-editor` |
| [p2p-sync](examples/p2p-sync) | P2P sync with network partitions | `npm run dev -w p2p-sync` |
| [crdt-playground](examples/crdt-playground) | Interactive CRDT exploration | `npm run dev -w crdt-playground` |
| [geometric-transforms](examples/geometric-transforms) | Rotor rotations visualized | `npm run dev -w geometric-transforms` |
| [gpu-benchmark](examples/gpu-benchmark) | WebGPU vs CPU performance | `npm run dev -w gpu-benchmark` |
| [testing-showcase](examples/testing-showcase) | Algebraic testing patterns | `npm run dev -w testing-showcase` |
| [purescript-counter](examples/purescript-counter) | Counter in PureScript | See example README |
| [purescript-todo](examples/purescript-todo) | Todo list in PureScript | See example README |

## Project Structure

```
cliffy/
├── cliffy-core/           # Rust FRP implementation (85 tests)
│   └── src/
│       ├── behavior.rs    # Behavior<T> - continuous signals
│       ├── event.rs       # Event<T> - discrete occurrences
│       ├── combinators.rs # when, ifElse, combine
│       ├── component.rs   # Component model
│       ├── dataflow.rs    # Dataflow graph IR
│       └── geometric.rs   # GA conversion (internal)
├── cliffy-wasm/           # WASM bindings (@cliffy-ga/core on npm)
│   ├── src/
│   │   ├── lib.rs         # WASM exports
│   │   └── protocols.rs   # CRDT/VectorClock bindings
│   └── pkg/
│       ├── html.ts        # Algebraic TSX implementation
│       └── cliffy_wasm.js
├── cliffy-tsukoshi/       # Pure TypeScript geometric state (zero deps)
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
├── cliffy-protocols/      # Distributed state (42 tests)
├── cliffy-gpu/            # WebGPU/SIMD acceleration (18 tests)
├── cliffy-test/           # Algebraic testing framework (30 tests)
├── cliffy-loadtest/       # Scale testing simulator (15 tests)
├── tools/
│   └── create-cliffy/     # Project scaffolding CLI
├── examples/              # 14 example applications
└── docs/                  # Documentation
```

## cliffy-tsukoshi

For environments without WASM support (mobile apps, edge functions, etc.), **cliffy-tsukoshi** provides the geometric state management core as pure TypeScript:

```typescript
import { GeometricState, Rotor, ReactiveState } from 'cliffy-tsukoshi';

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

~530 lines, zero dependencies, 64 tests. See [cliffy-tsukoshi/README.md](cliffy-tsukoshi/README.md) for full documentation.

## Building from Source

### Prerequisites

- Rust (stable)
- wasm-pack (`cargo install wasm-pack`)
- Node.js 18+

### Build

```bash
npm run build          # Build WASM + post-process
npm run build:release  # Optimized release build
npm run dev            # Watch mode for development
```

### Test

```bash
# Run all tests
cargo test --workspace  # 250 tests

# Run specific crate tests
cargo test -p cliffy-core
cargo test -p cliffy-protocols
```

### Development Server

```bash
# Run an example
cd examples/tsx-counter
npm install
npm run dev
```

## Why Geometric Algebra?

Cliffy uses [Clifford Algebra](https://en.wikipedia.org/wiki/Clifford_algebra) (GA3 = Cl(3,0)) internally to represent state. This provides:

- **Unified representation**: Scalars, vectors, and higher-grade elements in one structure
- **Natural transformations**: Rotations, translations, scaling as algebraic operations
- **Conflict resolution**: Geometric mean provides coordination-free CRDT merging
- **Mathematical elegance**: Clean composition of transformations

**You never need to know this.** The geometric algebra is purely an implementation detail. The public API exposes familiar FRP primitives.

## Documentation

- [Getting Started](docs/getting-started.md) - Installation and first app
- [API Reference](docs/api-reference.md) - Complete API documentation
- [FRP Guide](docs/frp-guide.md) - Behavior, Event, and combinators in depth
- [Algebraic TSX Guide](docs/algebraic-tsx-guide.md) - Declarative UI patterns
- [Distributed State Guide](docs/distributed-state-guide.md) - CRDTs and sync
- [Testing Guide](docs/testing-guide.md) - Algebraic testing patterns
- [Migration Guide](docs/migration-guide.md) - Coming from React/Vue
- [PureScript FFI Patterns](docs/purescript-ffi-patterns.md) - PureScript integration
- [Architecture Decision Records](docs/architecture/) - Design rationale

## Roadmap

See [ROADMAP.md](ROADMAP.md) for the full development plan.

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 0 | Done | Algebraic Testing Framework |
| Phase 1 | Done | Geometric State Foundation |
| Phase 2 | Done | Distributed State (CRDT) |
| Phase 3 | Planned | Synchronization (WebRTC, persistence) |
| Phase 4 | Done | Algebraic TSX Components |
| Phase 5 | Done | Edge Computing (WebGPU) |
| Phase 6 | **v0.3.0** | Production Readiness |
| Phase 7 | Planned | Native Mobile (Trebek) |

## License

Apache-2.0. See [LICENSE](LICENSE). Industrial Algebra is the copyright holder;
contributions are made under the [CLA](https://github.com/Industrial-Algebra/.github/blob/main/CLA.md).
