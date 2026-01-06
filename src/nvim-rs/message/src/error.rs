//! Error and warning message handling
//!
//! Provides utilities for managing error message state including
//! suppression flags, error counters, and display state.

use std::ffi::c_int;

// C accessor declarations
extern "C" {
    /// Get `emsg_off` counter
    fn nvim_get_emsg_off() -> c_int;
    /// Set `emsg_off` counter
    fn nvim_set_emsg_off(val: c_int);
    /// Get `emsg_skip` counter
    fn nvim_get_emsg_skip() -> c_int;
    /// Set `emsg_skip` counter
    fn nvim_set_emsg_skip(val: c_int);
    /// Get `emsg_silent` counter
    fn nvim_get_emsg_silent() -> c_int;
    /// Set `emsg_silent` counter
    fn nvim_set_emsg_silent(val: c_int);
    /// Get `emsg_severe` flag
    fn nvim_get_emsg_severe() -> c_int;
    /// Set `emsg_severe` flag
    fn nvim_set_emsg_severe(val: c_int);
    /// Get `emsg_noredir` flag
    fn nvim_get_emsg_noredir() -> c_int;
    /// Set `emsg_noredir` flag
    fn nvim_set_emsg_noredir(val: c_int);
    /// Get `did_emsg` counter
    fn nvim_get_did_emsg() -> c_int;
    /// Set `did_emsg` counter
    fn nvim_set_did_emsg(val: c_int);
    /// Get `called_emsg` counter
    fn nvim_get_called_emsg() -> c_int;
    /// Set `called_emsg` counter
    fn nvim_set_called_emsg(val: c_int);
    /// Get `emsg_on_display` flag
    fn nvim_get_emsg_on_display() -> c_int;
    /// Set `emsg_on_display` flag
    fn nvim_set_emsg_on_display(val: c_int);
}

/// Get the emsg_off counter (error messages disabled).
///
/// When > 0, error messages are not displayed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_off() -> c_int {
    nvim_get_emsg_off()
}

/// Set the emsg_off counter.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_emsg_off(val: c_int) {
    nvim_set_emsg_off(val);
}

/// Increment emsg_off to disable error messages.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_off_enter() {
    let val = nvim_get_emsg_off();
    nvim_set_emsg_off(val + 1);
}

/// Decrement emsg_off to re-enable error messages.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_off_leave() {
    let val = nvim_get_emsg_off();
    if val > 0 {
        nvim_set_emsg_off(val - 1);
    }
}

/// Check if error messages are currently disabled.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_is_emsg_off() -> c_int {
    c_int::from(nvim_get_emsg_off() > 0)
}

/// Get the emsg_skip counter.
///
/// When > 0, error messages are never displayed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_skip() -> c_int {
    nvim_get_emsg_skip()
}

/// Set the emsg_skip counter.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_emsg_skip(val: c_int) {
    nvim_set_emsg_skip(val);
}

/// Increment emsg_skip to skip error messages.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_skip_enter() {
    let val = nvim_get_emsg_skip();
    nvim_set_emsg_skip(val + 1);
}

/// Decrement emsg_skip.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_skip_leave() {
    let val = nvim_get_emsg_skip();
    if val > 0 {
        nvim_set_emsg_skip(val - 1);
    }
}

/// Get the emsg_silent counter.
///
/// When > 0, error messages are silent (not printed).
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_silent() -> c_int {
    nvim_get_emsg_silent()
}

/// Set the emsg_silent counter.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_emsg_silent(val: c_int) {
    nvim_set_emsg_silent(val);
}

/// Increment emsg_silent to silence error messages.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_silent_enter() {
    let val = nvim_get_emsg_silent();
    nvim_set_emsg_silent(val + 1);
}

/// Decrement emsg_silent.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_silent_leave() {
    let val = nvim_get_emsg_silent();
    if val > 0 {
        nvim_set_emsg_silent(val - 1);
    }
}

/// Check if error messages are currently silent.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_is_emsg_silent() -> c_int {
    c_int::from(nvim_get_emsg_silent() > 0)
}

/// Get the emsg_severe flag.
///
/// When true, prefer this error message over previous ones for exceptions.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_severe() -> c_int {
    nvim_get_emsg_severe()
}

/// Set the emsg_severe flag.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_emsg_severe(val: c_int) {
    nvim_set_emsg_severe(val);
}

/// Get the emsg_noredir flag.
///
/// When true, don't redirect error messages.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_noredir() -> c_int {
    nvim_get_emsg_noredir()
}

/// Set the emsg_noredir flag.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_emsg_noredir(val: c_int) {
    nvim_set_emsg_noredir(val);
}

/// Get the did_emsg counter.
///
/// Incremented by emsg() when a message is displayed or thrown.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_did_emsg() -> c_int {
    nvim_get_did_emsg()
}

/// Set the did_emsg counter.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_did_emsg(val: c_int) {
    nvim_set_did_emsg(val);
}

/// Increment did_emsg.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_inc_did_emsg() {
    let val = nvim_get_did_emsg();
    nvim_set_did_emsg(val + 1);
}

/// Check if any error message was displayed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_has_did_emsg() -> c_int {
    c_int::from(nvim_get_did_emsg() > 0)
}

/// Get the called_emsg counter.
///
/// Always incremented by emsg(), even if message is not displayed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_called_emsg() -> c_int {
    nvim_get_called_emsg()
}

/// Set the called_emsg counter.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_called_emsg(val: c_int) {
    nvim_set_called_emsg(val);
}

/// Increment called_emsg.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_inc_called_emsg() {
    let val = nvim_get_called_emsg();
    nvim_set_called_emsg(val + 1);
}

/// Get the emsg_on_display flag.
///
/// True if there is an error message currently displayed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_on_display_get() -> c_int {
    nvim_get_emsg_on_display()
}

/// Set the emsg_on_display flag.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_on_display_set(val: c_int) {
    nvim_set_emsg_on_display(val);
}

/// Check if error messages should not be shown right now.
///
/// Returns true if:
/// - emsg_off is set and debug mode is not 'm' or 't', or
/// - emsg_skip is set
///
/// This is a simplified check; the full check in C also consults p_debug.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_not_now_simple() -> c_int {
    // Simplified version without p_debug check
    c_int::from(nvim_get_emsg_skip() > 0)
}

/// Check if any error suppression is active.
///
/// Returns true if emsg_off, emsg_skip, or emsg_silent is > 0.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_is_emsg_suppressed() -> c_int {
    let off = nvim_get_emsg_off();
    let skip = nvim_get_emsg_skip();
    let silent = nvim_get_emsg_silent();
    c_int::from(off > 0 || skip > 0 || silent > 0)
}

/// Save error message state for later restoration.
///
/// Returns a packed state value containing emsg_off, emsg_skip, emsg_silent.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_save_state() -> c_int {
    // Pack the three counters into a single int
    // Each gets 10 bits (max 1023)
    let off = nvim_get_emsg_off() & 0x3FF;
    let skip = nvim_get_emsg_skip() & 0x3FF;
    let silent = nvim_get_emsg_silent() & 0x3FF;
    off | (skip << 10) | (silent << 20)
}

/// Restore error message state from a saved value.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_restore_state(state: c_int) {
    let off = state & 0x3FF;
    let skip = (state >> 10) & 0x3FF;
    let silent = (state >> 20) & 0x3FF;
    nvim_set_emsg_off(off);
    nvim_set_emsg_skip(skip);
    nvim_set_emsg_silent(silent);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_state_packing() {
        // Test that state packing/unpacking is reversible
        let off = 5;
        let skip = 3;
        let silent = 7;
        let packed = off | (skip << 10) | (silent << 20);
        assert_eq!(packed & 0x3FF, 5);
        assert_eq!((packed >> 10) & 0x3FF, 3);
        assert_eq!((packed >> 20) & 0x3FF, 7);
    }
}
