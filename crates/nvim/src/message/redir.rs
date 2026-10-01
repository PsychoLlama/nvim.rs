//! Where a message goes besides the screen.
//!
//! `:redir` (to a variable, a register or a file) and `'verbosefile'` both
//! tee the message stream; [`redir_write`] is the tee, and the `verbose_*`
//! pair brackets the sections of code that write to it.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::message_fmt::c_str;
use crate::option::vars::P_VFILE;
use crate::os::fs::CFile;
use crate::semsg;
use crate::types::Failed;
use core::ffi::{CStr, c_char, c_int};

/// The `msg_ext` kind a verbose message carries.
///
/// [`verbose_enter`] compares `msg_ext_kind` against this to recognise a
/// verbose section it is already inside.
const VERBOSE_KIND: &CStr = c"verbose";

/// Is `'verbosefile'` set to anything?
fn verbosefile_set() -> bool {
    P_VFILE.first_byte() != 0
}

/// [`msg_keep`] inside a `verbose_enter`/`verbose_leave` pair.
///
/// # Safety
/// `s` must be a valid C string.
pub fn verb_msg(s: &CStr) -> c_int {
    verbose_enter();
    let n = msg(s, 0) as c_int;
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
    if verbosefile_set() && verbose_fd.with(Option::is_none) {
        let _ = verbose_open();
    }
    // SAFETY: as above.
    if !redirecting() {
        return;
    }

    // One space to every sink this message is going to.
    let pad = || {
        capture_bytes(b" ");
        if redir_reg.get() != 0 {
            // SAFETY: a one-byte literal.
            unsafe { write_reg_contents(redir_reg.get(), c" ".as_ptr(), 1, 1) };
        } else if redir_vname.get() {
            // SAFETY: as above.
            unsafe { var_redir_str(c" ".as_ptr(), -1) };
        } else {
            redir_fd.with(|file| file.as_ref().map(|file| file.putc(b' ')));
        }
        verbose_fd.with(|file| file.as_ref().map(|file| file.putc(b' ')));
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
    capture_bytes(bytes);
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
    let to_redir_fd =
        redir_reg.get() == 0 && !redir_vname.get() && msg_capture.with(Option::is_none);
    for &byte in bytes {
        if to_redir_fd {
            redir_fd.with(|file| file.as_ref().map(|file| file.putc(byte)));
        }
        verbose_fd.with(|file| file.as_ref().map(|file| file.putc(byte)));
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
    redir_fd.with(Option::is_some)
        || verbosefile_set()
        || redir_reg.get() != 0
        || redir_vname.get()
        || msg_capture.with(Option::is_some)
}

/// Append `bytes` to the capture, if one is running.
fn capture_bytes(bytes: &[u8]) {
    msg_capture.update(|buffer| {
        if let Some(buffer) = buffer {
            buffer.extend_from_slice(bytes);
        }
    });
}

/// Start capturing message output, answering the capture this one
/// interrupts -- which [`capture_finish`] puts back.
///
/// A capture nested inside another collects only its own output, as
/// upstream's: the outer one sees nothing of it.
pub(crate) fn capture_start() -> Option<Vec<u8>> {
    msg_capture.replace(Some(Vec::new()))
}

/// Stop capturing, put back the capture [`capture_start`] interrupted, and
/// answer what was collected.
pub(crate) fn capture_finish(outer: Option<Vec<u8>>) -> Vec<u8> {
    msg_capture.replace(outer).unwrap_or_default()
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
        msg_ext_set_kind(VERBOSE_KIND);
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
        msg_ext_set_kind(previous.as_cstr());
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
    verbose_fd.set(None);
    verbose_did_open.set(false);
}

/// Open `'verbosefile'` for appending, once.
pub fn verbose_open() -> Result<(), Failed> {
    if verbose_fd.with(Option::is_none) && !verbose_did_open.get() {
        // Only give the error message once.
        verbose_did_open.set(true);
        verbose_fd.set(p_vfile(|value| CFile::open(value, c"a")));
        if verbose_fd.with(Option::is_none) {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg0 = p_vfile(|value| unsafe { c_str(value.as_ptr().cast_mut()) });
            semsg!("E484: Can't open file {arg0}");
            return Err(Failed);
        }
    }
    Ok(())
}
