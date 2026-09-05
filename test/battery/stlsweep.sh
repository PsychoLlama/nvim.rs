#!/usr/bin/env bash
# Differential oracle for the statusline family (batch B19):
# statusline.rs -- `build_stl_str_hl` (1,751 lines, the port's single
# biggest function), `win_redr_custom`, `redraw_ruler`, `draw_tabline`
# and the click-definition arenas.
#
#   stlsweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# The gap this closes: B19's survey found THREE snapshots of
# `build_stl_str_hl` in all of scrsweep -- two `statusline=`, six `%{`
# and ZERO `%@` -- against 41 distinct `STL_*` item letters plus groups,
# min/max width, `%<`, `%=`, `%{%..%}`, `%!`, click definitions and a
# hundred-deep evaluation wall.  A wrong answer here is a silently wrong
# status line, so every case reports the ANSWER -- the string, its
# display WIDTH, and where requested the highlight records.
#
# The driver is `nvim_eval_statusline()`, which answers
# `{str, width, highlights}` for an arbitrary format in an arbitrary
# window context.  Screen snapshots are used only where it cannot reach
# (click records, the ruler, a real draw); see the .lua header.
#
# Produces, under <outdir>:
#
#   <label>.txt       canonical report, read top to bottom and diffed
#                     as-is.  One tagged line per answer:
#                       =  the string and width the format produced
#                       H  its highlight records
#                       !  the error it raised
#                       A  the case's own extra question
#                       X  one crashprobe child's answer
#   <label>.struct    canonical (sorted-key) JSON, one line per labelled
#                     answer, LC_ALL=C sorted AFTER the scrub.
#   <label>.stderr    what nvim wrote to the prompt.  s9 runs a block of
#                     failing redraws *uncaptured* in `-c` children,
#                     which is the only view of this family's message
#                     path (E539/E540/E542 and the `%{}` errors).
#
# SANDBOX.  Everything is confined to $WORK, a fresh `mktemp -d` unless
# STLSWEEP_WORK says otherwise: cwd is $WORK, $HOME is $WORK/home,
# $TMPDIR is $WORK/tmp and $PATH is pinned to $WORK/bin.  `env -i` means
# nothing else leaks in.
#
# THE BINARY IS COPIED TO $WORK/bin/nvim, because s5, s6, s8, s9 and s91
# all spawn children and they must find a binary inside the sandbox.
#
# THE WORK DIRECTORY IS A CONSTANT LENGTH (`/tmp/stlsweep.XXXXXXXXXX`).
# `%f`/`%F`/`%t` print buffer names and the s8/s6 screen rows are cut to
# 80 columns; two runs whose $WORK differ in LENGTH are not comparable.
# Two runs whose $WORK differ only in content are, and proving that is
# the acceptance test -- see stlverify.sh.
#
# NOT SCRUBBED AWAY: the RUNTIME path.  `:help` is one of s1's nine
# buffer states and `%F` prints `<RT>/doc/help.txt`; pass the SAME
# runtime to both sides of a paired run or every `help` row differs.
#
# THREE KINDS OF CHILD, and each is here for a reason the parent cannot
# serve:
#   * `--embed` + RPC (s6, s8).  A `%@Func@` handler only runs when the
#     main input loop dispatches the click.  In this `-l` process
#     `nvim_input_mouse` leaves the key sitting in the typeahead --
#     `getchar(0)` fishes it straight back out -- and no handler fires.
#     NO UI IS ATTACHED: the internal grid is drawn headless (which is
#     what `screenstring()` reads) and attaching one would make the
#     child stream `redraw` notifications back at a parent that has no
#     handler for them, which closes the channel and kills it (B19-1).
#   * `-c` (s5's sandbox, s9's messages).  In process a Vimscript error
#     becomes a Lua error and `pcall` eats it, so nvim never displays
#     anything.  nvim caps `-c`/`--cmd` at TEN in total, so s9 chunks
#     at five.
#   * `-l --crash N` (s91), one input each, so a death is one diffable
#     ABORTED row rather than a truncated report.
#
# The nvim invocation is wrapped in `timeout` and its exit status is
# appended to the report as a final `exit <code>` line: 124 is the
# harness's verdict on a wedge, 134 an abort.
#
# STLSWEEP_ONLY is a Lua pattern matched against each section name; it
# exists for iterating on one section, not for gating -- `%n` prints
# buffer numbers, so skipping a section renumbers the ones after it.
# STLSWEEP_TRACE=1 mirrors each section name to stderr and must be off
# for a baseline, because it writes into the .stderr artifact.

set -euo pipefail

if [[ $# -ne 4 ]]; then
  sed -n '2,80p' "$0" >&2
  exit 2
fi

NVIM=$(realpath "$1")
RUNTIME=$(realpath "$2")
OUT=$(realpath -m "$3")
LABEL=$4
HERE=$(cd "$(dirname "$0")" && pwd)

LIMIT=${STLSWEEP_TIMEOUT:-900}

# Constant length, always: `/tmp/stlsweep.` + ten characters.
OWNED=0
if [[ -n ${STLSWEEP_WORK:-} ]]; then
  WORK=$STLSWEEP_WORK
  rm -rf "$WORK"
  mkdir -p "$WORK"
else
  WORK=$(mktemp -d /tmp/stlsweep.XXXXXXXXXX)
  OWNED=1
fi

mkdir -p "$OUT"
umask 022
mkdir -p "$WORK/home" "$WORK/bin" "$WORK/tmp" "$WORK/f"
chmod 755 "$WORK" "$WORK/home" "$WORK/bin" "$WORK/tmp" "$WORK/f"

cp "$NVIM" "$WORK/bin/nvim"
chmod 755 "$WORK/bin/nvim"

# stdin, explicitly: in `-l` script mode a case that ends up asking the
# real input stream reads whatever the caller's terminal has.
: >"$WORK/empty"

set +e
timeout -k 5 "$LIMIT" \
  env -i \
  HOME="$WORK/home" \
  PATH="$WORK/bin" \
  TMPDIR="$WORK/tmp" \
  TERM=dumb \
  SHELL=/bin/sh \
  LANG=C.UTF-8 \
  VIMRUNTIME="$RUNTIME" \
  XDG_CONFIG_HOME="$WORK/home/.config" \
  XDG_DATA_HOME="$WORK/home/.local/share" \
  XDG_STATE_HOME="$WORK/home/.local/state" \
  XDG_CACHE_HOME="$WORK/home/.cache" \
  NVIM_TEST=1 \
  STL_WORK="$WORK" \
  STL_STRUCT="$OUT/$LABEL.struct.raw" \
  STLSWEEP_ONLY="${STLSWEEP_ONLY:-}" \
  STLSWEEP_TRACE="${STLSWEEP_TRACE:-}" \
  "$WORK/bin/nvim" --headless -u NONE -i NONE \
  --cmd "cd $WORK" \
  -l "$HERE/stlsweep.lua" \
  <"$WORK/empty" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr.raw"
status=$?
set -e

# Scrubs the driver has to do rather than the Lua, because they land in
# messages the sweep never gets to touch.
sed -E \
  -e "s#$WORK#<WORK>#g" \
  -e "s#$HERE/stlsweep\.lua#<SCRIPT>#g" \
  -e "s#\.\.\.[^ \"]*/stlsweep\.lua#<SCRIPT>#g" \
  -e "s#$RUNTIME#<RT>#g" \
  -e 's#nvim\.[0-9]+\.[0-9]+#nvim.<PID>.<SEQ>#g' \
  -e 's#nvim\.[A-Za-z0-9_.-]+/[A-Za-z0-9]+#nvim.<U>/<T>#g' \
  <"$OUT/$LABEL.stderr.raw" >"$OUT/$LABEL.stderr"
rm -f "$OUT/$LABEL.stderr.raw"

printf 'exit %d\n' "$status" >>"$OUT/$LABEL.txt"

touch "$OUT/$LABEL.struct.raw"
LC_ALL=C sort "$OUT/$LABEL.struct.raw" >"$OUT/$LABEL.struct"
rm -f "$OUT/$LABEL.struct.raw"

if [[ $OWNED -eq 1 && -z ${STLSWEEP_KEEP:-} ]]; then
  rm -rf "$WORK"
fi

printf '%s: report %d lines, struct %d lines, stderr %d lines, exit %d\n' \
  "$LABEL" "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.struct")" \
  "$(wc -l <"$OUT/$LABEL.stderr")" "$status"
