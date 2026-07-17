{
  description = "Port of Neovim to Rust";

  inputs = {
    # Linux-only: the build.rs link line (`-lrt`, `-ldl`/`-lutil`,
    # `--export-dynamic`) and the c2rust-transpiled sources are GNU/Linux-bound,
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
          date = "2023-04-15";
          sha256 = "sha256-MbIq5CSCT5DjO4iLNNERhJ5YPth50hzBE9tUtC/UR3o=";
        }).withComponents
          [
            "cargo"
            "clippy"
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

            rustPlatform = pkgs.makeRustPlatform {
              cargo = toolchain;
              rustc = toolchain;
            };

            src = lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.unions [
                ./Cargo.toml
                ./Cargo.lock
                ./build.rs
                ./lib.rs
                ./src
                ./runtime
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
            '';

            packages = [
              (mkToolchain pkgs)
              pkgs.cmake
              pkgs.ninja
              pkgs.pkg-config
              pkgs.nixfmt
              pkgs.prettier
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
