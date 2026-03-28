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

    /// Add a byte to the state, accumulating until a full character is ready.
    ///
    /// Returns `true` when a complete character (possibly multi-byte or
    /// special key sequence) has been accumulated and is ready to process.
    ///
    /// This is the Rust equivalent of C `gotchars_add_byte`.
    pub fn add_byte(&mut self, byte: u8) -> bool {
        let c_byte = c_int::from(byte);
        self.buf[self.buflen] = byte;
        self.buflen += 1;
        let mut c = c_byte;
        let in_special = self.pending_special > 0;
        let in_mbyte = self.pending_mbyte > 0;

        if in_special {
            self.pending_special -= 1;
        } else if c == c_int::from(K_SPECIAL) {
            // When receiving a special key sequence, store it until we have all
            // the bytes and we can decide what to do with it.
            self.pending_special = 2;
        }

        if self.pending_special > 0 {
            self.prev_c = c;
            return false;
        }

        if in_mbyte {
            self.pending_mbyte -= 1;
        } else {
            if in_special {
                if self.prev_c == c_int::from(KS_MODIFIER) {
                    // When receiving a modifier, wait for the modified key.
                    self.prev_c = c;
                    return false;
                }
                c = crate::stuff::to_special(self.prev_c, c);
            }
            // When receiving a multibyte character, store it until we have all
            // the bytes, so that it won't be split between two buffer blocks,
            // and delete_buff_tail() will work properly.
            let mb_len = crate::stuff::mb_byte2len_check_pub(c);
            self.pending_mbyte = if mb_len > 1 { mb_len as u32 - 1 } else { 0 };
        }

        if self.pending_mbyte > 0 {
            self.prev_c = c;
            return false;
        }

        self.prev_c = c;
        true
    }

    /// NUL-terminate the buffer contents (for passing to C).
    pub const fn nul_terminate(&mut self) {
        if self.buflen < self.buf.len() {
            self.buf[self.buflen] = 0;
        }
    }
}

// =============================================================================
// C FFI Accessor Functions
// =============================================================================

// last_recorded_len moved from C static to Rust (Phase 3); exported #[no_mangle] for C access.
#[no_mangle]
pub static mut last_recorded_len: usize = 0;

extern "C" {
    /// Get the current recording register (0 if not recording)
    fn nvim_get_reg_recording() -> c_int;
    /// reg_executing: register being executed or zero
    static mut reg_executing: c_int;
    /// pending_end_reg_executing: clear reg_executing at a safe moment
    static mut pending_end_reg_executing: bool;
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
/// Accesses `last_recorded_len` static directly.
#[no_mangle]
pub unsafe extern "C" fn rs_get_last_recorded_len() -> usize {
    last_recorded_len
}

/// Set the length of the last recorded sequence.
///
/// # Safety
/// Accesses `last_recorded_len` static directly.
#[no_mangle]
pub unsafe extern "C" fn rs_set_last_recorded_len(len: usize) {
    last_recorded_len = len;
}

/// Add to the length of the last recorded sequence.
///
/// # Safety
/// Accesses `last_recorded_len` static directly.
#[no_mangle]
pub unsafe extern "C" fn rs_add_last_recorded_len(len: usize) {
    last_recorded_len += len;
}

/// Check if redo buffer modifications are blocked.
#[no_mangle]
pub unsafe extern "C" fn rs_is_block_redo() -> c_int {
    c_int::from(crate::buffheader::is_block_redo())
}

/// When peeking and not getting a character, reg_executing cannot be cleared
/// yet, so set a flag to clear it later.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_check_end_reg_executing(advance: c_int) {
    let re = reg_executing;
    let tb_maplen = nvim_get_typebuf_maplen();
    let pending = pending_end_reg_executing;

    if re != 0 && (tb_maplen == 0 || pending) {
        if advance != 0 {
            reg_executing = 0;
            pending_end_reg_executing = false;
        } else {
            pending_end_reg_executing = true;
        }
    }
}

/// `check_end_reg_executing(bool advance)` -- Phase 1 export replacing C wrapper
///
/// # Safety
/// Calls C accessor functions.
#[export_name = "check_end_reg_executing"]
pub unsafe extern "C" fn check_end_reg_executing_export(advance: bool) {
    rs_check_end_reg_executing(c_int::from(advance));
}

// =============================================================================
// Phase 4: gotchars / ungetchars / gotchars_ignore
// =============================================================================

extern "C" {
    // (on_key_buf_process removed - now calls Rust function directly)
    /// debug_did_msg global (direct access)
    static mut debug_did_msg: bool;
    /// maptick: tick for each non-mapped char
    static mut maptick: c_int;
    /// scriptout: FILE* for -w output
    static mut scriptout: *mut std::ffi::c_void;
    /// p_uc: 'updatecount' option (OptInt = i64)
    static p_uc: i64;
    /// p_fs: 'fsync' option
    static p_fs: c_int;
    /// ml_sync_all: sync all memfiles
    fn ml_sync_all(check_file: c_int, check_char: c_int, do_fsync: bool);
    /// fputc: write a byte to a FILE
    fn fputc(c: c_int, stream: *mut std::ffi::c_void) -> c_int;
    /// p_sc: 'showcmd' option
    static p_sc: c_int;
    /// msg_silent: suppress messages when non-zero
    static mut msg_silent: c_int;
    /// mb_unescape: unescape a key sequence, advancing the pointer
    fn mb_unescape(pp: *mut *const std::ffi::c_char) -> *const std::ffi::c_char;
    /// utf_ptr2char: convert UTF-8 pointer to codepoint
    fn utf_ptr2char(p: *const u8) -> c_int;
    /// merge_modifiers: merge modifier bits into key, return result
    fn merge_modifiers(c: c_int, modifiers: *mut c_int) -> c_int;
    /// add_to_showcmd: add a character to the showcmd display
    fn add_to_showcmd(c: c_int) -> bool;
}

/// Static counter for updatescript (mirrors C's `static int count`).
static mut UPDATESCRIPT_COUNT: c_int = 0;

/// Write typed character to script file, sync memfiles when threshold reached.
///
/// # Safety
/// Accesses C globals scriptout, p_uc, p_fs and calls ml_sync_all.
pub(crate) unsafe fn updatescript(c: c_int) {
    if c != 0 {
        let sout = std::ptr::read(std::ptr::addr_of!(scriptout));
        if !sout.is_null() {
            fputc(c, sout);
        }
    }
    let idle = c == 0;
    let uc = std::ptr::read(std::ptr::addr_of!(p_uc));
    let count = std::ptr::addr_of_mut!(UPDATESCRIPT_COUNT);
    *count += 1;
    if idle || (uc > 0 && *count >= uc as c_int) {
        let fs = std::ptr::read(std::ptr::addr_of!(p_fs));
        ml_sync_all(c_int::from(idle), 1, fs != 0 || idle);
        *count = 0;
    }
}

/// Static gotchars state for rs_gotchars.
static mut GOTCHARS_STATE: GotcharsState = GotcharsState::new();

/// Static gotchars state for add_byte_to_showcmd (separate instance from GOTCHARS_STATE).
static mut SHOWCMD_STATE: GotcharsState = GotcharsState::new();

/// K_SPECIAL byte value (0x80)
const K_SPECIAL_BYTE: u8 = 0x80;
/// KS_MODIFIER byte value (252)
const KS_MODIFIER_BYTE: u8 = 252;

/// Add a single byte to 'showcmd' for a partially matched mapping.
/// Calls add_to_showcmd() when a full key has been received.
///
/// # Safety
/// Calls C functions for showcmd display.
#[no_mangle]
pub unsafe extern "C" fn rs_add_byte_to_showcmd(byte: u8) {
    let sc = std::ptr::read(std::ptr::addr_of!(p_sc));
    let ms = std::ptr::read(std::ptr::addr_of!(msg_silent));
    if sc == 0 || ms != 0 {
        return;
    }

    let state = &mut *std::ptr::addr_of_mut!(SHOWCMD_STATE);
    if !state.add_byte(byte) {
        return;
    }

    // NUL-terminate and reset buflen
    state.nul_terminate();
    state.reset_buflen();

    let mut modifiers: c_int = 0;
    let mut c: c_int = 0; // NUL

    let ptr_start = state.buf.as_ptr();
    let mut ptr = ptr_start;

    // Check for modifier prefix: K_SPECIAL KS_MODIFIER mod_byte
    if *ptr == K_SPECIAL_BYTE && *ptr.add(1) == KS_MODIFIER_BYTE && *ptr.add(2) != 0 {
        modifiers = c_int::from(*ptr.add(2));
        ptr = ptr.add(3);
    }

    if *ptr != 0 {
        let mut cpp = ptr.cast::<std::ffi::c_char>();
        let mb_ptr = mb_unescape(std::ptr::addr_of_mut!(cpp));
        if mb_ptr.is_null() {
            c = c_int::from(*ptr);
            ptr = ptr.add(1);
        } else {
            c = utf_ptr2char(mb_ptr.cast::<u8>());
            ptr = cpp.cast::<u8>();
        }
        if c <= 0x7f {
            let mut modifiers_after = modifiers;
            let mod_c = merge_modifiers(c, std::ptr::addr_of_mut!(modifiers_after));
            if modifiers_after == 0 {
                modifiers = 0;
                c = mod_c;
            }
        }
    }

    if modifiers != 0 {
        add_to_showcmd(c_int::from(K_SPECIAL_BYTE));
        add_to_showcmd(c_int::from(KS_MODIFIER_BYTE));
        add_to_showcmd(modifiers);
    }
    if c != 0 {
        add_to_showcmd(c);
    }
    while *ptr != 0 {
        add_to_showcmd(c_int::from(*ptr));
        ptr = ptr.add(1);
    }
}

/// Write typed characters to script file.
/// If recording is on, put the character in the record buffer.
///
/// # Safety
/// `chars` must point to at least `len` valid bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_gotchars(chars: *const u8, len: usize) {
    let slice = std::slice::from_raw_parts(chars, len);
    let state = &mut *std::ptr::addr_of_mut!(GOTCHARS_STATE);

    for &byte in slice {
        if !state.add_byte(byte) {
            continue;
        }

        // Handle one byte at a time; no translation to be done.
        for i in 0..state.buflen {
            updatescript(c_int::from(state.buf[i]));
        }

        // Process on_key_buf (handles on_key_ignore_len subtraction)
        crate::orchestrator::on_key_buf_process(state.buf.as_ptr(), state.buflen);

        if nvim_get_reg_recording() != 0 {
            state.nul_terminate();
            crate::buffheader::recordbuff().add(&state.buf[..state.buflen]);
            last_recorded_len += state.buflen;
        }

        state.buflen = 0;
    }

    crate::rs_may_sync_undo();

    // output "debug mode" message next time in debug mode
    debug_did_msg = false;

    // Since characters have been typed, consider the following to be in
    // another mapping. Search string will be kept in history.
    maptick += 1;
}

/// Record an <Ignore> key.
#[export_name = "gotchars_ignore"]
pub unsafe extern "C" fn rs_gotchars_ignore() {
    let nop_buf: [u8; 3] = [K_SPECIAL, crate::stuff::KS_EXTRA, crate::stuff::KE_IGNORE];
    crate::orchestrator::on_key_ignore_len_add(3);
    rs_gotchars(nop_buf.as_ptr(), 3);
}

/// Undo the last gotchars() for "len" bytes. To be used when putting a typed
/// character back into the typeahead buffer, thus gotchars() will be called
/// again. Only affects recorded characters.
#[export_name = "ungetchars"]
pub unsafe extern "C" fn rs_ungetchars(len: c_int) {
    if nvim_get_reg_recording() == 0 {
        return;
    }
    crate::buffheader::recordbuff().delete_tail(len as usize);
    last_recorded_len = last_recorded_len.saturating_sub(len as usize);
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

    #[test]
    fn test_add_byte_ascii() {
        let mut state = GotcharsState::new();
        // Single ASCII byte should be a complete character
        assert!(state.add_byte(b'A'));
        assert_eq!(state.buflen, 1);
        assert_eq!(state.buf[0], b'A');
    }

    #[test]
    fn test_add_byte_k_special_sequence() {
        let mut state = GotcharsState::new();
        // K_SPECIAL starts a 3-byte sequence
        assert!(!state.add_byte(K_SPECIAL)); // not complete yet
        assert!(!state.add_byte(253)); // KS_EXTRA, still waiting
        assert!(state.add_byte(4)); // KE_IGNORE, now complete
        assert_eq!(state.buflen, 3);
    }

    #[test]
    fn test_add_byte_utf8_multibyte() {
        let mut state = GotcharsState::new();
        // 2-byte UTF-8 character (é = 0xC3 0xA9)
        assert!(!state.add_byte(0xC3)); // lead byte, expect 1 more
        assert!(state.add_byte(0xA9)); // continuation, complete
        assert_eq!(state.buflen, 2);
    }

    #[test]
    fn test_add_byte_modifier_sequence() {
        let mut state = GotcharsState::new();
        // K_SPECIAL + KS_MODIFIER + modifier_byte + actual key
        assert!(!state.add_byte(K_SPECIAL)); // start special
        assert!(!state.add_byte(KS_MODIFIER)); // modifier indicator
        assert!(!state.add_byte(0x04)); // modifier value (ctrl)
        assert!(state.add_byte(b'a')); // modified key, complete
        assert_eq!(state.buflen, 4);
    }
}
