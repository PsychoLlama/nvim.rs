//! Edit mode state queries for Neovim
//!
//! This crate provides Rust implementations of edit-related functions
//! from `src/nvim/edit.c`. Uses accessor pattern for static variable access.

#![allow(unsafe_code)] // FFI requires unsafe

use std::ffi::{c_char, c_int};

// C accessor functions for edit state.
// These are defined in edit.c and provide safe access to static variables.
extern "C" {
    /// Get the `ins_need_undo` static variable.
    fn nvim_get_ins_need_undo() -> c_int;
    /// Get the `can_cindent` static variable.
    fn nvim_get_can_cindent() -> c_int;
    /// Get buf->b_prompt_text for a buffer.
    fn nvim_buf_get_b_prompt_text(buf: *const std::ffi::c_void) -> *const c_char;
    /// Get curbuf handle.
    fn nvim_get_curbuf() -> *const std::ffi::c_void;
}

/// Check if undo is needed for insert mode.
///
/// Returns the value of the static `ins_need_undo` variable.
#[inline]
fn ins_need_undo_get_impl() -> bool {
    // SAFETY: nvim_get_ins_need_undo is a simple global accessor
    unsafe { nvim_get_ins_need_undo() != 0 }
}

/// FFI wrapper for `ins_need_undo_get`.
#[no_mangle]
pub extern "C" fn rs_ins_need_undo_get() -> c_int {
    c_int::from(ins_need_undo_get_impl())
}

/// Get whether cindenting may be done on this line.
///
/// # Safety
/// Calls C accessor function for `can_cindent` static.
#[no_mangle]
pub unsafe extern "C" fn rs_get_can_cindent() -> c_int {
    nvim_get_can_cindent()
}

/// Default prompt text used when buffer has no custom prompt.
const DEFAULT_PROMPT: &[u8] = b"% \0";

/// Get the effective prompt text for the specified buffer.
///
/// Returns the buffer's `b_prompt_text` if set, otherwise returns "% ".
///
/// # Safety
/// The buf pointer must be a valid `buf_T` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_prompt_text(buf: *const std::ffi::c_void) -> *const c_char {
    let prompt = nvim_buf_get_b_prompt_text(buf);
    if prompt.is_null() {
        DEFAULT_PROMPT.as_ptr().cast()
    } else {
        prompt
    }
}

/// Get the effective prompt for the current buffer.
///
/// # Safety
/// Accesses curbuf global.
#[no_mangle]
pub unsafe extern "C" fn rs_prompt_text() -> *const c_char {
    rs_buf_prompt_text(nvim_get_curbuf())
}

#[cfg(test)]
mod tests {
    // Note: Tests would need to mock the C accessor functions
}
