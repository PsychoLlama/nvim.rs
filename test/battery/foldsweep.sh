#!/usr/bin/env bash
# Differential oracle for the fold TREE (batch B-fold): fold/mod.rs,
# level.rs, open_close.rs, adjust.rs, marker.rs, text.rs, builtins.rs.
#
#   foldsweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# Phase 20's survey (p20-1 §D.2 GAP 3) found the fold *display* watched
# from six directions and fold *persistence* by sessgold's `put_view`
# block, while the fold tree under edits was watched by nothing.  This
# is that oracle.  Every case is a key sequence or an Ex command run
# against a fixed buffer, and the answer recorded is the tree as the
# editor will admit it: foldlevel() per line, the closed ranges from
# foldclosed()/foldclosedend(), foldtextresult() for each closed fold,
# the cursor (which is the only way zj/zk/[z/]z are observable), the
# eleven fold options, and -- for the cases that ask -- a LEVEL SCAN
# walking 'foldlevel' 0..N and recording the closed ranges at each step.
#
# Produces, under <outdir>:
#
#   <label>.txt       canonical report -- four to six lines per case
#                     (B buffer if the keys changed it, L levels,
#                     C closed ranges, T fold text, S cursor+options,
#                     Z the level scan, A an extra expression), in a
#                     fixed emission order, read top to bottom and
#                     diffed as-is.
#   <label>.struct    canonical (sorted-key) JSON, one line per labelled
#                     answer, then LC_ALL=C sorted.  The readable report
#                     escapes and flattens; this one keeps the shape,
#                     and sorting it makes the diff independent of where
#                     in the run an answer was produced.  Sorted AFTER
#                     the scrub, never before.
#   <label>.stderr    what nvim wrote to the prompt.  In a headless
#                     process that is where messages land, and s19 runs
#                     a whole section uncaptured with 'report' at 0 so
#                     that E350 ("Cannot create fold with current
#                     'foldmethod'"), E351, E352, E490 ("No fold found")
#                     and the "N lines folded" texts reach this
#                     artifact -- the only view of msg_* for this
#                     subsystem.
#
# There is deliberately NO byte layer.  The only fold bytes that reach
# a disk are `:mkview`'s fold block, and that is sessgold's golden
# (p20-1 §D.1); duplicating it here would give two baselines for one
# fact.  What this sweep adds is the tree BEFORE it is serialised.
#
# Run once per binary, then `diff` the three <label> outputs.  Both runs
# use the same fixed (short) work directory: absolute paths reach the
# report through error texts, and v:progpath names the binary under
# test, which is a *different* path on each side by construction (the
# driver scrubs it).
#
# The nvim invocation is wrapped in `timeout`, and its exit status is
# appended to the report as a final `exit <code>` line.  124 is the
# harness's verdict on a wedge; 134 an abort.  A mutation that makes
# `foldRemove`'s `continue` spin, or `foldOpenCursor`'s loop never
# reach DONE_ACTION == 0, shows up here as 124 rather than as a diff --
# see the traps in 1787242636-foldmutate.py.
#
# FOLDSWEEP_ONLY is a Lua pattern matched against each section name; it
# exists for iterating on one section, not for gating.
# FOLDSWEEP_TRACE=1 mirrors each section name to stderr, which is the
# only way to see where a wedged run stopped -- and it must be OFF for
# a baseline, because .stderr is a compared artifact.

set -euo pipefail

if [[ $# -ne 4 ]]; then
  sed -n '2,60p' "$0" >&2
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
WORK=${FOLDSWEEP_WORK:-/tmp/foldsweep}
LIMIT=${FOLDSWEEP_TIMEOUT:-600}

mkdir -p "$OUT"
rm -rf "$WORK"
mkdir -p "$WORK/home"

# stdin, explicitly.  In `-l` script mode a case that ends up asking the
# real input stream reads whatever the caller's terminal has; an empty
# file makes that EOF, deterministically, on every machine.
: >"$WORK/empty"

# `env -i` so the report cannot pick up the caller's locale, editor
# configuration or XDG directories.  FOLDSWEEP_* have to be listed here
# or they never reach the child.
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
  FOLD_WORK="$WORK" \
  FOLD_STRUCT="$OUT/$LABEL.struct.raw" \
  FOLDSWEEP_ONLY="${FOLDSWEEP_ONLY:-}" \
  FOLDSWEEP_TRACE="${FOLDSWEEP_TRACE:-}" \
  "$NVIM" --headless -u NONE -i NONE \
  -l "$HERE/foldsweep.lua" \
  <"$WORK/empty" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr.raw"
status=$?
set -e

# The scrubs the driver cannot do from inside a `:normal` sequence.
# `#<n>` is undo's monotonic change number (one counter for the whole
# run, so it is a function of every case above the one that printed
# it); the "N seconds ago" phrase is wall-clock and not reproducible at
# all.  The work directory reaches stderr through error texts that the
# Lua never sees.
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
