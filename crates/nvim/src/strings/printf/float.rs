//! The floating-point conversions.
//!
//! `%f`, `%F`, `%e`, `%E`, `%g` and `%G`, and the two trims `%g` needs
//! afterwards.  Split out of [`super::emit`] for the file-size cap; the
//! seam is that everything here goes through libc's `snprintf`, because
//! the exact digits a platform prints are the platform's business.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use core::ffi::{c_char, c_int};

use super::emit::{Args, Body, Conversion, TMP};
use super::{TMP_LEN, infinity_str};
use crate::ascii::ascii_isdigit;
use crate::cstr;
use crate::memory::xstrlcpy;
use crate::os::cshim::snprintf;
use crate::strings::vim_strchr;
use crate::types::size_t;

/// `%f`, `%F`, `%e`, `%E`, `%g` and `%G`.
///
/// Everything but infinity and NaN is handed to libc's `snprintf` with a
/// format built here, because the exact digits are the platform's business.
/// `%g` is not passed through: it is resolved to `%f` or `%e` first, and
/// the trailing zeros it would have dropped are removed afterwards.
///
/// # Safety
///
/// The argument `args` reaches next must have been passed as a `double`,
/// which is what `c`'s conversion names.
pub(super) unsafe fn render_float(
    c: &mut Conversion,
    args: &mut Args,
    tmp: &mut [c_char; TMP],
) -> Body {
    let f = unsafe { args.next_float() };
    // Not `f.abs()`: the C tests `f < 0`, so -0.0 stays -0.0.
    let abs_f = if f < 0.0 { -f } else { f };
    let mut remove_trailing_zeroes = false;

    if matches!(c.fmt_spec, b'g' | b'G') {
        // The range in which `%g` chooses fixed notation.
        c.fmt_spec = if (0.001..10000000.0).contains(&abs_f) || abs_f == 0.0 {
            if c.fmt_spec.is_ascii_uppercase() {
                b'F'
            } else {
                b'f'
            }
        } else if c.fmt_spec == b'g' {
            b'e'
        } else {
            b'E'
        };
        remove_trailing_zeroes = true;
    }

    // A fixed-notation value this large would not fit the scratch
    // buffer, so it prints as infinity too.
    if f.is_infinite() || (matches!(c.fmt_spec, b'f' | b'F') && abs_f > 1.0e307) {
        let sign = c.fmt_spec as c_char;
        let text = infinity_str(f > 0.0, sign, c.force_sign, c.space_for_positive);
        let out = tmp.as_mut_ptr();
        unsafe { xstrlcpy(out, text.as_ptr(), TMP) };
        c.zero_padding = false;
        return Body::Tmp(unsafe { cstr::bytes_at(tmp.as_ptr()) }.len());
    }
    if f.is_nan() {
        let nan = if c.fmt_spec.is_ascii_uppercase() {
            c"NAN"
        } else {
            c"nan"
        };
        let into = tmp.as_mut_ptr().cast::<u8>();
        unsafe { into.copy_from(nan.as_ptr().cast(), 4) };
        c.zero_padding = false;
        return Body::Tmp(3);
    }

    // Build the format libc gets: '%', an optional sign flag, an
    // optional precision, and the conversion.
    let mut format = [0 as c_char; 40];
    format[0] = b'%' as c_char;
    let mut l: size_t = 1;
    if c.force_sign {
        format[l] = if c.space_for_positive { b' ' } else { b'+' } as c_char;
        l += 1;
    }
    if c.precision_specified {
        // Bound the precision so the result still fits `tmp`: a fixed
        // conversion also spends digits on the integer part.
        let mut max_prec = (TMP_LEN - 10) as size_t;
        if matches!(c.fmt_spec, b'f' | b'F') && abs_f > 1.0 {
            max_prec -= abs_f.log10() as size_t;
        }
        c.precision = c.precision.min(max_prec);
        let out = unsafe { format.as_mut_ptr().add(l) };
        let room = format.len() - l;
        let prec = c.precision as c_int;
        l += unsafe { snprintf(out, room, c".%d".as_ptr(), prec) as size_t };
    }
    debug_assert!(l + 1 < format.len());
    // libc has no `%F`; it prints the same digits as `%f`.
    format[l] = if c.fmt_spec == b'F' { b'f' } else { c.fmt_spec } as c_char;
    format[l + 1] = 0;

    let mut str_arg_l = unsafe { snprintf(tmp.as_mut_ptr(), TMP, format.as_ptr(), f) as size_t };
    debug_assert!(str_arg_l < TMP);

    if remove_trailing_zeroes {
        str_arg_l = unsafe { trim_float(c, tmp, str_arg_l) };
    } else {
        str_arg_l = unsafe { trim_exponent_width(c, tmp, str_arg_l) };
    }

    // A zero-padded signed value keeps its sign in front of the zeros.
    if c.zero_padding && c.min_field_width > str_arg_l && (tmp[0] as u8 == b'-' || c.force_sign) {
        c.zeros_to_pad = c.min_field_width - str_arg_l;
        c.zero_insertion_ind = 1;
    }
    Body::Tmp(str_arg_l)
}

/// Delete one byte at `at`, terminator included, and report the new length.
///
/// # Safety
///
/// `at` must point at `len` bytes the caller owns, readable and writable,
/// unaliased for the call.
unsafe fn delete_byte(at: *mut c_char, len: size_t) -> size_t {
    let n_len = unsafe { cstr::bytes_at(at.add(1)) }.len();
    unsafe { at.cast::<u8>().copy_from(at.add(1).cast(), n_len + 1) };
    len - 1
}

/// `%g`'s trailing-zero removal.
///
/// In fixed notation the zeros are at the end; in exponential notation they
/// are in front of the exponent, and the exponent itself also loses its `+`
/// and its own leading zeros first.
///
/// # Safety
///
/// `len` must be the length of the NUL-terminated rendering already in `tmp`,
/// and no greater than `TMP - 1`: the walk starts at `tmp[len - 1]` and reads
/// the terminator.
unsafe fn trim_float(c: &Conversion, tmp: &mut [c_char; TMP], mut len: size_t) -> size_t {
    let mut tp;
    if matches!(c.fmt_spec, b'f' | b'F') {
        tp = unsafe { tmp.as_mut_ptr().add(len).sub(1) };
    } else {
        // `as_mut_ptr`, not `as_ptr`: `delete_byte` writes through what
        // this hands back, and a pointer derived from a *shared* borrow
        // of `tmp` only grants read permission (Stacked Borrows).
        let e = if c.fmt_spec == b'e' { b'e' } else { b'E' } as c_int;
        tp = unsafe { vim_strchr(tmp.as_mut_ptr().cast_const(), e) };
        if tp.is_null() {
            return len;
        }
        if unsafe { *tp.add(1) as u8 } == b'+' {
            len = unsafe { delete_byte(tp.add(1), len) };
        }
        // Leading zeros of the exponent, past its sign.
        let i = if unsafe { *tp.add(1) as u8 } == b'-' {
            2
        } else {
            1
        };
        while unsafe { *tp.add(i) as u8 } == b'0' {
            len = unsafe { delete_byte(tp.add(i), len) };
        }
        tp = unsafe { tp.sub(1) };
    }

    // An explicit precision asked for those zeros; keep them.
    if !c.precision_specified {
        // Never past `tmp[2]`, so `0.0` keeps a digit either side of
        // the point.
        while tp > unsafe { tmp.as_mut_ptr().add(2) }
            && unsafe { *tp as u8 } == b'0'
            && unsafe { *tp.sub(1) as u8 } != b'.'
        {
            len = unsafe { delete_byte(tp, len) };
            tp = unsafe { tp.sub(1) };
        }
    }
    len
}

/// Normalise an exponent that libc padded to three digits down to two.
///
/// # Safety
///
/// `len` must be the length of the NUL-terminated rendering already in `tmp`,
/// and no greater than `TMP - 1`.
unsafe fn trim_exponent_width(c: &Conversion, tmp: &mut [c_char; TMP], len: size_t) -> size_t {
    // Only the conversion's own case is looked for, so `%f` -- which
    // has no exponent -- never matches.
    let e = if c.fmt_spec == b'e' { b'e' } else { b'E' } as c_int;
    let tp = unsafe { vim_strchr(tmp.as_ptr(), e) };
    if !tp.is_null()
        && matches!(unsafe { *tp.add(1) as u8 }, b'+' | b'-')
        && unsafe { *tp.add(2) as u8 } == b'0'
        && ascii_isdigit(unsafe { *tp.add(3) as c_int })
        && ascii_isdigit(unsafe { *tp.add(4) as c_int })
    {
        return unsafe { delete_byte(tp.add(2), len) };
    }
    len
}
