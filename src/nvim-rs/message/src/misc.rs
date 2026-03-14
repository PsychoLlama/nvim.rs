//! Miscellaneous message functions
//!
//! Provides Rust implementations for various message-related utilities
//! that don't fit neatly into other modules.

use std::ffi::{c_char, c_int};

// ============================================================================
// C Function Declarations
// ============================================================================

extern "C" {
    // Home directory handling (Phase 77: now implemented in Rust)
    fn nvim_home_replace_save_null(fname: *const c_char) -> *mut c_char;
    fn nvim_xfree(ptr: *mut c_char);
    fn msg_outtrans(str: *const c_char, hl_id: c_int, hist: bool) -> c_int;

    // For msg_outtrans_long (Phase 80)
    fn msg_outtrans_len(msgstr: *const c_char, len: c_int, hl_id: c_int, hist: bool) -> c_int;
    fn msg_puts_hl(s: *const c_char, hl_id: c_int, hist: bool);
    fn nvim_get_columns() -> c_int;
    fn nvim_get_msg_col() -> c_int;

    // Note: msg_source is wrapped in error.rs

    // UI refresh (Phase 1: implementations moved to Rust, C provides accessor wrappers)
    fn nvim_msg_ui_refresh_impl();
    fn nvim_msg_ui_flush_impl();

    // Phase 4: msg_scroll_flush accessors
    fn nvim_msg_grid_is_throttled() -> c_int;
    fn nvim_msg_grid_set_throttled(val: c_int);
    fn nvim_get_msg_grid_pos_at_flush() -> c_int;
    fn nvim_set_msg_grid_pos_at_flush(val: c_int);
    fn nvim_get_msg_grid_pos() -> c_int;
    fn nvim_get_msg_scrolled() -> c_int;
    fn nvim_get_msg_scrolled_at_flush() -> c_int;
    fn nvim_set_msg_scrolled_at_flush(val: c_int);
    fn nvim_get_msg_grid_scroll_discount() -> c_int;
    fn nvim_set_msg_grid_scroll_discount(val: c_int);
    fn nvim_msg_set_pos_for_scroll(pos: c_int);
    fn nvim_msg_grid_scroll_up(to_scroll: c_int);
    fn nvim_get_rows() -> c_int;
    fn nvim_msg_grid_get_rows() -> c_int;
    fn nvim_msg_grid_flush_dirty_line(row: c_int);

    // Phase 1: message_filtered C implementation
    fn nvim_message_filtered_impl(msg: *const c_char) -> bool;

    // For msg_clr_cmdline
    fn nvim_get_cmdline_row() -> c_int;
    fn nvim_set_msg_row(val: c_int);
    fn nvim_set_msg_col(val: c_int);

    // Cursor positioning
    fn msg_cursor_goto(row: c_int, col: c_int);

    // State accessors
    fn nvim_get_msg_silent() -> c_int;
    fn nvim_get_emsg_on_display() -> c_int;
    fn nvim_set_emsg_on_display(val: c_int);
    fn nvim_get_msg_scroll() -> c_int;
    fn nvim_set_msg_scroll(val: c_int);
    fn nvim_get_did_wait_return() -> c_int;
    fn nvim_get_emsg_silent() -> c_int;
    // nvim_get_in_assert_fails returns bool (defined in normal_shim.c)
    fn nvim_get_in_assert_fails() -> bool;
    fn nvim_ui_has_messages() -> c_int;
    // nvim_ui_flush is defined in change_ffi.c
    fn nvim_ui_flush();
    // nvim_os_delay is defined in change_ffi.c (long ms, bool allow_input)
    fn nvim_os_delay(ms: std::ffi::c_long, allow_input: bool);

    // keep_msg raw setters (nvim_set_keep_msg_raw in message.c does xfree+xstrdup)
    fn nvim_set_keep_msg_raw(s: *const c_char);
    fn nvim_set_keep_msg_more(val: c_int);
    fn nvim_set_keep_msg_hl_id(val: c_int);

    // For messaging()
    fn nvim_get_p_lz() -> c_int;
    fn nvim_char_avail() -> c_int;
    fn nvim_get_key_typed() -> c_int;
    fn nvim_get_p_ch() -> i64;

    // For msg_make
    fn skipwhite(s: *const c_char) -> *mut c_char;
    fn msg_putchar(c: c_int);

    // For messagesopt_changed (Phase 86)
    fn nvim_get_p_mopt() -> *const c_char;
    fn strnequal(s1: *const c_char, s2: *const c_char, n: usize) -> bool;
    fn getdigits_int(pp: *mut *mut c_char, strict: bool, def: c_int) -> c_int;
    fn nvim_set_msg_flags(val: c_int);
    fn nvim_set_msg_wait(val: c_int);
    fn nvim_set_msg_hist_max(val: c_int);
    fn msg_hist_clear(keep: c_int);
}

// ============================================================================
// Easter Egg
// ============================================================================

/// Show a special message if the argument matches a secret phrase.
///
/// # Safety
/// - `arg` must be a valid NUL-terminated C string
#[export_name = "msg_make"]
pub unsafe extern "C" fn rs_msg_make(arg: *const c_char) {
    const STR: &[u8] = b"eeffoc";
    const RS: &[u8] = b"Plon#dqg#vxjduB";

    let arg = skipwhite(arg).cast_const();
    let mut idx: usize = 5;
    let mut p = arg;
    let mut matched = true;
    loop {
        if *p == 0 {
            break;
        }
        // Compare *p (i8) with STR[idx] (u8) — STR chars are all ASCII (<128)
        #[allow(clippy::cast_possible_wrap)]
        if *p != STR[idx] as i8 {
            matched = false;
            break;
        }
        p = p.add(1);
        if idx == 0 {
            break;
        }
        idx -= 1;
    }
    if matched && idx == 0 && *p == 0 {
        msg_putchar(c_int::from(b'\n'));
        for &b in RS {
            msg_putchar(c_int::from(b) - 3);
        }
    }
}

// ============================================================================
// Output Translation Functions
// ============================================================================

// Note: rs_msg_outtrans() and rs_msg_outtrans_len() are defined in format.rs

/// Highlight face for special characters (HLF_8 = 8)
const HLF_8: c_int = 8;

/// Output a potentially long string with truncation.
///
/// If the string is too long for the screen, shows "..." at the middle.
/// Truncates by showing the start and end with "..." in the middle.
///
/// # Arguments
/// * `longstr` - The string to output
/// * `hl_id` - Highlight group ID
///
/// # Safety
/// - `longstr` must be a valid NUL-terminated C string
#[export_name = "msg_outtrans_long"]
pub unsafe extern "C" fn rs_msg_outtrans_long(longstr: *const c_char, hl_id: c_int) {
    // Calculate strlen
    let mut p = longstr;
    while *p != 0 {
        p = p.offset(1);
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let len = (p as usize - longstr as usize) as c_int;
    let mut slen = len;
    let room = nvim_get_columns() - nvim_get_msg_col();
    if nvim_ui_has_messages() == 0 && len > room && room >= 20 {
        slen = (room - 3) / 2;
        msg_outtrans_len(longstr, slen, hl_id, false);
        msg_puts_hl(c"...".as_ptr(), HLF_8, false);
    }
    msg_outtrans_len(longstr.offset((len - slen) as isize), slen, hl_id, false);
}

// ============================================================================
// Path Display Functions
// ============================================================================

/// Display a filename with home directory replaced by ~.
///
/// Replaces the home directory prefix with ~ and outputs with highlight 0.
///
/// # Arguments
/// * `fname` - The filename to display
///
/// # Safety
/// - `fname` must be a valid NUL-terminated C string
#[export_name = "msg_home_replace"]
pub unsafe extern "C" fn rs_msg_home_replace(fname: *const c_char) {
    rs_msg_home_replace_hl(fname, 0);
}

/// Display a filename with home directory replaced by ~ and given highlight.
///
/// # Arguments
/// * `fname` - The filename to display
/// * `hl_id` - Highlight group ID
///
/// # Safety
/// - `fname` must be a valid NUL-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_msg_home_replace_hl(fname: *const c_char, hl_id: c_int) {
    let name = nvim_home_replace_save_null(fname);
    msg_outtrans(name.cast_const(), hl_id, false);
    nvim_xfree(name);
}

// ============================================================================
// Source Location Functions
// ============================================================================

// Note: rs_msg_source() is defined in error.rs

// ============================================================================
// Prompt and Delay Functions
// ============================================================================

/// Check if a delay is needed before next message.
///
/// Used to ensure messages are visible before proceeding.
///
/// # Arguments
/// * `check_msg_scroll` - If true, also check msg_scroll state
///
/// # Safety
/// Calls C accessor functions and may block.
#[export_name = "msg_check_for_delay"]
pub unsafe extern "C" fn rs_msg_check_for_delay(check_msg_scroll: c_int) {
    let check = check_msg_scroll != 0;
    if (nvim_get_emsg_on_display() != 0 || (check && nvim_get_msg_scroll() != 0))
        && nvim_get_did_wait_return() == 0
        && nvim_get_emsg_silent() == 0
        && !nvim_get_in_assert_fails()
        && nvim_ui_has_messages() == 0
    {
        nvim_ui_flush();
        nvim_os_delay(1006, true);
        nvim_set_emsg_on_display(0);
        if check {
            nvim_set_msg_scroll(0);
        }
    }
}

// ============================================================================
// Keep Message Functions
// ============================================================================

/// Set the "keep_msg" string that is re-displayed after redraw.
///
/// Frees the old value. Skips when ext_messages UI is active.
/// Sets keep_msg_more to false and updates highlight.
///
/// # Arguments
/// * `s` - The message string, or NULL to clear
/// * `hl_id` - Highlight group ID for the message
///
/// # Safety
/// Calls C accessor/mutator functions that manage allocated memory.
#[export_name = "set_keep_msg"]
pub unsafe extern "C" fn rs_set_keep_msg(s: *const c_char, hl_id: c_int) {
    // Kept message is not cleared and re-emitted with ext_messages: #20416.
    if nvim_ui_has_messages() != 0 {
        return;
    }

    if s.is_null() || nvim_get_msg_silent() != 0 {
        nvim_set_keep_msg_raw(std::ptr::null());
    } else {
        nvim_set_keep_msg_raw(s);
    }
    nvim_set_keep_msg_more(0);
    nvim_set_keep_msg_hl_id(hl_id);
}

// ============================================================================
// UI Coordination Functions
// ============================================================================

/// Refresh the message area UI.
///
/// Calls ui_call_grid_resize and ui_ext_msg_set_pos when kUIMultigrid is active.
///
/// # Safety
/// Calls C accessor that modifies UI state.
#[export_name = "msg_ui_refresh"]
pub unsafe extern "C" fn rs_msg_ui_refresh() {
    nvim_msg_ui_refresh_impl();
}

/// Flush pending UI updates for messages.
///
/// Updates comp index position when kUIMultigrid is active.
///
/// # Safety
/// Calls C accessor that emits UI events.
#[export_name = "msg_ui_flush"]
pub unsafe extern "C" fn rs_msg_ui_flush() {
    nvim_msg_ui_flush_impl();
}

/// Flush scroll-related UI updates to clients.
///
/// Coalesces throttled message grid scrolling into a single grid_scroll
/// event per screen update.
///
/// # Panics
/// Panics if the scroll accounting invariants are violated (pos_delta or
/// to_scroll is negative), which indicates a bug in the scroll bookkeeping.
///
/// # Safety
/// Calls C accessor functions that modify UI state.
#[allow(clippy::cast_sign_loss)]
#[export_name = "msg_scroll_flush"]
pub unsafe extern "C" fn rs_msg_scroll_flush() {
    if nvim_msg_grid_is_throttled() != 0 {
        nvim_msg_grid_set_throttled(0);
        let pos_delta = nvim_get_msg_grid_pos_at_flush() - nvim_get_msg_grid_pos();
        assert!(pos_delta >= 0);
        let delta = (nvim_get_msg_scrolled() - nvim_get_msg_scrolled_at_flush())
            .min(nvim_msg_grid_get_rows());

        if pos_delta > 0 {
            nvim_msg_set_pos_for_scroll(nvim_get_msg_grid_pos());
        }

        let to_scroll = delta - pos_delta - nvim_get_msg_grid_scroll_discount();
        assert!(to_scroll >= 0);

        if to_scroll > 0 && nvim_get_msg_grid_pos() == 0 {
            nvim_msg_grid_scroll_up(to_scroll);
        }

        let rows = nvim_get_rows();
        let start = (rows - delta.max(1)).max(0);
        for i in start..rows {
            let row = i - nvim_get_msg_grid_pos();
            assert!(row >= 0);
            nvim_msg_grid_flush_dirty_line(row);
        }
    }
    nvim_set_msg_scrolled_at_flush(nvim_get_msg_scrolled());
    nvim_set_msg_grid_scroll_discount(0);
    nvim_set_msg_grid_pos_at_flush(nvim_get_msg_grid_pos());
}

// Note: rs_msg_reset_scroll() is defined in scrollback.rs

// ============================================================================
// Cursor Functions
// ============================================================================

/// Position the cursor in the message area.
///
/// # Arguments
/// * `row` - Target row
/// * `col` - Target column
///
/// # Safety
/// Calls C function that modifies display state.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_cursor_goto(row: c_int, col: c_int) {
    msg_cursor_goto(row, col);
}

// ============================================================================
// Clearing Functions
// ============================================================================

/// Clear the command line area.
///
/// # Safety
/// Calls C accessor functions that modify display state.
#[export_name = "msg_clr_cmdline"]
pub unsafe extern "C" fn rs_msg_clr_cmdline() {
    nvim_set_msg_row(nvim_get_cmdline_row());
    nvim_set_msg_col(0);
    crate::output_core::rs_msg_clr_eos_force_exported();
}

/// Force clear to end of screen even if not needed (rs_ alias).
///
/// # Safety
/// Delegates to the Rust msg_clr_eos_force implementation.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_clr_eos_force() {
    crate::output_core::rs_msg_clr_eos_force_exported();
}

// ============================================================================
// Message Enable Check
// ============================================================================

/// Return true if printing messages should currently be done.
///
/// Checks 'lazyredraw', character availability, and cmdheight/ext_messages.
///
/// # Safety
/// Calls C accessor functions.
#[export_name = "messaging"]
#[must_use]
pub unsafe extern "C" fn rs_messaging() -> bool {
    // TODO(bfredl): with general support for "async" messages with p_ch,
    // this should be re-enabled.
    !(nvim_get_p_lz() != 0 && nvim_char_avail() != 0 && nvim_get_key_typed() == 0)
        && (nvim_get_p_ch() > 0 || nvim_ui_has_messages() != 0)
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Check if message output should be suppressed.
///
/// Returns true if msg_silent is set.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_should_suppress() -> c_int {
    c_int::from(nvim_get_msg_silent() != 0)
}

// ============================================================================
// Messages Option Parsing (Phase 86)
// ============================================================================

// kOptMoptFlag constants (from build/src/nvim/auto/option_vars.generated.h)
const K_OPT_MOPT_FLAG_HIT_ENTER: c_int = 0x01;
const K_OPT_MOPT_FLAG_WAIT: c_int = 0x02;
const K_OPT_MOPT_FLAG_HISTORY: c_int = 0x04;

/// Parse and apply the 'messagesopt' option.
///
/// Returns OK (0) on success or FAIL (2) on error.
///
/// # Safety
/// Reads the global `p_mopt` option and calls C accessors.
#[export_name = "messagesopt_changed"]
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub unsafe extern "C" fn rs_messagesopt_changed() -> c_int {
    const OK: c_int = 1;
    const FAIL: c_int = 0;

    const OPT_HIT_ENTER: &[u8] = b"hit-enter";
    const OPT_WAIT: &[u8] = b"wait:";
    const OPT_HISTORY: &[u8] = b"history:";

    let mut messages_flags_new: c_int = 0;
    let mut messages_wait_new: c_int = 0;
    let mut messages_history_new: c_int = 0;

    let mut p: *mut c_char = nvim_get_p_mopt().cast_mut();

    while *p != 0 {
        if strnequal(p, OPT_HIT_ENTER.as_ptr().cast(), OPT_HIT_ENTER.len()) {
            p = p.add(OPT_HIT_ENTER.len());
            messages_flags_new |= K_OPT_MOPT_FLAG_HIT_ENTER;
        } else if strnequal(p, OPT_WAIT.as_ptr().cast(), OPT_WAIT.len())
            && (*p.add(OPT_WAIT.len()) as u8).is_ascii_digit()
        {
            p = p.add(OPT_WAIT.len());
            messages_wait_new = getdigits_int(std::ptr::addr_of_mut!(p), false, c_int::MAX);
            messages_flags_new |= K_OPT_MOPT_FLAG_WAIT;
        } else if strnequal(p, OPT_HISTORY.as_ptr().cast(), OPT_HISTORY.len())
            && (*p.add(OPT_HISTORY.len()) as u8).is_ascii_digit()
        {
            p = p.add(OPT_HISTORY.len());
            messages_history_new = getdigits_int(std::ptr::addr_of_mut!(p), false, c_int::MAX);
            messages_flags_new |= K_OPT_MOPT_FLAG_HISTORY;
        }

        if *p != b',' as c_char && *p != 0 {
            return FAIL;
        }
        if *p == b',' as c_char {
            p = p.add(1);
        }
    }

    // Either "wait" or "hit-enter" is required
    if messages_flags_new & (K_OPT_MOPT_FLAG_HIT_ENTER | K_OPT_MOPT_FLAG_WAIT) == 0 {
        return FAIL;
    }

    // "history" must be set
    if messages_flags_new & K_OPT_MOPT_FLAG_HISTORY == 0 {
        return FAIL;
    }

    // "history" must be <= 10000
    if messages_history_new > 10000 {
        return FAIL;
    }

    // "wait" must be <= 10000
    if messages_wait_new > 10000 {
        return FAIL;
    }

    nvim_set_msg_flags(messages_flags_new);
    nvim_set_msg_wait(messages_wait_new);
    nvim_set_msg_hist_max(messages_history_new);
    msg_hist_clear(messages_history_new);

    OK
}

// ============================================================================
// Filter Check Function (Phase 1)
// ============================================================================

/// Check if a message is filtered by the current `:filter` pattern.
///
/// Returns true when `:filter pattern` was used and `msg` does not match
/// `pattern` (or matches if `filter!` was used).
///
/// # Arguments
/// * `msg` - The message string to test against the filter
///
/// # Safety
/// - `msg` must be a valid NUL-terminated C string
/// - Calls C accessor that performs regex matching
#[must_use]
#[export_name = "message_filtered"]
pub unsafe extern "C" fn rs_message_filtered(msg: *const c_char) -> bool {
    nvim_message_filtered_impl(msg)
}

#[cfg(test)]
mod tests {
    // Integration tests would require mocking C functions
}
