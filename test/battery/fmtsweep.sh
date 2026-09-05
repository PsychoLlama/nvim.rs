#!/usr/bin/env bash
# Differential oracle for the indent and format layers (batch B15):
# indent_c.rs (4,308 lines), textformat.rs (1,153) and indent.rs (683).
#
#   fmtsweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# `get_c_indent` alone is 1,866 lines -- the largest single function in
# the batch, and the one item the plan says must decompose.  `test_cindent`
# is the only thing that covers it (5,516 lines, no screendumps, so it is
# a real gate); nothing at all is a fine-grained gate for `gq`,
# 'formatoptions', 'comments', 'formatlistpat' or the
# 'formatexpr'/'indentexpr' callbacks.  No existing differential reaches
# any of the three files.
#
# Every case builds a named fixture under one option set and reports
# either an indentation *profile* -- indent()/cindent()/lispindent() for
# every line, which asks the engines without changing anything -- or the
# buffer that a key sequence produced.  The two halves are deliberately
# separate: `cindent(l)` reads the lines above `l` as they are, so an
# already-indented buffer is a different question from a flat one, and
# s01 asks both.
#
# Produces, under <outdir>:
#
#   <label>.txt       canonical report, one line per probe, in a fixed
#                     emission order, read top to bottom.
#   <label>.struct    canonical (sorted-key) JSON, one line per labelled
#                     answer, then LC_ALL=C sorted.  Sorted AFTER the
#                     scrub, never before.
#   <label>.stderr    what nvim wrote to the prompt.  s90 runs the error
#                     arms uncaptured (E518/E521/E474 and the
#                     'equalprg'/'formatprg' shell-out failures), so this
#                     artifact carries signal rather than being empty.
#
# There is no `.bin`: unlike utfsweep, nothing here answers in raw bytes
# -- an indent is a number and a formatted line is text the report's
# escaper reproduces exactly.
#
# EXTERNAL PROGRAMS.  s90 sets 'equalprg' and 'formatprg' to a path that
# does not exist, on purpose: the message that produces is the artifact.
# Nothing in the sweep runs a *real* filter, so the report does not
# depend on the host having one.  `env -i` leaves PATH at /usr/bin:/bin
# and SHELL is set explicitly so that the failure message is the same on
# every machine.
#
# The nvim invocation is wrapped in `timeout`, and its exit status is
# appended to the report as a final `exit <code>` line.  124 is the
# harness's verdict on a wedge; 134 an abort.
#
# FMTSWEEP_ONLY is a Lua pattern matched against each section name; it
# exists for iterating on one section, not for gating.
# FMTSWEEP_TRACE=1 mirrors each section name to stderr, which is the
# only way to see where a wedged run stopped.

set -euo pipefail

if [[ $# -ne 4 ]]; then
  sed -n '2,55p' "$0" >&2
  exit 2
fi

NVIM=$(realpath "$1")
RUNTIME=$(realpath "$2")
OUT=$(realpath -m "$3")
LABEL=$4
HERE=$(cd "$(dirname "$0")" && pwd)

# Short on purpose, and the same for every run: absolute paths reach the
# report through error texts, and a long work directory leaves
# half-elided paths behind that the scrub cannot match.
WORK=${FMTSWEEP_WORK:-/tmp/fmtsweep}
LIMIT=${FMTSWEEP_TIMEOUT:-1800}

mkdir -p "$OUT"
rm -rf "$WORK"
mkdir -p "$WORK/home" "$WORK/tmp" "$WORK/bin" "$WORK/files"

for tool in sh; do
  if p=$(command -v "$tool" 2>/dev/null); then
    ln -sf "$(realpath "$p")" "$WORK/bin/$tool"
  fi
done

# stdin, explicitly.  In `-l` script mode a case that ends up asking the
# real input stream reads whatever the caller's terminal has; an empty
# file makes that EOF, deterministically, on every machine.
: >"$WORK/empty"

# `env -i` so the report cannot pick up the caller's locale, editor
# configuration or XDG directories.
set +e
timeout -k 5 "$LIMIT" \
  env -i \
  HOME="$WORK/home" \
  PATH="$WORK/bin:/usr/bin:/bin" \
  TERM=dumb \
  SHELL=/bin/sh \
  LANG=C.UTF-8 \
  TMPDIR="$WORK/tmp" \
  VIMRUNTIME="$RUNTIME" \
  XDG_CONFIG_HOME="$WORK/home/.config" \
  XDG_DATA_HOME="$WORK/home/.local/share" \
  XDG_STATE_HOME="$WORK/home/.local/state" \
  XDG_CACHE_HOME="$WORK/home/.cache" \
  NVIM_TEST=1 \
  FMT_WORK="$WORK" \
  FMT_STRUCT="$OUT/$LABEL.struct.raw" \
  FMTSWEEP_ONLY="${FMTSWEEP_ONLY:-}" \
  FMTSWEEP_TRACE="${FMTSWEEP_TRACE:-}" \
  "$NVIM" --headless -u NONE -i NONE \
  -l "$HERE/fmtsweep.lua" \
  <"$WORK/empty" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr.raw"
status=$?
set -e

# The scrubs the driver has to do rather than the Lua: a temp file name
# carries six random characters and a monotonic counter, and reaches
# stderr through the 'equalprg'/'formatprg' shell-out failures.  Undo's
# messages carry a wall clock and a monotonic counter that nothing
# in-process can reach.
sed -E \
  -e "s#$WORK#<WORK>#g" \
  -e 's#<WORK>/tmp/nvim\.[^/]*/[A-Za-z0-9]+/[0-9]+#<TMPFILE>#g' \
  -e 's#<WORK>/tmp/nvim\.[^/]*/[A-Za-z0-9]+#<TMPDIR>#g' \
  -e 's/#[0-9]+/#N/g' \
  -e 's/[0-9]+ (second|minute|hour|day)s? ago/N ago/g' \
  <"$OUT/$LABEL.stderr.raw" >"$OUT/$LABEL.stderr"
rm -f "$OUT/$LABEL.stderr.raw"

printf 'exit %d\n' "$status" >>"$OUT/$LABEL.txt"

touch "$OUT/$LABEL.struct.raw"
LC_ALL=C sort "$OUT/$LABEL.struct.raw" >"$OUT/$LABEL.struct"
rm -f "$OUT/$LABEL.struct.raw"

printf '%s: report %d lines, struct %d lines, stderr %d lines, exit %d\n' \
  "$LABEL" "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.struct")" \
  "$(wc -l <"$OUT/$LABEL.stderr")" "$status"
