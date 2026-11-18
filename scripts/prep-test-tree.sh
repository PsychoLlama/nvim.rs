#!/usr/bin/env bash
# Everything the test suites need before their first test, done once per run
# (the harnesses call this; a run split across workers calls it before
# starting them). Safe to run beside another suite's prep: one lock per
# build dir, and nothing a running suite reads is ever briefly missing.
#
# Usage: prep-test-tree.sh [functional|unit|benchmark|oldtest...]
#
# Always: lays out the build dir like the CMake one the suites expect
# ($BUILD_DIR in test/old/testdir/runnvim.sh, test_build_dir in
# test/cmakeconfig/paths.lua):
#   <build>/bin/nvim  -> hard link to $NVIM_BIN (default target/debug/nvim)
#   <build>/runtime   -> doc/ link to the in-tree docs + generated syntax files
#   <build>/lib/nvim  -> Nix-built treesitter parsers
# and generates runtime/doc/tags. Per suite named: functional/benchmark build
# the C fixture programs next to the tested nvim; unit builds unit-fixtures.so
# and the ffi cdefs chunk. (oldtest needs nothing more here; its testdir copy
# is per run, made by run-oldtest.sh.)
#
# The build dir is $NVIM_TEST_BUILD_DIR (default target/); the ASan recipes
# give it target/asan/tree, so an ASan run never relinks the target/bin/nvim
# a plain run beside it is testing.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
: "${NVIM_DEPS_PREFIX:?NVIM_DEPS_PREFIX must be set (enter the flake dev shell)}"

build=$(realpath -m "${NVIM_TEST_BUILD_DIR:-$root/target}")
mkdir -p "$build"
exec 9>"$build/.prep.lock"
flock 9

# Replace a symlink only when it points elsewhere: `ln -sfn` unlinks first,
# and a suite running beside this one would see the path missing.
link() { # target linkname
  [[ -L $2 && $(readlink "$2") == "$1" ]] || ln -sfn "$1" "$2"
}

mkdir -p "$build/lib"
link "$NVIM_DEPS_PREFIX/lib/nvim" "$build/lib/nvim"

# Only doc/: upstream's build runtime held generated files (helptags), not a
# full runtime copy. Exposing everything (e.g. compiler/*.vim) would double
# every runtime file in &rtp and break tests that list runtime files.
if [[ -L $build/runtime ]]; then rm "$build/runtime"; fi
mkdir -p "$build/runtime"
link "$root/runtime/doc" "$build/runtime/doc"

# Specs resolve nvim through $BUILD_DIR/bin/nvim (and fs_spec asserts that
# layout), so the tested binary is exposed here; $NVIM_BIN overrides the
# default cargo build (e.g. the ASan recipes point it at target/asan). A real
# directory and a hard link, not symlinks: fs_spec's upward find() expects
# `bin` to stat as a directory, and vim.fs.dir()/find() report an unfollowed
# symlink as type 'link' where the specs expect 'file'. Refreshed every run,
# so a cargo rebuild (new inode) can't leave it stale; swapped in by rename,
# so a suite already running never finds it missing.
if [[ -L $build/bin ]]; then rm "$build/bin"; fi
mkdir -p "$build/bin"
nvim_bin=${NVIM_BIN:-$root/target/debug/nvim}
# Missing binary is not an error here; the harness reports it with a better
# message ("run `just build` first") when it checks $NVIM_PRG.
if [[ -e $nvim_bin && ! $nvim_bin -ef $build/bin/nvim ]]; then
  ln -f "$nvim_bin" "$build/bin/.nvim.new"
  mv -f "$build/bin/.nvim.new" "$build/bin/nvim"
fi

export NVIM_LOG_FILE=${NVIM_LOG_FILE:-$build/prep-nvim.log}
if [[ -x $build/bin/nvim ]]; then
  # Upstream's build runtime also carried generated files (the vimscript
  # syntax tables); only specs that opt in via add_builddir_to_rtp() see
  # them. Runs after the bin/ link: the generators run under the built nvim.
  "$root/scripts/gen.sh" --nvim "$build/bin/nvim" --runtime "$build/runtime"

  # The help tags every suite's :help reads. Generated here, once, instead of
  # by the specs that need them (help_spec used to rewrite the in-tree file
  # mid-run, under the other suites' feet); written beside the docs and
  # renamed into place, so a reader never sees half a file.
  doc=$root/runtime/doc
  if [[ ! -f $doc/tags || -n $(find "$doc" -maxdepth 1 -name '*.txt' -newer "$doc/tags" -print -quit) ]]; then
    tmp=$(mktemp -d "$build/helptags.XXXXXX")
    ln -s "$doc"/*.txt "$tmp/"
    "$build/bin/nvim" -u NONE -i NONE --headless -c "helptags ++t $tmp" -c 'qa!'
    mv -f "$tmp/tags" "$doc/tags"
    rm -rf "$tmp"
  fi
fi

# The C fixture programs (upstream: test/functional/fixtures/CMakeLists.txt),
# next to the nvim binary under test, where testnvim.lua's testprg() expects
# them. tty-test and streams-test need libuv from the Nix deps prefix.
build_fixtures() {
  local bin_dir fixtures=$root/test/functional/fixtures
  bin_dir=$(dirname "${NVIM_PRG:-$build/bin/nvim}")
  local uv_flags=(-I"$NVIM_DEPS_PREFIX/include" "$NVIM_DEPS_PREFIX/lib/libuv.a" -lpthread -ldl -lrt -lm)
  fixture() {
    local out=$bin_dir/$1 src=$fixtures/$2
    shift 2
    if [[ ! -x $out || $src -nt $out ]]; then
      echo "compiling test fixture: $(basename "$out")" >&2
      cc -O2 -o "$out.new" "$src" "$@"
      mv -f "$out.new" "$out"
    fi
  }
  fixture tty-test tty-test.c "${uv_flags[@]}"
  fixture streams-test streams-test.c "${uv_flags[@]}"
  fixture shell-test shell-test.c
  fixture pwsh-test shell-test.c # fake pwsh for make_filter_cmd() tests
  fixture printargs-test printargs-test.c
  fixture printenv-test printenv-test.c
}

for suite in "$@"; do
  case $suite in
    functional | benchmark) build_fixtures ;;
    unit)
      # The unit-test helper library; see build-unit-fixtures.sh (shared
      # with the ABI ledger, which reads its undefined-symbol list). It
      # also refreshes the ffi.cdef surface (tools/ffigen) that
      # test/unit/testutil.lua loads from <build>/ffi.
      "$root/scripts/build-unit-fixtures.sh" "$(dirname "${NVIM_PRG:-$build/bin/nvim}")/unit-fixtures.so"
      if [[ $build != "$root/target" ]]; then link "$root/target/ffi" "$build/ffi"; fi
      ;;
    oldtest) ;;
    *)
      echo "prep-test-tree.sh: unknown suite: $suite" >&2
      exit 64
      ;;
  esac
done
