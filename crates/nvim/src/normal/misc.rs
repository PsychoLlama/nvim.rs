//! Commands that do not belong to a family: the no-ops, the error
//! handler, `:`, the CTRL-key odds and ends, and leaving a mode.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::winlayer::{Buf, Win};
use core::ptr;

use crate::buffer::{buflist_getfile, fileinfo};
use crate::cursor::check_cursor_col;
use crate::drawscreen::{UPD_CLEAR, UPD_INVERTED, redraw_curbuf_later, redraw_later, showmode};
use crate::ex_docmd::{DoCmdOpts, do_cmdline, do_cmdline_cmd};
use crate::ex_getln::{compute_cmdrow, getexline};
use crate::getchar::{
    getcmdkeycmd, map_execute_lua, paste_repeat, stuff_readbuf, stuff_readbuf_char,
    stuff_readbuf_number,
};
use crate::help::ex_help;
use crate::keycodes::{Ctrl_C, Ctrl_G, Ctrl_N, K_COMMAND, K_IGNORE, K_LUA};
use crate::main::{
    KeyTyped, clear_cmdline, cmdwin_result, cmdwin_type, curwin, did_emsg, ex_normal_busy,
    finish_op, firstwin, got_int, may_garbage_collect, mode_displayed, redraw_mode,
    restart_VIsual_select, restart_edit, typebuf_was_empty,
};
use crate::memline::ml_get_len;
use crate::message::{msg, msg_ext_set_trigger};
use crate::normal::{
    CA_COMMAND_BUSY, CmdArg, GETF_ALT, GETF_SETMARK, NULL, check_clear_op, check_clear_op_quit,
    clear_op, clear_op_beep, end_visual_mode, kMTCharWise, nv_left, nv_operator, nv_pcmark,
    set_visual_select, v_visop, visual_active, visual_select,
};
use crate::options::kOptBoFlagEsc;
use crate::os::cshim::gettext;
use crate::state::{may_trigger_modechanged, state_handle_k_event};
use crate::syntax::syn_stack_free_all;
use crate::types::{LineGetter, NUL, OP_NOP, cmdarg_T, linenr_T};
use crate::ui::vim_beep;
use crate::undo::any_buf_is_changed;
use crate::window::do_window;
use core::ffi::{c_int, c_uint};

/// A key the command loop must swallow without doing anything: it marks the
/// command busy so nothing else acts on it.
pub(crate) unsafe fn nv_ignore(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    ca.retval |= CA_COMMAND_BUSY as c_int;
}

/// A key with no effect at all -- unlike [`nv_ignore`], the command still
/// counts as having run.
pub(crate) unsafe fn nv_nop(_cap: *mut cmdarg_T) {}

/// A key that is not a command: beep and drop whatever was pending.
pub(crate) unsafe fn nv_error(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    clear_op_beep(ca.op());
}

/// `<Help>`: open the help window.
pub(crate) unsafe fn nv_help(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if !check_clear_op_quit(ca.op()) {
        unsafe { ex_help(ptr::null_mut()) };
    }
}

/// `:`, and the two synthetic keys that carry a command or a Lua callback in
/// from a mapping.
pub(crate) unsafe fn nv_colon(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    let is_cmdkey = ca.cmdchar == K_COMMAND;
    let is_lua = ca.cmdchar == K_LUA;
    // A plain `:` during a selection is the `:` *operator*, which puts the
    // selection's range on the command line. The synthetic keys are not.
    if visual_active() && !is_cmdkey && !is_lua {
        unsafe { nv_operator(cap) };
        return;
    }
    let mut op = ca.op();
    if op.op_type != OP_NOP {
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
        unsafe { compute_cmdrow() };
    }
    let cmd_result = if is_lua {
        unsafe { map_execute_lua(true, false) }
    } else {
        let getline: LineGetter = if is_cmdkey {
            Some(getcmdkeycmd)
        } else {
            Some(getexline)
        };
        let opts = if op.op_type != OP_NOP {
            DoCmdOpts::KEEPLINE
        } else {
            DoCmdOpts::NONE
        };
        unsafe { do_cmdline(ptr::null_mut(), getline, NULL, opts) != 0 }
    };
    unsafe { msg_ext_set_trigger(c"".as_ptr()) };
    if !cmd_result {
        clear_op(op);
    } else if op.op_type != OP_NOP
        && (op.start.lnum > cur_buf().b_ml.ml_line_count
            || op.start.col > unsafe { ml_get_len(op.start.lnum) }
            || did_emsg.get() != 0)
    {
        // The command moved or deleted the line the operator started on,
        // so there is nothing left to apply it to.
        clear_op_beep(op);
    }
}

/// `CTRL-G`: report the file's position -- or toggle between Visual and
/// Select mode when a selection is up.
pub(crate) unsafe fn nv_ctrlg(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if visual_active() {
        set_visual_select(!visual_select());
        unsafe { may_trigger_modechanged() };
        unsafe { showmode() };
    } else if !check_clear_op(ca.op()) {
        unsafe { fileinfo(ca.count0, 0, true) };
    }
}

/// `CTRL-H`: one character left -- or delete the selection in Select mode.
pub(crate) unsafe fn nv_ctrlh(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if visual_active() && visual_select() {
        ca.cmdchar = 'x' as c_int;
        unsafe { v_visop(cap) };
    } else {
        unsafe { nv_left(cap) };
    }
}

/// `CTRL-L`: throw the screen away and redraw it, and let syntax highlighting
/// that timed out try again.
pub(crate) unsafe fn nv_clear(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if check_clear_op(ca.op()) {
        return;
    }
    unsafe { syn_stack_free_all(cur_win().w_s) };
    // Upstream walks `firstwin` -- the *current* tab page's windows --
    // even though the loop reads as if it might walk another one's.
    let mut wp = firstwin.get();
    while !wp.is_null() {
        unsafe { (*(*wp).w_s).b_syn_slow = false };
        wp = unsafe { (*wp).w_next };
    }
    unsafe { redraw_later(curwin.get(), UPD_CLEAR) };
}

/// `CTRL-O`: jump back in the jump list -- or leave Select mode for one
/// command.
pub(crate) unsafe fn nv_ctrlo(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
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
        unsafe { nv_pcmark(cap) };
    }
}

/// `CTRL-^`: edit the alternate file.
pub(crate) unsafe fn nv_hat(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if !check_clear_op_quit(ca.op()) {
        let flags = GETF_SETMARK as c_int | GETF_ALT as c_int;
        unsafe { buflist_getfile(ca.count0, 0 as linenr_T, flags, 0) };
    }
}

/// `CTRL-W`: a window command. `CTRL-W :` is `:` with the window prefix
/// dropped.
pub(crate) unsafe fn nv_window(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if ca.nchar == ':' as c_int {
        ca.cmdchar = ':' as c_int;
        ca.nchar = NUL;
        unsafe { nv_colon(cap) };
    } else if !check_clear_op(ca.op()) {
        do_window(ca.nchar, ca.count0, NUL);
    }
}

/// `CTRL-Z`: suspend, through `:stop` so that 'autowrite' and the autocommands
/// happen.
pub(crate) unsafe fn nv_suspend(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    clear_op(ca.op());
    if visual_active() {
        end_visual_mode();
    }
    unsafe { do_cmdline_cmd(c"st".as_ptr()) };
}

/// `CTRL-\`: only `CTRL-\ CTRL-N` and `CTRL-\ CTRL-G` exist, and both mean
/// "back to Normal mode".
pub(crate) unsafe fn nv_normal(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
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
        unsafe { redraw_curbuf_later(UPD_INVERTED) };
    }
}

/// `<Esc>` and `CTRL-C`. The table's argument says which: `CTRL-C` is the one
/// that offers the "how do I quit" hint.
pub(crate) unsafe fn nv_esc(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    // Nothing was pending, so the key had no work to do and is worth a
    // beep or a hint.
    let no_reason =
        ca.op().op_type == OP_NOP && ca.opcount == 0 && ca.count0 == 0 && ca.op().regname == 0;
    if ca.arg != 0 {
        if restart_edit.get() == 0 && cmdwin_type.get() == 0 && !visual_active() && no_reason {
            let hint = if unsafe { any_buf_is_changed() } {
                c"Type  :qa!  and press <Enter> to abandon all changes and exit Nvim"
            } else {
                c"Type  :qa  and press <Enter> to exit Nvim"
            };
            unsafe { msg(gettext(hint.as_ptr()), 0) };
        }
        if restart_edit.get() != 0 {
            redraw_mode.set(true);
        }
        restart_edit.set(0);
        if cmdwin_type.get() != 0 {
            cmdwin_result.set(K_IGNORE);
            got_int.set(false);
            return;
        }
    } else if cmdwin_type.get() != 0 && ex_normal_busy.get() != 0 && typebuf_was_empty.get() {
        // `:normal` in the command-line window ran out of keys: leave the
        // window open rather than acting on the <Esc> it synthesised.
        cmdwin_result.set(K_IGNORE);
        return;
    }
    if visual_active() {
        end_visual_mode();
        unsafe { check_cursor_col(curwin.get()) };
        cur_win().w_set_curswant = true;
        unsafe { redraw_curbuf_later(UPD_INVERTED) };
    } else if no_reason {
        unsafe { vim_beep(kOptBoFlagEsc as c_uint) };
    }
    clear_op(ca.op());
}

/// The key the terminal sends to repeat a bracketed paste.
pub(crate) unsafe fn nv_paste(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    unsafe { paste_repeat(ca.count1) };
}

/// The synthetic key that stands for "the event loop has work": run it, then
/// tell the command loop whether a mode was waiting to be restarted.
pub(crate) unsafe fn nv_event(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    // An event's callback is not a safe point for a collection: it may be
    // holding values the marker cannot see.
    may_garbage_collect.set(false);
    let may_restart = restart_edit.get() != 0 || restart_VIsual_select.get() != 0;
    // SAFETY: `cap` is the caller's live command argument.
    unsafe { state_handle_k_event() };
    finish_op.set(false);
    if may_restart {
        // The callback may have left insert or Select mode pending, and
        // the command loop must not treat this key as having finished a
        // command.
        ca.retval |= CA_COMMAND_BUSY as c_int;
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
