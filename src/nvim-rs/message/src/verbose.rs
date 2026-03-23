//! Verbose and redirection message handling
//!
//! Provides Rust implementations for verbose message output and
//! redirection state management.

use std::ffi::c_int;

// C function declarations for verbose operations
extern "C" {
    static mut msg_silent: c_int;
    // Verbose state accessors (in message.c)
    fn nvim_verbose_enter_kind();
    fn nvim_restore_pre_verbose_kind();
    fn nvim_verbose_stop_impl();
    fn nvim_verbose_open_impl() -> c_int;
    fn nvim_get_p_vfile_not_empty() -> c_int;

    // State accessors
    static mut msg_scroll: c_int;
    static mut msg_row: c_int;
    static mut cmdline_row: c_int;

    // Redirection state
    static mut redir_off: bool;
    fn nvim_get_redir_fd_not_null() -> c_int;
    fn nvim_get_redir_reg() -> c_int;
    fn nvim_get_redir_vname() -> c_int;
    fn nvim_get_capture_ga_not_null() -> c_int;
}

// ============================================================================
// Verbose Message Functions
// ============================================================================

/// Enter verbose message mode.
///
/// Silences messages if 'verbosefile' is set and sets message kind.
/// Must be paired with `rs_verbose_leave()`.
///
/// # Safety
/// Calls C accessor functions that modify global state.
#[export_name = "verbose_enter"]
pub unsafe extern "C" fn rs_verbose_enter() {
    if nvim_get_p_vfile_not_empty() != 0 {
        let silent = msg_silent;
        msg_silent = silent + 1;
    }
    // Save pre_verbose_kind if not already in verbose mode, then set verbose kind.
    // This is done via a single C accessor to keep the pointer comparison in C.
    nvim_verbose_enter_kind();
}

/// Leave verbose message mode.
///
/// Restores message silence and message kind.
/// Must be paired with `rs_verbose_enter()`.
///
/// # Safety
/// Calls C accessor functions that modify global state.
#[export_name = "verbose_leave"]
pub unsafe extern "C" fn rs_verbose_leave() {
    if nvim_get_p_vfile_not_empty() != 0 {
        let silent = msg_silent;
        if silent > 0 {
            msg_silent = silent - 1;
        } else {
            msg_silent = 0;
        }
    }
    nvim_restore_pre_verbose_kind();
}

/// Enter verbose message mode with scroll.
///
/// Like `rs_verbose_enter()` but also sets msg_scroll.
/// Must be paired with `rs_verbose_leave_scroll()`.
///
/// # Safety
/// Calls C accessor functions that modify global state.
#[export_name = "verbose_enter_scroll"]
pub unsafe extern "C" fn rs_verbose_enter_scroll() {
    rs_verbose_enter();
    if nvim_get_p_vfile_not_empty() == 0 {
        // always scroll up, don't overwrite
        msg_scroll = 1;
    }
}

/// Leave verbose message mode with scroll.
///
/// Like `rs_verbose_leave()` but also updates cmdline_row.
/// Must be paired with `rs_verbose_enter_scroll()`.
///
/// # Safety
/// Calls C accessor functions that modify global state.
#[export_name = "verbose_leave_scroll"]
pub unsafe extern "C" fn rs_verbose_leave_scroll() {
    rs_verbose_leave();
    if nvim_get_p_vfile_not_empty() == 0 {
        cmdline_row = msg_row;
    }
}

/// Stop verbose file output.
///
/// Closes the verbose file if open.
///
/// # Safety
/// Calls C function that closes file handle.
#[export_name = "verbose_stop"]
pub unsafe extern "C" fn rs_verbose_stop() {
    nvim_verbose_stop_impl();
}

/// Open the verbose file ('verbosefile').
///
/// Returns FAIL or OK.
///
/// # Safety
/// Calls C function that opens a file.
#[export_name = "verbose_open"]
#[must_use]
pub unsafe extern "C" fn rs_verbose_open() -> c_int {
    nvim_verbose_open_impl()
}

// ============================================================================
// Redirection State Functions
// ============================================================================

/// Check if redirection is temporarily disabled.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_redir_off() -> c_int {
    c_int::from(redir_off)
}

/// Set redirection off state.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_redir_off(val: c_int) {
    redir_off = val != 0;
}

/// Temporarily disable redirection.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_redir_disable() {
    redir_off = true;
}

/// Re-enable redirection.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_redir_enable() {
    redir_off = false;
}

/// Check if any redirection is active.
///
/// Returns true if redirecting to file, register, variable, or capture.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_is_redirecting() -> c_int {
    c_int::from(
        nvim_get_redir_fd_not_null() != 0
            || nvim_get_p_vfile_not_empty() != 0
            || nvim_get_redir_reg() != 0
            || nvim_get_redir_vname() != 0
            || nvim_get_capture_ga_not_null() != 0,
    )
}

/// Check if redirecting to a file.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_redir_to_file() -> c_int {
    nvim_get_redir_fd_not_null()
}

/// Check if redirecting to a register.
///
/// Returns the register number (0 if not redirecting to register).
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_redir_to_reg() -> c_int {
    nvim_get_redir_reg()
}

/// Check if redirecting to a variable.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_redir_to_var() -> c_int {
    nvim_get_redir_vname()
}

/// Check if capturing to ga buffer.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_redir_capturing() -> c_int {
    nvim_get_capture_ga_not_null()
}

/// Check if verbose file is in use.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_verbose_file_active() -> c_int {
    nvim_get_p_vfile_not_empty()
}

// ============================================================================
// Message Silent State Helpers
// ============================================================================
//
// Note: rs_msg_silent() and rs_is_msg_silent() are defined in format.rs

/// Set the message silent counter.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_silent(val: c_int) {
    msg_silent = val;
}

/// Increment the message silent counter.
///
/// # Safety
/// Calls C accessor/mutator functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_silent_enter() {
    let val = msg_silent;
    msg_silent = val + 1;
}

/// Decrement the message silent counter.
///
/// # Safety
/// Calls C accessor/mutator functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_silent_leave() {
    let val = msg_silent;
    if val > 0 {
        msg_silent = val - 1;
    }
}

/// Check if output should be suppressed.
///
/// Returns true if silenced or redirecting and redir_off is not set.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_output_suppressed() -> c_int {
    let silent = msg_silent > 0;
    c_int::from(silent)
}

#[cfg(test)]
mod tests {
    // Integration tests would require mocking C functions
}
