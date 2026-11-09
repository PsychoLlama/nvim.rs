//! Selecting an operator, a register or a recording, and replaying them.
//!
//! Several of these are two commands sharing a key: `u` is undo at the top
//! level but the `gu` operator once one is pending or a selection is up, and
//! `q` is a recording unless the pending operator is `gq`. The redirection is
//! always the same shape -- rewrite `cmd_arg` as the `g` form and call
//! [`nv_operator`].

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::types::Key;
use crate::winlayer::Win;

use crate::buffer::{buf_is_prompt, current_buf};
use crate::edit::{BeginlineOpts, beginline, cursor_down, prompt_curpos_editable};
use crate::eval::vars::{set_reg_var, set_vim_var_string};
use crate::getchar::state::{GOT_INT, REG_EXECUTING, REG_RECORDED};
use crate::getchar::{plain_vgetc, start_redo, stuff_readbuf_char};
use crate::guard::Keys;
use crate::keycodes::SpecialKey;
use crate::message::{emsg, emsg_invreg};
use crate::normal::defs::CmdArg;
use crate::normal::{
    MT_LINE_WISE, check_clear_op, check_clear_op_quit, clear_op_beep,
    e_cmdline_window_already_open, langmap_adjust, visual_active, visual_select,
};
use crate::ops::OpType;
use crate::ops::{get_extra_op_char, get_op_char, get_op_type, op_is_change};
use crate::os::cshim::gettext;
use crate::os::input::line_breakcheck;
use crate::register::{
    RegName, do_execreg, do_record, get_expr_register, register_name, valid_yank_reg,
};
use crate::state::mode::{VIsual_select_reg, arrow_used, restart_edit};
use crate::types::Vv;
use crate::undo::{u_redo, u_undo, u_undoline};
use crate::winlayer::graph::cmdwin_type;

/// Re-run this command as the two-character `g<nchar>` operator instead.
fn as_g_operator(cmd_arg: &mut CmdArg, nchar: u8) {
    cmd_arg.cmdchar = Key::ascii(b'g');
    cmd_arg.nchar = Key::byte(nchar);
    nv_operator(cmd_arg);
}

/// Play a register back `count1` times, stopping at the first failure or
/// interrupt.
fn replay(cmd_arg: &mut CmdArg, regname: RegName) {
    while cmd_arg.count1 != 0 && !GOT_INT.get() {
        cmd_arg.count1 -= 1;
        if do_execreg(regname, false, false, false).is_err() {
            clear_op_beep(cmd_arg.op());
            break;
        }
        line_breakcheck();
    }
}

/// `@@`: replay whatever `@` last played.
pub(crate) fn nv_regreplay(cmd_arg: &mut CmdArg) {
    if check_clear_op(cmd_arg.op()) {
        return;
    }
    replay(cmd_arg, REG_RECORDED.get());
}

/// `@`: replay a named register.
pub(crate) fn nv_at(cmd_arg: &mut CmdArg) {
    if check_clear_op(cmd_arg.op()) {
        return;
    }
    // `@=` prompts for an expression; a cancelled prompt does nothing.
    if cmd_arg.nchar.is('=') && !get_expr_register() {
        return;
    }
    let Some(regname) = register_name(cmd_arg.nchar) else {
        // A key that is not even a byte names no register: what replaying
        // it would have said.
        emsg_invreg(cmd_arg.nchar.code());
        clear_op_beep(cmd_arg.op());
        return;
    };
    replay(cmd_arg, regname);
}

/// `u`: undo, or the `gu` operator when one is already pending or a Visual
/// selection is up.
pub(crate) fn nv_undo(cmd_arg: &mut CmdArg) {
    if cmd_arg.op().op_type == OpType::Lower || visual_active() {
        as_g_operator(cmd_arg, b'u');
    } else {
        nv_kundo(cmd_arg);
    }
}

/// `u` proper.
pub(crate) fn nv_kundo(cmd_arg: &mut CmdArg) {
    if check_clear_op_quit(cmd_arg.op()) {
        return;
    }
    u_undo(cmd_arg.count1);
    Win::current().w_set_curswant = true;
}

/// `U`: undo the whole line, or the `gU` operator.
pub(crate) fn nv_undo_line(cmd_arg: &mut CmdArg) {
    if cmd_arg.op().op_type == OpType::Upper || visual_active() {
        as_g_operator(cmd_arg, b'U');
        return;
    }
    if check_clear_op_quit(cmd_arg.op()) {
        return;
    }
    u_undoline();
    Win::current().w_set_curswant = true;
}

/// `"`: name the register the next command works on.
pub(crate) fn nv_regname(cmd_arg: &mut CmdArg) {
    if check_clear_op(cmd_arg.op()) {
        return;
    }
    // `"=` prompts for the expression register's contents up front.
    if cmd_arg.nchar.is('=') && !get_expr_register() {
        cmd_arg.nchar = Key::NUL;
    }
    if let Some(regname) =
        register_name(cmd_arg.nchar).filter(|&name| name != 0 && valid_yank_reg(name, false))
    {
        cmd_arg.op().regname = regname;
        // The count so far belongs to the command, not to the `"`.
        cmd_arg.opcount = cmd_arg.count0;
        set_reg_var(cmd_arg.op().regname);
    } else {
        clear_op_beep(cmd_arg.op());
    }
}

/// `.`: repeat the last change.
pub(crate) fn nv_dot(cmd_arg: &mut CmdArg) {
    if check_clear_op_quit(cmd_arg.op()) {
        return;
    }
    // The insert half is only replayed when insert mode was left by a
    // command rather than by an arrow key, which ends the change.
    let repeat_insert = !restart_edit.get().is_nul() && !arrow_used.get();
    if start_redo(cmd_arg.count0, repeat_insert).is_err() {
        clear_op_beep(cmd_arg.op());
    }
}

/// `CTRL-R`: redo -- or, in Select mode, the register the replacement text
/// should go to.
pub(crate) fn nv_redo_or_register(cmd_arg: &mut CmdArg) {
    if visual_select() && visual_active() {
        let unmapped = Keys::unmapped();
        let mut reg = plain_vgetc();
        langmap_adjust(&mut reg, true);
        drop(unmapped);
        // `"` names the unnamed register, which is spelled 0 here.
        if reg.is('"') {
            reg = Key::NUL;
        }
        VIsual_select_reg.set(
            register_name(reg)
                .filter(|&name| valid_yank_reg(name, true))
                .unwrap_or(0),
        );
        return;
    }
    if check_clear_op_quit(cmd_arg.op()) {
        return;
    }
    u_redo(cmd_arg.count1);
    Win::current().w_set_curswant = true;
}

/// Start an operator, or apply the pending one to whole lines when it is the
/// same one again (`dd`, `yy`, `gugu`).
pub(crate) fn nv_operator(cmd_arg: &mut CmdArg) {
    let op_type = get_op_type(cmd_arg.cmdchar, cmd_arg.nchar);
    // A prompt buffer only lets its own last line be changed.
    if buf_is_prompt(current_buf()) && op_is_change(op_type) && !prompt_curpos_editable() {
        clear_op_beep(cmd_arg.op());
        return;
    }
    if op_type == cmd_arg.op().op_type {
        nv_lineop(cmd_arg);
    } else if !check_clear_op(cmd_arg.op()) {
        cmd_arg.op().start = Win::current().w_cursor;
        cmd_arg.op().op_type = op_type;
        set_op_var(op_type);
    }
}

/// Publish the pending operator as `v:operator`.
pub(crate) fn set_op_var(optype: OpType) {
    if optype == OpType::Nop {
        set_vim_var_string(Vv::Operator, None);
        return;
    }
    // Always two bytes and a terminator: a one-character operator has NUL as
    // its second, and the length handed over is 2 either way.
    set_vim_var_string(
        Vv::Operator,
        Some(&[get_op_char(optype), get_extra_op_char(optype)]),
    );
}

/// The linewise form of an operator: `count1` lines from this one.
pub(crate) fn nv_lineop(cmd_arg: &mut CmdArg) {
    cmd_arg.op().motion_type = MT_LINE_WISE;
    let op = cmd_arg.op();
    if cursor_down(cmd_arg.count1 - 1, op.op_type == OpType::Nop).is_err() {
        clear_op_beep(op);
    } else if (op.op_type == OpType::Delete
        && !op.motion_force.is('v')
        && op.motion_force != Key::CTRL_V)
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
pub(crate) fn nv_record(cmd_arg: &mut CmdArg) {
    if cmd_arg.op().op_type == OpType::Format {
        as_g_operator(cmd_arg, b'q');
        return;
    }
    if check_clear_op(cmd_arg.op()) {
        return;
    }
    // `q:`, `q/` and `q?` open the command-line window instead.
    if cmd_arg.nchar.is(':') || cmd_arg.nchar.is('/') || cmd_arg.nchar.is('?') {
        if cmdwin_type.get() != 0 {
            emsg(gettext(e_cmdline_window_already_open));
            return;
        }
        stuff_readbuf_char(cmd_arg.nchar);
        stuff_readbuf_char(SpecialKey::Cmdwin.into());
    } else if REG_EXECUTING.get() == 0 && do_record(cmd_arg.nchar).is_err() {
        clear_op_beep(cmd_arg.op());
    }
}
