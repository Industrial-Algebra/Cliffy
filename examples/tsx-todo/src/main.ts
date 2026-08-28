/**
 * Cliffy TSX Todo Example
 *
 * Demonstrates:
 * - List state management with Behavior
 * - Form handling with input binding
 * - Derived state (filtered lists, counts)
 * - Dynamic list rendering with html template
 * - Event handlers for CRUD operations
 */

import init, { behavior, combine } from '@industrialalgebra/cliffy-core';
import { html, mount } from '@industrialalgebra/cliffy-core/html';

// Todo item type
interface Todo {
  id: number;
  text: string;
  completed: boolean;
}

// Filter type
type Filter = 'all' | 'active' | 'completed';

async function main() {
  // Initialize WASM module
  await init();

  // Application state
  let nextId = 1;
  const todos = behavior<Todo[]>([]);
  const filter = behavior<Filter>('all');
  const inputText = behavior('');

  // Derived state - automatically updates when dependencies change
  const filteredTodos = combine(todos, filter, (items: Todo[], f: Filter) => {
    switch (f) {
      case 'active':
        return items.filter(t => !t.completed);
      case 'completed':
        return items.filter(t => t.completed);
      default:
        return items;
    }
  });

  const activeCount = todos.map((items: Todo[]) =>
    items.filter(t => !t.completed).length
  );

  const completedCount = todos.map((items: Todo[]) =>
    items.filter(t => t.completed).length
  );

  const totalCount = todos.map((items: Todo[]) => items.length);

  // Event handlers
  const addTodo = () => {
    const text = inputText.sample().trim();
    if (text) {
      todos.update((items: Todo[]) => [
        ...items,
        { id: nextId++, text, completed: false }
      ]);
      inputText.set('');
    }
  };

  const toggleTodo = (id: number) => {
    todos.update((items: Todo[]) =>
      items.map(t =>
        t.id === id ? { ...t, completed: !t.completed } : t
      )
    );
  };

  const deleteTodo = (id: number) => {
    todos.update((items: Todo[]) => items.filter(t => t.id !== id));
  };

  const setFilter = (f: Filter) => {
    filter.set(f);
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter') {
      addTodo();
    }
  };

  const handleInput = (e: Event) => {
    const target = e.target as HTMLInputElement;
    inputText.set(target.value);
  };

  // Filter button class based on current filter
  const filterClass = (f: Filter) =>
    filter.map((current: Filter) => current === f ? 'active' : '');

  // Render a single todo item
  const renderTodo = (todo: Todo) => html`
    <li class="todo-item ${todo.completed ? 'completed' : ''}">
      <input
        type="checkbox"
        checked=${todo.completed}
        onchange=${() => toggleTodo(todo.id)}
      />
      <span>${todo.text}</span>
      <button onclick=${() => deleteTodo(todo.id)}>Delete</button>
    </li>
  `;

  // Render the todo list or empty state
  const renderList = filteredTodos.map((items: Todo[]) => {
    if (items.length === 0) {
      return html`<div class="empty-state">No todos yet. Add one above!</div>`;
    }
    return html`
      <ul class="todo-list">
        ${items.map(renderTodo)}
      </ul>
    `;
  });

  // Main app template
  const app = html`
    <div class="todo-app">
      <h1>Todo List</h1>

      <div class="add-form">
        <input
          type="text"
          placeholder="What needs to be done?"
          value=${inputText}
          oninput=${handleInput}
          onkeydown=${handleKeyDown}
        />
        <button onclick=${addTodo}>Add</button>
      </div>

      <div class="filters">
        <button class=${filterClass('all')} onclick=${() => setFilter('all')}>
          All
        </button>
        <button class=${filterClass('active')} onclick=${() => setFilter('active')}>
          Active
        </button>
        <button class=${filterClass('completed')} onclick=${() => setFilter('completed')}>
          Completed
        </button>
      </div>

      ${renderList}

      <div class="stats">
        <span>${activeCount} active</span>
        <span>${completedCount} completed</span>
        <span>${totalCount} total</span>
      </div>
    </div>
  `;

  // Mount to DOM
  mount(app, '#app');

  console.log('Cliffy TSX Todo initialized');
}

main().catch(console.error);
