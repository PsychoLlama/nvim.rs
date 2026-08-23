//! Selecting an operator, a register or a recording, and replaying them.
//!
//! Several of these are two commands sharing a key: `u` is undo at the top
//! level but the `gu` operator once one is pending or a selection is up, and
//! `q` is a recording unless the pending operator is `gq`. The redirection is
//! always the same shape -- rewrite `cap` as the `g` form and call
//! [`nv_operator`].

#![deny(unsafe_op_in_unsafe_fn)]

use core::ptr;

use crate::buffer::bt_prompt;
use crate::edit::{BeginlineOpts, beginline, cursor_down, prompt_curpos_editable};
use crate::eval::vars::{set_reg_var, set_vim_var_string};
use crate::getchar::{plain_vgetc, start_redo, stuff_readbuf_char};
use crate::guard::Keys;
use crate::keycodes::{Ctrl_V, KE_CMDWIN};
use crate::main::{
    VIsual_active, VIsual_select, VIsual_select_reg, arrow_used, cmdwin_type, curbuf, curwin,
    got_int, reg_executing, reg_recorded, restart_edit,
};
use crate::message::emsg;
use crate::normal::{
    checkclearop, checkclearopq, clearopbeep, e_cmdline_window_already_open, kMTLineWise,
    langmap_adjust,
};
use crate::ops::{get_extra_op_char, get_op_char, get_op_type, op_is_change};
use crate::os::cshim::gettext;
use crate::os::input::line_breakcheck;
use crate::register::{do_execreg, do_record, get_expr_register, valid_yank_reg};
use crate::types::{
    FAIL, NUL, OP_DELETE, OP_FORMAT, OP_LOWER, OP_LSHIFT, OP_NOP, OP_RSHIFT, OP_UPPER, OP_YANK, Vv,
    cmdarg_T,
};
use crate::undo::{u_redo, u_undo, u_undoline};
use core::ffi::{c_char, c_int};

/// Re-run this command as the two-character `g<nchar>` operator instead.
unsafe fn as_g_operator(cap: *mut cmdarg_T, nchar: u8) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        (*cap).cmdchar = 'g' as c_int;
        (*cap).nchar = nchar as c_int;
        nv_operator(cap);
    }
}

/// Play a register back `count1` times, stopping at the first failure or
/// interrupt.
unsafe fn replay(cap: *mut cmdarg_T, regname: c_int) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        while (*cap).count1 != 0 && !got_int.get() {
            (*cap).count1 -= 1;
            if do_execreg(regname, 0, 0, 0) == 0 {
                clearopbeep((*cap).oap);
                break;
            }
            line_breakcheck();
        }
    }
}

/// `@@`: replay whatever `@` last played.
pub(crate) unsafe fn nv_regreplay(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if checkclearop((*cap).oap) {
            return;
        }
        replay(cap, reg_recorded.get());
    }
}

/// `@`: replay a named register.
pub(crate) unsafe fn nv_at(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if checkclearop((*cap).oap) {
            return;
        }
        // `@=` prompts for an expression; a cancelled prompt does nothing.
        if (*cap).nchar == '=' as c_int && get_expr_register() == NUL {
            return;
        }
        replay(cap, (*cap).nchar);
    }
}

/// `u`: undo, or the `gu` operator when one is already pending or a Visual
/// selection is up.
pub(crate) unsafe fn nv_undo(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if (*(*cap).oap).op_type == OP_LOWER || VIsual_active.get() {
            as_g_operator(cap, b'u');
        } else {
            nv_kundo(cap);
        }
    }
}

/// `u` proper.
pub(crate) unsafe fn nv_kundo(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if checkclearopq((*cap).oap) {
            return;
        }
        u_undo((*cap).count1);
        (*curwin.get()).w_set_curswant = true;
    }
}

/// `U`: undo the whole line, or the `gU` operator.
pub(crate) unsafe fn nv_undo_line(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if (*(*cap).oap).op_type == OP_UPPER || VIsual_active.get() {
            as_g_operator(cap, b'U');
            return;
        }
        if checkclearopq((*cap).oap) {
            return;
        }
        u_undoline();
        (*curwin.get()).w_set_curswant = true;
    }
}

/// `"`: name the register the next command works on.
pub(crate) unsafe fn nv_regname(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if checkclearop((*cap).oap) {
            return;
        }
        // `"=` prompts for the expression register's contents up front.
        if (*cap).nchar == '=' as c_int {
            (*cap).nchar = get_expr_register();
        }
        if (*cap).nchar != NUL && valid_yank_reg((*cap).nchar, false) {
            (*(*cap).oap).regname = (*cap).nchar;
            // The count so far belongs to the command, not to the `"`.
            (*cap).opcount = (*cap).count0;
            set_reg_var((*(*cap).oap).regname);
        } else {
            clearopbeep((*cap).oap);
        }
    }
}

/// `.`: repeat the last change.
pub(crate) unsafe fn nv_dot(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if checkclearopq((*cap).oap) {
            return;
        }
        // The insert half is only replayed when insert mode was left by a
        // command rather than by an arrow key, which ends the change.
        let repeat_insert = restart_edit.get() != 0 && !arrow_used.get();
        if start_redo((*cap).count0, repeat_insert) == 0 {
            clearopbeep((*cap).oap);
        }
    }
}

/// `CTRL-R`: redo -- or, in Select mode, the register the replacement text
/// should go to.
pub(crate) unsafe fn nv_redo_or_register(cap: *mut cmdarg_T) {
    if VIsual_select.get() && VIsual_active.get() {
        // SAFETY: reads one key with mappings suppressed.
        unsafe {
            let unmapped = Keys::unmapped();
            let mut reg = plain_vgetc();
            langmap_adjust(&mut reg, true);
            drop(unmapped);
            // `"` names the unnamed register, which is spelled 0 here.
            if reg == '"' as c_int {
                reg = 0;
            }
            VIsual_select_reg.set(if valid_yank_reg(reg, true) { reg } else { 0 });
        }
        return;
    }
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if checkclearopq((*cap).oap) {
            return;
        }
        u_redo((*cap).count1);
        (*curwin.get()).w_set_curswant = true;
    }
}

/// Start an operator, or apply the pending one to whole lines when it is the
/// same one again (`dd`, `yy`, `gugu`).
pub(crate) unsafe fn nv_operator(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let op_type = get_op_type((*cap).cmdchar, (*cap).nchar);
        // A prompt buffer only lets its own last line be changed.
        if bt_prompt(curbuf.get()) && op_is_change(op_type) && !prompt_curpos_editable() {
            clearopbeep((*cap).oap);
            return;
        }
        if op_type == (*(*cap).oap).op_type {
            nv_lineop(cap);
        } else if !checkclearop((*cap).oap) {
            (*(*cap).oap).start = (*curwin.get()).w_cursor;
            (*(*cap).oap).op_type = op_type;
            set_op_var(op_type);
        }
    }
}

/// Publish the pending operator as `v:operator`.
pub(crate) fn set_op_var(optype: c_int) {
    if optype == OP_NOP {
        // SAFETY: a null string with length 0 clears the variable.
        unsafe { set_vim_var_string(Vv::Operator, ptr::null(), 0) };
        return;
    }
    // Always two bytes and a terminator: a one-character operator has NUL as
    // its second, and the length handed over is 2 either way.
    let mut opchars: [c_char; 3] = [0; 3];
    // SAFETY: both answers are single bytes of an operator's spelling.
    unsafe {
        let opchar0 = get_op_char(optype);
        debug_assert!((0..=255).contains(&opchar0));
        opchars[0] = opchar0 as c_char;
        let opchar1 = get_extra_op_char(optype);
        debug_assert!((0..=255).contains(&opchar1));
        opchars[1] = opchar1 as c_char;
        set_vim_var_string(Vv::Operator, opchars.as_mut_ptr(), 2);
    }
}

/// The linewise form of an operator: `count1` lines from this one.
pub(crate) unsafe fn nv_lineop(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        (*(*cap).oap).motion_type = kMTLineWise;
        let oap = (*cap).oap;
        if cursor_down((*cap).count1 - 1, (*oap).op_type == OP_NOP) == 0 {
            clearopbeep(oap);
        } else if ((*oap).op_type == OP_DELETE
            && (*oap).motion_force != 'v' as c_int
            && (*oap).motion_force != Ctrl_V)
            || (*oap).op_type == OP_LSHIFT
            || (*oap).op_type == OP_RSHIFT
        {
            // A delete or a shift leaves the cursor at the start of the line,
            // on the first non-blank only if 'startofline' says so.
            beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
        } else if (*oap).op_type != OP_YANK {
            beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
        }
    }
}

/// `q`: start or stop a recording -- or open the command-line window, or the
/// `gq` operator when that is what is pending.
pub(crate) unsafe fn nv_record(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if (*(*cap).oap).op_type == OP_FORMAT {
            as_g_operator(cap, b'q');
            return;
        }
        if checkclearop((*cap).oap) {
            return;
        }
        // `q:`, `q/` and `q?` open the command-line window instead.
        if (*cap).nchar == ':' as c_int
            || (*cap).nchar == '/' as c_int
            || (*cap).nchar == '?' as c_int
        {
            if cmdwin_type.get() != 0 {
                emsg(gettext(e_cmdline_window_already_open.as_ptr()));
                return;
            }
            stuff_readbuf_char((*cap).nchar);
            stuff_readbuf_char(-(253 + ((KE_CMDWIN as c_int) << 8)));
        } else if reg_executing.get() == 0 && do_record((*cap).nchar) == FAIL {
            clearopbeep((*cap).oap);
        }
    }
}
