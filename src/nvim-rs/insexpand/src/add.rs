//! Completion match addition operations.
//!
//! This module provides Rust implementations of ins_compl_add and
//! ins_compl_add_infercase, replacing the C versions in insexpand_shim.c.

#![allow(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::if_not_else,
    clippy::ptr_cast_constness,
    clippy::borrow_as_ptr
)]

use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

use crate::match_list::{
    is_first_match, nvim_compl_get_curr_match, nvim_compl_get_first_match,
    nvim_compl_set_curr_match, nvim_compl_set_first_match, ComplMatch,
};

// Return values (must match C defines in vim_defs.h)
const OK: c_int = 1;
const FAIL: c_int = 0;
const NOTDONE: c_int = 2;

// Direction constants (must match C enum in vim_defs.h)
const K_DIRECTION_NOT_SET: c_int = 0;
const FORWARD: c_int = 1;

// CP flags (must match C enum in insexpand_shim.c)
const CP_ORIGINAL_TEXT: c_int = 1;
const CP_FREE_FNAME: c_int = 2;
const CP_CONT_S_IPOS: c_int = 4;
const CP_EQUAL: c_int = 8;
const CP_ICASE: c_int = 16;
const CP_FAST: c_int = 32;

// CPT indices (must match C enum in insexpand.h)
const CPT_COUNT: c_int = 4;

// FUZZY_SCORE_NONE = INT_MIN (from fuzzy.h)
const FUZZY_SCORE_NONE: c_int = c_int::MIN;

extern "C" {
    // Breakcheck functions
    fn fast_breakcheck();
    fn os_breakcheck();

    // got_int global
    #[link_name = "got_int"]
    static mut got_int_add: bool;

    // String functions
    fn strlen(s: *const c_char) -> usize;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;

    // Memory functions
    #[link_name = "xfree"]
    fn xfree_add(p: *mut u8);

    // compl_T allocator
    fn nvim_compl_T_alloc() -> *mut c_void;

    // compl_T field setters (Phase 6)
    fn nvim_compl_match_set_flags(m: ComplMatch, f: c_int);
    fn nvim_compl_match_set_cp_str(m: ComplMatch, s: *const c_char, l: usize);
    fn nvim_compl_match_get_cp_fname(m: ComplMatch) -> *const c_char;
    fn nvim_compl_match_set_cp_fname_dup(m: ComplMatch, f: *const c_char);
    fn nvim_compl_match_set_cp_fname_ref(m: ComplMatch, f: *const c_char);
    fn nvim_compl_match_set_hl_attrs(m: ComplMatch, abbr: c_int, kind: c_int);
    fn nvim_compl_match_set_cpt_source_idx(m: ComplMatch, idx: c_int);
    fn nvim_compl_match_set_cp_text_take(m: ComplMatch, i: c_int, s: *mut c_char);
    fn nvim_compl_match_set_cp_text_copy(m: ComplMatch, i: c_int, s: *const c_char);
    fn nvim_compl_match_set_user_data_move(m: ComplMatch, tv: *mut c_void);

    // compl_T field getters (used in dedup check)
    fn nvim_compl_match_get_cp_str_data(m: ComplMatch) -> *const c_char;
    fn nvim_compl_match_get_cp_str_size(m: ComplMatch) -> usize;
    fn nvim_compl_match_get_next(m: ComplMatch) -> ComplMatch;
    fn nvim_compl_match_set_next(m: ComplMatch, next: ComplMatch);
    fn nvim_compl_match_get_prev(m: ComplMatch) -> ComplMatch;
    fn nvim_compl_match_set_prev(m: ComplMatch, prev: ComplMatch);
    fn nvim_compl_match_get_flags(m: ComplMatch) -> c_int;
    fn nvim_compl_match_get_score(m: ComplMatch) -> c_int;
    fn nvim_compl_match_set_score(m: ComplMatch, score: c_int);
    fn nvim_compl_match_at_original_text(m: ComplMatch) -> c_int;
    fn nvim_compl_match_set_cp_number(m: ComplMatch, num: c_int);

    // Rust functions callable via FFI
    fn rs_is_nearest_active() -> c_int;
    fn rs_cot_fuzzy() -> c_int;
    fn rs_ins_compl_del_pum();
    fn rs_ins_compl_preinsert_longest() -> c_int;
    fn rs_ins_compl_longest_match(m: ComplMatch);
    fn rs_ins_compl_infercase_gettext(
        str: *const c_char,
        char_len: c_int,
        compl_char_len: c_int,
        min_len: c_int,
        tofree: *mut *mut c_char,
    ) -> *mut c_char;

    // For utfc_ptr2len (MB_PTR_ADV)
    fn utfc_ptr2len(ptr: *const c_char) -> c_int;

    // For ins_compl_add_infercase: p_ic, b_p_inf, orig_text
    fn nvim_get_p_inf() -> c_int;
}

/// Free a cptext array (mirrors C `free_cptext`).
/// Calls xfree on each non-null element.
unsafe fn free_cptext(cptext: *mut [*mut c_char; 4]) {
    if cptext.is_null() {
        return;
    }
    for i in 0..(CPT_COUNT as usize) {
        let p = (*cptext)[i];
        if !p.is_null() {
            xfree_add(p.cast::<u8>());
        }
    }
}

/// Add a match to the completion list.
///
/// Mirrors C `ins_compl_add`. Returns OK, NOTDONE (duplicate), or FAIL.
///
/// # Safety
/// Requires valid global completion state.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_add(
    str: *const c_char,
    len: c_int,
    fname: *const c_char,
    cptext: *mut [*mut c_char; 4],
    cptext_allocated: c_int,
    user_data: *mut c_void,
    cdir: c_int,
    flags_arg: c_int,
    adup: c_int,
    user_hl: *const c_int,
    score: c_int,
) -> c_int {
    let cptext_allocated = cptext_allocated != 0;
    let adup = adup != 0;

    let dir = if cdir == K_DIRECTION_NOT_SET {
        crate::vars::nvim_get_compl_direction()
    } else {
        cdir
    };
    let mut flags = flags_arg;

    if flags & CP_FAST != 0 {
        fast_breakcheck();
    } else {
        os_breakcheck();
    }
    if got_int_add {
        if cptext_allocated {
            free_cptext(cptext);
        }
        return FAIL;
    }

    let len = if len < 0 { strlen(str) as c_int } else { len };

    // Deduplication: if same match already present, skip.
    let first = nvim_compl_get_first_match();
    if !first.is_null() && !adup {
        let mut m = first;
        loop {
            if nvim_compl_match_at_original_text(m) == 0 {
                let data = nvim_compl_match_get_cp_str_data(m);
                let size = nvim_compl_match_get_cp_str_size(m);
                if !data.is_null()
                    && strncmp(data, str, len as usize) == 0
                    && (size <= len as usize || *data.add(len as usize) == b'\0' as c_char)
                {
                    if rs_is_nearest_active() != 0
                        && score > 0
                        && score < nvim_compl_match_get_score(m)
                    {
                        nvim_compl_match_set_score(m, score);
                    }
                    if cptext_allocated {
                        free_cptext(cptext);
                    }
                    return NOTDONE;
                }
            }
            let next = nvim_compl_match_get_next(m);
            if next.is_null() || is_first_match(next) {
                break;
            }
            m = next;
        }
    }

    // Remove popup menu before modifying the list.
    rs_ins_compl_del_pum();

    // Allocate new match.
    let match_ptr = ComplMatch(nvim_compl_T_alloc());
    nvim_compl_match_set_cp_number(
        match_ptr,
        if flags & CP_ORIGINAL_TEXT != 0 { 0 } else { -1 },
    );
    nvim_compl_match_set_cp_str(match_ptr, str, len as usize);

    // Set cp_fname
    let curr = nvim_compl_get_curr_match();
    if !fname.is_null() {
        if !curr.is_null() {
            let curr_fname = nvim_compl_match_get_cp_fname(curr);
            if !curr_fname.is_null()
                && strncmp(fname, curr_fname, strlen(fname).max(strlen(curr_fname)) + 1) == 0
            {
                nvim_compl_match_set_cp_fname_ref(match_ptr, curr_fname);
            } else {
                nvim_compl_match_set_cp_fname_dup(match_ptr, fname);
                flags |= CP_FREE_FNAME;
            }
        } else {
            nvim_compl_match_set_cp_fname_dup(match_ptr, fname);
            flags |= CP_FREE_FNAME;
        }
    }
    nvim_compl_match_set_flags(match_ptr, flags);
    nvim_compl_match_set_hl_attrs(
        match_ptr,
        if !user_hl.is_null() { *user_hl } else { -1 },
        if !user_hl.is_null() {
            *user_hl.add(1)
        } else {
            -1
        },
    );
    nvim_compl_match_set_score(match_ptr, score);
    nvim_compl_match_set_cpt_source_idx(match_ptr, crate::vars::nvim_get_cpt_sources_index());

    // Copy cptext
    if !cptext.is_null() {
        for i in 0..(CPT_COUNT as usize) {
            let p = (*cptext)[i];
            if p.is_null() {
                continue;
            }
            if *p != b'\0' as c_char {
                if cptext_allocated {
                    nvim_compl_match_set_cp_text_take(match_ptr, i as c_int, p);
                } else {
                    nvim_compl_match_set_cp_text_copy(match_ptr, i as c_int, p);
                }
            } else if cptext_allocated {
                xfree_add(p.cast::<u8>());
            }
        }
    }

    // Copy user_data
    if !user_data.is_null() {
        nvim_compl_match_set_user_data_move(match_ptr, user_data);
    }

    // Link into list
    let first = nvim_compl_get_first_match();
    if first.is_null() {
        // Empty list: match has no prev or next.
        nvim_compl_match_set_next(match_ptr, ComplMatch::null());
        nvim_compl_match_set_prev(match_ptr, ComplMatch::null());
    } else if rs_cot_fuzzy() != 0
        && score != FUZZY_SCORE_NONE
        && crate::vars::nvim_get_compl_get_longest() != 0
    {
        // Sorted insertion by score (descending) for fuzzy+longest.
        let mut current = nvim_compl_match_get_next(first);
        let mut prev = first;
        let mut inserted = false;
        while !current.is_null() && current != first {
            if nvim_compl_match_get_score(current) < score {
                nvim_compl_match_set_next(match_ptr, current);
                nvim_compl_match_set_prev(match_ptr, nvim_compl_match_get_prev(current));
                let cprev = nvim_compl_match_get_prev(current);
                if !cprev.is_null() {
                    nvim_compl_match_set_next(cprev, match_ptr);
                }
                nvim_compl_match_set_prev(current, match_ptr);
                inserted = true;
                break;
            }
            prev = current;
            current = nvim_compl_match_get_next(current);
        }
        if !inserted {
            nvim_compl_match_set_next(prev, match_ptr);
            nvim_compl_match_set_prev(match_ptr, prev);
            nvim_compl_match_set_next(match_ptr, first);
            nvim_compl_match_set_prev(first, match_ptr);
        }
    } else if dir == FORWARD {
        let curr = nvim_compl_get_curr_match();
        nvim_compl_match_set_next(match_ptr, nvim_compl_match_get_next(curr));
        nvim_compl_match_set_prev(match_ptr, curr);
    } else {
        // BACKWARD
        let curr = nvim_compl_get_curr_match();
        nvim_compl_match_set_next(match_ptr, curr);
        nvim_compl_match_set_prev(match_ptr, nvim_compl_match_get_prev(curr));
    }

    // Fix up neighbor pointers.
    let mnext = nvim_compl_match_get_next(match_ptr);
    if !mnext.is_null() {
        nvim_compl_match_set_prev(mnext, match_ptr);
    }
    let mprev = nvim_compl_match_get_prev(match_ptr);
    if !mprev.is_null() {
        nvim_compl_match_set_next(mprev, match_ptr);
    } else {
        // No previous node → this is the new first match.
        nvim_compl_set_first_match(match_ptr);
    }
    nvim_compl_set_curr_match(match_ptr);

    // Update longest common string if needed.
    if crate::vars::nvim_get_compl_get_longest() != 0
        && (flags & CP_ORIGINAL_TEXT) == 0
        && rs_cot_fuzzy() == 0
        && rs_ins_compl_preinsert_longest() == 0
    {
        rs_ins_compl_longest_match(match_ptr);
    }

    OK
}

/// Infer case from original text and add a match.
///
/// Mirrors C `ins_compl_add_infercase`. Returns OK, NOTDONE, or FAIL.
///
/// # Safety
/// Requires valid global completion state.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn rs_ins_compl_add_infercase(
    str_arg: *mut c_char,
    len: c_int,
    icase: c_int,
    fname: *const c_char,
    dir: c_int,
    cont_s_ipos: c_int,
    score: c_int,
) -> c_int {
    let mut str = str_arg;
    let mut flags: c_int = 0;
    let mut tofree: *mut c_char = ptr::null_mut();

    if crate::vars::nvim_get_p_ic() != 0 && nvim_get_p_inf() != 0 && len > 0 {
        // Count multi-byte chars in completion string.
        let char_len = {
            let mut p = str as *const c_char;
            let mut n = 0;
            while *p != b'\0' as c_char {
                p = p.add(utfc_ptr2len(p) as usize);
                n += 1;
            }
            n
        };
        // Count multi-byte chars in original text.
        let compl_char_len = {
            let orig_data = crate::vars::nvim_get_compl_orig_text_data();
            if orig_data.is_null() {
                0
            } else {
                let mut p = orig_data;
                let mut n = 0;
                while *p != b'\0' as c_char {
                    p = p.add(utfc_ptr2len(p) as usize);
                    n += 1;
                }
                n
            }
        };
        let min_len = char_len.min(compl_char_len);
        str = rs_ins_compl_infercase_gettext(str, char_len, compl_char_len, min_len, &mut tofree);
    }

    if cont_s_ipos != 0 {
        flags |= CP_CONT_S_IPOS;
    }
    if icase != 0 {
        flags |= CP_ICASE;
    }

    let res = rs_ins_compl_add(
        str,
        len,
        fname,
        ptr::null_mut(),
        0, // cptext_allocated = false
        ptr::null_mut(),
        dir,
        flags,
        0, // adup = false
        ptr::null(),
        score,
    );
    if !tofree.is_null() {
        xfree_add(tofree.cast::<u8>());
    }
    res
}

/// Check if a completion match's string equals a given prefix.
///
/// Mirrors `ins_compl_equal` in C: checks `CP_EQUAL` flag first (always
/// matches), then `CP_ICASE` for case-insensitive comparison, then plain
/// `strncmp`.
///
/// Returns 1 if match, 0 if no match.
///
/// # Safety
/// `m` must be a valid completion match handle. `str` must point to a valid
/// byte sequence of at least `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_equal(
    m: ComplMatch,
    str: *const c_char,
    len: usize,
) -> c_int {
    if m.is_null() || str.is_null() {
        return 0;
    }

    let flags = nvim_compl_match_get_flags(m);

    if flags & CP_EQUAL != 0 {
        return 1;
    }

    let data = nvim_compl_match_get_cp_str_data(m);
    if data.is_null() {
        return 0;
    }

    let result = if flags & CP_ICASE != 0 {
        strncasecmp(data, str, len)
    } else {
        strncmp(data, str, len)
    };

    c_int::from(result == 0)
}

extern "C" {
    fn strncasecmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_constants() {
        assert_eq!(FORWARD, 1);
    }

    #[test]
    fn test_cp_flags() {
        assert_eq!(CP_ORIGINAL_TEXT, 1);
        assert_eq!(CP_FREE_FNAME, 2);
        assert_eq!(CP_CONT_S_IPOS, 4);
        assert_eq!(CP_EQUAL, 8);
        assert_eq!(CP_ICASE, 16);
        assert_eq!(CP_FAST, 32);
    }

    #[test]
    fn test_return_values() {
        assert_eq!(OK, 1);
        assert_eq!(FAIL, 0);
        assert_eq!(NOTDONE, 2);
    }
}
