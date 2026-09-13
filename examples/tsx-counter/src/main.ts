/**
 * Cliffy TSX Counter Example
 *
 * Demonstrates:
 * - Behavior for reactive state
 * - html tagged template for declarative UI
 * - Event handlers with onclick
 * - Derived behaviors with map()
 * - Conditional CSS classes
 */

import init, {
  Behavior, behavior, combine } from '@industrialalgebra/cliffy-core';
import { html, mount } from '@industrialalgebra/cliffy-core/html';

async function main() {
  // Initialize WASM module
  await init();

  // Create reactive state
  const count = behavior(0);

  // Derived behaviors - automatically update when count changes
  const doubled = count.map((n: number) => n * 2);
  const isPositive = count.map((n: number) => n > 0);
  const isNegative = count.map((n: number) => n < 0);

  // Derive CSS class based on count value
  const statusClass = count.map((n: number) => {
    if (n > 0) return 'positive';
    if (n < 0) return 'negative';
    return 'zero';
  });

  // Event handlers
  const increment = () => count.update((n: number) => n + 1);
  const decrement = () => count.update((n: number) => n - 1);
  const reset = () => count.set(0);

  // Build reactive UI with html tagged template
  // Behaviors in ${} automatically update the DOM when they change
  const app = html`
    <div class="counter">
      <h1>${count}</h1>

      <div class="controls">
        <button class="decrement" onclick=${decrement}>-</button>
        <button class="reset" onclick=${reset}>Reset</button>
        <button class="increment" onclick=${increment}>+</button>
      </div>

      <p class="status">
        Doubled: <span class=${statusClass}>${doubled}</span>
      </p>
    </div>
  `;

  // Mount to DOM
  mount(app, '#app');

  console.log('Cliffy TSX Counter initialized');
}

main().catch(console.error);
