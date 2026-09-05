#!/usr/bin/env bash
# Build the working tree and diff its marksweep against the stored
# baseline.  Run from anywhere; everything is absolute.
#
#   markverify.sh [label]          # default label: cur
#
# The baseline lives next to this script in markbase/ and was
# produced at commit 44c6e8d630 -- B22's base, the last revision before
# anything in the marktree family is split, graduated or rewritten.  It
# is the *pre-rewrite* behaviour of `marktree_put` / `marktree_del_itr` /
# `marktree_splice` / `marktree_move`, of `split_node` / `merge_node` /
# `pivot_left` / `pivot_right`, of the plain, filtered and overlap
# iterator walks, of the intersection sets and the meta counts, and of
# `mt_inspect` itself.
#
# WHY IT EXISTS.  `marktree_check` and `marktree_check_intersections`
# have ZERO production callers: they run under `test/unit/
# marktree_spec.lua` and nowhere else.  Before this sweep NOTHING in any
# oracle read a node boundary, a level, a `p_idx` or an intersection set
# through a running editor -- the functional suite passes over a
# structurally corrupt tree for as long as the marks that come back
# happen to be right, and structural corruption is exactly what
# splice.rs and rebalance.rs produce when they produce anything.  The
# observable, `nvim__buf_debug_extmarks`, was ITSELF untested: nothing
# in `test/` called it, so the sweep gates the API as well as the tree.
#
# BASELINED FOUR-WAY ON DAY ONE, WITH NO EXCEPTION SET.  The stored
# artifacts are IDENTICAL, byte for byte, on every kept side:
# ~/agents/scratch/b22-2/nvim-44c6e8d630 (B22's pre-batch side, and
# byte-identical to target/debug/nvim at the base),
# ~/agents/scratch/b21-2/nvim-74ff723db9 (B21's),
# ~/agents/scratch/b17-19/nvim-b00f1ef7e0 (B18's) and
# ~/agents/scratch/p0-2/nvim-ed789235ab (phase 16's).  `ed789235ab`
# predates `d5937996ca`, which demoted 107 carried-over C `assert()`s in
# this family to `debug_assert!` -- B22-2 proved that demotion inert
# between debug builds, and this sweep confirms it end to end.  The
# `cinoptions=>2147483648` abort that DOES separate `ed789235ab` from
# the other sides lives in charset.rs, outside the marktree family, and
# no case here goes near it.
#
# FOUR ARTIFACTS, THREE LAYERS, and which one moves tells you what
# happened:
#
#   base.txt     one line per case: node / level / key / intersection /
#                mark counts, three digests, plus whatever scalars the
#                case measured.  Moving ALONE means a count or a lever.
#   base.tree    the SHAPE layer: per case the per-level aggregate, one
#                normalised line per node (`n3 par=n0 lvl=0 pidx=2 nk=19
#                ix=[..] k=[..]`) and the plain positional dump
#                verbatim.  Moving ALONE means the tree is built
#                differently -- a split, a merge, a pivot or an
#                intersection set -- with every answer a user can see
#                still correct.  NOTHING ELSE IN THIS TREE CAN SEE THAT.
#   base.marks   the BEHAVIOUR layer: every mark the details walk
#                returns, fixed field order.  Moving ALONE is a
#                user-visible change.
#   base.stderr  EMPTY at the baseline, and that is the assertion.
#
# THE REPORT CARRIES A `## <section> rows=N` LINE PER SECTION.  Those
# counts are the standing assertion that no section went silently empty;
# at the baseline they are:
#
#   m0-canary 12 · m1-putdel 35 · m2-splice 37 · m3-collapse 18 ·
#   m4-undo 14 · m5-churn 15 · m6-filter 22 · m7-overlap 31 ·
#   m91-abortprobe 21
#
# ... summing to `## TOTAL rows=205`, with artifacts 216 / 4,116 /
# 6,178 / 0 lines (the report adds the nine section lines, `## TOTAL`
# and `exit 0`).  A `rows=0` with everything else unchanged is a harness
# bug, not a regression.
#
# SIX ROWS ARE THE LOAD-BEARING ASSERTIONS.
#
#   * `m0/fill-19` vs `m0/fill-20` -- `nodes=1 lvls=1 keys=19` and then
#     `nodes=3 lvls=2 keys=20`.  `MT_BRANCH_FACTOR` is 10, so a node
#     holds 2*10-1 = 19 keys and the twentieth splits the root.  These
#     two rows ARE that constant; if they stop bracketing the split,
#     every shape row below them is describing a different tree.
#   * `m0/ix-lever ix_before=0 ix_short=0 ix_long=8`.  Only a range
#     covering the WHOLE of a node lands in that node's intersection
#     set, so a two-line pair populates nothing and a hundred-line one
#     populates eight.  This is the sweep's intersection contract: if
#     `ix_long` collapses to zero, m7's overlap deltas are measuring an
#     empty set and the whole intersect half of the sweep is inert.
#   * `m0/stable dot_stable=true keys_live=true` -- the dump is a pure
#     function of the tree (twice in a row is byte-identical) and the
#     `keys` argument actually changes it.  The oracle's own
#     determinism.
#   * `m0/dot-vs-plain agree=true` -- the two renderings count the same
#     nodes.  They are produced by two different recursions over the
#     same structure, so this is a free cross-check on both.
#   * `m6/pivot/*` -- the only rows that see the meta bookkeeping a
#     PIVOT or a MERGE does.  They need all three of: every fifth mark
#     decorated, 50-80% of the marks then deleted, and the height read
#     PER RANGE (`grid=`).  The whole-buffer `h1=` survives errors a
#     sixty-line window does not, so do not "simplify" them.
#   * every `m7/*` row's `delta=` -- the overlap walk minus the plain
#     walk over the SAME range.  That difference IS the intersection set
#     read back (`marktree_itr_get_overlap` / `_step_overlap` is its
#     only reader).  A section of `delta=0` means the sets are not being
#     consulted, whatever the counts say.
#   * `m91/groups cases=20 aborted=0`.  THE SWEEP HAS NO EXPECTED ABORT:
#     any abort at all is a regression, by construction.
#
# TAKES ~1.5 s in the sweep plus the build.  Twenty `--headless` children
# (m91), each of which has exited before its row is printed.  If a run
# is killed, look for children with ppid 1 and KILL THEM BY PID -- never
# by pattern.
#
# When a slice *adds* cases, land the additions against unchanged
# behaviour first (0 removed lines, N added, which is itself the proof
# that they are pure), re-baseline, and only then land the behaviour
# change.  Nothing here prints a buffer or window HANDLE and every case
# builds its own namespace, so extmark ids restart at 1 per case and an
# inserted case renumbers no row below it.
#
# Regenerate the baseline only when a behaviour change is *intended*
# and reviewed:
#
#   marksweep.sh <nvim> <runtime> \
#       test/battery/markbase base
#
# ... and `just build` first: a mutation harness leaves the binary built
# from its last mutant, and a baseline taken from that compares mutant
# against mutant forever after.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
BASELINE=${MARK_BASELINE:-$HERE/markbase}
OUT=${SWEEP_OUT:-/tmp/marksweep-out}
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

rm -f "$OUT/$LABEL.txt" "$OUT/$LABEL.tree" "$OUT/$LABEL.marks" "$OUT/$LABEL.stderr"
"$HERE/marksweep.sh" "$REPO/target/debug/nvim" "$REPO/runtime" \
  "$OUT" "$LABEL" 2>&1 | tail -1

fail=0
for part in txt tree marks stderr; do
  if diff -q "$BASELINE/base.$part" "$OUT/$LABEL.$part" >/dev/null; then
    echo "$part: IDENTICAL"
  else
    echo "$part: DIFFERS"
    # `|| true`: `set -e` plus `pipefail` would abort on the first
    # differing artifact and the ones that say *which* layer moved would
    # never be compared.
    diff -a "$BASELINE/base.$part" "$OUT/$LABEL.$part" | head -60 || true
    fail=1
  fi
done
exit $fail
