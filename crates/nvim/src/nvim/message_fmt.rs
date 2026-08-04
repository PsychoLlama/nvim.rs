//! Compile-checked message formatting: the Rust-side face of the message.rs
//! entry points.
//!
//! The transpiled tree reports messages through variadic C-style calls
//! (`semsg(fmt, ...)`, `smsg(hl_id, fmt, ...)`) whose format strings the
//! compiler cannot check against their arguments. Rewritten modules migrate
//! those call sites to the [`semsg!`]/[`smsg!`] macros, which format with
//! `format_args!` (checked at compile time) and hand the finished message to
//! the same non-variadic cores (`emsg`, `msg`) the variadic wrappers use.
//!
//! On gettext: the variadic call sites translate their format strings through
//! `gettext()` at runtime, which a compile-time format string cannot express.
//! This tree ships no message catalogs — there is no `po/` directory in the
//! source tree and no `share/locale/` in the installed output — so `gettext`
//! returns its argument unchanged and the macros drop the call instead of
//! preserving a no-op. If catalogs ever come back, translation needs a design
//! that survives Rust format strings; grep for these macros to find every
//! message that opted out.
//!
//! Migrated call sites inline their message text as a Rust string literal
//! (`format_args!` requires a literal, so shared C format constants like
//! `e_trailing_arg` cannot be passed through). The C constant keeps serving
//! the unmigrated callers until its last one migrates.
#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::message::{emsg, emsg_not_now, msg};
use core::ffi::{c_char, c_int};
use core::fmt;

/// Format `args` and report the result as an error message. The Rust-side
/// equivalent of `semsg()`: returns `true` when the message was output or
/// errors are currently suppressed.
pub fn emsg_fmt(args: fmt::Arguments<'_>) -> bool {
    // SAFETY: reads message-state globals; main thread, like every message
    // call. Checked before formatting so suppressed errors cost nothing,
    // matching the variadic wrapper.
    if unsafe { emsg_not_now() } {
        return true;
    }
    let text = to_message(args);
    // SAFETY: `text` is NUL-terminated and outlives the call; emsg copies
    // what it keeps.
    unsafe { emsg(text.as_ptr() as *const c_char) }
}

/// Format `args` and show it as a regular message with `hl_id` highlighting.
/// The Rust-side equivalent of `smsg()`.
pub fn msg_fmt(hl_id: c_int, args: fmt::Arguments<'_>) -> bool {
    let text = to_message(args);
    // SAFETY: `text` is NUL-terminated and outlives the call; msg copies
    // what it keeps.
    unsafe { msg(text.as_ptr() as *const c_char, hl_id) }
}

/// The formatted message as a NUL-terminated buffer. An interior NUL from an
/// argument ends the message there, as it would have in the C caller.
fn to_message(args: fmt::Arguments<'_>) -> String {
    let mut s = args.to_string();
    if let Some(nul) = s.find('\0') {
        s.truncate(nul);
    }
    s.push('\0');
    s
}

/// `semsg()` with a compile-checked format string: report a formatted error
/// message. Evaluates to `bool` like the variadic original.
#[macro_export]
macro_rules! semsg {
    ($($arg:tt)*) => {
        $crate::src::nvim::message_fmt::emsg_fmt(::core::format_args!($($arg)*))
    };
}

/// `smsg()` with a compile-checked format string: show a formatted message
/// with the given highlight id.
#[macro_export]
macro_rules! smsg {
    ($hl_id:expr, $($arg:tt)*) => {
        $crate::src::nvim::message_fmt::msg_fmt($hl_id, ::core::format_args!($($arg)*))
    };
}
