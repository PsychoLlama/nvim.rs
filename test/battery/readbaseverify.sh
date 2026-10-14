#!/usr/bin/env bash
# Build the working tree and read a recorded set of on-disk goldens with
# it; diff what it decoded against what the recording binary decoded.
#
#   readbaseverify.sh <goldens> [label]     # default label: cur
#
# Unlike every other row, the baseline is NOT cut from `test/battery/BASE`:
# it is <goldens>/expected.txt, written when the goldens were recorded
# (`readbase.sh --record <nvim> <vimruntime> <goldens>`), by the same
# binary that wrote the artifacts.  The goldens are a recording, not a
# function of the tree, so they live outside it and the row takes their
# directory as an argument; `battery.sh <label> <goldens>` runs it, and
# without one the battery skips it.  Recording them once, from a binary
# taken before a change to the swap, ShaDa or undo code, is what lets
# every later binary prove it still reads the old bytes -- the `pers` and
# `undo` rows only read back what the binary under test itself wrote.
#
# Cost: ~2 s (42 short nvim processes).
#
# Environment:
#   SWEEP_OUT   where this run's report lands (default: /tmp/readbase-out)
#   NVIM        skip the build, use this binary instead
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
REPO=${REPO:-$(cd "$HERE/../.." && pwd)}
GOLD=${1:?usage: readbaseverify.sh <goldens> [label]}
LABEL=${2:-cur}
OUT=${SWEEP_OUT:-/tmp/readbase-out}
LOG=$OUT/build-$LABEL.log

if [[ ! -f $GOLD/expected.txt ]]; then
  echo "txt: NO BASELINE ($GOLD/expected.txt; record with readbase.sh --record)"
  exit 1
fi

mkdir -p "$OUT"
cd "$REPO"
if [[ -n ${NVIM:-} ]]; then
  BIN=$(realpath "$NVIM")
else
  if ! just build >"$LOG" 2>&1; then
    echo "BUILD FAILED -- see $LOG" >&2
    grep -E '^(error|warning)' "$LOG" | head -60 >&2
    exit 1
  fi
  BIN=$REPO/target/debug/nvim
fi

rm -f "$OUT/$LABEL.txt"
"$HERE/readbase.sh" "$BIN" "$REPO/runtime" "$GOLD" "$OUT" "$LABEL"

if diff -q "$GOLD/expected.txt" "$OUT/$LABEL.txt" >/dev/null; then
  echo "txt: IDENTICAL"
else
  echo "txt: DIFFERS"
  # -a: the report carries latin1 bytes.
  diff -a "$GOLD/expected.txt" "$OUT/$LABEL.txt" | head -60 || true
  exit 1
fi
