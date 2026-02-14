//! Core search engine for match highlighting
//!
//! Migrates: `init_search_hl`, `next_search_hl`

use std::ffi::c_int;
use std::ptr;

// =============================================================================
// Opaque Handle Types
// =============================================================================

/// Opaque handle to a C `win_T` structure.
#[repr(C)]
pub struct WinHandle {
    _opaque: [u8; 0],
}

/// Opaque handle to a C `buf_T` structure.
#[repr(C)]
pub struct BufHandle {
    _opaque: [u8; 0],
}

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

/// Opaque handle to a C `regprog_T` structure.
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
    fn nvim_match_hl_set_lnum(shl: *mut MatchHlHandle, lnum: i32);
    fn nvim_match_hl_rm_startpos_lnum(shl: *mut MatchHlHandle, idx: c_int) -> i32;
    fn nvim_match_hl_rm_startpos_col(shl: *mut MatchHlHandle, idx: c_int) -> i32;
    fn nvim_match_hl_rm_endpos_lnum(shl: *mut MatchHlHandle, idx: c_int) -> i32;
    fn nvim_match_hl_rm_endpos_col(shl: *mut MatchHlHandle, idx: c_int) -> i32;
    fn nvim_match_hl_get_buf(shl: *mut MatchHlHandle) -> *mut BufHandle;
    fn nvim_match_hl_set_buf(shl: *mut MatchHlHandle, buf: *mut BufHandle);
    fn nvim_match_hl_set_first_lnum(shl: *mut MatchHlHandle, lnum: i32);
    fn nvim_match_hl_set_attr(shl: *mut MatchHlHandle, attr: c_int);
    fn nvim_match_hl_set_tm(shl: *mut MatchHlHandle, msec: i64);
    fn nvim_match_hl_get_tm_ptr(shl: *mut MatchHlHandle) -> *mut u8;
    fn nvim_match_hl_get_regprog(shl: *mut MatchHlHandle) -> *mut RegProgHandle;
    fn nvim_match_hl_set_regprog(shl: *mut MatchHlHandle, rp: *mut RegProgHandle);
    fn nvim_match_hl_copy_rm_from_item(shl: *mut MatchHlHandle, m: *mut MatchItemHandle);
    fn nvim_match_hl_regprog_is_copy(shl: *mut MatchHlHandle, cur: *mut MatchItemHandle) -> c_int;

    // matchitem_T accessors
    fn nvim_match_get_head(wp: *mut WinHandle) -> *mut MatchItemHandle;
    fn nvim_match_item_next(m: *mut MatchItemHandle) -> *mut MatchItemHandle;
    fn nvim_match_item_get_hlg_id(m: *mut MatchItemHandle) -> c_int;
    fn nvim_match_item_get_hl(m: *mut MatchItemHandle) -> *mut MatchHlHandle;
    fn nvim_match_item_sync_regprog(m: *mut MatchItemHandle, shl: *mut MatchHlHandle);

    // Function wrappers
    fn nvim_match_vim_regexec_multi(
        shl: *mut MatchHlHandle,
        win: *mut WinHandle,
        lnum: i32,
        col: i32,
        timed_out: *mut c_int,
    ) -> c_int;
    fn nvim_match_vim_regfree(rp: *mut RegProgHandle);
    fn nvim_match_set_no_hlsearch(flag: c_int);
    fn nvim_match_ml_get_byte(buf: *mut BufHandle, lnum: i32, col: i32) -> c_int;
    fn nvim_match_utfc_ptr2len(buf: *mut BufHandle, lnum: i32, col: i32) -> c_int;
    fn nvim_match_has_cpo_search() -> c_int;
    fn nvim_match_get_search_first_line() -> i32;
    fn nvim_match_get_search_last_line() -> i32;
    fn nvim_match_get_p_rdt() -> i64;
    fn nvim_match_get_HLF_L() -> c_int;
    fn nvim_match_win_get_buffer(wp: *mut WinHandle) -> *mut BufHandle;

    // Existing global accessors
    fn nvim_get_called_emsg() -> c_int;
    fn nvim_get_got_int() -> c_int;
    fn nvim_set_got_int(val: c_int);
    fn nvim_syn_id2attr(hl_id: c_int) -> c_int;
    fn nvim_win_hl_attr(wp: *mut WinHandle, hlf: c_int) -> c_int;
    fn nvim_profile_passed_limit(tm: *mut u8) -> c_int;

    // next_search_hl_pos (Phase 3)
    fn rs_next_search_hl_pos(
        shl: *mut MatchHlHandle,
        lnum: i32,
        match_item: *mut MatchItemHandle,
        mincol: i32,
    ) -> c_int;
}

// =============================================================================
// init_search_hl
// =============================================================================

/// Initialize search highlighting for a window.
///
/// # Safety
///
/// `wp` and `search_hl` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_init_search_hl(wp: *mut WinHandle, search_hl: *mut MatchHlHandle) {
    let buf = nvim_match_win_get_buffer(wp);
    let p_rdt = nvim_match_get_p_rdt();

    let mut cur = nvim_match_get_head(wp);
    while !cur.is_null() {
        let hl = nvim_match_item_get_hl(cur);

        // shl->rm = cur->mit_match
        nvim_match_hl_copy_rm_from_item(hl, cur);

        let hlg_id = nvim_match_item_get_hlg_id(cur);
        if hlg_id == 0 {
            nvim_match_hl_set_attr(hl, 0);
        } else {
            nvim_match_hl_set_attr(hl, nvim_syn_id2attr(hlg_id));
        }
        nvim_match_hl_set_buf(hl, buf);
        nvim_match_hl_set_lnum(hl, 0);
        nvim_match_hl_set_first_lnum(hl, 0);
        // Set the time limit to 'redrawtime'
        nvim_match_hl_set_tm(hl, p_rdt);

        cur = nvim_match_item_next(cur);
    }

    nvim_match_hl_set_buf(search_hl, buf);
    nvim_match_hl_set_lnum(search_hl, 0);
    nvim_match_hl_set_first_lnum(search_hl, 0);
    let hlf_l = nvim_match_get_HLF_L();
    nvim_match_hl_set_attr(search_hl, nvim_win_hl_attr(wp, hlf_l));

    // time limit is set at the toplevel, for all windows
}

// =============================================================================
// next_search_hl
// =============================================================================

/// Search for a next 'hlsearch' or match.
///
/// # Safety
///
/// `search_hl` must be a valid pointer. `shl` must be valid.
/// `win` may be null. `cur` may be null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_next_search_hl(
    win: *mut WinHandle,
    search_hl: *mut MatchHlHandle,
    shl: *mut MatchHlHandle,
    lnum: i32,
    mincol: i32,
    cur: *mut MatchItemHandle,
) {
    let called_emsg_before = nvim_get_called_emsg();

    // For :{range}s/pat only highlight inside the range
    if (lnum < nvim_match_get_search_first_line() || lnum > nvim_match_get_search_last_line())
        && cur.is_null()
    {
        nvim_match_hl_set_lnum(shl, 0);
        return;
    }

    let shl_lnum = nvim_match_hl_get_lnum(shl);
    if shl_lnum != 0 {
        // Check for three situations:
        // 1. If the "lnum" is below a previous match, start a new search.
        // 2. If the previous match includes "mincol", use it.
        // 3. Continue after the previous match.
        let l = shl_lnum + nvim_match_hl_rm_endpos_lnum(shl, 0)
            - nvim_match_hl_rm_startpos_lnum(shl, 0);
        if lnum > l {
            nvim_match_hl_set_lnum(shl, 0);
        } else if lnum < l || nvim_match_hl_rm_endpos_col(shl, 0) > mincol {
            return;
        }
    }

    let mut nmatched: c_int;

    // Repeat searching for a match until one is found that includes "mincol"
    // or none is found in this line.
    loop {
        // Stop searching after passing the time limit.
        let tm_ptr = nvim_match_hl_get_tm_ptr(shl);
        if nvim_profile_passed_limit(tm_ptr) != 0 {
            nvim_match_hl_set_lnum(shl, 0); // no match found in time
            break;
        }

        // Three situations:
        // 1. No useful previous match: search from start of line.
        // 2. Not Vi compatible or empty match: continue at next character.
        //    Break the loop if this is beyond the end of the line.
        // 3. Vi compatible searching: continue at end of previous match.
        let matchcol;
        let cur_shl_lnum = nvim_match_hl_get_lnum(shl);
        if cur_shl_lnum == 0 {
            matchcol = 0;
        } else if nvim_match_has_cpo_search() == 0
            || (nvim_match_hl_rm_endpos_lnum(shl, 0) == 0
                && nvim_match_hl_rm_endpos_col(shl, 0) <= nvim_match_hl_rm_startpos_col(shl, 0))
        {
            let start_col = nvim_match_hl_rm_startpos_col(shl, 0);
            let buf = nvim_match_hl_get_buf(shl);
            let byte = nvim_match_ml_get_byte(buf, lnum, start_col);
            if byte == 0 {
                // NUL - past end of line
                nvim_match_hl_set_lnum(shl, 0);
                break;
            }
            matchcol = start_col + nvim_match_utfc_ptr2len(buf, lnum, start_col);
        } else {
            matchcol = nvim_match_hl_rm_endpos_col(shl, 0);
        }

        nvim_match_hl_set_lnum(shl, lnum);
        let regprog = nvim_match_hl_get_regprog(shl);
        if !regprog.is_null() {
            // Remember whether shl->rm is using a copy of the regprog in
            // cur->mit_match.
            let regprog_is_copy = !ptr::eq(shl, search_hl)
                && !cur.is_null()
                && nvim_match_hl_regprog_is_copy(shl, cur) != 0;

            let mut timed_out: c_int = 0;

            nmatched = nvim_match_vim_regexec_multi(shl, win, lnum, matchcol, &raw mut timed_out);

            // Copy the regprog, in case it got freed and recompiled.
            if regprog_is_copy {
                nvim_match_item_sync_regprog(cur, shl);
            }

            if nvim_get_called_emsg() > called_emsg_before
                || nvim_get_got_int() != 0
                || timed_out != 0
            {
                // Error while handling regexp: stop using this regexp.
                if ptr::eq(shl, search_hl) {
                    // don't free regprog in the match list, it's a copy
                    let rp = nvim_match_hl_get_regprog(shl);
                    nvim_match_vim_regfree(rp);
                    nvim_match_set_no_hlsearch(1);
                }
                nvim_match_hl_set_regprog(shl, ptr::null_mut());
                nvim_match_hl_set_lnum(shl, 0);
                nvim_set_got_int(0); // avoid the "Type :quit to exit Vim" message
                break;
            }
        } else if !cur.is_null() {
            nmatched = rs_next_search_hl_pos(shl, lnum, cur, matchcol);
        } else {
            nmatched = 0;
        }

        if nmatched == 0 {
            nvim_match_hl_set_lnum(shl, 0); // no match found
            break;
        }
        if nvim_match_hl_rm_startpos_lnum(shl, 0) > 0
            || nvim_match_hl_rm_startpos_col(shl, 0) >= mincol
            || nmatched > 1
            || nvim_match_hl_rm_endpos_col(shl, 0) > mincol
        {
            let new_lnum = nvim_match_hl_get_lnum(shl) + nvim_match_hl_rm_startpos_lnum(shl, 0);
            nvim_match_hl_set_lnum(shl, new_lnum);
            break; // useful match found
        }
    }
}
