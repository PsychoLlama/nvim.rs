#!/usr/bin/env bash
# Differential oracle for the vterm family (batch B21):
# crates/nvim/src/nvim/vterm/ -- parser.rs (`vterm_input_write`, the
# escape-sequence state machine, the C0/C1 dispatch and the control
# strings), state.rs (`on_text`/`on_control`/`on_escape`/`on_csi`/
# `on_osc`/`on_dcs`, the cursor and scroll-region arithmetic, the mode
# table, DECRQSS, the selection decoder and the key-encoding stack),
# screen.rs (the cell buffers, the damage merge, scrollback push/pop,
# reflow, the altscreen), pen.rs (the whole SGR alphabet), csi.rs,
# dcs.rs, mode.rs, damage.rs, cell.rs, color.rs, geometry.rs,
# output.rs, selection.rs, text.rs, encoding.rs, keyboard.rs and
# mouse.rs -- plus the 33 `repr(C)` types in
# types/vterm{,_internal,_keycodes}.rs.
#
#   vtsweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# The gap this closes: B21's survey S3, hole 2 (no differential
# anywhere over the emulator) and hole 1 (upstream's own
# `62screen_damage` case is `pending()`, so damage merging has NO unit
# coverage at all -- section v9 is that coverage).
#
# `nvim -u NONE -i NONE -l`, NOT `--headless`, NO UI, NO PTY, NO
# `:terminal`.  The emulator's thirty `#[no_mangle]` entry points are
# exported from the binary, so a Lua script reaches them through
# `ffi.C` with nothing between: no autocommands, no main loop, no
# generated `unit-cdefs.h` and no compiled fixture.  That is what makes
# the sweep runnable, unchanged, against a kept binary built months
# earlier -- the cdef travels with the sweep instead of with the tree.
#
# WHY `-l` AND NOT `-ll`.  `-ll` is the tighter mode and was the first
# choice: no editor at all.  It CRASHES.  `utf_char2cells` reads
# `p_ambw` ('ambiwidth') unconditionally on the way to deciding a
# non-wide character's width, and under `-ll` no option has been
# initialised -- the pointer is null, and the first leading combining
# character the emulator has to measure aborts the process
# (`mbyte/cells.rs:99`, "null pointer dereference").  Under `-ll`
# `vim.o`, `vim.api` and `vim.fn` do not exist either, so the option
# cannot be set from the script, and `p_ambw` is not an exported
# symbol.  `-l` runs the same script with the options initialised to
# their compiled defaults (`ambiwidth=single`, `emoji=on`) and is just
# as textual and just as loop-less.  THAT IS ALSO WHY $VIMRUNTIME IS AN
# ARGUMENT: `-l` starts an editor, and it must find one.  Nothing the
# sweep answers depends on the runtime -- `-u NONE -i NONE` means no
# vimrc, no shada and no plugin -- but the process will complain
# without it.
#
# THE cdef IS AN INDEPENDENT STATEMENT OF THE ABI, and section v0 is the
# assertion that it still holds -- `ffi.sizeof`/`ffi.alignof` of every
# type that crosses, `ffi.offsetof` of every field the later sections
# read, and seven live probes that drive the emulator and read the value
# back out of the struct.  A layout drift shows up in v0 as a changed
# number instead of as garbage two thousand rows lower.
#
# ONE ABI DETAIL IS LOAD-BEARING (see the .lua header for the long
# version).  LuaJIT cannot build a callback whose C prototype takes a
# struct BY VALUE, and libvterm's damage, moverect, movecursor,
# putglyph, erase, scrollrect, osc, dcs, apc, pm, sos and selection-set
# slots all do.  The sweep declares those slots with the System V
# x86-64 REGISTER DECOMPOSITION of the same signature (a `VTermRect` is
# two general registers, a `VTermPos` one, a `VTermStringFragment` two)
# and reassembles the struct on the Lua side.  This is x86-64 SysV only.
#
# Produces, under <outdir>:
#
#   <label>.txt       canonical report, diffed as-is.  One line per
#                     case: the label, then the answer -- the emulator's
#                     callback stream, the bytes it wrote back, the
#                     residual damage bookkeeping, every scalar of
#                     `VTermState`, and the whole cell grid run-length
#                     encoded per row with its lineinfo.
#   <label>.struct    canonical (sorted-key) JSON, one line per case,
#                     LC_ALL=C sorted AFTER the scrub.  Carries the
#                     uncapped forms of what the report caps.
#   <label>.stderr    what the process and its v91 children wrote to the
#                     prompt.  EMPTY at the baseline, and that is the
#                     assertion.
#
# SANDBOX.  Everything is confined to $WORK, a fresh `mktemp -d` unless
# VTSWEEP_WORK says otherwise: cwd is $WORK, $HOME is $WORK/home,
# $TMPDIR is $WORK/tmp and $PATH is pinned to $WORK/bin.  `env -i` means
# nothing else leaks in.  THE BINARY IS COPIED TO $WORK/bin/nvim,
# because v91 spawns children and they must find one inside the sandbox.
# The work directory is a CONSTANT LENGTH (`/tmp/vtsweep.XXXXXXXXXX`).
#
# VTSWEEP_ONLY is a Lua pattern matched against each section name; it
# exists for iterating on one section, not for gating.  VTSWEEP_TRACE=1
# mirrors section names to stderr and must be off for a baseline,
# because it writes into the .stderr artifact.

set -euo pipefail

if [[ $# -ne 4 ]]; then
  sed -n '2,95p' "$0" >&2
  exit 2
fi

NVIM=$(realpath "$1")
RUNTIME=$(realpath "$2")
OUT=$(realpath -m "$3")
LABEL=$4
HERE=$(cd "$(dirname "$0")" && pwd)

LIMIT=${VTSWEEP_TIMEOUT:-1800}

OWNED=0
if [[ -n ${VTSWEEP_WORK:-} ]]; then
  WORK=$VTSWEEP_WORK
  rm -rf "$WORK"
  mkdir -p "$WORK"
else
  WORK=$(mktemp -d /tmp/vtsweep.XXXXXXXXXX)
  OWNED=1
fi

mkdir -p "$OUT"
umask 022
mkdir -p "$WORK/home" "$WORK/bin" "$WORK/tmp"
chmod 755 "$WORK" "$WORK/home" "$WORK/bin" "$WORK/tmp"

cp "$NVIM" "$WORK/bin/nvim"
chmod 755 "$WORK/bin/nvim"
# v91 re-executes the sweep as a child; give it a copy inside the
# sandbox so no host path is ever on a child's command line.
cp "$HERE/vtsweep.lua" "$WORK/vtsweep.lua"
# v91's children are spawned through `/bin/sh` with $PATH pinned to the
# sandbox, so the one external tool they need has to be inside it.
ln -sf "$(command -v timeout)" "$WORK/bin/timeout"

set +e
timeout -k 5 "$LIMIT" \
  env -i \
  HOME="$WORK/home" \
  PATH="$WORK/bin" \
  TMPDIR="$WORK/tmp" \
  TERM=dumb \
  SHELL=/bin/sh \
  LANG=C.UTF-8 \
  XDG_CONFIG_HOME="$WORK/home/.config" \
  XDG_DATA_HOME="$WORK/home/.local/share" \
  XDG_STATE_HOME="$WORK/home/.local/state" \
  XDG_CACHE_HOME="$WORK/home/.cache" \
  NVIM_TEST=1 \
  VIMRUNTIME="$RUNTIME" \
  VT_NVIM="$WORK/bin/nvim" \
  VT_SCRIPT="$WORK/vtsweep.lua" \
  VT_STRUCT="$OUT/$LABEL.struct.raw" \
  VTSWEEP_ONLY="${VTSWEEP_ONLY:-}" \
  VTSWEEP_TRACE="${VTSWEEP_TRACE:-}" \
  "$WORK/bin/nvim" -u NONE -i NONE -l "$WORK/vtsweep.lua" \
  </dev/null \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr.raw"
status=$?
set -e

sed -E \
  -e "s#$WORK#<WORK>#g" \
  -e "s#$HERE#<TOOLS>#g" \
  -e "s#$RUNTIME#<RT>#g" \
  <"$OUT/$LABEL.stderr.raw" >"$OUT/$LABEL.stderr"
rm -f "$OUT/$LABEL.stderr.raw"

# The report can carry the work directory only through a child's stderr,
# which v91 folds into its own row; scrub it the same way.
sed -E -i -e "s#$WORK#<WORK>#g" -e "s#$HERE#<TOOLS>#g" -e "s#$RUNTIME#<RT>#g" "$OUT/$LABEL.txt"

printf 'exit %d\n' "$status" >>"$OUT/$LABEL.txt"

touch "$OUT/$LABEL.struct.raw"
sed -E -e "s#$WORK#<WORK>#g" -e "s#$HERE#<TOOLS>#g" -e "s#$RUNTIME#<RT>#g" "$OUT/$LABEL.struct.raw" \
  | LC_ALL=C sort >"$OUT/$LABEL.struct"
rm -f "$OUT/$LABEL.struct.raw"

if [[ $OWNED -eq 1 && -z ${VTSWEEP_KEEP:-} ]]; then
  rm -rf "$WORK"
fi

printf '%s: report %d lines, struct %d lines, stderr %d lines, exit %d\n' \
  "$LABEL" "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.struct")" \
  "$(wc -l <"$OUT/$LABEL.stderr")" "$status"
