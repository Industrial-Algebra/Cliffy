# Cliffy API Reference

This document provides a complete API reference for Cliffy's FRP primitives and rendering systems.

## Overview

Cliffy provides two layers:
1. **FRP Primitives** — `Behavior` and `Event` for reactive state
2. **Rendering** — Algebraic TSX for declarative UI

```typescript
import init, { behavior, event, combine, wedge } from '@industrialalgebra/cliffy-core';
import { html, mount } from '@industrialalgebra/cliffy-core/html';

await init(); // Initialize WASM module

// State (FRP)
const count = behavior(0);

// UI (Algebraic TSX)
const app = html`<button onclick=${() => count.update(n => n + 1)}>
  Clicks: ${count}
</button>`;

mount(app, '#app');
```

---

## FRP Primitives

### behavior(initialValue)

Creates a time-varying value that always has a current value.

```typescript
const count = behavior(0);
const user = behavior<User | null>(null);
const items = behavior<string[]>([]);
```

**Methods:**

| Method | Description | Example |
|--------|-------------|---------|
| `sample()` | Get current value | `count.sample()` → `0` |
| `set(value)` | Set to new value | `count.set(10)` |
| `update(fn)` | Transform value | `count.update(n => n + 1)` |
| `subscribe(fn)` | React to changes | `count.subscribe(n => console.log(n))` |
| `map(fn)` | Derive new behavior | `count.map(n => n * 2)` |

**Important:** `behavior.map()` transforms the behavior itself, not an array inside it. For arrays, use `behavior.map(arr => arr.map(...))`.

### event()

Creates a discrete event stream for things that happen at specific moments.

```typescript
const clicks = event<MouseEvent>();
const messages = event<string>();
const submits = event<FormData>();
```

**Methods:**

| Method | Description | Example |
|--------|-------------|---------|
| `emit(value)` | Fire an event | `clicks.emit(mouseEvent)` |
| `subscribe(fn)` | Listen for events | `clicks.subscribe(e => handle(e))` |
| `map(fn)` | Transform values | `clicks.map(e => e.clientX)` |
| `filter(fn)` | Select events | `clicks.filter(e => e.button === 0)` |
| `merge(other)` | Combine streams | `clicks.merge(touches)` |
| `fold(init, fn)` | Accumulate into Behavior | `clicks.fold(0, (n, _) => n + 1)` |

---

## Combinators

### combine(a, b, fn)

Combine multiple behaviors into one derived behavior.

```typescript
const width = behavior(10);
const height = behavior(20);
const area = combine(width, height, (w, h) => w * h);
// area updates when either width or height changes
```

### condition.project(fn)

Project a value through a boolean condition. Returns the value when true, null when false.

```typescript
const isLoggedIn = behavior(false);
const greeting = isLoggedIn.project(() => 'Welcome back!');
// greeting is 'Welcome back!' when logged in, null when not
```

### condition.select(thenFn, elseFn)

Select between two values based on a boolean condition.

```typescript
const isDark = behavior(false);
const theme = isDark.select(() => 'dark', () => 'light');
// theme is 'dark' or 'light' based on isDark
```

### wedge(...behaviors)

Combine 3 or more behaviors using the GA-inspired wedge product.

```typescript
const width = behavior(10);
const height = behavior(20);
const depth = behavior(5);
const volume = wedge(width, height, depth).map((w, h, d) => w * h * d);
// volume updates when any dimension changes
```

---

## Rendering: Algebraic TSX

Cliffy provides two approaches to reactive UI rendering.

### TypeScript: html Tagged Template

```typescript
import { behavior } from '@industrialalgebra/cliffy-core';
import { html, mount } from '@industrialalgebra/cliffy-core/html';

const count = behavior(0);

const app = html`
  <div class="counter">
    <h1>Count: ${count}</h1>
    <button onclick=${() => count.update(n => n + 1)}>+</button>
    <button onclick=${() => count.update(n => n - 1)}>-</button>
  </div>
`;

mount(app, '#app');
```

**Key features:**
- Behaviors in `${}` automatically update the DOM
- Event handlers receive native events
- No virtual DOM or reconciliation

**html Template Syntax:**

| Interpolation | Effect |
|---------------|--------|
| `${behavior}` | Text content that updates automatically |
| `${string}` | Static text content |
| `${() => void}` | Event handler (for `onclick`, `oninput`, etc.) |
| `class="${behavior}"` | Dynamic attribute |

### PureScript: Type-Safe Html DSL

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

**Cliffy.Html Elements:**

| Function | Description |
|----------|-------------|
| `div`, `div_` | Container (with/without attrs) |
| `span`, `span_` | Inline container |
| `button`, `button_` | Button element |
| `h1_`, `h2_`, `p_` | Text elements (no attrs) |
| `input`, `form` | Form elements |
| `ul`, `ol`, `li` | List elements |
| `text` | Static text node |
| `behaviorText` | Reactive text (updates automatically) |
| `behaviorHtml` | Reactive HTML content |
| `mount` | Attach to DOM |

**Cliffy.Html.Attributes:**

| Function | Description |
|----------|-------------|
| `className` | CSS class |
| `id` | Element ID |
| `style` | Inline styles |
| `href`, `src`, `alt` | Link/media attributes |
| `type_`, `value`, `placeholder` | Input attributes |
| `disabled`, `checked`, `hidden` | Boolean attributes |
| `dataAttr` | data-* attributes |

**Cliffy.Html.Events:**

| Function | Description |
|----------|-------------|
| `onClick` | Click handler |
| `onChange`, `onInput` | Input handlers |
| `onSubmit` | Form submit |
| `onFocus`, `onBlur` | Focus handlers |
| `onKeyDown`, `onKeyUp` | Keyboard handlers |
| `onMouseEnter`, `onMouseLeave` | Mouse handlers |

---

## Direct DOM Projection

For lower-level control, use `DOMProjection` directly. See [DOM Projection Guide](./dom-projection-guide.md).

```typescript
import { behavior, DOMProjection } from '@industrialalgebra/cliffy-core';

const count = behavior(0);
const display = document.getElementById('display')!;

// Create projection
const textProj = DOMProjection.text(display, s => `Count: ${s}`);

// Connect to behavior
count.subscribe(value => textProj.update(String(value)));
```

**DOMProjection Types:**

| Method | Description |
|--------|-------------|
| `DOMProjection.text(el, fn)` | Text content |
| `DOMProjection.style(el, prop, fn)` | CSS property |
| `DOMProjection.attribute(el, attr, fn)` | HTML attribute |
| `DOMProjection.classToggle(el, cls, fn)` | Toggle CSS class |
| `DOMProjection.data(el, key, fn)` | data-* attribute |

---

## Helper Functions

### stateToTransform(x, y, z, rotation, scale)

Create CSS transform string.

```typescript
const transform = stateToTransform(100, 50, 0, 45, 1.5);
// "translate3d(100px, 50px, 0px) rotate(45deg) scale(1.5)"
```

### stateToColor(r, g, b, a)

Create RGBA color string (values 0-1).

```typescript
const color = stateToColor(1.0, 0.5, 0.0, 0.8);
// "rgba(255, 127, 0, 0.8)"
```

---

## TypeScript vs PureScript

| Operation | TypeScript | PureScript |
|-----------|------------|------------|
| Create behavior | `behavior(0)` | `behavior 0` |
| Update | `count.update(n => n + 1)` | `update (_ + 1) count` |
| Sample | `count.sample()` | `sample count` |
| Subscribe | `count.subscribe(cb)` | `subscribe cb count` |
| Map | `count.map(f)` | `mapBehavior f count` |
| Create event | `event<T>()` | `event` |
| Emit | `clicks.emit(e)` | `emit e clicks` |
| Fold | `clicks.fold(0, f)` | `fold 0 f clicks` |

PureScript uses curried functions with the behavior/event as the last argument, enabling point-free style.

---

## Quick Reference

### State Management

```typescript
// Create
const count = behavior(0);
const clicks = event<void>();

// Read
count.sample();

// Write
count.set(10);
count.update(n => n + 1);
clicks.emit(undefined);

// React
count.subscribe(n => console.log(n));
clicks.subscribe(() => count.update(n => n + 1));

// Derive
const doubled = count.map(n => n * 2);
const area = combine(width, height, (w, h) => w * h);
const clickCount = clicks.fold(0, (n, _) => n + 1);
```

### Rendering (TypeScript)

```typescript
import { html, mount } from '@industrialalgebra/cliffy-core/html';

const app = html`
  <div>
    <span>${behavior}</span>
    <button onclick=${handler}>Click</button>
  </div>
`;

mount(app, '#app');
```

### Rendering (PureScript)

```purescript
import Cliffy.Html (div, span, button, text, behaviorText, mount)
import Cliffy.Html.Events (onClick)

let app = div []
      [ span [] [ behaviorText count ]
      , button [ onClick handler ] [ text "Click" ]
      ]

mount app "#app"
```

---

## Next Steps

- [Getting Started](./getting-started.md) — Build your first app
- [FRP Guide](./frp-guide.md) — Reactive patterns in depth
- [DOM Projection Guide](./dom-projection-guide.md) — Lower-level rendering control
- [Examples](../examples/) — Full applications
