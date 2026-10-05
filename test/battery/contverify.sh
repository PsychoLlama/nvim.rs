#!/usr/bin/env bash
# Build the working tree and diff its container corpora against the cut
# baseline.  Run from anywhere; everything is absolute.
#
#   contverify.sh [label]         # default label: cur
#
# The interpreter's CONTAINERS: lists, dictionaries and blobs, as the user
# sees them.  Three corpora, each a plain Vimscript file run in its own
# headless nvim that writes one line per observation to $DIFFOUT:
#
#   contlist.vim  list sharing and copies (shallow, deep, the deepcopy memo,
#                 cycles), what a callee keeps, the `:for` cursor over a list
#                 that is edited mid-loop, locks, `remove()`/`add()` refusing
#                 a locked list, slices and their assignment, and the
#                 collector reached from the top level and from a function.
#   contdict.vim  dictionary SLOT ORDER (`keys()`/`values()`/`items()` over
#                 every growth boundary of the table and over several key
#                 shapes) -- user-visible, so it is a contract -- plus
#                 ownership, watchers, locks and the collector.
#   contblob.vim  blob literals, sharing, ranges and their assignment,
#                 conversions to and from lists and strings, and the error
#                 arm of every operation.
#
# Each corpus began as a hand-run comparison script for a rewrite of one
# container (the list and dict rewrites, then the blob's `Vec`); each was
# run against a recorded output of the build before.  Here the other side
# is the binary `test/battery/BASE` pins, like every row, so they gate every
# rewrite of the containers from now on.
#
# Artifacts, per corpus: base.<corpus> (the report) and base.<corpus>.stderr
# (what reached the prompt: a corpus that lets an error escape a function
# shows its `Error in command line..script <HERE>/...` trace there, with the
# script directory masked as <HERE>).  Each stderr ends with the process's
# `exit N`: 124 is the harness timeout and 134 an abort.
#
# Deterministic by construction -- no time, no pid, no path, no address
# reaches either artifact -- and checked: three runs of one binary and a
# run from a second working directory were byte-identical.  It costs ~1 s.
set -uo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
OUT=${SWEEP_OUT:-/tmp/contsweep-out}
REPO=${REPO:-$(cd "$HERE/../.." && pwd)}
LABEL=${1:-cur}
LOG=$OUT/build-$LABEL.log
LIMIT=${CONT_TIMEOUT:-120}
# Cut mode.  `baseline.sh` re-enters this script as `--cut <nvim> <dir>` to
# cut the pinned baseline from the reference binary; it shares the ONE sweep
# loop below with the head run, so the two sides of the differential cannot
# drift apart.  It skips the build and the diff.
CUT=
if [[ ${1:-} == --cut ]]; then CUT=$2; OUT=$3; LABEL=base; LOG=/dev/null; fi

mkdir -p "$OUT"
cd "$REPO" || exit 1
if [[ -z $CUT ]] && ! just build >"$LOG" 2>&1; then
  echo "BUILD FAILED -- see $LOG" >&2
  grep -E '^(error|warning)' "$LOG" | head -60 >&2
  exit 1
fi
NVIM=${CUT:-$REPO/target/debug/nvim}

# A fixed, short work directory, as every row has: nothing here should
# reach a path, and if something ever does it is the same one on both sides.
WORK=/tmp/contsweep-work
rm -rf "$WORK"
mkdir -p "$WORK/home"
cd "$WORK" || exit 1
: >"$WORK/empty"

BASELINE=
if [ -z "$CUT" ]; then
  BASELINE=${CONT_BASELINE:-$("$HERE/baseline.sh" cont)}
fi

fail=0
for corpus in list dict blob; do
  out=$OUT/$LABEL.$corpus
  rm -f "$out" "$out.stderr"
  # `env -i`: no locale, editor configuration or XDG directory of the
  # caller's reaches the run.
  timeout -k 5 "$LIMIT" \
    env -i HOME="$WORK/home" PATH=/usr/bin:/bin TERM=dumb SHELL=/bin/sh \
    LANG=C.UTF-8 VIMRUNTIME="$REPO/runtime" DIFFOUT="$out" \
    "$NVIM" --headless -u NONE -i NONE -n --cmd 'set noswapfile' \
    -S "$HERE/cont$corpus.vim" <"$WORK/empty" >"$out.stderr.raw" 2>&1
  status=$?
  sed "s#$HERE#<HERE>#g" "$out.stderr.raw" >"$out.stderr"
  rm -f "$out.stderr.raw"
  printf '\nexit %d\n' "$status" >>"$out.stderr"
  touch "$out"
  if [ -n "$CUT" ]; then
    echo "$corpus: CUT ($(wc -l <"$out") lines, exit $status)"
    continue
  fi
  for part in "$corpus" "$corpus.stderr"; do
    if diff -q "$BASELINE/base.$part" "$OUT/$LABEL.$part" >/dev/null 2>&1; then
      echo "$part: IDENTICAL ($(wc -l <"$OUT/$LABEL.$part") lines)"
    else
      echo "$part: DIFFERS"
      # -a: a report may carry high bytes, and diff would otherwise call
      # it binary and print nothing useful.
      diff -a "$BASELINE/base.$part" "$OUT/$LABEL.$part" | head -40
      fail=1
    fi
  done
done
exit $fail
