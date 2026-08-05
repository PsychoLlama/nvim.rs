//! Strings and buffer text.
//!
//! An API `String` is a pointer and a length, and may or may not own its
//! bytes: `*_to_string` copies, `*_as_string` borrows. Getting that wrong
//! is a leak or a double free, so the name of every function here says
//! which it is.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{CAR, NL, NUL, STRING_INIT, api_set_error, arena_string, arena_take_arraybuilder};
use crate::src::nvim::api::private::validate::api_err_invalid;
use crate::src::nvim::kvec::InitVec;
use crate::src::nvim::memline::{ml_get_buf, ml_get_buf_len};
use crate::src::nvim::memory::{memchrsub, xmemdupz, xstrndup};
use crate::src::nvim::os::libc::{strlen, strnlen};
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::types::{
    Arena, Array, ArrayBuilder, Error, String_0, buf_T, garray_T, int64_t, kErrorTypeValidation,
    kObjectTypeString, linenr_T, object, object_data, size_t,
};
use core::ffi::c_char;
use core::{mem, ptr};

// -- Strings ---------------------------------------------------------------

/// A copy of the C string `str`, owned by the caller.
pub(crate) unsafe fn cstr_to_string(str: *const c_char) -> String_0 {
    // SAFETY: `str` is null or NUL-terminated.
    unsafe {
        if str.is_null() {
            return STRING_INIT;
        }
        cbuf_to_string(str, strlen(str))
    }
}

/// A copy of `size` bytes of `buf`, owned by the caller and NUL-terminated
/// however many NULs the bytes themselves hold.
pub(crate) unsafe fn cbuf_to_string(buf: *const c_char, size: size_t) -> String_0 {
    // SAFETY: `buf` has `size` readable bytes.
    unsafe {
        String_0 {
            data: xmemdupz(buf.cast(), size).cast(),
            size,
        }
    }
}

/// A NUL-terminated copy of `str`'s bytes, owned by the caller.
pub(crate) unsafe fn string_to_cstr(str: String_0) -> *mut c_char {
    // SAFETY: `str` has `size` readable bytes.
    unsafe { xstrndup(str.data, str.size) }
}

/// `str` viewed as an API string, borrowing rather than copying.
pub(crate) unsafe fn cstr_as_string(str: *const c_char) -> String_0 {
    // SAFETY: `str` is null or NUL-terminated.
    unsafe {
        if str.is_null() {
            return STRING_INIT;
        }
        String_0 {
            data: str as *mut c_char,
            size: strlen(str),
        }
    }
}

/// [`cstr_as_string`] for a buffer that need not be NUL-terminated within
/// `maxsize` bytes.
pub(crate) unsafe fn cstrn_as_string(str: *mut c_char, maxsize: size_t) -> String_0 {
    // SAFETY: `str` has `maxsize` readable bytes.
    unsafe {
        String_0 {
            data: str,
            size: strnlen(str, maxsize),
        }
    }
}

/// Take `ga`'s buffer as an API string, leaving the growarray empty.
pub(crate) unsafe fn ga_take_string(ga: *mut garray_T) -> String_0 {
    // SAFETY: `ga` is the caller's growarray of bytes.
    unsafe {
        let str = String_0 {
            data: (*ga).ga_data.cast(),
            size: (*ga).ga_len as size_t,
        };
        (*ga).ga_data = ptr::null_mut();
        (*ga).ga_len = 0;
        (*ga).ga_maxlen = 0;
        str
    }
}

/// Split `input` into one array item per line, arena-allocating the lines.
///
/// Line breaks are `\n`, or `\r` and `\r\n` as well with `crlf`. A NUL in
/// the text stands for a newline, as it does everywhere a buffer line is
/// passed as a C string, and is turned back into one. Text that ends *with*
/// a break gets a trailing empty item, so that the array round-trips.
pub(crate) unsafe fn string_to_array(input: String_0, crlf: bool, arena: *mut Arena) -> Array {
    // SAFETY: `input` has `size` readable bytes.
    unsafe {
        let mut ret: ArrayBuilder = mem::zeroed();
        let mut items = InitVec::new(
            &mut ret.size,
            &mut ret.capacity,
            &mut ret.items,
            &mut ret.init_array,
        );
        items.init();

        let mut i: size_t = 0;
        while i < input.size {
            let start = input.data.add(i);
            let mut end = start;
            let mut line_len: size_t = 0;
            while line_len < input.size - i {
                end = start.add(line_len);
                if *end == NL || (crlf && *end == CAR) {
                    break;
                }
                line_len += 1;
            }
            i += line_len;
            let ends_line = *end == NL || (crlf && *end == CAR);
            if crlf && *end == CAR && i + 1 < input.size && *end.add(1) == NL {
                i += 1;
            }

            let s = arena_string(
                arena,
                String_0 {
                    data: start,
                    size: line_len,
                },
            );
            memchrsub(s.data.cast(), NUL, NL, line_len);
            items.push(object {
                type_0: kObjectTypeString,
                data: object_data { string: s },
            });
            if i + 1 == input.size && ends_line {
                items.push(object {
                    type_0: kObjectTypeString,
                    data: object_data {
                        string: STRING_INIT,
                    },
                });
            }
            i += 1;
        }
        arena_take_arraybuilder(arena, &raw mut ret)
    }
}

// -- Buffer text -----------------------------------------------------------

/// Turn a signed, end-relative line index into a 1-based line number,
/// clamping it into the buffer and reporting through `oob` that it had to.
///
/// `end_exclusive` allows one past the last line, which is what an
/// end-of-range index means.
pub(crate) unsafe fn normalize_index(
    buf: *mut buf_T,
    index: int64_t,
    end_exclusive: bool,
    oob: *mut bool,
) -> int64_t {
    // SAFETY: `buf` is a loaded buffer and `oob` the caller's flag.
    unsafe {
        assert!((*buf).b_ml.ml_line_count > 0);
        let max_index = ((*buf).b_ml.ml_line_count + end_exclusive as linenr_T - 1) as int64_t;
        let mut index = if index < 0 {
            max_index + index + 1
        } else {
            index
        };
        if index > max_index {
            *oob = true;
            index = max_index;
        } else if index < 0 {
            *oob = true;
            index = 0;
        }
        index + 1
    }
}

/// The text of line `lnum` between the two columns, as a *borrowed* string
/// into the buffer's own line. Negative columns count back from the end.
pub(crate) unsafe fn buf_get_text(
    buf: *mut buf_T,
    lnum: int64_t,
    start_col: int64_t,
    end_col: int64_t,
    err: *mut Error,
) -> String_0 {
    // SAFETY: `buf` is a loaded buffer and `err` the caller's error slot.
    unsafe {
        if lnum >= i64::from(MAXLNUM) {
            api_err_invalid(
                err,
                c"line index".as_ptr(),
                c"out of range".as_ptr(),
                0,
                false,
            );
            return STRING_INIT;
        }
        let bufstr = ml_get_buf(buf, lnum as linenr_T);
        let line_length = ml_get_buf_len(buf, lnum as linenr_T) as int64_t;

        let relative = |col: int64_t| if col < 0 { line_length + col + 1 } else { col };
        let start_col = relative(start_col).clamp(0, line_length);
        let end_col = relative(end_col).clamp(0, line_length);
        if start_col > end_col {
            let msg = c"start_col must be less than or equal to end_col".as_ptr();
            api_set_error(err, kErrorTypeValidation, msg);
            return STRING_INIT;
        }
        String_0 {
            data: bufstr.offset(start_col as isize),
            size: (end_col - start_col) as size_t,
        }
    }
}
