//! Handing an owned Rust buffer to a caller that releases it with `xfree`.
//!
//! The tree's global allocator *is* libc's (`allocator.rs`), so a `Vec`, a
//! `Box<[T]>` or a `CString` may be given to code that ends its life with
//! `free`. These two helpers are that handover written down once, with the
//! two traps it has:
//!
//! * the result must be NUL-terminated, because the receiver reads it as a C
//!   string; and
//! * an *empty* boxed slice is a dangling address, not a heap one, so an
//!   empty answer stays the null pointer `xfree` accepts.
//!
//! They are the replacement for a `garray_T` local whose `ga_data` was the
//! value being returned.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_char;
use std::ffi::CString;

/// `text` as a NUL-terminated string the caller owns and `xfree`s.
///
/// Interior NULs are kept, exactly as a byte `garray_T`'s were: the receiver
/// stops at the first one, which is the behaviour it always had.
pub(crate) fn owned_cstr(mut text: Vec<u8>) -> *mut c_char {
    text.push(0);
    Box::into_raw(text.into_boxed_slice()).cast::<c_char>()
}

/// `strings` as a `char **` of that many owned strings, all of which the
/// caller `xfree`s. Empty answers as a null pointer.
pub(crate) fn owned_cstr_array(strings: Vec<CString>) -> *mut *mut c_char {
    if strings.is_empty() {
        return core::ptr::null_mut();
    }
    let raw: Vec<*mut c_char> = strings.into_iter().map(CString::into_raw).collect();
    Box::into_raw(raw.into_boxed_slice()).cast::<*mut c_char>()
}
