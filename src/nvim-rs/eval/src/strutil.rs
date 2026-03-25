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
    clippy::borrow_as_ptr,
    clashing_extern_declarations
)]

use std::ffi::{c_char, c_int, c_void};

use super::typval::TypvalT as TypvalTRepr;

/// VARNUMBER_MAX is INT64_MAX (verified by _Static_assert in eval.c)
const VARNUMBER_MAX: i64 = i64::MAX;

extern "C" {
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
    fn xmemdupz(data: *const c_void, len: usize) -> *mut c_char;
}

// =============================================================================
// Phase 4 (eval_shim pass 4): save_tv_as_string
// =============================================================================

extern "C" {
    fn nvim_tv_get_type(tv: *mut c_void) -> c_int;
    #[link_name = "tv_get_string_chk"]
    fn nvim_eval_tv_string_chk(tv: *mut c_void) -> *const c_char;
    #[link_name = "rs_buflist_findnr"]
    fn nvim_buflist_findnr(nr: c_int) -> *mut c_void; // buf_T*
    #[link_name = "nvim_buf_get_ml_line_count"]
    fn nvim_eval_buf_line_count(buf: *mut c_void) -> c_int;
    #[link_name = "ml_get_buf"]
    fn nvim_eval_ml_get_buf(buf: *mut c_void, lnum: i32) -> *const c_char;
    fn nvim_list_first_item(list: *mut c_void) -> *mut c_void; // listitem_T*
    fn nvim_list_item_next(list: *mut c_void, item: *mut c_void) -> *mut c_void; // listitem_T*
    fn nvim_list_item_get_string(item: *mut c_void) -> *const c_char;
    fn xmalloc(size: usize) -> *mut c_void;
    fn strlen(s: *const c_char) -> usize;
}

/// VAR_UNKNOWN type constant
const VAR_UNKNOWN_S: c_int = 0;
/// VAR_NUMBER type constant
const VAR_NUMBER_S: c_int = 1;
/// VAR_LIST type constant
const VAR_LIST_S: c_int = 4;

/// Saves a typval_T as a string.
///
/// For lists or buffers, replaces NLs with NUL and separates items with NLs.
///
/// # Safety
/// - `tv` must be a valid non-null typval pointer
/// - `len` must be a valid non-null pointer
///
/// # C equivalent
/// Replaces the C `save_tv_as_string` function in eval_shim.c.
#[export_name = "save_tv_as_string"]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_save_tv_as_string(
    tv: *mut c_void,
    len: *mut isize,
    endnl: bool,
    crlf: bool,
) -> *mut c_char {
    *len = 0;

    let vtype = nvim_tv_get_type(tv);

    if vtype == VAR_UNKNOWN_S {
        return std::ptr::null_mut();
    }

    // For non-list, non-number types: convert to string.
    if vtype != VAR_LIST_S && vtype != VAR_NUMBER_S {
        let ret_str = nvim_eval_tv_string_chk(tv);
        if ret_str.is_null() {
            *len = -1;
            return std::ptr::null_mut();
        }
        let slen = strlen(ret_str);
        *len = slen as isize;
        return xmemdupz(ret_str as *const c_void, slen);
    }

    // VAR_NUMBER: treat as buffer-id.
    if vtype == VAR_NUMBER_S {
        let bufnr = (*tv.cast::<TypvalTRepr>()).vval.v_number;
        let buf = nvim_buflist_findnr(bufnr as c_int);
        if buf.is_null() {
            crate::errors::semsg_e_nobufnr(bufnr);
            *len = -1;
            return std::ptr::null_mut();
        }

        // First pass: calculate length.
        let line_count = nvim_eval_buf_line_count(buf);
        for lnum in 1..=line_count {
            let p = nvim_eval_ml_get_buf(buf, lnum);
            let mut pp = p;
            while *pp != 0 {
                *len += 1;
                pp = pp.add(1);
            }
            *len += 1; // newline per line
        }

        if *len == 0 {
            return std::ptr::null_mut();
        }

        // Allocate and fill.
        let ret = xmalloc((*len) as usize + 1) as *mut c_char;
        let mut end = ret;
        for lnum in 1..=line_count {
            let p = nvim_eval_ml_get_buf(buf, lnum);
            let mut pp = p;
            while *pp != 0 {
                *end = if *pp == b'\n' as c_char { 0 } else { *pp };
                end = end.add(1);
                pp = pp.add(1);
            }
            *end = b'\n' as c_char;
            end = end.add(1);
        }
        *end = 0;
        *len = end.offset_from(ret) as isize;
        return ret;
    }

    // VAR_LIST: iterate items, replacing NL with NUL.
    let list = (*tv.cast::<TypvalTRepr>()).vval.v_list;

    // First pass: calculate total length.
    let mut item = nvim_list_first_item(list);
    while !item.is_null() {
        let s = nvim_list_item_get_string(item);
        *len += strlen(s) as isize + if crlf { 2 } else { 1 };
        item = nvim_list_item_next(list, item);
    }

    if *len == 0 {
        return std::ptr::null_mut();
    }

    let extra = if endnl {
        if crlf {
            2
        } else {
            1
        }
    } else {
        0
    };
    let ret = xmalloc((*len) as usize + extra) as *mut c_char;
    let mut end = ret;

    let mut item = nvim_list_first_item(list);
    while !item.is_null() {
        let next = nvim_list_item_next(list, item);
        let s = nvim_list_item_get_string(item);
        let mut p = s;
        while *p != 0 {
            *end = if *p == b'\n' as c_char { 0 } else { *p };
            end = end.add(1);
            p = p.add(1);
        }
        if endnl || !next.is_null() {
            if crlf {
                *end = b'\r' as c_char;
                end = end.add(1);
            }
            *end = b'\n' as c_char;
            end = end.add(1);
        }
        item = next;
    }
    *end = 0;
    *len = end.offset_from(ret) as isize;
    ret
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
            nbyte += utfc_ptr2len(s.add(nbyte)) as usize;
            nchar -= 1;
        }
    } else {
        nbyte = str_len;
        while nchar < 0 && nbyte > 0 {
            nbyte -= 1;
            nbyte -= utf_head_off(s, s.add(nbyte)) as usize;
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
            nbyte += utfc_ptr2len(s.add(nbyte)) as usize;
            clen += 1;
        }
        nchar = clen + index;
        if nchar < 0 {
            return std::ptr::null_mut();
        }
    }

    let mut nbyte: usize = 0;
    while nchar > 0 && nbyte < slen {
        nbyte += utfc_ptr2len(s.add(nbyte)) as usize;
        nchar -= 1;
    }
    if nbyte >= slen {
        return std::ptr::null_mut();
    }
    xmemdupz(
        s.add(nbyte).cast::<c_void>(),
        utfc_ptr2len(s.add(nbyte)) as usize,
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
            eb += utfc_ptr2len(s.add(eb as usize)) as isize;
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

// =============================================================================
// Phase 5 (eval_shim pass 4): typval_tostring
// =============================================================================

extern "C" {
    fn nvim_encode_tv2string_wrapper(tv: *mut c_void) -> *mut c_char;
    fn nvim_tv_get_vstring(tv: *mut c_void) -> *mut c_char;
    fn xstrdup(s: *const c_char) -> *mut c_char;
}

/// VAR_STRING type constant
const VAR_STRING_TS: c_int = 2;

/// Empty string sentinel for null vstring.
static EMPTY_STR: &[u8] = b"\0";

/// Convert any typval to a string representation, never giving an error.
///
/// When `quotes` is true, adds quotes around string values.
/// Returns an allocated string.
///
/// # Safety
/// - `arg` may be null (returns "(does not exist)").
/// - If non-null, must be a valid pointer to a typval_T.
#[export_name = "typval_tostring"]
pub unsafe extern "C" fn rs_typval_tostring(arg: *mut c_void, quotes: bool) -> *mut c_char {
    if arg.is_null() {
        let msg = b"(does not exist)\0";
        return xstrdup(msg.as_ptr() as *const c_char);
    }
    if !quotes && nvim_tv_get_type(arg) == VAR_STRING_TS {
        let s = nvim_tv_get_vstring(arg).cast_const();
        let s_nn = if s.is_null() {
            EMPTY_STR.as_ptr() as *const c_char
        } else {
            s
        };
        return xstrdup(s_nn);
    }
    nvim_encode_tv2string_wrapper(arg)
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
