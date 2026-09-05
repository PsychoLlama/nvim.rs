#!/usr/bin/env bash
# Build the working tree and diff its spellsweep against the stored
# baseline.  Run from anywhere; everything is absolute.
#
#   spellverify.sh [label]        # default label: cur
#
# The baseline lives next to this script in spellbase/ and was
# produced at commit 47e44ab7c6 (the phase-18 close, i.e. phase 19's base)
# by the P19 close, after the original baseline was lost with the dead
# session scratchpad this script used to hardcode.  Regenerate it only
# when a behaviour change is *intended* and reviewed:
#
#   SPELLSWEEP_WORK=/tmp/spellsweep-verify \
#     spellsweep.sh <nvim> <runtime> \
#       test/battery/spellbase base
#
# The sweep reuses one work directory path for every run because the
# generated .spl/.sug bytes and the report both record it.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
BASELINE=${SPELL_BASELINE:-$HERE/spellbase}
OUT=${SWEEP_OUT:-/tmp/spellsweep-out}
REPO=${REPO:-$(cd "$HERE/../.." && pwd)}
LABEL=${1:-cur}

mkdir -p "$OUT"
cd "$REPO"
if ! just build >"$OUT/build-$LABEL.log" 2>&1; then
  echo "BUILD FAILED -- see $OUT/build-$LABEL.log" >&2
  grep -E '^(error|warning)' "$OUT/build-$LABEL.log" | head -60 >&2
  exit 1
fi

rm -rf "$OUT/$LABEL.txt" "$OUT/$LABEL.hashes" "$OUT/$LABEL-files"
SPELLSWEEP_WORK=${SPELLSWEEP_WORK:-/tmp/spellsweep-verify} \
  "$HERE/spellsweep.sh" "$REPO/target/debug/nvim" \
  "$REPO/runtime" "$OUT" "$LABEL" 2>&1 | tail -2

fail=0
if diff -q "$BASELINE/base.txt" "$OUT/$LABEL.txt" >/dev/null; then
  echo "report: IDENTICAL"
else
  echo "report: DIFFERS"
  diff "$BASELINE/base.txt" "$OUT/$LABEL.txt" | head -60
  fail=1
fi
if diff -q "$BASELINE/base.hashes" "$OUT/$LABEL.hashes" >/dev/null; then
  echo "bytes: IDENTICAL"
else
  echo "bytes: DIFFERS"
  diff "$BASELINE/base.hashes" "$OUT/$LABEL.hashes" | head -40
  fail=1
fi
exit $fail
