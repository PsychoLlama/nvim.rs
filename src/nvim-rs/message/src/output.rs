//! Message output utilities
//!
//! Provides Rust implementations for message output state management
//! and coordination with the display system.

use std::ffi::c_int;

// C accessor declarations
extern "C" {
    static mut msg_silent: c_int;
    /// Get `msg_didany` flag
    static mut msg_didany: bool;
    /// Set `msg_didany` flag
    /// Get `msg_didout` flag
    static mut msg_didout: bool;
    /// Set `msg_didout` flag
    /// Get `msg_nowait` flag
    static mut msg_nowait: bool;
    /// Set `msg_nowait` flag
    /// Get `msg_no_more` flag
    static mut msg_no_more: bool;
    /// Get `lines_left` counter
    static mut lines_left: c_int;
    /// Set `lines_left` counter
    /// Get `need_wait_return` flag
    static mut need_wait_return: bool;
    /// Set `need_wait_return` flag
    /// Get `msg_scrolled` global
    static mut msg_scrolled: c_int;
    /// Get `msg_scrolled_ign` flag
    static mut msg_scrolled_ign: bool;
    /// Get `emsg_on_display` flag
    static mut emsg_on_display: bool;
    /// Set `emsg_on_display` flag
    /// Get `need_fileinfo` flag
    static mut need_fileinfo: bool;
    /// Set `need_fileinfo` flag
    /// Get `p_ch` (cmdheight) option
    static mut p_ch: i64;
    /// Check if UI has messages capability
    fn nvim_ui_has_messages() -> c_int;
}

/// Check if any message was output.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_didany() -> c_int {
    c_int::from(msg_didany)
}

/// Set the "message was output" flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_didany(val: c_int) {
    msg_didany = (val) != 0;
}

/// Check if something was written to the current line.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_didout() -> c_int {
    c_int::from(msg_didout)
}

/// Set the "wrote to current line" flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_didout(val: c_int) {
    msg_didout = (val) != 0;
}

/// Check if message should not wait.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_nowait() -> c_int {
    c_int::from(msg_nowait)
}

/// Set the "no wait" flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_nowait(val: c_int) {
    msg_nowait = (val) != 0;
}

/// Get the lines left counter.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_lines_left() -> c_int {
    lines_left
}

/// Set the lines left counter.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_lines_left(val: c_int) {
    lines_left = val;
}

/// Check if wait_return is needed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_need_wait_return() -> c_int {
    c_int::from(need_wait_return)
}

/// Set the need_wait_return flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_need_wait_return(val: c_int) {
    need_wait_return = (val) != 0;
}

/// Check if error message is on display.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_on_display() -> c_int {
    c_int::from(emsg_on_display)
}

/// Set the emsg_on_display flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_emsg_on_display(val: c_int) {
    emsg_on_display = (val) != 0;
}

/// Check if file info is needed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_need_fileinfo() -> c_int {
    c_int::from(need_fileinfo)
}

/// Set the need_fileinfo flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_need_fileinfo(val: c_int) {
    need_fileinfo = (val) != 0;
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
    let scrolled_ign = msg_scrolled_ign;

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
    let ll = lines_left;
    let no_more = msg_no_more;

    c_int::from(ll == 0 && !no_more)
}

/// Decrement lines_left counter and return true if more prompt needed.
///
/// # Safety
/// Calls C accessor and mutator functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_dec_lines_left() -> c_int {
    let ll = lines_left;
    if ll > 0 {
        lines_left = ll - 1;
    }
    rs_msg_need_more()
}

// ============================================================================
// Phase 425: Wait/Return Prompt Functions
// ============================================================================

extern "C" {
    // Wait return functions
    fn wait_return(redraw: c_int);
    static mut no_wait_return: c_int;
    fn nvim_get_vgetc_busy() -> c_int;
}

/// Wait for the user to press a key and optionally redraw.
///
/// This is the main function for "Press ENTER to continue" prompts.
///
/// # Arguments
/// * `redraw` - If true, redraw the screen after the wait
///
/// # Safety
/// Calls C function that blocks for user input.
#[no_mangle]
pub unsafe extern "C" fn rs_wait_return(redraw: c_int) {
    wait_return(redraw);
}

/// Wait for return with screen redraw.
///
/// Convenience wrapper that always redraws after waiting.
///
/// # Safety
/// Calls C function that blocks for user input.
#[no_mangle]
pub unsafe extern "C" fn rs_wait_return_redraw() {
    wait_return(1);
}

/// Wait for return without screen redraw.
///
/// Convenience wrapper that doesn't redraw after waiting.
///
/// # Safety
/// Calls C function that blocks for user input.
#[no_mangle]
pub unsafe extern "C" fn rs_wait_return_no_redraw() {
    wait_return(0);
}

/// Get the no_wait_return counter.
///
/// When > 0, wait_return() won't wait for a key.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_no_wait_return() -> c_int {
    no_wait_return
}

/// Increment no_wait_return to prevent waiting.
///
/// # Safety
/// Calls C function.
#[no_mangle]
pub unsafe extern "C" fn rs_no_wait_return_enter() {
    no_wait_return += 1;
}

/// Decrement no_wait_return.
///
/// # Safety
/// Calls C function.
#[no_mangle]
pub unsafe extern "C" fn rs_no_wait_return_leave() {
    if no_wait_return > 0 {
        no_wait_return -= 1;
    }
}

/// Check if waiting for return is currently blocked.
///
/// Returns true if any condition prevents wait_return from waiting:
/// - msg_silent is set
/// - vgetc_busy is set
/// - no_wait_return is set
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_wait_return_blocked() -> c_int {
    let vgetc_busy = nvim_get_vgetc_busy();
    c_int::from(msg_silent != 0 || vgetc_busy > 0 || no_wait_return > 0)
}

/// Check if wait_return should be called.
///
/// Returns true if need_wait_return is set and waiting is not blocked.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_should_wait_return() -> c_int {
    let need_wait = need_wait_return;
    let blocked = rs_wait_return_blocked();

    c_int::from(need_wait && blocked == 0)
}

/// Set need_wait_return flag.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_request_wait_return() {
    need_wait_return = true;
}

/// Clear need_wait_return flag.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_clear_wait_return() {
    need_wait_return = false;
}

/// Reset wait return state after handling.
///
/// Clears need_wait_return and msg_didout flags.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_reset_wait_return_state() {
    need_wait_return = false;
    msg_didout = false;
}

// ============================================================================
// Phase 5: msg_puts_len migrated to Rust
// ============================================================================

extern "C" {
    fn nvim_redir_write(str_: *const std::ffi::c_char, maxlen: isize);
    fn nvim_msg_hist_add_len(s: *const std::ffi::c_char, len: c_int, hl_id: c_int);
    fn msg_use_printf() -> c_int;
    fn nvim_msg_puts_printf(str_: *const std::ffi::c_char, len: isize);
    fn nvim_msg_puts_display(str_: *const std::ffi::c_char, len: c_int, hl_id: c_int);
    fn nvim_msg_show_empty();
    static mut headless_mode: bool;
    static mut default_grid: crate::ScreenGrid;
    static mut msg_col: c_int;
}

/// Write message string with highlight and redirection.
///
/// This is the core low-level output routine. It:
/// 1. Writes to any active redirection targets
/// 2. Returns early for silent/empty messages
/// 3. Optionally adds to message history
/// 4. Sets need_wait_return if scrolled
/// 5. Dispatches to printf or display rendering
///
/// # Arguments
/// * `str_` - The string to write (NUL-terminated, length bytes)
/// * `len` - Length of string, or -1 to use NUL-terminator
/// * `hl_id` - Highlight group ID (0 for default)
/// * `hist` - If true, add to message history
///
/// # Safety
/// - `str_` must be a valid pointer to at least `len` bytes (or NUL-terminated if len == -1)
#[export_name = "msg_puts_len"]
pub unsafe extern "C" fn rs_msg_puts_len(
    str_: *const std::ffi::c_char,
    len: isize,
    hl_id: c_int,
    hist: bool,
) {
    // If redirection is on, also write to the redirection file.
    nvim_redir_write(str_, len);

    // Don't print anything when using ":silent cmd" or empty message.
    let first_byte = *str_.cast::<u8>();
    if msg_silent != 0 || first_byte == 0 {
        if first_byte == 0 && nvim_ui_has_messages() != 0 {
            nvim_msg_show_empty();
        }
        return;
    }

    if hist {
        nvim_msg_hist_add_len(str_, c_int::try_from(len).unwrap_or(c_int::MAX), hl_id);
    }

    // When writing something to the screen after it has scrolled, requires a
    // wait-return prompt later.
    let overflow = nvim_ui_has_messages() == 0 && {
        let threshold = c_int::from(p_ch == 0);
        msg_scrolled > threshold
    };

    if overflow && !msg_scrolled_ign {
        // Check if str_ == "\r" - single CR character
        let is_cr_only = *str_.cast::<u8>() == b'\r'
            && (len < 0 || len == 1)
            && (len >= 0 || *str_.add(1).cast::<u8>() == 0);
        if !is_cr_only {
            need_wait_return = true;
        }
    }
    msg_didany = true; // remember that something was outputted

    if msg_use_printf() != 0 {
        let saved_msg_col = msg_col;
        nvim_msg_puts_printf(str_, len);
        if headless_mode {
            msg_col = saved_msg_col;
        }
    }
    if msg_use_printf() == 0 || (headless_mode && !default_grid.chars.is_null()) {
        nvim_msg_puts_display(str_, c_int::try_from(len).unwrap_or(c_int::MAX), hl_id);
    }

    need_fileinfo = false;
}

#[cfg(test)]
mod tests {
    // Integration tests would require mocking C functions
}
