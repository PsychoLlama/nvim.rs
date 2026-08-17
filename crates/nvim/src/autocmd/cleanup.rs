//! Deleting autocommands, and the deferred cleanup that makes it safe.
//!
//! An `AutoCmd` is never freed while a firing walk might be holding a
//! pointer to it: [`aucmd_del`] marks it and sets `au_need_clean`, and
//! [`au_cleanup`] does the actual removal once no walk is live.  The
//! `aupat_*` half is the buffer-local pattern format -- `<buffer=N>` --
//! which is a pattern in the same table but matched by buffer number, and
//! [`aubuflocal_remove`] is what a wiped-out buffer's patterns go through.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::smsg_c;

/// Mark one autocommand deleted: free everything it owns and null its
/// pattern, which is the flag [`au_cleanup`] compacts on.
///
/// The row itself stays in place.  A walk may be standing on it, and the
/// pattern list it is walking is indexed, so the row may not move until
/// `autocmd_busy` is clear.
pub(crate) unsafe extern "C" fn aucmd_del(ac: *mut AutoCmd) {
    unsafe {
        // `XFREE_CLEAR`, three times, over `*mut c_char` fields.
        let xfree_clear = |slot: *mut *mut ::core::ffi::c_char| {
            xfree((*slot).cast::<::core::ffi::c_void>());
            *slot = ::core::ptr::null_mut();
        };

        let pat = (*ac).pat;
        if !pat.is_null() {
            (*pat).refcount = (*pat).refcount.wrapping_sub(1);
            // The last autocommand on a pattern takes the pattern with it.
            if (*pat).refcount == 0 {
                xfree_clear(&raw mut (*pat).pat);
                vim_regfree((*pat).reg_prog);
                xfree(pat.cast::<::core::ffi::c_void>());
            }
        }
        (*ac).pat = ::core::ptr::null_mut();

        // A handler is either a command string or a callback, never both.
        if (*ac).handler_cmd.is_null() {
            callback_free(&raw mut (*ac).handler_fn);
        } else {
            xfree_clear(&raw mut (*ac).handler_cmd);
        }
        xfree_clear(&raw mut (*ac).desc);

        au_need_clean.set(true);
    }
}

/// Delete every autocommand `group` defined for `event`.
pub unsafe extern "C" fn aucmd_del_for_event_and_group(event: event_T, group: ::core::ffi::c_int) {
    unsafe {
        let acs = au_event_vec(event);
        let mut i: usize = 0;
        while i < (*acs).size {
            let ac = (*acs).items.add(i);
            if !(*ac).pat.is_null() && (*(*ac).pat).group == group {
                aucmd_del(ac);
            }
            i = i.wrapping_add(1);
        }
        au_cleanup();
    }
}

/// Compact the marked-deleted rows out of every event's list.
///
/// This is the deferred half of [`aucmd_del`] and it is *only* safe while
/// no walk is live: `autocmd_busy` says one is, and then the rows have to
/// stay where a walk's index can still reach them.  Every call site is
/// therefore a point where the editor is between autocommands.
pub(crate) unsafe extern "C" fn au_cleanup() {
    unsafe {
        if autocmd_busy.get() || !au_need_clean.get() {
            return;
        }

        for event in 0..NUM_EVENTS {
            let acs = au_event_vec(event);
            let mut nsize: usize = 0;
            let mut i: usize = 0;
            while i < (*acs).size {
                let ac = (*acs).items.add(i);
                if nsize != i {
                    *(*acs).items.add(nsize) = *ac;
                }
                if !(*ac).pat.is_null() {
                    nsize = nsize.wrapping_add(1);
                }
                i = i.wrapping_add(1);
            }
            if nsize == 0 {
                // `kv_destroy`: an event with nothing left gives its block
                // back rather than keeping the capacity.
                xfree((*acs).items.cast::<::core::ffi::c_void>());
                (*acs).capacity = 0;
                (*acs).size = 0;
                (*acs).items = ::core::ptr::null_mut();
            } else {
                (*acs).size = nsize;
            }
        }

        au_need_clean.set(false);
    }
}

/// The autocommand list for `event`, for the API's readers.
pub unsafe extern "C" fn au_get_autocmds_for_event(event: event_T) -> *mut AutoCmdVec {
    au_event_vec(event)
}

/// Drop every `<buffer=N>` autocommand naming `buf`, which is being freed.
pub unsafe extern "C" fn aubuflocal_remove(buf: *mut buf_T) {
    unsafe {
        // A walk in progress may be about to match on this buffer number;
        // clear it rather than let it match a freed buffer.
        let mut apc = active_apc_list.get();
        while !apc.is_null() {
            if (*buf).handle == (*apc).arg_bufnr {
                (*apc).arg_bufnr = 0;
            }
            apc = (*apc).next;
        }

        for event in 0..NUM_EVENTS {
            let acs = au_event_vec(event);
            let mut i: usize = 0;
            while i < (*acs).size {
                let ac = (*acs).items.add(i);
                if !(*ac).pat.is_null() && (*(*ac).pat).buflocal_nr == (*buf).handle {
                    aucmd_del(ac);
                    if p_verbose.get() >= 6 {
                        verbose_enter();
                        smsg_c!(
                            0,
                            gettext(c"auto-removing autocommand: %s <buffer=%d>".as_ptr()),
                            event_nr2name(event),
                            (*buf).handle,
                        );
                        verbose_leave();
                    }
                }
                i = i.wrapping_add(1);
            }
        }
        au_cleanup();
    }
}

/// Whether `pat` is one of the buffer-local pattern spellings:
/// `<buffer>`, `<buffer=N>` or `<buffer=abuf>`.
pub unsafe extern "C" fn aupat_is_buflocal(
    pat: *const ::core::ffi::c_char,
    patlen: ::core::ffi::c_int,
) -> bool {
    unsafe {
        patlen >= 8
            && strncmp(pat, c"<buffer".as_ptr(), 7) == 0
            && *pat.add(patlen as usize - 1) == b'>' as ::core::ffi::c_char
    }
}

/// The buffer number a buffer-local pattern names, or 0 when it names one
/// that cannot be resolved.
pub unsafe extern "C" fn aupat_get_buflocal_nr(
    pat: *const ::core::ffi::c_char,
    patlen: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        debug_assert!(aupat_is_buflocal(pat, patlen));

        // "<buffer>"
        if patlen == 8 {
            return (*curbuf.get()).handle;
        }

        if patlen > 9 && *pat.add(7) == b'=' as ::core::ffi::c_char {
            // "<buffer=abuf>"
            if patlen == 13 && strncasecmp(pat, c"<buffer=abuf>".as_ptr(), 13) == 0 {
                return autocmd_bufnr.get();
            }
            // "<buffer=123>": digits, and nothing but digits, up to the '>'.
            if skipdigits(pat.add(8)) == pat.add(patlen as usize - 1).cast_mut() {
                return atoi(pat.add(8));
            }
        }

        0
    }
}

/// Write the canonical `<buffer=N>` spelling of a buffer-local pattern
/// into `dest`, which must hold `BUFLOCAL_PAT_LEN` bytes.
pub unsafe extern "C" fn aupat_normalize_buflocal_pat(
    dest: *mut ::core::ffi::c_char,
    pat: *const ::core::ffi::c_char,
    patlen: ::core::ffi::c_int,
    mut buflocal_nr: ::core::ffi::c_int,
) {
    unsafe {
        debug_assert!(aupat_is_buflocal(pat, patlen));

        if buflocal_nr == 0 {
            buflocal_nr = (*curbuf.get()).handle;
        }
        snprintf(
            dest,
            BUFLOCAL_PAT_LEN as size_t,
            c"<buffer=%d>".as_ptr(),
            buflocal_nr,
        );
    }
}
