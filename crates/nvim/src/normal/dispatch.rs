//! Turning a keystroke into a command: the table lookup, the count and
//! register that may precede it, the extra character some commands ask for,
//! and the bookkeeping that follows the handler.
//!
//! [`normal_execute`] is the whole of it, once per key. `find_command`
//! resolves a character to a row of `nv_cmds`, `normal_get_command_count`
//! stacks the digits before it, and `normal_get_additional_char` reads the
//! second character a row asks for.
//!
//! Per-key path: `GlobalCell` through `get`/`set`/`ptr`, never `with`, and no
//! iterator adaptors or closures-as-helpers -- at opt-level 0 nothing inlines
//! but `#[inline(always)]`.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::charset::vim_isprintc;
use crate::digraph::get_digraph;
use crate::drawscreen::showmode;
use crate::edit::edit;
use crate::eval::vars::{set_reg_var, set_vcount};
use crate::ex_docmd::do_sleep;
use crate::fold::fold_open_cursor;
use crate::getchar::{
    beep_flush, gotchars_ignore, ins_char_typebuf, plain_vgetc, readbuf1_empty, stuff_empty,
    typeahead, ungetchars, vpeekc, vungetc,
};
use crate::guard::{Allow, Keys};
use crate::keycodes::{
    Ctrl_BSL, Ctrl_G, Ctrl_K, Ctrl_N, Ctrl_W, K_DEL, K_DOWN, K_END, K_HOME, K_KENTER, K_LEFT,
    K_RIGHT, K_S_END, K_S_HOME, K_S_LEFT, K_S_RIGHT, K_UP, K_ZERO, KE_C_LEFT, KE_C_RIGHT, KE_EVENT,
    KE_IGNORE, KE_KDEL, KE_MOUSEMOVE, simplify_mod_mask,
};
use crate::main::{
    KeyStuffed, KeyTyped, State, VIsual_select_reg, clear_cmdline, curwin, did_cursorhold,
    fdo_flags, finish_op, km_startsel, langmap_mapchar, mod_mask, mode_displayed, motion_force,
    msg_col, msg_didout, msg_nowait, no_u_sync, no_zero_mapping, opcount, p_langmap, p_lrm, p_tm,
    p_ttm, restart_VIsual_select, restart_edit, vgetc_busy, vgetc_char, vgetc_mod_mask,
};
use crate::mapping::langmap_adjust_mb;
use crate::mark::checkpcmark;
use crate::mbyte::{
    mb_check_adjust_col, utf_char2bytes, utf_char2len, utf_iscomposing, utf8len_tab,
};
use crate::memory::xfree;
use crate::normal::{
    B_IMODE_LMAP, CA_COMMAND_BUSY, CAR, CmdArg, ESC, GRAPHEME_STATE_INIT, MOD_MASK_SHIFT, NL,
    NV_CMDS, NV_CMDS_SIZE, NV_KEEPREG, NV_LANG, NV_NCW, NV_RL, NV_SS, NV_SSS, NormalState,
    NormalStateRef, add_to_showcmd, check_text_or_curbuf_locked, clear_showcmd, del_from_showcmd,
    do_check_scrollbind, normal_handle_special_visual_command, normal_need_additional_char,
    normal_need_redraw_mode_message, normal_redraw_mode_message, nv_cmds, set_vcount_ca,
    set_visual_select, start_selection, visual_active, visual_select,
};
use crate::ops::{Op, do_pending_operator, get_op_type};
use crate::register::get_default_register_name;
use crate::state::{
    MODE_LANGMAP, MODE_LREPLACE, MODE_NORMAL, MODE_NORMAL_BUSY, MODE_REPLACE, MODE_SELECT,
    get_real_state, may_trigger_modechanged,
};
use crate::types::{
    CpoFlag, GraphemeState, NUL, OP_COLON, OP_NOP, OptInt, VimState, cmdarg_T, int16_t, int64_t,
    oparg_T,
};
use crate::ui::{ui_cursor_shape, ui_cursor_shape_no_check_conceal, ui_flush};
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int, c_uint, c_void};

use crate::getchar::{
    append_to_redobuff, append_to_redobuff_char, append_to_redobuff_number, reset_redobuff,
};
use crate::r#move::{do_check_cursorbind, validate_cursor};
use crate::option::cpo_has;

/// `find_command` returns a row index that has to fit an `int16_t`.
const _: () = assert!(NV_CMDS_SIZE <= i16::MAX as usize);

/// Order two `nv_cmds` rows by the character they answer to.
///
/// `nv_cmds` in ascending `|cmd_char|` order, and how far a direct index
/// works.
///
/// Upstream builds this with `qsort` at startup, and then measures the
/// leading run where the `i`th smallest character *is* `i` — the control
/// characters are dense and start at NUL — so that `find_command` can index
/// straight in below that and only binary-search above it. The table is a
/// `const`, so both answers are: an insertion sort over 188 rows is nothing
/// to a `const fn`, and there is no startup pass and no `qsort` comparator
/// left to keep an `extern "C"` signature for.
///
/// A special key is stored as a negative character and sorts by magnitude,
/// which is what lets `find_command` search for `-cmdchar`.
const fn sorted_nv_cmds() -> ([int16_t; NV_CMDS_SIZE], c_int) {
    let mut idx = [0 as int16_t; NV_CMDS_SIZE];
    let mut i = 0;
    while i < NV_CMDS_SIZE {
        idx[i] = i as int16_t;
        i += 1;
    }
    // Insertion sort: `qsort` is not stable, so this leans on no two rows
    // sharing a character, which the assertion below holds them to.
    let mut i = 1;
    while i < NV_CMDS_SIZE {
        let mut j = i;
        while j > 0
            && NV_CMDS[idx[j - 1] as usize].cmd_char.abs() > NV_CMDS[idx[j] as usize].cmd_char.abs()
        {
            let swap = idx[j - 1];
            idx[j - 1] = idx[j];
            idx[j] = swap;
            j -= 1;
        }
        i += 1;
    }
    let mut linear = 0;
    while linear < NV_CMDS_SIZE as c_int
        && linear == NV_CMDS[idx[linear as usize] as usize].cmd_char
    {
        linear += 1;
    }
    (idx, linear - 1)
}

/// [`sorted_nv_cmds`]'s two answers.
const NV_SORTED: ([int16_t; NV_CMDS_SIZE], c_int) = sorted_nv_cmds();
const NV_CMD_IDX: [int16_t; NV_CMDS_SIZE] = NV_SORTED.0;
const NV_MAX_LINEAR: c_int = NV_SORTED.1;

/// No two rows share a character: the sort above is stable and `find_command`
/// answers with the first match, so a duplicate would make which row wins
/// depend on the table's source order.
const _: () = {
    let mut i = 1;
    while i < NV_CMDS_SIZE {
        assert!(
            NV_CMDS[NV_CMD_IDX[i - 1] as usize].cmd_char.abs()
                < NV_CMDS[NV_CMD_IDX[i] as usize].cmd_char.abs(),
            "two nv_cmds rows share a command character"
        );
        i += 1;
    }
};

/// The row of `nv_cmds` that answers to `cmdchar`, or -1.
pub(crate) fn find_command(cmdchar: c_int) -> c_int {
    if cmdchar >= 0x100 {
        return -1;
    }
    // A special key is stored negative and searched for by magnitude.
    let cmdchar = cmdchar.abs();
    const _: () = assert!(NV_MAX_LINEAR < NV_CMDS_SIZE as c_int);

    {
        if cmdchar <= NV_MAX_LINEAR {
            return NV_CMD_IDX[cmdchar as usize] as c_int;
        }
        let mut bot = NV_MAX_LINEAR + 1;
        let mut top = NV_CMDS_SIZE as c_int - 1;
        while bot <= top {
            let i = (top + bot) / 2;
            let c = NV_CMDS[NV_CMD_IDX[i as usize] as usize].cmd_char.abs();
            if cmdchar == c {
                return NV_CMD_IDX[i as usize] as c_int;
            } else if cmdchar > c {
                bot = i + 1;
            } else {
                top = i - 1;
            }
        }
        -1
    }
}

/// Whether a key just read should be put through 'langmap'.
///
/// The three call sites differ only in the extra condition they add, and all
/// three of those conditions are pure reads, so passing one in as a `bool`
/// evaluates it where the C's `&&` chain would have short-circuited past it
/// without changing anything.
#[inline(always)]
fn langmap_wanted(condition: bool) -> bool {
    // SAFETY: 'langmap' is a NUL-terminated option string.
    let have_langmap = unsafe { *p_langmap.get() } as c_int != 0;
    let from_a_map = if vgetc_busy.get() != 0 {
        typeahead().maplen() == 0
    } else {
        KeyTyped.get()
    };
    have_langmap && condition && (p_lrm.get() != 0 || from_a_map) && KeyStuffed.get() == 0
}

/// Translate one key through 'langmap', if this key and this moment call for
/// it.
#[inline(always)]
pub(crate) fn langmap_adjust(c: &mut c_int, condition: bool) {
    if *c >= 0 && langmap_wanted(condition) {
        *c = if *c < 256 {
            langmap_mapchar.with(|map| map[*c as usize] as c_int)
        } else {
            langmap_adjust_mb(*c)
        };
    }
}

/// Where `normal_get_additional_char` puts the character it reads.
///
/// The transpiled form is a `*mut c_int` aimed at one of two fields of `ca`,
/// and later compares that pointer against `&ca.extra_char` to decide what to
/// do next. Naming the two slots makes that test a `match` rather than a
/// pointer identity.
#[derive(PartialEq, Clone, Copy)]
enum Slot {
    /// Nothing more to read.
    None,
    /// `ca.nchar`: the ordinary "second character" of a command.
    NChar,
    /// `ca.extra_char`: the third, for `gr`, `g'`, `` g` `` and `g CTRL-\`.
    Extra,
}

/// Which slot, and in which mode, this command's extra character is read.
///
/// Answers `(slot, literal, replace)`. `literal` suppresses digraphs and
/// 'langmap'; `replace` puts the editor in Replace mode while waiting, so the
/// cursor shape says what is about to happen.
unsafe fn additional_char_slot(s: *mut NormalState) -> (Slot, bool, bool) {
    // SAFETY (throughout): `s` is the caller's live state.
    let mut ns = unsafe { NormalStateRef::new(s) };
    if ns.ca.cmdchar != 'g' as c_int {
        return (Slot::NChar, false, ns.ca.cmdchar == 'r' as c_int);
    }
    // `g` reads its own second character first, and only some of them
    // take a third.
    ns.ca.nchar = unsafe { plain_vgetc() };
    langmap_adjust(&mut ns.ca.nchar, true);
    ns.need_flushbuf |= add_to_showcmd(ns.ca.nchar);
    match ns.ca.nchar {
        c if c == 'r' as c_int => (Slot::Extra, false, true),
        c if c == '\'' as c_int || c == '`' as c_int || c == Ctrl_BSL => (Slot::Extra, true, false),
        _ => (Slot::None, false, false),
    }
}

/// Read a `CTRL-\` follow-up, waiting up to 'ttimeoutlen' for it.
///
/// `CTRL-\ CTRL-N` and `CTRL-\ CTRL-G` are commands of their own; anything
/// else is put back for the next command to read.
unsafe fn resolve_ctrl_backslash(s: *mut NormalState) {
    // SAFETY (throughout): `s` is the caller's live state.
    let mut ns = unsafe { NormalStateRef::new(s) };
    let mut towait = if p_ttm.get() >= 0 {
        p_ttm.get() as c_int
    } else {
        p_tm.get() as c_int
    };
    loop {
        ns.c = unsafe { vpeekc() };
        if !(ns.c <= 0 && towait > 0) {
            break;
        }
        unsafe { do_sleep(towait.min(50) as int64_t, false) };
        towait -= 50;
    }
    if ns.c > 0 {
        ns.c = unsafe { plain_vgetc() };
        if ns.c != Ctrl_N && ns.c != Ctrl_G {
            vungetc(ns.c);
        } else {
            ns.ca.cmdchar = Ctrl_BSL;
            ns.ca.nchar = ns.c;
            ns.idx = find_command(ns.ca.cmdchar);
            debug_assert!(ns.idx >= 0);
        }
    }
}

/// Collect the combining characters that belong with the character just read.
///
/// Only for a command flagged `NV_LANG` -- `f`, `t`, `r` and friends, where
/// the argument is a real character rather than a command key.
unsafe fn read_composing_tail(s: *mut NormalState) {
    // SAFETY (throughout): `s` is the caller's live state; every write to
    // `nchar_composing` is bounded by its own length below.
    let mut ns = unsafe { NormalStateRef::new(s) };
    let mapped = Allow::mapping();
    let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
    let mut prev_code = ns.ca.nchar;
    loop {
        ns.c = unsafe { vpeekc() };
        if !(ns.c > 0 && (ns.c >= 0x100 || utf8len_tab[unsafe { vpeekc() } as usize] as c_int > 1))
        {
            break;
        }
        ns.c = unsafe { plain_vgetc() };
        if !unsafe { utf_iscomposing(prev_code, ns.c, &raw mut state) } {
            vungetc(ns.c);
            break;
        }
        // The base character is only encoded once a tail turns up.
        if ns.ca.nchar_len == 0 {
            ns.ca.nchar_len =
                unsafe { utf_char2bytes(ns.ca.nchar, ns.ca.nchar_composing.as_mut_ptr()) };
        }
        if ns.ca.nchar_len + utf_char2len(ns.c) < size_of::<[c_char; 32]>() as c_int {
            let at = ns.ca.nchar_len as isize;
            let tail = unsafe { ns.ca.nchar_composing.as_mut_ptr().offset(at) };
            ns.ca.nchar_len += unsafe { utf_char2bytes(ns.c, tail) };
        }
        prev_code = ns.c;
    }
    let at = ns.ca.nchar_len as usize;
    ns.ca.nchar_composing[at] = NUL as c_char;
    drop(mapped);
    // The keys are recorded for a redo, not fed through undo syncing.
    no_u_sync.set(no_u_sync.get() + 1);
    unsafe { gotchars_ignore() };
    no_u_sync.set(no_u_sync.get() - 1);
}

/// Read the second (and sometimes third) character of a command.
pub(crate) unsafe fn normal_get_additional_char(s: *mut NormalState) {
    // SAFETY (throughout): `s` is the caller's live state and `s.idx` is a valid row.
    // Nothing read here is a mapping or a command; it is an argument.
    let mut ns = unsafe { NormalStateRef::new(s) };
    let _raw_key = Keys::unmapped_with_codes();
    did_cursorhold.set(true);

    let (slot, lit, repl) = unsafe { additional_char_slot(ns.raw()) };
    let lang = repl || nv_cmds[ns.idx as usize].cmd_flags as c_int & NV_LANG != 0;

    if slot != Slot::None {
        // SAFETY: `s` is the caller's live state.
        let cp: *mut c_int = match slot {
            Slot::NChar => &raw mut ns.ca.nchar,
            Slot::Extra => &raw mut ns.ca.extra_char,
            Slot::None => unreachable!(),
        };
        if repl {
            State.set(MODE_REPLACE);
            unsafe { ui_cursor_shape_no_check_conceal() };
        }
        // A language-mapped argument is read *with* mappings on, which is
        // the whole point of 'iminsert' being lmap.
        let langmap_active = lang && cur_buf().b_p_iminsert == B_IMODE_LMAP as OptInt;
        let mapped = langmap_active.then(Allow::mapping_with_codes);
        if langmap_active {
            State.set(if repl { MODE_LREPLACE } else { MODE_LANGMAP });
        }
        unsafe { *cp = plain_vgetc() };
        drop(mapped);
        State.set(MODE_NORMAL_BUSY);
        ns.need_flushbuf |= add_to_showcmd(unsafe { *cp });

        if !lit {
            // CTRL-K starts a digraph, unless 'cpoptions' says otherwise.
            if unsafe { *cp } == Ctrl_K
                && (nv_cmds[ns.idx as usize].cmd_flags as c_int & NV_LANG != 0
                    || slot == Slot::Extra)
                && !cpo_has(CpoFlag::DIGRAPH)
            {
                ns.c = get_digraph(false);
                if ns.c > 0 {
                    unsafe { *cp = ns.c };
                    // Take the CTRL-K and its two characters back out of
                    // the echoed command before showing the result.
                    del_from_showcmd(3);
                    ns.need_flushbuf |= add_to_showcmd(unsafe { *cp });
                }
            }
            langmap_adjust(&mut unsafe { *cp }, !lang);
        }

        if slot == Slot::Extra
            && ns.ca.nchar == Ctrl_BSL
            && (ns.ca.extra_char == Ctrl_N || ns.ca.extra_char == Ctrl_G)
        {
            // `g CTRL-\ CTRL-N` is really `CTRL-\ CTRL-N`.
            ns.ca.cmdchar = Ctrl_BSL;
            ns.ca.nchar = ns.ca.extra_char;
            ns.idx = find_command(ns.ca.cmdchar);
        } else if (ns.ca.nchar == 'n' as c_int || ns.ca.nchar == 'N' as c_int)
            && ns.ca.cmdchar == 'g' as c_int
        {
            // `gn`/`gN` take the operator from the character after them.
            unsafe { *ns.ca.oap }.op_type = get_op_type(unsafe { *cp }, NUL);
        } else if unsafe { *cp } == Ctrl_BSL {
            unsafe { resolve_ctrl_backslash(ns.raw()) };
        }

        if lang {
            unsafe { read_composing_tail(ns.raw()) };
        }
    }
}

/// Mirror a horizontal command for a right-to-left window.
pub(crate) unsafe fn normal_invert_horizontal(s: *mut NormalState) {
    // SAFETY (throughout): `s` is the caller's live state.
    let mut ns = unsafe { NormalStateRef::new(s) };
    const K_C_LEFT: c_int = -(253 + ((KE_C_LEFT as c_int) << 8));
    const K_C_RIGHT: c_int = -(253 + ((KE_C_RIGHT as c_int) << 8));
    ns.ca.cmdchar = match ns.ca.cmdchar {
        c if c == 'l' as c_int => 'h' as c_int,
        K_RIGHT => K_LEFT,
        K_S_RIGHT => K_S_LEFT,
        K_C_RIGHT => K_C_LEFT,
        c if c == 'h' as c_int => 'l' as c_int,
        K_LEFT => K_RIGHT,
        K_S_LEFT => K_S_RIGHT,
        K_C_LEFT => K_C_RIGHT,
        c if c == '>' as c_int => '<' as c_int,
        c if c == '<' as c_int => '>' as c_int,
        other => other,
    };
    ns.idx = find_command(ns.ca.cmdchar);
}

/// Read the digits, and possibly the CTRL-W, in front of a command.
///
/// Answers whether a CTRL-W was consumed, in which case the caller loops:
/// `CTRL-W` takes a count of its own after it.
pub(crate) unsafe fn normal_get_command_count(s: *mut NormalState) -> bool {
    // SAFETY: `s` is the caller's live normal-mode state.
    let mut ns = unsafe { NormalStateRef::new(s) };
    // Select mode swallows printable keys as replacement text, digits too.
    if visual_active() && visual_select() {
        return false;
    }
    // SAFETY: `s` is the caller's live state.
    const K_KDEL: c_int = -(253 + ((KE_KDEL as c_int) << 8));
    while (ns.c >= '1' as c_int && ns.c <= '9' as c_int)
        || (ns.ca.count0 != 0 && (ns.c == K_DEL || ns.c == K_KDEL || ns.c == '0' as c_int))
    {
        if ns.c == K_DEL || ns.c == K_KDEL {
            ns.ca.count0 /= 10;
            // Four columns: <Del> is echoed as its key name.
            del_from_showcmd(4);
        } else if ns.ca.count0 > 99999999 {
            // Saturate rather than overflow; nine digits is the most a
            // count is ever allowed to be.
            ns.ca.count0 = 999999999;
        } else {
            ns.ca.count0 = ns.ca.count0 * 10 + (ns.c - '0' as c_int);
        }
        if ns.toplevel && readbuf1_empty() {
            unsafe { set_vcount_ca(&raw mut ns.ca, &mut ns.set_prevcount) };
        }
        let raw_key = ns.ctrl_w.then(Keys::unmapped_with_codes);
        // A '0' here is a count digit, not the "go to column 0" command,
        // so it must not be mapped.
        no_zero_mapping.set(no_zero_mapping.get() + 1);
        ns.c = unsafe { plain_vgetc() };
        langmap_adjust(&mut ns.c, true);
        no_zero_mapping.set(no_zero_mapping.get() - 1);
        drop(raw_key);
        ns.need_flushbuf |= add_to_showcmd(ns.c);
    }

    // CTRL-W takes the count read so far as its own, then a second count
    // for the window command after it.
    if ns.c == Ctrl_W && !ns.ctrl_w && ns.oa.op_type == OP_NOP {
        ns.ctrl_w = true;
        ns.ca.opcount = ns.ca.count0;
        ns.ca.count0 = 0;
        let raw_key = Keys::unmapped_with_codes();
        ns.c = unsafe { plain_vgetc() };
        langmap_adjust(&mut ns.c, true);
        drop(raw_key);
        ns.need_flushbuf |= add_to_showcmd(ns.c);
        return true;
    }
    false
}

/// Everything that happens after the handler has run.
pub(crate) unsafe fn normal_finish_command(s: *mut NormalState) {
    // SAFETY: `s` is the caller's live normal-mode state.
    let mut ns = unsafe { NormalStateRef::new(s) };
    const K_IGNORE: c_int = -(253 + ((KE_IGNORE as c_int) << 8));
    const K_MOUSEMOVE: c_int = -(253 + ((KE_MOUSEMOVE as c_int) << 8));
    const K_EVENT: c_int = -(253 + ((KE_EVENT as c_int) << 8));

    // SAFETY: `s` is the caller's live state.
    let mut did_visual_op = false;
    if !ns.command_finished {
        // A command that is not itself an operator, and does not claim
        // NV_KEEPREG, releases the register it was given.
        if !finish_op.get()
            && ns.oa.op_type == 0
            && (ns.idx < 0 || nv_cmds[ns.idx as usize].cmd_flags as c_int & NV_KEEPREG == 0)
        {
            unsafe { clearop(&raw mut ns.oa) };
            unsafe { set_reg_var(get_default_register_name()) };
        }
        if ns.old_mapped_len > 0 {
            ns.old_mapped_len = typeahead().maplen();
        }
        if ns.ca.cmdchar != K_IGNORE && ns.ca.cmdchar != K_MOUSEMOVE {
            did_visual_op = visual_active() && ns.oa.op_type != OP_NOP && ns.oa.op_type != OP_COLON;
            unsafe { do_pending_operator(&raw mut ns.ca, ns.old_col, false) };
        }
        if unsafe { normal_need_redraw_mode_message(ns.raw()) } {
            normal_redraw_mode_message();
        }
    }
    msg_nowait.set(false);

    if finish_op.get() || did_visual_op {
        unsafe { set_reg_var(get_default_register_name()) };
    }
    let prev_finish_op = finish_op.get();
    if ns.oa.op_type == OP_NOP {
        finish_op.set(false);
        unsafe { may_trigger_modechanged() };
    }
    // The cursor shape says whether an operator is pending, and `r`/`gr`
    // change it while they wait.
    if prev_finish_op
        || ns.ca.cmdchar == 'r' as c_int
        || (ns.ca.cmdchar == 'g' as c_int && ns.ca.nchar == 'r' as c_int)
    {
        unsafe { ui_cursor_shape() };
    }
    if ns.oa.op_type == OP_NOP && ns.oa.regname == 0 && ns.ca.cmdchar != K_EVENT {
        clear_showcmd();
    }
    unsafe { checkpcmark() };
    unsafe { xfree(ns.ca.searchbuf.cast::<c_void>()) };
    unsafe { mb_check_adjust_col(curwin.get().cast::<c_void>()) };

    if cur_win().w_onebuf_opt.wo_scb != 0 && ns.toplevel {
        unsafe { validate_cursor(curwin.get()) };
        unsafe { do_check_scrollbind(true) };
    }
    if cur_win().w_onebuf_opt.wo_crb != 0 && ns.toplevel {
        unsafe { validate_cursor(curwin.get()) };
        unsafe { do_check_cursorbind() };
    }

    // A command may have asked for insert mode or for Select mode to be
    // resumed; neither happens until the command is completely done.
    let want_insert = restart_edit.get() != 0 && !visual_active() && ns.old_mapped_len == 0;
    if ns.oa.op_type == OP_NOP
        && (want_insert || restart_VIsual_select.get() == 1)
        && ns.ca.retval & CA_COMMAND_BUSY as c_int == 0
        && stuff_empty()
        && ns.oa.regname == 0
    {
        if restart_VIsual_select.get() == 1 {
            set_visual_select(true);
            VIsual_select_reg.set(0);
            unsafe { may_trigger_modechanged() };
            unsafe { showmode() };
            restart_VIsual_select.set(0);
        }
        if want_insert {
            unsafe { edit(restart_edit.get(), false, 1) };
        }
    }
    // 2 means "next command", 1 means "this one"; the countdown is here.
    if restart_VIsual_select.get() == 2 {
        restart_VIsual_select.set(1);
    }
    opcount.set(ns.ca.opcount);
}

/// One normal-mode command, from its first key to the end of its effects.
///
/// Keeps the raw signature: it is installed as a `state_execute_callback` and
/// `state_enter` calls it through that pointer.
pub(crate) unsafe fn normal_execute(state: *mut VimState, key: c_int) -> c_int {
    const K_IGNORE: c_int = -(253 + ((KE_IGNORE as c_int) << 8));
    const K_EVENT: c_int = -(253 + ((KE_EVENT as c_int) << 8));

    // SAFETY: `state` is the `VimState` at the head of the `NormalState` the
    // caller handed to `state_enter`.
    let s = state as *mut NormalState;
    // SAFETY: `state` is the caller's live normal-mode state.
    let mut ns = unsafe { NormalStateRef::new(s) };
    ns.command_finished = false;
    ns.ctrl_w = false;
    ns.old_col = cur_win().w_curswant as c_int;
    ns.c = key;
    langmap_adjust(&mut ns.c, get_real_state() != MODE_SELECT);

    if restart_edit.get() == 0 {
        ns.old_mapped_len = 0;
    } else if ns.old_mapped_len != 0
        || (visual_active() && ns.mapped_len == 0 && typeahead().maplen() > 0)
    {
        ns.old_mapped_len = typeahead().maplen();
    }

    if ns.c == NUL {
        ns.c = K_ZERO;
    }

    // In Select mode a printable key replaces the selection: the key is
    // put back for insert mode to read and the command becomes a change.
    if visual_active()
        && visual_select()
        && (unsafe { vim_isprintc(ns.c) } || ns.c == NL || ns.c == CAR || ns.c == K_KENTER)
    {
        let len = unsafe { ins_char_typebuf(vgetc_char.get(), vgetc_mod_mask.get(), true) };
        if KeyTyped.get() {
            ungetchars(len);
        }
        ns.c = if restart_edit.get() != 0 {
            'd' as c_int
        } else {
            'c' as c_int
        };
        msg_nowait.set(true);
        ns.old_mapped_len = 0;
    }

    ns.need_flushbuf = add_to_showcmd(ns.c);
    while unsafe { normal_get_command_count(ns.raw()) } {}

    if ns.c == K_EVENT {
        // An event is not a command: the count it interrupted is stashed
        // for the real command that follows.
        ns.oa.prev_opcount = ns.ca.opcount;
        ns.oa.prev_count0 = ns.ca.count0;
    } else if ns.ca.opcount != 0 {
        // An operator count and a motion count multiply, saturating.
        if ns.ca.count0 != 0 {
            if ns.ca.opcount >= 999999999 / ns.ca.count0 {
                ns.ca.count0 = 999999999;
            } else {
                ns.ca.count0 *= ns.ca.opcount;
            }
        } else {
            ns.ca.count0 = ns.ca.opcount;
        }
    }
    ns.ca.opcount = ns.ca.count0;
    ns.ca.count1 = if ns.ca.count0 == 0 { 1 } else { ns.ca.count0 };
    if ns.toplevel && readbuf1_empty() {
        let (n0, n1) = (ns.ca.count0 as int64_t, ns.ca.count1 as int64_t);
        unsafe { set_vcount(n0, n1, ns.set_prevcount) };
    }

    if ns.ctrl_w {
        ns.ca.nchar = ns.c;
        ns.ca.cmdchar = Ctrl_W;
    } else {
        ns.ca.cmdchar = ns.c;
    }
    ns.idx = find_command(ns.ca.cmdchar);

    if ns.idx < 0 {
        unsafe { clearopbeep(&raw mut ns.oa) };
        ns.command_finished = true;
    } else if (nv_cmds[ns.idx as usize].cmd_flags as c_int & NV_NCW != 0
        && unsafe { check_text_or_curbuf_locked(&raw mut ns.oa) })
        || (visual_active() && unsafe { normal_handle_special_visual_command(ns.raw()) })
    {
        ns.command_finished = true;
    } else {
        if cur_win().w_onebuf_opt.wo_rl != 0
            && KeyTyped.get()
            && KeyStuffed.get() == 0
            && nv_cmds[ns.idx as usize].cmd_flags as c_int & NV_RL != 0
        {
            unsafe { normal_invert_horizontal(ns.raw()) };
        }
        if unsafe { normal_need_additional_char(ns.raw()) } {
            unsafe { normal_get_additional_char(ns.raw()) };
        }
        if ns.need_flushbuf {
            unsafe { ui_flush() };
        }
        if ns.ca.cmdchar != K_IGNORE && ns.ca.cmdchar != K_EVENT {
            did_cursorhold.set(false);
        }
        State.set(MODE_NORMAL);

        if ns.ca.nchar == ESC || ns.ca.extra_char == ESC {
            unsafe { clearop(&raw mut ns.oa) };
            ns.command_finished = true;
        } else {
            if ns.ca.cmdchar != K_IGNORE {
                msg_didout.set(false);
                msg_col.set(0);
            }
            ns.old_pos = cur_win().w_cursor;

            // 'keymodel' startsel: a shifted special key starts a
            // selection and then acts as its unshifted self.
            if !visual_active() && km_startsel.get() {
                let flags = nv_cmds[ns.idx as usize].cmd_flags as c_int;
                if flags & NV_SS != 0 {
                    start_selection();
                    unsafe { unshift_special(&raw mut ns.ca) };
                    ns.idx = find_command(ns.ca.cmdchar);
                    debug_assert!(ns.idx >= 0);
                } else if flags & NV_SSS != 0 && mod_mask.get() & MOD_MASK_SHIFT != 0 {
                    start_selection();
                    mod_mask.set(mod_mask.get() & !MOD_MASK_SHIFT);
                }
            }

            ns.ca.arg = nv_cmds[ns.idx as usize].cmd_arg as c_int;
            let run = nv_cmds[ns.idx as usize]
                .cmd_func
                .expect("every nv_cmds row has a handler");
            unsafe { run(&raw mut ns.ca) };
        }
    }
    unsafe { normal_finish_command(ns.raw()) };
    1
}

/// Record a command for `.`, taking its second character from `cap`.
pub(crate) unsafe fn prep_redo_cmd(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    prep_redo(ca.op().regname, ca.count0, NUL, ca.cmdchar, NUL, NUL, NUL);
    // A character with a combining tail is replayed as its whole encoding.
    if ca.nchar_len > 0 {
        unsafe { append_to_redobuff(ca.nchar_composing.as_mut_ptr()) };
    } else {
        append_to_redobuff_char(ca.nchar);
    }
}

/// Record a command for `.`: an optional register, a count, and up to five
/// command characters.
pub(crate) fn prep_redo(
    regname: c_int,
    num: c_int,
    cmd1: c_int,
    cmd2: c_int,
    cmd3: c_int,
    cmd4: c_int,
    cmd5: c_int,
) {
    prep_redo_num2(regname, num, cmd1, cmd2, 0, cmd3, cmd4, cmd5);
}

/// As [`prep_redo`], with a second count between the second and third
/// command characters -- which is what `z<n><CR>` and the `[count]` forms of
/// the `Z` commands need.
#[allow(clippy::too_many_arguments)]
pub(crate) fn prep_redo_num2(
    regname: c_int,
    num1: c_int,
    cmd1: c_int,
    cmd2: c_int,
    num2: c_int,
    cmd3: c_int,
    cmd4: c_int,
    cmd5: c_int,
) {
    // SAFETY: all of these append to the redo buffer, which grows itself.
    unsafe { reset_redobuff() };
    if regname != 0 {
        append_to_redobuff_char('"' as c_int);
        append_to_redobuff_char(regname);
    }
    if num1 != 0 {
        append_to_redobuff_number(num1);
    }
    for cmd in [cmd1, cmd2] {
        if cmd != NUL {
            append_to_redobuff_char(cmd);
        }
    }
    if num2 != 0 {
        append_to_redobuff_number(num2);
    }
    for cmd in [cmd3, cmd4, cmd5] {
        if cmd != NUL {
            append_to_redobuff_char(cmd);
        }
    }
}

// A live operator is all the `clear*` entry points need, and [`Op`] already
// carries that promise, so they are safe functions. `clearop` and
// `clearopbeep` keep a raw-pointer shim beside them for the callers outside
// `normal/` that still hold one.

/// Beep and clear the operator if one is pending. Answers whether it was.
pub(crate) fn check_clear_op(op: Op) -> bool {
    if op.op_type == OP_NOP {
        return false;
    }
    clear_op_beep(op);
    true
}

/// As [`check_clear_op`], and also refuse while a Visual selection is up --
/// for commands that make no sense applied to one.
pub(crate) fn check_clear_op_quit(op: Op) -> bool {
    if op.op_type == OP_NOP && !visual_active() {
        return false;
    }
    clear_op_beep(op);
    true
}

/// Forget the pending operator, its register and its forced motion kind.
pub(crate) fn clear_op(mut op: Op) {
    op.op_type = OP_NOP;
    op.regname = 0;
    op.motion_force = NUL;
    op.use_reg_one = false;
    motion_force.set(NUL);
}

/// [`clear_op`] through a raw pointer.
///
/// # Safety
/// `oap` must be a live operator.
pub(crate) unsafe fn clearop(oap: *mut oparg_T) {
    // SAFETY: the caller promises a live operator.
    clear_op(unsafe { Op::new(oap) });
}

/// [`clear_op`], and say so.
///
/// The beep also flushes the typeahead, which is what makes a failed command
/// abandon the rest of a mapping or a `:normal` argument.
pub(crate) fn clear_op_beep(op: Op) {
    clear_op(op);
    // SAFETY: touches only message and typeahead state.
    unsafe { beep_flush() };
}

/// [`clear_op_beep`] through a raw pointer.
///
/// # Safety
/// `oap` must be a live operator.
pub(crate) unsafe fn clearopbeep(oap: *mut oparg_T) {
    // SAFETY: the caller promises a live operator.
    clear_op_beep(unsafe { Op::new(oap) });
}

/// Read one more key for a command that takes several, with mappings and
/// 'langmap' handled the way a command character wants them, and show it in
/// the 'showcmd' area.
pub(crate) unsafe fn read_command_char() -> c_int {
    let raw_key = Keys::unmapped_with_codes();
    // SAFETY: transpiled input machinery, plain value arguments.
    let mut c = unsafe { plain_vgetc() };
    langmap_adjust(&mut c, true);
    drop(raw_key);
    add_to_showcmd(c);
    c
}

/// Open a fold the cursor has landed in, if the 'foldopen' flag for this kind
/// of movement is set, the key was typed rather than mapped, and no operator
/// is waiting for the motion to finish.
pub(crate) unsafe fn may_fold_open(cap: *mut cmdarg_T, fdo_flag: c_uint) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if fdo_flags.get() & fdo_flag != 0 && KeyTyped.get() && ca.op().op_type == OP_NOP {
        unsafe { fold_open_cursor() };
    }
}

/// Turn a shifted special key into its unshifted self.
pub(crate) unsafe fn unshift_special(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    const K_S_UP: c_int = -1277;
    const K_S_DOWN: c_int = -1533;
    // SAFETY: `cap` is the caller's live command argument.
    ca.cmdchar = match ca.cmdchar {
        K_S_RIGHT => K_RIGHT,
        K_S_LEFT => K_LEFT,
        K_S_UP => K_UP,
        K_S_DOWN => K_DOWN,
        K_S_HOME => K_HOME,
        K_S_END => K_END,
        other => other,
    };
    ca.cmdchar = simplify_mod_mask(ca.cmdchar);
}

/// Make room on the command line for whatever is about to be shown there.
pub(crate) fn may_clear_cmdline() {
    if mode_displayed.get() {
        // The mode message is there; let the redraw take it away.
        clear_cmdline.set(true);
    } else {
        clear_showcmd();
    }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
