#!/usr/bin/env bash
# Differential oracle for the option family:
# crates/nvim/src/nvim/options/{mod,flags,index,lookup,values,
# table_1..table_5}.rs and optionstr.rs -- the option TABLE itself (every
# name, short name, type, scope, default and flag), `do_set` and its
# `= += -= ^= & &vim < ! inv no` alphabet, `:set` / `:setlocal` /
# `:setglobal` / `:set all&` / `:verbose set` / `:options`, the
# global-local convention, what a fresh window and a fresh buffer
# inherit, and every `E` code `do_set` raises.
#
#   optsweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# THE HOLE IT FILLS.  There was no option-behaviour differential at all.
# Twenty-two oracles SET options -- every one of them does -- and not one
# reads the table back.  `vimoption_T` holds its value behind a
# `var: *mut c_void`, and the five generated `table_*.rs` files are the
# only description of what that pointer means for each of the 374
# entries; a rewrite that gets ONE entry's type, scope or default wrong
# produces a plausible editor that is quietly wrong about one option, and
# nothing in the functional suite enumerates the table.  So o1 prints one
# row PER OPTION -- metadata AND value -- and o5 pokes all 374.
#
# THE DENYLIST IS DECLARED, NOT SILENT.  Seventeen options are skipped by
# o5's poke because setting them runs something (the clipboard provider,
# a keymap script, a spell download, a shell) or reconfigures the harness
# in a way `set {o}&` cannot undo (`verbose`, `verbosefile`, `debug`,
# `writedelay`, `redrawdebug`, `shada`).  Each is still a ROW, marked
# `SKIPPED reason=...`, so the section's row count equals the option
# count and a later slice can see exactly what is not covered.
#
# `harness()` RE-APPLIES THE SANDBOX'S OWN SETTINGS after every `&` and
# every `all&`.  `&` restores the COMPILED default, not the harness's
# `report=9999 nomore columns=80 ...`, so without it every row after the
# first `&` would be measured under a different editor -- and o3 and o5
# alone issue nearly a thousand of them.
#
# Produces, under <outdir>:
#
#   <label>.txt     canonical report: one line per case, a
#                   `## <section> rows=N` line per section, `## TOTAL`
#                   and a final `exit N`.
#   <label>.opts    the BULK layer: `:set all`, `:setglobal all`,
#                   `:setlocal all`, `:verbose set`, the `:options`
#                   window's whole buffer and the info dict for all 374
#                   options, verbatim -- plus every message o3 and o5
#                   provoked.  Moving alone means a printer or a message
#                   moved with every scalar answer still right.
#   <label>.stderr  what the process and its children wrote to the
#                   prompt.  EMPTY at the baseline; that is the
#                   assertion.
#
# `--headless -c`, NOT `-l`: o6 splits windows and o7 opens the
# `:options` window, and under `-l` `full_screen` is false.
#
# SANDBOX.  $WORK is a fresh `mktemp -d` unless OPTSWEEP_WORK says
# otherwise; cwd is $WORK, $HOME is $WORK/home, $TMPDIR is $WORK/tmp and
# $PATH is $WORK/bin, which holds a copy of the binary under test and
# nothing else.  `env -i`.  Several options print a path -- `runtimepath`,
# `backupdir`, `shada`, `helpfile` -- so the scrub replaces $WORK and
# $VIMRUNTIME everywhere, including in the report itself.
#
# ORPHANS.  o91 runs plain `--headless` children through `system()`, each
# under its own `timeout -k 2 30`.  If a run is killed, look for children
# with ppid 1 and KILL THEM BY PID -- never by pattern, which would take
# out the operator's own editor.
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

LIMIT=${OPTSWEEP_TIMEOUT:-900}

# o91's children run under `timeout`, and the sweep runs under `env -i`
# with $PATH pointing at the sandbox: resolve it here, absolutely.
TIMEOUT_BIN=$(command -v timeout)

OWNED=0
if [[ -n ${OPTSWEEP_WORK:-} ]]; then
  WORK=$OPTSWEEP_WORK
  rm -rf "$WORK"
  mkdir -p "$WORK"
else
  WORK=$(mktemp -d /tmp/optsweep.XXXXXXXXXX)
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
  OPT_WORK="$WORK" \
  OPT_NVIM="$WORK/bin/nvim" \
  OPT_TIMEOUT="$TIMEOUT_BIN" \
  OPT_DUMP="$OUT/$LABEL.opts.raw" \
  OPTSWEEP_ONLY="${OPTSWEEP_ONLY:-}" \
  "$WORK/bin/nvim" --headless -u NONE -i NONE \
  --cmd "cd $WORK" \
  --cmd 'set noswapfile shell=/bin/sh undolevels=1000' \
  -c 'set columns=80 lines=24 laststatus=0 showtabline=0 ruler noshowcmd report=9999 nomore shortmess=aoOtTIcCF' \
  -c "luafile $HERE/optsweep.lua" \
  -c 'qa!' \
  <"$WORK/empty" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr.raw"
status=$?
set -e

scrub() { # in out
  sed -E \
    -e "s#$WORK#<WORK>#g" \
    -e "s#$HERE/optsweep\.lua#<SCRIPT>#g" \
    -e "s#\.\.\.[^ \"]*/optsweep\.lua#<SCRIPT>#g" \
    -e "s#$RUNTIME#<RT>#g" \
    -e 's#0x[0-9a-f]+#<ADDR>#g' \
    -e 's#\.rs:[0-9]+:[0-9]+#.rs:<LINE>#g' \
    -e 's#\.rs:[0-9]+#.rs:<LINE>#g' \
    <"$1" >"$2"
}

scrub "$OUT/$LABEL.stderr.raw" "$OUT/$LABEL.stderr"
rm -f "$OUT/$LABEL.stderr.raw"

touch "$OUT/$LABEL.opts.raw"
scrub "$OUT/$LABEL.opts.raw" "$OUT/$LABEL.opts"
rm -f "$OUT/$LABEL.opts.raw"

# The report itself is written to stdout by the editor and carries option
# VALUES, half a dozen of which are paths into the sandbox.
scrub "$OUT/$LABEL.txt" "$OUT/$LABEL.txt.scrubbed"
mv "$OUT/$LABEL.txt.scrubbed" "$OUT/$LABEL.txt"

printf 'exit %d\n' "$status" >>"$OUT/$LABEL.txt"

if [[ $OWNED -eq 1 && -z ${OPTSWEEP_KEEP:-} ]]; then
  rm -rf "$WORK"
fi

printf '%s: report %d lines, opts %d lines, stderr %d lines, exit %d\n' \
  "$LABEL" "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.opts")" \
  "$(wc -l <"$OUT/$LABEL.stderr")" "$status"
