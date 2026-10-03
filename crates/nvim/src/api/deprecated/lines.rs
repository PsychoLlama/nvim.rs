//! The `buffer_*` line accessors of the 0.1 API.
//!
//! Six functions over one-based, inclusive line numbers -- `convert_index` is
//! the translation into the modern zero-based, end-exclusive spelling -- each
//! forwarding to `nvim_buf_get_lines` or `nvim_buf_set_lines`.

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

/// Insert `lines` before line `lnum`.
pub fn buffer_insert(buffer: BufferHandle, lnum: Integer, lines: Array) -> Result<(), Error> {
    nvim_buf_set_lines(0, buffer, lnum, lnum, true, lines)
}

pub fn buffer_get_line(buffer: BufferHandle, index: Integer) -> Result<String_0, Error> {
    let index = convert_index(index as int64_t) as Integer;
    let no_lua = ::core::ptr::null_mut::<lua_State>();
    // SAFETY: a null `lua_State` asks for the API
    // representation rather than a Lua one.
    let slice: Array = unsafe { nvim_buf_get_lines(0, buffer, index, index + 1, true, no_lua) }?;
    if slice.is_empty() {
        return Ok(String_0::NULL);
    }
    let first = &slice[0];
    Ok(first.as_string().unwrap_or(&String_0::NULL).clone())
}

/// Replace line `index` (the 0.1 API's numbering) with `line`.
pub fn buffer_set_line(buffer: BufferHandle, index: Integer, line: String_0) -> Result<(), Error> {
    let mut array = Array::with_capacity(1);
    array.push(Object::string(line));
    let index = convert_index(index as int64_t) as Integer;
    nvim_buf_set_lines(0, buffer, index, index + 1, true, array)
}

/// Delete line `index` (the 0.1 API's numbering).
pub fn buffer_del_line(buffer: BufferHandle, index: Integer) -> Result<(), Error> {
    let index = convert_index(index as int64_t) as Integer;
    nvim_buf_set_lines(0, buffer, index, index + 1, true, Array::EMPTY)
}

pub fn buffer_get_line_slice(
    buffer: BufferHandle,
    start: Integer,
    end: Integer,
    include_start: Boolean,
    include_end: Boolean,
) -> Result<Array, Error> {
    let start = (convert_index(start as int64_t) + int64_t::from(!include_start)) as Integer;
    let end = (convert_index(end as int64_t) + int64_t::from(include_end)) as Integer;
    let no_lua = ::core::ptr::null_mut::<lua_State>();
    // SAFETY: as `buffer_get_line`.
    unsafe { nvim_buf_get_lines(0, buffer, start, end, false, no_lua) }
}

/// Replace a range of lines, the 0.1 API's way: each end may be in or out.
pub fn buffer_set_line_slice(
    buffer: BufferHandle,
    start: Integer,
    end: Integer,
    include_start: Boolean,
    include_end: Boolean,
    replacement: Array,
) -> Result<(), Error> {
    let start = (convert_index(start as int64_t) + int64_t::from(!include_start)) as Integer;
    let end = (convert_index(end as int64_t) + int64_t::from(include_end)) as Integer;
    nvim_buf_set_lines(0, buffer, start, end, false, replacement)
}

/// The 0.1 API's one-based, inclusive line number as the modern zero-based,
/// end-exclusive one. A negative index counts back from the end, and loses a
/// line in the translation because the two spellings disagree about where
/// the end is.
fn convert_index(index: int64_t) -> int64_t {
    if index < 0 { index - 1 } else { index }
}
