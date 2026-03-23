//! Normal mode pre-iteration check.
//!
//! This module provides the Rust implementation of `normal_check()`
//! from `src/nvim/normal.c`. Runs before each normal mode iteration:
//! processes stuff buffer, handles interrupts, manages redraw, and
//! handles ex mode / command-line window exits.

use std::ffi::c_int;

use crate::dispatch::types::NormalStateHandle;
use crate::WinHandle;

extern "C" {
    static mut got_int: bool;
    static mut State: c_int;
    static must_redraw: c_int;
    static redraw_mode: c_int;
}

// =============================================================================
// Constants (verified with _Static_assert in normal.c)
// =============================================================================

const MODE_NORMAL: c_int = 0x01;
const MODE_NORMAL_BUSY: c_int = 0x1001;
const MODE_INSERT: c_int = 0x10;
const OP_NOP: c_int = 0;
const CA_COMMAND_BUSY: c_int = 1;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    static mut do_redraw: bool;
}

extern "C" {
    // NormalState accessors
    fn nvim_ns_get_cmdwin(s: NormalStateHandle) -> bool;
    fn nvim_ns_get_noexmode(s: NormalStateHandle) -> bool;
    fn nvim_ns_get_previous_got_int(s: NormalStateHandle) -> bool;
    fn nvim_ns_set_previous_got_int(s: NormalStateHandle, val: bool);
    fn nvim_ns_get_toplevel(s: NormalStateHandle) -> bool;
    fn nvim_ns_get_set_prevcount(s: NormalStateHandle) -> bool;
    fn nvim_ns_set_set_prevcount(s: NormalStateHandle, val: bool);
    fn nvim_ns_prepare_ca(s: NormalStateHandle);
    fn nvim_ns_set_mapped_len(s: NormalStateHandle, val: c_int);
    fn nvim_ns_get_ca_ptr(s: NormalStateHandle) -> *mut std::ffi::c_void;
    fn nvim_ns_get_oa_ptr(s: NormalStateHandle) -> *mut std::ffi::c_void;
    fn nvim_ns_get_old_pos_lnum(s: NormalStateHandle) -> c_int;
    fn nvim_ns_get_old_pos_col(s: NormalStateHandle) -> c_int;

    // oparg_T accessors
    fn nvim_oap_get_op_type_ptr(oap: *mut std::ffi::c_void) -> c_int;
    fn nvim_oap_get_regname_ptr(oap: *mut std::ffi::c_void) -> c_int;
    fn nvim_oap_get_prev_opcount_ptr(oap: *mut std::ffi::c_void) -> c_int;
    fn nvim_oap_get_prev_count0_ptr(oap: *mut std::ffi::c_void) -> c_int;
    fn nvim_oap_set_prev_opcount(oap: *mut std::ffi::c_void, val: c_int);
    fn nvim_oap_set_prev_count0(oap: *mut std::ffi::c_void, val: c_int);

    // cmdarg_T accessors
    fn nvim_cap_get_retval(cap: *mut std::ffi::c_void) -> c_int;
    fn nvim_cap_set_opcount(cap: *mut std::ffi::c_void, val: c_int);
    fn nvim_cap_set_count0(cap: *mut std::ffi::c_void, val: c_int);

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
    fn nvim_get_diff_need_scrollbind() -> bool;
    fn nvim_set_diff_need_scrollbind(val: bool);
    fn nvim_get_time_fd_not_null() -> bool;
    fn nvim_get_cmdwin_result() -> c_int;
    fn nvim_set_may_garbage_collect(val: bool);
    fn nvim_stuff_empty() -> bool;
    fn nvim_get_finish_op() -> c_int;
    fn nvim_set_finish_op(val: bool);
    fn nvim_get_global_busy() -> bool;
    fn nvim_set_did_check_timestamps(val: bool);
    fn nvim_get_need_check_timestamps() -> bool;
    fn nvim_get_need_wait_return() -> c_int;
    fn nvim_wait_return(redraw: bool);
    fn nvim_get_restart_edit() -> c_int;
    fn nvim_get_opcount() -> c_int;
    fn nvim_get_VIsual_active() -> c_int;
    fn nvim_get_KeyTyped() -> bool;

    // Message/display globals
    // nvim_get_p_smd: inlined (Phase 39, use p_smd directly)
    #[link_name = "p_smd"]
    static p_smd: c_int;
    fn nvim_get_msg_silent() -> c_int;
    fn nvim_get_clear_cmdline() -> bool;
    fn nvim_get_redraw_cmdline() -> bool;
    fn nvim_get_msg_didout() -> c_int; // defined in message.c
    fn nvim_get_msg_didany() -> c_int; // defined in message.c
    fn nvim_set_msg_didany(val: c_int); // defined in message.c
    fn nvim_get_msg_scroll_val() -> bool;
    fn nvim_set_msg_scroll_val(val: bool);
    fn nvim_get_msg_nowait_val() -> bool;
    fn nvim_get_emsg_on_display() -> c_int; // defined in message.c
    fn nvim_set_emsg_on_display(val: c_int); // defined in message.c
    fn nvim_get_emsg_silent() -> c_int;
    fn nvim_get_in_assert_fails() -> bool;
    fn nvim_get_did_wait_return_val() -> bool;
    fn nvim_set_did_emsg(val: c_int); // defined in message.c

    // Cursor globals
    fn nvim_get_cursor_lnum() -> c_int;
    fn nvim_get_cursor_col() -> c_int;

    // Redraw/display globals
    fn nvim_get_keep_msg_not_null() -> bool;
    fn nvim_get_need_fileinfo() -> c_int; // defined in message.c
    fn nvim_set_need_fileinfo(val: c_int); // defined in message.c

    // Function wrappers
    fn discard_current_exception();
    fn nvim_state_no_longer_safe();
    fn nvim_setcursor_wrapper();
    fn nvim_update_topline_call();
    fn nvim_validate_cursor();
    fn nvim_curtab_needs_diff_update() -> bool;
    #[link_name = "ex_diffupdate"]
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
    fn rs_op_pending() -> bool;
    fn nvim_ui_cursor_shape_wrapper();
    fn nvim_ui_flush_wrapper();
    fn nvim_may_trigger_modechanged();
    fn nvim_typebuf_maplen_wrapper() -> c_int;
    fn nvim_typebuf_typed() -> bool;
    fn nvim_ui_has_messages() -> c_int; // defined in message.c
    fn nvim_os_delay_wrapper(ms: c_int, can_interrupt: bool);
    fn nvim_showmode();
    fn nvim_show_cursor_info_later();
    fn nvim_update_screen_call();
    fn nvim_redraw_statuslines_call();
    fn nvim_curbuf_set_b_last_used();
    fn nvim_keep_msg_display_and_free();
    fn nvim_shortmess_fileinfo() -> bool;
    fn nvim_fileinfo_call();
    fn nvim_may_clear_sb_text_call();
    fn nvim_redraw_mode_msg_keep_msg();
    fn readbuf1_empty() -> bool;
    fn rs_set_vcount_ca(cap: *mut std::ffi::c_void, set_prevcount: *mut bool);

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
    if unsafe { got_int } {
        if nvim_ns_get_noexmode(s)
            && nvim_get_global_busy()
            && !nvim_get_exmode_active()
            && nvim_ns_get_previous_got_int(s)
        {
            // Typed two CTRL-C in a row: go back to ex mode as if "Q" was
            // used and keep "got_int" set, so that it aborts ":g".
            nvim_set_exmode_active(true);
            State = MODE_NORMAL;
        } else if !nvim_get_global_busy() || !nvim_get_exmode_active() {
            if !nvim_get_quit_more() {
                // flush all buffers
                nvim_vgetc_and_discard();
            }
            unsafe {
                got_int = false;
            }
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
    if nvim_get_finish_op() == 0 && nvim_has_event_cursormoved() && nvim_last_cursormoved_check() {
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

/// Rust implementation of normal_redraw.
///
/// Before each redraw in normal mode: update topline, cursor, draw screen,
/// display keep_msg, show fileinfo, clear error state.
///
/// # Safety
/// `s` is unused (no NormalState fields accessed).
#[no_mangle]
pub unsafe extern "C" fn rs_normal_redraw(_s: NormalStateHandle) {
    // Before redrawing, ensure w_topline and cursor are up to date.
    nvim_update_topline_call();
    nvim_validate_cursor();

    nvim_show_cursor_info_later();

    if unsafe { must_redraw } != 0 {
        nvim_update_screen_call();
    } else {
        nvim_redraw_statuslines_call();
        if nvim_get_redraw_cmdline() || nvim_get_clear_cmdline() || unsafe { redraw_mode } != 0 {
            nvim_showmode();
        }
    }

    nvim_curbuf_set_b_last_used();

    // Display message after redraw.
    if nvim_get_keep_msg_not_null() {
        nvim_keep_msg_display_and_free();
    }

    // Show fileinfo after redraw.
    if nvim_get_need_fileinfo() != 0 && !nvim_shortmess_fileinfo() {
        nvim_fileinfo_call();
        nvim_set_need_fileinfo(0);
    }

    nvim_set_emsg_on_display(0); // can delete error message now
    nvim_set_did_emsg(0);
    nvim_set_msg_didany(0); // reset lines_left in msg_start()
    nvim_may_clear_sb_text_call(); // clear scroll-back text on next msg

    nvim_setcursor_wrapper();
}

/// Rust implementation of normal_need_redraw_mode_message.
///
/// Returns true if we need to wait before showing the mode message.
///
/// # Safety
/// `s` must be a valid NormalState pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_normal_need_redraw_mode_message(s: NormalStateHandle) -> bool {
    let ca = nvim_ns_get_ca_ptr(s);
    let oa = nvim_ns_get_oa_ptr(s);

    (
        // 'showmode' is set and messages can be printed
        (p_smd != 0
            && nvim_get_msg_silent() == 0
            // must restart insert mode or just entered visual mode
            && (nvim_get_restart_edit() != 0
                || (nvim_get_VIsual_active() != 0
                    && nvim_ns_get_old_pos_lnum(s) == nvim_get_cursor_lnum()
                    && nvim_ns_get_old_pos_col(s) == nvim_get_cursor_col()))
            // command-line must be cleared or redrawn
            && (nvim_get_clear_cmdline() || nvim_get_redraw_cmdline())
            // some message was printed or scrolled
            && (nvim_get_msg_didout() != 0
                || (nvim_get_msg_didany() != 0 && nvim_get_msg_scroll_val()))
            // it is fine to remove the current message
            && !nvim_get_msg_nowait_val()
            // the command was the result of direct user input and not a mapping
            && nvim_get_KeyTyped())
            // must restart insert mode, not in visual mode and error message showing
            || (nvim_get_restart_edit() != 0
                && nvim_get_VIsual_active() == 0
                && nvim_get_msg_scroll_val()
                && nvim_get_emsg_on_display() != 0)
    )
    // no register was used
    && nvim_oap_get_regname_ptr(oa) == 0
    && (nvim_cap_get_retval(ca) & CA_COMMAND_BUSY == 0)
    && nvim_stuff_empty()
    && nvim_typebuf_typed()
    && nvim_get_emsg_silent() == 0
    && !nvim_get_in_assert_fails()
    && !nvim_get_did_wait_return_val()
    && nvim_oap_get_op_type_ptr(oa) == OP_NOP
}

/// Rust implementation of normal_redraw_mode_message.
///
/// Waits for user input before the mode message is drawn.
///
/// # Safety
/// `s` must be a valid NormalState pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_normal_redraw_mode_message(_s: NormalStateHandle) {
    let save_state = State;

    // Draw the cursor with the right shape here
    if nvim_get_restart_edit() != 0 {
        State = MODE_INSERT;
    }

    // If need to redraw and there is a keep_msg, redraw before the delay
    nvim_redraw_mode_msg_keep_msg();

    nvim_setcursor_wrapper();
    nvim_ui_cursor_shape_wrapper(); // show different cursor shape
    nvim_ui_flush_wrapper();
    if nvim_ui_has_messages() == 0 && (nvim_get_msg_scroll_val() || nvim_get_emsg_on_display() != 0)
    {
        nvim_os_delay_wrapper(1003, true); // wait at least one second
    }
    if nvim_ui_has_messages() != 0 {
        nvim_os_delay_wrapper(3003, false); // wait up to three seconds
    }
    State = save_state;

    nvim_set_msg_scroll_val(false);
    nvim_set_emsg_on_display(0);
}

/// Rust implementation of normal_prepare.
///
/// Initializes cmdarg_T, manages opcount/finish_op, restores K_EVENT counts,
/// sets State to MODE_NORMAL_BUSY, and calls rs_set_vcount_ca.
///
/// # Safety
/// `s` must be a valid NormalState pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_normal_prepare(s: NormalStateHandle) {
    // CLEAR_FIELD(s->ca); s->ca.oap = &s->oa;
    nvim_ns_prepare_ca(s);

    let ca = nvim_ns_get_ca_ptr(s);
    let oa = nvim_ns_get_oa_ptr(s);

    // Use a count remembered from before entering an operator.
    nvim_cap_set_opcount(ca, nvim_get_opcount());

    // Finish_op tells us to finish the operation before returning.
    let old_finish_op = nvim_get_finish_op();
    let new_finish_op = nvim_oap_get_op_type_ptr(oa) != OP_NOP;
    nvim_set_finish_op(new_finish_op);
    if new_finish_op != (old_finish_op != 0) {
        nvim_ui_cursor_shape_wrapper();
    }
    nvim_may_trigger_modechanged();

    nvim_ns_set_set_prevcount(s, false);
    // When not finishing an operator and no register name typed, reset count.
    if !new_finish_op && nvim_oap_get_regname_ptr(oa) == 0 {
        nvim_cap_set_opcount(ca, 0);
        nvim_ns_set_set_prevcount(s, true);
    }

    // Restore counts from before receiving K_EVENT.
    let prev_opcount = nvim_oap_get_prev_opcount_ptr(oa);
    let prev_count0 = nvim_oap_get_prev_count0_ptr(oa);
    if prev_opcount > 0 || prev_count0 > 0 {
        nvim_cap_set_opcount(ca, prev_opcount);
        nvim_cap_set_count0(ca, prev_count0);
        nvim_oap_set_prev_opcount(oa, 0);
        nvim_oap_set_prev_count0(oa, 0);
    }

    nvim_ns_set_mapped_len(s, nvim_typebuf_maplen_wrapper());
    State = MODE_NORMAL_BUSY;

    // Set v:count here when called from main() and not a stuffed command.
    if nvim_ns_get_toplevel(s) && readbuf1_empty() {
        let mut set_prevcount = nvim_ns_get_set_prevcount(s);
        rs_set_vcount_ca(ca, std::ptr::addr_of_mut!(set_prevcount));
        nvim_ns_set_set_prevcount(s, set_prevcount);
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
        discard_current_exception();
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
    } else if do_redraw || nvim_stuff_empty() {
        // Ensure curwin->w_topline and curwin->w_leftcol are up to date
        // before triggering a WinScrolled autocommand.
        nvim_update_topline_call();
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
        rs_normal_redraw(s);
        do_redraw = false;

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

    rs_normal_prepare(s);
    1
}
