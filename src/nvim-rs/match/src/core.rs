//! Core match management operations
//!
//! This module implements the core match lifecycle functions:
//! - `rs_match_add` — add a pattern-based match
//! - `rs_match_add_pos` — add a position-based match
//! - `rs_match_delete` — delete a match by ID
//! - `rs_clear_matches` — delete all matches
//! - `rs_get_match` — find a match by ID

use std::ffi::{c_char, c_int};
use std::ptr;

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

/// Opaque handle to a C `regprog_T` structure.
#[repr(C)]
pub struct RegProgHandle {
    _opaque: [u8; 0],
}

// =============================================================================
// FFI — Accessor functions (in match.c)
// =============================================================================

extern "C" {
    // Window match list
    fn nvim_match_get_head(wp: *mut WinHandle) -> *mut MatchItemHandle;
    fn nvim_match_set_head(wp: *mut WinHandle, head: *mut MatchItemHandle);
    fn nvim_match_get_next_id(wp: *mut WinHandle) -> c_int;
    fn nvim_match_set_next_id(wp: *mut WinHandle, id: c_int);

    // Match item navigation
    fn nvim_match_item_next(m: *mut MatchItemHandle) -> *mut MatchItemHandle;
    fn nvim_match_item_set_next(m: *mut MatchItemHandle, next: *mut MatchItemHandle);
    fn nvim_match_item_get_id(m: *mut MatchItemHandle) -> c_int;
    fn nvim_match_item_get_priority(m: *mut MatchItemHandle) -> c_int;
    fn nvim_match_item_get_toplnum(m: *mut MatchItemHandle) -> i32;
    fn nvim_match_item_get_botlnum(m: *mut MatchItemHandle) -> i32;

    // Match item allocation / deallocation
    fn nvim_match_alloc() -> *mut MatchItemHandle;
    fn nvim_match_free(m: *mut MatchItemHandle);
    fn nvim_match_alloc_positions(count: usize) -> *mut LlposT;

    // Match item field setters
    fn nvim_match_item_set_id(m: *mut MatchItemHandle, id: c_int);
    fn nvim_match_item_set_priority(m: *mut MatchItemHandle, priority: c_int);
    fn nvim_match_item_set_pattern(m: *mut MatchItemHandle, pat: *const c_char);
    fn nvim_match_item_set_hlg_id(m: *mut MatchItemHandle, hlg_id: c_int);
    fn nvim_match_item_set_conceal_char(m: *mut MatchItemHandle, ch: c_int);
    fn nvim_match_item_set_toplnum(m: *mut MatchItemHandle, lnum: i32);
    fn nvim_match_item_set_botlnum(m: *mut MatchItemHandle, lnum: i32);
    fn nvim_match_item_set_regprog(m: *mut MatchItemHandle, regprog: *mut RegProgHandle);
    fn nvim_match_item_set_rmm_ic(m: *mut MatchItemHandle, ic: c_int);
    fn nvim_match_item_set_rmm_maxcol(m: *mut MatchItemHandle, maxcol: i32);
    fn nvim_match_item_set_pos_array(m: *mut MatchItemHandle, arr: *mut LlposT, count: c_int);

    // Position array access
    fn nvim_match_pos_set(arr: *mut LlposT, idx: c_int, lnum: i32, col: i32, len: c_int);

    // External C API wrappers (prefixed nvim_match_ to avoid symbol conflicts)
    fn nvim_match_syn_check_group(grp: *const c_char, len: usize) -> c_int;
    fn nvim_match_vim_regcomp(pat: *const c_char, flags: c_int) -> *mut RegProgHandle;
    fn nvim_match_utf_ptr2char(p: *const c_char) -> c_int;
    fn nvim_match_redraw_later(wp: *mut WinHandle, rtype: c_int);
    fn nvim_match_redraw_win_range_later(wp: *mut WinHandle, top: i32, bot: i32);

    // Error message wrappers
    fn nvim_semsg_id_taken(id: i64);
    fn nvim_semsg_invalid_id(id: i64);
    fn nvim_semsg_invalid_delete_id(id: i64);
    fn nvim_semsg_id_not_found(id: i64);
    fn nvim_semsg_invarg2(arg: *const c_char);

}

// Constants are now Rust-side (match/src/lib.rs), guarded by _Static_assert in match.c.
use crate::{RE_MAGIC, UPD_SOME_VALID, UPD_VALID};

// =============================================================================
// Position type (matches C llpos_T)
// =============================================================================

/// Matches C `llpos_T` layout.
#[repr(C)]
pub struct LlposT {
    pub lnum: i32,
    pub col: i32,
    pub len: c_int,
}

// =============================================================================
// match_add — pattern-based
// =============================================================================

/// Add a pattern-based match to the window's match list.
///
/// This replaces the core of C `match_add()` for the case where
/// `pos_list == NULL` (i.e. a regex pattern, not position list).
///
/// # Safety
///
/// All pointer parameters must be valid. `wp` must point to a valid `win_T`.
/// `grp` must be a valid C string. `pat` may be null (position-only matches
/// are handled by `rs_match_add_pos`). `conceal_char` may be null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_match_add(
    wp: *mut WinHandle,
    grp: *const c_char,
    pat: *const c_char,
    prio: c_int,
    id: c_int,
    conceal_char: *const c_char,
) -> c_int {
    // Validate group
    if grp.is_null() || *grp == 0 {
        return -1;
    }
    // Validate pattern (non-null for pattern matches)
    if !pat.is_null() && *pat == 0 {
        return -1;
    }

    // Validate ID
    if id < -1 || id == 0 {
        nvim_semsg_invalid_id(i64::from(id));
        return -1;
    }

    // Resolve ID
    let resolved_id;
    if id == -1 {
        resolved_id = nvim_match_get_next_id(wp);
        nvim_match_set_next_id(wp, resolved_id + 1);
    } else {
        // Check for conflicts
        let mut cur = nvim_match_get_head(wp);
        while !cur.is_null() {
            if nvim_match_item_get_id(cur) == id {
                nvim_semsg_id_taken(i64::from(id));
                return -1;
            }
            cur = nvim_match_item_next(cur);
        }
        // Ensure next_id stays ahead
        if nvim_match_get_next_id(wp) < id + 100 {
            nvim_match_set_next_id(wp, id + 100);
        }
        resolved_id = id;
    }

    // Check highlight group
    let grp_len = libc::strlen(grp);
    let hlg_id = nvim_match_syn_check_group(grp, grp_len);
    if hlg_id == 0 {
        return -1;
    }

    // Compile regex if pattern provided
    let mut regprog: *mut RegProgHandle = ptr::null_mut();
    if !pat.is_null() {
        regprog = nvim_match_vim_regcomp(pat, RE_MAGIC);
        if regprog.is_null() {
            nvim_semsg_invarg2(pat);
            return -1;
        }
    }

    // Build new match item
    let m = nvim_match_alloc();
    nvim_match_item_set_id(m, resolved_id);
    nvim_match_item_set_priority(m, prio);
    if !pat.is_null() {
        nvim_match_item_set_pattern(m, pat); // accessor does xstrdup
    }
    nvim_match_item_set_hlg_id(m, hlg_id);
    nvim_match_item_set_regprog(m, regprog);
    nvim_match_item_set_rmm_ic(m, 0);
    nvim_match_item_set_rmm_maxcol(m, 0);

    // Conceal character
    if !conceal_char.is_null() && *conceal_char != 0 {
        nvim_match_item_set_conceal_char(m, nvim_match_utf_ptr2char(conceal_char));
    } else {
        nvim_match_item_set_conceal_char(m, 0);
    }

    // Insert in priority order
    insert_by_priority(wp, m, prio);

    nvim_match_redraw_later(wp, UPD_SOME_VALID);
    resolved_id
}

/// Add a position-based match to the window's match list.
///
/// The caller (C code in `f_matchaddpos` / `f_setmatches`) extracts positions
/// from the `VimL` list into parallel arrays before calling this function.
///
/// # Safety
///
/// All pointer parameters must be valid. `lnums`, `cols`, `lens` must point
/// to arrays of at least `count` elements.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_match_add_pos(
    wp: *mut WinHandle,
    grp: *const c_char,
    prio: c_int,
    id: c_int,
    conceal_char: *const c_char,
    lnums: *const i32,
    cols: *const i32,
    lens: *const c_int,
    count: c_int,
) -> c_int {
    // Validate group
    if grp.is_null() || *grp == 0 {
        return -1;
    }

    // Validate ID
    if id < -1 || id == 0 {
        nvim_semsg_invalid_id(i64::from(id));
        return -1;
    }

    // Resolve ID
    let resolved_id;
    if id == -1 {
        resolved_id = nvim_match_get_next_id(wp);
        nvim_match_set_next_id(wp, resolved_id + 1);
    } else {
        let mut cur = nvim_match_get_head(wp);
        while !cur.is_null() {
            if nvim_match_item_get_id(cur) == id {
                nvim_semsg_id_taken(i64::from(id));
                return -1;
            }
            cur = nvim_match_item_next(cur);
        }
        if nvim_match_get_next_id(wp) < id + 100 {
            nvim_match_set_next_id(wp, id + 100);
        }
        resolved_id = id;
    }

    // Check highlight group
    let grp_len = libc::strlen(grp);
    let hlg_id = nvim_match_syn_check_group(grp, grp_len);
    if hlg_id == 0 {
        return -1;
    }

    // Build new match item
    let m = nvim_match_alloc();
    nvim_match_item_set_id(m, resolved_id);
    nvim_match_item_set_priority(m, prio);
    nvim_match_item_set_hlg_id(m, hlg_id);
    nvim_match_item_set_regprog(m, ptr::null_mut());
    nvim_match_item_set_rmm_ic(m, 0);
    nvim_match_item_set_rmm_maxcol(m, 0);

    // Conceal character
    if !conceal_char.is_null() && *conceal_char != 0 {
        nvim_match_item_set_conceal_char(m, nvim_match_utf_ptr2char(conceal_char));
    } else {
        nvim_match_item_set_conceal_char(m, 0);
    }

    // Allocate and fill position array
    let mut rtype = UPD_SOME_VALID;

    if count > 0 {
        let pos_arr = nvim_match_alloc_positions(count as usize);
        nvim_match_item_set_pos_array(m, pos_arr, count);

        let mut toplnum: i32 = 0;
        let mut botlnum: i32 = 0;

        for i in 0..count {
            let lnum = *lnums.add(i as usize);
            let col = *cols.add(i as usize);
            let len = *lens.add(i as usize);

            nvim_match_pos_set(pos_arr, i, lnum, col, len);

            if lnum > 0 {
                if toplnum == 0 || lnum < toplnum {
                    toplnum = lnum;
                }
                if botlnum == 0 || lnum >= botlnum {
                    botlnum = lnum + 1;
                }
            }
        }

        if toplnum != 0 {
            nvim_match_redraw_win_range_later(wp, toplnum, botlnum);
            nvim_match_item_set_toplnum(m, toplnum);
            nvim_match_item_set_botlnum(m, botlnum);
            rtype = UPD_VALID;
        }
    }

    // Insert in priority order
    insert_by_priority(wp, m, prio);

    nvim_match_redraw_later(wp, rtype);
    resolved_id
}

// =============================================================================
// match_delete
// =============================================================================

/// Delete a match by ID from the window's match list.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_match_delete(wp: *mut WinHandle, id: c_int, perr: c_int) -> c_int {
    if id < 1 {
        if perr != 0 {
            nvim_semsg_invalid_delete_id(i64::from(id));
        }
        return -1;
    }

    let mut rtype = UPD_SOME_VALID;

    // Find the match
    let head = nvim_match_get_head(wp);
    let mut cur = head;
    let mut prev = cur;

    while !cur.is_null() && nvim_match_item_get_id(cur) != id {
        prev = cur;
        cur = nvim_match_item_next(cur);
    }

    if cur.is_null() {
        if perr != 0 {
            nvim_semsg_id_not_found(i64::from(id));
        }
        return -1;
    }

    // Unlink from list
    if cur == prev {
        // At head
        nvim_match_set_head(wp, nvim_match_item_next(cur));
    } else {
        nvim_match_item_set_next(prev, nvim_match_item_next(cur));
    }

    // Check if we need range redraw
    let toplnum = nvim_match_item_get_toplnum(cur);
    if toplnum != 0 {
        let botlnum = nvim_match_item_get_botlnum(cur);
        nvim_match_redraw_win_range_later(wp, toplnum, botlnum);
        rtype = UPD_VALID;
    }

    nvim_match_free(cur);
    nvim_match_redraw_later(wp, rtype);
    0
}

// =============================================================================
// match_delete (exported entry point with bool ABI)
// =============================================================================

/// Delete a match by ID from the window's match list.
///
/// This is the exported C entry point. It accepts `bool perr` (C `_Bool`)
/// matching the callers' ABI, and delegates to `rs_match_delete`.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
#[export_name = "match_delete"]
pub unsafe extern "C" fn match_delete_export(wp: *mut WinHandle, id: c_int, perr: bool) -> c_int {
    rs_match_delete(wp, id, c_int::from(perr))
}

// =============================================================================
// clear_matches
// =============================================================================

/// Delete all matches in the window's match list.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
#[export_name = "clear_matches"]
pub unsafe extern "C" fn rs_clear_matches(wp: *mut WinHandle) {
    loop {
        let head = nvim_match_get_head(wp);
        if head.is_null() {
            break;
        }
        let next = nvim_match_item_next(head);
        nvim_match_free(head);
        nvim_match_set_head(wp, next);
    }
    nvim_match_redraw_later(wp, UPD_SOME_VALID);
}

// =============================================================================
// get_match
// =============================================================================

/// Get a match by ID from the window's match list.
///
/// Returns null if not found.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
#[export_name = "get_match"]
pub unsafe extern "C" fn rs_get_match(wp: *mut WinHandle, id: c_int) -> *mut MatchItemHandle {
    let mut cur = nvim_match_get_head(wp);
    while !cur.is_null() {
        if nvim_match_item_get_id(cur) == id {
            return cur;
        }
        cur = nvim_match_item_next(cur);
    }
    ptr::null_mut()
}

// =============================================================================
// Helpers
// =============================================================================

/// Insert a match item into the list in ascending priority order.
unsafe fn insert_by_priority(wp: *mut WinHandle, m: *mut MatchItemHandle, prio: c_int) {
    let head = nvim_match_get_head(wp);
    let mut cur = head;
    let mut prev = cur;

    while !cur.is_null() && prio >= nvim_match_item_get_priority(cur) {
        prev = cur;
        cur = nvim_match_item_next(cur);
    }

    if cur == prev {
        // Insert at head (list empty or priority < head's priority)
        nvim_match_set_head(wp, m);
    } else {
        nvim_match_item_set_next(prev, m);
    }
    nvim_match_item_set_next(m, cur);
}

// =============================================================================
// Tests
// =============================================================================

// Tests for core functions require FFI linking and are covered by
// integration tests (smoke-test, functional tests).
