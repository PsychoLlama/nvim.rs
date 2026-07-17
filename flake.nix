{
  description = "Development environment";

  inputs = {
    systems.url = "github:nix-systems/default";
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

      overlays = [ fenix.overlays.default ];

      eachSystem = lib.flip lib.mapAttrs (
        lib.genAttrs (import systems) (
          system:
          import nixpkgs {
            inherit system overlays;
          }
        )
      );
    in

    {
      devShells = eachSystem (
        system: pkgs: {
          default = pkgs.mkShell {
            packages = [
              (
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
                ]
              )
              # Native toolchain for building neovim's bundled C dependencies
              # (LuaJIT, libuv, tree-sitter, …) under `.deps/`, which the Rust
              # crate links against. cmake drives the `cmake.deps` build and
              # emits a Ninja build; a C compiler comes from the stdenv shell.
              pkgs.cmake
              pkgs.ninja
              pkgs.pkg-config
              pkgs.nixfmt
              pkgs.prettier
              pkgs.stylua
              pkgs.treefmt
              pkgs.luaPackages.luacheck
            ];
          };
        }
      );
    };
}
