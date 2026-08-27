# Changelog

All notable changes to Cliffy are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
[SemVer](https://semver.org/).

## [Unreleased]

### Changed

- **Toolchain: nightly-first** (operator decision 2026-08-27, test-driving
  the 2026 Rust toolkit). `rust-toolchain.toml` pins
  `nightly-2026-08-14` +`rust-src` — date-pinned because floating nightly
  1.100.0 (2026-08-25) hits a rustc trait-solver recursion overflow on
  wgpu's `Send` chains in clippy builds (stable unaffected; re-float when
  it clears). CI lints on the pinned nightly via file-driven installs
  (`dtolnay/rust-toolchain@master`) and tests on **stable + floating
  nightly** lanes via `RUSTUP_TOOLCHAIN` override — the library stays
  crates.io-compatible and regressions surface in CI first. Aligns with
  amari/minuet/Kagome CI convention.
- **wgpu 29 → 30** — unblocks dependabot #275; two API fixes
  (`apply_limit_buckets: false`, `get_mapped_range` → `Result`); clears the
  doc future-incompat warning.
- **CI: cargo-nextest** for unit/integration tests (faster, retries);
  doctests stay on `cargo test --doc` (nextest doesn't run them).
- All cargo-invoking workflows install the toolchain from
  `rust-toolchain.toml` (`dtolnay/rust-toolchain@master`).
- **License: MIT → Apache-2.0** (operator decision 2026-08-27, per IA
  ecosystem standard). LICENSE file added; all crate manifests and the 39
  source files carry Apache-2.0 SPDX identifiers. Published npm artifacts
  ≤ 0.3.1 remain MIT as shipped.
- Repository URLs updated `justinelliottcobb/Cliffy` →
  `Industrial-Algebra/Cliffy`.
- `rust-toolchain.toml` pins `stable` + rustfmt/clippy.
- CI: clippy now runs `--all-features`; crate publishing is tag-driven
  (`v*` tags), matching the IA release contract.

### Added

- CONTRIBUTING.md with the CLA requirement and the verification matrix.
- This CHANGELOG, backfilled from git history.

### Deprecated

- `cliffy_protocols::GeometricCRDT` and `geometric_mean` (crdt module):
  the geometric merge is mathematically unsound — merges annihilate to
  `GA3::zero()` (dead replay loop), op IDs collide across replicas, and the
  exp-mean is not a mean on the manifold. Verified 2026-08-25; see
  [the salvage plan](docs/plans/2026-08-26-geometric-crdt-salvage.md).
  Superseded by `ObservationSet` in the v0.4.0 cycle. The wasm
  `GeometricCRDT` wrapper carries the same deprecation.
- `cliffy_protocols::GA3Lattice`: the magnitude-dominance join violates the
  join-semilattice hull property (`join(+1, -1) = cosh(1)`). Use
  `ComponentLattice` (componentwise max/min) — the boring, correct floor.
  `VectorClock` and the FRP core are unaffected.
- npm: deprecation draft for `@cliffy-ga/core` prepared at
  `docs/deprecations/@cliffy-ga-core.md` — **operator approval required
  before executing**.

## [0.4.0] — Planned

The salvage + typed-algebra cycle: `ObservationSet` CRDT (G-Set semantics),
rotor consensus projection (Markley eigen-mean), value-oracle tests ported
from the 2026-08-25 probes, Amari 0.23 typed primitives. See the
[salvage plan](docs/plans/2026-08-26-geometric-crdt-salvage.md) and the
[modernization handoff](docs/handoff/2026-08-27-modernization-session.md).

## [0.3.1] — 2026-03-03 (npm only)

Dependency updates; published to npm as `@cliffy-ga/core@0.3.1` (last
publish 2026-03-03).

> **Durable tag mapping** (Phase 0 record): **no git tag exists for 0.3.1.**
> The version fields were bumped in PR #171 (commit `4021016`, merged
> 2026-03-03); the npm tarball was published from that state. This entry is
> the canonical mapping — a retroactive tag was deliberately **not** created
> because publishing is now tag-driven and a `v*` tag on that commit would
> fire a crates.io publish that has never happened.

## [0.3.0] — 2026-03-02

Tagged `v0.3.0` (`af98067`). Amari 0.19.0 across the workspace (typed
rotors/vectors/bivectors available), enhanced `cliffy-protocols` CRDT and
consensus, Dependabot enabled, all packages bumped to v0.3.0.

## [0.2.0] — 2026-02-21

Tagged `v0.2.0` (`9853402`). P2P sync over WebRTC with peer discovery,
delta encoding for efficient state transfer, Vite plugin, scaffolding
templates, and the p2p-sync / document-editor / multiplayer-game examples.

## [0.1.2] — 2026-02-05

Tagged `v0.1.2` (`19af99a`). `@cliffy-ga/core` npm polish: GA-inspired
`wedge`/`Blade` combinators (replacing `combine3`/`combine4`),
`Behavior::project`/`select`, `.blend()` interpolation; examples suite
expansion.

## [0.1.0] — 2026-02-04

Tagged `v0.1.0` (`7cf93ca`). First public release: core FRP primitives
(Behavior/Event/combinators), WASM bindings with Algebraic TSX, distributed
state protocols (CRDT, sync, storage), geometric testing framework, GPU
acceleration, 13 example applications, PureScript bindings, `create-cliffy`
CLI.

---

### Release-notes footnote: the dropped 0.3.2

A `release/v0.3.2` maintenance branch was drafted (2026-03, carrying repo
URL updates and a version bump) but never merged or tagged. On 2026-08-27 it
was **intentionally dropped** (operator decision): its unique content was
folded into the modernization PRs and the next release targets **0.4.0**
(the CRDT-salvage cycle). The branch was deleted after the fold.
