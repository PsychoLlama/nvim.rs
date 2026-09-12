//! Where a message goes besides the screen.
//!
//! `:redir` (to a variable, a register or a file) and `'verbosefile'` both
//! tee the message stream; [`redir_write`] is the tee, and the `verbose_*`
//! pair brackets the sections of code that write to it.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::message_fmt::c_str;
use crate::semsg;
use crate::types::Failed;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// The `msg_ext` kind a verbose message carries.
///
/// [`verbose_enter`] compares `msg_ext_kind` against this to recognise a
/// verbose section it is already inside.
const VERBOSE_KIND: &CStr = c"verbose";

/// The message kind in force when the current verbose section started.
static pre_verbose_kind: GlobalCell<String_0> = GlobalCell::new(String_0::NULL);

/// The `'verbosefile'` handle, opened lazily by [`verbose_open`].
static verbose_fd: GlobalCell<*mut FILE> = GlobalCell::new(ptr::null_mut());

/// Whether opening `'verbosefile'` has been attempted, so the failure is
/// reported once rather than on every message.
static verbose_did_open: GlobalCell<bool> = GlobalCell::new(false);

/// The column [`redir_write`] has written up to, tracked separately from
/// `msg_col` because the redirection sees no screen.
pub(crate) static redir_col: GlobalCell<c_int> = GlobalCell::new(0);

/// Is `'verbosefile'` set to anything?
fn verbosefile_set() -> bool {
    unsafe { *p_vfile.get() != 0 }
}

/// [`msg_keep`] inside a `verbose_enter`/`verbose_leave` pair.
///
/// # Safety
/// `s` must be a valid C string.
pub unsafe fn verb_msg(s: *const c_char) -> c_int {
    verbose_enter();
    let n = unsafe { msg_keep(s, 0, false, false) as c_int };
    verbose_leave();
    n
}

/// Copy a message to `:redir`'s destination and to `'verbosefile'`.
///
/// An empty message is not one: nothing is written, no column is padded and
/// none is tracked. Upstream took that branch only when the caller passed an
/// explicit zero length, and padded for an empty *string* passed with the
/// `-1` length that is now gone.
pub(crate) fn redir_write(bytes: &[u8]) {
    if bytes.is_empty() {
        return;
    }
    // Don't do anything for displaying prompts and the like.
    if redir_off.get() {
        return;
    }
    // If 'verbosefile' is set prepare for writing in that file.
    // SAFETY: `p_vfile` holds a valid option string.
    if verbosefile_set() && verbose_fd.get().is_null() {
        let _ = verbose_open();
    }
    // SAFETY: as above.
    if !redirecting() {
        return;
    }

    // One space to every sink this message is going to.
    let pad = || {
        if !capture_ga.get().is_null() {
            // SAFETY: the cell holds a live growable array.
            unsafe { ga_concat_len(capture_ga.get(), c" ".as_ptr(), 1) };
        }
        if redir_reg.get() != 0 {
            // SAFETY: a one-byte literal.
            unsafe { write_reg_contents(redir_reg.get(), c" ".as_ptr(), 1, 1) };
        } else if redir_vname.get() {
            // SAFETY: as above.
            unsafe { var_redir_str(c" ".as_ptr(), -1) };
        } else if !redir_fd.get().is_null() {
            // SAFETY: the cell holds an open stream.
            unsafe { fputs(c" ".as_ptr(), redir_fd.get()) };
        }
        if !verbose_fd.get().is_null() {
            // SAFETY: as above.
            unsafe { fputs(c" ".as_ptr(), verbose_fd.get()) };
        }
    };

    // If the string doesn't start with CR or NL, go to msg_col.
    if !matches!(bytes[0], b'\n' | b'\r') {
        while redir_col.get() < msg_col.get() {
            pad();
            redir_col.set(redir_col.get() + 1);
        }
    }

    let text = bytes.as_ptr().cast::<c_char>();
    let len = bytes.len();
    if !capture_ga.get().is_null() {
        // SAFETY: the cell holds a live growable array, and `len` bytes
        // follow `text`.
        unsafe { ga_concat_len(capture_ga.get(), text, len) };
    }
    if redir_reg.get() != 0 {
        // SAFETY: as above.
        unsafe { write_reg_contents(redir_reg.get(), text, len as ssize_t, 1) };
    }
    if redir_vname.get() {
        // SAFETY: as above.
        unsafe { var_redir_str(text, len as c_int) };
    }

    // Write and adjust the current column. The file sinks are fed byte by
    // byte because the column has to be tracked byte by byte anyway.
    for &byte in bytes {
        if redir_reg.get() == 0
            && !redir_vname.get()
            && capture_ga.get().is_null()
            && !redir_fd.get().is_null()
        {
            // SAFETY: the cell holds an open stream.
            unsafe { putc(c_int::from(byte), redir_fd.get()) };
        }
        if !verbose_fd.get().is_null() {
            // SAFETY: as above.
            unsafe { putc(c_int::from(byte), verbose_fd.get()) };
        }
        match byte {
            b'\r' | b'\n' => redir_col.set(0),
            b'\t' => redir_col.set(redir_col.get() + 8 - redir_col.get() % 8),
            _ => redir_col.set(redir_col.get() + 1),
        }
    }

    if msg_silent.get() != 0 {
        // Should update msg_col.
        msg_col.set(redir_col.get());
    }
}

/// Is anything teeing the message stream?
pub fn redirecting() -> bool {
    !redir_fd.get().is_null()
        || verbosefile_set()
        || redir_reg.get() != 0
        || redir_vname.get()
        || !capture_ga.get().is_null()
}

/// Before giving a verbose message. Must always be paired with
/// [`verbose_leave`].
pub fn verbose_enter() {
    if verbosefile_set() {
        msg_silent.set(msg_silent.get() + 1);
    }
    // Don't set the verbose kind if message continuity is wanted, as with
    // last_set_msg().
    if !msg_ext_skip_verbose.get() {
        if msg_ext_kind.with(|kind| kind.as_bytes() != VERBOSE_KIND.to_bytes()) {
            pre_verbose_kind.set(msg_ext_kind.with(String_0::clone));
        }
        unsafe { msg_ext_set_kind(VERBOSE_KIND.as_ptr()) };
    }
    msg_ext_skip_verbose.set(false);
}

/// After giving a verbose message. Must always be paired with
/// [`verbose_enter`].
pub fn verbose_leave() {
    if verbosefile_set() {
        msg_silent.set(msg_silent.get() - 1);
        if msg_silent.get() < 0 {
            msg_silent.set(0);
        }
    }
    let previous = pre_verbose_kind.take();
    if !previous.is_null() {
        // SAFETY: an owned, NUL-terminated kind.
        unsafe { msg_ext_set_kind(previous.data()) };
    }
}

/// [`verbose_enter`], and scroll rather than overwrite when the message is
/// going to be displayed.
pub fn verbose_enter_scroll() {
    verbose_enter();
    if !verbosefile_set() {
        // Always scroll up, don't overwrite.
        msg_scroll.set(1);
    }
}

/// [`verbose_leave`], and leave the command line below a displayed message.
pub fn verbose_leave_scroll() {
    verbose_leave();
    if !verbosefile_set() {
        cmdline_row.set(msg_row.get());
    }
}

/// `'verbosefile'` changed: stop writing to the old one.
pub fn verbose_stop() {
    if !verbose_fd.get().is_null() {
        unsafe { fclose(verbose_fd.get()) };
        verbose_fd.set(ptr::null_mut());
    }
    verbose_did_open.set(false);
}

/// Open `'verbosefile'` for appending, once.
pub fn verbose_open() -> Result<(), Failed> {
    if verbose_fd.get().is_null() && !verbose_did_open.get() {
        // Only give the error message once.
        verbose_did_open.set(true);
        verbose_fd.set(unsafe { os_fopen(p_vfile.get(), c"a".as_ptr()) });
        if verbose_fd.get().is_null() {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg0 = unsafe { c_str(p_vfile.get()) };
            semsg!("E484: Can't open file {arg0}");
            return Err(Failed);
        }
    }
    Ok(())
}
