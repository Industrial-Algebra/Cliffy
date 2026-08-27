# amari-core Gap Report — from Cliffy, Amari's first consumer

**Date:** 2026-08-27
**Author:** the Cliffy Phase 1 session (salvage implementation)
**Versions verified:** amari-core 0.23.0 (crates.io registry; Cliffy's pin)
and 0.24.1 (Amari workspace, `develop`-adjacent source + live probes)
**Context:** Phase 1 of the geometric-CRDT salvage
([plan](../plans/2026-08-26-geometric-crdt-salvage.md)) required a
serialize-and-reconstruct rotor type, deterministic across replicas — the
first hard consumer-grade stress test of amari's rotor surface.

---

## 0. The correction this report almost got wrong (read this first)

This session initially concluded — and briefly reported — that
`Rotor::apply_to_vector` was semantically broken, citing a y-axis rotor
"annihilating" the x-unit vector (norm 1 → 0, a mathematical impossibility
for a rotation).

**That conclusion was wrong, and the error is instructive.** The probe read
`Vector::as_slice()` positions `[0..3]` as vector components. `as_slice()`
returns the **full multivector coefficient layout** —
`[scalar, e1, e2, e12, e3, e13, e23, e123]` — so a vector's components live
at indices `1, 2, 4`. The "annihilated" y-component was sitting, intact, at
index 4. Re-probed correctly, amari's rotor math is **textbook-correct**
across everything we tested (section 1).

The gap report therefore is not about broken math. It is about an API
surface whose shape caused an experienced consumer — actively looking for
upstream bugs, primed by Cliffy's own CRDT failure — to *confidently
misdiagnose* the library within an hour. That is a real API-quality gap,
and it is the kind `amari-discovery`'s capability model cannot currently
see.

## 1. What was verified sound (0.23.0 coefficients + 0.24.1 live probes)

| Property | Result |
|---|---|
| `from_axis_angle` right-handedness, all three axes (+90°: x→fixed, y: x→−z, z: x→y) | ✅ exact (1e-12) |
| Norm preservation under `apply_to_vector`, arbitrary axes & angles | ✅ (1e-10) |
| `from_bivector` plane rotations: e12: e1→e2, e23: e2→e3, e31: e1→−e3 | ✅ exact |
| `from_axis_angle(z)` ≡ `from_bivector(e12)` (amari's own test) | ✅ |
| Coefficient layout stable 0.23.0 ↔ 0.24.1 (x: e23=−s, y: e13=+s, z: e12=−s at +90°) | ✅ |

Cliffy now pins the full amari↔Hamilton mapping by test
(`cliffy-protocols/src/projection.rs::mapping_pins_amari_conventions`):
`w = c[0]`, `x = −c[6]` (e23), `y = +c[5]` (e13), `z = −c[3]` (e12).

## 2. Gaps, ranked

### Gap 1 — `Vector::as_slice()` is a trapdoor (HIGH, ergonomics/docs)

A method named on a three-component geometric type returning an
eight-coefficient layout in which the natural read (`[0..3]`) yields
`(scalar, e1, e2)` — silently, for every pure vector, since the real
components are at `1, 2, 4`. There are no `x() / y() / z() / components()`
accessors to steer consumers right.

**Impact:** false upstream-bug conviction in under an hour (section 0);
any consumer composing `Vector` with slice-based APIs (`bytemuck`,
`Float32Array`, hand-rolled serialization) will corrupt data silently.

**Recommendations:**
- Add `pub fn components(&self) -> [f64; 3]` (and/or `x()`, `y()`, `z()`)
  to `Vector<3,0,0>`;
- Document the layout **on `as_slice` itself**, or rename to
  `as_multivector_slice()` (breaking; a doc-fix is non-breaking and 90% of
  the value);
- An all-axes value-oracle test (see §4) would have caught nothing here —
  this gap is about the *access path*, which points again at naming.

### Gap 2 — no serde impls on the geometric types (HIGH for distributed consumers)

`Cargo.toml` declares `serialize = ["serde"]`, but `Rotor`, `Vector`,
`Bivector`, and `Multivector` carry no `Serialize`/`Deserialize` impls in
either version. The feature wires the dependency without producing a wire
format. For a distributed-state consumer (Cliffy's entire premise) this is
the difference between a blessed format and every consumer inventing one.

**Recommendation:** bless coefficient serde — `[f64; 8]` for multivectors,
`[f64; 3]` for vectors, `[f64; 4]` for rotors (Cliffy's `serde_ga3` and
Phase 1 `RotorObservation { w, x, y, z }` are working references; the
latter is deliberately named-field serde, which survives layout changes).

### Gap 3 — rotor construction asymmetry (MEDIUM)

`Rotor::as_slice()` reads coefficients; nothing reconstructs from them.
The only construction paths are semantic (`from_bivector`,
`from_axis_angle`, `from_vectors`). A serialize-reconstruct cycle must
round-trip through axis–angle — introducing a few ulps of drift (fine for
Cliffy: the drift is a deterministic function of the input, and the
determinism contract only requires that; but coefficient-exact
reconstruction is what CRDT and GPU-boundary consumers actually want).

**Recommendation:** `Rotor::try_from_even_components([f64; 4]) -> Option<Self>`
(validating unit-norm within tolerance) — the typed-algebra roadmap's
"phantom types for verification" style fits here.

### Gap 4 — value-oracle coverage shape (LOW, testing)

The rotor suite tests the e12 plane via `from_bivector` and z-axis
equivalence at the coefficient level; per-axis *behavioral* oracles
(rotate a known vector, assert the result vector) are absent. Amari's math
is right, so this is hardening, not repair — but Cliffy's whole salvage
lesson is that agreement-style oracles (coeff A ≡ coeff B) certify less
than value oracles (this input → that output). All-axes rotation tests now
exist on the consumer side (§1) and are offered upstream below.

### Gap 5 — Bivector component-order docs (LOW)

`Bivector::from_components(xy, xz, yz)` ordering — and the
`e31 = −e13` identity needed for y-axis duals — is discoverable only by
reading source or probing. One doc paragraph on the type would close it.

## 3. What this means for amari-discovery

The catalog advertises `amari:amari-core:rotor:rotation` as
**`Stability: Stable`, `cost: Low`** — and the *mathematics* is exactly
that. But the capability model has no representation for Gaps 1–3: the
ergonomic trap (1), the missing wire format (2), and the constructor
asymmetry (3) are all invisible to a catalog built on structure and
curated semantics. This is a concrete instance of the 2026-08-07 rabbit
hole's §6 question — "does the public narrative, feature graph, catalog
surface, and human import path agree?" — answered from the consumer side:
the catalog can be structurally true and still steer a consumer into a
trapdoor. Capability descriptors with a `verified-bytes` or
`consumer-probe` dimension (the discovery probes already have the
machinery) would close this.

## 4. Offered back upstream

Cliffy can contribute, as tests or probe descriptors:
- the all-axes right-handedness + norm-preservation oracle suite (§1 —
  three tests, ~60 lines, zero dependencies beyond amari itself);
- the pinned amari↔Hamilton coefficient table as a documented constant
  (it is *the* answer to "how do I get a quaternion out of this rotor");
- this report's Gap 2/3 implementations (coefficient serde +
  `try_from_even_components`) as PRs if wanted.

## 5. The meta-lesson

Cliffy's salvage was born from tests that certified agreement instead of
value. This report was nearly born from a probe that certified a
misreading instead of a behavior. Same disease, opposite direction: in
both cases the fix was the same — pin the *specified value*, against the
*documented layout*, for the *actual question being asked*. Consumers owe
libraries value oracles; libraries owe consumers layouts that can't be
misread by accident.
