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

use crate::src::nvim::charset::vim_isprintc;
use crate::src::nvim::digraph::get_digraph;
use crate::src::nvim::drawscreen::showmode;
use crate::src::nvim::edit::edit;
use crate::src::nvim::eval::vars::{set_reg_var, set_vcount};
use crate::src::nvim::ex_docmd::do_sleep;
use crate::src::nvim::fold::foldOpenCursor;
use crate::src::nvim::getchar::{
    beep_flush, gotchars_ignore, ins_char_typebuf, plain_vgetc, readbuf1_empty, stuff_empty,
    typebuf_maplen, ungetchars, vpeekc, vungetc,
};
use crate::src::nvim::keycodes::{
    K_DEL, K_DOWN, K_END, K_HOME, K_KENTER, K_LEFT, K_RIGHT, K_S_END, K_S_HOME, K_S_LEFT,
    K_S_RIGHT, K_UP, K_ZERO, simplify_key,
};
use crate::src::nvim::main::{
    KeyStuffed, KeyTyped, State, VIsual_active, VIsual_select, VIsual_select_reg, allow_keys,
    clear_cmdline, curbuf, curwin, did_cursorhold, fdo_flags, finish_op, km_startsel,
    langmap_mapchar, mod_mask, mode_displayed, motion_force, msg_col, msg_didout, msg_nowait,
    no_mapping, no_u_sync, no_zero_mapping, opcount, p_cpo, p_langmap, p_lrm, p_tm, p_ttm,
    restart_VIsual_select, restart_edit, vgetc_busy, vgetc_char, vgetc_mod_mask,
};
use crate::src::nvim::mapping::langmap_adjust_mb;
use crate::src::nvim::mark::checkpcmark;
use crate::src::nvim::mbyte::{
    mb_check_adjust_col, utf_char2bytes, utf_char2len, utf_iscomposing, utf8len_tab,
};
use crate::src::nvim::memory::xfree;
use crate::src::nvim::normal::{
    B_IMODE_LMAP, CA_COMMAND_BUSY, CAR, CPO_DIGRAPH, Ctrl_BSL, Ctrl_G, Ctrl_K, Ctrl_N, Ctrl_W, ESC,
    GRAPHEME_STATE_INIT, KE_C_LEFT, KE_C_RIGHT, KE_EVENT, KE_IGNORE, KE_KDEL, KE_MOUSEMOVE,
    MOD_MASK_SHIFT, NL, NUL, NV_CMDS_SIZE, NV_KEEPREG, NV_LANG, NV_NCW, NV_RL, NV_SS, NV_SSS,
    NormalState, OP_COLON, OP_NOP, add_to_showcmd, check_text_or_curbuf_locked, clear_showcmd,
    del_from_showcmd, do_check_scrollbind, normal_handle_special_visual_command,
    normal_need_additional_char, normal_need_redraw_mode_message, normal_redraw_mode_message,
    nv_cmd_idx, nv_cmds, nv_max_linear, set_vcount_ca, start_selection,
};
use crate::src::nvim::ops::{do_pending_operator, get_op_type};
use crate::src::nvim::os::libc::qsort;
use crate::src::nvim::register::get_default_register_name;
use crate::src::nvim::state::{
    MODE_LANGMAP, MODE_LREPLACE, MODE_NORMAL, MODE_NORMAL_BUSY, MODE_REPLACE, MODE_SELECT,
    get_real_state, may_trigger_modechanged,
};
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{
    GraphemeState, OptInt, VimState, cmdarg_T, int16_t, int64_t, oparg_T,
};
use crate::src::nvim::ui::{ui_cursor_shape, ui_cursor_shape_no_check_conceal, ui_flush};
use core::ffi::{c_char, c_int, c_uint, c_void};

use crate::src::nvim::getchar::{
    AppendCharToRedobuff, AppendNumberToRedobuff, AppendToRedobuff, ResetRedobuff,
};
use crate::src::nvim::r#move::{do_check_cursorbind, validate_cursor};

/// `find_command` returns a row index that has to fit an `int16_t`.
const _: () = assert!(NV_CMDS_SIZE <= i16::MAX as usize);

/// Order two `nv_cmds` rows by the character they answer to.
///
/// A special key is a negative character; it sorts by its magnitude, which is
/// what lets `find_command` search for `-cmdchar`.
///
/// Kept `extern "C"`: `qsort` calls it through a C function pointer.
unsafe extern "C" fn nv_compare(s1: *const c_void, s2: *const c_void) -> c_int {
    // SAFETY: `qsort` hands back pointers to two elements of `nv_cmd_idx`,
    // each a valid row index.
    unsafe {
        let c1 = (*nv_cmds.ptr())[*(s1 as *const int16_t) as usize]
            .cmd_char
            .abs();
        let c2 = (*nv_cmds.ptr())[*(s2 as *const int16_t) as usize]
            .cmd_char
            .abs();
        c1.cmp(&c2) as c_int
    }
}

/// Build the sorted index into `nv_cmds`, and measure how far a direct index
/// works.
///
/// After the sort, `nv_cmd_idx[i]` is the row whose character is `i`th
/// smallest. For a leading run the `i`th smallest character *is* `i` -- the
/// control characters are dense and start at NUL -- so `find_command` can
/// index straight in below `nv_max_linear` and only binary-search above it.
pub(crate) fn init_normal_cmds() {
    // SAFETY: both arrays are `NV_CMDS_SIZE` long and `nv_compare` only reads
    // row indices out of the one being sorted.
    unsafe {
        for i in 0..NV_CMDS_SIZE {
            (*nv_cmd_idx.ptr())[i] = i as int16_t;
        }
        qsort(
            nv_cmd_idx.ptr().cast::<c_void>(),
            NV_CMDS_SIZE,
            size_of::<int16_t>(),
            Some(nv_compare),
        );
        let mut i = 0;
        while i < NV_CMDS_SIZE as c_int
            && i == (*nv_cmds.ptr())[(*nv_cmd_idx.ptr())[i as usize] as usize].cmd_char
        {
            i += 1;
        }
        nv_max_linear.set(i - 1);
    }
}

/// The row of `nv_cmds` that answers to `cmdchar`, or -1.
pub(crate) fn find_command(cmdchar: c_int) -> c_int {
    if cmdchar >= 0x100 {
        return -1;
    }
    // A special key is stored negative and searched for by magnitude.
    let cmdchar = cmdchar.abs();
    debug_assert!(nv_max_linear.get() < NV_CMDS_SIZE as c_int);

    // SAFETY: every index below is bounded by `NV_CMDS_SIZE`.
    unsafe {
        if cmdchar <= nv_max_linear.get() {
            return (*nv_cmd_idx.ptr())[cmdchar as usize] as c_int;
        }
        let mut bot = nv_max_linear.get() + 1;
        let mut top = NV_CMDS_SIZE as c_int - 1;
        while bot <= top {
            let i = (top + bot) / 2;
            let c = (*nv_cmds.ptr())[(*nv_cmd_idx.ptr())[i as usize] as usize]
                .cmd_char
                .abs();
            if cmdchar == c {
                return (*nv_cmd_idx.ptr())[i as usize] as c_int;
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
    // SAFETY: 'langmap' is a non-empty-or-empty C string option.
    unsafe {
        *p_langmap.get() as c_int != 0
            && condition
            && (p_lrm.get() != 0
                || if vgetc_busy.get() != 0 {
                    typebuf_maplen() == 0
                } else {
                    KeyTyped.get()
                })
            && KeyStuffed.get() == 0
    }
}

/// Translate one key through 'langmap', if this key and this moment call for
/// it.
#[inline(always)]
pub(crate) fn langmap_adjust(c: &mut c_int, condition: bool) {
    if *c >= 0 && langmap_wanted(condition) {
        // SAFETY: `langmap_mapchar` is 256 entries and `*c` is below that.
        *c = unsafe {
            if *c < 256 {
                (*langmap_mapchar.ptr())[*c as usize] as c_int
            } else {
                langmap_adjust_mb(*c)
            }
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
    // SAFETY: `s` is the caller's live state.
    unsafe {
        if (*s).ca.cmdchar != 'g' as c_int {
            return (Slot::NChar, false, (*s).ca.cmdchar == 'r' as c_int);
        }
        // `g` reads its own second character first, and only some of them
        // take a third.
        (*s).ca.nchar = plain_vgetc();
        langmap_adjust(&mut (*s).ca.nchar, true);
        (*s).need_flushbuf |= add_to_showcmd((*s).ca.nchar);
        match (*s).ca.nchar {
            c if c == 'r' as c_int => (Slot::Extra, false, true),
            c if c == '\'' as c_int || c == '`' as c_int || c == Ctrl_BSL => {
                (Slot::Extra, true, false)
            }
            _ => (Slot::None, false, false),
        }
    }
}

/// Read a `CTRL-\` follow-up, waiting up to 'ttimeoutlen' for it.
///
/// `CTRL-\ CTRL-N` and `CTRL-\ CTRL-G` are commands of their own; anything
/// else is put back for the next command to read.
unsafe fn resolve_ctrl_backslash(s: *mut NormalState) {
    // SAFETY: `s` is the caller's live state.
    unsafe {
        let mut towait = if p_ttm.get() >= 0 {
            p_ttm.get() as c_int
        } else {
            p_tm.get() as c_int
        };
        loop {
            (*s).c = vpeekc();
            if !((*s).c <= 0 && towait > 0) {
                break;
            }
            do_sleep(towait.min(50) as int64_t, false);
            towait -= 50;
        }
        if (*s).c > 0 {
            (*s).c = plain_vgetc();
            if (*s).c != Ctrl_N && (*s).c != Ctrl_G {
                vungetc((*s).c);
            } else {
                (*s).ca.cmdchar = Ctrl_BSL;
                (*s).ca.nchar = (*s).c;
                (*s).idx = find_command((*s).ca.cmdchar);
                debug_assert!((*s).idx >= 0);
            }
        }
    }
}

/// Collect the combining characters that belong with the character just read.
///
/// Only for a command flagged `NV_LANG` -- `f`, `t`, `r` and friends, where
/// the argument is a real character rather than a command key.
unsafe fn read_composing_tail(s: *mut NormalState) {
    // SAFETY: `s` is the caller's live state; every write to
    // `nchar_composing` is bounded by its own length below.
    unsafe {
        (*no_mapping.ptr()) -= 1;
        let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
        let mut prev_code = (*s).ca.nchar;
        loop {
            (*s).c = vpeekc();
            if !((*s).c > 0
                && ((*s).c >= 0x100 || (*utf8len_tab.ptr())[vpeekc() as usize] as c_int > 1))
            {
                break;
            }
            (*s).c = plain_vgetc();
            if !utf_iscomposing(prev_code, (*s).c, &raw mut state) {
                vungetc((*s).c);
                break;
            }
            // The base character is only encoded once a tail turns up.
            if (*s).ca.nchar_len == 0 {
                (*s).ca.nchar_len =
                    utf_char2bytes((*s).ca.nchar, (*s).ca.nchar_composing.as_mut_ptr());
            }
            if (*s).ca.nchar_len + utf_char2len((*s).c) < size_of::<[c_char; 32]>() as c_int {
                (*s).ca.nchar_len += utf_char2bytes(
                    (*s).c,
                    (*s).ca
                        .nchar_composing
                        .as_mut_ptr()
                        .offset((*s).ca.nchar_len as isize),
                );
            }
            prev_code = (*s).c;
        }
        (*s).ca.nchar_composing[(*s).ca.nchar_len as usize] = NUL as c_char;
        (*no_mapping.ptr()) += 1;
        // The keys are recorded for a redo, not fed through undo syncing.
        (*no_u_sync.ptr()) += 1;
        gotchars_ignore();
        (*no_u_sync.ptr()) -= 1;
    }
}

/// Read the second (and sometimes third) character of a command.
pub(crate) unsafe fn normal_get_additional_char(s: *mut NormalState) {
    // SAFETY: `s` is the caller's live state and `s.idx` is a valid row.
    unsafe {
        // Nothing read here is a mapping or a command; it is an argument.
        (*no_mapping.ptr()) += 1;
        (*allow_keys.ptr()) += 1;
        did_cursorhold.set(true);

        let (slot, lit, repl) = additional_char_slot(s);
        let lang = repl || (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_LANG != 0;

        if slot != Slot::None {
            let cp: *mut c_int = match slot {
                Slot::NChar => &raw mut (*s).ca.nchar,
                Slot::Extra => &raw mut (*s).ca.extra_char,
                Slot::None => unreachable!(),
            };
            if repl {
                State.set(MODE_REPLACE);
                ui_cursor_shape_no_check_conceal();
            }
            // A language-mapped argument is read *with* mappings on, which is
            // the whole point of 'iminsert' being lmap.
            let langmap_active = lang && (*curbuf.get()).b_p_iminsert == B_IMODE_LMAP as OptInt;
            if langmap_active {
                (*no_mapping.ptr()) -= 1;
                (*allow_keys.ptr()) -= 1;
                State.set(if repl { MODE_LREPLACE } else { MODE_LANGMAP });
            }
            *cp = plain_vgetc();
            if langmap_active {
                (*no_mapping.ptr()) += 1;
                (*allow_keys.ptr()) += 1;
            }
            State.set(MODE_NORMAL_BUSY);
            (*s).need_flushbuf |= add_to_showcmd(*cp);

            if !lit {
                // CTRL-K starts a digraph, unless 'cpoptions' says otherwise.
                if *cp == Ctrl_K
                    && ((*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_LANG != 0
                        || slot == Slot::Extra)
                    && vim_strchr(p_cpo.get(), CPO_DIGRAPH).is_null()
                {
                    (*s).c = get_digraph(false);
                    if (*s).c > 0 {
                        *cp = (*s).c;
                        // Take the CTRL-K and its two characters back out of
                        // the echoed command before showing the result.
                        del_from_showcmd(3);
                        (*s).need_flushbuf |= add_to_showcmd(*cp);
                    }
                }
                langmap_adjust(&mut *cp, !lang);
            }

            if slot == Slot::Extra
                && (*s).ca.nchar == Ctrl_BSL
                && ((*s).ca.extra_char == Ctrl_N || (*s).ca.extra_char == Ctrl_G)
            {
                // `g CTRL-\ CTRL-N` is really `CTRL-\ CTRL-N`.
                (*s).ca.cmdchar = Ctrl_BSL;
                (*s).ca.nchar = (*s).ca.extra_char;
                (*s).idx = find_command((*s).ca.cmdchar);
            } else if ((*s).ca.nchar == 'n' as c_int || (*s).ca.nchar == 'N' as c_int)
                && (*s).ca.cmdchar == 'g' as c_int
            {
                // `gn`/`gN` take the operator from the character after them.
                (*(*s).ca.oap).op_type = get_op_type(*cp, NUL);
            } else if *cp == Ctrl_BSL {
                resolve_ctrl_backslash(s);
            }

            if lang {
                read_composing_tail(s);
            }
        }

        (*no_mapping.ptr()) -= 1;
        (*allow_keys.ptr()) -= 1;
    }
}

/// Mirror a horizontal command for a right-to-left window.
pub(crate) unsafe fn normal_invert_horizontal(s: *mut NormalState) {
    // SAFETY: `s` is the caller's live state.
    unsafe {
        const K_C_LEFT: c_int = -(253 + ((KE_C_LEFT as c_int) << 8));
        const K_C_RIGHT: c_int = -(253 + ((KE_C_RIGHT as c_int) << 8));
        (*s).ca.cmdchar = match (*s).ca.cmdchar {
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
        (*s).idx = find_command((*s).ca.cmdchar);
    }
}

/// Read the digits, and possibly the CTRL-W, in front of a command.
///
/// Answers whether a CTRL-W was consumed, in which case the caller loops:
/// `CTRL-W` takes a count of its own after it.
pub(crate) unsafe fn normal_get_command_count(s: *mut NormalState) -> bool {
    // Select mode swallows printable keys as replacement text, digits too.
    if VIsual_active.get() && VIsual_select.get() {
        return false;
    }
    // SAFETY: `s` is the caller's live state.
    unsafe {
        const K_KDEL: c_int = -(253 + ((KE_KDEL as c_int) << 8));
        while ((*s).c >= '1' as c_int && (*s).c <= '9' as c_int)
            || ((*s).ca.count0 != 0
                && ((*s).c == K_DEL || (*s).c == K_KDEL || (*s).c == '0' as c_int))
        {
            if (*s).c == K_DEL || (*s).c == K_KDEL {
                (*s).ca.count0 /= 10;
                // Four columns: <Del> is echoed as its key name.
                del_from_showcmd(4);
            } else if (*s).ca.count0 > 99999999 {
                // Saturate rather than overflow; nine digits is the most a
                // count is ever allowed to be.
                (*s).ca.count0 = 999999999;
            } else {
                (*s).ca.count0 = (*s).ca.count0 * 10 + ((*s).c - '0' as c_int);
            }
            if (*s).toplevel && readbuf1_empty() {
                set_vcount_ca(&raw mut (*s).ca, &mut (*s).set_prevcount);
            }
            if (*s).ctrl_w {
                (*no_mapping.ptr()) += 1;
                (*allow_keys.ptr()) += 1;
            }
            // A '0' here is a count digit, not the "go to column 0" command,
            // so it must not be mapped.
            (*no_zero_mapping.ptr()) += 1;
            (*s).c = plain_vgetc();
            langmap_adjust(&mut (*s).c, true);
            (*no_zero_mapping.ptr()) -= 1;
            if (*s).ctrl_w {
                (*no_mapping.ptr()) -= 1;
                (*allow_keys.ptr()) -= 1;
            }
            (*s).need_flushbuf |= add_to_showcmd((*s).c);
        }

        // CTRL-W takes the count read so far as its own, then a second count
        // for the window command after it.
        if (*s).c == Ctrl_W && !(*s).ctrl_w && (*s).oa.op_type == OP_NOP as c_int {
            (*s).ctrl_w = true;
            (*s).ca.opcount = (*s).ca.count0;
            (*s).ca.count0 = 0;
            (*no_mapping.ptr()) += 1;
            (*allow_keys.ptr()) += 1;
            (*s).c = plain_vgetc();
            langmap_adjust(&mut (*s).c, true);
            (*no_mapping.ptr()) -= 1;
            (*allow_keys.ptr()) -= 1;
            (*s).need_flushbuf |= add_to_showcmd((*s).c);
            return true;
        }
        false
    }
}

/// Everything that happens after the handler has run.
pub(crate) unsafe fn normal_finish_command(s: *mut NormalState) {
    const K_IGNORE: c_int = -(253 + ((KE_IGNORE as c_int) << 8));
    const K_MOUSEMOVE: c_int = -(253 + ((KE_MOUSEMOVE as c_int) << 8));
    const K_EVENT: c_int = -(253 + ((KE_EVENT as c_int) << 8));

    // SAFETY: `s` is the caller's live state.
    unsafe {
        let mut did_visual_op = false;
        if !(*s).command_finished {
            // A command that is not itself an operator, and does not claim
            // NV_KEEPREG, releases the register it was given.
            if !finish_op.get()
                && (*s).oa.op_type == 0
                && ((*s).idx < 0
                    || (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_KEEPREG == 0)
            {
                clearop(&raw mut (*s).oa);
                set_reg_var(get_default_register_name());
            }
            if (*s).old_mapped_len > 0 {
                (*s).old_mapped_len = typebuf_maplen();
            }
            if (*s).ca.cmdchar != K_IGNORE && (*s).ca.cmdchar != K_MOUSEMOVE {
                did_visual_op = VIsual_active.get()
                    && (*s).oa.op_type != OP_NOP as c_int
                    && (*s).oa.op_type != OP_COLON as c_int;
                do_pending_operator(&raw mut (*s).ca, (*s).old_col, false);
            }
            if normal_need_redraw_mode_message(s) {
                normal_redraw_mode_message();
            }
        }
        msg_nowait.set(false);

        if finish_op.get() || did_visual_op {
            set_reg_var(get_default_register_name());
        }
        let prev_finish_op = finish_op.get();
        if (*s).oa.op_type == OP_NOP as c_int {
            finish_op.set(false);
            may_trigger_modechanged();
        }
        // The cursor shape says whether an operator is pending, and `r`/`gr`
        // change it while they wait.
        if prev_finish_op
            || (*s).ca.cmdchar == 'r' as c_int
            || ((*s).ca.cmdchar == 'g' as c_int && (*s).ca.nchar == 'r' as c_int)
        {
            ui_cursor_shape();
        }
        if (*s).oa.op_type == OP_NOP as c_int && (*s).oa.regname == 0 && (*s).ca.cmdchar != K_EVENT
        {
            clear_showcmd();
        }
        checkpcmark();
        xfree((*s).ca.searchbuf.cast::<c_void>());
        mb_check_adjust_col(curwin.get().cast::<c_void>());

        if (*curwin.get()).w_onebuf_opt.wo_scb != 0 && (*s).toplevel {
            validate_cursor(curwin.get());
            do_check_scrollbind(true);
        }
        if (*curwin.get()).w_onebuf_opt.wo_crb != 0 && (*s).toplevel {
            validate_cursor(curwin.get());
            do_check_cursorbind();
        }

        // A command may have asked for insert mode or for Select mode to be
        // resumed; neither happens until the command is completely done.
        let want_insert =
            restart_edit.get() != 0 && !VIsual_active.get() && (*s).old_mapped_len == 0;
        if (*s).oa.op_type == OP_NOP as c_int
            && (want_insert || restart_VIsual_select.get() == 1)
            && (*s).ca.retval & CA_COMMAND_BUSY as c_int == 0
            && stuff_empty()
            && (*s).oa.regname == 0
        {
            if restart_VIsual_select.get() == 1 {
                VIsual_select.set(true);
                VIsual_select_reg.set(0);
                may_trigger_modechanged();
                showmode();
                restart_VIsual_select.set(0);
            }
            if want_insert {
                edit(restart_edit.get(), false, 1);
            }
        }
        // 2 means "next command", 1 means "this one"; the countdown is here.
        if restart_VIsual_select.get() == 2 {
            restart_VIsual_select.set(1);
        }
        opcount.set((*s).ca.opcount);
    }
}

/// One normal-mode command, from its first key to the end of its effects.
///
/// Kept `extern "C"`: it is installed as a `state_execute_callback` and
/// `state_enter` calls it through that pointer.
pub(crate) unsafe extern "C" fn normal_execute(state: *mut VimState, key: c_int) -> c_int {
    const K_IGNORE: c_int = -(253 + ((KE_IGNORE as c_int) << 8));
    const K_EVENT: c_int = -(253 + ((KE_EVENT as c_int) << 8));

    // SAFETY: `state` is the `VimState` at the head of the `NormalState` the
    // caller handed to `state_enter`.
    unsafe {
        let s = state as *mut NormalState;
        (*s).command_finished = false;
        (*s).ctrl_w = false;
        (*s).old_col = (*curwin.get()).w_curswant as c_int;
        (*s).c = key;
        langmap_adjust(&mut (*s).c, get_real_state() != MODE_SELECT);

        if restart_edit.get() == 0 {
            (*s).old_mapped_len = 0;
        } else if (*s).old_mapped_len != 0
            || (VIsual_active.get() && (*s).mapped_len == 0 && typebuf_maplen() > 0)
        {
            (*s).old_mapped_len = typebuf_maplen();
        }

        if (*s).c == NUL {
            (*s).c = K_ZERO;
        }

        // In Select mode a printable key replaces the selection: the key is
        // put back for insert mode to read and the command becomes a change.
        if VIsual_active.get()
            && VIsual_select.get()
            && (vim_isprintc((*s).c) || (*s).c == NL || (*s).c == CAR || (*s).c == K_KENTER)
        {
            let len = ins_char_typebuf(vgetc_char.get(), vgetc_mod_mask.get(), true);
            if KeyTyped.get() {
                ungetchars(len);
            }
            (*s).c = if restart_edit.get() != 0 {
                'd' as c_int
            } else {
                'c' as c_int
            };
            msg_nowait.set(true);
            (*s).old_mapped_len = 0;
        }

        (*s).need_flushbuf = add_to_showcmd((*s).c);
        while normal_get_command_count(s) {}

        if (*s).c == K_EVENT {
            // An event is not a command: the count it interrupted is stashed
            // for the real command that follows.
            (*s).oa.prev_opcount = (*s).ca.opcount;
            (*s).oa.prev_count0 = (*s).ca.count0;
        } else if (*s).ca.opcount != 0 {
            // An operator count and a motion count multiply, saturating.
            if (*s).ca.count0 != 0 {
                if (*s).ca.opcount >= 999999999 / (*s).ca.count0 {
                    (*s).ca.count0 = 999999999;
                } else {
                    (*s).ca.count0 *= (*s).ca.opcount;
                }
            } else {
                (*s).ca.count0 = (*s).ca.opcount;
            }
        }
        (*s).ca.opcount = (*s).ca.count0;
        (*s).ca.count1 = if (*s).ca.count0 == 0 {
            1
        } else {
            (*s).ca.count0
        };
        if (*s).toplevel && readbuf1_empty() {
            set_vcount(
                (*s).ca.count0 as int64_t,
                (*s).ca.count1 as int64_t,
                (*s).set_prevcount,
            );
        }

        if (*s).ctrl_w {
            (*s).ca.nchar = (*s).c;
            (*s).ca.cmdchar = Ctrl_W;
        } else {
            (*s).ca.cmdchar = (*s).c;
        }
        (*s).idx = find_command((*s).ca.cmdchar);

        if (*s).idx < 0 {
            clearopbeep(&raw mut (*s).oa);
            (*s).command_finished = true;
        } else if (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_NCW != 0
            && check_text_or_curbuf_locked(&raw mut (*s).oa)
        {
            (*s).command_finished = true;
        } else if VIsual_active.get() && normal_handle_special_visual_command(s) {
            (*s).command_finished = true;
        } else {
            if (*curwin.get()).w_onebuf_opt.wo_rl != 0
                && KeyTyped.get()
                && KeyStuffed.get() == 0
                && (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_RL != 0
            {
                normal_invert_horizontal(s);
            }
            if normal_need_additional_char(s) {
                normal_get_additional_char(s);
            }
            if (*s).need_flushbuf {
                ui_flush();
            }
            if (*s).ca.cmdchar != K_IGNORE && (*s).ca.cmdchar != K_EVENT {
                did_cursorhold.set(false);
            }
            State.set(MODE_NORMAL);

            if (*s).ca.nchar == ESC || (*s).ca.extra_char == ESC {
                clearop(&raw mut (*s).oa);
                (*s).command_finished = true;
            } else {
                if (*s).ca.cmdchar != K_IGNORE {
                    msg_didout.set(false);
                    msg_col.set(0);
                }
                (*s).old_pos = (*curwin.get()).w_cursor;

                // 'keymodel' startsel: a shifted special key starts a
                // selection and then acts as its unshifted self.
                if !VIsual_active.get() && km_startsel.get() {
                    let flags = (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int;
                    if flags & NV_SS != 0 {
                        start_selection();
                        unshift_special(&raw mut (*s).ca);
                        (*s).idx = find_command((*s).ca.cmdchar);
                        debug_assert!((*s).idx >= 0);
                    } else if flags & NV_SSS != 0 && mod_mask.get() & MOD_MASK_SHIFT != 0 {
                        start_selection();
                        (*mod_mask.ptr()) &= !MOD_MASK_SHIFT;
                    }
                }

                (*s).ca.arg = (*nv_cmds.ptr())[(*s).idx as usize].cmd_arg as c_int;
                ((*nv_cmds.ptr())[(*s).idx as usize].cmd_func)
                    .expect("every nv_cmds row has a handler")(&raw mut (*s).ca);
            }
        }
        normal_finish_command(s);
        1
    }
}

/// Record a command for `.`, taking its second character from `cap`.
pub(crate) unsafe fn prep_redo_cmd(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        prep_redo(
            (*(*cap).oap).regname,
            (*cap).count0,
            NUL,
            (*cap).cmdchar,
            NUL,
            NUL,
            NUL,
        );
        // A character with a combining tail is replayed as its whole encoding.
        if (*cap).nchar_len > 0 {
            AppendToRedobuff((*cap).nchar_composing.as_mut_ptr());
        } else {
            AppendCharToRedobuff((*cap).nchar);
        }
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
    unsafe {
        ResetRedobuff();
        if regname != 0 {
            AppendCharToRedobuff('"' as c_int);
            AppendCharToRedobuff(regname);
        }
        if num1 != 0 {
            AppendNumberToRedobuff(num1);
        }
        for cmd in [cmd1, cmd2] {
            if cmd != NUL {
                AppendCharToRedobuff(cmd);
            }
        }
        if num2 != 0 {
            AppendNumberToRedobuff(num2);
        }
        for cmd in [cmd3, cmd4, cmd5] {
            if cmd != NUL {
                AppendCharToRedobuff(cmd);
            }
        }
    }
}

/// Beep and clear the operator if one is pending. Answers whether it was.
pub(crate) unsafe fn checkclearop(oap: *mut oparg_T) -> bool {
    // SAFETY: `oap` is the caller's live operator.
    unsafe {
        if (*oap).op_type == OP_NOP as c_int {
            return false;
        }
        clearopbeep(oap);
        true
    }
}

/// As [`checkclearop`], and also refuse while a Visual selection is up --
/// for commands that make no sense applied to one.
pub(crate) unsafe fn checkclearopq(oap: *mut oparg_T) -> bool {
    // SAFETY: `oap` is the caller's live operator.
    unsafe {
        if (*oap).op_type == OP_NOP as c_int && !VIsual_active.get() {
            return false;
        }
        clearopbeep(oap);
        true
    }
}

/// Forget the pending operator, its register and its forced motion kind.
pub(crate) unsafe fn clearop(oap: *mut oparg_T) {
    // SAFETY: `oap` is the caller's live operator.
    unsafe {
        (*oap).op_type = OP_NOP as c_int;
        (*oap).regname = 0;
        (*oap).motion_force = NUL;
        (*oap).use_reg_one = false;
    }
    motion_force.set(NUL);
}

/// [`clearop`], and say so.
///
/// The beep also flushes the typeahead, which is what makes a failed command
/// abandon the rest of a mapping or a `:normal` argument.
pub(crate) unsafe fn clearopbeep(oap: *mut oparg_T) {
    // SAFETY: `oap` is the caller's live operator.
    unsafe { clearop(oap) };
    // SAFETY: touches only message and typeahead state.
    unsafe { beep_flush() };
}

/// Read one more key for a command that takes several, with mappings and
/// 'langmap' handled the way a command character wants them, and show it in
/// the 'showcmd' area.
pub(crate) unsafe fn read_command_char() -> c_int {
    // SAFETY: adjusts the two counters that suppress mapping around one read.
    unsafe {
        (*no_mapping.ptr()) += 1;
        (*allow_keys.ptr()) += 1;
        let mut c = plain_vgetc();
        langmap_adjust(&mut c, true);
        (*no_mapping.ptr()) -= 1;
        (*allow_keys.ptr()) -= 1;
        add_to_showcmd(c);
        c
    }
}

/// Open a fold the cursor has landed in, if the 'foldopen' flag for this kind
/// of movement is set, the key was typed rather than mapped, and no operator
/// is waiting for the motion to finish.
pub(crate) unsafe fn may_fold_open(cap: *mut cmdarg_T, fdo_flag: c_uint) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if fdo_flags.get() & fdo_flag != 0
            && KeyTyped.get()
            && (*(*cap).oap).op_type == OP_NOP as c_int
        {
            foldOpenCursor();
        }
    }
}

/// Turn a shifted special key into its unshifted self.
pub(crate) unsafe fn unshift_special(cap: *mut cmdarg_T) {
    const K_S_UP: c_int = -1277;
    const K_S_DOWN: c_int = -1533;
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        (*cap).cmdchar = match (*cap).cmdchar {
            K_S_RIGHT => K_RIGHT,
            K_S_LEFT => K_LEFT,
            K_S_UP => K_UP,
            K_S_DOWN => K_DOWN,
            K_S_HOME => K_HOME,
            K_S_END => K_END,
            other => other,
        };
        (*cap).cmdchar = simplify_key((*cap).cmdchar, mod_mask.ptr());
    }
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
