#!/usr/bin/env bash
# Lay out target/ like the CMake build dir the test suites expect ($BUILD_DIR
# in test/old/testdir/runnvim.sh, test_build_dir in test/cmakeconfig/paths.lua):
#   target/runtime  -> in-tree runtime (helptags are generated in-tree)
#   target/lib/nvim -> Nix-built treesitter parsers
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
: "${NVIM_DEPS_PREFIX:?NVIM_DEPS_PREFIX must be set (enter the flake dev shell)}"

mkdir -p "$root/target/lib"
ln -sfn "$NVIM_DEPS_PREFIX/lib/nvim" "$root/target/lib/nvim"

# Only doc/: upstream's build runtime held generated files (helptags), not a
# full runtime copy. Exposing everything (e.g. compiler/*.vim) would double
# every runtime file in &rtp and break tests that list runtime files.
if [[ -L $root/target/runtime ]]; then rm "$root/target/runtime"; fi
mkdir -p "$root/target/runtime"
ln -sfn ../../runtime/doc "$root/target/runtime/doc"
# Specs resolve nvim through $BUILD_DIR/bin/nvim (and fs_spec asserts that
# layout), so the tested binary is exposed here; $NVIM_BIN overrides the
# default cargo build (e.g. the ASan recipes point it at target/asan). A real
# directory and a hard link, not symlinks: fs_spec's upward find() expects
# `bin` to stat as a directory, and vim.fs.dir()/find() report an unfollowed
# symlink as type 'link' where the specs expect 'file'. Refreshed every run,
# so a cargo rebuild (new inode) can't leave it stale.
if [[ -L $root/target/bin ]]; then rm "$root/target/bin"; fi
mkdir -p "$root/target/bin"
nvim_bin=${NVIM_BIN:-$root/target/debug/nvim}
# Missing binary is not an error here; the harness reports it with a better
# message ("run `just build` first") when it checks $NVIM_PRG.
if [[ -e $nvim_bin ]]; then
  ln -f "$nvim_bin" "$root/target/bin/nvim"
fi

# Upstream's build runtime also carried the generated vimscript syntax
# tables; only specs that opt in via add_builddir_to_rtp() see them. Runs
# after the bin/ link: the generator is the built nvim itself.
"$root/scripts/gen-vimvim.sh"
