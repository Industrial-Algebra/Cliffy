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
