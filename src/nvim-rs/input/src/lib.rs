//! Input handling utilities for Neovim
//!
//! This module provides types and utilities for input handling,
//! including terminal detection, input buffer management, and input state.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::borrow_as_ptr)]

use std::ffi::c_int;

// =============================================================================
// Buffer Size Constants
// =============================================================================

/// Read buffer size
pub const READ_BUFFER_SIZE: usize = 0xfff;

/// Input buffer size (read buffer * 4 + max key code len)
pub const INPUT_BUFFER_SIZE: usize = READ_BUFFER_SIZE * 4 + MAX_KEY_CODE_LEN;

/// Maximum key code length
pub const MAX_KEY_CODE_LEN: usize = 21;

/// Get read buffer size
#[no_mangle]
pub extern "C" fn rs_read_buffer_size() -> usize {
    READ_BUFFER_SIZE
}

/// Get input buffer size
#[no_mangle]
pub extern "C" fn rs_input_buffer_size() -> usize {
    INPUT_BUFFER_SIZE
}

/// Get max key code length
#[no_mangle]
pub extern "C" fn rs_max_key_code_len() -> usize {
    MAX_KEY_CODE_LEN
}

// =============================================================================
// Input State
// =============================================================================

/// Input state flags
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputState {
    /// Whether we've reached EOF
    pub eof: bool,
    /// Whether we're blocking on input
    pub blocking: bool,
    /// Whether stdin was used
    pub used_stdin: bool,
}

/// Create default input state
#[no_mangle]
pub extern "C" fn rs_input_state_default() -> InputState {
    InputState {
        eof: false,
        blocking: false,
        used_stdin: false,
    }
}

/// Check if input is at EOF
#[no_mangle]
pub unsafe extern "C" fn rs_input_is_eof(state: *const InputState) -> bool {
    if state.is_null() {
        return false;
    }
    (*state).eof
}

/// Check if input is blocking
#[no_mangle]
pub unsafe extern "C" fn rs_input_is_blocking(state: *const InputState) -> bool {
    if state.is_null() {
        return false;
    }
    (*state).blocking
}

// =============================================================================
// Input Buffer Ring State
// =============================================================================

/// Ring buffer state for input
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct InputRingBuffer {
    /// Read position offset
    pub read_offset: usize,
    /// Write position offset
    pub write_offset: usize,
    /// Buffer capacity
    pub capacity: usize,
}

/// Initialize input ring buffer state
#[no_mangle]
pub extern "C" fn rs_input_ring_init(capacity: usize) -> InputRingBuffer {
    InputRingBuffer {
        read_offset: 0,
        write_offset: 0,
        capacity,
    }
}

/// Get available bytes in ring buffer
#[no_mangle]
pub unsafe extern "C" fn rs_input_ring_available(buf: *const InputRingBuffer) -> usize {
    if buf.is_null() {
        return 0;
    }

    let buf = &*buf;
    if buf.write_offset >= buf.read_offset {
        buf.write_offset - buf.read_offset
    } else {
        buf.capacity - buf.read_offset + buf.write_offset
    }
}

/// Get free space in ring buffer
#[no_mangle]
pub unsafe extern "C" fn rs_input_ring_free_space(buf: *const InputRingBuffer) -> usize {
    if buf.is_null() {
        return 0;
    }

    let buf = &*buf;
    buf.capacity - rs_input_ring_available(buf) - 1
}

/// Check if ring buffer is empty
#[no_mangle]
pub unsafe extern "C" fn rs_input_ring_is_empty(buf: *const InputRingBuffer) -> bool {
    if buf.is_null() {
        return true;
    }

    let buf = &*buf;
    buf.read_offset == buf.write_offset
}

// =============================================================================
// CursorHold Event State
// =============================================================================

/// State for CursorHold event timing
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CursorHoldState {
    /// Time waiting for CursorHold event (in ms)
    pub time: c_int,
    /// tb_change_cnt when waiting started
    pub tb_change_cnt: c_int,
}

/// Initialize CursorHold state
#[no_mangle]
pub extern "C" fn rs_cursorhold_state_init() -> CursorHoldState {
    CursorHoldState {
        time: 0,
        tb_change_cnt: 0,
    }
}

/// Reset CursorHold state
#[no_mangle]
pub unsafe extern "C" fn rs_cursorhold_reset(state: *mut CursorHoldState) {
    if state.is_null() {
        return;
    }

    (*state).time = 0;
    (*state).tb_change_cnt = 0;
}

// =============================================================================
// Breakcheck Level
// =============================================================================

/// Breakcheck levels for checking user interrupts
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakcheckLevel {
    /// Normal breakcheck (100 lines)
    Normal = 0,
    /// Line breakcheck (32 lines)
    Line = 1,
    /// Fast breakcheck (4000 bytes)
    Fast = 2,
    /// Very fast breakcheck (4000 bytes, no events)
    VeryFast = 3,
}

/// Line count for normal breakcheck
pub const BREAKCHECK_NORMAL_LINES: c_int = 100;

/// Line count for line breakcheck
pub const BREAKCHECK_LINE_LINES: c_int = 32;

/// Byte count for fast breakcheck
pub const BREAKCHECK_FAST_BYTES: c_int = 4000;

/// Get line count for normal breakcheck
#[no_mangle]
pub extern "C" fn rs_breakcheck_normal_lines() -> c_int {
    BREAKCHECK_NORMAL_LINES
}

/// Get line count for line breakcheck
#[no_mangle]
pub extern "C" fn rs_breakcheck_line_lines() -> c_int {
    BREAKCHECK_LINE_LINES
}

/// Get byte count for fast breakcheck
#[no_mangle]
pub extern "C" fn rs_breakcheck_fast_bytes() -> c_int {
    BREAKCHECK_FAST_BYTES
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_sizes() {
        assert_eq!(rs_read_buffer_size(), 0xfff);
        assert_eq!(rs_max_key_code_len(), 21);

        let expected_input_size = 0xfff * 4 + 21;
        assert_eq!(rs_input_buffer_size(), expected_input_size);
    }

    #[test]
    fn test_input_state() {
        let state = rs_input_state_default();
        assert!(!state.eof);
        assert!(!state.blocking);
        assert!(!state.used_stdin);

        unsafe {
            assert!(!rs_input_is_eof(&state));
            assert!(!rs_input_is_blocking(&state));
        }
    }

    #[test]
    fn test_ring_buffer() {
        let buf = rs_input_ring_init(1024);
        assert_eq!(buf.capacity, 1024);
        assert_eq!(buf.read_offset, 0);
        assert_eq!(buf.write_offset, 0);

        unsafe {
            assert!(rs_input_ring_is_empty(&buf));
            assert_eq!(rs_input_ring_available(&buf), 0);
            assert_eq!(rs_input_ring_free_space(&buf), 1023);
        }
    }

    #[test]
    fn test_cursorhold_state() {
        let mut state = rs_cursorhold_state_init();
        assert_eq!(state.time, 0);
        assert_eq!(state.tb_change_cnt, 0);

        state.time = 100;
        state.tb_change_cnt = 5;

        unsafe { rs_cursorhold_reset(&mut state) };
        assert_eq!(state.time, 0);
        assert_eq!(state.tb_change_cnt, 0);
    }

    #[test]
    fn test_breakcheck_constants() {
        assert_eq!(rs_breakcheck_normal_lines(), 100);
        assert_eq!(rs_breakcheck_line_lines(), 32);
        assert_eq!(rs_breakcheck_fast_bytes(), 4000);
    }
}
