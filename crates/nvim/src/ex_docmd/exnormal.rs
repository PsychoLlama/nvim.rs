//! `:normal` and the insert-mode entry commands: re-entering the
//! Normal-mode state machine from an Ex command, and putting back the
//! state that re-entry disturbs.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::edit::{
    byte, clear_oparg, emsg, gettext, ins_typebuf, ui_cursor_shape, update_topline_cursor,
    utfc_ptr2len,
};
use crate::cstr;
use crate::guard::Depth;
use crate::types::CmdIdx;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use crate::drawscreen::{clearmode, showmode};

use crate::ex_docmd::modifier::expr_map_locked;
use crate::ex_docmd::{KS_SPECIAL, REMAP_NONE, REMAP_YES};
use crate::getchar::{restore_typeahead, save_typeahead, stuff_empty, typeahead, vpeekc};

use crate::ex_docmd::state::ex_normal_busy;
use crate::getchar::state::{got_int, pending_end_reg_executing, reg_executing};
use crate::keycodes::{Ctrl_C, K_SPECIAL, KE_FILLER};
use crate::message::e_secure;
use crate::message::state::{msg_didout, msg_scroll};
use crate::option::vars::p_mmd;
use crate::state::mode::{
    State, finish_op, force_restart_edit, opcount, restart_edit, stop_insert_mode,
};

use crate::memory::{xfree, xmalloc};

use crate::mouse::setmouse;
use crate::r#move::check_cursor_moved;
use crate::normal::{normal_cmd, set_cursor_for_append_to_line, visual_active};

use crate::state::{MODE_INSERT, MODE_TERMINAL};
use crate::types::{ColNr, ExArg, NUL, OpArg, SaveState, size_t};

use crate::winlayer::{Buf, Win};

/// Save the state `:normal` is about to disturb.
///
/// Answers whether the typeahead could be saved; when it could not, the
/// caller must not run anything, because there would be nowhere to put the
/// user's own pending keys back.
///
/// # Safety
///
/// `sst` must point at a live `SaveState`, unaliased for the call.
pub unsafe fn save_current_state(sst: *mut SaveState) -> bool {
    // SAFETY: the caller's own `SaveState`, live for the call.
    let s = unsafe { &mut *sst };
    s.save_msg_scroll = msg_scroll.get();
    s.save_restart_edit = restart_edit.get();
    s.save_msg_didout = msg_didout.get();
    s.save_state = State.get();
    s.save_finish_op = finish_op.get();
    s.save_opcount = opcount.get();
    s.save_reg_executing = reg_executing.get();
    s.save_pending_end_reg_executing = pending_end_reg_executing.get();
    msg_scroll.set(0);
    // Not entering Insert mode from here.
    restart_edit.set(0);
    unsafe { save_typeahead(&raw mut s.tabuf) };
    s.tabuf.typebuf_valid
}

/// Put it all back.
///
/// # Safety
///
/// `sst` must point at a live `SaveState`, unaliased for the call.
pub unsafe fn restore_current_state(sst: *mut SaveState) {
    // SAFETY: as `save_current_state`.
    let s = unsafe { &*sst };
    unsafe { restore_typeahead(&raw mut (*sst).tabuf) };
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
    State.set(s.save_state);
    ui_cursor_shape();
}

/// `:normal` — run the argument as normal-mode keys.
pub(crate) fn ex_normal(excmd: &mut ExArg) {
    if !Buf::current().terminal.is_null() && State.get() & MODE_TERMINAL != 0 {
        emsg(c"Can't re-enter normal mode from terminal mode".as_ptr());
        return;
    }
    if expr_map_locked() {
        emsg(gettext(e_secure.as_ptr()));
        return;
    }
    if ex_normal_busy.get() as crate::types::OptInt >= p_mmd.get() {
        emsg(gettext(c"E192: Recursive use of :normal too deep".as_ptr()));
        return;
    }

    let arg = unsafe { escape_k_special(excmd.arg) };
    let busy = Depth::of(&ex_normal_busy);
    let mut save_state = SaveState::default();
    if unsafe { save_current_state(&raw mut save_state) } {
        loop {
            // With a range, the keys are run once per line, from the
            // first column.
            if excmd.addr_count != 0 {
                Win::current().w_cursor.lnum = excmd.line1;
                excmd.line1 += 1;
                Win::current().w_cursor.col = 0 as ColNr;
                check_cursor_moved(Win::current());
            }
            unsafe {
                exec_normal_cmd(
                    if arg.is_null() { excmd.arg } else { arg },
                    if excmd.forceit {
                        REMAP_NONE as c_int
                    } else {
                        REMAP_YES as c_int
                    },
                    false,
                )
            };
            if !(excmd.addr_count > 0 && excmd.line1 <= excmd.line2 && !got_int.get()) {
                break;
            }
        }
    }
    update_topline_cursor();
    unsafe { restore_current_state(&raw mut save_state) };
    drop(busy);
    setmouse();
    ui_cursor_shape();
    unsafe { xfree(arg as *mut c_void) };
}

/// Escape any 0x80 byte inside a multibyte character, so that the
/// typeahead does not read it as the start of a special key.
///
/// Answers null — not a copy — when there is nothing to escape, which is
/// the common case; the caller then uses the original.
///
/// # Safety
///
/// `src` must point at a NUL-terminated string, unaliased for the call.
unsafe fn escape_k_special(src: *mut c_char) -> *mut c_char {
    // Count the extra bytes first, so the copy can be sized exactly.
    let mut extra = 0;
    let mut p = src;
    while byte(p) != NUL {
        let mut l = utfc_ptr2len(p) - 1;
        while l > 0 {
            p = unsafe { p.add(1) };
            if byte(p) == K_SPECIAL as c_char as c_int {
                extra += 2;
            }
            l -= 1;
        }
        p = unsafe { p.add(1) };
    }
    if extra == 0 {
        return ptr::null_mut();
    }

    let out = unsafe { xmalloc(cstr::bytes_at(src).len() + extra as size_t + 1) } as *mut c_char;
    let mut len = 0;
    let mut p = src;
    while byte(p) != NUL {
        unsafe { *out.offset(len) = *p };
        len += 1;
        let mut l = utfc_ptr2len(p) - 1;
        while l > 0 {
            p = unsafe { p.add(1) };
            unsafe { *out.offset(len) = *p };
            len += 1;
            if byte(p) == K_SPECIAL as c_char as c_int {
                unsafe { *out.offset(len) = KS_SPECIAL as c_char };
                len += 1;
                unsafe { *out.offset(len) = KE_FILLER as c_char };
                len += 1;
            }
            l -= 1;
        }
        // Terminated inside the loop, so that a `break` on a bad
        // sequence still leaves a valid string.
        unsafe { *out.offset(len) = NUL as c_char };
        p = unsafe { p.add(1) };
    }
    out
}

/// `:startinsert`, `:startreplace` and `:startgreplace`.
pub(crate) fn ex_startinsert(excmd: &mut ExArg) {
    if excmd.forceit {
        if Win::current().w_cursor.lnum == 0 {
            Win::current().w_cursor.lnum = 1;
        }
        set_cursor_for_append_to_line();
    }
    if State.get() & MODE_INSERT != 0 {
        return;
    }
    let idx = excmd.cmdidx;
    // The upper-case forms are what `edit()` reads as "started from
    // here" rather than "restarted".
    restart_edit.set(if idx == CmdIdx::startinsert {
        'a' as c_int
    } else if idx == CmdIdx::startreplace {
        'R' as c_int
    } else {
        'V' as c_int
    });
    if !excmd.forceit {
        if idx == CmdIdx::startinsert {
            restart_edit.set('i' as c_int);
        }
        Win::current().w_curswant = 0 as ColNr;
    }
    if visual_active() {
        showmode();
    }
}

/// `:stopinsert`.
pub(crate) fn ex_stopinsert(_excmd: &mut ExArg) {
    restart_edit.set(0);
    stop_insert_mode.set(true);
    clearmode();
}

/// Put `cmd` into the typeahead and run it as normal-mode keys.
///
/// # Safety
///
/// `cmd` must point at a NUL-terminated string, unaliased for the call.
pub unsafe fn exec_normal_cmd(cmd: *mut c_char, remap: c_int, silent: bool) {
    let _ = ins_typebuf(cmd, remap, 0, true, silent);
    exec_normal(false, false);
}

/// Run normal-mode commands until the typeahead is spent.
pub fn exec_normal(was_typed: bool, use_vpeekc: bool) {
    let mut oa: OpArg = unsafe { core::mem::zeroed() };
    clear_oparg(&raw mut oa);
    finish_op.set(false);
    let mut c: c_int;
    while (!stuff_empty()
        || (was_typed || typeahead().maplen() != 0) && !typeahead().is_empty()
        // `use_vpeekc` also runs whatever the *user* has typed, but
        // stops at a CTRL-C rather than swallowing it.
        || use_vpeekc && {
            c = vpeekc();
            c != NUL
        } && c != Ctrl_C)
        && !got_int.get()
    {
        update_topline_cursor();
        unsafe { normal_cmd(&raw mut oa, true) };
    }
}
