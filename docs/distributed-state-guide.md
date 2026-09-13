# Distributed State

> **Status**: `cliffy-protocols` ships the Phase 1 distributed-state floor:
> `ObservationSet` (a grow-only set CRDT) plus deterministic geometric
> projections. WASM bindings (`ObservationSet`) are available in
> `@industrialalgebra/cliffy-core` as of 0.4.0.
>
> 0.4.0 **removed** the previous `GeometricCRDT`, `GA3Lattice`, and
> `consensus` modules: their merge was not a semilattice join (it could
> annihilate data and depended on op-id collision behavior). See the
> [0.4.0 changelog](../CHANGELOG.md) for the full rationale.

## The Core Idea

**The merge is boring; the render is geometric.**

- The CRDT is a **set of observations**. Merge is set union — associative,
  commutative, idempotent. Convergence is trivial by construction; no
  geometric operation can break it.
- The geometry happens **after** merge, as a *deterministic projection* of
  the observation set: scalar means, vector means, and rotor consensus.
  Because every node projects the same set with the same function, all
  nodes compute the same value.

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Node A    │     │   Node B    │     │   Node C    │
│ observes 5  │     │ observes 3  │     │ observes 7  │
└──────┬──────┘     └──────┬──────┘     └──────┬──────┘
       │                   │                   │
       └───────────────────┼───────────────────┘
                           │
                    merge = set UNION
                  {5, 3, 7} on every node
                           │
                  deterministic projection
                     (e.g., scalar mean)
                           │
                           ▼
                         5.0
```

## Rust API (cliffy-protocols)

### ObservationSet

```rust
use cliffy_protocols::{Observation, ObservationSet};
use uuid::Uuid;

let node_a = Uuid::new_v4();
let mut local = ObservationSet::new();

// Observations are keyed by (participant, sequence) — ids cannot
// collide across nodes.
local.insert(Observation::new_scalar(node_a, 1, 5.0));
local.insert(Observation::new_vector(node_a, 2, vector_observation));
local.insert(Observation::new_rotor(node_a, 3, rotor_observation));

// Merge is union. Order doesn't matter; duplicates are absorbed.
local.merge(&remote);
```

### Deterministic projections

Projections are free functions over the set:

```rust
use cliffy_protocols::{scalar_mean, vector_mean, rotor_consensus};

// Scalar mean — None on an empty set
let mean: Option<f64> = scalar_mean(&local);

// Vector mean (componentwise)
let v: Option<VectorObservation> = vector_mean(&local);

// Rotor consensus — Markley eigen-mean over the 4x4 quaternion
// accumulator, computed with an in-house Jacobi eigensolve
// (deterministic, no external LAPACK dependency).
let r: Option<RotorObservation> = rotor_consensus(&local);
```

### The ComponentLattice floor

`cliffy_protocols::ComponentLattice` remains as the sound lattice floor
(componentwise min/max join/meet on multivector coefficients). It is
conservative and always converges — unlike the removed magnitude-based
`GA3Lattice`.

## TypeScript / WASM

The same surface is exposed over WASM in `@industrialalgebra/cliffy-core`:

```typescript
import init, { ObservationSet } from '@industrialalgebra/cliffy-core';

await init();

const set = new ObservationSet();
set.observeScalar('node-a', 1, 5.0);
set.observeScalar('node-b', 1, 10.0);
set.merge(remoteSet);

set.scalarMean();      // 7.5
set.vectorMean();      // componentwise mean or undefined
set.rotorConsensus();  // Markley eigen-mean rotor or undefined
```

Try it live in the [crdt-playground example](../examples/crdt-playground),
which exposes each projection as an interactive probe panel.

## Design notes

- **Why a set?** A CRDT must be a semilattice (associative, commutative,
  idempotent join) to guarantee convergence. Set union is the canonical
  semilattice. The previous design tried to make the *geometry* the merge
  (geometric mean, magnitude dominance); none of those operations are
  semilattice joins over multivectors, and the system could destroy data.
- **Why project deterministically?** Any pure function of a converged set
  converges. Consensus quality becomes a math problem (which projection?)
  instead of a distributed-systems problem (does the merge converge?).
- **What about consensus protocols?** Phase 2 rebuilds richer consensus
  (e.g., Schubert-gated observations with GrantRef semantics, and the
  Knopper U2 encoding contract) on top of this floor. See
  [ROADMAP.md](../ROADMAP.md).

## Further reading

- [ROADMAP.md](../ROADMAP.md) — where the sync layer (WebRTC, persistence) lands
- [CHANGELOG.md](../CHANGELOG.md) — the 0.4.0 cutover, with failure modes of
  the removed machinery
- [geometric-algebra-primer.md](geometric-algebra-primer.md) — the GA3
  background behind rotors and projections
