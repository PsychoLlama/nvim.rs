//! Type inspection functions for VimL.
//!
//! This module implements type functions from `src/nvim/eval/funcs.c`:
//! - `type()` - returns the type of a value
//! - `typename()` - returns the type name as a string
//! - `isnumber()` - check if value is a number
//! - `islist()` - check if value is a list
//! - `isdict()` - check if value is a dictionary
//! - `isfloat()` - check if value is a float
//! - `isstring()` - check if value is a string

use std::ffi::c_void;

use super::dispatch::{
    argvar_at, rettv_set_bool, rettv_set_number, tv_get_type, TypevalPtrMut, VarType,
};

// =============================================================================
// VAR_TYPE constants (from typval_defs.h)
// =============================================================================

/// VAR_TYPE_NUMBER = 0
const VAR_TYPE_NUMBER: i64 = 0;
/// VAR_TYPE_STRING = 1
const VAR_TYPE_STRING: i64 = 1;
/// VAR_TYPE_FUNC = 2
const VAR_TYPE_FUNC: i64 = 2;
/// VAR_TYPE_LIST = 3
const VAR_TYPE_LIST: i64 = 3;
/// VAR_TYPE_DICT = 4
const VAR_TYPE_DICT: i64 = 4;
/// VAR_TYPE_FLOAT = 5
const VAR_TYPE_FLOAT: i64 = 5;
/// VAR_TYPE_BOOL = 6
const VAR_TYPE_BOOL: i64 = 6;
/// VAR_TYPE_SPECIAL = 7
const VAR_TYPE_SPECIAL: i64 = 7;
/// VAR_TYPE_BLOB = 10
const VAR_TYPE_BLOB: i64 = 10;

// =============================================================================
// type() function
// =============================================================================

/// "type()" function - returns the type of a value
///
/// Returns:
/// - 0 for Number
/// - 1 for String
/// - 2 for Funcref
/// - 3 for List
/// - 4 for Dictionary
/// - 5 for Float
/// - 6 for Boolean
/// - 7 for Special (null)
/// - 10 for Blob
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_type"]
pub unsafe extern "C" fn rs_f_type(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);

    let vtype = tv_get_type(arg0);
    let n = match vtype {
        VarType::Number => VAR_TYPE_NUMBER,
        VarType::String => VAR_TYPE_STRING,
        VarType::Func | VarType::Partial => VAR_TYPE_FUNC,
        VarType::List => VAR_TYPE_LIST,
        VarType::Dict => VAR_TYPE_DICT,
        VarType::Float => VAR_TYPE_FLOAT,
        VarType::Bool => VAR_TYPE_BOOL,
        VarType::Special => VAR_TYPE_SPECIAL,
        VarType::Blob => VAR_TYPE_BLOB,
        VarType::Unknown => -1, // This should not happen in normal code
    };
    rettv_set_number(rettv, n);
}

// =============================================================================
// Type check functions (isXXX)
// =============================================================================

/// "isnumber()" function - check if value is a number.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[no_mangle]
pub unsafe extern "C" fn rs_f_isnumber(argvars: *const c_void, rettv: *mut c_void) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);
    let vtype = tv_get_type(arg0);
    rettv_set_bool(rettv, matches!(vtype, VarType::Number));
}

/// "isfloat()" function - check if value is a float.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[no_mangle]
pub unsafe extern "C" fn rs_f_isfloat(argvars: *const c_void, rettv: *mut c_void) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);
    let vtype = tv_get_type(arg0);
    rettv_set_bool(rettv, matches!(vtype, VarType::Float));
}

/// "isstring()" function - check if value is a string.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[no_mangle]
pub unsafe extern "C" fn rs_f_isstring(argvars: *const c_void, rettv: *mut c_void) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);
    let vtype = tv_get_type(arg0);
    rettv_set_bool(rettv, matches!(vtype, VarType::String));
}

/// "islist()" function - check if value is a list.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[no_mangle]
pub unsafe extern "C" fn rs_f_islist(argvars: *const c_void, rettv: *mut c_void) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);
    let vtype = tv_get_type(arg0);
    rettv_set_bool(rettv, matches!(vtype, VarType::List));
}

/// "isdict()" function - check if value is a dictionary.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[no_mangle]
pub unsafe extern "C" fn rs_f_isdict(argvars: *const c_void, rettv: *mut c_void) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);
    let vtype = tv_get_type(arg0);
    rettv_set_bool(rettv, matches!(vtype, VarType::Dict));
}

/// "isfunc()" function - check if value is a funcref.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[no_mangle]
pub unsafe extern "C" fn rs_f_isfunc(argvars: *const c_void, rettv: *mut c_void) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);
    let vtype = tv_get_type(arg0);
    rettv_set_bool(rettv, matches!(vtype, VarType::Func | VarType::Partial));
}

/// "isblob()" function - check if value is a blob.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[no_mangle]
pub unsafe extern "C" fn rs_f_isblob(argvars: *const c_void, rettv: *mut c_void) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);
    let vtype = tv_get_type(arg0);
    rettv_set_bool(rettv, matches!(vtype, VarType::Blob));
}

/// "isbool()" function - check if value is a boolean.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[no_mangle]
pub unsafe extern "C" fn rs_f_isbool(argvars: *const c_void, rettv: *mut c_void) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);
    let vtype = tv_get_type(arg0);
    rettv_set_bool(rettv, matches!(vtype, VarType::Bool));
}

// =============================================================================
// typename() function
// =============================================================================

/// Type name strings for VimL values.
///
/// These match the strings returned by typename() in VimL.
const TYPE_NAME_NUMBER: &[u8] = b"number";
const TYPE_NAME_STRING: &[u8] = b"string";
const TYPE_NAME_FUNC: &[u8] = b"func";
const TYPE_NAME_LIST: &[u8] = b"list<unknown>";
const TYPE_NAME_DICT: &[u8] = b"dict<unknown>";
const TYPE_NAME_FLOAT: &[u8] = b"float";
const TYPE_NAME_BOOL: &[u8] = b"bool";
const TYPE_NAME_SPECIAL: &[u8] = b"special";
const TYPE_NAME_BLOB: &[u8] = b"blob";
const TYPE_NAME_UNKNOWN: &[u8] = b"unknown";

/// Get the type name for a VarType.
#[must_use]
pub const fn type_name_for_vartype(vtype: VarType) -> &'static [u8] {
    match vtype {
        VarType::Number => TYPE_NAME_NUMBER,
        VarType::String => TYPE_NAME_STRING,
        VarType::Func | VarType::Partial => TYPE_NAME_FUNC,
        VarType::List => TYPE_NAME_LIST,
        VarType::Dict => TYPE_NAME_DICT,
        VarType::Float => TYPE_NAME_FLOAT,
        VarType::Bool => TYPE_NAME_BOOL,
        VarType::Special => TYPE_NAME_SPECIAL,
        VarType::Blob => TYPE_NAME_BLOB,
        VarType::Unknown => TYPE_NAME_UNKNOWN,
    }
}

/// FFI: Get type name length for a VarType code.
#[no_mangle]
pub extern "C" fn rs_f_typename_len(vtype: i32) -> usize {
    let vtype = VarType::from_c_int(vtype).unwrap_or(VarType::Unknown);
    type_name_for_vartype(vtype).len()
}

/// FFI: Get type name pointer for a VarType code.
#[no_mangle]
pub extern "C" fn rs_f_typename_ptr(vtype: i32) -> *const u8 {
    let vtype = VarType::from_c_int(vtype).unwrap_or(VarType::Unknown);
    type_name_for_vartype(vtype).as_ptr()
}

// =============================================================================
// Type conversion helpers
// =============================================================================

/// Convert a float to an integer (truncates toward zero).
///
/// VimL float2nr() behavior.
#[inline]
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub const fn float_to_nr(f: f64) -> i64 {
    f as i64
}

/// Convert an integer to a float.
///
/// VimL nr2float() behavior.
#[inline]
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub const fn nr_to_float(n: i64) -> f64 {
    n as f64
}

/// FFI: float to number conversion helper (pure function, not typval dispatch).
#[no_mangle]
pub extern "C" fn rs_float_to_nr(f: f64) -> i64 {
    float_to_nr(f)
}

/// FFI: number to float conversion helper (pure function, not typval dispatch).
#[no_mangle]
pub extern "C" fn rs_nr_to_float(n: i64) -> f64 {
    nr_to_float(n)
}

// =============================================================================
// String/List conversion helpers
// =============================================================================

/// Convert a string to a list of character codes (codepoints).
///
/// VimL str2list() behavior - each element is a Unicode codepoint.
#[must_use]
pub fn str_to_charlist(s: &[u8]) -> Vec<i64> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < s.len() {
        let (codepoint, char_len) = decode_utf8_char(&s[i..]);
        result.push(i64::from(codepoint));
        i += char_len;
    }

    result
}

/// Decode a single UTF-8 character.
/// Returns (codepoint, byte_length).
fn decode_utf8_char(s: &[u8]) -> (u32, usize) {
    if s.is_empty() {
        return (0, 0);
    }

    let b0 = s[0];

    // ASCII (0xxxxxxx)
    if b0 & 0x80 == 0 {
        return (u32::from(b0), 1);
    }

    // 2-byte (110xxxxx 10xxxxxx)
    if b0 & 0xE0 == 0xC0 && s.len() >= 2 && s[1] & 0xC0 == 0x80 {
        let cp = (u32::from(b0 & 0x1F) << 6) | u32::from(s[1] & 0x3F);
        return (cp, 2);
    }

    // 3-byte (1110xxxx 10xxxxxx 10xxxxxx)
    if b0 & 0xF0 == 0xE0 && s.len() >= 3 && s[1] & 0xC0 == 0x80 && s[2] & 0xC0 == 0x80 {
        let cp =
            (u32::from(b0 & 0x0F) << 12) | (u32::from(s[1] & 0x3F) << 6) | u32::from(s[2] & 0x3F);
        return (cp, 3);
    }

    // 4-byte (11110xxx 10xxxxxx 10xxxxxx 10xxxxxx)
    if b0 & 0xF8 == 0xF0
        && s.len() >= 4
        && s[1] & 0xC0 == 0x80
        && s[2] & 0xC0 == 0x80
        && s[3] & 0xC0 == 0x80
    {
        let cp = (u32::from(b0 & 0x07) << 18)
            | (u32::from(s[1] & 0x3F) << 12)
            | (u32::from(s[2] & 0x3F) << 6)
            | u32::from(s[3] & 0x3F);
        return (cp, 4);
    }

    // Invalid UTF-8, return byte value
    (u32::from(b0), 1)
}

/// Convert a list of character codes to a string.
///
/// VimL list2str() behavior - each element is a Unicode codepoint.
#[must_use]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
pub fn charlist_to_str(codes: &[i64]) -> Vec<u8> {
    let mut result = Vec::with_capacity(codes.len() * 4); // Max 4 bytes per char

    for &code in codes {
        if !(0..=0x0010_FFFF).contains(&code) {
            continue; // Skip invalid codes
        }
        let cp = code as u32;
        encode_utf8_char(cp, &mut result);
    }

    result
}

/// Encode a codepoint as UTF-8 and append to buffer.
#[allow(clippy::cast_possible_truncation)]
fn encode_utf8_char(cp: u32, buf: &mut Vec<u8>) {
    if cp < 0x80 {
        buf.push(cp as u8);
    } else if cp < 0x800 {
        buf.push(0xC0 | (cp >> 6) as u8);
        buf.push(0x80 | (cp & 0x3F) as u8);
    } else if cp < 0x10000 {
        buf.push(0xE0 | (cp >> 12) as u8);
        buf.push(0x80 | ((cp >> 6) & 0x3F) as u8);
        buf.push(0x80 | (cp & 0x3F) as u8);
    } else if cp < 0x0011_0000 {
        buf.push(0xF0 | (cp >> 18) as u8);
        buf.push(0x80 | ((cp >> 12) & 0x3F) as u8);
        buf.push(0x80 | ((cp >> 6) & 0x3F) as u8);
        buf.push(0x80 | (cp & 0x3F) as u8);
    }
    // Invalid codepoints (>= 0x110000) are silently skipped
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_type_constants() {
        // Verify constants match C definitions
        assert_eq!(VAR_TYPE_NUMBER, 0);
        assert_eq!(VAR_TYPE_STRING, 1);
        assert_eq!(VAR_TYPE_FUNC, 2);
        assert_eq!(VAR_TYPE_LIST, 3);
        assert_eq!(VAR_TYPE_DICT, 4);
        assert_eq!(VAR_TYPE_FLOAT, 5);
        assert_eq!(VAR_TYPE_BOOL, 6);
        assert_eq!(VAR_TYPE_SPECIAL, 7);
        assert_eq!(VAR_TYPE_BLOB, 10);
    }

    #[test]
    fn test_type_names() {
        assert_eq!(type_name_for_vartype(VarType::Number), b"number");
        assert_eq!(type_name_for_vartype(VarType::String), b"string");
        assert_eq!(type_name_for_vartype(VarType::List), b"list<unknown>");
        assert_eq!(type_name_for_vartype(VarType::Dict), b"dict<unknown>");
        assert_eq!(type_name_for_vartype(VarType::Float), b"float");
        assert_eq!(type_name_for_vartype(VarType::Bool), b"bool");
    }

    #[test]
    fn test_float_to_nr() {
        assert_eq!(float_to_nr(3.7), 3);
        assert_eq!(float_to_nr(-3.7), -3);
        assert_eq!(float_to_nr(0.0), 0);
        assert_eq!(float_to_nr(100.999), 100);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_nr_to_float() {
        assert_eq!(nr_to_float(42), 42.0);
        assert_eq!(nr_to_float(-17), -17.0);
        assert_eq!(nr_to_float(0), 0.0);
    }

    #[test]
    fn test_str_to_charlist_ascii() {
        let codes = str_to_charlist(b"abc");
        assert_eq!(codes, vec![97, 98, 99]);
    }

    #[test]
    fn test_str_to_charlist_utf8() {
        // "€" is U+20AC, encoded as E2 82 AC in UTF-8
        let codes = str_to_charlist(b"\xE2\x82\xAC");
        assert_eq!(codes, vec![0x20AC]);
    }

    #[test]
    fn test_str_to_charlist_mixed() {
        // "a€b" - ASCII + UTF-8 + ASCII
        let codes = str_to_charlist(b"a\xE2\x82\xACb");
        assert_eq!(codes, vec![97, 0x20AC, 98]);
    }

    #[test]
    fn test_charlist_to_str_ascii() {
        let s = charlist_to_str(&[97, 98, 99]);
        assert_eq!(s, b"abc");
    }

    #[test]
    fn test_charlist_to_str_utf8() {
        let s = charlist_to_str(&[0x20AC]); // Euro sign
        assert_eq!(s, b"\xE2\x82\xAC");
    }

    #[test]
    fn test_charlist_roundtrip() {
        let original = b"Hello \xE2\x82\xAC World";
        let codes = str_to_charlist(original);
        let reconstructed = charlist_to_str(&codes);
        assert_eq!(reconstructed, original);
    }

    #[test]
    fn test_decode_utf8_ascii() {
        assert_eq!(decode_utf8_char(b"a"), (97, 1));
        assert_eq!(decode_utf8_char(b"Z"), (90, 1));
    }

    #[test]
    fn test_decode_utf8_2byte() {
        // "¢" U+00A2 = C2 A2
        assert_eq!(decode_utf8_char(b"\xC2\xA2"), (0xA2, 2));
    }

    #[test]
    fn test_decode_utf8_3byte() {
        // "€" U+20AC = E2 82 AC
        assert_eq!(decode_utf8_char(b"\xE2\x82\xAC"), (0x20AC, 3));
    }

    #[test]
    fn test_decode_utf8_4byte() {
        // "😀" U+1F600 = F0 9F 98 80
        assert_eq!(decode_utf8_char(b"\xF0\x9F\x98\x80"), (0x1F600, 4));
    }
}
