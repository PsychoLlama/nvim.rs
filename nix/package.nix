# The `nvim` executable: the c2rust-transpiled Neovim, linked against the
# prebuilt C dependencies from `deps.nix`.
#
# Two things make this build unusual:
#   1. `build.rs` would normally shell out to cmake.deps to build the C
#      libraries. We short-circuit that with `$NVIM_DEPS_PREFIX`, pointing it at
#      the already-built `nvim-deps` prefix so the sandboxed build never needs
#      the network or cmake.
#   2. The transpiled crate depends on `c2rust-bitfields` by an absolute path
#      into a developer checkout. We fetch that crate from its upstream fork and
#      rewrite the path so the dependency resolves inside the sandbox.
{
  lib,
  rustPlatform,
  fetchFromGitHub,
  nvim-deps,
  # Source scoped to the Rust crate + runtime files.
  src,
}:

let
  # `c2rust-bitfields` (and its sibling derive macro) live in the c2rust fork,
  # referenced from Cargo.toml by an absolute developer path. Fetch the fork so
  # `postPatch` can repoint that path at the store.
  c2rust = fetchFromGitHub {
    owner = "PsychoLlama";
    repo = "c2rust";
    rev = "5fb2f166bf6d06d073117723a23e4632cb397d70";
    hash = "sha256-nfELh98lu5eCG132ia0Xtnv5pzWvmdhetzzx6Eq0xps=";
  };
in

rustPlatform.buildRustPackage {
  pname = "nvim";
  version = "0.12.4";

  inherit src;

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  # Repoint the out-of-tree `c2rust-bitfields` path dependency at the fetched
  # fork. `--replace-fail` makes a Cargo.toml refactor that moves the path fail
  # loudly here instead of silently building against nothing.
  postPatch = ''
    substituteInPlace Cargo.toml \
      --replace-fail \
        '/home/overlord/projects/immunant/c2rust/c2rust-bitfields' \
        '${c2rust}/c2rust-bitfields'
  '';

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
    license = lib.licenses.asl20;
    mainProgram = "nvim";
    platforms = lib.platforms.linux;
  };
}
