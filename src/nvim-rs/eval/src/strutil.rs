//! String and float utility functions migrated from eval.c.
//!
//! - `string2float`: Parse "inf"/"nan"/strtod
//! - `char_from_string`: Get character at index (negative indexing)
//! - `char_idx2byte`: Convert character index to byte offset (private helper)
//! - `string_slice`: Slice string by character indices

#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr,
    clippy::borrow_as_ptr
)]

use std::ffi::{c_char, c_int, c_void};

/// VARNUMBER_MAX is INT64_MAX (verified by _Static_assert in eval.c)
const VARNUMBER_MAX: i64 = i64::MAX;

extern "C" {
    fn rs_utfc_ptr2len(p: *const c_char) -> c_int;
    fn rs_utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
    fn xmemdupz(data: *const c_void, len: usize) -> *mut c_char;
}

/// Case-insensitive prefix check for ASCII bytes.
#[inline]
unsafe fn strnicmp_prefix(s: *const u8, prefix: &[u8]) -> bool {
    for (i, &p) in prefix.iter().enumerate() {
        let c = *s.add(i);
        if !c.eq_ignore_ascii_case(&p) {
            return false;
        }
    }
    true
}

/// Parse a string into a float value.
///
/// Handles "inf", "-inf", "nan" explicitly (for MS-Windows compatibility),
/// then falls through to `strtod`.
///
/// Returns the number of bytes consumed from `text`.
///
/// # Safety
///
/// `text` and `ret_value` must be valid, non-null pointers.
/// `text` must be a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_string2float(text: *const c_char, ret_value: *mut f64) -> usize {
    let t = text.cast::<u8>();

    // MS-Windows does not deal with "inf" and "nan" properly
    if strnicmp_prefix(t, b"inf") {
        *ret_value = f64::INFINITY;
        return 3;
    }
    if strnicmp_prefix(t, b"-inf") {
        *ret_value = f64::NEG_INFINITY;
        return 4;
    }
    if strnicmp_prefix(t, b"nan") {
        *ret_value = f64::NAN;
        return 3;
    }

    let mut end: *mut c_char = std::ptr::null_mut();
    *ret_value = libc::strtod(text, &raw mut end);
    end.offset_from(text) as usize
}

/// Get the byte index for character index `idx` in string `str` with length
/// `str_len`. Composing characters are included.
///
/// If going over the end, returns `str_len`.
/// If `idx` is negative, counts from the end (-1 is the last character).
/// When going over the start, returns -1.
///
/// # Safety
///
/// `str` must be a valid pointer to at least `str_len` bytes.
unsafe fn char_idx2byte(s: *const c_char, str_len: usize, idx: i64) -> isize {
    let mut nchar = idx;
    let mut nbyte: usize = 0;

    if nchar >= 0 {
        while nchar > 0 && nbyte < str_len {
            nbyte += rs_utfc_ptr2len(s.add(nbyte)) as usize;
            nchar -= 1;
        }
    } else {
        nbyte = str_len;
        while nchar < 0 && nbyte > 0 {
            nbyte -= 1;
            nbyte -= rs_utf_head_off(s, s.add(nbyte)) as usize;
            nchar += 1;
        }
        if nchar < 0 {
            return -1;
        }
    }
    nbyte as isize
}

/// Return the character `str[index]` where `index` is the character index,
/// including composing characters.
///
/// If `index` is out of range, returns NULL.
/// Negative index counts from the end.
///
/// # Safety
///
/// `str` may be null (returns null). If non-null, must be a valid
/// null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_char_from_string(s: *const c_char, index: i64) -> *mut c_char {
    if s.is_null() {
        return std::ptr::null_mut();
    }
    let slen = libc::strlen(s);

    let mut nchar = index;

    // A negative index counts from the end
    if index < 0 {
        let mut clen: i64 = 0;
        let mut nbyte: usize = 0;
        while nbyte < slen {
            nbyte += rs_utfc_ptr2len(s.add(nbyte)) as usize;
            clen += 1;
        }
        nchar = clen + index;
        if nchar < 0 {
            return std::ptr::null_mut();
        }
    }

    let mut nbyte: usize = 0;
    while nchar > 0 && nbyte < slen {
        nbyte += rs_utfc_ptr2len(s.add(nbyte)) as usize;
        nchar -= 1;
    }
    if nbyte >= slen {
        return std::ptr::null_mut();
    }
    xmemdupz(
        s.add(nbyte).cast::<c_void>(),
        rs_utfc_ptr2len(s.add(nbyte)) as usize,
    )
}

/// Return the slice `str[first : last]` using character indexes.
/// Composing characters are included.
///
/// `exclusive` is true for `slice()`.
///
/// Returns NULL when the result is empty.
///
/// # Safety
///
/// `str` may be null (returns null). If non-null, must be a valid
/// null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_string_slice(
    s: *const c_char,
    first: i64,
    last: i64,
    exclusive: bool,
) -> *mut c_char {
    if s.is_null() {
        return std::ptr::null_mut();
    }
    let slen = libc::strlen(s);
    let mut start_byte = char_idx2byte(s, slen, first);
    if start_byte < 0 {
        start_byte = 0; // first index very negative: use zero
    }

    let end_byte = if (last == -1 && !exclusive) || last == VARNUMBER_MAX {
        slen as isize
    } else {
        let mut eb = char_idx2byte(s, slen, last);
        if !exclusive && eb >= 0 && eb < slen as isize {
            // end index is inclusive
            eb += rs_utfc_ptr2len(s.add(eb as usize)) as isize;
        }
        eb
    };

    if start_byte >= slen as isize || end_byte <= start_byte {
        return std::ptr::null_mut();
    }
    xmemdupz(
        s.add(start_byte as usize).cast::<c_void>(),
        (end_byte - start_byte) as usize,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_string2float_inf() {
        let text = CString::new("inf").unwrap();
        let mut val: f64 = 0.0;
        let consumed = unsafe { rs_string2float(text.as_ptr(), &raw mut val) };
        assert_eq!(consumed, 3);
        assert!(val.is_infinite() && val > 0.0);
    }

    #[test]
    fn test_string2float_neg_inf() {
        let text = CString::new("-inf").unwrap();
        let mut val: f64 = 0.0;
        let consumed = unsafe { rs_string2float(text.as_ptr(), &raw mut val) };
        assert_eq!(consumed, 4);
        assert!(val.is_infinite() && val < 0.0);
    }

    #[test]
    fn test_string2float_nan() {
        let text = CString::new("nan").unwrap();
        let mut val: f64 = 0.0;
        let consumed = unsafe { rs_string2float(text.as_ptr(), &raw mut val) };
        assert_eq!(consumed, 3);
        assert!(val.is_nan());
    }

    #[test]
    fn test_string2float_number() {
        let text = CString::new("2.75").unwrap();
        let mut val: f64 = 0.0;
        let consumed = unsafe { rs_string2float(text.as_ptr(), &raw mut val) };
        assert_eq!(consumed, 4);
        assert!((val - 2.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_string2float_case_insensitive() {
        let text = CString::new("INF").unwrap();
        let mut val: f64 = 0.0;
        let consumed = unsafe { rs_string2float(text.as_ptr(), &raw mut val) };
        assert_eq!(consumed, 3);
        assert!(val.is_infinite() && val > 0.0);

        let text = CString::new("NaN").unwrap();
        let consumed = unsafe { rs_string2float(text.as_ptr(), &raw mut val) };
        assert_eq!(consumed, 3);
        assert!(val.is_nan());
    }

    #[test]
    fn test_string2float_zero() {
        let text = CString::new("0").unwrap();
        let mut val: f64 = 1.0;
        let consumed = unsafe { rs_string2float(text.as_ptr(), &raw mut val) };
        assert_eq!(consumed, 1);
        assert!((val - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_string2float_scientific() {
        let text = CString::new("1.5e2").unwrap();
        let mut val: f64 = 0.0;
        let consumed = unsafe { rs_string2float(text.as_ptr(), &raw mut val) };
        assert_eq!(consumed, 5);
        assert!((val - 150.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_varnumber_max_constant() {
        assert_eq!(VARNUMBER_MAX, i64::MAX);
    }
}
