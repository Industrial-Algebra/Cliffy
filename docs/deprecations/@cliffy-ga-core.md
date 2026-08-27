# Deprecation Draft: `@cliffy-ga/core`

**Status: DRAFT — awaiting operator approval. The agent does not execute
this; the operator runs the command below verbatim if satisfied.**

**Proposed on:** 2026-08-27 (modernization session, Phase 0 item 2 of the
[salvage plan](../plans/2026-08-26-geometric-crdt-salvage.md))

## Context

`@cliffy-ga/core` ≤ 0.3.1 (last publish 2026-03-03, ~103 downloads/mo)
ships the WASM build whose geometric CRDT merge is **verified broken**
(2026-08-25 probes, independently re-run 2026-08-25 on develop `282fbbd`):

- `merge` annihilates to zero (the replay loop is dead code under the dedup
  guard)
- op IDs minted from `operations.len()` collide across replicas
- `join(+1, -1)` returns `cosh(1)`, outside the hull of its arguments
- the flagship convergence test was vacuous (agreement oracle, not a value
  oracle)

The reactivity layer (`behavior`, `event`, Algebraic TSX) is sound. The
successor path is the `ObservationSet` CRDT + deterministic geometric
projection (v0.4.0 cycle).

## Proposed command

```bash
npm deprecate @cliffy-ga/core@'*' \
  "Reactivity is sound, but the geometric CRDT merge is broken (merges annihilate to zero) — do not use the CRDT/merge/lattice APIs. Successor: ObservationSet CRDT + deterministic projection, planned for 0.4.0. See https://github.com/Industrial-Algebra/Cliffy/blob/develop/docs/plans/2026-08-26-geometric-crdt-salvage.md"
```

## Notes for the operator

- Deprecating `'*'` covers all published versions (0.1.0–0.3.1). If you
  prefer to warn only on affected versions, `@'>=0.1.0'` is equivalent
  here since the CRDT shipped from the first release.
- The successor package naming/scope (`@industrialalgebra/…` vs
  `@cliffy-ga/…`) is an open ecosystem decision (see the
  [handoff](../handoff/2026-08-27-modernization-session.md) §5) — deprecating
  now does not foreclose either choice.
- `npm deprecate` is reversible (`npm deprecate <pkg>@<ver> ""` clears the
  message).
