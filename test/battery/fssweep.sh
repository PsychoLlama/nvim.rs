#!/usr/bin/env bash
# Differential oracle for the eval filesystem family (batch B18):
# eval/fs.rs and its six children (name, path, find, dir, read, write) --
# the thirty-three Vimscript builtins that ask the filesystem a question
# or change it.  B18's survey found this family with **no differential at
# all**: zero evalsweep/varsweep hits across all twenty-odd of them.
#
#   fssweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# The gap this closes: `modify_fname` is 331 lines of `*mut char` cursor
# arithmetic behind ONE builtin, `fs/dir.rs` (562 lines) is the whole
# mutating half, and the functional/old suites reach both only through
# incidental use.  A wrong answer here is a silently wrong path, not an
# error -- so every case reports the ANSWER, and the mutating cases
# report the resulting TREE, not "did it succeed".
#
# Produces, under <outdir>:
#
#   <label>.txt       canonical report, read top to bottom and diffed
#                     as-is.  One tagged line per answer:
#                       =  the value the builtin returned
#                       !  the error it raised
#                       T  a canonical listing of a directory tree
#                       A  the case's own extra question
#   <label>.struct    canonical (sorted-key) JSON, one line per labelled
#                     answer, LC_ALL=C sorted AFTER the scrub, so the
#                     diff does not depend on where in the run an answer
#                     was produced.
#   <label>.stderr    what nvim wrote to the prompt.  s90 runs a block of
#                     failing commands *uncaptured*, which is the only
#                     view of this family's message path (E484/E485/E739
#                     /E482/E739, `:cd` and `:mkdir` failures).
#
# SANDBOX.  Everything is confined to $WORK, a fresh `mktemp -d` unless
# FSSWEEP_WORK says otherwise: cwd is $WORK, $HOME is $WORK/home, $TMPDIR
# is $WORK/tmp (so `tempname()` lands inside the sandbox and is scrubbed
# with everything else), and **$PATH is pinned to $WORK/bin** so that
# `executable()` and `exepath()` answer from a fixture and not from the
# caller's environment.  `env -i` means nothing else leaks in.  The umask
# is fixed at 022 and every fixture file is explicitly chmod'ed, because
# `getfperm` is one of the answers.
#
# THE BINARY IS COPIED TO $WORK/bin/nvim and run from there, so that
# `exepath('nvim')` is a fixture answer and s91's children have a binary
# reachable from inside the sandbox.
#
# THE WORK DIRECTORY IS A CONSTANT LENGTH (`/tmp/fssweep.XXXXXXXXXX`).
# Messages are truncated to the headless 80-column screen and a longer
# path leaves half-elided text behind that the scrub cannot recognise;
# two runs whose $WORK differ in LENGTH are not comparable.  Two runs
# whose $WORK differ only in content are, and proving that is the
# acceptance test for this oracle -- see fsverify.sh.
#
# THE FIXTURE IS REBUILT PER SECTION, each in its own root under
# $WORK/f/<section>, and s6 rebuilds it once more per numbered
# subdirectory so that no mutating case can observe another's leftovers.
# It is built with `vim.uv` directly, never with the builtins under test.
#
# NOT SORTED: `readdir()` and `glob()`.  `readdir_core` calls
# `sort_strings` on its own result and `gen_expand_wildcards` sorts too,
# so their order IS behaviour and IS deterministic; sorting them here
# would blind the oracle to exactly the regression it exists to catch.
# (The B18 plan asked for a sort on the assumption that filesystem order
# leaked through.  It does not -- verified in fileio/tempfile.rs:280.)
#
# NOT PRINTED: `getftime()`'s value.  s8 sets three known mtimes with
# `fs_utime` and reports only the ORDERING among them plus `> 0`.
# `tempname()` is reduced to its shape.
#
# The nvim invocation is wrapped in `timeout` and its exit status is
# appended to the report as a final `exit <code>` line: 124 is the
# harness's verdict on a wedge, 134 an abort.  s91 runs the inputs that
# may kill the editor in a *child* each, so a crash there is one
# diffable ABORTED row rather than a truncated report.
#
# FSSWEEP_ONLY is a Lua pattern matched against each section name; it
# exists for iterating on one section, not for gating.
# FSSWEEP_TRACE=1 mirrors each section name to stderr and must be off
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

LIMIT=${FSSWEEP_TIMEOUT:-900}

# Constant length, always: `/tmp/fssweep.` + ten characters.  A caller
# that overrides it has to keep the length if the output is to be
# compared with a baseline.
OWNED=0
if [[ -n ${FSSWEEP_WORK:-} ]]; then
  WORK=$FSSWEEP_WORK
  rm -rf "$WORK"
  mkdir -p "$WORK"
else
  WORK=$(mktemp -d /tmp/fssweep.XXXXXXXXXX)
  OWNED=1
fi

mkdir -p "$OUT"
umask 022
mkdir -p "$WORK/home" "$WORK/bin" "$WORK/tmp" "$WORK/f"
chmod 755 "$WORK" "$WORK/home" "$WORK/bin" "$WORK/tmp" "$WORK/f"

cp "$NVIM" "$WORK/bin/nvim"
chmod 755 "$WORK/bin/nvim"

# The $PATH fixture.  `executable()` and `exepath()` answer from exactly
# these: an executable, a file that is not, a directory whose name looks
# like a command, and a symlink to the executable.
printf '#!/bin/sh\nexit 0\n' >"$WORK/bin/fixexe"
chmod 755 "$WORK/bin/fixexe"
printf 'not executable\n' >"$WORK/bin/fixnoexe"
chmod 644 "$WORK/bin/fixnoexe"
mkdir -p "$WORK/bin/fixdir"
chmod 755 "$WORK/bin/fixdir"
ln -sf fixexe "$WORK/bin/fixlink"

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
  FS_WORK="$WORK" \
  FS_STRUCT="$OUT/$LABEL.struct.raw" \
  FSSWEEP_ONLY="${FSSWEEP_ONLY:-}" \
  FSSWEEP_TRACE="${FSSWEEP_TRACE:-}" \
  "$WORK/bin/nvim" --headless -u NONE -i NONE \
  --cmd "cd $WORK" \
  -l "$HERE/fssweep.lua" \
  <"$WORK/empty" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr.raw"
status=$?
set -e

# Scrubs the driver has to do rather than the Lua, because they land in
# messages the sweep never gets to touch.
sed -E \
  -e "s#$WORK#<WORK>#g" \
  -e "s#$HERE/fssweep\.lua#<SCRIPT>#g" \
  -e "s#\.\.\.[^ \"]*/fssweep\.lua#<SCRIPT>#g" \
  -e "s#$RUNTIME#<RT>#g" \
  -e 's#nvim\.[0-9]+\.[0-9]+#nvim.<PID>.<SEQ>#g' \
  -e 's#nvim\.[A-Za-z0-9_.-]+/[A-Za-z0-9]+#nvim.<U>/<T>#g' \
  <"$OUT/$LABEL.stderr.raw" >"$OUT/$LABEL.stderr"
rm -f "$OUT/$LABEL.stderr.raw"

printf 'exit %d\n' "$status" >>"$OUT/$LABEL.txt"

touch "$OUT/$LABEL.struct.raw"
LC_ALL=C sort "$OUT/$LABEL.struct.raw" >"$OUT/$LABEL.struct"
rm -f "$OUT/$LABEL.struct.raw"

if [[ $OWNED -eq 1 && -z ${FSSWEEP_KEEP:-} ]]; then
  # The mode-000 fixture file is readable-by-owner-less but the
  # directory holding it is not, so a plain `rm -rf` is enough.
  rm -rf "$WORK"
fi

printf '%s: report %d lines, struct %d lines, stderr %d lines, exit %d\n' \
  "$LABEL" "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.struct")" \
  "$(wc -l <"$OUT/$LABEL.stderr")" "$status"
