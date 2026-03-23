//! Popup menu selection management.
//!
//! This module handles the `pum_set_selected` logic: scrolling the
//! popup menu to show the selected item, and opening/updating the
//! preview window for completion item info.

use std::ffi::{c_char, c_int, c_uint, c_void};

use crate::display::{BufHandle, WinHandle};
use crate::PUM_STATE;

/// Opaque handle to a `tabpage_T`.
#[repr(C)]
pub struct TabHandle {
    _private: [u8; 0],
}

// C accessor functions for selection/preview operations.
extern "C" {
    static Rows: c_int;
    // COT flags
    fn rs_get_cot_flags() -> c_uint;

    // Preview window hide
    fn win_float_find_preview() -> *mut WinHandle;
    fn nvim_pum_win_config_float_hide(wp: *mut WinHandle);

    // Global state

    // Autocmd / redraw control
    fn block_autocmds();
    fn unblock_autocmds();

    // Window operations
    fn win_enter(wp: *mut WinHandle, undo_sync: bool);
    fn prepare_tagpreview(undo_sync: bool) -> bool;
    fn win_float_create(enter: bool, new_buf: bool) -> *mut WinHandle;

    // Window field queries
    fn nvim_win_get_pvw(wp: *mut WinHandle) -> c_int;
    fn nvim_win_get_float_is_info(wp: *mut WinHandle) -> c_int;
    // Curbuf reuse check (multi-field, keep C wrapper for now)
    fn nvim_pum_curbuf_can_reuse() -> c_int;

    // Buffer operations
    fn buf_clear();
    fn do_ecmd(
        fnum: c_int,
        ffname: *const c_char,
        sfname: *const c_char,
        eap: *mut c_void,
        newlnum: c_int,
        flags: c_int,
        oldwin: *mut WinHandle,
    ) -> c_int;
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
    fn nvim_win_get_w_height(wp: *mut WinHandle) -> c_int;

    // Buffer state
    fn nvim_buf_set_b_changed(buf: *mut BufHandle, val: bool);
    fn nvim_buf_set_b_p_ma(buf: *mut BufHandle, val: c_int);
    fn nvim_win_buf_line_count(wp: *mut WinHandle) -> c_int;

    // Cursor/topline (via existing nvim_win_* accessors from win_struct.rs)
    fn nvim_win_get_topline(wp: *mut WinHandle) -> c_int;
    fn nvim_win_set_topline(wp: *mut WinHandle, val: c_int);
    fn nvim_win_set_cursor_lnum(wp: *mut WinHandle, lnum: c_int);
    fn nvim_win_set_cursor_col(wp: *mut WinHandle, col: c_int);

    // Window/tabpage validity (calls Rust rs_win_valid/rs_valid_tabpage directly)
    #[link_name = "rs_win_valid"]
    fn pum_rs_win_valid(wp: *mut WinHandle) -> c_int;
    #[link_name = "rs_valid_tabpage"]
    fn pum_rs_valid_tabpage(tp: *mut TabHandle) -> c_int;
    fn goto_tabpage_tp(tp: *mut TabHandle, trigger_enter: bool, trigger_leave: bool);

    // Completion state
    fn rs_ins_compl_active() -> c_int;

    // Redraw helpers
    fn nvim_win_set_redr_status(wp: *mut WinHandle, val: c_int);
    fn validate_cursor(wp: *mut WinHandle);
    fn redraw_later(wp: *mut WinHandle, update_type: c_int);
    fn update_topline(wp: *mut WinHandle);
    fn update_screen();

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
    /// C global: `p_pvh` (preview height option, `OptInt = int64_t`).
    static p_pvh: i64;
    /// C global: `cmdwin_type`.
    static cmdwin_type: c_int;
    /// C global: `g_do_tagpreview`.
    static mut g_do_tagpreview: c_int;
    /// C global: `curwin` (current window pointer).
    static mut curwin: *mut WinHandle;
    /// C global: `curtab` (current tabpage pointer).
    static mut curtab: *mut TabHandle;
    /// C global: `curbuf` (current buffer pointer).
    static mut curbuf: *mut BufHandle;
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
#[allow(
    clippy::too_many_lines,
    clippy::cognitive_complexity,
    clippy::cast_possible_truncation
)]
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
    let has_info = |idx: c_int| -> bool {
        !PUM_STATE.array.is_null() && !(*PUM_STATE.array.offset(idx as isize)).pum_info.is_null()
    };
    if use_float && (pum_selected < 0 || !has_info(pum_selected)) {
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
        if has_info(pum_selected)
            && Rows > 10
            && repeat <= 1
            && (cur_cot_flags & (K_OPT_COT_FLAG_PREVIEW | K_OPT_COT_FLAG_POPUP)) != 0
            && !((cur_cot_flags & K_OPT_COT_FLAG_PREVIEW) != 0 && cmdwin_type != 0)
        {
            let curwin_save = curwin;
            let curtab_save = curtab;

            if use_float {
                block_autocmds();
            }

            // Open a preview window. 3 lines by default. Prefer
            // 'previewheight' if set and smaller.
            let mut preview_height = 3;
            let pvh = p_pvh as c_int;
            if pvh > 0 && pvh < preview_height {
                preview_height = pvh;
            }
            g_do_tagpreview = preview_height;

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
            g_do_tagpreview = 0;

            if nvim_win_get_pvw(curwin) != 0 || nvim_win_get_float_is_info(curwin) != 0 {
                let mut res = OK;
                if !resized && nvim_pum_curbuf_can_reuse() != 0 {
                    // Already a "wipeout" buffer, make it empty.
                    buf_clear();
                } else {
                    // Don't want to sync undo in the current buffer.
                    no_u_sync += 1;
                    // do_ecmd(0, NULL, NULL, NULL, ECMD_ONE=1, 0, NULL)
                    res = do_ecmd(
                        0,
                        std::ptr::null(),
                        std::ptr::null(),
                        std::ptr::null_mut(),
                        1,
                        0,
                        std::ptr::null_mut(),
                    );
                    no_u_sync -= 1;

                    if res == OK {
                        // Edit a new, empty buffer. Set options for a "wipeout" buffer.
                        nvim_pum_set_wipeout_options();
                    }
                }

                if res == OK {
                    let mut lnum: i32 = 0;
                    let mut max_info_width: c_int = 0;
                    let info = (*PUM_STATE.array.offset(pum_selected as isize)).pum_info;
                    let buf = curbuf;
                    rs_pum_preview_set_text(
                        buf,
                        info,
                        std::ptr::addr_of_mut!(lnum),
                        std::ptr::addr_of_mut!(max_info_width),
                    );

                    // Increase the height of the preview window to show the
                    // text, but no more than 'previewheight' lines.
                    if repeat == 0 && !use_float {
                        let p_pvh_val = p_pvh as c_int;
                        if p_pvh_val > 0 && lnum > p_pvh_val {
                            lnum = p_pvh_val;
                        }
                        if nvim_win_get_w_height(curwin) < lnum {
                            rs_win_setheight(lnum);
                            resized = true;
                        }
                    }

                    nvim_buf_set_b_changed(curbuf, false);
                    nvim_buf_set_b_p_ma(curbuf, 0);

                    if pum_selected == prev_selected {
                        let topline = nvim_win_get_topline(curwin);
                        let line_count = nvim_win_buf_line_count(curwin);
                        if topline > line_count {
                            nvim_win_set_topline(curwin, line_count);
                        }
                    } else {
                        nvim_win_set_topline(curwin, 1);
                    }
                    nvim_win_set_cursor_lnum(curwin, 1);
                    nvim_win_set_cursor_col(curwin, 0);

                    if use_float {
                        // adjust floating window by actual height and max info text width
                        rs_pum_adjust_info_position(curwin, max_info_width);
                    }

                    if (curwin != curwin_save && pum_rs_win_valid(curwin_save) != 0)
                        || (curtab != curtab_save && pum_rs_valid_tabpage(curtab_save) != 0)
                    {
                        if curtab != curtab_save && pum_rs_valid_tabpage(curtab_save) != 0 {
                            goto_tabpage_tp(curtab_save, false, false);
                        }

                        // When the first completion is done and the preview
                        // window is not resized, skip the preview window's
                        // status line redrawing.
                        if rs_ins_compl_active() != 0 && !resized {
                            nvim_win_set_redr_status(curwin, 0);
                        }

                        // Return cursor to where we were
                        validate_cursor(curwin);
                        redraw_later(curwin, UPD_SOME_VALID);

                        // When the preview window was resized we need to
                        // update the view on the buffer. Only go back to
                        // the window when needed, otherwise it will always be
                        // redrawn.
                        if resized && pum_rs_win_valid(curwin_save) != 0 {
                            no_u_sync += 1;
                            win_enter(curwin_save, false);
                            no_u_sync -= 1;
                            update_topline(curwin);
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
