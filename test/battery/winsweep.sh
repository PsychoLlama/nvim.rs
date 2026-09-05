#!/usr/bin/env bash
# Differential oracle for the window family (batch B20): window.rs --
# `do_window`'s letter dispatch, `win_split`/`win_split_ins`,
# `win_equal`/`win_equal_rec`, `frame_new_height`/`frame_new_width`,
# `frame_setheight`/`frame_setwidth`/`frame_minheight`/`frame_minwidth`,
# `winframe_remove`/`winframe_find_altwin`/`winframe_restore`,
# `win_close`/`win_close_othertab`/`close_others`, `win_enter_ext`/
# `win_goto`/`win_goto_ver`/`win_goto_hor`, `win_exchange`/`win_rotate`/
# `win_totop`/`win_splitmove`/`win_move_after`, `win_new_tabpage`/
# `leave_tabpage`/`enter_tabpage`/`tabpage_move`/`goto_tabpage*`,
# `win_setheight_win`/`win_setwidth_win`/`win_new_height`/
# `scroll_to_fraction`/`win_fix_scroll`, `last_status`/`command_height`/
# `win_comp_pos`/`frame_comp_pos`, `win_size_save`/`win_size_restore`
# and the snapshot family -- plus winfloat.rs end to end:
# `win_new_float`, `win_config_float`, `win_set_minimal_style`,
# `win_border_height`/`win_border_width`, `float_zindex_cmp`,
# `win_float_remove` (`:fclose`), `win_check_anchored_floats`,
# `win_float_anchor_laststatus`, `win_reconfig_floats`,
# `win_float_find_altwin`, and window.rs's `ui_ext_win_position`.
#
#   winsweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# The gap this closes: B20's survey S5, holes 1 and 2.  Across all
# SEVENTEEN existing differentials every `wincmd` letter was navigation
# or sizing -- `j w l p t k | _ o c b zz` -- and NONE of
# `x r R H J K L T = + - < > s v n q ^`; nothing anywhere read
# `winrestcmd()`; and every `relative=` was `'editor'` with `anchor`
# appearing exactly ZERO times in any oracle source.
#
# TEXTUAL, ONE PROCESS, AND *NOT* `-l`.  Everything this family does
# that can be observed is a number or a string -- `winlayout()`,
# `winrestcmd()`, `getwininfo()`, `nvim_win_get_config()` -- so the
# sweep is a single headless process, exactly as stlsweep and menusweep
# are.  It is driven by a `--headless` process that runs the sweep from
# a `VimEnter` autocmd rather than by `-l`, and both halves of that are
# deliberate:
#
#   `-l` sets `silent_mode`; `full_screen` is `!silent_mode`; and
#   `did_set_cmdheight` is guarded by `full_screen` -- so under `-l`
#   `'cmdheight'` HAS NO LAYOUT EFFECT AT ALL.  Measured: `set
#   cmdheight=0|2|3|5` leaves every window height unchanged under `-l`
#   and moves them by exactly the delta under `--headless`.
#
# AND IT RUNS FROM `VimEnter`, NOT FROM `-c`.  `exe_commands()` -- the
# `-c` list -- runs at `main/entry.rs:511`, four lines BEFORE
# `RedrawingDisabled.set(0)`, so a `:redraw` issued from `-c` does
# nothing whatsoever.  That matters because a float's SCREEN position
# (`w_winrow`/`w_wincol`, which is what `getwininfo()` reports) is
# written by `ui_ext_win_position` from `win_ui_flush(true)`, called
# only from `update_screen()`.  Without a working redraw the entire
# `relative` x `anchor` matrix answers the config's own `row`/`col`
# back -- 96 identical rows -- and `anchor`, `fixed` and the two
# on-screen clamps are unmeasured.  `snap()` therefore paints TWICE
# before every answer: the first redraw allocates the float's grid
# (`win_ui_flush` skips a window whose `w_grid_alloc.chars` is null),
# the second positions it.
#
# Two sections still need a main input loop and use b19-3's `Child`:
#
#   * s7.  `WinResized` and `WinScrolled` fire from
#     `may_trigger_win_scrolled_resized`, which is called from
#     `normal_check`, `edit`'s redraw and the terminal loop -- never
#     from a script.  A thousand `:resize`s in this process fire
#     neither.  The child turns its loop with a numbered
#     `<Cmd>let g:wntick=N<CR>` marker through the typeahead, because
#     `nvim_input` is FAST and `nvim_exec_lua` DEFERRED and only a key
#     round-trip proves the loop actually turned.
#   * s91.  The inputs that may kill the editor get one child each.
#
# HANDLES ARE NEVER PRINTED RAW.  A window id is a process-global
# counter, so a case that opens a window renumbers every later case's
# ids and an inserted case would re-baseline everything below it.
# `snap()` renumbers every window, buffer and tabpage handle to an
# ordinal assigned in `getwininfo()` order -- tab order, then `winnr`
# order -- which is the order the user sees, so the ordinals are
# themselves an assertion about window ORDER.
#
# Produces, under <outdir>:
#
#   <label>.txt       canonical report, diffed as-is.  One tagged line
#                     per case:
#                       =  a case: the command's own output, then the
#                          whole picture (layout, winrestcmd, every
#                          window's geometry, every float's config)
#                       X  one crashprobe child's verdict
#   <label>.struct    canonical (sorted-key) JSON, one line per case,
#                     LC_ALL=C sorted AFTER the scrub.  Carries EVERY
#                     field the report compresses.
#   <label>.stderr    what the process and its children wrote to the
#                     prompt.  EMPTY at the baseline, and that is the
#                     assertion.
#
# SANDBOX.  Everything is confined to $WORK, a fresh `mktemp -d` unless
# WINSWEEP_WORK says otherwise: cwd is $WORK, $HOME is $WORK/home,
# $TMPDIR is $WORK/tmp and $PATH is pinned to $WORK/bin.  `env -i` means
# nothing else leaks in.  THE BINARY IS COPIED TO $WORK/bin/nvim,
# because s7 and s91 spawn children and they must find one inside the
# sandbox.
#
# THE WORK DIRECTORY IS A CONSTANT LENGTH (`/tmp/winsweep.XXXXXXXXXX`).
# Two runs whose $WORK differ only in CONTENT are comparable and proving
# that is the acceptance test (winverify.sh); two whose $WORK differ in
# LENGTH are not.
#
# WINSWEEP_ONLY is a Lua pattern matched against each section name; it
# exists for iterating on one section, not for gating.
# WINSWEEP_TRACE=1 mirrors section names to stderr and must be off for
# a baseline, because it writes into the .stderr artifact.

set -euo pipefail

if [[ $# -ne 4 ]]; then
  sed -n '2,110p' "$0" >&2
  exit 2
fi

NVIM=$(realpath "$1")
RUNTIME=$(realpath "$2")
OUT=$(realpath -m "$3")
LABEL=$4
HERE=$(cd "$(dirname "$0")" && pwd)

LIMIT=${WINSWEEP_TIMEOUT:-900}

# Constant length, always: `/tmp/winsweep.` + ten characters.
OWNED=0
if [[ -n ${WINSWEEP_WORK:-} ]]; then
  WORK=$WINSWEEP_WORK
  rm -rf "$WORK"
  mkdir -p "$WORK"
else
  WORK=$(mktemp -d /tmp/winsweep.XXXXXXXXXX)
  OWNED=1
fi

mkdir -p "$OUT"
umask 022
mkdir -p "$WORK/home" "$WORK/bin" "$WORK/tmp"
chmod 755 "$WORK" "$WORK/home" "$WORK/bin" "$WORK/tmp"

cp "$NVIM" "$WORK/bin/nvim"
chmod 755 "$WORK/bin/nvim"

: >"$WORK/empty"
# Two real files, so `wincmd ^` has an alternate FILE (it answers E23
# against a scratch buffer) and `:pedit`/`:split <file>` have a target.
printf 'alpha one\nalpha two\nalpha three\n' >"$WORK/alpha.txt"
printf 'beta one\nbeta two\n' >"$WORK/beta.txt"

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
  WIN_WORK="$WORK" \
  WIN_LUA="$HERE/winsweep.lua" \
  WIN_STRUCT="$OUT/$LABEL.struct.raw" \
  WINSWEEP_ONLY="${WINSWEEP_ONLY:-}" \
  WINSWEEP_TRACE="${WINSWEEP_TRACE:-}" \
  "$WORK/bin/nvim" --headless -u NONE -i NONE \
  --cmd "cd $WORK" \
  --cmd 'autocmd VimEnter * ++once lua local ok, e = pcall(dofile, vim.env.WIN_LUA) if not ok then io.stderr:write("SWEEP-FATAL " .. tostring(e) .. "\n") end vim.cmd("qa!")' \
  <"$WORK/empty" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr.raw"
status=$?
set -e

sed -E \
  -e "s#$WORK#<WORK>#g" \
  -e "s#$HERE/winsweep\.lua#<SCRIPT>#g" \
  -e "s#\.\.\.[^ \"]*/winsweep\.lua#<SCRIPT>#g" \
  -e "s#$RUNTIME#<RT>#g" \
  -e 's#nvim\.[0-9]+\.[0-9]+#nvim.<PID>.<SEQ>#g' \
  -e 's#nvim\.[A-Za-z0-9_.-]+/[A-Za-z0-9]+#nvim.<U>/<T>#g' \
  <"$OUT/$LABEL.stderr.raw" >"$OUT/$LABEL.stderr"
rm -f "$OUT/$LABEL.stderr.raw"

printf 'exit %d\n' "$status" >>"$OUT/$LABEL.txt"

touch "$OUT/$LABEL.struct.raw"
LC_ALL=C sort "$OUT/$LABEL.struct.raw" >"$OUT/$LABEL.struct"
rm -f "$OUT/$LABEL.struct.raw"

if [[ $OWNED -eq 1 && -z ${WINSWEEP_KEEP:-} ]]; then
  rm -rf "$WORK"
fi

printf '%s: report %d lines, struct %d lines, stderr %d lines, exit %d\n' \
  "$LABEL" "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.struct")" \
  "$(wc -l <"$OUT/$LABEL.stderr")" "$status"
