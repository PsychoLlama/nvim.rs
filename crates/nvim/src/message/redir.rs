//! Where a message goes besides the screen.
//!
//! `:redir` (to a variable, a register or a file) and `'verbosefile'` both
//! tee the message stream; [`redir_write`] is the tee, and the `verbose_*`
//! pair brackets the sections of code that write to it.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::semsg_c;
use crate::types::{FAIL, OK};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// The `msg_ext` kind a verbose message carries.
///
/// [`verbose_enter`] compares `msg_ext_kind` against this **by pointer** to
/// recognise a verbose section it is already inside. The C got that identity
/// from its compiler pooling two occurrences of the same string literal, so
/// this is one named constant rather than two literals: the guard then holds
/// by construction instead of by codegen.
const VERBOSE_KIND: &CStr = c"verbose";

/// The message kind in force when the current verbose section started.
static pre_verbose_kind: GlobalCell<*const c_char> = GlobalCell::new(ptr::null());

/// The `'verbosefile'` handle, opened lazily by [`verbose_open`].
static verbose_fd: GlobalCell<*mut FILE> = GlobalCell::new(ptr::null_mut());

/// Whether opening `'verbosefile'` has been attempted, so the failure is
/// reported once rather than on every message.
static verbose_did_open: GlobalCell<bool> = GlobalCell::new(false);

/// The column [`redir_write`] has written up to, tracked separately from
/// `msg_col` because the redirection sees no screen.
pub(crate) static redir_col: GlobalCell<c_int> = GlobalCell::new(0);

/// Is `'verbosefile'` set to anything?
///
/// # Safety
/// Only that `p_vfile` holds a valid string, which the option code
/// guarantees.
unsafe fn verbosefile_set() -> bool {
    unsafe { *p_vfile.get() != 0 }
}

/// [`msg_keep`] inside a `verbose_enter`/`verbose_leave` pair.
///
/// # Safety
/// `s` must be a valid C string.
pub unsafe fn verb_msg(s: *const c_char) -> c_int {
    unsafe { verbose_enter() };
    let n = unsafe { msg_keep(s, 0, false, false) as c_int };
    unsafe { verbose_leave() };
    n
}

/// Copy a message to `:redir`'s destination and to `'verbosefile'`.
///
/// `maxlen` is the byte count to write, or -1 for the whole of `str`.
///
/// # Safety
/// `str` must be a valid C string, readable for `maxlen` bytes when that is
/// not negative.
pub(crate) unsafe fn redir_write(str: *const c_char, maxlen: ptrdiff_t) {
    if maxlen == 0 {
        return;
    }
    // Don't do anything for displaying prompts and the like.
    if redir_off.get() {
        return;
    }
    // If 'verbosefile' is set prepare for writing in that file.
    if unsafe { verbosefile_set() } && verbose_fd.get().is_null() {
        unsafe { verbose_open() };
    }
    if !unsafe { redirecting() } {
        return;
    }

    // One space to every sink this message is going to. A closure rather
    // than a fn: it inherits the enclosing `unsafe` block, where a
    // separate `unsafe`-declared fn would need one of its own.
    let pad = || {
        if !capture_ga.get().is_null() {
            unsafe { ga_concat_len(capture_ga.get(), c" ".as_ptr(), 1) };
        }
        if redir_reg.get() != 0 {
            unsafe { write_reg_contents(redir_reg.get(), c" ".as_ptr(), 1, 1) };
        } else if redir_vname.get() {
            unsafe { var_redir_str(c" ".as_ptr(), -1) };
        } else if !redir_fd.get().is_null() {
            unsafe { fputs(c" ".as_ptr(), redir_fd.get()) };
        }
        if !verbose_fd.get().is_null() {
            unsafe { fputs(c" ".as_ptr(), verbose_fd.get()) };
        }
    };

    // If the string doesn't start with CR or NL, go to msg_col.
    if unsafe { *str } != b'\n' as c_char && unsafe { *str } != b'\r' as c_char {
        while redir_col.get() < msg_col.get() {
            pad();
            redir_col.set(redir_col.get() + 1);
        }
    }

    let len = if maxlen == -1 {
        unsafe { strlen(str) }
    } else {
        maxlen as size_t
    };
    if !capture_ga.get().is_null() {
        unsafe { ga_concat_len(capture_ga.get(), str, len) };
    }
    if redir_reg.get() != 0 {
        unsafe { write_reg_contents(redir_reg.get(), str, len as ssize_t, 1) };
    }
    if redir_vname.get() {
        unsafe { var_redir_str(str, maxlen as c_int) };
    }

    // Write and adjust the current column. The file sinks are fed byte by
    // byte because the column has to be tracked byte by byte anyway.
    let mut s = str;
    while unsafe { *s } != 0
        && (maxlen < 0 || (unsafe { s.offset_from(str) as c_int as ptrdiff_t }) < maxlen)
    {
        if redir_reg.get() == 0
            && !redir_vname.get()
            && capture_ga.get().is_null()
            && !redir_fd.get().is_null()
        {
            unsafe { putc(*s as c_int, redir_fd.get()) };
        }
        if !verbose_fd.get().is_null() {
            unsafe { putc(*s as c_int, verbose_fd.get()) };
        }
        match unsafe { *s as u8 } {
            b'\r' | b'\n' => redir_col.set(0),
            b'\t' => redir_col.set(redir_col.get() + 8 - redir_col.get() % 8),
            _ => redir_col.set(redir_col.get() + 1),
        }
        s = unsafe { s.add(1) };
    }

    if msg_silent.get() != 0 {
        // Should update msg_col.
        msg_col.set(redir_col.get());
    }
}

/// Is anything teeing the message stream?
///
/// # Safety
/// Only that `p_vfile` holds a valid string.
pub unsafe fn redirecting() -> bool {
    !redir_fd.get().is_null()
        || unsafe { verbosefile_set() }
        || redir_reg.get() != 0
        || redir_vname.get()
        || !capture_ga.get().is_null()
}

/// Before giving a verbose message. Must always be paired with
/// [`verbose_leave`].
///
/// # Safety
/// Only that `p_vfile` holds a valid string.
pub unsafe fn verbose_enter() {
    if unsafe { verbosefile_set() } {
        msg_silent.set(msg_silent.get() + 1);
    }
    // Don't set the verbose kind if message continuity is wanted, as with
    // last_set_msg().
    if !msg_ext_skip_verbose.get() {
        if msg_ext_kind.get() != VERBOSE_KIND.as_ptr() {
            pre_verbose_kind.set(msg_ext_kind.get());
        }
        unsafe { msg_ext_set_kind(VERBOSE_KIND.as_ptr()) };
    }
    msg_ext_skip_verbose.set(false);
}

/// After giving a verbose message. Must always be paired with
/// [`verbose_enter`].
///
/// # Safety
/// Only that `p_vfile` holds a valid string.
pub unsafe fn verbose_leave() {
    if unsafe { verbosefile_set() } {
        msg_silent.set(msg_silent.get() - 1);
        if msg_silent.get() < 0 {
            msg_silent.set(0);
        }
    }
    if !pre_verbose_kind.get().is_null() {
        unsafe { msg_ext_set_kind(pre_verbose_kind.get()) };
        pre_verbose_kind.set(ptr::null());
    }
}

/// [`verbose_enter`], and scroll rather than overwrite when the message is
/// going to be displayed.
///
/// # Safety
/// See [`verbose_enter`].
pub unsafe fn verbose_enter_scroll() {
    unsafe { verbose_enter() };
    if !unsafe { verbosefile_set() } {
        // Always scroll up, don't overwrite.
        msg_scroll.set(1);
    }
}

/// [`verbose_leave`], and leave the command line below a displayed message.
///
/// # Safety
/// See [`verbose_leave`].
pub unsafe fn verbose_leave_scroll() {
    unsafe { verbose_leave() };
    if !unsafe { verbosefile_set() } {
        cmdline_row.set(msg_row.get());
    }
}

/// `'verbosefile'` changed: stop writing to the old one.
///
/// # Safety
/// Only that no other thread is using the handle.
pub unsafe fn verbose_stop() {
    if !verbose_fd.get().is_null() {
        unsafe { fclose(verbose_fd.get()) };
        verbose_fd.set(ptr::null_mut());
    }
    verbose_did_open.set(false);
}

/// Open `'verbosefile'` for appending, once.
///
/// # Safety
/// Only that `p_vfile` holds a valid string.
pub unsafe fn verbose_open() -> c_int {
    if verbose_fd.get().is_null() && !verbose_did_open.get() {
        // Only give the error message once.
        verbose_did_open.set(true);
        verbose_fd.set(unsafe { os_fopen(p_vfile.get(), c"a".as_ptr()) });
        if verbose_fd.get().is_null() {
            let msg = gettext(e_notopen);
            unsafe { semsg_c!(msg, p_vfile.get()) };
            return FAIL;
        }
    }
    OK
}
