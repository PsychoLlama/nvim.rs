#!/usr/bin/env bash
# Differential oracle for the marktree family (batch B22):
# crates/nvim/src/nvim/marktree.rs and marktree/{splice,iter,rebalance,
# check,inspect,node,intersect,key,meta}.rs -- `marktree_put` /
# `marktree_del_itr` / `marktree_splice` / `marktree_move`, `split_node` /
# `merge_node` / `pivot_left` / `pivot_right`, the iterator (plain,
# filtered and overlap walks), the intersection sets, the meta counts and
# `mt_inspect` itself.
#
#   marksweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# THE HOLE IT FILLS.  `marktree_check` / `marktree_check_intersections`
# have ZERO production callers -- they run under `test/unit/
# marktree_spec.lua` and nowhere else -- so no oracle in the tree could
# see tree SHAPE through a running editor.  The functional suite passes
# over a structurally corrupt tree for as long as the marks that come
# back happen to be right, which is exactly what splice.rs and
# rebalance.rs get wrong when they get anything wrong.
#
# THE OBSERVABLE is `nvim__buf_debug_extmarks(buf, keys, dot)` -- the
# node-by-node dump WITH each node's intersection set, at full internal
# resolution -- plus `nvim_buf_get_extmarks(.., {details = true})` for
# every case.  Nothing in `test/` called that API before this sweep, so
# the sweep gates the API as well as the tree.
#
# DETERMINISM.  The dump carries no address: the plain format is
# positions and ids, and the `dot` format names each node by its PARENT
# CHAIN (`MTNode_b1_a3` = 'a'+level, p_idx), which the Lua renumbers to
# emission-order ordinals anyway.  Every case builds its own buffer and
# its own namespace, so extmark ids restart at 1 and an inserted case
# renumbers nothing below it.  No buffer or window HANDLE is ever
# printed.  Randomised sections use an LCG with a fixed seed.
#
# `--headless -c`, NOT `-l`: m6 reads `getwininfo().textoff` after a
# `redraw`, and under `-l` `full_screen` is false and the redraw is not
# the same one (B20-2).  Nothing here needs a pty or an RPC child except
# m91, whose probes are plain `--headless -c` children.
#
# Produces, under <outdir>:
#
#   <label>.txt     canonical report: one line per case, a
#                   `## <section> rows=N` line per section, `## TOTAL`
#                   and a final `exit N`.
#   <label>.tree    the SHAPE layer: per case, the per-level aggregate,
#                   one normalised line per node (`n3 par=n0 lvl=0
#                   pidx=2 nk=19 ix=[..] k=[..]`) and the plain
#                   positional dump verbatim, wrapped at 120 columns.
#                   Trees over 80 nodes / 4,000 dump bytes are digested.
#   <label>.marks   the BEHAVIOUR layer: per case, every mark the
#                   details walk returns, rendered with a FIXED field
#                   order (the details dict is a Lua hash; iterating it
#                   would answer a different order run to run).
#   <label>.stderr  what the process and its children wrote to the
#                   prompt.  EMPTY at the baseline; that is the
#                   assertion.
#
# The three layers answer three different questions, which is the whole
# point of splitting them: `.marks` moving alone is a behaviour change a
# user can see; `.tree` moving alone is a shape change no other oracle in
# this tree can see at all; `.txt` moving alone is a count or a lever.
#
# SANDBOX.  $WORK is a fresh `mktemp -d` unless MARKSWEEP_WORK says
# otherwise; cwd is $WORK, $HOME is $WORK/home, $TMPDIR is $WORK/tmp and
# $PATH is $WORK/bin, which holds a copy of the binary under test and
# nothing else.  `env -i`.
#
# ORPHANS.  m91 runs ~20 plain `--headless` children through `system()`,
# each of which has exited before the row is printed.  If a run is killed
# by the timeout, look for children with ppid 1 and KILL THEM BY PID --
# never by pattern, which would take out the operator's own editor.
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

LIMIT=${MARKSWEEP_TIMEOUT:-900}

OWNED=0
if [[ -n ${MARKSWEEP_WORK:-} ]]; then
  WORK=$MARKSWEEP_WORK
  rm -rf "$WORK"
  mkdir -p "$WORK"
else
  WORK=$(mktemp -d /tmp/marksweep.XXXXXXXXXX)
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
  MARK_WORK="$WORK" \
  MARK_NVIM="$WORK/bin/nvim" \
  MARK_TREE="$OUT/$LABEL.tree.raw" \
  MARK_MARKS="$OUT/$LABEL.marks.raw" \
  MARKSWEEP_ONLY="${MARKSWEEP_ONLY:-}" \
  "$WORK/bin/nvim" --headless -u NONE -i NONE \
  --cmd "cd $WORK" \
  --cmd 'set noswapfile shell=/bin/sh undolevels=1000' \
  -c 'set columns=80 lines=24 laststatus=0 showtabline=0 ruler noshowcmd report=9999 nomore shortmess=aoOtTIcCF' \
  -c "luafile $HERE/marksweep.lua" \
  -c 'qa!' \
  <"$WORK/empty" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr.raw"
status=$?
set -e

scrub() { # in out
  sed -E \
    -e "s#$WORK#<WORK>#g" \
    -e "s#$HERE/marksweep\.lua#<SCRIPT>#g" \
    -e "s#\.\.\.[^ \"]*/marksweep\.lua#<SCRIPT>#g" \
    -e "s#$RUNTIME#<RT>#g" \
    -e 's#0x[0-9a-f]+#<ADDR>#g' \
    -e 's#\.rs:[0-9]+:[0-9]+#.rs:<LINE>#g' \
    <"$1" >"$2"
}

scrub "$OUT/$LABEL.stderr.raw" "$OUT/$LABEL.stderr"
rm -f "$OUT/$LABEL.stderr.raw"

for part in tree marks; do
  touch "$OUT/$LABEL.$part.raw"
  scrub "$OUT/$LABEL.$part.raw" "$OUT/$LABEL.$part"
  rm -f "$OUT/$LABEL.$part.raw"
done

printf 'exit %d\n' "$status" >>"$OUT/$LABEL.txt"

if [[ $OWNED -eq 1 && -z ${MARKSWEEP_KEEP:-} ]]; then
  rm -rf "$WORK"
fi

printf '%s: report %d lines, tree %d lines, marks %d lines, stderr %d lines, exit %d\n' \
  "$LABEL" "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.tree")" \
  "$(wc -l <"$OUT/$LABEL.marks")" "$(wc -l <"$OUT/$LABEL.stderr")" "$status"
