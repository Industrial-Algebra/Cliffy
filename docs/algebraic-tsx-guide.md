# Algebraic TSX Guide

Algebraic TSX is Cliffy's tagged template literal API for building reactive DOM interfaces. It provides a declarative way to create HTML elements that automatically update when your reactive Behaviors change.

## Overview

Unlike frameworks that use virtual DOM diffing, Algebraic TSX creates real DOM elements with direct bindings to your reactive state. When a Behavior changes, only the specific text nodes or attributes that depend on it are updated, with no reconciliation step.

```typescript
import { behavior } from '@industrialalgebra/cliffy-core';
import { html, mount } from '@industrialalgebra/cliffy-core/html';

const count = behavior(0);

const app = html`
  <div>
    <h1>Count: ${count}</h1>
    <button onclick=${() => count.update(n => n + 1)}>+</button>
  </div>
`;

mount(app, '#app');
```

## Installation

Import the `html` template tag and `mount` function from `@industrialalgebra/cliffy-core/html`:

```typescript
import init, { behavior, combine } from '@industrialalgebra/cliffy-core';
import { html, mount } from '@industrialalgebra/cliffy-core/html';

async function main() {
  // Initialize WASM module first
  await init();

  // Your app code here
}

main();
```

## Basic Usage

### The `html` Template Tag

The `html` tag creates DOM elements from template literals:

```typescript
const element = html`<div class="container">Hello, World!</div>`;
```

You can include any valid HTML:

```typescript
const form = html`
  <form class="login-form">
    <label for="email">Email</label>
    <input type="email" id="email" name="email" />
    <button type="submit">Submit</button>
  </form>
`;
```

### The `mount` Function

Use `mount` to attach your element to the DOM:

```typescript
const app = html`<div>My App</div>`;

// Mount using a CSS selector
mount(app, '#app');

// Or mount to an element directly
const container = document.getElementById('app');
mount(app, container);
```

The `mount` function returns a cleanup function that removes the element and unsubscribes all reactive bindings:

```typescript
const cleanup = mount(app, '#app');

// Later: remove the app and clean up
cleanup();
```

## Reactive Values

When you include a Behavior in your template, the DOM automatically updates whenever the Behavior changes:

```typescript
const count = behavior(0);

const app = html`
  <div>
    <p>Current count: ${count}</p>
    <button onclick=${() => count.update(n => n + 1)}>Increment</button>
  </div>
`;

mount(app, '#app');
// Clicking the button updates the paragraph automatically
```

### Derived Behaviors

Use `map()` to create derived values that update automatically:

```typescript
const count = behavior(0);
const doubled = count.map(n => n * 2);
const isEven = count.map(n => n % 2 === 0);

const app = html`
  <div>
    <p>Count: ${count}</p>
    <p>Doubled: ${doubled}</p>
    <p>Is even: ${isEven}</p>
  </div>
`;
```

### Combined Behaviors

Use `combine()` to derive values from multiple Behaviors:

```typescript
const firstName = behavior('John');
const lastName = behavior('Doe');

const fullName = combine(firstName, lastName, (first, last) => `${first} ${last}`);

const app = html`
  <p>Full name: ${fullName}</p>
`;
```

## Event Handlers

Attach event handlers using `on*` attributes with function values:

```typescript
const handleClick = () => console.log('Clicked!');
const handleMouseOver = (e) => console.log('Mouse at:', e.clientX, e.clientY);

const app = html`
  <button
    onclick=${handleClick}
    onmouseover=${handleMouseOver}
  >
    Click me
  </button>
`;
```

Common event handlers:
- `onclick` - Mouse clicks
- `oninput` - Input value changes
- `onchange` - Input change (on blur)
- `onkeydown` / `onkeyup` - Keyboard events
- `onsubmit` - Form submission
- `onfocus` / `onblur` - Focus events

### Inline Event Handlers

You can define handlers inline:

```typescript
const count = behavior(0);

const app = html`
  <div>
    <button onclick=${() => count.update(n => n - 1)}>-</button>
    <span>${count}</span>
    <button onclick=${() => count.update(n => n + 1)}>+</button>
  </div>
`;
```

## Conditional Rendering

Use `map()` to conditionally render different content:

```typescript
const isLoggedIn = behavior(false);

const content = isLoggedIn.map(loggedIn => {
  if (loggedIn) {
    return html`<p>Welcome back!</p>`;
  } else {
    return html`<p>Please log in.</p>`;
  }
});

const app = html`
  <div>
    ${content}
    <button onclick=${() => isLoggedIn.update(v => !v)}>
      Toggle Login
    </button>
  </div>
`;
```

### Conditional CSS Classes

Map a Behavior to a class string:

```typescript
const count = behavior(0);

const statusClass = count.map(n => {
  if (n > 0) return 'positive';
  if (n < 0) return 'negative';
  return 'zero';
});

const app = html`
  <span class=${statusClass}>${count}</span>
`;
```

### Showing/Hiding Elements

Use a Behavior for conditional display:

```typescript
const isVisible = behavior(true);

const visibilityStyle = isVisible.map(v => v ? '' : 'display: none');

const app = html`
  <div>
    <button onclick=${() => isVisible.update(v => !v)}>Toggle</button>
    <p style=${visibilityStyle}>This can be hidden</p>
  </div>
`;
```

## List Rendering

Use `map()` on a Behavior containing an array to render lists:

```typescript
interface Item {
  id: number;
  name: string;
}

const items = behavior<Item[]>([
  { id: 1, name: 'Apple' },
  { id: 2, name: 'Banana' },
  { id: 3, name: 'Cherry' },
]);

const renderItem = (item: Item) => html`
  <li>${item.name}</li>
`;

const list = items.map(itemList => html`
  <ul>
    ${itemList.map(renderItem)}
  </ul>
`);

const app = html`
  <div>
    <h2>Fruits</h2>
    ${list}
  </div>
`;
```

### Dynamic Lists with Actions

```typescript
interface Todo {
  id: number;
  text: string;
  completed: boolean;
}

const todos = behavior<Todo[]>([]);

const toggleTodo = (id: number) => {
  todos.update(items =>
    items.map(t => t.id === id ? { ...t, completed: !t.completed } : t)
  );
};

const deleteTodo = (id: number) => {
  todos.update(items => items.filter(t => t.id !== id));
};

const renderTodo = (todo: Todo) => html`
  <li class="${todo.completed ? 'completed' : ''}">
    <input
      type="checkbox"
      checked=${todo.completed}
      onchange=${() => toggleTodo(todo.id)}
    />
    <span>${todo.text}</span>
    <button onclick=${() => deleteTodo(todo.id)}>Delete</button>
  </li>
`;

const todoList = todos.map(items => html`
  <ul class="todo-list">
    ${items.map(renderTodo)}
  </ul>
`);
```

### Empty State

Handle empty lists with conditional rendering:

```typescript
const items = behavior<string[]>([]);

const content = items.map(list => {
  if (list.length === 0) {
    return html`<div class="empty-state">No items yet.</div>`;
  }
  return html`
    <ul>
      ${list.map(item => html`<li>${item}</li>`)}
    </ul>
  `;
});
```

## Nested Templates

Templates can be nested and composed:

```typescript
const Header = (title: string) => html`
  <header>
    <h1>${title}</h1>
  </header>
`;

const Footer = () => html`
  <footer>
    <p>Copyright 2024</p>
  </footer>
`;

const app = html`
  <div class="app">
    ${Header('My Application')}
    <main>
      <p>Content goes here</p>
    </main>
    ${Footer()}
  </div>
`;
```

### Reusable Components

Create reusable components as functions:

```typescript
interface ButtonProps {
  label: string;
  onClick: () => void;
  variant?: 'primary' | 'secondary';
}

const Button = ({ label, onClick, variant = 'primary' }: ButtonProps) => html`
  <button class="btn btn-${variant}" onclick=${onClick}>
    ${label}
  </button>
`;

const app = html`
  <div>
    ${Button({ label: 'Save', onClick: () => save(), variant: 'primary' })}
    ${Button({ label: 'Cancel', onClick: () => cancel(), variant: 'secondary' })}
  </div>
`;
```

## Form Handling

### Text Input

```typescript
const inputText = behavior('');

const handleInput = (e: Event) => {
  const target = e.target as HTMLInputElement;
  inputText.set(target.value);
};

const app = html`
  <div>
    <input
      type="text"
      value=${inputText}
      oninput=${handleInput}
      placeholder="Type something..."
    />
    <p>You typed: ${inputText}</p>
  </div>
`;
```

### Form Submission

```typescript
const email = behavior('');
const password = behavior('');

const handleSubmit = (e: Event) => {
  e.preventDefault();
  console.log('Submitting:', email.sample(), password.sample());
};

const app = html`
  <form onsubmit=${handleSubmit}>
    <input
      type="email"
      value=${email}
      oninput=${(e: Event) => email.set((e.target as HTMLInputElement).value)}
      placeholder="Email"
    />
    <input
      type="password"
      value=${password}
      oninput=${(e: Event) => password.set((e.target as HTMLInputElement).value)}
      placeholder="Password"
    />
    <button type="submit">Login</button>
  </form>
`;
```

### Checkbox

```typescript
const isChecked = behavior(false);

const app = html`
  <label>
    <input
      type="checkbox"
      checked=${isChecked}
      onchange=${() => isChecked.update(v => !v)}
    />
    I agree to the terms
  </label>
`;
```

### Select Dropdown

```typescript
const selected = behavior('option1');

const handleChange = (e: Event) => {
  selected.set((e.target as HTMLSelectElement).value);
};

const app = html`
  <select onchange=${handleChange}>
    <option value="option1">Option 1</option>
    <option value="option2">Option 2</option>
    <option value="option3">Option 3</option>
  </select>
  <p>Selected: ${selected}</p>
`;
```

### Keyboard Events

```typescript
const inputText = behavior('');

const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Enter') {
    console.log('Submitted:', inputText.sample());
    inputText.set('');
  }
};

const app = html`
  <input
    type="text"
    value=${inputText}
    oninput=${(e: Event) => inputText.set((e.target as HTMLInputElement).value)}
    onkeydown=${handleKeyDown}
    placeholder="Press Enter to submit"
  />
`;
```

## CSS Classes

### Static Classes

```typescript
const app = html`
  <div class="container main-content">
    <p class="text-primary">Styled text</p>
  </div>
`;
```

### Dynamic Classes

Use a Behavior that returns a class string:

```typescript
const isActive = behavior(false);
const buttonClass = isActive.map(active => active ? 'btn active' : 'btn');

const app = html`
  <button
    class=${buttonClass}
    onclick=${() => isActive.update(v => !v)}
  >
    Toggle
  </button>
`;
```

### Filter Button Pattern

Common pattern for filter buttons:

```typescript
type Filter = 'all' | 'active' | 'completed';
const currentFilter = behavior<Filter>('all');

const filterClass = (f: Filter) =>
  currentFilter.map(current => current === f ? 'filter-btn active' : 'filter-btn');

const app = html`
  <div class="filters">
    <button class=${filterClass('all')} onclick=${() => currentFilter.set('all')}>
      All
    </button>
    <button class=${filterClass('active')} onclick=${() => currentFilter.set('active')}>
      Active
    </button>
    <button class=${filterClass('completed')} onclick=${() => currentFilter.set('completed')}>
      Completed
    </button>
  </div>
`;
```

## Complete Example

Here is a complete counter application demonstrating the key concepts:

```typescript
import init, { behavior, combine } from '@industrialalgebra/cliffy-core';
import { html, mount } from '@industrialalgebra/cliffy-core/html';

async function main() {
  await init();

  // State
  const count = behavior(0);

  // Derived state
  const doubled = count.map(n => n * 2);
  const statusClass = count.map(n => {
    if (n > 0) return 'positive';
    if (n < 0) return 'negative';
    return 'zero';
  });

  // Event handlers
  const increment = () => count.update(n => n + 1);
  const decrement = () => count.update(n => n - 1);
  const reset = () => count.set(0);

  // UI
  const app = html`
    <div class="counter">
      <h1 class=${statusClass}>${count}</h1>

      <div class="controls">
        <button onclick=${decrement}>-</button>
        <button onclick=${reset}>Reset</button>
        <button onclick=${increment}>+</button>
      </div>

      <p>Doubled: ${doubled}</p>
    </div>
  `;

  mount(app, '#app');
}

main();
```

## Next Steps

- [FRP Guide](./frp-guide.md) - Learn more about Behavior and Event patterns
- [DOM Projection Guide](./dom-projection-guide.md) - Low-level DOM binding API
- [Examples](../examples/) - See full applications

---

## How It Works

For those interested in the internal architecture, Algebraic TSX works differently from virtual DOM frameworks.

### Direct Projection vs Virtual DOM

Traditional virtual DOM frameworks:
```
State Change -> Render -> Virtual Tree -> Diff -> Patch -> DOM
```

Algebraic TSX:
```
State Change -> Projection -> DOM Property
```

When you use a Behavior in a template, a subscription is created that updates only that specific DOM node or attribute when the Behavior changes. There is no virtual tree, no diffing algorithm, and no reconciliation.

### How Template Processing Works

1. The `html` tag parses your template literal into an HTML string with placeholder markers
2. The HTML is parsed using a native `<template>` element
3. The resulting DOM tree is walked to find placeholders
4. For each placeholder containing a Behavior, a subscription is created that updates just that DOM location
5. Event handlers are attached directly to elements

This approach provides O(1) targeted updates rather than O(n) diffing, with no virtual node allocation overhead.

### Internal Types

The rendering system uses these internal types (in Rust, not exposed via WASM):

- **Component trait** - Geometric morphisms from state to elements
- **DataflowGraph** - Static representation of data transformations
- **Element tree** - Render output structure (not virtual DOM)

For more details on the architecture, see [Architecture Documentation](./architecture/).
