{
  description = "Neovim development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        # Lua environment with required packages for building
        luaEnv = pkgs.luajit.withPackages (ps: with ps; [
          lpeg
          luabitop
          mpack
        ]);

        # Build dependencies
        buildInputs = with pkgs; [
          libuv
          luajitPackages.libluv
          tree-sitter
          unibilium
          utf8proc
          msgpack-c
          libiconv
          libvterm-neovim
        ];

        # Native build dependencies (tools needed at build time)
        nativeBuildInputs = with pkgs; [
          cmake
          ninja
          pkg-config
          gettext
          luaEnv
        ];

        # Rust toolchain for migration
        rustToolchain = with pkgs; [
          rustc
          cargo
          clippy
          rustfmt
          rust-analyzer
        ];

        # Dependencies for running tests
        checkInputs = with pkgs; [
          nodejs
          fish
          (python3.withPackages (ps: with ps; [
            pynvim
            msgpack
          ]))
        ];

      in
      {
        # Development shell - use `nix develop`
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;

          packages = checkInputs ++ rustToolchain ++ [ pkgs.just ];

          # Ensure pkg-config can find our dependencies
          PKG_CONFIG_PATH = pkgs.lib.makeSearchPath "lib/pkgconfig" buildInputs;

          # Help CMake find dependencies
          CMAKE_PREFIX_PATH = pkgs.lib.makeSearchPath "" buildInputs;
        };

        # Build the package - use `nix build`
        packages.default = pkgs.stdenv.mkDerivation {
          pname = "neovim";
          version = "0.12.0-dev";

          src = ./.;

          inherit buildInputs nativeBuildInputs;

          cmakeFlags = [
            "-DCMAKE_BUILD_TYPE=Release"
            "-DUSE_BUNDLED=OFF"
          ];

          # Let CMake use Ninja
          cmakeGenerator = "Ninja";

          meta = with pkgs.lib; {
            description = "Vim-fork focused on extensibility and usability";
            homepage = "https://neovim.io";
            license = licenses.asl20;
            platforms = platforms.unix;
          };
        };
      }
    );
}
