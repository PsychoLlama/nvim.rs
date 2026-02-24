//! Completion case inference.
//!
//! This module provides case inference for completion text, matching the
//! case style of the originally typed text (infercase feature).
//! The core logic remains in C due to heavy use of multibyte C functions.

#![allow(dead_code, unused_imports)]
use std::os::raw::{c_char, c_int};

// C compound accessor for the infercase logic
extern "C" {
    fn nvim_ins_compl_infercase_gettext_impl(
        str_ptr: *const c_char,
        char_len: c_int,
        compl_char_len: c_int,
        min_len: c_int,
        tofree: *mut *mut c_char,
    ) -> *mut c_char;
}

/// Infer the case of completed text based on the originally typed text.
///
/// Returns the case-adjusted completion text. If a new allocation was made,
/// `*tofree` is set to the allocated buffer (caller must free it).
///
/// # Safety
/// - `str_ptr` must point to a valid NUL-terminated C string.
/// - `tofree` must be a valid pointer to a mutable `*mut c_char` (initialized to null).
/// - The returned pointer is valid until `*tofree` is freed.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_infercase_gettext(
    str_ptr: *const c_char,
    char_len: c_int,
    compl_char_len: c_int,
    min_len: c_int,
    tofree: *mut *mut c_char,
) -> *mut c_char {
    nvim_ins_compl_infercase_gettext_impl(str_ptr, char_len, compl_char_len, min_len, tofree)
}

#[cfg(test)]
mod tests {
    // No pure-logic tests possible here since all logic is in C.
    // Integration tests for case inference are handled by existing test suite.
    #[test]
    fn test_module_exists() {
        // Module compiles successfully.
    }
}
