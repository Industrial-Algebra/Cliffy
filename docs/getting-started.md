# Getting Started with Cliffy

Cliffy is a reactive UI framework built on geometric algebra. This guide will get you up and running with your first Cliffy application.

## Prerequisites

- Node.js 18+ (for web development)

## Installation

Use the scaffolding tool to create a new project:

```bash
npx create-cliffy my-app --template typescript-vite
cd my-app
npm install
npm run dev
```

Available templates:
- `typescript-vite` (default) - TypeScript + Vite
- `bun` - Bun runtime
- `purescript` - PureScript with type-safe Html DSL

## Quick Start with Algebraic TSX

The fastest way to build reactive UIs is with the `html` tagged template:

```typescript
import { behavior } from '@industrialalgebra/cliffy-core';
import { html, mount } from '@industrialalgebra/cliffy-core/html';

// Create reactive state
const count = behavior(0);

// Build reactive UI - Behaviors automatically update the DOM
const app = html`
  <div class="counter">
    <h1>Count: ${count}</h1>
    <button onclick=${() => count.update(n => n + 1)}>+</button>
    <button onclick=${() => count.update(n => n - 1)}>-</button>
  </div>
`;

// Mount to DOM, returns cleanup function
const cleanup = mount(app, '#app');
```

Key features:
- **No virtual DOM** - Behaviors subscribe directly to DOM nodes
- **Automatic updates** - Values in `${}` update when Behaviors change
- **Event handlers** - Use `onclick`, `onchange`, etc. directly
- **Cleanup included** - `mount()` returns a cleanup function

### PureScript Alternative

For full type safety, use the PureScript DSL:

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

## Core Concepts

Cliffy uses **Functional Reactive Programming (FRP)** with two fundamental primitives:

### Behavior: Time-Varying Values

A `Behavior` represents a value that changes over time. Think of it as a cell in a spreadsheet that can update.

```typescript
import { Behavior } from '@industrialalgebra/cliffy-core';

// Create a behavior with initial value
const count = new Behavior(0);

// Read the current value
console.log(count.sample());  // 0

// Update the value with a transform function
count.update(n => n + 1);
console.log(count.sample());  // 1

// Set directly
count.set(100);
console.log(count.sample());  // 100
```

### Event: Discrete Occurrences

An `Event` represents discrete happenings like clicks or key presses.

```typescript
import { Event } from '@industrialalgebra/cliffy-core';

// Create an event stream
const clicks = new Event();

// Subscribe to events
clicks.subscribe(value => {
    console.log('Clicked!', value);
});

// Emit an event
clicks.emit({ x: 100, y: 200 });
```

### Reactive Subscriptions

Behaviors notify subscribers when they change:

```typescript
const count = new Behavior(0);

// Subscribe to changes
count.subscribe(value => {
    console.log('Count is now:', value);
});

count.set(1);  // Logs: "Count is now: 1"
count.set(2);  // Logs: "Count is now: 2"
```

### Derived Behaviors with `map`

Create new behaviors that automatically update:

```typescript
const count = new Behavior(5);
const doubled = count.map(n => n * 2);

console.log(doubled.sample());  // 10

count.set(10);
console.log(doubled.sample());  // 20 - Automatically updated!
```

> **Note:** `Behavior.map()` is not the same as `Array.map()`. It's a functor operation that creates a new Behavior which stays synchronized with the source. The transform function is called whenever the source changes, not once over a collection. This follows classical FRP semantics where Behaviors are functors, and `map` lifts a pure function into the reactive context.
>
> This is also a target for future optimization—the reactive graph can potentially batch, memoize, or even compile chains of `map` operations.

### Combining Behaviors

Combine multiple behaviors into one:

```typescript
import { Behavior, combine } from '@industrialalgebra/cliffy-core';

const width = new Behavior(10);
const height = new Behavior(20);
const area = combine(width, height, (w, h) => w * h);

console.log(area.sample());  // 200

width.set(15);
console.log(area.sample());  // 300 - Automatically recalculated!
```

## Your First App: Counter

Here's a complete counter example:

```typescript
import { Behavior, Event } from '@industrialalgebra/cliffy-core';

// State
const count = new Behavior(0);

// Derived state
const displayText = count.map(n => `Count: ${n}`);

// Subscribe to render
displayText.subscribe(text => {
    document.getElementById('display')!.textContent = text;
});

// Events
const increment = new Event<void>();
const decrement = new Event<void>();

// Wire events to state
increment.subscribe(() => count.update(n => n + 1));
decrement.subscribe(() => count.update(n => n - 1));

// Connect to DOM
document.getElementById('inc')!.onclick = () => increment.emit();
document.getElementById('dec')!.onclick = () => decrement.emit();
```

```html
<div id="display">Count: 0</div>
<button id="inc">+</button>
<button id="dec">-</button>
```

## Combinators

Cliffy provides combinators for common patterns:

### `when` - Conditional Values

```typescript
import { Behavior, when } from '@industrialalgebra/cliffy-core';

const showMessage = new Behavior(true);
const message = when(showMessage, () => "Hello!");

console.log(message.sample());  // "Hello!"

showMessage.set(false);
console.log(message.sample());  // null
```

### `ifElse` - Conditional Selection

```typescript
import { Behavior, ifElse } from '@industrialalgebra/cliffy-core';

const isDarkMode = new Behavior(false);
const theme = ifElse(isDarkMode, () => "dark", () => "light");

console.log(theme.sample());  // "light"

isDarkMode.set(true);
console.log(theme.sample());  // "dark"
```

### `fold` - Accumulate Events

```typescript
import { Event } from '@industrialalgebra/cliffy-core';

const clicks = new Event<void>();
const clickCount = clicks.fold(0, (n, _) => n + 1);

console.log(clickCount.sample());  // 0

clicks.emit();
clicks.emit();
console.log(clickCount.sample());  // 2
```

## Event Transformations

### `map` - Transform Event Values

```typescript
const numbers = new Event<number>();
const doubled = numbers.map(n => n * 2);

doubled.subscribe(n => console.log('Got:', n));

numbers.emit(5);  // Logs: "Got: 10"
```

### `filter` - Select Events

```typescript
const numbers = new Event<number>();
const evens = numbers.filter(n => n % 2 === 0);

evens.subscribe(n => console.log('Even:', n));

numbers.emit(1);  // Nothing
numbers.emit(2);  // Logs: "Even: 2"
numbers.emit(3);  // Nothing
numbers.emit(4);  // Logs: "Even: 4"
```

### `merge` - Combine Event Streams

```typescript
const clicks = new Event<string>();
const keys = new Event<string>();
const inputs = clicks.merge(keys);

inputs.subscribe(s => console.log('Input:', s));

clicks.emit('click');  // Logs: "Input: click"
keys.emit('key');      // Logs: "Input: key"
```

## Geometric State (Advanced)

For animations, physics, and explicit geometric control, use `GeometricState`:

```typescript
import { GeometricState, Rotor, Translation } from '@industrialalgebra/cliffy-core';

// Create state from a 3D position
const pos = GeometricState.fromVector(1, 0, 0);

// Apply a 90-degree rotation in the XY plane (around Z axis)
const rot = Rotor.xy(Math.PI / 2);
const rotated = pos.applyRotor(rot);

// Apply a translation
const trans = new Translation(1, 0, 0);
const translated = rotated.applyTranslation(trans);

// Read the result
const [x, y, z] = translated.asVector();
console.log(x, y, z);  // ~1, ~1, ~0
```

## DOM Projections

For efficient DOM updates without virtual DOM, use `DOMProjection`. Projections map state changes directly to specific DOM properties—no diffing required.

### Projection Types

```typescript
import { Behavior, DOMProjection } from '@industrialalgebra/cliffy-core';

const count = new Behavior(0);
const isActive = new Behavior(false);

// Text content projection
const display = document.getElementById('display')!;
const textProj = DOMProjection.text(display);
count.subscribe(n => textProj.update(`Count: ${n}`));

// Style projection - updates a single CSS property
const styleProj = DOMProjection.style(display, 'color');
count.subscribe(n => styleProj.update(n > 10 ? 'green' : 'black'));

// Attribute projection - updates an HTML attribute
const button = document.getElementById('submit')!;
const attrProj = DOMProjection.attribute(button, 'disabled');
isActive.subscribe(active => attrProj.update(active ? null : 'disabled'));

// Class toggle projection - adds/removes a CSS class
const classProj = DOMProjection.classToggle(display, 'highlight');
count.subscribe(n => classProj.update(n > 5));

// Data attribute projection - updates data-* attributes
const dataProj = DOMProjection.data(display, 'count');
count.subscribe(n => dataProj.update(String(n)));
```

### Batched Updates with Scheduler

When updating multiple DOM properties, use `ProjectionScheduler` to batch updates to the next animation frame:

```typescript
import { Behavior, DOMProjection, ProjectionScheduler } from '@industrialalgebra/cliffy-core';

const scheduler = new ProjectionScheduler();
const state = new Behavior({ count: 0, label: 'Items' });

const el = document.getElementById('counter')!;
const textProj = DOMProjection.text(el);
const styleProj = DOMProjection.style(el, 'opacity');
const classProj = DOMProjection.classToggle(el, 'empty');

state.subscribe(({ count, label }) => {
    // All updates batched to single animation frame
    scheduler.schedule(textProj, `${label}: ${count}`);
    scheduler.schedule(styleProj, count === 0 ? '0.5' : '1');
    scheduler.schedule(classProj, count === 0);
});
```

### ElementProjections Builder

For multiple projections on one element, use the builder pattern:

```typescript
import { Behavior, ElementProjections } from '@industrialalgebra/cliffy-core';

const todo = new Behavior({ text: 'Buy milk', done: false });
const li = document.createElement('li');

const projections = new ElementProjections(li)
    .text(({ text }) => text)
    .classToggle('completed', ({ done }) => done)
    .style('textDecoration', ({ done }) => done ? 'line-through' : 'none')
    .data('status', ({ done }) => done ? 'done' : 'pending');

todo.subscribe(value => projections.update(value));
```

## Building Components

Components in Cliffy are factory functions that return an object with Behaviors, Events, and lifecycle methods.

### Simple Component: Counter

```typescript
import { Behavior, Event, DOMProjection, ProjectionScheduler } from '@industrialalgebra/cliffy-core';

function createCounter(initialValue = 0) {
    // Internal state
    const count = new Behavior(initialValue);

    // Events (public interface for actions)
    const increment = new Event<void>();
    const decrement = new Event<void>();
    const reset = new Event<void>();

    // Wire events to state
    increment.subscribe(() => count.update(n => n + 1));
    decrement.subscribe(() => count.update(n => n - 1));
    reset.subscribe(() => count.set(initialValue));

    // Derived state
    const isZero = count.map(n => n === 0);
    const displayText = count.map(n => `Count: ${n}`);

    // Mount function creates DOM and projections
    function mount(container: HTMLElement) {
        const scheduler = new ProjectionScheduler();

        // Create DOM structure
        const wrapper = document.createElement('div');
        wrapper.className = 'counter';

        const display = document.createElement('span');
        display.className = 'counter-display';

        const incBtn = document.createElement('button');
        incBtn.textContent = '+';
        incBtn.onclick = () => increment.emit();

        const decBtn = document.createElement('button');
        decBtn.textContent = '-';
        decBtn.onclick = () => decrement.emit();

        const resetBtn = document.createElement('button');
        resetBtn.textContent = 'Reset';
        resetBtn.onclick = () => reset.emit();

        wrapper.append(decBtn, display, incBtn, resetBtn);
        container.appendChild(wrapper);

        // Set up projections
        const textProj = DOMProjection.text(display);
        const classProj = DOMProjection.classToggle(decBtn, 'disabled');

        displayText.subscribe(text => scheduler.schedule(textProj, text));
        isZero.subscribe(zero => scheduler.schedule(classProj, zero));

        // Return unmount function
        return () => {
            wrapper.remove();
        };
    }

    return {
        // State (read-only access)
        count,
        isZero,
        displayText,

        // Events (for external triggering)
        increment,
        decrement,
        reset,

        // Lifecycle
        mount
    };
}

// Usage
const counter = createCounter(10);
const unmount = counter.mount(document.getElementById('app')!);

// External access to state
counter.count.subscribe(n => console.log('Counter changed:', n));

// External triggering
counter.increment.emit();  // Count: 11
```

### Component with Props: Toggle

```typescript
import { Behavior, Event, DOMProjection } from '@industrialalgebra/cliffy-core';

interface ToggleProps {
    label: string;
    initialState?: boolean;
    onChange?: (value: boolean) => void;
}

function createToggle({ label, initialState = false, onChange }: ToggleProps) {
    const isOn = new Behavior(initialState);
    const toggle = new Event<void>();

    toggle.subscribe(() => isOn.update(v => !v));

    // Notify parent on change
    if (onChange) {
        isOn.subscribe(onChange);
    }

    function mount(container: HTMLElement) {
        const wrapper = document.createElement('label');
        wrapper.className = 'toggle';

        const checkbox = document.createElement('input');
        checkbox.type = 'checkbox';
        checkbox.checked = initialState;
        checkbox.onchange = () => toggle.emit();

        const span = document.createElement('span');
        span.textContent = label;

        wrapper.append(checkbox, span);
        container.appendChild(wrapper);

        // Sync checkbox with state
        const checkedProj = DOMProjection.attribute(checkbox, 'checked');
        isOn.subscribe(on => checkedProj.update(on ? 'checked' : null));

        return () => wrapper.remove();
    }

    return { isOn, toggle, mount };
}

// Usage
const darkMode = createToggle({
    label: 'Dark Mode',
    initialState: false,
    onChange: (dark) => document.body.classList.toggle('dark', dark)
});
darkMode.mount(document.getElementById('settings')!);
```

## Composing Components

### Parent-Child Communication

```typescript
import { Behavior, Event, combine } from '@industrialalgebra/cliffy-core';

function createTemperatureConverter() {
    const celsius = new Behavior(20);
    const fahrenheit = celsius.map(c => c * 9/5 + 32);

    // Child components
    const celsiusInput = createNumberInput({
        label: '°C',
        value: celsius,
        onChange: (v) => celsius.set(v)
    });

    const fahrenheitInput = createNumberInput({
        label: '°F',
        value: fahrenheit,
        onChange: (f) => celsius.set((f - 32) * 5/9)
    });

    function mount(container: HTMLElement) {
        const wrapper = document.createElement('div');
        wrapper.className = 'temperature-converter';

        const celsiusContainer = document.createElement('div');
        const fahrenheitContainer = document.createElement('div');

        wrapper.append(celsiusContainer, fahrenheitContainer);
        container.appendChild(wrapper);

        const unmountCelsius = celsiusInput.mount(celsiusContainer);
        const unmountFahrenheit = fahrenheitInput.mount(fahrenheitContainer);

        return () => {
            unmountCelsius();
            unmountFahrenheit();
            wrapper.remove();
        };
    }

    return { celsius, fahrenheit, mount };
}
```

### Component Composition: Form

```typescript
import { Behavior, Event, combine } from '@industrialalgebra/cliffy-core';

interface FormFieldProps {
    name: string;
    label: string;
    initialValue?: string;
    validator?: (value: string) => string | null;
}

function createFormField({ name, label, initialValue = '', validator }: FormFieldProps) {
    const value = new Behavior(initialValue);
    const error = value.map(v => validator ? validator(v) : null);
    const isValid = error.map(e => e === null);
    const change = new Event<string>();

    change.subscribe(v => value.set(v));

    function mount(container: HTMLElement) {
        const wrapper = document.createElement('div');
        wrapper.className = 'form-field';

        const labelEl = document.createElement('label');
        labelEl.textContent = label;
        labelEl.htmlFor = name;

        const input = document.createElement('input');
        input.id = name;
        input.name = name;
        input.value = initialValue;
        input.oninput = () => change.emit(input.value);

        const errorEl = document.createElement('span');
        errorEl.className = 'error';

        wrapper.append(labelEl, input, errorEl);
        container.appendChild(wrapper);

        // Projections
        const errorTextProj = DOMProjection.text(errorEl);
        const errorVisProj = DOMProjection.style(errorEl, 'display');
        const inputClassProj = DOMProjection.classToggle(input, 'invalid');

        error.subscribe(err => {
            errorTextProj.update(err || '');
            errorVisProj.update(err ? 'block' : 'none');
        });
        isValid.subscribe(valid => inputClassProj.update(!valid));

        return () => wrapper.remove();
    }

    return { value, error, isValid, change, mount };
}

function createForm() {
    const nameField = createFormField({
        name: 'name',
        label: 'Name',
        validator: v => v.length < 2 ? 'Name must be at least 2 characters' : null
    });

    const emailField = createFormField({
        name: 'email',
        label: 'Email',
        validator: v => !v.includes('@') ? 'Invalid email address' : null
    });

    const isFormValid = combine(
        nameField.isValid,
        emailField.isValid,
        (a, b) => a && b
    );

    const submit = new Event<void>();
    const formData = combine(
        nameField.value,
        emailField.value,
        (name, email) => ({ name, email })
    );

    function mount(container: HTMLElement) {
        const form = document.createElement('form');
        form.onsubmit = (e) => {
            e.preventDefault();
            if (isFormValid.sample()) {
                submit.emit();
            }
        };

        const nameContainer = document.createElement('div');
        const emailContainer = document.createElement('div');

        const submitBtn = document.createElement('button');
        submitBtn.type = 'submit';
        submitBtn.textContent = 'Submit';

        form.append(nameContainer, emailContainer, submitBtn);
        container.appendChild(form);

        const unmountName = nameField.mount(nameContainer);
        const unmountEmail = emailField.mount(emailContainer);

        // Disable submit when invalid
        const disabledProj = DOMProjection.attribute(submitBtn, 'disabled');
        isFormValid.subscribe(valid => disabledProj.update(valid ? null : 'disabled'));

        return () => {
            unmountName();
            unmountEmail();
            form.remove();
        };
    }

    return { nameField, emailField, isFormValid, formData, submit, mount };
}

// Usage
const form = createForm();
form.mount(document.getElementById('app')!);

form.submit.subscribe(() => {
    const data = form.formData.sample();
    console.log('Form submitted:', data);
});
```

### Dynamic Component Lists

```typescript
import { Behavior, Event } from '@industrialalgebra/cliffy-core';

interface TodoItem {
    id: number;
    text: string;
    done: boolean;
}

function createTodoList() {
    const items = new Behavior<TodoItem[]>([]);
    const newTodoText = new Behavior('');

    // Events
    const addTodo = new Event<void>();
    const toggleTodo = new Event<number>();
    const deleteTodo = new Event<number>();

    // Wire events
    addTodo.subscribe(() => {
        const text = newTodoText.sample().trim();
        if (text) {
            items.update(list => [
                ...list,
                { id: Date.now(), text, done: false }
            ]);
            newTodoText.set('');
        }
    });

    toggleTodo.subscribe(id => {
        items.update(list =>
            list.map(item =>
                item.id === id ? { ...item, done: !item.done } : item
            )
        );
    });

    deleteTodo.subscribe(id => {
        items.update(list => list.filter(item => item.id !== id));
    });

    // Derived state
    const activeCount = items.map(list => list.filter(t => !t.done).length);
    const completedCount = items.map(list => list.filter(t => t.done).length);

    function mount(container: HTMLElement) {
        const wrapper = document.createElement('div');
        wrapper.className = 'todo-list';

        // Input section
        const inputSection = document.createElement('div');
        const input = document.createElement('input');
        input.placeholder = 'What needs to be done?';
        input.oninput = () => newTodoText.set(input.value);
        input.onkeypress = (e) => {
            if (e.key === 'Enter') addTodo.emit();
        };

        const addBtn = document.createElement('button');
        addBtn.textContent = 'Add';
        addBtn.onclick = () => addTodo.emit();

        inputSection.append(input, addBtn);

        // List section
        const list = document.createElement('ul');

        // Stats section
        const stats = document.createElement('div');
        stats.className = 'stats';

        wrapper.append(inputSection, list, stats);
        container.appendChild(wrapper);

        // Track mounted item components
        const mountedItems = new Map<number, () => void>();

        // Sync input value
        newTodoText.subscribe(text => {
            input.value = text;
        });

        // Sync list - reconcile mounted items with state
        items.subscribe(todoItems => {
            const currentIds = new Set(todoItems.map(t => t.id));

            // Remove items that no longer exist
            for (const [id, unmount] of mountedItems) {
                if (!currentIds.has(id)) {
                    unmount();
                    mountedItems.delete(id);
                }
            }

            // Add/update items
            for (const item of todoItems) {
                if (!mountedItems.has(item.id)) {
                    const li = document.createElement('li');

                    const checkbox = document.createElement('input');
                    checkbox.type = 'checkbox';
                    checkbox.checked = item.done;
                    checkbox.onchange = () => toggleTodo.emit(item.id);

                    const span = document.createElement('span');
                    span.textContent = item.text;
                    if (item.done) span.style.textDecoration = 'line-through';

                    const deleteBtn = document.createElement('button');
                    deleteBtn.textContent = '×';
                    deleteBtn.onclick = () => deleteTodo.emit(item.id);

                    li.append(checkbox, span, deleteBtn);
                    list.appendChild(li);

                    mountedItems.set(item.id, () => li.remove());
                }
            }
        });

        // Sync stats
        combine(activeCount, completedCount, (active, completed) =>
            `${active} active, ${completed} completed`
        ).subscribe(text => {
            stats.textContent = text;
        });

        return () => {
            for (const unmount of mountedItems.values()) {
                unmount();
            }
            wrapper.remove();
        };
    }

    return {
        items,
        newTodoText,
        activeCount,
        completedCount,
        addTodo,
        toggleTodo,
        deleteTodo,
        mount
    };
}

// Usage
const todoList = createTodoList();
todoList.mount(document.getElementById('app')!);
```

## Next Steps

- **Start simple**: Use `html` tagged templates for quick reactive UIs
- **Go deeper**: Read the [FRP Guide](./frp-guide.md) for reactive patterns
- **Advanced rendering**: Learn the [DOM Projection Guide](./dom-projection-guide.md) for fine-grained control
- **Type safety**: Try the [PureScript template](../cliffy-purescript/README.md) for compile-time guarantees
- **Understand the math**: Explore the [Geometric Algebra Primer](./geometric-algebra-primer.md)
- **See examples**: Check out the [examples](../examples/) for complete applications
- **Learn the design**: See the [Architecture docs](./architecture/) for design decisions

## Key Differences from React/Redux

| Concept | React/Redux | Cliffy |
|---------|------------|--------|
| State | `useState`, Redux store | `Behavior` |
| Derived state | `useMemo`, selectors | `behavior.map()` |
| Events | Callbacks, actions | `Event` |
| Side effects | `useEffect` | `subscribe()` |
| Combining state | Multiple hooks | `combine()` |
| DOM updates | Virtual DOM diffing | Direct `DOMProjection` |

Cliffy's approach is more declarative: you describe the relationships between values, and updates propagate automatically. DOM updates happen directly without virtual DOM reconciliation.
