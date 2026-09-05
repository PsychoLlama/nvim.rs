#!/usr/bin/env bash
# Differential oracle for the Ex-command API surface (batch B17):
# nvim_parse_cmd() and nvim_cmd(), and through them ex_docmd's parser,
# ex_cmds.rs, ex_cmds2.rs, ex_eval.rs, ex_session.rs, usercmd.rs,
# debugger.rs, help.rs, digraph.rs, cmdhist.rs and arglist.
#
#   cmdsweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# The largest API surface that had only the functional suite behind it.
# Every case is an *Ex command line*: it goes through nvim_parse_cmd, and
# where it is safe to run, the parse Dict goes on into nvim_cmd and the
# whole observable effect is recorded -- captured output, the error, the
# scratch buffer, the cursor, the changedtick delta, and whatever extra
# question the case asked.
#
# Produces, under <outdir>:
#
#   <label>.txt       canonical report, read top to bottom and diffed
#                     as-is.  One or more tagged lines per case:
#                       P  the parse Dict, flattened to one line
#                       !  the error a parse or an execution raised
#                       O  what nvim_cmd captured with output = true
#                       B  the scratch buffer afterwards
#                       S  cursor / tick delta / mode / window shape
#                       A  the case's own extra question
#   <label>.struct    canonical (sorted-key) JSON, one line per labelled
#                     answer, LC_ALL=C sorted AFTER the scrub.  The
#                     report flattens and escapes; this keeps the shape,
#                     and sorting makes the diff independent of where in
#                     the run an answer was produced.
#   <label>.stderr    what nvim wrote to the prompt.  In a headless
#                     process that is where messages land, and s20 runs a
#                     block of commands *uncaptured* with 'report' at 0
#                     so that ":N more lines", "N substitutions on N
#                     lines", the E-numbers and the :command / :digraphs
#                     / :history listings reach this artifact.  It is the
#                     only view of msg_* for this subsystem.
#
# SANDBOX.  Everything that touches the filesystem is confined to $WORK
# (default /tmp/cmdsweep, removed and recreated on every run): the cwd is
# $WORK, HOME is $WORK/home, and the only commands allowed to write are
# aimed at names under it.  Nothing outside $WORK is read or written, and
# `env -i` means the report cannot pick up the caller's locale, editor
# configuration or XDG directories.
#
# The nvim invocation is wrapped in `timeout` and its exit status is
# appended to the report as a final `exit <code>` line: 124 is the
# harness's verdict on a wedge, 134 an abort.  s91 runs the inputs that
# may kill the editor in a *child* each, so a crash there is one diffable
# ABORTED row rather than a truncated report.
#
# CMDSWEEP_ONLY is a Lua pattern matched against each section name; it
# exists for iterating on one section, not for gating.
# CMDSWEEP_TRACE=1 mirrors each section name to stderr, which is the only
# way to see where a wedged run stopped -- and must be off for a baseline,
# because it writes into the .stderr artifact.

set -euo pipefail

if [[ $# -ne 4 ]]; then
  sed -n '2,58p' "$0" >&2
  exit 2
fi

NVIM=$(realpath "$1")
RUNTIME=$(realpath "$2")
OUT=$(realpath -m "$3")
LABEL=$4
HERE=$(cd "$(dirname "$0")" && pwd)

# Short on purpose, and the same for every run: messages are truncated to
# the (headless, 80 column) screen and a long work directory leaves
# half-elided paths behind that the scrub cannot recognise.
WORK=${CMDSWEEP_WORK:-/tmp/cmdsweep}
LIMIT=${CMDSWEEP_TIMEOUT:-900}

mkdir -p "$OUT"
rm -rf "$WORK"
mkdir -p "$WORK/home" "$WORK/files"

# stdin, explicitly.  In `-l` script mode a case that ends up asking the
# real input stream reads whatever the caller's terminal has; an empty
# file makes that EOF, deterministically, on every machine.
: >"$WORK/empty"

set +e
timeout -k 5 "$LIMIT" \
  env -i \
  HOME="$WORK/home" \
  PATH=/usr/bin:/bin \
  TERM=dumb \
  SHELL=/bin/sh \
  LANG=C.UTF-8 \
  VIMRUNTIME="$RUNTIME" \
  XDG_CONFIG_HOME="$WORK/home/.config" \
  XDG_DATA_HOME="$WORK/home/.local/share" \
  XDG_STATE_HOME="$WORK/home/.local/state" \
  XDG_CACHE_HOME="$WORK/home/.cache" \
  NVIM_TEST=1 \
  CMD_WORK="$WORK" \
  CMD_STRUCT="$OUT/$LABEL.struct.raw" \
  CMDSWEEP_ONLY="${CMDSWEEP_ONLY:-}" \
  CMDSWEEP_TRACE="${CMDSWEEP_TRACE:-}" \
  "$NVIM" --headless -u NONE -i NONE \
  --cmd "cd $WORK" \
  -l "$HERE/cmdsweep.lua" \
  <"$WORK/empty" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr.raw"
status=$?
set -e

# Two scrubs the driver has to do rather than the Lua, because they land
# in messages the sweep never gets to touch:
#   * undo's own report carries a wall clock AND a monotonic change
#     number ("1 change; before #10767  0 seconds ago");
#   * the work directory reaches messages through file names, and the
#     binary under test names itself in a handful of them.
sed -E \
  -e "s#$WORK#<WORK>#g" \
  -e "s#$HERE/cmdsweep\.lua line [0-9]+#<SCRIPT> line N#g" \
  -e "s#\.\.\.[^ \"]*/cmdsweep\.lua line [0-9]+#<SCRIPT> line N#g" \
  -e "s#$HERE/cmdsweep\.lua#<SCRIPT>#g" \
  -e "s#\.\.\.[^ \"]*/cmdsweep\.lua#<SCRIPT>#g" \
  -e 's/#[0-9]+/#N/g' \
  -e 's/[0-9]+ (second|minute|hour|day)s? ago/N ago/g' \
  -e "s#$RUNTIME#<RUNTIME>#g" \
  -e 's#[^ ]*/target/debug/nvim#<NVIM>#g' \
  -e 's#[^ ]*/nvim-[0-9a-f]+#<NVIM>#g' \
  -e 's#[^ ]*/vim/_core/#<CORE>/#g' \
  -e 's#/tmp/nvim\.[a-zA-Z0-9_]+/[a-zA-Z0-9]+/nvim\.[0-9]+\.[0-9]+#<SERVER>#g' \
  <"$OUT/$LABEL.stderr.raw" >"$OUT/$LABEL.stderr"
rm -f "$OUT/$LABEL.stderr.raw"

printf 'exit %d\n' "$status" >>"$OUT/$LABEL.txt"

touch "$OUT/$LABEL.struct.raw"
LC_ALL=C sort "$OUT/$LABEL.struct.raw" >"$OUT/$LABEL.struct"
rm -f "$OUT/$LABEL.struct.raw"

printf '%s: report %d lines, struct %d lines, stderr %d lines, exit %d\n' \
  "$LABEL" "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.struct")" \
  "$(wc -l <"$OUT/$LABEL.stderr")" "$status"
