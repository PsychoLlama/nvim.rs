#!/usr/bin/env bash
# Differential oracle for the regexp family:
# crates/nvim/src/nvim/regexp.rs and regexp/{api,bt,bt/*,nfa,nfa/*,parse,
# chars,mbyte,equi_class,context,submatch,substitute}.rs -- `vim_regcomp`
# over every magic level, `vim_regexec_nl` / `vim_regexec_multi` /
# `vim_regexec_prog`, the BACKTRACKING and the NFA engine SIDE BY SIDE,
# the zero-width and position atoms, the submatch bookkeeping, the
# `substitute()` replacement alphabet and the error paths.
#
#   resweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# THE HOLE IT FILLS.  Of the twenty-two differentials that existed before
# it, only `opsweep` reaches the engines at all, and only through `:s`.
# Nothing anywhere drove `'regexpengine'`, so the whole NFA engine and
# the whole backtracking engine were covered by whichever one the
# default happened to pick, on whichever handful of patterns a `:s` case
# needed.  `rex` -- the `regexec_T` both engines thread through
# themselves -- is the single largest `cell_ptr` consumer in the tree,
# and a bug in it shows on ONE engine and not the other.
#
# THE AXIS THAT MATTERS is therefore `'regexpengine'`: every corpus row
# runs under `re=0` (NFA, falling back to BT when compilation fails --
# the default, and the only setting that exercises the fallback), `re=1`
# (BT only) and `re=2` (NFA only).  `r10/enginedelta` counts the corpus
# rows on which 1 and 2 disagree, and `r10/deltalist` names them.  That
# count IS the sweep's assertion that the option does anything: if it
# collapses, one engine is not being reached and two thirds of the rows
# are measuring the other one twice.
#
# ERRORS ARE ROWS, NOT ABORTS.  An invalid pattern is a first-class
# answer here -- the two engines refuse the same input with different
# `E` codes more often than they agree -- so every driver call is
# `pcall`ed, the message is normalised to its `E<n>: <text>` tail (the
# Lua traceback and the `vim/_core/editor` frame that `pcall` prepends
# are not properties of the engines) and lands in the `.err` layer.
#
# Produces, under <outdir>:
#
#   <label>.txt     canonical report: one line per case, a
#                   `## <section> rows=N` line per section, `## TOTAL`
#                   and a final `exit N`.
#   <label>.err     the MESSAGE layer: every error any driver raised,
#                   in emission order, `<tag> <driver> <E-code: text>`.
#                   Moving alone means a message or an `E` code changed
#                   with every match answer still right.
#   <label>.stderr  what the process and its children wrote to the
#                   prompt.  EMPTY at the baseline; that is the
#                   assertion.
#
# `--headless -c`, NOT `-l`: r4's position atoms (`\%V`, `\%#`, `\%23v`)
# and r5's `search()` rows need a real window, and under `-l`
# `full_screen` is false.
#
# SANDBOX.  $WORK is a fresh `mktemp -d` unless RESWEEP_WORK says
# otherwise; cwd is $WORK, $HOME is $WORK/home, $TMPDIR is $WORK/tmp and
# $PATH is $WORK/bin, which holds a copy of the binary under test and
# nothing else.  `env -i`.
#
# ORPHANS.  r91 runs plain `--headless` children through `system()`, each
# under its own `timeout -k 2 30` because catastrophic backtracking is a
# real property of the BT engine and one probe provokes it deliberately.
# If a run is killed, look for children with ppid 1 and KILL THEM BY PID
# -- never by pattern, which would take out the operator's own editor.
set -euo pipefail

if [[ $# -ne 4 ]]; then
  sed -n '2,70p' "$0" >&2
  exit 2
fi

NVIM=$(realpath "$1")
RUNTIME=$(realpath "$2")
OUT=$(realpath -m "$3")
LABEL=$4
HERE=$(cd "$(dirname "$0")" && pwd)

LIMIT=${RESWEEP_TIMEOUT:-900}

# r91's children run under `timeout`, and the sweep runs under `env -i`
# with $PATH pointing at the sandbox: resolve it here, absolutely.
TIMEOUT_BIN=$(command -v timeout)

OWNED=0
if [[ -n ${RESWEEP_WORK:-} ]]; then
  WORK=$RESWEEP_WORK
  rm -rf "$WORK"
  mkdir -p "$WORK"
else
  WORK=$(mktemp -d /tmp/resweep.XXXXXXXXXX)
  OWNED=1
fi

mkdir -p "$OUT"
umask 022
mkdir -p "$WORK/home" "$WORK/bin" "$WORK/tmp"
chmod 755 "$WORK" "$WORK/home" "$WORK/bin" "$WORK/tmp"

cp "$NVIM" "$WORK/bin/nvim"
chmod 755 "$WORK/bin/nvim"

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
  RE_WORK="$WORK" \
  RE_NVIM="$WORK/bin/nvim" \
  RE_TIMEOUT="$TIMEOUT_BIN" \
  RE_ERR="$OUT/$LABEL.err.raw" \
  RESWEEP_ONLY="${RESWEEP_ONLY:-}" \
  "$WORK/bin/nvim" --headless -u NONE -i NONE \
  --cmd "cd $WORK" \
  --cmd 'set noswapfile shell=/bin/sh undolevels=1000' \
  -c 'set columns=80 lines=24 laststatus=0 showtabline=0 ruler noshowcmd report=9999 nomore shortmess=aoOtTIcCF' \
  -c "luafile $HERE/resweep.lua" \
  -c 'qa!' \
  <"$WORK/empty" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr.raw"
status=$?
set -e

scrub() { # in out
  sed -E \
    -e "s#$WORK#<WORK>#g" \
    -e "s#$HERE/resweep\.lua#<SCRIPT>#g" \
    -e "s#\.\.\.[^ \"]*/resweep\.lua#<SCRIPT>#g" \
    -e "s#$RUNTIME#<RT>#g" \
    -e 's#0x[0-9a-f]+#<ADDR>#g' \
    -e 's#\.rs:[0-9]+:[0-9]+#.rs:<LINE>#g' \
    -e 's#\.rs:[0-9]+#.rs:<LINE>#g' \
    <"$1" >"$2"
}

scrub "$OUT/$LABEL.stderr.raw" "$OUT/$LABEL.stderr"
rm -f "$OUT/$LABEL.stderr.raw"

touch "$OUT/$LABEL.err.raw"
scrub "$OUT/$LABEL.err.raw" "$OUT/$LABEL.err"
rm -f "$OUT/$LABEL.err.raw"

# The report itself is written to stdout by the editor, so it needs the
# same scrub -- an error message can carry the work directory.
scrub "$OUT/$LABEL.txt" "$OUT/$LABEL.txt.scrubbed"
mv "$OUT/$LABEL.txt.scrubbed" "$OUT/$LABEL.txt"

printf 'exit %d\n' "$status" >>"$OUT/$LABEL.txt"

if [[ $OWNED -eq 1 && -z ${RESWEEP_KEEP:-} ]]; then
  rm -rf "$WORK"
fi

printf '%s: report %d lines, err %d lines, stderr %d lines, exit %d\n' \
  "$LABEL" "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.err")" \
  "$(wc -l <"$OUT/$LABEL.stderr")" "$status"
