//! Implementation of `ex_retab()` — the `:retab` command.

use nvim_ex_cmds_types::ExArg;
use std::ffi::{c_char, c_int, c_void};

use crate::{rs_tabstop_count, rs_tabstop_eq, rs_tabstop_first, rs_tabstop_fromto, rs_tabstop_set};

type LinenrT = i32;
type ColnrT = i32;

const NUL: c_char = 0;
const OK: c_int = 1;
const FAIL: c_int = 0;
const UPD_NOT_VALID: c_int = 40;
const MAXCOL: c_int = 0x7fffffff;
const KEXTMARK_UNDO: c_int = 1;

/// Handle for exarg_T.
type EapHandle = *mut ExArg;
/// Opaque handle for buf_T.
type BufHandle = *mut c_void;

#[inline]
unsafe fn ascii_iswhite(c: c_char) -> bool {
    c == b' ' as c_char || c == b'\t' as c_char
}

#[inline]
unsafe fn ascii_isdigit(c: c_char) -> bool {
    c >= b'0' as c_char && c <= b'9' as c_char
}

// C accessor functions
extern "C" {
    static mut got_int: bool;
    // exarg_T field accessors
    fn nvim_eap_get_forceit(eap: EapHandle) -> bool;

    // Buffer operations
    fn nvim_ml_get(lnum: LinenrT) -> *mut c_char;
    fn nvim_ml_get_len(lnum: LinenrT) -> ColnrT;
    fn nvim_ml_replace(lnum: LinenrT, line: *mut c_char, copy: bool) -> c_int;
    fn nvim_curbuf_get_ml_line_ptr() -> *mut c_char;

    // Undo
    fn nvim_u_save(top: LinenrT, bot: LinenrT) -> c_int;
    fn nvim_u_clearline();

    // Buffer properties
    fn nvim_curbuf_get_p_et() -> bool;
    fn nvim_retab_curbuf_get_p_ts() -> i64;
    fn nvim_retab_curbuf_get_p_vts_array() -> *mut c_int;
    fn nvim_retab_curbuf_set_p_ts(val: i64);
    fn nvim_retab_curbuf_set_p_vts_array(val: *mut c_int);

    // Curwin
    fn nvim_curwin_w_p_list() -> c_int;
    fn nvim_curwin_set_w_p_list(val: c_int);
    fn nvim_curwin_get_w_curswant() -> ColnrT;
    fn nvim_coladvance(col: ColnrT);

    // Character sizing
    fn nvim_indent_win_chartabsize(ptr: *const c_char, vcol: ColnrT) -> ColnrT;
    fn nvim_utfc_ptr2len(p: *const c_char) -> c_int;

    // Misc
    fn skipwhite(s: *const c_char) -> *mut c_char;
    fn line_breakcheck();
    fn redraw_curbuf_later(typ: c_int);
    fn nvim_indent_changed_lines(first: LinenrT, last: LinenrT, xtra: LinenrT);
    fn nvim_set_option_direct_vts(str: *const c_char);
    fn nvim_emsg_interr();
    fn nvim_indent_get_curbuf() -> BufHandle;

    // Extmarks
    fn nvim_extmark_splice_cols(
        buf: BufHandle,
        start_row: c_int,
        start_col: ColnrT,
        old_col: ColnrT,
        new_col: ColnrT,
        undo: c_int,
    );

    // Memory
    #[link_name = "xfree"]
    fn nvim_xfree(ptr: *mut c_void);
}

// Rust functions already available in this crate
use crate::rs_emsg_text_too_long;

/// Implementation of the `:retab` command.
///
/// # Safety
/// - `eap` must be a valid exarg_T handle.
/// - Accesses global editor state (single-threaded).
#[export_name = "ex_retab"]
pub unsafe extern "C" fn rs_ex_retab(eap: EapHandle) {
    let mut got_tab = false;
    let mut num_spaces: c_int = 0;
    let mut start_col: c_int = 0;
    let mut start_vcol: i64 = 0;
    let mut new_line_valid = true; // replaces `new_line = (char *)1` sentinel
    let mut new_vts_array: *mut c_int = std::ptr::null_mut();

    let mut first_line: LinenrT = 0;
    let mut last_line: LinenrT = 0;
    let mut is_indent_only = false;

    let save_list = nvim_curwin_w_p_list();
    nvim_curwin_set_w_p_list(0); // don't want list mode here

    let mut ptr = (*eap).arg;

    // Check for "-indentonly" flag
    if starts_with_indentonly(ptr) && is_whitespace_or_nul(*ptr.add(11)) {
        is_indent_only = true;
        ptr = skipwhite(ptr.add(11));
    }

    let new_ts_str_start = ptr;
    if !rs_tabstop_set(ptr, &mut new_vts_array) {
        return;
    }
    while ascii_isdigit(*ptr) || *ptr == b',' as c_char {
        ptr = ptr.add(1);
    }

    // This ensures that either new_vts_array and new_ts_str are freshly
    // allocated, or new_vts_array points to an existing array and new_ts_str
    // is null.
    let mut new_ts_str: *mut c_char = std::ptr::null_mut();
    if new_vts_array.is_null() {
        new_vts_array = nvim_retab_curbuf_get_p_vts_array();
    } else {
        let len = ptr.offset_from(new_ts_str_start) as usize;
        new_ts_str = xmemdupz(new_ts_str_start, len);
    }

    let line1 = (*eap).line1;
    let line2 = (*eap).line2;
    let forceit = nvim_eap_get_forceit(eap);

    let mut lnum = line1;
    while !unsafe { got_int } && lnum <= line2 {
        ptr = nvim_ml_get(lnum);
        let mut old_len = nvim_ml_get_len(lnum) as c_int;
        let mut col: c_int = 0;
        let mut vcol: i64 = 0;
        let mut did_undo = false;

        loop {
            if ascii_iswhite(*ptr.offset(col as isize)) {
                if !got_tab && num_spaces == 0 {
                    // First consecutive white-space
                    start_vcol = vcol;
                    start_col = col;
                }
                if *ptr.offset(col as isize) == b' ' as c_char {
                    num_spaces += 1;
                } else {
                    got_tab = true;
                }
            } else {
                if got_tab || (forceit && num_spaces > 1) {
                    // Retabulate this string of white-space
                    let len_val = (vcol - start_vcol) as c_int;
                    num_spaces = len_val;
                    let mut len = len_val;
                    let mut num_tabs: c_int = 0;

                    if !nvim_curbuf_get_p_et() {
                        let ts = nvim_retab_curbuf_get_p_ts() as c_int;
                        let result = rs_tabstop_fromto(
                            start_vcol as c_int,
                            vcol as c_int,
                            ts,
                            new_vts_array,
                        );
                        num_tabs = result.ntabs;
                        num_spaces = result.nspcs;
                    }

                    if nvim_curbuf_get_p_et() || got_tab || (num_spaces + num_tabs < len) {
                        if !did_undo {
                            did_undo = true;
                            if nvim_u_save(lnum - 1, lnum + 1) == FAIL {
                                new_line_valid = false; // flag out-of-memory
                                break;
                            }
                        }

                        // len is actual number of white characters used
                        len = num_spaces + num_tabs;
                        let new_len = old_len - col + start_col + len + 1;
                        if new_len <= 0 || new_len == MAXCOL {
                            rs_emsg_text_too_long();
                            break;
                        }
                        let new_line = nvim_memory::xmalloc(new_len as usize).cast::<c_char>();

                        if start_col > 0 {
                            std::ptr::copy_nonoverlapping(ptr, new_line, start_col as usize);
                        }
                        std::ptr::copy(
                            ptr.offset(col as isize),
                            new_line.offset((start_col + len) as isize),
                            (old_len as usize) - (col as usize) + 1,
                        );
                        let fill_ptr = new_line.offset(start_col as isize);
                        for i in 0..len {
                            *fill_ptr.offset(i as isize) = if i < num_tabs {
                                b'\t' as c_char
                            } else {
                                b' ' as c_char
                            };
                        }
                        if nvim_ml_replace(lnum, new_line, false) == OK {
                            // "new_line" may have been copied
                            ptr = nvim_curbuf_get_ml_line_ptr();
                            let curbuf = nvim_indent_get_curbuf();
                            nvim_extmark_splice_cols(
                                curbuf,
                                lnum - 1,
                                0,
                                old_len as ColnrT,
                                (new_len - 1) as ColnrT,
                                KEXTMARK_UNDO,
                            );
                        } else {
                            ptr = new_line;
                        }
                        if first_line == 0 {
                            first_line = lnum;
                        }
                        last_line = lnum;
                        old_len = new_len - 1;
                        col = start_col + len;
                    }
                }
                got_tab = false;
                num_spaces = 0;

                if is_indent_only {
                    break;
                }
            }

            if *ptr.offset(col as isize) == NUL {
                break;
            }
            vcol += nvim_indent_win_chartabsize(ptr.offset(col as isize), vcol as ColnrT) as i64;
            if vcol >= MAXCOL as i64 {
                rs_emsg_text_too_long();
                break;
            }
            col += nvim_utfc_ptr2len(ptr.offset(col as isize));
        }

        if !new_line_valid {
            // out of memory
            break;
        }
        line_breakcheck();
        lnum += 1;
    }

    if unsafe { got_int } {
        nvim_emsg_interr();
    }

    // If a single value was given then it can be considered equal to
    // either the value of 'tabstop' or the value of 'vartabstop'.
    let cur_vts = nvim_retab_curbuf_get_p_vts_array();
    let ts_unchanged = (rs_tabstop_count(cur_vts) == 0
        && rs_tabstop_count(new_vts_array) == 1
        && nvim_retab_curbuf_get_p_ts() == i64::from(rs_tabstop_first(new_vts_array)))
        || (rs_tabstop_count(cur_vts) > 0 && rs_tabstop_eq(cur_vts, new_vts_array));
    if !ts_unchanged {
        redraw_curbuf_later(UPD_NOT_VALID);
    }
    if first_line != 0 {
        nvim_indent_changed_lines(first_line, last_line + 1, 0);
    }

    nvim_curwin_set_w_p_list(save_list); // restore 'list'

    if !new_ts_str.is_null() {
        // set the new tabstop
        let old_vts_ary = nvim_retab_curbuf_get_p_vts_array();

        if rs_tabstop_count(old_vts_ary) > 0 || rs_tabstop_count(new_vts_array) > 1 {
            nvim_set_option_direct_vts(new_ts_str);
            nvim_retab_curbuf_set_p_vts_array(new_vts_array);
            nvim_xfree(old_vts_ary.cast());
        } else {
            // 'vartabstop' wasn't in use and a single value was given to
            // retab then update 'tabstop'.
            nvim_retab_curbuf_set_p_ts(i64::from(rs_tabstop_first(new_vts_array)));
            nvim_xfree(new_vts_array.cast());
        }
        nvim_xfree(new_ts_str.cast());
    }
    nvim_coladvance(nvim_curwin_get_w_curswant());

    nvim_u_clearline();
}

/// Duplicate `len` bytes from `src` into a NUL-terminated allocation.
unsafe fn xmemdupz(src: *const c_char, len: usize) -> *mut c_char {
    let dst = nvim_memory::xmalloc(len + 1).cast::<c_char>();
    std::ptr::copy_nonoverlapping(src, dst, len);
    *dst.add(len) = NUL;
    dst
}

#[inline]
fn is_whitespace_or_nul(c: c_char) -> bool {
    c == b' ' as c_char || c == b'\t' as c_char || c == 0
}

/// Check if `ptr` starts with "-indentonly" (11 bytes).
#[inline]
unsafe fn starts_with_indentonly(ptr: *const c_char) -> bool {
    let expected = b"-indentonly";
    for (i, &byte) in expected.iter().enumerate() {
        if *ptr.add(i) != byte as c_char {
            return false;
        }
    }
    true
}
