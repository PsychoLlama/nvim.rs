//! Printf helper functions: type classification and formatting utilities.
//!
//! Implements the format type enum, `format_typeof`, `format_typename`,
//! `infinity_str`, `format_overflow_error`, `get_unsigned_int`,
//! `adjust_types`, `parse_fmt_types`, and the full printf formatting engine
//! (`rs_vim_vsnprintf_extracted`) previously defined in strings.c.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::if_not_else)]
#![allow(clippy::manual_c_str_literals)]
#![allow(clippy::while_immutable_condition)]
#![allow(clippy::bool_to_int_with_if)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::option_map_or_none)]
#![allow(clippy::self_assignment)]
#![allow(clippy::useless_let_if_seq)]
#![allow(clippy::float_arithmetic)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::if_let_mutex)]
#![allow(clippy::redundant_field_names)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::needless_pass_by_ref_mut)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(clippy::let_and_return)]
#![allow(clippy::ref_as_ptr)]
#![allow(clippy::comparison_chain)]
#![allow(clippy::suboptimal_flops)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::items_after_test_module)]
#![allow(clippy::wildcard_imports)]

use std::ffi::{c_char, c_int, c_uint, c_void};

use libc;

// =============================================================================
// C FFI
// =============================================================================

extern "C" {
    fn gettext(msgid: *const c_char) -> *const c_char;
    fn semsg(fmt: *const c_char, ...) -> bool;
    pub(crate) fn emsg(s: *const c_char) -> bool;
    fn xstrnsave(s: *const c_char, len: usize) -> *mut c_char;
    pub(crate) fn xfree(ptr: *mut c_void);
    #[link_name = "xcalloc"]
    fn nvim_xcalloc(count: usize, size: usize) -> *mut c_void;
    fn xrealloc(ptr: *mut c_void, size: usize) -> *mut c_void;
    // e_val_too_large: "E1510: Value too large: %s"
    static e_val_too_large: *const c_char;

    // UTF8 / multibyte helpers
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn utf_ptr2cells(p: *const c_char) -> c_int;

    // Memory search
    fn xmemscan(addr: *const c_void, c: c_char, size: usize) -> *mut c_void;
    fn xstrchrnul(str_: *const c_char, c: c_int) -> *mut c_char;
    fn xstrlcpy(dst: *mut c_char, src: *const c_char, dstsize: usize) -> usize;

    // typval accessors (using *mut c_void to match viml.rs declarations)
    fn tv_get_number_chk(tv: *mut c_void, error: *mut bool) -> i64;
    fn tv_get_string_chk(tv: *mut c_void) -> *const c_char;
    fn encode_tv2echo(tv: *mut c_void, len: *mut usize) -> *mut c_char;
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
        let mut seq_arg: c_int = 0;

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
                        seq_arg += 1;
                        any_arg = 1;
                        check_pos_arg!();
                        if rs_adjust_types(ap_types, seq_arg, num_posarg, arg) == FAIL {
                            return error(ap_types, num_posarg);
                        }
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
                            seq_arg += 1;
                            any_arg = 1;
                            check_pos_arg!();
                            if rs_adjust_types(ap_types, seq_arg, num_posarg, arg2) == FAIL {
                                return error(ap_types, num_posarg);
                            }
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

                ptype = p; // Save position before length modifiers for type resolution
                if pos_arg != -1 {
                    any_pos = 1;
                    check_pos_arg!();
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
                            seq_arg += 1;
                            any_arg = 1;
                            check_pos_arg!();
                            if rs_adjust_types(
                                ap_types,
                                seq_arg,
                                num_posarg,
                                ptype as *const c_char,
                            ) == FAIL
                            {
                                return error(ap_types, num_posarg);
                            }
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

// VAR_* type constants (matching typval_defs.h VarType enum)
const VAR_NUMBER: c_int = 1;
const VAR_STRING: c_int = 2;
const VAR_FLOAT: c_int = 6;

// =============================================================================
// FmtArg: pre-extracted argument union (passed from C to Rust)
// =============================================================================

/// Tagged union holding one pre-extracted printf argument.
/// The C `extract_va_args` function fills an array of these by calling va_arg.
#[repr(C)]
pub struct FmtArg {
    pub tag: c_int, // TYPE_INT, TYPE_STRING, etc.
    pub val: FmtArgVal,
}

/// The value part of a pre-extracted printf argument.
#[repr(C)]
pub union FmtArgVal {
    pub i: c_int,
    pub l: libc::c_long,
    pub ll: libc::c_longlong,
    pub z: isize,
    pub u: c_uint,
    pub ul: libc::c_ulong,
    pub ull: libc::c_ulonglong,
    pub uz: usize,
    pub f: f64,
    pub s: *const c_char,
    pub p: *const c_void,
}

// =============================================================================
// typval helpers (tv_nr, tv_str, tv_ptr, tv_float) -- Rust implementation
// =============================================================================

static E_PRINTF: &[u8] = b"E766: Insufficient arguments for printf()\0";
static E_FLOAT_ARG: &[u8] = b"E807: Expected Float argument for printf()\0";

/// Get a number from tvs[*idxp - 1], incrementing *idxp. Returns 0 on error.
unsafe fn tv_nr_rs(tvs: *const c_void, idxp: *mut c_int) -> i64 {
    unsafe {
        let idx = *idxp - 1;
        let tv = (tvs as *const TvForTypval).add(idx as usize);
        if (*tv).v_type == VAR_UNKNOWN {
            emsg(gettext(E_PRINTF.as_ptr().cast()));
            return 0;
        }
        *idxp += 1;
        let mut err = false;
        let n = tv_get_number_chk(tv as *mut TvForTypval as *mut c_void, &mut err);
        if err {
            0
        } else {
            n
        }
    }
}

/// Get a string from tvs[*idxp - 1], incrementing *idxp.
/// Sets *tofree to a heap string that must be freed if conversion was needed.
/// Returns NULL on error.
unsafe fn tv_str_rs(
    tvs: *const c_void,
    idxp: *mut c_int,
    tofree: *mut *mut c_char,
) -> *const c_char {
    unsafe {
        let idx = *idxp - 1;
        let tv = (tvs as *const TvForTypval).add(idx as usize);
        if (*tv).v_type == VAR_UNKNOWN {
            emsg(gettext(E_PRINTF.as_ptr().cast()));
            return std::ptr::null();
        }
        *idxp += 1;
        if (*tv).v_type == VAR_STRING || (*tv).v_type == VAR_NUMBER {
            *tofree = std::ptr::null_mut();
            tv_get_string_chk(tv as *mut TvForTypval as *mut c_void)
        } else {
            let s = encode_tv2echo(tv as *mut TvForTypval as *mut c_void, std::ptr::null_mut());
            *tofree = s;
            s as *const c_char
        }
    }
}

/// Get a pointer from tvs[*idxp - 1], incrementing *idxp.
unsafe fn tv_ptr_rs(tvs: *const c_void, idxp: *mut c_int) -> *const c_void {
    unsafe {
        let idx = *idxp - 1;
        let tv = (tvs as *const TvForTypval).add(idx as usize);
        if (*tv).v_type == VAR_UNKNOWN {
            emsg(gettext(E_PRINTF.as_ptr().cast()));
            return std::ptr::null();
        }
        *idxp += 1;
        // vval is the union; for pointer types v_string is at the same offset
        // as v_list, v_dict etc. -- all are pointer-sized at offset 0 of vval.
        // We read it as a raw pointer via the vval u64 field.
        (*tv).vval as usize as *const c_void
    }
}

/// Get a float from tvs[*idxp - 1], incrementing *idxp. Returns 0.0 on error.
unsafe fn tv_float_rs(tvs: *const c_void, idxp: *mut c_int) -> f64 {
    unsafe {
        let idx = *idxp - 1;
        let tv = (tvs as *const TvForTypval).add(idx as usize);
        if (*tv).v_type == VAR_UNKNOWN {
            emsg(gettext(E_PRINTF.as_ptr().cast()));
            return 0.0;
        }
        *idxp += 1;
        if (*tv).v_type == VAR_FLOAT {
            // vval is a u64; float_T is double (f64) at same offset
            f64::from_bits((*tv).vval)
        } else if (*tv).v_type == VAR_NUMBER {
            (*tv).vval as i64 as f64
        } else {
            emsg(gettext(E_FLOAT_ARG.as_ptr().cast()));
            0.0
        }
    }
}

// =============================================================================
// rs_vim_vsnprintf_extracted: the full printf formatting engine
// =============================================================================

/// Write a formatted string using either pre-extracted va_list args or typval_T array.
///
/// This is the core printf engine, migrated from C `vim_vsnprintf_typval`.
///
/// # Arguments
/// - `str`: output buffer (may be NULL if str_m is 0)
/// - `str_m`: output buffer size
/// - `fmt`: format string (may be NULL, treated as "")
/// - `args`: pre-extracted argument array (used when tvs is NULL); may be NULL when tvs != NULL
/// - `num_args`: number of elements in args
/// - `ap_types`: positional argument type array from rs_parse_fmt_types
/// - `num_posarg`: number of positional arguments (0 = sequential)
/// - `tvs`: typval_T array for VimL printf() (NULL for C callers)
///
/// Returns number of characters that would have been written (excluding NUL).
///
/// # Safety
/// All pointers must be valid for their declared purpose.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_vsnprintf_extracted(
    str_: *mut c_char,
    str_m: usize,
    fmt: *const c_char,
    args: *const FmtArg,
    num_args: c_int,
    ap_types: *const *const c_char,
    num_posarg: c_int,
    tvs: *const c_void,
) -> c_int {
    let _ = num_args; // bounds checking not needed: ap_types provides count
    let _ = ap_types; // used only for extraction (already done in C); kept for API clarity

    unsafe {
        let mut str_l: usize = 0;
        let mut str_avail = str_l < str_m;

        let empty: &[u8] = b"\0";
        let p_start: *const u8 = if fmt.is_null() {
            empty.as_ptr()
        } else {
            fmt as *const u8
        };
        let mut p = p_start;

        // arg_idx: 1-based index for sequential (non-positional) args
        let mut arg_idx: c_int = 1;

        // Helper macro: write a byte to output
        macro_rules! write_byte {
            ($b:expr) => {
                if str_avail {
                    let avail = str_m - str_l;
                    if avail > 0 {
                        *str_.add(str_l) = $b as c_char;
                        str_avail = str_l + 1 < str_m;
                    }
                }
                str_l += 1;
            };
        }

        // Helper macro: write n bytes from src
        macro_rules! write_bytes {
            ($src:expr, $n:expr) => {{
                let n: usize = $n;
                if str_avail && n > 0 {
                    let avail = str_m - str_l;
                    let copy = n.min(avail);
                    std::ptr::copy_nonoverlapping($src as *const c_char, str_.add(str_l), copy);
                    str_avail = n < avail;
                }
                str_l += n;
            }};
        }

        // Helper: get next sequential FmtArg
        macro_rules! next_arg {
            () => {{
                let idx = (arg_idx - 1) as usize;
                arg_idx += 1;
                args.add(idx)
            }};
        }

        while *p != NUL {
            if *p != b'%' {
                // Copy literal bytes up to the next '%' or NUL
                let n = (xstrchrnul((p as *const c_char).add(1), b'%' as c_int) as *const u8)
                    .offset_from(p) as usize;
                write_bytes!(p, n);
                p = p.add(n);
            } else {
                let mut min_field_width: usize = 0;
                let mut precision: usize = 0;
                let mut zero_padding = false;
                let mut precision_specified = false;
                let mut justify_left = false;
                let mut alternate_form = false;
                let mut force_sign = false;
                let mut space_for_positive: c_int = 1;
                let mut length_modifier: u8 = NUL;

                // Temporary buffer for numeric conversions
                const TMP_LEN: usize = 350;
                let mut tmp = [0u8; TMP_LEN];
                let mut str_arg: *const c_char = std::ptr::null();
                let mut str_arg_l: usize = 0;
                let mut uchar_arg: u8 = 0;
                let mut number_of_zeros_to_pad: usize = 0;
                let mut zero_padding_insertion_ind: usize = 0;
                let mut fmt_spec: u8 = NUL;
                let mut tofree: *mut c_char = std::ptr::null_mut();
                let mut pos_arg: c_int = -1;

                p = p.add(1); // skip '%'

                // Check for positional argument specifier NNN$
                let mut ptype = p;
                while (*ptype).is_ascii_digit() {
                    ptype = ptype.add(1);
                }

                if *ptype == b'$' {
                    // Positional argument
                    let digstart = p as *const c_char;
                    let mut uj: c_uint = 0;
                    let mut pp = p as *const c_char;
                    if rs_get_unsigned_int(digstart, &mut pp, &mut uj, !tvs.is_null()) == FAIL {
                        // error already emitted
                        break;
                    }
                    p = pp as *const u8;
                    pos_arg = uj as c_int;
                    p = p.add(1); // skip '$'
                }

                // Parse flags
                loop {
                    match *p {
                        b'0' => {
                            zero_padding = true;
                            p = p.add(1);
                        }
                        b'-' => {
                            justify_left = true;
                            p = p.add(1);
                        }
                        b'+' => {
                            force_sign = true;
                            space_for_positive = 0;
                            p = p.add(1);
                        }
                        b' ' => {
                            force_sign = true;
                            p = p.add(1);
                        }
                        b'#' => {
                            alternate_form = true;
                            p = p.add(1);
                        }
                        b'\'' => {
                            p = p.add(1);
                        }
                        _ => break,
                    }
                }

                // Parse field width
                if *p == b'*' {
                    let digstart = (p as *const c_char).add(1);
                    p = p.add(1);

                    let fw_arg_idx = if (*p).is_ascii_digit() {
                        // Positional field width
                        let mut uj: c_uint = 0;
                        let mut pp = p as *const c_char;
                        if rs_get_unsigned_int(digstart, &mut pp, &mut uj, !tvs.is_null()) == FAIL {
                            break;
                        }
                        p = pp as *const u8;
                        // expect '$'
                        p = p.add(1); // skip '$'
                        let idx = uj as c_int;
                        idx
                    } else {
                        let idx = arg_idx;
                        arg_idx += 1;
                        idx
                    };

                    let j = if !tvs.is_null() {
                        let mut idx = fw_arg_idx;
                        tv_nr_rs(tvs, &mut idx) as i64
                    } else {
                        let a = args.add((fw_arg_idx - 1) as usize);
                        (*a).val.i as i64
                    };

                    if j > MAX_ALLOWED_STRING_WIDTH as i64 {
                        if !tvs.is_null() {
                            rs_format_overflow_error(digstart);
                            break;
                        }
                        min_field_width = MAX_ALLOWED_STRING_WIDTH as usize;
                    } else if j >= 0 {
                        min_field_width = j as usize;
                    } else {
                        min_field_width = (-j) as usize;
                        justify_left = true;
                    }
                } else if (*p).is_ascii_digit() {
                    let digstart = p as *const c_char;
                    let mut uj: c_uint = 0;
                    let mut pp = p as *const c_char;
                    if rs_get_unsigned_int(digstart, &mut pp, &mut uj, !tvs.is_null()) == FAIL {
                        break;
                    }
                    p = pp as *const u8;
                    min_field_width = uj as usize;
                }

                // Parse precision
                if *p == b'.' {
                    p = p.add(1);
                    precision_specified = true;

                    if (*p).is_ascii_digit() {
                        let digstart = p as *const c_char;
                        let mut uj: c_uint = 0;
                        let mut pp = p as *const c_char;
                        if rs_get_unsigned_int(digstart, &mut pp, &mut uj, !tvs.is_null()) == FAIL {
                            break;
                        }
                        p = pp as *const u8;
                        precision = uj as usize;
                    } else if *p == b'*' {
                        let digstart = p as *const c_char;
                        p = p.add(1);

                        let prec_arg_idx = if (*p).is_ascii_digit() {
                            let mut uj: c_uint = 0;
                            let mut pp = p as *const c_char;
                            if rs_get_unsigned_int(digstart, &mut pp, &mut uj, !tvs.is_null())
                                == FAIL
                            {
                                break;
                            }
                            p = pp as *const u8;
                            p = p.add(1); // skip '$'
                            uj as c_int
                        } else {
                            let idx = arg_idx;
                            arg_idx += 1;
                            idx
                        };

                        let j = if !tvs.is_null() {
                            let mut idx = prec_arg_idx;
                            tv_nr_rs(tvs, &mut idx) as i64
                        } else {
                            let a = args.add((prec_arg_idx - 1) as usize);
                            (*a).val.i as i64
                        };

                        if j > MAX_ALLOWED_STRING_WIDTH as i64 {
                            if !tvs.is_null() {
                                rs_format_overflow_error(digstart);
                                break;
                            }
                            precision = MAX_ALLOWED_STRING_WIDTH as usize;
                        } else if j >= 0 {
                            precision = j as usize;
                        } else {
                            precision_specified = false;
                            precision = 0;
                        }
                    }
                }

                // Parse length modifiers: h, l, ll, z
                if *p == b'h' || *p == b'l' || *p == b'z' {
                    length_modifier = *p;
                    p = p.add(1);
                    if length_modifier == b'l' && *p == b'l' {
                        length_modifier = b'L';
                        p = p.add(1);
                    }
                }

                fmt_spec = *p;

                // Common synonyms
                match fmt_spec {
                    b'i' => {
                        fmt_spec = b'd';
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

                // For tvs path, d/u/o/x/X with no length modifier: use 'L' (long long)
                if !tvs.is_null()
                    && length_modifier == NUL
                    && matches!(fmt_spec, b'd' | b'u' | b'o' | b'x' | b'X')
                {
                    length_modifier = b'L';
                }

                // Determine positional arg index
                if pos_arg != -1 {
                    arg_idx = pos_arg;
                }

                // Process conversion specifier
                match fmt_spec {
                    b'%' | b'c' | b's' | b'S' => {
                        str_arg_l = 1;
                        match fmt_spec {
                            b'%' => {
                                str_arg = p as *const c_char;
                            }
                            b'c' => {
                                let j = if !tvs.is_null() {
                                    tv_nr_rs(tvs, &mut arg_idx) as i32
                                } else {
                                    let a = next_arg!();
                                    (*a).val.i
                                };
                                uchar_arg = j as u8;
                                str_arg = (&uchar_arg as *const u8).cast::<c_char>();
                            }
                            b's' | b'S' => {
                                str_arg = if !tvs.is_null() {
                                    tv_str_rs(tvs, &mut arg_idx, &mut tofree)
                                } else {
                                    let a = next_arg!();
                                    (*a).val.s
                                };

                                if str_arg.is_null() {
                                    str_arg = b"[NULL]\0".as_ptr().cast::<c_char>();
                                    str_arg_l = 6;
                                } else if !precision_specified {
                                    str_arg_l = libc::strlen(str_arg);
                                } else if precision == 0 {
                                    str_arg_l = 0;
                                } else {
                                    str_arg_l = (xmemscan(
                                        str_arg.cast::<c_void>(),
                                        NUL as c_char,
                                        precision.min(0x7fff_ffff),
                                    )
                                        as *const c_char)
                                        .offset_from(str_arg)
                                        as usize;
                                }

                                if fmt_spec == b'S' {
                                    let mut i: usize = 0;
                                    let mut p1 = str_arg;
                                    while *p1 != 0 {
                                        let cell = utf_ptr2cells(p1) as usize;
                                        if precision_specified && i + cell > precision {
                                            break;
                                        }
                                        i += cell;
                                        p1 = p1.add(utfc_ptr2len(p1) as usize);
                                    }
                                    str_arg_l = p1.offset_from(str_arg) as usize;
                                    if min_field_width != 0 {
                                        min_field_width += str_arg_l - i;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    b'd' | b'u' | b'b' | b'B' | b'o' | b'x' | b'X' | b'p' => {
                        let mut arg_sign: c_int = 0;
                        let mut arg: i128 = 0;
                        let mut uarg: u128 = 0;
                        let mut ptr_arg: *const c_void = std::ptr::null();

                        if fmt_spec == b'p' {
                            ptr_arg = if !tvs.is_null() {
                                tv_ptr_rs(tvs, &mut arg_idx)
                            } else {
                                let a = next_arg!();
                                (*a).val.p
                            };
                            if !ptr_arg.is_null() {
                                arg_sign = 1;
                            }
                        } else if fmt_spec == b'b' || fmt_spec == b'B' {
                            uarg = if !tvs.is_null() {
                                tv_nr_rs(tvs, &mut arg_idx) as u128
                            } else {
                                let a = next_arg!();
                                (*a).val.ull as u128
                            };
                            arg_sign = if uarg != 0 { 1 } else { 0 };
                        } else if fmt_spec == b'd' {
                            arg = match length_modifier {
                                NUL => {
                                    if !tvs.is_null() {
                                        tv_nr_rs(tvs, &mut arg_idx) as i32 as i128
                                    } else {
                                        let a = next_arg!();
                                        (*a).val.i as i128
                                    }
                                }
                                b'h' => {
                                    if !tvs.is_null() {
                                        tv_nr_rs(tvs, &mut arg_idx) as i16 as i128
                                    } else {
                                        let a = next_arg!();
                                        (*a).val.i as i16 as i128
                                    }
                                }
                                b'l' => {
                                    if !tvs.is_null() {
                                        tv_nr_rs(tvs, &mut arg_idx) as libc::c_long as i128
                                    } else {
                                        let a = next_arg!();
                                        (*a).val.l as i128
                                    }
                                }
                                b'L' => {
                                    if !tvs.is_null() {
                                        tv_nr_rs(tvs, &mut arg_idx) as i128
                                    } else {
                                        let a = next_arg!();
                                        (*a).val.ll as i128
                                    }
                                }
                                b'z' => {
                                    if !tvs.is_null() {
                                        tv_nr_rs(tvs, &mut arg_idx) as isize as i128
                                    } else {
                                        let a = next_arg!();
                                        (*a).val.z as i128
                                    }
                                }
                                _ => 0,
                            };
                            arg_sign = if arg > 0 {
                                1
                            } else if arg < 0 {
                                -1
                            } else {
                                0
                            };
                        } else {
                            // unsigned
                            uarg = match length_modifier {
                                NUL => {
                                    if !tvs.is_null() {
                                        tv_nr_rs(tvs, &mut arg_idx) as u32 as u128
                                    } else {
                                        let a = next_arg!();
                                        (*a).val.u as u128
                                    }
                                }
                                b'h' => {
                                    if !tvs.is_null() {
                                        tv_nr_rs(tvs, &mut arg_idx) as u16 as u128
                                    } else {
                                        let a = next_arg!();
                                        (*a).val.u as u16 as u128
                                    }
                                }
                                b'l' => {
                                    if !tvs.is_null() {
                                        tv_nr_rs(tvs, &mut arg_idx) as libc::c_ulong as u128
                                    } else {
                                        let a = next_arg!();
                                        (*a).val.ul as u128
                                    }
                                }
                                b'L' => {
                                    if !tvs.is_null() {
                                        tv_nr_rs(tvs, &mut arg_idx) as u64 as u128
                                    } else {
                                        let a = next_arg!();
                                        (*a).val.ull as u128
                                    }
                                }
                                b'z' => {
                                    if !tvs.is_null() {
                                        tv_nr_rs(tvs, &mut arg_idx) as usize as u128
                                    } else {
                                        let a = next_arg!();
                                        (*a).val.uz as u128
                                    }
                                }
                                _ => 0,
                            };
                            arg_sign = if uarg != 0 { 1 } else { 0 };
                        }

                        // Build numeric string in tmp
                        str_arg = tmp.as_ptr().cast::<c_char>();
                        str_arg_l = 0;

                        // For d/i/u/o/x/X: precision overrides zero_padding
                        if precision_specified {
                            zero_padding = false;
                        }

                        if fmt_spec == b'd' {
                            if force_sign && arg_sign >= 0 {
                                tmp[str_arg_l] = if space_for_positive == 1 { b' ' } else { b'+' };
                                str_arg_l += 1;
                            }
                        } else if alternate_form {
                            if arg_sign != 0 && matches!(fmt_spec, b'x' | b'X' | b'b' | b'B') {
                                tmp[str_arg_l] = b'0';
                                str_arg_l += 1;
                                tmp[str_arg_l] = fmt_spec;
                                str_arg_l += 1;
                            }
                        }

                        zero_padding_insertion_ind = str_arg_l;
                        if !precision_specified {
                            precision = 1;
                        }

                        if precision == 0 && arg_sign == 0 {
                            // zero value with precision 0: empty string
                        } else {
                            match fmt_spec {
                                b'p' => {
                                    let written = libc::snprintf(
                                        tmp.as_mut_ptr().add(str_arg_l).cast::<c_char>(),
                                        TMP_LEN - str_arg_l,
                                        b"%p\0".as_ptr().cast::<c_char>(),
                                        ptr_arg,
                                    );
                                    str_arg_l += written as usize;
                                }
                                b'd' => {
                                    let written = libc::snprintf(
                                        tmp.as_mut_ptr().add(str_arg_l).cast::<c_char>(),
                                        TMP_LEN - str_arg_l,
                                        b"%jd\0".as_ptr().cast::<c_char>(),
                                        arg as libc::intmax_t,
                                    );
                                    str_arg_l += written as usize;
                                }
                                b'b' | b'B' => {
                                    // binary
                                    let mut bits: usize = 0;
                                    let mut b = uarg;
                                    for i in (1..=128usize).rev() {
                                        if (b >> (i - 1)) & 1 != 0 {
                                            bits = i;
                                            break;
                                        }
                                    }
                                    let mut i = bits;
                                    while i > 0 {
                                        i -= 1;
                                        tmp[str_arg_l] =
                                            if (uarg >> i) & 1 != 0 { b'1' } else { b'0' };
                                        str_arg_l += 1;
                                    }
                                }
                                _ => {
                                    // unsigned: o, u, x, X
                                    // Build format: %ju/%jo/%jx/%jX
                                    let fmt_ch = fmt_spec;
                                    let fmt_buf = [b'%', b'j', fmt_ch, b'\0'];
                                    let written = libc::snprintf(
                                        tmp.as_mut_ptr().add(str_arg_l).cast::<c_char>(),
                                        TMP_LEN - str_arg_l,
                                        fmt_buf.as_ptr().cast::<c_char>(),
                                        uarg as libc::uintmax_t,
                                    );
                                    str_arg_l += written as usize;
                                }
                            }

                            debug_assert!(str_arg_l < TMP_LEN);

                            // Move zero_padding_insertion_ind past sign and '0x' prefix
                            if zero_padding_insertion_ind < str_arg_l
                                && tmp[zero_padding_insertion_ind] == b'-'
                            {
                                zero_padding_insertion_ind += 1;
                            }
                            if zero_padding_insertion_ind + 1 < str_arg_l
                                && tmp[zero_padding_insertion_ind] == b'0'
                                && matches!(
                                    tmp[zero_padding_insertion_ind + 1],
                                    b'x' | b'X' | b'b' | b'B'
                                )
                            {
                                zero_padding_insertion_ind += 2;
                            }
                        }

                        {
                            let num_of_digits = str_arg_l - zero_padding_insertion_ind;
                            if alternate_form
                                && fmt_spec == b'o'
                                && !(zero_padding_insertion_ind < str_arg_l
                                    && tmp[zero_padding_insertion_ind] == b'0')
                            {
                                if !precision_specified || precision < num_of_digits + 1 {
                                    precision = num_of_digits + 1;
                                }
                            }
                            if num_of_digits < precision {
                                number_of_zeros_to_pad = precision - num_of_digits;
                            }
                        }

                        if !justify_left && zero_padding {
                            let n =
                                min_field_width.saturating_sub(str_arg_l + number_of_zeros_to_pad);
                            if n > 0 {
                                number_of_zeros_to_pad += n;
                            }
                        }
                    }

                    b'f' | b'F' | b'e' | b'E' | b'g' | b'G' => {
                        let mut format = [0u8; 40];
                        let mut remove_trailing_zeroes = false;

                        let mut f = if !tvs.is_null() {
                            tv_float_rs(tvs, &mut arg_idx)
                        } else {
                            let a = next_arg!();
                            (*a).val.f
                        };

                        let abs_f = if f < 0.0 { -f } else { f };

                        if fmt_spec == b'g' || fmt_spec == b'G' {
                            if (abs_f >= 0.001 && abs_f < 10_000_000.0) || abs_f == 0.0 {
                                fmt_spec = if fmt_spec.is_ascii_uppercase() {
                                    b'F'
                                } else {
                                    b'f'
                                };
                            } else {
                                fmt_spec = if fmt_spec == b'g' { b'e' } else { b'E' };
                            }
                            remove_trailing_zeroes = true;
                        }

                        if f.is_infinite() || (matches!(fmt_spec, b'f' | b'F') && abs_f > 1.0e307) {
                            let inf_str = rs_infinity_str(
                                f > 0.0,
                                fmt_spec as c_char,
                                if force_sign { 1 } else { 0 },
                                space_for_positive,
                            );
                            xstrlcpy(tmp.as_mut_ptr().cast::<c_char>(), inf_str, TMP_LEN);
                            str_arg_l = libc::strlen(tmp.as_ptr().cast::<c_char>());
                            zero_padding = false;
                        } else if f.is_nan() {
                            let nan_str: &[u8] = if fmt_spec.is_ascii_uppercase() {
                                b"NAN"
                            } else {
                                b"nan"
                            };
                            tmp[0] = nan_str[0];
                            tmp[1] = nan_str[1];
                            tmp[2] = nan_str[2];
                            str_arg_l = 3;
                            zero_padding = false;
                        } else {
                            // Regular float
                            format[0] = b'%';
                            let mut l: usize = 1;
                            if force_sign {
                                format[l] = if space_for_positive == 1 { b' ' } else { b'+' };
                                l += 1;
                            }
                            if precision_specified {
                                let max_prec =
                                    if (fmt_spec == b'f' || fmt_spec == b'F') && abs_f > 1.0 {
                                        (TMP_LEN - 10).saturating_sub(abs_f.log10() as usize)
                                    } else {
                                        TMP_LEN - 10
                                    };
                                let p_eff = precision.min(max_prec);
                                let written = libc::snprintf(
                                    format.as_mut_ptr().add(l).cast::<c_char>(),
                                    format.len() - l,
                                    b".%d\0".as_ptr().cast::<c_char>(),
                                    p_eff as c_int,
                                );
                                l += written as usize;
                            }

                            // 'F' -> 'f' for snprintf
                            format[l] = if fmt_spec == b'F' { b'f' } else { fmt_spec };
                            l += 1;
                            format[l] = NUL;

                            // Ensure f is passed correctly (handle -0.0 etc.)
                            if fmt_spec == b'F' {
                                // already converted above
                                f = f;
                            }

                            let written = libc::snprintf(
                                tmp.as_mut_ptr().cast::<c_char>(),
                                TMP_LEN,
                                format.as_ptr().cast::<c_char>(),
                                f,
                            );
                            str_arg_l = written as usize;
                            debug_assert!(str_arg_l < TMP_LEN);

                            if remove_trailing_zeroes {
                                // Using %g/%G: remove superfluous zeroes
                                let tp: *mut u8 = if fmt_spec == b'f' || fmt_spec == b'F' {
                                    // point tp at last char
                                    tmp.as_mut_ptr().add(str_arg_l - 1)
                                } else {
                                    // find 'e' or 'E'
                                    let e_ch = if fmt_spec == b'e' { b'e' } else { b'E' };
                                    let found = tmp[..str_arg_l]
                                        .iter()
                                        .position(|&b| b == e_ch)
                                        .map(|i| tmp.as_mut_ptr().add(i));

                                    if let Some(ep) = found {
                                        // remove superfluous '+' from exponent
                                        if *ep.add(1) == b'+' {
                                            // change "1.0e+07" to "1.0e07"
                                            let rest_len = str_arg_l
                                                - (ep.add(2).offset_from(tmp.as_ptr()) as usize);
                                            std::ptr::copy(ep.add(2), ep.add(1), rest_len + 1);
                                            str_arg_l -= 1;
                                        }

                                        // remove leading zeroes from exponent
                                        let i: usize = if *ep.add(1) == b'-' { 2 } else { 1 };
                                        let mut tp2 = ep.add(i);
                                        while *tp2 == b'0' {
                                            let rest_len = str_arg_l
                                                - (tp2.add(1).offset_from(tmp.as_ptr()) as usize);
                                            std::ptr::copy(tp2.add(1), tp2, rest_len + 1);
                                            str_arg_l -= 1;
                                        }

                                        // tp points to the char before 'e'
                                        ep.sub(1)
                                    } else {
                                        std::ptr::null_mut()
                                    }
                                };

                                if !tp.is_null() && !precision_specified {
                                    let mut tp2 = tp;
                                    while tp2 > tmp.as_mut_ptr().add(2)
                                        && *tp2 == b'0'
                                        && *tp2.sub(1) != b'.'
                                    {
                                        let pos = tp2.offset_from(tmp.as_ptr()) as usize;
                                        let rest_len = str_arg_l - pos - 1;
                                        std::ptr::copy(tp2.add(1), tp2, rest_len + 1);
                                        str_arg_l -= 1;
                                        tp2 = tp2.sub(1);
                                    }
                                }
                            } else {
                                // Be consistent: remove leading zero from exponent
                                let e_ch = if fmt_spec == b'e' { b'e' } else { b'E' };
                                if let Some(ep_idx) =
                                    tmp[..str_arg_l].iter().position(|&b| b == e_ch)
                                {
                                    let ep = tmp.as_mut_ptr().add(ep_idx);
                                    if ((*ep.add(1) == b'+') || (*ep.add(1) == b'-'))
                                        && *ep.add(2) == b'0'
                                        && ep_idx + 4 < str_arg_l
                                        && tmp[ep_idx + 3].is_ascii_digit()
                                        && tmp[ep_idx + 4].is_ascii_digit()
                                    {
                                        let rest_len = str_arg_l - (ep_idx + 3) - 1;
                                        std::ptr::copy(ep.add(3), ep.add(2), rest_len + 1);
                                        str_arg_l -= 1;
                                    }
                                }
                            }
                        }

                        if zero_padding
                            && min_field_width > str_arg_l
                            && (tmp[0] == b'-' || force_sign)
                        {
                            number_of_zeros_to_pad = min_field_width - str_arg_l;
                            zero_padding_insertion_ind = 1;
                        }
                        str_arg = tmp.as_ptr().cast::<c_char>();
                    }

                    _ => {
                        // Unrecognized conversion: keep format char as-is
                        zero_padding = false;
                        justify_left = true;
                        min_field_width = 0;
                        str_arg = p as *const c_char;
                        str_arg_l = 0;
                        if *p != NUL {
                            str_arg_l = 1;
                        }
                    }
                }

                if *p != NUL {
                    p = p.add(1); // step over conversion specifier
                }

                // Emit output: left padding, zero prefix, zeros, string, right padding

                // Left padding (spaces or zeros)
                if !justify_left {
                    let total = str_arg_l + number_of_zeros_to_pad;
                    if min_field_width > total {
                        let pn = min_field_width - total;
                        let pad_ch = if zero_padding { b'0' } else { b' ' };
                        for _ in 0..pn {
                            write_byte!(pad_ch);
                        }
                    }
                }

                // Zero padding: first emit sign or "0x" prefix
                if number_of_zeros_to_pad == 0 {
                    zero_padding_insertion_ind = 0;
                } else {
                    if zero_padding_insertion_ind > 0 {
                        write_bytes!(str_arg, zero_padding_insertion_ind);
                    }
                    for _ in 0..number_of_zeros_to_pad {
                        write_byte!(b'0');
                    }
                }

                // Emit the string (minus the already-emitted prefix)
                if str_arg_l > zero_padding_insertion_ind {
                    let sn = str_arg_l - zero_padding_insertion_ind;
                    write_bytes!(str_arg.add(zero_padding_insertion_ind), sn);
                }

                // Right padding
                if justify_left {
                    let total = str_arg_l + number_of_zeros_to_pad;
                    if min_field_width > total {
                        let pn = min_field_width - total;
                        for _ in 0..pn {
                            write_byte!(b' ');
                        }
                    }
                }

                xfree(tofree.cast::<c_void>());
            }
        }

        // NUL-terminate
        if str_m > 0 {
            let pos = str_l.min(str_m - 1);
            *str_.add(pos) = 0;
        }

        // Check for too many arguments (tvs path only)
        if !tvs.is_null() {
            let check_idx = if num_posarg != 0 {
                num_posarg
            } else {
                arg_idx - 1
            };
            let tv = (tvs as *const TvForTypval).add(check_idx as usize);
            if (*tv).v_type != VAR_UNKNOWN {
                emsg(gettext(
                    b"E767: Too many arguments to printf()\0".as_ptr().cast(),
                ));
            }
        }

        str_l as c_int
    }
}
