#!/usr/bin/env bash
# Build the working tree and diff its keysweep against the stored B13
# baseline.  Run from anywhere; everything is absolute.
#
#   keyverify.sh [label]          # default label: cur
#
# The baseline lives next to this script in keybase/ and was
# produced from commit 3b60d34f69 -- the pre-batch tree, i.e. B13's P0
# close, before any of the input modules is rewritten.  Regenerate it
# only when a behaviour change is *intended* and reviewed:
#
#   keysweep.sh <nvim> <runtime> \
#       test/battery/keybase base
#
# The sweep's own work directory is a fixed short path (/tmp/ksweep):
# absolute paths reach `:map <buffer>` listings and the fixture names,
# and messages are truncated to 80 columns, so two runs from different
# work directories compare as all-different.
#
# All three artifacts are compared, stderr included: nvim's own messages
# go to the prompt, which in a headless process is stderr, and for
# "recording @q", the mode messages and several error texts that is the
# only view of them.  The report's final `exit N` line is the hang
# assertion -- 124 is the harness timeout, and this subsystem's
# characteristic regression is a hang, not a wrong answer.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
BASELINE=${KEY_BASELINE:-$HERE/keybase}
OUT=${SWEEP_OUT:-/tmp/keysweep-out}
REPO=${REPO:-$(cd "$HERE/../.." && pwd)}
LABEL=${1:-cur}
LOG=$OUT/build-$LABEL.log

mkdir -p "$OUT"
cd "$REPO"
if ! just build >"$LOG" 2>&1; then
  echo "BUILD FAILED -- see $LOG" >&2
  grep -E '^(error|warning)' "$LOG" | head -60 >&2
  exit 1
fi

rm -f "$OUT/$LABEL.txt" "$OUT/$LABEL.struct" "$OUT/$LABEL.stderr"
"$HERE/keysweep.sh" "$REPO/target/debug/nvim" "$REPO/runtime" \
  "$OUT" "$LABEL" 2>&1 | tail -1

fail=0
for part in txt struct stderr; do
  if diff -q "$BASELINE/base.$part" "$OUT/$LABEL.$part" >/dev/null; then
    echo "$part: IDENTICAL"
  else
    echo "$part: DIFFERS"
    # -a: the reports carry escaped latin1 bytes, and diff would
    # otherwise call them binary and print nothing useful.
    # `|| true`: `set -e` plus `pipefail` would otherwise abort on the
    # first differing artifact, and the remaining ones -- the ones that
    # say *which* layer moved -- would never be compared.
    diff -a "$BASELINE/base.$part" "$OUT/$LABEL.$part" | head -60 || true
    fail=1
  fi
done
exit $fail
