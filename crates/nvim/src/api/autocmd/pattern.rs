//! Turning the `pattern` / `buffer` options into a pattern list.
//!
//! `get_patterns_from_pattern_or_buf` is the one place the two mutually
//! exclusive spellings are reconciled: a pattern string (or array of them,
//! each `<buffer>`-expanded and path-normalised) or a buffer number that
//! becomes a single `<buffer=N>` pattern.  `unpack_string_or_array` is the
//! "one or many" decoder it and the event list share.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api::private::validate::err_expected;
use core::ffi::CStr;

/// `v` as a list of strings: one string, or an array of them, under the
/// option name `k`.
pub(crate) fn unpack_string_or_array(
    v: Option<Object>,
    k: &CStr,
    required: bool,
) -> Result<Array, Error> {
    let Some(v) = v.filter(|v| !v.is_nil()) else {
        if required {
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
        let array = v.into_array().expect("the arm above matched an Array");
        check_string_array(&array, k, true)?;
        return Ok(array);
    }
    let got = api_typename(v.kind());
    Err(err_expected(k, c"Array or String", Some(got)))
}

/// The patterns the `pattern`/`buffer` options name, or `fallback` alone
/// when they name none (`None` for no fallback).
pub(crate) fn get_patterns_from_pattern_or_buf(
    pattern: Option<&Object>,
    has_buf: bool,
    buffer: BufferHandle,
    fallback: Option<&CStr>,
) -> Result<Array, Error> {
    /// One pattern string per `,`-separated span of `text`, which is what
    /// `aucmd_span_pattern` walks.
    fn push_spans(patterns: &mut Array, text: &String_0) {
        let text = text.as_bytes();
        let (mut start, mut len) = aucmd_span_pattern(text);
        while len != 0 {
            let end = start + len;
            patterns.push(Object::string(String_0::from_bytes(&text[start..end])));
            (start, len) = aucmd_span_pattern(&text[end..]);
            start += end;
        }
    }

    let mut patterns = Array::EMPTY;
    let pattern = pattern.filter(|pattern| !pattern.is_nil());
    if let Some(string) = pattern.and_then(Object::as_string) {
        push_spans(&mut patterns, string);
    } else if let Some(array) = pattern.and_then(Object::as_array) {
        check_string_array(array, c"pattern", true)?;
        for entry in array {
            let entry = entry
                .as_string()
                .expect("`check_string_array` accepted only Strings");
            push_spans(&mut patterns, entry);
        }
    } else if let Some(pattern) = pattern {
        let want = c"String or Table";
        let got = api_typename(pattern.kind());
        return Err(err_expected(c"pattern", want, Some(got)));
    } else if has_buf {
        let b = find_buffer_by_handle(buffer)?;
        let number = b.map_or(0, |b| b.handle);
        patterns.push(Object::string(String_0::from(
            format!("<buffer={number}>").as_str(),
        )));
    }
    if patterns.is_empty()
        && let Some(fallback) = fallback
    {
        patterns.push(Object::string(String_0::from_cstr(fallback)));
    }
    Ok(patterns)
}
