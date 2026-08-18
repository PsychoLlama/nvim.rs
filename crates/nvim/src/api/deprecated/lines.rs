//! The `buffer_*` line accessors of the 0.1 API.
//!
//! Six functions over one-based, inclusive line numbers -- `convert_index` is
//! the translation into the modern zero-based, end-exclusive spelling -- each
//! forwarding to `nvim_buf_get_lines` or `nvim_buf_set_lines`.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;

pub unsafe extern "C" fn buffer_insert(
    mut buffer: Buffer,
    mut lnum: Integer,
    mut lines: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    unsafe {
        nvim_buf_set_lines(0 as uint64_t, buffer, lnum, lnum, true, lines, arena, err);
    }
}

pub unsafe extern "C" fn buffer_get_line(
    mut buffer: Buffer,
    mut index: Integer,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> String_0 {
    unsafe {
        let mut rv: String_0 = String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0 as size_t,
        };
        index = convert_index(index as int64_t) as Integer;
        let mut slice: Array = nvim_buf_get_lines(
            0 as uint64_t,
            buffer,
            index,
            index + 1 as Integer,
            true,
            arena,
            ::core::ptr::null_mut::<lua_State>(),
            err,
        );
        if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int)
            && slice.size != 0
        {
            rv = (*slice.items.offset(0 as ::core::ffi::c_int as isize))
                .data
                .string;
        }
        return rv;
    }
}

pub unsafe extern "C" fn buffer_set_line(
    mut buffer: Buffer,
    mut index: Integer,
    mut line: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    unsafe {
        let mut l: Object = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed { string: line },
        };
        let mut array: Array = Array {
            size: 1 as size_t,
            capacity: 0,
            items: &raw mut l,
        };
        index = convert_index(index as int64_t) as Integer;
        nvim_buf_set_lines(
            0 as uint64_t,
            buffer,
            index,
            index + 1 as Integer,
            true,
            array,
            arena,
            err,
        );
    }
}

pub unsafe extern "C" fn buffer_del_line(
    mut buffer: Buffer,
    mut index: Integer,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    unsafe {
        let mut array: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        index = convert_index(index as int64_t) as Integer;
        nvim_buf_set_lines(
            0 as uint64_t,
            buffer,
            index,
            index + 1 as Integer,
            true,
            array,
            arena,
            err,
        );
    }
}

pub unsafe extern "C" fn buffer_get_line_slice(
    mut buffer: Buffer,
    mut start: Integer,
    mut end: Integer,
    mut include_start: Boolean,
    mut include_end: Boolean,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    unsafe {
        start = (convert_index(start as int64_t) + !include_start as ::core::ffi::c_int as int64_t)
            as Integer;
        end = (convert_index(end as int64_t) + include_end as int64_t) as Integer;
        return nvim_buf_get_lines(
            0 as uint64_t,
            buffer,
            start,
            end,
            false,
            arena,
            ::core::ptr::null_mut::<lua_State>(),
            err,
        );
    }
}

pub unsafe extern "C" fn buffer_set_line_slice(
    mut buffer: Buffer,
    mut start: Integer,
    mut end: Integer,
    mut include_start: Boolean,
    mut include_end: Boolean,
    mut replacement: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    unsafe {
        start = (convert_index(start as int64_t) + !include_start as ::core::ffi::c_int as int64_t)
            as Integer;
        end = (convert_index(end as int64_t) + include_end as int64_t) as Integer;
        nvim_buf_set_lines(
            0 as uint64_t,
            buffer,
            start,
            end,
            false,
            replacement,
            arena,
            err,
        );
    }
}

fn convert_index(mut index: int64_t) -> int64_t {
    return if index < 0 as int64_t {
        index - 1 as int64_t
    } else {
        index
    };
}
