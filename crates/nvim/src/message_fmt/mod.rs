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
use crate::os::cshim::{gettext, gettext_template};
use core::ffi::{CStr, c_char, c_int, c_long, c_uint, c_ulong};
use core::fmt;
use core::fmt::Write as _;
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
            Some(s) => write_bytes(f, s.to_bytes()),
        }
    }
}

/// Write `bytes` into a Rust message, keeping the bytes that are not UTF-8.
///
/// The messages this tree formats quote file names, patterns and document
/// text, none of which is guaranteed to be UTF-8, and vim's `printf` copied
/// whatever bytes it was given. A `fmt::Formatter` only takes `&str`, so a
/// byte that is not UTF-8 goes through as a private-use character
/// (`U+F700 + byte`) that [`to_message`] turns back into the byte on the way
/// out. Nothing else in the tree uses that block, and a message that really
/// held one would only be rendering it as itself.
///
/// A precision truncates to that many *bytes*, which is what `%.*s` means.
fn write_bytes(f: &mut fmt::Formatter<'_>, bytes: &[u8]) -> fmt::Result {
    let mut rest = match f.precision() {
        Some(p) => &bytes[..bytes.len().min(p)],
        None => bytes,
    };
    loop {
        match core::str::from_utf8(rest) {
            Ok(text) => return f.write_str(text),
            Err(bad) => {
                let (good, tail) = rest.split_at(bad.valid_up_to());
                f.write_str(core::str::from_utf8(good).unwrap_or(""))?;
                let len = bad.error_len().unwrap_or(tail.len()).max(1);
                for &byte in &tail[..len] {
                    let escaped = char::from_u32(ESCAPE_BASE + u32::from(byte));
                    f.write_char(escaped.unwrap_or(char::REPLACEMENT_CHARACTER))?;
                }
                rest = &tail[len..];
            }
        }
    }
}

/// The private-use block [`write_bytes`] escapes a raw byte into.
const ESCAPE_BASE: u32 = 0xf700;

/// A `&CStr` as a message argument, keeping bytes that are not UTF-8 --
/// [`c_str`] for a caller that already has the borrow.
pub(crate) fn msg_cstr(text: &CStr) -> CDisplay<'_> {
    CDisplay(Some(text))
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
        return BytesDisplay(None);
    }
    let bytes = p.cast::<u8>();
    // SAFETY: the caller's contract; `c_char` and `u8` share a layout.
    BytesDisplay(Some(unsafe { core::slice::from_raw_parts(bytes, len) }))
}

/// Bytes as a message argument, rendered like [`CDisplay`]. `None` is a null
/// pointer, which vim's printf writes as `[NULL]` whatever the precision.
pub(crate) struct BytesDisplay<'a>(Option<&'a [u8]>);

impl BytesDisplay<'_> {
    /// Write a null pointer as *nothing*, the way the **C library**'s `%.*s`
    /// does at precision zero, rather than as vim's `[NULL]`.
    ///
    /// The two printfs disagree on exactly this input: vim's substitutes
    /// `[NULL]` whatever the precision, while the C library never reads the
    /// pointer when the precision is zero and so writes no bytes at all. The
    /// API's refusals are formatted by upstream's `api_set_error`, which goes
    /// through the C library — and an API `String` carrying no bytes *is*
    /// `(data: NULL, size: 0)`, so `Invalid key: '%.*s'` on such a key reads
    /// `Invalid key: ''` and an RPC client sees that text.
    ///
    /// Only for arguments whose null pointer always arrives with a zero
    /// length, which is every `String` the API decodes and every
    /// `lua_tolstring` that refused. A null pointer with a *real* length is
    /// the C library's `(null)`, which this does not spell.
    pub(crate) fn null_as_empty(self) -> Self {
        match self.0 {
            None => BytesDisplay(Some(&[])),
            some => BytesDisplay(some),
        }
    }
}

impl fmt::Display for BytesDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            None => f.write_str("[NULL]"),
            Some(bytes) => write_bytes(f, bytes),
        }
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
    BytesDisplay(Some(bytes))
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

/// The bytes a formatted message names.
///
/// `format_args!` can only carry a `&str`, so [`write_bytes`] puts a byte
/// that is not valid UTF-8 through as `U+F700 + byte`; this is the other
/// half, and it is why a file name or a pattern reaches the screen — or the
/// log — byte for byte rather than as a run of `U+FFFD`.
pub(crate) fn to_bytes(text: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(text.len());
    for ch in text.chars() {
        match u32::from(ch)
            .checked_sub(ESCAPE_BASE)
            .and_then(|byte| u8::try_from(byte).ok())
        {
            Some(byte) => bytes.push(byte),
            None => {
                let mut buf = [0u8; 4];
                bytes.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
            }
        }
    }
    bytes
}

/// The formatted message as the C string the message layer takes, truncated
/// where the C wrapper's scratch buffer truncated it.
///
/// `cap` is that buffer's size, terminator included. An interior NUL from an
/// argument ends the message there, as it did in the C caller.
pub(crate) fn to_message(text: String, cap: usize) -> CString {
    let mut bytes = to_bytes(&text);
    if let Some(nul) = bytes.iter().position(|&b| b == 0) {
        bytes.truncate(nul);
    }
    bytes.truncate(cap - 1);
    CString::new(bytes).unwrap_or_default()
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

/// Render a message whose *template* is data rather than a literal, through
/// the same interpreter a translated message takes.
///
/// The compile-checked macros cannot serve a caller whose format is chosen at
/// runtime: `ngettext`'s plural form, which only the catalogue knows, or a
/// shared `e_*` constant behind a helper two dozen call sites reach. Those
/// render here. The *template* is unchecked, as it must be -- but the
/// arguments are [`TrArg`]s, so the class of bug the variadic macros carried
/// is gone either way: nothing here can read a machine word at the wrong
/// width, and a template asking for a conversion its argument is not gets the
/// argument's own rendering.
///
/// `template` is translated first, so a call site passes the msgid.
pub(crate) fn tr_template(template: &'static CStr, args: &[TrArg<'_>]) -> String {
    render_template_c(gettext(template), args)
}

/// [`tr_template`] for a template that is *already* translated -- what
/// `ngettext` answers, whose plural form the caller must not look up twice.
pub(crate) fn render_template_c(template: &CStr, args: &[TrArg<'_>]) -> String {
    template::render_template(&template.to_string_lossy(), args)
}

/// Report an already-formatted message as an error: [`semsg!`] for a caller
/// that built the text somewhere else, typically because two call sites share
/// the reporting but not the message.
pub(crate) fn emsg_text(text: String) -> bool {
    report_emsg(|| text)
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

/// The argument types vim's `printf` can read through a C variadic call.
///
/// A variadic passes what it is handed, byte for byte, and the compiler
/// checks nothing against the format string. `&CStr` is a *fat* pointer, so
/// a `%s` reading one takes the length word for the rest of the string -- a
/// segfault, and a silent one at the call site. This trait is the bound that
/// keeps one out: it is implemented for the thin scalars and pointers vim's
/// `printf` conversions actually consume, and deliberately not for `&CStr`,
/// where the fix is `.as_ptr()`.
///
/// Nothing in the tree calls a variadic with a *message* any more. What is
/// left is `encode`'s two `concat_num` helpers, which hand a number to
/// `vim_snprintf` with a format of their own; the bound is what says the
/// number is one a variadic can carry.
pub(crate) trait CArg {}

macro_rules! c_arg_scalars {
    ($($t:ty),* $(,)?) => { $(impl CArg for $t {})* };
}

c_arg_scalars!(c_int, c_uint, c_long, c_ulong, usize, isize, f64);

impl<T> CArg for *const T {}
impl<T> CArg for *mut T {}

/// [`tr_template`] with its arguments spelled inline.
///
/// The escape hatch from the checked family, for a format that is data. Every
/// use is a place where the message text lives somewhere the compiler cannot
/// see it, and each one carries a note saying where.
#[macro_export]
macro_rules! tr_c {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::message_fmt::tr_template(
            $fmt,
            &[$($crate::message_fmt::TrArg::of(&$arg)),*],
        )
    };
}

/// [`tr_c!`] for a template `ngettext` already translated.
#[macro_export]
macro_rules! tr_plural {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::message_fmt::render_template_c(
            $fmt,
            &[$($crate::message_fmt::TrArg::of(&$arg)),*],
        )
    };
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
    fn bytes_that_are_not_utf8_survive_the_round_trip() {
        // A file name vim would quote verbatim. `format_args!` can only carry
        // a `&str`, so the byte goes through as a private-use character and
        // comes back out of `to_message` as itself.
        let name = super::msg_bytes(b"caf\xe9.txt");
        let text = crate::tr!("E484: Can't open file {name}");
        assert_eq!(
            to_message(text, 1025).to_bytes(),
            b"E484: Can't open file caf\xe9.txt"
        );
        // A byte that is not UTF-8 in the middle of text that is.
        let mixed = super::msg_bytes("\u{2026}\u{c2}".as_bytes());
        assert_eq!(
            to_message(crate::tr!("{mixed}"), 1025).to_bytes(),
            "\u{2026}\u{c2}".as_bytes()
        );
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
