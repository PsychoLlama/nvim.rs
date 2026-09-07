//! Commands that do not belong to a family: the no-ops, the error
//! handler, `:`, the CTRL-key odds and ends, and leaving a mode.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::keycodes::{Ctrl_C, Ctrl_G, Ctrl_N, Key};
use crate::winlayer::{Buf, Win, windows};
use core::ptr;

use crate::buffer::{buflist_getfile, fileinfo};
use crate::cursor::check_cursor_col;
use crate::drawscreen::state::{clear_cmdline, mode_displayed, redraw_mode};
use crate::drawscreen::{UPD_CLEAR, UPD_INVERTED, redraw_curbuf_later, redraw_later, showmode};
use crate::eval::gc::may_garbage_collect;
use crate::ex_docmd::state::ex_normal_busy;
use crate::ex_docmd::{DoCmdOpts, do_cmdline, do_cmdline_cmd};
use crate::ex_getln::{compute_cmdrow, getexline};
use crate::getchar::state::{KeyTyped, got_int, typebuf_was_empty};
use crate::getchar::{
    getcmdkeycmd, map_execute_lua, paste_repeat, stuff_readbuf, stuff_readbuf_char,
    stuff_readbuf_number,
};
use crate::help::ex_help;
use crate::memline::ml_get_len;
use crate::message::state::did_emsg;
use crate::message::{msg, msg_ext_set_trigger};
use crate::normal::{
    CA_COMMAND_BUSY, CmdArgRef, GETF_ALT, GETF_SETMARK, NULL, check_clear_op, check_clear_op_quit,
    clear_op, clear_op_beep, end_visual_mode, kMTCharWise, nv_left, nv_operator, nv_pcmark,
    set_visual_select, v_visop, visual_active, visual_select,
};
use crate::options::kOptBoFlagEsc;
use crate::os::cshim::gettext;
use crate::state::mode::{finish_op, restart_VIsual_select, restart_edit};
use crate::state::{may_trigger_modechanged, state_handle_k_event};
use crate::syntax::{cur_syn_block, syn_stack_free_all};
use crate::types::{CmdArg, LineGetter, LineNr, NUL, OpType};
use crate::ui::vim_beep;
use crate::undo::any_buf_is_changed;
use crate::window::do_window;
use crate::winlayer::graph::{cmdwin_result, cmdwin_type};
use core::ffi::{c_int, c_uint};

/// A key the command loop must swallow without doing anything: it marks the
/// command busy so nothing else acts on it.
pub(crate) unsafe fn nv_ignore(cmd_arg: *mut CmdArg) {
    // SAFETY: `cmd_arg` is the caller's live command argument.
    let mut ca = unsafe { CmdArgRef::new(cmd_arg) };
    ca.retval |= CA_COMMAND_BUSY.cast_signed();
}

/// A key with no effect at all -- unlike [`nv_ignore`], the command still
/// counts as having run.
pub(crate) unsafe fn nv_nop(_cmd_arg: *mut CmdArg) {}

/// A key that is not a command: beep and drop whatever was pending.
pub(crate) unsafe fn nv_error(cmd_arg: *mut CmdArg) {
    // SAFETY (throughout): `cmd_arg` is the caller's live command argument.
    let ca = unsafe { CmdArgRef::new(cmd_arg) };
    clear_op_beep(ca.op());
}

/// `<Help>`: open the help window.
pub(crate) unsafe fn nv_help(cmd_arg: *mut CmdArg) {
    // SAFETY (throughout): `cmd_arg` is the caller's live command argument.
    let ca = unsafe { CmdArgRef::new(cmd_arg) };
    if !check_clear_op_quit(ca.op()) {
        unsafe { ex_help(ptr::null_mut()) };
    }
}

/// `:`, and the two synthetic keys that carry a command or a Lua callback in
/// from a mapping.
pub(crate) unsafe fn nv_colon(cmd_arg: *mut CmdArg) {
    // SAFETY (throughout): `cmd_arg` is the caller's live command argument.
    let ca = unsafe { CmdArgRef::new(cmd_arg) };
    let is_cmdkey = ca.cmdchar == Key::Command.code();
    let is_lua = ca.cmdchar == Key::Lua.code();
    // A plain `:` during a selection is the `:` *operator*, which puts the
    // selection's range on the command line. The synthetic keys are not.
    if visual_active() && !is_cmdkey && !is_lua {
        unsafe { nv_operator(cmd_arg) };
        return;
    }
    let mut op = ca.op();
    if op.op_type != OpType::Nop {
        op.motion_type = kMTCharWise;
        op.inclusive = false;
    } else if ca.count0 != 0 && !is_cmdkey && !is_lua {
        // A count in front of `:` becomes a range: `3:` is `:.,.+2`.
        stuff_readbuf_char('.' as c_int);
        if ca.count0 > 1 {
            unsafe { stuff_readbuf(c",.+".as_ptr()) };
            stuff_readbuf_number(ca.count0 - 1);
        }
    }
    // A typed `:` scrolls the message area up to make room for the
    // command line; a mapped one leaves the display alone.
    if KeyTyped.get() {
        unsafe { msg_ext_set_trigger(c"typed_cmd".as_ptr()) };
        compute_cmdrow();
    }
    let cmd_result = if is_lua {
        unsafe { map_execute_lua(true, false) }
    } else {
        let getline: LineGetter = if is_cmdkey {
            Some(getcmdkeycmd)
        } else {
            Some(getexline)
        };
        let opts = if op.op_type != OpType::Nop {
            DoCmdOpts::KEEPLINE
        } else {
            DoCmdOpts::NONE
        };
        unsafe { do_cmdline(ptr::null_mut(), getline, NULL, opts).is_ok() }
    };
    unsafe { msg_ext_set_trigger(c"".as_ptr()) };
    if !cmd_result {
        clear_op(op);
    } else if op.op_type != OpType::Nop
        && (op.start.lnum > Buf::current().b_ml.ml_line_count
            || op.start.col > ml_get_len(op.start.lnum)
            || did_emsg.get() != 0)
    {
        // The command moved or deleted the line the operator started on,
        // so there is nothing left to apply it to.
        clear_op_beep(op);
    }
}

/// `CTRL-G`: report the file's position -- or toggle between Visual and
/// Select mode when a selection is up.
pub(crate) unsafe fn nv_ctrlg(cmd_arg: *mut CmdArg) {
    // SAFETY (throughout): `cmd_arg` is the caller's live command argument.
    let ca = unsafe { CmdArgRef::new(cmd_arg) };
    if visual_active() {
        set_visual_select(!visual_select());
        unsafe { may_trigger_modechanged() };
        unsafe { showmode() };
    } else if !check_clear_op(ca.op()) {
        unsafe { fileinfo(ca.count0, 0, true) };
    }
}

/// `CTRL-H`: one character left -- or delete the selection in Select mode.
pub(crate) unsafe fn nv_ctrlh(cmd_arg: *mut CmdArg) {
    // SAFETY (throughout): `cmd_arg` is the caller's live command argument.
    let mut ca = unsafe { CmdArgRef::new(cmd_arg) };
    if visual_active() && visual_select() {
        ca.cmdchar = 'x' as c_int;
        unsafe { v_visop(cmd_arg) };
    } else {
        unsafe { nv_left(cmd_arg) };
    }
}

/// `CTRL-L`: throw the screen away and redraw it, and let syntax highlighting
/// that timed out try again.
pub(crate) unsafe fn nv_clear(cmd_arg: *mut CmdArg) {
    // SAFETY (throughout): `cmd_arg` is the caller's live command argument.
    let ca = unsafe { CmdArgRef::new(cmd_arg) };
    if check_clear_op(ca.op()) {
        return;
    }
    syn_stack_free_all(cur_syn_block());
    // Upstream walks `firstwin` -- the *current* tab page's windows --
    // even though the loop reads as if it might walk another one's.
    for wp in windows() {
        let block = wp.w_s;
        // SAFETY: a live window's syntax block.
        unsafe { (*block).b_syn_slow = false };
    }
    redraw_later(Win::current(), UPD_CLEAR);
}

/// `CTRL-O`: jump back in the jump list -- or leave Select mode for one
/// command.
pub(crate) unsafe fn nv_ctrlo(cmd_arg: *mut CmdArg) {
    // SAFETY (throughout): `cmd_arg` is the caller's live command argument.
    let mut ca = unsafe { CmdArgRef::new(cmd_arg) };
    if visual_active() && visual_select() {
        set_visual_select(false);
        unsafe { may_trigger_modechanged() };
        unsafe { showmode() };
        // 2 means "one command, then back to Select mode".
        restart_VIsual_select.set(2);
    } else {
        // A backwards jump is a negative count to the same handler `CTRL-I`
        // uses forwards.
        ca.count1 = -ca.count1;
        unsafe { nv_pcmark(cmd_arg) };
    }
}

/// `CTRL-^`: edit the alternate file.
pub(crate) unsafe fn nv_hat(cmd_arg: *mut CmdArg) {
    // SAFETY (throughout): `cmd_arg` is the caller's live command argument.
    let ca = unsafe { CmdArgRef::new(cmd_arg) };
    if !check_clear_op_quit(ca.op()) {
        let flags = GETF_SETMARK.cast_signed() | GETF_ALT.cast_signed();
        let _ = unsafe { buflist_getfile(ca.count0, 0 as LineNr, flags, 0) };
    }
}

/// `CTRL-W`: a window command. `CTRL-W :` is `:` with the window prefix
/// dropped.
pub(crate) unsafe fn nv_window(cmd_arg: *mut CmdArg) {
    // SAFETY (throughout): `cmd_arg` is the caller's live command argument.
    let mut ca = unsafe { CmdArgRef::new(cmd_arg) };
    if ca.nchar == ':' as c_int {
        ca.cmdchar = ':' as c_int;
        ca.nchar = NUL;
        unsafe { nv_colon(cmd_arg) };
    } else if !check_clear_op(ca.op()) {
        do_window(ca.nchar, ca.count0, NUL);
    }
}

/// `CTRL-Z`: suspend, through `:stop` so that 'autowrite' and the autocommands
/// happen.
pub(crate) unsafe fn nv_suspend(cmd_arg: *mut CmdArg) {
    // SAFETY (throughout): `cmd_arg` is the caller's live command argument.
    let ca = unsafe { CmdArgRef::new(cmd_arg) };
    clear_op(ca.op());
    if visual_active() {
        end_visual_mode();
    }
    let _ = unsafe { do_cmdline_cmd(c"st".as_ptr()) };
}

/// `CTRL-\`: only `CTRL-\ CTRL-N` and `CTRL-\ CTRL-G` exist, and both mean
/// "back to Normal mode".
pub(crate) unsafe fn nv_normal(cmd_arg: *mut CmdArg) {
    // SAFETY (throughout): `cmd_arg` is the caller's live command argument.
    let ca = unsafe { CmdArgRef::new(cmd_arg) };
    if ca.nchar != Ctrl_N && ca.nchar != Ctrl_G {
        clear_op_beep(ca.op());
        return;
    }
    clear_op(ca.op());
    if restart_edit.get() != 0 && mode_displayed.get() {
        clear_cmdline.set(true);
    }
    restart_edit.set(0);
    if cmdwin_type.get() != 0 {
        cmdwin_result.set(Ctrl_C);
    }
    if visual_active() {
        end_visual_mode();
        redraw_curbuf_later(UPD_INVERTED);
    }
}

/// `<Esc>` and `CTRL-C`. The table's argument says which: `CTRL-C` is the one
/// that offers the "how do I quit" hint.
pub(crate) unsafe fn nv_esc(cmd_arg: *mut CmdArg) {
    // SAFETY (throughout): `cmd_arg` is the caller's live command argument.
    let ca = unsafe { CmdArgRef::new(cmd_arg) };
    // Nothing was pending, so the key had no work to do and is worth a
    // beep or a hint.
    let no_reason =
        ca.op().op_type == OpType::Nop && ca.opcount == 0 && ca.count0 == 0 && ca.op().regname == 0;
    if ca.arg != 0 {
        if restart_edit.get() == 0 && cmdwin_type.get() == 0 && !visual_active() && no_reason {
            let hint = if any_buf_is_changed() {
                c"Type  :qa!  and press <Enter> to abandon all changes and exit Nvim"
            } else {
                c"Type  :qa  and press <Enter> to exit Nvim"
            };
            msg(gettext(hint), 0);
        }
        if restart_edit.get() != 0 {
            redraw_mode.set(true);
        }
        restart_edit.set(0);
        if cmdwin_type.get() != 0 {
            cmdwin_result.set(Key::Ignore.code());
            got_int.set(false);
            return;
        }
    } else if cmdwin_type.get() != 0 && ex_normal_busy.get() != 0 && typebuf_was_empty.get() {
        // `:normal` in the command-line window ran out of keys: leave the
        // window open rather than acting on the <Esc> it synthesised.
        cmdwin_result.set(Key::Ignore.code());
        return;
    }
    if visual_active() {
        end_visual_mode();
        check_cursor_col(Win::current());
        Win::current().w_set_curswant = true;
        redraw_curbuf_later(UPD_INVERTED);
    } else if no_reason {
        unsafe { vim_beep(kOptBoFlagEsc as c_uint) };
    }
    clear_op(ca.op());
}

/// The key the terminal sends to repeat a bracketed paste.
pub(crate) unsafe fn nv_paste(cmd_arg: *mut CmdArg) {
    // SAFETY (throughout): `cmd_arg` is the caller's live command argument.
    let ca = unsafe { CmdArgRef::new(cmd_arg) };
    unsafe { paste_repeat(ca.count1) };
}

/// The synthetic key that stands for "the event loop has work": run it, then
/// tell the command loop whether a mode was waiting to be restarted.
pub(crate) unsafe fn nv_event(cmd_arg: *mut CmdArg) {
    // SAFETY (throughout): `cmd_arg` is the caller's live command argument.
    let mut ca = unsafe { CmdArgRef::new(cmd_arg) };
    // An event's callback is not a safe point for a collection: it may be
    // holding values the marker cannot see.
    may_garbage_collect.set(false);
    let may_restart = restart_edit.get() != 0 || restart_VIsual_select.get() != 0;
    // SAFETY: `cmd_arg` is the caller's live command argument.
    unsafe { state_handle_k_event() };
    finish_op.set(false);
    if may_restart {
        // The callback may have left insert or Select mode pending, and
        // the command loop must not treat this key as having finished a
        // command.
        ca.retval |= CA_COMMAND_BUSY.cast_signed();
    }
}
