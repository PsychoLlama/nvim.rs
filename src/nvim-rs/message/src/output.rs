//! Message output utilities
//!
//! Provides Rust implementations for message output state management
//! and coordination with the display system.

use std::ffi::c_int;

// C accessor declarations
extern "C" {
    /// Get `msg_didany` flag
    fn nvim_get_msg_didany() -> c_int;
    /// Set `msg_didany` flag
    fn nvim_set_msg_didany(val: c_int);
    /// Get `msg_didout` flag
    fn nvim_get_msg_didout() -> c_int;
    /// Set `msg_didout` flag
    fn nvim_set_msg_didout(val: c_int);
    /// Get `msg_nowait` flag
    fn nvim_get_msg_nowait() -> c_int;
    /// Set `msg_nowait` flag
    fn nvim_set_msg_nowait(val: c_int);
    /// Get `msg_no_more` flag
    fn nvim_get_msg_no_more() -> c_int;
    /// Get `lines_left` counter
    fn nvim_get_lines_left() -> c_int;
    /// Set `lines_left` counter
    fn nvim_set_lines_left(val: c_int);
    /// Get `need_wait_return` flag
    fn nvim_get_need_wait_return() -> c_int;
    /// Set `need_wait_return` flag
    fn nvim_set_need_wait_return(val: c_int);
    /// Get `msg_scrolled` global
    fn nvim_get_msg_scrolled() -> c_int;
    /// Get `msg_scrolled_ign` flag
    fn nvim_get_msg_scrolled_ign() -> c_int;
    /// Get `emsg_on_display` flag
    fn nvim_get_emsg_on_display() -> c_int;
    /// Set `emsg_on_display` flag
    fn nvim_set_emsg_on_display(val: c_int);
    /// Get `need_fileinfo` flag
    fn nvim_get_need_fileinfo() -> c_int;
    /// Set `need_fileinfo` flag
    fn nvim_set_need_fileinfo(val: c_int);
    /// Get `p_ch` (cmdheight) option
    fn nvim_get_p_ch() -> i64;
    /// Check if UI has messages capability
    fn nvim_ui_has_messages() -> c_int;
}

/// Check if any message was output.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_didany() -> c_int {
    nvim_get_msg_didany()
}

/// Set the "message was output" flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_didany(val: c_int) {
    nvim_set_msg_didany(val);
}

/// Check if something was written to the current line.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_didout() -> c_int {
    nvim_get_msg_didout()
}

/// Set the "wrote to current line" flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_didout(val: c_int) {
    nvim_set_msg_didout(val);
}

/// Check if message should not wait.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_nowait() -> c_int {
    nvim_get_msg_nowait()
}

/// Set the "no wait" flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_nowait(val: c_int) {
    nvim_set_msg_nowait(val);
}

/// Get the lines left counter.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_lines_left() -> c_int {
    nvim_get_lines_left()
}

/// Set the lines left counter.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_lines_left(val: c_int) {
    nvim_set_lines_left(val);
}

/// Check if wait_return is needed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_need_wait_return() -> c_int {
    nvim_get_need_wait_return()
}

/// Set the need_wait_return flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_need_wait_return(val: c_int) {
    nvim_set_need_wait_return(val);
}

/// Check if error message is on display.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_on_display() -> c_int {
    nvim_get_emsg_on_display()
}

/// Set the emsg_on_display flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_emsg_on_display(val: c_int) {
    nvim_set_emsg_on_display(val);
}

/// Check if file info is needed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_need_fileinfo() -> c_int {
    nvim_get_need_fileinfo()
}

/// Set the need_fileinfo flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_need_fileinfo(val: c_int) {
    nvim_set_need_fileinfo(val);
}

/// Check if message display has overflowed (scrolled).
///
/// Returns true when a message has been written after scrolling
/// and wait_return may be needed.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_overflow() -> c_int {
    let msg_scrolled = nvim_get_msg_scrolled();
    let p_ch = nvim_get_p_ch();
    let ui_has_messages = nvim_ui_has_messages() != 0;

    // Threshold is 1 when cmdheight is 0, otherwise 0
    let threshold = c_int::from(p_ch == 0);
    c_int::from(!ui_has_messages && msg_scrolled > threshold)
}

/// Check if need_wait_return should be set after output.
///
/// Returns true when:
/// - Overflow condition met AND
/// - Not ignoring scrolled messages AND
/// - Output is not just a CR
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_check_wait_return() -> c_int {
    let overflow = rs_msg_overflow() != 0;
    let scrolled_ign = nvim_get_msg_scrolled_ign() != 0;

    c_int::from(overflow && !scrolled_ign)
}

/// Check if the more prompt should be shown.
///
/// Returns true when lines_left has reached 0 and the more
/// prompt should be displayed for pagination.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_need_more() -> c_int {
    let lines_left = nvim_get_lines_left();
    let msg_no_more = nvim_get_msg_no_more() != 0;

    c_int::from(lines_left == 0 && !msg_no_more)
}

/// Decrement lines_left counter and return true if more prompt needed.
///
/// # Safety
/// Calls C accessor and mutator functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_dec_lines_left() -> c_int {
    let lines_left = nvim_get_lines_left();
    if lines_left > 0 {
        nvim_set_lines_left(lines_left - 1);
    }
    rs_msg_need_more()
}

#[cfg(test)]
mod tests {
    // Integration tests would require mocking C functions
}
