#!/usr/bin/env bash
# Run the legacy Vim test suite (test/old/testdir) against the cargo build.
#
# Usage: run-oldtest.sh <all|test_name...|clean>
#
#   all         the whole suite, always from scratch
#   <names...>  only the named tests, e.g. test_arglist
#   clean       delete the run's copy of the testdir, run nothing
#
# $NVIM_PRG selects the binary under test (default target/debug/nvim); the ASan
# recipes point it at their own build. Every run executes in a fresh copy of
# the testdir under $NVIM_OLDTEST_RUN_DIR (default target/oldtest; see below),
# and writes a per-file timing table there (timings.tsv) and to
# <build dir>/timings/oldtest.tsv.
#
# The mode is mandatory because the obvious invocation — no arguments, meaning
# "run everything" — could report a green suite while executing nothing. make's
# default target deletes `messages` up front, each test appends its results to
# it as it runs, and the summary is computed from that file at the end. Every
# test memoizes its output as test_*.res, so on a tree that has already run the
# suite make considers all of it up to date, skips it, and summarizes an empty
# `messages`:
#
#     Executed:     0 Tests
#      Skipped:     0 Tests
#       Failed:     0 Tests
#     ALL DONE
#
# Exit code 0, indistinguishable from a real pass, and that is the steady state
# — any tree that has run the suite once is warm.
#
# Every run therefore starts from a fresh copy rather than exposing the
# incrementality. Resuming
# has no value here: the summary can only describe the tests that ran in the
# current pass, so a partial run cannot produce a trustworthy full-suite result
# no matter how much work it skips. Named tests are unaffected either way, since
# their rule removes its own .res before rebuilding it.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)

# The user's own log, which no test child may write to (checked at the end).
source "$root/scripts/user-log-guard.sh"
user_log_mark

if [[ $# -eq 0 ]]; then
  echo "usage: $0 <all|test_name...|clean>" >&2
  exit 64
fi

# Lays out the build dir the way the suite expects: $BUILD_DIR/runtime and
# $BUILD_DIR/lib/nvim, both added to &runtimepath by runtest.vim. runnvim.sh
# derives $BUILD_DIR from $NVIM_PRG as its parent's parent.
if [[ ${NVIM_TEST_SKIP_PREP:-} != 1 ]]; then
  "$root/scripts/prep-test-tree.sh" oldtest
fi

NVIM_PRG=${NVIM_PRG:-$root/target/debug/nvim}
if [[ ! -x $NVIM_PRG ]]; then
  echo "$0: $NVIM_PRG not found; run \`just build\` first" >&2
  exit 66
fi

# The run root. The suite does not run in test/old/testdir but in a fresh
# copy of it, <run root>/testdir: every test deletes X* in its cwd before it
# starts and appends to a shared `messages`, so two runs in one directory
# wreck each other. The tests reach the tree as ../../../runtime (and
# ../../../crates), so the root sits exactly two levels below the repo root:
# target/oldtest by default, target/<anything> for another run beside it.
run_dir=$(realpath -m "${NVIM_OLDTEST_RUN_DIR:-$root/target/oldtest}")
if [[ $(realpath -m "$run_dir/../..") != "$root" ]]; then
  echo "$0: NVIM_OLDTEST_RUN_DIR=$run_dir must sit two levels below $root" >&2
  echo "  (e.g. $root/target/oldtest-2): the tests reach the runtime as ../../../runtime" >&2
  exit 64
fi

# The log path is fixed when Nvim starts; setup.vim's `let $NVIM_LOG_FILE`
# comes too late. Without this a debug build logs every test's DBG lines to
# the user's own $XDG_STATE_HOME/nvim/nvim.log.
export NVIM_LOG_FILE=${NVIM_LOG_FILE:-$run_dir/nvim.log}

testdir=$root/test/old/testdir

# Reject an unknown test name before running anything.
#
# make walks its goals left to right, so a typo in the middle of a list runs
# every name before it and *then* dies with "No rule to make target" — a
# fifteen-minute run that ends in an error and has to be repeated. The suite's
# names are exactly its `test_*.vim` files, so the check is a file test.
bad=()
for arg in "$@"; do
  case $arg in
    all | clean | nolog | report | newtests | newtestssilent) continue ;;
  esac
  [[ -f $testdir/${arg%.res}.vim ]] || bad+=("$arg")
done
if (( ${#bad[@]} )); then
  for arg in "${bad[@]}"; do
    echo "$0: no such test: $arg" >&2
    # A name that is nearly right is nearly always a prefix of one that
    # exists (test_jumps for test_jumplist), so shorten it until something
    # matches and report that.
    stem=$arg
    near=
    while [[ ${#stem} -gt 5 && -z $near ]]; do
      near=$(cd "$testdir" && ls "$stem"*.vim 2>/dev/null | sed 's/\.vim$//' || true)
      stem=${stem%?}
    done
    [[ -n $near ]] && echo "  did you mean: $(echo "$near" | tr '\n' ' ')" >&2
  done
  echo "$0: nothing ran; \`ls $testdir/test_*.vim\` lists the suite" >&2
  exit 64
fi

run_testdir=$run_dir/testdir
rm -rf "$run_testdir"
if [[ $# -eq 1 && $1 == clean ]]; then
  exit 0
fi

# Copy what the checkout holds under test/old (tracked and
# untracked-but-not-ignored, so a test being written is included), never the
# ignored leftovers of an in-place run: a stale test_*.res would make make
# skip its test. The run root mirrors test/old, not just the testdir:
# test_syntax views ../memfile_test.c.
(cd "$root/test/old" && git ls-files -z --cached --others --exclude-standard) |
  tar -C "$root/test/old" --null --no-recursion -T - -cf - | tar -C "$run_dir" -xf -

make=(make -C "$run_testdir" NVIM_PRG="$NVIM_PRG")
# The generated inputs, once, before any test (a split run copies the
# prepared directory).
"${make[@]}" --no-print-directory opt_test.vim .gdbinit >/dev/null

status=0
log=$run_dir/make.log
if [[ $# -eq 1 && $1 == all ]]; then
  "${make[@]}" 2>&1 | tee "$log" || status=$?
  timing_source=$run_testdir/messages
else
  # A named target deletes `messages` before its test and prints it after,
  # so only make's output holds every named test's lines.
  "${make[@]}" "$@" 2>&1 | tee "$log" || status=$?
  timing_source=$log
fi

# The run's timing table (per test file: seconds, tests), folded into the
# build dir's stable one. From the lines runtest.vim already writes:
#   From test_x.vim:
#   Executed 26 tests                        in   0.677101 seconds
source "$root/scripts/test-timings.sh"
if [[ -f $timing_source ]]; then
  sed 's/\x1b\[[0-9;]*m//g' "$timing_source" | awk '
    /^From test_.*\.vim:$/ { file = substr($2, 1, length($2) - 1); next }
    file != "" && /^Executed [0-9]+ tests? +in +[0-9.]+ seconds/ {
      printf "%s\t%s\t%s\n", $5, $2, file; file = ""
    }' >"$run_dir/timings.rows"
  if [[ -s $run_dir/timings.rows ]]; then
    timings_sort "$run_dir/timings.rows" >"$run_dir/timings.tsv"
    timings_fold "$run_dir/timings.tsv" "$(dirname "$(dirname "$NVIM_PRG")")/timings/oldtest.tsv"
  fi
fi
user_log_check || ((status)) || status=1
exit "$status"
