# Geometric CRDT Salvage — Design + Roadmap

**Date:** 2026-08-26
**Trigger:** `RABBIT_HOLE_2026-08-25_Cliffy.md` (independently verified: merge
annihilation to zero, cosh(1) join, op-ID collisions, vacuous convergence test).
**Status of the code today:** `cliffy-protocols::crdt` and `GA3Lattice::join` are
**broken and must not be consumed downstream**. `ComponentLattice`,
`vector_clock`, and the cliffy-core FRP reactivity are sound.
**Purpose:** own the broken math publicly, and give Cliffy the road to the
defensible geometric-consensus path (Knopper identity-restoration orbit: typed
rotors, slerp/Fréchet means, Borsalino for heavy merge-math).

---

## 1. Verdict

Salvageable — **as a redesign, not a patch.**

- Three mechanical bugs (dead replay loop, `len()`-minted op IDs, vacuous
  convergence oracle) are afternoon fixes.
- The *semantics* were wrong: a magnitude-dominance join with an exp-based
  tie-break is not a join-semilattice, and no corrected interpolation fixes
  that class — see §2.
- The idea survives in a smaller, sounder form: **"conflict = distance,
  resolution = interpolation on a manifold" is correct *as a deterministic
  projection over an attributed observation set*** — not as a merge function.

## 2. The load-bearing design insight

A binary Fréchet/geometric mean is commutative and idempotent but **not
associative** (mean-of-means ≠ grand mean). Therefore *no* mean-based binary
`merge` forms a join-semilattice, and a state-based CRDT built on one cannot
guarantee strong eventual consistency. This is not an implementation flaw;
it is why the original architecture could not be fixed in place.

The sound architecture separates the two concerns:

- **The CRDT is an `ObservationSet`**: a grow-only set of *attributed*
  observations — `(participant_id, seq, vector_clock, payload, grant_ref)` —
  whose merge is plain set union. Union is associative, commutative,
  idempotent: a true semilattice. Convergence is trivial.
- **The geometry is a projection**: the consensus value is a *deterministic
  pure function* of the set, computed on demand. Equal sets ⇒ equal values,
  by construction.

This is the Anima §8.C pattern applied to distributed state: **the
observation set is the raw canonical form; the geometric consensus is a
replayable deterministic projection.** The original design's error was making
the merge itself geometric. The salvage keeps the geometry where it is
mathematically honest — in the render.

## 3. What dies, survives, is born

**Dies (deleted with an epitaph note, not fixed):**
- `GeometricCRDT::merge` replay architecture (dead code under the dedup guard).
- `geometric_mean` (arithmetic mean of exponentials; exp/log are not inverses
  on multivectors — no closed form exists to converge to).
- `GA3Lattice` magnitude-dominance join (hull-violating; not a semilattice).
- `geometric_product` as a *cross-replica* operation. Per-participant
  sequences still apply transforms in order; reconciliation across replicas
  is union + projection, so no total order is ever required.

**Survives:**
- `ComponentLattice` — componentwise max/min, a genuinely correct semilattice.
  It becomes the floor for per-grade state (counters, vectors) that has no
  business on a manifold.
- `vector_clock` (with its sort transitivity caveat documented).
- cliffy-core FRP reactivity — untouched by this failure.

**Is born (v0.4.0 "Typed Algebra", resurrected from BACKLOG):**
- `ObservationSet` CRDT (G-Set semantics, attributed observations).
- Typed payloads: `Rotor<3,0,0>` (amari) for orientation-like state; scalars
  and vectors via the componentwise floor.
- **Rotor consensus projection**: unit rotors in Cl⁺(3,0,0) are quaternions;
  the chordal L₂ mean has a closed form — dominant eigenvector of
  `M = Σ wᵢ qᵢ qᵢᵀ` (Markley et al. 2007). CPU-cheap; the eigen-sum is a
  fold over the set (maintainable as a derived cache, never canonical).
  Document the double-cover sign sync as a deterministic projection step
  (canonicalize each observation's sign against a set-determined reference).
- **Weighted means by provenance**: weights must be a deterministic function
  of set-determined grant metadata (never local trust tables, or renders
  diverge). This is the seam where Schubert provenance tiers enter —
  capability-weighted consensus, the Anima-specific novelty.

## 4. Road (three phases)

### Phase 0 — Hygiene and honesty (now; operator decisions embedded)

1. Push or drop the two orphaned commits so `origin/develop` reflects
   reality. (Note: any branch cut from local HEAD carries them; merging this
   plan's PR implicitly publishes them to develop — "push" is recommended,
   they are real history.)
2. npm deprecation notice for `@cliffy-ga/core` (already intended via the
   Leptos/website effort): reactivity sound, geometric merge broken, successor
   path documented here.
3. Feature-gate or `#[deprecated]`-mark `cliffy-protocols::crdt` and
   `GA3Lattice::join` with the failure mode in the doc comment.
4. Record the `0.3.1` ↔ commit mapping durably (no tag exists today).

### Phase 1 — The sound floor (cliffy-protocols v0.4)

1. `ObservationSet` with attributed observations; merge = union;
   participant-scoped op IDs (`(participant_id, seq)` — the §8.B canonical
   identity already provides the vocabulary).
2. Rotor consensus projection (Markley eigen-mean), componentwise floor for
   non-manifold state.
3. Tests with **value oracles**: the regression probes from the rabbit hole
   become permanent tests; property tests assert set-laws + deterministic
   projection; the old convergence test gains the oracle it always lacked
   (`merge(a,b)` must equal a *specified* value, not merely itself).
4. Delete the dead machinery; BACKLOG v0.4.0 items checked off or re-scoped.

DoD: the February question is answered by a test suite, not a report.

### Phase 2 — The Anima road (Knopper orbit)

1. **U2 alignment**: observation payloads adopt the Knopper encoding
   contract (semantically-true encodings, defined blade assignments) —
   Cliffy becomes a named consumer of that contract, alongside Knopper
   machines and the web tier.
2. **Schubert-gated observations**: each observation carries a grant ref;
   merge admits per policy — a *capability-gated CRDT*.
3. **Borsalino for heavy merge-math**: large observation sets and
   higher-grade payloads (the Grassmannian tier Schubert arbitration needs)
   offload to verified GPU GA kernels; results are consumed as projections.
   Directive #3 of the Knopper positioning, verbatim.
4. Consumers: the Knopper collaboration lane (roadmaps 05/06), Miriami's
   sync story, and the website's collaborative-surface far edge — all
   unblocked on Phase 1, enhanced by Phase 2.

## 5. Non-goals (explicit)

- No attempt to make binary geometric means associative (§2 — it is not
  possible; the G-Set form is the design, not a workaround).
- No geometric consensus for state with no manifold structure — counters
  and maps take the boring floor (the parable of `ComponentLattice`).
- No cliffy-gpu resurrection inside this effort (dangling
  `amari-migration` branch stays archived; Borsalino is the offload path).

## 6. The line for the epitaph

> The original design tried to make the merge geometric. The salvage makes
> the *render* geometric and the merge boring — which is what the algebra
> was saying all along.
