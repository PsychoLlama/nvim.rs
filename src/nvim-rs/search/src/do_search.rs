//! High-level search function `do_search()` migrated from search.c.
//!
//! This module implements the top-level search command that powers
//! `/pattern`, `?pattern`, and chained searches like `/foo/;/bar`.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::too_many_lines)]

use std::ffi::{c_char, c_int, c_longlong, c_void};

use crate::helpers::options;
use crate::searchit::{self, SearchitResult};

// =============================================================================
// Constants (verified with _Static_assert in search.c)
// =============================================================================

const FORWARD: c_int = 1;
const BACKWARD: c_int = -1;
const FAIL: c_int = 0;
const RE_LAST: c_int = 2;
const NUL: c_char = 0;

// =============================================================================
// Type aliases
// =============================================================================

type LinenrT = i32;
type ColnrT = c_int;
type WinHandle = *mut c_void;
type BufHandle = *mut c_void;
type OapHandle = *mut c_void;

// =============================================================================
// FFI result types (must match C typedefs in search.c)
// =============================================================================

#[repr(C)]
struct DoSearchPos {
    lnum: LinenrT,
    col: ColnrT,
}

#[repr(C)]
struct DoSearchEchoResult {
    msgbuf: *mut c_char,
    msgbuflen: usize,
    show_search_stats: c_int,
}

#[repr(C)]
struct DoSearchPostOffset {
    lnum: LinenrT,
    col: ColnrT,
    retval: c_int,
    has_offset: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct SavedSearchOff {
    dir: c_char,
    line: c_int,
    end: c_int,
    off: c_longlong,
}

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    // do_search-specific helpers
    fn nvim_do_search_check_lineoff() -> c_int;
    fn nvim_do_search_clear_lineoff();
    fn nvim_do_search_get_dirc() -> c_int;
    fn nvim_do_search_set_dirc(dirc: c_int);
    fn nvim_do_search_fold_adjust(dirc: c_int, lnum: LinenrT, col: ColnrT) -> DoSearchPos;
    fn nvim_do_search_hlsearch_on(options: c_int);
    fn nvim_do_search_get_search_pat() -> *mut c_char;
    fn nvim_do_search_get_subst_pat() -> *mut c_char;
    fn nvim_do_search_get_subst_patlen() -> usize;
    fn nvim_do_search_skip_regexp(
        pat: *mut c_char,
        delim: c_int,
        newp: *mut *mut c_char,
    ) -> *mut c_char;
    fn nvim_do_search_set_searchcmdlen(val: c_int);
    fn nvim_do_search_get_searchcmdlen() -> c_int;
    fn nvim_do_search_set_off(off_line: c_int, off_end: c_int, off_off: c_longlong);
    fn nvim_do_search_get_off_end() -> c_int;
    fn nvim_do_search_get_off_line() -> c_int;
    fn nvim_do_search_get_off_off() -> c_longlong;
    fn nvim_do_search_echo(
        dirc: c_int,
        options: c_int,
        searchstr: *const c_char,
        searchstrlen: usize,
    ) -> DoSearchEchoResult;
    fn nvim_do_search_echo_free(msgbuf: *mut c_char);
    fn nvim_do_search_pre_offset(lnum: LinenrT, col: ColnrT) -> DoSearchPos;
    fn nvim_do_search_post_offset(
        lnum: LinenrT,
        col: ColnrT,
        options: c_int,
        pat_has_semicolon: c_int,
    ) -> DoSearchPostOffset;
    fn nvim_do_search_show_top_bot(dirc: c_int, pos_lnum: LinenrT, pos_col: ColnrT) -> c_int;
    fn nvim_do_search_set_oap_inclusive(oap: OapHandle);
    fn nvim_do_search_autocmd_wrapped();
    fn nvim_do_search_show_stats(
        dirc: c_int,
        pos_lnum: LinenrT,
        pos_col: ColnrT,
        show_top_bot: c_int,
        msgbuf: *mut c_char,
        msgbuflen: usize,
        count: c_int,
        has_offset: c_int,
    );
    fn nvim_do_search_emsg_e386();
    fn nvim_do_search_emsg_noprevre();
    fn nvim_do_search_finish(options: c_int, lnum: LinenrT, col: ColnrT);
    fn nvim_do_search_save_off() -> SavedSearchOff;
    fn nvim_do_search_restore_off(saved: SavedSearchOff);
    fn nvim_do_search_get_cursor() -> DoSearchPos;

    // Existing accessors
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_get_curbuf() -> BufHandle;

    // String helpers
    fn strlen(s: *const c_char) -> usize;
    fn xfree(ptr: *mut c_void);
}

// =============================================================================
// ASCII helpers
// =============================================================================

fn ascii_isdigit(c: u8) -> bool {
    c.is_ascii_digit()
}

fn atol_ptr(p: *const c_char) -> i64 {
    unsafe {
        let mut s = p;
        let negative = *s == b'-' as c_char;
        if *s == b'+' as c_char || *s == b'-' as c_char {
            s = s.add(1);
        }
        let mut val: i64 = 0;
        while (*s as u8).is_ascii_digit() {
            val = val * 10 + (*s as u8 - b'0') as i64;
            s = s.add(1);
        }
        if negative {
            -val
        } else {
            val
        }
    }
}

// =============================================================================
// Main do_search implementation
// =============================================================================

/// Rust implementation of do_search().
///
/// # Safety
/// All pointer arguments must be valid for the duration of the call.
#[no_mangle]
pub unsafe extern "C" fn rs_do_search(
    oap: OapHandle,
    dirc_in: c_int,
    search_delim: c_int,
    pat_in: *mut c_char,
    patlen_in: usize,
    count: c_int,
    search_options: c_int,
    // searchit_arg_T fields
    has_sia: c_int,
    sa_stop_lnum: LinenrT,
    sa_tm: *mut c_void,
    sa_timed_out_out: *mut c_int,
    sa_wrapped_out: *mut c_int,
) -> c_int {
    // Initialize searchcmdlen
    nvim_do_search_set_searchcmdlen(0);

    // A line offset is not remembered (vi compatible)
    if nvim_do_search_check_lineoff() != 0 {
        nvim_do_search_clear_lineoff();
    }

    // Save the offset for SEARCH_KEEP
    let saved_off = nvim_do_search_save_off();

    // Start position = cursor
    let cursor = nvim_do_search_get_cursor();
    let mut pos_lnum = cursor.lnum;
    let mut pos_col = cursor.col;

    // Find out search direction
    let mut dirc = if dirc_in == 0 {
        nvim_do_search_get_dirc()
    } else {
        nvim_do_search_set_dirc(dirc_in);
        dirc_in
    };

    if (search_options & options::SEARCH_REV) != 0 {
        dirc = if dirc == b'/' as c_int {
            b'?' as c_int
        } else {
            b'/' as c_int
        };
    }

    // Fold adjustment
    let fold_pos = nvim_do_search_fold_adjust(dirc, pos_lnum, pos_col);
    pos_lnum = fold_pos.lnum;
    pos_col = fold_pos.col;

    // Turn hlsearch back on
    nvim_do_search_hlsearch_on(search_options);

    let mut pat = pat_in;
    let mut patlen = patlen_in;
    let mut search_delim_cur = search_delim;
    let mut strcopy: *mut c_char = std::ptr::null_mut();
    let mut retval: c_int;
    let mut has_offset = false;

    // Main search loop (for chained searches with ';')
    loop {
        let mut show_top_bot_msg = false;
        let mut searchstr: *mut c_char = pat;
        let mut searchstrlen = patlen;
        let mut dircp: *mut c_char = std::ptr::null_mut();

        // Use previous pattern if current is empty
        if pat.is_null() || *pat == NUL || *pat == search_delim_cur as c_char {
            if nvim_do_search_get_search_pat().is_null() {
                if nvim_do_search_get_subst_pat().is_null() {
                    nvim_do_search_emsg_noprevre();
                    retval = 0;
                    break;
                }
                searchstr = nvim_do_search_get_subst_pat();
                searchstrlen = nvim_do_search_get_subst_patlen();
            } else {
                // make search_regcomp() use spats[RE_SEARCH].pat
                searchstr = c"".as_ptr() as *mut c_char;
                searchstrlen = 0;
            }
        }

        if !pat.is_null() && *pat != NUL {
            // Find end of regular expression
            let ps = strcopy;
            let p = nvim_do_search_skip_regexp(pat, search_delim_cur, &mut strcopy);
            if strcopy != ps {
                let len = strlen(strcopy);
                let cmdlen = nvim_do_search_get_searchcmdlen();
                nvim_do_search_set_searchcmdlen(cmdlen + (patlen as c_int - len as c_int));
                pat = strcopy;
                patlen = len;
                searchstr = strcopy;
                searchstrlen = len;
            }

            let mut p_cur = p;
            if *p_cur == search_delim_cur as c_char {
                searchstrlen = p_cur.offset_from(pat) as usize;
                dircp = p_cur;
                *p_cur = NUL;
                p_cur = p_cur.add(1);
            }

            // Reset offsets
            nvim_do_search_set_off(0, 0, 0);

            // Check for line offset or character offset
            let pc = *p_cur as u8;
            if pc == b'+' || pc == b'-' || ascii_isdigit(pc) {
                nvim_do_search_set_off(1, 0, 0); // line offset
            } else if (search_options & options::SEARCH_OPT) != 0
                && (pc == b'e' || pc == b's' || pc == b'b')
            {
                if pc == b'e' {
                    nvim_do_search_set_off(
                        nvim_do_search_get_off_line(),
                        1,
                        nvim_do_search_get_off_off(),
                    );
                }
                p_cur = p_cur.add(1);
            }

            let pc2 = *p_cur as u8;
            if ascii_isdigit(pc2) || pc2 == b'+' || pc2 == b'-' {
                let off_val = if ascii_isdigit(pc2) || ascii_isdigit(*p_cur.add(1) as u8) {
                    atol_ptr(p_cur)
                } else if pc2 == b'-' {
                    -1i64
                } else {
                    1i64
                };
                nvim_do_search_set_off(
                    nvim_do_search_get_off_line(),
                    nvim_do_search_get_off_end(),
                    off_val,
                );
                p_cur = p_cur.add(1);
                while (*p_cur as u8).is_ascii_digit() {
                    p_cur = p_cur.add(1);
                }
            }

            // Compute length of search command for get_address()
            let cmdlen = nvim_do_search_get_searchcmdlen();
            nvim_do_search_set_searchcmdlen(cmdlen + p_cur.offset_from(pat) as c_int);

            patlen -= p_cur.offset_from(pat) as usize;
            pat = p_cur;
        }

        // Echo the search pattern (batch helper in C)
        let echo_result = nvim_do_search_echo(dirc, search_options, searchstr, searchstrlen);

        // Pre-searchit character offset subtraction
        let pre_off = nvim_do_search_pre_offset(pos_lnum, pos_col);
        pos_lnum = pre_off.lnum;
        pos_col = pre_off.col;

        // Build searchit options
        let searchit_opts = nvim_do_search_get_off_end() * options::SEARCH_END
            + (search_options
                & (options::SEARCH_KEEP
                    | options::SEARCH_PEEK
                    | options::SEARCH_HIS
                    | options::SEARCH_MSG
                    | options::SEARCH_START
                    | (if !pat.is_null() && *pat == b';' as c_char {
                        0
                    } else {
                        options::SEARCH_NOOF
                    })));

        let search_dir = if dirc == b'/' as c_int {
            FORWARD
        } else {
            BACKWARD
        };

        // Call searchit (Rust-internal!)
        let mut sit_result = SearchitResult {
            retval: 0,
            pos_lnum,
            pos_col,
            pos_coladd: 0,
            end_lnum: 0,
            end_col: 0,
            end_coladd: 0,
            end_pos_set: 0,
            sa_timed_out: 0,
            sa_wrapped: 0,
        };

        let win = nvim_get_curwin();
        let buf = nvim_get_curbuf();

        let c = searchit::rs_searchit(
            win,
            buf,
            pos_lnum,
            pos_col,
            0, // coladd
            0, // no end_pos
            search_dir,
            searchstr,
            searchstrlen,
            count,
            searchit_opts,
            RE_LAST,
            sa_stop_lnum,
            sa_tm,
            has_sia,
            &mut sit_result,
        );

        pos_lnum = sit_result.pos_lnum;
        pos_col = sit_result.pos_col;

        // Update sia fields if present
        if has_sia != 0 {
            if !sa_timed_out_out.is_null() {
                *sa_timed_out_out = sit_result.sa_timed_out;
            }
            if !sa_wrapped_out.is_null() && sit_result.sa_wrapped != 0 {
                *sa_wrapped_out = 1;
            }
        }

        // Restore delimiter
        if !dircp.is_null() {
            *dircp = search_delim_cur as c_char;
        }

        // Check for wrap-around message
        if nvim_do_search_show_top_bot(dirc, pos_lnum, pos_col) != 0 {
            show_top_bot_msg = true;
        }

        if c == FAIL {
            retval = 0;
            // Cleanup echo
            if !echo_result.msgbuf.is_null() {
                nvim_do_search_echo_free(echo_result.msgbuf);
            }
            break;
        }

        // Set oap->inclusive if needed
        nvim_do_search_set_oap_inclusive(oap);
        retval = 1;

        // Fire SearchWrapped autocmd
        if has_sia != 0 && !sa_wrapped_out.is_null() && *sa_wrapped_out != 0 {
            nvim_do_search_autocmd_wrapped();
        }

        // Add character and/or line offset
        let pat_has_semi = if !pat.is_null() && *pat == b';' as c_char {
            1
        } else {
            0
        };
        let post = nvim_do_search_post_offset(pos_lnum, pos_col, search_options, pat_has_semi);
        pos_lnum = post.lnum;
        pos_col = post.col;
        if post.retval == 2 {
            retval = 2;
        }
        if post.has_offset != 0 {
            has_offset = true;
        }

        // Show search stats
        if echo_result.show_search_stats != 0 {
            nvim_do_search_show_stats(
                dirc,
                pos_lnum,
                pos_col,
                c_int::from(show_top_bot_msg),
                echo_result.msgbuf,
                echo_result.msgbuflen,
                count,
                c_int::from(has_offset),
            );
        }

        // Free echo buffer
        if !echo_result.msgbuf.is_null() {
            nvim_do_search_echo_free(echo_result.msgbuf);
        }

        // Check for chained search (;)
        if (search_options & options::SEARCH_OPT) == 0 || pat.is_null() || *pat != b';' as c_char {
            break;
        }

        pat = pat.add(1);
        dirc = *pat as u8 as c_int;
        search_delim_cur = dirc;
        if dirc != b'?' as c_int && dirc != b'/' as c_int {
            retval = 0;
            nvim_do_search_emsg_e386();
            break;
        }
        pat = pat.add(1);
        patlen -= 1;
    }

    // Finish: set mark and cursor
    if retval != 0 {
        nvim_do_search_finish(search_options, pos_lnum, pos_col);
    }

    // Restore spats offset if SEARCH_KEEP
    if (search_options & options::SEARCH_KEEP) != 0 {
        nvim_do_search_restore_off(saved_off);
    }

    // Free strcopy
    if !strcopy.is_null() {
        xfree(strcopy as *mut c_void);
    }

    retval
}

// =============================================================================
// do_search: C-ABI entry point accepting searchit_arg_T*
// =============================================================================

/// do_search: C-ABI entry point matching the original C function signature.
///
/// Accepts oparg_T* and searchit_arg_T* directly and decomposes them.
///
/// # Safety
/// All pointer arguments must be valid.
#[unsafe(export_name = "do_search")]
pub unsafe extern "C" fn do_search_export(
    oap: OapHandle,
    dirc: c_int,
    search_delim: c_int,
    pat: *mut c_char,
    patlen: usize,
    count: c_int,
    options: c_int,
    sia: *mut crate::searchit::SearchitArgT,
) -> c_int {
    let has_sia = if sia.is_null() { 0 } else { 1 };
    let (sa_stop_lnum, sa_tm, sa_timed_out_init, sa_wrapped_init) = if sia.is_null() {
        (0i32, std::ptr::null_mut(), 0, 0)
    } else {
        let s = &*sia;
        (s.sa_stop_lnum, s.sa_tm, s.sa_timed_out, s.sa_wrapped)
    };

    let mut sa_timed_out = sa_timed_out_init;
    let mut sa_wrapped = sa_wrapped_init;

    let retval = rs_do_search(
        oap,
        dirc,
        search_delim,
        pat,
        patlen,
        count,
        options,
        has_sia,
        sa_stop_lnum,
        sa_tm,
        &mut sa_timed_out,
        &mut sa_wrapped,
    );

    if !sia.is_null() {
        let s = &mut *sia;
        s.sa_timed_out = sa_timed_out;
        s.sa_wrapped = sa_wrapped;
    }

    retval
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_isdigit() {
        assert!(ascii_isdigit(b'0'));
        assert!(ascii_isdigit(b'9'));
        assert!(!ascii_isdigit(b'a'));
        assert!(!ascii_isdigit(b'+'));
    }

    #[test]
    fn test_constants() {
        assert_eq!(FORWARD, 1);
        assert_eq!(BACKWARD, -1);
        assert_eq!(FAIL, 0);
        assert_eq!(RE_LAST, 2);
    }
}
