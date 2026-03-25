//! Error reporting functions for the buffer crate.
//!
//! Consolidates all error-message wrappers that were previously C functions in
//! `buffer_shim.c`. Each function calls `gettext()` on the English message string
//! (which serves as the gettext msgid) then dispatches to `emsg()`/`semsg()`.
//!
//! The English strings are copied byte-for-byte from the C source so that
//! gettext lookup is identical regardless of where the string literal lives.

#![allow(unsafe_code)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::doc_markdown)]

use std::ffi::{c_char, c_int};

use crate::messages::gt;
use crate::BufHandle;

extern "C" {
    fn emsg(s: *const c_char) -> bool;
    fn semsg(fmt: *const c_char, ...);
    fn nvim_buf_get_b_fname(buf: BufHandle) -> *const c_char;
    fn nvim_buf_get_b_ffname(buf: BufHandle) -> *const c_char;
}

// ---------------------------------------------------------------------------
// Error string constants (copied exactly from buffer_shim.c / errors.h).
// ---------------------------------------------------------------------------

/// "E84: No modified buffer found"
const E84: &std::ffi::CStr = c"E84: No modified buffer found";

/// "E85: There is no listed buffer"
const E85: &std::ffi::CStr = c"E85: There is no listed buffer";

/// "E87: Cannot go beyond last buffer"
const E87: &std::ffi::CStr = c"E87: Cannot go beyond last buffer";

/// "E88: Cannot go before first buffer"
const E88: &std::ffi::CStr = c"E88: Cannot go before first buffer";

/// "E86: Buffer %ld does not exist" -- from errors.h `e_nobufnr`
/// (PRId64 expands to "ld" on Linux x86_64)
const E_NOBUFNR: &std::ffi::CStr = c"E86: Buffer %ld does not exist";

/// "E93: More than one match for %s"
const E93: &std::ffi::CStr = c"E93: More than one match for %s";

/// "E94: No matching buffer for %s"
const E94: &std::ffi::CStr = c"E94: No matching buffer for %s";

/// "E95: Buffer with this name already exists"
const E95: &std::ffi::CStr = c"E95: Buffer with this name already exists";

/// "E23: No alternate file" -- same as MSG_E_NOALT, but used for emsg() variant
const E_NOALT: &std::ffi::CStr = c"E23: No alternate file";

/// "E937: Attempt to delete a buffer that is in use: %s"
const E937: &std::ffi::CStr = c"E937: Attempt to delete a buffer that is in use: %s";

// ---------------------------------------------------------------------------
// Error wrapper functions.
// ---------------------------------------------------------------------------

/// Emit E90 "Cannot unload last buffer".
pub unsafe fn emsg_e90() {
    emsg(gt(c"E90: Cannot unload last buffer".as_ptr()));
}

/// Emit E84 "No modified buffer found".
pub unsafe fn emsg_e84() {
    emsg(gt(E84.as_ptr()));
}

/// Emit E85 "There is no listed buffer".
pub unsafe fn emsg_e85() {
    emsg(gt(E85.as_ptr()));
}

/// Emit E87 "Cannot go beyond last buffer".
pub unsafe fn emsg_e87() {
    emsg(gt(E87.as_ptr()));
}

/// Emit E88 "Cannot go before first buffer".
pub unsafe fn emsg_e88() {
    emsg(gt(E88.as_ptr()));
}

/// Emit E86 "Buffer %ld does not exist" with buffer number.
pub unsafe fn semsg_e_nobufnr(count: i64) {
    semsg(gt(E_NOBUFNR.as_ptr()), count);
}

/// Emit E93 "More than one match for %s" with pattern.
pub unsafe fn blfp_errmsg_e93(pattern: *const c_char) {
    semsg(gt(E93.as_ptr()), pattern);
}

/// Emit E94 "No matching buffer for %s" with pattern.
pub unsafe fn blfp_errmsg_e94(pattern: *const c_char) {
    semsg(gt(E94.as_ptr()), pattern);
}

/// Emit E95 "Buffer with this name already exists".
pub unsafe fn emsg_e95_buffer_exists() {
    emsg(gt(E95.as_ptr()));
}

/// Emit E23 "No alternate file".
pub unsafe fn emsg_noalt() {
    emsg(gt(E_NOALT.as_ptr()));
}

/// Emit E937 "Attempt to delete a buffer that is in use: %s".
///
/// Prefers `buf->b_fname`, falls back to `buf->b_ffname`, then "[No Name]".
pub unsafe fn emsg_e937_buf_in_use(buf: BufHandle) {
    let b_fname = nvim_buf_get_b_fname(buf);
    let display = if b_fname.is_null() {
        let b_ffname = nvim_buf_get_b_ffname(buf);
        if b_ffname.is_null() {
            c"[No Name]".as_ptr()
        } else {
            b_ffname
        }
    } else {
        b_fname
    };
    semsg(gt(E937.as_ptr()), display);
}

/// Return type used by emsg-returning functions (`c_int`).
pub type CInt = c_int;
