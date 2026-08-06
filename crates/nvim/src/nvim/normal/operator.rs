//! Selecting an operator, a register or a recording, and replaying them.
//!
//! Several of these are two commands sharing a key: `u` is undo at the top
//! level but the `gu` operator once one is pending or a selection is up, and
//! `q` is a recording unless the pending operator is `gq`. The redirection is
//! always the same shape -- rewrite `cap` as the `g` form and call
//! [`nv_operator`].

#![deny(unsafe_op_in_unsafe_fn)]

use core::ptr;

use crate::src::nvim::buffer::bt_prompt;
use crate::src::nvim::edit::{beginline, cursor_down, prompt_curpos_editable};
use crate::src::nvim::eval::vars::{set_reg_var, set_vim_var_string};
use crate::src::nvim::getchar::{plain_vgetc, start_redo, stuffcharReadbuff};
use crate::src::nvim::keycodes::{Ctrl_V, KE_CMDWIN};
use crate::src::nvim::main::{
    VIsual_active, VIsual_select, VIsual_select_reg, arrow_used, cmdwin_type, curbuf, curwin,
    got_int, no_mapping, reg_executing, reg_recorded, restart_edit,
};
use crate::src::nvim::message::emsg;
use crate::src::nvim::normal::{
    BL_FIX, BL_SOL, BL_WHITE, FAIL, NUL, checkclearop, checkclearopq, clearopbeep,
    e_cmdline_window_already_open, false_0, kMTLineWise, langmap_adjust, true_0,
};
use crate::src::nvim::ops::{get_extra_op_char, get_op_char, get_op_type, op_is_change};
use crate::src::nvim::os::input::line_breakcheck;
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::register::{do_execreg, do_record, get_expr_register, valid_yank_reg};
use crate::src::nvim::types::{
    OP_DELETE, OP_FORMAT, OP_LOWER, OP_LSHIFT, OP_NOP, OP_RSHIFT, OP_UPPER, OP_YANK, VV_OP,
    cmdarg_T,
};
use crate::src::nvim::undo::{u_redo, u_undo, u_undoline};
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
            if do_execreg(regname, false_0, false_0, false_0) == false_0 {
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
        if (*(*cap).oap).op_type == OP_LOWER as c_int || VIsual_active.get() {
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
        (*curwin.get()).w_set_curswant = true_0;
    }
}

/// `U`: undo the whole line, or the `gU` operator.
pub(crate) unsafe fn nv_Undo(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if (*(*cap).oap).op_type == OP_UPPER as c_int || VIsual_active.get() {
            as_g_operator(cap, b'U');
            return;
        }
        if checkclearopq((*cap).oap) {
            return;
        }
        u_undoline();
        (*curwin.get()).w_set_curswant = true_0;
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
        if start_redo((*cap).count0, repeat_insert) == false_0 {
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
            (*no_mapping.ptr()) += 1;
            let mut reg = plain_vgetc();
            langmap_adjust(&mut reg, true);
            (*no_mapping.ptr()) -= 1;
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
        (*curwin.get()).w_set_curswant = true_0;
    }
}

/// Start an operator, or apply the pending one to whole lines when it is the
/// same one again (`dd`, `yy`, `gugu`).
pub(crate) unsafe fn nv_operator(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let op_type = get_op_type((*cap).cmdchar, (*cap).nchar);
        // A prompt buffer only lets its own last line be changed.
        if bt_prompt(curbuf.get()) && op_is_change(op_type) != 0 && !prompt_curpos_editable() {
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
    if optype == OP_NOP as c_int {
        // SAFETY: a null string with length 0 clears the variable.
        unsafe { set_vim_var_string(VV_OP, ptr::null(), 0) };
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
        set_vim_var_string(VV_OP, opchars.as_mut_ptr(), 2);
    }
}

/// The linewise form of an operator: `count1` lines from this one.
pub(crate) unsafe fn nv_lineop(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        (*(*cap).oap).motion_type = kMTLineWise;
        let oap = (*cap).oap;
        if cursor_down((*cap).count1 - 1, (*oap).op_type == OP_NOP as c_int) == false_0 {
            clearopbeep(oap);
        } else if ((*oap).op_type == OP_DELETE as c_int
            && (*oap).motion_force != 'v' as c_int
            && (*oap).motion_force != Ctrl_V)
            || (*oap).op_type == OP_LSHIFT as c_int
            || (*oap).op_type == OP_RSHIFT as c_int
        {
            // A delete or a shift leaves the cursor at the start of the line,
            // on the first non-blank only if 'startofline' says so.
            beginline(BL_SOL as c_int | BL_FIX as c_int);
        } else if (*oap).op_type != OP_YANK as c_int {
            beginline(BL_WHITE as c_int | BL_FIX as c_int);
        }
    }
}

/// `q`: start or stop a recording -- or open the command-line window, or the
/// `gq` operator when that is what is pending.
pub(crate) unsafe fn nv_record(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if (*(*cap).oap).op_type == OP_FORMAT as c_int {
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
            stuffcharReadbuff((*cap).nchar);
            stuffcharReadbuff(-(253 + ((KE_CMDWIN as c_int) << 8)));
        } else if reg_executing.get() == 0 && do_record((*cap).nchar) == FAIL {
            clearopbeep((*cap).oap);
        }
    }
}
