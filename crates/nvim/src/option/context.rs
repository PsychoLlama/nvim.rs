//! Reading and writing an option as another window or buffer sees it.
//!
//! The API lets a caller name a window or buffer that is not the current
//! one. Rather than teach every accessor about that, these make the named
//! one current for the duration of a single get or set — which is also why
//! they are separate from `set.rs`: entering a buffer borrows the
//! autocommand window and can fire autocommands, so the choice of when to
//! do it is a decision of its own.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use crate::api::private::helpers::api_set_error;
use crate::autocmd::{aucmd_prepbuf, aucmd_restbuf};
use crate::eval::window::{restore_win_noblock, switch_win_noblock};
use crate::main::{curbuf, curwin};
use crate::types::{
    Error, FAIL, OptIndex, OptScope, OptVal, OptValData, aco_save_T, buf_T, kErrorTypeException,
    kErrorTypeNone, kFalse, scid_T, switchwin_T, win_T,
};
use crate::window::win_find_tabpage;

use super::{
    get_option_value, kOptScopeBuf, kOptScopeWin, kOptValTypeNil, set_option_direct,
    set_option_value_handle_tty,
};

/// [`set_option_direct`] with another window or buffer standing in for the
/// current one.
///
/// This deliberately does not go through [`OptionContext`]: `aucmd_prepbuf`
/// has side effects of its own, and a direct write is supposed to have none.
/// Swapping the two globals is enough because nothing on this path looks at
/// anything else.
///
/// # Safety
///
/// `from` must be the live window or buffer `scope` names.
pub unsafe fn set_option_direct_for(
    opt_idx: OptIndex,
    value: OptVal,
    opt_flags: c_int,
    set_sid: scid_T,
    scope: OptScope,
    from: *mut c_void,
) {
    let save_curbuf = curbuf.get();
    let save_curwin = curwin.get();
    match scope {
        kOptScopeWin => {
            curwin.set(from.cast::<win_T>());
            // SAFETY: the caller's `from` is a live window.
            curbuf.set(unsafe { (*curwin.get()).w_buffer });
        }
        kOptScopeBuf => curbuf.set(from.cast::<buf_T>()),
        _ => {}
    }
    set_option_direct(opt_idx, value, opt_flags, set_sid);
    curwin.set(save_curwin);
    curbuf.set(save_curbuf);
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
    Win(switchwin_T),
    Buf(aco_save_T),
}

impl OptionContext {
    /// Fresh scratch space for the given scope.
    pub(crate) fn new(scope: OptScope) -> Self {
        match scope {
            kOptScopeWin => OptionContext::Win(switchwin_T {
                sw_curwin: ptr::null_mut(),
                sw_curtab: ptr::null_mut(),
                sw_same_win: false,
                sw_visual_active: false,
            }),
            kOptScopeBuf => OptionContext::Buf(aco_save_T::default()),
            _ => OptionContext::Global,
        }
    }

    /// Make `from` current, reporting whether anything was switched — which
    /// is also whether [`OptionContext::leave`] has to be called. A window
    /// that could not be entered sets `err`.
    ///
    /// # Safety
    ///
    /// `from` must be the live window or buffer this context's scope names,
    /// and `err` a valid error slot.
    pub(crate) unsafe fn enter(&mut self, from: *mut c_void, err: *mut Error) -> bool {
        // SAFETY: the caller's `from` matches the scope, and `err` is valid.
        unsafe {
            match self {
                OptionContext::Global => false,
                OptionContext::Win(switchwin) => {
                    let win = from.cast::<win_T>();
                    if win == curwin.get() {
                        return false;
                    }
                    if switch_win_noblock(switchwin, win, win_find_tabpage(win), true) == FAIL {
                        restore_win_noblock(switchwin, true);
                        if (*err).type_0 == kErrorTypeNone {
                            api_set_error(
                                err,
                                kErrorTypeException,
                                c"Problem while switching windows".as_ptr(),
                            );
                        }
                        return false;
                    }
                    true
                }
                OptionContext::Buf(aco) => {
                    let buf = from.cast::<buf_T>();
                    if buf == curbuf.get() {
                        return false;
                    }
                    aucmd_prepbuf(aco, buf);
                    true
                }
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
        unsafe {
            match self {
                OptionContext::Global => {}
                OptionContext::Win(switchwin) => restore_win_noblock(switchwin, true),
                OptionContext::Buf(aco) => aucmd_restbuf(aco),
            }
        }
    }
}

/// [`get_option_value`] as another window or buffer sees it.
///
/// # Safety
///
/// `from` must be the live window or buffer `scope` names, and `err` a valid
/// error slot.
pub unsafe fn get_option_value_for(
    opt_idx: OptIndex,
    opt_flags: c_int,
    scope: OptScope,
    from: *mut c_void,
    err: *mut Error,
) -> OptVal {
    let mut ctx = OptionContext::new(scope);
    // SAFETY: the caller's `from` matches `scope`, and `err` is valid.
    let switched = unsafe { ctx.enter(from, err) };
    // SAFETY: `err` is valid.
    if unsafe { (*err).type_0 } != kErrorTypeNone {
        return OptVal {
            type_0: kOptValTypeNil,
            data: OptValData { boolean: kFalse },
        };
    }
    let value = get_option_value(opt_idx, opt_flags);
    if switched {
        // SAFETY: `enter` reported a switch and nothing has moved since.
        unsafe { ctx.leave() };
    }
    value
}

/// [`set_option_value_handle_tty`] on another window or buffer, reporting a
/// rejection through `err`.
///
/// # Safety
///
/// `name` must be NUL-terminated, `from` the live window or buffer `scope`
/// names, and `err` a valid error slot.
pub unsafe fn set_option_value_for(
    name: *const c_char,
    opt_idx: OptIndex,
    value: OptVal,
    opt_flags: c_int,
    scope: OptScope,
    from: *mut c_void,
    err: *mut Error,
) {
    let mut ctx = OptionContext::new(scope);
    // SAFETY: the caller's `from` matches `scope`, and `err` is valid.
    let switched = unsafe { ctx.enter(from, err) };
    // SAFETY: `err` is valid.
    if unsafe { (*err).type_0 } != kErrorTypeNone {
        return;
    }
    // SAFETY: the caller's `name` is NUL-terminated.
    let errmsg = unsafe { set_option_value_handle_tty(name, opt_idx, value, opt_flags) };
    if !errmsg.is_null() {
        // SAFETY: `err` is valid and `errmsg` NUL-terminated.
        unsafe { api_set_error(err, kErrorTypeException, c"%s".as_ptr(), errmsg) };
    }
    if switched {
        // SAFETY: `enter` reported a switch and nothing has moved since.
        unsafe { ctx.leave() };
    }
}
