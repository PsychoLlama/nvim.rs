//! Global variable accessors for Neovim window management.
//!
//! This module provides Rust `#[export_name]` implementations of C accessor
//! functions that read and write global Neovim variables (curwin, curbuf,
//! curtab, option globals, etc.).
//!
//! All C functions replaced by this module are one-liners that simply read
//! or write a global variable.

use std::ffi::{c_char, c_int};

use crate::{win_struct::win_ref, BufHandle, TabpageHandle, WinHandle};

// -------------------------------------------------------------------------
// C global variable declarations
// -------------------------------------------------------------------------

extern "C" {
    // Window globals
    static mut curwin: WinHandle;
    static mut firstwin: WinHandle;
    static mut lastwin: WinHandle;
    static mut prevwin: WinHandle;

    // Buffer globals
    static mut curbuf: BufHandle;

    // Tabpage globals
    static mut curtab: TabpageHandle;
    static mut first_tabpage: TabpageHandle;
    static mut lastused_tabpage: TabpageHandle;

    // Frame globals
    static mut topframe: *mut crate::Frame;

    // Screen dimensions
    static mut Rows: c_int;
    static mut Columns: c_int;

    // State
    static mut State: c_int;
    static mut exiting: bool;
    static mut stop_insert_mode: bool;
    static mut skip_win_fix_cursor: bool;
    static mut skip_win_fix_scroll: bool;
    static mut redraw_cmdline: bool;

    // Message position
    static mut msg_row: c_int;
    static mut msg_col: c_int;

    // Options (from option_vars.h)
    static mut p_wmw: i64; // winminwidth
    static mut p_wh: i64; // winheight
    static mut p_wmh: i64; // winminheight
    static mut p_wiw: i64; // winwidth
    static mut p_so: i64; // scrolloff
    static mut p_siso: i64; // sidescrolloff
    static mut p_ls: i64; // laststatus
    static mut p_stal: i64; // showtabline
    static mut p_ph: i64; // previewheight
    static mut p_pw: i64; // previewwidth
    static mut p_pmw: i64; // pumwidth
    static mut p_pvh: i64; // previewheight (pum)
    static mut p_sb: bool; // splitbelow
    static mut p_spr: bool; // splitright
    static mut p_ea: bool; // equalalways
    static mut p_ch: i64; // cmdheight (window p_ch)
    static mut p_ead: *const c_char; // eadirection
    static mut p_cpo: *mut c_char; // cpoptions
    static mut p_sbr: *mut c_char; // showbreak
    static mut p_wbr: *const c_char; // winbar
}

// -------------------------------------------------------------------------
// C accessor helpers (for compound globals)
// -------------------------------------------------------------------------

extern "C" {
    /// Get the w_handle field from a window (its integer ID).
    fn nvim_win_get_handle(wp: WinHandle) -> c_int;

    /// Get the w_buffer field from a window.
    fn nvim_win_get_w_buffer(wp: WinHandle) -> BufHandle;

    /// Set tp_firstwin on a tabpage.
    fn nvim_tabpage_set_firstwin(tp: TabpageHandle, wp: WinHandle);

    /// Set tp_lastwin on a tabpage.
    fn nvim_tabpage_set_lastwin(tp: TabpageHandle, wp: WinHandle);

    /// Get rs_tabline_height (Rust function).
    #[link_name = "rs_tabline_height"]
    fn tabline_height() -> c_int;

    /// Get rs_global_stl_height (Rust function).
    #[link_name = "rs_global_stl_height"]
    fn global_stl_height() -> c_int;
}

// -------------------------------------------------------------------------
// curwin accessors
// -------------------------------------------------------------------------

/// Get the current window handle.
///
/// # Safety
/// This accesses the C global `curwin`.
#[must_use]
#[export_name = "nvim_get_curwin"]
pub unsafe extern "C" fn get_curwin() -> WinHandle {
    curwin
}

/// Set the current window.
///
/// # Safety
/// `wp` must be a valid window pointer or null.
#[export_name = "nvim_set_curwin"]
pub unsafe extern "C" fn set_curwin(wp: WinHandle) {
    curwin = wp;
}

/// Set curwin = wp (alias for nvim_set_curwin).
///
/// # Safety
/// `wp` must be a valid window pointer or null.
#[export_name = "nvim_set_curwin_to_wp"]
pub unsafe extern "C" fn set_curwin_to_wp(wp: WinHandle) {
    curwin = wp;
}

/// Set curwin = NULL.
///
/// # Safety
/// Accesses C global curwin.
#[export_name = "nvim_set_curwin_null"]
pub unsafe extern "C" fn set_curwin_null() {
    curwin = WinHandle::null();
}

/// Get the current window's integer handle (wp->handle).
///
/// # Safety
/// curwin must be a valid non-null window pointer.
#[must_use]
#[export_name = "nvim_get_curwin_handle"]
pub unsafe extern "C" fn get_curwin_handle() -> c_int {
    nvim_win_get_handle(curwin)
}

/// Get curwin->w_p_wfb (winfixbuf option), returned as c_int.
///
/// # Safety
/// curwin must be a valid non-null window pointer.
#[must_use]
#[export_name = "nvim_get_curwin_p_wfb"]
pub unsafe extern "C" fn get_curwin_p_wfb() -> c_int {
    win_ref(curwin).w_p_wfb()
}

// -------------------------------------------------------------------------
// firstwin / lastwin accessors
// -------------------------------------------------------------------------

/// Get the first window.
///
/// # Safety
/// Accesses C global firstwin.
#[must_use]
#[export_name = "nvim_get_firstwin"]
pub unsafe extern "C" fn get_firstwin() -> WinHandle {
    firstwin
}

/// Get the last window.
///
/// # Safety
/// Accesses C global lastwin.
#[must_use]
#[export_name = "nvim_get_lastwin"]
pub unsafe extern "C" fn get_lastwin() -> WinHandle {
    lastwin
}

/// Set firstwin and sync curtab->tp_firstwin.
///
/// # Safety
/// `wp` must be a valid window pointer or null.
#[export_name = "nvim_set_firstwin"]
pub unsafe extern "C" fn set_firstwin(wp: WinHandle) {
    firstwin = wp;
    if !curtab.is_null() {
        nvim_tabpage_set_firstwin(curtab, wp);
    }
}

/// Set lastwin and sync curtab->tp_lastwin.
///
/// # Safety
/// `wp` must be a valid window pointer or null.
#[export_name = "nvim_set_lastwin"]
pub unsafe extern "C" fn set_lastwin(wp: WinHandle) {
    lastwin = wp;
    if !curtab.is_null() {
        nvim_tabpage_set_lastwin(curtab, wp);
    }
}

// -------------------------------------------------------------------------
// prevwin accessor
// -------------------------------------------------------------------------

/// Get prevwin.
///
/// # Safety
/// Accesses C global prevwin.
#[must_use]
#[export_name = "nvim_get_prevwin"]
pub unsafe extern "C" fn get_prevwin() -> WinHandle {
    prevwin
}

/// Set prevwin.
///
/// # Safety
/// `wp` must be a valid window pointer or null.
#[export_name = "nvim_set_prevwin"]
pub unsafe extern "C" fn set_prevwin(wp: WinHandle) {
    prevwin = wp;
}

// -------------------------------------------------------------------------
// curbuf accessors
// -------------------------------------------------------------------------

/// Get curbuf.
///
/// # Safety
/// Accesses C global curbuf.
#[must_use]
#[export_name = "nvim_get_curbuf"]
pub unsafe extern "C" fn get_curbuf() -> BufHandle {
    curbuf
}

/// Set curbuf.
///
/// # Safety
/// `buf` must be a valid buffer pointer or null.
#[export_name = "nvim_set_curbuf"]
pub unsafe extern "C" fn set_curbuf(buf: BufHandle) {
    curbuf = buf;
}

/// Set curbuf = curwin->w_buffer.
///
/// # Safety
/// curwin must be a valid non-null window with a valid buffer.
#[export_name = "nvim_set_curbuf_from_curwin"]
pub unsafe extern "C" fn set_curbuf_from_curwin() {
    curbuf = nvim_win_get_w_buffer(curwin);
}

// -------------------------------------------------------------------------
// curtab / first_tabpage / lastused_tabpage accessors
// -------------------------------------------------------------------------

/// Get curtab.
///
/// # Safety
/// Accesses C global curtab.
#[must_use]
#[export_name = "nvim_get_curtab"]
pub unsafe extern "C" fn get_curtab() -> TabpageHandle {
    curtab
}

/// Set curtab.
///
/// # Safety
/// `tp` must be a valid tabpage pointer or null.
#[export_name = "nvim_set_curtab"]
pub unsafe extern "C" fn set_curtab(tp: TabpageHandle) {
    curtab = tp;
}

/// Get first_tabpage.
///
/// # Safety
/// Accesses C global first_tabpage.
#[must_use]
#[export_name = "nvim_get_first_tabpage"]
pub unsafe extern "C" fn get_first_tabpage() -> TabpageHandle {
    first_tabpage
}

/// Set first_tabpage.
///
/// # Safety
/// `tp` must be a valid tabpage pointer or null.
#[export_name = "nvim_set_first_tabpage"]
pub unsafe extern "C" fn set_first_tabpage(tp: TabpageHandle) {
    first_tabpage = tp;
}

/// Get lastused_tabpage.
///
/// # Safety
/// Accesses C global lastused_tabpage.
#[must_use]
#[export_name = "nvim_get_lastused_tabpage"]
pub unsafe extern "C" fn get_lastused_tabpage() -> TabpageHandle {
    lastused_tabpage
}

/// Get lastused_tabpage (raw alias).
///
/// # Safety
/// Accesses C global lastused_tabpage.
#[must_use]
#[export_name = "nvim_get_lastused_tabpage_raw"]
pub unsafe extern "C" fn get_lastused_tabpage_raw() -> TabpageHandle {
    lastused_tabpage
}

/// Set lastused_tabpage = NULL.
///
/// # Safety
/// Accesses C global lastused_tabpage.
#[export_name = "nvim_set_lastused_tabpage_null"]
pub unsafe extern "C" fn set_lastused_tabpage_null() {
    lastused_tabpage = TabpageHandle::null();
}

// -------------------------------------------------------------------------
// topframe accessor
// -------------------------------------------------------------------------

/// Get topframe.
///
/// # Safety
/// Accesses C global topframe.
#[must_use]
#[export_name = "nvim_get_topframe"]
pub unsafe extern "C" fn get_topframe() -> *mut crate::Frame {
    topframe
}

/// Set topframe.
///
/// # Safety
/// `fr` must be a valid frame pointer or null.
#[export_name = "nvim_set_topframe"]
pub unsafe extern "C" fn set_topframe(fr: *mut crate::Frame) {
    topframe = fr;
}

/// Get ROWS_AVAIL = Rows - p_ch - rs_tabline_height() - rs_global_stl_height().
///
/// # Safety
/// Accesses C globals Rows and p_ch, and calls Rust height functions.
#[must_use]
#[export_name = "nvim_get_rows_avail"]
pub unsafe extern "C" fn get_rows_avail() -> c_int {
    #[allow(clippy::cast_possible_truncation)]
    let ch = p_ch as i32;
    Rows - ch - tabline_height() - global_stl_height()
}

/// Get exiting (1 if true, 0 if false).
///
/// # Safety
/// Accesses C global exiting.
#[must_use]
#[export_name = "nvim_get_exiting"]
pub unsafe extern "C" fn get_exiting() -> c_int {
    c_int::from(exiting)
}

/// Get stop_insert_mode (1 if true, 0 if false).
///
/// # Safety
/// Accesses C global stop_insert_mode.
#[must_use]
#[export_name = "nvim_get_stop_insert_mode"]
pub unsafe extern "C" fn get_stop_insert_mode() -> c_int {
    c_int::from(stop_insert_mode)
}

/// Set stop_insert_mode.
///
/// # Safety
/// Accesses C global stop_insert_mode.
#[export_name = "nvim_set_stop_insert_mode"]
pub unsafe extern "C" fn set_stop_insert_mode(val: c_int) {
    stop_insert_mode = val != 0;
}

/// Get skip_win_fix_cursor (1 if true, 0 if false).
///
/// # Safety
/// Accesses C global skip_win_fix_cursor.
#[must_use]
#[export_name = "nvim_get_skip_win_fix_cursor"]
pub unsafe extern "C" fn get_skip_win_fix_cursor() -> c_int {
    c_int::from(skip_win_fix_cursor)
}

/// Get skip_win_fix_scroll (1 if true, 0 if false).
///
/// # Safety
/// Accesses C global skip_win_fix_scroll.
#[must_use]
#[export_name = "nvim_get_skip_win_fix_scroll"]
pub unsafe extern "C" fn get_skip_win_fix_scroll() -> c_int {
    c_int::from(skip_win_fix_scroll)
}

/// Set redraw_cmdline flag.
///
/// # Safety
/// Accesses C global redraw_cmdline.
#[export_name = "nvim_set_redraw_cmdline"]
pub unsafe extern "C" fn set_redraw_cmdline(val: bool) {
    redraw_cmdline = val;
}

// -------------------------------------------------------------------------
// Window option globals
// -------------------------------------------------------------------------

/// Get p_wmw (winminwidth).
///
/// # Safety
/// Accesses C global p_wmw.
#[must_use]
#[export_name = "nvim_get_p_wmw"]
pub unsafe extern "C" fn get_p_wmw() -> i64 {
    p_wmw
}

/// Set p_wmw (winminwidth).
///
/// # Safety
/// Accesses C global p_wmw.
#[export_name = "nvim_set_p_wmw"]
pub unsafe extern "C" fn set_p_wmw(val: i64) {
    p_wmw = val;
}

/// Get p_wh (winheight).
///
/// # Safety
/// Accesses C global p_wh.
#[must_use]
#[export_name = "nvim_get_p_wh"]
pub unsafe extern "C" fn get_p_wh() -> i64 {
    p_wh
}

/// Set p_wh (winheight).
///
/// # Safety
/// Accesses C global p_wh.
#[export_name = "nvim_set_p_wh"]
pub unsafe extern "C" fn set_p_wh(val: i64) {
    p_wh = val;
}

/// Get p_wmh (winminheight).
///
/// # Safety
/// Accesses C global p_wmh.
#[must_use]
#[export_name = "nvim_get_p_wmh"]
pub unsafe extern "C" fn get_p_wmh() -> i64 {
    p_wmh
}

/// Set p_wmh (winminheight).
///
/// # Safety
/// Accesses C global p_wmh.
#[export_name = "nvim_set_p_wmh"]
pub unsafe extern "C" fn set_p_wmh(val: i64) {
    p_wmh = val;
}

/// Get p_wiw (winwidth).
///
/// # Safety
/// Accesses C global p_wiw.
#[must_use]
#[export_name = "nvim_get_p_wiw"]
pub unsafe extern "C" fn get_p_wiw() -> i64 {
    p_wiw
}

/// Set p_wiw (winwidth).
///
/// # Safety
/// Accesses C global p_wiw.
#[export_name = "nvim_set_p_wiw"]
pub unsafe extern "C" fn set_p_wiw(val: i64) {
    p_wiw = val;
}

/// Get p_so (scrolloff).
///
/// # Safety
/// Accesses C global p_so.
#[must_use]
#[export_name = "nvim_get_p_so"]
pub unsafe extern "C" fn get_p_so() -> i64 {
    p_so
}

/// Get p_siso (sidescrolloff).
///
/// # Safety
/// Accesses C global p_siso.
#[must_use]
#[export_name = "nvim_get_p_siso"]
pub unsafe extern "C" fn get_p_siso() -> i64 {
    p_siso
}

/// Get p_ls (laststatus).
///
/// # Safety
/// Accesses C global p_ls.
#[must_use]
#[export_name = "nvim_get_p_ls"]
pub unsafe extern "C" fn get_p_ls() -> i64 {
    p_ls
}

/// Get p_stal (showtabline).
///
/// # Safety
/// Accesses C global p_stal.
#[must_use]
#[export_name = "nvim_get_p_stal"]
pub unsafe extern "C" fn get_p_stal() -> i64 {
    p_stal
}

/// Get p_ph (previewheight).
///
/// # Safety
/// Accesses C global p_ph.
#[must_use]
#[export_name = "nvim_get_p_ph"]
pub unsafe extern "C" fn get_p_ph() -> i64 {
    p_ph
}

/// Get p_pw (previewwidth).
///
/// # Safety
/// Accesses C global p_pw.
#[must_use]
#[export_name = "nvim_get_p_pw"]
pub unsafe extern "C" fn get_p_pw() -> i64 {
    p_pw
}

/// Get p_pmw (pumwidth).
///
/// # Safety
/// Accesses C global p_pmw.
#[must_use]
#[export_name = "nvim_get_p_pmw"]
pub unsafe extern "C" fn get_p_pmw() -> i64 {
    p_pmw
}

/// Get p_pvh (previewheight for popup menu).
///
/// # Safety
/// Accesses C global p_pvh.
#[must_use]
#[export_name = "nvim_get_p_pvh"]
pub unsafe extern "C" fn get_p_pvh() -> c_int {
    #[allow(clippy::cast_possible_truncation)]
    {
        p_pvh as c_int
    }
}

/// Get p_sb (splitbelow) as c_int.
///
/// # Safety
/// Accesses C global p_sb.
#[must_use]
#[export_name = "nvim_get_p_sb"]
pub unsafe extern "C" fn get_p_sb() -> c_int {
    c_int::from(p_sb)
}

/// Get p_spr (splitright) as c_int.
///
/// # Safety
/// Accesses C global p_spr.
#[must_use]
#[export_name = "nvim_get_p_spr"]
pub unsafe extern "C" fn get_p_spr() -> c_int {
    c_int::from(p_spr)
}

/// Get p_ea (equalalways) as c_int.
///
/// # Safety
/// Accesses C global p_ea.
#[must_use]
#[export_name = "nvim_get_p_ea"]
pub unsafe extern "C" fn get_p_ea() -> c_int {
    c_int::from(p_ea)
}

/// Get p_ch (cmdheight) as i64.
///
/// # Safety
/// Accesses C global p_ch.
#[must_use]
#[export_name = "nvim_get_window_p_ch"]
pub unsafe extern "C" fn get_window_p_ch() -> i64 {
    p_ch
}

/// Get p_ead (eadirection) raw pointer.
///
/// # Safety
/// Accesses C global p_ead.
#[must_use]
#[export_name = "nvim_get_p_ead"]
pub unsafe extern "C" fn get_p_ead() -> *const c_char {
    p_ead
}

/// Get first char of p_ead as c_int (0 if empty/null).
///
/// # Safety
/// Accesses C global p_ead.
#[must_use]
#[export_name = "nvim_get_p_ead_char"]
pub unsafe extern "C" fn get_p_ead_char() -> c_int {
    if p_ead.is_null() || *p_ead == 0 {
        0
    } else {
        #[allow(clippy::cast_sign_loss)]
        c_int::from(*p_ead as u8)
    }
}

/// Get p_cpo (cpoptions) raw pointer.
///
/// # Safety
/// Accesses C global p_cpo.
#[must_use]
#[export_name = "nvim_get_p_cpo"]
pub unsafe extern "C" fn get_p_cpo() -> *mut c_char {
    p_cpo
}

/// Get p_sbr (showbreak) raw pointer.
///
/// # Safety
/// Accesses C global p_sbr.
#[must_use]
#[export_name = "nvim_get_p_sbr"]
pub unsafe extern "C" fn get_p_sbr() -> *mut c_char {
    p_sbr
}

/// Get 1 if p_wbr points to NUL (empty winbar option), else 0.
///
/// # Safety
/// Accesses C global p_wbr.
#[must_use]
#[export_name = "nvim_get_p_wbr_empty"]
pub unsafe extern "C" fn get_p_wbr_empty() -> c_int {
    c_int::from(p_wbr.is_null() || *p_wbr == 0)
}
