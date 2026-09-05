#!/usr/bin/env bash
# Build the working tree and diff its optsweep against the stored
# baseline.  Run from anywhere; everything is absolute.
#
#   optverify.sh [label]          # default label: cur
#
# The baseline lives next to this script in optbase/ and was
# produced at commit 5c829b911e -- phase 17's base, the last revision
# before `vimoption_T`'s `var: *mut c_void` is typed and the five
# generated `options/table_*.rs` files are regenerated.  It is the
# *pre-rewrite* content of the option TABLE: every name, short name,
# type, scope, default, flag and current value of all 374 options, the
# whole `= += -= ^= & &vim < ! inv no` alphabet of `do_set`, the
# global-local convention, what a fresh window and a fresh buffer
# inherit, and every `E` code `do_set` raises.
#
# WHY IT EXISTS.  There was no option-behaviour differential at all.
# Twenty-two oracles SET options -- every one of them does -- and not one
# reads the table back.  A rewrite that gets ONE entry's type, scope or
# default wrong produces a plausible editor that is quietly wrong about
# one option, and nothing in the functional suite enumerates the table.
#
# THREE ARTIFACTS, TWO LAYERS:
#
#   base.txt     one row PER OPTION in o1 (metadata + value), o2 (the
#                three `?` printers) and o3 (`&` vs `&vim`) and o5 (a
#                type-appropriate poke through BOTH the API and `:set`),
#                plus the scope, error and modifier sections.  Moving
#                ALONE is a behaviour change a user can see.
#   base.opts    the BULK layer: `:set all`, `:setglobal all`,
#                `:setlocal all`, `:verbose set`, the whole `:options`
#                window buffer and the info dict for all 374 options,
#                verbatim, plus every message o3 and o5 provoked.
#                Moving ALONE means a PRINTER or a message text moved
#                with every scalar answer still right.
#   base.stderr  EMPTY at the baseline, and that is the assertion.
#
# THE REPORT CARRIES A `## <section> rows=N` LINE PER SECTION.  At the
# baseline they are:
#
#   o0-canary 5 · o1-table 374 · o2-show 374 · o3-defaults 375 ·
#   o4-verbose 14 · o5-poke 374 · o6-scope 132 · o7-dumps 6 ·
#   o8-errors 72 · o9-modify 65 · o91-abortprobe 17
#
# ... summing to `## TOTAL rows=1808`, with artifacts 1,821 / 2,053 / 0
# lines (the report adds the eleven section lines, `## TOTAL` and
# `exit 0`).  **o1, o2, o3 and o5 must each equal the option count**:
# `o0/count options=374` is the same number four more times, and a
# section that falls short is a harness bug, not a regression.
#
# BASELINED FOUR-WAY ON DAY ONE, WITH NO EXCEPTION SET.  All three
# artifacts are byte-identical on every kept side:
# ~/agents/scratch/b22-2/nvim-44c6e8d630, b21-2/nvim-74ff723db9,
# b17-19/nvim-b00f1ef7e0 and p0-2/nvim-ed789235ab.  ANY diff on ANY side
# is a regression.
#
# SIX ROWS ARE THE LOAD-BEARING ASSERTIONS.
#
#   * `o0/count options=374 global=231 win=51 buf=92 bool=121 num=74
#     str=179 global_local=33 commalist=89 flaglist=8 dup=296`.  That is
#     the whole table in one line: eleven independent counts, and o1/o2/
#     o3/o5's row counts are the same 374 four more times.
#   * `o0/shortnames shortnames=341 dupes=0 unresolvable=0 misresolved=0
#     d=1a1d9ede246d` -- every abbreviation is unique, resolves through
#     `:set`, AND resolves to ITS OWN option: `:set {abbr}?` prints the
#     FULL name, so the whole abbreviation half of `find_option_index` is
#     observable, and `.opts` carries all 341 pairs.  Nothing else in the
#     tree reads that mapping.
#   * `o0/allamp idempotent=true` -- `:set all&` twice is the same
#     editor.
#   * `o0/wasset n=13 [...]` -- exactly the thirteen options the sandbox
#     itself sets.  A fourteenth means something is setting an option
#     behind the harness's back.
#   * `o3/ampdiff differ=0 of=374` -- Neovim carries ONE default per
#     option where Vim carries two, so `&` and `&vim` agree everywhere.
#     A non-zero number is a second default table reappearing.
#   * `o91/groups cases=16 aborted=0`.  THE SWEEP HAS NO EXPECTED ABORT:
#     any abort at all is a regression, by construction.
#
# WHAT IS NOT COVERED, DELIBERATELY.  o5 skips seventeen options -- each
# still a `SKIPPED reason=...` ROW, so the count stays 374 -- because
# poking them runs something (clipboard provider, keymap script, spell
# download, shell) or reconfigures the harness in a way `set {o}&` cannot
# undo (`verbose`, `verbosefile`, `debug`, `writedelay`, `redrawdebug`,
# `shada`, `shadafile`).  o6 drives 33 options, not all 374: the
# per-window / per-buffer matrix costs four rows and two window
# operations per option and the full cross-product buys repetition, not
# coverage.  `:set {o}!` on a boolean is in o9; the `c` flag of `:s` and
# an interactive `:set` prompt are in neither.
#
# TAKES ~2 s in the sweep plus the build.  Sixteen `--headless` children
# (o91), each under `timeout -k 2 30` and reaped before its row is
# printed.  If a run is killed, look for children with ppid 1 and KILL
# THEM BY PID -- never by pattern.
#
# THREE TRAPS FOR A LATER SLICE.
#
#   * o6's `number` row is deliberately global-FALSE / local-TRUE, and
#     `relativenumber` sits beside it: `copy_winopt` copies `wo_nu` and
#     `wo_rnu` on adjacent lines, and a swap between them is invisible
#     while the two agree -- which they do at every default.  o9's
#     `number` shape carries `+=3 -=2 ^=4` for the same reason: `+=` on a
#     NUMBER is arithmetic in a different arm of `get_option_newval`, and
#     "the command was accepted" is true whichever way it goes, so the row
#     must read the VALUE back.
#
#   * `harness()` re-applies the sandbox's own settings after every `&`
#     and every `all&`, because `&` restores the COMPILED default.  o3
#     and o5 issue nearly a thousand of them; drop `harness()` and every
#     row after the first is measured under a different editor.
#   * Every digest is taken over text NORMALISED IN LUA (`norm()`), not
#     over the raw string: `:set all` prints `runtimepath`, `backupdir`,
#     `shada` and `helpfile`, all of which carry the sandbox path.  The
#     driver's scrub fixes the artifact but cannot reach inside a digest.
#
# When a slice *adds* cases, land the additions against unchanged
# behaviour first (0 removed lines, N added), re-baseline, and only then
# land the behaviour change.
#
# Regenerate the baseline only when a behaviour change is *intended* and
# reviewed:
#
#   optsweep.sh <nvim> <runtime> \
#       test/battery/optbase base
#
# ... and `just build` first: a mutation harness leaves the binary built
# from its last mutant, and a baseline taken from that compares mutant
# against mutant forever after.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
BASELINE=${OPT_BASELINE:-$HERE/optbase}
OUT=${SWEEP_OUT:-/tmp/optsweep-out}
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

rm -f "$OUT/$LABEL.txt" "$OUT/$LABEL.opts" "$OUT/$LABEL.stderr"
"$HERE/optsweep.sh" "$REPO/target/debug/nvim" "$REPO/runtime" \
  "$OUT" "$LABEL" 2>&1 | tail -1

fail=0
for part in txt opts stderr; do
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
