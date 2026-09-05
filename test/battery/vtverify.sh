#!/usr/bin/env bash
# Build the working tree and diff its vtsweep against the stored
# baseline.  Run from anywhere; everything is absolute.
#
#   vtverify.sh [label]          # default label: cur
#
# The baseline lives next to this script in vtbase/ and was
# produced at commit 74ff723db9 -- B21's base, the last revision before
# any of crates/nvim/src/nvim/vterm/ is graduated, split or rewritten.
# It is the *pre-rewrite* behaviour of the emulator: `vterm_input_write`
# and the escape-sequence state machine (parser.rs), the C0/C1 dispatch,
# every CSI (csi.rs), the whole SGR pen alphabet (pen.rs), the mode
# table and DECSTBM/DECSLRM/DECOM/DECAWM (mode.rs, state.rs), DECRQSS
# (dcs.rs), the UTF-8 decoder and the charset designators (encoding.rs,
# text.rs), OSC titles/hyperlinks and the OSC 52 selection decoder
# (selection.rs), the mouse encoders (mouse.rs), the key encoders
# (keyboard.rs), the cell buffers, damage merging, scrollback and reflow
# (screen.rs, damage.rs, cell.rs), the colour model (color.rs) and the
# reply writer (output.rs) -- plus the layout of all 33 `repr(C)` types.
#
# BASELINED THREE-WAY ON DAY ONE.  The stored artifacts are IDENTICAL,
# byte for byte, on all three kept sides --
# ~/agents/scratch/b21-2/nvim-74ff723db9 (B21's pre-batch side),
# ~/agents/scratch/b17-19/nvim-b00f1ef7e0 (B18's) and
# ~/agents/scratch/p0-2/nvim-ed789235ab (phase 16's) -- each re-proved by
# behaviour first (`set cinoptions=>2147483648` rc = 0 / 0 / **134**,
# all three answering `@@ -2 +2 @@` to `vim.diff{ctxlen=-1}`).
#
# RE-BASELINED AT B21-9 (`nvim.rs` c613f92ab5), and that re-baseline
# opened the sweep's FIRST old-side exception set.  `putglyph`'s
# continuation loop no longer runs off the last column, so
# `v91/one-by-one` answers instead of aborting.  ALL THREE KEPT SIDES
# PREDATE THE FIX AND STILL ABORT ON THAT ONE ROW: their diff against
# this baseline is exactly
#
#   txt     `v91/one-by-one` + `v91 groups cases=32 aborted=`  (2 rows)
#   struct  `v91/one-by-one`                                   (1 row)
#   stderr  empty, IDENTICAL
#
# ... and nothing else -- re-confirmed on all three at B21-9.  A THIRD
# differing txt row, or ANY row outside v91, means something moved.
# Against the WORKING TREE there is still no exception set: any diff at
# all is a bug.
#
# All three artifacts are compared.  `base.stderr` is EMPTY at the
# baseline and that is the assertion: the report and the canonical JSON
# are written by the sweep itself, so anything on the prompt is a
# message no case expected -- or a v91 child that died where it should
# not have.
#
# THE REPORT CARRIES A `## <section> rows=N` LINE PER SECTION.  Those
# counts are the standing assertion that no section went silently empty;
# at the baseline they are:
#
#   v0-canary 57 · v1-parser 493 · v2-csi 540 · v3-sgr 243 ·
#   v4-modes 162 · v5-utf8 195 · v6-osc 200 · v7-mouse 689 ·
#   v8-keyboard 2307 · v9-damage 384 · v10-resize 61 ·
#   v91-abortprobe 33
#
# ... summing to `## TOTAL rows=5388`, and the artifacts are 5,390 /
# 5,344 / 0 lines -- the report adds `## TOTAL` and `exit 0`.  A
# `rows=0` with everything else unchanged is a harness bug, not a
# regression.
#
# FIVE ROWS ARE THE LOAD-BEARING ASSERTIONS.
#
#   * `## v0-canary rows=57`, and every `v0/size/*` and `v0/off/*` row.
#     They are the ABI: the sweep's `ffi.cdef` is a hand-written,
#     self-contained statement of the layout of all 33 `repr(C)` types,
#     and it was checked field by field against the ffigen-generated
#     `target/ffi/unit-cdefs.h` (36 sizes, 36 alignments and 108 offsets,
#     all equal).  If B21's `types/vterm.rs` work changes a layout, THESE
#     ROWS MOVE FIRST and every row below them becomes meaningless --
#     read v0 before believing any other diff.  The seven `v0/live/*`
#     rows drive the emulator and read the value back out of the struct,
#     so a shift that happens to preserve `sizeof` still shows up.
#   * `v91 groups cases=32 aborted=0`, and `v91/one-by-one` answers
#     `PROBE one-by-one ok=true r=60`.  Until B21-9 this row was the
#     sweep's one baselined ABORT: a 1x1 terminal fed a wide glyph died
#     at `vterm/screen.rs:136`, "null pointer dereference", because
#     `putglyph` guarded the lead cell and not the continuation cells
#     `getcell` answers null for past the last column.  Unlike the
#     mousesweep precedent this was NOT the port's own defect -- the
#     vendored C is byte-faithful and upstream segfaults there too (see
#     ~/agents/context/1786212071-upstream-neovim-bugs/) -- and c613f92ab5
#     fixed it anyway, because it is a reachable crash.  ANY ABORTED ROW
#     IS NOW A REGRESSION; the three kept sides' abort on this one row is
#     the documented exception set above.
#   * `v9/m{0,1,2,3}/*` must not all read the same `ev={...}`.  The four
#     merge levels are the whole point of the section (upstream's own
#     `62screen_damage` is `pending()`, so nothing else anywhere covers
#     damage merging).  `merge=0` emits a rect per cell, `merge=1` per
#     row, `merge=2` per screen and `merge=3` coalesces scrolls -- and
#     what is NOT emitted is answered by the `pend{...}` field, because
#     `vterm_screen_flush_damage` is not an exported symbol and the
#     residual merge state IS the rest of the answer.
#   * `v7/btn/{x10,utf8,rxvt,sgr}/*` must answer FOUR different `out=`
#     encodings, and `v8/key/{plain,ckm,kpam,both}/*` must answer
#     different escapes for the cursor and keypad keys.  Those two
#     sections are the only gate anywhere on mouse.rs and keyboard.rs,
#     which are pure encoders with no other observable.
#   * `v1/*/bytes` must equal `v1/*/whole` wherever the sequence is
#     complete.  The four split strategies exist to prove the parser
#     keeps no state it should not across an `input_write` boundary; a
#     diff on the `bytes` rows alone is a resumption bug.  The same
#     applies to `v5/split/*`, which is the ONLY gate on the
#     recombination window at the top of `text::print`: inside one
#     write the cluster is gathered by `print`'s own loop and
#     `combine_pos`/`combine_width` are never consulted.
#   * `v2/erasecont/*` must show `1` in the CONTINUATION position of
#     some row's `lineinfo` prefix.  It is the only gate on
#     `VTermState::erase`'s continuation arm, because `paint()` writes
#     rows four columns short of the right edge and so sets no marks at
#     all -- every other erase row answers `000:` whatever that arm
#     does.  Both of these were added at B21-3 after `vtmutate` reported
#     NOT CAUGHT: the gap was a missing GESTURE, not a missing answer.
#
# The report's final `exit N` line is the hang/crash assertion -- 124 is
# the harness timeout and 134 an abort.
#
# TAKES ~5 s in the sweep plus the build.  Thirty-two `-l` children for
# v91 and nothing else; no pty, no UI, no RPC, no orphans possible.
#
# THE JIT IS OFF INSIDE THE SWEEP and must stay off.  See the .lua
# header: by-value struct arguments arrive as register pairs that the
# script re-reads through a cdata alias LuaJIT's optimiser does not
# relate, and a compiled trace forwards a stale load.  With the JIT on,
# two runs of the SAME binary differ on scattered `movecursor` rows --
# and because LuaJIT 2.1 seeds its hot counters from a PRNG, which rows
# move changes every run.
#
# When a slice *adds* cases, land the additions against unchanged
# behaviour first (0 removed lines, N added, which is itself the proof
# that they are pure), re-baseline, and only then land the behaviour
# change -- so that the second delta is nothing but the behaviour.
# Nothing in a label is a process-global counter, so an inserted case
# does not renumber the rows below it.
#
# Regenerate the baseline only when a behaviour change is *intended* and
# reviewed:
#
#   vtsweep.sh <nvim> <runtime> \
#       test/battery/vtbase base
#
# ... and `just build` first: a mutation harness leaves the binary built
# from its last mutant, and a baseline taken from that compares mutant
# against mutant forever after.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
BASELINE=${VT_BASELINE:-$HERE/vtbase}
OUT=${SWEEP_OUT:-/tmp/vtsweep-out}
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
"$HERE/vtsweep.sh" "$REPO/target/debug/nvim" "$REPO/runtime" \
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
