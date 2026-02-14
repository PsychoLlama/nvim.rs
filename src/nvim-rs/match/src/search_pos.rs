//! Position-based match search
//!
//! Migrates: `next_search_hl_pos`

use std::ffi::c_int;

// =============================================================================
// Opaque Handle Types
// =============================================================================

/// Opaque handle to a C `match_T` structure.
#[repr(C)]
pub struct MatchHlHandle {
    _opaque: [u8; 0],
}

/// Opaque handle to a C `matchitem_T` structure.
#[repr(C)]
pub struct MatchItemHandle {
    _opaque: [u8; 0],
}

// =============================================================================
// FFI Declarations
// =============================================================================

extern "C" {
    // match_T (MatchHlHandle) setters
    fn nvim_match_hl_set_lnum(shl: *mut MatchHlHandle, lnum: i32);
    fn nvim_match_hl_set_is_addpos(shl: *mut MatchHlHandle, val: c_int);
    fn nvim_match_hl_set_has_cursor(shl: *mut MatchHlHandle, val: c_int);
    fn nvim_match_hl_rm_set_startpos(shl: *mut MatchHlHandle, idx: c_int, lnum: i32, col: i32);
    fn nvim_match_hl_rm_set_endpos(shl: *mut MatchHlHandle, idx: c_int, lnum: i32, col: i32);

    // matchitem_T position accessors
    fn nvim_match_item_get_pos_cur(m: *mut MatchItemHandle) -> c_int;
    fn nvim_match_item_set_pos_cur(m: *mut MatchItemHandle, cur: c_int);
    fn nvim_match_item_get_pos_count(m: *mut MatchItemHandle) -> c_int;
    fn nvim_match_item_pos_get_lnum(m: *mut MatchItemHandle, idx: c_int) -> i32;
    fn nvim_match_item_pos_get_col(m: *mut MatchItemHandle, idx: c_int) -> i32;
    fn nvim_match_item_pos_get_len(m: *mut MatchItemHandle, idx: c_int) -> c_int;
    fn nvim_match_item_pos_swap(m: *mut MatchItemHandle, idx1: c_int, idx2: c_int);
}

/// MAXCOL value from C (0x7fffffff).
const MAXCOL: i32 = 0x7fff_ffff;

// =============================================================================
// next_search_hl_pos
// =============================================================================

/// Search for a position match on the given line.
///
/// Updates `shl` with the match info if found. Returns 1 if found, 0 otherwise.
///
/// # Safety
///
/// `shl` and `match_item` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_next_search_hl_pos(
    shl: *mut MatchHlHandle,
    lnum: i32,
    match_item: *mut MatchItemHandle,
    mincol: i32,
) -> c_int {
    let mut found: c_int = -1;

    nvim_match_hl_set_lnum(shl, 0);

    let pos_cur = nvim_match_item_get_pos_cur(match_item);
    let pos_count = nvim_match_item_get_pos_count(match_item);

    for i in pos_cur..pos_count {
        let pos_lnum = nvim_match_item_pos_get_lnum(match_item, i);
        let pos_col = nvim_match_item_pos_get_col(match_item, i);
        let pos_len = nvim_match_item_pos_get_len(match_item, i);

        if pos_lnum == 0 {
            break;
        }
        if pos_len == 0 && pos_col < mincol {
            continue;
        }
        if pos_lnum == lnum {
            if found >= 0 {
                // if this match comes before the one at "found" then swap them
                let found_col = nvim_match_item_pos_get_col(match_item, found);
                if pos_col < found_col {
                    nvim_match_item_pos_swap(match_item, i, found);
                }
            } else {
                found = i;
            }
        }
    }

    nvim_match_item_set_pos_cur(match_item, 0);

    if found >= 0 {
        let found_col = nvim_match_item_pos_get_col(match_item, found);
        let found_len = nvim_match_item_pos_get_len(match_item, found);

        let start = if found_col == 0 { 0 } else { found_col - 1 };
        let end = if found_col == 0 {
            MAXCOL
        } else {
            start + found_len
        };

        nvim_match_hl_set_lnum(shl, lnum);
        nvim_match_hl_rm_set_startpos(shl, 0, 0, start);
        nvim_match_hl_rm_set_endpos(shl, 0, 0, end);
        nvim_match_hl_set_is_addpos(shl, 1);
        nvim_match_hl_set_has_cursor(shl, 0);
        nvim_match_item_set_pos_cur(match_item, found + 1);
        return 1;
    }

    0
}
