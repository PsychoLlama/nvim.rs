//! Core search function `searchit()` migrated from search.c.
//!
//! This module implements the low-level search engine that powers `/`, `?`,
//! `n`, `N`, `*`, and all pattern-based search commands.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::too_many_lines)]

use std::ffi::{c_char, c_int, c_void};

use crate::helpers::options;

// =============================================================================
// Constants (verified with _Static_assert in search.c)
// =============================================================================

const FORWARD: c_int = 1;
const BACKWARD: c_int = -1;
const FAIL: c_int = 0;
const MAXCOL: c_int = 0x7fffffff;

// =============================================================================
// Type aliases
// =============================================================================

type LinenrT = i32;
type ColnrT = c_int;

/// Opaque handle to `buf_T`
type BufHandle = *mut c_void;
/// Opaque handle to `win_T`
type WinHandle = *mut c_void;
/// Opaque handle to `regmmatch_T`
type RegmmatchHandle = *mut c_void;
/// Opaque handle to `proftime_T`
type ProftimeHandle = *mut c_void;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    static mut got_int: bool;
    // Pattern compilation / regex
    fn nvim_search_regcomp(
        pat: *mut c_char,
        patlen: usize,
        pat_use: c_int,
        options: c_int,
        regmatch: RegmmatchHandle,
    ) -> c_int;
    fn nvim_searchit_regexec_multi(
        regmatch: RegmmatchHandle,
        win: WinHandle,
        buf: BufHandle,
        lnum: LinenrT,
        col: ColnrT,
        tm: ProftimeHandle,
        timed_out: *mut c_int,
    ) -> c_int;
    fn nvim_searchit_regfree(regmatch: RegmmatchHandle);
    fn nvim_regmatch_regprog_is_null(regmatch: RegmmatchHandle) -> c_int;
    fn nvim_regmatch_startpos_lnum(regmatch: RegmmatchHandle, idx: c_int) -> LinenrT;
    fn nvim_regmatch_startpos_col(regmatch: RegmmatchHandle, idx: c_int) -> ColnrT;
    fn nvim_regmatch_endpos_lnum(regmatch: RegmmatchHandle, idx: c_int) -> LinenrT;
    fn nvim_regmatch_endpos_col(regmatch: RegmmatchHandle, idx: c_int) -> ColnrT;
    fn nvim_regmatch_rmm_matchcol(regmatch: RegmmatchHandle) -> ColnrT;

    // Buffer / line accessors
    #[link_name = "ml_get_buf"]
    fn nvim_ml_get_buf(buf: BufHandle, lnum: LinenrT) -> *mut c_char;
    fn nvim_ml_get_buf_len(buf: BufHandle, lnum: LinenrT) -> ColnrT;
    fn nvim_buf_get_line_count(buf: BufHandle) -> LinenrT;

    // Multibyte
    fn nvim_utfc_ptr2len(p: *const c_char) -> c_int;
    #[link_name = "utf_head_off"]
    fn nvim_utf_head_off(base: *const c_char, p: *const c_char) -> c_int;

    // Global state
    static mut called_emsg: c_int;
    fn nvim_get_rc_did_emsg() -> c_int;
    fn nvim_get_p_ws() -> c_int;
    fn nvim_cpo_has_search() -> c_int;

    // Shortmess
    fn shortmess(x: c_int) -> bool;

    // Incsearch / char avail
    fn nvim_char_avail() -> c_int;
    fn line_breakcheck();

    // Timeout
    fn nvim_profile_passed_limit(tm: ProftimeHandle) -> c_int;

    // Search match state setters
    fn nvim_set_search_match_lines(val: c_int);
    fn nvim_set_search_match_endcol(val: c_int);

    // Error messages
    fn nvim_searchit_emsg_patnotf(p_ws_val: c_int, lnum: LinenrT);
    fn nvim_searchit_emsg_invalid();
    fn nvim_searchit_emsg_interr();
    fn nvim_searchit_give_warning(dir: c_int);

    // Regmmatch allocation
    fn nvim_regmmatch_alloc() -> RegmmatchHandle;
    #[link_name = "xfree"]
    fn nvim_regmmatch_free(rm: RegmmatchHandle);
}

// =============================================================================
// Shortmess constants (verified with _Static_assert in search.c)
// =============================================================================

const SHM_SEARCH: c_int = b's' as c_int;
const SHM_SEARCHCOUNT: c_int = b'S' as c_int;

// =============================================================================
// Helper: first_submatch
// =============================================================================

/// Return the number of the first subpattern that matched.
/// Return zero if none of them matched.
fn first_submatch(regmatch: RegmmatchHandle) -> c_int {
    unsafe {
        for submatch in 1..=9 {
            if nvim_regmatch_startpos_lnum(regmatch, submatch) >= 0 {
                return submatch;
            }
            if submatch == 9 {
                return 0;
            }
        }
        0
    }
}

// =============================================================================
// Searchit result
// =============================================================================

/// Result from rs_searchit, written back to caller's pos_T and end_pos.
#[repr(C)]
pub struct SearchitResult {
    /// FAIL (0) on failure, submatch+1 on success
    pub retval: c_int,
    /// Result position: line number
    pub pos_lnum: LinenrT,
    /// Result position: column
    pub pos_col: ColnrT,
    /// Result position: coladd
    pub pos_coladd: ColnrT,
    /// End position: line number
    pub end_lnum: LinenrT,
    /// End position: column
    pub end_col: ColnrT,
    /// End position: coladd
    pub end_coladd: ColnrT,
    /// Whether end_pos was written
    pub end_pos_set: c_int,
    /// Updated sa_timed_out
    pub sa_timed_out: c_int,
    /// Updated sa_wrapped
    pub sa_wrapped: c_int,
}

// =============================================================================
// Main searchit implementation
// =============================================================================

/// Rust implementation of searchit().
///
/// # Safety
/// All pointer arguments must be valid for the duration of the call.
#[no_mangle]
pub unsafe extern "C" fn rs_searchit(
    win: WinHandle,
    buf: BufHandle,
    pos_lnum: LinenrT,
    pos_col: ColnrT,
    pos_coladd: ColnrT,
    has_end_pos: c_int,
    dir: c_int,
    pat: *mut c_char,
    patlen: usize,
    count: c_int,
    search_options: c_int,
    pat_use: c_int,
    sa_stop_lnum: LinenrT,
    sa_tm: ProftimeHandle,
    has_extra_arg: c_int,
    result: *mut SearchitResult,
) -> c_int {
    let res = &mut *result;
    // Initialize result
    res.retval = FAIL;
    res.pos_lnum = pos_lnum;
    res.pos_col = pos_col;
    res.pos_coladd = pos_coladd;
    res.end_lnum = 0;
    res.end_col = 0;
    res.end_coladd = 0;
    res.end_pos_set = 0;
    res.sa_timed_out = 0;
    res.sa_wrapped = 0;

    let stop_lnum = if has_extra_arg != 0 { sa_stop_lnum } else { 0 };
    let tm: ProftimeHandle = if has_extra_arg != 0 {
        sa_tm
    } else {
        std::ptr::null_mut()
    };
    let has_timed_out = has_extra_arg != 0;

    let line_count = nvim_buf_get_line_count(buf);

    // Allocate regmmatch_T on the heap
    let regmatch = nvim_regmmatch_alloc();

    // Compile the search pattern
    let regcomp_opts = search_options & (options::SEARCH_HIS + options::SEARCH_KEEP);
    if nvim_search_regcomp(pat, patlen, pat_use, regcomp_opts, regmatch) == FAIL {
        if (search_options & options::SEARCH_MSG) != 0 && nvim_get_rc_did_emsg() == 0 {
            nvim_searchit_emsg_invalid();
        }
        nvim_regmmatch_free(regmatch);
        return FAIL;
    }

    let search_from_match_end = nvim_cpo_has_search() != 0;
    let called_emsg_before = called_emsg;

    // Working position
    let mut pos_l = res.pos_lnum;
    let mut pos_c = res.pos_col;

    let mut count_left = count;
    let mut found;
    let mut submatch: c_int = 0;
    let mut first_match = true;
    let mut break_loop = false;
    #[allow(unused_assignments)]
    let mut final_lnum: LinenrT = 0; // track lnum for error messages

    // do { ... } while (--count > 0 && found)
    loop {
        // Compute extra_col for start position handling
        let start_char_len;
        if pos_c == MAXCOL {
            start_char_len = 0;
        } else if pos_l >= 1 && pos_l <= line_count && pos_c < MAXCOL - 2 {
            let ptr = nvim_ml_get_buf(buf, pos_l);
            let line_len = nvim_ml_get_buf_len(buf, pos_l);
            if line_len <= pos_c {
                start_char_len = 1;
            } else {
                start_char_len = nvim_utfc_ptr2len(ptr.add(pos_c as usize));
            }
        } else {
            start_char_len = 1;
        }

        let extra_col = if dir == FORWARD {
            if (search_options & options::SEARCH_START) != 0 {
                0
            } else {
                start_char_len
            }
        } else if (search_options & options::SEARCH_START) != 0 {
            start_char_len
        } else {
            0
        };

        let start_pos_lnum = pos_l;
        let start_pos_col = pos_c;
        found = false;
        let mut at_first_line = true;

        if pos_l == 0 {
            pos_l = 1;
            pos_c = 0;
            at_first_line = false;
        }

        let mut lnum: LinenrT;
        if dir == BACKWARD && start_pos_col == 0 && (search_options & options::SEARCH_START) == 0 {
            lnum = pos_l - 1;
            at_first_line = false;
        } else {
            lnum = pos_l;
        }

        // loop twice if 'wrapscan' set
        for wrap_loop in 0..=1i32 {
            // Inner line-scanning loop
            while lnum > 0 && lnum <= line_count {
                // Stop after checking stop_lnum
                if stop_lnum != 0
                    && (if dir == FORWARD {
                        lnum > stop_lnum
                    } else {
                        lnum < stop_lnum
                    })
                {
                    break;
                }

                // Stop after passing time limit
                if !tm.is_null() && nvim_profile_passed_limit(tm) != 0 {
                    break;
                }

                // Look for a match somewhere in line "lnum"
                let col: ColnrT = if at_first_line && (search_options & options::SEARCH_COL) != 0 {
                    pos_c
                } else {
                    0
                };

                let mut nmatched = nvim_searchit_regexec_multi(
                    regmatch,
                    win,
                    buf,
                    lnum,
                    col,
                    tm,
                    if has_timed_out {
                        &mut res.sa_timed_out
                    } else {
                        std::ptr::null_mut()
                    },
                );

                // vim_regexec_multi() may clear "regprog"
                if nvim_regmatch_regprog_is_null(regmatch) != 0 {
                    break;
                }

                // Abort on error or timeout
                if called_emsg > called_emsg_before || (has_timed_out && res.sa_timed_out != 0) {
                    break;
                }

                if nmatched > 0 {
                    let mut matchpos_lnum = nvim_regmatch_startpos_lnum(regmatch, 0);
                    let mut matchpos_col = nvim_regmatch_startpos_col(regmatch, 0);
                    let mut endpos_lnum = nvim_regmatch_endpos_lnum(regmatch, 0);
                    let mut endpos_col = nvim_regmatch_endpos_col(regmatch, 0);
                    submatch = first_submatch(regmatch);

                    // Get line pointer (may be past end for "\n\zs")
                    let mut ptr = if lnum + matchpos_lnum > line_count {
                        c"".as_ptr() as *mut c_char
                    } else {
                        nvim_ml_get_buf(buf, lnum + matchpos_lnum)
                    };

                    // Forward search in the first line
                    if dir == FORWARD && at_first_line {
                        let mut match_ok = true;

                        while matchpos_lnum == 0
                            && (if (search_options & options::SEARCH_END) != 0 && first_match {
                                nmatched == 1 && endpos_col - 1 < start_pos_col + extra_col
                            } else {
                                matchpos_col
                                    - (if *ptr.add(matchpos_col as usize) == 0 {
                                        1
                                    } else {
                                        0
                                    })
                                    < start_pos_col + extra_col
                            })
                        {
                            let matchcol = if search_from_match_end {
                                if nmatched > 1 {
                                    match_ok = false;
                                    break;
                                }
                                if endpos_col == matchpos_col
                                    && *ptr.add(matchpos_col as usize) != 0
                                {
                                    endpos_col + nvim_utfc_ptr2len(ptr.add(endpos_col as usize))
                                } else {
                                    endpos_col
                                }
                            } else {
                                let rmm = nvim_regmatch_rmm_matchcol(regmatch);
                                if *ptr.add(rmm as usize) != 0 {
                                    rmm + nvim_utfc_ptr2len(ptr.add(rmm as usize))
                                } else {
                                    rmm
                                }
                            };

                            if matchcol == 0 && (search_options & options::SEARCH_START) != 0 {
                                break;
                            }

                            if *ptr.add(matchcol as usize) == 0 {
                                match_ok = false;
                                break;
                            }

                            nmatched = nvim_searchit_regexec_multi(
                                regmatch,
                                win,
                                buf,
                                lnum,
                                matchcol,
                                tm,
                                if has_timed_out {
                                    &mut res.sa_timed_out
                                } else {
                                    std::ptr::null_mut()
                                },
                            );
                            if nmatched == 0 {
                                match_ok = false;
                                break;
                            }

                            if nvim_regmatch_regprog_is_null(regmatch) != 0 {
                                break;
                            }

                            matchpos_lnum = nvim_regmatch_startpos_lnum(regmatch, 0);
                            matchpos_col = nvim_regmatch_startpos_col(regmatch, 0);
                            endpos_lnum = nvim_regmatch_endpos_lnum(regmatch, 0);
                            endpos_col = nvim_regmatch_endpos_col(regmatch, 0);
                            submatch = first_submatch(regmatch);

                            if matchpos_lnum != 0 {
                                break;
                            }
                            // Re-get line pointer (multi-line search may invalidate it)
                            ptr = nvim_ml_get_buf(buf, lnum);
                        }

                        if !match_ok {
                            // Advance to next line
                            lnum += dir;
                            at_first_line = false;
                            continue;
                        }
                    }

                    // Backward search: find last match before cursor
                    if dir == BACKWARD {
                        let mut match_ok = false;

                        loop {
                            let cur_start_lnum = nvim_regmatch_startpos_lnum(regmatch, 0);
                            let cur_start_col = nvim_regmatch_startpos_col(regmatch, 0);
                            let cur_end_lnum = nvim_regmatch_endpos_lnum(regmatch, 0);
                            let cur_end_col = nvim_regmatch_endpos_col(regmatch, 0);

                            if wrap_loop != 0
                                || (if (search_options & options::SEARCH_END) != 0 {
                                    lnum + cur_end_lnum < start_pos_lnum
                                        || (lnum + cur_end_lnum == start_pos_lnum
                                            && cur_end_col - 1 < start_pos_col + extra_col)
                                } else {
                                    lnum + cur_start_lnum < start_pos_lnum
                                        || (lnum + cur_start_lnum == start_pos_lnum
                                            && cur_start_col < start_pos_col + extra_col)
                                })
                            {
                                match_ok = true;
                                matchpos_lnum = cur_start_lnum;
                                matchpos_col = cur_start_col;
                                endpos_lnum = cur_end_lnum;
                                endpos_col = cur_end_col;
                                submatch = first_submatch(regmatch);
                            } else {
                                break;
                            }

                            // Try to find another match after this one
                            let matchcol = if search_from_match_end {
                                if nmatched > 1 {
                                    break;
                                }
                                if endpos_col == matchpos_col
                                    && *ptr.add(matchpos_col as usize) != 0
                                {
                                    endpos_col + nvim_utfc_ptr2len(ptr.add(endpos_col as usize))
                                } else {
                                    endpos_col
                                }
                            } else {
                                if matchpos_lnum > 0 {
                                    break;
                                }
                                if *ptr.add(matchpos_col as usize) != 0 {
                                    matchpos_col + nvim_utfc_ptr2len(ptr.add(matchpos_col as usize))
                                } else {
                                    matchpos_col
                                }
                            };

                            if *ptr.add(matchcol as usize) == 0 {
                                if !tm.is_null() && nvim_profile_passed_limit(tm) != 0 {
                                    match_ok = false;
                                }
                                break;
                            }

                            nmatched = nvim_searchit_regexec_multi(
                                regmatch,
                                win,
                                buf,
                                lnum + matchpos_lnum,
                                matchcol,
                                tm,
                                if has_timed_out {
                                    &mut res.sa_timed_out
                                } else {
                                    std::ptr::null_mut()
                                },
                            );
                            if nmatched == 0 {
                                if !tm.is_null() && nvim_profile_passed_limit(tm) != 0 {
                                    match_ok = false;
                                }
                                break;
                            }

                            if nvim_regmatch_regprog_is_null(regmatch) != 0 {
                                break;
                            }

                            // Re-get line pointer
                            ptr = nvim_ml_get_buf(buf, lnum + matchpos_lnum);
                        }

                        if !match_ok {
                            // Advance to next line
                            lnum += dir;
                            at_first_line = false;
                            continue;
                        }
                    }

                    // With SEARCH_END option, move to last char of match
                    if (search_options & options::SEARCH_END) != 0
                        && (search_options & options::SEARCH_NOOF) == 0
                        && !(matchpos_lnum == endpos_lnum && matchpos_col == endpos_col)
                    {
                        pos_l = lnum + endpos_lnum;
                        pos_c = endpos_col;
                        if endpos_col == 0 {
                            if pos_l > 1 {
                                pos_l -= 1;
                                pos_c = nvim_ml_get_buf_len(buf, pos_l);
                            }
                        } else {
                            pos_c -= 1;
                            if pos_l <= line_count {
                                let line_ptr = nvim_ml_get_buf(buf, pos_l);
                                pos_c -= nvim_utf_head_off(line_ptr, line_ptr.add(pos_c as usize));
                            }
                        }
                        if has_end_pos != 0 {
                            res.end_lnum = lnum + matchpos_lnum;
                            res.end_col = matchpos_col;
                            res.end_pos_set = 1;
                        }
                    } else {
                        pos_l = lnum + matchpos_lnum;
                        pos_c = matchpos_col;
                        if has_end_pos != 0 {
                            res.end_lnum = lnum + endpos_lnum;
                            res.end_col = endpos_col;
                            res.end_pos_set = 1;
                        }
                    }
                    res.pos_coladd = 0;
                    if has_end_pos != 0 {
                        res.end_coladd = 0;
                    }
                    found = true;
                    first_match = false;

                    // Set variables used for 'incsearch' highlighting
                    nvim_set_search_match_lines(endpos_lnum - matchpos_lnum);
                    nvim_set_search_match_endcol(endpos_col);
                    break;
                }

                line_breakcheck();
                if unsafe { got_int } {
                    break;
                }

                // Cancel searching if a character was typed (for 'incsearch')
                if (search_options & options::SEARCH_PEEK) != 0
                    && ((lnum - pos_l) & 0x3f) == 0
                    && nvim_char_avail() != 0
                {
                    break_loop = true;
                    break;
                }

                if wrap_loop != 0 && lnum == start_pos_lnum {
                    break;
                }

                lnum += dir;
                at_first_line = false;
            }

            at_first_line = false;

            // Check if regprog was cleared
            if nvim_regmatch_regprog_is_null(regmatch) != 0 {
                break;
            }

            // Stop conditions
            if nvim_get_p_ws() == 0
                || stop_lnum != 0
                || unsafe { got_int }
                || called_emsg > called_emsg_before
                || (has_timed_out && res.sa_timed_out != 0)
                || break_loop
                || found
                || wrap_loop != 0
            {
                break;
            }

            // Wrap around
            lnum = if dir == BACKWARD { line_count } else { 1 };

            if !shortmess(SHM_SEARCH)
                && shortmess(SHM_SEARCHCOUNT)
                && (search_options & options::SEARCH_MSG) != 0
            {
                nvim_searchit_give_warning(dir);
            }

            if has_extra_arg != 0 {
                res.sa_wrapped = 1;
            }
        }

        final_lnum = lnum;

        if unsafe { got_int }
            || called_emsg > called_emsg_before
            || (has_timed_out && res.sa_timed_out != 0)
            || break_loop
        {
            break;
        }

        count_left -= 1;
        if count_left <= 0 || !found {
            break;
        }
    }

    nvim_searchit_regfree(regmatch);
    nvim_regmmatch_free(regmatch);

    if !found {
        if unsafe { got_int } {
            nvim_searchit_emsg_interr();
        } else if (search_options & options::SEARCH_MSG) == options::SEARCH_MSG {
            nvim_searchit_emsg_patnotf(nvim_get_p_ws(), final_lnum);
        }
        return FAIL;
    }

    // A pattern like "\n\zs" may go past the last line
    if pos_l > line_count {
        pos_l = line_count;
        pos_c = nvim_ml_get_buf_len(buf, pos_l);
        if pos_c > 0 {
            pos_c -= 1;
        }
    }

    res.pos_lnum = pos_l;
    res.pos_col = pos_c;
    res.retval = submatch + 1;

    submatch + 1
}

// =============================================================================
// C struct types for direct export_name functions
// =============================================================================

/// pos_T layout matching the C struct.
#[repr(C)]
pub struct PosT {
    pub lnum: i32,
    pub col: i32,
    pub coladd: i32,
}

/// searchit_arg_T layout matching the C struct.
/// proftime_T* is opaque, so we use *mut c_void.
#[repr(C)]
pub struct SearchitArgT {
    pub sa_stop_lnum: i32,
    pub sa_tm: *mut c_void,
    pub sa_timed_out: c_int,
    pub sa_wrapped: c_int,
}

/// searchit: C-ABI entry point matching the original C function signature.
///
/// Accepts pos_T* and searchit_arg_T* directly and decomposes them for rs_searchit.
///
/// # Safety
/// All pointer arguments must be valid.
#[unsafe(export_name = "searchit")]
pub unsafe extern "C" fn searchit_export(
    win: WinHandle,
    buf: BufHandle,
    pos: *mut PosT,
    end_pos: *mut PosT,
    dir: c_int,
    pat: *mut c_char,
    patlen: usize,
    count: c_int,
    options: c_int,
    pat_use: c_int,
    extra_arg: *mut SearchitArgT,
) -> c_int {
    let pos_ref = &mut *pos;
    let has_end_pos = if end_pos.is_null() { 0 } else { 1 };
    let (sa_stop_lnum, sa_tm, has_extra_arg) = if extra_arg.is_null() {
        (0i32, std::ptr::null_mut(), 0)
    } else {
        let ea = &*extra_arg;
        (ea.sa_stop_lnum, ea.sa_tm, 1)
    };

    let mut result = SearchitResult {
        retval: 0,
        pos_lnum: pos_ref.lnum,
        pos_col: pos_ref.col,
        pos_coladd: pos_ref.coladd,
        end_lnum: 0,
        end_col: 0,
        end_coladd: 0,
        end_pos_set: 0,
        sa_timed_out: 0,
        sa_wrapped: 0,
    };

    let retval = rs_searchit(
        win,
        buf,
        pos_ref.lnum,
        pos_ref.col,
        pos_ref.coladd,
        has_end_pos,
        dir,
        pat,
        patlen,
        count,
        options,
        pat_use,
        sa_stop_lnum,
        sa_tm,
        has_extra_arg,
        &mut result,
    );

    pos_ref.lnum = result.pos_lnum;
    pos_ref.col = result.pos_col;
    pos_ref.coladd = result.pos_coladd;

    if !end_pos.is_null() && result.end_pos_set != 0 {
        let ep = &mut *end_pos;
        ep.lnum = result.end_lnum;
        ep.col = result.end_col;
        ep.coladd = result.end_coladd;
    }

    if !extra_arg.is_null() {
        let ea = &mut *extra_arg;
        ea.sa_timed_out = result.sa_timed_out;
        if result.sa_wrapped != 0 {
            ea.sa_wrapped = 1;
        }
    }

    retval
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(FORWARD, 1);
        assert_eq!(BACKWARD, -1);
        assert_eq!(FAIL, 0);
        assert_eq!(MAXCOL, 0x7fffffff);
        assert_eq!(SHM_SEARCH, b's' as c_int);
        assert_eq!(SHM_SEARCHCOUNT, b'S' as c_int);
    }

    #[test]
    fn test_searchit_result_size() {
        // Ensure the result struct is reasonable size
        assert!(std::mem::size_of::<SearchitResult>() > 0);
        assert!(std::mem::size_of::<SearchitResult>() <= 64);
    }
}
