#!/usr/bin/env bash
# Build the working tree and diff its foldsweep against the stored
# baseline.  Run from anywhere; everything is absolute.
#
#   foldverify.sh [label]          # default label: cur
#
# The baseline lives NEXT TO THIS SCRIPT in foldbase/ --
# never in a session scratchpad, which is the mistake that cost
# `spellverify` a phase (p19-close).  Its provenance, and the recipe
# for regenerating it, are in foldbase/COMMIT.
#
# It is the PRE-REWRITE behaviour of fold/: cut at phase 20 slice 12,
# before S13 touches a line of the batch, which is the whole point of
# standing this oracle up first.  Regenerate it only when a behaviour
# change is *intended* and reviewed:
#
#   just build      # a mutation harness leaves the binary built from
#                   # its last mutant; a baseline taken from that
#                   # compares mutant against mutant forever after
#   FOLDSWEEP_WORK=/tmp/fsweep-verify \
#     test/battery/foldsweep.sh \
#       $REPO/target/debug/nvim $REPO/runtime \
#       test/battery/foldbase base
#   git -C $REPO rev-parse HEAD > test/battery/foldbase/COMMIT
#
# All three artifacts are compared, stderr included: nvim's own messages
# go to the prompt, which in a headless process is stderr, and s19 is
# the only place E350/E351/E352/E490 and the "N lines folded" texts are
# visible at all.  The report's final `exit N` line is the hang/crash
# assertion -- 124 is the harness timeout and 134 an abort.
#
# When a slice *adds* cases, land the additions against unchanged
# behaviour first (0 removed lines, N added, which is itself the proof
# that they are pure), re-baseline, and only then land the behaviour
# change -- so that the second delta is nothing but the behaviour.
#
# Overrides: FOLD_BASELINE, SWEEP_OUT, FOLDSWEEP_WORK, REPO, NVIM.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
BASELINE=${FOLD_BASELINE:-$HERE/foldbase}
OUT=${SWEEP_OUT:-/tmp/foldsweep-out}
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

NVIM=${NVIM:-$REPO/target/debug/nvim}

rm -f "$OUT/$LABEL.txt" "$OUT/$LABEL.struct" "$OUT/$LABEL.stderr"
"$HERE/foldsweep.sh" "$NVIM" "$REPO/runtime" "$OUT" "$LABEL" 2>&1 | tail -1

fail=0
for part in txt struct stderr; do
  if diff -q "$BASELINE/base.$part" "$OUT/$LABEL.$part" >/dev/null; then
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
