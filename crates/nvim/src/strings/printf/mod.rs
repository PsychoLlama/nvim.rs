//! The `vim_snprintf` family: the argument a conversion reads, and the safe
//! entry points.
//!
//! One formatter (`emit`) serves two argument sources: an array of typed
//! [`FmtArg`] values, which the `vim_snprintf!` macros build on the caller's
//! stack, and the `TypVal` array Vimscript's `printf()` passes -- the reason
//! `tv_nr`/`tv_str`/`tv_ptr`/`tv_float` exist: they read one argument out of
//! that array with the type checking a value array has no need for.
//!
//! [`format_into`] is the safe spelling over a slice. The pointer spellings
//! the rest of the tree still calls -- a raw destination, a raw format, raw
//! `%s` pointers -- are in `strings/raw.rs`, the one file of `strings` that
//! reads through a pointer.

#![forbid(unsafe_code)]

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
use super::raw::Values;

/// One argument of a `vim_snprintf`, at the type the caller holds it.
///
/// The `vim_snprintf!` macros build an array of these from their arguments,
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
    /// A raw pointer: `%p` prints its address. `%s` reads it as a C string
    /// only through the pointer fronts in `strings/raw.rs`, whose callers vouch for it;
    /// anywhere else it prints as `[NULL]`.
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
/// [`FmtArg::Ptr`] handed to `%s` prints as `[NULL]` here: only the pointer
/// fronts may read through one.
pub fn format_into(out: &mut [u8], fmt: &[u8], args: &[FmtArg<'_>]) -> usize {
    let fmt = fmt.split(|&b| b == 0).next().unwrap_or_default();
    format(out, fmt, Source::Values(Values::new(args)))
}

/// A formatted length as the C `int` it travelled in: the low 32 bits, so a
/// result past 2 GiB reads as negative, as upstream's did.
pub(in crate::strings) fn length_as_int(len: usize) -> c_int {
    crate::narrow::number_as_int(i64::try_from(len).unwrap_or(i64::MAX))
}

/// Raised when the format asks for an argument `printf()` was not given.
const E_INSUFFICIENT_ARGS: &CStr = c"E766: Insufficient arguments for printf()";
const E_EXPECTED_FLOAT: &CStr = c"E807: Expected Float argument for printf()";

/// The `*idxp`-th Vimscript argument, or `None` with `E766` raised.
///
/// Indexing is one-based -- the C writes `tvs[*idxp - 1]` at every fetcher.
/// The index moves on only when an argument was actually there.
fn next_arg<'a>(tvs: &'a [TypVal], idxp: &mut c_int) -> Option<&'a TypVal> {
    let Some(tv) = usize::try_from(*idxp - 1).ok().and_then(|i| tvs.get(i)) else {
        emsg(gettext(E_INSUFFICIENT_ARGS));
        return None;
    };
    *idxp += 1;
    Some(tv)
}

/// `value` as `%g` formats it, written into `buffer`; answers the length,
/// the terminator excluded. The buffer is `NUMBUFLEN` bytes, which `%g`
/// never fills.
pub(crate) fn format_float_g(value: Float, buffer: &mut [u8; NUMBUFLEN as usize]) -> usize {
    let len = format(
        buffer,
        b"%g",
        Source::Values(Values::new(&[FmtArg::Float(value)])),
    );
    len.min(buffer.len() - 1)
}

/// The next argument as a number; 0 if it is not one.
pub(crate) fn tv_nr(tvs: &[TypVal], idxp: &mut c_int) -> VarNumber {
    let Some(tv) = next_arg(tvs, idxp) else {
        return 0;
    };
    tv_get_number_chk(tv).unwrap_or(0)
}

/// The next argument as a string; `None` when there is none.
///
/// A String is read in place and a Number is rendered into `numbuf`, which
/// the caller lends; anything else is rendered as `:echo` would render it,
/// up to its first NUL, into an answer of its own.
pub(crate) fn tv_str<'b>(
    tvs: &'b [TypVal],
    idxp: &mut c_int,
    numbuf: &'b mut NumBuf,
) -> Option<Cow<'b, [u8]>> {
    let tv = next_arg(tvs, idxp)?;
    if matches!(tv.v_type(), VAR_STRING | VAR_NUMBER) {
        numbuf
            .string_chk(tv)
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
/// `E807` and zero.
pub(crate) fn tv_float(tvs: &[TypVal], idxp: &mut c_int) -> Float {
    let Some(tv) = next_arg(tvs, idxp) else {
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
/// The caller's `did_emsg` survives: it is cleared before the format is read
/// so that only *this* call's errors are seen, and OR-ed back afterwards.
pub(crate) fn format_typvals(fmt: &TypVal, args: &[TypVal]) -> Option<Vec<u8>> {
    use crate::message::state::did_emsg;
    let saved_did_emsg = did_emsg.get();
    did_emsg.set(0);
    let mut buf = NumBuf::new();
    let fmt = buf.string(fmt).to_bytes();
    let len = length_as_int(format(&mut [], fmt, Source::Typvals(args)));
    let len = usize::try_from(len).unwrap_or(0);
    let out = (did_emsg.get() == 0).then(|| {
        let mut out = vec![0u8; len + 1];
        format(&mut out, fmt, Source::Typvals(args));
        out.truncate(len);
        out
    });
    did_emsg.set(did_emsg.get() | saved_did_emsg);
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
