//! The floating-point conversions.
//!
//! `%f`, `%F`, `%e`, `%E`, `%g` and `%G`, and the two trims `%g` needs
//! afterwards.  Split out of [`super::emit`] for the file-size cap.
//!
//! The digits are libc's, through `os::cshim::format_double` -- the one
//! conversion the formatter still hands the C library, because glibc renders
//! them more cheaply than Rust does; under Miri they are Rust's, which are
//! the same (see `render_digits`).

#![forbid(unsafe_code)]

#[cfg(miri)]
use core::fmt::Write as _;

#[cfg(miri)]
use super::emit::TmpWriter;
use super::emit::{Args, Body, Conversion, TMP};
use super::infinity_str;
use crate::ascii::ascii_isdigit;

/// `%f`, `%F`, `%e`, `%E`, `%g` and `%G`.
///
/// `%g` is not rendered as such: it is resolved to `%f` or `%e` first, and
/// the trailing zeros it would have dropped are removed afterwards.
pub(super) fn render_float(
    c: &mut Conversion,
    args: &mut Args,
    tmp: &mut [u8; TMP],
) -> Body<'static> {
    let f = args.next_float();
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
        let text = infinity_str(f > 0.0, c.fmt_spec, c.force_sign, c.space_for_positive);
        tmp[..text.len()].copy_from_slice(text);
        c.zero_padding = false;
        return Body::Tmp(text.len());
    }
    if f.is_nan() {
        let nan: &[u8] = if c.fmt_spec.is_ascii_uppercase() {
            b"NAN"
        } else {
            b"nan"
        };
        tmp[..3].copy_from_slice(nan);
        c.zero_padding = false;
        return Body::Tmp(3);
    }

    // The C builds a libc format here: '%', an optional sign flag, an
    // optional precision, and the conversion.
    let precision = if c.precision_specified {
        // Bound the precision so the result still fits `tmp`: a fixed
        // conversion also spends digits on the integer part.
        let mut max_prec = TMP - 10;
        if matches!(c.fmt_spec, b'f' | b'F') && abs_f > 1.0 {
            max_prec -= abs_f.log10() as usize;
        }
        c.precision = c.precision.min(max_prec);
        c.precision
    } else {
        6 // C's default
    };

    let sign = c
        .force_sign
        .then_some(if c.space_for_positive { b' ' } else { b'+' });
    // libc has no `%F` for finite values that differs from `%f`.
    let conv = if c.fmt_spec == b'F' { b'f' } else { c.fmt_spec };
    let mut str_arg_l = render_digits(tmp, sign, precision, conv, f);
    debug_assert!(str_arg_l < TMP);

    if remove_trailing_zeroes {
        str_arg_l = trim_float(c, tmp, str_arg_l);
    } else {
        str_arg_l = trim_exponent_width(c, tmp, str_arg_l);
    }

    // A zero-padded signed value keeps its sign in front of the zeros.
    if c.zero_padding && c.min_field_width > str_arg_l && (tmp[0] == b'-' || c.force_sign) {
        c.zeros_to_pad = c.min_field_width - str_arg_l;
        c.zero_insertion_ind = 1;
    }
    Body::Tmp(str_arg_l)
}

/// `f` as C's `%<sign>.<precision><conv>` renders it, into `tmp`; answers
/// the length. libc's digits: glibc's `printf` is markedly cheaper than
/// Rust's exact formatter for the six-digit `%g` that `string()` of every
/// Float pays (Rust falls back to its bignum Dragon for most of them,
/// +0.3 % of `evalbench`), and the digits are the same either way.
#[cfg(not(miri))]
fn render_digits(
    tmp: &mut [u8; TMP],
    sign: Option<u8>,
    precision: usize,
    conv: u8,
    f: f64,
) -> usize {
    crate::os::cshim::format_double(tmp, sign, precision, conv, f)
}

/// Under Miri, which cannot call the C library, the same digits from Rust's
/// exact formatter: `{:.N}` and `{:.Ne}` are correctly rounded, ties to even,
/// which is what glibc prints (held to libc by the formatter's oracle and a
/// scratch comparison of 300,000 doubles at precisions up to 340); only the
/// exponent is spelled differently, and [`render_exponent`] respells it.
#[cfg(miri)]
fn render_digits(
    tmp: &mut [u8; TMP],
    sign: Option<u8>,
    precision: usize,
    conv: u8,
    f: f64,
) -> usize {
    let mut out = TmpWriter { buf: tmp, len: 0 };
    // The sign flag: a value whose sign bit is clear gains a `+` or a space.
    if let Some(sign) = sign
        && f.is_sign_positive()
    {
        out.buf[0] = sign;
        out.len = 1;
    }
    if conv == b'f' {
        let written = write!(out, "{f:.precision$}");
        debug_assert!(written.is_ok());
        out.len
    } else {
        render_exponent(out, f, precision, conv == b'E')
    }
}

/// `f` as C's `%.{precision}e` (or `%E`) spells it, appended to `out`;
/// answers the length written. Rust writes the mantissa C does but the
/// exponent bare (`e3`, `e-7`); C writes a sign and at least two digits.
#[cfg(miri)]
fn render_exponent(mut out: TmpWriter, f: f64, precision: usize, upper: bool) -> usize {
    let start = out.len;
    let written = write!(out, "{f:.precision$e}");
    debug_assert!(written.is_ok());
    let Some(e) = out.buf[start..out.len].iter().position(|&b| b == b'e') else {
        return out.len;
    };
    let e = start + e;
    let (negative, digits) = match out.buf[e + 1] {
        b'-' => (true, e + 2..out.len),
        _ => (false, e + 1..out.len),
    };
    let mut exponent = [0u8; 4];
    let n = digits.len();
    exponent[..n].copy_from_slice(&out.buf[digits]);
    out.buf[e] = if upper { b'E' } else { b'e' };
    out.buf[e + 1] = if negative { b'-' } else { b'+' };
    let mut at = e + 2;
    if n < 2 {
        out.buf[at] = b'0';
        at += 1;
    }
    out.buf[at..at + n].copy_from_slice(&exponent[..n]);
    at + n
}

/// Delete the byte at `at` from the `len` bytes of `tmp`; answers the new
/// length.
fn delete_byte(tmp: &mut [u8; TMP], at: usize, len: usize) -> usize {
    tmp.copy_within(at + 1..len, at);
    len - 1
}

/// The byte at `at` of a rendering `len` long, or the terminator past it.
fn byte(tmp: &[u8; TMP], len: usize, at: usize) -> u8 {
    if at < len { tmp[at] } else { 0 }
}

/// `%g`'s trailing-zero removal.
///
/// In fixed notation the zeros are at the end; in exponential notation they
/// are in front of the exponent, and the exponent itself also loses its `+`
/// and its own leading zeros first.
fn trim_float(c: &Conversion, tmp: &mut [u8; TMP], mut len: usize) -> usize {
    let mut tp;
    if matches!(c.fmt_spec, b'f' | b'F') {
        tp = len - 1;
    } else {
        let e = if c.fmt_spec == b'e' { b'e' } else { b'E' };
        let Some(at) = tmp[..len].iter().position(|&b| b == e) else {
            return len;
        };
        tp = at;
        if byte(tmp, len, tp + 1) == b'+' {
            len = delete_byte(tmp, tp + 1, len);
        }
        // Leading zeros of the exponent, past its sign.
        let i = if byte(tmp, len, tp + 1) == b'-' { 2 } else { 1 };
        while byte(tmp, len, tp + i) == b'0' {
            len = delete_byte(tmp, tp + i, len);
        }
        tp -= 1;
    }

    // An explicit precision asked for those zeros; keep them.
    if !c.precision_specified {
        // Never past `tmp[2]`, so `0.0` keeps a digit either side of
        // the point.
        while tp > 2 && tmp[tp] == b'0' && tmp[tp - 1] != b'.' {
            len = delete_byte(tmp, tp, len);
            tp -= 1;
        }
    }
    len
}

/// Normalise an exponent padded to three digits down to two.
fn trim_exponent_width(c: &Conversion, tmp: &mut [u8; TMP], len: usize) -> usize {
    // Only the conversion's own case is looked for, so `%f` -- which
    // has no exponent -- never matches.
    let e = if c.fmt_spec == b'e' { b'e' } else { b'E' };
    let at = |i: usize| byte(tmp, len, i);
    if let Some(tp) = tmp[..len].iter().position(|&b| b == e)
        && matches!(at(tp + 1), b'+' | b'-')
        && at(tp + 2) == b'0'
        && ascii_isdigit(at(tp + 3).into())
        && ascii_isdigit(at(tp + 4).into())
    {
        return delete_byte(tmp, tp + 2, len);
    }
    len
}
