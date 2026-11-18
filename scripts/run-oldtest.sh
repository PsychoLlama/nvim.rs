#!/usr/bin/env bash
# Run the legacy Vim test suite (test/old/testdir) against the cargo build.
#
# Usage: run-oldtest.sh [--jobs=N] <all|test_name...|clean>
#
#   all         the whole suite, always from scratch
#   <names...>  only the named tests, e.g. test_arglist
#   clean       delete the run's root, run nothing
#   --jobs=N    (or NVIM_TEST_JOBS=N; opt-in, default 1 = serial) run the
#               test files N at a time with GNU parallel, each worker in a
#               root of its own, longest first by
#               <build dir>/timings/oldtest.tsv; one merged report
#
# $NVIM_PRG selects the binary under test (default target/debug/nvim); the ASan
# recipes point it at their own build. Every run executes in a fresh root,
# $NVIM_OLDTEST_RUN_DIR (default target/oldtest; see below), and writes a
# per-file timing table there (timings.tsv) and to
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

# The test files a split run keeps off the parallel part: they run one by
# one after it, each with the reason it cannot share the machine. A test that
# fails only under load is fixed first (wait on a condition, not a clock);
# this list is for what cannot be.
SERIAL_TAIL=(
  "test_search_stat: 'n' counts matches for its [N/M] under a fixed 40 ms clock (SEARCH_STAT_DEF_TIMEOUT) a test cannot widen; an ASan build beside eleven workers misses it"
)

jobs=${NVIM_TEST_JOBS:-1}
names=()
for arg in "$@"; do
  if [[ $arg == --jobs=* ]]; then
    jobs=${arg#--jobs=}
  else
    names+=("$arg")
  fi
done
set -- "${names[@]}"
if [[ $# -eq 0 ]]; then
  echo "usage: $0 [--jobs=N] <all|test_name...|clean>" >&2
  exit 64
fi
if [[ ! $jobs =~ ^[1-9][0-9]*$ ]]; then
  echo "$0: --jobs/NVIM_TEST_JOBS wants a worker count, got '$jobs'" >&2
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
# copy of it: every test deletes X* in its cwd before it starts and appends
# to a shared `messages`, so two runs in one directory wreck each other.
#
# The root is a checkout of its own, as far as the tests can see (make_root
# below): the tests reach the tree as ../../../runtime and ../../../crates,
# runnvim.sh makes ../../.. their XDG_CONFIG_HOME and $VIMRUNTIME, and
# setup.vim puts swap files beside the file edited ('directory' ^= .), so a
# test that opens a help file writes `.help.txt.swp` into that runtime's
# doc/. Against the real checkout that wrote into the repo, and two workers
# opening one help file at once would meet each other's swap file (E325).
run_dir=$(realpath -m "${NVIM_OLDTEST_RUN_DIR:-$root/target/oldtest}")
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

if ((jobs > 1)); then
  for arg in "$@"; do
    case $arg in
      nolog | report | newtests | newtestssilent | clean)
        echo "$0: --jobs runs \`all\` or named tests, not \`$arg\`" >&2
        exit 64
        ;;
    esac
  done
fi

# The marker makes the root ours to delete: a mistyped NVIM_OLDTEST_RUN_DIR
# naming a real directory is refused, not emptied (anything under target/ is
# build output already).
if [[ -e $run_dir && ! -e $run_dir/.nvim-oldtest-run && $run_dir != "$root"/target/* ]]; then
  echo "$0: $run_dir exists and is not an oldtest run root; refusing to delete it" >&2
  exit 64
fi
chmod -R u+rwX "$run_dir" 2>/dev/null || true
rm -rf "$run_dir"
if [[ $# -eq 1 && $1 == clean ]]; then
  exit 0
fi

build=$(dirname "$(dirname "$NVIM_PRG")")

# make_root <dir>: <dir> as a checkout the tests cannot tell from the real one:
#
#   test/old/   what the checkout holds under test/old (tracked and
#               untracked-but-not-ignored, so a test being written is
#               included), never the ignored leftovers of an in-place run: a
#               stale test_*.res would make make skip its test. All of
#               test/old, not just the testdir: test_syntax views
#               ../memfile_test.c.
#   runtime/    a copy of runtime/ (12 MB, ~0.1 s), so swap files land here
#   crates      a link to crates/ (gen_opt_test.vim, test_cmdmods read it)
#   build/      the build dir the tests see as $BUILD_DIR: bin/nvim and
#               lib/nvim link to <build dir>'s, runtime/ holds its generated
#               syntax/ and doc/ -> this root's runtime/doc (test_help's
#               :help reaches the docs through it)
#   xdg/        XDG_STATE_HOME/XDG_DATA_HOME/XDG_CACHE_HOME (runnvim.sh only
#               moves the config home): 'directory' falls back to the state
#               home's swap/, which was the user's own
make_root() {
  local dir=$1
  mkdir -p "$dir/test/old" "$dir/build/bin" "$dir/build/runtime" "$dir/build/lib" \
    "$dir/xdg/state" "$dir/xdg/data" "$dir/xdg/cache"
  touch "$dir/.nvim-oldtest-run"
  (cd "$root/test/old" && git ls-files -z --cached --others --exclude-standard) |
    tar -C "$root/test/old" --null --no-recursion -T - -cf - | tar -C "$dir/test/old" -xf -
  cp -a "$root/runtime" "$dir/runtime"
  ln -s "$root/crates" "$dir/crates"
  ln -s "$(realpath "$NVIM_PRG")" "$dir/build/bin/nvim"
  ln -s "$build/lib/nvim" "$dir/build/lib/nvim"
  ln -s ../../runtime/doc "$dir/build/runtime/doc"
  if [[ -d $build/runtime/syntax ]]; then
    cp -a "$build/runtime/syntax" "$dir/build/runtime/syntax"
  fi
}

# root_env <dir>: the environment a run in <dir> needs on top of make's.
root_env() {
  export XDG_STATE_HOME=$1/xdg/state XDG_DATA_HOME=$1/xdg/data XDG_CACHE_HOME=$1/xdg/cache
  export NVIM_OLDTEST_BUILD_DIR=$1/build
}

make_root "$run_dir"
run_testdir=$run_dir/test/old/testdir
make=(make -C "$run_testdir" NVIM_PRG="$NVIM_PRG")
# The generated inputs, once, before any test (a split run copies the
# prepared root).
"${make[@]}" --no-print-directory opt_test.vim .gdbinit >/dev/null

# One test file of a split run, in worker slot <slot>'s root (reused test
# after test, as the serial run reuses its one testdir). Runs in GNU
# parallel's shell (exported). Keeps the test's `messages` (and `test.log`
# when it failed) for the merged report, and appends
# "<result> <secs> <test>" to results.tsv.
oldtest_job() { # <slot> <test>
  local slot=$1 test=$2
  local dir=$OLDTEST_DIR/w$slot
  local testdir=$dir/test/old/testdir log=$OLDTEST_DIR/logs/$test.log
  local start=$EPOCHREALTIME status=0 result=ok
  (
    root_env "$dir"
    make -C "$testdir" --no-print-directory NVIM_PRG="$NVIM_PRG" "$test"
  ) >"$log" 2>&1 </dev/null || status=$?
  [[ -f $testdir/messages ]] && cp "$testdir/messages" "$OLDTEST_DIR/messages.d/$test"
  if ((status)); then
    result=FAIL
    if [[ -f $testdir/test.log ]]; then
      cp "$testdir/test.log" "$OLDTEST_DIR/testlog.d/$test"
    else
      echo "Test $test failed: make exited $status, see $log" >"$OLDTEST_DIR/testlog.d/$test"
    fi
  fi
  local secs
  secs=$(awk -v a="$start" -v b="$EPOCHREALTIME" 'BEGIN { printf "%.1f", b - a }')
  printf '%s\t%s\t%s\n' "$result" "$secs" "$test" >>"$OLDTEST_DIR/results.tsv"
  printf '%-4s %6ss  %s\n' "$result" "$secs" "$test"
}

# The split run: the test files N at a time, longest first (a file the
# timing table does not know counts as the longest), then upstream's own
# report (`make report`: summarize.vim over every test's `messages`, the
# failures from every failing test's `test.log`) in the run root's testdir.
run_split() {
  local start=$SECONDS slot test tail_test status=0
  local -a tests=()
  if [[ $# -eq 1 && $1 == all ]]; then
    mapfile -t tests < <("${make[@]}" -s --no-print-directory list-tests)
  else
    tests=("${@%.res}")
  fi
  for ((slot = 1; slot <= jobs; slot++)); do
    mkdir -p "$run_dir/w$slot"
    touch "$run_dir/w$slot/.nvim-oldtest-run"
    cp -a "$run_dir/test" "$run_dir/runtime" "$run_dir/crates" "$run_dir/build" "$run_dir/xdg" \
      "$run_dir/w$slot/"
  done
  mkdir -p "$run_dir/logs" "$run_dir/messages.d" "$run_dir/testlog.d"
  : >"$run_dir/results.tsv"
  local -a parallel_tests=() tail_tests=()
  for test in "${tests[@]}"; do
    for tail_test in "${SERIAL_TAIL[@]%%:*}"; do
      if [[ $test == "$tail_test" ]]; then
        tail_tests+=("$test")
        continue 2
      fi
    done
    parallel_tests+=("$test")
  done
  echo "${#parallel_tests[@]} test files on $jobs workers, then ${#tail_tests[@]} serially; logs in $run_dir/logs"

  export OLDTEST_DIR=$run_dir NVIM_PRG
  export -f oldtest_job root_env
  ((${#parallel_tests[@]})) && printf '%s\n' "${parallel_tests[@]}" | awk -v table="$build/timings/oldtest.tsv" '
    BEGIN { while ((getline line < table) > 0) { split(line, f, "\t"); secs[f[3]] = f[1] } }
    { file = $0 ".vim"; print ((file in secs) ? secs[file] : 1e9) "\t" $0 }' |
    sort -t$'\t' -k1,1gr -k2,2 | cut -f2 |
    SHELL=$(command -v bash) parallel --will-cite -j "$jobs" oldtest_job '{%}' '{}' || true
  for test in "${tail_tests[@]}"; do
    oldtest_job 1 "$test"
  done

  # The merged inputs of the report, in the serial run's order.
  : >"$run_testdir/messages"
  rm -f "$run_testdir/test.log"
  for test in "${tests[@]}"; do
    [[ -f $run_dir/messages.d/$test ]] && cat "$run_dir/messages.d/$test" >>"$run_testdir/messages"
    [[ -f $run_dir/testlog.d/$test ]] && cat "$run_dir/testlog.d/$test" >>"$run_testdir/test.log"
  done
  "${make[@]}" --no-print-directory report || status=1
  awk -F'\t' -v dir="$run_dir" -v jobs="$jobs" -v secs=$((SECONDS - start)) '
    { files++; if ($1 != "ok") { bad++; report = report sprintf("FAILED  %s\n    log:  %s/logs/%s.log\n", $3, dir, $3) } }
    END {
      printf "\n======== %d test files on %d workers in %d s\n", files, jobs, secs
      if (bad) { printf "%d test files failed:\n%s", bad, report; exit 1 }
    }' "$run_dir/results.tsv" || status=1
  if (($(wc -l <"$run_dir/results.tsv") != ${#tests[@]})); then
    echo "$0: $(wc -l <"$run_dir/results.tsv") of ${#tests[@]} test files reported a result" >&2
    status=1
  fi
  return "$status"
}

status=0
log=$run_dir/make.log
if ((jobs > 1)); then
  run_split "$@" 2>&1 | tee "$log" || status=$?
  timing_source=$run_testdir/messages
else
  root_env "$run_dir"
  if [[ $# -eq 1 && $1 == all ]]; then
    "${make[@]}" 2>&1 | tee "$log" || status=$?
    timing_source=$run_testdir/messages
  else
    # A named target deletes `messages` before its test and prints it after,
    # so only make's output holds every named test's lines.
    "${make[@]}" "$@" 2>&1 | tee "$log" || status=$?
    timing_source=$log
  fi
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
    timings_fold "$run_dir/timings.tsv" "$build/timings/oldtest.tsv"
  fi
fi
user_log_check || ((status)) || status=1
exit "$status"
