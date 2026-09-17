//! Strings and buffer text.
//!
//! An API `String` owns its bytes, so there is nothing to get wrong about
//! who frees them: this file holds the half of [`String_0`] that has to
//! touch the pointer -- the allocating constructors, `Clone`, `Drop` and
//! the two readers -- because `types/` forbids `unsafe`.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::{CAR, NL};
use crate::api::private::validate::err_out_of_range;
use crate::cstr;
use crate::memline::{ml_get_buf, ml_get_buf_len};
use crate::memory::XString;
use crate::memory::{memchrsub, xfree, xmemdupz, xstrndup};
use crate::pos::MAXLNUM;
use crate::types::{Array, Error, LineNr, NUL, Object, String_0, int64_t, size_t};
use crate::winlayer::Buf;
use ::libc::strnlen;
use core::ffi::{CStr, c_char};
use core::slice;

// -- Strings ---------------------------------------------------------------

/// The half of [`String_0`] that touches the pointer, which cannot live
/// with the type: `types/` forbids `unsafe`.
///
/// The string owns an `xmalloc`ed block of `len() + 1` bytes with a NUL at
/// `len()`, or is [`String_0::NULL`] and owns nothing. That invariant is
/// what makes the two readers *safe*: there is no second answer to "whose
/// bytes are these and how many".
impl String_0 {
    /// The bytes, not counting the terminator.
    ///
    /// [`String_0::NULL`] answers the empty slice: `slice::from_raw_parts`
    /// may not be handed a null pointer even for a zero length, and the
    /// empty answer is what every caller wants there.
    pub fn as_bytes(&self) -> &[u8] {
        if self.is_null() {
            return &[];
        }
        // SAFETY: the type's invariant -- a non-null string owns `len()`
        // readable bytes, and the borrow is this string's.
        unsafe { slice::from_raw_parts(self.data().cast::<u8>(), self.len()) }
    }

    /// The string as a C string, stopping at the terminator this type
    /// always writes.
    ///
    /// [`String_0::NULL`] answers `c""`, as [`as_bytes`](String_0::as_bytes)
    /// answers the empty slice. The bytes are *not* re-measured: a string
    /// holding an interior NUL comes back truncated at it, which is what
    /// every C consumer of `data()` sees anyway.
    pub fn as_cstr(&self) -> &CStr {
        if self.is_null() {
            return c"";
        }
        // SAFETY: the type's invariant -- a non-null string is
        // NUL-terminated, and the borrow is this string's.
        unsafe { CStr::from_ptr(self.data()) }
    }

    /// Take ownership of `size` bytes at `data`.
    ///
    /// # Safety
    ///
    /// `data` must be null -- and then `size` zero -- or an `xmalloc`ed
    /// block of at least `size + 1` bytes with a NUL at `data[size]`, which
    /// nothing else frees or writes.
    pub unsafe fn from_owned_parts(data: *mut c_char, size: size_t) -> Self {
        let mut str = String_0::NULL;
        let (into_data, into_size) = str.parts_mut();
        // SAFETY: the two addresses are `str`'s own fields, and what the
        // caller promised about `data` is exactly the type's invariant.
        unsafe {
            *into_data = data;
            *into_size = size;
        }
        str
    }

    /// Shorten the string to `size` bytes, writing a terminator there.
    ///
    /// Only shortens: the block is not reallocated, so the bytes past `size`
    /// are still the string's and are released with it.
    ///
    /// # Safety
    ///
    /// `size` must not be past the string's current length.
    pub unsafe fn truncate(&mut self, size: size_t) {
        debug_assert!(size <= self.len());
        let (data, len) = self.parts_mut();
        // SAFETY: `size` is inside the block, which has room for a
        // terminator at its own end and therefore at any earlier offset.
        unsafe {
            if !(*data).is_null() {
                *(*data).add(size) = 0;
            }
            *len = size;
        }
    }

    /// A copy of `bytes`, NUL-terminated, owned by the answer.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        // SAFETY: `bytes` is a live slice, and `xmemdupz` answers a block of
        // `len + 1` bytes with the terminator already written.
        let data = unsafe { xmemdupz(bytes.as_ptr().cast(), bytes.len()) };
        // SAFETY: that block is the answer's own.
        unsafe { String_0::from_owned_parts(data.cast(), bytes.len()) }
    }

    /// A copy of `size` bytes at `data`, NUL-terminated, owned by the
    /// answer. A null `data` -- which is how the editor spells "no value"
    /// wherever it carries a pointer and a length -- answers the null
    /// string, since no slice may be built over one.
    ///
    /// # Safety
    ///
    /// `data` must be null, or point at `size` readable bytes.
    pub unsafe fn from_raw_bytes(data: *const c_char, size: size_t) -> Self {
        if data.is_null() {
            return String_0::NULL;
        }
        // SAFETY: the caller's promise, and `data` is not null.
        Self::from_bytes(unsafe { slice::from_raw_parts(data.cast::<u8>(), size) })
    }

    /// A copy of `str`'s bytes, stopping at its terminator.
    pub fn from_cstr(str: &CStr) -> Self {
        Self::from_bytes(str.to_bytes())
    }

    /// Take over an [`XString`]'s block.
    ///
    /// The two types hold the same invariant -- an `xmalloc`ed block of
    /// `size + 1` bytes with a NUL at `size` -- so this handover is the one
    /// that needs no promise from its caller, and it is how a string built
    /// in Rust reaches the API layer without a second copy.
    pub fn from_xstring(string: XString) -> Self {
        let size = string.len();
        // SAFETY: `XString`'s invariant is exactly this type's.
        unsafe { String_0::from_owned_parts(string.into_raw(), size) }
    }
}

impl Clone for String_0 {
    /// A copy with its own allocation. [`String_0::NULL`] stays null rather
    /// than becoming the empty string.
    fn clone(&self) -> Self {
        if self.is_null() {
            return String_0::NULL;
        }
        Self::from_bytes(self.as_bytes())
    }
}

impl Drop for String_0 {
    fn drop(&mut self) {
        // SAFETY: the type's invariant -- the block is the string's own, and
        // `xfree` accepts a null pointer.
        unsafe { xfree(self.data().cast()) };
    }
}

impl From<&CStr> for String_0 {
    /// [`from_cstr`](String_0::from_cstr): a copy of `s`'s bytes.
    fn from(s: &CStr) -> Self {
        Self::from_cstr(s)
    }
}

impl From<&[u8]> for String_0 {
    fn from(bytes: &[u8]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<&str> for String_0 {
    fn from(text: &str) -> Self {
        Self::from_bytes(text.as_bytes())
    }
}

impl PartialEq for String_0 {
    /// Byte equality. The null string equals only itself: it is a value of
    /// its own, not the empty string.
    fn eq(&self, other: &Self) -> bool {
        self.is_null() == other.is_null() && self.as_bytes() == other.as_bytes()
    }
}

impl Eq for String_0 {}

/// A copy of the C string `str`, owned by the caller.
///
/// # Safety
///
/// `str` must be null or point at a NUL-terminated string.
pub(crate) unsafe fn cstr_to_string(str: *const c_char) -> String_0 {
    if str.is_null() {
        return String_0::NULL;
    }
    // SAFETY: `str` is NUL-terminated.
    String_0::from_bytes(unsafe { cstr::bytes_at(str) })
}

/// A copy of `size` bytes of `buf`, owned by the caller and NUL-terminated
/// however many NULs the bytes themselves hold.
///
/// # Safety
///
/// `buf` must point at `size` readable bytes.
pub(crate) unsafe fn cbuf_to_string(buf: *const c_char, size: size_t) -> String_0 {
    // SAFETY: `buf` has `size` readable bytes.
    unsafe { String_0::from_raw_bytes(buf, size) }
}

/// A copy of `str`'s bytes, stopping at a terminator within `maxsize` bytes
/// or at `maxsize`, whichever comes first.
///
/// # Safety
///
/// `str` must point at `maxsize` readable bytes.
pub(crate) unsafe fn cstrn_to_string(str: *const c_char, maxsize: size_t) -> String_0 {
    // SAFETY: the caller's promise.
    let len = unsafe { strnlen(str, maxsize) };
    // SAFETY: as above; `len` is at most `maxsize`.
    unsafe { String_0::from_raw_bytes(str, len) }
}

/// A NUL-terminated copy of `str`'s bytes, owned by the caller.
pub(crate) fn string_to_cstr(str: &String_0) -> *mut c_char {
    // SAFETY: `str` names its own bytes, which are NUL-terminated.
    unsafe { xstrndup(str.data(), str.len()) }
}

/// Split `input` into one array item per line, each line its own string.
///
/// Line breaks are `\n`, or `\r` and `\r\n` as well with `crlf`. A NUL in
/// the text stands for a newline, as it does everywhere a buffer line is
/// passed as a C string, and is turned back into one. Text that ends *with*
/// a break gets a trailing empty item, so that the array round-trips.
pub(crate) fn string_to_array(input: &String_0, crlf: bool) -> Array {
    let bytes = input.as_bytes();
    let is_break = |byte: u8| byte == NL as u8 || (crlf && byte == CAR as u8);
    let mut items: Vec<Object> = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let line_len = bytes[at..]
            .iter()
            .position(|&byte| is_break(byte))
            .unwrap_or(bytes.len() - at);
        let line = String_0::from_bytes(&bytes[at..at + line_len]);
        // SAFETY: `line` names its own `line_len` bytes.
        unsafe { memchrsub(line.data().cast(), NUL as c_char, NL, line_len) };
        items.push(Object::string(line));

        at += line_len;
        let ends_line = at < bytes.len();
        // A CRLF counts as one break, so the LF is stepped over as well.
        if crlf && ends_line && bytes[at] == CAR as u8 && bytes.get(at + 1) == Some(&(NL as u8)) {
            at += 1;
        }
        if at + 1 == bytes.len() && ends_line {
            // Text that ends with a break round-trips through a trailing
            // empty item.
            items.push(Object::string(String_0::NULL));
        }
        at += 1;
    }
    Array::from(items)
}

// -- Buffer text -----------------------------------------------------------

/// Turn a signed, end-relative line index into a 1-based line number,
/// clamping it into the buffer and reporting through `oob` that it had to.
///
/// `end_exclusive` allows one past the last line, which is what an
/// end-of-range index means.
///
/// # Safety
///
/// `oob` must point at a writable `bool` the caller owns.
pub(crate) unsafe fn normalize_index(
    buffer: Buf,
    index: int64_t,
    end_exclusive: bool,
    oob: *mut bool,
) -> int64_t {
    // SAFETY: the caller's promise -- `buffer` is a loaded buffer.
    let line_count = buffer.b_ml.ml_line_count;
    debug_assert!(line_count > 0);
    let max_index = (line_count + end_exclusive as LineNr - 1) as int64_t;
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

/// The text of line `lnum` between the two columns, copied out of the
/// buffer's own line. Negative columns count back from the end.
pub(crate) fn buf_get_text(
    buffer: Buf,
    lnum: int64_t,
    start_col: int64_t,
    end_col: int64_t,
) -> Result<String_0, Error> {
    if lnum >= i64::from(MAXLNUM) {
        return Err(err_out_of_range(c"line index"));
    }
    // SAFETY: the caller's promise -- `buffer` is a loaded buffer, and `lnum`
    // is below `MAXLNUM`.
    let bufstr = unsafe { ml_get_buf(buffer, lnum as LineNr) };
    let line_length = ml_get_buf_len(buffer, lnum as LineNr) as int64_t;

    let relative = |col: int64_t| if col < 0 { line_length + col + 1 } else { col };
    let start_col = relative(start_col).clamp(0, line_length);
    let end_col = relative(end_col).clamp(0, line_length);
    if start_col > end_col {
        let why = c"start_col must be less than or equal to end_col";
        return Err(Error::validation(why));
    }
    // SAFETY: `start_col` and `end_col` were clamped into the line, whose
    // bytes `ml_get_buf` answered.
    let text = unsafe {
        slice::from_raw_parts(
            bufstr.cast::<u8>().offset(start_col as isize),
            (end_col - start_col) as size_t,
        )
    };
    Ok(String_0::from_bytes(text))
}
