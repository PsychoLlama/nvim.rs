//! Where a message goes besides the screen.
//!
//! `:redir` (to a variable, a register or a file) and `'verbosefile'` both
//! tee the message stream; [`redir_write`] is the tee, and the `verbose_*`
//! pair brackets the sections of code that write to it.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::message_fmt::msg_cstr;
use crate::option::vars::P_VFILE;
use crate::os::fs::CFile;
use crate::semsg;
use crate::types::Failed;
use core::ffi::{CStr, c_int};

/// The `msg_ext` kind a verbose message carries.
///
/// [`verbose_enter`] compares `MSG_EXT_KIND` against this to recognise a
/// verbose section it is already inside.
const VERBOSE_KIND: &CStr = c"verbose";

/// Is `'verbosefile'` set to anything?
fn verbosefile_set() -> bool {
    P_VFILE.first_byte() != 0
}

/// [`msg`] inside a `verbose_enter`/`verbose_leave` pair.
pub fn verb_msg(s: &CStr) -> c_int {
    verbose_enter();
    let n = c_int::from(msg(s, 0));
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
    if REDIR_OFF.get() {
        return;
    }
    // If 'verbosefile' is set prepare for writing in that file.
    if verbosefile_set() && VERBOSE_FD.with(Option::is_none) {
        let _ = verbose_open();
    }
    if !redirecting() {
        return;
    }

    // One space to every sink this message is going to.
    let pad = || {
        capture_bytes(b" ");
        if REDIR_REG.get() != 0 {
            write_reg_contents(REDIR_REG.get(), b" ", true);
        } else if REDIR_VNAME.get() {
            var_redir_str(b" ");
        } else {
            REDIR_FD.with(|file| file.as_ref().map(|file| file.putc(b' ')));
        }
        VERBOSE_FD.with(|file| file.as_ref().map(|file| file.putc(b' ')));
    };

    // If the string doesn't start with CR or NL, go to msg_col.
    if !matches!(bytes[0], b'\n' | b'\r') {
        while REDIR_COL.get() < MSG_COL.get() {
            pad();
            REDIR_COL.set(REDIR_COL.get() + 1);
        }
    }

    capture_bytes(bytes);
    if REDIR_REG.get() != 0 {
        write_reg_contents(REDIR_REG.get(), bytes, true);
    }
    if REDIR_VNAME.get() {
        var_redir_str(bytes);
    }

    // Write and adjust the current column. The file sinks are fed byte by
    // byte because the column has to be tracked byte by byte anyway.
    let to_redir_fd =
        REDIR_REG.get() == 0 && !REDIR_VNAME.get() && MSG_CAPTURE.with(Option::is_none);
    for &byte in bytes {
        if to_redir_fd {
            REDIR_FD.with(|file| file.as_ref().map(|file| file.putc(byte)));
        }
        VERBOSE_FD.with(|file| file.as_ref().map(|file| file.putc(byte)));
        match byte {
            b'\r' | b'\n' => REDIR_COL.set(0),
            b'\t' => REDIR_COL.set(REDIR_COL.get() + 8 - REDIR_COL.get() % 8),
            _ => REDIR_COL.set(REDIR_COL.get() + 1),
        }
    }

    if MSG_SILENT.get() != 0 {
        // Should update msg_col.
        MSG_COL.set(REDIR_COL.get());
    }
}

/// Is anything teeing the message stream?
pub fn redirecting() -> bool {
    REDIR_FD.with(Option::is_some)
        || verbosefile_set()
        || REDIR_REG.get() != 0
        || REDIR_VNAME.get()
        || MSG_CAPTURE.with(Option::is_some)
}

/// Append `bytes` to the capture, if one is running.
fn capture_bytes(bytes: &[u8]) {
    MSG_CAPTURE.update(|buffer| {
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
    MSG_CAPTURE.replace(Some(Vec::new()))
}

/// Stop capturing, put back the capture [`capture_start`] interrupted, and
/// answer what was collected.
pub(crate) fn capture_finish(outer: Option<Vec<u8>>) -> Vec<u8> {
    MSG_CAPTURE.replace(outer).unwrap_or_default()
}

/// Before giving a verbose message. Must always be paired with
/// [`verbose_leave`].
pub fn verbose_enter() {
    if verbosefile_set() {
        MSG_SILENT.set(MSG_SILENT.get() + 1);
    }
    // Don't set the verbose kind if message continuity is wanted, as with
    // last_set_msg().
    if !MSG_EXT_SKIP_VERBOSE.get() {
        if MSG_EXT_KIND.with(|kind| kind.as_bytes() != VERBOSE_KIND.to_bytes()) {
            PRE_VERBOSE_KIND.set(MSG_EXT_KIND.with(ApiString::clone));
        }
        msg_ext_set_kind(VERBOSE_KIND);
    }
    MSG_EXT_SKIP_VERBOSE.set(false);
}

/// After giving a verbose message. Must always be paired with
/// [`verbose_enter`].
pub fn verbose_leave() {
    if verbosefile_set() {
        MSG_SILENT.set(MSG_SILENT.get() - 1);
        if MSG_SILENT.get() < 0 {
            MSG_SILENT.set(0);
        }
    }
    let previous = PRE_VERBOSE_KIND.take();
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
        MSG_SCROLL.set(1);
    }
}

/// [`verbose_leave`], and leave the command line below a displayed message.
pub fn verbose_leave_scroll() {
    verbose_leave();
    if !verbosefile_set() {
        CMDLINE_ROW.set(MSG_ROW.get());
    }
}

/// `'verbosefile'` changed: stop writing to the old one.
pub fn verbose_stop() {
    VERBOSE_FD.set(None);
    VERBOSE_DID_OPEN.set(false);
}

/// Open `'verbosefile'` for appending, once.
pub fn verbose_open() -> Result<(), Failed> {
    if VERBOSE_FD.with(Option::is_none) && !VERBOSE_DID_OPEN.get() {
        // Only give the error message once.
        VERBOSE_DID_OPEN.set(true);
        VERBOSE_FD.set(p_vfile(|value| CFile::open(value, c"a")));
        if VERBOSE_FD.with(Option::is_none) {
            let name = p_vfile(CStr::to_owned);
            let name = msg_cstr(&name);
            semsg!("E484: Can't open file {name}");
            return Err(Failed);
        }
    }
    Ok(())
}
