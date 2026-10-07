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

use core::ffi::c_char;

use crate::autocmd::{aucmd_prepbuf, aucmd_restbuf};
use crate::eval::window::{restore_win_noblock, switch_win_noblock};
use crate::types::{
    AcoSave, Error, OptIndex, OptScope, OptVal, OptionSetFlags, ScriptId, SwitchWin,
};
use crate::window::win_find_tabpage;
use crate::winlayer::graph::{switch_buffer, switch_to};
use crate::winlayer::{Buf, Win};

use super::{
    get_option_value, kOptScopeBuf, kOptScopeGlobal, kOptScopeWin, set_option_direct,
    set_option_value_handle_tty,
};

/// Which window or buffer an option is written as, for
/// [`set_option_direct_for`].
///
/// The scope and the thing it names travel together, so there is nothing to
/// cast: the `OptScope` tag plus a `void *` this used to take could be
/// mismatched at a call site and the mistake would only show as a window
/// pointer being read as a buffer.
#[derive(Clone, Copy)]
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

impl OptionTarget {
    /// The option scope this target is: what a `None` target (the global
    /// scope) is to the option table.
    pub(crate) fn scope_of(target: Option<Self>) -> OptScope {
        match target {
            None => kOptScopeGlobal,
            Some(OptionTarget::Win(_)) => kOptScopeWin,
            Some(OptionTarget::Buf(_)) => kOptScopeBuf,
        }
    }
}

/// Somewhere to stand while reading or writing another window's or buffer's
/// options: the scratch space the switch back needs.
///
/// The two scopes need different machinery: a window is switched to with
/// `switch_win_noblock`, a buffer is borrowed through the autocommand window
/// with `aucmd_prepbuf`.
enum OptionContext {
    Win(SwitchWin),
    Buf(AcoSave),
}

impl OptionContext {
    /// Make `target` current, answering the context to [`leave`] -- `None`
    /// when nothing had to be switched: the global scope, or a target that
    /// is already current.
    ///
    /// [`leave`]: OptionContext::leave
    fn enter(target: Option<OptionTarget>) -> Result<Option<Self>, Error> {
        match target {
            None => Ok(None),
            Some(OptionTarget::Win(win)) if win.is_current() => Ok(None),
            Some(OptionTarget::Buf(buf)) if buf.is_current() => Ok(None),
            Some(OptionTarget::Win(win)) => {
                let mut switchwin = SwitchWin {
                    sw_curwin: None,
                    sw_curtab: None,
                    sw_same_win: false,
                    sw_visual_active: false,
                };
                let tab = win_find_tabpage(win.id());
                if switch_win_noblock(&mut switchwin, win, tab, true).is_err() {
                    restore_win_noblock(&mut switchwin, true);
                    return Err(Error::exception(c"Problem while switching windows"));
                }
                Ok(Some(OptionContext::Win(switchwin)))
            }
            Some(OptionTarget::Buf(buf)) => {
                let mut aco = AcoSave::default();
                // SAFETY: `aco` is this frame's, and `buf` live as above.
                unsafe { aucmd_prepbuf(&raw mut aco, buf) };
                Ok(Some(OptionContext::Buf(aco)))
            }
        }
    }

    /// Undo the switch [`OptionContext::enter`] made.
    fn leave(mut self) {
        match &mut self {
            OptionContext::Win(switchwin) => restore_win_noblock(switchwin, true),
            // SAFETY: the scratch space `enter` filled, and nothing else has
            // moved the current buffer since.
            OptionContext::Buf(aco) => unsafe { aucmd_restbuf(aco) },
        }
    }
}

/// [`get_option_value`] as `target` sees it; `None` is the global scope.
pub(crate) fn get_option_value_for(
    opt_idx: OptIndex,
    opt_flags: OptionSetFlags,
    target: Option<OptionTarget>,
) -> Result<OptVal, Error> {
    let ctx = OptionContext::enter(target)?;
    let value = get_option_value(opt_idx, opt_flags);
    if let Some(ctx) = ctx {
        ctx.leave();
    }
    Ok(value)
}

/// [`set_option_value_handle_tty`] on `target`; `None` is the global scope.
///
/// # Safety
///
/// `name` must be NUL-terminated.
pub(crate) unsafe fn set_option_value_for(
    name: *const c_char,
    opt_idx: OptIndex,
    value: OptVal,
    opt_flags: OptionSetFlags,
    target: Option<OptionTarget>,
) -> Result<(), Error> {
    let ctx = OptionContext::enter(target)?;
    // SAFETY: the caller's `name` is NUL-terminated.
    let errmsg = unsafe { set_option_value_handle_tty(name, opt_idx, value, opt_flags) };
    if let Some(ctx) = ctx {
        ctx.leave();
    }
    errmsg.map_err(|errmsg| Error::exception(errmsg.as_cstr()))
}
