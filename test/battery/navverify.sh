#!/usr/bin/env bash
# Build the working tree and diff its navsweep against the stored B11
# baseline.  Run from anywhere; everything is absolute.
#
#   navverify.sh [label]         # default label: cur
#
# The baseline lives next to this script in navbase/ and was
# produced from commit d086fb238f (the B11-1 mechanical shrink, itself
# proven identical to the pre-slice tree by an A/B against a 748f18d8a9
# binary).  Regenerate it only when a behaviour change is *intended* and
# reviewed:
#
#   navsweep.sh <nvim> <runtime> test/battery/navbase base
#
# The sweep's own work directory is a fixed path (/tmp/nsweep) because
# absolute paths land in quickfix entries, tag stacks and expanded names;
# two runs from different work directories compare as all-different.
#
# All three artifacts are compared, stderr included: the `:tselect`
# listing (print_tag_list) is written to the prompt, which in a headless
# process is stderr, and it is the only view of that function.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
BASELINE=${NAV_BASELINE:-$HERE/navbase}
OUT=${SWEEP_OUT:-/tmp/navsweep-out}
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
"$HERE/navsweep.sh" "$REPO/target/debug/nvim" "$REPO/runtime" \
  "$OUT" "$LABEL" 2>&1 | tail -1

fail=0
for part in txt struct stderr; do
  if diff -q "$BASELINE/base.$part" "$OUT/$LABEL.$part" >/dev/null; then
    echo "$part: IDENTICAL"
  else
    echo "$part: DIFFERS"
    # -a: the reports carry escaped latin1 bytes, and diff would
    # otherwise call them binary and print nothing useful.
    # `|| true`: `set -e` plus `pipefail` would otherwise abort the whole
    # script on the first differing artifact -- diff exits 1, and the
    # remaining artifacts (the ones that say *which* layer moved) would
    # never be compared.  persverify.sh has the same bug; fix it there too.
    diff -a "$BASELINE/base.$part" "$OUT/$LABEL.$part" | head -60 || true
    fail=1
  fi
done
exit $fail
