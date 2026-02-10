//! Macro recording and playback
//!
//! This module provides Rust implementations for macro recording state
//! and related functions.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

use std::ffi::c_int;

/// Maximum bytes for multi-byte character plus special key sequence
const GOTCHARS_BUF_SIZE: usize = 3 * 4 + 4; // MB_MAXBYTES * 3 + 4

/// State for adding bytes to a recording or 'showcmd'.
///
/// This mirrors the C `gotchars_state_T` structure.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct GotcharsState {
    /// Buffer for accumulating bytes
    buf: [u8; GOTCHARS_BUF_SIZE],
    /// Previous character (for detecting special sequences)
    prev_c: c_int,
    /// Number of bytes currently in buf
    buflen: usize,
    /// Number of pending special key bytes
    pending_special: u32,
    /// Number of pending multibyte character bytes
    pending_mbyte: u32,
}

impl Default for GotcharsState {
    fn default() -> Self {
        Self::new()
    }
}

impl GotcharsState {
    /// Create a new, empty state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buf: [0; GOTCHARS_BUF_SIZE],
            prev_c: 0,
            buflen: 0,
            pending_special: 0,
            pending_mbyte: 0,
        }
    }

    /// Clear the state buffer.
    pub const fn clear(&mut self) {
        self.buflen = 0;
        self.prev_c = 0;
        self.pending_special = 0;
        self.pending_mbyte = 0;
    }

    /// Get the current buffer contents.
    #[must_use]
    pub fn buffer(&self) -> &[u8] {
        &self.buf[..self.buflen]
    }

    /// Get the buffer length.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.buflen
    }

    /// Check if the buffer is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.buflen == 0
    }

    /// Check if we're waiting for more bytes (special sequence or multibyte).
    #[must_use]
    pub const fn is_pending(&self) -> bool {
        self.pending_special > 0 || self.pending_mbyte > 0
    }

    /// Reset the buffer length to zero, keeping other state.
    pub const fn reset_buflen(&mut self) {
        self.buflen = 0;
    }
}

// =============================================================================
// C FFI Accessor Functions
// =============================================================================

#[allow(dead_code)]
extern "C" {
    /// Get the current recording register (0 if not recording)
    fn nvim_get_reg_recording() -> c_int;
    /// Get last_recorded_len
    fn nvim_get_last_recorded_len() -> usize;
    /// Set last_recorded_len
    fn nvim_set_last_recorded_len(val: usize);
    /// Add last_recorded_len
    fn nvim_add_last_recorded_len(val: usize);
    /// Check if block_redo is set
    fn nvim_get_block_redo() -> c_int;
    /// Get reg_executing
    fn nvim_get_reg_executing() -> c_int;
    /// Set reg_executing
    fn nvim_set_reg_executing(val: c_int);
    /// Get pending_end_reg_executing
    fn nvim_get_pending_end_reg_executing() -> c_int;
    /// Set pending_end_reg_executing
    fn nvim_set_pending_end_reg_executing(val: c_int);
    /// Get typebuf.tb_maplen
    fn nvim_get_typebuf_maplen() -> c_int;
}

/// Check if we are currently recording a macro.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_is_recording() -> c_int {
    c_int::from(nvim_get_reg_recording() != 0)
}

/// Get the register being recorded into (0 if not recording).
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_get_recording_register() -> c_int {
    nvim_get_reg_recording()
}

/// Get the length of the last recorded sequence.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_get_last_recorded_len() -> usize {
    nvim_get_last_recorded_len()
}

/// Set the length of the last recorded sequence.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_last_recorded_len(len: usize) {
    nvim_set_last_recorded_len(len);
}

/// Add to the length of the last recorded sequence.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_add_last_recorded_len(len: usize) {
    nvim_add_last_recorded_len(len);
}

/// Check if redo buffer modifications are blocked.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_is_block_redo() -> c_int {
    nvim_get_block_redo()
}

/// When peeking and not getting a character, reg_executing cannot be cleared
/// yet, so set a flag to clear it later.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_check_end_reg_executing(advance: c_int) {
    let reg_executing = nvim_get_reg_executing();
    let tb_maplen = nvim_get_typebuf_maplen();
    let pending = nvim_get_pending_end_reg_executing() != 0;

    if reg_executing != 0 && (tb_maplen == 0 || pending) {
        if advance != 0 {
            nvim_set_reg_executing(0);
            nvim_set_pending_end_reg_executing(0);
        } else {
            nvim_set_pending_end_reg_executing(1);
        }
    }
}

// =============================================================================
// Key Constants for special sequences
// =============================================================================

/// K_SPECIAL byte that introduces a special key sequence
pub const K_SPECIAL: u8 = 0x80;

/// KS_MODIFIER byte that indicates a modifier follows
pub const KS_MODIFIER: u8 = 252;

/// Compute TO_SPECIAL(a, b) = a * 256 + b
#[must_use]
pub const fn to_special(a: c_int, b: c_int) -> c_int {
    a * 256 + b
}

/// Get the byte length for a UTF-8 leading byte.
///
/// Returns 1 for ASCII, 2-4 for valid UTF-8 lead bytes, 1 for invalid.
#[must_use]
pub const fn mb_byte2len(byte: u8) -> usize {
    // Note: ASCII (< 0x80) and invalid continuation bytes (0x80-0xbf)
    // both return 1, so we handle them together
    if byte < 0xc0 {
        1
    } else if byte < 0xe0 {
        2
    } else if byte < 0xf0 {
        3
    } else if byte < 0xf8 {
        4
    } else {
        1 // invalid (>= 0xf8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gotchars_state_new() {
        let state = GotcharsState::new();
        assert!(state.is_empty());
        assert!(!state.is_pending());
    }

    #[test]
    fn test_gotchars_state_clear() {
        let mut state = GotcharsState::new();
        state.buflen = 5;
        state.pending_special = 2;
        state.clear();
        assert!(state.is_empty());
        assert!(!state.is_pending());
    }

    #[test]
    fn test_mb_byte2len() {
        // ASCII
        assert_eq!(mb_byte2len(b'A'), 1);
        assert_eq!(mb_byte2len(0x7f), 1);

        // 2-byte UTF-8
        assert_eq!(mb_byte2len(0xc2), 2);
        assert_eq!(mb_byte2len(0xdf), 2);

        // 3-byte UTF-8
        assert_eq!(mb_byte2len(0xe0), 3);
        assert_eq!(mb_byte2len(0xef), 3);

        // 4-byte UTF-8
        assert_eq!(mb_byte2len(0xf0), 4);
        assert_eq!(mb_byte2len(0xf4), 4);
    }

    #[test]
    fn test_to_special() {
        assert_eq!(to_special(0, 0), 0);
        assert_eq!(to_special(1, 0), 256);
        assert_eq!(to_special(1, 1), 257);
    }
}
