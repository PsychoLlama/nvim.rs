//! Change recording functions for buffer modification tracking.
//!
//! This module provides functions to mark buffers as modified and handle
//! readonly file warnings.

use std::ffi::{c_char, c_int, c_long};

use crate::{BufHandle, HLF_W, VV_WARNINGMSG};

/// UIExtension value for kUIMessages (ui_defs.h)
const K_UI_MESSAGES: c_int = 4;

// =============================================================================
// C Accessor Functions (extern declarations)
// =============================================================================

extern "C" {
    static Rows: c_int;
    static mut redraw_tabline: bool;
    static mut need_maketitle: bool;
}

#[allow(dead_code)]
extern "C" {
    // Buffer field accessors
    fn nvim_buf_get_b_changed(buf: BufHandle) -> bool;
    fn nvim_buf_set_b_changed(buf: BufHandle, val: bool);
    fn nvim_buf_get_b_changed_invalid(buf: BufHandle) -> bool;
    fn nvim_buf_set_b_changed_invalid(buf: BufHandle, val: bool);
    fn nvim_buf_get_b_did_warn(buf: BufHandle) -> bool;
    fn nvim_buf_set_b_did_warn(buf: BufHandle, val: bool);
    fn nvim_buf_get_b_p_ro(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_ro_locked(buf: BufHandle) -> c_int;
    fn nvim_buf_set_b_ro_locked(buf: BufHandle, val: c_int);
    fn nvim_buf_get_b_may_swap(buf: BufHandle) -> bool;
    fn nvim_bt_dontwrite(buf: BufHandle) -> bool;

    // Global state accessors
    fn nvim_get_curbuf() -> BufHandle;
    fn nvim_get_autocmd_busy() -> bool;
    fn nvim_get_highlight_match() -> c_int;
    fn nvim_set_highlight_match(val: c_int);
    fn nvim_curbufIsChanged() -> c_int;

    // Message functions
    fn nvim_msg_start();
    fn nvim_msg_source(attr: c_int);
    fn nvim_msg_ext_set_kind(kind: *const c_char);
    fn nvim_msg_puts_hl(msg: *const c_char, attr: c_int, right: bool);
    fn nvim_msg_clr_eos();
    fn nvim_msg_end();
    fn nvim_msg_silent() -> c_int;
    fn nvim_silent_mode() -> bool;
    fn nvim_ui_active() -> bool;
    fn ui_has(ext: c_int) -> bool;
    fn nvim_set_vim_var_string(idx: c_int, val: *const c_char, len: c_int);

    // Redraw functions
    fn nvim_redraw_buf_status_later(buf: BufHandle);
    fn nvim_set_redraw_cmdline(val: bool);

    // Other functions
    fn nvim_ml_setflags(buf: BufHandle);
    fn nvim_ml_open_file(buf: BufHandle);
    fn nvim_buf_inc_changedtick(buf: BufHandle);
    fn nvim_apply_autocmds_filechangedro(buf: BufHandle);
    fn nvim_showmode();
    fn nvim_ui_flush();
    fn nvim_os_delay(ms: c_long, allow_input: bool);
    fn nvim_wait_return(redraw: bool);
    static mut msg_scroll: c_int;
    static mut need_wait_return: bool;
    static mut emsg_silent: c_int;
    fn nvim_in_assert_fails() -> bool;
    static mut msg_row: c_int;
    static mut msg_col: c_int;

    // Gettext function
    fn nvim_gettext(s: *const c_char) -> *const c_char;
}

// =============================================================================
// Warning Message
// =============================================================================

/// Warning message for changing readonly file.
const W10_WARNING: &[u8] = b"W10: Warning: Changing a readonly file\0";

// =============================================================================
// Change Recording Functions
// =============================================================================

/// Internal part of changed(), no user interaction.
///
/// Also used for recovery. This marks the buffer as changed and updates
/// the UI elements that reflect buffer state.
fn changed_internal_impl(buf: BufHandle) {
    // SAFETY: All accessors are safe C functions
    unsafe {
        nvim_buf_set_b_changed(buf, true);
        nvim_buf_set_b_changed_invalid(buf, true);
        nvim_ml_setflags(buf);
        nvim_redraw_buf_status_later(buf);
        redraw_tabline = true;
        need_maketitle = true;
    }
}

/// FFI wrapper for `changed_internal`.
///
/// Internal part of changed(), no user interaction.
/// Also used for recovery.
#[export_name = "changed_internal"]
pub extern "C" fn rs_changed_internal(buf: BufHandle) {
    changed_internal_impl(buf);
}

/// Give a warning when changing a readonly file.
///
/// If the file is readonly, give a warning message with the first change.
/// Don't do this for autocommands.
/// Doesn't use emsg(), because it flushes the macro buffer.
/// If we have undone all changes b_changed will be false, but "b_did_warn"
/// will be true.
///
/// "col" is the column for the message; non-zero when in insert mode and
/// 'showmode' is on.
///
/// Careful: may trigger autocommands that reload the buffer.
fn change_warning_impl(buf: BufHandle, col: c_int) {
    // SAFETY: All accessors are safe C functions
    unsafe {
        // Check if we need to show a warning
        if nvim_buf_get_b_did_warn(buf) {
            return;
        }
        if nvim_curbufIsChanged() != 0 {
            return;
        }
        if nvim_get_autocmd_busy() {
            return;
        }
        if nvim_buf_get_b_p_ro(buf) == 0 {
            return;
        }

        // Increment ro_locked to prevent changes during autocmd
        let ro_locked = nvim_buf_get_b_ro_locked(buf);
        nvim_buf_set_b_ro_locked(buf, ro_locked + 1);

        // Trigger FileChangedRO autocmd
        nvim_apply_autocmds_filechangedro(buf);

        // Decrement ro_locked
        nvim_buf_set_b_ro_locked(buf, ro_locked);

        // Check if autocmd cleared the readonly flag
        if nvim_buf_get_b_p_ro(buf) == 0 {
            return;
        }

        // Do what msg() does, but with a column offset if the warning should
        // be after the mode message.
        nvim_msg_start();

        if msg_row == Rows - 1 {
            msg_col = col;
        }

        nvim_msg_source(HLF_W);
        nvim_msg_ext_set_kind(c"wmsg".as_ptr());

        // Get the translated warning message
        let warning_msg = nvim_gettext(W10_WARNING.as_ptr().cast());
        nvim_msg_puts_hl(warning_msg, HLF_W, true);
        nvim_set_vim_var_string(VV_WARNINGMSG, warning_msg, -1);

        nvim_msg_clr_eos();
        nvim_msg_end();

        // Give the user time to read the message
        if nvim_msg_silent() == 0
            && !nvim_silent_mode()
            && nvim_ui_active()
            && !ui_has(K_UI_MESSAGES)
        {
            nvim_ui_flush();
            nvim_os_delay(1002, true);
        }

        nvim_buf_set_b_did_warn(buf, true);
        nvim_set_redraw_cmdline(false); // don't redraw and erase the message

        if msg_row < Rows - 1 {
            nvim_showmode();
        }
    }
}

/// FFI wrapper for `change_warning`.
///
/// If the file is readonly, give a warning message with the first change.
#[export_name = "change_warning"]
pub extern "C" fn rs_change_warning(buf: BufHandle, col: c_int) {
    change_warning_impl(buf, col);
}

/// Call this function when something in a buffer is changed.
///
/// Most often called through changed_bytes() and changed_lines(), which also
/// mark the area of the display to be redrawn.
///
/// Careful: may trigger autocommands that reload the buffer.
fn changed_impl(buf: BufHandle) {
    // SAFETY: All accessors are safe C functions
    unsafe {
        if !nvim_buf_get_b_changed(buf) {
            let save_msg_scroll = msg_scroll;

            // Give a warning about changing a read-only file.  This may also
            // check-out the file, thus change "curbuf"!
            change_warning_impl(buf, 0);

            // Create a swap file if that is wanted.
            // Don't do this for "nofile" and "nowrite" buffer types.
            if nvim_buf_get_b_may_swap(buf) && !nvim_bt_dontwrite(buf) {
                let save_need_wait_return = need_wait_return;

                need_wait_return = false;
                nvim_ml_open_file(buf);

                // The ml_open_file() can cause an ATTENTION message.
                // Wait two seconds, to make sure the user reads this unexpected
                // message.  Since we could be anywhere, call wait_return() now,
                // and don't let the emsg() set msg_scroll.
                if need_wait_return
                    && emsg_silent == 0
                    && !nvim_in_assert_fails()
                    && !ui_has(K_UI_MESSAGES)
                {
                    nvim_ui_flush();
                    nvim_os_delay(2002, true);
                    nvim_wait_return(true);
                    msg_scroll = save_msg_scroll;
                } else {
                    need_wait_return = save_need_wait_return;
                }
            }
            changed_internal_impl(buf);
        }

        nvim_buf_inc_changedtick(buf);

        // If a pattern is highlighted, the position may now be invalid.
        nvim_set_highlight_match(0);
    }
}

/// FFI wrapper for `changed`.
///
/// Call this function when something in a buffer is changed.
/// Most often called through changed_bytes() and changed_lines().
#[export_name = "changed"]
pub extern "C" fn rs_changed(buf: BufHandle) {
    changed_impl(buf);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warning_message_is_null_terminated() {
        assert!(W10_WARNING.ends_with(&[0]));
    }
}
