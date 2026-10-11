//! Format specifiers: typing them, and the `%N$` positional pass.
//!
//! `format_typeof` reduces a conversion plus its length modifier to one of the
//! `ArgType` classes, and `format_typename` names that class for an error
//! message.  `parse_fmt_types` is the pre-pass positional arguments force: the
//! whole format is walked first to learn each position's type, `adjust_types`
//! recording one and rejecting a position used at two incompatible types.
//!
//! The format is a byte slice and every cursor an offset into it; a read past
//! its end answers the NUL the C string had there ([`byte_at`]), which is what
//! keeps every scan below the C one, step for step.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::semsg;
use core::ffi::{CStr, c_int, c_uint};

use crate::ascii::ascii_isdigit;
use crate::message_fmt::{msg_bytes, msg_cstr};
use crate::os::cshim::gettext;

/// The format string cannot be used, and the `E15xx` saying why has already
/// been reported.
///
/// The positional pre-pass either types every `%N$` in the format or gives up
/// on the whole format; there is no partial answer to carry.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct BadFormat;

/// A field width or precision may not exceed 1 MiB.
pub(crate) const MAX_ALLOWED_STRING_WIDTH: c_int = 1048576;

/// The byte at `at`, or the terminator past the end.
pub(super) fn byte_at(fmt: &[u8], at: usize) -> u8 {
    fmt.get(at).copied().unwrap_or(0)
}

/// Whether the byte at `at` is a decimal digit.
fn digit_at(fmt: &[u8], at: usize) -> bool {
    ascii_isdigit(c_int::from(byte_at(fmt, at)))
}

/// The classes a conversion plus its length modifier reduces to.
///
/// Two uses of one position must agree on it: `%1$d %1$s` names one
/// argument as two different things.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArgType {
    Unknown,
    Int,
    LongInt,
    LongLongInt,
    SignedSizeT,
    UnsignedInt,
    UnsignedLongInt,
    UnsignedLongLongInt,
    SizeT,
    Pointer,
    Percent,
    Char,
    Str,
    Float,
}

impl ArgType {
    /// The name this class is reported by in `E1502`/`E1504`.
    fn name(self) -> &'static CStr {
        match self {
            ArgType::Int => c"int",
            ArgType::LongInt => c"long int",
            ArgType::LongLongInt => c"long long int",
            ArgType::SignedSizeT => c"signed size_t",
            ArgType::UnsignedInt => c"unsigned int",
            ArgType::UnsignedLongInt => c"unsigned long int",
            ArgType::UnsignedLongLongInt => c"unsigned long long int",
            ArgType::SizeT => c"size_t",
            ArgType::Pointer => c"pointer",
            ArgType::Percent => c"percent",
            ArgType::Char => c"char",
            ArgType::Str => c"string",
            ArgType::Float => c"float",
            ArgType::Unknown => c"unknown",
        }
    }
}

/// The class of the conversion `spec` opens with, length modifier included.
///
/// `spec` starts *inside* a format string, at the first character after the
/// flags/width/precision -- so it may open with a length modifier, and it
/// runs on past the end of the conversion.
pub(crate) fn format_typeof(spec: &[u8]) -> ArgType {
    let mut at = 0;
    // Allowed length modifiers: none, h, l, ll (recorded as 'L'), z.
    let mut length_modifier = 0;
    if matches!(byte_at(spec, at), b'h' | b'l' | b'z') {
        length_modifier = byte_at(spec, at);
        at += 1;
        if length_modifier == b'l' && byte_at(spec, at) == b'l' {
            length_modifier = b'L';
            at += 1;
        }
    }

    // Synonyms, each of which implies a length modifier of its own.
    let (fmt_spec, length_modifier) = match byte_at(spec, at) {
        b'i' => (b'd', length_modifier),
        b'*' => (b'd', b'h'),
        b'D' => (b'd', b'l'),
        b'U' => (b'u', b'l'),
        b'O' => (b'o', b'l'),
        other => (other, length_modifier),
    };

    match fmt_spec {
        b'%' => ArgType::Percent,
        b'c' => ArgType::Char,
        b's' | b'S' => ArgType::Str,
        b'f' | b'F' | b'e' | b'E' | b'g' | b'G' => ArgType::Float,
        b'p' => ArgType::Pointer,
        // `b`/`B` always read the widest unsigned type.
        b'b' | b'B' => ArgType::UnsignedLongLongInt,
        // `d` is the only signed one; `u o x X` are unsigned.
        b'd' => match length_modifier {
            // char and short arguments are promoted to int.
            0 | b'h' => ArgType::Int,
            b'l' => ArgType::LongInt,
            b'L' => ArgType::LongLongInt,
            b'z' => ArgType::SignedSizeT,
            _ => ArgType::Unknown,
        },
        b'u' | b'o' | b'x' | b'X' => match length_modifier {
            0 | b'h' => ArgType::UnsignedInt,
            b'l' => ArgType::UnsignedLongInt,
            b'L' => ArgType::UnsignedLongLongInt,
            b'z' => ArgType::SizeT,
            _ => ArgType::Unknown,
        },
        _ => ArgType::Unknown,
    }
}

/// The translated name of `spec`'s class, for an error message.
fn format_typename(spec: &[u8]) -> &'static CStr {
    gettext(format_typeof(spec).name())
}

/// Where in the format each position was typed: `types[n - 1]` is the offset
/// of the conversion that named argument `n` first, or `None`.
pub(super) type PosTypes = Vec<Option<usize>>;

/// Record that positional argument `arg` (one-based) is used at the type
/// the conversion at `fmt[spec..]` spells, growing `types` to fit and
/// rejecting a position used at two incompatible types.
fn adjust_types(
    types: &mut PosTypes,
    arg: c_int,
    fmt: &[u8],
    spec: usize,
) -> Result<(), BadFormat> {
    let Ok(slot) = usize::try_from(arg - 1) else {
        let spec = msg_bytes(&fmt[spec.min(fmt.len())..]);
        semsg!("E1505: Invalid format specifier: {spec}");
        return Err(BadFormat);
    };
    if types.len() <= slot {
        types.resize(slot + 1, None);
    }

    if let Some(seen) = types[slot] {
        let (seen_text, now_text) = (&fmt[seen..], &fmt[spec..]);
        if byte_at(fmt, seen) == b'*' || byte_at(fmt, spec) == b'*' {
            // One of the two uses this position as a `*` field width,
            // so the *other* one has to be an integer. If both are,
            // there is nothing left to check.
            let other = if byte_at(fmt, spec) == b'*' {
                seen
            } else {
                spec
            };
            if byte_at(fmt, other) != b'*' && !matches!(byte_at(fmt, other), b'd' | b'i') {
                let was = msg_cstr(format_typename(seen_text));
                let now = msg_cstr(format_typename(now_text));
                semsg!(
                    "E1502: Positional argument {} used as field width reused as different type: {was}/{now}",
                    arg
                );
                return Err(BadFormat);
            }
        } else if format_typeof(now_text) != format_typeof(seen_text) {
            let now = msg_cstr(format_typename(now_text));
            let was = msg_cstr(format_typename(seen_text));
            semsg!(
                "E1504: Positional argument {} type used inconsistently: {now}/{was}",
                arg
            );
            return Err(BadFormat);
        }
    }

    types[slot] = Some(spec);
    Ok(())
}

/// `E1510`, quoting only the digits at `fmt[start..]` that overflowed.
pub(crate) fn format_overflow_error(fmt: &[u8], start: usize) {
    let mut end = start;
    while digit_at(fmt, end) {
        end += 1;
    }
    let digits = msg_bytes(&fmt[start.min(fmt.len())..end.min(fmt.len())]);
    semsg!("E1510: Value too large: {digits}");
}

/// Read the decimal number at `*at`, advancing it past the digits.
///
/// The scan itself stops once the value passes `MAX_ALLOWED_STRING_WIDTH`,
/// so an absurdly long run of digits cannot overflow the accumulator.
/// `overflow_err` then decides between raising `E1510` (quoting the digits
/// from `start`) and clamping -- `printf()` raises, an internal
/// `vim_snprintf` clamps. The first byte is taken as a digit unchecked, as
/// the C does: `%$d` reads `'$'` as one, and overflows.
pub(crate) fn get_unsigned_int(
    fmt: &[u8],
    start: usize,
    at: &mut usize,
    overflow_err: bool,
) -> Option<c_uint> {
    let digit = |b: u8| (c_int::from(b) - c_int::from(b'0')).cast_unsigned();
    let mut uj = digit(byte_at(fmt, *at));
    *at += 1;
    while digit_at(fmt, *at) && uj < MAX_ALLOWED_STRING_WIDTH.cast_unsigned() {
        uj = 10u32
            .wrapping_mul(uj)
            .wrapping_add(digit(byte_at(fmt, *at)));
        *at += 1;
    }
    if uj > MAX_ALLOWED_STRING_WIDTH.cast_unsigned() {
        if overflow_err {
            format_overflow_error(fmt, start);
            return None;
        }
        uj = MAX_ALLOWED_STRING_WIDTH.cast_unsigned();
    }
    Some(uj)
}

/// Walk `fmt` once, recording the type of every `%N$` position.
///
/// `typed` is set for Vimscript's `printf()`, whose out-of-range widths are
/// worth reporting; `argc` is how many arguments it was handed, which a
/// position may not pass. On failure `types` is cleared, so the caller may
/// simply stop.
pub(crate) fn parse_fmt_types(
    types: &mut PosTypes,
    fmt: &[u8],
    typed: bool,
    argc: Option<usize>,
) -> Result<(), BadFormat> {
    let result = scan_fmt_types(types, fmt, typed, argc);
    if result.is_err() {
        types.clear();
    }
    result
}

/// `parse_fmt_types`' body; the caller owns the cleanup.
fn scan_fmt_types(
    types: &mut PosTypes,
    fmt: &[u8],
    typed: bool,
    argc: Option<usize>,
) -> Result<(), BadFormat> {
    // A format may address its arguments positionally (`%2$s`) or in
    // order, never both.
    let mut any_pos = false;
    let mut any_arg = false;
    let mut p = 0;

    macro_rules! check_pos_arg {
        () => {
            if any_pos && any_arg {
                let arg0 = msg_bytes(fmt);
                semsg!("E1500: Cannot mix positional and non-positional arguments: {arg0}");
                return Err(BadFormat);
            }
        };
    }
    macro_rules! invalid_specifier {
        () => {{
            let arg0 = msg_bytes(fmt);
            semsg!("E1505: Invalid format specifier: {arg0}");
            return Err(BadFormat);
        }};
    }

    while p < fmt.len() {
        if fmt[p] != b'%' {
            // Skip to the next conversion in one step.
            p = fmt[p + 1..]
                .iter()
                .position(|&b| b == b'%')
                .map_or(fmt.len(), |at| p + 1 + at);
            continue;
        }

        let pstart = p + 1;
        p += 1; // step over the '%'

        // A leading run of digits followed by '$' is a position.
        let mut pos_arg = -1;
        let mut ptype = p;
        while digit_at(fmt, ptype) {
            ptype += 1;
        }
        if byte_at(fmt, ptype) == b'$' {
            if byte_at(fmt, p) == b'0' {
                invalid_specifier!(); // a '0' flag before the position
            }
            let uj = get_unsigned_int(fmt, pstart, &mut p, typed).ok_or(BadFormat)?;
            pos_arg = uj.cast_signed();
            any_pos = true;
            check_pos_arg!();
            p += 1; // step over the '$'
        }

        // Flags. Which ones win over which is the emitter's problem;
        // here they are only skipped.
        while matches!(byte_at(fmt, p), b'0' | b'-' | b'+' | b' ' | b'#' | b'\'') {
            p += 1;
        }

        // Field width, then precision: the same two shapes each time,
        // `*` (an argument, possibly positional) or a literal number.
        let mut arg = p;
        if byte_at(fmt, arg) == b'*' {
            p += 1;
            if digit_at(fmt, p) {
                let uj = get_unsigned_int(fmt, arg + 1, &mut p, typed).ok_or(BadFormat)?;
                if byte_at(fmt, p) != b'$' {
                    invalid_specifier!();
                }
                p += 1;
                any_pos = true;
                check_pos_arg!();
                adjust_types(types, uj.cast_signed(), fmt, arg)?;
            } else {
                any_arg = true;
                check_pos_arg!();
            }
        } else if digit_at(fmt, p) {
            // A literal width. Read it only to reject a `$` after it:
            // that would be a position in a place one cannot appear.
            let digstart = p;
            get_unsigned_int(fmt, digstart, &mut p, typed).ok_or(BadFormat)?;
            if byte_at(fmt, p) == b'$' {
                invalid_specifier!();
            }
        }

        if byte_at(fmt, p) == b'.' {
            p += 1;
            arg = p;
            if byte_at(fmt, arg) == b'*' {
                p += 1;
                if digit_at(fmt, p) {
                    let uj = get_unsigned_int(fmt, arg + 1, &mut p, typed).ok_or(BadFormat)?;
                    if byte_at(fmt, p) != b'$' {
                        invalid_specifier!();
                    }
                    any_pos = true;
                    check_pos_arg!();
                    p += 1;
                    adjust_types(types, uj.cast_signed(), fmt, arg)?;
                } else {
                    any_arg = true;
                    check_pos_arg!();
                }
            } else if digit_at(fmt, p) {
                let digstart = p;
                get_unsigned_int(fmt, digstart, &mut p, typed).ok_or(BadFormat)?;
                if byte_at(fmt, p) == b'$' {
                    invalid_specifier!();
                }
            }
        }

        if pos_arg != -1 {
            any_pos = true;
            check_pos_arg!();
            // The recorded type starts after the flags and width, not
            // at the '%': `%1$-8.3f` is typed as `f`, not as `-8.3f`.
            ptype = p;
        }

        // Length modifiers. `format_typeof` re-reads them off `ptype`,
        // so they are only stepped over here.
        if matches!(byte_at(fmt, p), b'h' | b'l' | b'z') {
            let length_modifier = byte_at(fmt, p);
            p += 1;
            if length_modifier == b'l' && byte_at(fmt, p) == b'l' {
                p += 1;
            }
        }

        const KNOWN: &[u8] = b"i*duoDUOxXbBcsSpfFeEgG";
        if KNOWN.contains(&byte_at(fmt, p)) {
            if pos_arg != -1 {
                adjust_types(types, pos_arg, fmt, ptype)?;
            } else {
                any_arg = true;
                check_pos_arg!();
            }
        } else if pos_arg != -1 {
            // A position on something that is not a conversion.
            let arg0 = msg_bytes(fmt);
            semsg!("E1500: Cannot mix positional and non-positional arguments: {arg0}");
            return Err(BadFormat);
        }

        if p < fmt.len() {
            p += 1; // step over the conversion
        }
    }

    // Every position from 1 to the highest one used must have been
    // typed, and must have an argument behind it.
    for (arg_idx, typed_at) in types.iter().enumerate() {
        if typed_at.is_none() {
            let arg1 = msg_bytes(fmt);
            semsg!(
                "E1501: format argument {} unused in $-style format: {arg1}",
                arg_idx + 1
            );
            return Err(BadFormat);
        }
        if argc.is_some_and(|argc| arg_idx >= argc) {
            let arg1 = msg_bytes(fmt);
            semsg!(
                "E1503: Positional argument {} out of bounds: {arg1}",
                arg_idx + 1
            );
            return Err(BadFormat);
        }
    }
    Ok(())
}
