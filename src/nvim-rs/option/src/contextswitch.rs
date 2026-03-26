//! Context-switch helpers for getting/setting options in window/buffer scope.
//!
//! Rust implementation of switch_option_context, restore_option_context,
//! get_option_value_for, and set_option_value_for from option_shim.c.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_void};
use std::mem::MaybeUninit;

use nvim_api::Error;

use crate::storage::OptVal;
use crate::{BufHandle, WinHandle};

// =============================================================================
// Struct sizes (validated by _Static_assert in option_shim.c)
// =============================================================================

/// sizeof(switchwin_T) — validated by _Static_assert in option_shim.c.
const SWITCHWIN_SIZE: usize = 24;

/// sizeof(aco_save_T) — validated by _Static_assert in option_shim.c.
const ACO_SAVE_SIZE: usize = 64;

// =============================================================================
// OptScope constants (must match C enum kOptScope*)
// =============================================================================

const K_OPT_SCOPE_WIN: c_int = 1;
const K_OPT_SCOPE_BUF: c_int = 2;

// =============================================================================
// C FFI declarations
// =============================================================================

type OptIndex = c_int;

extern "C" {
    static curwin: WinHandle;
    static curbuf: BufHandle;

    fn rs_win_find_tabpage(win: WinHandle) -> *mut c_void;

    fn nvim_option_switch_win_noblock(
        switchwin: *mut c_void,
        win: *mut c_void,
        tabpage: *mut c_void,
    ) -> c_int;
    fn nvim_option_restore_win_noblock(switchwin: *mut c_void);
    fn nvim_option_aucmd_prepbuf(aco: *mut c_void, buf: *mut c_void);
    fn nvim_option_aucmd_restbuf(aco: *mut c_void);

    fn nvim_option_get_option_value(opt_idx: OptIndex, opt_flags: c_int) -> OptVal;
    fn nvim_option_set_value_handle_tty(
        name: *const c_char,
        opt_idx: OptIndex,
        value: OptVal,
        opt_flags: c_int,
    ) -> *const c_char;

    fn api_set_error(err: *mut Error, err_type: c_int, fmt: *const c_char, ...);
}

/// kErrorTypeException constant (matches C enum).
const K_ERROR_TYPE_EXCEPTION: c_int = 1;

/// FAIL constant (matches C define).
const FAIL: c_int = 0;

/// kErrorTypeNone: no error (matches C enum kErrorTypeNone = -1).
const K_ERROR_TYPE_NONE: c_int = -1;

/// Context union: either a switchwin_T or aco_save_T, allocated on stack.
enum CtxStorage {
    Win(MaybeUninit<[u8; SWITCHWIN_SIZE]>),
    Buf(MaybeUninit<[u8; ACO_SAVE_SIZE]>),
    None,
}

impl CtxStorage {
    fn as_ptr(&mut self) -> *mut c_void {
        match self {
            Self::Win(ref mut s) => s.as_mut_ptr().cast(),
            Self::Buf(ref mut s) => s.as_mut_ptr().cast(),
            Self::None => std::ptr::null_mut(),
        }
    }
}

/// Switch current window/buffer context for option get/set.
///
/// Returns true if the context was actually switched (and must be restored).
/// Sets `err` on failure.
unsafe fn switch_option_context(
    ctx: &mut CtxStorage,
    scope: c_int,
    from: *mut c_void,
    err: *mut Error,
) -> bool {
    match scope {
        K_OPT_SCOPE_WIN => {
            let win = from;
            if std::ptr::eq(win, curwin) {
                return false;
            }
            let sw_ptr = ctx.as_ptr();
            let tabpage = rs_win_find_tabpage(win);
            if nvim_option_switch_win_noblock(sw_ptr, win, tabpage) == FAIL {
                nvim_option_restore_win_noblock(sw_ptr);
                if (*err).err_type != K_ERROR_TYPE_NONE {
                    return false;
                }
                api_set_error(
                    err,
                    K_ERROR_TYPE_EXCEPTION,
                    c"Problem while switching windows".as_ptr(),
                );
                return false;
            }
            true
        }
        K_OPT_SCOPE_BUF => {
            let buf = from;
            if std::ptr::eq(buf, curbuf) {
                return false;
            }
            nvim_option_aucmd_prepbuf(ctx.as_ptr(), buf);
            true
        }
        _ => false,
    }
}

/// Restore the context previously switched by switch_option_context.
unsafe fn restore_option_context(ctx: &mut CtxStorage, scope: c_int) {
    match scope {
        K_OPT_SCOPE_WIN => nvim_option_restore_win_noblock(ctx.as_ptr()),
        K_OPT_SCOPE_BUF => nvim_option_aucmd_restbuf(ctx.as_ptr()),
        _ => {}
    }
}

/// Get option value for buffer / window.
///
/// Rust implementation of C `get_option_value_for`.
#[no_mangle]
pub unsafe extern "C" fn rs_get_option_value_for(
    opt_idx: OptIndex,
    opt_flags: c_int,
    scope: c_int,
    from: *mut c_void,
    err: *mut Error,
) -> OptVal {
    let mut ctx = match scope {
        K_OPT_SCOPE_WIN => CtxStorage::Win(MaybeUninit::uninit()),
        K_OPT_SCOPE_BUF => CtxStorage::Buf(MaybeUninit::uninit()),
        _ => CtxStorage::None,
    };

    let switched = switch_option_context(&mut ctx, scope, from, err);
    if !err.is_null() && (*err).err_type != K_ERROR_TYPE_NONE {
        return OptVal::nil();
    }

    let retv = nvim_option_get_option_value(opt_idx, opt_flags);

    if switched {
        restore_option_context(&mut ctx, scope);
    }

    retv
}

/// Set option value for buffer / window.
///
/// Rust implementation of C `set_option_value_for`.
#[no_mangle]
pub unsafe extern "C" fn rs_set_option_value_for(
    name: *const c_char,
    opt_idx: OptIndex,
    value: OptVal,
    opt_flags: c_int,
    scope: c_int,
    from: *mut c_void,
    err: *mut Error,
) {
    let mut ctx = match scope {
        K_OPT_SCOPE_WIN => CtxStorage::Win(MaybeUninit::uninit()),
        K_OPT_SCOPE_BUF => CtxStorage::Buf(MaybeUninit::uninit()),
        _ => CtxStorage::None,
    };

    let switched = switch_option_context(&mut ctx, scope, from, err);
    if !err.is_null() && (*err).err_type != K_ERROR_TYPE_NONE {
        return;
    }

    let errmsg = nvim_option_set_value_handle_tty(name, opt_idx, value, opt_flags);
    if !errmsg.is_null() {
        api_set_error(err, K_ERROR_TYPE_EXCEPTION, c"%s".as_ptr(), errmsg);
    }

    if switched {
        restore_option_context(&mut ctx, scope);
    }
}
