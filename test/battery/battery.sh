#!/usr/bin/env bash
# Run all thirty-two baselined differentials + the paired startup probe.
#
# THIS IS THE CANONICAL COPY.  Every close before phase 20 kept the
# battery in its own session scratchpad and copied it forward, which is
# how three finished oracles sat unwired for a phase, and the same habit
# is what cost `spellverify` its baseline (p19-close).  It now lives
# beside the verify scripts it calls, like every baseline does.  Earlier
# copies (p18-close/, p19-close/, ...) are history, not the battery.
#
# P27-3 moved the whole harness into the checkout as test/battery/, so the
# only copy of the thirty-two baselines is no longer one developer's home
# directory.  Read README.md before re-cutting anything.
#
#   just battery [label]
#   battery.sh <label>
#   BATTERY_LOGDIR=... battery.sh <label>
#
# Logs land in $BATTERY_LOGDIR (default $REPO/target/battery-logs).
# B17-5 added `sessgold` (the :mksession byte-golden) and the rehomed
# ex probe (parse + excmd), both baselined at cda7f911f3.
# B18-5 added `fssweep` (the eval filesystem family), baselined at
# 04c4762a05.  It is the slowest single oracle here (~30 s, almost all
# of it s91's 54 children) and takes the battery to ~8 min.
# B19-2 added `stlsweep` (statusline.rs -- `build_stl_str_hl`, the
# click-definition arenas, the ruler and the tabline), baselined at
# 5e23ad6128.  It costs ~3 s.
# B19-3 added `mousesweep` (mouse.rs -- `do_mouse`, `jump_to_mouse`,
# the wheel, the click-definition dispatch and `getmousepos()`), also
# baselined at 5e23ad6128.  It costs ~15 s; every case runs in an
# `--embed` child, because only the main input loop dispatches a mouse
# key.
# B19-4 added `menusweep` (menu.rs -- `ex_menu`, `add_menu_path`, the
# listing, `menu_get`/`menu_info`, completion, `:menutranslate`,
# `:emenu` and `:popup`), also baselined at 5e23ad6128.  It costs ~11 s;
# s11/s12 and s91 run in `--embed` children, because `:emenu` outside
# Normal mode needs a real mode and `:popup` freezes the event loop.
# B20-2 added `winsweep` (window.rs + winfloat.rs -- every `wincmd`
# letter, the split/close/only/hide matrix, the resize and geometry
# options, tabpages, the float `relative` x `anchor` x `zindex` x
# `border` x title/footer x `bufpos` matrix, `nvim_win_set_config` and
# the Win*/Tab* autocmd ORDER), baselined at 46c4bf3a3a.  It costs
# ~11 s; s7 and s91 run in `--embed` children, because
# `WinResized`/`WinScrolled` fire only from `normal_check`.
# B20-3 added `bufsweep` (buffer.rs -- the `:ls` flag matrix and its
# column-40 pad, buffer addressing and the `:bnext` walk, the
# `:bdelete`/`:bwipeout`/`:bunload` lifecycle, `buflist_findpat`,
# `ExpandBufnames` including the `'wildmode'` `lastused` rotation,
# `fileinfo`/CTRL-G, a `chk_modeline` corpus, `getbufinfo()` and the
# Buf* autocmd ORDER), baselined at 46c4bf3a3a.  It costs ~12 s; b3, b9,
# b91 and b5's `'wildmode'` block run ONE FRESH CHILD PER CASE, because
# a raw bufnr is part of the answer and only a fresh editor makes it
# stable.
# B21-3 added `vtsweep` (crates/nvim/src/nvim/vterm/ -- the escape
# sequence parser and its split-write resumption, every CSI, the SGR pen
# alphabet incl. 256/RGB/colon sub-parameters and DECRQSS, the mode
# table and DECSTBM/DECOM/DECAWM/altscreen/kitty stack, the UTF-8
# decoder and the charset designators, OSC/DCS/APC/PM/SOS and the OSC 52
# selection decoder, the X10/UTF8/RXVT/SGR mouse encoders, every
# VTermKey over DECCKM/DECKPAM/LNM, DAMAGE MERGING at all four
# VTermDamageSize levels -- the one thing upstream's own vterm_spec
# leaves `pending()` -- resize/reflow/scrollback, and the layout of all
# 33 `repr(C)` types), baselined at 74ff723db9.  It costs ~5 s.  It is
# the only oracle here that needs NO editor state: `nvim -u NONE -i NONE
# -l` plus a self-contained `ffi.cdef` reaching the emulator's thirty
# `no_mangle` entry points through `ffi.C`.  Its 32 v91 children are
# plain `-l` processes; no pty, no RPC, no orphans possible.
# B21-4 added `termsweep` (terminal.rs + terminal/{mode,refresh,
# callbacks,input,scrollback,termrequest}.rs -- `terminal_open` and the
# `g:terminal_color_N` palette, `terminal_receive` and the vterm feed,
# the damage/settermprop/sb_push/sb_pop callbacks, `refresh_*`,
# `terminal_check_size` and the scrollback ring,
# `terminal_get_line_attributes` PER CELL (which no oracle read before),
# terminal-mode key and mouse encoding, `terminal_enter`/`leave`,
# `TermRequest` + the reply writer + `terminal_notify_theme` + the
# OSC 52 clipboard decoder, and the real pty path with its
# `[Process exited]` extmark), ADOPTED from the never-baselined
# phase-15 tool and baselined at 74ff723db9.  It costs ~14 s.  Twenty
# three `--embed` children, one of which attaches a UI over a RAW
# socket; t91 expects TWO aborts and names both.
# B22-3 added `marksweep` (marktree.rs + marktree/{splice,iter,
# rebalance,check,inspect,node,intersect,key,meta}.rs -- `marktree_put`
# / `marktree_del_itr` / `marktree_splice` / `marktree_move`, the
# `split_node`/`merge_node`/`pivot_left`/`pivot_right` rebalance, the
# plain, meta-FILTERED and OVERLAP iterator walks, the intersection sets
# and `mt_inspect` itself), baselined at 44c6e8d630.  It costs ~4 s.  It
# is the only oracle in this tree that can see tree SHAPE through a
# running editor -- `marktree_check` has zero production callers -- and
# its observable, `nvim__buf_debug_extmarks`, was called by nothing in
# `test/`, so the sweep gates the API too.  Twenty `--headless` children
# in m91, all reaped before their row is printed; `aborted=0` is the
# baseline, so ANY abort is a regression.
# P17-2 added `resweep` (regexp.rs + regexp/{api,bt,bt/*,nfa,nfa/*,parse,
# chars,mbyte,equi_class,context,submatch,substitute}.rs -- `vim_regcomp`
# over every magic level, `vim_regexec_nl`/`_multi`/`_prog`, the
# BACKTRACKING and the NFA engine SIDE BY SIDE under `'regexpengine'`
# 0/1/2, the zero-width and position atoms, the submatch bookkeeping, the
# `substitute()` replacement alphabet and every `E` code the parsers
# raise) and `optsweep` (option.rs + option/*, options/* and optionstr/*
# -- the option TABLE itself, one row per option for all 374, `do_set`'s
# `= += -= ^= & &vim < ! inv no` alphabet, `:set`/`:setlocal`/
# `:setglobal`/`all&`/`:verbose set`/`:options`, the global-local
# convention and what a fresh window or buffer inherits), both baselined
# at 5c829b911e.  They cost ~8 s and ~2 s.  Before them nothing anywhere
# set `'regexpengine'` -- one of the two engines was covered by accident
# and the other not at all -- and nothing anywhere read the option table
# back after setting it.  Sixteen `--headless` children each, all under
# `timeout -k 2 30` (r91 provokes catastrophic backtracking on purpose);
# `aborted=0` is the baseline for both, so ANY abort is a regression.
# P20-4 wired the last three finished-but-unwired byte goldens, taking
# the battery from 24 to 27:
#   `spellverify` (spellfile.rs + spellsuggest.rs -- 30 :mkspell corpus
# cases and the 35 .spl/.sug files they produce, scrubbed and hashed),
# baselined at 47e44ab7c6, phase 19's base.  It costs ~34 s and is now
# the slowest single oracle in the battery, ahead of fssweep's ~30 s.
#   `persverify` (memline.rs, memfile.rs, shada.rs, fileio.rs,
# bufwrite.rs -- 13 swap-file byte goldens plus the ShaDa write/read/
# merge/damage matrix), re-baselined at 96d70c6f79.  It costs ~1 s.  Its
# shada half was NOT a stored-baseline differential before: the header
# embeds `nvim.rs <version>` and that string's length moves every entry
# offset, so the p19 close could only run it as a base-vs-HEAD
# self-differential.  shadamask.py now rewrites the version to
# a fixed-width placeholder before anything is decoded, and the sweep is
# a plain stored-baseline diff like every other entry here.
#   `undoverify` (the undofile golden -- 20 cases, 20 `.un~`, decoded
# field by field), baselined at 96d70c6f79 by p20-3.  It costs ~0.3 s.
# P20-12 closed the last two coverage gaps the survey named, taking
# the battery from 27 to 29:
#   `foldverify` (fold/{mod,level,open_close,adjust,marker,text,
# builtins,session}.rs -- the fold TREE, which nothing watched: the
# display is covered from six directions and `:mkview`'s fold block by
# sessgold, but not the tree under edits.  419 cases: zf/zF/zd/zD/zE
# and the whole z{o,O,c,C,a,A,R,M,r,m,v,x,X,n,N,i} alphabet, zj/zk/[z/
# ]z, all six foldmethods, nesting/splitting/merging, 'foldminlines'
# 'foldnestmax' 'foldignore' 'foldlevelstart', :move/:d/:put/:g//d
# across fold boundaries, foldtextresult(), and a LEVEL SCAN that walks
# 'foldlevel' 0..N per case so the artifact carries the tree's SHAPE
# rather than just which lines are closed), baselined at 070c9cfc00.
# It costs ~1 s.
#   `jmarkverify` (mark/{mod,lookup,adjust,jumplist,show,builtins,
# shada}.rs -- named marks, the jumplist and the changelist, which had
# neither a differential nor an anchor: `marksweep` is the *marktree*
# oracle and perssweep watches only the PERSISTED form.  317 cases:
# every a-z A-Z 0-9 slot and the tick family, :marks/:jumps/:changes
# captured verbatim BESIDE getmarklist()/getjumplist()/getchangelist()
# so a divergence between the two surfaces over one slot is one line,
# mark adjustment under dd/o/:m/:t/:d/:g//d/J/<</:s//\r/, the
# JUMPLISTSIZE clamp driven past 100 on both lists, deleted-buffer
# marks, :lockmarks/:keepjumps/:keepmarks, and a shada round trip),
# baselined at 070c9cfc00.  It costs ~1 s.  Buffer HANDLES appear in
# neither artifact: the driver prints filenames.
# Usage: battery.sh <label>
set -u
LABEL="${1:?label}"
# Everything is resolved from this script's own directory, so the harness
# travels with the checkout: $T is test/battery, $REPO its grandparent.
T=$(cd "$(dirname "$0")" && pwd)
REPO=${REPO:-$(cd "$T/../.." && pwd)}
LOGDIR=${BATTERY_LOGDIR:-$REPO/target/battery-logs}
mkdir -p "$LOGDIR"
cd "$REPO" || exit 1

fail=0
run() { # name script
  local n=$1; shift
  "$@" "$LABEL" > "$LOGDIR/$LABEL.$n.log" 2>&1
  local rc=$?
  local res
  # scrsweep prints "  txt IDENTICAL" with no colon; every other verify.sh
  # prints "txt: IDENTICAL".  Accept both -- with the colon required this
  # reported "(no verdict lines)" for scrsweep on every run (B16-18).
  res=$(grep -aoE '[a-z]+ *:? *(IDENTICAL|DIFFERS)' "$LOGDIR/$LABEL.$n.log" | tr '\n' ' ')
  [ -z "$res" ] && res="(no verdict lines; rc=$rc)"
  printf '%-10s rc=%s %s\n' "$n" "$rc" "$res"
  [ "$rc" -ne 0 ] && fail=1
  return 0
}

run keysweep  "$T/keyverify.sh"
run scrsweep  "$T/scrverify.sh"
run evalsweep "$T/evalverify.sh"
run ausweep   "$T/auverify.sh"
run varsweep  "$T/varsverify.sh"
run opsweep   "$T/opverify.sh"
run fmtsweep  "$T/fmtverify.sh"
run diffsweep "$T/diffverify.sh"
run utfsweep  "$T/utfverify.sh"
run cmdsweep  "$T/cmdverify.sh"
run rtsweep   "$T/rtverify.sh"
run sessgold  "$T/sessverify.sh"
run exprobe   "$T/exverify.sh"
run fssweep   "$T/fsverify.sh"
run stlsweep  "$T/stlverify.sh"
run mousesweep "$T/mouseverify.sh"
run menusweep "$T/menuverify.sh"
run winsweep  "$T/winverify.sh"
run bufsweep  "$T/bufverify.sh"
run vtsweep   "$T/vtverify.sh"
run termsweep "$T/termverify.sh"
run marksweep "$T/markverify.sh"
run resweep   "$T/reverify.sh"
run optsweep  "$T/optverify.sh"
# The three P20-4 additions.  Appended rather than slotted in beside
# their subsystems so the twenty-four rows above keep their order and a
# run of this battery can still be diffed against p19-close's logs.
run spellsweep "$T/spellverify.sh"
run perssweep  "$T/persverify.sh"
run undosweep  "$T/undoverify.sh"
# The two P20-12 additions, appended for the same reason: the
# twenty-seven rows above keep their order and a run can still be
# diffed against p20-4's logs.
run foldsweep  "$T/foldverify.sh"
run jmarksweep "$T/jmarkverify.sh"
# The P20-13 addition, appended for the same reason once more: the
# twenty-nine rows above keep their order.  `navsweep`
# (navsweep.sh -- file_search.rs, path.rs, quickfix.rs,
# search.rs, tag.rs, baselined at d086fb238f back in B11) has been
# GREEN BUT UNWIRED ever since; p20-12 surfaced it while retiring
# navmutate6 and the orchestrator ruled it in here.  It is the only
# oracle over `search/`, which still has zero mutation anchors, so
# without this row that subsystem is watched by nothing at all.  Cost:
# ~13 s.
run navsweep   "$T/navverify.sh"
# The P21-3 addition, appended for the same reason: the thirty rows
# above keep their order.  `decodesweep` is `1785820781-decodediff.sh`
# turned into a stored-baseline row -- json, msgpack, vim.json,
# vim.mpack and the msgpack-RPC transport, baselined at 068df58956.
# It had been a PAIRED (two-binary) differential since B15 and was left
# out of the battery for that reason; all five corpora were measured
# byte-stable across three runs and across a change of cwd, so the
# pairing was not fundamental.  Its rpc corpus is the only
# stored-baseline oracle in the tree that round-trips the whole Object
# type matrix, which is what gates `api/private/helpers/value.rs`.
# Cost: ~40 s, almost all of it the rpc corpus's forty children.
run decodesweep "$T/decodeverify.sh"
# The P22-4 addition, appended for the same reason once more: the
# thirty-one rows above keep their order.  `inssweep`
# (inssweep.sh -- crates/nvim/src/insexpand/*.rs and the
# `Insstart`/`compl_*` globals: every `'complete'` source letter, every
# `'completeopt'` flag, the whole CTRL-X alphabet, `compl_leader`
# editing while the menu is up, `ins_compl_next`'s wraparound, the three
# completion callbacks, `complete()`/`complete_add()`/`complete_check()`/
# `nvim__complete_set`, `v:completed_item` and the
# CompleteChanged/DonePre/Done ORDER), baselined at cb90ea330c.  It
# costs ~3 s.  It closes the p22-1 survey's first oracle GAP: insexpand
# is the second-largest `cell_ptr` family in the tree and had no
# differential at all -- `cxprobe` is *cmdline* expansion and
# `1785449630-insweep.sh` is the TUI input layer.  It is the only oracle
# here that reports ONE LINE PER KEYSTROKE rather than per case, because
# insert-completion is a state that exists only BETWEEN two keys: every
# case runs in an `--embed` child and is observed with a DEFERRED
# `nvim_exec_lua` while the menu is still up.  Thirteen children (one
# standing, twelve in i91), all reaped before their row is printed;
# `aborted=0` is the baseline, so ANY abort is a regression.
run inssweep   "$T/insverify.sh"

echo "=== startup probe (paired)"
PBASE=${PROBE_BASELINE:-$T/probebase/base.txt}
python3 "$T/startprobe.py" "$REPO/target/debug/nvim" \
  "$LOGDIR/$LABEL.p.txt" > "$LOGDIR/$LABEL.probe.log" 2>&1
prc=$?
if diff -q "$PBASE" "$LOGDIR/$LABEL.p.txt" >/dev/null; then
  echo "probe: IDENTICAL (rc=$prc)"
else
  echo "probe: DIFFERS (rc=$prc)"
  diff -a "$PBASE" "$LOGDIR/$LABEL.p.txt" | head -40
  fail=1
fi
echo "BATTERY_EXIT=$fail"
# ...and exit with it.  Up to P20-4 this script printed the verdict and
# then returned 0, so `diffbattery.sh x || echo broken` never fired and
# every caller had to grep.  The BATTERY_EXIT line stays for the callers
# that already do.
exit "$fail"
