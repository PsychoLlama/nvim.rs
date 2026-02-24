//! Normal mode pre-iteration check.
//!
//! This module provides the Rust implementation of `normal_check()`
//! from `src/nvim/normal.c`. Runs before each normal mode iteration:
//! processes stuff buffer, handles interrupts, manages redraw, and
//! handles ex mode / command-line window exits.

use std::ffi::c_int;

use crate::dispatch::types::NormalStateHandle;
use crate::WinHandle;

// =============================================================================
// Constants (verified with _Static_assert in normal.c)
// =============================================================================

const MODE_NORMAL: c_int = 0x01;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // NormalState accessors
    fn nvim_ns_get_cmdwin(s: NormalStateHandle) -> bool;
    fn nvim_ns_get_noexmode(s: NormalStateHandle) -> bool;
    fn nvim_ns_get_previous_got_int(s: NormalStateHandle) -> bool;
    fn nvim_ns_set_previous_got_int(s: NormalStateHandle, val: bool);

    // Global accessors
    fn nvim_get_did_throw_direct() -> bool;
    fn nvim_get_ex_normal_busy() -> c_int;
    fn nvim_get_exmode_active() -> bool;
    fn nvim_set_exmode_active(val: bool);
    fn nvim_set_msg_scroll(val: c_int);
    fn nvim_set_quit_more(val: bool);
    fn nvim_get_quit_more() -> bool;
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
    fn nvim_get_finish_op() -> c_int;
    fn nvim_get_got_int() -> c_int;
    fn nvim_set_got_int(val: c_int);
    fn nvim_get_global_busy() -> bool;
    fn nvim_set_did_check_timestamps(val: bool);
    fn nvim_get_need_check_timestamps() -> bool;
    fn nvim_get_need_wait_return() -> c_int;
    fn nvim_wait_return(redraw: bool);
    fn nvim_get_restart_edit() -> c_int;

    // Function wrappers
    fn nvim_discard_current_exception_wrapper();
    fn nvim_state_no_longer_safe();
    fn nvim_setcursor_wrapper();
    fn nvim_update_topline_curwin_wrapper();
    fn nvim_validate_cursor();
    fn nvim_curtab_needs_diff_update() -> bool;
    fn rs_diff_ex_diffupdate(eap: *mut std::ffi::c_void);
    fn nvim_curtab_clear_diff_update();
    fn nvim_check_scrollbind_zero_wrapper();
    fn nvim_time_msg_first_screen_and_finish();
    fn nvim_may_make_initial_scroll_size_snapshot_wrapper();
    fn nvim_update_curswant_wrapper();
    fn nvim_do_exmode_wrapper();
    fn nvim_check_timestamps_call(focus: bool);
    fn nvim_may_trigger_win_scrolled_resized_call();
    fn nvim_may_trigger_safestate_call(safe: bool);
    fn nvim_char_avail_call() -> bool;
    fn nvim_fdo_has_all_flag() -> bool;
    fn nvim_vgetc_and_discard();
    fn nvim_set_State(val: c_int);
    fn rs_op_pending() -> bool;

    // Fold functions (from fold crate)
    fn rs_foldAdjustVisual();
    fn rs_hasAnyFolding(win: WinHandle) -> c_int;
    fn rs_foldCheckClose();
    fn rs_foldOpenCursor();

    // Phase 5 autocmd-check accessors
    fn nvim_has_event_cursormoved() -> bool;
    fn nvim_last_cursormoved_check() -> bool;
    fn nvim_apply_autocmds_cursormoved();
    fn nvim_update_last_cursormoved();
    fn nvim_has_event_textchanged() -> bool;
    fn nvim_curbuf_changedtick_changed() -> bool;
    fn nvim_apply_autocmds_textchanged();
    fn nvim_curbuf_update_last_changedtick();
    fn nvim_has_event_bufmodifiedset() -> bool;
    fn nvim_curbuf_b_changed_invalid_get() -> bool;
    fn nvim_apply_autocmds_bufmodifiedset();
    fn nvim_curbuf_b_changed_invalid_clear();

    fn nvim_normal_redraw_impl(s: NormalStateHandle);
    fn nvim_normal_prepare_wrapper(s: NormalStateHandle);
    fn nvim_get_curwin() -> WinHandle;
}

// =============================================================================
// Inlined Phase 4 helper implementations
// =============================================================================

/// Inline of normal_check_stuff_buffer.
unsafe fn normal_check_stuff_buffer(s: NormalStateHandle) {
    let _ = s; // s is not used in the body
    if nvim_stuff_empty() {
        nvim_set_did_check_timestamps(false);

        if nvim_get_need_check_timestamps() {
            nvim_check_timestamps_call(false);
        }

        if nvim_get_need_wait_return() != 0 {
            // if wait_return still needed call it now
            nvim_wait_return(false);
        }
    }
}

/// Inline of normal_check_interrupt.
unsafe fn normal_check_interrupt(s: NormalStateHandle) {
    if nvim_get_got_int() != 0 {
        if nvim_ns_get_noexmode(s)
            && nvim_get_global_busy()
            && !nvim_get_exmode_active()
            && nvim_ns_get_previous_got_int(s)
        {
            // Typed two CTRL-C in a row: go back to ex mode as if "Q" was
            // used and keep "got_int" set, so that it aborts ":g".
            nvim_set_exmode_active(true);
            nvim_set_State(MODE_NORMAL);
        } else if !nvim_get_global_busy() || !nvim_get_exmode_active() {
            if !nvim_get_quit_more() {
                // flush all buffers
                nvim_vgetc_and_discard();
            }
            nvim_set_got_int(0);
        }
        nvim_ns_set_previous_got_int(s, true);
    } else {
        nvim_ns_set_previous_got_int(s, false);
    }
}

/// Inline of normal_check_window_scrolled.
unsafe fn normal_check_window_scrolled(_s: NormalStateHandle) {
    if nvim_get_finish_op() == 0 {
        nvim_may_trigger_win_scrolled_resized_call();
    }
}

/// Inline of normal_check_safe_state.
unsafe fn normal_check_safe_state(_s: NormalStateHandle) {
    nvim_may_trigger_safestate_call(!rs_op_pending() && nvim_get_restart_edit() == 0);
}

/// Inline of normal_check_cursor_moved.
unsafe fn normal_check_cursor_moved() {
    if nvim_get_finish_op() == 0
        && nvim_has_event_cursormoved()
        && nvim_last_cursormoved_check()
    {
        nvim_apply_autocmds_cursormoved();
        nvim_update_last_cursormoved();
    }
}

/// Inline of normal_check_text_changed.
unsafe fn normal_check_text_changed() {
    if nvim_get_finish_op() == 0
        && nvim_has_event_textchanged()
        && nvim_curbuf_changedtick_changed()
    {
        nvim_apply_autocmds_textchanged();
        nvim_curbuf_update_last_changedtick();
    }
}

/// Inline of normal_check_buffer_modified.
unsafe fn normal_check_buffer_modified() {
    if nvim_get_finish_op() == 0
        && nvim_has_event_bufmodifiedset()
        && nvim_curbuf_b_changed_invalid_get()
    {
        nvim_apply_autocmds_bufmodifiedset();
        nvim_curbuf_b_changed_invalid_clear();
    }
}

/// Inline of normal_check_folds.
unsafe fn normal_check_folds(_s: NormalStateHandle) {
    // Include a closed fold completely in the Visual area.
    rs_foldAdjustVisual();

    // When 'foldclose' is set, apply 'foldlevel' to folds that don't
    // contain the cursor.
    // When 'foldopen' is "all", open the fold(s) under the cursor.
    // This may mark the window for redrawing.
    if rs_hasAnyFolding(nvim_get_curwin()) != 0 && !nvim_char_avail_call() {
        rs_foldCheckClose();

        if nvim_fdo_has_all_flag() {
            rs_foldOpenCursor();
        }
    }
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
    normal_check_stuff_buffer(s);
    normal_check_interrupt(s);

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

        normal_check_cursor_moved();
        normal_check_text_changed();
        normal_check_window_scrolled(s);
        normal_check_buffer_modified();
        normal_check_safe_state(s);

        // Updating diffs from changed() does not always work properly,
        // esp. updating folds. Do an update just before redrawing if needed.
        if nvim_curtab_needs_diff_update() {
            rs_diff_ex_diffupdate(std::ptr::null_mut());
            nvim_curtab_clear_diff_update();
        }

        // Scroll-binding for diff mode may have been postponed until
        // here. Avoids doing it for every change.
        if nvim_get_diff_need_scrollbind() {
            nvim_check_scrollbind_zero_wrapper();
            nvim_set_diff_need_scrollbind(false);
        }

        normal_check_folds(s);
        nvim_normal_redraw_impl(s);
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
