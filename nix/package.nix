# The `nvim` executable: the c2rust-transpiled Neovim, linked against the
# prebuilt C dependencies from `deps.nix`.
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
}:

rustPlatform.buildRustPackage {
  pname = "nvim";
  version = "0.12.4";

  inherit src;

  cargoLock = {
    lockFile = ../Cargo.lock;

    # `c2rust-bitfields` (and its derive macro) are a git dependency on the
    # c2rust fork; buildRustPackage needs the vendored-source hash to fetch it in
    # the sandbox. One entry covers every crate from that git source.
    outputHashes = {
      "c2rust-bitfields-0.22.1" = "sha256-ZUPK27at1YQwN8nSO+Alxs+vAXH7u+RAc0PgSZ3BLh0=";
    };
  };

  # Link against the prebuilt C deps instead of building them via cmake.deps.
  env.NVIM_DEPS_PREFIX = "${nvim-deps}";

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

  # The transpiled sources have no test harness wired up here.
  doCheck = false;

  meta = {
    description = "Neovim v0.12.4, transpiled to Rust with c2rust";
    homepage = "https://github.com/PsychoLlama/nvim.rs";
    # Apache-2.0 overall, Vim-license contributions, and the in-tree LGPL
    # ports (xdiff: LGPL-2.1+, unibilium: LGPL-3.0+). See LICENSE.txt.
    license = with lib.licenses; [
      asl20
      vim
      lgpl21Plus
      lgpl3Plus
    ];
    mainProgram = "nvim";
    platforms = lib.platforms.linux;
  };
}
