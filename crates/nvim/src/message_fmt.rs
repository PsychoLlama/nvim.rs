//! Compile-checked message formatting: the Rust-side face of the message.rs
//! entry points.
//!
//! The transpiled tree reports messages through `printf`-style calls whose
//! format strings the compiler cannot check against their arguments. Two
//! macro families live here, and they are the two ends of that migration:
//!
//! - [`semsg!`]/[`smsg!`] format with `format_args!` — checked at compile
//!   time — and hand the finished message to the same non-variadic cores
//!   (`emsg`, `msg`) the C wrappers used. This is where a call site lands
//!   once its module is rewritten.
//! - [`semsg_c!`] and its seven siblings still speak vim's own `printf`,
//!   formatting through `vim_snprintf` exactly as the C wrapper did. This is
//!   where the ~700 unrewritten call sites sit today, and the `_c` suffix is
//!   how to find them.
//!
//! Both families exist so that the wrappers themselves need not be C
//! *variadic functions*: a variadic call is stable Rust, a variadic
//! definition is not.
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

use crate::message::{emsg_not_now, emsg_ptr, msg_ptr};
use core::ffi::{CStr, c_char, c_int};
use core::fmt;

/// Format `args` and report the result as an error message. The Rust-side
/// equivalent of `semsg()`: returns `true` when the message was output or
/// errors are currently suppressed.
pub(crate) fn emsg_fmt(args: fmt::Arguments<'_>) -> bool {
    // SAFETY: reads message-state globals; main thread, like every message
    // call. Checked before formatting so suppressed errors cost nothing,
    // matching the variadic wrapper.
    if unsafe { emsg_not_now() } {
        return true;
    }
    let text = to_message(args);
    // SAFETY: `text` is NUL-terminated and outlives the call; emsg copies
    // what it keeps.
    unsafe { emsg_ptr(text.as_ptr() as *const c_char) }
}

/// Format `args` and show it as a regular message with `hl_id` highlighting.
/// The Rust-side equivalent of `smsg()`.
pub(crate) fn msg_fmt(hl_id: c_int, args: fmt::Arguments<'_>) -> bool {
    let text = to_message(args);
    // SAFETY: `text` is NUL-terminated and outlives the call; msg copies
    // what it keeps.
    unsafe { msg_ptr(text.as_ptr() as *const c_char, hl_id) }
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

/// The format string one of the `_c` macros was handed, as the pointer
/// [`vim_snprintf`](crate::strings::vim_snprintf) takes.
///
/// A call site writes either `c_fmt!(gettext(c"E1: %s"))` -- a `&CStr`, which
/// is what translation answers now -- or a raw pointer it is still carrying.
/// Both spellings reach `vim_snprintf` the same way; the trait is only here
/// so that one macro body accepts both while the `_c` family retires.
pub(crate) trait CFormat {
    /// The format string as a pointer.
    fn format_ptr(self) -> *const c_char;
}

impl CFormat for &CStr {
    fn format_ptr(self) -> *const c_char {
        self.as_ptr()
    }
}

impl CFormat for *const c_char {
    fn format_ptr(self) -> *const c_char {
        self
    }
}

impl CFormat for *mut c_char {
    fn format_ptr(self) -> *const c_char {
        self.cast_const()
    }
}

/// [`CFormat::format_ptr`] as a function, which is what the macros spell.
#[doc(hidden)]
pub(crate) fn c_format(fmt: impl CFormat) -> *const c_char {
    fmt.format_ptr()
}

/// `semsg()` with a compile-checked format string: report a formatted error
/// message. Evaluates to `bool` like the variadic original.
#[macro_export]
macro_rules! semsg {
    ($($arg:tt)*) => {
        $crate::message_fmt::emsg_fmt(::core::format_args!($($arg)*))
    };
}

/// `smsg()` with a compile-checked format string: show a formatted message
/// with the given highlight id.
#[macro_export]
macro_rules! smsg {
    ($hl_id:expr, $($arg:tt)*) => {
        $crate::message_fmt::msg_fmt($hl_id, ::core::format_args!($($arg)*))
    };
}

// The `_c` family: the same entry points, still speaking vim's own printf
// (the format language `vim_snprintf` implements, which is not Rust's), for
// the call sites that have not been rewritten yet.
//
// Each expands to the two halves the variadic wrapper used to sit between: a
// `vim_snprintf` into the wrapper's own scratch buffer, then the non-variadic
// tail that reports it. That leaves the message bytes, the buffer sizes and
// so the truncation identical, and takes the C-variadic *definitions* — which
// only a nightly compiler can write — out of the tree; a variadic *call* is
// stable Rust.
//
// The `_c` suffix marks a call site as unmigrated, and is how to find them:
// a rewrite replaces `semsg_c!(gettext(c"E1: %s".as_ptr()), p)` with
// `semsg!("E1: {}", …)`, whose format string the compiler checks.
//
// Where errors can be suppressed (`semsg`, `siemsg`, `semsg_multiline`) the
// wrapper tested `emsg_not_now()` *before* formatting, and so does the macro:
// a suppressed error must cost nothing, and the unit specs assert exactly
// that by counting allocations across a `emsg_skip`-guarded call. But its
// caller had already evaluated the arguments by then, so the suppressed arm
// evaluates them too — each expression appears in both arms and runs in
// exactly one of them.

/// `semsg()`: report a `printf`-formatted error. Evaluates to `bool`.
#[macro_export]
macro_rules! semsg_c {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {{
        let fmt = $crate::message_fmt::c_format($fmt);
        if $crate::message::emsg_not_now() {
            $(let _ = $arg;)*
            true
        } else {
            let mut errbuf = $crate::message::semsg_errbuf();
            $crate::strings::vim_snprintf(
                errbuf.as_mut_ptr(),
                $crate::message::SEMSG_ERRBUF_LEN,
                fmt,
                $($arg,)*
            );
            $crate::message::semsg_report(&errbuf)
        }
    }};
}

/// `siemsg()`: report a `printf`-formatted *internal* error. Same effect as
/// [`semsg_c!`] — the name is the intent.
#[macro_export]
macro_rules! siemsg_c {
    ($($arg:tt)*) => {{
        let _: bool = $crate::semsg_c!($($arg)*);
    }};
}

/// `semsg_multiline()`: report a `printf`-formatted error of `ext_messages`
/// kind `kind`, keeping embedded newlines. Evaluates to `bool`.
#[macro_export]
macro_rules! semsg_multiline_c {
    ($kind:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        let kind = $kind;
        let fmt = $crate::message_fmt::c_format($fmt);
        if $crate::message::emsg_not_now() {
            $(let _ = $arg;)*
            true
        } else {
            let mut errbuf = $crate::message::semsg_multiline_errbuf();
            $crate::strings::vim_snprintf(
                errbuf.as_mut_ptr(),
                $crate::message::SEMSG_MULTILINE_ERRBUF_LEN,
                fmt,
                $($arg,)*
            );
            $crate::message::semsg_multiline_report(&errbuf, kind)
        }
    }};
}

/// `msg_schedule_semsg()`: report a `printf`-formatted error from the main
/// loop, for callers that cannot show one where they are.
#[macro_export]
macro_rules! msg_schedule_semsg_c {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {{
        let mut msgbuf = $crate::message::msg_iobuff();
        $crate::strings::vim_snprintf(
            msgbuf.as_mut_ptr(),
            $crate::message::MSG_IOBUFF_LEN,
            $crate::message_fmt::c_format($fmt),
            $($arg,)*
        );
        $crate::message::msg_schedule_semsg_finish(&msgbuf);
    }};
}

/// `msg_schedule_semsg_multiline()`: [`msg_schedule_semsg_c!`] keeping
/// embedded newlines.
#[macro_export]
macro_rules! msg_schedule_semsg_multiline_c {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {{
        let mut msgbuf = $crate::message::msg_iobuff();
        $crate::strings::vim_snprintf(
            msgbuf.as_mut_ptr(),
            $crate::message::MSG_IOBUFF_LEN,
            $crate::message_fmt::c_format($fmt),
            $($arg,)*
        );
        $crate::message::msg_schedule_semsg_multiline_finish(&msgbuf);
    }};
}

/// `swmsg()`: show a `printf`-formatted warning, `hl` selecting
/// `'warningmsg'` highlighting.
#[macro_export]
macro_rules! swmsg_c {
    ($hl:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        let hl = $hl;
        let mut msgbuf = $crate::message::msg_iobuff();
        $crate::strings::vim_snprintf(
            msgbuf.as_mut_ptr(),
            $crate::message::MSG_IOBUFF_LEN,
            $crate::message_fmt::c_format($fmt),
            $($arg,)*
        );
        $crate::message::swmsg_finish(&msgbuf, hl);
    }};
}

/// `smsg()`: show a `printf`-formatted message with the given highlight id.
/// Evaluates to `c_int`, as the wrapper did.
#[macro_export]
macro_rules! smsg_c {
    ($hl_id:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        let hl_id = $hl_id;
        let mut msgbuf = $crate::message::msg_iobuff();
        $crate::strings::vim_snprintf(
            msgbuf.as_mut_ptr(),
            $crate::message::MSG_IOBUFF_LEN,
            $crate::message_fmt::c_format($fmt),
            $($arg,)*
        );
        $crate::message::smsg_finish(&msgbuf, hl_id)
    }};
}

/// `smsg_keep()`: [`smsg_c!`], keeping the message displayed.
#[macro_export]
macro_rules! smsg_keep_c {
    ($hl_id:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        let hl_id = $hl_id;
        let mut msgbuf = $crate::message::msg_iobuff();
        $crate::strings::vim_snprintf(
            msgbuf.as_mut_ptr(),
            $crate::message::MSG_IOBUFF_LEN,
            $crate::message_fmt::c_format($fmt),
            $($arg,)*
        );
        $crate::message::smsg_keep_finish(&msgbuf, hl_id)
    }};
}
