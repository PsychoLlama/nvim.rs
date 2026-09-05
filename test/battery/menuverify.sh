#!/usr/bin/env bash
# Build the working tree and diff its menusweep against the stored
# baseline.  Run from anywhere; everything is absolute.
#
#   menuverify.sh [label]         # default label: cur
#
# The baseline lives next to this script in menubase/ and was
# produced at commit 5e23ad6128 -- B18-16, the last revision before any
# of menu.rs is carved or rewritten.  It is the *pre-rewrite* behaviour
# of `ex_menu` (the option/priority/enable parse and the four dispatch
# arms), `add_menu_path` (the tree insert, the popup copies and the
# `menu_translate_tab_and_shift` name split), `remove_menu`,
# `menu_enable_recurse`, `show_menus` / `show_menus_recursive` (the
# listing text), `menu_get_recursive` / `menu_get`, `menuitem_getinfo` /
# `f_menu_info`, `set_context_in_menu_cmd` + `get_menu_name` /
# `get_menu_names` (completion), `get_menu_cmd_modes` /
# `get_menu_mode_str` / `popup_mode_name`, `menu_name_skip` /
# `menu_namecmp` / `menu_name_equal` / `menu_unescape_name` /
# `menu_text`, `ex_menutranslate` / `menutrans_lookup` /
# `menu_skip_part`, `menu_find` / `menu_getbyname` / `find_menu`,
# `ex_emenu` / `execute_menu` and `show_popupmenu`.  It was also run
# against ~/agents/scratch/b19-1/nvim-5e23ad6128 (B19's pre-batch side),
# ~/agents/scratch/b17-19/nvim-b00f1ef7e0 (B18's) and
# ~/agents/scratch/p0-2/nvim-ed789235ab (phase 16's) and is IDENTICAL on
# all three, which is the proof that this family has not moved since P0.
# Regenerate it only when a behaviour change is *intended* and reviewed:
#
#   menusweep.sh <nvim> <runtime> \
#       test/battery/menubase base
#
# ... and `just build` first: a mutation harness leaves the binary built
# from its last mutant, and a baseline taken from that compares mutant
# against mutant forever after.
#
# All three artifacts are compared.  `base.stderr` is EMPTY at the
# baseline and that is the assertion: the report and the canonical JSON
# are written by the sweep itself, so anything on the prompt is a
# message no case expected -- or a child that died.
#
# THE REPORT CARRIES A `## <section> rows=N` LINE PER SECTION.  Those
# counts are the standing assertion that no section went silently empty;
# at the baseline they are:
#
#   s0-default 7 · s1-cmds 91 · s2-modes 45 · s3-priority 19 ·
#   s4-names 75 · s5-listing 38 · s6-get 122 · s7-unmenu 33 ·
#   s8-enable 31 · s9-complete 58 · s10-translate 40 · s11-emenu 54 ·
#   s12-popup 14 · s91-crashprobe 21
#
# ... summing to `## TOTAL rows=676` (the section rows plus the two
# `##` lines each section prints), and the artifacts are 678 / 812 / 0
# lines -- the report adds `## TOTAL` and `exit 0`.  A `rows=0` with
# everything else unchanged is a harness bug, not a regression.
#
# THREE ROWS ARE THE LOAD-BEARING ASSERTIONS.
#
#   * `s91 groups cases=20 aborted=0` -- NOTHING in the menu family
#     kills the editor at the baseline, including an 800-level path, a
#     20,000-byte rhs and 500 top-level menus.  Any ABORTED row is a
#     regression.
#   * every `s12/pop/*` row reads `pum_at_first_timer = 0`.  That is
#     the frozen-event-loop finding, not a cosmetic field: a one-shot
#     timer armed 150 ms before the popup opens does not get to run
#     until `<Esc>` closes it, so what it sees is `pumvisible() == 0`.
#     All EIGHT `s12/pop/*` rows read 0 at the baseline.
#     A `1` would mean the loop kept turning (the upstream bug fixed);
#     a `-1` that the timer never fired at all.
#   * `s11/sid/viaCmd` answers `i` where `s11/sid/viaLua` answers `n`.
#     `execute_menu`'s Insert arm requires `current_sctx.sc_sid == 0`,
#     so the same `:emenu`, in the same Insert mode, picks a DIFFERENT
#     menu index depending on whether a script issued it.  Both rows
#     reading the same thing means that rule has been lost.
#
# The report's final `exit N` line is the hang/crash assertion -- 124 is
# the harness timeout and 134 an abort.
#
# S0 IS COUPLED TO THE RUNTIME AND NOTHING ELSE IS.  `-u NONE` does not
# suppress `runtime/lua/vim/_defaults.lua`'s fifteen-entry `PopUp` menu;
# s0 records it deliberately (it is the only real menu tree here) and
# every later section wipes it.  An edit to `_defaults.lua` re-baselines
# s0 alone.
#
# TAKES ~11 s.  Two `--embed` children for s11/s12 plus s91's twenty.
#
# When a slice *adds* cases, land the additions against unchanged
# behaviour first (0 removed lines, N added, which is itself the proof
# that they are pure), re-baseline, and only then land the behaviour
# change -- so that the second delta is nothing but the behaviour.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
BASELINE=${MENU_BASELINE:-$HERE/menubase}
OUT=${SWEEP_OUT:-/tmp/menusweep-out}
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
"$HERE/menusweep.sh" "$REPO/target/debug/nvim" "$REPO/runtime" \
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
