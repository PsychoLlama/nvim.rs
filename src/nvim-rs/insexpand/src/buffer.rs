//! Buffer name completion and buffer scanning support.
//!
//! This module provides helper functions for buffer name completion
//! and buffer scanning during keyword completion.
//! The core buffer operations remain in C.

use std::os::raw::c_int;

// C accessor functions
extern "C" {
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_direction() -> c_int;
    fn nvim_get_compl_interrupted() -> c_int;
    fn nvim_get_compl_started() -> c_int;
    fn nvim_get_compl_cont_status() -> c_int;
}

// CTRL-X mode for buffer names
const CTRL_X_BUFNAMES: c_int = 18;

// Continuation status flags
const CONT_LOCAL: c_int = 32;

/// Check if completion was interrupted during buffer name search.
#[no_mangle]
pub unsafe extern "C" fn rs_buffer_was_interrupted() -> c_int {
    nvim_get_compl_interrupted()
}

/// Get the current completion direction for buffer name search.
#[no_mangle]
pub unsafe extern "C" fn rs_buffer_get_direction() -> c_int {
    nvim_get_compl_direction()
}

// =============================================================================
// Phase 3: Buffer Scanning Support Functions
// =============================================================================

/// Check if we're in buffer names completion mode.
#[no_mangle]
pub unsafe extern "C" fn rs_buffer_is_bufnames_mode() -> c_int {
    c_int::from(nvim_get_ctrl_x_mode() == CTRL_X_BUFNAMES)
}

/// Check if buffer scanning should be local only.
///
/// Returns true if CONT_LOCAL flag is set (scan current buffer only).
#[no_mangle]
pub unsafe extern "C" fn rs_buffer_scan_is_local() -> c_int {
    let cont_status = nvim_get_compl_cont_status();
    c_int::from((cont_status & CONT_LOCAL) != 0)
}

/// Check if buffer scanning is active.
///
/// Returns true if completion started and not interrupted.
#[no_mangle]
pub unsafe extern "C" fn rs_buffer_scan_is_active() -> c_int {
    let started = nvim_get_compl_started();
    let interrupted = nvim_get_compl_interrupted();
    c_int::from(started != 0 && interrupted == 0)
}

/// Check if buffer scanning can continue.
///
/// Same as rs_buffer_scan_is_active but more descriptive name.
#[no_mangle]
pub unsafe extern "C" fn rs_buffer_scan_can_continue() -> c_int {
    rs_buffer_scan_is_active()
}

/// Check if direction is forward for buffer scanning.
#[no_mangle]
pub unsafe extern "C" fn rs_buffer_scan_is_forward() -> c_int {
    c_int::from(nvim_get_compl_direction() > 0)
}

/// Check if direction is backward for buffer scanning.
#[no_mangle]
pub unsafe extern "C" fn rs_buffer_scan_is_backward() -> c_int {
    c_int::from(nvim_get_compl_direction() < 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_bufnames_constant() {
        assert_eq!(CTRL_X_BUFNAMES, 18);
    }

    #[test]
    fn test_cont_local_constant() {
        assert_eq!(CONT_LOCAL, 32);
    }
}
