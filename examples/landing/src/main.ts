import init, { behavior, combine } from '@industrialalgebra/cliffy-core';
import { html, mount } from '@industrialalgebra/cliffy-core/html';
import { examples, categories, type Example } from './examples';

async function main() {
  await init();

  // State
  const filter = behavior('all');
  const searchQuery = behavior('');

  // Derived: filtered examples
  const filteredExamples = combine(filter, searchQuery, (cat: string, query: string): Example[] => {
    return examples.filter(ex => {
      const matchesCategory = cat === 'all' || ex.category === cat;
      const matchesSearch = query === '' ||
        ex.name.toLowerCase().includes(query.toLowerCase()) ||
        ex.description.toLowerCase().includes(query.toLowerCase()) ||
        ex.features.some(f => f.toLowerCase().includes(query.toLowerCase()));
      return matchesCategory && matchesSearch;
    });
  });

  // Category buttons
  const categoryButtons = categories.map((cat: { id: string; name: string }) => {
    const isActive = filter.map((f: string) => f === cat.id ? 'active' : '');
    return html`
      <button
        class=${isActive}
        onclick=${() => filter.set(cat.id)}
      >
        ${cat.name}
      </button>
    `;
  });

  // Example cards - reactive to filtered list
  const exampleCards = filteredExamples.map((exList: Example[]) => {
    if (exList.length === 0) {
      return html`
        <div class="empty-state">
          <h3>No examples found</h3>
          <p>Try a different search term or category.</p>
        </div>
      `;
    }

    return html`
      <div class="examples-grid">
        ${exList.map(ex => html`
          <a href="/${ex.slug}/" class="example-card">
            <h3>${ex.name}</h3>
            <p>${ex.description}</p>
            <div class="tags">
              <span class="tag category-tag">${ex.category}</span>
              ${ex.features.map(f => html`<span class="tag">${f}</span>`)}
            </div>
          </a>
        `)}
      </div>
    `;
  });

  const app = html`
    <div class="landing">
      <header class="hero">
        <h1>Cliffy Examples</h1>
        <p>Classical FRP + Geometric Algebra in action. Explore reactive patterns, distributed state, and creative tools.</p>
        <div class="hero-links">
          <a href="https://github.com/Industrial-Algebra/Cliffy" class="primary" target="_blank">
            GitHub
          </a>
          <a href="https://www.npmjs.com/package/@industrialalgebra/cliffy-core" class="secondary" target="_blank">
            npm: @industrialalgebra/cliffy-core
          </a>
        </div>
      </header>

      <nav class="filters">
        ${categoryButtons}
      </nav>

      <div class="search-container">
        <input
          type="search"
          placeholder="Search examples..."
          oninput=${(e: Event) => searchQuery.set((e.target as HTMLInputElement).value)}
        />
      </div>

      ${exampleCards}

      <footer>
        <p>Built with Cliffy v0.1.2 - Classical FRP without the complexity</p>
        <div class="links">
          <a href="https://github.com/Industrial-Algebra/Cliffy" target="_blank">GitHub</a>
          <a href="https://www.npmjs.com/package/@industrialalgebra/cliffy-core" target="_blank">npm</a>
          <a href="https://github.com/Industrial-Algebra/Cliffy/blob/main/docs/getting-started.md" target="_blank">Docs</a>
        </div>
      </footer>
    </div>
  `;

  mount(app, '#app');
}

main().catch(console.error);
