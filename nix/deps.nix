# Neovim's bundled C dependencies (LuaJIT, libuv, luv, lpeg, tree-sitter and
# its parsers, unibilium, utf8proc), built once into a self-contained prefix.
#
# `cmake.deps` normally *downloads* these tarballs at build time, which a
# sandboxed Nix build can't do. Instead Nix fetches every source from
# `deps.json` (the source of truth) and generates the `deps.txt` cmake.deps
# reads, pointing each entry at its prefetched tarball in the store. The build
# never touches the network — cmake is only the offline *builder* here, Nix
# owns the sourcing.
#
# The result is the `usr/` prefix cmake.deps installs: static archives under
# `lib/` + `lib64/`, and the compiled tree-sitter parsers under
# `lib/nvim/parser/`. `build.rs` links against it when `$NVIM_DEPS_PREFIX`
# points here.
{
  lib,
  stdenv,
  fetchurl,
  writeText,
  cmake,
  ninja,
  gnumake,
  pkg-config,
  # Source scoped to just the cmake machinery (`cmake.deps/` + `cmake/`), so a
  # change to the Rust sources doesn't force these C deps to rebuild.
  src,
}:

let
  manifest = lib.filterAttrs (name: _: !(lib.hasPrefix "_" name)) (lib.importJSON ./deps.json);

  # Fetch each source tarball. The sha256 comes straight from the manifest.
  sources = lib.mapAttrs (_: dep: fetchurl { inherit (dep) url sha256; }) manifest;

  # The `deps.txt` cmake.deps reads, generated from the manifest with every
  # `<NAME>_URL` pointed at its prefetched tarball. cmake treats a bare local
  # path as a `file://` source and copies it in; integrity is already
  # guaranteed by fetchurl, so the build runs with `DEPS_IGNORE_SHA`.
  depsTxt = writeText "deps.txt" (
    lib.concatStrings (lib.mapAttrsToList (name: tarball: "${name}_URL ${tarball}\n") sources)
  );
in

stdenv.mkDerivation {
  pname = "nvim-deps";
  version = "0.12.4";

  inherit src;

  nativeBuildInputs = [
    cmake
    ninja
    gnumake # LuaJIT and luv are GNU-make builds driven by ExternalProject
    pkg-config
  ];

  postPatch = ''
    cp ${depsTxt} cmake.deps/deps.txt
  '';

  # We invoke cmake by hand against `cmake.deps/`; there is no top-level
  # CMakeLists for the default configure hook to pick up.
  dontUseCmakeConfigure = true;

  buildPhase = ''
    runHook preBuild

    cmake -S cmake.deps -B .deps -G Ninja \
      -D CMAKE_BUILD_TYPE=RelWithDebInfo \
      -D DEPS_IGNORE_SHA=TRUE
    cmake --build .deps

    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    cp -r .deps/usr $out
    runHook postInstall
  '';

  # This prefix is consumed almost entirely as static archives, so nix's fixup
  # buys nothing here and actively gets in the way: its lib64→lib consolidation
  # fails on the mixed `lib`/`lib64` tree cmake.deps installs. The one thing
  # fixup would clean — a leftover nix-store RUNPATH on the parser `.so`s — is
  # inert anyway (their only NEEDED lib is libc), and the final `nvim` package
  # re-runs fixup over the parsers it copies out, so nothing ships dirty.
  dontFixup = true;
}
