#!/usr/bin/env bash
# Run the legacy Vim test suite (test/old/testdir) against the cargo build.
#
# Usage: run-oldtest.sh <all|test_name...|clean>
#
#   all         the whole suite, always from scratch
#   <names...>  only the named tests, e.g. test_arglist
#   clean       delete test artifacts, run nothing
#
# $NVIM_PRG selects the binary under test (default target/debug/nvim); the ASan
# recipes point it at their own build.
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
# `all` therefore cleans first rather than exposing the incrementality. Resuming
# has no value here: the summary can only describe the tests that ran in the
# current pass, so a partial run cannot produce a trustworthy full-suite result
# no matter how much work it skips. Named tests are unaffected either way, since
# their rule removes its own .res before rebuilding it.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)

if [[ $# -eq 0 ]]; then
  echo "usage: $0 <all|test_name...|clean>" >&2
  exit 64
fi

# Lays out target/ the way the suite expects: $BUILD_DIR/runtime and
# $BUILD_DIR/lib/nvim, both added to &runtimepath by runtest.vim. runnvim.sh
# derives $BUILD_DIR from $NVIM_PRG as its parent's parent.
"$root/scripts/prep-test-tree.sh"

NVIM_PRG=${NVIM_PRG:-$root/target/debug/nvim}
if [[ ! -x $NVIM_PRG ]]; then
  echo "$0: $NVIM_PRG not found; run \`just build\` first" >&2
  exit 66
fi

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

make=(make -C "$testdir" NVIM_PRG="$NVIM_PRG")
if [[ $# -eq 1 && $1 == all ]]; then
  "${make[@]}" clean
  exec "${make[@]}"
fi
exec "${make[@]}" "$@"
