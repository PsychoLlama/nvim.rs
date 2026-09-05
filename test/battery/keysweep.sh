#!/usr/bin/env bash
# Differential oracle for the typeahead and mapping subsystem (batch B13):
# getchar.rs, mapping.rs and keycodes.rs.
#
#   keysweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# Produces, under <outdir>:
#
#   <label>.txt       canonical report -- every :map listing, every buffer
#                     state a key sequence left behind, every error text
#   <label>.struct    canonical (sorted-key) JSON, one line per labelled
#                     answer: every field maparg()/mapcheck()/maplist()/
#                     nvim_get_keymap() returns, plus every recorded
#                     buffer/cursor/register/mode tuple.  The readable
#                     report is blind to a dropped `lhsrawalt` or a wrong
#                     `mode_bits`; this artifact is not.
#   <label>.stderr    what nvim wrote to the prompt.  In a headless
#                     process that is where error messages and the
#                     overflow of a long `:map` listing land, and for
#                     some of them it is the only view.
#
# Run once per binary, then `diff` the three <label> outputs.  Both runs
# use the same fixed (short) work directory: absolute paths reach the
# report through `expand()`-shaped answers and `:map <buffer>` listings.
#
# The nvim invocation is wrapped in `timeout`, and its exit status is
# appended to the report as a final `exit <code>` line.  This subsystem's
# characteristic regression is a *hang* -- an ambiguous mapping that
# never resolves, a `getchar()` that never returns -- and a hang has to
# be a loud diff, not a wedged harness.  124 is timeout's verdict.
#
# KEYSWEEP_ONLY is a Lua pattern matched against each section name; it
# exists for iterating on one section, not for gating.

set -euo pipefail

if [[ $# -ne 4 ]]; then
  sed -n '2,35p' "$0" >&2
  exit 2
fi

NVIM=$(realpath "$1")
RUNTIME=$(realpath "$2")
OUT=$(realpath -m "$3")
LABEL=$4
HERE=$(cd "$(dirname "$0")" && pwd)

# Short on purpose: messages are truncated to the (headless, 80 column)
# screen and a long work directory leaves half-elided paths behind that
# the scrub cannot recognise.
WORK=${KEYSWEEP_WORK:-/tmp/ksweep}
# A clean run is ~22 s.  180 is an eight-fold margin and bounds what a
# hang mutation costs; a hang is meant to be a diff, not an afternoon.
LIMIT=${KEYSWEEP_TIMEOUT:-180}

mkdir -p "$OUT"
rm -rf "$WORK"
mkdir -p "$WORK/home"

# A small fixture tree.  The mapping sections need almost nothing; what
# they do need is a real file (for `<buffer>` maps to attach to and for
# the `<C-r><C-f>`-shaped answers to name) at a stable path.
printf 'alpha beta gamma\nsecond line here\nthird line\n' >"$WORK/one.txt"
printf 'other file\n' >"$WORK/two.txt"
# stdin, explicitly.  In `-l` script mode a case that ends up asking the
# real input stream for a key reads whatever the caller's terminal has;
# an empty file makes that EOF, deterministically, on every machine.
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
  KEY_WORK="$WORK" \
  KEY_STRUCT="$OUT/$LABEL.struct" \
  KEYSWEEP_ONLY="${KEYSWEEP_ONLY:-}" \
  "$NVIM" --headless -u NONE -i NONE \
  -l "$HERE/keysweep.lua" \
  <"$WORK/empty" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr"
status=$?
set -e

printf 'exit %d\n' "$status" >>"$OUT/$LABEL.txt"
touch "$OUT/$LABEL.struct"

# `:undo` reports how long ago the change was, and that ticks over
# between two runs of the same binary.  The Lua side masks it in the
# report; the message reaches stderr too, so mask it there as well.
sed -i -E 's/[0-9]+ (second|minute|hour)s? ago/<AGO>/g' "$OUT/$LABEL.stderr"

printf '%s: report %d lines, struct %d lines, stderr %d lines, exit %d\n' \
  "$LABEL" "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.struct")" \
  "$(wc -l <"$OUT/$LABEL.stderr")" "$status"
