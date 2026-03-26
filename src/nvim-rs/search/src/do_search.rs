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

use crate::direction;
use crate::helpers::options;
use crate::pattern;
use crate::searchit::{self, SearchitResult};
use crate::state;

// =============================================================================
// Constants (verified with _Static_assert in search.c)
// =============================================================================

const FORWARD: c_int = 1;
const BACKWARD: c_int = -1;
const FAIL: c_int = 0;
const RE_LAST: c_int = 2;
const NUL: c_char = 0;
const MAXCOL: ColnrT = 0x7fff_ffff;
const SHM_SEARCHCOUNT: c_int = b'S' as c_int;
const K_UI_MESSAGES: c_int = 4;
const SEARCH_STAT_BUF_LEN: usize = 16;
/// EVENT_SEARCHWRAPPED = 95 (from auevents_enum.generated.h)
const EVENT_SEARCHWRAPPED: c_int = 95;

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

// Saved offset state (now Rust-native, no longer crossing FFI boundary)
#[derive(Clone, Copy)]
struct SavedSearchOff {
    dir: c_char,
    line: bool,
    end: bool,
    off: c_longlong,
}

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    // do_search-specific helpers still in C
    fn nvim_do_search_check_lineoff() -> c_int;
    fn nvim_do_search_hlsearch_on(options: c_int);
    fn nvim_do_search_skip_regexp(
        pat: *mut c_char,
        delim: c_int,
        newp: *mut *mut c_char,
    ) -> *mut c_char;
    fn nvim_do_search_set_searchcmdlen(val: c_int);
    fn nvim_do_search_get_searchcmdlen() -> c_int;
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

    // fold helpers
    fn nvim_do_search_hasFolding_fwd(lnum: *mut LinenrT) -> c_int;
    fn nvim_do_search_hasFolding_bwd(lnum: *mut LinenrT) -> c_int;

    // cursor read/write
    fn nvim_search_get_curwin_cursor_lnum() -> LinenrT;
    fn nvim_search_get_curwin_cursor_col() -> ColnrT;
    fn nvim_set_curwin_cursor_lnum(lnum: LinenrT);
    fn nvim_set_curwin_cursor_col(col: ColnrT);
    fn nvim_set_curwin_cursor_coladd(coladd: ColnrT);
    fn nvim_curwin_set_curswant(val: bool);

    // operator-pending accessor
    fn nvim_oap_set_inclusive(oap: OapHandle, val: bool);

    // incl/decl position helpers (pos_T by components); return -1 at buffer boundary
    fn nvim_search_incl_pos(lnum: *mut c_int, col: *mut c_int, coladd: *mut c_int) -> c_int;
    fn nvim_search_decl_pos(lnum: *mut c_int, col: *mut c_int, coladd: *mut c_int) -> c_int;

    // message helpers
    fn nvim_messaging() -> bool;
    fn msg_start();
    fn msg_ext_set_kind(kind: *const c_char);
    fn msg_outtrans(s: *const c_char, attr: c_int, right: bool);
    fn msg_clr_eos();
    fn msg_check();
    fn msg_strtrunc(s: *const c_char, force: c_int) -> *mut c_char;
    fn gotocmdline(clr: bool);
    fn ui_flush();
    fn ui_busy_start();
    fn ui_busy_stop();
    fn ui_has(ext: c_int) -> bool;
    fn nvim_curwin_rl_with_rlc_s() -> c_int;
    fn nvim_utf_iscomposing_first(c: c_int) -> c_int;
    fn utf_ptr2char(p: *const c_char) -> c_int;
    fn reverse_text(s: *const c_char) -> *mut c_char;

    // shortmess
    fn shortmess(x: c_int) -> bool;

    // error messages
    fn emsg(s: *const c_char) -> c_int;
    fn nvim_emsg_noprevre();

    // autocmds
    fn apply_autocmds(
        event: c_int,
        fname: *const c_char,
        fname_io: *const c_char,
        force: bool,
        buf: *mut c_void,
    ) -> bool;

    // mark
    fn setpcmark();

    // globals
    static mut msg_nowait: bool;
    static msg_silent: c_int;
    static mut msg_scrolled: c_int;
    static Rows: c_int;
    static msg_row: c_int;
    static Columns: c_int;
    static sc_col: c_int;
    static cmd_silent: bool;

    // Existing accessors
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_get_curbuf() -> BufHandle;

    // String helpers
    fn strlen(s: *const c_char) -> usize;
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn memmove(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
}

// =============================================================================
// ASCII helpers
// =============================================================================

fn ascii_isdigit(c: u8) -> bool {
    c.is_ascii_digit()
}

// =============================================================================
// Migrated do_search helper functions (formerly in search_shim.c)
// =============================================================================

/// Fold adjustment for do_search position.
/// Forward: adjusts lnum to end of fold. Backward: adjusts to start.
unsafe fn do_search_fold_adjust(dirc: c_int, lnum: LinenrT, col: ColnrT) -> DoSearchPos {
    let mut pos_lnum = lnum;
    let mut pos_col = col;
    if dirc == b'/' as c_int {
        if nvim_do_search_hasFolding_fwd(&mut pos_lnum) != 0 {
            pos_col = MAXCOL - 2;
        }
    } else if nvim_do_search_hasFolding_bwd(&mut pos_lnum) != 0 {
        pos_col = 0;
    }
    DoSearchPos {
        lnum: pos_lnum,
        col: pos_col,
    }
}

/// Get cursor position for do_search start.
unsafe fn do_search_get_cursor() -> DoSearchPos {
    DoSearchPos {
        lnum: nvim_search_get_curwin_cursor_lnum(),
        col: nvim_search_get_curwin_cursor_col(),
    }
}

/// Set oap->inclusive if search has end offset.
unsafe fn do_search_set_oap_inclusive(oap: OapHandle) {
    if !oap.is_null() && state::get_spat_off_end(state::RE_SEARCH) {
        nvim_oap_set_inclusive(oap, true);
    }
}

/// Fire EVENT_SEARCHWRAPPED autocmd.
unsafe fn do_search_autocmd_wrapped() {
    apply_autocmds(
        EVENT_SEARCHWRAPPED,
        std::ptr::null(),
        std::ptr::null(),
        false,
        std::ptr::null_mut(),
    );
}

/// Emit E386 error.
unsafe fn do_search_emsg_e386() {
    emsg(c"E386: Expected '?' or '/'  after ';'".as_ptr());
}

/// Emit e_noprevre error.
unsafe fn do_search_emsg_noprevre() {
    nvim_emsg_noprevre();
}

/// Check if search wrapped (show top/bot msg).
unsafe fn do_search_show_top_bot(dirc: c_int, pos_lnum: LinenrT, pos_col: ColnrT) -> c_int {
    if shortmess(b's' as c_int) {
        return 0;
    }
    let cur_lnum = nvim_search_get_curwin_cursor_lnum();
    let cur_col = nvim_search_get_curwin_cursor_col();
    // lt(pos, cursor): pos < cursor
    let pos_lt_cursor = pos_lnum < cur_lnum || (pos_lnum == cur_lnum && pos_col < cur_col);
    // lt(cursor, pos): cursor < pos
    let cursor_lt_pos = cur_lnum < pos_lnum || (cur_lnum == pos_lnum && cur_col < pos_col);
    if (dirc == b'/' as c_int && pos_lt_cursor) || (dirc == b'?' as c_int && cursor_lt_pos) {
        1
    } else {
        0
    }
}

/// Set pcmark and cursor for search result.
unsafe fn do_search_finish(search_options: c_int, lnum: LinenrT, col: ColnrT) {
    if (search_options & options::SEARCH_MARK) != 0 {
        setpcmark();
    }
    nvim_set_curwin_cursor_lnum(lnum);
    nvim_set_curwin_cursor_col(col);
    nvim_set_curwin_cursor_coladd(0);
    nvim_curwin_set_curswant(true);
}

/// Pre-searchit character offset subtraction.
/// Handles the "?pat?e+2" / "/pat/s-2" case.
unsafe fn do_search_pre_offset(lnum: LinenrT, col: ColnrT) -> DoSearchPos {
    let mut pos_lnum = lnum;
    let mut pos_col = col;
    let mut pos_coladd: c_int = 0;
    let off = state::get_spat_off_off(state::RE_SEARCH);

    if !state::get_spat_off_line(state::RE_SEARCH) && off != 0 && pos_col < MAXCOL - 2 {
        if off > 0 {
            let mut c = off;
            while c > 0 {
                if nvim_search_decl_pos(&mut pos_lnum, &mut pos_col, &mut pos_coladd) == -1 {
                    break;
                }
                c -= 1;
            }
            if c > 0 {
                pos_lnum = 0;
                pos_col = MAXCOL;
            }
        } else {
            let mut c = off;
            while c < 0 {
                if nvim_search_incl_pos(&mut pos_lnum, &mut pos_col, &mut pos_coladd) == -1 {
                    break;
                }
                c += 1;
            }
            if c < 0 {
                // curbuf->b_ml.ml_line_count + 1 (approximate large value)
                pos_lnum = 0x7fff_ffff; // sentinel: past end of buffer
                pos_col = 0;
            }
        }
    }

    DoSearchPos {
        lnum: pos_lnum,
        col: pos_col,
    }
}

/// Post-searchit line/char offset addition.
unsafe fn do_search_post_offset(
    lnum: LinenrT,
    col: ColnrT,
    search_options: c_int,
    pat_has_semicolon: c_int,
) -> DoSearchPostOffset {
    let mut result = DoSearchPostOffset {
        lnum,
        col,
        retval: 1,
        has_offset: 0,
    };
    let mut pos_lnum = lnum;
    let mut pos_col = col;
    let mut pos_coladd: c_int = 0;
    let org_lnum = lnum;
    let org_col = col;

    let off_line = state::get_spat_off_line(state::RE_SEARCH);
    let off_off = state::get_spat_off_off(state::RE_SEARCH);

    if (search_options & options::SEARCH_NOOF) == 0 || pat_has_semicolon != 0 {
        if off_line {
            let c = pos_lnum as i64 + off_off;
            if c < 1 {
                pos_lnum = 1;
            } else if c > 0x7fff_ffff {
                // past end: clamp
                pos_lnum = 0x7fff_ffff;
            } else {
                pos_lnum = c as LinenrT;
            }
            pos_col = 0;
            result.retval = 2;
        } else if pos_col < MAXCOL - 2 {
            let mut c = off_off;
            if c > 0 {
                while c > 0 {
                    if nvim_search_incl_pos(&mut pos_lnum, &mut pos_col, &mut pos_coladd) == -1 {
                        break;
                    }
                    c -= 1;
                }
            } else {
                while c < 0 {
                    if nvim_search_decl_pos(&mut pos_lnum, &mut pos_col, &mut pos_coladd) == -1 {
                        break;
                    }
                    c += 1;
                }
            }
        }
        // Check if position changed (like C's equalpos)
        if pos_lnum != org_lnum || pos_col != org_col {
            result.has_offset = 1;
        }
    }

    result.lnum = pos_lnum;
    result.col = pos_col;
    result
}

/// Batch helper: handle echo/display section of do_search.
/// Returns: msgbuf (allocated), sets show_search_stats.
/// Caller must xfree() the returned buffer.
#[allow(clippy::cast_possible_truncation)]
unsafe fn do_search_echo(
    dirc: c_int,
    search_options: c_int,
    searchstr: *const c_char,
    searchstrlen: usize,
) -> DoSearchEchoResult {
    let result_null = DoSearchEchoResult {
        msgbuf: std::ptr::null_mut(),
        msgbuflen: 0,
        show_search_stats: 0,
    };

    if !((search_options & options::SEARCH_ECHO) != 0
        && nvim_messaging()
        && msg_silent == 0
        && (!cmd_silent || !shortmess(SHM_SEARCHCOUNT)))
    {
        return result_null;
    }

    let mut off_buf = [0u8; 40];
    let mut off_len: usize = 0;

    msg_start();
    msg_ext_set_kind(c"search_cmd".as_ptr());

    // Read search offset fields from Rust state
    let off_end = state::get_spat_off_end(state::RE_SEARCH);
    let off_line = state::get_spat_off_line(state::RE_SEARCH);
    let off_off = state::get_spat_off_off(state::RE_SEARCH);

    if !cmd_silent && (off_line || off_end || off_off != 0) {
        off_buf[off_len] = dirc as u8;
        off_len += 1;
        if off_end {
            off_buf[off_len] = b'e';
            off_len += 1;
        } else if !off_line {
            off_buf[off_len] = b's';
            off_len += 1;
        }
        off_buf[off_len] = 0; // NUL terminate
        if off_off != 0 || off_line {
            // Format the offset value
            let formatted = format!("{:+}", off_off);
            let bytes = formatted.as_bytes();
            let copy_len = bytes.len().min(off_buf.len() - off_len - 1);
            off_buf[off_len..off_len + copy_len].copy_from_slice(&bytes[..copy_len]);
            off_len += copy_len;
        }
    }

    // Get the pattern to display
    let p: *const c_char;
    let plen: usize;
    if !searchstr.is_null() && *searchstr != NUL {
        p = searchstr;
        plen = searchstrlen;
    } else {
        p = pattern::get_search_pattern();
        plen = pattern::get_search_pattern_len();
    }

    let msgbufsize: usize = if !shortmess(SHM_SEARCHCOUNT) || cmd_silent {
        if ui_has(K_UI_MESSAGES) {
            0
        } else if msg_scrolled != 0 && !cmd_silent {
            ((Rows - msg_row) * Columns - 1) as usize
        } else {
            ((Rows - msg_row - 1) * Columns + sc_col - 1) as usize
        }
    } else {
        plen + off_len + 3
    };

    let msgbufsize = if msgbufsize < plen + off_len + SEARCH_STAT_BUF_LEN + 3 {
        plen + off_len + SEARCH_STAT_BUF_LEN + 3
    } else {
        msgbufsize
    };

    let msgbuf = xmalloc(msgbufsize) as *mut c_char;
    memset(msgbuf as *mut c_void, b' ' as c_int, msgbufsize);
    let msgbuflen = msgbufsize - 1;
    *msgbuf.add(msgbuflen) = NUL;

    if !cmd_silent {
        ui_busy_start();
        *msgbuf = dirc as c_char;
        if nvim_utf_iscomposing_first(utf_ptr2char(p)) != 0 {
            *msgbuf.add(1) = b' ' as c_char;
            memmove(msgbuf.add(2) as *mut c_void, p as *const c_void, plen);
        } else {
            memmove(msgbuf.add(1) as *mut c_void, p as *const c_void, plen);
        }
        if off_len > 0 {
            memmove(
                msgbuf.add(plen + 1) as *mut c_void,
                off_buf.as_ptr() as *const c_void,
                off_len,
            );
        }

        let trunc = msg_strtrunc(msgbuf, 1);
        let msgbuf_final;
        let msgbuflen_final;
        if !trunc.is_null() {
            xfree(msgbuf as *mut c_void);
            msgbuf_final = trunc;
            msgbuflen_final = strlen(trunc);
        } else {
            msgbuf_final = msgbuf;
            msgbuflen_final = msgbuflen;
        }

        let (msgbuf_out, msgbuflen_out) = if nvim_curwin_rl_with_rlc_s() != 0 {
            let r = reverse_text(msgbuf_final);
            xfree(msgbuf_final as *mut c_void);
            let rlen = strlen(r);
            // Shift content: remove leading spaces
            let mut rp = r;
            while *rp == b' ' as c_char {
                rp = rp.add(1);
            }
            let pat_len = r.add(rlen).offset_from(rp) as usize;
            memmove(r as *mut c_void, rp as *const c_void, pat_len);
            let spaces = rp.offset_from(r) as usize;
            if spaces >= pat_len {
                memset(rp as *mut c_void, b' ' as c_int, pat_len);
            } else {
                memset(r.add(pat_len) as *mut c_void, b' ' as c_int, spaces);
            }
            (r, rlen)
        } else {
            (msgbuf_final, msgbuflen_final)
        };

        msg_outtrans(msgbuf_out, 0, false);
        msg_clr_eos();
        msg_check();
        gotocmdline(false);
        ui_flush();
        ui_busy_stop();
        msg_nowait = true;

        return DoSearchEchoResult {
            msgbuf: msgbuf_out,
            msgbuflen: msgbuflen_out,
            show_search_stats: c_int::from(!shortmess(SHM_SEARCHCOUNT)),
        };
    }

    DoSearchEchoResult {
        msgbuf,
        msgbuflen,
        show_search_stats: c_int::from(!shortmess(SHM_SEARCHCOUNT)),
    }
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
        // Clear line offset but preserve end flag
        direction::rs_set_search_offset_line_end_off(
            0,
            c_int::from(state::get_spat_off_end(state::RE_SEARCH)),
            0,
        );
    }

    // Save the offset for SEARCH_KEEP (now Rust-native)
    let saved_off = SavedSearchOff {
        dir: direction::get_search_direction(),
        line: state::get_spat_off_line(state::RE_SEARCH),
        end: state::get_spat_off_end(state::RE_SEARCH),
        off: state::get_spat_off_off(state::RE_SEARCH),
    };

    // Start position = cursor
    let cursor = do_search_get_cursor();
    let mut pos_lnum = cursor.lnum;
    let mut pos_col = cursor.col;

    // Find out search direction
    let mut dirc = if dirc_in == 0 {
        direction::get_search_direction() as u8 as c_int
    } else {
        direction::set_search_direction(dirc_in as c_char);
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
    let fold_pos = do_search_fold_adjust(dirc, pos_lnum, pos_col);
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
            if pattern::get_search_pattern().is_null() {
                if pattern::get_subst_pattern().is_null() {
                    do_search_emsg_noprevre();
                    retval = 0;
                    break;
                }
                searchstr = pattern::get_subst_pattern() as *mut c_char;
                searchstrlen = pattern::get_subst_pattern_len();
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
            direction::rs_set_search_offset_line_end_off(0, 0, 0);

            // Check for line offset or character offset
            let pc = *p_cur as u8;
            if pc == b'+' || pc == b'-' || ascii_isdigit(pc) {
                direction::rs_set_search_offset_line_end_off(1, 0, 0); // line offset
            } else if (search_options & options::SEARCH_OPT) != 0
                && (pc == b'e' || pc == b's' || pc == b'b')
            {
                if pc == b'e' {
                    direction::rs_set_search_offset_line_end_off(
                        c_int::from(state::get_spat_off_line(state::RE_SEARCH)),
                        1,
                        state::get_spat_off_off(state::RE_SEARCH),
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
                direction::rs_set_search_offset_line_end_off(
                    c_int::from(state::get_spat_off_line(state::RE_SEARCH)),
                    c_int::from(state::get_spat_off_end(state::RE_SEARCH)),
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

        // Echo the search pattern
        let echo_result = do_search_echo(dirc, search_options, searchstr, searchstrlen);

        // Pre-searchit character offset subtraction
        let pre_off = do_search_pre_offset(pos_lnum, pos_col);
        pos_lnum = pre_off.lnum;
        pos_col = pre_off.col;

        // Build searchit options
        let searchit_opts = c_int::from(state::get_spat_off_end(state::RE_SEARCH))
            * options::SEARCH_END
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
        if do_search_show_top_bot(dirc, pos_lnum, pos_col) != 0 {
            show_top_bot_msg = true;
        }

        if c == FAIL {
            retval = 0;
            // Cleanup echo
            if !echo_result.msgbuf.is_null() {
                xfree(echo_result.msgbuf as *mut c_void);
            }
            break;
        }

        // Set oap->inclusive if needed
        do_search_set_oap_inclusive(oap);
        retval = 1;

        // Fire SearchWrapped autocmd
        if has_sia != 0 && !sa_wrapped_out.is_null() && *sa_wrapped_out != 0 {
            do_search_autocmd_wrapped();
        }

        // Add character and/or line offset
        let pat_has_semi = if !pat.is_null() && *pat == b';' as c_char {
            1
        } else {
            0
        };
        let post = do_search_post_offset(pos_lnum, pos_col, search_options, pat_has_semi);
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
            xfree(echo_result.msgbuf as *mut c_void);
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
            do_search_emsg_e386();
            break;
        }
        pat = pat.add(1);
        patlen -= 1;
    }

    // Finish: set mark and cursor
    if retval != 0 {
        do_search_finish(search_options, pos_lnum, pos_col);
    }

    // Restore spats offset if SEARCH_KEEP (Rust-native, no C round-trip)
    if (search_options & options::SEARCH_KEEP) != 0 {
        // restore dir without calling set_vv_searchforward (matching original behavior)
        direction::set_search_direction_raw(saved_off.dir);
        direction::rs_set_search_offset_line_end_off(
            c_int::from(saved_off.line),
            c_int::from(saved_off.end),
            saved_off.off,
        );
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
