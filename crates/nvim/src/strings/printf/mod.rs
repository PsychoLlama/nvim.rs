//! The `vim_snprintf` family: entry points and argument fetchers.
//!
//! Every spelling funnels into `vim_vsnprintf_typval`, which can be handed
//! either a C `va_list` or an array of `TypVal` -- the latter being how
//! Vimscript's `printf()` passes its arguments, and the reason
//! `tv_nr`/`tv_str`/`tv_ptr`/`tv_float` exist: they read one argument out of
//! that array with the type checking C's varargs cannot do.  `arena_printf`
//! is the one spelling that formats into a growable buffer rather than a
//! fixed one, and it goes through the *libc* `vsnprintf` rather than through
//! this file's formatter.
//!
//! The variadic entry points stay variadic: turning a variadic call site into
//! a macro is phase 16's tree-wide sweep, and it has to stay mechanical.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use core::ffi::{CStr, VaList, c_char, c_int, c_void};
use core::ptr;

use crate::eval::encode::encode_tv2echo;
use crate::eval::typval::{NumBuf, tv_get_number_chk};
use crate::memory::{xfree, xmalloc};
use crate::message::emsg;
use crate::os::cshim::{gettext, vsnprintf};
use crate::types::{Float, String_0, TypVal, VAR_FLOAT, VAR_NUMBER, VAR_STRING, VarNumber, size_t};

// The carve of the transpiled module; see each child's docs.
mod emit;
mod float;
mod spec;

pub use self::emit::*;

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

/// The next argument as a number; 0 if it is not one.
///
pub(crate) fn tv_nr(tvs: &[TypVal], idxp: &mut c_int) -> VarNumber {
    let Some(tv) = next_arg(tvs, idxp) else {
        return 0;
    };
    tv_get_number_chk(tv).unwrap_or(0)
}

/// The next argument as a string.
///
/// A String is read in place and a Number is rendered into `numbuf`, which
/// the caller lends and which must outlive the answer; anything else is
/// rendered as `:echo` would render it, and `*tofree` then owns that.
pub(crate) fn tv_str(
    tvs: &[TypVal],
    idxp: &mut c_int,
    tofree: &mut *mut c_char,
    numbuf: &mut NumBuf,
) -> *const c_char {
    let Some(tv) = next_arg(tvs, idxp) else {
        return ptr::null();
    };
    if matches!(tv.v_type(), VAR_STRING | VAR_NUMBER) {
        *tofree = ptr::null_mut();
        numbuf.string_ptr_chk(tv)
    } else {
        // SAFETY: a live value; the rendering is a fresh allocation.
        *tofree = unsafe { encode_tv2echo(tv, ptr::null_mut()) };
        *tofree
    }
}

/// The next argument as a pointer, for `%p`.
///
/// Every pointer-shaped value -- String, Func, List, Dict, Blob, Partial --
/// occupies the same union slot, and `%p` is the address of whichever one the
/// argument is; see [`TypVal::payload_address`], which is why this one read
/// is not keyed on the tag.
///
pub(crate) fn tv_ptr(tvs: &[TypVal], idxp: &mut c_int) -> *const c_void {
    match next_arg(tvs, idxp) {
        Some(tv) => tv.payload_address(),
        None => ptr::null(),
    }
}

/// The next argument as a float; a Number is widened, anything else is
/// `E807` and zero.
///
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

/// Append a formatted value to the string already in `str`.
///
/// # Safety
///
/// `str` must point at `str_m` writable bytes the caller owns holding a NUL-
/// terminated string, unaliased for the call; the result is appended to it.
/// `fmt` must point at a NUL-terminated format, and the variadic arguments
/// must be exactly the ones its conversions name, at the types they name --
/// the list is read blind.
pub unsafe extern "C" fn vim_snprintf_add(
    str: *mut c_char,
    str_m: size_t,
    fmt: *const c_char,
    args: ...
) -> c_int {
    let len = unsafe { cstr::bytes_at(str) }.len();
    let space = str_m.saturating_sub(len);
    unsafe { vim_vsnprintf(str.add(len), space, fmt, args.clone()) }
}

/// Write a formatted value to `str`.
///
/// Returns the number of bytes, excluding the NUL, that *would* have been
/// written had `str_m` been large enough — which is why it is not safe to
/// use as a buffer offset. See `vim_snprintf_safelen`.
///
/// # Safety
///
/// `fmt` must point at a NUL-terminated format, and the variadic arguments
/// must be exactly the ones its conversions name, at the types they name --
/// the list is read blind. `str` must point at `str_m` writable bytes the
/// caller owns, unaliased for the call; it is left NUL-terminated whenever
/// `str_m` is not zero.
pub unsafe extern "C" fn vim_snprintf(
    str: *mut c_char,
    str_m: size_t,
    fmt: *const c_char,
    args: ...
) -> c_int {
    unsafe { vim_vsnprintf(str, str_m, fmt, args.clone()) }
}

/// Like `vim_snprintf` but with a return value that can safely increment a
/// buffer length: never greater than `str_m - 1`.
///
/// # Safety
///
/// `fmt` must point at a NUL-terminated format, and the variadic arguments
/// must be exactly the ones its conversions name, at the types they name --
/// the list is read blind. `str` must point at `str_m` writable bytes the
/// caller owns, unaliased for the call; it is left NUL-terminated whenever
/// `str_m` is not zero.
pub unsafe extern "C" fn vim_snprintf_safelen(
    str: *mut c_char,
    str_m: size_t,
    fmt: *const c_char,
    args: ...
) -> size_t {
    if str_m == 0 {
        return 0;
    }
    let str_l = unsafe { vim_vsnprintf_typval(str, str_m, fmt, args.clone(), None) };
    if str_l < 0 {
        unsafe { *str = 0 };
        return 0;
    }
    (str_l as size_t).min(str_m - 1)
}

/// # Safety
///
/// `str` must point at a NUL-terminated string, unaliased for the call. `fmt`
/// must point at a NUL-terminated string.
pub unsafe fn vim_vsnprintf(
    str: *mut c_char,
    str_m: size_t,
    fmt: *const c_char,
    ap: VaList,
) -> c_int {
    unsafe { vim_vsnprintf_typval(str, str_m, fmt, ap, None) }
}

/// How infinity prints, for every combination of sign, forced sign, the
/// space-for-positive flag, and the conversion's case.
///
/// The index is `positive * (1 + force_sign + force_sign * space)`, which
/// is 0 for a negative value and 1, 2 or 3 for a positive one depending on
/// which sign flags are set; an uppercase conversion adds 4.
pub(crate) fn infinity_str(
    positive: bool,
    fmt_spec: c_char,
    force_sign: bool,
    space_for_positive: bool,
) -> &'static CStr {
    const TABLE: [&CStr; 8] = [
        c"-inf", c"inf", c"+inf", c" inf", c"-INF", c"INF", c"+INF", c" INF",
    ];
    let force_sign = c_int::from(force_sign);
    let mut idx =
        c_int::from(positive) * (1 + force_sign + force_sign * c_int::from(space_for_positive));
    if (fmt_spec as u8).is_ascii_uppercase() {
        idx += 4;
    }
    TABLE[idx as usize]
}

/// The scratch buffer `vim_vsnprintf_typval` renders one conversion into.
const TMP_LEN: c_int = 350;

/// `vsnprintf` into a fresh api [`String_0`], which owns the bytes.
///
/// Upstream calls this `arena_printf` and hands it the arena the rendering
/// is to be cut from. It took an arena until the api value types started
/// owning their storage; the rendering is measured and then written into a
/// block of its own, and there is nothing left for an arena to do.
///
/// # Safety
///
/// `fmt` must point at a NUL-terminated format, and the variadic arguments
/// must be exactly the ones its conversions name, at the types they name --
/// the list is read blind.
pub unsafe extern "C" fn printf_string(fmt: *const c_char, args: ...) -> String_0 {
    // SAFETY: the caller's format and argument list, read twice -- `clone`
    // is what makes a second pass over a `va_list` legal.
    let printed = unsafe { vsnprintf(ptr::null_mut(), 0, fmt, args.clone()) };
    if printed < 0 {
        return String_0::NULL;
    }
    let room = printed as size_t + 1;
    // SAFETY: `xmalloc` answers `room` writable bytes or aborts.
    let buf = unsafe { xmalloc(room) } as *mut c_char;
    // SAFETY: as above; `room` is what the measuring pass asked for.
    let printed = unsafe { vsnprintf(buf, room, fmt, args.clone()) };
    if printed < 0 {
        // SAFETY: the block this function just made.
        unsafe { xfree(buf.cast()) };
        return String_0::NULL;
    }
    // SAFETY: `vsnprintf` filled and terminated the block, which the answer
    // takes over.
    unsafe { String_0::from_owned_parts(buf, printed as size_t) }
}
