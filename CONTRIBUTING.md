# Contributing to Cliffy

Thank you for your interest in contributing!

## Contributor License Agreement

All contributors must sign the
[CLA](https://github.com/Industrial-Algebra/.github/blob/main/CLA.md).
The CLA covers all Industrial Algebra projects — sign it once, contribute
anywhere. Pull requests cannot be merged without a signed CLA on file.

## Development Workflow

Cliffy follows the [IA gitflow](https://github.com/Industrial-Algebra/Cliffy/blob/develop/docs/plans/2026-08-27-modernization-and-phase0.md)
discipline:

1. Branch from `develop` (`feature/*`, `fix/*`, `chore/*`, `docs/*`).
2. Implement with TDD — failing test first, then minimal implementation.
3. Open a PR against `develop`. PRs are reviewed and merged by a human.
4. Releases cut `release/v*` branches from `develop`, merge to `main`,
   tag `v*` (the tag triggers publish), then backmerge `main → develop`
   **with a merge commit — never squash a backmerge**.

## Verification Matrix

The default toolchain is **nightly** (`rust-toolchain.toml`); CI lints and
tests on nightly **and** keeps a stable test lane (the library must stay
crates.io-compatible). To check stable locally:

```bash
RUSTUP_TOOLCHAIN=stable cargo test --workspace   # rustup env beats the file
```

Every PR must pass:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace      # unit/integration (or: cargo test --workspace)
cargo test --doc --workspace       # doctests (nextest doesn't run them)
cargo doc --workspace --no-deps --all-features   # zero warnings
```

For TypeScript packages (`cliffy-tsukoshi`), also `npm run build && npm test`.

## Coding Standards

See `CLAUDE.md` in the repo root and the IA coding standards: phantom types
for compile-time safety, `Result` over panics, exhaustive matches, additive
feature gates, every public item documented. Before writing new geometry,
check `amari-core` — compose, don't recreate.

## License

Contributions are licensed Apache-2.0 (see [LICENSE](LICENSE)); the CLA
grants Industrial Algebra the right to relicense.
