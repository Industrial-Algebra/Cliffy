# Cliffy Modernization + Phase 0 Implementation Plan

> **REQUIRED SUB-SKILL:** Use the executing-plans skill (or subagent-driven development) to implement this plan task-by-task.

**Goal:** Bring Cliffy up to current IA code/gitflow standards and execute salvage Phase 0 (hygiene + honesty), leaving develop as the clean base for the v0.4.0 salvage cycle.

**Architecture:** Four sequential PRs to `develop` (left open for the operator to merge — agents never self-merge): (1) a `main → develop` backmerge that rejoins the diverged graphs, (2) licensing + provenance (Apache-2.0, per operator decision 2026-08-27), (3) tooling/docs hygiene (toolchain pin, CHANGELOG with the missing tag mapping, doc warnings, tag-driven publish), (4) Phase 0 deprecation gating of the broken CRDT machinery.

**Tech Stack:** Rust workspace (6 crates, amari-core 0.23.0), GitHub Actions, gh CLI.

**Context documents:**
- `docs/handoff/2026-08-27-modernization-session.md` — the mandate
- `docs/plans/2026-08-26-geometric-crdt-salvage.md` — the salvage plan (Phase 0 source)
- Skills: `ia-gitflow`, `ia-licensing`, `ia-coding-standards`, `verification-before-completion`

**Operator decisions already made (2026-08-27):**
- License: **Apache-2.0** (MIT → Apache-2.0; nothing yet on crates.io, so this is cheap now)
- `release/v0.3.2`: **drop it** — fold the repo-URL fixes into develop, aim the next release at v0.4.0

**Standing rules:**
- PRs target `develop`; the operator merges. PR bodies carry the bolded merge-commit warning where relevant.
- Verification matrix before claiming any PR ready: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace`, `cargo doc --workspace --no-deps --all-features` (zero warnings).
- After a rebase: `git push origin <branch> --force` (dual-push remote; `--force-with-lease` fails across mirrors).

---

## PR 1 — Backmerge: rejoin main ↔ develop

**Why first:** develop (+39) and main (+7) diverged both ways — the same dependabot PRs were squash-merged to main *and* merge-committed to develop. Everything else lands on a rejoined graph. Independent of PRs 2–4 (no file overlap), but should merge before the 0.4.0 cycle starts.

**Branch:** `chore/backmerge-main-to-develop` off `origin/develop`

### Task 1.1: Create the backmerge branch and merge main

```bash
git checkout develop && git pull
git checkout -b chore/backmerge-main-to-develop
git merge origin/main --no-ff -m "merge: backmerge main → develop (rejoin graphs after dual dependabot merges)"
```

Expected: conflicts in `examples/*/package.json`, `examples/package.json`, `tools/create-cliffy/package.json{,.lock}`, `vite-plugin-algebraic-tsx/package.json` — the content-duplicated dependabot bumps. **Resolution rule: develop's content wins** (develop is the superset; verify per file with `git diff` if unsure). `.github/workflows/project.yml` (main-only, the auto-project routing workflow) must arrive cleanly — confirm it exists after the merge: `ls .github/workflows/project.yml`.

### Task 1.2: Verify

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

Expected: all pass (223 tests). If Cargo.lock conflicts arise, `cargo update --workspace` minimally or resolve by regeneration, then re-test.

### Task 1.3: Push + open PR

```bash
git push -u origin chore/backmerge-main-to-develop
```

Open PR `chore/backmerge-main-to-develop → develop` via `gh pr create --body-file` (quoted heredoc). Body includes:
- what the 7 main-only commits were and that their content was already on develop
- that `project.yml` is the only net-new file
- **bolded: "merge with a merge-commit — do not squash"** (ia-gitflow Rule 2)

---

## PR 2 — Licensing + provenance

**Branch:** `chore/apache2-licensing` off updated `origin/develop` (after PR 1 merges; otherwise rebase later)

### Task 2.1: LICENSE + CONTRIBUTING.md

- Create `LICENSE`: full Apache-2.0 text from `https://www.apache.org/licenses/LICENSE-2.0.txt` (curl it; verify 201 lines / the appendix "APPENDIX" heading present).
- Create `CONTRIBUTING.md`: short doc — dev workflow (gitflow, PRs to develop, verification matrix), and the mandatory line: `All contributors must sign the [CLA](https://github.com/Industrial-Algebra/.github/blob/main/CLA.md).`

### Task 2.2: Manifest fields

All six `cliffy-*/Cargo.toml`:
- `license = "MIT"` → `license = "Apache-2.0"` (5 crates); **add** the field to `cliffy-wasm/Cargo.toml` (currently missing entirely).
- `repository = "https://github.com/justinelliottcobb/Cliffy"` → `"https://github.com/Industrial-Algebra/Cliffy"` (content salvaged from the dropped `release/v0.3.2`; that branch is deleted after this PR merges — see Task 2.5).
- Same repository fix in root `package.json`, `cliffy-tsukoshi/package.json`, `tools/create-cliffy/package.json`, `vite-plugin-algebraic-tsx/package.json` (mirror the release branch's diff: `git diff origin/develop...origin/release/v0.3.2 -- '*.json'`).

### Task 2.3: SPDX headers on all source files

Every `.rs` file in the six crates' `src/` (39 files) gets:

```rust
// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0
```

Script-assisted (prepend unless an SPDX line already exists), then `cargo fmt --all` to normalize.

### Task 2.4: README + verify

- README.md: update any license badge/section MIT → Apache-2.0.
- Full verification matrix. Expected: green, zero doc warnings delta.

### Task 2.5: Push + open PR + note branch deletion

PR `chore/apache2-licensing → develop`. Body notes the MIT→Apache-2.0 decision (operator, 2026-08-27), cites `ia-licensing`, and recommends the operator delete `origin/release/v0.3.2` after merge (its unique content — URL fixes — is now in develop; the 0.3.2 version bump is intentionally dropped per decision).

---

## PR 3 — Tooling + docs hygiene

**Branch:** `chore/tooling-docs-hygiene` off updated `origin/develop` (rebase after PR 2 — both touch `.rs` files)

### Task 3.1: rust-toolchain.toml

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

Matches Ijima/Lonis/Sakamoto/Proserpina convention and the CI's dtolnay stable pin.

### Task 3.2: CHANGELOG.md (backfill + durable tag mapping)

Keep-a-Changelog format. Sections: Unreleased (0.4.0 — salvage cycle: note license change, CRDT deprecation incoming, typed algebra); then backfilled:
- `[0.3.1] — 2026-03-03` — npm-only release; **durable mapping: version fields bumped in #171 (4021016); no git tag was created** (the Phase 0 "record durably" item)
- `[0.3.0]` / `[0.2.0]` / `[0.1.x]` from git tags + ROADMAP release-history section
- Note: `release/v0.3.2` was drafted (repo-URL fixes + version bump) and intentionally dropped 2026-08-27; next release targets 0.4.0.

### Task 3.3: Fix the 28 cargo doc warnings

Locate with `cargo doc --workspace --no-deps --all-features 2>&1 | grep -A2 "^warning"`. Known classes:
- unresolved link to `i`/`5`/`6`/`7` — markdown ordered/implicit lists flush against text lines being parsed as links; fix with a blank line before the list or backticking.
- unclosed HTML tag `<T>`/`<f64>` — wrap generics in backticks or escape.

Do **not** reword API semantics; formatting only. Verify: `cargo doc --workspace --no-deps --all-features` → **0 warnings**.

### Task 3.4: CI + publish alignment

- `ci.yml`: clippy step → `cargo clippy --workspace --all-targets --all-features -- -D warnings` (match the local matrix).
- `publish-crates.yml`: trigger switches from push-to-main+paths to **`on: push: tags: ['v*']`** (+ keep `workflow_dispatch` for manual/dry-run). This is the ia-gitflow tag-driven publish contract.
- `.gitignore`: add `.pi/` (local agent state currently shows as untracked noise).

### Task 3.5: Verify + PR

Full verification matrix. PR `chore/tooling-docs-hygiene → develop`.

---

## PR 4 — Phase 0: gate the broken CRDT machinery

**Branch:** `feature/phase0-crdt-deprecation` off updated `origin/develop`. Source: salvage plan §Phase 0 + the verified failure modes.

### Task 4.1: Deprecation attributes + failure-mode docs

In `cliffy-protocols/src/`:
- `crdt.rs`: `#[deprecated]` on `GeometricCRDT` (or at minimum `merge` + `create_operation`) with doc comments stating the verified failure: *"merge annihilation to GA3::zero() (replay loop is dead code under the dedup guard); op IDs minted from operations.len() collide across empty replicas; geometric_mean averages exponentials where exp/log are not inverse. See docs/plans/2026-08-26-geometric-crdt-salvage.md. Superseded by ObservationSet (v0.4.0)."*
- `lattice.rs`: `#[deprecated]` on `GA3Lattice::join` only — magnitude-dominance join violates the join-semilattice hull property (join(+1,-1) = cosh(1) ≈ 1.543). **`ComponentLattice` and `vector_clock` are NOT deprecated** — they are the sound floor.
- Fix fallout: tests referencing deprecated items get `#[allow(deprecated)]` at the test-module level with a comment (they pin current behavior until Phase 1 replaces them); wasm bindings in `cliffy-wasm/src/protocols.rs` get matching deprecation doc notes (JS-visible `@deprecated` in doc comments).
- Compile-clean under `-D warnings` (deprecation lints in tests are allowed explicitly, not globally).

### Task 4.2: npm deprecation draft (operator executes)

Create `docs/deprecations/@cliffy-ga-core.md` with the exact proposed command and message: reactivity sound; geometric merge broken (annihilation); successor path = salvage plan; link the handoff. **The agent does not run `npm deprecate`** — the operator approves and executes.

### Task 4.3: Verify + PR

Full matrix + `cargo test --workspace` still 223 green. PR `feature/phase0-crdt-deprecation → develop`. Body cross-references PR #301/#303 and the salvage plan; notes this closes Phase 0 items 2–4 (item 1, push-or-drop, already resolved by PR #301 carrying the orphaned commits).

---

## Post-plan (not in scope here)

1. **Retroactive v0.3.1 tag** (operator action, from the CHANGELOG mapping): `git tag -a v0.3.1 -m "npm-only release; mapping recorded in CHANGELOG" 4021016 && git push origin v0.3.1` — optional; publish workflow is now tag-driven, and `v*` tags on pre-0.4.0 commits would fire crates.io publishes that have never happened. **Recommendation: skip the retroactive tag; the CHANGELOG mapping is the durable record.**
2. **Stale remote branches** (operator): delete `amari-migration` (fossil), `release/v0.3.2` (folded), `cliffy-alive-*` (feature-branch archived), old `feature/phase-*` if merged.
3. **Phase 1 planning session**: ObservationSet + rotor consensus projection, full TDD (probes → permanent value-oracle tests first). Separate plan doc.
4. **v0.4.0 version bump** via `ia-version-bump` only when Phase 1 lands; typed-algebra BACKLOG items re-scoped against amari 0.23 (they assume 0.19 APIs).
