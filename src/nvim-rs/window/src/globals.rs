//! Global variable accessors for Neovim window management.
//!
//! This module provides Rust `#[export_name]` implementations of C accessor
//! functions that read and write global Neovim variables (curwin, curbuf,
//! curtab, option globals, etc.).
//!
//! All C functions replaced by this module are one-liners that simply read
//! or write a global variable.

use std::ffi::{c_char, c_int, c_uint};

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
    static mut firstbuf: BufHandle; // first buffer in list

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
    static mut p_sel: *mut c_char; // 'selection' option

    // Additional globals for Phase 1 elimination
    static mut RedrawingDisabled: c_int;
    static mut p_acd: c_int; // 'autochdir'
    static mut p_tpm: i64; // 'tabpagemax' (OptInt = i64)
    static mut p_confirm: c_int; // 'confirm'
    static mut p_write: c_int; // 'write'
    static mut starting: c_int; // NO_SCREEN / JUST_STARTED / ...
    static mut spell_redraw_lnum: i32; // linenr_T
    static mut dy_flags: c_uint; // 'display' option flags
    static mut empty_string_option: c_char; // char[] empty_string_option
    static mut au_pending_free_win: WinHandle;
    static mut postponed_split_tab: c_int;
    static mut last_chdir_reason: *mut c_char;
    static mut cmdline_win: WinHandle;
    static mut cmdwin_win: WinHandle; // window of cmdline window or NULL
    static mut cmdwin_old_curwin: WinHandle; // curwin before opening cmdline window

    // Phase 3 globals
    static mut got_int: bool; // interrupt flag
    static mut display_tick: u64; // disptick_T (uint64_t)
    static mut VIsual_active: bool; // visual mode active
    static mut cmdline_row: c_int; // row of command line
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

/// Set curwin = wp and curbuf = wp->w_buffer (if wp is non-null).
///
/// Replaces the C shim `nvim_set_curwin_from_wp()` in window_shim.c.
///
/// # Safety
/// `wp` must be a valid window pointer or null.
#[export_name = "nvim_set_curwin_from_wp"]
pub unsafe extern "C" fn set_curwin_from_wp(wp: WinHandle) {
    if !wp.is_null() {
        curwin = wp;
        curbuf = nvim_win_get_w_buffer(wp);
    }
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

// -------------------------------------------------------------------------
// Phase 1: Global variable accessors (replacing C one-liners)
// -------------------------------------------------------------------------

/// Get RedrawingDisabled.
///
/// # Safety
/// Accesses C global RedrawingDisabled.
#[must_use]
#[export_name = "nvim_get_RedrawingDisabled"]
pub unsafe extern "C" fn get_redrawing_disabled() -> c_int {
    RedrawingDisabled
}

/// Set RedrawingDisabled.
///
/// # Safety
/// Accesses C global RedrawingDisabled.
#[export_name = "nvim_set_RedrawingDisabled"]
pub unsafe extern "C" fn set_redrawing_disabled(val: c_int) {
    RedrawingDisabled = val;
}

/// Increment RedrawingDisabled.
///
/// # Safety
/// Accesses C global RedrawingDisabled.
#[export_name = "nvim_inc_RedrawingDisabled"]
pub unsafe extern "C" fn inc_redrawing_disabled() {
    RedrawingDisabled += 1;
}

/// Decrement RedrawingDisabled.
///
/// # Safety
/// Accesses C global RedrawingDisabled.
#[export_name = "nvim_dec_RedrawingDisabled"]
pub unsafe extern "C" fn dec_redrawing_disabled() {
    RedrawingDisabled -= 1;
}

/// Get p_acd ('autochdir') as c_int (1 if true, 0 if false).
///
/// # Safety
/// Accesses C global p_acd.
#[must_use]
#[export_name = "nvim_get_p_acd"]
pub unsafe extern "C" fn get_p_acd() -> c_int {
    c_int::from(p_acd != 0)
}

/// Get p_tpm ('tabpagemax').
///
/// # Safety
/// Accesses C global p_tpm.
#[must_use]
#[export_name = "nvim_get_p_tpm"]
pub unsafe extern "C" fn get_p_tpm() -> i64 {
    p_tpm
}

/// Get p_confirm ('confirm') as c_int (1 if true, 0 if false).
///
/// # Safety
/// Accesses C global p_confirm.
#[must_use]
#[export_name = "nvim_get_p_confirm"]
pub unsafe extern "C" fn get_p_confirm() -> c_int {
    c_int::from(p_confirm != 0)
}

/// Get p_write ('write') as c_int (1 if true, 0 if false).
///
/// # Safety
/// Accesses C global p_write.
#[must_use]
#[export_name = "nvim_get_p_write"]
pub unsafe extern "C" fn get_p_write() -> c_int {
    c_int::from(p_write != 0)
}

/// Get starting.
///
/// # Safety
/// Accesses C global starting.
#[must_use]
#[export_name = "nvim_get_starting"]
pub unsafe extern "C" fn get_starting() -> c_int {
    starting
}

/// Get spell_redraw_lnum.
///
/// # Safety
/// Accesses C global spell_redraw_lnum.
#[must_use]
#[export_name = "nvim_get_spell_redraw_lnum"]
pub unsafe extern "C" fn get_spell_redraw_lnum() -> i32 {
    spell_redraw_lnum
}

/// Set spell_redraw_lnum.
///
/// # Safety
/// Accesses C global spell_redraw_lnum.
#[export_name = "nvim_set_spell_redraw_lnum"]
pub unsafe extern "C" fn set_spell_redraw_lnum(val: i32) {
    spell_redraw_lnum = val;
}

/// Get dy_flags ('display' option flags).
///
/// # Safety
/// Accesses C global dy_flags.
#[must_use]
#[export_name = "nvim_get_dy_flags"]
pub unsafe extern "C" fn get_dy_flags() -> c_int {
    #[allow(clippy::cast_possible_wrap)]
    {
        dy_flags as c_int
    }
}

/// Get empty_string_option pointer.
///
/// # Safety
/// Accesses C global empty_string_option.
#[must_use]
#[export_name = "nvim_get_empty_string_option"]
pub unsafe extern "C" fn get_empty_string_option() -> *mut c_char {
    &raw mut empty_string_option
}

/// Get au_pending_free_win.
///
/// # Safety
/// Accesses C global au_pending_free_win.
#[must_use]
#[export_name = "nvim_get_au_pending_free_win"]
pub unsafe extern "C" fn get_au_pending_free_win() -> WinHandle {
    au_pending_free_win
}

/// Set au_pending_free_win.
///
/// # Safety
/// Accesses C global au_pending_free_win.
#[export_name = "nvim_set_au_pending_free_win"]
pub unsafe extern "C" fn set_au_pending_free_win(wp: WinHandle) {
    au_pending_free_win = wp;
}

/// Get postponed_split_tab.
///
/// # Safety
/// Accesses C global postponed_split_tab.
#[must_use]
#[export_name = "nvim_get_postponed_split_tab"]
pub unsafe extern "C" fn get_postponed_split_tab() -> c_int {
    postponed_split_tab
}

/// Set postponed_split_tab.
///
/// # Safety
/// Accesses C global postponed_split_tab.
#[export_name = "nvim_set_postponed_split_tab"]
pub unsafe extern "C" fn set_postponed_split_tab(val: c_int) {
    postponed_split_tab = val;
}

/// Set firstwin = NULL.
///
/// # Safety
/// Accesses C global firstwin.
#[export_name = "nvim_set_firstwin_null"]
pub unsafe extern "C" fn set_firstwin_null() {
    firstwin = WinHandle::null();
}

/// Set lastwin = NULL.
///
/// # Safety
/// Accesses C global lastwin.
#[export_name = "nvim_set_lastwin_null"]
pub unsafe extern "C" fn set_lastwin_null() {
    lastwin = WinHandle::null();
}

/// Set lastused_tabpage from Rust (alias for nvim_set_lastused_tabpage).
///
/// # Safety
/// `tp` must be a valid tabpage pointer or null.
#[export_name = "nvim_set_lastused_tabpage_from_rust"]
pub unsafe extern "C" fn set_lastused_tabpage_from_rust(tp: TabpageHandle) {
    lastused_tabpage = tp;
}

/// Get curwin as raw pointer (alias for nvim_get_curwin).
///
/// # Safety
/// Accesses C global curwin.
#[must_use]
#[export_name = "nvim_get_curwin_ptr"]
pub unsafe extern "C" fn get_curwin_ptr() -> WinHandle {
    curwin
}

/// Set curwin from raw pointer (alias for nvim_set_curwin).
///
/// # Safety
/// `wp` must be a valid window pointer or null.
#[export_name = "nvim_set_curwin_ptr"]
pub unsafe extern "C" fn set_curwin_ptr(wp: WinHandle) {
    curwin = wp;
}

/// Get firstwin as raw pointer (alias for nvim_get_firstwin).
///
/// # Safety
/// Accesses C global firstwin.
#[must_use]
#[export_name = "nvim_get_firstwin_ptr"]
pub unsafe extern "C" fn get_firstwin_ptr() -> WinHandle {
    firstwin
}

/// Get curbuf as raw pointer (alias for nvim_get_curbuf).
///
/// # Safety
/// Accesses C global curbuf.
#[must_use]
#[export_name = "nvim_get_curbuf_ptr"]
pub unsafe extern "C" fn get_curbuf_ptr() -> BufHandle {
    curbuf
}

/// Set last_chdir_reason = NULL.
///
/// # Safety
/// Accesses C global last_chdir_reason.
#[export_name = "nvim_set_last_chdir_reason_null"]
pub unsafe extern "C" fn set_last_chdir_reason_null() {
    last_chdir_reason = std::ptr::null_mut();
}

/// Set cmdline_win = NULL.
///
/// # Safety
/// Accesses C global cmdline_win.
#[export_name = "nvim_set_cmdline_win_null"]
pub unsafe extern "C" fn set_cmdline_win_null() {
    cmdline_win = WinHandle::null();
}

/// Get Rows (screen height).
///
/// # Safety
/// Accesses C global Rows.
#[must_use]
#[export_name = "nvim_get_Rows"]
pub unsafe extern "C" fn get_rows() -> c_int {
    Rows
}

/// Get Columns (screen width).
///
/// # Safety
/// Accesses C global Columns.
#[must_use]
#[export_name = "nvim_get_Columns"]
pub unsafe extern "C" fn get_columns() -> c_int {
    Columns
}

/// Set p_ch (cmdheight). Alias for existing set_p_ch functionality.
///
/// # Safety
/// Accesses C global p_ch.
#[export_name = "nvim_set_p_ch"]
pub unsafe extern "C" fn set_p_ch_alias(val: i64) {
    p_ch = val;
}

/// Get p_sel ('selection' option) pointer.
///
/// Replaces the C shim `nvim_get_p_sel()` in window_shim.c.
///
/// # Safety
/// Accesses C global p_sel.
#[must_use]
#[export_name = "nvim_get_p_sel"]
pub unsafe extern "C" fn get_p_sel() -> *const c_char {
    p_sel
}

// -------------------------------------------------------------------------
// Phase 3: Constants and global accessors for cross-crate use
// -------------------------------------------------------------------------

/// STATUS_HEIGHT constant (height of status line = 1).
#[must_use]
#[export_name = "nvim_get_status_height_const"]
pub const extern "C" fn get_status_height_const() -> c_int {
    1
}

/// UPD_VALID constant (= 10, buffer not changed).
#[must_use]
#[export_name = "nvim_get_upd_valid"]
pub const extern "C" fn get_upd_valid() -> c_int {
    10
}

/// UPD_NOT_VALID constant (= 40, buffer needs complete redraw).
#[must_use]
#[export_name = "nvim_get_upd_not_valid"]
pub const extern "C" fn get_upd_not_valid() -> c_int {
    40
}

/// Get VIsual_active as c_int (1 if true, 0 if false).
///
/// # Safety
/// Accesses C global VIsual_active.
#[must_use]
#[export_name = "nvim_VIsual_active"]
pub unsafe extern "C" fn get_visual_active() -> c_int {
    c_int::from(VIsual_active)
}

/// Get got_int as c_int (1 if true, 0 if false).
///
/// # Safety
/// Accesses C global got_int.
#[must_use]
#[export_name = "nvim_syn_get_got_int"]
pub unsafe extern "C" fn get_got_int() -> c_int {
    c_int::from(got_int)
}

/// Get display_tick as c_int (truncated from u64).
///
/// # Safety
/// Accesses C global display_tick.
#[must_use]
#[export_name = "nvim_syn_get_display_tick"]
pub unsafe extern "C" fn get_display_tick() -> c_int {
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    {
        display_tick as c_int
    }
}

/// Get Rows for syntax use.
///
/// # Safety
/// Accesses C global Rows.
#[must_use]
#[export_name = "nvim_syn_get_rows"]
pub unsafe extern "C" fn get_syn_rows() -> c_int {
    Rows
}

/// Get Columns for syntax use.
///
/// # Safety
/// Accesses C global Columns.
#[must_use]
#[export_name = "nvim_syn_get_columns"]
pub unsafe extern "C" fn get_syn_columns() -> c_int {
    Columns
}

/// Update cmdline_row = Rows - p_ch.
///
/// Replaces the C shim `nvim_update_cmdline_row()` in window_shim.c.
///
/// # Safety
/// Accesses C globals Rows, p_ch, cmdline_row.
#[export_name = "nvim_update_cmdline_row"]
pub unsafe extern "C" fn update_cmdline_row() {
    #[allow(clippy::cast_possible_truncation)]
    {
        cmdline_row = Rows - p_ch as c_int;
    }
}

/// Get firstbuf (first buffer in list).
///
/// Replaces the C shim `nvim_get_firstbuf_wrapper()` in window_shim.c.
///
/// # Safety
/// Accesses C global firstbuf.
#[must_use]
#[export_name = "nvim_get_firstbuf_wrapper"]
pub unsafe extern "C" fn get_firstbuf_wrapper() -> BufHandle {
    firstbuf
}

/// Get cmdwin_win (window of cmdline window, or null).
///
/// Replaces the C shim `nvim_get_cmdwin_win()` in window_shim.c.
///
/// # Safety
/// Accesses C global cmdwin_win.
#[must_use]
#[export_name = "nvim_get_cmdwin_win"]
pub unsafe extern "C" fn get_cmdwin_win() -> WinHandle {
    cmdwin_win
}

/// Get cmdwin_old_curwin (curwin before opening cmdline window, or null).
///
/// Replaces the C shim `nvim_get_cmdwin_old_curwin()` in window_shim.c.
///
/// # Safety
/// Accesses C global cmdwin_old_curwin.
#[must_use]
#[export_name = "nvim_get_cmdwin_old_curwin"]
pub unsafe extern "C" fn get_cmdwin_old_curwin() -> WinHandle {
    cmdwin_old_curwin
}

/// Check if win == cmdline_win (ext_cmdline window).
///
/// Replaces the C shim `nvim_win_is_cmdline_win()` in window_shim.c.
///
/// # Safety
/// Accesses C global cmdline_win.
#[must_use]
#[export_name = "nvim_win_is_cmdline_win"]
pub unsafe extern "C" fn win_is_cmdline_win(win: WinHandle) -> c_int {
    c_int::from(!win.is_null() && win.as_ptr() == cmdline_win.as_ptr())
}

/// Set cmdwin_win.
///
/// Replaces the C shim `nvim_set_cmdwin_win()` in cmdwin_shim.c.
///
/// # Safety
/// Accesses C global cmdwin_win.
#[export_name = "nvim_set_cmdwin_win"]
pub unsafe extern "C" fn set_cmdwin_win(wp: WinHandle) {
    cmdwin_win = wp;
}

/// Set cmdwin_old_curwin.
///
/// Replaces the C shim `nvim_set_cmdwin_old_curwin()` in cmdwin_shim.c.
///
/// # Safety
/// Accesses C global cmdwin_old_curwin.
#[export_name = "nvim_set_cmdwin_old_curwin"]
pub unsafe extern "C" fn set_cmdwin_old_curwin(wp: WinHandle) {
    cmdwin_old_curwin = wp;
}

/// Check if wp is the cmdline window (== cmdline_win).
///
/// Replaces the C shim `nvim_win_is_cmdwin()` in window_shim.c.
///
/// # Safety
/// Accesses C global cmdline_win.
#[must_use]
#[export_name = "nvim_win_is_cmdwin"]
pub unsafe extern "C" fn win_is_cmdwin(wp: WinHandle) -> c_int {
    c_int::from(!wp.is_null() && wp.as_ptr() == cmdline_win.as_ptr())
}
