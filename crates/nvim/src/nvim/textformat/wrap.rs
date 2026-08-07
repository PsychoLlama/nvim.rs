//! Auto-wrap: breaking the line being typed at 'textwidth'.
//!
//! [`internal_format`] is called from `edit.rs` for every character that
//! could take the line over the margin, and is reentrant with it: it calls
//! `open_line`, which runs the whole indent machinery and can call back in.
//! Two questions, in order -- *where* may this line be broken, and then what
//! it takes to actually break it there.

use super::*;
use crate::src::nvim::ascii::ascii_iswhite;
use crate::src::nvim::change::{get_leader_len, ins_bytes, ins_str, open_line};
use crate::src::nvim::charset::char2cells;
use crate::src::nvim::cursor::{
    coladvance, dec_cursor, gchar_cursor, get_cursor_line_len, get_cursor_line_ptr,
    get_cursor_pos_len, get_cursor_pos_ptr, inc_cursor, pchar_cursor,
};
use crate::src::nvim::drawscreen::{UPD_VALID, redraw_curbuf_later};
use crate::src::nvim::edit::{
    backspace_until_column, get_nolist_virtcol, set_can_cindent, undisplay_dollar,
};
use crate::src::nvim::indent::{change_indent, get_number_indent, set_indent};
use crate::src::nvim::main::{
    Insstart, State, can_si, can_si_back, curbuf, curwin, did_ai, did_si, got_int, old_indent,
    replace_offset,
};
use crate::src::nvim::mbyte::{
    utf_allow_break, utf_allow_break_before, utf_iscomposing_first, utf_ptr2char,
};
use crate::src::nvim::memory::xfree;
use crate::src::nvim::r#move::update_topline;
use crate::src::nvim::os::input::line_breakcheck;
use crate::src::nvim::pos::MAXCOL;
use crate::src::nvim::search::{FORWARD, check_linecomment};
use crate::src::nvim::state::VREPLACE_FLAG;
use crate::src::nvim::strings::xstrnsave;
use crate::src::nvim::types::{
    INSCHAR_COM_LIST, INSCHAR_DO_COM, INSCHAR_FORMAT, colnr_T, linenr_T, size_t,
};

pub unsafe extern "C" fn internal_format(
    mut textwidth: ::core::ffi::c_int,
    mut second_indent: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut format_only: bool,
    mut c: ::core::ffi::c_int,
) {
    let mut cc: ::core::ffi::c_int = 0;
    let mut save_char: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
    let mut haveto_redraw: bool = false;
    let fo_ins_blank: bool = has_format_option(FO_INS_BLANK);
    let fo_multibyte: bool = has_format_option(FO_MBYTE_BREAK);
    let fo_rigor_tw: bool = has_format_option(FO_RIGOROUS_TW);
    let fo_white_par: bool = has_format_option(FO_WHITE_PAR);
    let mut first_line: bool = true;
    let mut leader_len: colnr_T = 0;
    let mut no_leader: bool = false;
    let mut do_comments: bool = flags & INSCHAR_DO_COM as ::core::ffi::c_int != 0;
    let mut has_lbr: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_lbr;
    (*curwin.get()).w_onebuf_opt.wo_lbr = false_0;
    if (*curbuf.get()).b_p_ai == 0 && State.get() & VREPLACE_FLAG == 0 {
        cc = gchar_cursor();
        if ascii_iswhite(cc) {
            save_char = cc as ::core::ffi::c_char;
            pchar_cursor('x' as ::core::ffi::c_char);
        }
    }
    while !got_int.get() {
        let mut startcol: ::core::ffi::c_int = 0;
        let mut wantcol: ::core::ffi::c_int = 0;
        let mut foundcol: ::core::ffi::c_int = 0;
        let mut end_foundcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut orig_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut saved_text: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut col: colnr_T = 0;
        let mut did_do_comment: bool = false;
        let mut virtcol: colnr_T =
            get_nolist_virtcol() + char2cells(if c != NUL { c } else { gchar_cursor() });
        if virtcol <= textwidth {
            break;
        }
        if no_leader {
            do_comments = false;
        } else if flags & INSCHAR_FORMAT as ::core::ffi::c_int == 0
            && has_format_option(FO_WRAP_COMS) as ::core::ffi::c_int != 0
        {
            do_comments = true;
        }
        if do_comments {
            let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
            leader_len = get_leader_len(
                line,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                false,
                true,
            ) as colnr_T;
            if leader_len == 0 as ::core::ffi::c_int && (*curbuf.get()).b_p_cin != 0 {
                let mut comment_start: ::core::ffi::c_int = check_linecomment(line);
                if comment_start != MAXCOL as ::core::ffi::c_int {
                    leader_len = get_leader_len(
                        line.offset(comment_start as isize),
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                        false,
                        true,
                    ) as colnr_T;
                    if leader_len != 0 as ::core::ffi::c_int {
                        leader_len += comment_start;
                    }
                }
            }
        } else {
            leader_len = 0 as ::core::ffi::c_int as colnr_T;
        }
        if leader_len == 0 as ::core::ffi::c_int {
            no_leader = true;
        }
        if flags & INSCHAR_FORMAT as ::core::ffi::c_int == 0
            && leader_len == 0 as ::core::ffi::c_int
            && !has_format_option(FO_WRAP)
        {
            break;
        }
        startcol = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
        if startcol == 0 as ::core::ffi::c_int {
            break;
        }
        coladvance(curwin.get(), textwidth);
        wantcol = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
        (*curwin.get()).w_cursor.col = startcol as colnr_T;
        foundcol = 0 as ::core::ffi::c_int;
        let mut skip_pos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while !fo_ins_blank && !has_format_option(FO_INS_VI)
            || flags & INSCHAR_FORMAT as ::core::ffi::c_int != 0
            || (*curwin.get()).w_cursor.lnum != (*Insstart.ptr()).lnum
            || (*curwin.get()).w_cursor.col >= (*Insstart.ptr()).col
        {
            if (*curwin.get()).w_cursor.col == startcol && c != NUL {
                cc = c;
            } else {
                cc = gchar_cursor();
            }
            if ascii_iswhite(cc) as ::core::ffi::c_int != 0
                && !utf_iscomposing_first(utf_ptr2char(
                    get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
                ))
            {
                let mut end_col: colnr_T = (*curwin.get()).w_cursor.col;
                let mut wcc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
                    && (ascii_iswhite(cc) as ::core::ffi::c_int != 0
                        && !utf_iscomposing_first(utf_ptr2char(
                            get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
                        )))
                {
                    dec_cursor();
                    cc = gchar_cursor();
                    if wcc < 2 as ::core::ffi::c_int {
                        wcc += 1;
                    }
                }
                if (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int
                    && (ascii_iswhite(cc) as ::core::ffi::c_int != 0
                        && !utf_iscomposing_first(utf_ptr2char(
                            get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
                        )))
                {
                    break;
                } else {
                    if has_format_option(FO_PERIOD_ABBR) as ::core::ffi::c_int != 0
                        && cc == '.' as ::core::ffi::c_int
                        && wcc < 2 as ::core::ffi::c_int
                    {
                        continue;
                    }
                    if (*curwin.get()).w_cursor.col < leader_len {
                        break;
                    }
                    if has_format_option(FO_ONE_LETTER) {
                        if (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int {
                            break;
                        }
                        if (*curwin.get()).w_cursor.col <= leader_len {
                            break;
                        }
                        col = (*curwin.get()).w_cursor.col;
                        dec_cursor();
                        cc = gchar_cursor();
                        if ascii_iswhite(cc) as ::core::ffi::c_int != 0
                            && !utf_iscomposing_first(utf_ptr2char(
                                get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
                            ))
                        {
                            continue;
                        } else {
                            (*curwin.get()).w_cursor.col = col;
                        }
                    }
                    inc_cursor();
                    end_foundcol = end_col as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
                    foundcol = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                    if (*curwin.get()).w_cursor.col <= wantcol {
                        break;
                    }
                }
            } else if (cc >= 0x100 as ::core::ffi::c_int || !utf_allow_break_before(cc))
                && fo_multibyte as ::core::ffi::c_int != 0
            {
                let mut ncc: ::core::ffi::c_int = 0;
                let mut allow_break: bool = false;
                if (*curwin.get()).w_cursor.col != startcol {
                    if (*curwin.get()).w_cursor.col < leader_len {
                        break;
                    }
                    col = (*curwin.get()).w_cursor.col;
                    inc_cursor();
                    ncc = gchar_cursor();
                    allow_break = utf_allow_break(cc, ncc);
                    if (*curwin.get()).w_cursor.col != skip_pos
                        && allow_break as ::core::ffi::c_int != 0
                    {
                        foundcol = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                        end_foundcol = foundcol;
                        if (*curwin.get()).w_cursor.col <= wantcol {
                            break;
                        }
                    }
                    (*curwin.get()).w_cursor.col = col;
                }
                if (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int {
                    break;
                }
                ncc = cc;
                col = (*curwin.get()).w_cursor.col;
                dec_cursor();
                cc = gchar_cursor();
                if ascii_iswhite(cc) as ::core::ffi::c_int != 0
                    && !utf_iscomposing_first(utf_ptr2char(
                        get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
                    ))
                {
                    continue;
                } else {
                    if (*curwin.get()).w_cursor.col < leader_len {
                        break;
                    }
                    (*curwin.get()).w_cursor.col = col;
                    skip_pos = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                    allow_break = utf_allow_break(cc, ncc);
                    if allow_break {
                        foundcol = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                        end_foundcol = foundcol;
                    }
                    if (*curwin.get()).w_cursor.col <= wantcol {
                        let ncc_allow_break: bool = utf_allow_break_before(ncc);
                        if allow_break {
                            break;
                        }
                        if !ncc_allow_break && !fo_rigor_tw {
                            if (*curwin.get()).w_cursor.col == startcol {
                                foundcol = 0 as ::core::ffi::c_int;
                                end_foundcol = foundcol;
                                break;
                            } else {
                                col = (*curwin.get()).w_cursor.col;
                                inc_cursor();
                                cc = ncc;
                                ncc = gchar_cursor();
                                ncc = if ncc != NUL { ncc } else { c };
                                allow_break = utf_allow_break(cc, ncc);
                                if allow_break {
                                    foundcol = if ncc == NUL {
                                        0 as ::core::ffi::c_int
                                    } else {
                                        (*curwin.get()).w_cursor.col as ::core::ffi::c_int
                                    };
                                    end_foundcol = foundcol;
                                    break;
                                } else {
                                    (*curwin.get()).w_cursor.col = col;
                                }
                            }
                        }
                    }
                }
            }
            if (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int {
                break;
            }
            dec_cursor();
        }
        if foundcol == 0 as ::core::ffi::c_int {
            (*curwin.get()).w_cursor.col = startcol as colnr_T;
            break;
        } else {
            undisplay_dollar();
            if State.get() & VREPLACE_FLAG != 0 {
                orig_col = startcol;
            } else {
                replace_offset.set(startcol - end_foundcol);
            }
            (*curwin.get()).w_cursor.col = foundcol as colnr_T;
            loop {
                cc = gchar_cursor();
                if !(ascii_iswhite(cc) as ::core::ffi::c_int != 0
                    && !utf_iscomposing_first(utf_ptr2char(
                        get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
                    ))
                    && (!fo_white_par || (*curwin.get()).w_cursor.col < startcol))
                {
                    break;
                }
                inc_cursor();
            }
            startcol -= (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
            startcol = if startcol > 0 as ::core::ffi::c_int {
                startcol
            } else {
                0 as ::core::ffi::c_int
            };
            if State.get() & VREPLACE_FLAG != 0 {
                saved_text = xstrnsave(get_cursor_pos_ptr(), get_cursor_pos_len() as size_t);
                (*curwin.get()).w_cursor.col = orig_col as colnr_T;
                *saved_text.offset(startcol as isize) = NUL as ::core::ffi::c_char;
                if !fo_white_par {
                    backspace_until_column(foundcol);
                }
            } else if !fo_white_par {
                (*curwin.get()).w_cursor.col = foundcol as colnr_T;
            }
            open_line(
                FORWARD as ::core::ffi::c_int,
                OPENLINE_DELSPACES as ::core::ffi::c_int
                    + OPENLINE_MARKFIX as ::core::ffi::c_int
                    + (if fo_white_par as ::core::ffi::c_int != 0 {
                        OPENLINE_KEEPTRAIL as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    })
                    + (if do_comments as ::core::ffi::c_int != 0 {
                        OPENLINE_DO_COM as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    })
                    + OPENLINE_FORMAT as ::core::ffi::c_int
                    + (if flags & INSCHAR_COM_LIST as ::core::ffi::c_int != 0 {
                        OPENLINE_COM_LIST as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }),
                if flags & INSCHAR_COM_LIST as ::core::ffi::c_int != 0 {
                    second_indent
                } else {
                    old_indent.get()
                },
                &raw mut did_do_comment,
            );
            if flags & INSCHAR_COM_LIST as ::core::ffi::c_int == 0 {
                old_indent.set(0 as ::core::ffi::c_int);
            }
            if did_do_comment {
                no_leader = false;
            }
            replace_offset.set(0 as ::core::ffi::c_int);
            if first_line {
                if flags & INSCHAR_COM_LIST as ::core::ffi::c_int == 0 {
                    if second_indent < 0 as ::core::ffi::c_int
                        && has_format_option(FO_Q_NUMBER) as ::core::ffi::c_int != 0
                    {
                        second_indent =
                            get_number_indent((*curwin.get()).w_cursor.lnum - 1 as linenr_T);
                    }
                    if second_indent >= 0 as ::core::ffi::c_int {
                        if State.get() & VREPLACE_FLAG != 0 {
                            change_indent(
                                INDENT_SET as ::core::ffi::c_int,
                                second_indent,
                                false_0,
                                true,
                            );
                        } else if leader_len > 0 as ::core::ffi::c_int
                            && second_indent as colnr_T - leader_len > 0 as ::core::ffi::c_int
                        {
                            let mut padding: ::core::ffi::c_int =
                                second_indent - leader_len as ::core::ffi::c_int;
                            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            while i < padding {
                                ins_str(
                                    c" ".as_ptr() as *mut ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                        .wrapping_sub(1 as size_t),
                                );
                                i += 1;
                            }
                        } else {
                            set_indent(second_indent, SIN_CHANGED as ::core::ffi::c_int);
                        }
                    }
                }
                first_line = false;
            }
            if State.get() & VREPLACE_FLAG != 0 {
                ins_bytes(saved_text);
                xfree(saved_text as *mut ::core::ffi::c_void);
            } else {
                (*curwin.get()).w_cursor.col += startcol;
                let mut len: colnr_T = get_cursor_line_len();
                (*curwin.get()).w_cursor.col = if (*curwin.get()).w_cursor.col < len {
                    (*curwin.get()).w_cursor.col
                } else {
                    len
                };
            }
            haveto_redraw = true;
            set_can_cindent(true);
            did_ai.set(false);
            did_si.set(false);
            can_si.set(false);
            can_si_back.set(false);
            line_breakcheck();
        }
    }
    if save_char as ::core::ffi::c_int != NUL {
        pchar_cursor(save_char);
    }
    (*curwin.get()).w_onebuf_opt.wo_lbr = has_lbr;
    if !format_only && haveto_redraw as ::core::ffi::c_int != 0 {
        update_topline(curwin.get());
        redraw_curbuf_later(UPD_VALID);
    }
}
