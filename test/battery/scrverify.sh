#!/usr/bin/env bash
# scrverify — the B12 per-commit screen-pipeline gate.
#
#   scrverify.sh [label]
#
# Builds, sweeps, and diffs all three artifacts against a baseline CUT from
# the binary `test/battery/BASE` pins -- not committed, and cached under
# target/battery/base/<sha>/. ~70 s including the build, plus one reference
# build the first time the cache misses.
#
# Regenerate ONLY when a behaviour change is *intended* and reviewed -- and
# regeneration is a BASE BUMP, not a re-cut in place.  Write the new commit
# into `test/battery/BASE`, in a commit of its own whose body says what
# moved.  See README.md.
#
# RE-BASELINED AT B19-4 (still at 5e23ad6128): 4646/4998/867 -> 4984/5028/1950.
# Two additions and one flake fix, and the delta is exactly that -- 0 removed
# lines in .attrs, 2 in .vals; .txt moves 77 rows because `view-opts`
# snapshots after its edge walk.
#   * `screen-guicursor` dumps the recorded `mode_info` (which is
#     `mode_style_array`'s dump of the WHOLE eighteen-entry `shape_table`)
#     for eighteen `'guicursor'` values plus twenty malformed ones, and
#     reads `cursor_get_mode_idx` back through the `mode_change` event's
#     index in fifteen modes.  `'guicursor'` was in NO sweep before this.
#   * `view-motions` / `view-opts` / `view-horizontal` / `view-smoothscroll`
#     / `view-window` dump `winsaveview()` + `line('w0')`/`line('w$')` +
#     `screenpos()` after each of ~40 motions across `'scrolloff'`,
#     `'sidescrolloff'`, `'scrolljump'`, `'wrap'` and `'smoothscroll'` --
#     the first differential to NAME `update_topline`, `scroll_cursor_*`
#     and `curs_columns`.
#   * `syntime-report[2]`/`[3]` were FLAKY and are now sorted.  `:syntime
#     report` orders its rows by measured time, so `stReg body` and `stReg
#     END` swapped whenever the two patterns timed equal -- a false DIFFERS
#     in a full battery at B19-3.  `report_val`'s `sortbody` removes the
#     ordering; the column layout, the row count and the pattern text are
#     still pinned.
# Verified byte-identical across two runs and IDENTICAL against all three
# paired sides (b19-1/nvim-5e23ad6128, b17-19/nvim-b00f1ef7e0,
# p0-2/nvim-ed789235ab).  Cost 5 s -> 34 s.
#
# TWO HAZARDS THE NEW SCENARIOS PAID FOR.  A hit-enter prompt in the child
# BLOCKS THE MAIN LOOP, so the next `nvim_eval` never returns and the whole
# sweep wedges until its timeout -- every `view-*` scenario sets `nomore`
# and `shortmess+=sI` for that reason alone.  And the Screen only sees a UI
# event when the session is PUMPED: `mode_info_set` and `mode_change` arrive
# once per option assignment or mode change, not on a redraw, so reading
# `screen._mode_info` without `poke_eventloop()` + `screen:sleep()` first
# answers `n=0` and looks exactly like an option with no effect.
#
# RE-BASELINED AT P22-11 (at 6ff24614ba): 5008/5030/2084 -> 5027/5094/2087.
# One addition, and the delta is exactly that -- 0 removed lines in any of the
# three artifacts.
#   * `decor-provider-order` records the ORDER the decoration-provider
#     callbacks run in and where in the walk each sits, which nothing recorded
#     before (`decor-provider` only asserts the counts are non-zero, and a
#     screen dump shows a callback's RESULT, not when it ran).  Three
#     providers: one active throughout that places an ephemeral highlight from
#     every `on_line` and deletes a real mark from one of them (the deferred
#     free, the only reader of `running_decor_provider`); one that declines
#     the window from `on_win`, so its `on_line` must not be reached while its
#     `on_start`/`on_buf`/`on_end` still are; and one with an `on_range`.  A
#     second cycle is armed and dirtied in one chunk -- `on_buf` runs only for
#     a buffer with `b_mod_set` -- with a `:vsplit` open, so `on_win` runs
#     twice and the per-window reset is pinned too.  p22-1 §3.3 asked for this
#     row before S11 threaded `decor_state` down the draw pass.
#
#   ONE HAZARD IT PAID FOR.  The RPC pump can redraw between two requests, so
#   the callbacks fire an unpredictable number of TIMES; only the order inside
#   one cycle is stable.  The scenario counts cycles in `on_start` and records
#   the first one only.
#
# Diff ALL THREE. The grid (.txt) carries attribute *ids*, not values, so a
# colour-table change can be invisible there and loud in .attrs; and .vals is
# the only view of the answers that have no grid of their own (plines'
# virtcol/text_height, synID/synstack, getmatches, sign_getplaced, hlget).
# The `hlgroup-colortable` mutation was caught by .attrs+.vals with .txt
# IDENTICAL, which is the whole argument for keeping three files.
set -uo pipefail

here=$(dirname "$(realpath "$0")")
root=${SCRSWEEP_REPO:-$(dirname "$(dirname "$here")")}
label=${1:-head}
out=${SCRVERIFY_OUT:-$root/target/scrsweep/verify}
# Cut mode.  `baseline.sh` re-enters this script as `--cut <nvim> <dir>` to
# cut the pinned baseline from the reference binary; it shares the ONE sweep
# call below with the head run, so the two sides of the differential cannot
# drift apart.  It skips the build and the diff.
cut=
if [[ ${1:-} == --cut ]]; then cut=$2; out=$3; label=base; fi
nvim_bin=${cut:-$root/target/debug/nvim}

mkdir -p "$out"

if [[ -z $cut ]]; then
  echo "== build"
  ( cd "$root" && just build ) > "$out/build.log" 2>&1 || {
    echo "BUILD FAILED — see $out/build.log"; exit 1;
  }
fi

echo "== sweep"
"$here/scrsweep.sh" "$nvim_bin" "$out" "$label" || exit 1
if [[ -n $cut ]]; then exit 0; fi

# The baseline is CUT, not committed: `baseline.sh` runs this same sweep
# against the binary `test/battery/BASE` pins and caches the result under
# target/battery/base/<sha>/.  See README.md.
base=${SCR_BASELINE:-$("$here/baseline.sh" scr)}

rc=0
for ext in txt attrs vals; do
  if diff -q "$base/base.$ext" "$out/$label.$ext" > /dev/null 2>&1; then
    echo "  $ext IDENTICAL"
  else
    echo "  $ext DIFFERS:"
    # `|| true`: with `set -e` a non-zero diff through `head` aborts the loop
    # and the later artifacts are never compared (the persverify.sh bug).
    diff "$base/base.$ext" "$out/$label.$ext" | head -60 || true
    rc=1
  fi
done
exit $rc
