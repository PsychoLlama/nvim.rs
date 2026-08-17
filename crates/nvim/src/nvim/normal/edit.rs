//! Commands that change the text without entering insert mode, and the
//! ones whose whole job is to enter it.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ptr;

use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::buffer::{bt_prompt, buf_get_changedtick};
use crate::src::nvim::change::{
    changed_lines, del_chars, deleted_lines, ins_char, ins_char_bytes, open_line,
};
use crate::src::nvim::cursor::{
    check_cursor, coladvance, coladvance_force, gchar_cursor, get_cursor_pos_len,
    get_cursor_pos_ptr, getviscol, inc_cursor,
};
use crate::src::nvim::diff::nv_diffgetput;
use crate::src::nvim::drawscreen::win_cursorline_standout;
use crate::src::nvim::edit::{
    beginline, edit, get_literal, ins_copychar, prompt_curpos_editable, set_last_insert,
};
use crate::src::nvim::fold::{foldUpdateAfterInsert, hasFolding};
use crate::src::nvim::getchar::{AppendCharToRedobuff, AppendToRedobuff};
use crate::src::nvim::getchar::{stuff_empty, stuffReadbuff, stuffcharReadbuff, stuffnumReadbuff};
use crate::src::nvim::keycodes::{
    Ctrl_A, Ctrl_E, Ctrl_Q, Ctrl_V, Ctrl_Y, K_DEL, K_INS, K_KDEL, K_KINS,
};
use crate::src::nvim::main::{
    State, VIsual_active, VIsual_mode, cb_flags, curbuf, curwin, e_modifiable, got_int, msg_silent,
    p_sel, p_sta, p_to, p_ww, restart_edit,
};
use crate::src::nvim::mbyte::{mb_adjust_cursor, mb_charlen};
use crate::src::nvim::memline::{inc, ml_delete_flags, ml_get};
use crate::src::nvim::memory::xfree;
use crate::src::nvim::message::emsg;
use crate::src::nvim::normal::{
    BL_WHITE, CA_COMMAND_BUSY, CAR, DEL, ESC, FO_OPEN_COMS, ML_DEL_MESSAGE, ML_EMPTY, NL, NUL,
    OPENLINE_DO_COM, REPLACE_CR_NCHAR, REPLACE_NL_NCHAR, TAB, VALID_CROW, VIsual_mode_orig,
    checkclearop, checkclearopq, clearop, clearopbeep, false_0, nv_object, nv_operator, prep_redo,
    prep_redo_cmd, true_0, v_swap_corners, v_visop,
};
use crate::src::nvim::ops::{do_join, do_pending_operator, op_addsub, swapchar};
use crate::src::nvim::option::get_ve_flags;
use crate::src::nvim::options::{kOptCbFlagUnnamed, kOptCbFlagUnnamedplus, kOptVeFlagAll};
use crate::src::nvim::os::libc::{gettext, strlen};
use crate::src::nvim::pos::MAXCOL;
use crate::src::nvim::register::{copy_register, do_put, free_register};
use crate::src::nvim::search::{BACKWARD, FORWARD};
use crate::src::nvim::state::{MODE_INSERT, MODE_REPLACE, virtual_active};
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::textformat::{auto_format, has_format_option};
use crate::src::nvim::types::{
    OP_DELETE, OP_NOP, OP_NR_ADD, OP_NR_SUB, OP_TILDE, PUT_BLOCK_INNER, PUT_CURSEND, PUT_FIXINDENT,
    PUT_LINE, PUT_LINE_FORWARD, PUT_LINE_SPLIT, cmdarg_T, colnr_T, linenr_T, size_t, yankreg_T,
};
use crate::src::nvim::undo::{u_clearline, u_save, u_save_cursor, u_savesub};
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};

/// Refuse a change in a prompt buffer that is not on its own editable line.
unsafe fn prompt_refuses(cap: *mut cmdarg_T) -> bool {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if bt_prompt(curbuf.get()) && !prompt_curpos_editable() {
            clearopbeep((*cap).oap);
            return true;
        }
        false
    }
}

/// `CTRL-A` and `CTRL-X`: add to or subtract from the number under the cursor.
pub(crate) unsafe fn nv_addsub(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if prompt_refuses(cap) {
            return;
        }
        if !VIsual_active.get() && (*(*cap).oap).op_type == OP_NOP {
            // Not an operator: run it here and then put the operator back.
            prep_redo_cmd(cap);
            (*(*cap).oap).op_type = if (*cap).cmdchar == Ctrl_A {
                OP_NR_ADD
            } else {
                OP_NR_SUB
            };
            op_addsub((*cap).oap, (*cap).count1 as linenr_T, (*cap).arg != 0);
            (*(*cap).oap).op_type = OP_NOP;
        } else if VIsual_active.get() {
            nv_operator(cap);
        } else {
            clearop((*cap).oap);
        }
    }
}

/// `r`: replace `count1` characters with the one that follows.
pub(crate) unsafe fn nv_replace(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if checkclearop((*cap).oap) || prompt_refuses(cap) {
            return;
        }
        // `r CTRL-V` reads the next key literally. Only a byte-sized answer
        // still counts as literal: a larger one came from a digraph or a
        // `<C-u>` escape and behaves as an ordinary character.
        let mut literal = NUL;
        if (*cap).nchar == Ctrl_V || (*cap).nchar == Ctrl_Q {
            literal = Ctrl_V;
            (*cap).nchar = get_literal(false);
            if (*cap).nchar > DEL {
                literal = NUL;
            }
        }
        if (*cap).nchar < 0 {
            clearopbeep((*cap).oap);
            return;
        }
        if VIsual_active.get() {
            // The selection form is an operator; the interrupt a long
            // selection may have raised is not this command's business.
            if got_int.get() {
                got_int.set(false);
            }
            if literal != NUL {
                // A literal line break has to survive being carried through
                // `cap` as a character.
                if (*cap).nchar == CAR {
                    (*cap).nchar = REPLACE_CR_NCHAR;
                } else if (*cap).nchar == NL {
                    (*cap).nchar = REPLACE_NL_NCHAR;
                }
            }
            nv_operator(cap);
            return;
        }
        if virtual_active(curwin.get()) {
            if u_save_cursor() == false_0 {
                return;
            }
            if gchar_cursor() == NUL {
                // Past the end of the line: make room for the whole count and
                // then step back to where the replacing starts.
                coladvance_force(getviscol() + (*cap).count1);
                (*curwin.get()).w_cursor.col -= (*cap).count1;
            } else if gchar_cursor() == TAB {
                // Land on the tab's first cell, not the cell of it the cursor
                // happens to be showing on.
                coladvance_force(getviscol());
            }
        }
        // There have to be `count1` characters left on the line, counted both
        // ways: the byte length rules out a short line cheaply.
        if (get_cursor_pos_len() as size_t) < (*cap).count1 as c_uint as size_t
            || mb_charlen(get_cursor_pos_ptr()) < (*cap).count1
        {
            clearopbeep((*cap).oap);
            return;
        }
        // A tab that 'expandtab' or 'smarttab' would turn into spaces is
        // easier to get right by replaying the whole thing as `R<Tab><Esc>`.
        if literal != Ctrl_V
            && (*cap).nchar == '\t' as c_int
            && ((*curbuf.get()).b_p_et != 0 || p_sta.get() != 0)
        {
            stuffnumReadbuff((*cap).count1);
            stuffcharReadbuff('R' as c_int);
            stuffcharReadbuff('\t' as c_int);
            stuffcharReadbuff(ESC);
            return;
        }
        if u_save_cursor() == false_0 {
            return;
        }
        if literal != Ctrl_V && ((*cap).nchar == '\r' as c_int || (*cap).nchar == '\n' as c_int) {
            // Replacing with a line break splits the line, which is an insert.
            del_chars((*cap).count1, false_0);
            stuffcharReadbuff('\r' as c_int);
            stuffcharReadbuff(ESC);
            invoke_edit(cap, true_0, 'r' as c_int, false_0);
            foldUpdateAfterInsert();
            return;
        }

        prep_redo(
            (*(*cap).oap).regname,
            (*cap).count1,
            NUL,
            'r' as c_int,
            NUL,
            literal,
            0,
        );
        (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor;
        let old_state = State.get();
        if (*cap).nchar_len > 0 {
            AppendToRedobuff(&raw mut (*cap).nchar_composing as *mut c_char);
        } else {
            AppendCharToRedobuff((*cap).nchar);
        }
        for _ in 0..(*cap).count1 {
            // `ins_char` looks at 'State' to decide it is overwriting rather
            // than inserting.
            State.set(MODE_REPLACE);
            if (*cap).nchar == Ctrl_E || (*cap).nchar == Ctrl_Y {
                // `r CTRL-E` and `r CTRL-Y` copy from the line below or above.
                let from =
                    (*curwin.get()).w_cursor.lnum + if (*cap).nchar == Ctrl_Y { -1 } else { 1 };
                let c = ins_copychar(from);
                if c != NUL {
                    ins_char(c);
                } else {
                    // Nothing there to copy: leave the character alone and
                    // step over it.
                    (*curwin.get()).w_cursor.col += 1;
                }
            } else if (*cap).nchar_len != 0 {
                ins_char_bytes(
                    &raw mut (*cap).nchar_composing as *mut c_char,
                    (*cap).nchar_len as size_t,
                );
            } else {
                ins_char((*cap).nchar);
            }
            State.set(old_state);
        }
        (*curwin.get()).w_cursor.col -= 1;
        mb_adjust_cursor();
        (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
        (*curwin.get()).w_set_curswant = true_0;
        set_last_insert((*cap).nchar);
        foldUpdateAfterInsert();
    }
}

/// `R` and `gR`: replace mode, virtual with the argument set.
pub(crate) unsafe fn nv_Replace(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if VIsual_active.get() {
            // A selection is replaced linewise, which is `c` over whole lines.
            (*cap).cmdchar = 'c' as c_int;
            (*cap).nchar = NUL;
            VIsual_mode_orig.set(VIsual_mode.get());
            VIsual_mode.set('V' as c_int);
            nv_operator(cap);
            return;
        }
        if checkclearopq((*cap).oap) {
            return;
        }
        if (*curbuf.get()).b_p_ma == 0 {
            emsg(gettext(&raw const e_modifiable as *const c_char));
            return;
        }
        if virtual_active(curwin.get()) {
            coladvance(curwin.get(), getviscol());
        }
        invoke_edit(
            cap,
            false_0,
            if (*cap).arg != 0 {
                'V' as c_int
            } else {
                'R' as c_int
            },
            false_0,
        );
    }
}

/// `gr`: replace one character virtually -- the following text does not move.
pub(crate) unsafe fn nv_vreplace(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if VIsual_active.get() {
            (*cap).cmdchar = 'r' as c_int;
            (*cap).nchar = (*cap).extra_char;
            nv_replace(cap);
            return;
        }
        if checkclearopq((*cap).oap) {
            return;
        }
        if (*curbuf.get()).b_p_ma == 0 {
            emsg(gettext(&raw const e_modifiable as *const c_char));
            return;
        }
        if (*cap).extra_char == Ctrl_V || (*cap).extra_char == Ctrl_Q {
            (*cap).extra_char = get_literal(false);
        }
        // Replay the character through virtual replace mode. A control
        // character needs its own CTRL-V to survive the replay.
        if (*cap).extra_char < ' ' as c_int {
            stuffcharReadbuff(Ctrl_V);
        }
        stuffcharReadbuff((*cap).extra_char);
        stuffcharReadbuff(ESC);
        if virtual_active(curwin.get()) {
            coladvance(curwin.get(), getviscol());
        }
        invoke_edit(cap, true_0, 'v' as c_int, false_0);
    }
}

/// `~` when 'tildeop' is off: swap the case of `count1` characters.
pub(crate) unsafe fn n_swapchar(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if checkclearopq((*cap).oap) {
            return;
        }
        // An empty line has nothing to swap unless 'whichwrap' lets `~` move
        // to the next one.
        let wraps = !vim_strchr(p_ww.get(), '~' as c_int).is_null();
        if *ml_get((*curwin.get()).w_cursor.lnum) as c_int == NUL && !wraps {
            clearopbeep((*cap).oap);
            return;
        }
        prep_redo_cmd(cap);
        if u_save_cursor() == false_0 {
            return;
        }
        let startpos = (*curwin.get()).w_cursor;
        let mut did_change = false;
        let mut n = (*cap).count1;
        while n > 0 {
            did_change |= swapchar((*(*cap).oap).op_type, &raw mut (*curwin.get()).w_cursor);
            inc_cursor();
            if gchar_cursor() == NUL {
                if !(wraps && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count) {
                    break;
                }
                (*curwin.get()).w_cursor.lnum += 1;
                (*curwin.get()).w_cursor.col = 0;
                // Each further line needs its own undo entry.
                if n > 1 {
                    if u_savesub((*curwin.get()).w_cursor.lnum) == false_0 {
                        break;
                    }
                    u_clearline(curbuf.get());
                }
            }
            n -= 1;
        }
        check_cursor(curwin.get());
        (*curwin.get()).w_set_curswant = true_0;
        if did_change {
            changed_lines(
                curbuf.get(),
                startpos.lnum,
                startpos.col,
                (*curwin.get()).w_cursor.lnum + 1,
                0,
                true,
            );
            (*curbuf.get()).b_op_start = startpos;
            (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
            if (*curbuf.get()).b_op_end.col > 0 {
                (*curbuf.get()).b_op_end.col -= 1;
            }
        }
    }
}

/// `s` and `S`: substitute a character or a line.
pub(crate) unsafe fn nv_subst(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if prompt_refuses(cap) {
            return;
        }
        if VIsual_active.get() {
            if (*cap).cmdchar == 'S' as c_int {
                // `S` on a selection is linewise however the selection was
                // made; the original kind is remembered for the redo.
                VIsual_mode_orig.set(VIsual_mode.get());
                VIsual_mode.set('V' as c_int);
            }
            (*cap).cmdchar = 'c' as c_int;
            nv_operator(cap);
        } else {
            nv_optrans(cap);
        }
    }
}

/// `x`, `X`, `D`, `C`, `Y`: the one-key spellings of an operator and a motion.
pub(crate) unsafe fn nv_abbrev(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if (*cap).cmdchar == K_DEL || (*cap).cmdchar == K_KDEL {
            (*cap).cmdchar = 'x' as c_int;
        }
        if VIsual_active.get() {
            v_visop(cap);
        } else {
            nv_optrans(cap);
        }
    }
}

/// Replay a one-key command as the operator and motion it stands for.
pub(crate) unsafe fn nv_optrans(cap: *mut cmdarg_T) {
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
    unsafe {
        if !checkclearopq((*cap).oap) {
            if (*cap).count0 != 0 {
                stuffnumReadbuff((*cap).count0);
            }
            for (key, keys) in TRANSLATIONS {
                if key == (*cap).cmdchar {
                    stuffReadbuff(keys.as_ptr());
                    break;
                }
            }
        }
        // The count went into the replayed keys, so it must not also apply to
        // whatever they turn out to be.
        (*cap).opcount = 0;
    }
}

/// `o` and `O`: open a line below or above and start inserting on it.
pub(crate) unsafe fn n_opencmd(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if checkclearopq((*cap).oap) {
            return;
        }
        let win = curwin.get();
        let opening_above = (*cap).cmdchar == 'O' as c_int;
        // Open outside a closed fold rather than inside it.
        if opening_above {
            hasFolding(
                win,
                (*win).w_cursor.lnum,
                &raw mut (*win).w_cursor.lnum,
                ptr::null_mut(),
            );
        } else {
            hasFolding(
                win,
                (*win).w_cursor.lnum,
                ptr::null_mut(),
                &raw mut (*win).w_cursor.lnum,
            );
        }
        (*curbuf.get()).b_last_changedtick_i = buf_get_changedtick(curbuf.get());
        let undo_first = (*win).w_cursor.lnum - linenr_T::from(opening_above);
        let undo_last = (*win).w_cursor.lnum + linenr_T::from(!opening_above);
        let opened = u_save(undo_first, undo_last) != 0
            && open_line(
                if opening_above {
                    BACKWARD as c_int
                } else {
                    FORWARD as c_int
                },
                if has_format_option(FO_OPEN_COMS) {
                    OPENLINE_DO_COM as c_int
                } else {
                    0
                },
                0,
                ptr::null_mut(),
            );
        if opened {
            if win_cursorline_standout(win) {
                // The cursor line moved, so its highlight has to be redrawn.
                (*win).w_valid &= !VALID_CROW;
            }
            invoke_edit(cap, false_0, (*cap).cmdchar, true_0);
        }
    }
}

/// `~`: swap case, or the `g~` operator when 'tildeop' is on.
pub(crate) unsafe fn nv_tilde(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if p_to.get() == 0 && !VIsual_active.get() && (*(*cap).oap).op_type != OP_TILDE {
            if prompt_refuses(cap) {
                return;
            }
            n_swapchar(cap);
        } else {
            nv_operator(cap);
        }
    }
}

/// Put the cursor where `A` starts inserting: past the last character, or
/// past the last *cell* when 'virtualedit' is "all".
pub unsafe fn set_cursor_for_append_to_line() {
    // SAFETY: reads and writes the current window's cursor.
    unsafe {
        (*curwin.get()).w_set_curswant = true_0;
        if get_ve_flags(curwin.get()) == kOptVeFlagAll as c_uint {
            // Insert mode is what makes `coladvance` allow the position one
            // past the end.
            let save_state = State.get();
            State.set(MODE_INSERT);
            coladvance(curwin.get(), MAXCOL as c_int);
            State.set(save_state);
        } else {
            (*curwin.get()).w_cursor.col += strlen(get_cursor_pos_ptr()) as colnr_T;
        }
    }
}

/// `a`, `A`, `i` and `I`: enter insert mode.
pub(crate) unsafe fn nv_edit(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if (*cap).cmdchar == K_INS || (*cap).cmdchar == K_KINS {
            (*cap).cmdchar = 'i' as c_int;
        }
        // With a selection up, `A` and `I` insert at every line's end or
        // start; `a` and `i` name a text object instead.
        if VIsual_active.get() && ((*cap).cmdchar == 'A' as c_int || (*cap).cmdchar == 'I' as c_int)
        {
            v_visop(cap);
            return;
        }
        if ((*cap).cmdchar == 'a' as c_int || (*cap).cmdchar == 'i' as c_int)
            && ((*(*cap).oap).op_type != OP_NOP || VIsual_active.get())
        {
            nv_object(cap);
            return;
        }
        // A terminal buffer is not 'modifiable' and is still editable.
        if (*curbuf.get()).b_p_ma == 0 && (*curbuf.get()).terminal.is_null() {
            emsg(gettext(&raw const e_modifiable as *const c_char));
            clearop((*cap).oap);
            return;
        }
        if checkclearopq((*cap).oap) {
            return;
        }
        match u8::try_from((*cap).cmdchar) {
            Ok(b'A') => set_cursor_for_append_to_line(),
            Ok(b'I') => beginline(BL_WHITE as c_int),
            Ok(b'a') => {
                // `a` steps one right first. Under 'virtualedit' a position
                // inside a tab or past the end of the line moves by a cell.
                if virtual_active(curwin.get())
                    && ((*curwin.get()).w_cursor.coladd > 0
                        || *get_cursor_pos_ptr() as c_int == NUL
                        || *get_cursor_pos_ptr() as c_int == TAB)
                {
                    (*curwin.get()).w_cursor.coladd += 1;
                } else if *get_cursor_pos_ptr() as c_int != NUL {
                    inc_cursor();
                }
            }
            _ => {}
        }
        // Insert mode has no virtual column of its own, so anything but `A`
        // has to land on a real one first.
        if (*curwin.get()).w_cursor.coladd != 0 && (*cap).cmdchar != 'A' as c_int {
            let save_state = State.get();
            State.set(MODE_INSERT);
            coladvance(curwin.get(), getviscol());
            State.set(save_state);
        }
        invoke_edit(cap, false_0, (*cap).cmdchar, false_0);
    }
}

/// Enter insert mode and report back whether the command loop should treat
/// this command as still running.
///
/// 'restart_edit' is put back afterwards only if insert mode did not set one
/// itself: whatever it asked for wins over what was pending before.
pub(crate) unsafe fn invoke_edit(cap: *mut cmdarg_T, repl: c_int, cmd: c_int, startln: c_int) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        // A replay or leftover typeahead is allowed to resume a pending
        // insert; a fresh command is not.
        let restart_edit_save = if repl != 0 || !stuff_empty() {
            restart_edit.get()
        } else {
            0
        };
        restart_edit.set(0);
        // `o` and `O` already recorded the tick before opening the line.
        if (*cap).cmdchar != 'O' as c_int && (*cap).cmdchar != 'o' as c_int {
            (*curbuf.get()).b_last_changedtick_i = buf_get_changedtick(curbuf.get());
        }
        if edit(cmd, startln != 0, (*cap).count1) {
            (*cap).retval |= CA_COMMAND_BUSY as c_int;
        }
        if restart_edit.get() == 0 {
            restart_edit.set(restart_edit_save);
        }
    }
}

/// `J`: join lines.
pub(crate) unsafe fn nv_join(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if VIsual_active.get() {
            nv_operator(cap);
            return;
        }
        if checkclearop((*cap).oap) {
            return;
        }
        // Joining fewer than two lines means nothing; `J` and `1J` both join
        // this line with the next.
        (*cap).count0 = (*cap).count0.max(2);
        if (*curwin.get()).w_cursor.lnum + (*cap).count0 as linenr_T - 1
            > (*curbuf.get()).b_ml.ml_line_count
        {
            // A count that runs off the end joins what is left -- unless there
            // was no count, in which case there is nothing below to join to.
            if (*cap).count0 <= 2 {
                clearopbeep((*cap).oap);
                return;
            }
            (*cap).count0 =
                ((*curbuf.get()).b_ml.ml_line_count - (*curwin.get()).w_cursor.lnum + 1) as c_int;
        }
        prep_redo(
            (*(*cap).oap).regname,
            (*cap).count0,
            NUL,
            (*cap).cmdchar,
            NUL,
            NUL,
            (*cap).nchar,
        );
        // `gJ` arrives with `nchar` set and does not insert or remove spaces.
        do_join(
            (*cap).count0 as size_t,
            (*cap).nchar == NUL,
            true,
            true,
            true,
        );
    }
}

/// `p` and `P`.
pub(crate) unsafe fn nv_put(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe { nv_put_opt(cap, false) };
}

/// The put commands. `fix_indent` is the `]p`/`[p` family, which reindents the
/// text to the current line.
pub(crate) unsafe fn nv_put_opt(cap: *mut cmdarg_T, fix_indent: bool) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let win = curwin.get();
        let save_fen = (*win).w_onebuf_opt.wo_fen;
        if (*(*cap).oap).op_type != OP_NOP {
            // `dp` is not "delete, then put": it is the diff command.
            if (*(*cap).oap).op_type == OP_DELETE && (*cap).cmdchar == 'p' as c_int {
                clearop((*cap).oap);
                debug_assert!((*cap).opcount >= 0);
                nv_diffgetput(true, (*cap).opcount as size_t);
            } else {
                clearopbeep((*cap).oap);
            }
            return;
        }
        if bt_prompt(curbuf.get()) && !prompt_curpos_editable() {
            // On the prompt's own line, put in front of the prompt text
            // rather than refusing.
            if (*win).w_cursor.lnum == (*curbuf.get()).b_prompt_start.mark.lnum {
                (*win).w_cursor.col = (*curbuf.get()).b_prompt_start.mark.col;
                (*cap).cmdchar = 'P' as c_int;
            } else {
                clearopbeep((*cap).oap);
                return;
            }
        }

        let mut flags = 0;
        let mut dir;
        if fix_indent {
            dir = if (*cap).cmdchar == ']' as c_int && (*cap).nchar == 'p' as c_int {
                FORWARD as c_int
            } else {
                BACKWARD as c_int
            };
            flags |= PUT_FIXINDENT as c_int;
        } else {
            dir = if (*cap).cmdchar == 'P' as c_int
                || (((*cap).cmdchar == 'g' as c_int || (*cap).cmdchar == 'z' as c_int)
                    && (*cap).nchar == 'P' as c_int)
            {
                BACKWARD as c_int
            } else {
                FORWARD as c_int
            };
        }
        prep_redo_cmd(cap);
        // `gp` leaves the cursor after the new text; `zp` puts a blockwise
        // register without widening the lines it lands on.
        if (*cap).cmdchar == 'g' as c_int {
            flags |= PUT_CURSEND as c_int;
        } else if (*cap).cmdchar == 'z' as c_int {
            flags |= PUT_BLOCK_INNER as c_int;
        }

        let was_visual = VIsual_active.get();
        let mut savereg: *mut yankreg_T = ptr::null_mut();
        let mut emptied = false;
        if was_visual {
            let regname = (*(*cap).oap).regname;
            let keep_registers = (*cap).cmdchar == 'P' as c_int;
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
                savereg = copy_register(regname);
            }
            // The delete must not close or open folds under the selection.
            (*win).w_onebuf_opt.wo_fen = false_0;
            // The condition is upstream's; only the `.` register on a
            // charwise selection skips the delete.
            if !VIsual_active.get() || VIsual_mode.get() == 'V' as c_int || regname != '.' as c_int
            {
                (*cap).cmdchar = 'd' as c_int;
                (*cap).nchar = NUL;
                (*(*cap).oap).regname = if keep_registers { '_' as c_int } else { NUL };
                (*msg_silent.ptr()) += 1;
                nv_operator(cap);
                do_pending_operator(cap, 0, false);
                // The delete may have left the buffer with one empty line
                // that the put should not keep.
                emptied = (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0;
                (*msg_silent.ptr()) -= 1;
                (*(*cap).oap).regname = regname;
            }
            if VIsual_mode.get() == 'V' as c_int {
                flags |= PUT_LINE as c_int;
            } else if VIsual_mode.get() == 'v' as c_int {
                flags |= PUT_LINE_SPLIT as c_int;
            }
            if VIsual_mode.get() == Ctrl_V && dir == FORWARD as c_int {
                flags |= PUT_LINE_FORWARD as c_int;
            }
            // Put where the selection was, which is where the delete left the
            // cursor -- forwards only when it left it before the start.
            dir = BACKWARD as c_int;
            if (VIsual_mode.get() != 'V' as c_int
                && (*win).w_cursor.col < (*curbuf.get()).b_op_start.col)
                || (VIsual_mode.get() == 'V' as c_int
                    && (*win).w_cursor.lnum < (*curbuf.get()).b_op_start.lnum)
            {
                dir = FORWARD as c_int;
            }
            VIsual_active.set(true);
        }

        do_put((*(*cap).oap).regname, savereg, dir, (*cap).count1, flags);
        if !savereg.is_null() {
            free_register(savereg);
            xfree(savereg as *mut c_void);
        }
        if was_visual {
            if save_fen != 0 {
                (*win).w_onebuf_opt.wo_fen = true_0;
            }
            // Leave `gv` naming what was just put.
            (*curbuf.get()).b_visual.vi_start = (*curbuf.get()).b_op_start;
            (*curbuf.get()).b_visual.vi_end = (*curbuf.get()).b_op_end;
            if *p_sel.get() as c_int == 'e' as c_int {
                inc(&raw mut (*curbuf.get()).b_visual.vi_end);
            }
        }
        if emptied && *ml_get((*curbuf.get()).b_ml.ml_line_count) as c_int == NUL {
            ml_delete_flags((*curbuf.get()).b_ml.ml_line_count, ML_DEL_MESSAGE as c_int);
            deleted_lines((*curbuf.get()).b_ml.ml_line_count + 1, 1);
            if (*win).w_cursor.lnum > (*curbuf.get()).b_ml.ml_line_count {
                (*win).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
                coladvance(win, MAXCOL as c_int);
            }
        }
        auto_format(false, true);
    }
}

/// `o` and `O` -- or, with a pending delete, the diff command, and with a
/// selection, "swap to the other corner".
pub(crate) unsafe fn nv_open(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if (*(*cap).oap).op_type == OP_DELETE && (*cap).cmdchar == 'o' as c_int {
            // `do` is `:diffget`, not "delete, then open".
            clearop((*cap).oap);
            debug_assert!((*cap).opcount >= 0);
            nv_diffgetput(false, (*cap).opcount as size_t);
        } else if VIsual_active.get() {
            v_swap_corners((*cap).cmdchar);
        } else if bt_prompt(curbuf.get())
            && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_prompt_start.mark.lnum
        {
            clearopbeep((*cap).oap);
        } else {
            n_opencmd(cap);
        }
    }
}
