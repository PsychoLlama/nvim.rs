//! Turning the `pattern` / `buffer` options into a pattern list.
//!
//! `get_patterns_from_pattern_or_buf` is the one place the two mutually
//! exclusive spellings are reconciled: a pattern string (or array of them,
//! each `<buffer>`-expanded and path-normalised) or a buffer number that
//! becomes a single `<buffer=N>` pattern.  `unpack_string_or_array` is the
//! "one or many" decoder it and the event list share.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api::private::validate::err_expected;

/// # Safety
///
/// `k` must point at a NUL-terminated string, unaliased for the call.
pub(crate) unsafe fn unpack_string_or_array(
    v: Option<Object>,
    k: *mut ::core::ffi::c_char,
    required: bool,
) -> Result<Array, Error> {
    let Some(v) = v.filter(|v| !v.is_nil()) else {
        if required {
            // SAFETY: `k` is a NUL-terminated key.
            let k = unsafe { core::ffi::CStr::from_ptr(k) };
            return Err(err_expected(k, c"Array or String", Some(c"nil")));
        }
        return Ok(Array::EMPTY);
    };
    if matches!(v, Object::String(_)) {
        let mut arr: Array = Array::with_capacity(1);
        arr.push(v);
        return Ok(arr);
    }
    if matches!(v, Object::Array(_)) {
        // SAFETY: `k` is a NUL-terminated key.
        let key = unsafe { core::ffi::CStr::from_ptr(k) };
        let array = v.into_array().expect("the arm above matched an Array");
        check_string_array(&array, key, true)?;
        return Ok(array);
    }
    let got = api_typename(v.kind());
    // SAFETY: `k` is a NUL-terminated key.
    let k = unsafe { core::ffi::CStr::from_ptr(k) };
    Err(err_expected(k, c"Array or String", Some(got)))
}

/// # Safety
///
/// `pattern` must be a well-formed API object the caller owns for the call.
/// `fallback` must point at a NUL-terminated string, unaliased for the call.
/// `arena` must point at a live arena, which the memory this answers with is
/// taken from and must outlive.
pub(crate) unsafe fn get_patterns_from_pattern_or_buf(
    pattern: Option<&Object>,
    has_buf: bool,
    buffer: BufferHandle,
    fallback: *mut ::core::ffi::c_char,
) -> Result<Array, Error> {
    /// One pattern string per `,`-separated span of `text`, which is what
    /// `aucmd_span_pattern` walks.
    ///
    /// # Safety
    /// `text` must name its own NUL-terminated bytes.
    unsafe fn push_spans(patterns: &mut Array, text: &String_0) {
        let mut pat: *const ::core::ffi::c_char = text.data();
        // SAFETY: the caller's promise.
        let mut patlen: size_t = unsafe { aucmd_span_pattern(pat, &raw mut pat) };
        while patlen != 0 {
            // SAFETY: the span is `patlen` bytes of `text`.
            let span = unsafe { core::slice::from_raw_parts(pat.cast::<u8>(), patlen) };
            patterns.push(Object::string(String_0::from_bytes(span)));
            // SAFETY: as above.
            patlen = unsafe { aucmd_span_pattern(pat.add(patlen), &raw mut pat) };
        }
    }

    let mut patterns = Array::EMPTY;
    let pattern = pattern.filter(|pattern| !pattern.is_nil());
    if let Some(string) = pattern.and_then(Object::as_string) {
        // SAFETY: a keyset string names its own NUL-terminated bytes.
        unsafe { push_spans(&mut patterns, string) };
    } else if let Some(array) = pattern.and_then(Object::as_array) {
        check_string_array(array, c"pattern", true)?;
        for entry in array {
            let entry = entry
                .as_string()
                .expect("`check_string_array` accepted only Strings");
            // SAFETY: as above.
            unsafe { push_spans(&mut patterns, entry) };
        }
    } else if let Some(pattern) = pattern {
        let want = c"String or Table";
        let got = api_typename(pattern.kind());
        return Err(err_expected(c"pattern", want, Some(got)));
    } else if has_buf {
        let b = find_buffer_by_handle(buffer)?;
        // SAFETY: the verb matches the argument.
        patterns.push(Object::string(unsafe {
            printf_string(c"<buffer=%d>".as_ptr(), b.map_or(0, |b| b.handle))
        }));
    }
    if patterns.is_empty() && !fallback.is_null() {
        // SAFETY: the caller's NUL-terminated fallback pattern.
        patterns.push(Object::string(unsafe { cstr_to_string(fallback) }));
    }
    Ok(patterns)
}
