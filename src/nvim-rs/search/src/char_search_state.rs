//! Character search state
//!
//! Owns the static state for f/F/t/T character search commands.
//! Previously these lived as C static variables in search.c.

use std::ffi::{c_char, c_int};

// SAFETY: Neovim is single-threaded; these statics are only accessed from the
// main thread. This matches the invariant of the original C static variables.

/// The last character searched for (two slots: [0] = char, [1] = unused).
static mut LASTC: [u8; 2] = [0, 0];

/// Last direction of character search (FORWARD=1, BACKWARD=-1).
static mut LASTCDIR: c_int = 1; // FORWARD

/// Last search was a 't' (until) command if true, 'f' (to) if false.
static mut LAST_T_CMD: bool = true;

/// Byte representation of the last searched character (multi-byte support).
/// MAX_SCHAR_SIZE = 32, so +1 = 33 bytes.
static mut LASTC_BYTES: [u8; 33] = [0; 33];

/// Byte length of the last searched character (>1 for multi-byte).
static mut LASTC_BYTELEN: c_int = 1;

// FFI: utf_char2bytes is provided by nvim-mbyte crate (same link unit)
extern "C" {
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
}

// =============================================================================
// Getters
// =============================================================================

/// Get the last character search direction.
#[inline]
pub fn get_lastcdir() -> c_int {
    // SAFETY: single-threaded
    unsafe { LASTCDIR }
}

/// Get whether the last character search was a 't' command.
#[inline]
pub fn get_last_t_cmd() -> bool {
    // SAFETY: single-threaded
    unsafe { LAST_T_CMD }
}

/// Get a pointer to the lastc_bytes buffer.
#[inline]
pub fn get_lastc_bytes_ptr() -> *const c_char {
    // SAFETY: addr_of! is safe - no reference created to the mutable static.
    std::ptr::addr_of!(LASTC_BYTES).cast()
}

/// Get the byte length of the last searched character.
#[inline]
pub fn get_lastc_bytelen() -> c_int {
    // SAFETY: single-threaded
    unsafe { LASTC_BYTELEN }
}

/// Get a byte from the lastc array (bounds-checked).
#[inline]
pub fn get_lastc(idx: c_int) -> u8 {
    // SAFETY: single-threaded
    unsafe {
        if (0..2).contains(&idx) {
            LASTC[idx as usize]
        } else {
            0
        }
    }
}

// =============================================================================
// Setters
// =============================================================================

/// Set the last character search direction.
#[inline]
pub fn set_lastcdir(dir: c_int) {
    // SAFETY: single-threaded
    unsafe { LASTCDIR = dir }
}

/// Set whether the last character search was a 't' command.
#[inline]
pub fn set_last_t_cmd(val: bool) {
    // SAFETY: single-threaded
    unsafe { LAST_T_CMD = val }
}

/// Set the byte length of the last searched character.
#[inline]
pub fn set_lastc_bytelen(len: c_int) {
    // SAFETY: single-threaded
    unsafe { LASTC_BYTELEN = len }
}

/// Set a byte in the lastc array (bounds-checked).
#[inline]
pub fn set_lastc(idx: c_int, val: u8) {
    // SAFETY: single-threaded
    unsafe {
        if (0..2).contains(&idx) {
            LASTC[idx as usize] = val;
        }
    }
}

/// Bulk copy bytes into lastc_bytes, or clear it if len is 0.
///
/// # Safety
/// `s` must be a valid pointer to `len` bytes if `len > 0`, or may be null if `len == 0`.
pub unsafe fn set_lastc_bytes_raw(s: *const c_char, len: c_int) {
    let dst = std::ptr::addr_of_mut!(LASTC_BYTES).cast::<u8>();
    if len > 0 && !s.is_null() {
        std::ptr::copy_nonoverlapping(s.cast::<u8>(), dst, len as usize);
    } else {
        std::ptr::write_bytes(dst, 0, 33);
    }
}

/// Batch save lastc state for searchc().
///
/// Sets LASTC[0], LASTC_BYTELEN, and LASTC_BYTES.
/// If `nchar_len > 0`, copies composing_bytes to LASTC_BYTES.
/// Otherwise, encodes `c` using utf_char2bytes into LASTC_BYTES.
///
/// # Safety
/// If `nchar_len > 0`, `composing_bytes` must be a valid pointer to at least `nchar_len` bytes.
pub unsafe fn searchc_save_lastc_state(c: c_int, nchar_len: c_int, composing_bytes: *const c_char) {
    std::ptr::addr_of_mut!(LASTC).cast::<u8>().write(c as u8);
    let dst = std::ptr::addr_of_mut!(LASTC_BYTES).cast::<u8>();
    if nchar_len > 0 {
        LASTC_BYTELEN = nchar_len;
        std::ptr::copy_nonoverlapping(composing_bytes.cast::<u8>(), dst, nchar_len as usize);
    } else {
        LASTC_BYTELEN = utf_char2bytes(c, dst.cast());
    }
}
