#!/usr/bin/env bash
# Build the working tree and diff its expression-parser sweep against the
# cut baseline.  Run from anywhere; everything is absolute.
#
#   exprverify.sh [label]         # default label: cur
#
# The expression parser's oracle: WHERE the parser stops and WHAT it says.
# The other rows ask what a value *is*; none asks where `eval0` left the
# cursor, which error text names which rest-of-line, or what happens when
# the line is cut short -- and that is what a rewrite of the cursor moves.
# exprsweep.vim is the driver and its header says what it runs:
#
#   exprcorpus.txt  expressions, each whole and cut at EVERY BYTE, through
#                   eval(), `:let`, `:echo .. | echon`, `:execute`, `:call`
#                   and `:elseif` in skip mode;
#   exprcmds.txt    commands (lvalues, `:unlet`, `:lockvar`, `:function`
#                   argument lists, heredocs, control flow), each whole and
#                   cut at every byte, through execute();
#   reent           an expression that changes the text being evaluated:
#                   'foldexpr', 'indentexpr', an `<expr>` mapping, the status
#                   line and `:s/\=/`.
#
# Artifacts: base.txt (the report) and base.stderr (the prompt, with the
# script directory masked as <HERE>, ending in the process's `exit N`: 124 is
# the harness timeout and 134 an abort).
#
# The fixture is rebuilt before every evaluation, so a line's answer does
# not depend on the lines before it.  No time, pid or address reaches either
# artifact; three runs of one binary were byte-identical.  It costs ~45 s
# on a debug build, almost all of it the expression section's six entries
# over ~3,800 prefixes.
set -uo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
OUT=${SWEEP_OUT:-/tmp/exprsweep-out}
REPO=${REPO:-$(cd "$HERE/../.." && pwd)}
LABEL=${1:-cur}
LOG=$OUT/build-$LABEL.log
LIMIT=${EXPR_TIMEOUT:-300}
# Cut mode.  `baseline.sh` re-enters this script as `--cut <nvim> <dir>` to
# cut the pinned baseline from the reference binary; it shares the ONE sweep
# call below with the head run, so the two sides of the differential cannot
# drift apart.  It skips the build and the diff.
CUT=
if [[ ${1:-} == --cut ]]; then CUT=$2; OUT=$3; LABEL=base; LOG=/dev/null; fi

mkdir -p "$OUT"
cd "$REPO" || exit 1
if [[ -z $CUT ]] && ! just build >"$LOG" 2>&1; then
  echo "BUILD FAILED -- see $LOG" >&2
  grep -E '^(error|warning)' "$LOG" | head -60 >&2
  exit 1
fi
NVIM=${CUT:-$REPO/target/debug/nvim}

# A fixed, short work directory: `:p` and `%:p` answers carry it, and the
# buffer is named inside it, so it must be the same path on both sides.
WORK=/tmp/exprsweep-work
rm -rf "$WORK"
mkdir -p "$WORK/home"
cd "$WORK" || exit 1
: >"$WORK/empty"

BASELINE=
if [ -z "$CUT" ]; then
  BASELINE=${EXPR_BASELINE:-$("$HERE/baseline.sh" expr)}
fi

out=$OUT/$LABEL
rm -f "$out.txt" "$out.stderr"
# `env -i`: no locale, editor configuration or XDG directory of the
# caller's reaches the run.
timeout -k 5 "$LIMIT" \
  env -i HOME="$WORK/home" PATH=/usr/bin:/bin TERM=dumb SHELL=/bin/sh \
  LANG=C.UTF-8 VIMRUNTIME="$REPO/runtime" DIFFOUT="$out.txt" \
  "$NVIM" --headless -u NONE -i NONE -n --cmd 'set noswapfile' \
  "$WORK/buffer.txt" -S "$HERE/exprsweep.vim" \
  <"$WORK/empty" >"$out.stderr.raw" 2>&1
status=$?
sed "s#$HERE#<HERE>#g" "$out.stderr.raw" >"$out.stderr"
rm -f "$out.stderr.raw"
printf '\nexit %d\n' "$status" >>"$out.stderr"
touch "$out.txt"
if [ -n "$CUT" ]; then
  echo "expr: CUT ($(wc -l <"$out.txt") lines, exit $status)"
  exit 0
fi

fail=0
for part in txt stderr; do
  if diff -q "$BASELINE/base.$part" "$OUT/$LABEL.$part" >/dev/null 2>&1; then
    echo "$part: IDENTICAL ($(wc -l <"$OUT/$LABEL.$part") lines)"
  else
    echo "$part: DIFFERS"
    # -a: the report carries prefixes cut inside a multibyte character.
    diff -a "$BASELINE/base.$part" "$OUT/$LABEL.$part" | head -60
    fail=1
  fi
done
exit $fail
