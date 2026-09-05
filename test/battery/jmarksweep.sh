#!/usr/bin/env bash
# Differential oracle for named marks, the jumplist and the changelist
# (batch B-mark): mark/mod.rs, lookup.rs, adjust.rs, jumplist.rs,
# show.rs, builtins.rs, shada.rs.
#
#   jmarksweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# Phase 20's survey (p20-1 §D.2 GAP 2) found B-mark with NO dedicated
# differential and NO mutation anchors -- `marksweep` is the *marktree*
# oracle, and `perssweep`'s shada half watches only the PERSISTED form.
# This is the live-behaviour oracle.  Every case is a key sequence or an
# Ex command run against a fixed buffer, and the answer recorded is
# every store the subsystem owns: every mark that is set (a-z A-Z 0-9
# and the whole tick family, each as `buffer:line,col`), the jumplist
# with its index, the changelist with its index, the cursor, and -- for
# the cases that ask -- the rendered `:marks` / `:jumps` / `:changes`
# listings, which are a DIFFERENT surface over the same slots.
#
# Produces, under <outdir>:
#
#   <label>.txt       canonical report -- five lines per case (B buffer
#                     if the keys changed it, K marks, J jumplist,
#                     G changelist, P cursor), plus X lines for a
#                     captured listing and A for an extra expression,
#                     in a fixed emission order, read top to bottom and
#                     diffed as-is.
#   <label>.struct    canonical (sorted-key) JSON, one line per labelled
#                     answer, then LC_ALL=C sorted.  Sorted AFTER the
#                     scrub, never before.
#   <label>.stderr    what nvim wrote to the prompt.  s16 runs a whole
#                     section uncaptured with 'report' at 0, so E19,
#                     E20 ("Mark has invalid line number"), E78
#                     ("Unknown mark"), E475 and the :delmarks range
#                     errors reach this artifact -- the only view of
#                     msg_* for this subsystem.
#
# There is deliberately NO byte layer.  The mark bytes that reach a disk
# are the ShaDa file's, and that is `perssweep`'s golden (re-baselined
# byte-exact at p20-4).  What this sweep adds is what comes BACK: s10
# writes a shada, wipes every mark the process holds, and reads it in.
#
# Run once per binary, then `diff` the three <label> outputs.  Both runs
# use the same fixed (short) work directory, and the sweep chdir's into
# it: `:marks` and `:jumps` print a file name SHORTENED AGAINST THE
# PROCESS CWD, so two runs from two directories compare as
# all-different.
#
# BUFFER NUMBERS never appear.  Every buffer the sweep opens is a named
# file under $WORK and the report prints the basename; the driver keeps
# a handle->name registry so a mark in a WIPED buffer still names its
# file.  See the header of jmarksweep.lua for the rest of
# the nondeterminism list (Columns, timestamps, :filter).
#
# The nvim invocation is wrapped in `timeout`, and its exit status is
# appended to the report as a final `exit <code>` line.  124 is the
# harness's verdict on a wedge; 134 an abort.
#
# JMARKSWEEP_ONLY is a Lua pattern matched against each section name; it
# exists for iterating on one section, not for gating.
# JMARKSWEEP_TRACE=1 mirrors each section name to stderr -- and it must
# be OFF for a baseline, because .stderr is a compared artifact.

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

# Short on purpose, and the same for every run: `:marks` truncates its
# text column to the (headless, 80 column) screen, and a long work
# directory leaves half-elided paths behind that the scrub cannot
# recognise.
WORK=${JMARKSWEEP_WORK:-/tmp/jmarksweep}
LIMIT=${JMARKSWEEP_TIMEOUT:-600}

mkdir -p "$OUT"
rm -rf "$WORK"
mkdir -p "$WORK/home"

: >"$WORK/empty"

# `env -i` so the report cannot pick up the caller's locale, editor
# configuration or XDG directories -- and, the one that matters here,
# so `$NVIM_APPNAME`/`$XDG_STATE_HOME` cannot point the ShaDa file at
# the caller's real one.  s10 writes and reads a shada; it must be the
# sweep's own.
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
  JMARK_WORK="$WORK" \
  JMARK_STRUCT="$OUT/$LABEL.struct.raw" \
  JMARKSWEEP_ONLY="${JMARKSWEEP_ONLY:-}" \
  JMARKSWEEP_TRACE="${JMARKSWEEP_TRACE:-}" \
  "$NVIM" --headless -u NONE -i NONE \
  -l "$HERE/jmarksweep.lua" \
  <"$WORK/empty" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr.raw"
status=$?
set -e

# The scrubs the driver cannot do from inside a `:normal` sequence.
# `#<n>` is undo's monotonic change number (one counter for the whole
# run, so it is a function of every case above the one that printed
# it); "N seconds ago" is wall-clock; the work directory reaches stderr
# through error texts the Lua never sees.
sed -E \
  -e "s#$WORK#<WORK>#g" \
  -e 's/#[0-9]+/#N/g' \
  -e 's/[0-9]+ (second|minute|hour|day)s? ago/N ago/g' \
  -e 's/0x[0-9a-f]+/<ADDR>/g' \
  <"$OUT/$LABEL.stderr.raw" >"$OUT/$LABEL.stderr"
rm -f "$OUT/$LABEL.stderr.raw"

printf 'exit %d\n' "$status" >>"$OUT/$LABEL.txt"

touch "$OUT/$LABEL.struct.raw"
LC_ALL=C sort "$OUT/$LABEL.struct.raw" >"$OUT/$LABEL.struct"
rm -f "$OUT/$LABEL.struct.raw"

printf '%s: report %d lines, struct %d lines, stderr %d lines, exit %d\n' \
  "$LABEL" "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.struct")" \
  "$(wc -l <"$OUT/$LABEL.stderr")" "$status"
