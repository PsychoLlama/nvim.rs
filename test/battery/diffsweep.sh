#!/usr/bin/env bash
# Differential oracle for the diff subsystem (batch B15): diff.rs and
# the six vendored xdiff/ files.
#
#   diffsweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# No existing differential reaches either.  `test_diffmode` is the only
# thing that does, and **104 of its assertions are skipped screendumps**,
# which is why this is the batch's largest hole.
#
# Every case builds a tabpage of two (sometimes three) diff'd buffers
# from a named fixture pair, applies a 'diffopt' spelling, runs
# `:diffupdate`, and records the whole observable diff state:
#
#   * `diff_hlID(lnum, col)` per line -- and, for the `inline:` modes,
#     per *column*, which is the only way to see the inline diff from a
#     headless process at all (the highlight itself only exists on a
#     screen; f_diff_hlID resolves the same change list the drawer does).
#   * `diff_filler(lnum)` per line -- the filler lines that make the two
#     windows line up, i.e. what the hunk computation decided.
#   * the fold ranges 'foldmethod=diff' produces, which is diff_infold
#     over the same blocks read from the other end.
#   * `]c` / `[c` traversal from both ends (diff_move_to).
#   * `:diffget` / `:diffput` results (diffgetput + the line mapping).
#   * `:diffpatch`, `'diffexpr'`, `'diffanchors'`, and the topline/topfill
#     that diff_set_topline computes for a scroll-bound partner.
#   * **`vim.diff()`** (s18), which is lua/xdiff.rs and the *other*
#     consumer of xdiff/ -- the only one that reaches xdl_emit_diff and
#     the unified formatter (the editor path passes ctxlen 0 and a
#     hunk_func) or XDF_IGNORE_CR_AT_EOL ('diffopt' cannot spell it).
#
# Produces, under <outdir>:
#
#   <label>.txt       canonical report, several lines per case, in a
#                     fixed emission order, read top to bottom.
#   <label>.struct    canonical (sorted-key) JSON, one line per labelled
#                     answer, then LC_ALL=C sorted.  Sorted AFTER the
#                     scrub, never before.
#   <label>.stderr    what nvim wrote to the prompt.  s90 runs the error
#                     arms (E96/E97/E99/E100/E101/E102/E474/E816/E1549) and the
#                     `:diffupdate` message path uncaptured, so this
#                     artifact carries signal rather than being empty.
#
# EXTERNAL DIFF.  `diffopt=external` and `:diffpatch` shell out to
# `diff(1)` and `patch(1)`.  `env -i` leaves PATH at /usr/bin:/bin, where
# a NixOS host has `env` and `sh` and nothing else, so both would fail
# with E97 and the section would record "cannot create diffs" rather than
# a diff.  The driver resolves the two binaries from the *caller's* PATH
# and symlinks them into $WORK/bin, which is then first on the child's
# PATH: the sweep therefore depends on the host's diffutils version, but
# both sides of any A/B see the same one and the path itself is scrubbed.
# 'shell' comes from $SHELL, so SHELL=/bin/sh is set explicitly.
#
# $TMPDIR is pointed inside $WORK.  nvim's temp files are
# "$TMPDIR/nvim.<user>/<6 random chars>/<counter>": the random component
# differs per run by construction and the counter is one monotonic
# sequence for the whole run, so `v:fname_in` and friends are scrubbed to
# <TMPFILE> and the callback sections record whether a name was handed
# over, never what it was.
#
# The nvim invocation is wrapped in `timeout`, and its exit status is
# appended to the report as a final `exit <code>` line.  124 is the
# harness's verdict on a wedge; 134 an abort.
#
# DIFFSWEEP_ONLY is a Lua pattern matched against each section name; it
# exists for iterating on one section, not for gating.
# DIFFSWEEP_TRACE=1 mirrors each section name to stderr, which is the
# only way to see where a wedged run stopped.

set -euo pipefail

if [[ $# -ne 4 ]]; then
  sed -n '2,78p' "$0" >&2
  exit 2
fi

NVIM=$(realpath "$1")
RUNTIME=$(realpath "$2")
OUT=$(realpath -m "$3")
LABEL=$4
HERE=$(cd "$(dirname "$0")" && pwd)

# Short on purpose, and the same for every run: absolute paths reach the
# report through error texts and shell command lines, and a long work
# directory leaves half-elided paths behind that the scrub cannot match.
WORK=${DIFFSWEEP_WORK:-/tmp/diffsweep}
LIMIT=${DIFFSWEEP_TIMEOUT:-900}

mkdir -p "$OUT"
rm -rf "$WORK"
mkdir -p "$WORK/home" "$WORK/tmp" "$WORK/bin" "$WORK/files"

# The two external programs the subsystem shells out to.  Resolved here,
# where the caller's PATH still exists; recorded in the report as a
# present/absent flag by the Lua so that a host without them produces a
# diff on one line instead of a hundred.
for tool in diff patch sh; do
  if p=$(command -v "$tool" 2>/dev/null); then
    ln -sf "$(realpath "$p")" "$WORK/bin/$tool"
  fi
done

# stdin, explicitly.  In `-l` script mode a case that ends up asking the
# real input stream reads whatever the caller's terminal has; an empty
# file makes that EOF, deterministically, on every machine.
: >"$WORK/empty"

# `env -i` so the report cannot pick up the caller's locale, editor
# configuration or XDG directories.  DIFF_OPTIONS and DIFF are unset by
# construction, which matters: diff_file() removes DIFF_OPTIONS from the
# environment itself and a caller who has it set would otherwise be
# testing that removal rather than the diff.  DIFFSWEEP_TRACE has to be
# listed here or it never reaches the child.
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
  DIFF_WORK="$WORK" \
  DIFF_STRUCT="$OUT/$LABEL.struct.raw" \
  DIFFSWEEP_ONLY="${DIFFSWEEP_ONLY:-}" \
  DIFFSWEEP_TRACE="${DIFFSWEEP_TRACE:-}" \
  "$NVIM" --headless -u NONE -i NONE \
  -l "$HERE/diffsweep.lua" \
  <"$WORK/empty" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr.raw"
status=$?
set -e

# The scrubs the driver has to do rather than the Lua.  A temp file name
# carries six random characters and a monotonic counter, and reaches
# stderr through E97/E810 and through the external `diff` command line
# that shell errors echo back.  `--- <file>` / `+++ <file>` headers from
# a real unified diff carry a wall-clock mtime.
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
