#!/usr/bin/env bash
# Build the working tree and diff its synsweep against the stored
# baseline.  Run from anywhere; everything is absolute.
#
#   synverify.sh [label]          # default label: cur
#
# The baseline is CUT, not committed: it comes from the binary
# `test/battery/BASE` pins, cached under target/battery/base/<sha>/.
#
# WHY IT EXISTS.  The `re` row drives both engines over a corpus written
# for them -- a few hundred patterns.  The patterns users actually run
# the engines on are the runtime's syntax files: ~35,000 `syn match` and
# region `start=`/`skip=`/`end=` patterns in 771 files, ~21,000 of them
# distinct, written over thirty years by people who knew which corners
# the engines had.  This row compiles every distinct one under
# 'regexpengine' 0, 1 and 2 and matches it against its own source line
# and a fixed probe, so a rewritten parser or matcher that answers one of
# them differently -- or refuses it with a different `E` code -- shows
# here before it shows as a mis-highlighted file.  It is its own row so
# `re` keeps its budget.
#
# THREE ARTIFACTS:
#
#   base.txt     one row per distinct pattern (`e012=` when the three
#                settings agree), then `syn/files`, `syn/errors`,
#                `syn/enginedelta` and `syn/deltalist`.  The delta count
#                is a fingerprint, not a target: the two engines refuse
#                some syntax-only atoms with different codes.
#   base.err     the error messages, one per pattern x engine.
#   base.stderr  EMPTY at the baseline, and that is the assertion.
#
# THE CORPUS MOVES WITH THE RUNTIME.  The patterns are read from the
# tree's `runtime/syntax` on both sides of the differential, so a runtime
# update re-cuts the row (the cache key covers test/battery only -- a
# changed syntax file shows as DIFFERS until the next cut, which is a
# runtime change to review, not an engine change).  ~30 s.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
OUT=${SWEEP_OUT:-/tmp/synsweep-out}
REPO=${REPO:-$(cd "$HERE/../.." && pwd)}
LABEL=${1:-cur}
LOG=$OUT/build-$LABEL.log
# Cut mode: `baseline.sh` re-enters this script as `--cut <nvim> <dir>`;
# it shares the one sweep call below with the head run.
CUT=
if [[ ${1:-} == --cut ]]; then CUT=$2; OUT=$3; LABEL=base; LOG=/dev/null; fi
NVIM_BIN=${CUT:-$REPO/target/debug/nvim}

mkdir -p "$OUT"
cd "$REPO"
if [[ -z $CUT ]] && ! just build >"$LOG" 2>&1; then
  echo "BUILD FAILED -- see $LOG" >&2
  grep -E '^(error|warning)' "$LOG" | head -60 >&2
  exit 1
fi

rm -f "$OUT/$LABEL.txt" "$OUT/$LABEL.err" "$OUT/$LABEL.stderr"
"$HERE/synsweep.sh" "$NVIM_BIN" "$REPO/runtime" \
  "$OUT" "$LABEL" 2>&1 | tail -1

if [[ -n $CUT ]]; then exit 0; fi

BASELINE=${SYN_BASELINE:-$("$HERE/baseline.sh" syn)}
fail=0
for part in txt err stderr; do
  if diff -q "$BASELINE/base.$part" "$OUT/$LABEL.$part" >/dev/null; then
    echo "$part: IDENTICAL"
  else
    echo "$part: DIFFERS"
    diff -a "$BASELINE/base.$part" "$OUT/$LABEL.$part" | head -60 || true
    fail=1
  fi
done
exit $fail
