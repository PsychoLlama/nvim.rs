#!/usr/bin/env bash
# Build the working tree and diff its bufsweep against the stored
# baseline.  Run from anywhere; everything is absolute.
#
#   bufverify.sh [label]         # default label: cur
#
# The baseline is CUT, not committed: it comes from the binary
# `test/battery/BASE` pins, cached under target/battery/base/<sha>/.
# The row was first baselined at commit 46c4bf3a3a -- B19's close, the last revision before
# any of buffer.rs is carved or rewritten.  It is the *pre-rewrite*
# behaviour of `buflist_list` (the `:ls` renderer and its column-40
# pad), `chk_modeline`/`do_modelines`, `ExpandBufnames`, `fileinfo`,
# `buflist_findpat`/`buflist_match`/`fname_match`, `buflist_findnr`/
# `buflist_findname`/`buflist_findname_exp`/`buflist_findname_file_id`,
# `buflist_setfpos`/`buflist_findfmark`/`buflist_findlnum`,
# `buflist_new`/`open_buffer`/`close_buffer`/`buf_freeall`,
# `do_buffer_ext`/`do_bufdel`/`set_curbuf`/`enter_buffer`,
# `setfname`/`buf_set_name`/`buf_name_changed`/`setaltfname`,
# `bt_*`/`buf_spname`/`buf_get_fname` and `get_winopts`/`find_wininfo`.
#
# It was also run against ~/agents/scratch/b20-1/nvim-46c4bf3a3a (B20's
# pre-batch side), ~/agents/scratch/b17-19/nvim-b00f1ef7e0 (B18's) and
# ~/agents/scratch/p0-2/nvim-ed789235ab (phase 16's) and is IDENTICAL on
# all three, all three artifacts -- which is the proof that the buffer
# family has not moved since P0 and that there is no old-side exception
# set to carry.
#
# Regenerate ONLY when a behaviour change is *intended* and reviewed -- and
# regeneration is now a BASE BUMP, not a re-cut in place.  Write the new
# commit into `test/battery/BASE`, in a commit of its own whose body says
# what moved; the cache under target/battery/base/ is keyed by that sha, so
# every row re-cuts itself against the new binary on the next run.  See
# README.md.
#
# All three artifacts are compared.  `base.stderr` is EMPTY at the
# baseline and that is the assertion: the report and the canonical JSON
# are written by the sweep itself, so anything on the prompt is a
# message no case expected -- or a child that died where it should not
# have.
#
# THE REPORT CARRIES A `## <section> rows=N nbuf=N lastbuf=N` LINE PER
# SECTION.  Those counts are the standing assertion that no section
# went silently empty AND that no in-process section created or
# destroyed a buffer; at the baseline they are:
#
#   b0-defaults 48 · b1-lsflags 110 · b2-address 62 · b3-lifecycle 41 ·
#   b4-findpat 51 · b5-complete 57 · b6-fileinfo 102 · b7-modeline 72 ·
#   b8-bufinfo 35 · b9-auorder 28 · b91-crashprobe 26
#
# ... every one of them `nbuf=23 lastbuf=23`, summing to
# `## TOTAL rows=654` (the section rows plus the two `##` lines each
# section prints), and the artifacts are 656 / 631 / 0 lines -- the
# report adds `## TOTAL` and `exit 0`.  A `rows=0` with everything else
# unchanged is a harness bug, not a regression; an `nbuf=24` in one
# section and every section below it is a LEAK, and it renumbers the
# buffers those sections print.
#
# SEVEN ROWS ARE THE LOAD-BEARING ASSERTIONS.
#
#   * `b91 groups cases=25 aborted=0`.  NOTHING aborts at the baseline
#     -- unlike winsweep's four -- so any abort at all is a regression,
#     and a `:99999999bnext`, a `1,2147483647bdelete` or a 10,000-byte
#     modeline is where it would show.
#   * `b9/*` must carry `Buf...` events.  The sweep runs from a
#     `VimEnter` autocmd, and an autocommand fired while another
#     autocommand runs DOES NOT FIRE without `++nested` on the outer
#     one.  Without it every b9 row answers an EMPTY sequence and the
#     section looks perfectly healthy: right row count, no stderr, no
#     error.  If `b9/bdelete` ever reads `= rc=0 <tab>1:111`, the
#     `++nested` in bufsweep.sh or in `child()` has been lost.
#   * `b1/one/t` and `b1/plain/bang-t` must list `alpha.txt` FIRST and
#     then `modeline.txt`, and `b1/plain/bang-t` must then read
#     `helphelp.txt`, `[Quickfix List]` before everything with
#     `line 0`.  That order IS `buf_time_compare`, and the four
#     timestamps behind it exist only because the fixture spins until
#     the wall clock ticks between each of its four entering steps.  If
#     they ever tie, the order becomes glibc's business.
#   * `b5/wm/full_lastused/5` must answer `two.txt` and
#     `b5/wm/full/5` must answer `five.txt`.  That difference is
#     `ExpandBufnames`'s last block -- the `qsort` on `b_last_used` and
#     the rotation that moves the CURRENT buffer to the end -- and it
#     runs only under `WILD_BUFLASTUSED`, which only a real `<Tab>` at
#     a real command line with `'wildmode'` carrying `lastused` sets.
#     `getcompletion()` never reaches those thirty lines.
#   * `b7/mle/off-fde` must be E992 and `b7/mle/on-fde` must answer
#     `fde=1+1`.  An expression option in a modeline without
#     `'modelineexpr'` is E992 and reads exactly like a healthy row
#     (b19-2's trap), so both arms are explicit cases.
#   * `b6/help/file` must start with `<` (`msg_trunc` cut its head) and
#     `b6/help/ctrlg` must not.  `:file` is the `dont_truncate = false`
#     door into `fileinfo` and CTRL-G the `true` one; nothing else
#     separates them.
#   * `b3/badd-unlisted` must show buffer 6 in the PLAIN `:ls` and
#     `b9/badd-unlisted` must read `BufAdd:6 > BufCreate:6` with NO
#     `BufNew`.  Those two are the only gesture in the sweep that
#     reaches `buflist_new`'s `BLN_LISTED` re-listing block -- `:edit`
#     on a `:bdelete`d buffer looks like it should and does not,
#     because `close_buffer` cleared `b_p_initialized` and the
#     `buf_copy_options()` call above that block re-lists from the
#     global first.  Measured: without them the anchor is NOT CAUGHT.
#
# TWO THINGS ABOUT THE WORK DIRECTORY ARE LOAD-BEARING, and both are
# in bufsweep.sh: it is a CONSTANT LENGTH (`buflist_list` pads to
# column 40 using the real path, before any scrub) and it is DIGITS
# ONLY (`ExpandBufnames`/`buflist_findpat` match against `b_ffname`,
# which contains it -- a `mktemp` suffix carrying a `d` makes
# `getcompletion('d','buffer')` answer every buffer in the fixture).
#
# ONE THING OUTSIDE THE SANDBOX IS LOAD-BEARING: $VIMRUNTIME's length.
# `b6/help/file` is truncated by `msg_trunc`, which cuts to the screen
# width -- so the surviving text depends on how long the runtime path
# is.  Always pass `$REPO/runtime`, which is what this script does and
# what the three side runs used.
#
# The report's final `exit N` line is the hang/crash assertion -- 124 is
# the harness timeout and 134 an abort.
#
# TAKES ~12 s, about four of which are the fixture's four clock spins
# and the rest ninety-odd child processes (b3, b5's `'wildmode'` block,
# b9, b91).
#
# When a slice *adds* cases, land the additions against unchanged
# behaviour first (0 removed lines, N added, which is itself the proof
# that they are pure), re-baseline, and only then land the behaviour
# change -- so that the second delta is nothing but the behaviour.
# Buffer numbers ARE printed raw, on purpose (they are what `:ls`
# shows); what keeps an inserted case from renumbering everything below
# it is that the in-process sections never create a buffer and every
# churning section runs one fresh child per case.  Keep it that way.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
OUT=${SWEEP_OUT:-/tmp/bufsweep-out}
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
"$HERE/bufsweep.sh" "$NVIM_BIN" "$REPO/runtime" \
  "$OUT" "$LABEL" 2>&1 | tail -1

if [[ -n $CUT ]]; then exit 0; fi

# The baseline is CUT, not committed: `baseline.sh` runs this same sweep
# against the binary `test/battery/BASE` pins and caches the result under
# target/battery/base/<sha>/.  The first row to want it pays for the
# reference build.  See README.md.
BASELINE=${BUF_BASELINE:-$("$HERE/baseline.sh" buf)}
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
