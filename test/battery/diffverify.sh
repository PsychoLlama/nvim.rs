#!/usr/bin/env bash
# Build the working tree and diff its diffsweep against the stored B15
# baseline.  Run from anywhere; everything is absolute.
#
#   diffverify.sh [label]          # default label: cur
#
# The baseline lives next to this script in diffbase/ and was
# produced at commit bd1d648849 -- B15-2, the P0 prelude, which is the
# rev every B15 oracle baselines against.  It is the *pre-rewrite*
# behaviour of diff.rs and the six xdiff/ files: neither has been touched
# yet, which is the whole point of standing this oracle up first.
# Regenerate it only when a behaviour change is *intended* and reviewed:
#
#   diffsweep.sh <nvim> <runtime> \
#       test/battery/diffbase base
#
# ... and `just build` first: a mutation harness leaves the binary built
# from its last mutant, and a baseline taken from that compares mutant
# against mutant forever after.
#
# All three artifacts are compared, stderr included: nvim's own messages
# go to the prompt, which in a headless process is stderr, and s90 is the
# only place the E96/E97/E99/E100/E101/E102/E474/E816/E1549 arms and the
# ':diffget with report=0' message path are visible at all.  The report's
# final `exit N` line is the hang/crash assertion -- 124 is the harness
# timeout and 134 an abort.
#
# THIS SWEEP DEPENDS ON THE HOST'S diff(1) AND patch(1).  s10 and s12
# shell out to them (the driver symlinks whatever is on the caller's PATH
# into $WORK/bin).  Upgrading diffutils or patch on this machine will
# move those two sections' artifacts without a line of nvim changing;
# `x10 have-diff 1` / `P12 have-patch 1` at the top of each section is
# the tell that they ran at all.
#
# When a slice *adds* cases, land the additions against unchanged
# behaviour first (0 removed lines, N added, which is itself the proof
# that they are pure), re-baseline, and only then land the behaviour
# change -- so that the second delta is nothing but the behaviour.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
BASELINE=${DIFF_BASELINE:-$HERE/diffbase}
OUT=${SWEEP_OUT:-/tmp/diffsweep-out}
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
"$HERE/diffsweep.sh" "$REPO/target/debug/nvim" "$REPO/runtime" \
  "$OUT" "$LABEL" 2>&1 | tail -1

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
