//! Reading and writing an option as another window or buffer sees it.
//!
//! The API lets a caller name a window or buffer that is not the current
//! one. Rather than teach every accessor about that, these make the named
//! one current for the duration of a single get or set — which is also why
//! they are separate from `set.rs`: entering a buffer borrows the
//! autocommand window and can fire autocommands, so the choice of when to
//! do it is a decision of its own.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{c_char, c_void};

use crate::autocmd::{aucmd_prepbuf, aucmd_restbuf};
use crate::eval::window::{restore_win_noblock, switch_win_noblock};
use crate::types::{
    AcoSave, Buffer, Error, OptIndex, OptScope, OptVal, OptionSetFlags, ScriptId, SwitchWin, Window,
};
use crate::window::win_find_tabpage;
use crate::winlayer::graph::{switch_buffer, switch_to};
use crate::winlayer::{Buf, Win};

use super::{
    get_option_value, kOptScopeBuf, kOptScopeWin, set_option_direct, set_option_value_handle_tty,
};

/// Which window or buffer an option is written as, for
/// [`set_option_direct_for`].
///
/// The scope and the thing it names travel together, so there is nothing to
/// cast: the `OptScope` tag plus a `void *` this used to take could be
/// mismatched at a call site and the mistake would only show as a window
/// pointer being read as a buffer.
pub(crate) enum OptionTarget {
    /// A window, whose buffer comes with it — the option code reads both.
    Win(Win),
    /// A buffer, with the current window left where it is.
    Buf(Buf),
}

/// [`set_option_direct`] with another window or buffer standing in for the
/// current one.
///
/// This deliberately does not go through [`OptionContext`]: `aucmd_prepbuf`
/// has side effects of its own, and a direct write is supposed to have none.
/// Swapping the two globals is enough because nothing on this path looks at
/// anything else.
pub(crate) fn set_option_direct_for(
    opt_idx: OptIndex,
    value: OptVal,
    opt_flags: OptionSetFlags,
    set_sid: ScriptId,
    target: OptionTarget,
) {
    let saved = match target {
        OptionTarget::Win(win) => switch_to(win),
        OptionTarget::Buf(buf) => switch_buffer(buf),
    };
    set_option_direct(opt_idx, value, opt_flags, set_sid);
    saved.restore();
}

/// Somewhere to stand while reading or writing another window's or buffer's
/// options.
///
/// The two scopes need different machinery: a window is switched to with
/// `switch_win_noblock`, a buffer is borrowed through the autocommand window
/// with `aucmd_prepbuf`. Holding the scratch space for both in one value is
/// what keeps the callers from casting it through `void *`.
pub(crate) enum OptionContext {
    /// Nothing to switch: a global option is the same everywhere.
    Global,
    Win(SwitchWin),
    Buf(AcoSave),
}

impl OptionContext {
    /// Fresh scratch space for the given scope.
    pub(crate) fn new(scope: OptScope) -> Self {
        match scope {
            kOptScopeWin => OptionContext::Win(SwitchWin {
                sw_curwin: None,
                sw_curtab: None,
                sw_same_win: false,
                sw_visual_active: false,
            }),
            kOptScopeBuf => OptionContext::Buf(AcoSave::default()),
            _ => OptionContext::Global,
        }
    }

    /// Make `from` current, answering whether anything was switched — which
    /// is also whether [`OptionContext::leave`] has to be called.
    ///
    /// # Safety
    ///
    /// `from` must be the live window or buffer this context's scope names.
    pub(crate) unsafe fn enter(&mut self, from: *mut c_void) -> Result<bool, Error> {
        // SAFETY: the caller's `from` matches the scope.
        match self {
            OptionContext::Global => Ok(false),
            OptionContext::Win(switchwin) => {
                let win = from.cast::<Window>();
                if win == Win::current_raw() {
                    return Ok(false);
                }
                // SAFETY: `win` is the window this context named, still live.
                let win = unsafe { Win::new(win) };
                let tab = win_find_tabpage(win.id());
                if unsafe { switch_win_noblock(switchwin, win, tab, true) }.is_err() {
                    unsafe { restore_win_noblock(switchwin, true) };
                    return Err(Error::exception(c"Problem while switching windows"));
                }
                Ok(true)
            }
            OptionContext::Buf(aco) => {
                let buf = from.cast::<Buffer>();
                if buf == Buf::current_raw() {
                    return Ok(false);
                }
                unsafe { aucmd_prepbuf(aco, Buf::new(buf)) };
                Ok(true)
            }
        }
    }

    /// Undo an [`OptionContext::enter`] that reported a switch.
    ///
    /// # Safety
    ///
    /// Only after `enter` returned true, and before anything else has moved
    /// the current window or buffer.
    pub(crate) unsafe fn leave(&mut self) {
        // SAFETY: the caller has just entered this context.
        match self {
            OptionContext::Global => {}
            OptionContext::Win(switchwin) => unsafe { restore_win_noblock(switchwin, true) },
            OptionContext::Buf(aco) => unsafe { aucmd_restbuf(aco) },
        }
    }
}

/// [`get_option_value`] as another window or buffer sees it.
///
/// # Safety
///
/// `from` must be the live window or buffer `scope` names.
pub(crate) unsafe fn get_option_value_for(
    opt_idx: OptIndex,
    opt_flags: OptionSetFlags,
    scope: OptScope,
    from: *mut c_void,
) -> Result<OptVal, Error> {
    let mut ctx = OptionContext::new(scope);
    // SAFETY: the caller's `from` matches `scope`.
    let switched = unsafe { ctx.enter(from) }?;
    let value = get_option_value(opt_idx, opt_flags);
    if switched {
        // SAFETY: `enter` reported a switch and nothing has moved since.
        unsafe { ctx.leave() };
    }
    Ok(value)
}

/// [`set_option_value_handle_tty`] on another window or buffer.
///
/// # Safety
///
/// `name` must be NUL-terminated, `from` the live window or buffer `scope`
/// names.
pub(crate) unsafe fn set_option_value_for(
    name: *const c_char,
    opt_idx: OptIndex,
    value: OptVal,
    opt_flags: OptionSetFlags,
    scope: OptScope,
    from: *mut c_void,
) -> Result<(), Error> {
    let mut ctx = OptionContext::new(scope);
    // SAFETY: the caller's `from` matches `scope`.
    let switched = unsafe { ctx.enter(from) }?;
    // SAFETY: the caller's `name` is NUL-terminated.
    let errmsg = unsafe { set_option_value_handle_tty(name, opt_idx, value, opt_flags) };
    if switched {
        // SAFETY: `enter` reported a switch and nothing has moved since.
        unsafe { ctx.leave() };
    }
    errmsg.map_err(|errmsg| Error::exception(errmsg.as_cstr()))
}
