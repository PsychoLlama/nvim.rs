//! Changing directory, and telling autocommands about it.
//!
//! [`vim_chdir`] resolves a relative directory name along `'cdpath'` before
//! changing to it, [`vim_chdirfile`] changes to a file's own directory, and
//! [`do_autocmd_dirchanged`] fires `DirChangedPre`/`DirChanged` with the
//! `v:event` dictionary those events promise.

#![forbid(unsafe_code)]
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
use crate::memory::XString;
use crate::types::{
    Failed, MAXPATHL, kCdScopeGlobal, kCdScopeInvalid, kCdScopeTabpage, kCdScopeWindow,
};
use crate::winlayer::Buf;
use core::ffi::c_int;
use std::ffi::CStr;

/// Fire `DirChangedPre` (`pre`) or `DirChanged` for a move to `new_dir`.
///
/// The autocommand pattern is the scope's name — `"global"`, `"tabpage"` or
/// `"window"` — except for an automatic change, which is matched by `"auto"`.
/// The same words go into `v:event.scope`, alongside the new directory under
/// the key the event promises (`directory` before the move, `cwd` after).
pub(crate) fn do_autocmd_dirchanged(new_dir: &CStr, scope: CdScope, cause: CdCause, pre: bool) {
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
        kCdScopeInvalid => std::process::abort(),
        _ => c"",
    };
    let pattern: &CStr = match cause {
        kCdCauseAuto => c"auto",
        kCdCauseOther => std::process::abort(), // "Should never happen."
        _ => scope_name,                        // manual, or following a window change
    };

    let mut saved = SaveVEvent::default();
    let dict = get_v_event(&mut saved);
    let key: &CStr = if pre { c"directory" } else { c"cwd" };
    let _ = dict.edit().add_str(key.to_bytes(), Some(new_dir));
    let _ = dict.edit().add_str(b"scope", Some(scope_name));
    let _ = dict.edit().add_bool(
        b"changed_window",
        BoolVarValue::from(cause == kCdCauseWindow),
    );
    dict.edit().set_keys_readonly();

    fire_autocmds_for(
        event,
        Some(pattern),
        Some(new_dir),
        false,
        Buf::current_or_none(),
        None,
    );

    restore_v_event(dict, &mut saved);
    RECURSIVE.set(false);
}

/// Change to the directory holding `fname`.
///
/// Caller must call `shorten_fnames()`.
pub(crate) fn vim_chdirfile(fname: &CStr, cause: CdCause) -> Result<(), Failed> {
    let name = fname.to_bytes();
    let mut dir = name[..name.len().min(MAXPATHL as usize - 1)].to_vec();
    dir.truncate(tail_with_sep_index(&dir));
    let dir = cstr::owned(&dir);

    let cwd = dirname_within(MAXPATHL as usize).unwrap_or_default();
    if pathcmp(dir.to_bytes(), &cwd, None) == 0 {
        return Ok(()); // nothing to do
    }

    let announce = cause != kCdCauseOther;
    if announce {
        do_autocmd_dirchanged(&dir, kCdScopeWindow, cause, true);
    }
    if os_chdir(&dir) != 0 {
        return Err(Failed);
    }
    if announce {
        do_autocmd_dirchanged(&dir, kCdScopeWindow, cause, false);
    }
    Ok(())
}

/// Change directory to `new_dir`, searching `'cdpath'` for a relative name.
pub(crate) fn vim_chdir(new_dir: &CStr) -> c_int {
    // A copy: the search can run user code, which may rename the buffer.
    let rel_fname = Buf::current().name.full().map(XString::from_cstr);
    let rel_fname = rel_fname.as_ref().map(XString::as_cstr);
    match find_directory_in_path(new_dir.to_bytes(), FileNameOpts::MESS, rel_fname) {
        Some(dir_name) => os_chdir(dir_name.as_cstr()),
        None => -1,
    }
}
