# Technical Backlog

Work items organized by target release. See [ROADMAP.md](./ROADMAP.md) for full context on each milestone.

---

## v0.4.0 — Typed Algebra

### cliffy-core: Typed Rotor API

- [ ] Replace raw GA3 rotation with `Rotor<3,0,0>` — `GeometricState` rotation via `Rotor::apply()` instead of manual sandwich
- [ ] Replace raw GA3 vectors with `Vector<3,0,0>` — position/displacement with `Vector::normalize()` and `Vector::norm()`
- [ ] `GeometricState.blend()` via `Rotor::slerp()` for correct manifold interpolation
- [ ] Transform composition via `Rotor::compose()`
- [ ] Rotor construction via `Rotor::from_vectors()` where applicable

### cliffy-protocols: Type-Safe Distributed State

- [ ] Type CRDT operations with Rotor/Vector/Bivector
  - `GeometricOp::Sandwich` → takes `Rotor<3,0,0>`, uses `Rotor::apply()`
  - `GeometricOp::Exponential` → takes `Bivector<3,0,0>`, uses `exp()` then `Rotor::apply()`
  - Operation composition via `Rotor::compose()`
- [ ] Type delta encodings
  - `DeltaEncoding::Additive` wraps `Vector<3,0,0>` (displacement)
  - `DeltaEncoding::Multiplicative` wraps `Rotor<3,0,0>` (transformation)
  - `DeltaEncoding::Compressed` wraps `Bivector<3,0,0>` (log-space)
  - Use `Rotor::logarithm()` and `Multivector::exp()` for compressed encoding
- [ ] Use `Rotor::slerp()` for consensus — iterative slerp for Fréchet mean, `Rotor::power(weight)` for weighted proposals
- [ ] `norm_squared()` for magnitude comparisons — replace `magnitude()` in lattice comparisons, use `grade_magnitude()` for grade-aware operations
- [ ] Storage integrity with `VerifiedMultivector` — validate multivectors on snapshot load (requires amari-core `phantom-types` feature)

### cliffy-test: SMT Backend + Statistical Bounds

- [ ] Bump amari-flynn 0.17.0 → 0.19.0 (check rand 0.8 compatibility)
- [ ] Add SMT proof export to invariants
  - `ImpossibleInvariant::export_smt()` → `precondition_obligation(name, desc, 1.0)`
  - `RareInvariant::export_smt()` → `hoeffding_obligation(name, samples, epsilon, delta)`
  - Both return `SmtProofObligation` for `.smt2` files
  - Add `verify_with_smt()` for Monte Carlo vs SMT cross-verification
- [ ] Enrich `InvariantTestReport` with statistical bounds
  - `confidence_interval: Option<(f64, f64)>` field
  - Use `MonteCarloVerifier::estimate_probability()` → (estimate, lower, upper)
  - Use `hoeffding_bound(n, epsilon)` for required sample count
  - Use `confidence_interval(successes, total, confidence)` for CI

---

## v0.5.0 — Production Polish

- [ ] Implement `initHtml()` integration / export `DOMProjection` from cliffy-wasm
- [ ] Add TypeScript type definitions for `html.ts` (eliminate `any` casts)
- [ ] Add cleanup for orphaned subscriptions (WeakRef-based disposal pattern)
- [ ] Remove `any` types from PureScript Foreign.js (JSDoc types or convert to TS)
- [ ] Performance profiling of hot paths (subscription propagation, geometric operations)
- [ ] Audit race conditions in subscription graph

---

## v0.6.0 — API Coherence

- [ ] Unify TypeScript/PureScript API naming (document intentional differences, fix unintentional ones)
- [ ] PureScript bindings: decide maintain vs archive
- [ ] Expand Playwright E2E tests (per-example tests, cross-browser, performance regression)
- [ ] Example audit — verify all advertised features work (design-tool completeness, etc.)
- [ ] Update `examples/vite.config.shared.ts` for all template types
- [ ] Document editor enhancements:
  - [ ] Wire up Force Sync button with visible CRDT merge
  - [ ] Add remote user cursor position updates
  - [ ] Show CRDT operation history visualization
- [ ] Rewritten documentation (getting-started, API reference, architecture guide)
- [ ] Add example CI workflow (build all examples, run example-specific tests)
- [ ] Remove `archive/` — migrate `archive/cliffy-alive` into new cliffy-alive crate as reference, delete the rest
- [ ] Deprecate and remove dead code paths

---

## Deferred / 2.0

- [ ] Higher-dimensional algebras: PGA Cl(3,0,1) and CGA Cl(4,1,0)
- [ ] Schubert calculus conflict resolution (Grassmannian intersections, enumerative merge)
- [ ] AI-composable DSL design
- [ ] cliffy-alive: Replace custom CA with amari-automata `GeometricCA`
  - `InverseDesigner` for target layout → rule discovery
  - `SelfAssembler` / `UIAssembler` for self-assembling UI
  - `CayleyNavigator` for algebraic structure exploration

---

## Completed

<details>
<summary>Completed items (click to expand)</summary>

### Export/Packaging
- [x] Export `html.ts` from `cliffy-wasm` package — post-build script copies to pkg/, import via `cliffy-wasm/html`
- [x] Create main `Cliffy.purs` module — FRP primitive re-exports
- [x] Wire up PureScript package dependencies — git dependency in `packages.dhall.template`

### Documentation
- [x] Document Behavior.map() vs Array.map() distinction
- [x] Create Algebraic TSX migration guide (docs/migration-guide.md)
- [x] Update CLAUDE.md code examples with Algebraic TSX
- [x] Update getting-started.md with Algebraic TSX
- [x] Create API reference with rendering examples
- [x] Add Algebraic TSX section to ROADMAP.md
- [x] Audit existing examples for Algebraic TSX
- [x] Update README.md quick start with `npx create-cliffy`
- [x] Create cliffy-purescript README
- [x] Document PureScript FFI patterns (docs/purescript-ffi-patterns.md)

### Examples
- [x] Collaborative Whiteboard — real-time drawing with geometric transforms
- [x] Multiplayer Game — high-frequency state sync, interpolation
- [x] Shared Document Editor — CRDT text, presence indicators
- [x] Design Tool — complex geometric operations, undo/redo
- [x] tsx-counter — basic `html` tagged template usage
- [x] tsx-todo — list rendering, component composition
- [x] tsx-forms — form validation, GA-inspired API
- [x] purescript-counter — Cliffy.Html DSL
- [x] purescript-todo — ADTs for state, list rendering
- [x] testing-showcase — cliffy-test geometric invariants
- [x] geometric-transforms — rotor rotations, composition, slerp
- [x] crdt-playground — geometric CRDT operations
- [x] p2p-sync — WebRTC state synchronization
- [x] gpu-benchmark — WebGPU/WASM performance testing

</details>

---

## References

- [ROADMAP.md](./ROADMAP.md) — Full development roadmap
- [CLAUDE.md](./CLAUDE.md) — Development standards and architecture
