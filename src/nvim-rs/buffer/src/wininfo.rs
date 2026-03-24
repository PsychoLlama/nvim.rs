//! Window-info (`WinInfo`) cluster helpers.
//!
//! Implements `buflist_setfpos`, `get_winopts`, `buflist_findfmark`,
//! and the private `wininfo_other_tab_diff` / `find_wininfo` helpers.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(dead_code)] // Some extern fns declared for completeness

use std::ffi::{c_int, c_void};

use crate::{BufHandle, WinHandle};

// =============================================================================
// Opaque handle for WinInfo
// =============================================================================

/// Opaque handle to a `WinInfo` struct.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WinInfoHandle(*mut c_void);

impl WinInfoHandle {
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    // WinInfo vector accessors
    fn nvim_buf_wininfo_count(buf: BufHandle) -> usize;
    fn nvim_buf_wininfo_get(buf: BufHandle, i: usize) -> WinInfoHandle;
    fn nvim_wininfo_get_win(wip: WinInfoHandle) -> WinHandle;
    fn nvim_wininfo_get_optset(wip: WinInfoHandle) -> bool;
    fn nvim_wininfo_get_wo_diff(wip: WinInfoHandle) -> bool;
    fn nvim_wininfo_get_changelistidx(wip: WinInfoHandle) -> c_int;
    fn nvim_wininfo_get_mark_ptr(wip: WinInfoHandle) -> *mut c_void; // fmark_T*
    fn nvim_wininfo_get_fold_manual(wip: WinInfoHandle) -> bool;

    // WinInfo compound operations
    fn nvim_wininfo_win_in_curtab(wip: WinInfoHandle) -> bool;
    fn nvim_buf_wininfo_find_and_detach(
        buf: BufHandle,
        win: WinHandle,
        copy_options: bool,
        lnum_inout: *mut c_int,
    ) -> WinInfoHandle;
    fn nvim_buf_wininfo_prepend(buf: BufHandle, wip: WinInfoHandle);
    fn nvim_wininfo_set_mark(wip: WinInfoHandle, lnum: c_int, col: c_int, win: WinHandle);
    fn nvim_wininfo_copy_from_win(wip: WinInfoHandle, win: WinHandle);
    fn nvim_get_winopts_apply(wip: WinInfoHandle, buf: BufHandle) -> c_int;

    // curwin/win field accessors
    fn nvim_clear_winopt_curwin();
    fn nvim_curwin_set_changelistidx(val: c_int);
    fn nvim_curwin_config_is_minimal() -> bool;
    fn nvim_get_p_fdls() -> i64;
    fn nvim_curwin_set_p_fdl(val: c_int);
    fn nvim_didset_window_options_curwin();
    fn nvim_win_set_minimal_style_curwin();
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_win_get_changelistidx(wp: WinHandle) -> c_int;
    fn nvim_wininfo_set_changelistidx(wip: WinInfoHandle, val: c_int);
    fn nvim_wininfo_set_optset(wip: WinInfoHandle, val: bool);
    fn nvim_wininfo_set_fold_manual(wip: WinInfoHandle, val: bool);

    // Window buffer accessor
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;

    // Folding helpers (Rust exports, already exported from fold crate)
    fn rs_clearFolding(win: WinHandle);

    // Static no_position fmark_T accessor
    fn nvim_get_no_position_ptr() -> *mut c_void;
}

// =============================================================================
// Private helpers
// =============================================================================

/// Check that `wip` has 'diff' set and the diff is only for another tab page.
/// That's because a diff is local to a tab page.
///
/// # Safety
/// Must be called on main thread with valid Neovim state.
#[allow(clippy::if_not_else)]
unsafe fn wininfo_other_tab_diff(wip: WinInfoHandle) -> bool {
    if !nvim_wininfo_get_wo_diff(wip) {
        return false;
    }
    // Return false if wip->wi_win is in the current tab page
    !nvim_wininfo_win_in_curtab(wip)
}

/// Find info for the current window in buffer `buf`.
/// If not found, return the info for the most recently used window.
///
/// - `need_options`: skip entries where `wi_optset` is false.
/// - `skip_diff_buffer`: avoid windows with 'diff' set in another tab page.
///
/// Returns a null handle when there is no info.
///
/// # Safety
/// Must be called on main thread with valid Neovim state.
unsafe fn find_wininfo(
    buf: BufHandle,
    need_options: bool,
    skip_diff_buffer: bool,
) -> WinInfoHandle {
    let curwin = nvim_get_curwin();
    let count = nvim_buf_wininfo_count(buf);

    // First pass: look for curwin
    for i in 0..count {
        let wip = nvim_buf_wininfo_get(buf, i);
        let win = nvim_wininfo_get_win(wip);
        if win == curwin
            && (!skip_diff_buffer || !wininfo_other_tab_diff(wip))
            && (!need_options || nvim_wininfo_get_optset(wip))
        {
            return wip;
        }
    }

    // Second pass: fall back to first usable entry
    if skip_diff_buffer {
        for i in 0..count {
            let wip = nvim_buf_wininfo_get(buf, i);
            let wip_win = nvim_wininfo_get_win(wip);
            let win_has_buf = !wip_win.0.is_null() && nvim_win_get_buffer(wip_win) == buf;
            if !wininfo_other_tab_diff(wip)
                && (!need_options || nvim_wininfo_get_optset(wip) || win_has_buf)
            {
                return wip;
            }
        }
    } else if count > 0 {
        return nvim_buf_wininfo_get(buf, 0);
    }

    WinInfoHandle(std::ptr::null_mut())
}

// =============================================================================
// Public exported functions
// =============================================================================

/// Set the last cursor position in the `WinInfo` list for the current window.
///
/// Mirrors C `buflist_setfpos`.
///
/// # Safety
/// Must be called on the main thread with valid Neovim state.
#[unsafe(export_name = "buflist_setfpos")]
pub unsafe extern "C" fn rs_buflist_setfpos(
    buf: BufHandle,
    win: WinHandle,
    lnum: c_int,
    col: c_int,
    copy_options: bool,
) {
    let mut lnum_mut = lnum;
    let wip =
        nvim_buf_wininfo_find_and_detach(buf, win, copy_options, std::ptr::addr_of_mut!(lnum_mut));

    if lnum_mut != 0 {
        nvim_wininfo_set_mark(wip, lnum_mut, col, win);
    }
    if !win.0.is_null() {
        let changelistidx = nvim_win_get_changelistidx(win);
        nvim_wininfo_set_changelistidx(wip, changelistidx);
    }
    if copy_options && !win.0.is_null() {
        nvim_wininfo_copy_from_win(wip, win);
    }

    // Insert entry at front of vector
    nvim_buf_wininfo_prepend(buf, wip);
}

/// Reset the local window options to values last used in this window.
///
/// Mirrors C `get_winopts`.
///
/// # Safety
/// Must be called on the main thread with valid Neovim state.
#[unsafe(export_name = "get_winopts")]
pub unsafe extern "C" fn rs_get_winopts(buf: BufHandle) {
    nvim_clear_winopt_curwin();
    rs_clearFolding(nvim_get_curwin());

    let wip = find_wininfo(buf, true, true);
    nvim_get_winopts_apply(wip, buf);

    if !wip.is_null() {
        let changelistidx = nvim_wininfo_get_changelistidx(wip);
        nvim_curwin_set_changelistidx(changelistidx);
    }

    if nvim_curwin_config_is_minimal() {
        nvim_didset_window_options_curwin();
        nvim_win_set_minimal_style_curwin();
    }

    // Set 'foldlevel' to 'foldlevelstart' if it's not negative.
    let p_fdls = nvim_get_p_fdls();
    if p_fdls >= 0 {
        nvim_curwin_set_p_fdl(p_fdls as c_int);
    }
    nvim_didset_window_options_curwin();
}

/// Find the mark for the buffer `buf` for the current window.
///
/// Returns a pointer to a static `no_position` if no position is found.
///
/// Mirrors C `buflist_findfmark`.
///
/// # Safety
/// Must be called on the main thread with valid Neovim state.
#[must_use]
#[unsafe(export_name = "buflist_findfmark")]
pub unsafe extern "C" fn rs_buflist_findfmark(buf: BufHandle) -> *mut c_void {
    let wip = find_wininfo(buf, false, false);
    if wip.is_null() {
        nvim_get_no_position_ptr()
    } else {
        nvim_wininfo_get_mark_ptr(wip)
    }
}
