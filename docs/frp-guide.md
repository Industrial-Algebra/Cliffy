# Functional Reactive Programming with Cliffy

This guide is for developers familiar with React, Vue, or similar frameworks who want to understand Cliffy's approach to reactivity. We'll cover the core concepts and show how they map to patterns you already know.

## The Core Idea

In React/Vue, you manage state with hooks or reactive refs, and the framework re-renders when state changes. Cliffy takes a different approach based on **Functional Reactive Programming (FRP)**, where you describe relationships between values, and updates propagate automatically.

| Approach | Model | Updates |
|----------|-------|---------|
| React | State + re-render | Components re-execute |
| Vue | Reactive proxy | Dependency tracking |
| **Cliffy** | Dataflow graph | Values propagate through graph |

## Two Primitives: Behavior and Event

Cliffy has just two core types:

### Behavior<T>: Continuous Values

A `Behavior<T>` represents a value that exists at every point in time. Think of it as a cell in a spreadsheet—it always has a current value, and when dependencies change, it updates automatically.

```typescript
import { behavior } from '@industrialalgebra/cliffy-core';

// Create a behavior with initial value
const count = behavior(0);

// Read current value
console.log(count.sample()); // 0

// Update the value
count.set(10);
count.update(n => n + 1); // 11

// Subscribe to changes
count.subscribe(n => {
    console.log('Count changed to:', n);
});
```

**Key properties:**
- Always has a current value (call `sample()` anytime)
- Changes are observable via `subscribe()`
- Can derive new behaviors with `map()`

### Event<T>: Discrete Occurrences

An `Event<T>` represents things that happen at specific moments—clicks, key presses, network responses. Unlike Behaviors, Events don't have a "current value."

```typescript
import { event } from '@industrialalgebra/cliffy-core';

// Create an event stream
const clicks = event<MouseEvent>();

// Subscribe to events
clicks.subscribe(e => {
    console.log('Clicked at:', e.clientX, e.clientY);
});

// Emit an event (typically from DOM listener)
document.onclick = (e) => clicks.emit(e);
```

**Key properties:**
- No current value—only occurrences
- Can transform with `map()`, `filter()`
- Can convert to Behavior with `fold()`

## Mapping to React Patterns

### useState → behavior()

```typescript
// React
const [count, setCount] = useState(0);
setCount(count + 1);

// Cliffy
const count = behavior(0);
count.update(n => n + 1);
```

### useMemo → map()

```typescript
// React
const doubled = useMemo(() => count * 2, [count]);

// Cliffy
const doubled = count.map(n => n * 2);
// No dependency array needed—automatically tracked!
```

### Multiple useState → combine()

```typescript
// React
const [width, setWidth] = useState(10);
const [height, setHeight] = useState(20);
const area = useMemo(() => width * height, [width, height]);

// Cliffy
const width = behavior(10);
const height = behavior(20);
const area = combine(width, height, (w, h) => w * h);
// Updates automatically when either changes
```

### useEffect → subscribe()

```typescript
// React
useEffect(() => {
    console.log('Count changed:', count);
}, [count]);

// Cliffy
count.subscribe(n => {
    console.log('Count changed:', n);
});
```

### Event handlers → Event streams

```typescript
// React
<button onClick={() => setCount(c => c + 1)}>+</button>

// Cliffy
const incrementClicks = event<void>();
incrementClicks.subscribe(() => count.update(n => n + 1));
// In DOM setup:
button.onclick = () => incrementClicks.emit(undefined);
```

## The Key Difference: Declarative Dependencies

In React, you must manually specify dependencies:

```typescript
// React - easy to get wrong
const total = useMemo(() => {
    return items.reduce((sum, item) => sum + item.price, 0);
}, [items]); // Forgot a dependency? Silent bugs.
```

In Cliffy, dependencies are automatic:

```typescript
// Cliffy - dependencies tracked automatically
const total = items.map(list =>
    list.reduce((sum, item) => sum + item.price, 0)
);
// No dependency array. Can't forget anything.
```

## Combinators: Building Complex Behaviors

### map: Transform Values

```typescript
const celsius = behavior(20);
const fahrenheit = celsius.map(c => c * 9/5 + 32);
// fahrenheit automatically updates when celsius changes
```

### combine: Multiple Sources

```typescript
const firstName = behavior('John');
const lastName = behavior('Doe');
const fullName = combine(firstName, lastName, (f, l) => `${f} ${l}`);
// Updates when either name changes
```

### project: Conditional Values (formerly `when`)

```typescript
const isLoggedIn = behavior(false);
const greeting = isLoggedIn.project(() => 'Welcome back!');
// greeting is Some('Welcome back!') when logged in, None when not
```

> **Note:** The standalone `when(condition, fn)` function still works for backwards compatibility, but the method-based `condition.project(fn)` is now preferred.
>
> ```typescript
> // Deprecated (still works)
> const greeting = when(isLoggedIn, () => 'Welcome back!');
>
> // Preferred
> const greeting = isLoggedIn.project(() => 'Welcome back!');
> ```

### select: Conditional Selection (formerly `if_else`)

```typescript
const isDarkMode = behavior(false);
const theme = isDarkMode.select(
    () => 'dark',
    () => 'light'
);
```

> **Note:** The standalone `ifElse(condition, thenFn, elseFn)` function still works for backwards compatibility, but the method-based `condition.select(thenFn, elseFn)` is now preferred.
>
> ```typescript
> // Deprecated (still works)
> const theme = ifElse(isDarkMode, () => 'dark', () => 'light');
>
> // Preferred
> const theme = isDarkMode.select(() => 'dark', () => 'light');
> ```

### wedge: Combining 3+ Behaviors

When you need to combine three or more behaviors, use `wedge` for a cleaner API than nested `combine` calls:

```typescript
const x = behavior(1);
const y = behavior(2);
const z = behavior(3);

// Instead of nested combine:
// const sum = combine(x, combine(y, z, (b, c) => b + c), (a, bc) => a + bc);

// Use wedge for cleaner syntax:
const sum = wedge(x, y, z, (a, b, c) => a + b + c);
// Updates automatically when any input changes
```

`wedge` supports up to 8 input behaviors and automatically tracks all dependencies:

```typescript
const firstName = behavior('John');
const middleName = behavior('Q');
const lastName = behavior('Doe');
const suffix = behavior('Jr.');

const fullName = wedge(
    firstName, middleName, lastName, suffix,
    (first, middle, last, suf) => `${first} ${middle} ${last}, ${suf}`
);
// "John Q Doe, Jr."
```

The name `wedge` comes from the geometric algebra "wedge product" (exterior product), which combines multiple elements while preserving their independence - just as `wedge` combines multiple behaviors while tracking each dependency separately.

## Event Transformations

### map: Transform Event Values

```typescript
const clicks = event<MouseEvent>();
const positions = clicks.map(e => ({ x: e.clientX, y: e.clientY }));
```

### filter: Select Events

```typescript
const keyPresses = event<KeyboardEvent>();
const enterPresses = keyPresses.filter(e => e.key === 'Enter');
```

### merge: Combine Event Streams

```typescript
const mouseClicks = event<void>();
const touchTaps = event<void>();
const anyInteraction = mouseClicks.merge(touchTaps);
```

### fold: Accumulate into Behavior

```typescript
const clicks = event<void>();
const clickCount = clicks.fold(0, (count, _) => count + 1);
// clickCount is a Behavior that tracks total clicks
```

## Complete Example: Todo List

```typescript
import { behavior, event, combine } from '@industrialalgebra/cliffy-core';

interface Todo {
    id: number;
    text: string;
    done: boolean;
}

// State
const todos = behavior<Todo[]>([]);
const newTodoText = behavior('');
const filter = behavior<'all' | 'active' | 'completed'>('all');

// Derived state
const filteredTodos = combine(todos, filter, (list, f) => {
    switch (f) {
        case 'active': return list.filter(t => !t.done);
        case 'completed': return list.filter(t => t.done);
        default: return list;
    }
});

const activeCount = todos.map(list =>
    list.filter(t => !t.done).length
);

const allDone = activeCount.map(count => count === 0);

// Events
const addTodo = event<void>();
const toggleTodo = event<number>();
const deleteTodo = event<number>();
const clearCompleted = event<void>();

// Wire events to state updates
addTodo.subscribe(() => {
    const text = newTodoText.sample().trim();
    if (text) {
        todos.update(list => [
            ...list,
            { id: Date.now(), text, done: false }
        ]);
        newTodoText.set('');
    }
});

toggleTodo.subscribe(id => {
    todos.update(list =>
        list.map(t => t.id === id ? { ...t, done: !t.done } : t)
    );
});

deleteTodo.subscribe(id => {
    todos.update(list => list.filter(t => t.id !== id));
});

clearCompleted.subscribe(() => {
    todos.update(list => list.filter(t => !t.done));
});

// Subscribe to render
filteredTodos.subscribe(renderTodoList);
activeCount.subscribe(renderCounter);
```

## Patterns and Best Practices

### 1. Prefer Derived Behaviors Over Manual Updates

```typescript
// Bad: Manually keeping total in sync
const items = behavior<Item[]>([]);
const total = behavior(0);
items.subscribe(list => {
    total.set(list.reduce((s, i) => s + i.price, 0));
});

// Good: Derive automatically
const items = behavior<Item[]>([]);
const total = items.map(list =>
    list.reduce((s, i) => s + i.price, 0)
);
```

### 2. Use Events for User Actions

```typescript
// Events clearly separate "what happened" from "what to do"
const saveClicked = event<void>();
const deleteClicked = event<number>();

// Handle in one place
saveClicked.subscribe(() => {
    const data = formData.sample();
    api.save(data);
});
```

### 3. Combine for Cross-Cutting Concerns

```typescript
const user = behavior<User | null>(null);
const permissions = behavior<string[]>([]);

const canEdit = combine(user, permissions, (u, p) =>
    u !== null && p.includes('edit')
);

const canDelete = combine(user, permissions, (u, p) =>
    u !== null && p.includes('delete')
);
```

### 4. Use fold for Stateful Event Processing

```typescript
// Track last 5 values
const values = event<number>();
const history = values.fold([] as number[], (hist, val) =>
    [...hist.slice(-4), val]
);

// Debounce-like pattern
const searches = event<string>();
const debouncedSearch = searches.fold(
    { value: '', timer: null as number | null },
    (state, query) => {
        if (state.timer) clearTimeout(state.timer);
        return {
            value: query,
            timer: setTimeout(() => performSearch(query), 300)
        };
    }
).map(s => s.value);
```

## Anti-Patterns to Avoid

### Don't Use Behaviors for One-Time Events

```typescript
// Bad: Using behavior for navigation
const navigateTo = behavior<string | null>(null);
navigateTo.subscribe(url => {
    if (url) router.push(url);
});
navigateTo.set('/dashboard');

// Good: Use an event
const navigate = event<string>();
navigate.subscribe(url => router.push(url));
navigate.emit('/dashboard');
```

### Don't Subscribe Inside Subscribe

```typescript
// Bad: Nested subscriptions leak memory
outer.subscribe(a => {
    inner.subscribe(b => { // Creates new subscription each time!
        doSomething(a, b);
    });
});

// Good: Use combine
combine(outer, inner, (a, b) => {
    doSomething(a, b);
});
```

### Don't Call sample() in Render Loops

```typescript
// Bad: Polling in animation frame
function render() {
    const count = counter.sample(); // Called 60 times/sec
    draw(count);
    requestAnimationFrame(render);
}

// Good: Subscribe and only redraw on change
counter.subscribe(count => {
    draw(count);
});
```

## Why FRP Instead of Hooks?

| Aspect | React Hooks | Cliffy FRP |
|--------|-------------|------------|
| Dependencies | Manual arrays | Automatic |
| Stale closures | Common bug | Impossible |
| Rules | "Rules of hooks" | No rules needed |
| Testing | Mock hooks | Pure functions |
| Composition | Limited | Algebraic |
| Distributed | Extra work | Built-in |

Cliffy's FRP approach means:

1. **No dependency arrays** — can't forget dependencies
2. **No stale closures** — always sample latest values
3. **No rules of hooks** — compose however you want
4. **Better testing** — behaviors are just data
5. **Distributed-ready** — changes merge geometrically

## Next Steps

- [Getting Started Guide](./getting-started.md) — Build your first app
- [DOM Projection Guide](./dom-projection-guide.md) — Efficient DOM rendering
- [Geometric Algebra Primer](./geometric-algebra-primer.md) — The math behind the scenes
- [Examples](../examples/) — See full applications

## Quick Reference

| Operation | Code |
|-----------|------|
| Create behavior | `behavior(initialValue)` |
| Create event | `event<T>()` |
| Read current | `behavior.sample()` |
| Set value | `behavior.set(value)` |
| Update value | `behavior.update(fn)` |
| Subscribe | `behavior.subscribe(fn)` or `event.subscribe(fn)` |
| Transform | `behavior.map(fn)` or `event.map(fn)` |
| Combine (2 inputs) | `combine(a, b, fn)` |
| Combine (3+ inputs) | `wedge(a, b, c, ..., fn)` |
| Filter events | `event.filter(predicate)` |
| Merge events | `event1.merge(event2)` |
| Accumulate | `event.fold(initial, fn)` |
| Conditional | `condition.project(fn)` |
| Conditional else | `condition.select(thenFn, elseFn)` |
| ~~Conditional (deprecated)~~ | ~~`when(condition, fn)`~~ |
| ~~Conditional else (deprecated)~~ | ~~`ifElse(condition, thenFn, elseFn)`~~ |
