#!/usr/bin/env bash
# Build the working tree and diff its mousesweep against the stored
# baseline.  Run from anywhere; everything is absolute.
#
#   mouseverify.sh [label]         # default label: cur
#
# The baseline is CUT, not committed: it comes from the binary
# `test/battery/BASE` pins, cached under target/battery/base/<sha>/.
# The row was first baselined at commit 5e23ad6128 and RE-BASELINED TWICE: at B19-11's
# `getmousepos()` overflow fix (the only row it moved) and at B19-14,
# which ADDED three `s6/rightpress/onto-status-*` cases and removed
# nothing.  It is the *pre-rewrite*
# behaviour of `do_mouse` (695 lines: the button/click/drag switch, the
# click counter, the `'mousemodel'` fork, the middle-button paste, the
# CTRL-click tag jump and the status-line drag), `jump_to_mouse` (370),
# `ins_mouse`, `do_mousescroll` / `ins_mousescroll` /
# `do_mousescroll_horiz`, `mouse_comp_pos`, `vcol2col`,
# `mouse_find_win_inner` / `_outer` / `mouse_find_grid_win`,
# `mouse_check_grid`, `call_click_def_func`, `do_popup` and
# `f_getmousepos`.  It was also run against
# ~/agents/scratch/b19-1/nvim-5e23ad6128 (B19's pre-batch side),
# ~/agents/scratch/b17-19/nvim-b00f1ef7e0 (B18's) and
# ~/agents/scratch/p0-2/nvim-ed789235ab (phase 16's), three runs each,
# and is IDENTICAL on all three -- which is the proof that this family
# has not moved since P0.
#
# Regenerate ONLY when a behaviour change is *intended* and reviewed -- and
# regeneration is now a BASE BUMP, not a re-cut in place.  Write the new
# commit into `test/battery/BASE`, in a commit of its own whose body says
# what moved; the cache under target/battery/base/ is keyed by that sha, so
# every row re-cuts itself against the new binary on the next run.  See
# README.md.
#
# All three artifacts are compared.  `base.stderr` is EMPTY at the
# baseline and that is the assertion: every child is driven over RPC and
# a child that writes to the prompt has either died or printed a message
# no case expected.
#
# THE REPORT CARRIES A `## <section> rows=N` LINE PER SECTION.  Those
# counts are the standing assertion that no section went silently empty;
# at the baseline they are:
#
#   s1-buttons 170 · s2-multiclick 77 · s3-mouseopt 73 · s4-model 52 ·
#   s5-wheel 96 · s6-drag 44 · s7-clickdef 69 · s8-columns 99 ·
#   s9-modes 44 · s10-floats 42 · s11-mousepos 446 · s12-multigrid 33 ·
#   s13-gestures 59 · s91-crashprobe 25   (TOTAL 1357)
#
# ... and the artifacts are 1,359 / 1,328 / 0 lines.  A `rows=0` with
# everything else unchanged is a harness bug, not a regression.
#
# S6'S THREE `rightpress/onto-status-*` ROWS ARE B19-14'S ADDITION and
# they are the only thing in the sweep that reads `jump_to_mouse`'s
# `on_status_line` static.  A right DRAG onto a status line goes through
# `status_line_offset` instead and never touches it; the arm needs a
# right PRESS while `status_line_offset` is still 0 from an earlier
# click in the text, on a buffer TALLER than the window (with the arm
# broken the press falls through to the ordinary jump, which scrolls the
# view and starts Visual mode -- invisible on the nine-line fixture, so
# those three cases build their own 400-line one).  They also assert
# that no `'mousemodel'` opens the popup menu for a status-line right
# click: all three rows are equal, and one of them is `popup`, the model
# that would otherwise freeze the child (B19-3's upstream bug).
#
# TWO ROWS ARE THE LOAD-BEARING ASSERTIONS.  `s91 groups cases=24
# aborted=0`, and `s91/huge-col` answers `wincol = 2147483648`.
# RE-BASELINED AT B19-11 (`nvim.rs` 68882460c8): until then this row was
# the sweep's one baselined ABORT -- `nvim_input_mouse` accepts a column
# of INT_MAX, `f_getmousepos` computed `col + 1 + w_wincol_off` in `int`,
# the C wraps on the overflow and a checked build of the port trapped on
# it.  That was the port's own defect and B19-11 fixed it by summing in
# `VarNumber`, which is the width both fields have anyway.  ANY
# ABORTED ROW IS NOW A REGRESSION.
#
# ALL THREE KEPT SIDES STILL ABORT ON THAT ONE ROW, and that is the
# fix's documented divergence, not a failure: `b17-19/nvim-b00f1ef7e0`,
# `p0-2/nvim-ed789235ab` AND `b19-1/nvim-5e23ad6128` (the pre-batch
# side) all predate `68882460c8`.  Their diff against this baseline is
# exactly `s91/huge-col` plus the `aborted=` counter, and nothing else;
# a THIRD differing line means something moved.  Re-confirmed on all
# three at B19-14, after the s6 addition.
#
# And `s12/ui/attached` must read `n = 1` -- if the raw msgpack UI
# client stops attaching, every `ext_multigrid` row silently degrades
# into a duplicate of the single-grid ones.
#
# The report's final `exit N` line is the hang/crash assertion -- 124 is
# the harness timeout and 134 an abort.
#
# Does NOT need `runtime/doc/tags`, and does not read a help file: s3's
# `h` flag is measured against a SYNTHETIC `'buftype'=help` buffer, so
# $VIMRUNTIME only supplies the default `PopUp` menu (which s4 removes
# before defining its own).
#
# TAKES ~15 s.  Fourteen `--embed` children plus s91's twenty-four.
#
# When a slice *adds* cases, land the additions against unchanged
# behaviour first (0 removed lines, N added, which is itself the proof
# that they are pure), re-baseline, and only then land the behaviour
# change -- so that the second delta is nothing but the behaviour.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
OUT=${SWEEP_OUT:-/tmp/mousesweep-out}
REPO=${REPO:-$(cd "$HERE/../.." && pwd)}
LABEL=${1:-cur}
LOG=$OUT/build-$LABEL.log
# Cut mode.  `baseline.sh` re-enters this script as `--cut <nvim> <dir>` to
# cut the pinned baseline from the reference binary; it shares the ONE sweep
# call below with the head run, so the two sides of the differential cannot
# drift apart.  It skips the build and the diff.
CUT=
if [[ ${1:-} == --cut ]]; then CUT=$2; OUT=$3; LABEL=base; LOG=/dev/null; fi
NVIM_BIN=${CUT:-$REPO/target/debug/nvim}

mkdir -p "$OUT"
cd "$REPO"
if [[ -z $CUT ]] && ! just build >"$LOG" 2>&1; then
  echo "BUILD FAILED -- see $LOG" >&2
  grep -E '^(error|warning)' "$LOG" | head -60 >&2
  exit 1
fi

rm -f "$OUT/$LABEL.txt" "$OUT/$LABEL.struct" "$OUT/$LABEL.stderr"
"$HERE/mousesweep.sh" "$NVIM_BIN" "$REPO/runtime" \
  "$OUT" "$LABEL" 2>&1 | tail -1

if [[ -n $CUT ]]; then exit 0; fi

# The baseline is CUT, not committed: `baseline.sh` runs this same sweep
# against the binary `test/battery/BASE` pins and caches the result under
# target/battery/base/<sha>/.  The first row to want it pays for the
# reference build.  See README.md.
BASELINE=${MOUSE_BASELINE:-$("$HERE/baseline.sh" mouse)}
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
