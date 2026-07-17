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
# A few specs (e.g. functional/harness) locate nvim at $BUILD_DIR/bin/nvim
# rather than honoring $NVIM_PRG. A real directory (not a bin -> debug
# symlink): fs_spec's upward find() expects `bin` to stat as a directory.
if [[ -L $root/target/bin ]]; then rm "$root/target/bin"; fi
mkdir -p "$root/target/bin"
ln -sfn ../debug/nvim "$root/target/bin/nvim"
