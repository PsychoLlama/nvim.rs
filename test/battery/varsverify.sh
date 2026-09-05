#!/usr/bin/env bash
# Build the working tree and diff its varsweep against the stored B14
# baseline.  Run from anywhere; everything is absolute.
#
#   varsverify.sh [label]         # default label: cur
#
# The baseline lives next to this script in varsbase/ and was
# last produced at commit ac4b0e92c1 -- P23-13's close, i.e. the commit
# P23-14 starts from.  It was re-cut there only because P23-14 APPENDED
# s19 (the reference-counting surface: dictwatcheradd/del, which no
# oracle reached at all, and what test_garbagecollect_now() leaves
# behind); every line of s1-s18 is byte-identical to the previous
# baseline, taken at 5d5aeaf4fa -- B14-14, after the eval/vars rewrite,
# after userfunc.rs was carved, and after the E884 fix (O-B14-12).
# Sections s13-s18 cover the :function layer.  Regenerate it
# only when a behaviour change is *intended* and reviewed:
#
#   varsweep.sh <nvim> <runtime> \
#       test/battery/varsbase base
#
# ... and `just build` first: a mutation harness leaves the binary built
# from its last mutant, and a baseline taken from that compares mutant
# against mutant forever after.
#
# All three artifacts are compared, stderr included: nvim's own messages
# go to the prompt, which in a headless process is stderr, and the
# `:let` listing's column alignment reaches only that artifact when a
# case escapes `execute()`.  The report's final `exit N` line is the
# hang/crash assertion -- 124 is the harness timeout and 134 an abort.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
BASELINE=${VARS_BASELINE:-$HERE/varsbase}
OUT=${SWEEP_OUT:-/tmp/varsweep-out}
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
"$HERE/varsweep.sh" "$REPO/target/debug/nvim" "$REPO/runtime" \
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
