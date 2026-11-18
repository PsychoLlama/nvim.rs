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

# Before anything moves XDG_STATE_HOME: the user's own log, which no test
# child may write to (checked when the run ends).
source "$root/scripts/user-log-guard.sh"
user_log_mark

test_type=${1:-}
case "$test_type" in
  functional|unit|benchmark) shift ;;
  *)
    echo "usage: $0 <functional|unit|benchmark> [harness args...] [spec paths...]" >&2
    exit 64
    ;;
esac

# The build dir (upstream's $BUILD_DIR: bin/nvim, runtime/, lib/nvim) and
# the run root (upstream's $BUILD_DIR scratch: XDG sandbox, TMPDIR, logs).
# Each suite gets its own run root by default, so different suites run
# beside each other; two runs of one suite need distinct NVIM_TEST_RUN_DIRs.
# The ASan recipes set NVIM_TEST_BUILD_DIR, which moves both.
build=$(realpath -m "${NVIM_TEST_BUILD_DIR:-$root/target}")
export NVIM_TEST_BUILD_DIR=$build
run_dir=$(realpath -m "${NVIM_TEST_RUN_DIR:-$build/testrun/$test_type}")

# Tests run the bin/nvim link, not target/debug/nvim directly: specs like
# fs_spec assert nvim lives at $BUILD_DIR/bin/nvim, and $NVIM_BIN swaps in
# other builds (ASan recipes). Prep (which makes the link and the fixtures)
# comes before the -x check; NVIM_TEST_SKIP_PREP=1 skips it for a caller that
# already ran `prep-test-tree.sh <suite>`.
export NVIM_PRG=${NVIM_PRG:-$build/bin/nvim}
if [[ ${NVIM_TEST_SKIP_PREP:-} != 1 ]]; then
  "$root/scripts/prep-test-tree.sh" "$test_type"
fi
if [[ ! -x $NVIM_PRG ]]; then
  echo "$0: $NVIM_PRG not found; run \`just build\` first" >&2
  exit 66
fi

# Start each run from a fresh sandbox -- tests deliberately break it (bad
# tempdir perms, stale sockets) -- but leave it in place afterwards for
# post-mortems. (Upstream instead deleted it after every run.)
xdg=$run_dir/Xtest_xdg
# TMPDIR must be SHORT: nvim puts its server sockets under $TMPDIR when
# $XDG_RUNTIME_DIR is unset (the env of a child started with its own env),
# and unix sockets cap the path at ~107 bytes (sun_path). libuv silently
# truncates longer paths, which then collide ("--listen: address already in
# use"). A repo-relative Xtest_tmpdir exceeds the cap; upstream CI paths only
# pass because they are shorter. Keyed by the run root, so each run has its
# own.
tmpdir=${XDG_RUNTIME_DIR:-/tmp}/nvim.rs-Xtest-$(cksum <<<"$run_dir" | cut -d' ' -f1)
# The marker is what makes the root ours to delete: a mistyped
# NVIM_TEST_RUN_DIR naming a real directory is refused, not emptied.
if [[ -e $run_dir && ! -e $run_dir/.nvim-test-run ]]; then
  echo "$0: $run_dir exists and is not a test run root; refusing to delete it" >&2
  exit 64
fi
chmod -R u+rwX "$run_dir" "$tmpdir" 2>/dev/null || true
rm -rf "$run_dir" "$tmpdir"
mkdir -p "$xdg" "$tmpdir" "$run_dir/home"
touch "$run_dir/.nvim-test-run"

# The harness runs from inside the XDG sandbox; these links make repo-relative
# spec paths (test/…) and runtime lookups resolve from there.
ln -sfn "$root/runtime" "$xdg/runtime"
ln -sfn "$root/crates/nvim/src" "$xdg/src"
ln -sfn "$root/test" "$xdg/test"
ln -sfn "$root/README.md" "$xdg/README.md"

export NVIM_TEST=1
export LC_ALL=en_US.UTF-8
export VIMRUNTIME=$root/runtime
export XDG_CONFIG_HOME=$xdg/config
export XDG_DATA_HOME=$xdg/share
export XDG_STATE_HOME=$xdg/state
export XDG_CACHE_HOME=$xdg/cache
export XDG_RUNTIME_DIR=$tmpdir
# Children started with an env of their own get HOME passed through
# (testnvim.lua) and derive their XDG dirs from it: keep that in the run too.
export HOME=$run_dir/home
export NVIM_RPLUGIN_MANIFEST=$run_dir/Xtest_rplugin_manifest
unset XDG_DATA_DIRS NVIM TMUX
export NVIM_LOG_FILE=${NVIM_LOG_FILE:-$run_dir/nvim.log}
export VTERM_TEST_FILE=$run_dir/vterm_test_output
export NVIM_TEST_TIMINGS=$run_dir/timings.rows
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
status=0
"$NVIM_PRG" -ll "$root/test/runner.lua" -v \
  --helper="$root/test/$test_type/preload.lua" \
  --lpath="$root/crates/nvim/src/?.lua" \
  --lpath="$root/runtime/lua/?.lua" \
  --lpath='?.lua' \
  "${args[@]}" \
  "${paths[@]}" || status=$?

# The run's timing table, folded into the build dir's stable one.
source "$root/scripts/test-timings.sh"
if [[ -s $NVIM_TEST_TIMINGS ]]; then
  timings_sort "$NVIM_TEST_TIMINGS" >"$run_dir/timings.tsv"
  timings_fold "$run_dir/timings.tsv" "$build/timings/$test_type.tsv"
fi
user_log_check || ((status)) || status=1
exit "$status"
