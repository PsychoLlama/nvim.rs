//! Normal mode pre-iteration check.
//!
//! This module provides the Rust implementation of `normal_check()`
//! from `src/nvim/normal.c`. Runs before each normal mode iteration:
//! processes stuff buffer, handles interrupts, manages redraw, and
//! handles ex mode / command-line window exits.

use std::ffi::c_int;

use crate::dispatch::types::NormalStateHandle;
use crate::types::{CmdargT, NormalState};
use crate::WinHandle;

/// UIExtension value for kUIMessages (ui_defs.h)
const K_UI_MESSAGES: c_int = 4;

/// Cast `NormalStateHandle` to a typed `*mut NormalState`.
///
/// # Safety
/// The handle must be a valid non-null `NormalState*`.
#[inline]
unsafe fn ns(s: NormalStateHandle) -> *mut NormalState {
    s.as_ptr().cast::<NormalState>()
}

extern "C" {
    static mut msg_silent: c_int;
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
const K_OPT_FDO_FLAG_ALL: u32 = 0x01;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    static mut do_redraw: bool;
}

extern "C" {
    // oparg_T accessors (used for shared oap accessors, not ns-based)
    fn nvim_oap_get_op_type_ptr(oap: *mut std::ffi::c_void) -> c_int;
    fn nvim_oap_get_regname_ptr(oap: *mut std::ffi::c_void) -> c_int;

    // cmdarg_T accessors
    fn nvim_cap_get_retval(cap: *mut std::ffi::c_void) -> c_int;
    fn nvim_cap_set_count0(cap: *mut std::ffi::c_void, val: c_int);

    // Global accessors
    static did_throw: bool;
    fn nvim_get_ex_normal_busy() -> c_int;

    static mut exmode_active: bool;
    static mut msg_scroll: c_int;
    fn nvim_set_quit_more(val: bool);
    static mut quit_more: bool;
    static mut cmdwin_result: c_int;
    fn nvim_get_skip_redraw() -> bool;
    fn nvim_set_skip_redraw(val: bool);
    static diff_need_scrollbind: bool;
    fn nvim_set_diff_need_scrollbind(val: bool);
    fn nvim_get_time_fd_not_null() -> bool;
    static mut may_garbage_collect: bool;
    fn nvim_stuff_empty() -> bool;
    fn nvim_get_finish_op() -> c_int;
    fn nvim_set_finish_op(val: bool);
    fn nvim_get_global_busy() -> bool;
    fn nvim_set_did_check_timestamps(val: bool);
    fn nvim_get_need_check_timestamps() -> bool;
    static mut need_wait_return: bool;
    fn nvim_wait_return(redraw: bool);
    static mut restart_edit: c_int;
    fn nvim_get_opcount() -> c_int;
    static mut VIsual_active: bool;
    fn nvim_get_KeyTyped() -> bool;

    // Message/display globals
    // nvim_get_p_smd: inlined (Phase 39, use p_smd directly)
    #[link_name = "p_smd"]
    static p_smd: c_int;
    fn nvim_get_clear_cmdline() -> bool;
    static redraw_cmdline: bool;
    static mut msg_didout: bool;
    static mut msg_didany: bool;
    static mut msg_nowait: bool;
    static mut emsg_on_display: bool;
    static mut emsg_silent: c_int;
    fn nvim_get_in_assert_fails() -> bool;
    static did_wait_return: bool;
    static mut did_emsg: c_int; // defined in message.c

    // Cursor globals
    fn nvim_get_cursor_lnum() -> c_int;
    fn nvim_get_cursor_col() -> c_int;

    // Redraw/display globals
    static mut need_fileinfo: bool;

    // Function wrappers
    fn discard_current_exception();
    fn nvim_state_no_longer_safe();
    fn setcursor();
    fn nvim_validate_cursor();
    fn nvim_curtab_needs_diff_update() -> bool;
    #[link_name = "ex_diffupdate"]
    fn rs_diff_ex_diffupdate(eap: *mut std::ffi::c_void);
    fn nvim_curtab_clear_diff_update();
    fn check_scrollbind(vtopline_diff: c_int, leftcol_diff: c_int);
    fn nvim_time_msg_first_screen_and_finish();
    fn may_make_initial_scroll_size_snapshot();
    fn nvim_update_curswant_wrapper();
    fn do_exmode();
    fn nvim_check_timestamps_call(focus: bool);
    fn may_trigger_win_scrolled_resized();
    fn may_trigger_safestate(safe: bool);
    fn char_avail() -> bool;
    static fdo_flags: u32;
    fn vgetc() -> c_int;
    fn rs_op_pending() -> bool;
    fn nvim_ui_cursor_shape_wrapper();
    fn ui_flush();
    fn nvim_may_trigger_modechanged();
    fn typebuf_maplen() -> c_int;
    fn typebuf_typed() -> bool;
    fn ui_has(ext: c_int) -> bool;
    fn os_delay(ms: c_int, can_interrupt: bool);
    fn nvim_showmode();
    fn nvim_show_cursor_info_later();
    fn update_screen();
    fn redraw_statuslines();
    fn nvim_curbuf_set_b_last_used();
    fn nvim_shortmess_fileinfo() -> bool;
    fn nvim_fileinfo_call();
    fn may_clear_sb_text();
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
    fn update_topline(win: WinHandle);

    // Phase 5: keep_msg/msg globals for inlining nvim_keep_msg_display_and_free
    // and nvim_redraw_mode_msg_keep_msg
    static mut keep_msg: *mut std::ffi::c_char;
    static keep_msg_hl_id: c_int;
    static mut msg_hist_off: bool;
    fn xstrdup(s: *const std::ffi::c_char) -> *mut std::ffi::c_char;
    fn xfree(ptr: *mut std::ffi::c_void);
    fn msg(s: *const std::ffi::c_char, hl_id: c_int) -> c_int;
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

        if need_wait_return {
            // if wait_return still needed call it now
            nvim_wait_return(false);
        }
    }
}

/// Inline of normal_check_interrupt.
unsafe fn normal_check_interrupt(s: NormalStateHandle) {
    let sp = ns(s);
    if unsafe { got_int } {
        if (*sp).noexmode && nvim_get_global_busy() && !exmode_active && (*sp).previous_got_int {
            // Typed two CTRL-C in a row: go back to ex mode as if "Q" was
            // used and keep "got_int" set, so that it aborts ":g".
            exmode_active = true;
            State = MODE_NORMAL;
        } else if !nvim_get_global_busy() || !exmode_active {
            if !quit_more {
                // flush all buffers
                let _ = vgetc();
            }
            unsafe {
                got_int = false;
            }
        }
        (*sp).previous_got_int = true;
    } else {
        (*sp).previous_got_int = false;
    }
}

/// Inline of normal_check_window_scrolled.
unsafe fn normal_check_window_scrolled(_s: NormalStateHandle) {
    if nvim_get_finish_op() == 0 {
        may_trigger_win_scrolled_resized();
    }
}

/// Inline of normal_check_safe_state.
unsafe fn normal_check_safe_state(_s: NormalStateHandle) {
    may_trigger_safestate(!rs_op_pending() && restart_edit == 0);
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
    if rs_hasAnyFolding(nvim_get_curwin()) != 0 && !char_avail() {
        rs_foldCheckClose();

        if fdo_flags & K_OPT_FDO_FLAG_ALL != 0 {
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
    update_topline(nvim_get_curwin());
    nvim_validate_cursor();

    nvim_show_cursor_info_later();

    if unsafe { must_redraw } != 0 {
        update_screen();
    } else {
        redraw_statuslines();
        if redraw_cmdline || nvim_get_clear_cmdline() || unsafe { redraw_mode } != 0 {
            nvim_showmode();
        }
    }

    nvim_curbuf_set_b_last_used();

    // Display message after redraw.
    if !keep_msg.is_null() {
        // Inline of nvim_keep_msg_display_and_free
        let p = xstrdup(keep_msg);
        msg_hist_off = true;
        msg(p, keep_msg_hl_id);
        msg_hist_off = false;
        xfree(p.cast());
    }

    // Show fileinfo after redraw.
    if c_int::from(need_fileinfo) != 0 && !nvim_shortmess_fileinfo() {
        nvim_fileinfo_call();
        need_fileinfo = false;
    }

    emsg_on_display = false; // can delete error message now
    did_emsg = 0;
    msg_didany = false; // reset lines_left in msg_start()
    may_clear_sb_text(); // clear scroll-back text on next msg

    setcursor();
}

/// Rust implementation of normal_need_redraw_mode_message.
///
/// Returns true if we need to wait before showing the mode message.
///
/// # Safety
/// `s` must be a valid NormalState pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_normal_need_redraw_mode_message(s: NormalStateHandle) -> bool {
    let sp = ns(s);
    let ca = (&raw mut (*sp).ca).cast::<std::ffi::c_void>();
    let oa = (&raw mut (*sp).oa).cast::<std::ffi::c_void>();

    (
        // 'showmode' is set and messages can be printed
        (p_smd != 0
            && msg_silent == 0
            // must restart insert mode or just entered visual mode
            && (restart_edit != 0
                || (VIsual_active
                    && (*sp).old_pos.lnum == nvim_get_cursor_lnum()
                    && (*sp).old_pos.col == nvim_get_cursor_col()))
            // command-line must be cleared or redrawn
            && (nvim_get_clear_cmdline() || redraw_cmdline)
            // some message was printed or scrolled
            && (c_int::from(msg_didout) != 0
                || (c_int::from(msg_didany) != 0 && msg_scroll != 0))
            // it is fine to remove the current message
            && !msg_nowait
            // the command was the result of direct user input and not a mapping
            && nvim_get_KeyTyped())
            // must restart insert mode, not in visual mode and error message showing
            || (restart_edit != 0
                && !VIsual_active
                && msg_scroll != 0
                && c_int::from(emsg_on_display) != 0)
    )
    // no register was used
    && nvim_oap_get_regname_ptr(oa) == 0
    && (nvim_cap_get_retval(ca) & CA_COMMAND_BUSY == 0)
    && nvim_stuff_empty()
    && typebuf_typed()
    && emsg_silent == 0
    && !nvim_get_in_assert_fails()
    && !did_wait_return
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
    if restart_edit != 0 {
        State = MODE_INSERT;
    }

    // If need to redraw and there is a keep_msg, redraw before the delay
    // Inline of nvim_redraw_mode_msg_keep_msg
    if must_redraw != 0 && !keep_msg.is_null() && !emsg_on_display {
        let kmsg = keep_msg;
        keep_msg = std::ptr::null_mut();
        setcursor();
        update_screen();
        keep_msg = kmsg;
        let kmsg2 = xstrdup(keep_msg);
        msg(kmsg2, keep_msg_hl_id);
        xfree(kmsg2.cast());
    }

    setcursor();
    nvim_ui_cursor_shape_wrapper(); // show different cursor shape
    ui_flush();
    if !ui_has(K_UI_MESSAGES) && (msg_scroll != 0 || c_int::from(emsg_on_display) != 0) {
        os_delay(1003, true); // wait at least one second
    }
    if ui_has(K_UI_MESSAGES) {
        os_delay(3003, false); // wait up to three seconds
    }
    State = save_state;

    msg_scroll = 0;
    emsg_on_display = false;
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
    let sp = ns(s);

    // CLEAR_FIELD(s->ca); s->ca.oap = &s->oa;
    (*sp).ca = crate::types::CmdargT::default();
    (*sp).ca.oap = &raw mut (*sp).oa;

    let ca = (&raw mut (*sp).ca).cast::<std::ffi::c_void>();
    let oa = (&raw mut (*sp).oa).cast::<std::ffi::c_void>();

    // Use a count remembered from before entering an operator.
    (*ca.cast::<CmdargT>()).opcount = nvim_get_opcount();

    // Finish_op tells us to finish the operation before returning.
    let old_finish_op = nvim_get_finish_op();
    let new_finish_op = nvim_oap_get_op_type_ptr(oa) != OP_NOP;
    nvim_set_finish_op(new_finish_op);
    if new_finish_op != (old_finish_op != 0) {
        nvim_ui_cursor_shape_wrapper();
    }
    nvim_may_trigger_modechanged();

    (*sp).set_prevcount = false;
    // When not finishing an operator and no register name typed, reset count.
    if !new_finish_op && nvim_oap_get_regname_ptr(oa) == 0 {
        (*ca.cast::<CmdargT>()).opcount = 0;
        (*sp).set_prevcount = true;
    }

    // Restore counts from before receiving K_EVENT.
    let oap_typed = &raw mut (*sp).oa;
    let prev_opcount = (*oap_typed).prev_opcount;
    let prev_count0 = (*oap_typed).prev_count0;
    if prev_opcount > 0 || prev_count0 > 0 {
        (*ca.cast::<CmdargT>()).opcount = prev_opcount;
        nvim_cap_set_count0(ca, prev_count0);
        (*oap_typed).prev_opcount = 0;
        (*oap_typed).prev_count0 = 0;
    }

    (*sp).mapped_len = typebuf_maplen();
    State = MODE_NORMAL_BUSY;

    // Set v:count here when called from main() and not a stuffed command.
    if (*sp).toplevel && readbuf1_empty() {
        let mut set_prevcount = (*sp).set_prevcount;
        rs_set_vcount_ca(ca, std::ptr::addr_of_mut!(set_prevcount));
        (*sp).set_prevcount = set_prevcount;
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
    if did_throw && nvim_get_ex_normal_busy() == 0 {
        discard_current_exception();
    }

    if !exmode_active {
        msg_scroll = 0;
    }
    nvim_set_quit_more(false);

    nvim_state_no_longer_safe();

    // If skip redraw is set (for ":" in wait_return()), don't redraw now.
    // If there is nothing in the stuff_buffer or do_redraw is true,
    // update cursor and redraw.
    if nvim_get_skip_redraw() || exmode_active {
        nvim_set_skip_redraw(false);
        setcursor();
    } else if do_redraw || nvim_stuff_empty() {
        // Ensure curwin->w_topline and curwin->w_leftcol are up to date
        // before triggering a WinScrolled autocommand.
        update_topline(nvim_get_curwin());
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
        if diff_need_scrollbind {
            check_scrollbind(0, 0);
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
        may_make_initial_scroll_size_snapshot();
    }

    // May perform garbage collection when waiting for a character, but
    // only at the very toplevel.
    may_garbage_collect = !(*ns(s)).cmdwin && !(*ns(s)).noexmode;

    // Update w_curswant if w_set_curswant has been set.
    nvim_update_curswant_wrapper();

    if exmode_active {
        if (*ns(s)).noexmode {
            return 0;
        }
        do_exmode();
        return -1;
    }

    if (*ns(s)).cmdwin && cmdwin_result != 0 {
        return 0;
    }

    rs_normal_prepare(s);
    1
}
