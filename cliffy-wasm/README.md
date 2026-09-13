# cliffy-wasm

WebAssembly bindings for the Cliffy reactive framework.

## Overview

`cliffy-wasm` provides JavaScript/TypeScript bindings for Cliffy, enabling reactive UI development in browsers with geometric algebra state management.

## Installation

```bash
npm install @industrialalgebra/cliffy-core
```

## Quick Start

```typescript
import init, { behavior, html, mount } from '@industrialalgebra/cliffy-core';

await init();

const count = behavior(0);

const app = html`
  <div>
    <h1>Count: ${count}</h1>
    <button onclick=${() => count.update(n => n + 1)}>+</button>
    <button onclick=${() => count.update(n => n - 1)}>-</button>
  </div>
`;

mount(app, '#app');
```

## Core API

### Behavior<T>

Reactive state containers:

```typescript
import { behavior, combine } from '@industrialalgebra/cliffy-core';

const name = behavior('World');
const greeting = name.map(n => `Hello, ${n}!`);

name.subscribe(value => console.log('Name changed:', value));
name.set('Cliffy');

// Combine behaviors
const a = behavior(10);
const b = behavior(20);
const sum = combine(a, b, (x, y) => x + y);
```

### Event<T>

Discrete event streams:

```typescript
import { event } from '@industrialalgebra/cliffy-core';

const clicks = event();
const clickCount = clicks.fold(0, (acc, _) => acc + 1);

clicks.subscribe(e => console.log('Clicked!', e));
clicks.emit({ x: 100, y: 200 });
```

### Geometric State

Smooth interpolation and transformations:

```typescript
import { GeometricState, Rotor, Transform } from '@industrialalgebra/cliffy-core';

const state = GeometricState.fromVector(1, 0, 0);
const rotated = state.applyRotor(Rotor.xy(Math.PI / 2));

// Smooth interpolation
const mid = state.blend(target, 0.5);
```

### Distributed Protocols

Built-in CRDT and sync support:

```typescript
import { GeometricCRDT, VectorClock, SyncState } from '@industrialalgebra/cliffy-core';

const crdt = new GeometricCRDT('node-1', initialState);
crdt.add(5);

const merged = crdt.merge(otherCrdt);
```

### Testing Framework

Algebraic testing in JavaScript:

```typescript
import { testImpossible, testRare, Manifold } from '@industrialalgebra/cliffy-core';

const report = testImpossible('invariant name', () => {
  // Return true if invariant holds
  return someCondition();
}, 100);

expect(report.verified).toBe(true);
```

## Algebraic TSX

The `html` tagged template creates reactive DOM:

```typescript
import { html, mount, behavior } from '@industrialalgebra/cliffy-core';

const items = behavior(['Apple', 'Banana', 'Cherry']);

const app = html`
  <ul>
    ${items.map(list => list.map(item => html`<li>${item}</li>`))}
  </ul>
`;

mount(app, '#app');
```

Behaviors in templates automatically subscribe and update the DOM - no virtual DOM needed.

## Building from Source

```bash
# Install wasm-pack
cargo install wasm-pack

# Build the package
cd cliffy-wasm
wasm-pack build --target web

# The output is in pkg/
```

## TypeScript Support

Full TypeScript definitions are included:

```typescript
import type { Behavior, Event, GeometricState } from '@industrialalgebra/cliffy-core';

function createCounter(): Behavior<number> {
  return behavior(0);
}
```

## License

MIT
