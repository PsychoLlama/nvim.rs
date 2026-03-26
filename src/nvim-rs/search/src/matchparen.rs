//! Brace/bracket matching: `findmatchlimit()` and helpers.
//!
//! Migrated from search.c. Powers the `%` command, comment matching,
//! `#if`/`#endif` matching, and bracket pair lookups.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cognitive_complexity)]
#![allow(unused_assignments)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Constants (verified with _Static_assert in search.c)
// =============================================================================

const FORWARD: c_int = 1;
const BACKWARD: c_int = -1;
const NUL: c_char = 0;
const MAXCOL: c_int = 0x7fffffff;

// FM flags
const FM_BACKWARD: c_int = 0x01;
const FM_FORWARD: c_int = 0x02;
const FM_BLOCKSTOP: c_int = 0x04;

// CPO chars
const CPO_MATCH: c_int = b'%' as c_int;
const CPO_MATCHBSL: c_int = b'M' as c_int;

// MotionType
const KMT_LINEWISE: c_int = 1;

// =============================================================================
// Types
// =============================================================================

type LinenrT = i32;
type ColnrT = c_int;
type OapHandle = *mut c_void;

/// Result returned across FFI boundary.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FindMatchResult {
    pub found: bool,
    pub lnum: LinenrT,
    pub col: ColnrT,
}

impl FindMatchResult {
    const fn not_found() -> Self {
        Self {
            found: false,
            lnum: 0,
            col: 0,
        }
    }

    const fn found(lnum: LinenrT, col: ColnrT) -> Self {
        Self {
            found: true,
            lnum,
            col,
        }
    }
}

// TriState equivalent
const TRI_NONE: c_int = -1;
const TRI_FALSE: c_int = 0;
const TRI_TRUE: c_int = 1;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    static mut got_int: bool;
    // Line access
    fn nvim_search_ml_get(lnum: LinenrT) -> *const c_char;
    fn nvim_search_ml_get_len(lnum: LinenrT) -> ColnrT;
    fn nvim_search_get_line_count() -> LinenrT;

    // UTF-8
    fn nvim_utf_ptr2char(p: *const c_char) -> c_int;
    fn nvim_utfc_ptr2len(p: *const c_char) -> c_int;
    #[link_name = "utf_head_off"]
    fn nvim_utf_head_off(base: *const c_char, p: *const c_char) -> c_int;

    // Options
    fn nvim_get_p_cpo() -> *const c_char;
    fn nvim_search_get_curbuf_b_p_mps() -> *const c_char;
    fn nvim_search_get_curbuf_b_p_lisp() -> c_int;
    fn nvim_search_get_curwin_w_p_rl() -> c_int;

    // String helpers
    fn skipwhite(s: *const c_char) -> *const c_char;
    fn vim_strchr(s: *const c_char, c: c_int) -> *const c_char;

    // Cursor and buffer
    fn nvim_get_curwin_cursor_lnum() -> LinenrT;
    fn nvim_get_curwin_cursor_col() -> ColnrT;

    // Interrupt check
    fn line_breakcheck();

    // Comment check
    fn nvim_search_check_linecomment(line: *const c_char) -> c_int;

    // Oap setter
    fn nvim_search_set_oap_motion_type(oap: OapHandle, motion_type: c_int);

    // Memory
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);

    // find_rawstring_end needs ml_get for lines
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
}

// =============================================================================
// Helper: check_prevcol
// =============================================================================

/// Check if the character before `col` matches `ch`.
/// Returns true if match found. If `prevcol` is not null, sets it to the
/// column of the previous character.
unsafe fn check_prevcol(linep: *const c_char, col: c_int, ch: u8, prevcol: *mut c_int) -> bool {
    let mut c = col - 1;
    if c > 0 {
        c -= nvim_utf_head_off(linep, linep.add(c as usize));
    }
    if !prevcol.is_null() {
        *prevcol = c;
    }
    c >= 0 && *linep.add(c as usize) as u8 == ch
}

// =============================================================================
// Helper: find_rawstring_end
// =============================================================================

/// Check if a C++ raw string starting at `startpos` ends before `endpos`.
/// Returns true if the closing `)"delim"` is found.
unsafe fn find_rawstring_end(
    linep: *const c_char,
    startpos_lnum: LinenrT,
    startpos_col: ColnrT,
    endpos_lnum: LinenrT,
    endpos_col: ColnrT,
) -> bool {
    // Find the delimiter: R"delim(  — between the quote and the (
    let mut p = linep.add(startpos_col as usize + 1);
    while *p != NUL && *p != b'(' as c_char {
        p = p.add(1);
    }

    let delim_start = linep.add(startpos_col as usize + 1);
    let delim_len = p.offset_from(delim_start) as usize;

    // Allocate a copy of the delimiter
    let delim_copy = xmalloc(delim_len + 1) as *mut c_char;
    std::ptr::copy_nonoverlapping(delim_start, delim_copy, delim_len);
    *delim_copy.add(delim_len) = NUL;

    let mut found = false;
    for lnum in startpos_lnum..=endpos_lnum {
        let line = nvim_search_ml_get(lnum);
        let start_col = if lnum == startpos_lnum {
            startpos_col as usize + 1
        } else {
            0
        };
        let mut q = line.add(start_col);

        while *q != NUL {
            if lnum == endpos_lnum && (q.offset_from(line) as ColnrT) >= endpos_col {
                break;
            }
            if *q == b')' as c_char
                && strncmp(delim_copy, q.add(1), delim_len) == 0
                && *q.add(delim_len + 1) == b'"' as c_char
            {
                found = true;
                break;
            }
            q = q.add(1);
        }
        if found {
            break;
        }
    }

    xfree(delim_copy as *mut c_void);
    found
}

// =============================================================================
// Helper: find_mps_values
// =============================================================================

/// Look up a bracket/brace pair in the `matchpairs` option.
unsafe fn find_mps_values(
    initc: &mut c_int,
    findc: &mut c_int,
    backwards: &mut bool,
    switchit: bool,
) {
    let mut ptr = nvim_search_get_curbuf_b_p_mps();

    while *ptr != NUL {
        if nvim_utf_ptr2char(ptr) == *initc {
            if switchit {
                *findc = *initc;
                *initc = nvim_utf_ptr2char(ptr.add(nvim_utfc_ptr2len(ptr) as usize + 1));
                *backwards = true;
            } else {
                *findc = nvim_utf_ptr2char(ptr.add(nvim_utfc_ptr2len(ptr) as usize + 1));
                *backwards = false;
            }
            return;
        }
        let prev = ptr;
        ptr = ptr.add(nvim_utfc_ptr2len(ptr) as usize + 1);
        if nvim_utf_ptr2char(ptr) == *initc {
            if switchit {
                *findc = *initc;
                *initc = nvim_utf_ptr2char(prev);
                *backwards = false;
            } else {
                *findc = nvim_utf_ptr2char(prev);
                *backwards = true;
            }
            return;
        }
        ptr = ptr.add(nvim_utfc_ptr2len(ptr) as usize);
        if *ptr == b',' as c_char {
            ptr = ptr.add(1);
        }
    }
}

// =============================================================================
// Main: findmatchlimit
// =============================================================================

/// Rust implementation of findmatchlimit().
///
/// # Safety
/// All pointer arguments must be valid. Called from C wrapper.
#[no_mangle]
pub unsafe extern "C" fn rs_findmatchlimit(
    oap: OapHandle,
    initc_in: c_int,
    flags: c_int,
    maxtravel: i64,
) -> FindMatchResult {
    let mut pos_lnum = nvim_get_curwin_cursor_lnum();
    let mut pos_col = nvim_get_curwin_cursor_col();

    let mut linep = nvim_search_ml_get(pos_lnum);

    let mut initc = initc_in;
    let mut findc: c_int = 0;
    let mut count: c_int = 0;
    let mut backwards = false;
    let mut raw_string = false;
    let mut inquote = false;
    let mut hash_dir: c_int = 0;
    let mut comment_dir: c_int = 0;
    let mut traveled: i64 = 0;
    let mut ignore_cend = false;
    let mut match_escaped: c_int = 0;
    let mut comment_col: c_int = MAXCOL;
    let mut lispcomm = false;
    let lisp = nvim_search_get_curbuf_b_p_lisp() != 0;

    let p_cpo = nvim_get_p_cpo();
    let cpo_match = !vim_strchr(p_cpo, CPO_MATCH).is_null();
    let cpo_bsl = !vim_strchr(p_cpo, CPO_MATCHBSL).is_null();

    // Direction from flags
    let dir = if (flags & FM_BACKWARD) != 0 {
        BACKWARD
    } else if (flags & FM_FORWARD) != 0 {
        FORWARD
    } else {
        0
    };

    // Handle '/', '*', 'R' for comment matching, or '#' for preprocessor
    if initc == b'/' as c_int || initc == b'*' as c_int || initc == b'R' as c_int {
        comment_dir = dir;
        if initc == b'/' as c_int {
            ignore_cend = true;
        }
        backwards = dir != FORWARD;
        raw_string = initc == b'R' as c_int;
        initc = 0;
    } else if initc != b'#' as c_int && initc != 0 {
        find_mps_values(&mut initc, &mut findc, &mut backwards, true);
        if dir != 0 {
            backwards = dir != FORWARD;
        }
        if findc == 0 {
            return FindMatchResult::not_found();
        }
    } else {
        // '#' or no initc — look under cursor
        if initc == b'#' as c_int {
            hash_dir = dir;
        } else {
            if !cpo_match {
                let ptr = skipwhite(linep);
                if *ptr == b'#' as c_char && pos_col <= (ptr.offset_from(linep) as ColnrT) {
                    let ptr2 = skipwhite(ptr.add(1));
                    if strncmp(ptr2, c"if".as_ptr(), 2) == 0
                        || strncmp(ptr2, c"endif".as_ptr(), 5) == 0
                        || strncmp(ptr2, c"el".as_ptr(), 2) == 0
                    {
                        hash_dir = 1;
                    }
                } else if *linep.add(pos_col as usize) == b'/' as c_char {
                    if *linep.add(pos_col as usize + 1) == b'*' as c_char {
                        comment_dir = FORWARD;
                        backwards = false;
                        pos_col += 1;
                    } else if pos_col > 0 && *linep.add(pos_col as usize - 1) == b'*' as c_char {
                        comment_dir = BACKWARD;
                        backwards = true;
                        pos_col -= 1;
                    }
                } else if *linep.add(pos_col as usize) == b'*' as c_char {
                    if *linep.add(pos_col as usize + 1) == b'/' as c_char {
                        comment_dir = BACKWARD;
                        backwards = true;
                    } else if pos_col > 0 && *linep.add(pos_col as usize - 1) == b'/' as c_char {
                        comment_dir = FORWARD;
                        backwards = false;
                    }
                }
            }

            // If no comment or hash, look for brace on this line
            if hash_dir == 0 && comment_dir == 0 {
                if *linep.add(pos_col as usize) == NUL && pos_col > 0 {
                    pos_col -= 1;
                }
                loop {
                    initc = nvim_utf_ptr2char(linep.add(pos_col as usize));
                    if initc == 0 {
                        break;
                    }
                    find_mps_values(&mut initc, &mut findc, &mut backwards, false);
                    if findc != 0 {
                        break;
                    }
                    pos_col += nvim_utfc_ptr2len(linep.add(pos_col as usize));
                }
                if findc == 0 {
                    if !cpo_match && *skipwhite(linep) == b'#' as c_char {
                        hash_dir = 1;
                    } else {
                        return FindMatchResult::not_found();
                    }
                } else if !cpo_bsl {
                    let mut bslcnt = 0;
                    let mut col = pos_col;
                    while check_prevcol(linep, col, b'\\', &mut col) {
                        bslcnt += 1;
                    }
                    match_escaped = bslcnt & 1;
                }
            }
        }

        if hash_dir != 0 {
            // Handle #if/#else/#endif matching
            if !oap.is_null() {
                nvim_search_set_oap_motion_type(oap, KMT_LINEWISE);
            }
            if initc != b'#' as c_int {
                let ptr = skipwhite(skipwhite(linep).add(1));
                if strncmp(ptr, c"if".as_ptr(), 2) == 0 || strncmp(ptr, c"el".as_ptr(), 2) == 0 {
                    hash_dir = 1;
                } else if strncmp(ptr, c"endif".as_ptr(), 5) == 0 {
                    hash_dir = -1;
                } else {
                    return FindMatchResult::not_found();
                }
            }
            pos_col = 0;
            let line_count = nvim_search_get_line_count();
            while !unsafe { got_int } {
                if hash_dir > 0 {
                    if pos_lnum == line_count {
                        break;
                    }
                } else if pos_lnum == 1 {
                    break;
                }
                pos_lnum += hash_dir;
                linep = nvim_search_ml_get(pos_lnum);
                line_breakcheck();
                let ptr = skipwhite(linep);
                if *ptr != b'#' as c_char {
                    continue;
                }
                pos_col = ptr.offset_from(linep) as ColnrT;
                let ptr2 = skipwhite(ptr.add(1));
                if hash_dir > 0 {
                    if strncmp(ptr2, c"if".as_ptr(), 2) == 0 {
                        count += 1;
                    } else if strncmp(ptr2, c"el".as_ptr(), 2) == 0 {
                        if count == 0 {
                            return FindMatchResult::found(pos_lnum, pos_col);
                        }
                    } else if strncmp(ptr2, c"endif".as_ptr(), 5) == 0 {
                        if count == 0 {
                            return FindMatchResult::found(pos_lnum, pos_col);
                        }
                        count -= 1;
                    }
                } else if strncmp(ptr2, c"if".as_ptr(), 2) == 0 {
                    if count == 0 {
                        return FindMatchResult::found(pos_lnum, pos_col);
                    }
                    count -= 1;
                } else if initc_in == b'#' as c_int && strncmp(ptr2, c"el".as_ptr(), 2) == 0 {
                    if count == 0 {
                        return FindMatchResult::found(pos_lnum, pos_col);
                    }
                } else if strncmp(ptr2, c"endif".as_ptr(), 5) == 0 {
                    count += 1;
                }
            }
            return FindMatchResult::not_found();
        }
    }

    // Rightleft: reverse direction for bracket chars
    if nvim_search_get_curwin_w_p_rl() != 0 {
        let bracket_chars = c"()[]{}<>";
        if !vim_strchr(bracket_chars.as_ptr(), initc).is_null() {
            backwards = !backwards;
        }
    }

    let mut do_quotes: c_int = -1;
    let mut at_start: c_int = 0;
    let mut start_in_quotes: c_int = TRI_NONE;
    let mut match_pos_lnum: LinenrT = 0;
    let mut match_pos_col: ColnrT = 0;

    // Backward search: check for single-line comment
    if (backwards && comment_dir != 0) || lisp {
        comment_col = nvim_search_check_linecomment(linep);
    }
    if lisp && comment_col != MAXCOL && pos_col > comment_col {
        lispcomm = true;
    }

    // Main search loop
    while !unsafe { got_int } {
        if backwards {
            if lispcomm && pos_col < comment_col {
                break;
            }
            if pos_col == 0 {
                if pos_lnum == 1 {
                    break;
                }
                pos_lnum -= 1;

                if maxtravel > 0 {
                    traveled += 1;
                    if traveled > maxtravel {
                        break;
                    }
                }

                linep = nvim_search_ml_get(pos_lnum);
                pos_col = nvim_search_ml_get_len(pos_lnum);
                do_quotes = -1;
                line_breakcheck();

                if comment_dir != 0 || lisp {
                    comment_col = nvim_search_check_linecomment(linep);
                }
                if lisp && comment_col != MAXCOL {
                    pos_col = comment_col;
                }
            } else {
                pos_col -= 1;
                pos_col -= nvim_utf_head_off(linep, linep.add(pos_col as usize));
            }
        } else {
            // Forward search
            if *linep.add(pos_col as usize) == NUL
                || (lisp && comment_col != MAXCOL && pos_col == comment_col)
            {
                if pos_lnum == nvim_search_get_line_count() || lispcomm {
                    break;
                }
                pos_lnum += 1;

                if maxtravel != 0 {
                    traveled += 1;
                    if traveled > maxtravel {
                        break;
                    }
                }

                linep = nvim_search_ml_get(pos_lnum);
                pos_col = 0;
                do_quotes = -1;
                line_breakcheck();
                if lisp {
                    comment_col = nvim_search_check_linecomment(linep);
                }
            } else {
                pos_col += nvim_utfc_ptr2len(linep.add(pos_col as usize));
            }
        }

        // FM_BLOCKSTOP: stop at '{' or '}' in column 0
        if pos_col == 0
            && (flags & FM_BLOCKSTOP) != 0
            && (*linep.add(0) == b'{' as c_char || *linep.add(0) == b'}' as c_char)
        {
            if *linep.add(0) == findc as u8 as c_char && count == 0 {
                return FindMatchResult::found(pos_lnum, pos_col);
            }
            break;
        }

        // Comment matching
        if comment_dir != 0 {
            if comment_dir == FORWARD {
                if *linep.add(pos_col as usize) == b'*' as c_char
                    && *linep.add(pos_col as usize + 1) == b'/' as c_char
                {
                    return FindMatchResult::found(pos_lnum, pos_col + 1);
                }
            } else {
                // Backward comment search
                if pos_col == 0 {
                    continue;
                } else if raw_string {
                    if *linep.add(pos_col as usize - 1) == b'R' as c_char
                        && *linep.add(pos_col as usize) == b'"' as c_char
                        && !vim_strchr(linep.add(pos_col as usize + 1), b'(' as c_int).is_null()
                    {
                        let end_lnum = if count > 0 {
                            match_pos_lnum
                        } else {
                            nvim_get_curwin_cursor_lnum()
                        };
                        let end_col = if count > 0 {
                            match_pos_col
                        } else {
                            nvim_get_curwin_cursor_col()
                        };
                        if !find_rawstring_end(linep, pos_lnum, pos_col, end_lnum, end_col) {
                            count += 1;
                            match_pos_lnum = pos_lnum;
                            match_pos_col = pos_col - 1;
                        }
                        linep = nvim_search_ml_get(pos_lnum);
                    }
                } else if *linep.add(pos_col as usize - 1) == b'/' as c_char
                    && *linep.add(pos_col as usize) == b'*' as c_char
                    && (pos_col == 1 || *linep.add(pos_col as usize - 2) != b'*' as c_char)
                    && (pos_col as c_int) < comment_col
                {
                    count += 1;
                    match_pos_lnum = pos_lnum;
                    match_pos_col = pos_col - 1;
                } else if *linep.add(pos_col as usize - 1) == b'*' as c_char
                    && *linep.add(pos_col as usize) == b'/' as c_char
                {
                    if count > 0 {
                        pos_lnum = match_pos_lnum;
                        pos_col = match_pos_col;
                    } else if pos_col > 1
                        && *linep.add(pos_col as usize - 2) == b'/' as c_char
                        && pos_col as c_int <= comment_col
                    {
                        pos_col -= 2;
                    } else if ignore_cend {
                        continue;
                    } else {
                        return FindMatchResult::not_found();
                    }
                    return FindMatchResult::found(pos_lnum, pos_col);
                }
            }
            continue;
        }

        // Smart matching: handle quotes
        if cpo_match {
            do_quotes = 0;
        } else if do_quotes == -1 {
            at_start = do_quotes;
            let mut ptr = linep;
            while *ptr != NUL {
                if ptr == linep.add(pos_col as usize + if backwards { 1 } else { 0 }) {
                    at_start = do_quotes & 1;
                }
                if *ptr == b'"' as c_char
                    && (ptr == linep
                        || *ptr.sub(1) != b'\'' as c_char
                        || *ptr.add(1) != b'\'' as c_char)
                {
                    do_quotes += 1;
                }
                if *ptr == b'\\' as c_char && *ptr.add(1) != NUL {
                    ptr = ptr.add(1);
                }
                ptr = ptr.add(1);
            }
            do_quotes &= 1; // 1 with even number of quotes

            if do_quotes == 0 {
                inquote = false;
                if *ptr.sub(1) == b'\\' as c_char {
                    do_quotes = 1;
                    if start_in_quotes == TRI_NONE {
                        inquote = true;
                        start_in_quotes = TRI_TRUE;
                    } else if backwards {
                        inquote = true;
                    }
                }
                if pos_lnum > 1 {
                    let prev_line = nvim_search_ml_get(pos_lnum - 1);
                    let prev_len = nvim_search_ml_get_len(pos_lnum - 1);
                    if prev_len > 0 && *prev_line.add(prev_len as usize - 1) == b'\\' as c_char {
                        do_quotes = 1;
                        if start_in_quotes == TRI_NONE {
                            inquote = at_start != 0;
                            if inquote {
                                start_in_quotes = TRI_TRUE;
                            }
                        } else if !backwards {
                            inquote = true;
                        }
                    }
                    // ml_get() only keeps one line, refresh
                    linep = nvim_search_ml_get(pos_lnum);
                }
            }
        }
        if start_in_quotes == TRI_NONE {
            start_in_quotes = TRI_FALSE;
        }

        // Character switch
        let c = nvim_utf_ptr2char(linep.add(pos_col as usize));
        if c == 0 {
            // NUL: reset inquote at end of line
            if pos_col == 0 || *linep.add(pos_col as usize - 1) != b'\\' as c_char {
                inquote = false;
                start_in_quotes = TRI_FALSE;
            }
        } else if c == b'"' as c_int {
            // Quote handling
            if do_quotes != 0 {
                let mut col = pos_col - 1;
                while col >= 0 && *linep.add(col as usize) == b'\\' as c_char {
                    col -= 1;
                }
                if ((pos_col - 1 - col) & 1) == 0 {
                    inquote = !inquote;
                    start_in_quotes = TRI_FALSE;
                }
            }
        } else if c == b'\'' as c_int {
            // Single quote handling (skip 'x' and '\x')
            if !cpo_match && initc != b'\'' as c_int && findc != b'\'' as c_int {
                if backwards {
                    if pos_col > 1 {
                        if *linep.add(pos_col as usize - 2) == b'\'' as c_char {
                            pos_col -= 2;
                            continue;
                        } else if *linep.add(pos_col as usize - 2) == b'\\' as c_char
                            && pos_col > 2
                            && *linep.add(pos_col as usize - 3) == b'\'' as c_char
                        {
                            pos_col -= 3;
                            continue;
                        }
                    }
                } else if *linep.add(pos_col as usize + 1) != NUL {
                    if *linep.add(pos_col as usize + 1) == b'\\' as c_char
                        && *linep.add(pos_col as usize + 2) != NUL
                        && *linep.add(pos_col as usize + 3) == b'\'' as c_char
                    {
                        pos_col += 3;
                        continue;
                    } else if *linep.add(pos_col as usize + 2) == b'\'' as c_char {
                        pos_col += 2;
                        continue;
                    }
                }
            }
            // FALLTHROUGH to default
            if !handle_default_match(
                c,
                linep,
                pos_col,
                initc,
                findc,
                &mut count,
                cpo_bsl,
                match_escaped,
                inquote,
                start_in_quotes,
                lisp,
            ) {
                return FindMatchResult::not_found(); // not used, see handle_default_match
            }
            if count < 0 {
                // Signal from handle_default_match that match was found
                return FindMatchResult::found(pos_lnum, pos_col);
            }
        } else {
            // Default case (all other characters)
            // Lisp: skip #\( etc
            if lisp
                && !vim_strchr(c"(){}[]".as_ptr(), c).is_null()
                && pos_col > 1
                && check_prevcol(linep, pos_col, b'\\', std::ptr::null_mut())
                && check_prevcol(linep, pos_col - 1, b'#', std::ptr::null_mut())
            {
                // skip
            } else if (!inquote || start_in_quotes == TRI_TRUE) && (c == initc || c == findc) {
                let mut bslcnt = 0;
                if !cpo_bsl {
                    let mut col = pos_col;
                    while check_prevcol(linep, col, b'\\', &mut col) {
                        bslcnt += 1;
                    }
                }
                if cpo_bsl || (bslcnt & 1) == match_escaped {
                    if c == initc {
                        count += 1;
                    } else {
                        if count == 0 {
                            return FindMatchResult::found(pos_lnum, pos_col);
                        }
                        count -= 1;
                    }
                }
            }
        }
    }

    // End of loop
    if comment_dir == BACKWARD && count > 0 {
        return FindMatchResult::found(match_pos_lnum, match_pos_col);
    }
    FindMatchResult::not_found()
}

/// Handle the default match case for brace/bracket matching.
/// Returns true always. Sets `count` to -1 if a match was found at pos_col.
#[inline]
unsafe fn handle_default_match(
    c: c_int,
    linep: *const c_char,
    pos_col: ColnrT,
    initc: c_int,
    findc: c_int,
    count: &mut c_int,
    cpo_bsl: bool,
    match_escaped: c_int,
    inquote: bool,
    start_in_quotes: c_int,
    lisp: bool,
) -> bool {
    // Lisp: skip #\( etc
    if lisp
        && !vim_strchr(c"(){}[]".as_ptr(), c).is_null()
        && pos_col > 1
        && check_prevcol(linep, pos_col, b'\\', std::ptr::null_mut())
        && check_prevcol(linep, pos_col - 1, b'#', std::ptr::null_mut())
    {
        return true;
    }

    if (!inquote || start_in_quotes == TRI_TRUE) && (c == initc || c == findc) {
        let mut bslcnt = 0;
        if !cpo_bsl {
            let mut col = pos_col;
            while check_prevcol(linep, col, b'\\', &mut col) {
                bslcnt += 1;
            }
        }
        if cpo_bsl || (bslcnt & 1) == match_escaped {
            if c == initc {
                *count += 1;
            } else {
                if *count == 0 {
                    *count = -1; // signal match found
                    return true;
                }
                *count -= 1;
            }
        }
    }
    true
}

// =============================================================================
// Phase 8: showmatch — match-finding part
// =============================================================================

extern "C" {
    fn nvim_showmatch_get_p_ri() -> c_int;
    fn nvim_showmatch_find_and_check(
        out_lnum: *mut c_int,
        out_col: *mut c_int,
        out_coladd: *mut c_int,
    ) -> c_int;
    fn nvim_showmatch_beep();
    /// Perform the cursor-display-delay loop for showmatch.
    fn nvim_showmatch_display_cursor(match_lnum: c_int, match_col: c_int, match_coladd: c_int);
}

/// Scan matchpairs option, find match, and check visibility.
///
/// Returns 1 if a visible match was found (out_lnum/out_col/out_coladd are set),
/// 0 if no visible match (caller should just return from showmatch).
///
/// # Safety
/// All output pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_showmatch_find_match(
    c: c_int,
    out_lnum: *mut c_int,
    out_col: *mut c_int,
    out_coladd: *mut c_int,
) -> c_int {
    // Scan matchpairs option to see if c is one of the characters.
    // 'matchpairs' is "x:y,x:y"
    let mps = nvim_search_get_curbuf_b_p_mps();
    if mps.is_null() {
        return 0;
    }

    let w_p_rl = nvim_search_get_curwin_w_p_rl() != 0;
    let p_ri_val = nvim_showmatch_get_p_ri() != 0;

    let mut p = mps;
    let mut found_in_mps = false;
    while *p != 0 {
        let ch1 = nvim_utf_ptr2char(p);
        if ch1 == c && (w_p_rl ^ p_ri_val) {
            found_in_mps = true;
            break;
        }
        p = p.add(nvim_utfc_ptr2len(p) as usize + 1); // skip char + ':'
        let ch2 = nvim_utf_ptr2char(p);
        if ch2 == c && !(w_p_rl ^ p_ri_val) {
            found_in_mps = true;
            break;
        }
        p = p.add(nvim_utfc_ptr2len(p) as usize);
        if *p == 0 {
            return 0; // end of matchpairs, c not found
        }
        if *p == b',' as c_char {
            p = p.add(1);
        }
    }

    if !found_in_mps {
        return 0;
    }

    // Call batch C helper for findmatch + visibility check
    let result = nvim_showmatch_find_and_check(out_lnum, out_col, out_coladd);
    if result == -1 {
        // No match found, beep
        nvim_showmatch_beep();
        return 0;
    }
    if result == 0 {
        // Match not visible
        return 0;
    }

    1 // visible match found
}

/// Full showmatch implementation: find match position, then show it.
///
/// Replaces the C `showmatch()` function.  The match-finding part is pure
/// Rust; the cursor-move/display/delay loop is delegated to a thin C batch
/// helper (`nvim_showmatch_display_cursor`) that manipulates curwin, State,
/// dollar_vcol, and calls ui_cursor_shape / update_screen / os_delay.
///
/// # Safety
/// Must be called from the Neovim main thread.
#[unsafe(export_name = "showmatch")]
pub unsafe extern "C" fn rs_showmatch(c: c_int) {
    let mut match_lnum: c_int = 0;
    let mut match_col: c_int = 0;
    let mut match_coladd: c_int = 0;
    if rs_showmatch_find_match(c, &mut match_lnum, &mut match_col, &mut match_coladd) == 0 {
        return;
    }
    nvim_showmatch_display_cursor(match_lnum, match_col, match_coladd);
}

// =============================================================================
// findmatchlimit / findmatch: C-ABI entry points matching original signatures
// =============================================================================

/// Static position storage for findmatchlimit return value.
/// Matches the C pattern of returning &static_pos.
static mut FINDMATCH_POS: crate::searchit::PosT = crate::searchit::PosT {
    lnum: 0,
    col: 0,
    coladd: 0,
};

/// findmatchlimit: C-ABI entry point.
///
/// Returns pointer to a static PosT (matching C's static pos_T pattern),
/// or NULL if not found.
///
/// # Safety
/// oap must be a valid oparg_T pointer or NULL.
#[unsafe(export_name = "findmatchlimit")]
pub unsafe extern "C" fn findmatchlimit_export(
    oap: OapHandle,
    initc: c_int,
    flags: c_int,
    maxtravel: i64,
) -> *mut crate::searchit::PosT {
    let result = rs_findmatchlimit(oap, initc, flags, maxtravel);
    if !result.found {
        return std::ptr::null_mut();
    }
    FINDMATCH_POS.lnum = result.lnum;
    FINDMATCH_POS.col = result.col;
    FINDMATCH_POS.coladd = 0;
    std::ptr::addr_of_mut!(FINDMATCH_POS)
}

/// findmatch: C-ABI entry point. Calls findmatchlimit(oap, initc, 0, 0).
///
/// # Safety
/// oap must be a valid oparg_T pointer or NULL.
#[unsafe(export_name = "findmatch")]
pub unsafe extern "C" fn findmatch_export(
    oap: OapHandle,
    initc: c_int,
) -> *mut crate::searchit::PosT {
    findmatchlimit_export(oap, initc, 0, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(FORWARD, 1);
        assert_eq!(BACKWARD, -1);
        assert_eq!(MAXCOL, 0x7fffffff);
        assert_eq!(FM_BACKWARD, 0x01);
        assert_eq!(FM_FORWARD, 0x02);
        assert_eq!(FM_BLOCKSTOP, 0x04);
        assert_eq!(CPO_MATCH, b'%' as c_int);
        assert_eq!(CPO_MATCHBSL, b'M' as c_int);
        assert_eq!(KMT_LINEWISE, 1);
    }

    #[test]
    fn test_find_match_result() {
        let nf = FindMatchResult::not_found();
        assert!(!nf.found);

        let f = FindMatchResult::found(42, 7);
        assert!(f.found);
        assert_eq!(f.lnum, 42);
        assert_eq!(f.col, 7);
    }

    #[test]
    fn test_tristate() {
        assert_eq!(TRI_NONE, -1);
        assert_eq!(TRI_FALSE, 0);
        assert_eq!(TRI_TRUE, 1);
    }
}
