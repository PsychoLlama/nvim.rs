//! Format specifiers: typing them, and the `%N$` positional pass.
//!
//! `format_typeof` reduces a conversion plus its length modifier to one of the
//! `ArgType` classes, and `format_typename` names that class for an error
//! message.  `parse_fmt_types` is the pre-pass positional arguments force: with
//! `%N$` the arguments are not consumed in order, so the whole format has to be
//! walked first to learn each position's type, `adjust_types` recording one and
//! rejecting a position used at two incompatible types.  `skip_to_arg` is the
//! lookup that pass exists to serve -- a `va_list` can only be walked forwards,
//! so reaching argument N means rewinding to the start and stepping over N-1
//! arguments *of the types this pass recorded*.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::message_fmt::{c_str, c_str_len};
use crate::semsg;
use crate::siemsg;
use core::ffi::{
    CStr, VaList, c_char, c_double, c_int, c_long, c_longlong, c_uint, c_ulong, c_ulonglong, c_void,
};
use core::ptr;

use crate::ascii::ascii_isdigit;
use crate::memory::{xcalloc, xfree, xrealloc, xstrchrnul};
use crate::os::cshim::gettext;
use crate::types::{VAR_UNKNOWN, size_t, typval_T};

/// The format string cannot be used, and the `E15xx` saying why has already
/// been reported.
///
/// The positional pre-pass either types every `%N$` in the format or gives up
/// on the whole format; there is no partial answer to carry, which is why
/// this is a unit struct rather than the `Result<(), ()>` it replaces.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct BadFormat;

/// A field width or precision may not exceed 1 MiB.
pub(crate) const MAX_ALLOWED_STRING_WIDTH: c_int = 1048576;

/// The classes a conversion plus its length modifier reduces to.
///
/// This is the whole reason the positional pass exists: a `va_list` can
/// only be stepped by *reading* an argument, and reading one needs its
/// type, so `%2$s` cannot be served without first learning what `%1$` was.
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

/// The class of the conversion `spec` points at, length modifier included.
///
/// `spec` points *into* a format string, at the first character after the
/// flags/width/precision — so it may be a length modifier, and it is never
/// NUL-terminated at the end of the conversion.
pub(crate) unsafe fn format_typeof(spec: *const c_char) -> ArgType {
    let mut spec = spec;
    // Allowed length modifiers: none, h, l, ll (recorded as 'L'), z.
    let mut length_modifier = 0;
    if matches!(unsafe { *spec as u8 }, b'h' | b'l' | b'z') {
        length_modifier = unsafe { *spec as u8 };
        spec = unsafe { spec.add(1) };
        if length_modifier == b'l' && unsafe { *spec as u8 } == b'l' {
            length_modifier = b'L';
            spec = unsafe { spec.add(1) };
        }
    }

    // Synonyms, each of which implies a length modifier of its own.
    let (fmt_spec, length_modifier) = match unsafe { *spec as u8 } {
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
unsafe fn format_typename(spec: *const c_char) -> *const c_char {
    unsafe { gettext(format_typeof(spec).name()).as_ptr() }
}

/// Record that positional argument `arg` (one-based) is used at the type
/// `spec` spells, growing `ap_types` to fit and rejecting a position used
/// at two incompatible types.
unsafe fn adjust_types(
    ap_types: &mut *mut *const c_char,
    arg: c_int,
    num_posarg: &mut c_int,
    spec: *const c_char,
) -> Result<(), BadFormat> {
    if arg <= 0 {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let spec = unsafe { c_str(spec) };
        semsg!("E1505: Invalid format specifier: {spec}");
        return Err(BadFormat);
    }

    if ap_types.is_null() || *num_posarg < arg {
        let entries = arg as size_t;
        let bytes = entries * size_of::<*const c_char>();
        let grown = if ap_types.is_null() {
            unsafe { xcalloc(entries, size_of::<*const c_char>()) }
        } else {
            unsafe { xrealloc(*ap_types as *mut c_void, bytes) }
        } as *mut *const c_char;
        // `xrealloc` does not zero, so the new tail must be.
        for idx in *num_posarg..arg {
            unsafe { *grown.offset(idx as isize) = ptr::null() };
        }
        *ap_types = grown;
        *num_posarg = arg;
    }

    let slot = unsafe { (*ap_types).offset(arg as isize - 1) };
    let seen = unsafe { *slot };
    if !seen.is_null() {
        if unsafe { *seen as u8 } == b'*' || unsafe { *spec as u8 } == b'*' {
            // One of the two uses this position as a `*` field width,
            // so the *other* one has to be an integer. If both are,
            // there is nothing left to check.
            let other = if unsafe { *spec as u8 } == b'*' {
                seen
            } else {
                spec
            };
            if unsafe { *other as u8 } != b'*' && !matches!(unsafe { *other as u8 }, b'd' | b'i') {
                let was = unsafe { format_typename(seen) };
                let now = unsafe { format_typename(spec) };
                // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
                let (was, now) = unsafe { (c_str(was), c_str(now)) };
                semsg!(
                    "E1502: Positional argument {} used as field width reused as different type: {was}/{now}",
                    arg
                );
                return Err(BadFormat);
            }
        } else if unsafe { format_typeof(spec) } != unsafe { format_typeof(seen) } {
            let now = unsafe { format_typename(spec) };
            let was = unsafe { format_typename(seen) };
            // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
            let (now, was) = unsafe { (c_str(now), c_str(was)) };
            semsg!(
                "E1504: Positional argument {} type used inconsistently: {now}/{was}",
                arg
            );
            return Err(BadFormat);
        }
    }

    unsafe { *slot = spec };
    Ok(())
}

/// `E1510`, quoting only the digits that overflowed.
pub(crate) unsafe fn format_overflow_error(pstart: *const c_char) {
    let mut p = pstart;
    while ascii_isdigit(unsafe { *p as c_int }) {
        p = unsafe { p.add(1) };
    }
    let digits = unsafe { p.offset_from(pstart) } as c_int;
    // SAFETY: a message argument the caller holds as a NUL-terminated string.
    let pstart = unsafe { c_str_len(pstart, digits as usize) };
    semsg!("E1510: Value too large: {pstart}");
}

/// Read the decimal number at `*p`, advancing it past the digits.
///
/// The scan itself stops once the value passes `MAX_ALLOWED_STRING_WIDTH`,
/// so an absurdly long run of digits cannot overflow the accumulator.
/// `overflow_err` then decides between raising `E1510` and clamping —
/// `printf()` raises, an internal `vim_snprintf` clamps.
pub(crate) unsafe fn get_unsigned_int(
    pstart: *const c_char,
    p: &mut *const c_char,
    overflow_err: bool,
) -> Option<c_uint> {
    let digit = |c: c_char| (c as c_int - '0' as c_int) as c_uint;
    let mut uj = digit(unsafe { **p });
    *p = unsafe { p.add(1) };
    while ascii_isdigit(unsafe { **p as c_int }) && uj < MAX_ALLOWED_STRING_WIDTH as c_uint {
        uj = 10u32.wrapping_mul(uj).wrapping_add(digit(unsafe { **p }));
        *p = unsafe { p.add(1) };
    }
    if uj > MAX_ALLOWED_STRING_WIDTH as c_uint {
        if overflow_err {
            unsafe { format_overflow_error(pstart) };
            return None;
        }
        uj = MAX_ALLOWED_STRING_WIDTH as c_uint;
    }
    Some(uj)
}

/// Walk `fmt` once, recording the type of every `%N$` position.
///
/// On failure `ap_types` is freed and both outputs are reset, so the
/// caller may simply stop.
pub(crate) unsafe fn parse_fmt_types(
    ap_types: &mut *mut *const c_char,
    num_posarg: &mut c_int,
    fmt: *const c_char,
    tvs: *mut typval_T,
) -> Result<(), BadFormat> {
    if fmt.is_null() {
        return Ok(());
    }
    let result = unsafe { scan_fmt_types(ap_types, num_posarg, fmt, tvs) };
    if result.is_err() {
        unsafe { xfree(*ap_types as *mut c_void) };
        *ap_types = ptr::null_mut();
        *num_posarg = 0;
    }
    result
}

/// `parse_fmt_types`' body; the caller owns the cleanup.
unsafe fn scan_fmt_types(
    ap_types: &mut *mut *const c_char,
    num_posarg: &mut c_int,
    fmt: *const c_char,
    tvs: *mut typval_T,
) -> Result<(), BadFormat> {
    // Whether the arguments are typvals, which is what says an out-of-range
    // width is worth reporting rather than ignoring.
    let typed = !tvs.is_null();
    // A format may address its arguments positionally (`%2$s`) or in
    // order, never both.
    let mut any_pos = false;
    let mut any_arg = false;
    let mut p = fmt;

    macro_rules! check_pos_arg {
        () => {
            if any_pos && any_arg {
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let arg0 = unsafe { c_str(fmt) };
                semsg!("E1500: Cannot mix positional and non-positional arguments: {arg0}");
                return Err(BadFormat);
            }
        };
    }
    macro_rules! invalid_specifier {
        () => {{
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg0 = unsafe { c_str(fmt) };
            semsg!("E1505: Invalid format specifier: {arg0}");
            return Err(BadFormat);
        }};
    }

    while unsafe { *p } != 0 {
        if unsafe { *p as u8 } != b'%' {
            // Skip to the next conversion in one step.
            p = unsafe { xstrchrnul(p.add(1), b'%' as c_char) };
            continue;
        }

        let pstart = unsafe { p.add(1) };
        p = unsafe { p.add(1) }; // step over the '%'

        // A leading run of digits followed by '$' is a position.
        let mut pos_arg = -1;
        let mut ptype = p;
        while ascii_isdigit(unsafe { *ptype as c_int }) {
            ptype = unsafe { ptype.add(1) };
        }
        if unsafe { *ptype as u8 } == b'$' {
            if unsafe { *p as u8 } == b'0' {
                invalid_specifier!(); // a '0' flag before the position
            }
            let uj = unsafe { get_unsigned_int(pstart, &mut p, typed) }.ok_or(BadFormat)?;
            pos_arg = uj as c_int;
            any_pos = true;
            check_pos_arg!();
            p = unsafe { p.add(1) }; // step over the '$'
        }

        // Flags. Which ones win over which is the emitter's problem;
        // here they are only skipped.
        while matches!(
            unsafe { *p as u8 },
            b'0' | b'-' | b'+' | b' ' | b'#' | b'\''
        ) {
            p = unsafe { p.add(1) };
        }

        // Field width, then precision: the same two shapes each time,
        // `*` (an argument, possibly positional) or a literal number.
        let mut arg = p;
        if unsafe { *arg as u8 } == b'*' {
            p = unsafe { p.add(1) };
            if ascii_isdigit(unsafe { *p as c_int }) {
                let uj = unsafe { get_unsigned_int(arg.add(1), &mut p, typed) }.ok_or(BadFormat)?;
                if unsafe { *p as u8 } != b'$' {
                    invalid_specifier!();
                }
                p = unsafe { p.add(1) };
                any_pos = true;
                check_pos_arg!();
                unsafe { adjust_types(ap_types, uj as c_int, num_posarg, arg) }?;
            } else {
                any_arg = true;
                check_pos_arg!();
            }
        } else if ascii_isdigit(unsafe { *p as c_int }) {
            // A literal width. Read it only to reject a `$` after it:
            // that would be a position in a place one cannot appear.
            let digstart = p;
            unsafe { get_unsigned_int(digstart, &mut p, typed) }.ok_or(BadFormat)?;
            if unsafe { *p as u8 } == b'$' {
                invalid_specifier!();
            }
        }

        if unsafe { *p as u8 } == b'.' {
            p = unsafe { p.add(1) };
            arg = p;
            if unsafe { *arg as u8 } == b'*' {
                p = unsafe { p.add(1) };
                if ascii_isdigit(unsafe { *p as c_int }) {
                    let uj =
                        unsafe { get_unsigned_int(arg.add(1), &mut p, typed) }.ok_or(BadFormat)?;
                    if unsafe { *p as u8 } != b'$' {
                        invalid_specifier!();
                    }
                    any_pos = true;
                    check_pos_arg!();
                    p = unsafe { p.add(1) };
                    unsafe { adjust_types(ap_types, uj as c_int, num_posarg, arg) }?;
                } else {
                    any_arg = true;
                    check_pos_arg!();
                }
            } else if ascii_isdigit(unsafe { *p as c_int }) {
                let digstart = p;
                unsafe { get_unsigned_int(digstart, &mut p, typed) }.ok_or(BadFormat)?;
                if unsafe { *p as u8 } == b'$' {
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
        if matches!(unsafe { *p as u8 }, b'h' | b'l' | b'z') {
            let length_modifier = unsafe { *p as u8 };
            p = unsafe { p.add(1) };
            if length_modifier == b'l' && unsafe { *p as u8 } == b'l' {
                p = unsafe { p.add(1) };
            }
        }

        const KNOWN: &[u8] = b"i*duoDUOxXbBcsSpfFeEgG";
        if KNOWN.contains(&(unsafe { *p as u8 })) {
            if pos_arg != -1 {
                unsafe { adjust_types(ap_types, pos_arg, num_posarg, ptype) }?;
            } else {
                any_arg = true;
                check_pos_arg!();
            }
        } else if pos_arg != -1 {
            // A position on something that is not a conversion.
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg0 = unsafe { c_str(fmt) };
            semsg!("E1500: Cannot mix positional and non-positional arguments: {arg0}");
            return Err(BadFormat);
        }

        if unsafe { *p } != 0 {
            p = unsafe { p.add(1) }; // step over the conversion
        }
    }

    // Every position from 1 to the highest one used must have been
    // typed, and must have an argument behind it.
    for arg_idx in 0..*num_posarg {
        if unsafe { (*(*ap_types).offset(arg_idx as isize)).is_null() } {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg1 = unsafe { c_str(fmt) };
            semsg!(
                "E1501: format argument {} unused in $-style format: {arg1}",
                arg_idx + 1
            );
            return Err(BadFormat);
        }
        if !tvs.is_null() && unsafe { (*tvs.offset(arg_idx as isize)).v_type } == VAR_UNKNOWN {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg1 = unsafe { c_str(fmt) };
            semsg!(
                "E1503: Positional argument {} out of bounds: {arg1}",
                arg_idx + 1
            );
            return Err(BadFormat);
        }
    }
    Ok(())
}

/// Position `ap` on positional argument `*arg_idx`.
///
/// Hand-ported from neovim's static `skip_to_arg` in `src/nvim/strings.c`:
/// c2rust dropped the definition (it takes `va_list` by value, which its
/// variadic support could not translate) yet still emitted the 17 call sites
/// in `vim_vsnprintf_typval`.
///
/// A `va_list` only moves forwards, so reaching an argument that is behind
/// the cursor means restarting from `ap_start` and *reading* every argument
/// in between — at the types `parse_fmt_types` recorded, which is what
/// `ap_types` is for. `arg_cur` tracks where the list actually is; the
/// common case, the next argument in order, is the early return.
pub(crate) unsafe fn skip_to_arg<'f>(
    ap_types: *mut *const c_char,
    ap_start: VaList<'f>,
    ap: *mut VaList<'f>,
    arg_idx: *mut c_int,
    arg_cur: *mut c_int,
    fmt: *const c_char,
) {
    if unsafe { *arg_cur } + 1 == unsafe { *arg_idx } {
        unsafe { *arg_cur += 1 };
        unsafe { *arg_idx += 1 };
        return;
    }

    let arg_min = if unsafe { *arg_cur } >= unsafe { *arg_idx } {
        // Already past it: rewind (va_end + va_copy) and re-walk.
        unsafe { *ap = ap_start.clone() };
        0
    } else {
        unsafe { *arg_cur }
    };

    unsafe { *arg_cur = arg_min };
    while unsafe { *arg_cur } < unsafe { *arg_idx } - 1 {
        if ap_types.is_null() || unsafe { (*ap_types.offset(*arg_cur as isize)).is_null() } {
            // SAFETY: `arg_cur` is the caller's index cell and `fmt` its
            // NUL-terminated format string.
            let (at, fmt) = unsafe { (*arg_cur, c_str(fmt)) };
            siemsg!("E1507: Internal error: ap_types or ap_types[idx] is NULL: {fmt}: {at}");
            return;
        }
        // Consume one argument at its recorded width.
        match unsafe { format_typeof(*ap_types.offset(*arg_cur as isize)) } {
            ArgType::Percent | ArgType::Unknown => {}
            ArgType::Char | ArgType::Int => {
                unsafe { (*ap).next_arg::<c_int>() };
            }
            ArgType::Str => {
                unsafe { (*ap).next_arg::<*const c_char>() };
            }
            ArgType::Pointer => {
                unsafe { (*ap).next_arg::<*mut c_void>() };
            }
            ArgType::LongInt => {
                unsafe { (*ap).next_arg::<c_long>() };
            }
            ArgType::LongLongInt => {
                unsafe { (*ap).next_arg::<c_longlong>() };
            }
            // Implementation-defined, usually ptrdiff_t.
            ArgType::SignedSizeT => {
                unsafe { (*ap).next_arg::<isize>() };
            }
            ArgType::UnsignedInt => {
                unsafe { (*ap).next_arg::<c_uint>() };
            }
            ArgType::UnsignedLongInt => {
                unsafe { (*ap).next_arg::<c_ulong>() };
            }
            ArgType::UnsignedLongLongInt => {
                unsafe { (*ap).next_arg::<c_ulonglong>() };
            }
            ArgType::SizeT => {
                unsafe { (*ap).next_arg::<size_t>() };
            }
            ArgType::Float => {
                unsafe { (*ap).next_arg::<c_double>() };
            }
        }
        unsafe { *arg_cur += 1 };
    }

    // The caller reads an argument right after this returns, so the
    // cursor is moved on pre-emptively.
    unsafe { *arg_cur += 1 };
    unsafe { *arg_idx += 1 };
}
