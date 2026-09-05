#!/usr/bin/env bash
# Build the working tree and diff its fssweep against the stored
# baseline.  Run from anywhere; everything is absolute.
#
#   fsverify.sh [label]           # default label: cur
#
# The baseline lives next to this script in fsbase/ and was
# produced at commit 04c4762a05 -- B18-4, the last revision before any
# of eval/fs.rs's six children is rewritten.  It is the *pre-rewrite*
# behaviour of the whole family: name.rs's modify_fname, path.rs's
# resolve/simplify, find.rs's glob/readdir/findfile, dir.rs's whole
# mutating half, read.rs and write.rs.  It was also run against
# ~/agents/scratch/b17-19/nvim-b00f1ef7e0 (B18's pre-batch side) and is
# IDENTICAL there, which is the proof that nothing in B18-2/3/4 moved
# this family.  Regenerate it only when a behaviour change is *intended*
# and reviewed:
#
#   fssweep.sh <nvim> <runtime> \
#       test/battery/fsbase base
#
# ... and `just build` first: a mutation harness leaves the binary built
# from its last mutant, and a baseline taken from that compares mutant
# against mutant forever after.
#
# All three artifacts are compared, stderr included: s90 runs a block of
# failing commands through `-c` in children, which is the only spelling
# under which nvim *displays* an error and keeps going, so that artifact
# is the only view of this family's message path (E484/E482/E739/E17/
# E5060/E475/E1174/E730 and the `:cd` failures).  In-process, `vim.cmd`
# converts a Vimscript error into a Lua error and `pcall` swallows it --
# an s90 written in process comes out EMPTY while every other artifact
# looks healthy.
#
# THE REPORT CARRIES A `## <section> rows=N` LINE PER SECTION.  Those
# counts are the standing assertion that no section went silently empty;
# at the baseline they are:
#
#   s1-fnamemodify 774 · s2-pathcalc 154 · s3-glob 144 · s4-readdir 95 ·
#   s5-readwrite 218 · s6-mutate 1852 · s7-cwd 48 · s8-stat 221 ·
#   s9-errors 496 · s90-messages 1 · s91-crashprobe 55
#
# ... and the artifacts are 4,082 / 2,218 / 80 lines.  A `rows=0` with
# everything else unchanged is a harness bug, not a regression.
#
# The report's final `exit N` line is the hang/crash assertion -- 124 is
# the harness timeout and 134 an abort -- and s91's rows are the
# per-input version of the same question.  **s91 aborts nowhere at the
# baseline** (`k91 groups cases=54 aborted=0`), so any ABORTED row is a
# regression, not a re-baseline.
#
# Does NOT need `runtime/doc/tags`: nothing here drives `:help`.  The
# `runtime` argument only has to be a real VIMRUNTIME.
#
# TAKES ~30 s, almost all of it s91's 54 children.
#
# When a slice *adds* cases, land the additions against unchanged
# behaviour first (0 removed lines, N added, which is itself the proof
# that they are pure), re-baseline, and only then land the behaviour
# change -- so that the second delta is nothing but the behaviour.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
BASELINE=${FS_BASELINE:-$HERE/fsbase}
OUT=${SWEEP_OUT:-/tmp/fssweep-out}
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
"$HERE/fssweep.sh" "$REPO/target/debug/nvim" "$REPO/runtime" \
  "$OUT" "$LABEL" 2>&1 | tail -1

fail=0
for part in txt struct stderr; do
  if diff -q "$BASELINE/base.$part" "$OUT/$LABEL.$part" >/dev/null; then
    echo "$part: IDENTICAL"
  else
    echo "$part: DIFFERS"
    # -a: the reports escape high bytes, and diff would otherwise call
    # them binary and print nothing useful.
    # `|| true`: `set -e` plus `pipefail` would abort on the first
    # differing artifact and the ones that say *which* layer moved would
    # never be compared.
    diff -a "$BASELINE/base.$part" "$OUT/$LABEL.$part" | head -60 || true
    fail=1
  fi
done
exit $fail
