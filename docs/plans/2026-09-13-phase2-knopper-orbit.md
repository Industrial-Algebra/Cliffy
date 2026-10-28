# Phase 2 — The Anima Road (Knopper Orbit)

**Date**: 2026-09-13
**Status**: Proposal — awaiting operator review
**Prereqs**: Phase 1 shipped in v0.4.0 (`ObservationSet` + deterministic
projections; `GrantRef` hook present but unused; probe suite green)
**Source sketch**: `docs/plans/2026-08-26-geometric-crdt-salvage.md` §Phase 2

## 1. Intent

Phase 1 made the merge sound by making it boring (set union) and moving all
geometry into deterministic projections. Phase 2 makes the observations
*meaningful across systems* and *admissible by policy*:

1. **U2 alignment** — observation payloads adopt the Knopper encoding
   contract (semantically-true encodings, defined blade assignments). Cliffy
   becomes a named consumer of that contract alongside Knopper machines and
   the web tier.
2. **Schubert-gated observations** — the dormant `GrantRef` hook becomes a
   real capability model: merge admits per policy. A capability-gated CRDT.
3. **GPU-resident projection math** — large observation sets and higher-grade
   payloads offload to `cliffy-gpu` (wgpu/WebGPU). *Re-scoped from the
   original Borsalino item*: Borsalino is out of Cliffy's orbit (operator
   decision, 2026-08 — to-the-metal compute; Cliffy targets browsers).

Explicit non-goals (carried from the salvage plan): no binary geometric-mean
merge (provably non-associative — the G-Set form is the design, not a
workaround); no geometric consensus for state with no manifold structure.

## 2. Workstream 0 — tsukoshi salvage (the forgotten twin)

**Added 2026-09-20** in response to `RABBIT_HOLE_2026-09-20_Cliffy.md`
(IA-documents): the 0.4.0 salvage audited *modules*, not *products* —
`cliffy-tsukoshi` 0.4.0 still ships the pre-salvage unsound surface
(`GeometricCRDT` replay architecture with `nextOpId++` collisions,
`geometricMean`, `geometricJoin` magnitude dominance, a `consensus`
module), with no `ObservationSet` alternative in-package, under the new
org and the README's "No WASM? No problem" parity pitch.

**Operator decision 2026-09-20**: full salvage now, as Phase 2 WS0.

### Tasks

1. **Cross-runtime parity vectors** (the structural fix): a shared JSON
   vector set — observation sets + bit-exact expected `scalar_mean` /
   `vector_mean` / `rotor_consensus` outputs — generated once from the
   Rust implementation, checked in `cliffy-protocols` tests, the wasm
   bindings tests, and tsukoshi's vitest. The determinism contract
   ("equal sets ⇒ equal values", fold order IS the contract) becomes a
   cross-language oracle, answering the report's open question: *can a
   determinism contract survive a language boundary?*
2. **Port the floor to TypeScript** (`cliffy-tsukoshi/src/protocols/`):
   `ObservationSet` (G-Set, `(participant, seq)` keys), `scalarMean`,
   `vectorMean`, and `rotorConsensus` — the Markley accumulator plus a
   faithful port of the in-house cyclic Jacobi (fixed sweep order,
   50-sweep cap, lowest-index tie-break, hemisphere canonicalization
   against the first usable rotor, sign canonicalization). Bit-parity
   with the vector suite is the acceptance test.
3. **Cutover with epitaphs**: delete `GeometricCRDT`, `geometricMean`,
   `geometricJoin`, and the `consensus` module from tsukoshi (the
   componentwise `latticeJoin`/`latticeMeet` floor stays — the salvage
   blessed it). Epitaph notes naming the failure modes, mirroring the
   Rust cutover; README + package docs stop claiming CRDT parity.
4. **Version + release**: breaking removal ⇒ tsukoshi semver bump. The
   ecosystem's shared-version convention makes this a workspace-level
   decision (see §6, new open question 4).

**DoD**: the same JSON vectors pass in `cargo test`, the wasm test lane,
and `vitest`; the deleted surface is gone from the package index; the
report's "two doctrines under one version" criticism is answered by an
oracle, not a paragraph.

## 3. Workstream 1 — U2 encoding alignment

**Problem.** Today's payloads (`RotorObservation {w,x,y,z}` Hamilton,
`VectorObservation`) are Cliffy-local conventions, chosen in Phase 1 because
amari 0.24.1 has no rotor serde or coefficient constructor (gap report §2–3).
They are *correct* but *private*: nothing outside Cliffy can interpret an
observation payload without reading Cliffy source.

**Target.** Observation payloads are expressed in the Knopper encoding
contract's blade assignments — the same encoding Knopper machines and the
web tier consume — so an observation written by a Cliffy browser session is
semantically identical to one written by a Knopper machine.

### Tasks

1. **Contract discovery & pin.** Read the U2 encoding contract from the
   Knopper repo (Unit 2 encoding goal; IA-documents contract lineage) and pin
   a version. Deliverable: `docs/contracts/u2-encoding-pin.md` naming the
   contract version and the exact blade assignments for scalar / vector /
   rotor (and the Grade-2 bivector path if the contract defines one).
2. **Encoding boundary module** (`cliffy-protocols/src/encoding.rs`). Pure
   functions `to_u2(payload) -> U2Payload` / `from_u2(U2Payload) -> payload`
   with the pinned mapping. No changes to merge or projection internals —
   this is a *wire/interop* layer; Phase 1's internal representations stay
   canonical.
3. **Serde + wire schema.** Versioned serde for `U2Payload` with a
   `contract_version` field; rejection of unknown versions.
4. **Round-trip value oracles** (`cliffy-protocols/tests/u2_roundtrip.rs`):
   every payload kind survives `to_u2 ∘ from_u2` exactly; known-answer
   vectors transcribed from the contract document (not self-generated —
   the contract is the oracle); the pinned amari↔Hamilton mapping
   (`w=c[0], x=−c[6], y=+c[5], z=−c[3]`) reconciles with the contract's
   blade assignments or the discrepancy is documented and fenced.

**DoD**: a Knopper-machine-encoded rotor observation, pasted from the
contract's own test vectors, decodes and projects through
`rotor_consensus` to the contract's stated value.

## 4. Workstream 2 — Schubert-gated observations (GrantRef semantics)

**Problem.** `ObservationSet.merge` admits everything. Collaborative
surfaces need *admission control*: this peer may write observations about
the shared document's text but not its ACLs; this grant expires; this
grant covers rotors but not scalars.

**Hook already present**: `GrantRef(pub Uuid)` and
`Observation::with_grant` exist since Phase 1 — deliberately uninterpreted.

### Tasks

1. **Grant model** (`cliffy-protocols/src/grant.rs`). A `Grant` binds:
   grantee (participant `Uuid`), scope (payload kinds / namespaces),
   expiry, issuer. `GrantRef` resolves to a `Grant` via a `GrantStore`
   trait (in-memory impl first; persistence behind the existing
   `GeometricStore` pattern).
2. **Admission policy at merge.** `merge` gains a policy parameter:
   `merge_with_policy(&mut self, other, policy: &impl AdmissionPolicy)`.
   Default policy = admit-all (current behavior; back-compat for 0.4.x
   consumers). Schubert policy = admit only observations whose `GrantRef`
   resolves to a live, in-scope grant. Rejected observations are reported
   (structured `Rejection` log), never silently dropped.
3. **Lattice-law preservation proof.** The gated merge must remain a
   semilattice join *on the admitted sublattice*: property tests assert
   associativity/commutativity/idempotence for fixed policy + grant store
   state. Document the temporal caveat (an expiring grant changes the
   admitted set — convergence holds per epoch, and re-admission is
   monotone: an observation once admitted is never revoked by merge).
4. **Schubert bridge.** The grant *issuance* side lives in Schubert
   (capabilities-as-conditions; `schubert_cell_of` is the named encoder per
   the Knopper discovery work). Cliffy consumes an issued grant document
   (serde schema) and verifies signature/expiry locally — no online
   dependency at merge time.
5. **WASM surface.** `ObservationSet.mergeWithPolicy` + grant
   registration bindings; the crdt-playground gains a "gate" panel
   admitting/rejecting a peer's observations live.

**DoD**: two peers, one holding an expired text-scope grant and one a live
full-scope grant, exchange observation sets — value oracles assert exactly
which observations each peer's merged set contains, and that re-merging is
idempotent under both policies.

## 5. Workstream 3 — GPU-resident projections

**Problem.** `rotor_consensus`'s Jacobi eigensolve is O(n) in observations
plus a fixed 4×4 solve — fine at hundreds of observations. The Anima road
(the Grassmannian tier Schubert arbitration needs; large multi-participant
sets) wants orders of magnitude more, in the browser.

### Tasks

1. **Projection kernels in cliffy-gpu** (wgpu compute + the existing SIMD
   fallback via `wide` 1.7): batched quaternion-accumulator reduction for
   Markley; componentwise reductions for scalar/vector means.
2. **CPU/GPU parity value oracles**: identical inputs → identical outputs
   within a stated tolerance (deterministic CPU path remains the oracle;
   GPU path must match or the kernel is wrong — no "GPU is approximate"
   hand-wave).
3. **Bench gate**: `cliffy-core/benches`-style criterion benches at
   1k/10k/100k observations; the GPU path must win at 10k+ or it doesn't
   ship (wgpu dispatch overhead is real; honesty over novelty).

**DoD**: `rotor_consensus` over 10⁵ observations, GPU vs CPU parity
asserted in CI (SIMD fallback lane), bench numbers in the PR body.

## 6. Sequencing

WS1 and WS2 are independent and can run in parallel; both are pure
`cliffy-protocols` + WASM work. WS3 is independent of both. Suggested
order within the cycle: WS2 (grant model) → WS1 (encoding) → WS3 (GPU) —
the grant model is the least externally-blocked (WS1 pins an external
contract version; WS3 has hardware-variance risk in CI).

WS0 lands first (it is unblocked, internal-only, and closes a published
soundness gap). Then WS2 → WS1 → WS3 as before.

Each workstream lands as its own PR chain with its own probe/value-oracle
suite, following the Phase 1 pattern (plan doc → RED probes → GREEN →
cutover). Version target: **0.5.0**.

## 7. Open questions for the operator

0. *(Added 2026-10-10 — see §8.)* The Amari remediation changes the
   answers available below; read §8 first.
1. **Knopper contract version**: is the U2 encoding contract published at a
   pin-able version (IA-documents contract lineage), or is Cliffy WS1 the
   forcing function to version it?
2. **Grant signature scheme**: Schubert-issued grants — plain serde docs
   verified by structure, or signed (ed25519)? Signature adds real
   capability security at the cost of key management in the browser.
3. **Does 0.5.0 scope = all workstreams**, or is WS3 (GPU) allowed
   to slip to 0.5.x while the consensus semantics stabilize?
4. **Version coherence for WS0**: tsukoshi's breaking salvage under the
   shared-version convention — whole workspace to 0.5.0 (Rust crates
   unchanged but re-published for parity), or tsukoshi-only 0.5.0
   (explicit divergence, breaking the one-version-everywhere story the
   rabbit hole criticized)?

## 8. Amendment — Amari core audit and remediation (2026-10-10)

The Amari core-foundations audit (PR #276, merged 2026-10-09) found
mathematical inaccuracies in `amari-core`, and the maintainer's policy is
explicit: *mathematical correctness is paramount; compatibility is not a
constraint on these repairs; downstream consumers will be updated to the
corrected mathematics.* The remediation programme (CORE-W00..W09) lands
with **Amari 0.25.0**. Cliffy consumes `amari-core 0.24.1` — every audit
defect is live in our published substrate until we deliberately adopt
0.25.

### Cliffy's exposure map (audited 2026-10-10)

| Site | amari API | Audit finding | Fixed upstream? |
|---|---|---|---|
| `cliffy-core/src/transforms.rs` | `Rotor::inverse()` ×3 | CORE-01 (inverse wrong outside versor domain) → W03 | **No** |
| `cliffy-core/src/transforms.rs` | `Rotor::slerp()` ×3 | CORE-11 (rotor log/power loses double cover) → W07 | **No** |
| `cliffy-protocols/src/delta.rs` | `.exp()` ×1 | CORE-03/04 (exp dispatch) → W04 | **No** (GA3-Euclidean bivectors square to negative scalars; the demonstrated nilpotent/nonsimple failures are out-of-signature, but this is unverified-by-oracle, not verified) |
| coefficient arithmetic, `projection.rs`, `state.rs` | products, norms | CORE-02 (1e-14 pruning breaks associativity) → W01 | **Yes** (#282, on Amari develop → 0.25.0) |
| null/metric semantics | `norm()` | CORE-07 → W02 | **Yes** (#285, → 0.25.0) |
| `cliffy-protocols` wire path | conversions only (`from_amari_rotor`, `from_components`) + own `eigen.rs` | none — deliberately self-contained | n/a |
| **tsukoshi (pure TS)** | none | none | n/a |

### Amendments to this plan

1. **Open question 4 is answered by this table.** Ship **tsukoshi-only
   0.5.0** (WS0): tsukoshi has zero amari exposure, so its breaking
   release is clean regardless of the substrate. Defer the Rust workspace
   version bump until Amari 0.25 adoption — publishing a Rust 0.5.0 on
   known-defective 0.24.1 math would put a fresh version number on
   unfixed CORE-01/11 slerp/inverse paths.
2. **New workstream WS-A — Amari 0.25 adoption** (gated on Amari 0.25.0
   release; slot after WS2, before or parallel with WS1):
   - Bump `amari-core` to 0.25 and adopt breaking changes per the
     remediation's downstream guidance.
   - Review the audit diffs for `inverse`, `slerp`, `exp` specifically;
     adjust `transforms.rs` call sites (the versor-domain posture the
     audit blesses is the one Cliffy should already be in — rotors, not
     general multivectors).
   - Re-run the full matrix plus the WS0 parity vectors (expected stable:
     the wire projections don't consume amari numerics) and the
     geometric-mean invariants in `cliffy-test`.
   - Adoption evidence doubles as Amari's downstream-consumer validation
     — share results upstream.
3. **Until WS-A lands**: the 0.4.x/0.5.x documentation states plainly
   that `cliffy-core` rotor transforms inherit amari-core 0.24.1's known
   defects (CORE-01, CORE-11) and that the wire-critical path
   (ObservationSet + projections) does not. No emergency patch: the
   affected paths are single-node transform helpers, not the distributed
   soundness core — but users doing rotor interpolation deserve the
   warning.
4. **Sequencing**: WS0 (merging) → WS-L → WS2 → **WS-A (when 0.25.0
   ships)** → WS1 → WS3. If Amari 0.25.0 slips past WS1's completion, WS1
   proceeds (it is protocols-layer; its U2 contract is encoding, not
   numerics).

### WS-A empirical scope (probed 2026-10-16 against Amari develop @ 5f6ecce)

The full core remediation has landed on Amari develop (W00–W04 merged
through #304; W04 rotor/duality survived 5 review rounds, F1–F21) but is
**unreleased** — develop's amari-core is still 0.24.1, with 0.25.0 pending
amari-rewrite completion. A scratch worktree probe (path dependency, full
workspace check) gives the exact adoption scope:

**Compile breakage: 18 errors, confined to cliffy-core (9) and
cliffy-test (9); cliffy-protocols/wire path has ZERO direct errors**
(§8's "amari-light" claim, empirically confirmed).

| New API (remediated) | Old call | Cliffy sites |
|---|---|---|
| `Multivector::coefficient_norm()` (L2 of coefficients, explicitly not metric) | `.magnitude()` | 8: test/error.rs, test/generators.rs ×3, test/manifold.rs ×3, core/geometric.rs, core/projection.rs, core/state.rs, core/transforms.rs |
| `normalize() -> Option<Self>` (Some only for positive metric square; null → None, no tolerance) | `.normalize() -> Self` | 4: core/transforms.rs ×2, core/state.rs ×2 — each needs a policy decision (propagate vs. fallback) |
| `Rotor::from_bivector() -> Result<Self, RotorError>` | bare value | 1: core/transforms.rs `to_amari_rotor` |
| `Rotor::{inverse, slerp, logarithm, power} -> Result<_, RotorError>` | bare values | **0 direct sites** — cliffy-core's own Rotor wrapper owns inverse/slerp internally |

**The silent class (compiles clean, semantics changed) — the real WS-A
work:** W01 removes the 1e-14 coefficient pruning (bit-exact outputs in
cliffy-core paths may shift; the wire path is unaffected — own Jacobi);
W03 corrects general inverse/exp domains (Cliffy's sites are
GA3-Euclidean versors, likely behavior-equal, must be verified by the
matrix, not assumed); W02's equality/metric policy may move
`approx_eq`-based test expectations. WS-A's acceptance gate: full matrix
+ parity vectors + phase-1 probes green on the remediated substrate, with
any bit-shift explicitly attributed and dispositioned.

## 9. Workstream L — loadtest salvation (queued 2026-10-16)

Source: `IA-documents/RESEARCH_REPORTS/RABBIT_HOLE_2026-10-16_Cliffy.md`
("The Instrument That Was Never Run"). All headline findings verified
firsthand against `develop`:

- `cliffy-loadtest/src/simulator.rs` carries the fossil merge — `(a+b)*0.5`
  under the doc comment "CRDT merge via geometric mean," the exact
  non-semilattice doctrine the salvage executed, reimplemented inline
  (born 2026-01-29, `cdd05fc`, untouched by the salvage window).
- `test_simulation_convergence` asserts `check_count > 0` — the
  vacuous-success predicate, fourth instance in this repo (annihilated-
  replica agreement; publish workflow; loop counter; zero-test CI).
- CI's `scale_100`/`scale_1000` lines select zero tests (the crate has no
  `#[ignore]`d tests; the names are config constructors), wrapped in
  `|| true` + `continue-on-error` — the scale suite has never executed.
- `cliffy-protocols`, `cliffy-test`, and `tokio` are declared and unused;
  the README documents an API (`Scenario::builder`, `convergence_rate`,
  `p99_latency_ms`) that does not exist.
- The crate wears 0.4.0 without the salvage's content; unpublished on
  crates.io, so exposure is future-internal — Phase 2 evidence and the
  10,000-user claim would be its first real consumers, and its numbers
  would be wrong in the *plausible* way.

### The cure (operator-approved shape; execution queued after #342/#333 merge)

1. `SimulatedUser` becomes an `ObservationSet` client: users emit
   timestamped observations; merge = union (the boring semilattice).
   `network.rs` (topology/latency/jitter/loss) is doctrine-free plumbing
   and survives untouched.
2. Convergence assertions become value-oracular: after sync quiescence,
   every replica's set equals the union of all emitted observations (SEC),
   and projections are bit-identical across replicas. Threshold-vs-
   reference divergence is deleted, not relaxed.
3. Real `#[ignore]`d `scale_100`/`scale_1000` integration tests; CI lines
   lose `|| true` and select tests that exist. A vacuity probe family
   fossilizes all four instances (assert the selector matches >0 tests;
   assert `converged`, not `check_count`).
4. Wire the phantom deps: `cliffy-test` invariants on end-state,
   `cliffy-protocols` real, `tokio` used or dropped. README reconciled to
   the shipped API (the fictional builder API may be built deliberately
   later, or dropped).
5. CHANGELOG honesty line: 0.4.0's loadtest entry was version-bump-only.

### Sequencing

Amari-independent (no amari numerics in the cure), Rust-only (no
interaction with #342). Order: **#342/#333 merges → WS-L → WS2 → WS-A
(Amari 0.25) → WS1 → WS3**. WS-L goes first among Phase 2 workstreams
because Phase 2's own evidence needs (latency under load, convergence at
scale) consume this instrument; it must be trustworthy before it is cited.
