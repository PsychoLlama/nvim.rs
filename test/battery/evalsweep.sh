#!/usr/bin/env bash
# Differential oracle for the eval substrate (batch B14): eval/typval.rs,
# eval/encode.rs, eval/decode.rs, lua/converter.rs,
# api/private/converter.rs and api/private/validate.rs.
#
#   evalsweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# Produces, under <outdir>:
#
#   <label>.txt       canonical report -- every encoder's rendering of a
#                     shared corpus, every round trip, every error text.
#                     Emission order is fixed, so this artifact is read
#                     top to bottom and diffed as-is.
#   <label>.struct    canonical (sorted-key) JSON, one line per labelled
#                     answer, then LC_ALL=C sorted.  The readable report
#                     escapes and flattens; this one keeps the shape, and
#                     sorting it makes the diff independent of where in
#                     the run an answer was produced.  Sorted AFTER the
#                     scrub, never before -- a path is exactly the kind of
#                     answer that differs between two working directories
#                     and survives the three-runs rule.
#   <label>.stderr    what nvim wrote to the prompt.  In a headless
#                     process that is where messages land, and for some
#                     of the encoder and autoload errors it is the only
#                     view of them.
#
# Run once per binary, then `diff` the three <label> outputs.  Both runs
# use the same fixed (short) work directory: absolute paths reach the
# report through error texts and `expand()`-shaped answers.
#
# The nvim invocation is wrapped in `timeout`, and its exit status is
# appended to the report as a final `exit <code>` line.  A pure-function
# corpus should not be able to hang, but a rewrite of the encoder walker
# absolutely can loop on a self-referencing container, and that has to be
# a loud diff rather than a wedged harness.  124 is timeout's verdict.
#
# EVALSWEEP_ONLY is a Lua pattern matched against each section name; it
# exists for iterating on one section, not for gating.

set -euo pipefail

if [[ $# -ne 4 ]]; then
  sed -n '2,38p' "$0" >&2
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
WORK=${EVALSWEEP_WORK:-/tmp/evsweep}
# A clean run is a few seconds.  120 is a large margin and bounds what a
# looping-walker mutation costs.
LIMIT=${EVALSWEEP_TIMEOUT:-120}

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
# without it.
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
  EVAL_WORK="$WORK" \
  EVAL_STRUCT="$OUT/$LABEL.struct.raw" \
  EVALSWEEP_ONLY="${EVALSWEEP_ONLY:-}" \
  "$NVIM" --headless -u NONE -i NONE \
  -l "$HERE/evalsweep.lua" \
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
