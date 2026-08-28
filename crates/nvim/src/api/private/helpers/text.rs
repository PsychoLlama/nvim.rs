//! Strings and buffer text.
//!
//! An API `String` is a pointer and a length, and may or may not own its
//! bytes: `*_to_string` copies, `*_as_string` borrows. Getting that wrong
//! is a leak or a double free, so the name of every function here says
//! which it is.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{CAR, NL, api_set_error, arena_string, arena_take_arraybuilder};
use crate::api::private::validate::api_err_invalid;
use crate::kvec::InitVec;
use crate::memline::{ml_get_buf, ml_get_buf_len};
use crate::memory::{memchrsub, xmemdupz, xstrndup};
use crate::pos::MAXLNUM;
use crate::types::{
    Arena, Array, ArrayBuilder, Error, NUL, Object, String_0, buf_T, int64_t, kErrorTypeValidation,
    linenr_T, size_t,
};
use ::libc::{strlen, strnlen};
use core::ffi::c_char;
use core::{mem, slice};

// -- Strings ---------------------------------------------------------------

/// The reading half of [`String_0`], which cannot live with the type:
/// `types/` forbids `unsafe` and this dereferences the pointer.
impl String_0 {
    /// The bytes.
    ///
    /// [`String_0::NULL`] answers the empty slice: `slice::from_raw_parts`
    /// may not be handed a null pointer even for a zero length, and the
    /// empty answer is what every caller wants there.
    ///
    /// # Safety
    /// A non-null string must have [`len`](String_0::len) readable bytes at
    /// [`data`](String_0::data), unwritten for `'a`.
    pub unsafe fn as_bytes<'a>(&self) -> &'a [u8] {
        if self.is_null() {
            return &[];
        }
        // SAFETY: caller's contract.
        unsafe { slice::from_raw_parts(self.data().cast::<u8>(), self.len()) }
    }
}

/// A copy of the C string `str`, owned by the caller.
pub(crate) unsafe fn cstr_to_string(str: *const c_char) -> String_0 {
    // SAFETY: `str` is null or NUL-terminated.
    unsafe {
        if str.is_null() {
            return String_0::NULL;
        }
        cbuf_to_string(str, strlen(str))
    }
}

/// A copy of `size` bytes of `buf`, owned by the caller and NUL-terminated
/// however many NULs the bytes themselves hold.
pub(crate) unsafe fn cbuf_to_string(buf: *const c_char, size: size_t) -> String_0 {
    // SAFETY: `buf` has `size` readable bytes.
    unsafe { String_0::from_raw_parts(xmemdupz(buf.cast(), size).cast(), size) }
}

/// A NUL-terminated copy of `str`'s bytes, owned by the caller.
pub(crate) unsafe fn string_to_cstr(str: String_0) -> *mut c_char {
    // SAFETY: `str` has `size` readable bytes.
    unsafe { xstrndup(str.data(), str.len()) }
}

/// `str` viewed as an API string, borrowing rather than copying.
pub(crate) unsafe fn cstr_as_string(str: *const c_char) -> String_0 {
    // SAFETY: `str` is null or NUL-terminated.
    unsafe {
        if str.is_null() {
            return String_0::NULL;
        }
        String_0::from_raw_parts(str as *mut c_char, strlen(str))
    }
}

/// [`cstr_as_string`] for a buffer that need not be NUL-terminated within
/// `maxsize` bytes.
pub(crate) unsafe fn cstrn_as_string(str: *mut c_char, maxsize: size_t) -> String_0 {
    // SAFETY: `str` has `maxsize` readable bytes.
    unsafe { String_0::from_raw_parts(str, strnlen(str, maxsize)) }
}

/// Split `input` into one array item per line, arena-allocating the lines.
///
/// Line breaks are `\n`, or `\r` and `\r\n` as well with `crlf`. A NUL in
/// the text stands for a newline, as it does everywhere a buffer line is
/// passed as a C string, and is turned back into one. Text that ends *with*
/// a break gets a trailing empty item, so that the array round-trips.
pub(crate) unsafe fn string_to_array(input: String_0, crlf: bool, arena: *mut Arena) -> Array {
    // SAFETY: an `ArrayBuilder` is a size, a capacity, two pointers and an
    // inline array of plain-data objects, so all-zero is a valid value.
    let mut ret: ArrayBuilder = unsafe { mem::zeroed() };
    let mut items = InitVec::new(
        &mut ret.size,
        &mut ret.capacity,
        &mut ret.items,
        &mut ret.init_array,
    );
    items.init();

    let mut i: size_t = 0;
    while i < input.len() {
        let start = input.data().wrapping_add(i);
        let mut end = start;
        let mut line_len: size_t = 0;
        while line_len < input.len() - i {
            end = start.wrapping_add(line_len);
            // SAFETY: the caller's promise -- `end` is inside the input.
            let byte = unsafe { *end };
            if byte == NL || (crlf && byte == CAR) {
                break;
            }
            line_len += 1;
        }
        i += line_len;
        // SAFETY: as above -- `end` is the break the walk stopped at, or the
        // last byte it looked at.
        let at_break = unsafe { *end };
        let ends_line = at_break == NL || (crlf && at_break == CAR);
        // A CRLF counts as one break, so the LF is stepped over as well.
        if crlf && at_break == CAR && i + 1 < input.len() {
            // SAFETY: the byte after the CR is still inside the input.
            if unsafe { *end.add(1) } == NL {
                i += 1;
            }
        }

        let borrowed = String_0::from_raw_parts(start, line_len);
        // SAFETY: the line has `line_len` readable bytes at `start`.
        let s = unsafe { arena_string(arena, borrowed) };
        // SAFETY: `s` is that many bytes of the arena, this call's own.
        unsafe { memchrsub(s.data().cast(), NUL as c_char, NL, line_len) };
        items.push(Object::string(s));
        if i + 1 == input.len() && ends_line {
            // Text that ends with a break round-trips through a trailing
            // empty item.
            items.push(Object::string(String_0::NULL));
        }
        i += 1;
    }
    // SAFETY: `ret` is this frame's builder, filled in above.
    unsafe { arena_take_arraybuilder(arena, &raw mut ret) }
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
    // SAFETY: the caller's promise -- `buf` is a loaded buffer.
    let line_count = unsafe { (*buf).b_ml.ml_line_count };
    debug_assert!(line_count > 0);
    let max_index = (line_count + end_exclusive as linenr_T - 1) as int64_t;
    let mut index = if index < 0 {
        max_index + index + 1
    } else {
        index
    };
    if index > max_index {
        // SAFETY: the caller's promise -- `oob` is their flag.
        unsafe { *oob = true };
        index = max_index;
    } else if index < 0 {
        // SAFETY: as above.
        unsafe { *oob = true };
        index = 0;
    }
    index + 1
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
    if lnum >= i64::from(MAXLNUM) {
        let out_of_range = c"out of range".as_ptr();
        // SAFETY: the caller's promise about `err`; both strings are static.
        unsafe { api_err_invalid(err, c"line index".as_ptr(), out_of_range, 0, false) };
        return String_0::NULL;
    }
    // SAFETY: the caller's promise -- `buf` is a loaded buffer, and `lnum`
    // is below `MAXLNUM`.
    let bufstr = unsafe { ml_get_buf(buf, lnum as linenr_T) };
    // SAFETY: as above.
    let line_length = unsafe { ml_get_buf_len(buf, lnum as linenr_T) } as int64_t;

    let relative = |col: int64_t| if col < 0 { line_length + col + 1 } else { col };
    let start_col = relative(start_col).clamp(0, line_length);
    let end_col = relative(end_col).clamp(0, line_length);
    if start_col > end_col {
        let msg = c"start_col must be less than or equal to end_col".as_ptr();
        // SAFETY: the caller's promise about `err`.
        unsafe { api_set_error(err, kErrorTypeValidation, msg) };
        return String_0::NULL;
    }
    // SAFETY: `start_col` was clamped into the line.
    let text = unsafe { bufstr.offset(start_col as isize) };
    String_0::from_raw_parts(text, (end_col - start_col) as size_t)
}
