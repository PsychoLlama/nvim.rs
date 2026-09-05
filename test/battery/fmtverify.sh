#!/usr/bin/env bash
# Build the working tree and diff its fmtsweep against the stored B15
# baseline.  Run from anywhere; everything is absolute.
#
#   fmtverify.sh [label]          # default label: cur
#
# The baseline lives next to this script in fmtbase/ and was
# produced at commit bd1d648849 -- B15-2, the P0 prelude, which is the
# rev every B15 oracle baselines against.  It is the *pre-rewrite*
# behaviour of indent_c.rs, textformat.rs and indent.rs: none of them has
# been touched yet, which is the whole point of standing this oracle up
# first.  Regenerate it only when a behaviour change is *intended* and
# reviewed:
#
#   fmtsweep.sh <nvim> <runtime> \
#       test/battery/fmtbase base
#
# ... and `just build` first: a mutation harness leaves the binary built
# from its last mutant, and a baseline taken from that compares mutant
# against mutant forever after.
#
# All three artifacts are compared, stderr included: nvim's own messages
# go to the prompt, which in a headless process is stderr, and s90 is the
# only place the E474/E487/E518/E524 arms and the 'equalprg'/'formatprg'
# shell-out failures are visible at all.  The report's final `exit N`
# line is the hang/crash assertion -- 124 is the harness timeout and 134
# an abort.
#
# s91 SPAWNS CHILDREN, and it used to expect eleven of them to die:
# `cinoptions` values outside int range aborted the process through
# `getdigits_int`'s `assert!` (charset.rs:586), in the RELEASE build as
# well as the debug one.  P0.3 (`0d483454cc`) made the strict arm clamp
# instead, so all eleven now answer a saturated indent profile and the
# baseline was regenerated with them.  An `"out":"ABORTED"` row appearing
# in s91 again is a regression, not a pass.
#
# THIS SWEEP RUNS NO EXTERNAL FILTER.  s90 points 'equalprg'/'formatprg'
# at a path that does not exist on purpose, because the *message* is the
# artifact; nothing here depends on the host having fmt(1) or indent(1).
#
# When a slice *adds* cases, land the additions against unchanged
# behaviour first (0 removed lines, N added, which is itself the proof
# that they are pure), re-baseline, and only then land the behaviour
# change -- so that the second delta is nothing but the behaviour.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
BASELINE=${FMT_BASELINE:-$HERE/fmtbase}
OUT=${SWEEP_OUT:-/tmp/fmtsweep-out}
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
"$HERE/fmtsweep.sh" "$REPO/target/debug/nvim" "$REPO/runtime" \
  "$OUT" "$LABEL" 2>&1 | tail -1

fail=0
for part in txt struct stderr; do
  if cmp -s "$BASELINE/base.$part" "$OUT/$LABEL.$part"; then
    echo "$part: IDENTICAL"
  else
    echo "$part: DIFFERS"
    # -a: the reports carry escaped high bytes, and diff would otherwise
    # call them binary and print nothing useful.
    # `|| true`: `set -e` plus `pipefail` would otherwise abort on the
    # first differing artifact, and the remaining ones -- the ones that
    # say *which* layer moved -- would never be compared.
    diff -a "$BASELINE/base.$part" "$OUT/$LABEL.$part" | head -60 || true
    fail=1
  fi
done
exit $fail
