//! Shared helpers for calling the C-ABI surface from Rust tests.

use std::ffi::{c_char, CStr, CString};

use c2rust_neovim::src::nvim::memory::xfree;

/// A NUL-terminated copy of `s`, kept alive by the caller's binding.
pub fn cstr(s: impl Into<Vec<u8>>) -> CString {
    CString::new(s).unwrap()
}

/// Copy an allocated C string's bytes, then `xfree` the original — the same
/// "prove it was allocated" pattern the Lua specs used via `internalize`.
///
/// # Safety
/// `ptr` must be a valid NUL-terminated string from the `xmalloc` family.
pub unsafe fn take_bytes(ptr: *mut c_char) -> Vec<u8> {
    let owned = CStr::from_ptr(ptr).to_bytes().to_vec();
    xfree(ptr.cast());
    owned
}

/// [`take_bytes`], decoded as UTF-8.
///
/// # Safety
/// `ptr` must be a valid NUL-terminated string from the `xmalloc` family.
pub unsafe fn internalize(ptr: *mut c_char) -> String {
    String::from_utf8(take_bytes(ptr)).unwrap()
}
