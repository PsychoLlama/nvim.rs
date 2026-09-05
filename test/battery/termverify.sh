#!/usr/bin/env bash
# Build the working tree and diff its termsweep against the stored
# baseline.  Run from anywhere; everything is absolute.
#
#   termverify.sh [label]          # default label: cur
#
# The baseline is CUT, not committed: it comes from the binary
# `test/battery/BASE` pins, cached under target/battery/base/<sha>/.
# The row was first baselined at commit 74ff723db9 -- B21's base, the last revision before
# terminal.rs or terminal/ is narrowed or rewritten.  It is the
# *pre-rewrite* behaviour of the terminal family: `terminal_open` and
# `terminal_alloc` (incl. the `g:terminal_color_N` palette),
# `terminal_receive` and the vterm feed, the damage/moverect/movecursor/
# settermprop/sb_push/sb_pop callbacks, `refresh_terminal` /
# `refresh_screen` / `refresh_size` / `refresh_scrollback`,
# `terminal_check_size`, `adjust_scrollback` and the scrollback ring,
# `terminal_get_line_attributes`, terminal-mode key and mouse encoding,
# `terminal_enter`/`terminal_leave`, the `TermRequest` autocommand and
# the reply writer, `terminal_notify_theme`, the OSC 52 clipboard
# decoder, and the close/kill/wipe path with its `[Process exited]`
# extmark.
#
# ADOPTED at B21-4 from the never-baselined phase-15 tool
# `1785449630-termsweep.{sh,lua}`; all twenty of its scenario names are
# still cases here.  See `b21-4-termsweep.md`.
#
# BASELINED THREE-WAY ON DAY ONE.  The stored artifacts are IDENTICAL,
# byte for byte, on all three kept sides --
# ~/agents/scratch/b21-2/nvim-74ff723db9 (B21's pre-batch side, and
# byte-identical to target/debug/nvim at the base),
# ~/agents/scratch/b17-19/nvim-b00f1ef7e0 (B18's) and
# ~/agents/scratch/p0-2/nvim-ed789235ab (phase 16's) -- each re-proved
# by behaviour first (`set cinoptions=>2147483648` rc = 0 / 0 / **134**,
# all three answering `@@ -2 +2 @@` to `vim.diff{ctxlen=-1}`).
#
# RE-BASELINED TWICE, EACH TIME BY A CRASH FIX, AND EACH TIME THE
# OLD-SIDE EXCEPTION SET GREW BY THE ROW THAT WAS FIXED:
#
#   B21-9  (`nvim.rs` c613f92ab5)  `t91/onecol-wide`   -- the `putglyph`
#                                  crash, UPSTREAM's (still unfixed there)
#   B21-10 (`nvim.rs` d139fba127)  `t91/palette-unset` -- the
#                                  `g:terminal_color_N` double free, PORT-SIDE
#
# `t91 groups` now reads `aborted=0 cases=19`: THE SWEEP HAS NO EXPECTED
# ABORT LEFT, so ANY abort is a regression.  All three kept sides
# predate both fixes and still die on both probes; their diff against
# this baseline is exactly
#
#   txt     `t91/onecol-wide` + `t91/palette-unset`
#           + `t91/groups aborted=`                      (3 rows)
#   struct  the same three                               (3 rows)
#   stderr  empty, IDENTICAL
#
# ... and nothing else -- re-confirmed on all three at B21-10
# (`b21-2/nvim-74ff723db9`, `b17-19/nvim-b00f1ef7e0`,
# `p0-2/nvim-ed789235ab`, 6 differing diff lines each in txt and struct,
# 0 in stderr).  A FOURTH differing row, or ANY row outside t91, means
# something moved.
#
# Against the WORKING TREE there is still no exception set: besides the
# two fixes the family has moved exactly once since the older sides were
# built (`f4170015c2`, cursor shape, which swapped terminal.rs and
# refresh.rs onto the `shape_entry`/`update_shape_entry` accessors
# without changing what they do), and the sweep cannot see a cursor
# SHAPE anyway -- it is delivered to a UI as `mode_info_set` and nothing
# here reads that.
#
# All three artifacts are compared.  `base.stderr` is EMPTY at the
# baseline and that is the assertion: everything the sweep has to say
# it writes to the report, so anything on the prompt is a message no
# case expected -- or an `--embed` child that died where it should not
# have.  (One thing had to be installed to keep it empty: a Lua
# CLIPBOARD PROVIDER, because OSC 52 without one prints
# `E319: No "clipboard" provider found` and the payload is then
# unobservable.  t6's `clip=` field is that provider's log.  It is
# installed in t6's CHILD -- the whole section runs in one, because a
# `TermRequest` autocommand in THIS process also collects what the
# terminals of t1 and t3 left queued, and which case they land in is a
# race.)
#
# THE REPORT CARRIES A `## <section> rows=N` LINE PER SECTION.  Those
# counts are the standing assertion that no section went silently
# empty; at the baseline they are:
#
#   t0-canary 13 · t1-sb 15 · t2-geom 9 · t3-attr 25 · t4-attrui 26 ·
#   t5-input 41 · t6-request 25 · t7-pty 9 · t91-abortprobe 20
#
# ... summing to `## TOTAL rows=201`, and the artifacts are 203 / 183 /
# 0 lines -- the report adds `## TOTAL` and `exit 0`.  A `rows=0` with
# everything else unchanged is a harness bug, not a regression.
#
# SIX ROWS ARE THE LOAD-BEARING ASSERTIONS.
#
#   * `t0/lever after_grow=12 after_send=12 after_shrink=25`.  This is
#     the sweep's own determinism contract: the refresh is deferred
#     behind a 10 ms timer and the ONLY synchronous door into it from
#     script is a SHRINK of `'scrollback'`.  If a rewrite makes the
#     refresh eager (`after_send` moves) or makes the poke inert
#     (`after_shrink` stops moving), every other row in the sweep is
#     measuring something other than what it claims -- fix the lever
#     before believing anything else.
#   * `t0/size/*` and `t2/*`.  `terminal_check_size` takes the MAXIMUM
#     width over the windows showing the buffer and refuses only a
#     ZERO, so these rows are the whole geometry contract.  `t2/narrow`,
#     `t2/tall` and `t2/regrow_window` answer `screen_top` /
#     `splits=h6:top76/81,h18:top64/81,...` -- WHERE THE SCREEN STARTS,
#     probed by writing a marker into the top screen row between DECSC
#     and DECRC.  A scrollback pop or push moves lines between the ring
#     and the screen WITHOUT CHANGING THE BUFFER'S LINE COUNT, so a
#     line count is blind to `term_sb_pop` and `refresh_size`; these
#     are the only rows that see them.
#   * `t3/*` and `t4/*` -- `terminal_get_line_attributes`, which no
#     oracle in this tree read before B21-4.  Every `t3/attrs/*` row
#     must show a DIFFERENT attribute bit and the `t3/colors/*` rows
#     must distinguish `fg_indexed` from a resolved `foreground`: that
#     distinction is the `color_set` branch, i.e. whether the UI or the
#     editor owns the palette entry.  t4 answers the same corpus with a
#     UI ATTACHED (`rgb = true`, and again under `'termguicolors'`); at
#     the baseline t4 and t3 AGREE cell for cell, and that agreement is
#     the assertion -- `ui_rgb_attached()` is read by the code that
#     consumes `term_attrs`, and if a rewrite makes the answer depend
#     on it, these rows diverge first.
#   * `t5/key/*` must answer a DIFFERENT escape per key and
#     `t5/mouse/{x10,move,sgr,wheel}` four different encodings.  This
#     section is the only gate anywhere on terminal-mode input:
#     `nvim_open_term`'s `on_input` callback records the exact bytes a
#     child process would have received.  `t5/mode/*` is the only gate
#     on `terminal_enter`/`terminal_leave` (`mode=t` vs `nt`, the
#     `TermEnter`/`TermLeave` ORDER, and `<C-\><C-o>`'s one shot).
#   * `t6/reply/*` must stay six different byte strings, and
#     `t6/theme/true` must differ from `t6/theme/false`
#     (`terminal_notify_theme` answers only a terminal that asked, via
#     DECSET 2031).
#   * `t91 groups cases=19 aborted=0`.  BOTH of the aborts this sweep
#     was born with have been fixed, so ANY abort at all is now a
#     regression.
#       - `onecol-wide`: FIXED AT B21-9 (`c613f92ab5`), and this row is
#         why the exception set above exists.  A terminal ONE COLUMN
#         wide fed a double-width glyph used to die at
#         `vterm/screen.rs:136`: `putglyph`'s continuation loop
#         dereferenced the NULL `getcell` answers past the last column,
#         and `v0.12.4`'s `src/nvim/vterm/screen.c` has the identical
#         unguarded write -- UPSTREAM SEGFAULTS TOO, from a plain
#         `:terminal` in a `:vertical resize 1` window printing one CJK
#         character.  Filed as
#         `~/agents/context/1786212071-upstream-neovim-bugs/vterm-screen-putglyph-narrow-null-deref.md`;
#         the port carries the fix, upstream does not.  The row now
#         reads `alive=true res=ok said=`.
#       - `palette-unset`: FIXED AT B21-10 (`d139fba127`), the second
#         row of the exception set.  `let g:terminal_color_1 =
#         '#00ff88'`, open a terminal, then unset it -> double free.
#         `get_config_string` reads the variable through
#         `dict_get_value` with `reuse_strdata = true`, so the
#         `Object`'s string data POINTS AT THE VARIABLE'S OWN BYTES,
#         and `terminal_open` then called `xfree` on it.  **This one
#         was PORT-SIDE** -- upstream's `terminal.c` does not free it;
#         the free arrived with `adb3b9093e`, "Graduate terminal.rs",
#         whose doc comment asserted the wrong ownership.  Nothing was
#         filed upstream.  The row now reads `alive=true res=ok said=`.
#         (A RELATED defect in the same function IS upstream's and is
#         NOT fixed: `api_free_object` on a non-string
#         `g:terminal_color_N` frees the same borrowed bytes.  Filed as
#         `terminal-get-config-string-frees-borrowed-strings.md`; no
#         probe covers it.)
#
# TAKES ~15 s in the sweep plus the build.  Twenty-four `--embed`
# children (t3's palette case, t4, t5, t6, and one per t91 probe), all
# `jobstop`ped and `jobwait`ed; one of them attaches a UI over a RAW
# socket.  If a run is killed, look for children with ppid 1 and KILL
# THEM BY PID -- never by pattern.
#
# When a slice *adds* cases, land the additions against unchanged
# behaviour first (0 removed lines, N added, which is itself the proof
# that they are pure), re-baseline, and only then land the behaviour
# change.  No label here is a process-global counter and every mouse
# case builds its own terminal, so an inserted case does not renumber
# the rows below it.
#
# Regenerate ONLY when a behaviour change is *intended* and reviewed -- and
# regeneration is now a BASE BUMP, not a re-cut in place.  Write the new
# commit into `test/battery/BASE`, in a commit of its own whose body says
# what moved; the cache under target/battery/base/ is keyed by that sha, so
# every row re-cuts itself against the new binary on the next run.  See
# README.md.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
OUT=${SWEEP_OUT:-/tmp/termsweep-out}
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
"$HERE/termsweep.sh" "$NVIM_BIN" "$REPO/runtime" \
  "$OUT" "$LABEL" 2>&1 | tail -1

if [[ -n $CUT ]]; then exit 0; fi

# The baseline is CUT, not committed: `baseline.sh` runs this same sweep
# against the binary `test/battery/BASE` pins and caches the result under
# target/battery/base/<sha>/.  The first row to want it pays for the
# reference build.  See README.md.
BASELINE=${TERM_BASELINE:-$("$HERE/baseline.sh" term)}
fail=0
for part in txt struct stderr; do
  if diff -q "$BASELINE/base.$part" "$OUT/$LABEL.$part" >/dev/null; then
    echo "$part: IDENTICAL"
  else
    echo "$part: DIFFERS"
    # -a: the report escapes high bytes, and diff would otherwise call
    # it binary and print nothing useful.
    # `|| true`: `set -e` plus `pipefail` would abort on the first
    # differing artifact and the ones that say *which* layer moved
    # would never be compared.
    diff -a "$BASELINE/base.$part" "$OUT/$LABEL.$part" | head -60 || true
    fail=1
  fi
done
exit $fail
