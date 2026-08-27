# Cliffy Roadmap

## Vision

Cliffy is **geometric state management and distributed synchronization for any UI framework**.

It enables building collaborative applications at Google Docs scale (10,000+ concurrent users) where distributed systems problems become geometric algebra problems with closed-form solutions and guaranteed convergence.

```
Traditional approach:          Cliffy approach:
───────────────────            ─────────────────
Complex consensus protocols    Geometric algebra operations
Conflict resolution heuristics Geometric mean (always converges)
Coordination overhead          No coordination needed
State reconciliation           State as multivector projections
```

### Core Insight

All state transformations are geometric operations in Clifford algebra:
- **State changes** = rotors/versors (geometric transformations)
- **Conflicts** = geometric distance from expected manifold
- **Resolution** = geometric mean (closed-form, always converges)
- **Composition** = geometric product (associative)

### Integration Model

Cliffy is used *inside* existing frameworks, not *instead of* them. It provides the state and synchronization layer while React, Leptos, Yew, or vanilla JS handle rendering:

```typescript
// React component using Cliffy for state
const Counter = () => {
  const count = useBehavior(sharedCount); // Cliffy behavior → React state
  return <button onClick={() => sharedCount.update(n => n + 1)}>{count}</button>;
};

// Or standalone with Algebraic TSX (no framework)
const app = html`<button onclick=${() => count.update(n => n + 1)}>${count}</button>`;
mount(app, '#app');
```

---

## Current State — v0.3.0

### Rust Crates

| Crate | Tests | Description |
|-------|-------|-------------|
| `cliffy-core` | 85 | FRP primitives, Component trait, dataflow, projections, transforms |
| `cliffy-protocols` | 42 | CRDT, lattice join, delta sync, consensus, storage |
| `cliffy-test` | 30 | Geometric invariants, manifold testing, SMT proof export |
| `cliffy-gpu` | 18 | WebGPU compute + SIMD fallback |
| `cliffy-loadtest` | 15 | Scale simulation (100–10k peers) |
| `cliffy-wasm` | 4 | WASM bindings + Algebraic TSX (html tagged template) |

### TypeScript / JavaScript

| Package | Tests | Description |
|---------|-------|-------------|
| `cliffy-tsukoshi` | 113 (vitest) | Pure TS geometric state + distributed protocols |

### Tooling

- `tools/create-cliffy` — CLI scaffolding (`npx create-cliffy`)
- `vite-plugin` — Vite integration for Cliffy projects
- PureScript bindings (`cliffy-purescript`) with type-safe Html DSL
- 14 example applications deployed on Netlify

---

## Release History

### v0.1.x — First Public Release
- Core FRP primitives (Behavior, Event, combinators)
- WASM bindings with Algebraic TSX
- Distributed state protocols (CRDT, sync, storage)
- Geometric testing framework + GPU acceleration
- 13 example applications, PureScript bindings, `create-cliffy` CLI

### v0.2.0 — P2P Sync + Tooling
- WebRTC-based peer discovery and synchronization
- Delta encoding for efficient state transfer
- Vite plugin and improved scaffolding templates
- Additional examples (p2p-sync, document-editor, multiplayer-game)

### v0.3.0 — Amari 0.19.0 + Protocols + Hardening
- Upgraded to Amari 0.19.0 across workspace (typed rotors, vectors, bivectors available)
- Enhanced cliffy-protocols with improved CRDT and consensus
- Enabled Dependabot for dependency management
- Bumped all packages to v0.3.0

---

## Path to 1.0

### v0.4.0 — Typed Algebra

Replace raw `GA3` / `Multivector` usage with Amari 0.19's typed geometric primitives throughout the codebase.

**cliffy-core**
- [ ] Replace raw GA3 rotation with `Rotor<3,0,0>` via `Rotor::apply()` instead of manual sandwich product
- [ ] Replace raw GA3 vectors with `Vector<3,0,0>` for position/displacement
- [ ] `GeometricState.blend()` via `Rotor::slerp()` for correct manifold interpolation
- [ ] Transform composition via `Rotor::compose()`

**cliffy-protocols**
- [ ] Type CRDT operations: `GeometricOp::Sandwich` → `Rotor<3,0,0>`, `GeometricOp::Exponential` → `Bivector<3,0,0>`
- [ ] Type delta encodings: Additive → `Vector<3,0,0>`, Multiplicative → `Rotor<3,0,0>`, Compressed → `Bivector<3,0,0>`
- [ ] `Rotor::slerp()` for consensus (Fréchet mean on rotor manifold)
- [ ] `norm_squared()` optimization in hot paths (avoid sqrt in lattice comparisons)

**cliffy-test**
- [ ] Bump amari-flynn 0.17.0 → 0.19.0
- [ ] Add SMT proof export to invariants (`ImpossibleInvariant::export_smt()`, `RareInvariant::export_smt()`)
- [ ] Enrich `InvariantTestReport` with statistical bounds (confidence intervals, Hoeffding bounds)

**cliffy-protocols (storage)**
- [ ] `VerifiedMultivector` at deserialization boundaries for storage integrity

### v0.5.0 — Production Polish

Eliminate rough edges, memory leaks, and type holes that block real-world usage.

- [ ] WeakRef-based subscription cleanup (prevent memory leaks from orphaned subscriptions)
- [ ] Wire `initHtml()` / export `DOMProjection` from cliffy-wasm
- [ ] Eliminate `any` types from `html.ts` (proper generic types)
- [ ] Remove `any` types from PureScript Foreign.js (add JSDoc types or convert to TS)
- [ ] `VerifiedMultivector` validation at all deserialization boundaries
- [ ] Performance profiling and optimization of hot paths
- [ ] Audit and fix potential race conditions in subscription graph

### v0.6.0 — API Coherence

Consistent, documented, tested API surface across all bindings.

- [ ] Unified API naming across Rust/TypeScript/PureScript
- [ ] Decision on PureScript bindings: maintain or archive
- [ ] Rewritten documentation (getting-started, API reference, architecture guide)
- [ ] Playwright E2E tests for example applications
- [ ] Deprecate and remove dead code paths
- [ ] Example audit and cleanup (verify all advertised features work)
- [ ] Update `examples/vite.config.shared.ts` for all template types

### v1.0.0 — Stable Release

Public API commitment with semver guarantees.

- [ ] Stable `cliffy-core`, `cliffy-protocols`, `cliffy-test` on crates.io
- [ ] Stable `cliffy-wasm` and `cliffy-tsukoshi` on npm
- [ ] Comprehensive migration guide from 0.x
- [ ] API freeze: breaking changes require 2.0
- [ ] Security audit of distributed protocol layer
- [ ] Performance benchmarks published

---

## 1.x Ecosystem

Post-1.0 work focused on ecosystem growth and integration.

- **Industrial Algebra MCP integration** — Expose Cliffy state management through MCP for AI-assisted collaborative development
- **Additional framework adapters** — Svelte, Solid.js adapters alongside existing React/Leptos/Yew support
- **cliffy-alive stabilization** — Graduate from experimental to optional stable crate (see [Experimental](#experimental-cliffy-alive) below)
- **Community examples and templates** — Expanded `create-cliffy` templates, community-contributed examples
- **Enhanced tsukoshi** — React component library, hooks for framework integration, SSR support

---

## 2.0 Vision

Three pillars for the next major version.

### Higher-Dimensional Algebras

Move beyond GA3 Cl(3,0,0) to richer geometric spaces:

**PGA — Projective Geometric Algebra Cl(3,0,1)**
- Translations become rotors (unified motor algebra)
- Rigid body transformations as single geometric objects
- State spaces that naturally represent position + orientation

**CGA — Conformal Geometric Algebra Cl(4,1,0)**
- Circles, spheres, and inversions as first-class objects
- Intersection and containment tests via inner product
- Natural representation for UI layout constraints and physics

Each algebra upgrade is backward-compatible: existing GA3 code continues to work, but new state types can use richer representations.

### Schubert Calculus for Conflict Resolution

Replace heuristic merge strategies with enumerative geometry from Grassmannians. When two peers produce conflicting state:

1. Model each state as a point on a Grassmannian G(k,n)
2. Compute the Schubert intersection of the two subvarieties
3. The intersection count determines resolution strategy:

| Intersection | Meaning | Action |
|-------------|---------|--------|
| **Finite(0)** | Compatible but unsatisfiable in current algebra | Fallback to higher-dimensional algebra |
| **Finite(n)** | Exactly n valid resolutions | Enumerate and select (deterministic) |
| **Empty** | Structurally incompatible | Reject with proof of incompatibility |

This gives **enumerative merge** — provably correct conflict resolution where the number of valid merges is computed, not guessed. Reference: ShaperOS zero-intersection semantics.

### AI-Composable DSL

An API designed to be:
- **Readable** by humans who don't know geometric algebra
- **Writable** by AI agents who understand geometric semantics (via MCP)
- **Verifiable** via amari-flynn contracts + SMT proof obligations

```
collaborative("doc")
  .with_state(text, cursor)
  .sync(peers, Strategy::GeometricMean)
  .on_conflict(Resolution::Enumerate)
```

Code reads like a domain-specific language while compiling to efficient geometric operations. AI agents can compose applications by combining verified building blocks.

---

## Experimental: cliffy-alive

Living UI as a concurrent research track, developed on feature branches.

**Purpose**: Testbed for ShaperOS / Yatima Sprites concepts — genetic sub-agents that evolve UI behavior through cellular automata rules expressed as geometric algebra operations.

**Current state**: Archived crate with custom CA implementation.

**Future direction**:
- Replace custom CA with amari-automata `GeometricCA`
- `InverseDesigner` for target layout → rule discovery
- `SelfAssembler` / `UIAssembler` for self-assembling UI components
- `CayleyNavigator` for algebraic structure exploration

Ships as an optional experimental crate. Not gated on 1.0 milestones — progresses independently as research allows.

---

## Shelved

### Trebek (React Native + Lynx)

Native mobile rendering layer originally planned as Phase 12. **Shelved** because `cliffy-tsukoshi` already covers mobile and edge use cases as pure TypeScript — it runs anywhere JS runs, including React Native.

May revisit as a standalone project if demand for native geometric rendering materializes.

### archive/ Cleanup

The `archive/` directory contains superseded or abandoned crates: `cliffy-components`, `cliffy-dom`, `cliffy-frp`, `cliffy-gpu`, `cliffy-protocols`, `cliffy-typescript`, `cliffy-wasm-old`. These should be removed. The `archive/cliffy-alive` code should be migrated into the new `cliffy-alive` crate as reference material before deletion.

---

## Development Standards

### Idiomatic Rust
- Leverage the type system fully (enums, traits, generics)
- Prefer `Result`/`Option` over exceptions/nulls
- Use iterators and combinators over manual loops
- Follow Rust API guidelines

### Phantom Types for Type-Level Safety
```rust
pub struct Document<S: DocumentState> {
    content: String,
    _state: PhantomData<S>,
}
impl Document<Draft> {
    pub fn publish(self) -> Document<Published> { ... }
}
```

### Contracts via amari-flynn
```rust
#[requires(probability > 0.0 && probability < 1.0)]
#[ensures(result.probability() == probability)]
pub fn create_rare_event(probability: f64) -> RareEvent<()> { ... }
```

### Rayon for Parallelism
```rust
pub fn parallel_geometric_mean(states: &[GA3]) -> GA3 {
    states.par_iter()
        .map(|s| s.log())
        .reduce(GA3::zero, |a, b| &a + &b)
        .exp()
}
```

### Classical FRP
- `Behavior<T>` = `Time → T` (continuous, time-varying value)
- `Event<T>` = `[(Time, T)]` (discrete occurrences)
- Never implement React-style hooks — the reactive graph handles everything
