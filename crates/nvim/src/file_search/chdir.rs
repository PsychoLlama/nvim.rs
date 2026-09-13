//! Changing directory, and telling autocommands about it.
//!
//! [`vim_chdir`] resolves a relative directory name along `'cdpath'` before
//! changing to it, [`vim_chdirfile`] changes to a file's own directory, and
//! [`do_autocmd_dirchanged`] fires `DirChangedPre`/`DirChanged` with the
//! `v:event` dictionary those events promise.

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

use super::*;
use crate::cstr;
use crate::types::{
    Failed, MAXPATHL, kCdScopeGlobal, kCdScopeInvalid, kCdScopeTabpage, kCdScopeWindow,
};
use crate::winlayer::Buf;
use core::ffi::{c_char, c_int};
use core::ptr;
use std::ffi::CStr;

/// Fire `DirChangedPre` (`pre`) or `DirChanged` for a move to `new_dir`.
///
/// The autocommand pattern is the scope's name — `"global"`, `"tabpage"` or
/// `"window"` — except for an automatic change, which is matched by `"auto"`.
/// The same words go into `v:event.scope`, alongside the new directory under
/// the key the event promises (`directory` before the move, `cwd` after).
///
/// # Safety
///
/// `new_dir` must point at a NUL-terminated string, unaliased for the call.
pub(crate) unsafe fn do_autocmd_dirchanged(
    new_dir: *mut c_char,
    scope: CdScope,
    cause: CdCause,
    pre: bool,
) {
    // A DirChanged autocommand that changes directory itself must not
    // fire this again.
    static RECURSIVE: GlobalCell<bool> = GlobalCell::new(false);

    let event = if pre {
        AutoEvent::DirChangedPre
    } else {
        AutoEvent::DirChanged
    } as AutoEvent;
    if RECURSIVE.get() || !has_event(event) {
        return;
    }
    RECURSIVE.set(true);

    let scope_name: &CStr = match scope {
        kCdScopeGlobal => c"global",
        kCdScopeTabpage => c"tabpage",
        kCdScopeWindow => c"window",
        // "Should never happen." Any other value falls through upstream's
        // switch, leaving the buffer it would have named unwritten.
        kCdScopeInvalid => unsafe { abort() },
        _ => c"",
    };
    let pattern: &CStr = match cause {
        kCdCauseAuto => c"auto",
        kCdCauseOther => unsafe { abort() }, // "Should never happen."
        _ => scope_name,                     // manual, or following a window change
    };

    let mut saved = SaveVEvent::default();
    let dict = unsafe { get_v_event(&raw mut saved) };
    let key: &CStr = if pre { c"directory" } else { c"cwd" };
    let _ = unsafe { (*dict).add_str(key.to_bytes(), new_dir) };
    let _ = unsafe { (*dict).add_str(b"scope", scope_name.as_ptr().cast_mut()) };
    let _ = unsafe {
        (*dict).add_bool(
            b"changed_window",
            BoolVarValue::from(cause == kCdCauseWindow),
        )
    };
    unsafe { (*dict).set_keys_readonly() };

    unsafe {
        apply_autocmds(
            event,
            pattern.as_ptr().cast_mut(),
            new_dir,
            false,
            Buf::current_or_none(),
        )
    };

    unsafe { restore_v_event(dict, &raw mut saved) };
    RECURSIVE.set(false);
}

/// Change to the directory holding `fname`.
///
/// Caller must call `shorten_fnames()`.
///
/// @return  `Ok` or `Err`
///
/// # Safety
///
/// `fname` must point at a NUL-terminated string, unaliased for the call.
pub(crate) unsafe fn vim_chdirfile(fname: *mut c_char, cause: CdCause) -> Result<(), Failed> {
    let mut cwd = [0 as c_char; MAXPATHL as usize];
    let mut dir = [0 as c_char; MAXPATHL as usize];
    unsafe { xstrlcpy(dir.as_mut_ptr(), fname, MAXPATHL as usize) };
    unsafe { *path_tail_with_sep(dir.as_mut_ptr()) = 0 };

    let name_buff = cwd.as_mut_ptr();
    if unsafe { os_dirname(name_buff, MAXPATHL as usize) }.is_err() {
        unsafe { *name_buff = 0 };
    }
    if unsafe { pathcmp(dir.as_ptr(), name_buff, -1) } == 0 {
        return Ok(()); // nothing to do
    }

    let announce = cause != kCdCauseOther;
    if announce {
        unsafe { do_autocmd_dirchanged(dir.as_mut_ptr(), kCdScopeWindow, cause, true) };
    }
    if os_chdir(cstr::in_chars(&dir)) != 0 {
        return Err(Failed);
    }
    if announce {
        unsafe { do_autocmd_dirchanged(dir.as_mut_ptr(), kCdScopeWindow, cause, false) };
    }
    Ok(())
}

/// Change directory to `new_dir`, searching `'cdpath'` for a relative name.
///
/// # Safety
///
/// `new_dir` must point at a NUL-terminated string, unaliased for the call.
pub(crate) unsafe fn vim_chdir(new_dir: *mut c_char) -> c_int {
    let mut file_to_find: *mut c_char = ptr::null_mut();
    let mut search_ctx: *mut c_char = ptr::null_mut();
    let dir_len = unsafe { cstr::bytes_at(new_dir) }.len();
    let dir_name = unsafe {
        find_directory_in_path(
            new_dir,
            dir_len,
            FileNameOpts::MESS,
            Buf::current().b_ffname,
            &raw mut file_to_find,
            &raw mut search_ctx,
        )
    };
    unsafe { xfree(file_to_find.cast()) };
    unsafe { vim_findfile_cleanup(search_ctx.cast()) };

    if dir_name.is_null() {
        return -1;
    }
    let r = unsafe { os_chdir(cstr::at(dir_name)) };
    unsafe { xfree(dir_name.cast()) };
    r
}
