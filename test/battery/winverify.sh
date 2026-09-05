#!/usr/bin/env bash
# Build the working tree and diff its winsweep against the stored
# baseline.  Run from anywhere; everything is absolute.
#
#   winverify.sh [label]         # default label: cur
#
# The baseline lives next to this script in winbase/ and was
# produced at commit 46c4bf3a3a -- B19's close, the last revision before
# any of window.rs or winfloat.rs is carved or rewritten.  It is the
# *pre-rewrite* behaviour of `do_window`'s letter dispatch, `win_split`/
# `win_split_ins`, `win_equal`/`win_equal_rec`, `frame_new_height`/
# `frame_new_width`/`frame_setheight`/`frame_setwidth`/`frame_minheight`/
# `frame_minwidth`/`frame_comp_pos`, `winframe_remove`/
# `winframe_find_altwin`/`winframe_restore`, `win_close`/
# `win_close_othertab`/`close_others`, `win_enter_ext`/`win_goto`/
# `win_vert_neighbor`/`win_horz_neighbor`, `win_exchange`/`win_rotate`/
# `win_totop`/`win_splitmove`/`win_move_after`, `win_new_tabpage`/
# `leave_tabpage`/`enter_tabpage`/`tabpage_move`/`goto_tabpage*`,
# `win_setheight_win`/`win_setwidth_win`/`win_new_height`/
# `scroll_to_fraction`/`win_fix_scroll`, `last_status`/`command_height`/
# `win_comp_pos`, `win_size_save`/`win_size_restore` and the snapshot
# family -- plus winfloat.rs end to end and `ui_ext_win_position`.
#
# RE-BASELINED ONCE, at B20-11 (`ee08ac2b99`), for the FOUR `s91`
# crashprobe aborts and nothing else.  All three kept sides --
# ~/agents/scratch/b20-1/nvim-46c4bf3a3a (B20's pre-batch side),
# ~/agents/scratch/b17-19/nvim-b00f1ef7e0 (B18's) and
# ~/agents/scratch/p0-2/nvim-ed789235ab (phase 16's) -- were re-run
# against the new baseline and differ from it on EXACTLY those rows:
#
#   OLD-SIDE EXCEPTION SET (all three sides, identically):
#     base.txt     5 rows: s91/huge-resize, s91/huge-vresize,
#                  s91/huge-winheight, s91/float-bufpos-huge, and the
#                  `s91 groups cases=24 aborted=N` summary (0 here, 4
#                  there)
#     base.struct  the same four case rows
#     base.stderr  IDENTICAL
#
# The old sides still abort where they always did; the port now
# SURVIVES, because B20-11 sums those four sizes in a wider type and
# clamps (`window/arith.rs`, `window/config.rs`, `winfloat.rs`).  Every
# other row of every artifact is IDENTICAL on all three sides, which is
# still the proof that the window family has not moved since P0.
# Regenerate the baseline only when a behaviour change is *intended*
# and reviewed:
#
#   winsweep.sh <nvim> <runtime> \
#       test/battery/winbase base
#
# ... and `just build` first: a mutation harness leaves the binary built
# from its last mutant, and a baseline taken from that compares mutant
# against mutant forever after.
#
# All three artifacts are compared.  `base.stderr` is EMPTY at the
# baseline and that is the assertion: the report and the canonical JSON
# are written by the sweep itself, so anything on the prompt is a
# message no case expected -- or a child that died where it should not
# have.
#
# THE REPORT CARRIES A `## <section> rows=N` LINE PER SECTION.  Those
# counts are the standing assertion that no section went silently empty;
# at the baseline they are:
#
#   s0-defaults 12 · s1-wincmd 784 · s2-split 751 · s3-resize 437 ·
#   s4-tabpage 132 · s5-float 231 · s6-config 66 · s7-auorder 34 ·
#   s91-crashprobe 25
#
# ... summing to `## TOTAL rows=2490` (the section rows plus the two
# `##` lines each section prints), and the artifacts are 2,492 / 2,470 /
# 0 lines -- the report adds `## TOTAL` and `exit 0`.  A `rows=0` with
# everything else unchanged is a harness bug, not a regression.
#
# FOUR ROWS ARE THE LOAD-BEARING ASSERTIONS.
#
#   * `s91 groups cases=24 aborted=0`.  ANY abort is now a regression.
#     Four of the crashprobes killed a debug build before B20-11 and
#     all four were PORT-SIDE integer overflow where the C wraps --
#     `:resize 2147483647`, `:set winheight=2147483647` (subtract),
#     `:vertical resize 2147483647` and a `bufpos` of `{INT_MAX,
#     INT_MAX}` (add).  They are fixed by summing in a wider type and
#     clamping, so the answer is the largest window the layout can
#     honour rather than a wrapped-negative one, and the four rows now
#     read `dead=false ... alive=2`.
#   * `s5ra/*/NE/20_70` and `s5ra/*/SE/20_70` must NOT read the same
#     `@row,col` as their `NW`/`SW` twins.  The anchor arithmetic lives
#     in `ui_ext_win_position` and is only reachable through a real
#     redraw; if a change makes the sweep stop painting, all 96
#     `s5ra/*` rows collapse to the config's own row/col and the whole
#     matrix silently stops gating.
#   * `s3scr/cmdheight/*/N` must MOVE with N.  `did_set_cmdheight` is
#     guarded by `full_screen`; if the sweep is ever switched back to
#     `-l`, every one of those eighteen rows freezes.
#   * `s7/*` must contain `WinResized` and `WinScrolled` lines.  Those
#     two fire only from `may_trigger_win_scrolled_resized` in
#     `normal_check`, so an s7 with none of them means the child's main
#     loop never turned and the section is measuring nothing.
#   * `s5z/*/n1` closes ONE float and `s5z/*/n2` two, while the bare
#     `s5z/*/fclose` closes them ALL.  That is not a bug in the sweep:
#     `:fclose` reads `eap->line1`, whose `ADDR_OTHER` default is the
#     CURSOR LINE, and the fixture's cursor sits on line 30 (filed
#     upstream, `ex-docmd-fclose-count-defaults-to-cursor-line.md`).
#     `s5z/*/cursor1` is the same command with the cursor on line 1 and
#     closes exactly one.  If the `n1`/`n2` rows ever start closing
#     everything, `float_zindex_cmp` has stopped being gated.
#
# The report's final `exit N` line is the hang/crash assertion -- 124 is
# the harness timeout and 134 an abort.
#
# TAKES ~12 s.  One `--embed` child for s7 plus s91's twenty-four.
#
# When a slice *adds* cases, land the additions against unchanged
# behaviour first (0 removed lines, N added, which is itself the proof
# that they are pure), re-baseline, and only then land the behaviour
# change -- so that the second delta is nothing but the behaviour.
# Handles are printed as per-case ORDINALS, never raw, so an inserted
# case does not renumber the rows below it.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
BASELINE=${WIN_BASELINE:-$HERE/winbase}
OUT=${SWEEP_OUT:-/tmp/winsweep-out}
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
"$HERE/winsweep.sh" "$REPO/target/debug/nvim" "$REPO/runtime" \
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
