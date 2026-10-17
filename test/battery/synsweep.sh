#!/usr/bin/env bash
# Differential oracle for the regexp engines over the runtime's own
# syntax patterns: every `syn match`/`syn region` pattern in
# runtime/syntax/*.vim, deduplicated, compiled and matched under
# 'regexpengine' 0, 1 and 2.  See synsweep.lua for how the patterns are
# cut out and synverify.sh for why the row exists.
#
#   synsweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# Produces, under <outdir>:
#
#   <label>.txt     one row per unique pattern, then the totals, the
#                   engine-delta count and list, and a final `exit N`.
#   <label>.err     every error message, `<tag> e<N> <E-code: text>`.
#   <label>.stderr  what the process wrote to the prompt.  EMPTY at the
#                   baseline; that is the assertion.
#
# SANDBOX.  $WORK is a fresh `mktemp -d` unless SYNSWEEP_WORK says
# otherwise; cwd is $WORK, $HOME is $WORK/home, `env -i`.  No children.
set -euo pipefail

if [[ $# -ne 4 ]]; then
  sed -n '2,22p' "$0" >&2
  exit 2
fi

NVIM=$(realpath "$1")
RUNTIME=$(realpath "$2")
OUT=$(realpath -m "$3")
LABEL=$4
HERE=$(cd "$(dirname "$0")" && pwd)

LIMIT=${SYNSWEEP_TIMEOUT:-900}

OWNED=0
if [[ -n ${SYNSWEEP_WORK:-} ]]; then
  WORK=$SYNSWEEP_WORK
  rm -rf "$WORK"
  mkdir -p "$WORK"
else
  WORK=$(mktemp -d /tmp/synsweep.XXXXXXXXXX)
  OWNED=1
fi

mkdir -p "$OUT" "$WORK/home" "$WORK/tmp"
: >"$WORK/empty"

set +e
timeout -k 5 "$LIMIT" \
  env -i \
  HOME="$WORK/home" \
  PATH=/nonexistent \
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
  SYN_RT="$RUNTIME" \
  SYN_ERR="$OUT/$LABEL.err.raw" \
  ${SYN_TRACE:+SYN_TRACE="$SYN_TRACE"} \
  "$NVIM" --headless -u NONE -i NONE \
  --cmd "cd $WORK" \
  --cmd 'set noswapfile' \
  -c 'set columns=80 lines=24 nomore shortmess=aoOtTIcCF maxmempattern=1000' \
  -c "luafile $HERE/synsweep.lua" \
  -c 'qa!' \
  <"$WORK/empty" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr.raw"
status=$?
set -e

scrub() { # in out
  sed -E \
    -e "s#$WORK#<WORK>#g" \
    -e "s#$HERE/synsweep\.lua#<SCRIPT>#g" \
    -e "s#\.\.\.[^ \"]*/synsweep\.lua#<SCRIPT>#g" \
    -e "s#$RUNTIME#<RT>#g" \
    <"$1" >"$2"
}

scrub "$OUT/$LABEL.stderr.raw" "$OUT/$LABEL.stderr"
rm -f "$OUT/$LABEL.stderr.raw"
touch "$OUT/$LABEL.err.raw"
scrub "$OUT/$LABEL.err.raw" "$OUT/$LABEL.err"
rm -f "$OUT/$LABEL.err.raw"
scrub "$OUT/$LABEL.txt" "$OUT/$LABEL.txt.scrubbed"
mv "$OUT/$LABEL.txt.scrubbed" "$OUT/$LABEL.txt"
printf 'exit %d\n' "$status" >>"$OUT/$LABEL.txt"

if [[ $OWNED -eq 1 && -z ${SYNSWEEP_KEEP:-} ]]; then
  rm -rf "$WORK"
fi

printf '%s: report %d lines, err %d lines, stderr %d lines, exit %d\n' \
  "$LABEL" "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.err")" \
  "$(wc -l <"$OUT/$LABEL.stderr")" "$status"
