#!/usr/bin/env bash
# Differential oracle for the autocmd subsystem (batch B14): autocmd.rs
# and api/autocmd.rs, ahead of the B14-15/16 rewrite.
#
#   ausweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# Produces, under <outdir>:
#
#   <label>.txt       canonical report -- the event-order log.  Every
#                     section fires a scenario and prints the handlers
#                     that ran, in the order they ran, one per line.
#                     Emission order is fixed; read top to bottom.
#   <label>.struct    canonical (sorted-key) JSON, one line per labelled
#                     answer, then LC_ALL=C sorted.  The readable report
#                     flattens a log to text; this one keeps the shape of
#                     the structured answers (`nvim_get_autocmds` dicts,
#                     `v:event` contents), and sorting makes the diff
#                     independent of where in the run an answer appeared.
#                     Sorted AFTER the scrub, never before: a path is
#                     exactly the answer that differs between two working
#                     directories and survives the three-runs rule.
#   <label>.stderr    what nvim wrote to the prompt.  In a headless
#                     process that is where messages land, and for the
#                     `:autocmd` listings and several E-codes it is the
#                     only view of them.
#
# Run once per binary, then `diff` the three <label> outputs.  Both runs
# use the same fixed (short) work directory: absolute paths reach the
# report through `<afile>`, `<amatch>` and the pattern-matching section,
# which is the whole point of several of them.
#
# The nvim invocation is wrapped in `timeout`, and its exit status is
# appended to the report as a final `exit <code>` line.  This subsystem's
# regression shape is a *wedge*, not a wrong answer: a rewrite of the
# firing walk that fails to pop `active_apc_list`, or that re-enters
# itself, loops forever.  124 is timeout's verdict, 134 an abort.
#
# AUSWEEP_ONLY is a Lua pattern matched against each section name; it
# exists for iterating on one section, not for gating.

set -euo pipefail

if [[ $# -ne 4 ]]; then
  sed -n '2,40p' "$0" >&2
  exit 2
fi

NVIM=$(realpath "$1")
RUNTIME=$(realpath "$2")
OUT=$(realpath -m "$3")
LABEL=$4
HERE=$(cd "$(dirname "$0")" && pwd)

# Short on purpose, and the same for every run: messages are truncated to
# the (headless, 80 column) screen and a long work directory leaves
# half-elided paths behind that the scrub cannot recognise.  It also has
# to be short enough that a `<afile>` of a file two directories deep
# still fits on one line of a `:autocmd` listing.
WORK=${AUSWEEP_WORK:-/tmp/ausweep}
# A clean run is a couple of seconds.  120 bounds what a walker mutation
# that never terminates costs.
LIMIT=${AUSWEEP_TIMEOUT:-120}

mkdir -p "$OUT"
rm -rf "$WORK"
mkdir -p "$WORK/home" "$WORK/tree/sub" "$WORK/tree/sub/deep" "$WORK/other"

# The pattern-matching corpus.  Names chosen so that `*`, `**`, `?`,
# `{a,b}`, a comma list, a character class and an escaped literal each
# pick out a *different* subset -- a corpus every pattern matches gates
# nothing.
for f in one.txt two.txt three.log a.c ab.c "x,y.txt" 'q?mark.txt' \
  'br{ace}.txt' 'back\slash.txt' UPPER.TXT no_ext; do
  : >"$WORK/tree/$f"
done
: >"$WORK/tree/sub/one.txt"
: >"$WORK/tree/sub/deep/one.txt"
: >"$WORK/other/one.txt"
printf 'alpha\nbeta\ngamma\n' >"$WORK/tree/lines.txt"

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
  AU_WORK="$WORK" \
  AU_STRUCT="$OUT/$LABEL.struct.raw" \
  AUSWEEP_ONLY="${AUSWEEP_ONLY:-}" \
  AUSWEEP_TRACE="${AUSWEEP_TRACE:-}" \
  "$NVIM" --headless -u NONE -i NONE \
  -l "$HERE/ausweep.lua" \
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
