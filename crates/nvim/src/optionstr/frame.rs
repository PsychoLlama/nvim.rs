//! What the option table hands a `did_set_*` callback, and the three things
//! nearly every one of them does with it.
//!
//! A callback is called *after* the new value is already in the option's
//! variable, and reports a message when it does not like it — the caller
//! then puts the old value back. So a callback that only validates can
//! return early, but one that also updates derived state has to do the
//! validating first.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char};
use core::ptr;

use crate::main::e_invarg;
use crate::types::{optset_T, win_T};

/// "E474: Invalid argument", the message almost every string option's check
/// reports when it has nothing more specific to say.
pub(crate) fn invalid() -> Option<&'static CStr> {
    Some(e_invarg)
}

/// The option's value variable — a `char **`, since every option here is a
/// string.
pub(crate) fn varp(args: &optset_T) -> *mut *mut c_char {
    args.os_varp.string_var()
}

/// The window the set is happening in. Not necessarily the window whose
/// value is being set — see [`local_window`].
pub(crate) fn win(args: &optset_T) -> *mut win_T {
    args.os_win.cast::<win_T>()
}

/// The window whose own copy of the option is being set, or null when the
/// variable is the global copy instead.
///
/// The checks that take a window this way store what they worked out in it
/// when there is one, and only validate when there is not — which is how
/// `:setglobal` on a window-local option is vetted without disturbing any
/// window.
///
/// # Safety
/// `wp` is the window from [`win`] and `local` its own variable for this
/// option; the comparison is of addresses only.
pub(crate) unsafe fn local_window(
    varp: *mut *mut c_char,
    wp: *mut win_T,
    local: *mut *mut c_char,
) -> *mut win_T {
    if varp == local { wp } else { ptr::null_mut() }
}

/// The value the option held before this set, as a C string.
///
/// Every option in this module is a string one, so the frame's old value
/// is a string too; the accessor says so.
pub(crate) fn old_value(args: &optset_T) -> *const c_char {
    args.os_oldval
        .as_string()
        .expect("the table installs this callback on a string option only")
        .data()
}

/// The error buffer and its size, as the message helpers take them.
pub(crate) fn errbuf(args: &optset_T) -> (*mut c_char, usize) {
    (args.os_errbuf, args.os_errbuflen)
}
