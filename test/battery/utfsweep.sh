#!/usr/bin/env bash
# Differential oracle for the multibyte and string layers (batch B15):
# mbyte.rs (4,546 lines) and strings.rs (3,975), which no existing
# differential reaches.
#
#   utfsweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# The two files are the deepest leaves in the tree -- 165 and 113 files
# fan in -- so a regression in either surfaces as *message text* in four
# other sweeps rather than as a diff anybody can read.  This oracle asks
# them directly, through the Vimscript functions that front them and the
# `vim.*` Lua leaves that call the same C:
#
#   * the index family: charidx / byteidx / byteidxcomp / strgetchar /
#     strcharpart / strpart / utf16idx -- mb_utflen and
#     mb_utf_index_to_bytes read from both ends.
#   * the width family: strchars / strcharlen / strdisplaywidth /
#     strwidth / strlen / strutf16len, under **both** 'ambiwidth'
#     values and across setcellwidths() -- utf_char2cells and
#     utf_ptr2cells, i.e. the table this batch is about to fold.
#   * charclass() over a codepoint corpus, which is the only front door
#     to `utf_class_tab` (399 lines of range table).
#   * the escape family: keytrans / escape / shellescape / fnameescape /
#     tr / strtrans / substitute / matchstr, i.e. strings.rs's
#     vim_strsave_escaped_ext and vim_strsave_shellescape.
#   * **printf()'s whole format matrix** -- `vim_vsnprintf_typval` is
#     1,297 lines and *nothing in the tree tests it*.  Every conversion,
#     every flag, width, precision, `*` argument, `%<n>$` positional
#     argument, length modifier, float edge value, and multibyte string
#     under a width and a precision.
#   * iconv() and `:e ++enc=` round trips -- my_iconv_open,
#     string_convert_ext, enc_canonize.
#   * the Lua leaves (vim.str_utfindex, vim.str_byteindex,
#     vim.str_utf_pos/start/end, vim.stricmp, vim.iconv), which reach
#     the same mbyte entry points through a different caller.
#
# Produces, under <outdir>:
#
#   <label>.txt       canonical report, one line per probe, in a fixed
#                     emission order, read top to bottom.
#   <label>.struct    canonical (sorted-key) JSON, one line per labelled
#                     answer, then LC_ALL=C sorted.  Sorted AFTER the
#                     scrub, never before.
#   <label>.bin       the *raw bytes* of every string answer, framed
#                     \x1e<label>\x1f<bytes>.  The .txt escapes high
#                     bytes to \xNN so that a report line stays a report
#                     line; this artifact is the check that the escaper
#                     is not itself hiding a difference, and it is the
#                     only place an invalid or overlong sequence is
#                     compared as the bytes it actually is.
#   <label>.stderr    what nvim wrote to the prompt.  s90 runs the error
#                     arms (E475/E1174/E1210/E1305/E766/E767/E807/E1206
#                     and the iconv failures) uncaptured, so this
#                     artifact carries signal rather than being empty.
#
# HOST DEPENDENCY: iconv.  s08 and s09 convert between utf-8, latin1,
# cp1252, ucs-2, euc-jp and a few names that do not exist; which of them
# my_iconv_open accepts is a property of the host's iconv implementation,
# not of nvim.  Both sides of an A/B see the same one, and the section
# opens with `x08 have-iconv <n>` so that a host without it produces one
# differing line rather than a hundred.
#
# `%p` is only ever handed a *Number*: on a String argument tv_ptr hands
# back `vval.v_string`, an allocation address, which differs per run by
# construction.  Nothing else in this sweep prints an address, so the
# report deliberately does NOT scrub `0x...` -- `%p` and `%x` are the
# answer in several hundred cases.
#
# The nvim invocation is wrapped in `timeout`, and its exit status is
# appended to the report as a final `exit <code>` line.  124 is the
# harness's verdict on a wedge; 134 an abort.
#
# UTFSWEEP_ONLY is a Lua pattern matched against each section name; it
# exists for iterating on one section, not for gating.
# UTFSWEEP_TRACE=1 mirrors each section name to stderr, which is the
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
# report through error texts and shellescape()'s output, and a long work
# directory leaves half-elided paths behind that the scrub cannot match.
WORK=${UTFSWEEP_WORK:-/tmp/utfsweep}
LIMIT=${UTFSWEEP_TIMEOUT:-900}

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
# configuration or XDG directories.  LANG is the one that matters most
# here: enc_locale() reads it, and 'ambiwidth', 'fileencodings' and every
# iconv name in the sweep would otherwise be a function of the caller's
# shell.  SHELL is set explicitly because 'shell' comes from it and
# shellescape() answers differently for a csh.
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
  UTF_WORK="$WORK" \
  UTF_STRUCT="$OUT/$LABEL.struct.raw" \
  UTF_BIN="$OUT/$LABEL.bin" \
  UTFSWEEP_ONLY="${UTFSWEEP_ONLY:-}" \
  UTFSWEEP_TRACE="${UTFSWEEP_TRACE:-}" \
  "$NVIM" --headless -u NONE -i NONE \
  -l "$HERE/utfsweep.lua" \
  <"$WORK/empty" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr.raw"
status=$?
set -e

# The scrubs the driver has to do rather than the Lua: a temp file name
# carries six random characters and a monotonic counter and reaches
# stderr through the ++enc write path's error arms.
sed -E \
  -e "s#$WORK#<WORK>#g" \
  -e 's#<WORK>/tmp/nvim\.[^/]*/[A-Za-z0-9]+/[0-9]+#<TMPFILE>#g' \
  -e 's#<WORK>/tmp/nvim\.[^/]*/[A-Za-z0-9]+#<TMPDIR>#g' \
  <"$OUT/$LABEL.stderr.raw" >"$OUT/$LABEL.stderr"
rm -f "$OUT/$LABEL.stderr.raw"

printf 'exit %d\n' "$status" >>"$OUT/$LABEL.txt"

touch "$OUT/$LABEL.struct.raw" "$OUT/$LABEL.bin"
LC_ALL=C sort "$OUT/$LABEL.struct.raw" >"$OUT/$LABEL.struct"
rm -f "$OUT/$LABEL.struct.raw"

printf '%s: report %d lines, struct %d lines, bin %d bytes, stderr %d lines, exit %d\n' \
  "$LABEL" "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.struct")" \
  "$(wc -c <"$OUT/$LABEL.bin")" "$(wc -l <"$OUT/$LABEL.stderr")" "$status"
