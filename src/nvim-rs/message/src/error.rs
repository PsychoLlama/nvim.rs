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
    /// Check if p_debug contains a specific character
    fn nvim_p_debug_contains(c: c_int) -> c_int;
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
/// This matches the C function `emsg_not_now()` exactly.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_not_now() -> c_int {
    let emsg_off = nvim_get_emsg_off();
    let emsg_skip = nvim_get_emsg_skip();

    // If emsg_off > 0 and debug doesn't contain 'm' or 't', skip messages
    // If emsg_skip > 0, always skip messages
    let skip_due_to_off = emsg_off > 0
        && nvim_p_debug_contains(c_int::from(b'm')) == 0
        && nvim_p_debug_contains(c_int::from(b't')) == 0;

    c_int::from(skip_due_to_off || emsg_skip > 0)
}

/// Simplified check if error messages should not be shown.
///
/// Only checks emsg_skip, not the full emsg_not_now logic.
/// Use `rs_emsg_not_now` for the full check.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_not_now_simple() -> c_int {
    // Simplified version - only check emsg_skip
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

// Additional C accessor declarations for warning functionality
extern "C" {
    /// Get `did_emsg_syntax` flag
    fn nvim_get_did_emsg_syntax() -> c_int;
}

/// Check if did_emsg was set because of a syntax error.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_did_emsg_syntax() -> c_int {
    nvim_get_did_emsg_syntax()
}

/// Check if error should be output now.
///
/// Returns false (0) if error messages should not be shown because:
/// - emsg_off is set OR
/// - emsg_skip is set
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_now() -> c_int {
    let off = nvim_get_emsg_off();
    let skip = nvim_get_emsg_skip();
    c_int::from(off == 0 && skip == 0)
}

/// Reset error counters for a fresh start.
///
/// Clears did_emsg, called_emsg, and emsg_on_display.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_reset_counters() {
    nvim_set_did_emsg(0);
    nvim_set_called_emsg(0);
    nvim_set_emsg_on_display(0);
}

/// Combined check for all error suppression.
///
/// Returns the total "depth" of error suppression (sum of counters).
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_suppression_depth() -> c_int {
    nvim_get_emsg_off() + nvim_get_emsg_skip() + nvim_get_emsg_silent()
}

// ============================================================================
// Phase 423: Error Message Output Functions
// ============================================================================

extern "C" {
    // Error message output functions - call into C
    fn emsg_multiline(
        s: *const std::ffi::c_char,
        kind: *const std::ffi::c_char,
        hl_id: c_int,
        multiline: c_int,
    ) -> c_int;
    fn iemsg(s: *const std::ffi::c_char);

    // Source info functions
    fn msg_source(hl_id: c_int);
    fn reset_last_sourcing();
}

use std::ffi::c_char;

/// Display a simple error message.
///
/// This is the standard function for displaying error messages to the user.
/// It uses the default error highlight (HLF_E) and "emsg" kind.
///
/// Equivalent to C's `emsg()` function.
///
/// # Arguments
/// * `s` - The error message string (NUL-terminated)
///
/// # Returns
/// * `true` (1) if wait_return() was not called
/// * `false` (0) if wait_return() was called
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_emsg(s: *const c_char) -> c_int {
    static KIND: &[u8] = b"emsg\0";
    emsg_multiline(s, KIND.as_ptr().cast::<c_char>(), HLF_E, 0)
}

/// Display an error message with kind and highlight.
///
/// Full version of emsg with all options. Used for multiline error
/// messages and special highlighting.
///
/// # Arguments
/// * `s` - The error message string (NUL-terminated)
/// * `kind` - Message kind for ext_messages (e.g., "emsg")
/// * `hl_id` - Highlight group ID (typically HLF_E for errors)
/// * `multiline` - If true, handle embedded newlines specially
///
/// # Returns
/// * `true` (1) if wait_return() was not called
/// * `false` (0) if wait_return() was called
///
/// # Safety
/// - `s` and `kind` must be valid NUL-terminated C strings
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_multiline_full(
    s: *const c_char,
    kind: *const c_char,
    hl_id: c_int,
    multiline: c_int,
) -> c_int {
    emsg_multiline(s, kind, hl_id, multiline)
}

/// Display an internal error message.
///
/// For internal Neovim errors that shouldn't normally occur.
/// Always displayed regardless of suppression state.
///
/// # Arguments
/// * `s` - The error message string (NUL-terminated)
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_iemsg(s: *const c_char) {
    iemsg(s);
}

/// Display source info before an error message.
///
/// Shows the script name and line number where the error occurred.
///
/// # Arguments
/// * `hl_id` - Highlight group ID for the source info
///
/// # Safety
/// Calls C function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_source(hl_id: c_int) {
    msg_source(hl_id);
}

/// Reset the last sourcing info.
///
/// Clears the cached source name/line so it will be
/// displayed again for the next error.
///
/// # Safety
/// Calls C function.
#[no_mangle]
pub unsafe extern "C" fn rs_reset_last_sourcing() {
    reset_last_sourcing();
}

// ============================================================================
// Error Message Constants
// ============================================================================

/// Highlight face for error messages (HLF_E)
pub const HLF_E: c_int = 6;

/// Highlight face for warning messages (HLF_W)
pub const HLF_W: c_int = 44;

/// Internal error message prefix
pub const IEMSG_PREFIX: &[u8] = b"internal error: \0";

// ============================================================================
// Convenience Functions
// ============================================================================

/// Display an error with multiline handling.
///
/// Convenience wrapper for multiline error messages with standard error highlight.
///
/// # Arguments
/// * `s` - The error message string
/// * `kind` - Message kind for ext_messages
///
/// # Returns
/// Result from `rs_emsg_multiline_full`
///
/// # Safety
/// - `s` and `kind` must be valid NUL-terminated C strings
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_multiline_hl(s: *const c_char, kind: *const c_char) -> c_int {
    emsg_multiline(s, kind, HLF_E, 1)
}

/// Check if error output is currently possible.
///
/// Returns true if errors can be displayed right now.
/// Uses the simple check (emsg_off and emsg_skip).
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_can_output() -> c_int {
    // Use the simple check - same as rs_emsg_now
    let off = nvim_get_emsg_off();
    let skip = nvim_get_emsg_skip();
    c_int::from(off == 0 && skip == 0)
}

/// Begin an error message sequence.
///
/// Increments error counters to track that an error is being processed.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_begin() {
    rs_inc_called_emsg();
}

/// End an error message sequence.
///
/// Sets the emsg_on_display flag to indicate an error was shown.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_end() {
    nvim_set_emsg_on_display(1);
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

    #[test]
    fn test_max_packed_values() {
        // Test edge cases for packing - max values
        let off = 1023; // 0x3FF
        let skip = 1023;
        let silent = 1023;
        let packed = off | (skip << 10) | (silent << 20);
        assert_eq!(packed & 0x3FF, 1023);
        assert_eq!((packed >> 10) & 0x3FF, 1023);
        assert_eq!((packed >> 20) & 0x3FF, 1023);
    }

    #[test]
    fn test_zero_packed_values() {
        let packed = 0;
        assert_eq!(packed & 0x3FF, 0);
        assert_eq!((packed >> 10) & 0x3FF, 0);
        assert_eq!((packed >> 20) & 0x3FF, 0);
    }
}
