//! Pass-through wrappers that replace C shim functions in window_shim.c.
//!
//! Each function here is an `#[export_name]` Rust export that directly calls
//! the underlying C function, replacing a thin C wrapper. The C wrappers were
//! deleted from window_shim.c in Phase 8.
//!
//! # Safety
//! All functions are unsafe as they call into C code and access global state.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_int, c_void};

use crate::win_struct::{win_mut, win_ref};
use crate::{BufHandle, WinHandle};

// =============================================================================
// Underlying C functions (actual implementations, not wrappers)
// =============================================================================

extern "C" {
    fn validate_cursor(wp: WinHandle);
    fn redraw_all_later(r#type: c_int);
    fn redraw_later(wp: WinHandle, r#type: c_int);
    fn comp_col();
    fn compute_cmdrow();
    fn update_topline(wp: WinHandle);
    fn changed_line_abv_curs();
    fn do_autochdir();
    fn curwin_init();
    fn win_reconfig_floats();
    fn win_float_anchor_laststatus();
    fn win_float_update_statusline();
    fn status_redraw_all();
    fn msg_clr_eos_force();
    fn pum_ui_flush();
    fn msg_ui_flush();
    fn reset_dragwin();
    fn win_check_ns_hl(wp: WinHandle);
    fn win_redr_winbar(wp: WinHandle);
    fn win_redr_status(wp: WinHandle);
    fn draw_tabline();
    fn maketitle();
    fn update_screen();
    fn win_grid_alloc(wp: WinHandle);
    fn curs_columns(wp: WinHandle, may_scroll: c_int);
    fn check_cursor(wp: WinHandle);
    fn check_cursor_lnum(wp: WinHandle);
    fn setpcmark();
    fn beginline(flags: c_int);
    fn clear_matches(wp: WinHandle);
    fn free_jumplist(wp: WinHandle);
    fn win_enter(wp: WinHandle, undo_sync: bool);
    fn do_nv_ident(prefix: c_int, xchar: c_int);
    fn set_keep_msg(s: *const c_void, attr: c_int);
    fn xfree(ptr: *mut c_void);
    fn ui_has(cap: c_int) -> bool;
    fn ui_call_grid_destroy(handle: c_int);
    fn ui_call_win_hide(grid_handle: c_int);

    fn nvim_get_curwin() -> WinHandle;

    // --- nvim_status_redraw_all helpers ---
    fn nvim_get_firstwin() -> WinHandle;
    fn rs_global_stl_height() -> c_int;
}

// =============================================================================
// BL_SOL / BL_FIX constants (from move.h)
// =============================================================================

const BL_SOL: c_int = 2;
const BL_FIX: c_int = 4;

// kUITabline = 22, kUIMultigrid = 24 (from ui_defs.h)
const K_UI_TABLINE: c_int = 22;
const K_UI_MULTIGRID: c_int = 24;

// =============================================================================
// Exports
// =============================================================================

/// validate_cursor(curwin) wrapper.
#[export_name = "nvim_validate_cursor"]
pub unsafe extern "C" fn wrap_validate_cursor() {
    let curwin = nvim_get_curwin();
    validate_cursor(curwin);
}

/// redraw_all_later(type) wrapper.
#[export_name = "nvim_redraw_all_later"]
pub unsafe extern "C" fn wrap_redraw_all_later(r#type: c_int) {
    redraw_all_later(r#type);
}

/// if (wp) redraw_later(wp, type) wrapper.
#[export_name = "nvim_redraw_later_wrapper"]
pub unsafe extern "C" fn wrap_redraw_later(wp: WinHandle, r#type: c_int) {
    if !wp.is_null() {
        redraw_later(wp, r#type);
    }
}

/// comp_col() wrapper.
#[export_name = "nvim_comp_col"]
pub unsafe extern "C" fn wrap_comp_col() {
    comp_col();
}

/// compute_cmdrow() wrapper.
#[export_name = "nvim_compute_cmdrow"]
pub unsafe extern "C" fn wrap_compute_cmdrow() {
    compute_cmdrow();
}

/// update_topline(curwin) wrapper.
#[export_name = "nvim_update_topline_curwin_enter"]
pub unsafe extern "C" fn wrap_update_topline_curwin_enter() {
    let curwin = nvim_get_curwin();
    update_topline(curwin);
}

/// changed_line_abv_curs() wrapper.
#[export_name = "nvim_changed_line_abv_curs_wrap"]
pub unsafe extern "C" fn wrap_changed_line_abv_curs() {
    changed_line_abv_curs();
}

/// do_autochdir() wrapper.
#[export_name = "nvim_do_autochdir_wrap"]
pub unsafe extern "C" fn wrap_do_autochdir() {
    do_autochdir();
}

/// curwin_init() wrapper.
#[export_name = "nvim_curwin_init"]
pub unsafe extern "C" fn wrap_curwin_init() {
    curwin_init();
}

/// win_reconfig_floats() wrapper.
#[export_name = "nvim_win_reconfig_floats"]
pub unsafe extern "C" fn wrap_win_reconfig_floats() {
    win_reconfig_floats();
}

/// win_float_anchor_laststatus() wrapper.
#[export_name = "nvim_win_float_anchor_laststatus"]
pub unsafe extern "C" fn wrap_win_float_anchor_laststatus() {
    win_float_anchor_laststatus();
}

/// win_float_update_statusline() wrapper.
#[export_name = "nvim_win_float_update_statusline"]
pub unsafe extern "C" fn wrap_win_float_update_statusline() {
    win_float_update_statusline();
}

/// status_redraw_all() wrapper.
#[export_name = "nvim_status_redraw_all_wrapper"]
pub unsafe extern "C" fn wrap_status_redraw_all() {
    status_redraw_all();
}

/// msg_clr_eos_force() wrapper.
#[export_name = "nvim_msg_clr_eos_force"]
pub unsafe extern "C" fn wrap_msg_clr_eos_force() {
    msg_clr_eos_force();
}

/// pum_ui_flush() wrapper.
#[export_name = "nvim_pum_ui_flush_wrapper"]
pub unsafe extern "C" fn wrap_pum_ui_flush() {
    pum_ui_flush();
}

/// msg_ui_flush() wrapper.
#[export_name = "nvim_msg_ui_flush_wrapper"]
pub unsafe extern "C" fn wrap_msg_ui_flush() {
    msg_ui_flush();
}

/// reset_dragwin() wrapper.
#[export_name = "nvim_reset_dragwin"]
pub unsafe extern "C" fn wrap_reset_dragwin() {
    reset_dragwin();
}

/// win_check_ns_hl(wp) wrapper.
#[export_name = "nvim_win_check_ns_hl"]
pub unsafe extern "C" fn wrap_win_check_ns_hl(wp: WinHandle) {
    win_check_ns_hl(wp);
}

/// win_redr_winbar(wp) wrapper.
#[export_name = "nvim_win_redr_winbar"]
pub unsafe extern "C" fn wrap_win_redr_winbar(wp: WinHandle) {
    win_redr_winbar(wp);
}

/// win_redr_status(wp) wrapper.
#[export_name = "nvim_win_redr_status"]
pub unsafe extern "C" fn wrap_win_redr_status(wp: WinHandle) {
    win_redr_status(wp);
}

/// draw_tabline() wrapper.
#[export_name = "nvim_draw_tabline"]
pub unsafe extern "C" fn wrap_draw_tabline() {
    draw_tabline();
}

/// maketitle() wrapper.
#[export_name = "nvim_maketitle"]
pub unsafe extern "C" fn wrap_maketitle() {
    maketitle();
}

/// update_screen() wrapper.
#[export_name = "nvim_update_screen"]
pub unsafe extern "C" fn wrap_update_screen() {
    update_screen();
}

/// if (wp) win_grid_alloc(wp) wrapper.
#[export_name = "nvim_win_call_win_grid_alloc"]
pub unsafe extern "C" fn wrap_win_call_win_grid_alloc(wp: WinHandle) {
    if !wp.is_null() {
        win_grid_alloc(wp);
    }
}

/// if (wp) curs_columns(wp, true) wrapper.
#[export_name = "nvim_curs_columns_win"]
pub unsafe extern "C" fn wrap_curs_columns_win(wp: WinHandle) {
    if !wp.is_null() {
        curs_columns(wp, 1);
    }
}

/// check_cursor(wp) wrapper.
#[export_name = "nvim_check_cursor_win_wrapper"]
pub unsafe extern "C" fn wrap_check_cursor_win(wp: WinHandle) {
    check_cursor(wp);
}

/// check_cursor_lnum(curwin) wrapper.
#[export_name = "nvim_check_cursor_lnum_curwin"]
pub unsafe extern "C" fn wrap_check_cursor_lnum_curwin() {
    let curwin = nvim_get_curwin();
    check_cursor_lnum(curwin);
}

/// setpcmark() wrapper.
#[export_name = "nvim_setpcmark_curwin"]
pub unsafe extern "C" fn wrap_setpcmark_curwin() {
    setpcmark();
}

/// beginline(BL_SOL | BL_FIX) wrapper.
#[export_name = "nvim_beginline_sol_fix"]
pub unsafe extern "C" fn wrap_beginline_sol_fix() {
    beginline(BL_SOL | BL_FIX);
}

/// clear_matches(wp) wrapper.
#[export_name = "nvim_clear_matches_win"]
pub unsafe extern "C" fn wrap_clear_matches_win(wp: WinHandle) {
    clear_matches(wp);
}

/// free_jumplist(wp) wrapper.
#[export_name = "nvim_free_jumplist_win"]
pub unsafe extern "C" fn wrap_free_jumplist_win(wp: WinHandle) {
    free_jumplist(wp);
}

/// win_enter(wp, undo_sync != 0) wrapper.
#[export_name = "nvim_win_enter"]
pub unsafe extern "C" fn wrap_win_enter(wp: WinHandle, undo_sync: c_int) {
    win_enter(wp, undo_sync != 0);
}

/// do_nv_ident(prefix, xchar) wrapper.
#[export_name = "nvim_do_nv_ident"]
pub unsafe extern "C" fn wrap_do_nv_ident(prefix: c_int, xchar: c_int) {
    do_nv_ident(prefix, xchar);
}

/// set_keep_msg(NULL, 0) wrapper.
#[export_name = "nvim_set_keep_msg_null"]
pub unsafe extern "C" fn wrap_set_keep_msg_null() {
    set_keep_msg(std::ptr::null(), 0);
}

/// xfree(tp) wrapper for tabpage_T.
#[export_name = "nvim_xfree_tabpage_raw"]
pub unsafe extern "C" fn wrap_xfree_tabpage_raw(tp: *mut c_void) {
    xfree(tp);
}

/// xfree(wp) wrapper for win_T.
#[export_name = "nvim_xfree_win_raw"]
pub unsafe extern "C" fn wrap_xfree_win_raw(wp: *mut c_void) {
    xfree(wp);
}

/// xfree(frp) wrapper for frame_T.
#[export_name = "nvim_xfree_frame"]
pub unsafe extern "C" fn wrap_xfree_frame(frp: *mut c_void) {
    xfree(frp);
}

/// ui_has(kUITabline) wrapper.
#[must_use]
#[export_name = "nvim_ui_has_tabline"]
pub unsafe extern "C" fn wrap_ui_has_tabline() -> c_int {
    c_int::from(ui_has(K_UI_TABLINE))
}

/// ui_has(kUIMultigrid) wrapper.
#[must_use]
#[export_name = "nvim_ui_has_multigrid"]
pub unsafe extern "C" fn wrap_ui_has_multigrid() -> c_int {
    c_int::from(ui_has(K_UI_MULTIGRID))
}

/// ui_call_grid_destroy(handle) wrapper.
#[export_name = "nvim_ui_call_grid_destroy_handle"]
pub unsafe extern "C" fn wrap_ui_call_grid_destroy_handle(handle: c_int) {
    ui_call_grid_destroy(handle);
}

/// ui_call_win_hide(grid_handle) wrapper.
#[export_name = "nvim_win_ui_call_win_hide"]
pub unsafe extern "C" fn wrap_win_ui_call_win_hide(grid_handle: c_int) {
    ui_call_win_hide(grid_handle);
}

/// if (wp) ui_call_win_close(wp->w_grid_alloc.handle) wrapper.
///
/// Uses nvim_win_get_grid_alloc_handle to retrieve the grid handle.
#[export_name = "nvim_ui_call_win_close_win"]
pub unsafe extern "C" fn wrap_ui_call_win_close_win(wp: WinHandle) {
    if !wp.is_null() {
        let handle = crate::win_struct::win_grid_alloc_handle(wp);
        ui_call_win_close(handle);
    }
}

extern "C" {
    fn ui_call_win_close(handle: c_int);
}

// =============================================================================
// Phase 9: Buffer-via-window accessors (using WinStruct direct access + C APIs)
// =============================================================================

extern "C" {
    fn nvim_get_curbuf() -> BufHandle;
    fn buf_is_empty(buf: BufHandle) -> bool;
    fn nvim_buf_meta_total(buf: BufHandle, kind: c_int) -> c_int;
    fn is_aucmd_win(wp: WinHandle) -> c_int;
}

// kMTMetaInline=0, kMTMetaLines=1, kMTMetaSignHL=2, kMTMetaSignText=3 (from marktree_defs.h)
const K_MT_META_SIGN_TEXT: c_int = 3;
const K_MT_META_LINES: c_int = 1;

/// W_ENDROW(wp) = wp->w_winrow + wp->w_height
#[must_use]
#[allow(clippy::missing_const_for_fn)]
#[export_name = "nvim_win_get_endrow"]
pub unsafe extern "C" fn wrap_win_get_endrow(wp: WinHandle) -> c_int {
    let ws = win_ref(wp);
    ws.w_winrow + ws.w_height
}

/// W_ENDCOL(wp) = wp->w_wincol + wp->w_width
#[must_use]
#[allow(clippy::missing_const_for_fn)]
#[export_name = "nvim_win_get_endcol"]
pub unsafe extern "C" fn wrap_win_get_endcol(wp: WinHandle) -> c_int {
    let ws = win_ref(wp);
    ws.w_wincol + ws.w_width
}

// nvim_win_hl_attr: kept in C (win_hl_attr is static inline; can't link directly from Rust)

/// wp->w_buffer == buf comparison.
#[must_use]
#[export_name = "nvim_win_buffer_eq"]
pub unsafe extern "C" fn wrap_win_buffer_eq(wp: WinHandle, buf: BufHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    let win_buf = BufHandle(win_ref(wp).w_buffer);
    c_int::from(win_buf == buf)
}

/// buf_is_empty(wp->w_buffer) wrapper.
#[must_use]
#[export_name = "nvim_win_buf_is_empty"]
pub unsafe extern "C" fn wrap_win_buf_is_empty(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 1;
    }
    let buf = BufHandle(win_ref(wp).w_buffer);
    if buf.is_null() {
        return 1;
    }
    c_int::from(buf_is_empty(buf))
}

/// wp->w_buffer == curbuf comparison.
#[must_use]
#[export_name = "nvim_win_buf_is_curbuf"]
pub unsafe extern "C" fn wrap_win_buf_is_curbuf(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    let buf = BufHandle(win_ref(wp).w_buffer);
    let curbuf = nvim_get_curbuf();
    c_int::from(buf == curbuf)
}

/// buf_meta_total(wp->w_buffer, kMTMetaSignText) > 0.
#[must_use]
#[export_name = "nvim_win_buf_meta_total_signtext"]
pub unsafe extern "C" fn wrap_win_buf_meta_total_signtext(wp: WinHandle) -> c_int {
    let buf = BufHandle(win_ref(wp).w_buffer);
    c_int::from(nvim_buf_meta_total(buf, K_MT_META_SIGN_TEXT) > 0)
}

/// buf_meta_total(wp->w_buffer, kMTMetaLines) > 0.
#[must_use]
#[export_name = "nvim_win_buf_meta_total_lines"]
pub unsafe extern "C" fn wrap_win_buf_meta_total_lines(wp: WinHandle) -> c_int {
    let buf = BufHandle(win_ref(wp).w_buffer);
    c_int::from(nvim_buf_meta_total(buf, K_MT_META_LINES) > 0)
}

/// is_aucmd_win(wp) wrapper.
#[must_use]
#[export_name = "nvim_is_aucmd_win"]
pub unsafe extern "C" fn wrap_is_aucmd_win(wp: WinHandle) -> c_int {
    c_int::from(is_aucmd_win(wp) != 0)
}

// =============================================================================
// rs_emsg_id: error message dispatcher (replaces nvim_emsg_id in C shim)
// =============================================================================

extern "C" {
    fn emsg(s: *const std::ffi::c_char);
    fn gettext(msgid: *const std::ffi::c_char) -> *const std::ffi::c_char;
    /// e_noalt: "E23: No alternate file"
    static e_noalt: [std::ffi::c_char; 0];
    /// e_floatonly: "E5601: Cannot close window, only floating window would remain"
    static e_floatonly: [std::ffi::c_char; 0];
    /// e_floatexchange: "E5602: Cannot exchange or rotate float"
    static e_floatexchange: [std::ffi::c_char; 0];
    /// e_autocmd_close: "E813: Cannot close autocmd window"
    static e_autocmd_close: [std::ffi::c_char; 0];
    /// e_winfixbuf_cannot_go_to_buffer: "E1513: Cannot switch buffer. 'winfixbuf' is enabled"
    static e_winfixbuf_cannot_go_to_buffer: [std::ffi::c_char; 0];
    /// e_noroom: "E36: Not enough room"
    static e_noroom: [std::ffi::c_char; 0];
}

// Inline translated strings for IDs 0-6 and 12 (no C extern string variable).
// Must match window_shim.c nvim_emsg_id exactly (same gettext msgids).
const MSG_E444: &std::ffi::CStr = c"E444: Cannot close last window";
const MSG_E814: &std::ffi::CStr = c"E814: Cannot close window, only autocmd window would remain";
const MSG_E443: &std::ffi::CStr = c"E443: Cannot rotate when another window is split";
const MSG_E442: &std::ffi::CStr = c"E442: Can't split topleft and botright at the same time";
const MSG_E242: &std::ffi::CStr = c"E242: Can't split a window while closing another";
const MSG_E445: &std::ffi::CStr = c"E445: Other window contains changes";
const MSG_E441: &std::ffi::CStr = c"E441: There is no preview window";
const MSG_E1159: &std::ffi::CStr = c"E1159: Cannot split a window when closing the buffer";

/// Error message dispatcher, replacing the C `nvim_emsg_id` function.
///
/// IDs must match the constants defined in each Rust module's `EMSG_*` consts.
#[export_name = "nvim_emsg_id"]
pub unsafe extern "C" fn rs_emsg_id(id: c_int) {
    match id {
        0 => emsg(gettext(MSG_E444.as_ptr())),
        1 => emsg(gettext(MSG_E814.as_ptr())),
        2 => emsg(gettext(MSG_E443.as_ptr())),
        3 => emsg(gettext(MSG_E442.as_ptr())),
        4 => emsg(gettext(MSG_E242.as_ptr())),
        5 => emsg(gettext(MSG_E445.as_ptr())),
        6 => emsg(gettext(MSG_E441.as_ptr())),
        7 => emsg(gettext(e_noalt.as_ptr())),
        8 => emsg(e_floatonly.as_ptr()),
        9 => emsg(e_floatexchange.as_ptr()),
        10 => emsg(gettext(e_autocmd_close.as_ptr())),
        11 => emsg(gettext(e_winfixbuf_cannot_go_to_buffer.as_ptr())),
        12 => emsg(gettext(MSG_E1159.as_ptr())),
        13 => emsg(gettext(e_noroom.as_ptr())),
        _ => {}
    }
}

/// Rust port of `nvim_status_redraw_all` from window_shim.c.
///
/// Marks w_redr_status for all windows in the current tab that have a
/// status line or winbar height > 0, or are curwin and there is a global
/// status line. Does NOT call redraw_later (unlike `status_redraw_all`).
#[export_name = "nvim_status_redraw_all"]
pub unsafe extern "C" fn rs_nvim_status_redraw_all() {
    let curwin = nvim_get_curwin();
    let global_stl = rs_global_stl_height();
    let mut wp = nvim_get_firstwin();
    while !wp.is_null() {
        if win_ref(wp).w_status_height > 0 || (wp == curwin && global_stl > 0) {
            win_mut(wp).w_redr_status = true;
        }
        if win_ref(wp).w_winbar_height > 0 {
            win_mut(wp).w_redr_status = true;
        }
        wp = win_ref(wp).w_next;
    }
}
