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
#
# Sharding (functional only, opt-in): --jobs=N or NVIM_TEST_JOBS=N runs the
# spec files N at a time with GNU parallel, each in its own run root under
# this run's root, longest first by <build>/timings/functional.tsv; the
# SERIAL_TAIL specs below run one by one after the parallel part. Default 1
# is the plain serial run.
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

# The spec files a sharded run keeps off the parallel part: they run one by
# one after it, each with the reason it cannot share the machine. A test
# that fails only under load is fixed first (wait on a condition, not a
# clock); this list is for what cannot be.
SERIAL_TAIL=()

# One spec file of a sharded run, in worker slot <slot>'s root. Runs in GNU
# parallel's shell (exported); appends "<result> <secs> <spec>" to
# results.tsv, result being ok, FAIL, or none (no test matched the filter).
shard_job() { # <slot> <spec> [harness flags...]
  local slot=$1 spec=$2
  shift 2
  local name=${spec#test/functional/}
  name=${name//\//.}
  local log=$SHARD_DIR/logs/$name.log root=$SHARD_DIR/w$slot
  local start=$EPOCHREALTIME status=0 result=ok
  # A failed screen wait that leaves nvim at a prompt hangs the spec's
  # after_each for good (its `qall!` request waits behind the prompt): one
  # such spec once held a sharded run for half an hour. A spec gets
  # NVIM_TEST_SPEC_TIMEOUT seconds (default 900, three times the slowest
  # spec's ASan time under load), then its runner is killed and the spec fails.
  NVIM_TEST_JOBS=1 NVIM_TEST_SKIP_PREP=1 NVIM_TEST_RUN_DIR=$root TEST_COLORS=0 NVIM_TEST_SHARDED=1 \
    timeout -k 30 "${NVIM_TEST_SPEC_TIMEOUT:-900}" \
    "$SHARD_SELF" functional "$@" "$spec" >"$log" 2>&1 </dev/null || status=$?
  if ((status == 124 || status == 137)); then
    echo "run-tests.sh: $spec killed after NVIM_TEST_SPEC_TIMEOUT=${NVIM_TEST_SPEC_TIMEOUT:-900} s" >>"$log"
  fi
  if ((status)); then
    if grep -q '^No tests matched the current selection' "$log"; then
      result=none
    else
      result=FAIL
      mv "$root" "$SHARD_DIR/failed/$name"
    fi
  fi
  local secs
  secs=$(awk -v a="$start" -v b="$EPOCHREALTIME" 'BEGIN { printf "%.1f", b - a }')
  printf '%s\t%s\t%s\n' "$result" "$secs" "$spec" >>"$SHARD_DIR/results.tsv"
  [[ $result == none ]] || printf '%-4s %6ss  %s\n' "$result" "$secs" "$spec"
}

# The spec files under $paths, one per line, longest first by the timing
# table; a spec the table does not know counts as the longest.
shard_specs() {
  local p
  for p in "${paths[@]}"; do
    if [[ -d $root/$p ]]; then
      (cd "$root" && find "$p" -type f -name '*_spec.lua')
    else
      echo "$p"
    fi
  done | sort -u | awk -F'\t' -v table="$build/timings/$test_type.tsv" '
    BEGIN { while ((getline line < table) > 0) { split(line, f, "\t"); secs[f[3]] = f[1] } }
    { print (($0 in secs) ? secs[$0] : 1e9) "\t" $0 }' |
    sort -t$'\t' -k1,1gr -k2,2 | cut -f2
}

# The sharded run: the specs N at a time, then SERIAL_TAIL one by one, then
# one report over the per-spec logs.
run_shards() {
  local spec tail_spec start=$SECONDS
  local -a specs=() parallel_specs=() tail_specs=()
  mapfile -t specs < <(shard_specs)
  for spec in "${specs[@]}"; do
    for tail_spec in "${SERIAL_TAIL[@]%%:*}"; do
      [[ $spec == "$tail_spec" ]] && continue 2
    done
    parallel_specs+=("$spec")
  done
  for tail_spec in "${SERIAL_TAIL[@]%%:*}"; do
    for spec in "${specs[@]}"; do
      [[ $spec == "$tail_spec" ]] && tail_specs+=("$spec")
    done
  done
  echo "${#parallel_specs[@]} spec files on $jobs workers, then ${#tail_specs[@]} serially; logs in $run_dir/logs"

  export SHARD_DIR=$run_dir SHARD_SELF=$root/scripts/run-tests.sh
  export -f shard_job
  : >"$run_dir/results.tsv"
  if ((${#parallel_specs[@]})); then
    printf '%s\n' "${parallel_specs[@]}" |
      SHELL=$(command -v bash) parallel --will-cite -j "$jobs" -q \
        shard_job '{%}' '{}' "${args[@]}" || true
  fi
  for spec in "${tail_specs[@]}"; do
    shard_job tail "$spec" "${args[@]}"
  done

  # One summary over every spec's log (the reporter's own footer lines).
  local status=0
  awk -F'\t' -v dir="$run_dir" -v jobs="$jobs" -v secs=$((SECONDS - start)) '
    function count(lf,   line, in_summary) {
      in_summary = 0; summary = 0
      while ((getline line < lf) > 0) {
        if (line ~ /^========/) { in_summary = 1; summary = 1; continue }
        if (!in_summary) continue
        if (match(line, /^PASSED +[0-9]+/)) { split(line, w, / +/); passed += w[2] }
        else if (line ~ /^ [0-9]+ SKIPPED TEST/) { split(line, w, / +/); skipped += w[2] }
        else if (line ~ /^ [0-9]+ FAILED TEST/) { split(line, w, / +/); failed += w[2] }
        else if (line ~ /^ [0-9]+ ERROR/) { split(line, w, / +/); errors += w[2] }
        else if (line ~ /^(FAILED|ERROR) / && line !~ /listed below:$/) { lines[lf] = lines[lf] "    " line "\n" }
      }
      close(lf)
    }
    {
      result = $1; spec = $3; name = spec; sub(/^test\/functional\//, "", name); gsub(/\//, ".", name)
      lf = dir "/logs/" name ".log"
      files++
      if (result == "none") { none++; next }
      count(lf)
      if (result == "FAIL") {
        bad++
        if (!summary) { errors++; lines[lf] = "    no summary: the run died (see its log)\n" }
        report = report sprintf("FAILED  %s\n    log:  %s\n    root: %s/failed/%s\n%s", spec, lf, dir, name, lines[lf])
      }
    }
    END {
      printf "\n======== %d spec files on %d workers (%d matched nothing) in %d s\n", files, jobs, none, secs
      printf "PASSED  %d tests.\n", passed
      if (skipped) printf " %d SKIPPED TESTS\n", skipped
      if (failed) printf " %d FAILED TESTS\n", failed
      if (errors) printf " %d ERRORS\n", errors
      if (bad) printf "\n%d spec files failed:\n%s", bad, report
      exit (bad || passed + skipped + failed + errors == 0) ? 1 : 0
    }' "$run_dir/results.tsv" || status=1
  user_log_check || status=1
  return "$status"
}

# The build dir (upstream's $BUILD_DIR: bin/nvim, runtime/, lib/nvim) and
# the run root (upstream's $BUILD_DIR scratch: XDG sandbox, TMPDIR, logs).
# Each suite gets its own run root by default, so different suites run
# beside each other; two runs of one suite need distinct NVIM_TEST_RUN_DIRs.
# The ASan recipes set NVIM_TEST_BUILD_DIR, which moves both.
build=$(realpath -m "${NVIM_TEST_BUILD_DIR:-$root/target}")
export NVIM_TEST_BUILD_DIR=$build
run_dir=$(realpath -m "${NVIM_TEST_RUN_DIR:-$build/testrun/$test_type}")

# Split flags from spec paths; rewrite paths relative to the repo root so test
# ids stay stable ("test/functional/…") regardless of the caller's cwd.
jobs=${NVIM_TEST_JOBS:-1}
args=()
paths=()
for arg in "$@"; do
  if [[ $arg == --jobs=* ]]; then
    jobs=${arg#--jobs=}
  elif [[ $arg == -* ]]; then
    args+=("$arg")
  else
    abs=$(realpath "$arg")
    paths+=("${abs#"$root"/}")
  fi
done
if [[ ${#paths[@]} -eq 0 ]]; then
  paths=(test/$test_type)
fi
if [[ ! $jobs =~ ^[1-9][0-9]*$ ]]; then
  echo "$0: --jobs/NVIM_TEST_JOBS wants a worker count, got '$jobs'" >&2
  exit 64
fi
if ((jobs > 1)) && [[ $test_type != functional ]]; then
  echo "$0: only the functional suite shards" >&2
  exit 64
fi

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
# Sharded: this run's root holds one root per worker slot (w<slot>/, reused
# spec after spec), a log per spec (logs/), and the root of each spec that
# failed, moved aside before its slot is reused (failed/).
if ((jobs > 1)); then
  chmod -R u+rwX "$run_dir" 2>/dev/null || true
  rm -rf "$run_dir"
  mkdir -p "$run_dir/logs" "$run_dir/failed"
  touch "$run_dir/.nvim-test-run"
  status=0
  run_shards || status=$?
  exit "$status"
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
