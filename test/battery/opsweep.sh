#!/usr/bin/env bash
# Differential oracle for the text-operator core (batch B15): ops.rs,
# register.rs, change.rs, textobject.rs, edit.rs.
#
#   opsweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# No existing differential reaches any of those files.  Every case here
# is a *key sequence* -- the way a user reaches this subsystem -- run
# through nvim_feedkeys with the 'x' flag against a fixed buffer, and
# the answer recorded is the whole observable state afterwards: the
# buffer lines, the cursor (with curswant and coladd), the `'[ '] '< '>`
# marks, how many b:changedtick ticks the sequence spent, the mode it
# left behind, and every register that is not empty.
#
# Produces, under <outdir>:
#
#   <label>.txt       canonical report -- three or four lines per case
#                     (B buffer, S state, R registers, A the extra
#                     question), in a fixed emission order, read top to
#                     bottom and diffed as-is.
#   <label>.struct    canonical (sorted-key) JSON, one line per labelled
#                     answer, then LC_ALL=C sorted.  The readable report
#                     escapes and flattens; this one keeps the shape,
#                     and sorting it makes the diff independent of where
#                     in the run an answer was produced.  Sorted AFTER
#                     the scrub, never before.
#   <label>.stderr    what nvim wrote to the prompt.  In a headless
#                     process that is where messages land, and s20 runs
#                     a whole section uncaptured with 'report' at 0 so
#                     that "N fewer lines", "block of N lines yanked"
#                     and the E353/E354/E749 arms reach this artifact --
#                     the only view of msg_* for this subsystem.
#
# Run once per binary, then `diff` the three <label> outputs.  Both runs
# use the same fixed (short) work directory: absolute paths reach the
# report through error texts, and v:progpath names the binary under
# test, which is a *different* path on each side by construction (the
# driver scrubs it).
#
# `"+` and `"*` are served by a FIXTURE clipboard provider installed
# from the driver (g:clipboard, cache_enabled 0, backed by g:cbstore).
# The host clipboard is never read and never written, so the report does
# not depend on what the user last copied and the sweep is safe to run
# on a desktop session.
#
# The nvim invocation is wrapped in `timeout`, and its exit status is
# appended to the report as a final `exit <code>` line.  124 is the
# harness's verdict on a wedge; 134 an abort.  s21 spawns a *child*
# nvim over RPC (feedkeys 'x' can never let 'updatetime' expire), so a
# wedged child shows up as the parent's timeout too.
#
# OPSWEEP_ONLY is a Lua pattern matched against each section name; it
# exists for iterating on one section, not for gating.
# OPSWEEP_TRACE=1 mirrors each section name to stderr, which is the
# only way to see where a wedged run stopped.

set -euo pipefail

if [[ $# -ne 4 ]]; then
  sed -n '2,52p' "$0" >&2
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
WORK=${OPSWEEP_WORK:-/tmp/opsweep}
LIMIT=${OPSWEEP_TIMEOUT:-600}

mkdir -p "$OUT"
rm -rf "$WORK"
mkdir -p "$WORK/home"

# stdin, explicitly.  In `-l` script mode a case that ends up asking the
# real input stream reads whatever the caller's terminal has; an empty
# file makes that EOF, deterministically, on every machine.
: >"$WORK/empty"

# `env -i` so the report cannot pick up the caller's locale, editor
# configuration, XDG directories -- or, and this is the one that
# matters here, a DISPLAY/WAYLAND_DISPLAY that would let nvim find a
# real clipboard tool before g:clipboard is read.  OPSWEEP_TRACE has to
# be listed here or it never reaches the child.
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
  OPS_WORK="$WORK" \
  OPS_STRUCT="$OUT/$LABEL.struct.raw" \
  OPSWEEP_ONLY="${OPSWEEP_ONLY:-}" \
  OPSWEEP_TRACE="${OPSWEEP_TRACE:-}" \
  "$NVIM" --headless -u NONE -i NONE \
  -l "$HERE/opsweep.lua" \
  <"$WORK/empty" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr.raw"
status=$?
set -e

# The one scrub the driver has to do rather than the Lua: undo's own
# messages carry a wall-clock age ("1 change; before #10767  0 seconds
# ago") and a monotonic change number, neither of which the driver can
# reach from inside a `:normal` sequence.  The age is not reproducible
# at all; the change number is one counter for the whole run, so it is
# a function of every case above the one that printed it.
sed -E 's/#[0-9]+/#N/g; s/[0-9]+ (second|minute|hour|day)s? ago/N ago/g' \
  <"$OUT/$LABEL.stderr.raw" >"$OUT/$LABEL.stderr"
rm -f "$OUT/$LABEL.stderr.raw"

printf 'exit %d\n' "$status" >>"$OUT/$LABEL.txt"

touch "$OUT/$LABEL.struct.raw"
LC_ALL=C sort "$OUT/$LABEL.struct.raw" >"$OUT/$LABEL.struct"
rm -f "$OUT/$LABEL.struct.raw"

printf '%s: report %d lines, struct %d lines, stderr %d lines, exit %d\n' \
  "$LABEL" "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.struct")" \
  "$(wc -l <"$OUT/$LABEL.stderr")" "$status"
