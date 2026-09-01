//! Selecting an operator, a register or a recording, and replaying them.
//!
//! Several of these are two commands sharing a key: `u` is undo at the top
//! level but the `gu` operator once one is pending or a selection is up, and
//! `q` is a recording unless the pending operator is `gq`. The redirection is
//! always the same shape -- rewrite `cap` as the `g` form and call
//! [`nv_operator`].

#![deny(unsafe_op_in_unsafe_fn)]

use crate::winlayer::Win;
use core::ptr;

use crate::buffer::{buf_is_prompt, current_buf};
use crate::edit::{BeginlineOpts, beginline, cursor_down, prompt_curpos_editable};
use crate::eval::vars::{set_reg_var, set_vim_var_string};
use crate::getchar::{plain_vgetc, start_redo, stuff_readbuf_char};
use crate::guard::Keys;
use crate::keycodes::{Ctrl_V, KE_CMDWIN};
use crate::main::{
    VIsual_select_reg, arrow_used, cmdwin_type, got_int, reg_executing, reg_recorded, restart_edit,
};
use crate::message::emsg;
use crate::normal::{
    CmdArg, check_clear_op, check_clear_op_quit, clear_op_beep, e_cmdline_window_already_open,
    kMTLineWise, langmap_adjust, visual_active, visual_select,
};
use crate::ops::{get_extra_op_char, get_op_char, get_op_type, op_is_change};
use crate::os::cshim::gettext;
use crate::os::input::line_breakcheck;
use crate::register::{do_execreg, do_record, get_expr_register, valid_yank_reg};
use crate::types::{NUL, OpType, Vv, cmdarg_T};
use crate::undo::{u_redo, u_undo, u_undoline};
use core::ffi::{c_char, c_int};

/// Re-run this command as the two-character `g<nchar>` operator instead.
unsafe fn as_g_operator(cap: *mut cmdarg_T, nchar: u8) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    ca.cmdchar = 'g' as c_int;
    ca.nchar = nchar as c_int;
    unsafe { nv_operator(cap) };
}

/// Play a register back `count1` times, stopping at the first failure or
/// interrupt.
unsafe fn replay(cap: *mut cmdarg_T, regname: c_int) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    while ca.count1 != 0 && !got_int.get() {
        ca.count1 -= 1;
        if unsafe { do_execreg(regname, 0, 0, 0) }.is_err() {
            clear_op_beep(ca.op());
            break;
        }
        line_breakcheck();
    }
}

/// `@@`: replay whatever `@` last played.
pub(crate) unsafe fn nv_regreplay(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if check_clear_op(ca.op()) {
        return;
    }
    unsafe { replay(cap, reg_recorded.get()) };
}

/// `@`: replay a named register.
pub(crate) unsafe fn nv_at(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if check_clear_op(ca.op()) {
        return;
    }
    // `@=` prompts for an expression; a cancelled prompt does nothing.
    if ca.nchar == '=' as c_int && unsafe { get_expr_register() } == NUL {
        return;
    }
    unsafe { replay(cap, ca.nchar) };
}

/// `u`: undo, or the `gu` operator when one is already pending or a Visual
/// selection is up.
pub(crate) unsafe fn nv_undo(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if ca.op().op_type == OpType::Lower || visual_active() {
        unsafe { as_g_operator(cap, b'u') };
    } else {
        unsafe { nv_kundo(cap) };
    }
}

/// `u` proper.
pub(crate) unsafe fn nv_kundo(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if check_clear_op_quit(ca.op()) {
        return;
    }
    unsafe { u_undo(ca.count1) };
    cur_win().w_set_curswant = true;
}

/// `U`: undo the whole line, or the `gU` operator.
pub(crate) unsafe fn nv_undo_line(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if ca.op().op_type == OpType::Upper || visual_active() {
        unsafe { as_g_operator(cap, b'U') };
        return;
    }
    if check_clear_op_quit(ca.op()) {
        return;
    }
    unsafe { u_undoline() };
    cur_win().w_set_curswant = true;
}

/// `"`: name the register the next command works on.
pub(crate) unsafe fn nv_regname(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if check_clear_op(ca.op()) {
        return;
    }
    // `"=` prompts for the expression register's contents up front.
    if ca.nchar == '=' as c_int {
        ca.nchar = unsafe { get_expr_register() };
    }
    if ca.nchar != NUL && unsafe { valid_yank_reg(ca.nchar, false) } {
        ca.op().regname = ca.nchar;
        // The count so far belongs to the command, not to the `"`.
        ca.opcount = ca.count0;
        unsafe { set_reg_var(ca.op().regname) };
    } else {
        clear_op_beep(ca.op());
    }
}

/// `.`: repeat the last change.
pub(crate) unsafe fn nv_dot(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if check_clear_op_quit(ca.op()) {
        return;
    }
    // The insert half is only replayed when insert mode was left by a
    // command rather than by an arrow key, which ends the change.
    let repeat_insert = restart_edit.get() != 0 && !arrow_used.get();
    if unsafe { start_redo(ca.count0, repeat_insert) }.is_err() {
        clear_op_beep(ca.op());
    }
}

/// `CTRL-R`: redo -- or, in Select mode, the register the replacement text
/// should go to.
pub(crate) unsafe fn nv_redo_or_register(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if visual_select() && visual_active() {
        // SAFETY: reads one key with mappings suppressed.
        let unmapped = Keys::unmapped();
        let mut reg = plain_vgetc();
        langmap_adjust(&mut reg, true);
        drop(unmapped);
        // `"` names the unnamed register, which is spelled 0 here.
        if reg == '"' as c_int {
            reg = 0;
        }
        VIsual_select_reg.set(if unsafe { valid_yank_reg(reg, true) } {
            reg
        } else {
            0
        });
        return;
    }
    // SAFETY: `cap` is the caller's live command argument.
    if check_clear_op_quit(ca.op()) {
        return;
    }
    unsafe { u_redo(ca.count1) };
    cur_win().w_set_curswant = true;
}

/// Start an operator, or apply the pending one to whole lines when it is the
/// same one again (`dd`, `yy`, `gugu`).
pub(crate) unsafe fn nv_operator(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    let op_type = get_op_type(ca.cmdchar, ca.nchar);
    // A prompt buffer only lets its own last line be changed.
    if buf_is_prompt(current_buf()) && op_is_change(op_type) && !unsafe { prompt_curpos_editable() }
    {
        clear_op_beep(ca.op());
        return;
    }
    if op_type == ca.op().op_type {
        unsafe { nv_lineop(cap) };
    } else if !check_clear_op(ca.op()) {
        ca.op().start = cur_win().w_cursor;
        ca.op().op_type = op_type;
        set_op_var(op_type);
    }
}

/// Publish the pending operator as `v:operator`.
pub(crate) fn set_op_var(optype: OpType) {
    if optype == OpType::Nop {
        // SAFETY: a null string with length 0 clears the variable.
        unsafe { set_vim_var_string(Vv::Operator, ptr::null(), 0) };
        return;
    }
    // Always two bytes and a terminator: a one-character operator has NUL as
    // its second, and the length handed over is 2 either way.
    let mut opchars: [c_char; 3] = [0; 3];
    // SAFETY: both answers are single bytes of an operator's spelling.
    let opchar0 = get_op_char(optype);
    debug_assert!((0..=255).contains(&opchar0));
    opchars[0] = opchar0 as c_char;
    let opchar1 = get_extra_op_char(optype);
    debug_assert!((0..=255).contains(&opchar1));
    opchars[1] = opchar1 as c_char;
    unsafe { set_vim_var_string(Vv::Operator, opchars.as_mut_ptr(), 2) };
}

/// The linewise form of an operator: `count1` lines from this one.
pub(crate) unsafe fn nv_lineop(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    ca.op().motion_type = kMTLineWise;
    let mut op = ca.op();
    if unsafe { cursor_down(ca.count1 - 1, op.op_type == OpType::Nop) }.is_err() {
        clear_op_beep(op);
    } else if (op.op_type == OpType::Delete
        && op.motion_force != 'v' as c_int
        && op.motion_force != Ctrl_V)
        || op.op_type == OpType::Lshift
        || op.op_type == OpType::Rshift
    {
        // A delete or a shift leaves the cursor at the start of the line,
        // on the first non-blank only if 'startofline' says so.
        beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
    } else if op.op_type != OpType::Yank {
        beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
    }
}

/// `q`: start or stop a recording -- or open the command-line window, or the
/// `gq` operator when that is what is pending.
pub(crate) unsafe fn nv_record(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if ca.op().op_type == OpType::Format {
        unsafe { as_g_operator(cap, b'q') };
        return;
    }
    if check_clear_op(ca.op()) {
        return;
    }
    // `q:`, `q/` and `q?` open the command-line window instead.
    if ca.nchar == ':' as c_int || ca.nchar == '/' as c_int || ca.nchar == '?' as c_int {
        if cmdwin_type.get() != 0 {
            emsg(gettext(e_cmdline_window_already_open));
            return;
        }
        stuff_readbuf_char(ca.nchar);
        stuff_readbuf_char(-(253 + ((KE_CMDWIN as c_int) << 8)));
    } else if reg_executing.get() == 0 && unsafe { do_record(ca.nchar) }.is_err() {
        clear_op_beep(ca.op());
    }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
