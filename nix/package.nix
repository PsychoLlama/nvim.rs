# The `nvim` executable: Neovim ported to Rust, linked against the prebuilt C
# dependencies from `deps.nix`.
#
# `build.rs` would normally shell out to cmake.deps to build the C libraries. We
# short-circuit that with `$NVIM_DEPS_PREFIX`, pointing it at the already-built
# `nvim-deps` prefix so the sandboxed build never needs the network or cmake.
{
  lib,
  rustPlatform,
  nvim-deps,
  # Source scoped to the Rust crate + runtime files.
  src,
  # What `nvim --version` reports. build.rs would normally read this out of
  # git, but `src` is a filtered store path with no `.git`, so the flake has
  # to state it.
  nvimRsVersion ? "unknown",
}:

rustPlatform.buildRustPackage {
  pname = "nvim";
  version = "0.12.4";

  inherit src;

  # Every dependency comes from crates.io, so the lock file alone is enough:
  # no `outputHashes` entries for vendored git sources.
  cargoLock.lockFile = ../Cargo.lock;

  # Link against the prebuilt C deps instead of building them via cmake.deps.
  env.NVIM_DEPS_PREFIX = "${nvim-deps}";

  # The version banner, since build.rs can't derive one from the sandbox.
  env.NVIM_RS_VERSION = nvimRsVersion;

  # Bake the installed runtime + parser locations so the binary is relocatable
  # within the store with no env vars. `build.rs` honours these overrides.
  env.NVIM_DEFAULT_VIMRUNTIME_DIR = "${placeholder "out"}/share/nvim/runtime";
  env.NVIM_DEFAULT_LIB_DIR = "${placeholder "out"}/lib/nvim";

  # Install the runtime files and tree-sitter parsers alongside the binary, at
  # the baked-in paths above. buildRustPackage installs `$out/bin/nvim` itself.
  postInstall = ''
    mkdir -p $out/share/nvim
    cp -r runtime $out/share/nvim/runtime

    mkdir -p $out/lib/nvim
    cp -r ${nvim-deps}/lib/nvim/parser $out/lib/nvim/parser

    # License texts must travel with the binary: the LGPL'd xdiff/unibilium
    # ports are compiled in, and the (L)GPL requires conveying their texts.
    mkdir -p $out/share/doc/nvim
    cp -r LICENSE.txt licenses $out/share/doc/nvim/

    # Generate the vimscript syntax tables into the installed runtime, as
    # upstream releases ship them. The source runtime deliberately omits
    # generated.vim (the test suites' default runtime must not carry it), so
    # run the generator — the installed binary itself — over the vendored
    # metadata. Native-only, like helptags below.
    HOME=$(mktemp -d) bash scripts/gen.sh --nvim $out/bin/nvim \
      --runtime $out/share/nvim/runtime

    # Regenerate the help tags. Upstream's CMake build produced
    # `runtime/doc/tags` by running `:helptags`; with that tooling gone,
    # nothing else generates it and `:help <topic>` fails with "E433: No
    # tags file". Run the just-built binary against its own installed docs.
    # Native-only: this executes the target binary, which a cross build
    # could not — fine, since the flake is Linux-native anyway.
    HOME=$(mktemp -d) $out/bin/nvim --headless -u NONE \
      -c "helptags $out/share/nvim/runtime/doc" -c "qa!"
  '';

  # The suites run through `just` locally, not from the nix build.
  doCheck = false;

  meta = {
    description = "Neovim v0.12.4, ported to Rust";
    homepage = "https://github.com/PsychoLlama/nvim.rs";
    # Apache-2.0 overall, Vim-license contributions, and the in-tree ports
    # (xdiff: LGPL-2.1+ except xhistogram under the Eclipse Distribution
    # License v1.0 = 3-clause BSD; unibilium: LGPL-3.0+; utf8proc: MIT with
    # Unicode-license data tables). See LICENSE.txt.
    license = with lib.licenses; [
      asl20
      vim
      lgpl21Plus
      lgpl3Plus
      bsd3
      mit
      unicode-dfs-2015
    ];
    mainProgram = "nvim";
    platforms = lib.platforms.linux;
  };
}
