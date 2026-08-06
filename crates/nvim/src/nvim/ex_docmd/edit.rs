//! Commands that change the buffer text or the cursor, including
//! `:normal`, which re-enters the normal-mode state machine.
#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::charset::getdigits_int;
use crate::src::nvim::cursor::{check_cursor, check_cursor_col};
use crate::src::nvim::drawscreen::{
    UPD_VALID, clearmode, redraw_later, setcursor_mayforce, showmode,
};
use crate::src::nvim::edit::beginline;
use crate::src::nvim::event::r#loop::process_events_until;
use crate::src::nvim::ex_cmds::{
    do_move, ex_copy, ex_substitute, ex_substitute_preview, global_exe, print_line,
};
use crate::src::nvim::ex_docmd::address::get_address;
use crate::src::nvim::ex_docmd::cmdline::do_cmdline;
use crate::src::nvim::ex_docmd::modifier::expr_map_locked;
use crate::src::nvim::ex_docmd::scan::{find_nextcmd, get_flags};
use crate::src::nvim::ex_docmd::{
    BL_FIX, BL_SOL, BL_WHITE, CPO_EXECBUF, DOCMD_NOWAIT, DOCMD_VERBOSE, EXFLAG_LIST, EXFLAG_NR,
    FAIL, KS_SPECIAL, ML_EMPTY, NUL, OP_DELETE, OP_LSHIFT, OP_RSHIFT, OP_YANK, OPTION_MAGIC_OFF,
    OPTION_MAGIC_ON, PUT_CURSLINE, PUT_FIXINDENT, PUT_LINE, REMAP_NONE, REMAP_YES, kMTLineWise,
};
use crate::src::nvim::ex_getln::getexline;
use crate::src::nvim::fold::{foldCreate, foldManualAllowed, hasFolding, opFoldRange};
use crate::src::nvim::getchar::{
    beep_flush, ins_typebuf, restore_typeahead, save_typeahead, stuff_empty, typebuf_typed, vpeekc,
};
use crate::src::nvim::keycodes::{Ctrl_C, Ctrl_O, K_SPECIAL, KE_FILLER};
use crate::src::nvim::lua::executor::ex_lua;
use crate::src::nvim::main::{
    State, VIsual_active, curbuf, curwin, did_syncbind, e_argreq, e_empty_buffer, e_invarg2,
    e_invrange, e_secure, e_trailing_arg, e_undobang_cannot_redo_or_move_branch, ex_no_reprint,
    ex_normal_busy, exec_from_reg, finish_op, firstwin, force_restart_edit, got_int,
    magic_overruled, main_loop, msg_didout, msg_scroll, opcount, p_cpo, p_mmd,
    pending_end_reg_executing, reg_executing, restart_edit, stop_insert_mode, typebuf, virtual_op,
};
use crate::src::nvim::mark::{checkpcmark, setmark, setpcmark};
use crate::src::nvim::mbyte::utfc_ptr2len;
use crate::src::nvim::memline::{goto_byte, ml_clearmarked, ml_setmarked};
use crate::src::nvim::memory::{xfree, xmalloc};
use crate::src::nvim::message::{emsg, semsg, smsg};
use crate::src::nvim::mouse::setmouse;
use crate::src::nvim::r#move::{
    check_cursor_moved, cursor_correct, cursor_valid, scrolldown, scrollup, update_curswant,
    update_topline, validate_cursor,
};
use crate::src::nvim::normal::{
    end_visual_mode, get_vtopline, normal_cmd, set_cursor_for_append_to_line,
};
use crate::src::nvim::ops::{clear_oparg, do_join, op_delete, op_shift};
use crate::src::nvim::option::get_scrolloff_value;
use crate::src::nvim::os::input::os_breakcheck;
use crate::src::nvim::os::libc::{gettext, strlen};
use crate::src::nvim::plines::plines_m_win_fill;
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::register::{do_execreg, do_put, op_yank};
use crate::src::nvim::search::{BACKWARD, FORWARD};
use crate::src::nvim::state::{MODE_INSERT, MODE_TERMINAL};
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{
    CMD_delete, CMD_earlier, CMD_folddoclosed, CMD_foldopen, CMD_list, CMD_move, CMD_number,
    CMD_pound, CMD_rshift, CMD_smagic, CMD_startinsert, CMD_startreplace, CMD_yank, colnr_T,
    exarg_T, handle_T, int64_t, kFalse, kNone, linenr_T, oparg_T, optmagic_T, pos_T, save_state_T,
    size_t, ssize_t, uint8_t,
};
use crate::src::nvim::ui::{ui_busy_start, ui_busy_stop, ui_cursor_shape, ui_flush};
use crate::src::nvim::undo::u_clearline;
use crate::src::nvim::undo::{u_redo, u_undo, u_undo_and_forget, undo_time};

/// `:print`, `:number` and `:list`.
pub(crate) unsafe fn ex_print(eap: *mut exarg_T) {
    unsafe {
        if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
            emsg(gettext(&raw const e_empty_buffer as *const c_char));
        } else {
            let idx = (*eap).cmdidx as c_int;
            let numbered = idx == CMD_number as c_int
                || idx == CMD_pound as c_int
                || (*eap).flags & EXFLAG_NR != 0;
            let listed = idx == CMD_list as c_int || (*eap).flags & EXFLAG_LIST != 0;
            let mut line = (*eap).line1;
            while line <= (*eap).line2 && !got_int.get() {
                print_line(line, numbered, listed, line == (*eap).line1);
                line += 1;
                os_breakcheck();
            }
            setpcmark();
            (*curwin.get()).w_cursor.lnum = (*eap).line2;
            beginline(BL_SOL as c_int | BL_FIX as c_int);
        }
        // Ex mode has just printed the line itself; it must not print it
        // again.
        ex_no_reprint.set(true);
    }
}

/// `:goto` — the range is a byte offset, not a line number.
pub(crate) unsafe fn ex_goto(eap: *mut exarg_T) {
    unsafe {
        goto_byte((*eap).line2 as c_int);
    }
}

/// `:syncbind` — line up every 'scrollbind' window at the same relative
/// position.
pub(crate) unsafe fn ex_syncbind(_eap: *mut exarg_T) {
    unsafe {
        let old_linenr = (*curwin.get()).w_cursor.lnum;
        setpcmark();

        // The topline to use is the smallest that every bound window can
        // reach: one of them may be shorter than the rest.
        let mut vtopline: linenr_T = 1;
        if (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
            vtopline = get_vtopline(curwin.get()) as linenr_T;
            let mut wp = firstwin.get();
            while !wp.is_null() {
                if (*wp).w_onebuf_opt.wo_scb != 0 && !(*wp).w_buffer.is_null() {
                    let limit = plines_m_win_fill(wp, 1, (*(*wp).w_buffer).b_ml.ml_line_count)
                        as linenr_T
                        - get_scrolloff_value(curwin.get()) as linenr_T;
                    vtopline = vtopline.min(limit);
                }
                wp = (*wp).w_next;
            }
            vtopline = vtopline.max(1);
        }

        let mut wp = firstwin.get();
        while !wp.is_null() {
            if (*wp).w_onebuf_opt.wo_scb != 0 {
                let y = vtopline as c_int - get_vtopline(wp);
                if y > 0 {
                    scrollup(wp, y as linenr_T, true);
                } else {
                    scrolldown(wp, -(y as linenr_T), 1);
                }
                (*wp).w_scbind_pos = vtopline as c_int;
                redraw_later(wp, UPD_VALID);
                cursor_correct(wp);
                (*wp).w_redr_status = true;
            }
            wp = (*wp).w_next;
        }

        if (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
            did_syncbind.set(true);
            checkpcmark();
            // The cursor moved with the scroll; CTRL-O puts it back.
            if old_linenr != (*curwin.get()).w_cursor.lnum {
                let ctrl_o: [c_char; 2] = [Ctrl_O as c_char, 0];
                ins_typebuf(
                    ctrl_o.as_ptr() as *mut c_char,
                    REMAP_NONE as c_int,
                    0,
                    true,
                    false,
                );
            }
        }
    }
}

/// `:=` — the line number, unless something follows it, in which case it
/// is `:lua`'s alias.
pub(crate) unsafe fn ex_equal(eap: *mut exarg_T) {
    unsafe {
        if *(*eap).arg as c_int != NUL && *(*eap).arg as c_int != '|' as c_int {
            ex_lua(eap);
        } else {
            (*eap).nextcmd = find_nextcmd((*eap).arg);
            smsg(0, c"%ld".as_ptr(), (*eap).line2 as int64_t);
        }
    }
}

/// `:sleep` — the count is in seconds unless it is followed by `m`.
pub(crate) unsafe fn ex_sleep(eap: *mut exarg_T) {
    unsafe {
        if cursor_valid(curwin.get()) != 0 {
            setcursor_mayforce(curwin.get(), true);
        }
        let mut len = (*eap).line2 as int64_t;
        match *(*eap).arg as c_int {
            c if c == 'm' as c_int => {}
            c if c == NUL => len *= 1000,
            _ => {
                semsg(gettext(&raw const e_invarg2 as *const c_char), (*eap).arg);
                return;
            }
        }
        do_sleep(len, (*eap).forceit != 0);
    }
}

/// Wait `msec` milliseconds, still serving events, and stop early on an
/// interrupt.
pub unsafe fn do_sleep(msec: int64_t, hide_cursor: bool) {
    unsafe {
        if hide_cursor {
            ui_busy_start();
        }
        ui_flush();
        process_events_until(main_loop.ptr(), (*main_loop.ptr()).events, msec, || {
            got_int.get()
        });
        if got_int.get() {
            // Take the interrupt out of the typeahead.
            vpeekc();
        }
        if hide_cursor {
            ui_busy_stop();
        }
    }
}

/// `:delete`, `:yank`, `:<` and `:>` — the four normal-mode operators that
/// have an Ex spelling.
pub(crate) unsafe fn ex_operators(eap: *mut exarg_T) {
    unsafe {
        let mut oa: oparg_T = core::mem::zeroed();
        clear_oparg(&raw mut oa);
        oa.regname = (*eap).regname;
        oa.start.lnum = (*eap).line1;
        oa.end.lnum = (*eap).line2;
        oa.line_count = (*eap).line2 - (*eap).line1 + 1;
        oa.motion_type = kMTLineWise;
        // An Ex range is whole lines, so 'virtualedit' must not apply.
        virtual_op.set(kFalse);

        // `:yank` does not move the cursor, so it does not set the previous
        // context mark either.
        if (*eap).cmdidx as c_int != CMD_yank as c_int {
            setpcmark();
            (*curwin.get()).w_cursor.lnum = (*eap).line1;
            beginline(BL_SOL as c_int | BL_FIX as c_int);
        }
        if VIsual_active.get() {
            end_visual_mode();
        }

        match (*eap).cmdidx as c_int {
            c if c == CMD_delete as c_int => {
                oa.op_type = OP_DELETE as c_int;
                op_delete(&raw mut oa);
            }
            c if c == CMD_yank as c_int => {
                oa.op_type = OP_YANK as c_int;
                op_yank(&raw mut oa, true);
            }
            _ => {
                // In a 'rightleft' window the two shift commands swap.
                oa.op_type = if ((*eap).cmdidx as c_int == CMD_rshift as c_int) as c_int
                    ^ (*curwin.get()).w_onebuf_opt.wo_rl
                    != 0
                {
                    OP_RSHIFT as c_int
                } else {
                    OP_LSHIFT as c_int
                };
                op_shift(&raw mut oa, false, (*eap).amount);
            }
        }
        virtual_op.set(kNone);
        ex_may_print(eap);
    }
}

/// `:put`.
pub(crate) unsafe fn ex_put(eap: *mut exarg_T) {
    unsafe {
        put_lines(eap, PUT_LINE as c_int | PUT_CURSLINE as c_int);
    }
}

/// `:iput` — the same, re-indenting what is put.
pub(crate) unsafe fn ex_iput(eap: *mut exarg_T) {
    unsafe {
        put_lines(
            eap,
            PUT_LINE as c_int | PUT_CURSLINE as c_int | PUT_FIXINDENT as c_int,
        );
    }
}

/// `:0put` puts *above* line 1, which is spelled as a forced put at line 1.
unsafe fn put_lines(eap: *mut exarg_T, flags: c_int) {
    unsafe {
        if (*eap).line2 == 0 {
            (*eap).line2 = 1;
            (*eap).forceit = 1;
        }
        (*curwin.get()).w_cursor.lnum = (*eap).line2;
        check_cursor_col(curwin.get());
        do_put(
            (*eap).regname,
            ptr::null_mut(),
            if (*eap).forceit != 0 {
                BACKWARD as c_int
            } else {
                FORWARD as c_int
            },
            1,
            flags,
        );
    }
}

/// `:copy` and `:move` — both take a destination address after the
/// command, which is why they parse one more address here.
pub(crate) unsafe fn ex_copymove(eap: *mut exarg_T) {
    unsafe {
        let mut errormsg: *const c_char = ptr::null();
        let n = get_address(
            eap,
            &raw mut (*eap).arg,
            (*eap).addr_type,
            false,
            false,
            0,
            1,
            &raw mut errormsg,
        );
        if (*eap).arg.is_null() {
            if !errormsg.is_null() {
                emsg(errormsg);
            }
            (*eap).nextcmd = ptr::null_mut();
            return;
        }
        get_flags(eap);

        // `MAXLNUM` is what `get_address` answers for "no address at all".
        if n == MAXLNUM as linenr_T || n < 0 || n > (*curbuf.get()).b_ml.ml_line_count {
            emsg(gettext(&raw const e_invrange as *const c_char));
            return;
        }

        if (*eap).cmdidx as c_int == CMD_move as c_int {
            if do_move((*eap).line1, (*eap).line2, n) == FAIL {
                return;
            }
        } else {
            ex_copy((*eap).line1, (*eap).line2, n);
        }
        u_clearline(curbuf.get());
        beginline(BL_SOL as c_int | BL_FIX as c_int);
        ex_may_print(eap);
    }
}

/// Print the current line, if the command carried an `l`, `p` or `#` flag.
pub unsafe fn ex_may_print(eap: *mut exarg_T) {
    unsafe {
        if (*eap).flags != 0 {
            print_line(
                (*curwin.get()).w_cursor.lnum,
                (*eap).flags & EXFLAG_NR != 0,
                (*eap).flags & EXFLAG_LIST != 0,
                true,
            );
            ex_no_reprint.set(true);
        }
    }
}

/// `:smagic` and `:snomagic` — `:substitute` with 'magic' forced either
/// way for the duration.
pub(crate) unsafe fn ex_submagic(eap: *mut exarg_T) {
    unsafe {
        let saved = force_magic(eap);
        ex_substitute(eap);
        magic_overruled.set(saved);
    }
}

/// The 'inccommand' preview of the same.
pub(crate) unsafe fn ex_submagic_preview(
    eap: *mut exarg_T,
    cmdpreview_ns: c_int,
    cmdpreview_bufnr: handle_T,
) -> c_int {
    unsafe {
        let saved = force_magic(eap);
        let retv = ex_substitute_preview(eap, cmdpreview_ns, cmdpreview_bufnr);
        magic_overruled.set(saved);
        retv
    }
}

/// Override 'magic' for this command, answering what it was.
unsafe fn force_magic(eap: *mut exarg_T) -> optmagic_T {
    unsafe {
        let saved = magic_overruled.get();
        magic_overruled.set(if (*eap).cmdidx as c_int == CMD_smagic as c_int {
            OPTION_MAGIC_ON
        } else {
            OPTION_MAGIC_OFF
        } as optmagic_T);
        saved
    }
}

/// `:join`.
pub(crate) unsafe fn ex_join(eap: *mut exarg_T) {
    unsafe {
        (*curwin.get()).w_cursor.lnum = (*eap).line1;
        if (*eap).line1 == (*eap).line2 {
            // One line: join it with the next, unless a two-address range
            // said exactly one line, or there is no next line.
            if (*eap).addr_count >= 2 {
                return;
            }
            if (*eap).line2 == (*curbuf.get()).b_ml.ml_line_count {
                beep_flush();
                return;
            }
            (*eap).line2 += 1;
        }
        do_join(
            ((*eap).line2 as ssize_t - (*eap).line1 as ssize_t + 1) as size_t,
            (*eap).forceit == 0,
            true,
            true,
            true,
        );
        beginline(BL_WHITE as c_int | BL_FIX as c_int);
        ex_may_print(eap);
    }
}

/// `:@` — run the contents of a register as Ex commands.
///
/// The register's text goes into the typeahead, and command lines are read
/// out of it until it is empty. `prev_len` is what tells "empty" from
/// "there was already typeahead before this".
pub(crate) unsafe fn ex_at(eap: *mut exarg_T) {
    unsafe {
        let prev_len = (*typebuf.ptr()).tb_len;
        (*curwin.get()).w_cursor.lnum = (*eap).line2;
        check_cursor_col(curwin.get());

        let mut c = *(*eap).arg as uint8_t as c_int;
        if c == NUL {
            c = '@' as c_int;
        }
        // 'cpoptions' `e` makes `:@` run the register's last line
        // immediately rather than leaving it on the command line.
        if do_execreg(
            c,
            1,
            !vim_strchr(p_cpo.get(), CPO_EXECBUF).is_null() as c_int,
            1,
        ) == FAIL
        {
            beep_flush();
            return;
        }
        let save_efr = exec_from_reg.get();
        exec_from_reg.set(true);
        while !stuff_empty() || (*typebuf.ptr()).tb_len > prev_len {
            do_cmdline(
                ptr::null_mut(),
                Some(getexline),
                ptr::null_mut(),
                DOCMD_NOWAIT as c_int | DOCMD_VERBOSE as c_int,
            );
        }
        exec_from_reg.set(save_efr);
    }
}

/// `:undo`, and `:undo N` which goes to a numbered state.
///
/// `:undo! N` is different again: it *forgets* the states between here and
/// N rather than moving to it, so it can only go backwards along the
/// current branch.
pub(crate) unsafe fn ex_undo(eap: *mut exarg_T) {
    unsafe {
        if (*eap).addr_count != 1 {
            if (*eap).forceit != 0 {
                u_undo_and_forget(1, true);
            } else {
                u_undo(1);
            }
            return;
        }
        let step = (*eap).line2;
        if (*eap).forceit == 0 {
            undo_time(step as c_int, false, false, true);
            return;
        }

        if step >= (*curbuf.get()).b_u_seq_cur as linenr_T {
            emsg(gettext(
                &raw const e_undobang_cannot_redo_or_move_branch as *const c_char,
            ));
            return;
        }
        // Count how many states back `step` is along this branch.
        let mut uhp = if (*curbuf.get()).b_u_curhead.is_null() {
            (*curbuf.get()).b_u_newhead
        } else {
            (*curbuf.get()).b_u_curhead
        };
        let mut count = 0;
        while !uhp.is_null() && (*uhp).uh_seq as linenr_T > step {
            uhp = (*uhp).uh_next.ptr;
            count += 1;
        }
        // Running past it, or off the end, means `step` is on another
        // branch. Sequence 0 is the state before any change and is always
        // reachable.
        if step != 0 && (uhp.is_null() || ((*uhp).uh_seq as linenr_T) < step) {
            emsg(gettext(
                &raw const e_undobang_cannot_redo_or_move_branch as *const c_char,
            ));
            return;
        }
        u_undo_and_forget(count, true);
    }
}

/// `:redo`.
pub(crate) unsafe fn ex_redo(_eap: *mut exarg_T) {
    unsafe {
        u_redo(1);
    }
}

/// `:earlier` and `:later` — a count of changes, of seconds (`s`, `m`,
/// `h`, `d`) or of file writes (`f`).
pub(crate) unsafe fn ex_later(eap: *mut exarg_T) {
    unsafe {
        let mut count = 0;
        let mut sec = false;
        let mut file = false;
        let mut p = (*eap).arg;
        if *p as c_int == NUL {
            count = 1;
        } else if ascii_isdigit(*p as uint8_t as c_int) {
            count = getdigits_int(&raw mut p, false, 0);
            match *p as u8 {
                b's' => {
                    p = p.add(1);
                    sec = true;
                }
                // The three multiplications are `int` arithmetic on a
                // number the user typed, and the C wraps: `:later
                // 100000000d` is nine orders of magnitude past `INT_MAX`.
                // `undo_time` clamps whatever comes out.
                b'm' => {
                    p = p.add(1);
                    sec = true;
                    count = count.wrapping_mul(60);
                }
                b'h' => {
                    p = p.add(1);
                    sec = true;
                    count = count.wrapping_mul(60 * 60);
                }
                b'd' => {
                    p = p.add(1);
                    sec = true;
                    count = count.wrapping_mul(24 * 60 * 60);
                }
                b'f' => {
                    p = p.add(1);
                    file = true;
                }
                _ => {}
            }
        }
        if *p as c_int != NUL {
            semsg(gettext(&raw const e_invarg2 as *const c_char), (*eap).arg);
            return;
        }
        undo_time(
            if (*eap).cmdidx as c_int == CMD_earlier as c_int {
                count.wrapping_neg()
            } else {
                count
            },
            sec,
            file,
            false,
        );
    }
}

/// `:mark` and `:k`.
pub(crate) unsafe fn ex_mark(eap: *mut exarg_T) {
    unsafe {
        if *(*eap).arg as c_int == NUL {
            emsg(gettext(&raw const e_argreq as *const c_char));
            return;
        }
        if *(*eap).arg.add(1) as c_int != NUL {
            semsg(
                gettext(&raw const e_trailing_arg as *const c_char),
                (*eap).arg,
            );
            return;
        }
        // The mark is set at the first non-blank of the addressed line, so
        // the cursor goes there and comes back.
        let pos = (*curwin.get()).w_cursor;
        (*curwin.get()).w_cursor.lnum = (*eap).line2;
        beginline(BL_WHITE as c_int | BL_FIX as c_int);
        if setmark(*(*eap).arg as c_int) == FAIL {
            emsg(gettext(
                c"E191: Argument must be a letter or forward/backward quote".as_ptr(),
            ));
        }
        (*curwin.get()).w_cursor = pos;
    }
}

/// Put the cursor and the window back in agreement after a command that
/// moved either.
pub unsafe fn update_topline_cursor() {
    unsafe {
        check_cursor(curwin.get());
        update_topline(curwin.get());
        if (*curwin.get()).w_onebuf_opt.wo_wrap == 0 {
            validate_cursor(curwin.get());
        }
        update_curswant();
    }
}

/// Save the state `:normal` is about to disturb.
///
/// Answers whether the typeahead could be saved; when it could not, the
/// caller must not run anything, because there would be nowhere to put the
/// user's own pending keys back.
pub unsafe fn save_current_state(sst: *mut save_state_T) -> bool {
    unsafe {
        let s = &mut *sst;
        s.save_msg_scroll = msg_scroll.get();
        s.save_restart_edit = restart_edit.get();
        s.save_msg_didout = msg_didout.get();
        s.save_State = State.get();
        s.save_finish_op = finish_op.get();
        s.save_opcount = opcount.get();
        s.save_reg_executing = reg_executing.get();
        s.save_pending_end_reg_executing = pending_end_reg_executing.get();
        msg_scroll.set(0);
        // Not entering Insert mode from here.
        restart_edit.set(0);
        save_typeahead(&raw mut s.tabuf);
        s.tabuf.typebuf_valid
    }
}

/// Put it all back.
pub unsafe fn restore_current_state(sst: *mut save_state_T) {
    unsafe {
        let s = &*sst;
        restore_typeahead(&raw mut (*sst).tabuf);
        msg_scroll.set(s.save_msg_scroll);
        // A command that asked to enter Insert mode *after* `:normal`
        // finishes keeps that request; anything else is put back.
        if force_restart_edit.get() {
            force_restart_edit.set(false);
        } else {
            restart_edit.set(s.save_restart_edit);
        }
        finish_op.set(s.save_finish_op);
        opcount.set(s.save_opcount);
        reg_executing.set(s.save_reg_executing);
        pending_end_reg_executing.set(s.save_pending_end_reg_executing);
        msg_didout.set(msg_didout.get() || s.save_msg_didout);
        State.set(s.save_State);
        ui_cursor_shape();
    }
}

/// `:normal` — run the argument as normal-mode keys.
pub(crate) unsafe fn ex_normal(eap: *mut exarg_T) {
    unsafe {
        if !(*curbuf.get()).terminal.is_null() && State.get() & MODE_TERMINAL != 0 {
            emsg(c"Can't re-enter normal mode from terminal mode".as_ptr());
            return;
        }
        if expr_map_locked() {
            emsg(gettext(&raw const e_secure as *const c_char));
            return;
        }
        if ex_normal_busy.get() as crate::src::nvim::types::OptInt >= p_mmd.get() {
            emsg(gettext(c"E192: Recursive use of :normal too deep".as_ptr()));
            return;
        }

        let arg = escape_k_special((*eap).arg);
        *ex_normal_busy.ptr() += 1;
        let mut save_state: save_state_T = core::mem::zeroed();
        if save_current_state(&raw mut save_state) {
            loop {
                // With a range, the keys are run once per line, from the
                // first column.
                if (*eap).addr_count != 0 {
                    (*curwin.get()).w_cursor.lnum = (*eap).line1;
                    (*eap).line1 += 1;
                    (*curwin.get()).w_cursor.col = 0 as colnr_T;
                    check_cursor_moved(curwin.get());
                }
                exec_normal_cmd(
                    if arg.is_null() { (*eap).arg } else { arg },
                    if (*eap).forceit != 0 {
                        REMAP_NONE as c_int
                    } else {
                        REMAP_YES as c_int
                    },
                    false,
                );
                if !((*eap).addr_count > 0 && (*eap).line1 <= (*eap).line2 && !got_int.get()) {
                    break;
                }
            }
        }
        update_topline_cursor();
        restore_current_state(&raw mut save_state);
        *ex_normal_busy.ptr() -= 1;
        setmouse();
        ui_cursor_shape();
        xfree(arg as *mut c_void);
    }
}

/// Escape any 0x80 byte inside a multibyte character, so that the
/// typeahead does not read it as the start of a special key.
///
/// Answers null — not a copy — when there is nothing to escape, which is
/// the common case; the caller then uses the original.
unsafe fn escape_k_special(src: *mut c_char) -> *mut c_char {
    unsafe {
        // Count the extra bytes first, so the copy can be sized exactly.
        let mut extra = 0;
        let mut p = src;
        while *p as c_int != NUL {
            let mut l = utfc_ptr2len(p) - 1;
            while l > 0 {
                p = p.add(1);
                if *p as c_int == K_SPECIAL as c_char as c_int {
                    extra += 2;
                }
                l -= 1;
            }
            p = p.add(1);
        }
        if extra == 0 {
            return ptr::null_mut();
        }

        let out = xmalloc(strlen(src) + extra as size_t + 1) as *mut c_char;
        let mut len = 0;
        let mut p = src;
        while *p as c_int != NUL {
            *out.offset(len) = *p;
            len += 1;
            let mut l = utfc_ptr2len(p) - 1;
            while l > 0 {
                p = p.add(1);
                *out.offset(len) = *p;
                len += 1;
                if *p as c_int == K_SPECIAL as c_char as c_int {
                    *out.offset(len) = KS_SPECIAL as c_char;
                    len += 1;
                    *out.offset(len) = KE_FILLER as c_char;
                    len += 1;
                }
                l -= 1;
            }
            // Terminated inside the loop, so that a `break` on a bad
            // sequence still leaves a valid string.
            *out.offset(len) = NUL as c_char;
            p = p.add(1);
        }
        out
    }
}

/// `:startinsert`, `:startreplace` and `:startgreplace`.
pub(crate) unsafe fn ex_startinsert(eap: *mut exarg_T) {
    unsafe {
        if (*eap).forceit != 0 {
            if (*curwin.get()).w_cursor.lnum == 0 {
                (*curwin.get()).w_cursor.lnum = 1;
            }
            set_cursor_for_append_to_line();
        }
        if State.get() & MODE_INSERT != 0 {
            return;
        }
        let idx = (*eap).cmdidx as c_int;
        // The upper-case forms are what `edit()` reads as "started from
        // here" rather than "restarted".
        restart_edit.set(if idx == CMD_startinsert as c_int {
            'a' as c_int
        } else if idx == CMD_startreplace as c_int {
            'R' as c_int
        } else {
            'V' as c_int
        });
        if (*eap).forceit == 0 {
            if idx == CMD_startinsert as c_int {
                restart_edit.set('i' as c_int);
            }
            (*curwin.get()).w_curswant = 0 as colnr_T;
        }
        if VIsual_active.get() {
            showmode();
        }
    }
}

/// `:stopinsert`.
pub(crate) unsafe fn ex_stopinsert(_eap: *mut exarg_T) {
    unsafe {
        restart_edit.set(0);
        stop_insert_mode.set(true);
        clearmode();
    }
}

/// Put `cmd` into the typeahead and run it as normal-mode keys.
pub unsafe fn exec_normal_cmd(cmd: *mut c_char, remap: c_int, silent: bool) {
    unsafe {
        ins_typebuf(cmd, remap, 0, true, silent);
        exec_normal(false, false);
    }
}

/// Run normal-mode commands until the typeahead is spent.
pub unsafe fn exec_normal(was_typed: bool, use_vpeekc: bool) {
    unsafe {
        let mut oa: oparg_T = core::mem::zeroed();
        clear_oparg(&raw mut oa);
        finish_op.set(false);
        let mut c: c_int;
        while (!stuff_empty()
            || (was_typed || typebuf_typed() == 0) && (*typebuf.ptr()).tb_len > 0
            // `use_vpeekc` also runs whatever the *user* has typed, but
            // stops at a CTRL-C rather than swallowing it.
            || use_vpeekc && {
                c = vpeekc();
                c != NUL
            } && c != Ctrl_C)
            && !got_int.get()
        {
            update_topline_cursor();
            normal_cmd(&raw mut oa, true);
        }
    }
}

/// `:fold`.
pub(crate) unsafe fn ex_fold(eap: *mut exarg_T) {
    unsafe {
        if foldManualAllowed(true) != 0 {
            foldCreate(curwin.get(), range_start(eap), range_end(eap));
        }
    }
}

/// `:foldopen` and `:foldclose`.
pub(crate) unsafe fn ex_foldopen(eap: *mut exarg_T) {
    unsafe {
        opFoldRange(
            range_start(eap),
            range_end(eap),
            ((*eap).cmdidx as c_int == CMD_foldopen as c_int) as c_int,
            (*eap).forceit,
            false,
        );
    }
}

/// The range's first line, as a position in column 1.
unsafe fn range_start(eap: *const exarg_T) -> pos_T {
    pos_T {
        lnum: unsafe { (*eap).line1 },
        col: 1 as colnr_T,
        coladd: 0 as colnr_T,
    }
}

/// The range's last line, likewise.
unsafe fn range_end(eap: *const exarg_T) -> pos_T {
    pos_T {
        lnum: unsafe { (*eap).line2 },
        col: 1 as colnr_T,
        coladd: 0 as colnr_T,
    }
}

/// `:folddoopen` and `:folddoclosed` — run a command on every line that is
/// (or is not) inside a closed fold.
pub(crate) unsafe fn ex_folddo(eap: *mut exarg_T) {
    unsafe {
        let want_closed = ((*eap).cmdidx as c_int == CMD_folddoclosed as c_int) as c_int;
        let mut lnum = (*eap).line1;
        while lnum <= (*eap).line2 {
            if hasFolding(curwin.get(), lnum, ptr::null_mut(), ptr::null_mut()) as c_int
                == want_closed
            {
                ml_setmarked(lnum);
            }
            lnum += 1;
        }
        global_exe((*eap).arg);
        ml_clearmarked();
    }
}
