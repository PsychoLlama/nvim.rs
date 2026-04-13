//! Paragraph formatting for text formatting.
//!
//! This module provides the Rust implementation of `format_lines()`,
//! which formats paragraphs by joining lines and re-breaking them.

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

// =============================================================================
// Constants
// =============================================================================

const NUL: c_char = 0;
const MAXCOL: c_int = 0x7fffffff;
const FAIL: c_int = 0;

// INSCHAR flags
const INSCHAR_FORMAT: c_int = 1;
const INSCHAR_DO_COM: c_int = 2;
const INSCHAR_NO_FEX: c_int = 8;
const INSCHAR_COM_LIST: c_int = 16;

// Mode flags
const MODE_NORMAL: c_int = 0x01;
const MODE_INSERT: c_int = 0x10;

// Indent flags
const SIN_CHANGED: c_int = 1;

// Format option characters
const FO_Q_COMS: c_int = b'q' as c_int;
const FO_Q_SECOND: c_int = b'2' as c_int;
const FO_Q_NUMBER: c_int = b'n' as c_int;
const FO_WHITE_PAR: c_int = b'w' as c_int;

// =============================================================================
// C Accessor Functions
// =============================================================================

extern "C" {
    static mut got_int: bool;
    static mut State: c_int;
    // Cursor and window
    fn nvim_textfmt_get_curwin_cursor_lnum() -> c_int;
    fn nvim_textfmt_get_curwin_cursor_col() -> c_int;
    fn nvim_textfmt_set_curwin_cursor(lnum: c_int, col: c_int);
    fn nvim_textfmt_get_curwin() -> *mut c_void;
    fn nvim_textfmt_coladvance(win: *mut c_void, col: c_int);
    fn nvim_textfmt_dec_cursor();
    fn nvim_textfmt_gchar_cursor() -> c_int;
    fn nvim_textfmt_get_cursor_line_ptr() -> *const c_char;
    fn nvim_textfmt_get_cursor_line_len() -> c_int;

    // Buffer state
    fn nvim_textfmt_get_ml_line_count() -> c_int;

    // State

    // Format lines helpers
    fn nvim_textfmt_del_bytes(count: c_int, fixpos: bool, use_delcombine: bool);
    fn nvim_textfmt_do_join(
        count: c_int,
        insert_space: bool,
        save_undo: bool,
        use_fo: bool,
        setmark: bool,
    ) -> c_int;
    fn nvim_textfmt_lineempty(lnum: c_int) -> bool;
    fn nvim_textfmt_get_p_smd() -> c_int;
    fn nvim_textfmt_set_p_smd(val: c_int);
    fn nvim_textfmt_get_c_indent() -> c_int;
    fn nvim_textfmt_get_expr_indent() -> c_int;

    // Existing accessors
    fn insertchar(c: c_int, flags: c_int, second_indent: c_int);
    #[link_name = "mark_col_adjust"]
    fn nvim_mark_col_adjust(
        lnum: c_int,
        col: c_int,
        amount_lnum: c_int,
        amount_col: c_int,
        spaces_removed: c_int,
    );
    #[link_name = "check_linecomment"]
    fn nvim_check_linecomment(line: *const c_char) -> c_int;
    fn ui_cursor_shape();
    fn nvim_curbuf_get_b_p_lisp() -> c_int;
    fn nvim_curbuf_get_b_p_inde_ptr() -> *const c_char;
    fn line_breakcheck();
    #[link_name = "beep_flush"]
    fn nvim_beep_flush();
    fn nvim_textfmt_u_save_cursor() -> c_int;
    fn nvim_textfmt_get_number_indent(lnum: c_int) -> c_int;

    // Indent crate
    #[link_name = "get_indent"]
    fn rs_get_indent() -> c_int;
    #[link_name = "get_indent_lnum"]
    fn rs_get_indent_lnum(lnum: c_int) -> c_int;
    #[link_name = "set_indent"]
    fn rs_set_indent(amount: c_int, flags: c_int) -> bool;
    #[link_name = "get_lisp_indent"]
    fn rs_get_lisp_indent() -> c_int;
    #[link_name = "cindent_on"]
    fn rs_cindent_on() -> bool;

    // Charset crate
    #[link_name = "getwhitecols_curline"]
    fn rs_getwhitecols_curline() -> c_int;
}

// =============================================================================
// Helper Functions
// =============================================================================

#[inline]
fn ascii_isspace(c: c_int) -> bool {
    let c = c as u8;
    c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' || c == 11 || c == 12
}

// =============================================================================
// Implementation
// =============================================================================

/// Format lines starting at the cursor position.
///
/// # Arguments
/// * `line_count` - Number of lines to format. When negative, format until end of paragraph.
/// * `avoid_fex` - Don't use 'formatexpr'
///
/// # Safety
/// Accesses global state via C functions.
pub(crate) unsafe fn format_lines_impl(line_count: c_int, avoid_fex: bool) {
    let mut is_not_par: bool;
    let mut next_is_not_par: bool;
    let mut is_end_par: bool;
    let mut prev_is_end_par: bool = false;
    let mut next_is_start_par: bool = false;
    let mut leader_len: c_int = 0;
    let mut next_leader_len: c_int = 0;
    let mut leader_flags: *mut c_char = ptr::null_mut();
    let mut next_leader_flags: *mut c_char = ptr::null_mut();
    let mut advance: bool = true;
    let mut second_indent: c_int = -1;
    let mut first_par_line: bool = true;
    let mut need_set_indent: bool = true;
    let first_line = nvim_textfmt_get_curwin_cursor_lnum();
    let mut force_format: bool = false;
    let old_state = State;

    // length of a line to force formatting: 3 * 'tw'
    let max_len = crate::textwidth::comp_textwidth_impl(true) * 3;

    // check for 'q', '2', 'n' and 'w' in 'formatoptions'
    let do_comments = crate::has_format_option_impl(FO_Q_COMS);
    let mut do_comments_list: c_int = 0;
    let do_second_indent = crate::has_format_option_impl(FO_Q_SECOND);
    let do_number_indent = crate::has_format_option_impl(FO_Q_NUMBER);
    let do_trail_white = crate::has_format_option_impl(FO_WHITE_PAR);

    // Get info about the previous and current line.
    let cursor_lnum = nvim_textfmt_get_curwin_cursor_lnum();
    if cursor_lnum > 1 {
        is_not_par = crate::paragraph::fmt_check_par_impl(
            cursor_lnum - 1,
            &mut leader_len,
            &mut leader_flags,
            do_comments,
        );
    } else {
        is_not_par = true;
    }
    next_is_not_par = crate::paragraph::fmt_check_par_impl(
        cursor_lnum,
        &mut next_leader_len,
        &mut next_leader_flags,
        do_comments,
    );
    is_end_par = is_not_par || next_is_not_par;
    if !is_end_par && do_trail_white {
        is_end_par = !crate::paragraph::ends_in_white_impl(cursor_lnum - 1);
    }

    nvim_textfmt_set_curwin_cursor(cursor_lnum - 1, 0);
    let mut count: c_int = line_count;
    while count != 0 && !unsafe { got_int } {
        // Advance to next paragraph.
        if advance {
            let lnum = nvim_textfmt_get_curwin_cursor_lnum();
            nvim_textfmt_set_curwin_cursor(lnum + 1, 0);
            prev_is_end_par = is_end_par;
            is_not_par = next_is_not_par;
            leader_len = next_leader_len;
            leader_flags = next_leader_flags;
        }

        let cursor_lnum = nvim_textfmt_get_curwin_cursor_lnum();
        let ml_line_count = nvim_textfmt_get_ml_line_count();

        // The last line to be formatted.
        if count == 1 || cursor_lnum == ml_line_count {
            next_is_not_par = true;
            next_leader_len = 0;
            next_leader_flags = ptr::null_mut();
        } else {
            next_is_not_par = crate::paragraph::fmt_check_par_impl(
                cursor_lnum + 1,
                &mut next_leader_len,
                &mut next_leader_flags,
                do_comments,
            );
            if do_number_indent {
                next_is_start_par = nvim_textfmt_get_number_indent(cursor_lnum + 1) > 0;
            }
        }
        advance = true;
        is_end_par = is_not_par || next_is_not_par || next_is_start_par;
        if !is_end_par && do_trail_white {
            is_end_par = !crate::paragraph::ends_in_white_impl(cursor_lnum);
        }

        // Skip lines that are not in a paragraph.
        if is_not_par {
            if line_count < 0 {
                break;
            }
        } else {
            let cursor_lnum = nvim_textfmt_get_curwin_cursor_lnum();
            let ml_line_count = nvim_textfmt_get_ml_line_count();

            // For the first line of a paragraph, check indent of second line.
            // Don't do this for comments and empty lines.
            if first_par_line
                && (do_second_indent || do_number_indent)
                && prev_is_end_par
                && cursor_lnum < ml_line_count
            {
                if do_second_indent && !nvim_textfmt_lineempty(cursor_lnum + 1) {
                    if leader_len == 0 && next_leader_len == 0 {
                        // no comment found
                        second_indent = rs_get_indent_lnum(cursor_lnum + 1);
                    } else {
                        second_indent = next_leader_len;
                        do_comments_list = 1;
                    }
                } else if do_number_indent {
                    if leader_len == 0 && next_leader_len == 0 {
                        // no comment found
                        second_indent = nvim_textfmt_get_number_indent(cursor_lnum);
                    } else {
                        // get_number_indent() is now "comment aware"...
                        second_indent = nvim_textfmt_get_number_indent(cursor_lnum);
                        do_comments_list = 1;
                    }
                }
            }

            // When the comment leader changes, it's the end of the paragraph.
            if cursor_lnum >= ml_line_count
                || !crate::paragraph::same_leader_impl(
                    cursor_lnum,
                    leader_len,
                    leader_flags,
                    next_leader_len,
                    next_leader_flags,
                )
            {
                // Special case: If the next line starts with a line comment
                // and this line has a line comment after some text, the
                // paragraph doesn't really end.
                if next_leader_flags.is_null()
                    || libc::strncmp(next_leader_flags, c"://".as_ptr(), 3) != 0
                    || nvim_check_linecomment(nvim_textfmt_get_cursor_line_ptr()) == MAXCOL
                {
                    is_end_par = true;
                }
            }

            // If we have got to the end of a paragraph, or the line is
            // getting long, format it.
            if is_end_par || force_format {
                if need_set_indent {
                    let indent;

                    // Replace indent in first line of a paragraph with minimal
                    // number of tabs and spaces, according to current options.
                    let cursor_lnum = nvim_textfmt_get_curwin_cursor_lnum();
                    if cursor_lnum == first_line {
                        indent = rs_get_indent();
                    } else if nvim_curbuf_get_b_p_lisp() != 0 {
                        indent = rs_get_lisp_indent();
                    } else if rs_cindent_on() {
                        let inde_ptr = nvim_curbuf_get_b_p_inde_ptr();
                        if !inde_ptr.is_null() && *inde_ptr != NUL {
                            indent = nvim_textfmt_get_expr_indent();
                        } else {
                            indent = nvim_textfmt_get_c_indent();
                        }
                    } else {
                        indent = rs_get_indent();
                    }
                    rs_set_indent(indent, SIN_CHANGED);
                }

                // put cursor on last non-space
                State = MODE_NORMAL; // don't go past end-of-line
                nvim_textfmt_coladvance(nvim_textfmt_get_curwin(), MAXCOL);
                while {
                    nvim_textfmt_get_curwin_cursor_col() > 0
                        && ascii_isspace(nvim_textfmt_gchar_cursor())
                } {
                    nvim_textfmt_dec_cursor();
                }

                // do the formatting, without 'showmode'
                State = MODE_INSERT; // for open_line()
                let smd_save = nvim_textfmt_get_p_smd();
                nvim_textfmt_set_p_smd(0);

                let mut flags = INSCHAR_FORMAT;
                if do_comments {
                    flags += INSCHAR_DO_COM;
                }
                if do_comments && do_comments_list != 0 {
                    flags += INSCHAR_COM_LIST;
                }
                if avoid_fex {
                    flags += INSCHAR_NO_FEX;
                }
                insertchar(0, flags, second_indent);

                State = old_state;
                nvim_textfmt_set_p_smd(smd_save);
                // Cursor shape may have been updated (e.g. by :normal) in insertchar(),
                // so it needs to be updated here.
                ui_cursor_shape();

                second_indent = -1;
                // at end of par.: need to set indent of next par.
                need_set_indent = is_end_par;
                if is_end_par {
                    // When called with a negative line count, break at the
                    // end of the paragraph.
                    if line_count < 0 {
                        break;
                    }
                    first_par_line = true;
                }
                force_format = false;
            }

            // When still in same paragraph, join the lines together. But
            // first delete the leader from the second line.
            if !is_end_par {
                advance = false;
                let lnum = nvim_textfmt_get_curwin_cursor_lnum();
                nvim_textfmt_set_curwin_cursor(lnum + 1, 0);
                if line_count < 0 && nvim_textfmt_u_save_cursor() == FAIL {
                    break;
                }
                if next_leader_len > 0 {
                    nvim_textfmt_del_bytes(next_leader_len, false, false);
                    let lnum = nvim_textfmt_get_curwin_cursor_lnum();
                    nvim_mark_col_adjust(lnum, 0, 0, -next_leader_len, 0);
                } else if second_indent > 0 {
                    // the "leader" for FO_Q_SECOND
                    let indent = rs_getwhitecols_curline();
                    if indent > 0 {
                        nvim_textfmt_del_bytes(indent, false, false);
                        let lnum = nvim_textfmt_get_curwin_cursor_lnum();
                        nvim_mark_col_adjust(lnum, 0, 0, -indent, 0);
                    }
                }
                let lnum = nvim_textfmt_get_curwin_cursor_lnum();
                nvim_textfmt_set_curwin_cursor(lnum - 1, 0);
                if nvim_textfmt_do_join(2, true, false, false, false) == FAIL {
                    nvim_beep_flush();
                    break;
                }
                first_par_line = false;
                // If the line is getting long, format it next time
                force_format = nvim_textfmt_get_cursor_line_len() > max_len;
            }
        }
        line_breakcheck();
        count -= 1;
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Format lines in a paragraph.
///
/// # Safety
/// Accesses global state via C functions.
#[export_name = "format_lines"]
pub unsafe extern "C" fn rs_format_lines(line_count: c_int, avoid_fex: c_int) {
    format_lines_impl(line_count, avoid_fex != 0);
}
