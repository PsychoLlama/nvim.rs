//! The crate's C-string vocabulary.
//!
//! Fifteen modules each grew their own `cstr_at`/`cstr_opt`/`cstr_bytes`
//! one-liner around [`CStr`]; this is the one home. Three questions are
//! being asked, and the names say which:
//!
//! - **`at*`** — borrow the string a pointer points at. `unsafe`, because
//!   only the caller knows the string is live and terminated. Plain
//!   pointer-to-[`CStr`] is [`CStr::from_ptr`] and stays there; what lives
//!   here is the null case and the bytes case, which std does not spell.
//! - **`in_*`** — borrow the string a buffer *starts with*. Safe: the
//!   buffer bounds the search, and a buffer with no terminator answers the
//!   empty string rather than reading past the end.
//! - **`owned`** — copy bytes into a [`CString`].
//!
//! The borrowing forms hand out an unbounded lifetime, which is what every
//! caller needed: the string outlives the pointer variable it was read
//! from. That is the caller's obligation, stated once here.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char};
use core::slice;
use std::ffi::CString;

/// [`CStr::from_ptr`], answering `None` for a null pointer.
///
/// # Safety
/// A non-null `p` points at a NUL-terminated string that stays live and
/// unwritten for `'a`.
pub(crate) unsafe fn at_opt<'a>(p: *const c_char) -> Option<&'a CStr> {
    // SAFETY: caller's contract, minus the null case.
    (!p.is_null()).then(|| unsafe { CStr::from_ptr(p) })
}

/// The bytes of the string at `p`, without its terminator.
///
/// # Safety
/// [`at_opt`]'s contract, minus the null case.
pub(crate) unsafe fn bytes_at<'a>(p: *const c_char) -> &'a [u8] {
    // SAFETY: caller's contract.
    unsafe { CStr::from_ptr(p) }.to_bytes()
}

/// The string `buf` starts with.
///
/// A buffer holding no terminator answers `c""`: every caller is reading a
/// buffer some writer was supposed to terminate, and the empty string is
/// the answer that keeps a formatting bug from becoming a panic.
pub(crate) fn in_bytes(buf: &[u8]) -> &CStr {
    CStr::from_bytes_until_nul(buf).unwrap_or(c"")
}

/// [`in_bytes`] for a buffer a C callee wrote, typed `c_char`.
pub(crate) fn in_chars(buf: &[c_char]) -> &CStr {
    // SAFETY: `c_char` and `u8` have the same size and alignment, and every
    // bit pattern is valid for both.
    let bytes = unsafe { slice::from_raw_parts(buf.as_ptr().cast::<u8>(), buf.len()) };
    in_bytes(bytes)
}

/// `bytes` as an owned C string.
///
/// # Panics
/// If `bytes` holds an interior NUL. Callers pass text a parser has already
/// split on NUL; use [`in_bytes`] where the terminator is inside the bytes.
pub(crate) fn owned(bytes: &[u8]) -> CString {
    CString::new(bytes).expect("the bytes hold an interior NUL")
}
