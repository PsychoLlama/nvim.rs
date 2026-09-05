#!/usr/bin/env bash
# Differential oracle for the buffer family (batch B20): buffer.rs --
# `buflist_list` (the `:ls` renderer), `chk_modeline`, `ExpandBufnames`,
# `fileinfo`, `buflist_findpat`, `buflist_findname`/`_exp`/`_file_id`,
# `buflist_findnr`/`buflist_nr2name`, `buflist_setfpos`/
# `buflist_findfmark`/`buflist_findlnum`, `buflist_new`, `open_buffer`,
# `close_buffer`, `buf_freeall`/`free_buffer`, `do_buffer_ext`/
# `do_bufdel`/`set_curbuf`/`enter_buffer`, `setfname`/`buf_set_name`/
# `buf_name_changed`/`setaltfname`, `bt_*`/`buf_spname`/`buf_get_fname`,
# `do_modelines`, `ex_buffer_all` and `get_winopts`/`find_wininfo`.
#
#   bufsweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# The gap this closes: B20's survey S5, hole 3.  Across all EIGHTEEN
# existing differentials `:ls`/`:buffers` appeared only as a NAME in
# the ex probe and as a PARSE TARGET in cmdsweep -- no oracle read its
# output; `chk_modeline`'s 180 lines were reached only incidentally by
# `'formatoptions'` fixtures; `CTRL-G` appeared in five bench/probe
# files and no differential; `ExpandBufnames` and `buflist_findpat`
# were untested.  ~850 lines of pure string behaviour, ungated.
#
# TEXTUAL, ONE PROCESS FOR THE BULK, AND *NOT* `-l`.  Everything this
# family does that can be observed is a string (`:ls`'s six flag
# columns and its 40-column pad, `fileinfo`'s message, an error number)
# or a small number (a bufnr, a triple of `bufexists`/`buflisted`/
# `bufloaded`).  So the bulk runs in a single `--headless` process
# driven from a `VimEnter` autocmd, exactly as stlsweep, menusweep and
# winsweep do.  `-l` is avoided on b20-2's rule: `-l` sets
# `silent_mode`, `full_screen` is `!silent_mode`, and whole arms of an
# oracle vanish under it without any sign that they did.
#
# BUFFER NUMBERS ARE PRINTED RAW.  winsweep renumbers every window
# handle because a window id is an invisible process-global counter; a
# buffer number is the opposite -- `:ls` prints it in column one,
# `2CTRL-G` prints it, `bufnr()` returns it, and hiding it would stop
# the oracle gating the one thing a user reads.  Determinism is bought
# structurally instead:
#
#   * the in-process sections share ONE fixture built before any
#     section runs, and none of them may create or destroy a buffer.
#     Every `## <section>` line carries `nbuf=` and `lastbuf=`, so a
#     section that leaks a buffer is a one-line diff instead of a
#     silent renumbering of every section below it.
#   * b3 (lifecycle), b9 (autocmd order) and b91 (crashprobe) run ONE
#     FRESH CHILD PROCESS PER CASE.  Every child starts at bufnr 1, so
#     an inserted case renumbers nothing and a case that kills the
#     editor takes only itself.
#
# `b_last_used` IS A WALL CLOCK and `:ls t` sorts on it, with
# `buf_time_compare` answering 0 for a tie -- so two buffers entered in
# the same second sort in whatever order glibc's qsort lands on, and
# whether they tie at all depends on where the run falls inside a
# second.  The fixture therefore separates every ENTERING step with a
# spin until the clock ticks; four buffers get four strictly ordered
# timestamps and every other buffer has 0.  The rendered time is
# scrubbed (`undo_fmt_time` prints "N seconds ago" under 100 s and
# `%H:%M:%S` above it), so what the artifact keeps is the ORDER.  Those
# four spins are ~4 s of the sweep's ~12 s; the other 8 s are the
# ninety-odd child processes of b3, b5's `'wildmode'` block, b9 and b91.
#
# Produces, under <outdir>:
#
#   <label>.txt       canonical report, diffed as-is.  One tagged line
#                     per case:
#                       =  a case: what the command printed, then the
#                          answer (a `:ls` rendering, a triple set, a
#                          `getbufinfo` dict, an autocmd sequence)
#   <label>.struct    canonical (sorted-key) JSON, one line per case,
#                     LC_ALL=C sorted AFTER the scrub.  Carries every
#                     field the report compresses.
#   <label>.stderr    what the process and its children wrote to the
#                     prompt.  EMPTY at the baseline, and that is the
#                     assertion.
#
# SANDBOX.  Everything is confined to $WORK, a fresh `mktemp -d` unless
# BUFSWEEP_WORK says otherwise: cwd is $WORK, $HOME is $WORK/home,
# $TMPDIR is $WORK/tmp and $PATH is pinned to $WORK/bin.  `env -i` means
# nothing else leaks in.  THE BINARY IS COPIED TO $WORK/bin/nvim,
# because b3/b9/b91 spawn children and they must find one inside the
# sandbox.
#
# THE WORK DIRECTORY IS A CONSTANT LENGTH *AND DIGITS ONLY*
# (`/tmp/bufsweep.` + nine digits), and both halves are load-bearing.
# `buflist_list` pads each row to column 40 with
# `40 - vim_strsize(IObuff)`, computed on the REAL path before any
# scrub can see it -- so two runs whose $WORK differ in LENGTH are not
# comparable.  And `ExpandBufnames`/`buflist_findpat` match a pattern
# against `b_ffname` too, which CONTAINS the work directory -- so a
# `mktemp` suffix carrying a `d` makes `getcompletion('d','buffer')`
# answer every buffer in the fixture.  Digits collide with no pattern
# this sweep uses.
#
# `++nested` IS LOAD-BEARING AND WAS THE SLICE'S MOST EXPENSIVE BUG.
# The sweep runs from a `VimEnter` autocmd, and an autocommand
# triggered while another autocommand is executing DOES NOT FIRE unless
# the outer one is `++nested`.  Without it the whole b9 section
# answered an EMPTY event sequence for every one of its twenty-seven
# gestures and looked exactly like a healthy section: `rows=27`, no
# stderr, no error.  Both the parent and every child carry it.
#
# BUFSWEEP_ONLY is a Lua pattern matched against each section name; it
# exists for iterating on one section, not for gating.
# BUFSWEEP_TRACE=1 mirrors section names to stderr and must be off for
# a baseline, because it writes into the .stderr artifact.

set -euo pipefail

if [[ $# -ne 4 ]]; then
  sed -n '2,110p' "$0" >&2
  exit 2
fi

NVIM=$(realpath "$1")
RUNTIME=$(realpath "$2")
OUT=$(realpath -m "$3")
LABEL=$4
HERE=$(cd "$(dirname "$0")" && pwd)

LIMIT=${BUFSWEEP_TIMEOUT:-900}

# Constant length AND DIGITS ONLY: `/tmp/bufsweep.` + nine digits.
#
# `mktemp -d` draws its suffix from [0-9A-Za-z], and that is not good
# enough here.  `ExpandBufnames` and `buflist_findpat` match a pattern
# against `b_ffname` as well as `b_sfname`, and `b_ffname` contains the
# work directory -- so a run whose random suffix happened to contain a
# `d` answered `getcompletion('d', 'buffer')` with EVERY buffer while
# the next run answered four.  Two runs then differ for a reason that
# has nothing to do with the editor.  Digits cannot collide with any
# letter pattern the sweep uses, and nine of them keep the length
# constant, which `buflist_list`'s column-40 pad needs.
OWNED=0
if [[ -n ${BUFSWEEP_WORK:-} ]]; then
  WORK=$BUFSWEEP_WORK
  rm -rf "$WORK"
  mkdir -p "$WORK"
else
  WORK=
  for _ in 1 2 3 4 5 6 7 8 9 10; do
    try=$(printf '/tmp/bufsweep.%09d' "$(((RANDOM * 32768 + RANDOM) % 1000000000))")
    if mkdir "$try" 2>/dev/null; then
      WORK=$try
      OWNED=1
      break
    fi
  done
  if [[ -z $WORK ]]; then
    echo "bufsweep: cannot create a work directory" >&2
    exit 1
  fi
fi

mkdir -p "$OUT"
umask 022
mkdir -p "$WORK/home" "$WORK/bin" "$WORK/tmp"
chmod 755 "$WORK" "$WORK/home" "$WORK/bin" "$WORK/tmp"

cp "$NVIM" "$WORK/bin/nvim"
chmod 755 "$WORK/bin/nvim"

: >"$WORK/empty"

set +e
timeout -k 5 "$LIMIT" \
  env -i \
  HOME="$WORK/home" \
  PATH="$WORK/bin" \
  TMPDIR="$WORK/tmp" \
  TERM=dumb \
  SHELL=/bin/sh \
  LANG=C.UTF-8 \
  VIMRUNTIME="$RUNTIME" \
  XDG_CONFIG_HOME="$WORK/home/.config" \
  XDG_DATA_HOME="$WORK/home/.local/share" \
  XDG_STATE_HOME="$WORK/home/.local/state" \
  XDG_CACHE_HOME="$WORK/home/.cache" \
  NVIM_TEST=1 \
  BUF_WORK="$WORK" \
  BUF_LUA="$HERE/bufsweep.lua" \
  BUF_STRUCT="$OUT/$LABEL.struct.raw" \
  BUFSWEEP_ONLY="${BUFSWEEP_ONLY:-}" \
  BUFSWEEP_TRACE="${BUFSWEEP_TRACE:-}" \
  "$WORK/bin/nvim" --headless -u NONE -i NONE \
  --cmd "cd $WORK" \
  --cmd 'autocmd VimEnter * ++once ++nested lua local ok, e = pcall(dofile, vim.env.BUF_LUA) if not ok then io.stderr:write("SWEEP-FATAL " .. tostring(e) .. "\n") end vim.cmd("qa!")' \
  <"$WORK/empty" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr.raw"
status=$?
set -e

sed -E \
  -e "s#$WORK#<WORK>#g" \
  -e "s#$HERE/bufsweep\.lua#<SCRIPT>#g" \
  -e "s#\.\.\.[^ \"]*/bufsweep\.lua#<SCRIPT>#g" \
  -e "s#$RUNTIME#<RT>#g" \
  -e 's#nvim\.[0-9]+\.[0-9]+#nvim.<PID>.<SEQ>#g' \
  -e 's#nvim\.[A-Za-z0-9_.-]+/[A-Za-z0-9]+#nvim.<U>/<T>#g' \
  <"$OUT/$LABEL.stderr.raw" >"$OUT/$LABEL.stderr"
rm -f "$OUT/$LABEL.stderr.raw"

printf 'exit %d\n' "$status" >>"$OUT/$LABEL.txt"

touch "$OUT/$LABEL.struct.raw"
LC_ALL=C sort "$OUT/$LABEL.struct.raw" >"$OUT/$LABEL.struct"
rm -f "$OUT/$LABEL.struct.raw"

if [[ $OWNED -eq 1 && -z ${BUFSWEEP_KEEP:-} ]]; then
  rm -rf "$WORK"
fi

printf '%s: report %d lines, struct %d lines, stderr %d lines, exit %d\n' \
  "$LABEL" "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.struct")" \
  "$(wc -l <"$OUT/$LABEL.stderr")" "$status"
