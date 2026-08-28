//! The `buffer_*` line accessors of the 0.1 API.
//!
//! Six functions over one-based, inclusive line numbers -- `convert_index` is
//! the translation into the modern zero-based, end-exclusive spelling -- each
//! forwarding to `nvim_buf_get_lines` or `nvim_buf_set_lines`.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;

pub unsafe fn buffer_insert(
    buffer: Buffer,
    lnum: Integer,
    lines: Array,
    arena: *mut Arena,
) -> Result<(), Error> {
    // SAFETY: `lines` and `arena` are the caller's, live for the call.
    unsafe { nvim_buf_set_lines(0, buffer, lnum, lnum, true, lines, arena) }
}

pub unsafe fn buffer_get_line(
    buffer: Buffer,
    index: Integer,
    arena: *mut Arena,
) -> Result<String_0, Error> {
    let index = convert_index(index as int64_t) as Integer;
    let no_lua = ::core::ptr::null_mut::<lua_State>();
    // SAFETY: `arena` is the caller's; a null `lua_State` asks for the API
    // representation rather than a Lua one.
    let slice: Array =
        unsafe { nvim_buf_get_lines(0, buffer, index, index + 1, true, arena, no_lua) }?;
    if slice.size == 0 {
        return Ok(String_0::NULL);
    }
    // SAFETY: the array has an item, which the call above filled in.
    let first = unsafe { *slice.items };
    Ok(first.as_string().unwrap_or(String_0::NULL))
}

pub unsafe fn buffer_set_line(
    buffer: Buffer,
    index: Integer,
    line: String_0,
    arena: *mut Arena,
) -> Result<(), Error> {
    let mut l = Object::string(line);
    let array: Array = Array {
        size: 1 as size_t,
        capacity: 0,
        items: &raw mut l,
    };
    let index = convert_index(index as int64_t) as Integer;
    // SAFETY: `array` borrows this frame's object for the length of the
    // call, and `arena` is the caller's.
    unsafe { nvim_buf_set_lines(0, buffer, index, index + 1, true, array, arena) }
}

pub unsafe fn buffer_del_line(
    buffer: Buffer,
    index: Integer,
    arena: *mut Arena,
) -> Result<(), Error> {
    let index = convert_index(index as int64_t) as Integer;
    // SAFETY: an empty replacement borrows nothing; `arena` is the caller's.
    unsafe { nvim_buf_set_lines(0, buffer, index, index + 1, true, Array::EMPTY, arena) }
}

pub unsafe fn buffer_get_line_slice(
    buffer: Buffer,
    start: Integer,
    end: Integer,
    include_start: Boolean,
    include_end: Boolean,
    arena: *mut Arena,
) -> Result<Array, Error> {
    let start = (convert_index(start as int64_t) + !include_start as int64_t) as Integer;
    let end = (convert_index(end as int64_t) + include_end as int64_t) as Integer;
    let no_lua = ::core::ptr::null_mut::<lua_State>();
    // SAFETY: as `buffer_get_line`.
    unsafe { nvim_buf_get_lines(0, buffer, start, end, false, arena, no_lua) }
}

pub unsafe fn buffer_set_line_slice(
    buffer: Buffer,
    start: Integer,
    end: Integer,
    include_start: Boolean,
    include_end: Boolean,
    replacement: Array,
    arena: *mut Arena,
) -> Result<(), Error> {
    let start = (convert_index(start as int64_t) + !include_start as int64_t) as Integer;
    let end = (convert_index(end as int64_t) + include_end as int64_t) as Integer;
    // SAFETY: `replacement` and `arena` are the caller's.
    unsafe { nvim_buf_set_lines(0, buffer, start, end, false, replacement, arena) }
}

/// The 0.1 API's one-based, inclusive line number as the modern zero-based,
/// end-exclusive one. A negative index counts back from the end, and loses a
/// line in the translation because the two spellings disagree about where
/// the end is.
fn convert_index(index: int64_t) -> int64_t {
    if index < 0 { index - 1 } else { index }
}
