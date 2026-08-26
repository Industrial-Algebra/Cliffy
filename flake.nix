{
  description = "Cliffy — geometric state management and distributed synchronization";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      forAllSystems =
        f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          name = "cliffy-dev";
          packages = with pkgs; [
            # Toolchain comes from rust-toolchain.toml (pinned nightly +
            # rustfmt/clippy/rust-src); rustup honors the file automatically.
            rustup
            rust-analyzer

            # 2026 toolkit dev tools
            bacon # background clippy/test watcher — `bacon clippy`
            cargo-nextest # test runner — `cargo nextest run`
            cargo-seek # crate search/add TUI
            watchexec # general file watcher — `watchexec -e rs "cargo clippy"`
          ];

          shellHook = ''
            echo "Cliffy dev shell"
            echo "  toolchain: $(rustc --version 2>/dev/null || echo 'installing via rustup on first cargo run')"
            echo "  watcher:   bacon clippy | bacon test"
            echo "  tests:     cargo nextest run && cargo test --doc --workspace"
          '';
        };
      });
    };
}
