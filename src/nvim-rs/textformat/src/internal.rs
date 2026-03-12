//! Text-wrapping state machine for insert mode.
//!
//! This module provides the Rust implementation of `internal_format()`,
//! which handles line-breaking at textwidth during insert mode.

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Constants
// =============================================================================

const NUL: c_int = 0;
const MAXCOL: c_int = 0x7fffffff;

// INSCHAR flags
const INSCHAR_FORMAT: c_int = 1;
const INSCHAR_DO_COM: c_int = 2;
const INSCHAR_COM_LIST: c_int = 16;

// OPENLINE flags
const OPENLINE_DELSPACES: c_int = 0x01;
const OPENLINE_DO_COM: c_int = 0x02;
const OPENLINE_KEEPTRAIL: c_int = 0x04;
const OPENLINE_MARKFIX: c_int = 0x08;
const OPENLINE_COM_LIST: c_int = 0x10;
const OPENLINE_FORMAT: c_int = 0x20;

const FORWARD: c_int = 1;
const VREPLACE_FLAG: c_int = 0x200;
const UPD_VALID: c_int = 10;
const INDENT_SET: c_int = 1;
const SIN_CHANGED: c_int = 1;

// Format option characters
const FO_INS_BLANK: c_int = b'b' as c_int;
const FO_INS_VI: c_int = b'v' as c_int;
const FO_MBYTE_BREAK: c_int = b'm' as c_int;
const FO_RIGOROUS_TW: c_int = b']' as c_int;
const FO_WHITE_PAR: c_int = b'w' as c_int;
const FO_WRAP_COMS: c_int = b'c' as c_int;
const FO_WRAP: c_int = b't' as c_int;
const FO_ONE_LETTER: c_int = b'1' as c_int;
const FO_Q_NUMBER: c_int = b'n' as c_int;
const FO_PERIOD_ABBR: c_int = b'p' as c_int;

// =============================================================================
// C Accessor Functions
// =============================================================================

extern "C" {
    // Cursor/window
    fn nvim_textfmt_get_curwin_cursor_lnum() -> c_int;
    fn nvim_textfmt_get_curwin_cursor_col() -> c_int;
    fn nvim_textfmt_set_curwin_cursor(lnum: c_int, col: c_int);
    fn nvim_textfmt_get_curwin() -> *mut c_void;
    fn nvim_textfmt_coladvance(win: *mut c_void, col: c_int);
    fn nvim_textfmt_dec_cursor();
    fn nvim_textfmt_inc_cursor();
    fn nvim_textfmt_gchar_cursor() -> c_int;
    fn nvim_textfmt_get_cursor_line_ptr() -> *const c_char;
    fn nvim_textfmt_get_cursor_line_len() -> c_int;
    fn nvim_textfmt_whitechar(cc: c_int) -> bool;

    // Phase 3 specific
    fn nvim_textfmt_pchar_cursor(c: c_int);
    fn nvim_textfmt_undisplay_dollar();
    fn nvim_textfmt_backspace_until_column(col: c_int);
    fn nvim_textfmt_open_line(
        dir: c_int,
        flags: c_int,
        indent: c_int,
        did_do_comment: *mut bool,
    ) -> bool;
    fn nvim_textfmt_set_replace_offset(val: c_int);
    fn nvim_textfmt_utf_allow_break(cc: c_int, ncc: c_int) -> bool;
    fn nvim_textfmt_utf_allow_break_before(cc: c_int) -> bool;
    fn nvim_textfmt_get_curwin_w_p_lbr() -> c_int;
    fn nvim_textfmt_set_curwin_w_p_lbr(val: c_int);
    fn nvim_textfmt_get_cursor_pos_len() -> c_int;

    // Existing accessors from other modules
    fn get_nolist_virtcol() -> c_int;
    fn nvim_char2cells(c: c_int) -> c_int;
    fn nvim_get_cursor_pos_ptr() -> *const c_char;
    fn nvim_set_did_ai(val: bool);
    fn nvim_set_did_si(val: bool);
    fn nvim_set_can_si(val: bool);
    fn nvim_set_can_si_back(val: bool);
    fn nvim_set_can_cindent(val: bool);
    fn nvim_edit_update_topline(win: *mut c_void);
    fn nvim_line_breakcheck();
    fn nvim_curbuf_get_b_p_ai() -> c_int;
    fn nvim_curbuf_get_b_p_cin() -> c_int;
    fn nvim_get_State() -> c_int;
    fn nvim_get_got_int() -> c_int;
    fn nvim_get_Insstart_lnum() -> c_int;
    fn nvim_get_Insstart_col() -> c_int;
    fn nvim_xstrnsave(s: *const c_char, len: usize) -> *mut c_char;
    fn nvim_xfree(ptr: *mut c_void);
    fn nvim_ins_bytes(p: *const c_char);
    fn nvim_edit_ins_str(p: *const c_char, len: usize);
    fn nvim_textfmt_get_leader_len_simple(line: *const c_char) -> c_int;
    fn nvim_check_linecomment(line: *const c_char) -> c_int;
    fn nvim_textfmt_get_number_indent(lnum: c_int) -> c_int;
    fn nvim_textfmt_redraw_curbuf_later(typ: c_int);

    // Indent
    fn nvim_get_old_indent() -> c_int;
    fn nvim_set_old_indent(val: c_int);
    #[link_name = "set_indent"]
    fn rs_set_indent(amount: c_int, flags: c_int) -> bool;
    #[link_name = "change_indent"]
    fn rs_change_indent(typ: c_int, amount: c_int, round: c_int, call_changed_bytes: bool);
}

// =============================================================================
// Helpers
// =============================================================================

#[inline]
fn ascii_iswhite(c: c_int) -> bool {
    c == b' ' as c_int || c == b'\t' as c_int
}

// =============================================================================
// Implementation
// =============================================================================

/// Format text at the current insert position.
///
/// If the INSCHAR_COM_LIST flag is present, then the value of second_indent
/// will be the comment leader length sent to open_line().
///
/// # Safety
/// Accesses global state via C functions.
#[allow(clippy::too_many_lines)]
pub(crate) unsafe fn internal_format_impl(
    textwidth: c_int,
    mut second_indent: c_int,
    flags: c_int,
    format_only: bool,
    c: c_int,
) {
    let mut save_char: c_int = NUL;
    let mut haveto_redraw = false;
    let fo_ins_blank = crate::has_format_option_impl(FO_INS_BLANK);
    let fo_multibyte = crate::has_format_option_impl(FO_MBYTE_BREAK);
    let fo_rigor_tw = crate::has_format_option_impl(FO_RIGOROUS_TW);
    let fo_white_par = crate::has_format_option_impl(FO_WHITE_PAR);
    let mut first_line = true;
    let mut leader_len: c_int;
    let mut no_leader = false;
    let mut do_comments = (flags & INSCHAR_DO_COM) != 0;
    let has_lbr = nvim_textfmt_get_curwin_w_p_lbr();

    // make sure win_charsize() counts correctly
    nvim_textfmt_set_curwin_w_p_lbr(0);

    // When 'ai' is off we don't want a space under the cursor to be
    // deleted. Replace it with an 'x' temporarily.
    if nvim_curbuf_get_b_p_ai() == 0 && (nvim_get_State() & VREPLACE_FLAG) == 0 {
        let cc = nvim_textfmt_gchar_cursor();
        if ascii_iswhite(cc) {
            save_char = cc;
            nvim_textfmt_pchar_cursor(b'x' as c_int);
        }
    }

    // Repeat breaking lines, until the current line is not too long.
    while nvim_get_got_int() == 0 {
        let mut foundcol: c_int;
        let mut end_foundcol: c_int = 0;
        let mut orig_col: c_int = 0;
        let mut saved_text: *mut c_char = std::ptr::null_mut();
        let mut did_do_comment = false;
        let mut cc: c_int;

        let virtcol = get_nolist_virtcol()
            + nvim_char2cells(if c != NUL {
                c
            } else {
                nvim_textfmt_gchar_cursor()
            });
        if virtcol <= textwidth {
            break;
        }

        if no_leader {
            do_comments = false;
        } else if (flags & INSCHAR_FORMAT) == 0 && crate::has_format_option_impl(FO_WRAP_COMS) {
            do_comments = true;
        }

        // Don't break until after the comment leader
        if do_comments {
            let line = nvim_textfmt_get_cursor_line_ptr();
            leader_len = nvim_textfmt_get_leader_len_simple(line);
            if leader_len == 0 && nvim_curbuf_get_b_p_cin() != 0 {
                // Check for a line comment after code.
                let comment_start = nvim_check_linecomment(line);
                if comment_start != MAXCOL {
                    leader_len =
                        nvim_textfmt_get_leader_len_simple(line.add(comment_start as usize));
                    if leader_len != 0 {
                        leader_len += comment_start;
                    }
                }
            }
        } else {
            leader_len = 0;
        }

        // If the line doesn't start with a comment leader, then don't
        // start one in a following broken line.
        if leader_len == 0 {
            no_leader = true;
        }
        if (flags & INSCHAR_FORMAT) == 0
            && leader_len == 0
            && !crate::has_format_option_impl(FO_WRAP)
        {
            break;
        }
        let startcol = nvim_textfmt_get_curwin_cursor_col();
        if startcol == 0 {
            break;
        }

        // find column of textwidth border
        nvim_textfmt_coladvance(nvim_textfmt_get_curwin(), textwidth);
        let wantcol = nvim_textfmt_get_curwin_cursor_col();

        let lnum = nvim_textfmt_get_curwin_cursor_lnum();
        nvim_textfmt_set_curwin_cursor(lnum, startcol);
        foundcol = 0;
        let mut skip_pos: c_int = 0;

        // Find position to break at.
        // Stop at first entered white when 'formatoptions' has 'v'
        loop {
            let cursor_lnum = nvim_textfmt_get_curwin_cursor_lnum();
            let cursor_col = nvim_textfmt_get_curwin_cursor_col();

            if !(!fo_ins_blank && !crate::has_format_option_impl(FO_INS_VI))
                && (flags & INSCHAR_FORMAT) == 0
                && (cursor_lnum == nvim_get_Insstart_lnum() && cursor_col < nvim_get_Insstart_col())
            {
                break;
            }

            let cursor_col = nvim_textfmt_get_curwin_cursor_col();
            if cursor_col == startcol && c != NUL {
                cc = c;
            } else {
                cc = nvim_textfmt_gchar_cursor();
            }

            if nvim_textfmt_whitechar(cc) {
                // remember position of blank just before text
                let end_col = nvim_textfmt_get_curwin_cursor_col();

                // find start of sequence of blanks
                let mut wcc: c_int = 0;
                while nvim_textfmt_get_curwin_cursor_col() > 0 && nvim_textfmt_whitechar(cc) {
                    nvim_textfmt_dec_cursor();
                    cc = nvim_textfmt_gchar_cursor();
                    if wcc < 2 {
                        wcc += 1;
                    }
                }
                if nvim_textfmt_get_curwin_cursor_col() == 0 && nvim_textfmt_whitechar(cc) {
                    break; // only spaces in front of text
                }

                // Don't break after a period when 'formatoptions' has 'p' and
                // there are less than two spaces.
                if crate::has_format_option_impl(FO_PERIOD_ABBR) && cc == b'.' as c_int && wcc < 2 {
                    if nvim_textfmt_get_curwin_cursor_col() == 0 {
                        break;
                    }
                    nvim_textfmt_dec_cursor();
                    continue;
                }

                // Don't break until after the comment leader
                if nvim_textfmt_get_curwin_cursor_col() < leader_len {
                    break;
                }

                if crate::has_format_option_impl(FO_ONE_LETTER) {
                    // do not break after one-letter words
                    if nvim_textfmt_get_curwin_cursor_col() == 0 {
                        break; // one-letter word at begin
                    }
                    // do not break "#a b" when 'tw' is 2
                    if nvim_textfmt_get_curwin_cursor_col() <= leader_len {
                        break;
                    }
                    let col = nvim_textfmt_get_curwin_cursor_col();
                    nvim_textfmt_dec_cursor();
                    cc = nvim_textfmt_gchar_cursor();

                    if nvim_textfmt_whitechar(cc) {
                        if nvim_textfmt_get_curwin_cursor_col() == 0 {
                            break;
                        }
                        nvim_textfmt_dec_cursor();
                        continue; // one-letter, continue
                    }
                    let lnum = nvim_textfmt_get_curwin_cursor_lnum();
                    nvim_textfmt_set_curwin_cursor(lnum, col);
                }

                nvim_textfmt_inc_cursor();

                end_foundcol = end_col + 1;
                foundcol = nvim_textfmt_get_curwin_cursor_col();
                if nvim_textfmt_get_curwin_cursor_col() <= wantcol {
                    break;
                }
            } else if (cc >= 0x100 || !nvim_textfmt_utf_allow_break_before(cc)) && fo_multibyte {
                let mut ncc: c_int;
                let mut allow_break: bool;

                // Break after or before a multi-byte character.
                let cursor_col = nvim_textfmt_get_curwin_cursor_col();
                if cursor_col != startcol {
                    // Don't break until after the comment leader
                    if cursor_col < leader_len {
                        break;
                    }
                    let col = cursor_col;
                    nvim_textfmt_inc_cursor();
                    ncc = nvim_textfmt_gchar_cursor();
                    allow_break = nvim_textfmt_utf_allow_break(cc, ncc);

                    // If we have already checked this position, skip!
                    if nvim_textfmt_get_curwin_cursor_col() != skip_pos && allow_break {
                        foundcol = nvim_textfmt_get_curwin_cursor_col();
                        end_foundcol = foundcol;
                        if nvim_textfmt_get_curwin_cursor_col() <= wantcol {
                            break;
                        }
                    }
                    let lnum = nvim_textfmt_get_curwin_cursor_lnum();
                    nvim_textfmt_set_curwin_cursor(lnum, col);
                }

                if nvim_textfmt_get_curwin_cursor_col() == 0 {
                    break;
                }

                ncc = cc;
                let col = nvim_textfmt_get_curwin_cursor_col();

                nvim_textfmt_dec_cursor();
                cc = nvim_textfmt_gchar_cursor();

                if nvim_textfmt_whitechar(cc) {
                    if nvim_textfmt_get_curwin_cursor_col() == 0 {
                        break;
                    }
                    nvim_textfmt_dec_cursor();
                    continue; // break with space
                }
                // Don't break until after the comment leader.
                if nvim_textfmt_get_curwin_cursor_col() < leader_len {
                    break;
                }

                let lnum = nvim_textfmt_get_curwin_cursor_lnum();
                nvim_textfmt_set_curwin_cursor(lnum, col);
                skip_pos = col;

                allow_break = nvim_textfmt_utf_allow_break(cc, ncc);

                // Must handle this to respect line break prohibition.
                if allow_break {
                    foundcol = col;
                    end_foundcol = foundcol;
                }
                if col <= wantcol {
                    let ncc_allow_break = nvim_textfmt_utf_allow_break_before(ncc);

                    if allow_break {
                        break;
                    }
                    if !ncc_allow_break && !fo_rigor_tw {
                        // Enable at most 1 punct hang outside of textwidth.
                        if col == startcol {
                            // We are inserting a non-breakable char, postpone
                            // line break check to next insert.
                            foundcol = 0;
                            end_foundcol = 0;
                            break;
                        }

                        // Neither cc nor ncc is NUL if we are here, so
                        // it's safe to inc_cursor.
                        let col2 = nvim_textfmt_get_curwin_cursor_col();

                        nvim_textfmt_inc_cursor();
                        cc = ncc;
                        ncc = nvim_textfmt_gchar_cursor();
                        // handle insert
                        if ncc == NUL {
                            ncc = c;
                        }

                        allow_break = nvim_textfmt_utf_allow_break(cc, ncc);

                        if allow_break {
                            // Break only when we are not at end of line.
                            let val = if ncc == NUL {
                                0
                            } else {
                                nvim_textfmt_get_curwin_cursor_col()
                            };
                            foundcol = val;
                            end_foundcol = val;
                            break;
                        }
                        let lnum = nvim_textfmt_get_curwin_cursor_lnum();
                        nvim_textfmt_set_curwin_cursor(lnum, col2);
                    }
                }
            }
            if nvim_textfmt_get_curwin_cursor_col() == 0 {
                break;
            }
            nvim_textfmt_dec_cursor();
        }

        if foundcol == 0 {
            // no spaces, cannot break line
            let lnum = nvim_textfmt_get_curwin_cursor_lnum();
            nvim_textfmt_set_curwin_cursor(lnum, startcol);
            break;
        }

        // Going to break the line, remove any "$" now.
        nvim_textfmt_undisplay_dollar();

        // Offset between cursor position and line break is used by replace
        // stack functions. MODE_VREPLACE does not use this, and backspaces
        // over the text instead.
        if (nvim_get_State() & VREPLACE_FLAG) != 0 {
            orig_col = startcol; // Will start backspacing from here
        } else {
            nvim_textfmt_set_replace_offset(startcol - end_foundcol);
        }

        // adjust startcol for spaces that will be deleted and
        // characters that will remain on top line
        let lnum = nvim_textfmt_get_curwin_cursor_lnum();
        nvim_textfmt_set_curwin_cursor(lnum, foundcol);
        let mut startcol = startcol;
        loop {
            cc = nvim_textfmt_gchar_cursor();
            if !nvim_textfmt_whitechar(cc) {
                break;
            }
            if fo_white_par && nvim_textfmt_get_curwin_cursor_col() >= startcol {
                break;
            }
            nvim_textfmt_inc_cursor();
        }
        startcol -= nvim_textfmt_get_curwin_cursor_col();
        if startcol < 0 {
            startcol = 0;
        }

        if (nvim_get_State() & VREPLACE_FLAG) != 0 {
            // In MODE_VREPLACE state, we will backspace over the text to be
            // wrapped, so save a copy now to put on the next line.
            saved_text = nvim_xstrnsave(
                nvim_get_cursor_pos_ptr(),
                nvim_textfmt_get_cursor_pos_len() as usize,
            );
            let lnum = nvim_textfmt_get_curwin_cursor_lnum();
            nvim_textfmt_set_curwin_cursor(lnum, orig_col);
            *saved_text.add(startcol as usize) = 0;

            // Backspace over characters that will move to the next line
            if !fo_white_par {
                nvim_textfmt_backspace_until_column(foundcol);
            }
        } else {
            // put cursor after pos. to break line
            if !fo_white_par {
                let lnum = nvim_textfmt_get_curwin_cursor_lnum();
                nvim_textfmt_set_curwin_cursor(lnum, foundcol);
            }
        }

        // Split the line just before the margin.
        // Only insert/delete lines, but don't really redraw the window.
        let mut ol_flags = OPENLINE_DELSPACES + OPENLINE_MARKFIX;
        if fo_white_par {
            ol_flags += OPENLINE_KEEPTRAIL;
        }
        if do_comments {
            ol_flags += OPENLINE_DO_COM;
        }
        ol_flags += OPENLINE_FORMAT;
        if (flags & INSCHAR_COM_LIST) != 0 {
            ol_flags += OPENLINE_COM_LIST;
        }
        let ol_indent = if (flags & INSCHAR_COM_LIST) != 0 {
            second_indent
        } else {
            nvim_get_old_indent()
        };
        nvim_textfmt_open_line(FORWARD, ol_flags, ol_indent, &mut did_do_comment);
        if (flags & INSCHAR_COM_LIST) == 0 {
            nvim_set_old_indent(0);
        }

        // If a comment leader was inserted, may also do this on a following line.
        if did_do_comment {
            no_leader = false;
        }

        nvim_textfmt_set_replace_offset(0);
        if first_line {
            if (flags & INSCHAR_COM_LIST) == 0 {
                // This section is for auto-wrap of numeric lists.
                if second_indent < 0 && crate::has_format_option_impl(FO_Q_NUMBER) {
                    second_indent =
                        nvim_textfmt_get_number_indent(nvim_textfmt_get_curwin_cursor_lnum() - 1);
                }
                if second_indent >= 0 {
                    if (nvim_get_State() & VREPLACE_FLAG) != 0 {
                        rs_change_indent(INDENT_SET, second_indent, 0, true);
                    } else if leader_len > 0 && second_indent - leader_len > 0 {
                        let padding = second_indent - leader_len;
                        for _ in 0..padding {
                            nvim_edit_ins_str(c" ".as_ptr(), 1);
                        }
                    } else {
                        rs_set_indent(second_indent, SIN_CHANGED);
                    }
                }
            }
            first_line = false;
        }

        if (nvim_get_State() & VREPLACE_FLAG) != 0 {
            // In MODE_VREPLACE state we have backspaced over the text to be
            // moved, now we re-insert it into the new line.
            nvim_ins_bytes(saved_text);
            nvim_xfree(saved_text.cast());
        } else {
            // Check if cursor is not past the NUL off the line, cindent
            // may have added or removed indent.
            let lnum = nvim_textfmt_get_curwin_cursor_lnum();
            let new_col = nvim_textfmt_get_curwin_cursor_col() + startcol;
            let len = nvim_textfmt_get_cursor_line_len();
            nvim_textfmt_set_curwin_cursor(lnum, if new_col < len { new_col } else { len });
        }

        haveto_redraw = true;
        nvim_set_can_cindent(true);
        // moved the cursor, don't autoindent or cindent now
        nvim_set_did_ai(false);
        nvim_set_did_si(false);
        nvim_set_can_si(false);
        nvim_set_can_si_back(false);
        nvim_line_breakcheck();
    }

    if save_char != NUL {
        // put back space after cursor
        nvim_textfmt_pchar_cursor(save_char);
    }

    nvim_textfmt_set_curwin_w_p_lbr(has_lbr);

    if !format_only && haveto_redraw {
        nvim_edit_update_topline(nvim_textfmt_get_curwin());
        nvim_textfmt_redraw_curbuf_later(UPD_VALID);
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Format text at the current insert position.
///
/// # Safety
/// Accesses global state via C functions.
#[no_mangle]
pub unsafe extern "C" fn rs_internal_format(
    textwidth: c_int,
    second_indent: c_int,
    flags: c_int,
    format_only: c_int,
    c: c_int,
) {
    internal_format_impl(textwidth, second_indent, flags, format_only != 0, c);
}
