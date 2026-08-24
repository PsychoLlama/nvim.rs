{
  description = "Port of Neovim to Rust";

  inputs = {
    # Linux-only: the build.rs link line (`-lrt`, `-ldl`/`-lutil`,
    # `--export-dynamic`) and the ported sources are GNU/Linux-bound,
    # so there is nothing buildable to advertise on Darwin.
    systems.url = "github:nix-systems/default-linux";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
      systems,
    }:

    let
      inherit (nixpkgs) lib;

      eachSystem = lib.flip lib.mapAttrs (
        lib.genAttrs (import systems) (
          system:
          import nixpkgs {
            inherit system;
            overlays = [ fenix.overlays.default ];
          }
        )
      );

      mkToolchain =
        pkgs:
        (pkgs.fenix.toolchainOf {
          channel = "nightly";
          date = "2026-07-24";
          sha256 = "sha256-qd4wSkwz4JclURPHYV67uLzYcreME2kmkVGogQIzvMM=";
        }).withComponents
          [
            "cargo"
            "clippy"
            "miri"
            "rust-analyzer"
            "rust-src"
            "rustc"
            "rustfmt"
          ];
    in

    {
      packages = eachSystem (
        system: pkgs:

        let
          toolchain = mkToolchain pkgs;
        in

        rec {
          default = nvim;

          nvim-deps = pkgs.callPackage ./nix/deps.nix {
            src = lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.unions [
                ./cmake.deps
                ./cmake
              ];
            };
          };

          nvim = pkgs.callPackage ./nix/package.nix {
            inherit nvim-deps;

            # `src` below is a filtered store path with no `.git`, so build.rs
            # cannot work the version out for itself. Only the flake knows the
            # revision — and only as a sha: a release tag is invisible here, so
            # this always reads as a development build. Releases are cut by
            # `just package <version>`, which states the tag outright.
            # `shortRev` exists only for a clean checkout; a dirty tree has
            # `dirtyShortRev` (already suffixed `-dirty`), and a plain path
            # source has neither.
            nvimRsVersion = "dev-${self.shortRev or self.dirtyShortRev or "unknown"}";

            rustPlatform = pkgs.makeRustPlatform {
              cargo = toolchain;
              rustc = toolchain;
            };

            src = lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.unions [
                ./Cargo.toml
                ./Cargo.lock
                ./crates
                ./runtime
                # postInstall installs the license texts next to the binary.
                ./LICENSE.txt
                ./licenses
                # postInstall generates runtime files (syntax tables) with it.
                ./scripts/gen.sh
              ];
            };
          };
        }
      );

      devShells = eachSystem (
        system: pkgs: {
          default = pkgs.mkShell {
            # Link against the same Nix-built C deps as the package; build.rs
            # requires this to be set.
            NVIM_DEPS_PREFIX = "${self.packages.${system}.nvim-deps}";

            # Phase 5 drove the warning count to zero; fail fast on any
            # regression. CI inherits this through `nix develop`, so this is
            # the single source of truth. Opt out for a mid-refactor build
            # with `RUSTFLAGS= cargo build`; `just asan` overrides it with
            # the sanitizer flags (uninstrumented lint parity is fine there).
            RUSTFLAGS = "-D warnings";

            # `runtime/doc/tags` is a generated artifact (gitignored): upstream's
            # CMake build produced it, and the package regenerates it in
            # postInstall. A dev binary bakes the in-tree `runtime/` as its
            # runtime dir, so generate the tags here too, once, if missing —
            # otherwise `:help` fails with "E433: No tags file". The tags format
            # is stable, so nixpkgs' nvim yields the same output as ours without
            # needing a compiled binary to bootstrap from.
            shellHook = ''
              if [ -d runtime/doc ] && [ ! -f runtime/doc/tags ]; then
                ${pkgs.neovim}/bin/nvim --headless -u NONE \
                  -c 'helptags runtime/doc' -c 'qa!'
              fi

              # Set up git hooks.
              git config set extensions.worktreeConfig true
              git config set --worktree include.path "$(git rev-parse --show-toplevel)/.gitconfig"
            '';

            packages = [
              (mkToolchain pkgs)
              pkgs.just
              pkgs.cmake
              pkgs.ninja
              pkgs.pkg-config
              pkgs.nixfmt
              pkgs.prettier
              pkgs.ruff
              pkgs.stylua
              pkgs.treefmt
              pkgs.luaPackages.luacheck
              pkgs.lua-language-server
            ];
          };
        }
      );
    };
}
