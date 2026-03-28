//! Printf helper functions: type classification and formatting utilities.
//!
//! Implements the format type enum, `format_typeof`, `format_typename`, and
//! `infinity_str` previously defined in strings.c.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]

use std::ffi::{c_char, c_int};

// =============================================================================
// C FFI
// =============================================================================

extern "C" {
    fn gettext(msgid: *const c_char) -> *const c_char;
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
