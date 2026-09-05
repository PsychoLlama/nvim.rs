#!/usr/bin/env bash
# Differential oracle for the mouse family (batch B19): mouse.rs --
# `do_mouse` (695 lines), `jump_to_mouse` (370), `ins_mouse`,
# `do_mousescroll`/`ins_mousescroll`, `mouse_comp_pos`, `vcol2col`,
# `mouse_find_win_*`, `mouse_check_grid` and `f_getmousepos`.
#
#   mousesweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# The gap this closes: B19's survey found mouse input absent from EVERY
# differential -- `nvim_input_mouse` appears once in all of evalsweep,
# `getmousepos` once in keysweep, and scrsweep and opsweep have zero.
# 1,065 lines of `do_mouse` + `jump_to_mouse` were behind one functional
# spec.
#
# THE WHOLE SWEEP RUNS IN AN `--embed` CHILD, and that is not a choice.
# A mouse key is dispatched by the MAIN INPUT LOOP: in this `-l` parent
# `nvim_input_mouse` only parks `<80><fd>,` in the typeahead (`getchar(0)`
# fishes it straight back out) and `do_mouse` never runs -- the same trap
# stlsweep's `%@` handlers hit (B19-2).  So the parent is a driver: it
# starts a child per section over `jobstart(rpc = true)`, sends
# `nvim_input_mouse` / `nvim_input`, and reads the answer back with
# `nvim_exec_lua`.  NO UI IS ATTACHED except in s12, which needs one to
# turn `ext_multigrid` on; attaching an RPC-visible UI to a parent with
# no `redraw` handler kills the channel, so s12's UI client is a RAW
# (`rpc = false`) socket channel that writes fourteen hand-encoded
# msgpack bytes and discards everything the child sends back.
#
# TWO API CALLS ARE FAST AND ONE IS NOT.  `nvim_input` and
# `nvim_input_mouse` are dispatched immediately; `nvim_exec_lua` is
# deferred to the main loop.  `pum_show_popupmenu` runs a bare
# `vgetc()` loop with no `K_EVENT` arm, so while a `'mousemodel'=popup`
# menu is up the child answers NO `nvim_exec_lua` at all -- s4 drives
# the menu with `nvim_input` and only queries once it is down.
#
# Produces, under <outdir>:
#
#   <label>.txt       canonical report, diffed as-is.  One tagged line
#                     per case:
#                       =  the editor state the events produced
#                       A  a case's own extra question
#                       X  one crashprobe child's verdict
#   <label>.struct    canonical (sorted-key) JSON, one line per case,
#                     LC_ALL=C sorted AFTER the scrub.
#   <label>.stderr    what the parent and its children wrote to the
#                     prompt.
#
# SANDBOX.  Everything is confined to $WORK, a fresh `mktemp -d` unless
# MOUSESWEEP_WORK says otherwise: cwd is $WORK, $HOME is $WORK/home,
# $TMPDIR is $WORK/tmp and $PATH is pinned to $WORK/bin.  `env -i` means
# nothing else leaks in.  THE BINARY IS COPIED TO $WORK/bin/nvim,
# because every section spawns a child and it must find one inside the
# sandbox.
#
# THE WORK DIRECTORY IS A CONSTANT LENGTH (`/tmp/mousesweep.XXXXXXXXXX`)
# and s12's listening socket lives inside it.  Two runs whose $WORK
# differ only in CONTENT are comparable and proving that is the
# acceptance test (mouseverify.sh); two whose $WORK differ in LENGTH are
# not.
#
# DETERMINISM.  Multi-click detection compares the wall clock against
# `'mousetime'`, so EVERY case pins it: 0 makes no click a continuation
# of the previous one, 100000 makes every click one, and the difference
# between the two IS the click counter.  A case that left it at the
# default 500 would answer differently on a loaded machine.  Click-count
# state is also positional, so a section resets it explicitly (a click
# far away, or `'mousetime'` 0) between cases.
#
# MOUSESWEEP_ONLY is a Lua pattern matched against each section name; it
# exists for iterating on one section, not for gating.
# MOUSESWEEP_TRACE=1 mirrors section names to stderr and must be off for
# a baseline, because it writes into the .stderr artifact.

set -euo pipefail

if [[ $# -ne 4 ]]; then
  sed -n '2,70p' "$0" >&2
  exit 2
fi

NVIM=$(realpath "$1")
RUNTIME=$(realpath "$2")
OUT=$(realpath -m "$3")
LABEL=$4
HERE=$(cd "$(dirname "$0")" && pwd)

LIMIT=${MOUSESWEEP_TIMEOUT:-900}

# Constant length, always: `/tmp/mousesweep.` + ten characters.
OWNED=0
if [[ -n ${MOUSESWEEP_WORK:-} ]]; then
  WORK=$MOUSESWEEP_WORK
  rm -rf "$WORK"
  mkdir -p "$WORK"
else
  WORK=$(mktemp -d /tmp/mousesweep.XXXXXXXXXX)
  OWNED=1
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
  MOUSE_WORK="$WORK" \
  MOUSE_STRUCT="$OUT/$LABEL.struct.raw" \
  MOUSESWEEP_ONLY="${MOUSESWEEP_ONLY:-}" \
  MOUSESWEEP_TRACE="${MOUSESWEEP_TRACE:-}" \
  "$WORK/bin/nvim" --headless -u NONE -i NONE \
  --cmd "cd $WORK" \
  -l "$HERE/mousesweep.lua" \
  <"$WORK/empty" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr.raw"
status=$?
set -e

sed -E \
  -e "s#$WORK#<WORK>#g" \
  -e "s#$HERE/mousesweep\.lua#<SCRIPT>#g" \
  -e "s#\.\.\.[^ \"]*/mousesweep\.lua#<SCRIPT>#g" \
  -e "s#$RUNTIME#<RT>#g" \
  -e 's#nvim\.[0-9]+\.[0-9]+#nvim.<PID>.<SEQ>#g' \
  -e 's#nvim\.[A-Za-z0-9_.-]+/[A-Za-z0-9]+#nvim.<U>/<T>#g' \
  <"$OUT/$LABEL.stderr.raw" >"$OUT/$LABEL.stderr"
rm -f "$OUT/$LABEL.stderr.raw"

printf 'exit %d\n' "$status" >>"$OUT/$LABEL.txt"

touch "$OUT/$LABEL.struct.raw"
LC_ALL=C sort "$OUT/$LABEL.struct.raw" >"$OUT/$LABEL.struct"
rm -f "$OUT/$LABEL.struct.raw"

if [[ $OWNED -eq 1 && -z ${MOUSESWEEP_KEEP:-} ]]; then
  rm -rf "$WORK"
fi

printf '%s: report %d lines, struct %d lines, stderr %d lines, exit %d\n' \
  "$LABEL" "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.struct")" \
  "$(wc -l <"$OUT/$LABEL.stderr")" "$status"
