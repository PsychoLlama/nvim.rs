//! `:normal` and the insert-mode entry commands: re-entering the
//! Normal-mode state machine from an Ex command, and putting back the
//! state that re-entry disturbs.
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use super::edit::{ui_cursor_shape, update_topline_cursor};
use crate::getchar::{Remap, ins_typebuf};
use crate::guard::Depth;
use crate::keycodes::KS_SPECIAL;
use crate::mbyte::cluster_len;
use crate::message::{E_SECURE, emsg};
use crate::os::cshim::gettext;
use crate::types::{CmdIdx, Key, LineNr};
use core::ffi::CStr;
use std::ffi::CString;

use crate::drawscreen::{clearmode, showmode};

use crate::ex_docmd::modifier::expr_map_locked;
use crate::getchar::{restore_typeahead, save_typeahead, stuff_empty, typeahead, vpeekc};

use crate::ex_docmd::state::{EX_NORMAL_BUSY, SaveState};
use crate::getchar::state::{GOT_INT, PENDING_END_REG_EXECUTING, REG_EXECUTING};
use crate::keycodes::{K_SPECIAL, KE_FILLER};
use crate::message::state::{MSG_DIDOUT, MSG_SCROLL};
use crate::option::vars::p_mmd;
use crate::state::mode::{
    FINISH_OP, FORCE_RESTART_EDIT, OPCOUNT, RESTART_EDIT, STATE, STOP_INSERT_MODE,
};

use crate::mouse::setmouse;
use crate::r#move::check_cursor_moved;
use crate::normal::{normal_cmd, set_cursor_for_append_to_line, visual_active, with_frame_op};
use crate::ops::Op;

use crate::normal::defs::OpArg;
use crate::state::{MODE_INSERT, MODE_TERMINAL};
use crate::types::{ColNr, ExArg};

use crate::register::RegName;
use crate::winlayer::{Buf, Win};

/// Save the state `:normal` is about to disturb.
///
/// Answers whether the typeahead could be saved; when it could not, the
/// caller must not run anything, because there would be nowhere to put the
/// user's own pending keys back.
pub fn save_current_state(s: &mut SaveState) -> bool {
    s.save_msg_scroll = MSG_SCROLL.get();
    s.save_restart_edit = RESTART_EDIT.get();
    s.save_msg_didout = MSG_DIDOUT.get();
    s.save_state = STATE.get();
    s.save_finish_op = FINISH_OP.get();
    s.save_opcount = OPCOUNT.get();
    s.save_reg_executing = REG_EXECUTING.get().byte();
    s.save_pending_end_reg_executing = PENDING_END_REG_EXECUTING.get();
    MSG_SCROLL.set(0);
    // Not entering Insert mode from here.
    RESTART_EDIT.set(Key::NUL);
    save_typeahead(&mut s.tabuf);
    s.tabuf.typebuf_valid
}

/// Put it all back.
pub fn restore_current_state(s: &mut SaveState) {
    restore_typeahead(&mut s.tabuf);
    MSG_SCROLL.set(s.save_msg_scroll);
    // A command that asked to enter Insert mode *after* `:normal`
    // finishes keeps that request; anything else is put back.
    if FORCE_RESTART_EDIT.get() {
        FORCE_RESTART_EDIT.set(false);
    } else {
        RESTART_EDIT.set(s.save_restart_edit);
    }
    FINISH_OP.set(s.save_finish_op);
    OPCOUNT.set(s.save_opcount);
    REG_EXECUTING.set(RegName::from_byte(s.save_reg_executing));
    PENDING_END_REG_EXECUTING.set(s.save_pending_end_reg_executing);
    MSG_DIDOUT.set(MSG_DIDOUT.get() || s.save_msg_didout);
    STATE.set(s.save_state);
    ui_cursor_shape();
}

/// `:normal` — run the argument as normal-mode keys.
pub(crate) fn ex_normal(excmd: &mut ExArg) {
    if !Buf::current().terminal.is_null() && STATE.get() & MODE_TERMINAL != 0 {
        emsg(c"Can't re-enter normal mode from terminal mode");
        return;
    }
    if expr_map_locked() {
        emsg(gettext(E_SECURE));
        return;
    }
    if crate::types::OptInt::from(EX_NORMAL_BUSY.get()) >= p_mmd() {
        emsg(gettext(c"E192: Recursive use of :normal too deep"));
        return;
    }

    // The keys run once per line of the range; the line is this command's
    // own, and nothing the keys run can reach it.
    let arg = excmd.line.cstr_from(excmd.line.arg);
    let escaped = escape_k_special(arg.to_bytes())
        .map(|keys| CString::new(keys).expect("a command line argument ends at its NUL"));
    let keys = escaped.as_deref().unwrap_or(arg);
    let remap = if excmd.forceit { Remap::No } else { Remap::Yes };
    let busy = Depth::of(EX_NORMAL_BUSY);
    let mut save_state = SaveState::default();
    if save_current_state(&mut save_state) {
        loop {
            // With a range, the keys are run once per line, from the
            // first column.
            if excmd.addr_count != 0 {
                Win::current().w_cursor.lnum = excmd.line1;
                excmd.line1 += 1;
                Win::current().w_cursor.col = ColNr::ZERO;
                check_cursor_moved(Win::current());
            }
            exec_normal_cmd(keys, remap, false);
            if !(excmd.addr_count > 0 && excmd.line1 <= excmd.line2 && !GOT_INT.get()) {
                break;
            }
        }
    }
    update_topline_cursor();
    restore_current_state(&mut save_state);
    drop(busy);
    setmouse();
    ui_cursor_shape();
}

/// Escape any 0x80 byte inside a multibyte character, so that the
/// typeahead does not read it as the start of a special key.
///
/// Answers `None` -- not a copy -- when there is nothing to escape, which is
/// the common case; the caller then uses the original.
fn escape_k_special(src: &[u8]) -> Option<Vec<u8>> {
    // Only a 0x80 byte can need it.
    if !src.contains(&(K_SPECIAL)) {
        return None;
    }
    // The bytes after the first of each character, which are the ones a
    // 0x80 hides in.
    let trailing = |at: usize| &src[at + 1..at + cluster_len(&src[at..]).max(1)];
    let mut extra = 0;
    let mut at = 0;
    while at < src.len() {
        let tail = trailing(at);
        extra += 2 * tail.iter().filter(|&&b| b == K_SPECIAL).count();
        at += 1 + tail.len();
    }
    if extra == 0 {
        return None;
    }
    let mut out = Vec::with_capacity(src.len() + extra);
    let mut at = 0;
    while at < src.len() {
        out.push(src[at]);
        let tail = trailing(at);
        for &byte in tail {
            out.push(byte);
            if byte == K_SPECIAL {
                out.extend_from_slice(&[KS_SPECIAL, KE_FILLER]);
            }
        }
        at += 1 + tail.len();
    }
    Some(out)
}

/// `:startinsert`, `:startreplace` and `:startgreplace`.
pub(crate) fn ex_startinsert(excmd: &mut ExArg) {
    if excmd.forceit {
        if Win::current().w_cursor.lnum == LineNr::ZERO {
            Win::current().w_cursor.lnum = LineNr::ONE;
        }
        set_cursor_for_append_to_line();
    }
    if STATE.get() & MODE_INSERT != 0 {
        return;
    }
    let idx = excmd.cmdidx;
    // The upper-case forms are what `edit()` reads as "started from
    // here" rather than "restarted".
    RESTART_EDIT.set(if idx == CmdIdx::startinsert {
        Key::ascii(b'a')
    } else if idx == CmdIdx::startreplace {
        Key::ascii(b'R')
    } else {
        Key::ascii(b'V')
    });
    if !excmd.forceit {
        if idx == CmdIdx::startinsert {
            RESTART_EDIT.set(Key::ascii(b'i'));
        }
        Win::current().w_curswant = ColNr::ZERO;
    }
    if visual_active() {
        showmode();
    }
}

/// `:stopinsert`.
pub(crate) fn ex_stopinsert(_excmd: &mut ExArg) {
    RESTART_EDIT.set(Key::NUL);
    STOP_INSERT_MODE.set(true);
    clearmode();
}

/// Put `cmd` into the typeahead and run it as normal-mode keys.
pub fn exec_normal_cmd(cmd: &CStr, remap: Remap, silent: bool) {
    let _ = ins_typebuf(cmd.to_bytes(), remap, 0, true, silent);
    exec_normal(false, false);
}

/// Run normal-mode commands until the typeahead is spent.
pub fn exec_normal(was_typed: bool, use_vpeekc: bool) {
    FINISH_OP.set(false);
    with_frame_op(OpArg::default(), |op| {
        exec_normal_on(op, was_typed, use_vpeekc)
    });
}

/// [`exec_normal`]'s loop, every command pending on `op`.
fn exec_normal_on(op: Op, was_typed: bool, use_vpeekc: bool) {
    let mut c: Key;
    while (!stuff_empty()
        || (was_typed || typeahead().maplen() != 0) && !typeahead().is_empty()
        // `use_vpeekc` also runs whatever the *user* has typed, but
        // stops at a CTRL-C rather than swallowing it.
        || use_vpeekc && {
            c = vpeekc();
            !c.is_nul()
        } && c != Key::CTRL_C)
        && !GOT_INT.get()
    {
        update_topline_cursor();
        normal_cmd(op, true);
    }
}
