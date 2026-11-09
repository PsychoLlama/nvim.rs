//! The `vim_snprintf` family: the argument a conversion reads, and the safe
//! entry points.
//!
//! One formatter (`emit`) serves two argument sources: an array of typed
//! [`FmtArg`] values, which the `formatted!`/`format_into!` macros build on the caller's
//! stack, and the `TypVal` array Vimscript's `printf()` passes -- the reason
//! `tv_nr`/`tv_str`/`tv_ptr`/`tv_float` exist: they read one argument out of
//! that array with the type checking a value array has no need for.
//!
//! [`format_into`] is the spelling over a slice, and [`formatted`] the one
//! that answers an owned buffer. Neither reads through a pointer: a `%s`
//! argument is bytes, and a raw pointer handed to one prints as `[NULL]`.
//!
//! The macros are the type gate the C variadic never had. Each argument
//! goes through `FmtArg::from`, and an owner has no conversion, so it is a
//! compile error at the call site rather than two words of garbage in the
//! argument area (`spell_load_lang` once handed `vim_snprintf` an
//! `XString`):
//!
//! ```compile_fail,E0277
//! use neovim::{format_into, memory::XString};
//! fn spell_file(buf: &mut [u8], lang: &[u8], enc: &XString) {
//!     format_into!(buf, b"spell/%s.%s.spl", lang, enc);
//! }
//! ```
//!
//! The array is built where the macro is written, on the stack: a call
//! allocates nothing for its arguments.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::types::Failed;
use core::ffi::{CStr, c_int, c_long, c_uint, c_ulong, c_void};
use std::borrow::Cow;

use crate::eval::encode::tv2echo_bytes;
use crate::eval::typval::{NUMBUFLEN, NumBuf, tv_get_number_chk};
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::types::{Float, TypVal, VAR_FLOAT, VAR_NUMBER, VAR_STRING, VarNumber};

// The carve of the transpiled module; see each child's docs.
mod emit;
mod float;
mod spec;

pub(in crate::strings) use self::emit::{Source, format};

/// One argument of a `vim_snprintf`, at the type the caller holds it.
///
/// The [`formatted!`](crate::formatted!) and [`format_into!`](crate::format_into!) macros build an array of these from their arguments,
/// on the stack, through the `From` conversions below -- which are the type
/// gate the C variadic never had: an owner (`XString`, `Vec<u8>`) has none
/// and is a compile error at the call site. A conversion reads the value it
/// finds whatever its own letter says, the way the C read whatever was in
/// the register: an integer narrows to the conversion's width, a float
/// handed to `%d` reads as 0.
#[derive(Clone, Copy, Debug)]
pub enum FmtArg<'a> {
    /// A signed integer, widened.
    Int(i64),
    /// An unsigned integer, widened.
    UInt(u64),
    /// A double, for `%f %e %g` and their capitals.
    Float(f64),
    /// Text for `%s`/`%S`, exactly these bytes.
    Str(&'a [u8]),
    /// A raw pointer: `%p` prints its address, and `%s` prints it as
    /// `[NULL]` -- it is never read.
    Ptr(*const c_void),
}

macro_rules! fmt_arg_from {
    ($($ty:ty => $arm:ident as $wide:ty),* $(,)?) => {$(
        impl From<$ty> for FmtArg<'_> {
            #[inline(always)]
            fn from(value: $ty) -> Self {
                FmtArg::$arm(<$wide>::from(value))
            }
        }
    )*};
}

fmt_arg_from!(
    c_int => Int as i64,
    c_long => Int as i64,
    c_uint => UInt as u64,
    c_ulong => UInt as u64,
    f64 => Float as f64,
);

impl From<crate::types::Handle> for FmtArg<'_> {
    #[inline(always)]
    fn from(value: crate::types::Handle) -> Self {
        FmtArg::Int(i64::from(value))
    }
}

impl From<crate::types::LineNr> for FmtArg<'_> {
    #[inline(always)]
    fn from(value: crate::types::LineNr) -> Self {
        FmtArg::Int(i64::from(value))
    }
}

impl From<isize> for FmtArg<'_> {
    #[inline(always)]
    fn from(value: isize) -> Self {
        FmtArg::Int(value as i64)
    }
}

impl From<usize> for FmtArg<'_> {
    #[inline(always)]
    fn from(value: usize) -> Self {
        FmtArg::UInt(value as u64)
    }
}

impl<T> From<*const T> for FmtArg<'_> {
    #[inline(always)]
    fn from(value: *const T) -> Self {
        FmtArg::Ptr(value.cast())
    }
}

impl<T> From<*mut T> for FmtArg<'_> {
    #[inline(always)]
    fn from(value: *mut T) -> Self {
        FmtArg::Ptr(value.cast_const().cast())
    }
}

impl<'a> From<&'a CStr> for FmtArg<'a> {
    #[inline(always)]
    fn from(value: &'a CStr) -> Self {
        FmtArg::Str(value.to_bytes())
    }
}

/// `None` is the null pointer a C caller would have passed, which `%s`
/// prints as `[NULL]`.
impl<'a> From<Option<&'a CStr>> for FmtArg<'a> {
    #[inline(always)]
    fn from(value: Option<&'a CStr>) -> Self {
        value.map_or(FmtArg::Ptr(core::ptr::null()), |text| {
            FmtArg::Str(text.to_bytes())
        })
    }
}

impl<'a> From<&'a [u8]> for FmtArg<'a> {
    #[inline(always)]
    fn from(value: &'a [u8]) -> Self {
        FmtArg::Str(value)
    }
}

/// `fmt` formatted with `args` into `out`, NUL-terminated whenever `out` is
/// not empty; answers the length the result *would* have had, excluding the
/// NUL, so an answer at or past `out.len()` means it was cut short.
///
/// `fmt` ends at its first NUL, as the C string it stands for did. A
/// [`FmtArg::Ptr`] handed to `%s` prints as `[NULL]`.
pub fn format_into(out: &mut [u8], fmt: &[u8], args: &[FmtArg<'_>]) -> usize {
    let fmt = fmt.split(|&b| b == 0).next().unwrap_or_default();
    format(out, fmt, Source::Values(args))
}

/// [`format_into`] with its arguments spelled inline, gathered through
/// `FmtArg::from` on the caller's stack.
#[macro_export]
macro_rules! format_into {
    ($out:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::strings::format_into($out, $fmt, &[$($crate::strings::FmtArg::from($arg)),*])
    };
}

/// `fmt` formatted with `args`, as a `room`-byte buffer would hold it: the
/// text cut to `room - 1` bytes, without its terminator. Upstream formatted
/// these into a fixed scratch array; this is that array's contents, owned.
///
/// The text is measured first, so the answer is allocated at its own size
/// however large `room` is.
pub(crate) fn formatted(room: usize, fmt: &[u8], args: &[FmtArg<'_>]) -> Vec<u8> {
    let keep = format_into(&mut [], fmt, args).min(room.saturating_sub(1));
    let mut out = vec![0; keep + 1];
    format_into(&mut out, fmt, args);
    out.truncate(keep);
    out
}

/// [`formatted`] with its arguments spelled inline, gathered through
/// `FmtArg::from` as [`format_into!`](crate::format_into!) gathers them.
#[macro_export]
macro_rules! formatted {
    ($room:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::strings::formatted($room, $fmt, &[$($crate::strings::FmtArg::from($arg)),*])
    };
}

/// A formatted length as the C `int` it travelled in: the low 32 bits, so a
/// result past 2 GiB reads as negative, as upstream's did.
pub(in crate::strings) fn length_as_int(len: usize) -> c_int {
    crate::narrow::number_as_int(i64::try_from(len).unwrap_or(i64::MAX))
}

/// Raised when the format asks for an argument `printf()` was not given.
const E_INSUFFICIENT_ARGS: &CStr = c"E766: Insufficient arguments for printf()";
const E_EXPECTED_FLOAT: &CStr = c"E807: Expected Float argument for printf()";

/// The `*idxp`-th Vimscript argument, or `E766`, said.
///
/// Indexing is one-based -- the C writes `tvs[*idxp - 1]` at every fetcher.
/// The index moves on only when an argument was actually there.
fn next_arg<'a>(tvs: &'a [TypVal], idxp: &mut c_int) -> Result<&'a TypVal, Failed> {
    let Some(tv) = usize::try_from(*idxp - 1).ok().and_then(|i| tvs.get(i)) else {
        emsg(gettext(E_INSUFFICIENT_ARGS));
        return Err(Failed);
    };
    *idxp += 1;
    Ok(tv)
}

/// `value` as `%g` formats it, written into `buffer`; answers the length,
/// the terminator excluded. The buffer is `NUMBUFLEN` bytes, which `%g`
/// never fills.
pub(crate) fn format_float_g(value: Float, buffer: &mut [u8; NUMBUFLEN]) -> usize {
    let len = format(buffer, b"%g", Source::Values(&[FmtArg::Float(value)]));
    len.min(buffer.len() - 1)
}

/// The next argument as a number; 0 if it is not one, or is missing (E766
/// said), which is what the format then shows.
pub(crate) fn tv_nr(tvs: &[TypVal], idxp: &mut c_int) -> VarNumber {
    let Ok(tv) = next_arg(tvs, idxp) else {
        return 0;
    };
    tv_get_number_chk(tv).unwrap_or(0)
}

/// The next argument as a string; `None` when there is none (E766 said) or
/// it has no string form (said by the conversion).
///
/// A String is read in place and a Number is rendered into `numbuf`, which
/// the caller lends; anything else is rendered as `:echo` would render it,
/// up to its first NUL, into an answer of its own.
pub(crate) fn tv_str<'b>(
    tvs: &'b [TypVal],
    idxp: &mut c_int,
    numbuf: &'b mut NumBuf,
) -> Option<Cow<'b, [u8]>> {
    let tv = next_arg(tvs, idxp).ok()?;
    if matches!(tv.v_type(), VAR_STRING | VAR_NUMBER) {
        numbuf
            .string_chk(tv)
            .ok()
            .map(|text| Cow::Borrowed(text.to_bytes()))
    } else {
        let mut echoed = tv2echo_bytes(tv);
        if let Some(nul) = echoed.iter().position(|&b| b == 0) {
            echoed.truncate(nul);
        }
        Some(Cow::Owned(echoed))
    }
}

/// The next argument's address, for `%p`.
///
/// Every pointer-shaped value -- String, Func, List, Dict, Blob, Partial --
/// occupies the same union slot, and `%p` is the address of whichever one the
/// argument is; see [`TypVal::payload_address`], which is why this one read
/// is not keyed on the tag.
pub(crate) fn tv_ptr(tvs: &[TypVal], idxp: &mut c_int) -> usize {
    next_arg(tvs, idxp).map_or(0, |tv| tv.payload_address().addr())
}

/// The next argument as a float; a Number is widened, anything else is
/// `E807` and zero, and a missing one E766 and zero.
pub(crate) fn tv_float(tvs: &[TypVal], idxp: &mut c_int) -> Float {
    let Ok(tv) = next_arg(tvs, idxp) else {
        return 0.0;
    };
    match tv.v_type() {
        VAR_FLOAT => tv.float_or_zero(),
        VAR_NUMBER => tv.number_or_zero() as Float,
        _ => {
            emsg(gettext(E_EXPECTED_FLOAT));
            0.0
        }
    }
}

/// Vimscript's `printf()`: the String value `fmt` formatted with the typval
/// arguments `args`, measured and then written into an exact allocation.
/// `None` when reading the format or the measuring pass reported an error,
/// which skips the formatting pass.
///
/// The caller's `DID_EMSG` survives: it is cleared before the format is read
/// so that only *this* call's errors are seen, and OR-ed back afterwards.
pub(crate) fn format_typvals(fmt: &TypVal, args: &[TypVal]) -> Option<Vec<u8>> {
    use crate::message::state::DID_EMSG;
    let saved_did_emsg = DID_EMSG.get();
    DID_EMSG.set(0);
    let mut buf = NumBuf::new();
    let fmt = buf.string(fmt).to_bytes();
    let len = length_as_int(format(&mut [], fmt, Source::Typvals(args)));
    let len = usize::try_from(len).unwrap_or(0);
    let out = (DID_EMSG.get() == 0).then(|| {
        let mut out = vec![0u8; len + 1];
        format(&mut out, fmt, Source::Typvals(args));
        out.truncate(len);
        out
    });
    DID_EMSG.set(DID_EMSG.get() | saved_did_emsg);
    out
}

/// How infinity prints, for every combination of sign, forced sign, the
/// space-for-positive flag, and the conversion's case.
///
/// The index is `positive * (1 + force_sign + force_sign * space)`, which
/// is 0 for a negative value and 1, 2 or 3 for a positive one depending on
/// which sign flags are set; an uppercase conversion adds 4.
pub(crate) fn infinity_str(
    positive: bool,
    fmt_spec: u8,
    force_sign: bool,
    space_for_positive: bool,
) -> &'static [u8] {
    const TABLE: [&[u8]; 8] = [
        b"-inf", b"inf", b"+inf", b" inf", b"-INF", b"INF", b"+INF", b" INF",
    ];
    let force_sign = usize::from(force_sign);
    let mut idx =
        usize::from(positive) * (1 + force_sign + force_sign * usize::from(space_for_positive));
    if fmt_spec.is_ascii_uppercase() {
        idx += 4;
    }
    TABLE[idx]
}

#[cfg(test)]
mod tests {
    use super::{FmtArg, format_into};
    use core::ffi::{c_int, c_long};

    /// `format_into!` into a buffer of exactly `size` bytes, so a write past
    /// it panics; answers the length and the bytes up to the NUL.
    fn into_exact(size: usize, f: impl Fn(&mut [u8]) -> usize) -> (usize, Vec<u8>) {
        let mut buf = vec![0xa5u8; size];
        let len = f(&mut buf);
        let text = buf.iter().take_while(|&&b| b != 0).copied().collect();
        (len, text)
    }

    #[test]
    fn formats_and_truncates_at_every_size() {
        let name = c"na\u{ef}ve";
        let want = "[na\u{ef}ve|   42|ff  |2.72|x|%|0x10|-1.0e21]";
        for size in 0..=want.len() + 1 {
            let (len, text) = into_exact(size, |b| {
                crate::format_into!(
                    b,
                    b"[%s|%5d|%-4x|%.2f|%c|%%|%p|%g]",
                    name,
                    42 as c_int,
                    255 as c_int,
                    2.71875,
                    c_int::from(b'x'),
                    core::ptr::without_provenance::<u8>(16),
                    -1e21,
                )
            });
            assert_eq!(len, want.len());
            let keep = want.len().min(size.saturating_sub(1));
            assert_eq!(text, &want.as_bytes()[..keep], "size {size}");
        }
    }

    #[test]
    fn a_precision_bounds_how_much_text_is_taken() {
        // Three bytes and no terminator: the precision decides how many go
        // out, and a precision past the end stops at it.
        let bytes: &[u8] = b"abc";
        for (fmt, want) in [
            (&b"<%.3s>"[..], "<abc>"),
            (b"<%.2s>", "<ab>"),
            (b"<%.0s>", "<>"),
        ] {
            let (_, text) = into_exact(16, |b| crate::format_into!(b, fmt, bytes));
            assert_eq!(text, want.as_bytes());
        }
        let (_, text) = into_exact(16, |b| {
            crate::format_into!(b, b"<%.40s|%.*S>", c"hi", 9 as c_int, c"hi")
        });
        assert_eq!(text, b"<hi|hi>");
    }

    #[test]
    fn a_raw_pointer_is_never_read() {
        let mut out = [0u8; 32];
        let n = format_into(
            &mut out,
            b"%s %s %d %ld",
            &[
                FmtArg::from(c"x".as_ptr()),
                FmtArg::Str(b"bytes"),
                FmtArg::from(-3 as c_int),
                FmtArg::from(7 as c_long),
            ],
        );
        assert_eq!(&out[..n], b"[NULL] bytes -3 7");
        assert_eq!(out[n], 0);
    }
}
