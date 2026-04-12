//! Highlight preparation and update
//!
//! Migrates: `prepare_search_hl`, `prepare_search_hl_line`, `update_search_hl`

use std::ffi::c_int;
use std::ptr;

use crate::HLF_LC;

// =============================================================================
// Opaque Handle Types
// =============================================================================

#[repr(C)]
pub struct WinHandle {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct MatchHlHandle {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct MatchItemHandle {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct RegProgHandle {
    _opaque: [u8; 0],
}

// =============================================================================
// FFI Declarations
// =============================================================================

extern "C" {
    // match_T accessors
    fn nvim_match_hl_get_lnum(shl: *mut MatchHlHandle) -> i32;
    fn nvim_match_hl_get_startcol(shl: *mut MatchHlHandle) -> i32;
    fn nvim_match_hl_set_startcol(shl: *mut MatchHlHandle, col: i32);
    fn nvim_match_hl_get_endcol(shl: *mut MatchHlHandle) -> i32;
    fn nvim_match_hl_set_endcol(shl: *mut MatchHlHandle, col: i32);
    fn nvim_match_hl_get_attr(shl: *mut MatchHlHandle) -> c_int;
    fn nvim_match_hl_get_attr_cur(shl: *mut MatchHlHandle) -> c_int;
    fn nvim_match_hl_set_attr_cur(shl: *mut MatchHlHandle, attr: c_int);
    fn nvim_match_hl_set_is_addpos(shl: *mut MatchHlHandle, val: c_int);
    fn nvim_match_hl_get_has_cursor(shl: *mut MatchHlHandle) -> c_int;
    fn nvim_match_hl_set_has_cursor(shl: *mut MatchHlHandle, val: c_int);
    fn nvim_match_hl_get_first_lnum(shl: *mut MatchHlHandle) -> i32;
    fn nvim_match_hl_set_first_lnum(shl: *mut MatchHlHandle, lnum: i32);
    fn nvim_match_hl_get_regprog(shl: *mut MatchHlHandle) -> *mut RegProgHandle;
    fn nvim_match_hl_rm_startpos_lnum(shl: *mut MatchHlHandle, idx: c_int) -> i32;
    fn nvim_match_hl_rm_startpos_col(shl: *mut MatchHlHandle, idx: c_int) -> i32;
    fn nvim_match_hl_rm_endpos_lnum(shl: *mut MatchHlHandle, idx: c_int) -> i32;
    fn nvim_match_hl_rm_endpos_col(shl: *mut MatchHlHandle, idx: c_int) -> i32;

    // matchitem_T accessors
    fn nvim_match_get_head(wp: *mut WinHandle) -> *mut MatchItemHandle;
    fn nvim_match_item_next(m: *mut MatchItemHandle) -> *mut MatchItemHandle;
    fn nvim_match_item_get_priority(m: *mut MatchItemHandle) -> c_int;
    fn nvim_match_item_get_hl(m: *mut MatchItemHandle) -> *mut MatchHlHandle;
    fn nvim_match_item_get_hlg_id(m: *mut MatchItemHandle) -> c_int;
    fn nvim_match_item_get_conceal_char(m: *mut MatchItemHandle) -> c_int;
    fn nvim_match_item_get_pos_cur(m: *mut MatchItemHandle) -> c_int;
    fn nvim_match_item_set_pos_cur(m: *mut MatchItemHandle, cur: c_int);

    // Window accessors
    fn nvim_match_win_get_topline(wp: *mut WinHandle) -> i32;
    fn nvim_win_get_p_list(wp: *mut WinHandle) -> c_int;

    // Function wrappers
    fn nvim_match_re_multiline(rp: *mut RegProgHandle) -> c_int;
    fn nvim_match_hasFolding(wp: *mut WinHandle, lnum: i32) -> c_int;
    fn nvim_match_ml_get_buf_line(wp: *mut WinHandle, lnum: i32, line_out: *mut *mut u8);
    fn nvim_match_line_byte_at(line: *mut u8, col: i32) -> c_int;
    fn nvim_match_utfc_ptr2len_at(line: *mut u8, col: i32) -> c_int;
    fn nvim_match_syn_name2id_conceal() -> c_int;
    fn nvim_match_set_search_hl_has_cursor_lnum(lnum: i32);
    fn nvim_win_hl_attr(wp: *mut WinHandle, hlf: c_int) -> c_int;
    // nvim_match_get_HLF_LC() removed — use crate::HLF_LC constant

    // Delegated functions
    fn rs_next_search_hl(
        win: *mut WinHandle,
        search_hl: *mut MatchHlHandle,
        shl: *mut MatchHlHandle,
        lnum: i32,
        mincol: i32,
        cur: *mut MatchItemHandle,
    );
    fn rs_check_cur_search_hl(wp: *mut WinHandle, shl: *mut MatchHlHandle);
}

/// MAXCOL value from C (0x7fffffff).
const MAXCOL: i32 = 0x7fff_ffff;

/// Search highlight priority constant.
const SEARCH_HL_PRIORITY: c_int = 0;

// =============================================================================
// prepare_search_hl
// =============================================================================

/// Advance to the match in window `wp` line `lnum` or past it.
///
/// # Safety
///
/// All pointers must be valid.
#[export_name = "prepare_search_hl"]
pub unsafe extern "C" fn rs_prepare_search_hl(
    wp: *mut WinHandle,
    search_hl: *mut MatchHlHandle,
    lnum: i32,
) {
    let mut cur = nvim_match_get_head(wp);
    let mut shl_flag = false;
    let topline = nvim_match_win_get_topline(wp);

    while !cur.is_null() || !shl_flag {
        let shl;
        if shl_flag {
            shl = nvim_match_item_get_hl(cur);
        } else {
            shl = search_hl;
            shl_flag = true;
        }

        let regprog = nvim_match_hl_get_regprog(shl);
        let shl_lnum = nvim_match_hl_get_lnum(shl);
        if !regprog.is_null() && shl_lnum == 0 && nvim_match_re_multiline(regprog) != 0 {
            let first_lnum = nvim_match_hl_get_first_lnum(shl);
            if first_lnum == 0 {
                let mut fl = lnum;
                while fl > topline {
                    if nvim_match_hasFolding(wp, fl - 1) != 0 {
                        break;
                    }
                    fl -= 1;
                }
                nvim_match_hl_set_first_lnum(shl, fl);
            }
            if !cur.is_null() {
                nvim_match_item_set_pos_cur(cur, 0);
            }
            let mut pos_inprogress = true;
            let mut n: i32 = 0;
            loop {
                let fl = nvim_match_hl_get_first_lnum(shl);
                let rp = nvim_match_hl_get_regprog(shl);
                if fl >= lnum || (rp.is_null() && (cur.is_null() || !pos_inprogress)) {
                    break;
                }
                let cur_for_search = if ptr::eq(shl, search_hl) {
                    ptr::null_mut()
                } else {
                    cur
                };
                rs_next_search_hl(wp, search_hl, shl, fl, n, cur_for_search);
                pos_inprogress = !cur.is_null() && nvim_match_item_get_pos_cur(cur) != 0;
                let new_lnum = nvim_match_hl_get_lnum(shl);
                if new_lnum != 0 {
                    let new_first = new_lnum + nvim_match_hl_rm_endpos_lnum(shl, 0)
                        - nvim_match_hl_rm_startpos_lnum(shl, 0);
                    nvim_match_hl_set_first_lnum(shl, new_first);
                    n = nvim_match_hl_rm_endpos_col(shl, 0);
                } else {
                    nvim_match_hl_set_first_lnum(shl, fl + 1);
                    n = 0;
                }
            }
        }
        if !ptr::eq(shl, search_hl) && !cur.is_null() {
            cur = nvim_match_item_next(cur);
        }
    }
}

// =============================================================================
// prepare_search_hl_line
// =============================================================================

/// Prepare for 'hlsearch' and match highlighting in one window line.
///
/// # Safety
///
/// All pointers must be valid. `line` is a `char **` (pointer to line pointer).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_prepare_search_hl_line(
    wp: *mut WinHandle,
    lnum: i32,
    mincol: i32,
    line: *mut *mut u8,
    search_hl: *mut MatchHlHandle,
    search_attr: *mut c_int,
    search_attr_from_match: *mut c_int,
) -> c_int {
    let mut cur = nvim_match_get_head(wp);
    let mut shl_flag = false;
    let mut area_highlighting = false;

    while !cur.is_null() || !shl_flag {
        let shl;
        if shl_flag {
            shl = nvim_match_item_get_hl(cur);
        } else {
            shl = search_hl;
            shl_flag = true;
        }

        nvim_match_hl_set_startcol(shl, MAXCOL);
        nvim_match_hl_set_endcol(shl, MAXCOL);
        nvim_match_hl_set_attr_cur(shl, 0);
        nvim_match_hl_set_is_addpos(shl, 0);
        nvim_match_hl_set_has_cursor(shl, 0);
        if !cur.is_null() {
            nvim_match_item_set_pos_cur(cur, 0);
        }

        let cur_for_search = if ptr::eq(shl, search_hl) {
            ptr::null_mut()
        } else {
            cur
        };
        rs_next_search_hl(wp, search_hl, shl, lnum, mincol, cur_for_search);

        // Need to get the line again, a multi-line regexp may have made it invalid.
        nvim_match_ml_get_buf_line(wp, lnum, line);

        let shl_lnum = nvim_match_hl_get_lnum(shl);
        if shl_lnum != 0 && shl_lnum <= lnum {
            if shl_lnum == lnum {
                nvim_match_hl_set_startcol(shl, nvim_match_hl_rm_startpos_col(shl, 0));
            } else {
                nvim_match_hl_set_startcol(shl, 0);
            }
            if lnum
                == shl_lnum + nvim_match_hl_rm_endpos_lnum(shl, 0)
                    - nvim_match_hl_rm_startpos_lnum(shl, 0)
            {
                nvim_match_hl_set_endcol(shl, nvim_match_hl_rm_endpos_col(shl, 0));
            } else {
                nvim_match_hl_set_endcol(shl, MAXCOL);
            }

            // check if the cursor is in the match before changing the columns
            if ptr::eq(shl, search_hl) {
                rs_check_cur_search_hl(wp, shl);
            }

            // Highlight one character for an empty match.
            let startcol = nvim_match_hl_get_startcol(shl);
            let endcol = nvim_match_hl_get_endcol(shl);
            if startcol == endcol {
                let byte = nvim_match_line_byte_at(*line, endcol);
                if byte != 0 {
                    let len = nvim_match_utfc_ptr2len_at(*line, endcol);
                    nvim_match_hl_set_endcol(shl, endcol + len);
                } else {
                    nvim_match_hl_set_endcol(shl, endcol + 1);
                }
            }
            let startcol = nvim_match_hl_get_startcol(shl);
            if startcol < mincol {
                // match at leftcol
                let attr = nvim_match_hl_get_attr(shl);
                nvim_match_hl_set_attr_cur(shl, attr);
                *search_attr = attr;
                *search_attr_from_match = c_int::from(!ptr::eq(shl, search_hl));
            }
            area_highlighting = true;
        }
        if !ptr::eq(shl, search_hl) && !cur.is_null() {
            cur = nvim_match_item_next(cur);
        }
    }
    c_int::from(area_highlighting)
}

// =============================================================================
// update_search_hl
// =============================================================================

/// Update search/match highlighting for a position in a line.
///
/// # Safety
///
/// All pointers must be valid. `line` is a `char **` (pointer to line pointer).
#[allow(clippy::too_many_lines)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_update_search_hl(
    wp: *mut WinHandle,
    lnum: i32,
    col: i32,
    line: *mut *mut u8,
    search_hl: *mut MatchHlHandle,
    has_match_conc: *mut c_int,
    match_conc: *mut c_int,
    lcs_eol_todo: c_int,
    on_last_col: *mut c_int,
    search_attr_from_match: *mut c_int,
) -> c_int {
    let mut cur = nvim_match_get_head(wp);
    let mut shl_flag = false;

    let hlf_lc = HLF_LC;
    let conceal_id = nvim_match_syn_name2id_conceal();

    // Do this for 'search_hl' and the match list (ordered by priority).
    while !cur.is_null() || !shl_flag {
        let shl;
        if !shl_flag && (cur.is_null() || nvim_match_item_get_priority(cur) > SEARCH_HL_PRIORITY) {
            shl = search_hl;
            shl_flag = true;
        } else {
            shl = nvim_match_item_get_hl(cur);
        }
        if !cur.is_null() {
            nvim_match_item_set_pos_cur(cur, 0);
        }
        let mut pos_inprogress = true;

        loop {
            let regprog = nvim_match_hl_get_regprog(shl);
            if regprog.is_null() && (cur.is_null() || !pos_inprogress) {
                break;
            }

            let startcol = nvim_match_hl_get_startcol(shl);
            let endcol = nvim_match_hl_get_endcol(shl);

            if startcol != MAXCOL && col >= startcol && col < endcol {
                let next_col = col + nvim_match_utfc_ptr2len_at(*line, col);

                if endcol < next_col {
                    nvim_match_hl_set_endcol(shl, next_col);
                }

                // Highlight the match where the cursor is using the CurSearch group.
                if ptr::eq(shl, search_hl) && nvim_match_hl_get_has_cursor(shl) != 0 {
                    let lc_attr = nvim_win_hl_attr(wp, hlf_lc);
                    nvim_match_hl_set_attr_cur(shl, lc_attr);
                    if lc_attr != nvim_match_hl_get_attr(shl) {
                        nvim_match_set_search_hl_has_cursor_lnum(lnum);
                    }
                } else {
                    let attr = nvim_match_hl_get_attr(shl);
                    nvim_match_hl_set_attr_cur(shl, attr);
                }

                // Match with the "Conceal" group results in hiding the match.
                if !cur.is_null()
                    && !ptr::eq(shl, search_hl)
                    && conceal_id == nvim_match_item_get_hlg_id(cur)
                {
                    *has_match_conc = if col == startcol { 2 } else { 1 };
                    *match_conc = nvim_match_item_get_conceal_char(cur);
                } else {
                    *has_match_conc = 0;
                }
            } else if col == endcol {
                nvim_match_hl_set_attr_cur(shl, 0);

                let cur_for_search = if ptr::eq(shl, search_hl) {
                    ptr::null_mut()
                } else {
                    cur
                };
                rs_next_search_hl(wp, search_hl, shl, lnum, col, cur_for_search);
                pos_inprogress = !cur.is_null() && nvim_match_item_get_pos_cur(cur) != 0;

                // Need to get the line again
                nvim_match_ml_get_buf_line(wp, lnum, line);

                if nvim_match_hl_get_lnum(shl) == lnum {
                    nvim_match_hl_set_startcol(shl, nvim_match_hl_rm_startpos_col(shl, 0));
                    if nvim_match_hl_rm_endpos_lnum(shl, 0) == 0 {
                        nvim_match_hl_set_endcol(shl, nvim_match_hl_rm_endpos_col(shl, 0));
                    } else {
                        nvim_match_hl_set_endcol(shl, MAXCOL);
                    }

                    // check if the cursor is in the match
                    if ptr::eq(shl, search_hl) {
                        rs_check_cur_search_hl(wp, shl);
                    }

                    let startcol2 = nvim_match_hl_get_startcol(shl);
                    let endcol2 = nvim_match_hl_get_endcol(shl);
                    if startcol2 == endcol2 {
                        // highlight empty match, try again after it
                        let byte = nvim_match_line_byte_at(*line, endcol2);
                        if byte == 0 {
                            nvim_match_hl_set_endcol(shl, endcol2 + 1);
                        } else {
                            let len = nvim_match_utfc_ptr2len_at(*line, endcol2);
                            nvim_match_hl_set_endcol(shl, endcol2 + len);
                        }
                    }

                    // Loop to check if the match starts at the current position
                    continue;
                }
            }
            break;
        }
        if !ptr::eq(shl, search_hl) && !cur.is_null() {
            cur = nvim_match_item_next(cur);
        }
    }

    // Use attributes from match with highest priority among 'search_hl' and
    // the match list.
    *search_attr_from_match = 0;
    let mut search_attr = nvim_match_hl_get_attr_cur(search_hl);
    cur = nvim_match_get_head(wp);
    shl_flag = false;
    while !cur.is_null() || !shl_flag {
        let shl;
        if !shl_flag && (cur.is_null() || nvim_match_item_get_priority(cur) > SEARCH_HL_PRIORITY) {
            shl = search_hl;
            shl_flag = true;
        } else {
            shl = nvim_match_item_get_hl(cur);
        }
        let attr_cur = nvim_match_hl_get_attr_cur(shl);
        if attr_cur != 0 {
            search_attr = attr_cur;
            let endcol = nvim_match_hl_get_endcol(shl);
            *on_last_col = c_int::from(col + 1 >= endcol);
            *search_attr_from_match = c_int::from(!ptr::eq(shl, search_hl));
        }
        if !ptr::eq(shl, search_hl) && !cur.is_null() {
            cur = nvim_match_item_next(cur);
        }
    }

    // Only highlight one character after the last column.
    let byte = nvim_match_line_byte_at(*line, col);
    if byte == 0 && nvim_win_get_p_list(wp) != 0 && lcs_eol_todo == 0 {
        search_attr = 0;
    }
    search_attr
}

// =============================================================================
// Exported wrappers matching the exact C bool ABI
// =============================================================================

/// Exported entry point for `prepare_search_hl_line` matching C `bool` ABI.
///
/// # Safety
///
/// All pointers must be valid.
#[export_name = "prepare_search_hl_line"]
pub unsafe extern "C" fn prepare_search_hl_line_export(
    wp: *mut WinHandle,
    lnum: i32,
    mincol: i32,
    line: *mut *mut u8,
    search_hl: *mut MatchHlHandle,
    search_attr: *mut c_int,
    search_attr_from_match: *mut bool,
) -> bool {
    let mut search_attr_from_match_int = c_int::from(*search_attr_from_match);
    let result = rs_prepare_search_hl_line(
        wp,
        lnum,
        mincol,
        line,
        search_hl,
        search_attr,
        &raw mut search_attr_from_match_int,
    );
    *search_attr_from_match = search_attr_from_match_int != 0;
    result != 0
}

/// Exported entry point for `update_search_hl` matching C `bool` ABI.
///
/// # Safety
///
/// All pointers must be valid.
#[export_name = "update_search_hl"]
pub unsafe extern "C" fn update_search_hl_export(
    wp: *mut WinHandle,
    lnum: i32,
    col: i32,
    line: *mut *mut u8,
    search_hl: *mut MatchHlHandle,
    has_match_conc: *mut c_int,
    match_conc: *mut c_int,
    lcs_eol_todo: bool,
    on_last_col: *mut bool,
    search_attr_from_match: *mut bool,
) -> c_int {
    let mut on_last_col_int = c_int::from(*on_last_col);
    let mut search_attr_from_match_int = c_int::from(*search_attr_from_match);
    let result = rs_update_search_hl(
        wp,
        lnum,
        col,
        line,
        search_hl,
        has_match_conc,
        match_conc,
        c_int::from(lcs_eol_todo),
        &raw mut on_last_col_int,
        &raw mut search_attr_from_match_int,
    );
    *on_last_col = on_last_col_int != 0;
    *search_attr_from_match = search_attr_from_match_int != 0;
    result
}
