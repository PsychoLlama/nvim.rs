//! Whether the mouse is wanted, in this mode, right now.
//!
//! `'mouse'` lists the modes the mouse is enabled in, one letter each, and
//! the answer changes with the mode rather than with anything a UI did. The
//! UIs are only told when it changes — by [`ui_flush`](super::ui_flush),
//! which reads [`wanted`] — because a `mouse_on` per keystroke would be a
//! round trip per keystroke.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{State, VIsual_active, curbuf, p_mouse};
use crate::src::nvim::state::{
    MODE_ASKMORE, MODE_CMDLINE, MODE_EXTERNCMD, MODE_HITRETURN, MODE_INSERT, MODE_SETWSIZE,
};
use crate::src::nvim::strings::vim_strchr;
use core::ffi::{CStr, c_int};

/// What [`ui_check_mouse`] last worked out.
static has_mouse: GlobalCell<bool> = GlobalCell::new(false);

/// Whether the mouse is currently wanted.
pub(super) fn wanted() -> bool {
    has_mouse.get()
}

const MOUSE_NORMAL: c_int = b'n' as c_int;
const MOUSE_VISUAL: c_int = b'v' as c_int;
const MOUSE_INSERT: c_int = b'i' as c_int;
const MOUSE_COMMAND: c_int = b'c' as c_int;
const MOUSE_HELP: c_int = b'h' as c_int;
const MOUSE_RETURN: c_int = b'r' as c_int;
/// The modes `'mouse'`'s `a` covers.
const MOUSE_A: &CStr = c"nvich";

/// Recomputes whether the mouse is wanted in the current mode.
///
/// # Safety
///
/// Reads `'mouse'` and the current buffer.
pub unsafe fn ui_check_mouse() {
    has_mouse.set(false);
    if unsafe { *p_mouse.get() } == 0 {
        return;
    }
    let state = State.get();
    let checkfor = if VIsual_active.get() {
        MOUSE_VISUAL
    } else if state == MODE_HITRETURN || state == MODE_ASKMORE || state == MODE_SETWSIZE {
        MOUSE_RETURN
    } else if state & MODE_INSERT != 0 {
        MOUSE_INSERT
    } else if state & MODE_CMDLINE != 0 {
        MOUSE_COMMAND
    } else if state == MODE_EXTERNCMD {
        // The mouse belongs to whatever is running, not to us.
        b' ' as c_int
    } else {
        MOUSE_NORMAL
    };
    if unsafe { ui_mouse_has(checkfor) } {
        has_mouse.set(true);
    }
}

/// Whether `'mouse'` enables the mouse for `mode`.
///
/// # Safety
///
/// Reads `'mouse'` and the current buffer.
pub unsafe fn ui_mouse_has(mode: c_int) -> bool {
    let mut p = p_mouse.get();
    while unsafe { *p } != 0 {
        let flag = unsafe { *p } as c_int;
        let matched = match flag {
            // `a` is every mode but the hit-return prompt.
            _ if flag == b'a' as c_int => {
                !unsafe { vim_strchr(MOUSE_A.as_ptr().cast_mut(), mode) }.is_null()
            }
            MOUSE_HELP => mode != MOUSE_RETURN && unsafe { (*curbuf.get()).b_help },
            _ => mode == flag,
        };
        if matched {
            return true;
        }
        p = unsafe { p.add(1) };
    }
    false
}
