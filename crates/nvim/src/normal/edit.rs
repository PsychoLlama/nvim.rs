//! Commands that change the text without entering insert mode, and the
//! ones whose whole job is to enter it.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::memline::MlFlags;
use crate::winlayer::{Buf, Win};
use core::ptr;

use crate::ascii::ascii_isdigit;
use crate::buffer::{bt_prompt, buf_get_changedtick};
use crate::change::{changed_lines, del_chars, deleted_lines, ins_char, ins_char_bytes, open_line};
use crate::cursor::{
    check_cursor, coladvance, coladvance_force, gchar_cursor, get_cursor_pos_len,
    get_cursor_pos_ptr, getviscol, inc_cursor,
};
use crate::diff::nv_diffgetput;
use crate::drawscreen::win_cursorline_standout;
use crate::edit::{
    BeginlineOpts, beginline, edit, get_literal, ins_copychar, prompt_curpos_editable,
    set_last_insert,
};
use crate::fold::{fold_update_after_insert, has_folding};
use crate::getchar::{
    append_to_redobuff, append_to_redobuff_char, stuff_empty, stuff_readbuf, stuff_readbuf_char,
    stuff_readbuf_number,
};
use crate::guard::Suppress;
use crate::keycodes::{Ctrl_A, Ctrl_E, Ctrl_Q, Ctrl_V, Ctrl_Y, K_DEL, K_INS, K_KDEL, K_KINS};
use crate::main::{
    State, cb_flags, curbuf, curwin, e_modifiable, got_int, p_sel, p_sta, p_to, p_ww, restart_edit,
};
use crate::mbyte::{mb_adjust_cursor, mb_charlen};
use crate::memline::{inc, ml_delete_flags, ml_get};
use crate::memory::xfree;
use crate::message::emsg;
use crate::r#move::WinValid;
use crate::normal::{
    CA_COMMAND_BUSY, CAR, CmdArg, DEL, ESC, ML_DEL_MESSAGE, NL, OPENLINE_DO_COM, REPLACE_CR_NCHAR,
    REPLACE_NL_NCHAR, TAB, VIsual_mode_orig, VisualMode, check_clear_op, check_clear_op_quit,
    clear_op, clear_op_beep, nv_object, nv_operator, prep_redo, prep_redo_cmd, set_visual_active,
    set_visual_mode, v_swap_corners, v_visop, visual_active, visual_mode,
};
use crate::ops::{do_join, do_pending_operator, op_addsub, swapchar};
use crate::option::get_ve_flags;
use crate::options::{kOptCbFlagUnnamed, kOptCbFlagUnnamedplus, kOptVeFlagAll};
use crate::os::cshim::gettext;
use crate::pos::MAXCOL;
use crate::register::{copy_register, do_put, free_register};
use crate::search::{BACKWARD, FORWARD};
use crate::state::{MODE_INSERT, MODE_REPLACE, virtual_active};
use crate::strings::vim_strchr;
use crate::textformat::{auto_format, has_format_option};
use crate::types::{
    FoFlag, NUL, OP_DELETE, OP_NOP, OP_NR_ADD, OP_NR_SUB, OP_TILDE, PUT_BLOCK_INNER, PUT_CURSEND,
    PUT_FIXINDENT, PUT_LINE, PUT_LINE_FORWARD, PUT_LINE_SPLIT, cmdarg_T, colnr_T, linenr_T, size_t,
    yankreg_T,
};
use crate::undo::{u_clearline, u_save, u_save_cursor, u_savesub};
use ::libc::strlen;
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};

/// Refuse a change in a prompt buffer that is not on its own editable line.
unsafe fn prompt_refuses(cap: *mut cmdarg_T) -> bool {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if unsafe { bt_prompt(curbuf.get()) } && !unsafe { prompt_curpos_editable() } {
        clear_op_beep(ca.op());
        return true;
    }
    false
}

/// `CTRL-A` and `CTRL-X`: add to or subtract from the number under the cursor.
pub(crate) unsafe fn nv_addsub(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if unsafe { prompt_refuses(cap) } {
        return;
    }
    if !visual_active() && ca.op().op_type == OP_NOP {
        // Not an operator: run it here and then put the operator back.
        unsafe { prep_redo_cmd(cap) };
        ca.op().op_type = if ca.cmdchar == Ctrl_A {
            OP_NR_ADD
        } else {
            OP_NR_SUB
        };
        unsafe { op_addsub(ca.oap, ca.count1 as linenr_T, ca.arg != 0) };
        ca.op().op_type = OP_NOP;
    } else if visual_active() {
        unsafe { nv_operator(cap) };
    } else {
        clear_op(ca.op());
    }
}

/// `r`: replace `count1` characters with the one that follows.
pub(crate) unsafe fn nv_replace(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if check_clear_op(ca.op()) || unsafe { prompt_refuses(cap) } {
        return;
    }
    // `r CTRL-V` reads the next key literally. Only a byte-sized answer
    // still counts as literal: a larger one came from a digraph or a
    // `<C-u>` escape and behaves as an ordinary character.
    let mut literal = NUL;
    if ca.nchar == Ctrl_V || ca.nchar == Ctrl_Q {
        literal = Ctrl_V;
        ca.nchar = unsafe { get_literal(false) };
        if ca.nchar > DEL {
            literal = NUL;
        }
    }
    if ca.nchar < 0 {
        clear_op_beep(ca.op());
        return;
    }
    if visual_active() {
        // The selection form is an operator; the interrupt a long
        // selection may have raised is not this command's business.
        if got_int.get() {
            got_int.set(false);
        }
        if literal != NUL {
            // A literal line break has to survive being carried through
            // `cap` as a character.
            if ca.nchar == CAR {
                ca.nchar = REPLACE_CR_NCHAR;
            } else if ca.nchar == NL {
                ca.nchar = REPLACE_NL_NCHAR;
            }
        }
        unsafe { nv_operator(cap) };
        return;
    }
    if unsafe { virtual_active(curwin.get()) } {
        if u_save_cursor() == 0 {
            return;
        }
        if gchar_cursor() == NUL {
            // Past the end of the line: make room for the whole count and
            // then step back to where the replacing starts.
            unsafe { coladvance_force(getviscol() + ca.count1) };
            cur_win().w_cursor.col -= ca.count1;
        } else if gchar_cursor() == TAB {
            // Land on the tab's first cell, not the cell of it the cursor
            // happens to be showing on.
            unsafe { coladvance_force(getviscol()) };
        }
    }
    // There have to be `count1` characters left on the line, counted both
    // ways: the byte length rules out a short line cheaply.
    if (get_cursor_pos_len() as size_t) < ca.count1 as c_uint as size_t
        || unsafe { mb_charlen(get_cursor_pos_ptr()) } < ca.count1
    {
        clear_op_beep(ca.op());
        return;
    }
    // A tab that 'expandtab' or 'smarttab' would turn into spaces is
    // easier to get right by replaying the whole thing as `R<Tab><Esc>`.
    if literal != Ctrl_V && ca.nchar == '\t' as c_int && (cur_buf().b_p_et != 0 || p_sta.get() != 0)
    {
        stuff_readbuf_number(ca.count1);
        stuff_readbuf_char('R' as c_int);
        stuff_readbuf_char('\t' as c_int);
        stuff_readbuf_char(ESC);
        return;
    }
    if u_save_cursor() == 0 {
        return;
    }
    if literal != Ctrl_V && (ca.nchar == '\r' as c_int || ca.nchar == '\n' as c_int) {
        // Replacing with a line break splits the line, which is an insert.
        unsafe { del_chars(ca.count1, 0) };
        stuff_readbuf_char('\r' as c_int);
        stuff_readbuf_char(ESC);
        unsafe { invoke_edit(cap, 1, 'r' as c_int, 0) };
        unsafe { fold_update_after_insert() };
        return;
    }

    prep_redo(
        ca.op().regname,
        ca.count1,
        NUL,
        'r' as c_int,
        NUL,
        literal,
        0,
    );
    cur_buf().b_op_start = cur_win().w_cursor;
    let old_state = State.get();
    if ca.nchar_len > 0 {
        unsafe { append_to_redobuff(&raw mut ca.nchar_composing as *mut c_char) };
    } else {
        append_to_redobuff_char(ca.nchar);
    }
    for _ in 0..ca.count1 {
        // `ins_char` looks at 'State' to decide it is overwriting rather
        // than inserting.
        State.set(MODE_REPLACE);
        if ca.nchar == Ctrl_E || ca.nchar == Ctrl_Y {
            // `r CTRL-E` and `r CTRL-Y` copy from the line below or above.
            let from = cur_win().w_cursor.lnum + if ca.nchar == Ctrl_Y { -1 } else { 1 };
            let c = unsafe { ins_copychar(from) };
            if c != NUL {
                unsafe { ins_char(c) };
            } else {
                // Nothing there to copy: leave the character alone and
                // step over it.
                cur_win().w_cursor.col += 1;
            }
        } else if ca.nchar_len != 0 {
            let bytes = &raw mut ca.nchar_composing as *mut c_char;
            unsafe { ins_char_bytes(bytes, ca.nchar_len as size_t) };
        } else {
            unsafe { ins_char(ca.nchar) };
        }
        State.set(old_state);
    }
    cur_win().w_cursor.col -= 1;
    unsafe { mb_adjust_cursor() };
    cur_buf().b_op_end = cur_win().w_cursor;
    cur_win().w_set_curswant = true;
    unsafe { set_last_insert(ca.nchar) };
    unsafe { fold_update_after_insert() };
}

/// `R` and `gR`: replace mode, virtual with the argument set.
pub(crate) unsafe fn nv_replace_mode(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if visual_active() {
        // A selection is replaced linewise, which is `c` over whole lines.
        ca.cmdchar = 'c' as c_int;
        ca.nchar = NUL;
        VIsual_mode_orig.set(visual_mode());
        set_visual_mode(VisualMode::LINE);
        unsafe { nv_operator(cap) };
        return;
    }
    if check_clear_op_quit(ca.op()) {
        return;
    }
    if cur_buf().b_p_ma == 0 {
        unsafe { emsg(gettext(&raw const e_modifiable as *const c_char)) };
        return;
    }
    if unsafe { virtual_active(curwin.get()) } {
        unsafe { coladvance(curwin.get(), getviscol()) };
    }
    let kind = if ca.arg != 0 {
        'V' as c_int
    } else {
        'R' as c_int
    };
    unsafe { invoke_edit(cap, 0, kind, 0) };
}

/// `gr`: replace one character virtually -- the following text does not move.
pub(crate) unsafe fn nv_vreplace(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if visual_active() {
        ca.cmdchar = 'r' as c_int;
        ca.nchar = ca.extra_char;
        unsafe { nv_replace(cap) };
        return;
    }
    if check_clear_op_quit(ca.op()) {
        return;
    }
    if cur_buf().b_p_ma == 0 {
        unsafe { emsg(gettext(&raw const e_modifiable as *const c_char)) };
        return;
    }
    if ca.extra_char == Ctrl_V || ca.extra_char == Ctrl_Q {
        ca.extra_char = unsafe { get_literal(false) };
    }
    // Replay the character through virtual replace mode. A control
    // character needs its own CTRL-V to survive the replay.
    if ca.extra_char < ' ' as c_int {
        stuff_readbuf_char(Ctrl_V);
    }
    stuff_readbuf_char(ca.extra_char);
    stuff_readbuf_char(ESC);
    if unsafe { virtual_active(curwin.get()) } {
        unsafe { coladvance(curwin.get(), getviscol()) };
    }
    unsafe { invoke_edit(cap, 1, 'v' as c_int, 0) };
}

/// `~` when 'tildeop' is off: swap the case of `count1` characters.
pub(crate) unsafe fn n_swapchar(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if check_clear_op_quit(ca.op()) {
        return;
    }
    // An empty line has nothing to swap unless 'whichwrap' lets `~` move
    // to the next one.
    let wraps = !unsafe { vim_strchr(p_ww.get(), '~' as c_int) }.is_null();
    if unsafe { *ml_get(cur_win().w_cursor.lnum) } as c_int == NUL && !wraps {
        clear_op_beep(ca.op());
        return;
    }
    unsafe { prep_redo_cmd(cap) };
    if u_save_cursor() == 0 {
        return;
    }
    let startpos = cur_win().w_cursor;
    let mut did_change = false;
    let mut n = ca.count1;
    while n > 0 {
        did_change |= unsafe { swapchar(ca.op().op_type, &raw mut (*curwin.get()).w_cursor) };
        inc_cursor();
        if gchar_cursor() == NUL {
            if !(wraps && cur_win().w_cursor.lnum < cur_buf().b_ml.ml_line_count) {
                break;
            }
            cur_win().w_cursor.lnum += 1;
            cur_win().w_cursor.col = 0;
            // Each further line needs its own undo entry.
            if n > 1 {
                if unsafe { u_savesub(cur_win().w_cursor.lnum) } == 0 {
                    break;
                }
                unsafe { u_clearline(curbuf.get()) };
            }
        }
        n -= 1;
    }
    unsafe { check_cursor(curwin.get()) };
    cur_win().w_set_curswant = true;
    if did_change {
        let (from, col) = (startpos.lnum, startpos.col);
        let to = cur_win().w_cursor.lnum + 1;
        unsafe { changed_lines(curbuf.get(), from, col, to, 0, true) };
        cur_buf().b_op_start = startpos;
        cur_buf().b_op_end = cur_win().w_cursor;
        if cur_buf().b_op_end.col > 0 {
            cur_buf().b_op_end.col -= 1;
        }
    }
}

/// `s` and `S`: substitute a character or a line.
pub(crate) unsafe fn nv_subst(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if unsafe { prompt_refuses(cap) } {
        return;
    }
    if visual_active() {
        if ca.cmdchar == 'S' as c_int {
            // `S` on a selection is linewise however the selection was
            // made; the original kind is remembered for the redo.
            VIsual_mode_orig.set(visual_mode());
            set_visual_mode(VisualMode::LINE);
        }
        ca.cmdchar = 'c' as c_int;
        unsafe { nv_operator(cap) };
    } else {
        unsafe { nv_optrans(cap) };
    }
}

/// `x`, `X`, `D`, `C`, `Y`: the one-key spellings of an operator and a motion.
pub(crate) unsafe fn nv_abbrev(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if ca.cmdchar == K_DEL || ca.cmdchar == K_KDEL {
        ca.cmdchar = 'x' as c_int;
    }
    if visual_active() {
        unsafe { v_visop(cap) };
    } else {
        unsafe { nv_optrans(cap) };
    }
}

/// Replay a one-key command as the operator and motion it stands for.
pub(crate) unsafe fn nv_optrans(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    /// What each abbreviating key means. Upstream indexes two parallel arrays
    /// with a `strchr` offset, which reaches one past the end for a key that
    /// is not in the set -- unreachable, because only these keys route here,
    /// but not something a lookup has to make possible.
    const TRANSLATIONS: [(c_int, &CStr); 8] = [
        ('x' as c_int, c"dl"),
        ('X' as c_int, c"dh"),
        ('D' as c_int, c"d$"),
        ('C' as c_int, c"c$"),
        ('s' as c_int, c"cl"),
        ('S' as c_int, c"cc"),
        ('Y' as c_int, c"yy"),
        ('&' as c_int, c":s\r"),
    ];
    // SAFETY: `cap` is the caller's live command argument.
    if !check_clear_op_quit(ca.op()) {
        if ca.count0 != 0 {
            stuff_readbuf_number(ca.count0);
        }
        for (key, keys) in TRANSLATIONS {
            if key == ca.cmdchar {
                unsafe { stuff_readbuf(keys.as_ptr()) };
                break;
            }
        }
    }
    // The count went into the replayed keys, so it must not also apply to
    // whatever they turn out to be.
    ca.opcount = 0;
}

/// `o` and `O`: open a line below or above and start inserting on it.
pub(crate) unsafe fn n_opencmd(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if check_clear_op_quit(ca.op()) {
        return;
    }
    let mut win = cur_win();
    let opening_above = ca.cmdchar == 'O' as c_int;
    // Open outside a closed fold rather than inside it.
    let (lnum, edge) = (win.w_cursor.lnum, &raw mut win.w_cursor.lnum);
    let (first, last) = if opening_above {
        (edge, ptr::null_mut())
    } else {
        (ptr::null_mut(), edge)
    };
    // SAFETY: `win` is the live window and both ends are this frame's own.
    unsafe { has_folding(win.raw(), lnum, first, last) };
    cur_buf().b_last_changedtick_i = unsafe { buf_get_changedtick(curbuf.get()) };
    let undo_first = win.w_cursor.lnum - linenr_T::from(opening_above);
    let undo_last = win.w_cursor.lnum + linenr_T::from(!opening_above);
    let dir = if opening_above {
        BACKWARD as c_int
    } else {
        FORWARD as c_int
    };
    // SAFETY: reads the current buffer's 'formatoptions'.
    let flags = if has_format_option(FoFlag::OPEN_COMS) {
        OPENLINE_DO_COM as c_int
    } else {
        0
    };
    let opened =
        u_save(undo_first, undo_last) != 0 && unsafe { open_line(dir, flags, 0, ptr::null_mut()) };
    if opened {
        if unsafe { win_cursorline_standout(win.raw()) } {
            // The cursor line moved, so its highlight has to be redrawn.
            win.w_valid.clear(WinValid::CROW);
        }
        unsafe { invoke_edit(cap, 0, ca.cmdchar, 1) };
    }
}

/// `~`: swap case, or the `g~` operator when 'tildeop' is on.
pub(crate) unsafe fn nv_tilde(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if p_to.get() == 0 && !visual_active() && ca.op().op_type != OP_TILDE {
        if unsafe { prompt_refuses(cap) } {
            return;
        }
        unsafe { n_swapchar(cap) };
    } else {
        unsafe { nv_operator(cap) };
    }
}

/// Put the cursor where `A` starts inserting: past the last character, or
/// past the last *cell* when 'virtualedit' is "all".
pub(crate) unsafe fn set_cursor_for_append_to_line() {
    // SAFETY (throughout): reads and writes the current window's cursor.
    cur_win().w_set_curswant = true;
    if unsafe { get_ve_flags(curwin.get()) } == kOptVeFlagAll as c_uint {
        // Insert mode is what makes `coladvance` allow the position one
        // past the end.
        let save_state = State.get();
        State.set(MODE_INSERT);
        unsafe { coladvance(curwin.get(), MAXCOL as c_int) };
        State.set(save_state);
    } else {
        cur_win().w_cursor.col += unsafe { strlen(get_cursor_pos_ptr()) } as colnr_T;
    }
}

/// `a`, `A`, `i` and `I`: enter insert mode.
pub(crate) unsafe fn nv_edit(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if ca.cmdchar == K_INS || ca.cmdchar == K_KINS {
        ca.cmdchar = 'i' as c_int;
    }
    // With a selection up, `A` and `I` insert at every line's end or
    // start; `a` and `i` name a text object instead.
    if visual_active() && (ca.cmdchar == 'A' as c_int || ca.cmdchar == 'I' as c_int) {
        unsafe { v_visop(cap) };
        return;
    }
    if (ca.cmdchar == 'a' as c_int || ca.cmdchar == 'i' as c_int)
        && (ca.op().op_type != OP_NOP || visual_active())
    {
        unsafe { nv_object(cap) };
        return;
    }
    // A terminal buffer is not 'modifiable' and is still editable.
    if cur_buf().b_p_ma == 0 && cur_buf().terminal.is_null() {
        unsafe { emsg(gettext(&raw const e_modifiable as *const c_char)) };
        clear_op(ca.op());
        return;
    }
    if check_clear_op_quit(ca.op()) {
        return;
    }
    match u8::try_from(ca.cmdchar) {
        Ok(b'A') => unsafe { set_cursor_for_append_to_line() },
        Ok(b'I') => beginline(BeginlineOpts::WHITE),
        Ok(b'a') => {
            // `a` steps one right first. Under 'virtualedit' a position
            // inside a tab or past the end of the line moves by a cell.
            if unsafe { virtual_active(curwin.get()) }
                && (cur_win().w_cursor.coladd > 0
                    || unsafe { *get_cursor_pos_ptr() } as c_int == NUL
                    || unsafe { *get_cursor_pos_ptr() } as c_int == TAB)
            {
                cur_win().w_cursor.coladd += 1;
            } else if unsafe { *get_cursor_pos_ptr() } as c_int != NUL {
                inc_cursor();
            }
        }
        _ => {}
    }
    // Insert mode has no virtual column of its own, so anything but `A`
    // has to land on a real one first.
    if cur_win().w_cursor.coladd != 0 && ca.cmdchar != 'A' as c_int {
        let save_state = State.get();
        State.set(MODE_INSERT);
        unsafe { coladvance(curwin.get(), getviscol()) };
        State.set(save_state);
    }
    unsafe { invoke_edit(cap, 0, ca.cmdchar, 0) };
}

/// Enter insert mode and report back whether the command loop should treat
/// this command as still running.
///
/// 'restart_edit' is put back afterwards only if insert mode did not set one
/// itself: whatever it asked for wins over what was pending before.
pub(crate) unsafe fn invoke_edit(cap: *mut cmdarg_T, repl: c_int, cmd: c_int, startln: c_int) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    // A replay or leftover typeahead is allowed to resume a pending
    // insert; a fresh command is not.
    let restart_edit_save = if repl != 0 || !stuff_empty() {
        restart_edit.get()
    } else {
        0
    };
    restart_edit.set(0);
    // `o` and `O` already recorded the tick before opening the line.
    if ca.cmdchar != 'O' as c_int && ca.cmdchar != 'o' as c_int {
        cur_buf().b_last_changedtick_i = unsafe { buf_get_changedtick(curbuf.get()) };
    }
    if unsafe { edit(cmd, startln != 0, ca.count1) } {
        ca.retval |= CA_COMMAND_BUSY as c_int;
    }
    if restart_edit.get() == 0 {
        restart_edit.set(restart_edit_save);
    }
}

/// `J`: join lines.
pub(crate) unsafe fn nv_join(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if visual_active() {
        unsafe { nv_operator(cap) };
        return;
    }
    if check_clear_op(ca.op()) {
        return;
    }
    // Joining fewer than two lines means nothing; `J` and `1J` both join
    // this line with the next.
    ca.count0 = ca.count0.max(2);
    if cur_win().w_cursor.lnum + ca.count0 as linenr_T - 1 > cur_buf().b_ml.ml_line_count {
        // A count that runs off the end joins what is left -- unless there
        // was no count, in which case there is nothing below to join to.
        if ca.count0 <= 2 {
            clear_op_beep(ca.op());
            return;
        }
        ca.count0 = (cur_buf().b_ml.ml_line_count - cur_win().w_cursor.lnum + 1) as c_int;
    }
    prep_redo(
        ca.op().regname,
        ca.count0,
        NUL,
        ca.cmdchar,
        NUL,
        NUL,
        ca.nchar,
    );
    // `gJ` arrives with `nchar` set and does not insert or remove spaces.
    unsafe { do_join(ca.count0 as size_t, ca.nchar == NUL, true, true, true) };
}

/// `p` and `P`.
pub(crate) unsafe fn nv_put(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe { nv_put_opt(cap, false) };
}

/// The put commands. `fix_indent` is the `]p`/`[p` family, which reindents the
/// text to the current line.
pub(crate) unsafe fn nv_put_opt(cap: *mut cmdarg_T, fix_indent: bool) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    let mut win = cur_win();
    let save_fen = win.w_onebuf_opt.wo_fen;
    if ca.op().op_type != OP_NOP {
        // `dp` is not "delete, then put": it is the diff command.
        if ca.op().op_type == OP_DELETE && ca.cmdchar == 'p' as c_int {
            clear_op(ca.op());
            debug_assert!(ca.opcount >= 0);
            unsafe { nv_diffgetput(true, ca.opcount as size_t) };
        } else {
            clear_op_beep(ca.op());
        }
        return;
    }
    if unsafe { bt_prompt(curbuf.get()) } && !unsafe { prompt_curpos_editable() } {
        // On the prompt's own line, put in front of the prompt text
        // rather than refusing.
        if win.w_cursor.lnum == cur_buf().b_prompt_start.mark.lnum {
            win.w_cursor.col = cur_buf().b_prompt_start.mark.col;
            ca.cmdchar = 'P' as c_int;
        } else {
            clear_op_beep(ca.op());
            return;
        }
    }

    let mut flags = 0;
    let mut dir;
    if fix_indent {
        dir = if ca.cmdchar == ']' as c_int && ca.nchar == 'p' as c_int {
            FORWARD as c_int
        } else {
            BACKWARD as c_int
        };
        flags |= PUT_FIXINDENT as c_int;
    } else {
        dir = if ca.cmdchar == 'P' as c_int
            || ((ca.cmdchar == 'g' as c_int || ca.cmdchar == 'z' as c_int)
                && ca.nchar == 'P' as c_int)
        {
            BACKWARD as c_int
        } else {
            FORWARD as c_int
        };
    }
    unsafe { prep_redo_cmd(cap) };
    // `gp` leaves the cursor after the new text; `zp` puts a blockwise
    // register without widening the lines it lands on.
    if ca.cmdchar == 'g' as c_int {
        flags |= PUT_CURSEND as c_int;
    } else if ca.cmdchar == 'z' as c_int {
        flags |= PUT_BLOCK_INNER as c_int;
    }

    let was_visual = visual_active();
    let mut savereg: *mut yankreg_T = ptr::null_mut();
    let mut emptied = false;
    if was_visual {
        let regname = ca.op().regname;
        let keep_registers = ca.cmdchar == 'P' as c_int;
        // Putting over a selection deletes it first, and that delete would
        // otherwise overwrite the very register being put.
        let clipoverwrite = (regname == '+' as c_int || regname == '*' as c_int)
            && cb_flags.get()
                & (kOptCbFlagUnnamed as c_int | kOptCbFlagUnnamedplus as c_int) as c_uint
                != 0;
        if regname == 0
            || regname == '"' as c_int
            || clipoverwrite
            || ascii_isdigit(regname)
            || regname == '-' as c_int
        {
            savereg = unsafe { copy_register(regname) };
        }
        // The delete must not close or open folds under the selection.
        win.w_onebuf_opt.wo_fen = 0;
        // The condition is upstream's; only the `.` register on a
        // charwise selection skips the delete.
        if !visual_active() || visual_mode().is_line() || regname != '.' as c_int {
            ca.cmdchar = 'd' as c_int;
            ca.nchar = NUL;
            ca.op().regname = if keep_registers { '_' as c_int } else { NUL };
            let silenced = Suppress::messages();
            unsafe { nv_operator(cap) };
            unsafe { do_pending_operator(cap, 0, false) };
            // The delete may have left the buffer with one empty line
            // that the put should not keep.
            emptied = cur_buf().b_ml.ml_flags.has(MlFlags::EMPTY);
            drop(silenced);
            ca.op().regname = regname;
        }
        if visual_mode().is_line() {
            flags |= PUT_LINE as c_int;
        } else if visual_mode().is_char() {
            flags |= PUT_LINE_SPLIT as c_int;
        }
        if visual_mode().is_block() && dir == FORWARD as c_int {
            flags |= PUT_LINE_FORWARD as c_int;
        }
        // Put where the selection was, which is where the delete left the
        // cursor -- forwards only when it left it before the start.
        dir = BACKWARD as c_int;
        if (!visual_mode().is_line() && win.w_cursor.col < cur_buf().b_op_start.col)
            || (visual_mode().is_line() && win.w_cursor.lnum < cur_buf().b_op_start.lnum)
        {
            dir = FORWARD as c_int;
        }
        set_visual_active(true);
    }

    unsafe { do_put(ca.op().regname, savereg, dir, ca.count1, flags) };
    if !savereg.is_null() {
        unsafe { free_register(savereg) };
        unsafe { xfree(savereg as *mut c_void) };
    }
    if was_visual {
        if save_fen != 0 {
            win.w_onebuf_opt.wo_fen = 1;
        }
        // Leave `gv` naming what was just put.
        cur_buf().b_visual.vi_start = cur_buf().b_op_start;
        cur_buf().b_visual.vi_end = cur_buf().b_op_end;
        if unsafe { *p_sel.get() } as c_int == 'e' as c_int {
            unsafe { inc(&mut (*curbuf.get()).b_visual.vi_end) };
        }
    }
    if emptied && unsafe { *ml_get(cur_buf().b_ml.ml_line_count) } as c_int == NUL {
        unsafe { ml_delete_flags(cur_buf().b_ml.ml_line_count, ML_DEL_MESSAGE as c_int) };
        unsafe { deleted_lines(cur_buf().b_ml.ml_line_count + 1, 1) };
        if win.w_cursor.lnum > cur_buf().b_ml.ml_line_count {
            win.w_cursor.lnum = cur_buf().b_ml.ml_line_count;
            unsafe { coladvance(win.raw(), MAXCOL as c_int) };
        }
    }
    unsafe { auto_format(false, true) };
}

/// `o` and `O` -- or, with a pending delete, the diff command, and with a
/// selection, "swap to the other corner".
pub(crate) unsafe fn nv_open(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if ca.op().op_type == OP_DELETE && ca.cmdchar == 'o' as c_int {
        // `do` is `:diffget`, not "delete, then open".
        clear_op(ca.op());
        debug_assert!(ca.opcount >= 0);
        unsafe { nv_diffgetput(false, ca.opcount as size_t) };
    } else if visual_active() {
        unsafe { v_swap_corners(ca.cmdchar) };
    } else if unsafe { bt_prompt(curbuf.get()) }
        && cur_win().w_cursor.lnum < cur_buf().b_prompt_start.mark.lnum
    {
        clear_op_beep(ca.op());
    } else {
        unsafe { n_opencmd(cap) };
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
