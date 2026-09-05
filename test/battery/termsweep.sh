#!/usr/bin/env bash
# Differential oracle for the terminal family (batch B21):
# crates/nvim/src/nvim/terminal.rs and terminal/{mode,refresh,callbacks,
# input,scrollback,termrequest}.rs -- `terminal_open`/`terminal_alloc`,
# `terminal_receive` and the vterm feed, `refresh_terminal` /
# `refresh_screen` / `refresh_size` / `refresh_scrollback` /
# `refresh_cursor`, `terminal_check_size`, `term_sb_push` / `term_sb_pop`
# / `term_sb_clear` and `adjust_scrollback`, the OSC/DCS termprops and
# the `TermRequest` autocommand, the reply writer,
# `terminal_get_line_attributes` (which NO oracle read before this one),
# terminal-mode key and mouse encoding, and the close/wipe path.
#
#   termsweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# ADOPTED FROM `test/battery/1785449630-termsweep.{sh,lua}` (phase 15,
# B8/B9), which was never baselined.  What changed is in
# b21-4-termsweep.md; the short version is that the old tool ran twenty
# `jobstart(pty)` scenarios in twenty processes and settled each one
# with `vim.wait(400)` + `redraw` + `vim.wait(200)`, which is a race,
# not a barrier.  The scenarios are all still here, by their old names,
# but the ones whose subject is the emulator and the scrollback are
# driven the way B21-2 proved for `termchurn` instead.
#
# THE DRIVE, and why it is deterministic.  `nvim_open_term` +
# `nvim_chan_send` enters at exactly the `terminal_receive` a pty read
# takes -- same vterm feed, same damage callbacks, same `term_sb_push`,
# same `invalidate_terminal` -- with no child process, no pty and no
# scheduler in the answer.  The refresh it schedules is normally
# deferred behind a 10 ms timer; the one synchronous door into
# `refresh_terminal` from script is `did_set_scrollback`, which fires
# `on_scrollback_option_changed` ONLY WHEN THE VALUE SHRINKS, so the
# sweep grows `'scrollback'` by one and shrinks it back and the buffer
# is final on the next line.  Everything that is genuinely asynchronous
# -- a `TermRequest` autocommand, a reply written back through
# `on_input`, a pty child's exit -- is waited for BY ITS EFFECT
# (`vim.wait(N, function() return <the effect> end, 1)`), never by a
# fixed sleep.
#
# `--headless -c`, NOT `-l`: `-l` leaves `full_screen` false, and
# without it `'columns'` does not resize the grid (B20-2) -- the
# terminal's own size comes from the window, so every geometry case
# would answer the compiled default instead.  A UI is attached only in
# t4, and there only in a child.
#
# Produces, under <outdir>:
#
#   <label>.txt       canonical report, diffed as-is.  One line per
#                     case, plus a `## <section> rows=N` line per
#                     section and a final `exit N`.
#   <label>.struct    canonical (sorted-key) JSON, one line per case,
#                     LC_ALL=C sorted AFTER the scrub.
#   <label>.stderr    what the process and its children wrote to the
#                     prompt.  EMPTY at the baseline; that is the
#                     assertion.
#
# SANDBOX.  $WORK is a fresh `mktemp -d` unless TERM_WORK says
# otherwise; cwd is $WORK, $HOME is $WORK/home, $TMPDIR is $WORK/tmp and
# $PATH is $WORK/bin, which holds a copy of the binary under test and
# nothing else -- so a `:terminal` in t7 finds `/bin/sh` by absolute
# path and nothing of this machine leaks into an answer.  `env -i`.
#
# ORPHANS.  t4, t5 and t91 run `--embed` children; every one is
# `jobstop`ped and `jobwait`ed.  If a run is killed by the timeout, look
# for children with ppid 1 and KILL THEM BY PID -- never by pattern,
# which would take out the operator's own editor (cmdsweep s18).
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

LIMIT=${TERMSWEEP_TIMEOUT:-900}

# Constant length, always, and digits only after the prefix: a work
# directory shows up inside `b:term_title` and inside `:terminal`'s own
# buffer name, and a `mktemp` suffix that happened to carry a letter
# would change a completion answer (the bufsweep lesson).
OWNED=0
if [[ -n ${TERMSWEEP_WORK:-} ]]; then
  WORK=$TERMSWEEP_WORK
  rm -rf "$WORK"
  mkdir -p "$WORK"
else
  WORK=$(mktemp -d /tmp/termsweep.XXXXXXXXXX)
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
  TERM_WORK="$WORK" \
  TERM_STRUCT="$OUT/$LABEL.struct.raw" \
  TERMSWEEP_ONLY="${TERMSWEEP_ONLY:-}" \
  TERMSWEEP_TRACE="${TERMSWEEP_TRACE:-}" \
  "$WORK/bin/nvim" --headless -u NONE -i NONE \
  --cmd "cd $WORK" \
  --cmd 'set noswapfile shell=/bin/sh' \
  -c 'set columns=80 lines=24 laststatus=0 showtabline=0 ruler noshowcmd report=9999 nomore shortmess=aoOtTIcCF' \
  -c "luafile $HERE/termsweep.lua" \
  -c 'qa!' \
  <"$WORK/empty" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr.raw"
status=$?
set -e

sed -E \
  -e "s#$WORK#<WORK>#g" \
  -e "s#$HERE/termsweep\.lua#<SCRIPT>#g" \
  -e "s#\.\.\.[^ \"]*/termsweep\.lua#<SCRIPT>#g" \
  -e "s#$RUNTIME#<RT>#g" \
  -e 's#nvim\.[0-9]+\.[0-9]+#nvim.<PID>.<SEQ>#g' \
  -e 's#nvim\.[A-Za-z0-9_.-]+/[A-Za-z0-9]+#nvim.<U>/<T>#g' \
  <"$OUT/$LABEL.stderr.raw" >"$OUT/$LABEL.stderr"
rm -f "$OUT/$LABEL.stderr.raw"

printf 'exit %d\n' "$status" >>"$OUT/$LABEL.txt"

touch "$OUT/$LABEL.struct.raw"
LC_ALL=C sort "$OUT/$LABEL.struct.raw" >"$OUT/$LABEL.struct"
rm -f "$OUT/$LABEL.struct.raw"

if [[ $OWNED -eq 1 && -z ${TERMSWEEP_KEEP:-} ]]; then
  rm -rf "$WORK"
fi

printf '%s: report %d lines, struct %d lines, stderr %d lines, exit %d\n' \
  "$LABEL" "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.struct")" \
  "$(wc -l <"$OUT/$LABEL.stderr")" "$status"
