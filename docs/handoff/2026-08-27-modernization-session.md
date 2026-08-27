# Handoff: Cliffy Modernization — CRDT Salvage + Anima Web-Projection Road

**Date:** 2026-08-27
**From:** the Dominic-session that verified the CRDT failure and drafted the salvage
(2026-08-25 → 08-27 conversation arc).
**To:** the Cliffy modernization session.
**You have Ijima memory access** (fleet-connected): `memory_search` under project
`cliffy` returns the probe-verification and salvage-plan memories. This doc is
the durable map regardless.

---

## 1. Why this session exists (the three-day arc)

1. **2026-08-25** — the daily rabbit-hole report
   (`IA-documents/RESEARCH_REPORTS/RABBIT_HOLE_2026-08-25_Cliffy.md`) claimed the
   geometric CRDT does not work. The operator questioned the conclusion; the
   Dominic session **rebuilt and re-ran the probes independently** at develop
   `282fbbd` — all claims reproduced verbatim. The report is trustworthy.
2. **2026-08-26** — the operator decided: own the math publicly, salvage what's
   sound, and give Cliffy the road to the Knopper "typed-rotor / slerp /
   Fréchet-mean, Borsalino for heavy merge-math" path. The salvage plan was
   drafted and PRed (**this repo, PR #301**).
3. **In parallel** — Cliffy was designated **Anima's web-facing UI layer**
   (doctrine §8.F, IA-documents PR #5, merged; refinement PR #6 open), with the
   IA website re-cut as its first customer.

This session's mandate: **execute the salvage + modernize Cliffy toward the
web-projection role.**

## 2. The verified failure (do not re-litigate; do read the evidence)

Probes re-run 2026-08-25, outputs verbatim:

```
a.state before merge = 15
merged.state = 0                     ← merge annihilation
join(+1, -1) = 1.543080634815244     ← cosh(1); outside the hull of its args
node1 op id = 0, node2 op id = 0     ← len()-minted IDs collide
```

Root causes, all confirmed in source:

- `GeometricCRDT::merge` assigns `result.operations = merged_ops` *before* the
  replay loop; `apply_operation`'s dedup guard (`contains_key → return`) then
  short-circuits every replay. **The replay loop is dead code; every merge
  returns `GA3::zero()`.**
- `geometric_mean` is an arithmetic mean of exponentials (`map(exp)`, fold-add,
  ÷n) with the confession in a comment. exp/log are not inverses on
  multivectors.
- `create_operation` mints IDs from `self.operations.len()` — two empty nodes
  both mint id 0; merge's HashMap union silently drops the collision.
- The flagship `test_geometric_crdt_convergence` is **vacuous**: both replicas
  merge to zero, both agree, agreement is the only oracle. 240 tests green
  certifying annihilation.

**Why it was never surfaced:** the 2026-02-25 rabbit hole asked the question and
nothing downstream exercised the merge path (ShaperOS dep commented out;
Knopper collaboration lane unbuilt). Process fix adopted: rabbit-hole open
questions mint backlog items. **Lesson for this session: green is not a truth
claim — tests need value oracles, not agreement oracles.**

**Also verified:** origin/develop was two commits behind local HEAD `282fbbd`
(last human commit never pushed). PR #301 carries them; merging it resolves
push-or-drop as *push*.

## 3. The salvage plan (your execution target)

**Read first:** `docs/plans/2026-08-26-geometric-crdt-salvage.md` (this repo,
on the salvage-plan branch / PR #301).

Verdict: **salvageable as a redesign, not a patch.** The load-bearing math: a
binary geometric/Fréchet mean is commutative and idempotent but **not
associative** — no mean-based binary merge is a join-semilattice, so the
original architecture cannot be fixed in place. The sound architecture:

- **`ObservationSet` G-Set CRDT**: grow-only set of attributed observations
  `(participant_id, seq, clock, payload, grant_ref)`; merge = set union (true
  semilattice, trivially convergent). This is Anima's §8.C raw-canonical
  pattern applied to distributed state.
- **Geometry as deterministic projection**: consensus = pure function of the
  set, computed on demand (equal sets ⇒ equal values). Unit rotors in
  Cl⁺(3,0,0) are quaternions → chordal L₂ mean has a closed form: dominant
  eigenvector of `M = Σ wᵢqᵢqᵢᵀ` (Markley 2007). CPU-cheap; eigen-sum is a
  fold (derivable cache, never canonical). Double-cover sign sync is a
  deterministic projection step. Provenance weights must be set-determined
  (never local trust tables) — the **capability-gated CRDT**, the
  Anima-specific novelty.
- **Dies**: geometric_product cross-replica ops, magnitude-dominance join,
  exp-mean. **Survives**: `ComponentLattice` (the boring, correct floor),
  `vector_clock`, the FRP core (untouched). **Boring state takes the boring
  floor** — counters/maps never go on a manifold.

Phases (from the plan): **0** hygiene + honesty (gate/deprecate broken
modules; npm deprecation for `@cliffy-ga/core` — already intended via the
website effort; record the missing v0.3.1↔commit tag mapping) → **1** sound
floor (ObservationSet + rotor projection + **value-oracle tests**: the rabbit
hole probes become permanent tests; the convergence test gains a value
oracle) → **2** the Anima road (U2 encoding-contract consumer, Schubert-gated
observations, Borsalino offload for large sets/higher grades).

## 4. The Anima web-projection context (the modernization direction)

**Doctrine:** `IA-documents/Anima/ANIMA_ECOSYSTEM_DOCTRINE.md` — §8.F (Cliffy's
entry), §3 rendering-tiers row, §4 public-web-projection boundary. State
snapshot: `IA-documents/Anima/STATE_OF_ANIMA_2026-08-22.md`. §8.F refinement
(patterns-first) is in **IA-documents PR #6 (open)** — read it on the branch if
unmerged.

Key facts, as decided:

- **Patterns-first, crate-later**: no `cliffy-leptos` crate yet, and none is
  needed to start. The web tier = **Leptos + Mingot** (the operator's Leptos UI
  library — `../Mingot`, 58 PRs, precision inputs + node-graph; live capital,
  keep it) with Cliffy reactivity via documented patterns. `cliffy-leptos` is
  the **extraction target** when the bridge stabilizes *and* a second consumer
  appears (Wallace's web cockpit, Miriami's web lowering).
- **Two lanes**: projection/content (the IA website *re-cut* — Leptos + Mingot
  + Cliffy reactivity; SSR/SEO real) and app/collaborative (pure Cliffy-WASM).
- **Cliffy has no SSR story today** (one tsukoshi roadmap line). The
  projection lane does not bet the front door on building one.
- **Website mechanism**: content = Ijima blocks with provenance; rendering =
  deterministic projections (the §8.C pattern — same as the CRDT salvage's
  shape); egress = `public:*` scope-gated (Ijima boundary machinery exists);
  one real Lonis read-tool from day one. The collaborative-surface far edge is
  **gated on salvage Phase 1**.

Related ecosystem anchors you may need: Knopper identity restoration
(`../Knopper`, `docs/plans/2026-08-21-identity-restoration.md` on branch
`perf/o1-diff-and-rayon` — its U2 encoding contract is what salvage Phase 2
aligns with; maintainer directives 1–4 therein govern geometry handling);
embedding contract `IA-documents/CONTRACTS/wallace-knopper-embedding.md` v0.4.

## 5. Repo + ecosystem state (as of handoff)

- Local HEAD `282fbbd` = develop + 2 orphaned commits; **PR #301 (salvage
  plan) carries them — merge it first**, then this PR. Branch map:
  `docs/crdt-salvage-plan` → #301; this handoff's branch → this PR.
- npm: `@cliffy-ga/core@0.3.1` (last publish 2026-03-03; ~103 downloads/mo; no
  git tag exists for 0.3.1 — record the mapping durably during Phase 0).
- The `amari-migration` branch (2025-12-24, amari 0.13.1) is a fossil —
  archived, not a base. BACKLOG v0.4.0 "Typed Algebra" items are the
  resurrected plan.
- Repo conventions: gitflow (develop integration, feature branches, PRs LEFT
  OPEN for the operator to merge — **agents never self-merge**); IA coding
  standards (TDD, clippy `-D warnings` both configs, Apache-2.0); pre-commit
  gate runs fmt + clippy + full tests (~2 min; the loadtest crate accounts for
  ~60s of it).

## 6. Recommended order of work

1. Confirm both PRs merged (or flag to operator); checkout develop; pull.
2. **Phase 0** hygiene (gate the broken modules, deprecation notes, tag
   mapping, npm deprecation text for operator approval).
3. **Phase 1** sound floor, TDD: value-oracle tests first (port the probes),
   then `ObservationSet`, then the rotor consensus projection.
4. Web-patterns lane in parallel if capacity: document the
   Leptos↔`Behavior` bridging patterns against the website re-cut's needs.
5. Phase 2 items are cross-repo coordinated (Knopper U2 contract, Schubert
   gates, Borsalino) — propose, don't unilaterally depend, until those land.

*The line for the epitaph, from the salvage plan: the original design tried to
make the merge geometric; the salvage makes the render geometric and the merge
boring — which is what the algebra was saying all along.*
