# Phase 1 — The Sound Floor: ObservationSet + Deterministic Geometric Projection

> **REQUIRED SUB-SKILL:** Execute task-by-task (executing-plans / subagent-driven).

**Goal:** Implement salvage Phase 1 (docs/plans/2026-08-26-geometric-crdt-salvage.md §4): a genuinely convergent CRDT floor — `ObservationSet` (G-Set of attributed observations, merge = set union) with geometry as a *deterministic projection* (Markley chordal-L₂ eigen-mean for rotors; componentwise means for scalars/vectors) — proven by a value-oracle test suite ported from the 2026-08-25 rabbit-hole probes, then cut the repo over (delete the deprecated machinery, replace wasm bindings, migrate the examples).

**Architecture:** The CRDT is the set; the geometry is a pure function of the set. `merge` = BTreeMap union (associative, commutative, idempotent by construction). Consensus projections iterate observations in **canonical key order** `(participant_id, seq)` with **sequential folds** — equal sets ⇒ bit-identical values across replicas. Rotor consensus: sign-canonicalize against a set-determined reference, build `M = Σ wᵢ qᵢ qᵢᵀ`, dominant eigenvector via a deterministic cyclic Jacobi sweep (no external linalg dependency).

**Tech Stack:** amari-core 0.23 (`Rotor<3,0,0>`, `Vector<3,0,0>`, `Bivector`), quickcheck (property laws), wasm-bindgen (JS surface), existing `VectorClock`.

**DoD (from the salvage plan):** *the February question is answered by a test suite, not a report* — every verified failure mode (annihilation, cosh(1) join, op-ID collision, vacuous convergence) has a permanent value-oracle test that fails against the old design's outputs.

---

## Determinism contract (the design core)

1. **Canonical iteration**: `BTreeMap<ObservationKey, Observation>` — key = `(participant_id: Uuid, seq: u64)`. No HashMap anywhere in the set.
2. **Sequential folds** in all projections — no rayon (float summation order is part of the contract).
3. **Sign canonicalization** is set-determined: the first observation in canonical order is the hemisphere reference (`dot(q, ref) < 0 ⇒ negate q`). Never a local convention.
4. **Weights** (future grant tiers) must be pure functions of set-determined metadata — the `weights: impl Fn(&Observation) -> f64` hook documents this contract now.
5. The eigen-sum M is computed on demand in Phase 1 (derived cache is a later optimization — never canonical state).

## Rotor ↔ quaternion mapping (verify, don't assume)

amari `Rotor<3,0,0>` = even multivector `(scalar, e12, e13, e23)`. Multivector coefficients (GA3 layout, confirmed in cliffy-gpu docs): `[scalar, e1, e2, e12, e3, e13, e23, e123]`. Candidate mapping: `q = (w: c0, x: c3, y: c5, z: c6)` — **sign conventions must be pinned by a test** against `Rotor::from_axis_angle` + `apply_to_vector` (Task 4). All mapping code lives in one place (`projection.rs::rotor_to_quat` / `quat_to_rotor`).

## The Markley eigen-mean (reference: Markley et al. 2007, "Averaging Quaternions")

For unit quaternions `qᵢ` with weights `wᵢ`: the chordal-L₂ mean is the dominant **eigenvector** of the symmetric 4×4 profile matrix `M = Σ wᵢ qᵢ qᵢᵀ`. We implement cyclic Jacobi eigensolve internally (deterministic sweep order, 50-sweep cap, 1e-15 off-diagonal tolerance, max-eigenvalue tie-break to the lowest index) — no nalgebra dependency, ~80 lines, fully testable against known eigenpairs.

---

## PR 1 — `feature/phase1-observation-set` (additive, TDD)

Off `origin/develop`. Quickcheck added as dev-dep to cliffy-protocols.

### Task 1: The probe suite, RED first

**Files:** create `cliffy-protocols/tests/phase1_probes.rs` (integration test — the permanent "February answer" suite). Add this plan doc. 

Tests (names fixed — these are the oracles):

```rust
// 1. No annihilation: scalar observations +5 (node A) and +10 (node B);
//    after merge, scalar_mean == 7.5 EXACTLY. (Old design: merged.state == 0.)
#[test] fn probe_merge_does_not_annihilate()

// 2. Hull property: scalars +1 and -1 → mean == 0.0 EXACTLY.
//    (Old design: join(+1,-1) = cosh(1) = 1.543080634815244 — outside the hull.)
#[test] fn probe_join_stays_in_hull()

// 3. Participant-scoped identity: two nodes' FIRST observations coexist —
//    keys distinct, len == 2 after merge. (Old: both minted id 0; one dropped.)
#[test] fn probe_op_ids_are_participant_scoped()

// 4. Convergence WITH a value oracle: replicas diverge (10 vs 5), merge in
//    both directions → identical sets AND scalar_mean == 7.5 in both.
//    (Old test asserted only that two annihilated replicas agreed.)
#[test] fn probe_convergence_with_value_oracle()
```

Run: `cargo test -p cliffy-protocols --test phase1_probes` → **compile error** (ObservationSet doesn't exist) = RED. Commit.

### Task 2: Observation + ObservationSet (G-Set)

**Files:** create `cliffy-protocols/src/observation.rs`; modify `lib.rs` (module + re-exports).

```rust
/// Capability grant reference — opaque until Phase 2 (Schubert gating).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GrantRef(pub Uuid);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ObservationPayload {
    Scalar(f64),
    Vector(Vector<3, 0, 0>),   // amari typed vector; serde via coefficients if needed
    Rotor(Rotor<3, 0, 0>),     // amari typed rotor
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    pub participant_id: Uuid,
    pub seq: u64,
    pub clock: VectorClock,
    pub payload: ObservationPayload,
    pub grant_ref: Option<GrantRef>,
}

pub type ObservationKey = (Uuid, u64);

/// Grow-only set of attributed observations. Merge is plain union —
/// associative, commutative, idempotent by construction. The consensus
/// value is a deterministic projection (see `projection`), never merge state.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ObservationSet {
    observations: BTreeMap<ObservationKey, Observation>,
}

impl ObservationSet {
    #[must_use] pub fn new() -> Self
    /// Insert; returns false if the key already exists (idempotent).
    pub fn insert(&mut self, obs: Observation) -> bool
    /// Union-merge: absorbs every observation whose key is absent. Returns true if changed.
    pub fn merge(&mut self, other: &Self) -> bool
    #[must_use] pub fn len(&self) -> usize   // + is_empty (clippy)
    #[must_use] pub fn contains(&self, key: &ObservationKey) -> bool
    /// Canonical-order iteration (determinism contract).
    #[must_use] pub fn iter(&self) -> impl Iterator<Item = &Observation>
}
```

Notes: derive `PartialEq` on `VectorClock` if missing (additive). Payload serde: `Rotor`/`Vector` serialize via their multivector coefficients (`serde_ga3` pattern — implement `serde_with`-style helpers or store coefficients in the enum; decide at implementation, keep wire format `[f64; 8]`-compatible where possible). Unit tests: insert idempotence, merge union, key ordering. Run probes 2+3 → green; 1+4 still RED (no projections yet — they compile only after Task 6/7; keep projection calls in place, suite stays red until then). Commit.

### Task 3: Set-law property tests

**File:** `cliffy-protocols/tests/observation_laws.rs`, quickcheck dev-dep.

Arbitrary observations: random `Uuid`, small seq, payload = Scalar(rand) (rotors join in Task 6's law tests). Properties:
- `merge(a, b) == merge(b, a)` (set equality + identical iteration)
- `merge(merge(a,b),c) == merge(a,merge(b,c))`
- `merge(s, s) == s` (idempotence)
- merge change-detection is consistent with key-set delta

RED → GREEN. Commit.

### Task 4: Rotor↔quaternion mapping, pinned

**File:** `cliffy-protocols/src/projection.rs` (create; private helpers for now).

Test first (`projection.rs` unit tests): `from_axis_angle(z, π/2)` applied to `(1,0,0)` gives `(0,1,0)` (via `apply_to_vector`); the mapped quat is `(cos π/4, 0, 0, ±sin π/4)` — pin the sign; roundtrip `quat_to_rotor(rotor_to_quat(r))` ≡ identity under `apply_to_vector` for random rotors (quickcheck, tolerance 1e-12). If amari's convention differs from the candidate mapping, fix the mapping constants — one place, test-pinned. Commit.

### Task 5: Deterministic 4×4 symmetric eigensolve

**File:** create `cliffy-protocols/src/eigen.rs` (pub(crate)).

```rust
/// Cyclic Jacobi eigensolve for symmetric 4×4. Deterministic: fixed sweep
/// order, 50-sweep cap, off-diagonal Frobenius tolerance 1e-15.
/// Returns (eigenvalues, eigenvectors-as-columns), sorted is NOT applied —
/// callers pick max eigenvalue with lowest-index tie-break.
pub(crate) fn jacobi_eigen_4(m: [[f64; 4]; 4]) -> ([f64; 4], [[f64; 4]; 4])
```

Unit tests: known diagonal matrix; known 2×2-embedded case (`diag(2,1)` rotated); rank-1 outer product `q qᵀ` → dominant eigenvector ∥ q (this is the Markley consistency check); degenerate ties return a deterministic (repeatable) answer — same input twice ⇒ bit-identical output. Commit.

### Task 6: Rotor consensus (Markley) — the probes go green

**File:** `projection.rs` grows the public API (re-export from lib.rs).

```rust
/// Deterministic rotor consensus: Markley chordal-L₂ mean.
/// None if the set holds no usable Rotor payloads.
#[must_use]
pub fn rotor_consensus(set: &ObservationSet) -> Option<Rotor<3, 0, 0>>
#[must_use]
pub fn rotor_consensus_with_weights(set: &ObservationSet, weights: impl Fn(&Observation) -> f64) -> Option<Rotor<3, 0, 0>>
```

Algorithm (all in canonical order, sequential):
1. Filter Rotor payloads; skip non-normalizable (|m| < 1e-12, documented).
2. Hemisphere-canonicalize against the first surviving rotor.
3. Accumulate `M += wᵢ · outer(qᵢ, qᵢ)`.
4. `jacobi_eigen_4(M)` → dominant eigenvector (lowest-index tie-break) → normalize.
5. Canonicalize the result's own sign (w ≥ 0; lexicographic fallback) → `quat_to_rotor`.

Unit tests: single rotor → itself; duplicate rotor → same; `q` and `-q` → equals `q` alone (double cover); `+90°z` and `−90°z` → identity rotor (analytic); weight-2 dominance matches duplicating the observation; **determinism**: three different merge orders reaching the same set ⇒ bit-identical consensus (exact component equality). GREEN: probes 1+4 compile & pass (they use scalar_mean — arrives next task; if ordered after, swap task order so the suite goes fully green here). Commit.

### Task 7: Scalar & vector projections

**File:** `projection.rs`.

```rust
#[must_use] pub fn scalar_mean(set: &ObservationSet) -> Option<f64>
#[must_use] pub fn vector_mean(set: &ObservationSet) -> Option<Vector<3, 0, 0>>
```

Canonical-order arithmetic means (vector components via multivector coefficients `get(1), get(2), get(4)`). Tests: exact value oracles; mixed-payload sets only aggregate their kind. **Full probe suite green** — this is the Phase 1 DoD moment: `cargo test -p cliffy-protocols --test phase1_probes` all pass. Commit.

### Task 8: Surface polish + full matrix

- lib.rs crate docs: the §8.C pattern (raw canonical set + replayable projection), the determinism contract, a doctest mirroring the handoff's example.
- Feature-gate check: observation/projection modules need no features (serde always on, matching current crate shape).
- Full verification matrix, **both toolchain lanes** (pinned nightly + `RUSTUP_TOOLCHAIN=stable`), plus `cargo doc` zero warnings.
- Push, open PR (stacked directly on develop — nothing else open above it).

## PR 2 — `feature/phase1-cutover` (delete dead machinery, wasm + examples)

Off PR 1's branch. One atomic cutover — wasm/examples break if split.

### Task 9: Delete the deprecated Rust surface

- `crdt.rs`: delete `GeometricCRDT`, `GeometricOperation`, `OperationType`, `geometric_mean` (the whole file's public surface; keep the module file only if tests/helpers remain — else delete module).
- `lattice.rs`: delete `GA3Lattice` + its trait impl + tests; **keep `GeometricLattice` trait + `ComponentLattice`** (the sound floor).
- `consensus.rs`: delete entirely (built on the broken machinery; Phase 2 rebuilds consensus on the sound floor — check no other consumers first: `grep -r "GeometricConsensus\|consensus::" cliffy-*/src`).
- lib.rs re-exports, module docs; CHANGELOG `[Unreleased] → Removed` section (breaking, with migration pointer to ObservationSet).
- Deprecation-comment references in wasm/test files that said "dies with Phase 1" — they die here.

### Task 10: WASM surface cutover

`cliffy-wasm/src/protocols.rs`: replace `GeometricCRDT` bindings with `ObservationSet` + projections:
`new ObservationSet()`, `observeScalar(nodeId: &str, seq: f64|BigInt, value: f64)`, `observeRotor(nodeId, seq, w, x, y, z)` (normalized inside), `merge(other)`, `len()`, `scalarConsensus()`, `rotorConsensus() -> [w,x,y,z]` (JS array). `VectorClock` bindings unchanged. u64 via wasm-bindgen BigInt (modern browsers) — if ergonomics fight back, fall back to f64 seq with validation. Update lib.rs re-exports + the crate doc's deprecated CRDT section → ObservationSet example. Update wasm unit tests.

### Task 11: Example migration (5 examples)

- `crdt-playground` — the showcase: peers hold `ObservationSet`s; concurrent scalar edits; merge; display **scalarConsensus** value. Add a "probe panel" that renders the four Phase 1 oracle checks live (the annihilation story, visualized). This example becomes the demo of the salvage.
- `document-editor`, `multiplayer-game`, `p2p-sync` — mechanical: `new GeometricCRDT(id, x)` + `.add()` + `.merge()` call-sites → `ObservationSet` observe/merge + the projection call for display state. TS types via the new bindings.
- `testing-showcase` — imports only; swap types.
- Examples build in CI against the fresh wasm artifact — Browser E2E must stay green.

### Task 12: Docs + BACKLOG + full matrix

- BACKLOG.md: check off salvage Phase 1 items; re-scope v0.4.0 typed-algebra lines that assume amari 0.19 APIs (rotor/vector typing now REAL in observation.rs — note what remains: migrating cliffy-core transforms to typed rotors).
- ROADMAP.md v0.4.0 section: reflect the salvage shape (merge boring, render geometric).
- CHANGELOG: Removed/Added entries under Unreleased.
- Full matrix both lanes + `wasm-pack build` locally + examples typecheck (`npm run typecheck` in examples).

## Post-phase (not this plan)

1. `ia-version-bump` → **0.4.0** (breaking removals justify the minor in 0.x), then gitflow release when the operator wants to ship.
2. Phase 2 proposals (cross-repo, propose-don't-depend): Knopper U2 encoding contract alignment, Schubert-gated observations (GrantRef gets real semantics). ~~Borsalino offload~~ — **corrected (operator, 2026-08-27): dropped.** Borsalino is a to-the-metal GPU compute library; Cliffy targets web browsers, where the GPU path is wgpu/WebGPU via `cliffy-gpu`. Large-set merge-math, if ever needed in-browser, goes through WebGPU compute — not a native offload.
3. Derived M cache only if profiling demands it (bench first).

## Non-goals (explicit)

- No capability *enforcement* (GrantRef is carried, not checked — Phase 2).
- No consensus *protocol* machinery (rounds/votes — Phase 2 on the sound floor).
- No slerp-based consensus; the eigen-mean is the Phase 1 spec (slerp iterative means are order-dependent — exactly what died).
- No CRDT text/sequence types (document-editor keeps its TS-local editing; ObservationSet syncs presence/state).
