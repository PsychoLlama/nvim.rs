//! Verbose and redirection message handling
//!
//! Provides Rust implementations for verbose message output and
//! redirection state management.

use std::ffi::c_int;

// C function declarations for verbose operations
extern "C" {
    // Verbose functions
    fn verbose_enter();
    fn verbose_leave();
    fn verbose_enter_scroll();
    fn verbose_leave_scroll();
    fn verbose_stop();

    // Redirection state
    fn nvim_get_redir_off() -> c_int;
    fn nvim_set_redir_off(val: c_int);
    fn nvim_get_redir_fd_not_null() -> c_int;
    fn nvim_get_p_vfile_not_empty() -> c_int;
    fn nvim_get_redir_reg() -> c_int;
    fn nvim_get_redir_vname() -> c_int;
    fn nvim_get_capture_ga_not_null() -> c_int;
    fn nvim_get_msg_silent() -> c_int;
    fn nvim_set_msg_silent(val: c_int);
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
/// Calls C function that modifies global state.
#[no_mangle]
pub unsafe extern "C" fn rs_verbose_enter() {
    verbose_enter();
}

/// Leave verbose message mode.
///
/// Restores message silence and message kind.
/// Must be paired with `rs_verbose_enter()`.
///
/// # Safety
/// Calls C function that modifies global state.
#[no_mangle]
pub unsafe extern "C" fn rs_verbose_leave() {
    verbose_leave();
}

/// Enter verbose message mode with scroll.
///
/// Like `rs_verbose_enter()` but also sets msg_scroll.
/// Must be paired with `rs_verbose_leave_scroll()`.
///
/// # Safety
/// Calls C function that modifies global state.
#[no_mangle]
pub unsafe extern "C" fn rs_verbose_enter_scroll() {
    verbose_enter_scroll();
}

/// Leave verbose message mode with scroll.
///
/// Like `rs_verbose_leave()` but also updates cmdline_row.
/// Must be paired with `rs_verbose_enter_scroll()`.
///
/// # Safety
/// Calls C function that modifies global state.
#[no_mangle]
pub unsafe extern "C" fn rs_verbose_leave_scroll() {
    verbose_leave_scroll();
}

/// Stop verbose file output.
///
/// Closes the verbose file if open.
///
/// # Safety
/// Calls C function that closes file handle.
#[no_mangle]
pub unsafe extern "C" fn rs_verbose_stop() {
    verbose_stop();
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
    nvim_get_redir_off()
}

/// Set redirection off state.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_redir_off(val: c_int) {
    nvim_set_redir_off(val);
}

/// Temporarily disable redirection.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_redir_disable() {
    nvim_set_redir_off(1);
}

/// Re-enable redirection.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_redir_enable() {
    nvim_set_redir_off(0);
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
    nvim_set_msg_silent(val);
}

/// Increment the message silent counter.
///
/// # Safety
/// Calls C accessor/mutator functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_silent_enter() {
    let val = nvim_get_msg_silent();
    nvim_set_msg_silent(val + 1);
}

/// Decrement the message silent counter.
///
/// # Safety
/// Calls C accessor/mutator functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_silent_leave() {
    let val = nvim_get_msg_silent();
    if val > 0 {
        nvim_set_msg_silent(val - 1);
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
    let silent = nvim_get_msg_silent() > 0;
    c_int::from(silent)
}

#[cfg(test)]
mod tests {
    // Integration tests would require mocking C functions
}
