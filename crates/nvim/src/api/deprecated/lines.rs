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
    unsafe { nvim_buf_set_lines(0 as uint64_t, buffer, lnum, lnum, true, lines, arena) }
}

pub unsafe fn buffer_get_line(
    buffer: Buffer,
    index: Integer,
    arena: *mut Arena,
) -> Result<String_0, Error> {
    unsafe {
        let mut rv: String_0 =
            String_0::from_raw_parts(::core::ptr::null_mut::<::core::ffi::c_char>(), 0 as size_t);
        let index = convert_index(index as int64_t) as Integer;
        let slice: Array = nvim_buf_get_lines(
            0 as uint64_t,
            buffer,
            index,
            index + 1 as Integer,
            true,
            arena,
            ::core::ptr::null_mut::<lua_State>(),
        )?;
        if slice.size != 0 {
            rv = (*slice.items.offset(0 as ::core::ffi::c_int as isize))
                .data
                .string;
        }
        Ok(rv)
    }
}

pub unsafe fn buffer_set_line(
    buffer: Buffer,
    index: Integer,
    line: String_0,
    arena: *mut Arena,
) -> Result<(), Error> {
    unsafe {
        let mut l: Object = object {
            type_0: kObjectTypeString,
            data: object_data { string: line },
        };
        let array: Array = Array {
            size: 1 as size_t,
            capacity: 0,
            items: &raw mut l,
        };
        let index = convert_index(index as int64_t) as Integer;
        nvim_buf_set_lines(
            0 as uint64_t,
            buffer,
            index,
            index + 1 as Integer,
            true,
            array,
            arena,
        )
    }
}

pub unsafe fn buffer_del_line(
    buffer: Buffer,
    index: Integer,
    arena: *mut Arena,
) -> Result<(), Error> {
    unsafe {
        let array: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let index = convert_index(index as int64_t) as Integer;
        nvim_buf_set_lines(
            0 as uint64_t,
            buffer,
            index,
            index + 1 as Integer,
            true,
            array,
            arena,
        )
    }
}

pub unsafe fn buffer_get_line_slice(
    buffer: Buffer,
    start: Integer,
    end: Integer,
    include_start: Boolean,
    include_end: Boolean,
    arena: *mut Arena,
) -> Result<Array, Error> {
    unsafe {
        let start = (convert_index(start as int64_t)
            + !include_start as ::core::ffi::c_int as int64_t) as Integer;
        let end = (convert_index(end as int64_t) + include_end as int64_t) as Integer;
        nvim_buf_get_lines(
            0 as uint64_t,
            buffer,
            start,
            end,
            false,
            arena,
            ::core::ptr::null_mut::<lua_State>(),
        )
    }
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
    unsafe {
        let start = (convert_index(start as int64_t)
            + !include_start as ::core::ffi::c_int as int64_t) as Integer;
        let end = (convert_index(end as int64_t) + include_end as int64_t) as Integer;
        nvim_buf_set_lines(0 as uint64_t, buffer, start, end, false, replacement, arena)
    }
}

fn convert_index(mut index: int64_t) -> int64_t {
    if index < 0 as int64_t {
        index - 1 as int64_t
    } else {
        index
    }
}
