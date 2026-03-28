//! Printf helper functions: type classification and formatting utilities.
//!
//! Implements the format type enum, `format_typeof`, `format_typename`,
//! `infinity_str`, `format_overflow_error`, `get_unsigned_int`,
//! `adjust_types`, and `parse_fmt_types` previously defined in strings.c.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::if_not_else)]

use std::ffi::{c_char, c_int, c_uint, c_void};

use libc;

// =============================================================================
// C FFI
// =============================================================================

extern "C" {
    fn gettext(msgid: *const c_char) -> *const c_char;
    fn semsg(fmt: *const c_char, ...) -> bool;
    fn xstrnsave(s: *const c_char, len: usize) -> *mut c_char;
    fn xfree(ptr: *mut c_void);
    #[link_name = "xcalloc"]
    fn nvim_xcalloc(count: usize, size: usize) -> *mut c_void;
    fn xrealloc(ptr: *mut c_void, size: usize) -> *mut c_void;
    // e_val_too_large: "E1510: Value too large: %s"
    static e_val_too_large: *const c_char;
}

// =============================================================================
// Type enum
// =============================================================================

/// Format type classification constants -- matches the C enum in strings.c.
#[repr(i32)]
#[allow(dead_code)]
pub enum FormatType {
    Unknown = -1,
    Int = 0,
    LongInt = 1,
    LongLongInt = 2,
    SignedSizeT = 3,
    UnsignedInt = 4,
    UnsignedLongInt = 5,
    UnsignedLongLongInt = 6,
    SizeT = 7,
    Pointer = 8,
    Percent = 9,
    Char = 10,
    String = 11,
    Float = 12,
}

// Convenience constants matching the C enum integer values.
const TYPE_UNKNOWN: c_int = -1;
const TYPE_INT: c_int = 0;
const TYPE_LONGINT: c_int = 1;
const TYPE_LONGLONGINT: c_int = 2;
const TYPE_SIGNEDSIZET: c_int = 3;
const TYPE_UNSIGNEDINT: c_int = 4;
const TYPE_UNSIGNEDLONGINT: c_int = 5;
const TYPE_UNSIGNEDLONGLONGINT: c_int = 6;
const TYPE_SIZET: c_int = 7;
const TYPE_POINTER: c_int = 8;
const TYPE_PERCENT: c_int = 9;
const TYPE_CHAR: c_int = 10;
const TYPE_STRING: c_int = 11;
const TYPE_FLOAT: c_int = 12;

// NUL character
const NUL: u8 = 0;

// Return codes matching C OK/FAIL
const OK: c_int = 1;
const FAIL: c_int = -1;

// Maximum allowed string width for printf (1 MiB)
pub const MAX_ALLOWED_STRING_WIDTH: u32 = 1_048_576;

// Error message constants (untranslated, used with gettext at call time)
static E_POSITIONAL_NUM_FIELD_SPEC_REUSED: &[u8] =
    b"E1502: Positional argument %d used as field width reused as different type: %s/%s\0";
static E_POSITIONAL_ARG_NUM_TYPE_INCONSISTENT: &[u8] =
    b"E1504: Positional argument %d type used inconsistently: %s/%s\0";

// =============================================================================
// Type name string constants (untranslated, N_() equivalents)
// =============================================================================

static TYPENAME_UNKNOWN: &[u8] = b"unknown\0";
static TYPENAME_INT: &[u8] = b"int\0";
static TYPENAME_LONGINT: &[u8] = b"long int\0";
static TYPENAME_LONGLONGINT: &[u8] = b"long long int\0";
static TYPENAME_SIGNEDSIZET: &[u8] = b"signed size_t\0";
static TYPENAME_UNSIGNEDINT: &[u8] = b"unsigned int\0";
static TYPENAME_UNSIGNEDLONGINT: &[u8] = b"unsigned long int\0";
static TYPENAME_UNSIGNEDLONGLONGINT: &[u8] = b"unsigned long long int\0";
static TYPENAME_SIZET: &[u8] = b"size_t\0";
static TYPENAME_POINTER: &[u8] = b"pointer\0";
static TYPENAME_PERCENT: &[u8] = b"percent\0";
static TYPENAME_CHAR: &[u8] = b"char\0";
static TYPENAME_STRING: &[u8] = b"string\0";
static TYPENAME_FLOAT: &[u8] = b"float\0";

// =============================================================================
// format_typeof
// =============================================================================

/// Determine the format type classification from a format specifier string.
///
/// `type_spec` points to the character(s) after the `%` and any flags/width
/// in a printf format string (i.e. the length modifier + conversion char).
///
/// Returns one of the `TYPE_*` constants.
///
/// # Safety
/// `type_spec` must be a valid non-null pointer to at least one readable byte.
#[no_mangle]
pub unsafe extern "C" fn rs_format_typeof(type_spec: *const c_char) -> c_int {
    unsafe {
        let bytes = type_spec as *const u8;

        // allowed values: NUL, h, l, L
        let mut length_modifier: u8 = NUL;
        let mut offset: usize = 0;

        // parse 'h', 'l', 'll' and 'z' length modifiers
        let first = *bytes.add(offset);
        if first == b'h' || first == b'l' || first == b'z' {
            length_modifier = first;
            offset += 1;
            if length_modifier == b'l' && *bytes.add(offset) == b'l' {
                // double l = long long
                length_modifier = b'L';
                offset += 1;
            }
        }

        let mut fmt_spec = *bytes.add(offset);

        // common synonyms
        match fmt_spec {
            b'i' => {
                fmt_spec = b'd';
            }
            b'*' => {
                fmt_spec = b'd';
                length_modifier = b'h';
            }
            b'D' => {
                fmt_spec = b'd';
                length_modifier = b'l';
            }
            b'U' => {
                fmt_spec = b'u';
                length_modifier = b'l';
            }
            b'O' => {
                fmt_spec = b'o';
                length_modifier = b'l';
            }
            _ => {}
        }

        match fmt_spec {
            b'%' => TYPE_PERCENT,
            b'c' => TYPE_CHAR,
            b's' | b'S' => TYPE_STRING,
            b'd' | b'u' | b'b' | b'B' | b'o' | b'x' | b'X' | b'p' => {
                if fmt_spec == b'p' {
                    TYPE_POINTER
                } else if fmt_spec == b'b' || fmt_spec == b'B' {
                    TYPE_UNSIGNEDLONGLONGINT
                } else if fmt_spec == b'd' {
                    // signed
                    match length_modifier {
                        NUL | b'h' => TYPE_INT,
                        b'l' => TYPE_LONGINT,
                        b'L' => TYPE_LONGLONGINT,
                        b'z' => TYPE_SIGNEDSIZET,
                        _ => TYPE_UNKNOWN,
                    }
                } else {
                    // unsigned
                    match length_modifier {
                        NUL | b'h' => TYPE_UNSIGNEDINT,
                        b'l' => TYPE_UNSIGNEDLONGINT,
                        b'L' => TYPE_UNSIGNEDLONGLONGINT,
                        b'z' => TYPE_SIZET,
                        _ => TYPE_UNKNOWN,
                    }
                }
            }
            b'f' | b'F' | b'e' | b'E' | b'g' | b'G' => TYPE_FLOAT,
            _ => TYPE_UNKNOWN,
        }
    }
}

// =============================================================================
// format_typename
// =============================================================================

/// Return a translated human-readable type name for a format specifier.
///
/// # Safety
/// `type_spec` must be a valid non-null pointer to at least one readable byte.
#[no_mangle]
pub unsafe extern "C" fn rs_format_typename(type_spec: *const c_char) -> *const c_char {
    unsafe {
        let raw: *const u8 = match rs_format_typeof(type_spec) {
            TYPE_INT => TYPENAME_INT.as_ptr(),
            TYPE_LONGINT => TYPENAME_LONGINT.as_ptr(),
            TYPE_LONGLONGINT => TYPENAME_LONGLONGINT.as_ptr(),
            TYPE_UNSIGNEDINT => TYPENAME_UNSIGNEDINT.as_ptr(),
            TYPE_SIGNEDSIZET => TYPENAME_SIGNEDSIZET.as_ptr(),
            TYPE_UNSIGNEDLONGINT => TYPENAME_UNSIGNEDLONGINT.as_ptr(),
            TYPE_UNSIGNEDLONGLONGINT => TYPENAME_UNSIGNEDLONGLONGINT.as_ptr(),
            TYPE_SIZET => TYPENAME_SIZET.as_ptr(),
            TYPE_POINTER => TYPENAME_POINTER.as_ptr(),
            TYPE_PERCENT => TYPENAME_PERCENT.as_ptr(),
            TYPE_CHAR => TYPENAME_CHAR.as_ptr(),
            TYPE_STRING => TYPENAME_STRING.as_ptr(),
            TYPE_FLOAT => TYPENAME_FLOAT.as_ptr(),
            _ => TYPENAME_UNKNOWN.as_ptr(),
        };
        gettext(raw.cast::<c_char>())
    }
}

// =============================================================================
// infinity_str
// =============================================================================

// Return the string representation of infinity for printf().
// Matches the C function:
//   static const char *infinity_str(bool positive, char fmt_spec,
//                                   int force_sign, int space_for_positive)
static INFINITY_TABLE: [&[u8]; 8] = [
    b"-inf\0", b"inf\0", b"+inf\0", b" inf\0", b"-INF\0", b"INF\0", b"+INF\0", b" INF\0",
];

/// Return the infinity string for printf formatting.
///
/// # Safety
/// Always safe -- returns a pointer to a static string.
#[no_mangle]
pub unsafe extern "C" fn rs_infinity_str(
    positive: bool,
    fmt_spec: c_char,
    force_sign: c_int,
    space_for_positive: c_int,
) -> *const c_char {
    let mut idx: usize = if positive {
        (1 + force_sign + force_sign * space_for_positive) as usize
    } else {
        0
    };
    // ASCII_ISUPPER: A-Z
    let b = fmt_spec as u8;
    if b.is_ascii_uppercase() {
        idx += 4;
    }
    INFINITY_TABLE[idx].as_ptr().cast::<c_char>()
}

// =============================================================================
// format_overflow_error
// =============================================================================

/// Emit "Value too large" error for an oversized numeric format specifier.
///
/// # Safety
/// `pstart` must be a valid pointer to a sequence of ASCII digit bytes
/// followed by at least one non-digit byte (or NUL).
#[no_mangle]
pub unsafe extern "C" fn rs_format_overflow_error(pstart: *const c_char) {
    unsafe {
        let mut p = pstart as *const u8;
        while (*p).is_ascii_digit() {
            p = p.add(1);
        }
        let arglen = p.offset_from(pstart as *const u8) as usize;
        let argcopy = xstrnsave(pstart, arglen);
        semsg(gettext(e_val_too_large), argcopy);
        xfree(argcopy.cast::<c_void>());
    }
}

// =============================================================================
// get_unsigned_int
// =============================================================================

/// Parse decimal digits from format string into an unsigned int with overflow
/// checking.
///
/// On entry, `*p` must point to the first digit character.
/// On return, `*p` points past the last consumed digit.
///
/// Returns OK (1) or FAIL (-1).
///
/// # Safety
/// - `pstart` must be a valid pointer to the start of the digit sequence.
/// - `p` must be a valid non-null pointer to a `*const c_char`.
/// - `uj` must be a valid non-null pointer to a `c_uint`.
#[no_mangle]
pub unsafe extern "C" fn rs_get_unsigned_int(
    pstart: *const c_char,
    p: *mut *const c_char,
    uj: *mut c_uint,
    overflow_err: bool,
) -> c_int {
    unsafe {
        let first = *(*p as *const u8);
        *uj = (first - b'0') as c_uint;
        *p = (*p).add(1);

        while (**p as u8).is_ascii_digit() && *uj < MAX_ALLOWED_STRING_WIDTH {
            *uj = 10 * *uj + (**p as u8 - b'0') as c_uint;
            *p = (*p).add(1);
        }

        if *uj > MAX_ALLOWED_STRING_WIDTH {
            if overflow_err {
                rs_format_overflow_error(pstart);
                return FAIL;
            }
            *uj = MAX_ALLOWED_STRING_WIDTH;
        }

        OK
    }
}

// =============================================================================
// adjust_types
// =============================================================================

/// Manage the positional argument type tracking array.
///
/// Grows `*ap_types` as needed using `xcalloc`/`xrealloc`. Validates that
/// the same positional argument is not used with inconsistent types.
///
/// Returns OK (1) or FAIL (-1).
///
/// # Safety
/// All pointer arguments must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_adjust_types(
    ap_types: *mut *mut *const c_char,
    arg: c_int,
    num_posarg: *mut c_int,
    type_spec: *const c_char,
) -> c_int {
    unsafe {
        if (*ap_types).is_null() || *num_posarg < arg {
            let new_types: *mut *const c_char = if (*ap_types).is_null() {
                nvim_xcalloc(arg as usize, std::mem::size_of::<*const c_char>()).cast()
            } else {
                xrealloc(
                    (*ap_types).cast::<c_void>(),
                    arg as usize * std::mem::size_of::<*const c_char>(),
                )
                .cast()
            };

            for idx in *num_posarg..arg {
                *new_types.add(idx as usize) = std::ptr::null();
            }

            *ap_types = new_types;
            *num_posarg = arg;
        }

        let slot = (*ap_types).add(arg as usize - 1);

        if !(*slot).is_null() {
            let existing = *slot;
            if *existing.cast::<u8>() == b'*' || *type_spec.cast::<u8>() == b'*' {
                // One of them is a field-width specifier ('*')
                let pt: *const c_char = if *type_spec.cast::<u8>() == b'*' {
                    existing
                } else {
                    type_spec
                };

                if *pt.cast::<u8>() != b'*' {
                    match *pt.cast::<u8>() {
                        b'd' | b'i' => {}
                        _ => {
                            semsg(
                                gettext(E_POSITIONAL_NUM_FIELD_SPEC_REUSED.as_ptr().cast()),
                                arg,
                                rs_format_typename(existing),
                                rs_format_typename(type_spec),
                            );
                            return FAIL;
                        }
                    }
                }
            } else if rs_format_typeof(type_spec) != rs_format_typeof(existing) {
                semsg(
                    gettext(E_POSITIONAL_ARG_NUM_TYPE_INCONSISTENT.as_ptr().cast()),
                    arg,
                    rs_format_typename(type_spec),
                    rs_format_typename(existing),
                );
                return FAIL;
            }
        }

        *slot = type_spec;
        OK
    }
}

// =============================================================================
// parse_fmt_types
// =============================================================================

// Error strings for parse_fmt_types (N_() equivalents -- gettext called at use time)
static E_CANNOT_MIX_POSITIONAL: &[u8] =
    b"E1500: Cannot mix positional and non-positional arguments: %s\0";
static E_FMT_ARG_NR_UNUSED: &[u8] = b"E1501: format argument %d unused in $-style format: %s\0";
static E_POSITIONAL_NR_OUT_OF_BOUNDS: &[u8] = b"E1503: Positional argument %d out of bounds: %s\0";
static E_INVALID_FORMAT_SPECIFIER: &[u8] = b"E1505: Invalid format specifier: %s\0";

const VAR_UNKNOWN: c_int = 0;

/// First-pass parser that validates format string and builds positional
/// argument type array.
///
/// Equivalent to C `parse_fmt_types`. On error, frees `*ap_types` and resets
/// `*num_posarg` to 0.
///
/// # Safety
/// - `ap_types` and `num_posarg` must be valid non-null pointers.
/// - `fmt` must be a valid C string or null.
/// - `tvs` must be null or a valid pointer to a VAR_UNKNOWN-terminated
///   typval_T array.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_fmt_types(
    ap_types: *mut *mut *const c_char,
    num_posarg: *mut c_int,
    fmt: *const c_char,
    tvs: *const c_void,
) -> c_int {
    unsafe {
        if fmt.is_null() {
            return OK;
        }

        // Helper: error cleanup and return FAIL
        let error = |ap_types: *mut *mut *const c_char, num_posarg: *mut c_int| -> c_int {
            xfree((*ap_types).cast::<c_void>());
            *ap_types = std::ptr::null_mut();
            *num_posarg = 0;
            FAIL
        };

        let mut p = fmt as *const u8;
        let mut any_pos: c_int = 0;
        let mut any_arg: c_int = 0;

        macro_rules! check_pos_arg {
            () => {
                if any_pos != 0 && any_arg != 0 {
                    semsg(gettext(E_CANNOT_MIX_POSITIONAL.as_ptr().cast()), fmt);
                    return error(ap_types, num_posarg);
                }
            };
        }

        while *p != NUL {
            if *p != b'%' {
                // skip to next '%'
                let q = libc::strchr((p as *const c_char).add(1), b'%' as c_int);
                let n = if q.is_null() {
                    libc::strlen(p as *const c_char)
                } else {
                    q.offset_from(p as *const c_char) as usize
                };
                p = p.add(n);
            } else {
                // variable for positional arg
                let mut pos_arg: c_int = -1;
                let pstart = (p as *const c_char).add(1);

                p = p.add(1); // skip '%'

                // Check for positional argument specifier
                let mut ptype = p;
                while (*ptype).is_ascii_digit() {
                    ptype = ptype.add(1);
                }

                if *ptype == b'$' {
                    if *p == b'0' {
                        // 0 flag at the wrong place
                        semsg(gettext(E_INVALID_FORMAT_SPECIFIER.as_ptr().cast()), fmt);
                        return error(ap_types, num_posarg);
                    }

                    // Positional argument
                    let mut uj: c_uint = 0;
                    let mut pp = p as *const c_char;
                    if rs_get_unsigned_int(pstart, &mut pp, &mut uj, !tvs.is_null()) == FAIL {
                        return error(ap_types, num_posarg);
                    }
                    p = pp as *const u8;

                    pos_arg = uj as c_int;
                    any_pos = 1;
                    check_pos_arg!();

                    p = p.add(1); // skip '$'
                }

                // parse flags: 0, -, +, space, #, '
                while *p == b'0'
                    || *p == b'-'
                    || *p == b'+'
                    || *p == b' '
                    || *p == b'#'
                    || *p == b'\''
                {
                    p = p.add(1);
                }

                // parse field width
                let arg = p as *const c_char;
                if *p == b'*' {
                    p = p.add(1);

                    if (*p).is_ascii_digit() {
                        // Positional argument field width
                        let mut uj: c_uint = 0;
                        let mut pp = p as *const c_char;
                        if rs_get_unsigned_int(arg.add(1), &mut pp, &mut uj, !tvs.is_null()) == FAIL
                        {
                            return error(ap_types, num_posarg);
                        }
                        p = pp as *const u8;

                        if *p != b'$' {
                            semsg(gettext(E_INVALID_FORMAT_SPECIFIER.as_ptr().cast()), fmt);
                            return error(ap_types, num_posarg);
                        }
                        p = p.add(1); // skip '$'
                        any_pos = 1;
                        check_pos_arg!();

                        if rs_adjust_types(ap_types, uj as c_int, num_posarg, arg) == FAIL {
                            return error(ap_types, num_posarg);
                        }
                    } else {
                        any_arg = 1;
                        check_pos_arg!();
                    }
                } else if (*p).is_ascii_digit() {
                    let digstart = p as *const c_char;
                    let mut uj: c_uint = 0;
                    let mut pp = p as *const c_char;
                    if rs_get_unsigned_int(digstart, &mut pp, &mut uj, !tvs.is_null()) == FAIL {
                        return error(ap_types, num_posarg);
                    }
                    p = pp as *const u8;

                    if *p == b'$' {
                        semsg(gettext(E_INVALID_FORMAT_SPECIFIER.as_ptr().cast()), fmt);
                        return error(ap_types, num_posarg);
                    }
                }

                // parse precision
                if *p == b'.' {
                    p = p.add(1);
                    let arg2 = p as *const c_char;

                    if *p == b'*' {
                        p = p.add(1);

                        if (*p).is_ascii_digit() {
                            // Parse precision positional
                            let mut uj: c_uint = 0;
                            let mut pp = p as *const c_char;
                            if rs_get_unsigned_int(arg2.add(1), &mut pp, &mut uj, !tvs.is_null())
                                == FAIL
                            {
                                return error(ap_types, num_posarg);
                            }
                            p = pp as *const u8;

                            if *p == b'$' {
                                any_pos = 1;
                                check_pos_arg!();

                                p = p.add(1); // skip '$'

                                if rs_adjust_types(ap_types, uj as c_int, num_posarg, arg2) == FAIL
                                {
                                    return error(ap_types, num_posarg);
                                }
                            } else {
                                semsg(gettext(E_INVALID_FORMAT_SPECIFIER.as_ptr().cast()), fmt);
                                return error(ap_types, num_posarg);
                            }
                        } else {
                            any_arg = 1;
                            check_pos_arg!();
                        }
                    } else if (*p).is_ascii_digit() {
                        let digstart = p as *const c_char;
                        let mut uj: c_uint = 0;
                        let mut pp = p as *const c_char;
                        if rs_get_unsigned_int(digstart, &mut pp, &mut uj, !tvs.is_null()) == FAIL {
                            return error(ap_types, num_posarg);
                        }
                        p = pp as *const u8;

                        if *p == b'$' {
                            semsg(gettext(E_INVALID_FORMAT_SPECIFIER.as_ptr().cast()), fmt);
                            return error(ap_types, num_posarg);
                        }
                    }
                }

                if pos_arg != -1 {
                    any_pos = 1;
                    check_pos_arg!();
                    ptype = p;
                }

                // parse 'h', 'l', 'll' and 'z' length modifiers
                if *p == b'h' || *p == b'l' || *p == b'z' {
                    let lm = *p;
                    p = p.add(1);
                    if lm == b'l' && *p == b'l' {
                        p = p.add(1);
                    }
                }

                match *p {
                    b'i' | b'*' | b'd' | b'u' | b'o' | b'D' | b'U' | b'O' | b'x' | b'X' | b'b'
                    | b'B' | b'c' | b's' | b'S' | b'p' | b'f' | b'F' | b'e' | b'E' | b'g'
                    | b'G' => {
                        if pos_arg != -1 {
                            if rs_adjust_types(
                                ap_types,
                                pos_arg,
                                num_posarg,
                                ptype as *const c_char,
                            ) == FAIL
                            {
                                return error(ap_types, num_posarg);
                            }
                        } else {
                            any_arg = 1;
                            check_pos_arg!();
                        }
                    }
                    _ => {
                        if pos_arg != -1 {
                            semsg(gettext(E_CANNOT_MIX_POSITIONAL.as_ptr().cast()), fmt);
                            return error(ap_types, num_posarg);
                        }
                    }
                }

                if *p != NUL {
                    p = p.add(1); // step over conversion specifier
                }
            }
        }

        // Validate all positional args are used and within tvs bounds
        for arg_idx in 0..*num_posarg {
            if (*(*ap_types).add(arg_idx as usize)).is_null() {
                semsg(
                    gettext(E_FMT_ARG_NR_UNUSED.as_ptr().cast()),
                    arg_idx + 1,
                    fmt,
                );
                return error(ap_types, num_posarg);
            }

            if !tvs.is_null() {
                // Access tvs[arg_idx].v_type via repr(C) struct stride
                let tv_arr = tvs.cast::<TvForTypval>();
                if (*tv_arr.add(arg_idx as usize)).v_type == VAR_UNKNOWN {
                    semsg(
                        gettext(E_POSITIONAL_NR_OUT_OF_BOUNDS.as_ptr().cast()),
                        arg_idx + 1,
                        fmt,
                    );
                    return error(ap_types, num_posarg);
                }
            }
        }

        OK
    }
}

/// Minimal repr(C) mirror of typval_T used only for array striding.
/// Matches the full struct layout: v_type(4), v_lock(4), vval(8) = 16 bytes.
/// We only read v_type; the rest is padding to get the correct stride.
#[repr(C)]
struct TvForTypval {
    v_type: c_int,
    v_lock: c_int,
    vval: u64, // largest union member is a pointer (8 bytes)
}
