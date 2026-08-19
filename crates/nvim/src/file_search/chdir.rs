//! Changing directory, and telling autocommands about it.
//!
//! [`vim_chdir`] resolves a relative directory name along `'cdpath'` before
//! changing to it, [`vim_chdirfile`] changes to a file's own directory, and
//! [`do_autocmd_dirchanged`] fires `DirChangedPre`/`DirChanged` with the
//! `v:event` dictionary those events promise.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::{FAIL, OK, kCdScopeGlobal, kCdScopeInvalid, kCdScopeTabpage, kCdScopeWindow};
use core::ffi::{c_char, c_int};
use core::ptr;
use std::ffi::CStr;

/// Fire `DirChangedPre` (`pre`) or `DirChanged` for a move to `new_dir`.
///
/// The autocommand pattern is the scope's name — `"global"`, `"tabpage"` or
/// `"window"` — except for an automatic change, which is matched by `"auto"`.
/// The same words go into `v:event.scope`, alongside the new directory under
/// the key the event promises (`directory` before the move, `cwd` after).
pub unsafe fn do_autocmd_dirchanged(
    new_dir: *mut c_char,
    scope: CdScope,
    cause: CdCause,
    pre: bool,
) {
    unsafe {
        // A DirChanged autocommand that changes directory itself must not
        // fire this again.
        static RECURSIVE: GlobalCell<bool> = GlobalCell::new(false);

        let event = if pre {
            EVENT_DIRCHANGEDPRE
        } else {
            EVENT_DIRCHANGED
        } as event_T;
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
            kCdScopeInvalid => abort(),
            _ => c"",
        };
        let pattern: &CStr = match cause {
            kCdCauseAuto => c"auto",
            kCdCauseOther => abort(), // "Should never happen."
            _ => scope_name,          // manual, or following a window change
        };

        let mut saved = save_v_event_T::default();
        let dict = get_v_event(&raw mut saved);
        let key: &CStr = if pre { c"directory" } else { c"cwd" };
        tv_dict_add_str(dict, key.as_ptr(), key.count_bytes(), new_dir);
        tv_dict_add_str(
            dict,
            c"scope".as_ptr(),
            c"scope".count_bytes(),
            scope_name.as_ptr().cast_mut(),
        );
        tv_dict_add_bool(
            dict,
            c"changed_window".as_ptr(),
            c"changed_window".count_bytes(),
            (cause == kCdCauseWindow) as BoolVarValue,
        );
        tv_dict_set_keys_readonly(dict);

        apply_autocmds(
            event,
            pattern.as_ptr().cast_mut(),
            new_dir,
            false,
            curbuf.get(),
        );

        restore_v_event(dict, &raw mut saved);
        RECURSIVE.set(false);
    }
}

/// Change to the directory holding `fname`.
///
/// Caller must call `shorten_fnames()`.
///
/// @return  OK or FAIL
pub unsafe fn vim_chdirfile(fname: *mut c_char, cause: CdCause) -> c_int {
    unsafe {
        let mut dir = [0 as c_char; MAXPATHL];
        xstrlcpy(dir.as_mut_ptr(), fname, MAXPATHL);
        *path_tail_with_sep(dir.as_mut_ptr()) = 0;

        let name_buff = NameBuff.ptr().cast::<c_char>();
        if os_dirname(name_buff, MAXPATHL) != OK {
            *name_buff = 0;
        }
        if pathcmp(dir.as_ptr(), name_buff, -1) == 0 {
            return OK; // nothing to do
        }

        let announce = cause != kCdCauseOther;
        if announce {
            do_autocmd_dirchanged(dir.as_mut_ptr(), kCdScopeWindow, cause, true);
        }
        if os_chdir(dir.as_ptr()) != 0 {
            return FAIL;
        }
        if announce {
            do_autocmd_dirchanged(dir.as_mut_ptr(), kCdScopeWindow, cause, false);
        }
        OK
    }
}

/// Change directory to `new_dir`, searching `'cdpath'` for a relative name.
pub unsafe fn vim_chdir(new_dir: *mut c_char) -> c_int {
    unsafe {
        let mut file_to_find: *mut c_char = ptr::null_mut();
        let mut search_ctx: *mut c_char = ptr::null_mut();
        let dir_name = find_directory_in_path(
            new_dir,
            strlen(new_dir),
            FNAME_MESS as c_int,
            (*curbuf.get()).b_ffname,
            &raw mut file_to_find,
            &raw mut search_ctx,
        );
        xfree(file_to_find.cast());
        vim_findfile_cleanup(search_ctx.cast());

        if dir_name.is_null() {
            return -1;
        }
        let r = os_chdir(dir_name);
        xfree(dir_name.cast());
        r
    }
}
