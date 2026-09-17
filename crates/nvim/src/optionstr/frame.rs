//! What the option table hands a `did_set_*` callback, and the three things
//! nearly every one of them does with it.
//!
//! A callback is called *after* the new value is already in the option's
//! variable, and reports a message when it does not like it — the caller
//! then puts the old value back. So a callback that only validates can
//! return early, but one that also updates derived state has to do the
//! validating first.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::winlayer::Win;
use core::ffi::c_char;

use crate::option::StrVar;

use crate::memory::XString;
use crate::message::e_invarg;
use crate::types::{OptError, OptSet};

/// "E474: Invalid argument", the message almost every string option's check
/// reports when it has nothing more specific to say.
pub(crate) fn invalid() -> Result<(), OptError> {
    Err(e_invarg.into())
}

/// A rejection the callback formats itself, naming what it disliked.
///
/// `fill` is handed [`OptError::ROOM`] bytes plus a terminator, which is the
/// buffer upstream's `errbuf`/`errbuflen` pair named.
pub(crate) fn formatted(fill: impl FnOnce(*mut c_char)) -> Result<(), OptError> {
    Err(XString::filled(OptError::ROOM, fill).into())
}

/// The option's value variable — a string one, since every option here is.
pub(crate) fn varp(args: &OptSet) -> StrVar {
    args.os_varp.string_var()
}

/// The window the set is happening in. Not necessarily the window whose
/// value is being set — see [`local_window`].
pub(crate) fn win(args: &OptSet) -> Win {
    args.os_win
}

/// The window whose own copy of the option is being set, or null when the
/// variable is the global copy instead.
///
/// The checks that take a window this way store what they worked out in it
/// when there is one, and only validate when there is not — which is how
/// `:setglobal` on a window-local option is vetted without disturbing any
/// window.
///
/// `window` is the window from [`win`] and `local` its own variable for this
/// option. Which variable it is, not what it says, so nothing is read.
pub(crate) fn local_window(varp: StrVar, window: Win, local: *mut Option<XString>) -> Option<Win> {
    if varp == StrVar::Local(local) {
        Some(window)
    } else {
        None
    }
}

/// The value the option held before this set, as a C string.
///
/// Every option in this module is a string one, so the frame's old value
/// is a string too; the accessor says so.
pub(crate) fn old_value(args: &OptSet) -> *const c_char {
    args.os_oldval
        .as_string()
        .expect("the table installs this callback on a string option only")
        .data()
}
