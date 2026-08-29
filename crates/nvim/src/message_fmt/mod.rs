//! Compile-checked message formatting: the Rust-side face of the message.rs
//! entry points.
//!
//! The transpiled tree reported messages through `printf`-style calls whose
//! format strings the compiler could not check against their arguments. What
//! lives here is the replacement, and it is one mechanism:
//!
//! - a call site writes [`semsg!`]/[`smsg!`]/[`tr!`] with a *Rust* format
//!   string, which `format_args!` checks against the arguments at compile
//!   time, and
//! - the literal doubles as the msgid, so the message is still translatable:
//!   [`render`] asks the catalogue for the literal and, when a catalogue
//!   answers, renders the *translated* template through [`render_template`]
//!   instead.
//!
//! # The gettext trade-off, resolved
//!
//! `format_args!` needs a literal; `gettext` needs a runtime lookup. The two
//! meet by doing both: the literal is compiled *and* looked up. The
//! untranslated answer -- the only one in a tree that ships no catalogues --
//! costs one cached lookup and takes the compiled path, so translation is
//! free where it is unused. A translated answer takes the interpreter, which
//! reads the same conversions vim's own `printf` does (`%s`, `%d`, `%ld`,
//! `%c`, `%.*s`, `%5ld`, `%1$s`, `%%`) as well as Rust's `{}` and `{0}`, so a
//! catalogue written against either spelling of a message renders correctly.
//! The interpreter is unit-tested directly, against hand-built templates,
//! because no catalogue in the tree exercises it.
//!
//! The arguments reach the interpreter as [`TrArg`] -- a `&dyn Display` and
//! nothing else. That is what closes the hole the variadic macros left open:
//! there is no argument *width* for a template to get wrong, so a translator
//! cannot crash the editor by writing `%d` where the message had `%s`.
//!
//! # What is left of the C dialect
//!
//! [`vim_snprintf`](crate::strings::vim_snprintf) stays: `printf()`,
//! `:echo`'s formatting and the `'statusline'` evaluator hand it format
//! strings that are *data*, not literals, and those are its floor. The `_c`
//! message macros that used to sit on top of it are gone.
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::message::{
    MSG_IOBUFF_LEN, SEMSG_ERRBUF_LEN, SEMSG_MULTILINE_ERRBUF_LEN, emsg, emsg_multiline_text,
    emsg_not_now, msg, msg_keep_text, msg_schedule_semsg_text, swmsg_text,
};
use crate::os::cshim::gettext_template;
use core::ffi::{CStr, c_char, c_int, c_long, c_uint, c_ulong};
use core::fmt;
use std::ffi::CString;

mod template;

// ---------------------------------------------------------------------------
// Arguments

/// One message argument, as the template interpreter sees it.
///
/// Deliberately just a `Display`: the interpreter picks padding and width off
/// the *template*, and the value renders itself. A template asking for a
/// conversion the value is not -- `%d` against a string, say -- gets the
/// value's own rendering rather than a misread machine word, which is the
/// whole reason the variadic macros are gone.
#[derive(Clone, Copy)]
pub(crate) struct TrArg<'a>(&'a dyn fmt::Display);

impl<'a> TrArg<'a> {
    /// `value` as a message argument. Spelled by the macros, not by hand.
    #[doc(hidden)]
    pub(crate) fn of<T: fmt::Display>(value: &'a T) -> Self {
        Self(value)
    }
}

impl fmt::Display for TrArg<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A `%s` argument still held as a raw pointer: the C string at `p`, or
/// `[NULL]` for a null one -- which is exactly what
/// [`vim_snprintf`](crate::strings::vim_snprintf) writes for a null `%s`, so
/// a converted call site keeps its bytes.
///
/// Bytes that are not UTF-8 render as U+FFFD, as everywhere else the tree
/// puts a C string into a Rust message.
pub(crate) struct CDisplay<'a>(Option<&'a CStr>);

impl fmt::Display for CDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            None => f.write_str("[NULL]"),
            Some(s) => fmt::Display::fmt(&s.to_string_lossy(), f),
        }
    }
}

/// A pointer [`c_str`] takes: the two spellings of a C string the tree holds.
pub(crate) trait CPtr {
    /// The pointer, read-only.
    fn c_ptr(self) -> *const c_char;
}

impl CPtr for *const c_char {
    fn c_ptr(self) -> *const c_char {
        self
    }
}

impl CPtr for *mut c_char {
    fn c_ptr(self) -> *const c_char {
        self.cast_const()
    }
}

/// The C string at `p` as a message argument. See [`CDisplay`].
///
/// # Safety
/// `p` is null, or points at a NUL-terminated string that stays live and
/// unwritten for as long as the answer is used.
pub(crate) unsafe fn c_str<'a>(p: impl CPtr) -> CDisplay<'a> {
    // SAFETY: the caller's contract, minus the null case.
    CDisplay(unsafe { crate::cstr::at_opt(p.c_ptr()) })
}

/// A `%.*s` argument: at most `len` bytes of the string at `p`, cut back to a
/// character boundary rather than through one.
///
/// # Safety
/// `p` points at `len` readable bytes.
pub(crate) unsafe fn c_str_len<'a>(p: *const c_char, len: usize) -> BytesDisplay<'a> {
    if p.is_null() {
        return BytesDisplay(b"");
    }
    // SAFETY: the caller's contract; `c_char` and `u8` share a layout.
    BytesDisplay(unsafe { core::slice::from_raw_parts(p.cast::<u8>(), len) })
}

/// Bytes as a message argument, rendered like [`CDisplay`].
pub(crate) struct BytesDisplay<'a>(&'a [u8]);

impl fmt::Display for BytesDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&String::from_utf8_lossy(self.0), f)
    }
}

/// A `%p` argument: the address `p` holds, written as C's `%p` writes it.
///
/// A `Display` and not a `Pointer`, because that is the one thing a message
/// argument is allowed to be.
pub(crate) fn msg_addr<T: ?Sized>(p: *const T) -> AddrDisplay {
    AddrDisplay(p.cast::<()>() as usize)
}

/// An address as a message argument. See [`msg_addr`].
pub(crate) struct AddrDisplay(usize);

impl fmt::Display for AddrDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

/// `bytes` as a message argument. The safe sibling of [`c_str_len`].
pub(crate) fn msg_bytes(bytes: &[u8]) -> BytesDisplay<'_> {
    BytesDisplay(bytes)
}

// ---------------------------------------------------------------------------
// Rendering

/// The message `msgid` describes: the catalogue's template when it has one,
/// and `rendered` -- what `format_args!` already produced -- when it does not.
///
/// Spelled by the macros, not by hand.
#[doc(hidden)]
pub(crate) fn render(
    msgid: &'static str,
    rendered: fmt::Arguments<'_>,
    args: &[TrArg<'_>],
) -> String {
    match gettext_template(msgid) {
        Some(template) => template::render_template(template, args),
        None => rendered.to_string(),
    }
}

// ---------------------------------------------------------------------------
// The compile-time check

/// Reject a message literal that still holds a C `printf` conversion.
///
/// `format_args!` already checks the Rust placeholders against the arguments;
/// what it cannot see is a `%s` left behind by a half-finished conversion,
/// which would print itself and silently drop an argument. Spelled inside a
/// `const` block by the macros, so a stale conversion fails the build.
///
/// # Panics
/// At compile time, when `template` holds a conversion.
#[doc(hidden)]
pub(crate) const fn check_template(template: &str) {
    let b = template.as_bytes();
    let mut i = 0;
    while i + 1 < b.len() {
        if b[i] != b'%' {
            i += 1;
            continue;
        }
        if b[i + 1] == b'%' {
            i += 2;
            continue;
        }
        let mut j = i + 1;
        while j < b.len()
            && matches!(
                b[j],
                b'0'..=b'9' | b'-' | b'+' | b'.' | b'*' | b'#' | b'\'' | b' '
            )
        {
            j += 1;
        }
        while j < b.len() && matches!(b[j], b'h' | b'l' | b'L' | b'z' | b'j' | b't') {
            j += 1;
        }
        assert!(
            !(j < b.len() && matches!(b[j], b's' | b'd' | b'i' | b'u' | b'x' | b'X' | b'o')),
            "this message still holds a C printf conversion: write it as a Rust \
             placeholder and pass the argument, or spell the percent as `%%`"
        );
        i += 1;
    }
}

// ---------------------------------------------------------------------------
// Reporting

/// The formatted message as the C string the message layer takes, truncated
/// where the C wrapper's scratch buffer truncated it.
///
/// `cap` is that buffer's size, terminator included. An interior NUL from an
/// argument ends the message there, as it did in the C caller.
fn to_message(mut text: String, cap: usize) -> CString {
    if let Some(nul) = text.find('\0') {
        text.truncate(nul);
    }
    if text.len() >= cap {
        let mut cut = cap - 1;
        while cut > 0 && !text.is_char_boundary(cut) {
            cut -= 1;
        }
        text.truncate(cut);
    }
    CString::new(text).unwrap_or_default()
}

/// `semsg()`: format and report an error. Skipped, unformatted, when errors
/// are suppressed -- which is why the message arrives as a closure.
#[doc(hidden)]
pub(crate) fn report_emsg(message: impl FnOnce() -> String) -> bool {
    // SAFETY: reads message-state globals; main thread, like every message
    // call. Checked before formatting so a suppressed error costs nothing.
    if unsafe { emsg_not_now() } {
        return true;
    }
    emsg(&to_message(message(), SEMSG_ERRBUF_LEN))
}

/// `semsg_multiline()`: [`report_emsg`] keeping embedded newlines, reported
/// under the `ext_messages` kind `kind`.
#[doc(hidden)]
pub(crate) fn report_emsg_multiline(kind: &CStr, message: impl FnOnce() -> String) -> bool {
    // SAFETY: as [`report_emsg`].
    if unsafe { emsg_not_now() } {
        return true;
    }
    emsg_multiline_text(&to_message(message(), SEMSG_MULTILINE_ERRBUF_LEN), kind)
}

/// `msg_schedule_semsg()`: hand the formatted error to the main loop, for a
/// caller that cannot show one where it is.
#[doc(hidden)]
pub(crate) fn report_schedule_emsg(message: impl FnOnce() -> String) {
    msg_schedule_semsg_text(&to_message(message(), MSG_IOBUFF_LEN), false);
}

/// [`report_schedule_emsg`] keeping embedded newlines.
#[doc(hidden)]
pub(crate) fn report_schedule_emsg_multiline(message: impl FnOnce() -> String) {
    msg_schedule_semsg_text(&to_message(message(), MSG_IOBUFF_LEN), true);
}

/// `smsg()`: show a formatted message with the given highlight id.
#[doc(hidden)]
pub(crate) fn report_msg(hl_id: c_int, message: impl FnOnce() -> String) -> bool {
    msg(&to_message(message(), MSG_IOBUFF_LEN), hl_id)
}

/// `smsg_keep()`: [`report_msg`], keeping the message displayed.
#[doc(hidden)]
pub(crate) fn report_msg_keep(hl_id: c_int, message: impl FnOnce() -> String) -> bool {
    msg_keep_text(&to_message(message(), MSG_IOBUFF_LEN), hl_id)
}

/// `swmsg()`: show a formatted warning, `hl` selecting `'warningmsg'`
/// highlighting.
#[doc(hidden)]
pub(crate) fn report_warning(hl: bool, message: impl FnOnce() -> String) {
    swmsg_text(&to_message(message(), MSG_IOBUFF_LEN), hl);
}

/// Format and hand back the message, for the callers that do something else
/// with it. [`tr!`]'s tail.
#[doc(hidden)]
pub(crate) fn report_text(message: impl FnOnce() -> String) -> String {
    message()
}

// ---------------------------------------------------------------------------
// The macros

/// Bind the arguments once, check the literal, and hand `$report` a closure
/// that formats. Every message macro is one line of this.
///
/// The arms are per arity because each argument has to be *named* to reach
/// both `format_args!` and the [`TrArg`] slice while being evaluated once.
#[doc(hidden)]
#[macro_export]
macro_rules! __message {
    ($report:expr, $lit:literal $(,)?) => {{
        const { $crate::message_fmt::check_template($lit) };
        $report(|| $crate::message_fmt::render($lit, ::core::format_args!($lit), &[]))
    }};
    ($report:expr, $lit:literal, $a0:expr $(,)?) => {{
        const { $crate::message_fmt::check_template($lit) };
        let a0 = $a0;
        $report(|| {
            $crate::message_fmt::render(
                $lit,
                ::core::format_args!($lit, a0),
                &[$crate::message_fmt::TrArg::of(&a0)],
            )
        })
    }};
    ($report:expr, $lit:literal, $a0:expr, $a1:expr $(,)?) => {{
        const { $crate::message_fmt::check_template($lit) };
        let a0 = $a0;
        let a1 = $a1;
        $report(|| {
            $crate::message_fmt::render(
                $lit,
                ::core::format_args!($lit, a0, a1),
                &[
                    $crate::message_fmt::TrArg::of(&a0),
                    $crate::message_fmt::TrArg::of(&a1),
                ],
            )
        })
    }};
    ($report:expr, $lit:literal, $a0:expr, $a1:expr, $a2:expr $(,)?) => {{
        const { $crate::message_fmt::check_template($lit) };
        let a0 = $a0;
        let a1 = $a1;
        let a2 = $a2;
        $report(|| {
            $crate::message_fmt::render(
                $lit,
                ::core::format_args!($lit, a0, a1, a2),
                &[
                    $crate::message_fmt::TrArg::of(&a0),
                    $crate::message_fmt::TrArg::of(&a1),
                    $crate::message_fmt::TrArg::of(&a2),
                ],
            )
        })
    }};
    ($report:expr, $lit:literal, $a0:expr, $a1:expr, $a2:expr, $a3:expr $(,)?) => {{
        const { $crate::message_fmt::check_template($lit) };
        let a0 = $a0;
        let a1 = $a1;
        let a2 = $a2;
        let a3 = $a3;
        $report(|| {
            $crate::message_fmt::render(
                $lit,
                ::core::format_args!($lit, a0, a1, a2, a3),
                &[
                    $crate::message_fmt::TrArg::of(&a0),
                    $crate::message_fmt::TrArg::of(&a1),
                    $crate::message_fmt::TrArg::of(&a2),
                    $crate::message_fmt::TrArg::of(&a3),
                ],
            )
        })
    }};
    ($report:expr, $lit:literal, $a0:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr $(,)?) => {{
        const { $crate::message_fmt::check_template($lit) };
        let a0 = $a0;
        let a1 = $a1;
        let a2 = $a2;
        let a3 = $a3;
        let a4 = $a4;
        $report(|| {
            $crate::message_fmt::render(
                $lit,
                ::core::format_args!($lit, a0, a1, a2, a3, a4),
                &[
                    $crate::message_fmt::TrArg::of(&a0),
                    $crate::message_fmt::TrArg::of(&a1),
                    $crate::message_fmt::TrArg::of(&a2),
                    $crate::message_fmt::TrArg::of(&a3),
                    $crate::message_fmt::TrArg::of(&a4),
                ],
            )
        })
    }};
}

/// The formatted, translated message as a `String`. The other macros here are
/// this one plus a way of showing it.
#[macro_export]
macro_rules! tr {
    ($($arg:tt)*) => {
        $crate::__message!($crate::message_fmt::report_text, $($arg)*)
    };
}

/// `semsg()`: report a formatted error. Evaluates to `bool`, `true` when the
/// message was output or errors are suppressed.
#[macro_export]
macro_rules! semsg {
    ($($arg:tt)*) => {
        $crate::__message!($crate::message_fmt::report_emsg, $($arg)*)
    };
}

/// `siemsg()`: report a formatted *internal* error. Same effect as
/// [`semsg!`] -- the name is the intent.
#[macro_export]
macro_rules! siemsg {
    ($($arg:tt)*) => {{
        let _: bool = $crate::semsg!($($arg)*);
    }};
}

/// `semsg_multiline()`: report a formatted error of `ext_messages` kind
/// `kind`, keeping embedded newlines.
#[macro_export]
macro_rules! semsg_multiline {
    ($kind:expr, $($arg:tt)*) => {{
        let kind = $kind;
        $crate::__message!(
            |message| $crate::message_fmt::report_emsg_multiline(kind, message),
            $($arg)*
        )
    }};
}

/// `msg_schedule_semsg()`: report a formatted error from the main loop, for
/// callers that cannot show one where they are.
#[macro_export]
macro_rules! msg_schedule_semsg {
    ($($arg:tt)*) => {
        $crate::__message!($crate::message_fmt::report_schedule_emsg, $($arg)*)
    };
}

/// [`msg_schedule_semsg!`] keeping embedded newlines.
#[macro_export]
macro_rules! msg_schedule_semsg_multiline {
    ($($arg:tt)*) => {
        $crate::__message!($crate::message_fmt::report_schedule_emsg_multiline, $($arg)*)
    };
}

/// `swmsg()`: show a formatted warning, `hl` selecting `'warningmsg'`
/// highlighting.
#[macro_export]
macro_rules! swmsg {
    ($hl:expr, $($arg:tt)*) => {{
        let hl = $hl;
        $crate::__message!(|message| $crate::message_fmt::report_warning(hl, message), $($arg)*)
    }};
}

/// `smsg()`: show a formatted message with the given highlight id.
#[macro_export]
macro_rules! smsg {
    ($hl_id:expr, $($arg:tt)*) => {{
        let hl_id = $hl_id;
        $crate::__message!(|message| $crate::message_fmt::report_msg(hl_id, message), $($arg)*)
    }};
}

/// `smsg_keep()`: [`smsg!`], keeping the message displayed.
#[macro_export]
macro_rules! smsg_keep {
    ($hl_id:expr, $($arg:tt)*) => {{
        let hl_id = $hl_id;
        $crate::__message!(|message| $crate::message_fmt::report_msg_keep(hl_id, message), $($arg)*)
    }};
}

// ---------------------------------------------------------------------------
// The `_c` family: vim's own printf, for the call sites not yet converted.

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

/// The argument types vim's `printf` can read through a C variadic call.
///
/// A variadic passes what it is handed, byte for byte, and the compiler
/// checks nothing against the format string. That is survivable while every
/// message argument is a raw pointer; it is a trap now that `gettext` answers
/// a `&CStr`, because `&CStr` is a *fat* pointer and a `%s` reading one takes
/// the length word for the rest of the string -- a segfault, and a silent one
/// at the call site.
///
/// So the `_c` macros route every value argument through [`c_arg`], and this
/// trait is implemented for the thin scalars and pointers vim's `printf`
/// conversions actually consume. `&CStr` is deliberately not among them: the
/// fix at such a site is `.as_ptr()`.
pub(crate) trait CArg {}

macro_rules! c_arg_scalars {
    ($($t:ty),* $(,)?) => { $(impl CArg for $t {})* };
}

c_arg_scalars!(c_int, c_uint, c_long, c_ulong, usize, isize, f64);

impl<T> CArg for *const T {}
impl<T> CArg for *mut T {}

/// Identity, with [`CArg`]'s bound on it: the seam where a variadic message
/// argument is type-checked. Not meant to be called directly.
#[doc(hidden)]
#[inline(always)]
pub(crate) fn c_arg<T: CArg>(arg: T) -> T {
    arg
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
                $($crate::message_fmt::c_arg($arg),)*
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
                $($crate::message_fmt::c_arg($arg),)*
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
            $($crate::message_fmt::c_arg($arg),)*
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
            $($crate::message_fmt::c_arg($arg),)*
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
            $($crate::message_fmt::c_arg($arg),)*
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
            $($crate::message_fmt::c_arg($arg),)*
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
            $($crate::message_fmt::c_arg($arg),)*
        );
        $crate::message::smsg_keep_finish(&msgbuf, hl_id)
    }};
}

#[cfg(test)]
mod tests {
    use super::to_message;

    #[test]
    fn tr_formats_the_literal_when_nothing_translates_it() {
        // No catalogue ships with this tree, so `tr!` is `format_args!` plus
        // a lookup that misses.
        assert_eq!(crate::tr!("E32: No file name"), "E32: No file name");
        assert_eq!(
            crate::tr!("E1510: Value too large: {}", 7),
            "E1510: Value too large: 7"
        );
        let name = "x";
        assert_eq!(crate::tr!("E480: No match: {name}"), "E480: No match: x");
    }

    #[test]
    fn a_message_is_cut_where_the_c_buffer_cut_it() {
        let long = "a".repeat(2000);
        assert_eq!(to_message(long, 1025).to_bytes().len(), 1024);
        // An argument's interior NUL ends the message, as it did in C.
        assert_eq!(to_message("a\0b".to_owned(), 1025).to_bytes(), b"a");
        // The cut lands on a character boundary rather than inside one.
        let wide = "é".repeat(600);
        let cut = to_message(wide, 1025);
        assert!(cut.to_bytes().len() <= 1024 && core::str::from_utf8(cut.to_bytes()).is_ok());
    }
}
