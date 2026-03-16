//! Popup menu selection management.
//!
//! This module handles the `pum_set_selected` logic: scrolling the
//! popup menu to show the selected item, and opening/updating the
//! preview window for completion item info.

use std::ffi::{c_char, c_int, c_uint};

use crate::display::{BufHandle, WinHandle};
use crate::PUM_STATE;

/// Opaque handle to a `tabpage_T`.
#[repr(C)]
pub struct TabHandle {
    _private: [u8; 0],
}

// C accessor functions for selection/preview operations.
extern "C" {
    // COT flags
    fn rs_get_cot_flags() -> c_uint;

    // Preview window hide
    fn win_float_find_preview() -> *mut WinHandle;
    fn nvim_pum_win_config_float_hide(wp: *mut WinHandle);

    // Array info access
    fn nvim_pum_array_has_info(idx: c_int) -> c_int;
    fn nvim_pum_array_get_info(idx: c_int) -> *mut c_char;

    // Global state
    fn nvim_get_Rows() -> c_int;
    fn nvim_pum_get_p_pvh() -> c_int;
    fn nvim_pum_get_cmdwin_type() -> c_int;
    fn nvim_pum_set_g_do_tagpreview(val: c_int);

    // Autocmd / redraw control
    fn block_autocmds();
    fn unblock_autocmds();

    // Window operations
    fn win_enter(wp: *mut WinHandle, undo_sync: bool);
    fn prepare_tagpreview(undo_sync: bool) -> bool;
    fn win_float_create(enter: bool, new_buf: bool) -> *mut WinHandle;

    // Curwin/curbuf queries
    fn nvim_pum_curwin_is_pvw_or_info() -> c_int;
    fn nvim_pum_curbuf_can_reuse() -> c_int;

    // Buffer operations
    fn buf_clear();
    fn nvim_pum_do_ecmd() -> c_int;
    fn nvim_pum_set_wipeout_options();

    // Preview text
    fn rs_pum_preview_set_text(
        buf: *mut BufHandle,
        info: *mut c_char,
        lnum: *mut i32,
        max_width: *mut c_int,
    );
    fn rs_pum_adjust_info_position(wp: *mut WinHandle, width: c_int);

    // Window height
    fn rs_win_setheight(height: c_int);
    fn nvim_pum_curwin_get_height() -> c_int;

    // Buffer state
    fn nvim_pum_set_curbuf_changed(val: c_int);
    fn nvim_pum_set_curbuf_modifiable(val: c_int);
    fn nvim_pum_curbuf_line_count() -> c_int;

    // Cursor/topline
    fn nvim_pum_curwin_get_topline() -> c_int;
    fn nvim_pum_curwin_set_topline(val: c_int);
    fn nvim_pum_curwin_set_cursor(lnum: c_int, col: c_int);

    // Window/tabpage validity (calls Rust rs_win_valid/rs_valid_tabpage directly)
    #[link_name = "rs_win_valid"]
    fn pum_rs_win_valid(wp: *mut WinHandle) -> c_int;
    #[link_name = "rs_valid_tabpage"]
    fn pum_rs_valid_tabpage(tp: *mut TabHandle) -> c_int;
    fn goto_tabpage_tp(tp: *mut TabHandle, trigger_enter: bool, trigger_leave: bool);

    // Context save/restore
    fn nvim_pum_get_curwin() -> *mut WinHandle;
    fn nvim_pum_get_curtab() -> *mut TabHandle;
    fn nvim_pum_curwin_is(wp: *mut WinHandle) -> c_int;
    fn nvim_pum_curtab_is(tp: *mut TabHandle) -> c_int;

    // Completion state
    fn rs_ins_compl_active() -> c_int;

    // Redraw helpers
    fn nvim_pum_curwin_set_redr_status(val: c_int);
    fn validate_cursor(wp: *mut WinHandle);
    fn redraw_later(wp: *mut WinHandle, update_type: c_int);
    fn update_topline(wp: *mut WinHandle);
    fn update_screen();

    // Curbuf for preview text
    fn nvim_pum_get_curbuf() -> *mut BufHandle;

    // Scroll computation (Rust function, called via extern "C")
    fn rs_pum_compute_scroll(
        selected: c_int,
        current_first: c_int,
        height: c_int,
        size: c_int,
    ) -> c_int;
}

extern "C" {
    /// C global: `RedrawingDisabled` counter.
    static mut RedrawingDisabled: c_int;
    /// C global: `no_u_sync` counter.
    static mut no_u_sync: c_int;
}

/// `kOptCotFlagPopup` = 0x10.
const K_OPT_COT_FLAG_POPUP: c_uint = 0x10;
/// `kOptCotFlagPreview` = 0x08.
const K_OPT_COT_FLAG_PREVIEW: c_uint = 0x08;
/// `UPD_SOME_VALID` = 35.
const UPD_SOME_VALID: c_int = 35;
/// `OK` = 1.
const OK: c_int = 1;

/// Set the selected item in the popup menu and manage the preview window.
///
/// Handles scrolling so the selected item is visible, and opens/updates
/// the preview window if 'completeopt' includes "preview" or "popup".
///
/// Returns 1 if the preview window was resized, 0 otherwise.
///
/// # Safety
/// Calls many C accessor functions. The popup menu array must be valid.
#[no_mangle]
#[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
pub unsafe extern "C" fn rs_pum_set_selected(n: c_int, repeat: c_int) -> c_int {
    let mut resized = false;
    let prev_selected = PUM_STATE.selected;

    PUM_STATE.selected = n;
    let pum_selected = n;
    let pum_height = PUM_STATE.height;
    let pum_size = PUM_STATE.size;
    let cur_cot_flags = rs_get_cot_flags();
    let use_float = (cur_cot_flags & K_OPT_COT_FLAG_POPUP) != 0;

    // Close the floating preview window if 'selected' is -1, indicating a return to the original
    // state. It is also closed when the selected item has no corresponding info item.
    if use_float && (pum_selected < 0 || nvim_pum_array_has_info(pum_selected) == 0) {
        let wp = win_float_find_preview();
        if !wp.is_null() {
            nvim_pum_win_config_float_hide(wp);
        }
    }

    if pum_selected >= 0 && pum_selected < pum_size {
        // Compute new scroll position using the existing pure Rust function
        let pum_first = PUM_STATE.first;
        let new_first = rs_pum_compute_scroll(pum_selected, pum_first, pum_height, pum_size);
        PUM_STATE.first = new_first;

        // Show extra info in the preview window if there is something and
        // 'completeopt' contains "preview" or "popup".
        // Skip this when tried twice already.
        // Skip this also when there is not much room.
        // Skip this for command-window when 'completeopt' contains "preview".
        // NOTE: Be very careful not to sync undo!
        if nvim_pum_array_has_info(pum_selected) != 0
            && nvim_get_Rows() > 10
            && repeat <= 1
            && (cur_cot_flags & (K_OPT_COT_FLAG_PREVIEW | K_OPT_COT_FLAG_POPUP)) != 0
            && !((cur_cot_flags & K_OPT_COT_FLAG_PREVIEW) != 0 && nvim_pum_get_cmdwin_type() != 0)
        {
            let curwin_save = nvim_pum_get_curwin();
            let curtab_save = nvim_pum_get_curtab();

            if use_float {
                block_autocmds();
            }

            // Open a preview window. 3 lines by default. Prefer
            // 'previewheight' if set and smaller.
            let mut g_do_tagpreview = 3;
            let p_pvh = nvim_pum_get_p_pvh();
            if p_pvh > 0 && p_pvh < g_do_tagpreview {
                g_do_tagpreview = p_pvh;
            }
            nvim_pum_set_g_do_tagpreview(g_do_tagpreview);

            RedrawingDisabled += 1;
            // Prevent undo sync here, if an autocommand syncs undo weird
            // things can happen to the undo tree.
            no_u_sync += 1;

            if use_float {
                let wp = win_float_find_preview();
                if wp.is_null() {
                    let wp2 = win_float_create(true, true);
                    if !wp2.is_null() {
                        resized = true;
                    }
                } else {
                    win_enter(wp, false);
                }
            } else {
                resized = prepare_tagpreview(false);
            }

            no_u_sync -= 1;
            RedrawingDisabled -= 1;
            nvim_pum_set_g_do_tagpreview(0);

            if nvim_pum_curwin_is_pvw_or_info() != 0 {
                let mut res = OK;
                if !resized && nvim_pum_curbuf_can_reuse() != 0 {
                    // Already a "wipeout" buffer, make it empty.
                    buf_clear();
                } else {
                    // Don't want to sync undo in the current buffer.
                    no_u_sync += 1;
                    res = nvim_pum_do_ecmd();
                    no_u_sync -= 1;

                    if res == OK {
                        // Edit a new, empty buffer. Set options for a "wipeout" buffer.
                        nvim_pum_set_wipeout_options();
                    }
                }

                if res == OK {
                    let mut lnum: i32 = 0;
                    let mut max_info_width: c_int = 0;
                    let info = nvim_pum_array_get_info(pum_selected);
                    let buf = nvim_pum_get_curbuf();
                    rs_pum_preview_set_text(
                        buf,
                        info,
                        std::ptr::addr_of_mut!(lnum),
                        std::ptr::addr_of_mut!(max_info_width),
                    );

                    // Increase the height of the preview window to show the
                    // text, but no more than 'previewheight' lines.
                    if repeat == 0 && !use_float {
                        let p_pvh_val = nvim_pum_get_p_pvh();
                        if p_pvh_val > 0 && lnum > p_pvh_val {
                            lnum = p_pvh_val;
                        }
                        if nvim_pum_curwin_get_height() < lnum {
                            rs_win_setheight(lnum);
                            resized = true;
                        }
                    }

                    nvim_pum_set_curbuf_changed(0);
                    nvim_pum_set_curbuf_modifiable(0);

                    if pum_selected == prev_selected {
                        let topline = nvim_pum_curwin_get_topline();
                        let line_count = nvim_pum_curbuf_line_count();
                        if topline > line_count {
                            nvim_pum_curwin_set_topline(line_count);
                        }
                    } else {
                        nvim_pum_curwin_set_topline(1);
                    }
                    nvim_pum_curwin_set_cursor(1, 0);

                    if use_float {
                        // adjust floating window by actual height and max info text width
                        let curwin = nvim_pum_get_curwin();
                        rs_pum_adjust_info_position(curwin, max_info_width);
                    }

                    if (nvim_pum_curwin_is(curwin_save) == 0 && pum_rs_win_valid(curwin_save) != 0)
                        || (nvim_pum_curtab_is(curtab_save) == 0
                            && pum_rs_valid_tabpage(curtab_save) != 0)
                    {
                        if nvim_pum_curtab_is(curtab_save) == 0
                            && pum_rs_valid_tabpage(curtab_save) != 0
                        {
                            goto_tabpage_tp(curtab_save, false, false);
                        }

                        // When the first completion is done and the preview
                        // window is not resized, skip the preview window's
                        // status line redrawing.
                        if rs_ins_compl_active() != 0 && !resized {
                            nvim_pum_curwin_set_redr_status(0);
                        }

                        // Return cursor to where we were
                        let curwin_raw = nvim_pum_get_curwin();
                        validate_cursor(curwin_raw);
                        redraw_later(curwin_raw, UPD_SOME_VALID);

                        // When the preview window was resized we need to
                        // update the view on the buffer. Only go back to
                        // the window when needed, otherwise it will always be
                        // redrawn.
                        if resized && pum_rs_win_valid(curwin_save) != 0 {
                            no_u_sync += 1;
                            win_enter(curwin_save, false);
                            no_u_sync -= 1;
                            let cw = nvim_pum_get_curwin();
                            update_topline(cw);
                        }

                        // Update the screen before drawing the popup menu.
                        // Enable updating the status lines.
                        PUM_STATE.is_visible = 0;
                        update_screen();
                        PUM_STATE.is_visible = 1;

                        if !resized && pum_rs_win_valid(curwin_save) != 0 {
                            no_u_sync += 1;
                            win_enter(curwin_save, false);
                            no_u_sync -= 1;
                        }

                        // May need to update the screen again when there are
                        // autocommands involved.
                        PUM_STATE.is_visible = 0;
                        update_screen();
                        PUM_STATE.is_visible = 1;
                    }
                }
            }

            if use_float {
                unblock_autocmds();
            }
        }
    }

    resized as c_int
}
