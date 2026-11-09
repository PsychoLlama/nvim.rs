//! Changing directory, and telling autocommands about it.
//!
//! [`vim_chdir`] resolves a relative directory name along `'cdpath'` before
//! changing to it, [`vim_chdirfile`] changes to a file's own directory, and
//! [`do_autocmd_dirchanged`] fires `DirChangedPre`/`DirChanged` with the
//! `v:event` dictionary those events promise.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::cstr;
use crate::fileio::IoFailed;
use crate::memory::XString;
use crate::path::MAXPATHL;
use crate::types::{CD_SCOPE_GLOBAL, CD_SCOPE_INVALID, CD_SCOPE_TABPAGE, CD_SCOPE_WINDOW};
use crate::winlayer::Buf;
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
        CD_SCOPE_GLOBAL => c"global",
        CD_SCOPE_TABPAGE => c"tabpage",
        CD_SCOPE_WINDOW => c"window",
        // "Should never happen." Any other value falls through upstream's
        // switch, leaving the buffer it would have named unwritten.
        CD_SCOPE_INVALID => std::process::abort(),
        _ => c"",
    };
    let pattern: &CStr = match cause {
        CD_CAUSE_AUTO => c"auto",
        CD_CAUSE_OTHER => std::process::abort(), // "Should never happen."
        _ => scope_name,                         // manual, or following a window change
    };

    let mut saved = SaveVEvent::default();
    let dict = get_v_event(&mut saved);
    let key: &CStr = if pre { c"directory" } else { c"cwd" };
    let _ = dict.edit().add_str(key.to_bytes(), Some(new_dir));
    let _ = dict.edit().add_str(b"scope", Some(scope_name));
    let _ = dict.edit().add_bool(
        b"changed_window",
        BoolVarValue::from(cause == CD_CAUSE_WINDOW),
    );
    dict.edit().set_keys_readonly();

    fire_autocmds_for(
        event,
        Some(XString::from_cstr(pattern)),
        Some(XString::from_cstr(new_dir)),
        false,
        Buf::current_or_none(),
        None,
    );

    restore_v_event(dict, &mut saved);
    RECURSIVE.set(false);
}

/// Change to the directory holding `fname`.
///
/// Caller must call `shorten_fnames()`. Nothing is said when the change
/// fails.
pub(crate) fn vim_chdirfile(fname: &CStr, cause: CdCause) -> Result<(), IoFailed> {
    let name = fname.to_bytes();
    let mut dir = name[..name.len().min(MAXPATHL as usize - 1)].to_vec();
    dir.truncate(tail_with_sep_index(&dir));
    let dir = cstr::owned(&dir);

    let cwd = dirname_within(MAXPATHL as usize).unwrap_or_default();
    if pathcmp(dir.to_bytes(), &cwd, None) == 0 {
        return Ok(()); // nothing to do
    }

    let announce = cause != CD_CAUSE_OTHER;
    if announce {
        do_autocmd_dirchanged(&dir, CD_SCOPE_WINDOW, cause, true);
    }
    if os_chdir(&dir) != 0 {
        return Err(IoFailed);
    }
    if announce {
        do_autocmd_dirchanged(&dir, CD_SCOPE_WINDOW, cause, false);
    }
    Ok(())
}

/// Change directory to `new_dir`, searching `'cdpath'` for a relative name.
///
/// A name the search cannot find has been said (E344/E345); a directory
/// found but not entered has not. The caller says E472 either way.
pub(crate) fn vim_chdir(new_dir: &CStr) -> Result<(), IoFailed> {
    // A copy: the search can run user code, which may rename the buffer.
    let rel_fname = Buf::current().name.full().map(XString::from_cstr);
    let rel_fname = rel_fname.as_ref().map(XString::as_cstr);
    match find_directory_in_path(new_dir.to_bytes(), FileNameOpts::MESS, rel_fname) {
        Some(dir_name) if os_chdir(dir_name.as_cstr()) == 0 => Ok(()),
        _ => Err(IoFailed),
    }
}
