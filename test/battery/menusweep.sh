#!/usr/bin/env bash
# Differential oracle for the menu family (batch B19): menu.rs --
# `ex_menu` (253 lines), `add_menu_path` (230), `menu_get_recursive`
# (137), `menuitem_getinfo` (128), `execute_menu` (161),
# `show_menus`/`show_menus_recursive`, `remove_menu`,
# `menu_enable_recurse`, `set_context_in_menu_cmd` /
# `get_menu_name` / `get_menu_names` (completion),
# `get_menu_cmd_modes` / `get_menu_mode_str` / `popup_mode_name`,
# `menu_name_skip` / `menu_namecmp` / `menu_name_equal`,
# `menu_translate_tab_and_shift` / `menu_unescape_name` / `menu_text`,
# `ex_menutranslate` / `menutrans_lookup` / `menu_skip_part`,
# `menu_find` / `menu_getbyname` / `find_menu`, `show_popupmenu` and
# `f_menu_info`.
#
#   menusweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# The gap this closes: B19's survey found menu.rs (2,299 lines, 2,172
# unchecked) behind NO differential at all -- the whole family's only
# coverage was `test_menu` (663 lines / 19 `it`s) and
# `ex_cmds/menu_spec`.  It is the last phase-16 file in that state after
# B18-5 closed `eval/fs.rs`.
#
# MOSTLY TEXTUAL, AND THAT IS THE DESIGN.  Everything a menu does that
# can be observed is a string: the `:menu` listing (`show_menus`), the
# `menu_get()` tree, the `menu_info()` dict, the completion candidates
# and the error messages.  Those run in this `-l` process directly.
# Only two things cannot:
#
#   * `:emenu` in a mode other than Normal.  `execute_menu` picks the
#     mode index from `State` / `restart_edit` / `VIsual_active`, and
#     from an `-l` script the editor is never in Insert or Visual.  s11
#     drives an `--embed` child (b19-3's `Child`) so the mode is real.
#   * `:popup` on a menu that EXISTS.  `show_popupmenu` runs
#     `pum_show_popupmenu`, whose `vgetc()` loop has **no `K_EVENT`
#     arm** -- the whole event loop freezes until a key arrives, so the
#     call never returns in a process that has no keys coming (filed
#     upstream at B19-3).  s12 opens it in a child over `nvim_input`
#     (a FAST call, dispatched even inside that loop) and escapes it
#     with `<Esc>`; the error paths, which return before the loop, are
#     measured here.
#
# THE DEFAULT `PopUp` MENU IS PART OF THE MEASUREMENT.  `-u NONE` does
# not suppress it: `runtime/lua/vim/_defaults.lua` defines fifteen
# entries, and `menu_is_popup` copies each into five hidden per-mode
# roots (`PopUpn`, `PopUpv`, ...).  s0 records it once, deliberately --
# it is the only real-world menu tree in the sweep and the only
# exercise of the popup-copy path against something nobody wrote for a
# test.  EVERY OTHER SECTION REMOVES IT FIRST (`aunmenu *` +
# `aunmenu! *`), so s0 is the one place a runtime edit re-baselines.
#
# Produces, under <outdir>:
#
#   <label>.txt       canonical report, diffed as-is.  One tagged line
#                     per case:
#                       O  captured `:` output
#                       !  the error a command raised
#                       =  a structured answer (menu_get / menu_info /
#                          completion / the editor state after :emenu)
#                       X  one crashprobe child's verdict
#   <label>.struct    canonical (sorted-key) JSON, one line per case,
#                     LC_ALL=C sorted AFTER the scrub.
#   <label>.stderr    what the process and its children wrote to the
#                     prompt.
#
# SANDBOX.  Everything is confined to $WORK, a fresh `mktemp -d` unless
# MENUSWEEP_WORK says otherwise: cwd is $WORK, $HOME is $WORK/home,
# $TMPDIR is $WORK/tmp and $PATH is pinned to $WORK/bin.  `env -i` means
# nothing else leaks in.  THE BINARY IS COPIED TO $WORK/bin/nvim,
# because s11/s12/s91 spawn children and they must find one inside the
# sandbox.
#
# THE WORK DIRECTORY IS A CONSTANT LENGTH (`/tmp/menusweep.XXXXXXXXXX`).
# Two runs whose $WORK differ only in CONTENT are comparable and proving
# that is the acceptance test (menuverify.sh); two whose $WORK differ in
# LENGTH are not.
#
# MENUSWEEP_ONLY is a Lua pattern matched against each section name; it
# exists for iterating on one section, not for gating.
# MENUSWEEP_TRACE=1 mirrors section names to stderr and must be off for
# a baseline, because it writes into the .stderr artifact.

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

LIMIT=${MENUSWEEP_TIMEOUT:-900}

# Constant length, always: `/tmp/menusweep.` + ten characters.
OWNED=0
if [[ -n ${MENUSWEEP_WORK:-} ]]; then
  WORK=$MENUSWEEP_WORK
  rm -rf "$WORK"
  mkdir -p "$WORK"
else
  WORK=$(mktemp -d /tmp/menusweep.XXXXXXXXXX)
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
  MENU_WORK="$WORK" \
  MENU_STRUCT="$OUT/$LABEL.struct.raw" \
  MENUSWEEP_ONLY="${MENUSWEEP_ONLY:-}" \
  MENUSWEEP_TRACE="${MENUSWEEP_TRACE:-}" \
  "$WORK/bin/nvim" --headless -u NONE -i NONE \
  --cmd "cd $WORK" \
  -l "$HERE/menusweep.lua" \
  <"$WORK/empty" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr.raw"
status=$?
set -e

sed -E \
  -e "s#$WORK#<WORK>#g" \
  -e "s#$HERE/menusweep\.lua#<SCRIPT>#g" \
  -e "s#\.\.\.[^ \"]*/menusweep\.lua#<SCRIPT>#g" \
  -e "s#$RUNTIME#<RT>#g" \
  -e 's#nvim\.[0-9]+\.[0-9]+#nvim.<PID>.<SEQ>#g' \
  -e 's#nvim\.[A-Za-z0-9_.-]+/[A-Za-z0-9]+#nvim.<U>/<T>#g' \
  <"$OUT/$LABEL.stderr.raw" >"$OUT/$LABEL.stderr"
rm -f "$OUT/$LABEL.stderr.raw"

printf 'exit %d\n' "$status" >>"$OUT/$LABEL.txt"

touch "$OUT/$LABEL.struct.raw"
LC_ALL=C sort "$OUT/$LABEL.struct.raw" >"$OUT/$LABEL.struct"
rm -f "$OUT/$LABEL.struct.raw"

if [[ $OWNED -eq 1 && -z ${MENUSWEEP_KEEP:-} ]]; then
  rm -rf "$WORK"
fi

printf '%s: report %d lines, struct %d lines, stderr %d lines, exit %d\n' \
  "$LABEL" "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.struct")" \
  "$(wc -l <"$OUT/$LABEL.stderr")" "$status"
