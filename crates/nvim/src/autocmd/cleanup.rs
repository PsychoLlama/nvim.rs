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
use crate::winlayer::Buf;

/// Mark one autocommand deleted: free everything it owns and null its
/// pattern, which is the flag [`au_cleanup`] compacts on.
///
/// The row itself stays in place.  A walk may be standing on it, and the
/// pattern list it is walking is indexed, so the row may not move until
/// `autocmd_busy` is clear.
pub(crate) unsafe fn aucmd_del(ac: *mut AutoCmd) {
    // `XFREE_CLEAR`, three times, over `*mut c_char` fields.
    let xfree_clear = |slot: *mut *mut ::core::ffi::c_char| {
        unsafe { xfree((*slot).cast::<::core::ffi::c_void>()) };
        unsafe { *slot = ::core::ptr::null_mut() };
    };

    let pat = unsafe { (*ac).pat };
    if !pat.is_null() {
        // The last autocommand on a pattern takes the pattern with it.
        if unsafe { (*pat).refcount.release() } == 0 {
            xfree_clear(unsafe { &raw mut (*pat).pat });
            unsafe { vim_regfree((*pat).reg_prog) };
            unsafe { xfree(pat.cast::<::core::ffi::c_void>()) };
        }
    }
    unsafe { (*ac).pat = ::core::ptr::null_mut() };

    // A handler is either a command string or a callback, never both.
    if unsafe { (*ac).handler_cmd.is_null() } {
        unsafe { callback_free(&raw mut (*ac).handler_fn) };
    } else {
        xfree_clear(unsafe { &raw mut (*ac).handler_cmd });
    }
    xfree_clear(unsafe { &raw mut (*ac).desc });

    au_need_clean.set(true);
}

/// Delete every autocommand `group` defined for `event`.
///
/// Safe: it takes an event and a group id, not a pointer, and reaches the
/// rows only through the event's own vector.
pub fn aucmd_del_for_event_and_group(event: event_T, group: ::core::ffi::c_int) {
    let acs = au_event_vec(event);
    let mut i: usize = 0;
    while i < unsafe { (*acs).size } {
        let ac = unsafe { (*acs).items.add(i) };
        if !unsafe { (*ac).pat.is_null() } && unsafe { (*(*ac).pat).group } == group {
            unsafe { aucmd_del(ac) };
        }
        i = i.wrapping_add(1);
    }
    au_cleanup();
}

/// Compact the marked-deleted rows out of every event's list.
///
/// This is the deferred half of [`aucmd_del`] and it may only run while no
/// walk is live: `autocmd_busy` says one is, and then the rows have to stay
/// where a walk's index can still reach them.  Safe to call anywhere,
/// because that is the first thing it checks -- a call made during a walk
/// returns having done nothing, and the compaction waits for the point
/// where the editor is between autocommands.
pub(crate) fn au_cleanup() {
    if autocmd_busy.get() || !au_need_clean.get() {
        return;
    }

    for event in 0..NUM_EVENTS {
        let acs = au_event_vec(event);
        let mut nsize: usize = 0;
        let mut i: usize = 0;
        while i < unsafe { (*acs).size } {
            let ac = unsafe { (*acs).items.add(i) };
            if nsize != i {
                // A move, not a duplication: the source slot is either
                // this one or one the loop has already passed over, and
                // everything past `nsize` is dropped by the truncation
                // below.
                unsafe { *(*acs).items.add(nsize) = (*ac).clone() };
            }
            if !unsafe { (*ac).pat.is_null() } {
                nsize = nsize.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
        if nsize == 0 {
            // `kv_destroy`: an event with nothing left gives its block
            // back rather than keeping the capacity.
            unsafe { xfree((*acs).items.cast::<::core::ffi::c_void>()) };
            unsafe { (*acs).capacity = 0 };
            unsafe { (*acs).size = 0 };
            unsafe { (*acs).items = ::core::ptr::null_mut() };
        } else {
            unsafe { (*acs).size = nsize };
        }
    }

    au_need_clean.set(false);
}

/// The autocommand list for `event`, for the API's readers.
pub fn au_get_autocmds_for_event(event: event_T) -> *mut AutoCmdVec {
    au_event_vec(event)
}

/// Drop every `<buffer=N>` autocommand naming `buf`, which is being freed.
pub unsafe fn aubuflocal_remove(buf: Buf) {
    // A walk in progress may be about to match on this buffer number;
    // clear it rather than let it match a freed buffer.
    let mut apc = active_apc_list.get();
    while !apc.is_null() {
        if buf.handle == unsafe { (*apc).arg_bufnr } {
            unsafe { (*apc).arg_bufnr = 0 };
        }
        apc = unsafe { (*apc).next };
    }

    for event in 0..NUM_EVENTS {
        let acs = au_event_vec(event);
        let mut i: usize = 0;
        while i < unsafe { (*acs).size } {
            let ac = unsafe { (*acs).items.add(i) };
            if !unsafe { (*ac).pat.is_null() } && unsafe { (*(*ac).pat).buflocal_nr } == buf.handle
            {
                unsafe { aucmd_del(ac) };
                if p_verbose.get() >= 6 {
                    unsafe { verbose_enter() };
                    // SAFETY: the message macros expand to a `vim_snprintf` over the
                    // format literal above and the editor's message buffers.
                    unsafe {
                        smsg_c!(
                            0,
                            gettext(c"auto-removing autocommand: %s <buffer=%d>".as_ptr()),
                            event_nr2name(event),
                            buf.handle,
                        )
                    };
                    unsafe { verbose_leave() };
                }
            }
            i = i.wrapping_add(1);
        }
    }
    au_cleanup();
}

/// Whether `pat` is one of the buffer-local pattern spellings:
/// `<buffer>`, `<buffer=N>` or `<buffer=abuf>`.
pub unsafe fn aupat_is_buflocal(
    pat: *const ::core::ffi::c_char,
    patlen: ::core::ffi::c_int,
) -> bool {
    patlen >= 8
        && unsafe { strncmp(pat, c"<buffer".as_ptr(), 7) } == 0
        && unsafe { *pat.add(patlen as usize - 1) } == b'>' as ::core::ffi::c_char
}

/// The buffer number a buffer-local pattern names, or 0 when it names one
/// that cannot be resolved.
pub unsafe fn aupat_get_buflocal_nr(
    pat: *const ::core::ffi::c_char,
    patlen: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    debug_assert!(unsafe { aupat_is_buflocal(pat, patlen) });

    // "<buffer>"
    if patlen == 8 {
        return cur_buf().handle;
    }

    if patlen > 9 && unsafe { *pat.add(7) } == b'=' as ::core::ffi::c_char {
        // "<buffer=abuf>"
        if patlen == 13 && unsafe { strncasecmp(pat, c"<buffer=abuf>".as_ptr(), 13) } == 0 {
            return autocmd_bufnr.get();
        }
        // "<buffer=123>": digits, and nothing but digits, up to the '>'.
        if unsafe { skipdigits(pat.add(8)) } == unsafe { pat.add(patlen as usize - 1) }.cast_mut() {
            return unsafe { atoi(pat.add(8)) };
        }
    }

    0
}

/// Write the canonical `<buffer=N>` spelling of a buffer-local pattern
/// into `dest`, which must hold `BUFLOCAL_PAT_LEN` bytes.
pub unsafe fn aupat_normalize_buflocal_pat(
    dest: *mut ::core::ffi::c_char,
    pat: *const ::core::ffi::c_char,
    patlen: ::core::ffi::c_int,
    mut buflocal_nr: ::core::ffi::c_int,
) {
    debug_assert!(unsafe { aupat_is_buflocal(pat, patlen) });

    if buflocal_nr == 0 {
        buflocal_nr = cur_buf().handle;
    }
    unsafe {
        snprintf(
            dest,
            BUFLOCAL_PAT_LEN as size_t,
            c"<buffer=%d>".as_ptr(),
            buflocal_nr,
        )
    };
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
