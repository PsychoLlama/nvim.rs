//! High-level input orchestrator functions
//!
//! This module implements the top-level character input functions that
//! coordinate between the typeahead buffer, mappings, and the terminal.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

use std::ffi::{c_int, c_void};

// =============================================================================
// C FFI Declarations
// =============================================================================

extern "C" {
    /// vgetc: the main keyboard input handler (stays in C until Phase 4)
    fn vgetc() -> c_int;

    /// vgetorpeek: get next character from typeahead or keyboard
    fn vgetorpeek(advance: bool) -> c_int;

    /// get_keystroke: get a keystroke directly from the user
    fn get_keystroke(argvars: *mut c_void) -> c_int;

    /// garbage_collect: run the garbage collector
    fn garbage_collect(testing: bool);

    /// nvim_call_updatescript: call updatescript(c) from C
    fn nvim_call_updatescript(c: c_int);

    /// can_get_old_char: check if old_char is available
    fn rs_can_get_old_char() -> c_int;
    /// get_old_char: retrieve old_char
    fn rs_get_old_char() -> c_int;

    /// Get typebuf.tb_len
    fn nvim_get_typebuf_len() -> c_int;

    /// Get no_mapping global
    fn nvim_get_no_mapping() -> c_int;
    /// Set no_mapping global
    fn nvim_set_no_mapping(val: c_int);

    /// may_garbage_collect: set after garbagecollect() is called
    static may_garbage_collect: bool;

    /// test_disable_char_avail: disables char_avail() for testing
    static test_disable_char_avail: bool;
}

// Key code constants
const NUL: c_int = 0;
const ESC: c_int = 0x1b;

/// K_IGNORE: termcap2key(253, 53) = -(253 + (53 << 8))
const K_IGNORE: c_int = -((253) + (53 << 8));
/// K_VER_SCROLLBAR: termcap2key(249, 'X') = -(249 + (0x58 << 8))
const K_VER_SCROLLBAR: c_int = -((249) + (0x58 << 8));
/// K_HOR_SCROLLBAR: termcap2key(248, 'X') = -(248 + (0x58 << 8))
const K_HOR_SCROLLBAR: c_int = -((248) + (0x58 << 8));
/// K_MOUSEMOVE: termcap2key(253, 100) = -(253 + (100 << 8))
const K_MOUSEMOVE: c_int = -((253) + (100 << 8));

// =============================================================================
// Phase 1: Small pure orchestrators
// =============================================================================

/// Like vgetc(), but never returns NUL when called recursively.
///
/// Gets a key directly from the user if vgetc() returns NUL.
///
/// # Safety
/// Calls C functions.
#[must_use]
#[export_name = "safe_vgetc"]
pub unsafe extern "C" fn rs_safe_vgetc() -> c_int {
    let c = vgetc();
    if c == NUL {
        get_keystroke(std::ptr::null_mut())
    } else {
        c
    }
}

/// Like safe_vgetc(), but loops to handle K_IGNORE.
///
/// Also ignores scrollbar events and mouse move events.
///
/// # Safety
/// Calls C functions.
#[must_use]
#[export_name = "plain_vgetc"]
pub unsafe extern "C" fn rs_plain_vgetc() -> c_int {
    loop {
        let c = rs_safe_vgetc();
        if c != K_IGNORE && c != K_VER_SCROLLBAR && c != K_HOR_SCROLLBAR && c != K_MOUSEMOVE {
            return c;
        }
    }
}

/// Check if a character is available, such that vgetc() will not block.
///
/// Returns NUL if no character is available.
/// If the next character is a special character or multi-byte, the returned
/// character is not valid!
///
/// # Safety
/// Calls C functions.
#[must_use]
#[export_name = "vpeekc"]
pub unsafe extern "C" fn rs_vpeekc() -> c_int {
    if rs_can_get_old_char() != 0 {
        return rs_get_old_char();
    }
    vgetorpeek(false)
}

/// Check if any character is available, also half an escape sequence.
///
/// When no typeahead found, but there is something in the typeahead buffer,
/// it must be an ESC that is recognized as the start of a key code.
///
/// # Safety
/// Calls C functions.
#[must_use]
#[export_name = "vpeekc_any"]
pub unsafe extern "C" fn rs_vpeekc_any() -> c_int {
    let c = rs_vpeekc();
    if c == NUL && nvim_get_typebuf_len() > 0 {
        return ESC;
    }
    c
}

/// Call vpeekc() without causing anything to be mapped.
///
/// Returns true if a character is available, false otherwise.
///
/// # Safety
/// Calls C functions.
#[must_use]
#[export_name = "char_avail"]
pub unsafe extern "C" fn rs_char_avail() -> bool {
    if test_disable_char_avail {
        return false;
    }
    let nm = nvim_get_no_mapping();
    nvim_set_no_mapping(nm + 1);
    let retval = rs_vpeekc();
    nvim_set_no_mapping(nvim_get_no_mapping() - 1);
    retval != NUL
}

/// This function is called just before doing a blocking wait.
///
/// Thus after waiting 'updatetime' for a character to arrive.
///
/// # Safety
/// Calls C functions.
#[export_name = "before_blocking"]
pub unsafe extern "C" fn rs_before_blocking() {
    nvim_call_updatescript(0);
    if may_garbage_collect {
        garbage_collect(false);
    }
}
