//! Commands that do not belong to a family: the no-ops, the error
//! handler, `:`, the CTRL-key odds and ends, and leaving a mode.

#![deny(unsafe_op_in_unsafe_fn)]

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
    KeyTyped, clear_cmdline, cmdwin_result, cmdwin_type, curbuf, curwin, did_emsg, ex_normal_busy,
    finish_op, firstwin, got_int, may_garbage_collect, mode_displayed, redraw_mode,
    restart_VIsual_select, restart_edit, typebuf_was_empty,
};
use crate::memline::ml_get_len;
use crate::message::{msg, msg_ext_set_trigger};
use crate::normal::{
    CA_COMMAND_BUSY, GETF_ALT, GETF_SETMARK, NULL, checkclearop, checkclearopq, clearop,
    clearopbeep, end_visual_mode, kMTCharWise, nv_left, nv_operator, nv_pcmark, set_visual_select,
    v_visop, visual_active, visual_select,
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
    unsafe { (*cap).retval |= CA_COMMAND_BUSY as c_int };
}

/// A key with no effect at all -- unlike [`nv_ignore`], the command still
/// counts as having run.
pub(crate) unsafe fn nv_nop(_cap: *mut cmdarg_T) {}

/// A key that is not a command: beep and drop whatever was pending.
pub(crate) unsafe fn nv_error(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe { clearopbeep((*cap).oap) };
}

/// `<Help>`: open the help window.
pub(crate) unsafe fn nv_help(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if !checkclearopq((*cap).oap) {
            ex_help(ptr::null_mut());
        }
    }
}

/// `:`, and the two synthetic keys that carry a command or a Lua callback in
/// from a mapping.
pub(crate) unsafe fn nv_colon(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let is_cmdkey = (*cap).cmdchar == K_COMMAND;
        let is_lua = (*cap).cmdchar == K_LUA;
        // A plain `:` during a selection is the `:` *operator*, which puts the
        // selection's range on the command line. The synthetic keys are not.
        if visual_active() && !is_cmdkey && !is_lua {
            nv_operator(cap);
            return;
        }
        let oap = (*cap).oap;
        if (*oap).op_type != OP_NOP {
            (*oap).motion_type = kMTCharWise;
            (*oap).inclusive = false;
        } else if (*cap).count0 != 0 && !is_cmdkey && !is_lua {
            // A count in front of `:` becomes a range: `3:` is `:.,.+2`.
            stuff_readbuf_char('.' as c_int);
            if (*cap).count0 > 1 {
                stuff_readbuf(c",.+".as_ptr());
                stuff_readbuf_number((*cap).count0 - 1);
            }
        }
        // A typed `:` scrolls the message area up to make room for the
        // command line; a mapped one leaves the display alone.
        if KeyTyped.get() {
            msg_ext_set_trigger(c"typed_cmd".as_ptr());
            compute_cmdrow();
        }
        let cmd_result = if is_lua {
            map_execute_lua(true, false)
        } else {
            let getline: LineGetter = if is_cmdkey {
                Some(getcmdkeycmd)
            } else {
                Some(getexline)
            };
            do_cmdline(
                ptr::null_mut(),
                getline,
                NULL,
                if (*oap).op_type != OP_NOP {
                    DoCmdOpts::KEEPLINE
                } else {
                    DoCmdOpts::NONE
                },
            ) != 0
        };
        msg_ext_set_trigger(c"".as_ptr());
        if !cmd_result {
            clearop(oap);
        } else if (*oap).op_type != OP_NOP
            && ((*oap).start.lnum > (*curbuf.get()).b_ml.ml_line_count
                || (*oap).start.col > ml_get_len((*oap).start.lnum)
                || did_emsg.get() != 0)
        {
            // The command moved or deleted the line the operator started on,
            // so there is nothing left to apply it to.
            clearopbeep(oap);
        }
    }
}

/// `CTRL-G`: report the file's position -- or toggle between Visual and
/// Select mode when a selection is up.
pub(crate) unsafe fn nv_ctrlg(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if visual_active() {
            set_visual_select(!visual_select());
            may_trigger_modechanged();
            showmode();
        } else if !checkclearop((*cap).oap) {
            fileinfo((*cap).count0, 0, true);
        }
    }
}

/// `CTRL-H`: one character left -- or delete the selection in Select mode.
pub(crate) unsafe fn nv_ctrlh(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if visual_active() && visual_select() {
            (*cap).cmdchar = 'x' as c_int;
            v_visop(cap);
        } else {
            nv_left(cap);
        }
    }
}

/// `CTRL-L`: throw the screen away and redraw it, and let syntax highlighting
/// that timed out try again.
pub(crate) unsafe fn nv_clear(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if checkclearop((*cap).oap) {
            return;
        }
        syn_stack_free_all((*curwin.get()).w_s);
        // Upstream walks `firstwin` -- the *current* tab page's windows --
        // even though the loop reads as if it might walk another one's.
        let mut wp = firstwin.get();
        while !wp.is_null() {
            (*(*wp).w_s).b_syn_slow = false;
            wp = (*wp).w_next;
        }
        redraw_later(curwin.get(), UPD_CLEAR);
    }
}

/// `CTRL-O`: jump back in the jump list -- or leave Select mode for one
/// command.
pub(crate) unsafe fn nv_ctrlo(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if visual_active() && visual_select() {
            set_visual_select(false);
            may_trigger_modechanged();
            showmode();
            // 2 means "one command, then back to Select mode".
            restart_VIsual_select.set(2);
        } else {
            // A backwards jump is a negative count to the same handler `CTRL-I`
            // uses forwards.
            (*cap).count1 = -(*cap).count1;
            nv_pcmark(cap);
        }
    }
}

/// `CTRL-^`: edit the alternate file.
pub(crate) unsafe fn nv_hat(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if !checkclearopq((*cap).oap) {
            buflist_getfile(
                (*cap).count0,
                0 as linenr_T,
                GETF_SETMARK as c_int | GETF_ALT as c_int,
                0,
            );
        }
    }
}

/// `CTRL-W`: a window command. `CTRL-W :` is `:` with the window prefix
/// dropped.
pub(crate) unsafe fn nv_window(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if (*cap).nchar == ':' as c_int {
            (*cap).cmdchar = ':' as c_int;
            (*cap).nchar = NUL;
            nv_colon(cap);
        } else if !checkclearop((*cap).oap) {
            do_window((*cap).nchar, (*cap).count0, NUL);
        }
    }
}

/// `CTRL-Z`: suspend, through `:stop` so that 'autowrite' and the autocommands
/// happen.
pub(crate) unsafe fn nv_suspend(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        clearop((*cap).oap);
        if visual_active() {
            end_visual_mode();
        }
        do_cmdline_cmd(c"st".as_ptr());
    }
}

/// `CTRL-\`: only `CTRL-\ CTRL-N` and `CTRL-\ CTRL-G` exist, and both mean
/// "back to Normal mode".
pub(crate) unsafe fn nv_normal(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if (*cap).nchar != Ctrl_N && (*cap).nchar != Ctrl_G {
            clearopbeep((*cap).oap);
            return;
        }
        clearop((*cap).oap);
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
}

/// `<Esc>` and `CTRL-C`. The table's argument says which: `CTRL-C` is the one
/// that offers the "how do I quit" hint.
pub(crate) unsafe fn nv_esc(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        // Nothing was pending, so the key had no work to do and is worth a
        // beep or a hint.
        let no_reason = (*(*cap).oap).op_type == OP_NOP
            && (*cap).opcount == 0
            && (*cap).count0 == 0
            && (*(*cap).oap).regname == 0;
        if (*cap).arg != 0 {
            if restart_edit.get() == 0 && cmdwin_type.get() == 0 && !visual_active() && no_reason {
                let hint = if any_buf_is_changed() {
                    c"Type  :qa!  and press <Enter> to abandon all changes and exit Nvim"
                } else {
                    c"Type  :qa  and press <Enter> to exit Nvim"
                };
                msg(gettext(hint.as_ptr()), 0);
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
            check_cursor_col(curwin.get());
            (*curwin.get()).w_set_curswant = true;
            redraw_curbuf_later(UPD_INVERTED);
        } else if no_reason {
            vim_beep(kOptBoFlagEsc as c_uint);
        }
        clearop((*cap).oap);
    }
}

/// The key the terminal sends to repeat a bracketed paste.
pub(crate) unsafe fn nv_paste(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe { paste_repeat((*cap).count1) };
}

/// The synthetic key that stands for "the event loop has work": run it, then
/// tell the command loop whether a mode was waiting to be restarted.
pub(crate) unsafe fn nv_event(cap: *mut cmdarg_T) {
    // An event's callback is not a safe point for a collection: it may be
    // holding values the marker cannot see.
    may_garbage_collect.set(false);
    let may_restart = restart_edit.get() != 0 || restart_VIsual_select.get() != 0;
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        state_handle_k_event();
        finish_op.set(false);
        if may_restart {
            // The callback may have left insert or Select mode pending, and
            // the command loop must not treat this key as having finished a
            // command.
            (*cap).retval |= CA_COMMAND_BUSY as c_int;
        }
    }
}
