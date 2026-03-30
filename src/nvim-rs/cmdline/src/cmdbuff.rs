//! Command-line buffer allocation/deallocation/reallocation.
//!
//! Rust implementations of `alloc_cmdbuff`, `dealloc_cmdbuff`, `realloc_cmdbuff`.
//! These manage the `ccline.cmdbuff` allocation.

#![allow(unsafe_code)]

use std::ffi::{c_char, c_int};
use std::ptr;

// =============================================================================
// C extern declarations
// =============================================================================

unsafe extern "C" {
    fn xmalloc(size: usize) -> *mut c_char;
    fn xfree(ptr: *mut c_char);

    fn nvim_get_ccline_cmdbuff() -> *mut c_char;
    fn nvim_get_ccline_cmdbufflen() -> c_int;
    fn nvim_get_ccline_cmdlen() -> c_int;
    fn nvim_set_ccline_cmdbuff(buff: *mut c_char);
    fn nvim_set_ccline_cmdbufflen(len: c_int);
    fn nvim_set_ccline_cmdlen(len: c_int);

    /// Adjust ccline.xpc->xp_pattern after reallocation.
    /// old_buff is the previous value of ccline.cmdbuff before it was freed.
    fn nvim_realloc_cmdbuff_xp_fixup(old_buff: *const c_char);
}

// =============================================================================
// Exported implementations
// =============================================================================

/// Allocate a new command line buffer.
///
/// Assigns the new buffer to `ccline.cmdbuff` and `ccline.cmdbufflen`.
/// Adds extra space to avoid frequent reallocations.
///
/// Direct replacement for C `alloc_cmdbuff()`.
///
/// # Safety
///
/// Calls C functions to set global `ccline` state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn alloc_cmdbuff(len: c_int) {
    // Give some extra space to avoid having to allocate all the time.
    let actual_len = if len < 80 { 100 } else { len + 20 };
    let buf = xmalloc(actual_len as usize);
    nvim_set_ccline_cmdbuff(buf);
    nvim_set_ccline_cmdbufflen(actual_len);
}

/// Deallocate the command line buffer, updating the buffer size and length.
///
/// Direct replacement for C `dealloc_cmdbuff()`.
///
/// # Safety
///
/// Calls C functions to clear global `ccline` state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dealloc_cmdbuff() {
    let buf = nvim_get_ccline_cmdbuff();
    if !buf.is_null() {
        xfree(buf);
        nvim_set_ccline_cmdbuff(ptr::null_mut());
    }
    nvim_set_ccline_cmdlen(0);
    nvim_set_ccline_cmdbufflen(0);
}

/// Re-allocate the command line to length `len` + some extra.
///
/// If the current buffer is already large enough, does nothing.
/// Copies existing content and NUL-terminates. Also fixes up `ccline.xpc->xp_pattern`
/// if it pointed into the old buffer.
///
/// Returns 0 always (kept for Rust callers that check return value).
///
/// Direct replacement for C `realloc_cmdbuff()`.
///
/// # Safety
///
/// Calls C functions to access and set global `ccline` state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn realloc_cmdbuff(len: c_int) -> c_int {
    let current_len = nvim_get_ccline_cmdbufflen();
    if len < current_len {
        return 0; // no need to resize
    }

    let old_p = nvim_get_ccline_cmdbuff();
    let cmdlen = nvim_get_ccline_cmdlen();

    // Allocate new buffer (will also update ccline.cmdbuff and cmdbufflen).
    alloc_cmdbuff(len);

    let new_p = nvim_get_ccline_cmdbuff();

    // Copy existing content (there isn't always a NUL after the command,
    // but it may need to be there, thus copy up to the NUL and add a NUL).
    if !old_p.is_null() && !new_p.is_null() && cmdlen > 0 {
        ptr::copy_nonoverlapping(old_p, new_p, cmdlen as usize);
    }
    if !new_p.is_null() {
        *new_p.add(cmdlen as usize) = 0; // NUL terminate
    }

    // Fix up xpc->xp_pattern if it pointed into the old buffer.
    nvim_realloc_cmdbuff_xp_fixup(old_p);

    // Free old buffer.
    if !old_p.is_null() {
        xfree(old_p);
    }

    0
}
