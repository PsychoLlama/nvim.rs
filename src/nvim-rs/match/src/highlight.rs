//! Highlight helper functions
//!
//! Migrates: `check_cur_search_hl`, `get_prevcol_hl_flag`, `get_search_match_hl`

use std::ffi::c_int;

// =============================================================================
// Opaque Handle Types
// =============================================================================

/// Opaque handle to a C `win_T` structure.
#[repr(C)]
pub struct WinHandle {
    _opaque: [u8; 0],
}

/// Opaque handle to a C `matchitem_T` structure.
#[repr(C)]
pub struct MatchItemHandle {
    _opaque: [u8; 0],
}

/// Opaque handle to a C `match_T` structure.
#[repr(C)]
pub struct MatchHlHandle {
    _opaque: [u8; 0],
}

// =============================================================================
// FFI Declarations
// =============================================================================

extern "C" {
    // match_T (MatchHlHandle) accessors
    fn nvim_match_hl_get_lnum(shl: *mut MatchHlHandle) -> i32;
    fn nvim_match_hl_set_has_cursor(shl: *mut MatchHlHandle, val: c_int);
    fn nvim_match_hl_get_startcol(shl: *mut MatchHlHandle) -> i32;
    fn nvim_match_hl_get_endcol(shl: *mut MatchHlHandle) -> i32;
    fn nvim_match_hl_get_attr(shl: *mut MatchHlHandle) -> c_int;
    fn nvim_match_hl_get_is_addpos(shl: *mut MatchHlHandle) -> c_int;
    fn nvim_match_hl_rm_startpos_lnum(shl: *mut MatchHlHandle, idx: c_int) -> i32;
    fn nvim_match_hl_rm_startpos_col(shl: *mut MatchHlHandle, idx: c_int) -> i32;
    fn nvim_match_hl_rm_endpos_lnum(shl: *mut MatchHlHandle, idx: c_int) -> i32;
    fn nvim_match_hl_rm_endpos_col(shl: *mut MatchHlHandle, idx: c_int) -> i32;

    // Window accessors
    fn nvim_win_get_cursor_lnum(wp: *mut WinHandle) -> i32;
    fn nvim_win_get_cursor_col(wp: *mut WinHandle) -> i32;
    fn nvim_win_get_p_wrap(wp: *mut WinHandle) -> c_int;
    fn nvim_win_get_skipcol(wp: *mut WinHandle) -> i32;
    fn nvim_win_get_leftcol(wp: *mut WinHandle) -> i32;

    // Match list navigation
    fn nvim_match_get_head(wp: *mut WinHandle) -> *mut MatchItemHandle;
    fn nvim_match_item_next(m: *mut MatchItemHandle) -> *mut MatchItemHandle;
    fn nvim_match_item_get_priority(m: *mut MatchItemHandle) -> c_int;
    fn nvim_match_item_get_hl(m: *mut MatchItemHandle) -> *mut MatchHlHandle;
}

/// Search highlight priority constant.
const SEARCH_HL_PRIORITY: c_int = 0;

// =============================================================================
// check_cur_search_hl
// =============================================================================

/// Update `shl->has_cursor` based on the match in `shl` and the cursor position.
///
/// # Safety
///
/// `wp` and `shl` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_check_cur_search_hl(wp: *mut WinHandle, shl: *mut MatchHlHandle) {
    let linecount = nvim_match_hl_rm_endpos_lnum(shl, 0) - nvim_match_hl_rm_startpos_lnum(shl, 0);
    let shl_lnum = nvim_match_hl_get_lnum(shl);
    let cursor_lnum = nvim_win_get_cursor_lnum(wp);
    let cursor_col = nvim_win_get_cursor_col(wp);
    let start_col = nvim_match_hl_rm_startpos_col(shl, 0);
    let end_col = nvim_match_hl_rm_endpos_col(shl, 0);

    let has_cursor = cursor_lnum >= shl_lnum
        && cursor_lnum <= shl_lnum + linecount
        && (cursor_lnum > shl_lnum || cursor_col >= start_col)
        && (cursor_lnum < shl_lnum + linecount || cursor_col < end_col);

    nvim_match_hl_set_has_cursor(shl, c_int::from(has_cursor));
}

// =============================================================================
// get_prevcol_hl_flag
// =============================================================================

/// Check if the previous column should be highlighted for search/match.
///
/// # Safety
///
/// `wp` and `search_hl` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_get_prevcol_hl_flag(
    wp: *mut WinHandle,
    search_hl: *mut MatchHlHandle,
    curcol: i32,
) -> c_int {
    let mut prevcol = curcol;

    // We're not really at that column when skipping some text
    let skip_col = if nvim_win_get_p_wrap(wp) != 0 {
        nvim_win_get_skipcol(wp)
    } else {
        nvim_win_get_leftcol(wp)
    };
    if skip_col > prevcol {
        prevcol += 1;
    }

    // Check search_hl
    let is_addpos = nvim_match_hl_get_is_addpos(search_hl) != 0;
    let startcol = nvim_match_hl_get_startcol(search_hl);
    let endcol = nvim_match_hl_get_endcol(search_hl);
    if !is_addpos && (prevcol == startcol || (prevcol > startcol && endcol == i32::MAX)) {
        return 1;
    }

    // Check match list
    let mut cur = nvim_match_get_head(wp);
    while !cur.is_null() {
        let hl = nvim_match_item_get_hl(cur);
        let m_is_addpos = nvim_match_hl_get_is_addpos(hl) != 0;
        let m_startcol = nvim_match_hl_get_startcol(hl);
        let m_endcol = nvim_match_hl_get_endcol(hl);
        if !m_is_addpos && (prevcol == m_startcol || (prevcol > m_startcol && m_endcol == i32::MAX))
        {
            return 1;
        }
        cur = nvim_match_item_next(cur);
    }

    0
}

// =============================================================================
// get_search_match_hl
// =============================================================================

/// Get highlighting for the char after the text from search/match highlighting.
///
/// # Safety
///
/// `wp`, `search_hl`, and `char_attr` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_get_search_match_hl(
    wp: *mut WinHandle,
    search_hl: *mut MatchHlHandle,
    col: i32,
    char_attr: *mut c_int,
) {
    let mut cur = nvim_match_get_head(wp);
    let mut shl_flag = false;

    while !cur.is_null() || !shl_flag {
        let shl;
        if !shl_flag && (cur.is_null() || nvim_match_item_get_priority(cur) > SEARCH_HL_PRIORITY) {
            shl = search_hl;
            shl_flag = true;
        } else {
            shl = nvim_match_item_get_hl(cur);
        }

        let startcol = nvim_match_hl_get_startcol(shl);
        let is_addpos = nvim_match_hl_get_is_addpos(shl) != 0;
        if col - 1 == startcol && (shl == search_hl || !is_addpos) {
            *char_attr = nvim_match_hl_get_attr(shl);
        }

        if shl != search_hl && !cur.is_null() {
            cur = nvim_match_item_next(cur);
        }
    }
}
