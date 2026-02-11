//! Normal mode pre-iteration check.
//!
//! This module provides the Rust implementation of `normal_check()`
//! from `src/nvim/normal.c`. Runs before each normal mode iteration:
//! processes stuff buffer, handles interrupts, manages redraw, and
//! handles ex mode / command-line window exits.

use std::ffi::c_int;

use crate::dispatch::types::NormalStateHandle;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // NormalState accessors
    fn nvim_ns_get_cmdwin(s: NormalStateHandle) -> bool;
    fn nvim_ns_get_noexmode(s: NormalStateHandle) -> bool;

    // Static helper wrappers (all take void* for NormalState)
    fn nvim_normal_check_stuff_buffer_wrapper(s: NormalStateHandle);
    fn nvim_normal_check_interrupt_wrapper(s: NormalStateHandle);
    fn nvim_normal_check_cursor_moved_wrapper(s: NormalStateHandle);
    fn nvim_normal_check_text_changed_wrapper(s: NormalStateHandle);
    fn nvim_normal_check_window_scrolled_wrapper(s: NormalStateHandle);
    fn nvim_normal_check_buffer_modified_wrapper(s: NormalStateHandle);
    fn nvim_normal_check_safe_state_wrapper(s: NormalStateHandle);
    fn nvim_normal_check_folds_wrapper(s: NormalStateHandle);
    fn nvim_normal_redraw_wrapper(s: NormalStateHandle);
    fn nvim_normal_prepare_wrapper(s: NormalStateHandle);

    // Global accessors
    fn nvim_get_did_throw_direct() -> bool;
    fn nvim_get_ex_normal_busy() -> c_int;
    fn nvim_get_exmode_active() -> bool;
    fn nvim_set_msg_scroll(val: c_int);
    fn nvim_set_quit_more(val: bool);
    fn nvim_get_skip_redraw() -> bool;
    fn nvim_set_skip_redraw(val: bool);
    fn nvim_get_do_redraw() -> c_int;
    fn nvim_set_do_redraw(val: bool);
    fn nvim_get_diff_need_scrollbind() -> bool;
    fn nvim_set_diff_need_scrollbind(val: bool);
    fn nvim_get_time_fd_not_null() -> bool;
    fn nvim_get_cmdwin_result() -> c_int;
    fn nvim_set_may_garbage_collect(val: bool);
    fn nvim_stuff_empty() -> bool;

    // Function wrappers
    fn nvim_discard_current_exception_wrapper();
    fn nvim_state_no_longer_safe();
    fn nvim_setcursor_wrapper();
    fn nvim_update_topline_curwin_wrapper();
    fn nvim_validate_cursor();
    fn nvim_curtab_needs_diff_update() -> bool;
    fn nvim_ex_diffupdate_wrapper();
    fn nvim_curtab_clear_diff_update();
    fn nvim_check_scrollbind_zero_wrapper();
    fn nvim_time_msg_first_screen_and_finish();
    fn nvim_may_make_initial_scroll_size_snapshot_wrapper();
    fn nvim_update_curswant_wrapper();
    fn nvim_do_exmode_wrapper();
}

/// Pre-iteration check for normal mode.
///
/// Returns:
/// -  `1` if the iteration should continue normally
/// - `-1` if the iteration should be skipped (ex mode ran)
/// -  `0` if the main loop must exit
///
/// # Safety
/// `s` must be a valid NormalState pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_normal_check(s: NormalStateHandle) -> c_int {
    nvim_normal_check_stuff_buffer_wrapper(s);
    nvim_normal_check_interrupt_wrapper(s);

    // At the toplevel there is no exception handling. Discard any that
    // may be hanging around (e.g. from "interrupt" at the debug prompt).
    if nvim_get_did_throw_direct() && nvim_get_ex_normal_busy() == 0 {
        nvim_discard_current_exception_wrapper();
    }

    if !nvim_get_exmode_active() {
        nvim_set_msg_scroll(0);
    }
    nvim_set_quit_more(false);

    nvim_state_no_longer_safe();

    // If skip redraw is set (for ":" in wait_return()), don't redraw now.
    // If there is nothing in the stuff_buffer or do_redraw is true,
    // update cursor and redraw.
    if nvim_get_skip_redraw() || nvim_get_exmode_active() {
        nvim_set_skip_redraw(false);
        nvim_setcursor_wrapper();
    } else if nvim_get_do_redraw() != 0 || nvim_stuff_empty() {
        // Ensure curwin->w_topline and curwin->w_leftcol are up to date
        // before triggering a WinScrolled autocommand.
        nvim_update_topline_curwin_wrapper();
        nvim_validate_cursor();

        nvim_normal_check_cursor_moved_wrapper(s);
        nvim_normal_check_text_changed_wrapper(s);
        nvim_normal_check_window_scrolled_wrapper(s);
        nvim_normal_check_buffer_modified_wrapper(s);
        nvim_normal_check_safe_state_wrapper(s);

        // Updating diffs from changed() does not always work properly,
        // esp. updating folds. Do an update just before redrawing if needed.
        if nvim_curtab_needs_diff_update() {
            nvim_ex_diffupdate_wrapper();
            nvim_curtab_clear_diff_update();
        }

        // Scroll-binding for diff mode may have been postponed until
        // here. Avoids doing it for every change.
        if nvim_get_diff_need_scrollbind() {
            nvim_check_scrollbind_zero_wrapper();
            nvim_set_diff_need_scrollbind(false);
        }

        nvim_normal_check_folds_wrapper(s);
        nvim_normal_redraw_wrapper(s);
        nvim_set_do_redraw(false);

        // Now that we have drawn the first screen all the startup stuff
        // has been done, close any file for startup messages.
        if nvim_get_time_fd_not_null() {
            nvim_time_msg_first_screen_and_finish();
        }
        // After the first screen update may start triggering WinScrolled
        // autocmd events. Store all the scroll positions and sizes now.
        nvim_may_make_initial_scroll_size_snapshot_wrapper();
    }

    // May perform garbage collection when waiting for a character, but
    // only at the very toplevel.
    nvim_set_may_garbage_collect(!nvim_ns_get_cmdwin(s) && !nvim_ns_get_noexmode(s));

    // Update w_curswant if w_set_curswant has been set.
    nvim_update_curswant_wrapper();

    if nvim_get_exmode_active() {
        if nvim_ns_get_noexmode(s) {
            return 0;
        }
        nvim_do_exmode_wrapper();
        return -1;
    }

    if nvim_ns_get_cmdwin(s) && nvim_get_cmdwin_result() != 0 {
        return 0;
    }

    nvim_normal_prepare_wrapper(s);
    1
}
