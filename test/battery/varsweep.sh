#!/usr/bin/env bash
# Differential oracle for the variable layer (batch B14): eval/vars.rs.
#
#   varsweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# `evalsweep` asks what a *value* renders as and never issues a `:let`;
# this asks what `:let`, `:unlet`, `:const`, `:lockvar`, the scope
# dictionaries and the buffer/window/tab accessors *do*.  The two are
# disjoint on purpose and both are gates for the vars.rs slices.
#
# Produces, under <outdir>:
#
#   <label>.txt       canonical report -- one or two lines per case, in
#                     a fixed emission order, read top to bottom and
#                     diffed as-is.
#   <label>.struct    canonical (sorted-key) JSON, one line per labelled
#                     answer, then LC_ALL=C sorted.  The readable report
#                     escapes and flattens; this one keeps the shape,
#                     and sorting it makes the diff independent of where
#                     in the run an answer was produced.  Sorted AFTER
#                     the scrub, never before -- a path is exactly the
#                     kind of answer that differs between two working
#                     directories and survives the three-runs rule.
#   <label>.stderr    what nvim wrote to the prompt.  In a headless
#                     process that is where messages land; the `:let`
#                     listing and several of the lock errors reach only
#                     this artifact.
#
# Run once per binary, then `diff` the three <label> outputs.  Both runs
# use the same fixed (short) work directory: absolute paths reach the
# report through error texts, and v:progpath names the binary under
# test, which is a *different* path on each side by construction (the
# driver scrubs it).
#
# The nvim invocation is wrapped in `timeout`, and its exit status is
# appended to the report as a final `exit <code>` line.  124 is the
# harness's verdict on a wedge -- a lock walk that recurses through a
# self-referencing container is exactly that shape.
#
# VARSWEEP_ONLY is a Lua pattern matched against each section name; it
# exists for iterating on one section, not for gating.
# VARSWEEP_TRACE=1 mirrors each section name to stderr, which is the
# only way to see where a wedged run stopped.

set -euo pipefail

if [[ $# -ne 4 ]]; then
  sed -n '2,42p' "$0" >&2
  exit 2
fi

NVIM=$(realpath "$1")
RUNTIME=$(realpath "$2")
OUT=$(realpath -m "$3")
LABEL=$4
HERE=$(cd "$(dirname "$0")" && pwd)

# Short on purpose, and the same for every run: messages are truncated
# to the (headless, 80 column) screen and a long work directory leaves
# half-elided paths behind that the scrub cannot recognise.
WORK=${VARSWEEP_WORK:-/tmp/varsweep}
LIMIT=${VARSWEEP_TIMEOUT:-180}

mkdir -p "$OUT"
rm -rf "$WORK"
mkdir -p "$WORK/home"

printf 'alpha beta gamma\nsecond line here\nthird line\n' >"$WORK/one.txt"
# stdin, explicitly.  In `-l` script mode a case that ends up asking the
# real input stream reads whatever the caller's terminal has; an empty
# file makes that EOF, deterministically, on every machine.
: >"$WORK/empty"

# `env -i` so the report cannot pick up the caller's locale, editor
# configuration or XDG directories.  SHELL is /bin/sh: this is NixOS and
# /bin/bash does not exist, so anything shelling out silently fails
# without it.  VARSWEEP_TRACE has to be listed here or it never reaches
# the child.
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
  VARS_WORK="$WORK" \
  VARS_STRUCT="$OUT/$LABEL.struct.raw" \
  VARSWEEP_ONLY="${VARSWEEP_ONLY:-}" \
  VARSWEEP_TRACE="${VARSWEEP_TRACE:-}" \
  "$NVIM" --headless -u NONE -i NONE \
  -l "$HERE/varsweep.lua" \
  <"$WORK/empty" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr"
status=$?
set -e

printf 'exit %d\n' "$status" >>"$OUT/$LABEL.txt"

touch "$OUT/$LABEL.struct.raw"
LC_ALL=C sort "$OUT/$LABEL.struct.raw" >"$OUT/$LABEL.struct"
rm -f "$OUT/$LABEL.struct.raw"

printf '%s: report %d lines, struct %d lines, stderr %d lines, exit %d\n' \
  "$LABEL" "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.struct")" \
  "$(wc -l <"$OUT/$LABEL.stderr")" "$status"
