# Rust Toolkit Test-Drive — Nightly + Teeth + DevShell

> **REQUIRED SUB-SKILL:** Execute task-by-task (this session, per operator).

**Goal:** Make Cliffy the first IA repo to test-drive the 2026 Rust toolkit
(namtao.com/rust-toolkit-2026): nightly-first toolchain with dual-lane CI,
the panic-denying clippy configuration, cargo-nextest, and a NixOS-native
devShell — minus everything redundant.

**Source:** No Boilerplate's "My 2026 Rust Toolkit". Operator directives
(2026-08-27): integrate nearly everything; **skip devenv** (all machines
converting to NixOS; hercules-corona already is); skip redundant items.
**Toolchain decision:** nightly dev + dual-lane CI (stable test lane keeps
the library crates.io-compatible).

## What's in vs. out

| Toolkit item | Verdict | Rationale |
|---|---|---|
| Nightly toolchain | **IN** (PR A) | Matches ia-coding-standards example + amari/minuet/Kagome CI; resolves the ecosystem inconsistency #307 introduced (stable, following Ijima/Lonis) |
| Dual-lane CI (nightly lint/test + stable test) | **IN** (PR A) | Library bound for crates.io; stable lane catches nightly-only feature usage |
| Clippy teeth (`pedantic`/`nursery` deny + panic-denials) | **IN** (PR B) | Direct alignment with IA "Result, not panic" standard; curated allow-list for GA numeric reality |
| cargo-nextest | **IN** (PR A CI, PR C shell) | Faster, retry-capable runner; doctests stay on `cargo test --doc` (nextest doesn't run them) |
| bacon, cargo-seek, watchexec, rust-analyzer | **IN** (PR C devshell) | Dev conveniences |
| flake.nix devShell | **IN** (PR C) | NixOS-native devenv replacement |
| Criterion expansion (core propagation benches) | **IN** (PR C) | Only cliffy-gpu has benches today; v0.5.0 backlog lists hot-path profiling |
| itertools | **IN** (deferred adoption) | Workspace dep when Phase 1 lands real use sites |
| serde, rayon, Criterion, thiserror | already present | — |
| devenv.sh | **OUT** | Operator directive — NixOS native |
| cargo-generate | **OUT** | `create-cliffy` already scaffolds (TS templates) |
| eyre/color-eyre | **OUT** | App-level; `thiserror` is the IA library standard |
| neovim, jiff, clap, command-run, utoipa, reqwest, sqlx | **OUT** | Editor preference / no domain surface |
| leptos, dioxus | **OUT** (for now) | Leptos = the Phase-2 web lane per the handoff (patterns-first, no crate yet) |

## Evidence (2026-08-27)

- Verified clean on nightly 1.99.0 (2026-08-14): fmt, clippy `-D warnings`
  all-features, 223 tests, doctests.
- **Regression found during execution:** floating nightly 1.100.0
  (2026-08-25) hits a trait-solver recursion overflow evaluating wgpu's
  `Send` chains in check/clippy builds — wgpu 29 *and* 30 are affected;
  stable 1.98 and full builds/tests are unaffected. **Decision:** date-pin
  `nightly-2026-08-14` in `rust-toolchain.toml` (comment documents the
  regression + re-float condition). CI lint/wasm/scale/publish jobs install
  from the file (`dtolnay/rust-toolchain@master`, single source of truth);
  the nightly *test* lane stays floating deliberately, so future test-level
  regressions surface in CI before reaching devs.
- wgpu 29 → 30 folded into PR A: required to confirm the overflow wasn't a
  wgpu-29-only issue (it wasn't), clears the `cargo doc` future-incompat
  warning, and unblocks stalled dependabot #275 (two small API fixes:
  `apply_limit_buckets: false` preserving prior limits behavior; `get_mapped_range`
  now returns `Result`).

Risk accepted: floating nightly test lane can occasionally break CI;
pinned-date lint lane keeps the repo buildable; weekly-ish bump chore re-dates
the pin (fallback documented here).

---

## PR A — `chore/nightly-toolchain` (stacked on #308)

1. This plan doc rides in this PR (`docs/plans/2026-08-27-rust-toolkit-testdrive.md`).
2. `rust-toolchain.toml`: `channel = "nightly"`; components add `rust-src`
   (rust-analyzer std introspection on nightly).
3. `ci.yml`:
   - Rust Checks: `dtolnay/rust-toolchain@master` (installs exactly what
     `rust-toolchain.toml` pins) + wasm32 target (fmt + clippy stay as-is).
   - Rust Tests → matrix `[stable, nightly]`: each lane installs its
     toolchain explicitly and **overrides the file via `RUSTUP_TOOLCHAIN`**
     (rustup env beats rust-toolchain.toml — this is what makes the stable
     lane work against a nightly-pinning file, and keeps the nightly lane
     floating ahead of the pin). Install nextest via
     `taiki-e/install-action@v2`; run `cargo nextest run --workspace` +
     `cargo test --doc --workspace`.
   - WASM Build / WASM Unit Tests / Scale Tests / any cargo-invoking job:
     switch their toolchain install to file-driven `@master` (a
     nightly-pinning file breaks jobs that install only stable).
4. `benchmarks.yml`, `publish-crates.yml`, `security.yml`, `release.yml`:
   same `@master` file-driven install everywhere.
5. wgpu 29 → 30 (see Evidence) + the two API fixes in `cliffy-gpu/src/lib.rs`.
5. CONTRIBUTING.md: verification matrix now nightly-default; document
   `RUSTUP_TOOLCHAIN=stable` for stable-checking; nextest usage.
6. CHANGELOG Unreleased entry.

Verify: local `cargo +nightly` matrix (above) + `RUSTUP_TOOLCHAIN=stable
cargo clippy/test` still green. CI green on both lanes.

## PR B — `chore/clippy-teeth` (stacked on A)

1. Root `Cargo.toml` `[workspace.lints.clippy]`:
   `pedantic` deny, `nursery` deny, and the panic set: `unwrap_used`,
   `expect_used`, `panic`, `panic_in_result_fn`, `todo`, `unimplemented`,
   `unreachable`, `indexing_slicing`, `string_slice`,
   `arithmetic_side_effects`, `unchecked_time_subtraction`, `exit`.
   Curated, commented allows for numeric-library reality (`as_conversions`,
   `cast_*` family, and any pedantic lint whose fix-churn exceeds ~100
   mechanical edits — document each allow with a reason line).
2. `clippy.toml`: `allow-unwrap-in-tests`, `allow-expect-in-tests`,
   `allow-panic-in-tests`, `allow-indexing-slicing-in-tests` = true.
3. All six crate manifests: `[lints] workspace = true`.
4. RED first: apply lints, read the failure census, then fix (unwrap →
   `Result` in lib code — the 8 known in cliffy-protocols; indexing →
   `get`; benches' unwraps fixed or allowed with comment).
5. Doctests: the clippy.toml test allowances cover them; verify.

Verify: `cargo +nightly clippy --workspace --all-targets --all-features -- -D warnings` green; stable lane too; 223 tests + 29 doctests green; 0 doc warnings.

## PR C — `feat/devshell-benches` (stacked on B)

1. `flake.nix`: devShell with rustup, bacon, cargo-nextest, cargo-seek,
   watchexec, rust-analyzer (nixpkgs); `enterShell` version echo;
   `.gitignore` += `.direnv/`. No devenv — operator directive.
2. `cliffy-core/benches/propagation.rs` (criterion 0.8 dev-dep, matching
   gpu's setup): behavior update→N-subscriber fan-out, map-chain depth,
   event fold throughput. Makes `benchmarks.yml` exercise core, not just gpu.
3. CONTRIBUTING.md: devShell entry (`nix develop`), bacon/nextest/seek/
   watchexec recipes.
4. CHANGELOG entry.
5. itertools: only if a genuine adoption site emerged; otherwise deferred
   to Phase 1 (recorded here, not silently dropped).

## Out of scope / follow-ups

- Wgpu 30 (PR #275 open, dependabot) — clears the future-incompat warning.
- Pinned-nightly fallback if floating nightly breaks CI twice in a month.
- eyre adoption if/when a binary target exists (loadtest CLI, tooling).
