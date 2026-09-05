#!/usr/bin/env bash
# Build the working tree and diff its rtsweep against the stored
# baseline.  Run from anywhere; everything is absolute.
#
#   rtverify.sh [label]           # default label: cur
#
# The baseline lives next to this script in rtbase/ and was
# produced at commit 2ecc9b69f7 -- B16-4's runtime.rs carve, the last
# revision before any of runtime/'s eight children is rewritten.  It is
# the *pre-rewrite* behaviour of the whole family: rtp.rs's
# 'runtimepath' construction, search.rs's :runtime, cache.rs's search
# path, pack.rs's :packadd, expand.rs's completion, source.rs's :source,
# estack.rs's <sfile> and script.rs's registry.  Regenerate it only when
# a behaviour change is *intended* and reviewed:
#
#   rtsweep.sh <nvim> <runtime> \
#       test/battery/rtbase base
#
# ... and `just build` first: a mutation harness leaves the binary built
# from its last mutant, and a baseline taken from that compares mutant
# against mutant forever after.
#
# All three artifacts are compared, stderr included: s20 runs a block of
# commands through `-c` in children, which is the only spelling under
# which nvim *displays* an error and keeps going, so that artifact is
# the only view of this subsystem's message path (E471/E484/E919/E168,
# "Cannot source a directory", and the 'verbose' sourcing trace).  The
# report's final `exit N` line is the hang/crash assertion -- 124 is the
# harness timeout and 134 an abort -- and s91's rows are the per-input
# version of the same question.  **s91 aborts nowhere at the baseline**,
# so any ABORTED row is a regression, not a re-baseline.
#
# Does NOT need `runtime/doc/tags`: nothing here drives `:help`.  It does
# need the `runtime` argument to be a real VIMRUNTIME, because s01 asks
# what the default 'runtimepath' is and `$VIM` is derived from it.
#
# When a slice *adds* cases, land the additions against unchanged
# behaviour first (0 removed lines, N added, which is itself the proof
# that they are pure), re-baseline, and only then land the behaviour
# change -- so that the second delta is nothing but the behaviour.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
BASELINE=${RT_BASELINE:-$HERE/rtbase}
OUT=${SWEEP_OUT:-/tmp/rtsweep-out}
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
"$HERE/rtsweep.sh" "$REPO/target/debug/nvim" "$REPO/runtime" \
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
