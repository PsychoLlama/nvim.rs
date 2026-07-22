#!/usr/bin/env bash
# Run the Lua test harness (functional tests, benchmarks) against the cargo
# build. Replaces the deleted cmake/RunTests.cmake: same environment sandbox,
# same `nvim -ll test/runner.lua` invocation.
#
# Usage: run-tests.sh <functional|unit|benchmark> [harness args...] [spec paths...]
#
# Positional args are spec files/dirs (repo-relative or absolute). Flags are
# passed through to the harness; use the --flag=value form (e.g.
# --filter='pattern'), since this script cannot tell which flags consume a
# separate value argument. With no spec path, the whole suite runs.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)

test_type=${1:-}
case "$test_type" in
  functional|unit|benchmark) shift ;;
  *)
    echo "usage: $0 <functional|unit|benchmark> [harness args...] [spec paths...]" >&2
    exit 64
    ;;
esac

# Prep before the -x check: prep-test-tree.sh is what (re)creates the
# target/bin/nvim hard link the default NVIM_PRG points at. Tests run that
# link, not target/debug/nvim directly: specs like fs_spec assert nvim lives
# at $BUILD_DIR/bin/nvim, and $NVIM_BIN swaps in other builds (ASan recipes).
"$root/scripts/prep-test-tree.sh"

NVIM_PRG=${NVIM_PRG:-$root/target/bin/nvim}
if [[ ! -x $NVIM_PRG ]]; then
  echo "$0: $NVIM_PRG not found; run \`just build\` first" >&2
  exit 66
fi
bin_dir=$(dirname "$NVIM_PRG")

# Compile the C test-fixture programs (upstream: test/functional/fixtures/
# CMakeLists.txt) next to the nvim binary, where testnvim.lua's testprg()
# expects them. tty-test and streams-test need libuv from the Nix deps prefix.
fixtures=$root/test/functional/fixtures
: "${NVIM_DEPS_PREFIX:?NVIM_DEPS_PREFIX must be set (enter the flake dev shell)}"
uv_flags=(-I"$NVIM_DEPS_PREFIX/include" "$NVIM_DEPS_PREFIX/lib/libuv.a" -lpthread -ldl -lrt -lm)
build_fixture() {
  local out=$bin_dir/$1 src=$fixtures/$2
  shift 2
  if [[ ! -x $out || $src -nt $out ]]; then
    echo "compiling test fixture: $(basename "$out")" >&2
    cc -O2 -o "$out" "$src" "$@"
  fi
}
if [[ $test_type != unit ]]; then
  build_fixture tty-test tty-test.c "${uv_flags[@]}"
  build_fixture streams-test streams-test.c "${uv_flags[@]}"
  build_fixture shell-test shell-test.c
  build_fixture pwsh-test shell-test.c # fake pwsh for make_filter_cmd() tests
  build_fixture printargs-test printargs-test.c
  build_fixture printenv-test printenv-test.c
else
  # The unit-test helper library; see build-unit-fixtures.sh (shared with the
  # ABI ledger, which reads its undefined-symbol list).
  "$root/scripts/build-unit-fixtures.sh" "$bin_dir/unit-fixtures.so"
  # The ffi.cdef surface, generated from the Rust crate (tools/ffigen);
  # test/unit/testutil.lua loads it in place of preprocessed C headers.
  "$root/scripts/gen-unit-cdefs.sh"
fi

# Scratch area (upstream's $BUILD_DIR): XDG sandbox, TMPDIR, logs. Start each
# run from a fresh sandbox — tests deliberately break it (bad tempdir perms,
# stale sockets) — but leave it in place afterwards for post-mortems.
# (Upstream instead deleted it after every run.)
build_dir=$root/target/testrun
xdg=$build_dir/Xtest_xdg
# TMPDIR must be SHORT: nvim puts its server sockets under $TMPDIR when
# $XDG_RUNTIME_DIR is unset (the in-test env), and unix sockets cap the path
# at ~107 bytes (sun_path). libuv silently truncates longer paths, which then
# collide ("--listen: address already in use"). A repo-relative Xtest_tmpdir
# exceeds the cap; upstream CI paths only pass because they are shorter.
tmpdir=${XDG_RUNTIME_DIR:-/tmp}/nvim.rs-Xtest-$(cksum <<<"$root" | cut -d' ' -f1)
chmod -R u+rwX "$xdg" "$tmpdir" 2>/dev/null || true
rm -rf "$xdg" "$tmpdir" "$build_dir/Xtest_rplugin_manifest"
mkdir -p "$xdg" "$tmpdir"

# The harness runs from inside the XDG sandbox; these links make repo-relative
# spec paths (test/…) and runtime lookups resolve from there.
ln -sfn "$root/runtime" "$xdg/runtime"
ln -sfn "$root/src" "$xdg/src"
ln -sfn "$root/test" "$xdg/test"
ln -sfn "$root/README.md" "$xdg/README.md"

export NVIM_TEST=1
export LC_ALL=en_US.UTF-8
export VIMRUNTIME=$root/runtime
export XDG_CONFIG_HOME=$xdg/config
export XDG_DATA_HOME=$xdg/share
export XDG_STATE_HOME=$xdg/state
export NVIM_RPLUGIN_MANIFEST=$build_dir/Xtest_rplugin_manifest
unset XDG_DATA_DIRS NVIM TMUX
export NVIM_LOG_FILE=${NVIM_LOG_FILE:-$build_dir/nvim.log}
export NVIM_PRG
export TMPDIR=$tmpdir
export HISTFILE=/dev/null
export SYSTEM_NAME=$(uname -s) # used by test/testutil.lua
export SHELL=sh                # tests assume POSIX sh #24941 #6172

# Split flags from spec paths; rewrite paths relative to the repo root so test
# ids stay stable ("test/functional/…") regardless of the caller's cwd.
args=()
paths=()
for arg in "$@"; do
  if [[ $arg == -* ]]; then
    args+=("$arg")
  else
    abs=$(realpath "$arg")
    paths+=("${abs#"$root"/}")
  fi
done
if [[ ${#paths[@]} -eq 0 ]]; then
  paths=(test/$test_type)
fi

cd "$xdg"
exec "$NVIM_PRG" -ll "$root/test/runner.lua" -v \
  --helper="$root/test/$test_type/preload.lua" \
  --lpath="$root/src/?.lua" \
  --lpath="$root/runtime/lua/?.lua" \
  --lpath='?.lua' \
  "${args[@]}" \
  "${paths[@]}"
