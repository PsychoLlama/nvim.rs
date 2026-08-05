use crate::src::nvim::ascii::{ascii_isspace, ascii_iswhite};
use crate::src::nvim::change::{
    del_bytes, del_char, get_leader_len, ins_bytes, ins_str, open_line,
};
use crate::src::nvim::charset::{char2cells, getwhitecols_curline, skipwhite};
use crate::src::nvim::cursor::{
    check_cursor, check_cursor_col, coladvance, dec_cursor, gchar_cursor, get_cursor_line_len,
    get_cursor_line_ptr, get_cursor_pos_len, get_cursor_pos_ptr, inc_cursor, pchar_cursor,
};
use crate::src::nvim::drawscreen::{UPD_INVERTED, UPD_VALID, redraw_curbuf_later};
use crate::src::nvim::edit::{
    backspace_until_column, beginline, get_nolist_virtcol, insertchar, set_can_cindent,
    undisplay_dollar,
};
use crate::src::nvim::eval::eval_to_number;
use crate::src::nvim::eval::vars::{set_vim_var_char, set_vim_var_nr, set_vim_var_string};
use crate::src::nvim::getchar::beep_flush;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::indent::{
    change_indent, get_expr_indent, get_indent, get_indent_lnum, get_lisp_indent,
    get_number_indent, set_indent,
};
use crate::src::nvim::indent_c::{cindent_on, get_c_indent};
use crate::src::nvim::main::{
    Insstart, State, can_si, can_si_back, cmdmod, cmdwin_buf, curbuf, current_sctx, curtab, curwin,
    did_ai, did_si, firstwin, got_int, old_indent, p_paste, p_smd, replace_offset, sandbox,
    saved_cursor,
};
use crate::src::nvim::mark::mark_col_adjust;
use crate::src::nvim::mbyte::{
    utf_allow_break, utf_allow_break_before, utf_iscomposing_first, utf_ptr2char,
};
use crate::src::nvim::memline::{ml_get, ml_get_len, ml_replace};
use crate::src::nvim::memory::{xfree, xstrdup};
use crate::src::nvim::message::msgmore;
use crate::src::nvim::r#move::update_topline;
use crate::src::nvim::ops::do_join;
use crate::src::nvim::option::was_set_insecurely;
use crate::src::nvim::options::kOptFormatexpr;
use crate::src::nvim::os::input::line_breakcheck;
use crate::src::nvim::os::libc::strncmp;
use crate::src::nvim::pos::MAXCOL;
use crate::src::nvim::search::{FORWARD, check_linecomment};
use crate::src::nvim::state::{MODE_INSERT, MODE_NORMAL, VREPLACE_FLAG};
use crate::src::nvim::strings::{vim_strchr, xstrnsave};
use crate::src::nvim::textobject::startPS;
use crate::src::nvim::types::{
    CMOD_LOCKMARKS, VV_CHAR, VV_COUNT, VV_LNUM, colnr_T, linenr_T, oparg_T, pos_T, ptrdiff_t,
    sctx_T, size_t, uint8_t, varnumber_T, win_T,
};
use crate::src::nvim::ui::ui_cursor_shape;
use crate::src::nvim::undo::{u_save, u_save_cursor};
use crate::src::nvim::window::win_fdccol_count;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_int;
pub const kBufOptFormatexpr: C2Rust_Unnamed_14 = 36;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const OPENLINE_FORMAT: C2Rust_Unnamed_15 = 32;
pub const OPENLINE_COM_LIST: C2Rust_Unnamed_15 = 16;
pub const OPENLINE_MARKFIX: C2Rust_Unnamed_15 = 8;
pub const OPENLINE_KEEPTRAIL: C2Rust_Unnamed_15 = 4;
pub const OPENLINE_DO_COM: C2Rust_Unnamed_15 = 2;
pub const OPENLINE_DELSPACES: C2Rust_Unnamed_15 = 1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const INDENT_SET: C2Rust_Unnamed_18 = 1;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_19 = 4;
pub const BL_WHITE: C2Rust_Unnamed_19 = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const INSCHAR_COM_LIST: C2Rust_Unnamed_20 = 16;
pub const INSCHAR_NO_FEX: C2Rust_Unnamed_20 = 8;
pub const INSCHAR_DO_COM: C2Rust_Unnamed_20 = 2;
pub const INSCHAR_FORMAT: C2Rust_Unnamed_20 = 1;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const SIN_CHANGED: C2Rust_Unnamed_22 = 1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const OPT_LOCAL: C2Rust_Unnamed_23 = 2;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FO_WRAP: ::core::ffi::c_int = 't' as ::core::ffi::c_int;
pub const FO_WRAP_COMS: ::core::ffi::c_int = 'c' as ::core::ffi::c_int;
pub const FO_Q_COMS: ::core::ffi::c_int = 'q' as ::core::ffi::c_int;
pub const FO_Q_NUMBER: ::core::ffi::c_int = 'n' as ::core::ffi::c_int;
pub const FO_Q_SECOND: ::core::ffi::c_int = '2' as ::core::ffi::c_int;
pub const FO_INS_VI: ::core::ffi::c_int = 'v' as ::core::ffi::c_int;
pub const FO_INS_BLANK: ::core::ffi::c_int = 'b' as ::core::ffi::c_int;
pub const FO_MBYTE_BREAK: ::core::ffi::c_int = 'm' as ::core::ffi::c_int;
pub const FO_ONE_LETTER: ::core::ffi::c_int = '1' as ::core::ffi::c_int;
pub const FO_WHITE_PAR: ::core::ffi::c_int = 'w' as ::core::ffi::c_int;
pub const FO_AUTO: ::core::ffi::c_int = 'a' as ::core::ffi::c_int;
pub const FO_RIGOROUS_TW: ::core::ffi::c_int = ']' as ::core::ffi::c_int;
pub const FO_PERIOD_ABBR: ::core::ffi::c_int = 'p' as ::core::ffi::c_int;
pub const COM_START: ::core::ffi::c_int = 's' as ::core::ffi::c_int;
pub const COM_MIDDLE: ::core::ffi::c_int = 'm' as ::core::ffi::c_int;
pub const COM_END: ::core::ffi::c_int = 'e' as ::core::ffi::c_int;
pub const COM_FIRST: ::core::ffi::c_int = 'f' as ::core::ffi::c_int;
static did_add_space: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub unsafe extern "C" fn has_format_option(mut x: ::core::ffi::c_int) -> bool {
    if p_paste.get() != 0 {
        return false_0 != 0;
    }
    return !vim_strchr((*curbuf.get()).b_p_fo, x).is_null();
}
pub unsafe extern "C" fn internal_format(
    mut textwidth: ::core::ffi::c_int,
    mut second_indent: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut format_only: bool,
    mut c: ::core::ffi::c_int,
) {
    let mut cc: ::core::ffi::c_int = 0;
    let mut save_char: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
    let mut haveto_redraw: bool = false_0 != 0;
    let fo_ins_blank: bool = has_format_option(FO_INS_BLANK);
    let fo_multibyte: bool = has_format_option(FO_MBYTE_BREAK);
    let fo_rigor_tw: bool = has_format_option(FO_RIGOROUS_TW);
    let fo_white_par: bool = has_format_option(FO_WHITE_PAR);
    let mut first_line: bool = true_0 != 0;
    let mut leader_len: colnr_T = 0;
    let mut no_leader: bool = false_0 != 0;
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
        let mut did_do_comment: bool = false_0 != 0;
        let mut virtcol: colnr_T =
            get_nolist_virtcol() + char2cells(if c != NUL { c } else { gchar_cursor() });
        if virtcol <= textwidth {
            break;
        }
        if no_leader {
            do_comments = false_0 != 0;
        } else if flags & INSCHAR_FORMAT as ::core::ffi::c_int == 0
            && has_format_option(FO_WRAP_COMS) as ::core::ffi::c_int != 0
        {
            do_comments = true_0 != 0;
        }
        if do_comments {
            let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
            leader_len = get_leader_len(
                line,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                false_0 != 0,
                true_0 != 0,
            ) as colnr_T;
            if leader_len == 0 as ::core::ffi::c_int && (*curbuf.get()).b_p_cin != 0 {
                let mut comment_start: ::core::ffi::c_int = check_linecomment(line);
                if comment_start != MAXCOL as ::core::ffi::c_int {
                    leader_len = get_leader_len(
                        line.offset(comment_start as isize),
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                        false_0 != 0,
                        true_0 != 0,
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
            no_leader = true_0 != 0;
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
                no_leader = false_0 != 0;
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
                                true_0 != 0,
                            );
                        } else if leader_len > 0 as ::core::ffi::c_int
                            && second_indent as colnr_T - leader_len > 0 as ::core::ffi::c_int
                        {
                            let mut padding: ::core::ffi::c_int =
                                second_indent - leader_len as ::core::ffi::c_int;
                            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            while i < padding {
                                ins_str(
                                    b" \0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
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
                first_line = false_0 != 0;
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
            haveto_redraw = true_0 != 0;
            set_can_cindent(true_0 != 0);
            did_ai.set(false_0 != 0);
            did_si.set(false_0 != 0);
            can_si.set(false_0 != 0);
            can_si_back.set(false_0 != 0);
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
unsafe extern "C" fn fmt_check_par(
    mut lnum: linenr_T,
    mut leader_len: *mut ::core::ffi::c_int,
    mut leader_flags: *mut *mut ::core::ffi::c_char,
    mut do_comments: bool,
) -> ::core::ffi::c_int {
    let mut flags: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ptr: *mut ::core::ffi::c_char = ml_get(lnum);
    if do_comments {
        *leader_len = get_leader_len(ptr, leader_flags, false_0 != 0, true_0 != 0);
    } else {
        *leader_len = 0 as ::core::ffi::c_int;
    }
    if *leader_len > 0 as ::core::ffi::c_int {
        flags = *leader_flags;
        while *flags as ::core::ffi::c_int != 0
            && *flags as ::core::ffi::c_int != ':' as ::core::ffi::c_int
            && *flags as ::core::ffi::c_int != COM_END
        {
            flags = flags.offset(1);
        }
    }
    return (*skipwhite(ptr.offset(*leader_len as isize)) as ::core::ffi::c_int == NUL
        || *leader_len > 0 as ::core::ffi::c_int && *flags as ::core::ffi::c_int == COM_END
        || startPS(lnum, NUL, false_0 != 0) as ::core::ffi::c_int != 0)
        as ::core::ffi::c_int;
}
unsafe extern "C" fn ends_in_white(mut lnum: linenr_T) -> bool {
    let mut s: *mut ::core::ffi::c_char = ml_get(lnum);
    if *s as ::core::ffi::c_int == NUL {
        return false_0 != 0;
    }
    let mut l: colnr_T = ml_get_len(lnum) - 1 as colnr_T;
    return ascii_iswhite(*s.offset(l as isize) as uint8_t as ::core::ffi::c_int);
}
unsafe extern "C" fn same_leader(
    mut lnum: linenr_T,
    mut leader1_len: ::core::ffi::c_int,
    mut leader1_flags: *mut ::core::ffi::c_char,
    mut leader2_len: ::core::ffi::c_int,
    mut leader2_flags: *mut ::core::ffi::c_char,
) -> bool {
    let mut idx1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut idx2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if leader1_len == 0 as ::core::ffi::c_int {
        return leader2_len == 0 as ::core::ffi::c_int;
    }
    if !leader1_flags.is_null() {
        let mut p: *mut ::core::ffi::c_char = leader1_flags;
        while *p as ::core::ffi::c_int != 0 && *p as ::core::ffi::c_int != ':' as ::core::ffi::c_int
        {
            if *p as ::core::ffi::c_int == COM_FIRST {
                return leader2_len == 0 as ::core::ffi::c_int;
            }
            if *p as ::core::ffi::c_int == COM_END {
                return false_0 != 0;
            }
            if *p as ::core::ffi::c_int == COM_START {
                let mut line_len: ::core::ffi::c_int = ml_get_len(lnum);
                if line_len <= leader1_len {
                    return false_0 != 0;
                }
                if leader2_flags.is_null() || leader2_len == 0 as ::core::ffi::c_int {
                    return false_0 != 0;
                }
                p = leader2_flags;
                while *p as ::core::ffi::c_int != 0
                    && *p as ::core::ffi::c_int != ':' as ::core::ffi::c_int
                {
                    if *p as ::core::ffi::c_int == COM_MIDDLE {
                        return true_0 != 0;
                    }
                    p = p.offset(1);
                }
                return false_0 != 0;
            }
            p = p.offset(1);
        }
    }
    let mut line1: *mut ::core::ffi::c_char = xstrnsave(ml_get(lnum), ml_get_len(lnum) as size_t);
    idx1 = 0 as ::core::ffi::c_int;
    while ascii_iswhite(*line1.offset(idx1 as isize) as ::core::ffi::c_int) {
        idx1 += 1;
    }
    let mut line2: *mut ::core::ffi::c_char = ml_get(lnum + 1 as linenr_T);
    idx2 = 0 as ::core::ffi::c_int;
    while idx2 < leader2_len {
        if !ascii_iswhite(*line2.offset(idx2 as isize) as ::core::ffi::c_int) {
            let c2rust_fresh0 = idx1;
            idx1 = idx1 + 1;
            if *line1.offset(c2rust_fresh0 as isize) as ::core::ffi::c_int
                != *line2.offset(idx2 as isize) as ::core::ffi::c_int
            {
                break;
            }
        } else {
            while ascii_iswhite(*line1.offset(idx1 as isize) as ::core::ffi::c_int) {
                idx1 += 1;
            }
        }
        idx2 += 1;
    }
    xfree(line1 as *mut ::core::ffi::c_void);
    return idx2 == leader2_len && idx1 == leader1_len;
}
unsafe extern "C" fn paragraph_start(mut lnum: linenr_T) -> bool {
    let mut leader_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut leader_flags: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut next_leader_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut next_leader_flags: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lnum <= 1 as linenr_T {
        return true_0 != 0;
    }
    let mut p: *mut ::core::ffi::c_char = ml_get(lnum - 1 as linenr_T);
    if *p as ::core::ffi::c_int == NUL {
        return true_0 != 0;
    }
    let do_comments: bool = has_format_option(FO_Q_COMS);
    if fmt_check_par(
        lnum - 1 as linenr_T,
        &raw mut leader_len,
        &raw mut leader_flags,
        do_comments,
    ) != 0
    {
        return true_0 != 0;
    }
    if fmt_check_par(
        lnum,
        &raw mut next_leader_len,
        &raw mut next_leader_flags,
        do_comments,
    ) != 0
    {
        return true_0 != 0;
    }
    if has_format_option(FO_WHITE_PAR) as ::core::ffi::c_int != 0
        && !ends_in_white(lnum - 1 as linenr_T)
    {
        return true_0 != 0;
    }
    if has_format_option(FO_Q_NUMBER) as ::core::ffi::c_int != 0
        && get_number_indent(lnum) > 0 as ::core::ffi::c_int
    {
        return true_0 != 0;
    }
    if !same_leader(
        lnum - 1 as linenr_T,
        leader_len,
        leader_flags,
        next_leader_len,
        next_leader_flags,
    ) {
        return true_0 != 0;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn auto_format(mut trailblank: bool, mut prev_line: bool) {
    if !has_format_option(FO_AUTO) {
        return;
    }
    let mut pos: pos_T = (*curwin.get()).w_cursor;
    let mut old: *mut ::core::ffi::c_char = get_cursor_line_ptr();
    check_auto_format(false_0 != 0);
    let mut wasatend: bool = pos.col == get_cursor_line_len();
    if *old as ::core::ffi::c_int != NUL && !trailblank && wasatend as ::core::ffi::c_int != 0 {
        dec_cursor();
        let mut cc: ::core::ffi::c_int = gchar_cursor();
        if !(ascii_iswhite(cc) as ::core::ffi::c_int != 0
            && !utf_iscomposing_first(utf_ptr2char(
                get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
            )))
            && (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
            && has_format_option(FO_ONE_LETTER) as ::core::ffi::c_int != 0
        {
            dec_cursor();
        }
        cc = gchar_cursor();
        if ascii_iswhite(cc) as ::core::ffi::c_int != 0
            && !utf_iscomposing_first(utf_ptr2char(
                get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
            ))
        {
            (*curwin.get()).w_cursor = pos;
            return;
        }
        (*curwin.get()).w_cursor = pos;
    }
    if *old as ::core::ffi::c_int != NUL
        && !trailblank
        && !wasatend
        && pos.col > 0 as ::core::ffi::c_int
        && State.get() & MODE_INSERT != 0
    {
        let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
        if ascii_iswhite(
            *line.offset((pos.col as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int,
        ) as ::core::ffi::c_int
            != 0
            && !utf_iscomposing_first(utf_ptr2char(
                get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
            ))
        {
            (*curwin.get()).w_cursor = pos;
            return;
        }
    }
    if has_format_option(FO_WRAP_COMS) as ::core::ffi::c_int != 0
        && !has_format_option(FO_WRAP)
        && get_leader_len(
            old,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            false_0 != 0,
            true_0 != 0,
        ) == 0 as ::core::ffi::c_int
    {
        return;
    }
    if prev_line as ::core::ffi::c_int != 0 && !paragraph_start((*curwin.get()).w_cursor.lnum) {
        (*curwin.get()).w_cursor.lnum -= 1;
        if u_save_cursor() == FAIL {
            return;
        }
    }
    saved_cursor.set(pos);
    format_lines(-1 as linenr_T, false_0 != 0);
    (*curwin.get()).w_cursor = saved_cursor.get();
    (*saved_cursor.ptr()).lnum = 0 as ::core::ffi::c_int as linenr_T;
    if (*curwin.get()).w_cursor.lnum > (*curbuf.get()).b_ml.ml_line_count {
        (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
        coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
    } else {
        check_cursor_col(curwin.get());
    }
    if !wasatend && has_format_option(FO_WHITE_PAR) as ::core::ffi::c_int != 0 {
        let mut linep: *mut ::core::ffi::c_char = get_cursor_line_ptr();
        let mut len: colnr_T = get_cursor_line_len();
        if (*curwin.get()).w_cursor.col == len {
            let mut plinep: *mut ::core::ffi::c_char =
                xstrnsave(linep, (len as size_t).wrapping_add(2 as size_t));
            *plinep.offset(len as isize) = ' ' as ::core::ffi::c_char;
            *plinep.offset((len as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
                NUL as ::core::ffi::c_char;
            ml_replace((*curwin.get()).w_cursor.lnum, plinep, false_0 != 0);
            did_add_space.set(true_0 != 0);
        } else {
            check_auto_format(false_0 != 0);
        }
    }
    check_cursor(curwin.get());
}
pub unsafe extern "C" fn check_auto_format(mut end_insert: bool) {
    if !did_add_space.get() {
        return;
    }
    let mut cc: ::core::ffi::c_int = gchar_cursor();
    if !(ascii_iswhite(cc) as ::core::ffi::c_int != 0
        && !utf_iscomposing_first(utf_ptr2char(
            get_cursor_pos_ptr().offset(1 as ::core::ffi::c_int as isize),
        )))
    {
        did_add_space.set(false_0 != 0);
    } else {
        let mut c: ::core::ffi::c_int = ' ' as ::core::ffi::c_int;
        if !end_insert {
            inc_cursor();
            c = gchar_cursor();
            dec_cursor();
        }
        if c != NUL {
            del_char(false_0 != 0);
            did_add_space.set(false_0 != 0);
        }
    };
}
pub unsafe extern "C" fn comp_textwidth(mut ff: bool) -> ::core::ffi::c_int {
    let mut textwidth: ::core::ffi::c_int = (*curbuf.get()).b_p_tw as ::core::ffi::c_int;
    if textwidth == 0 as ::core::ffi::c_int && (*curbuf.get()).b_p_wm != 0 {
        textwidth = (*curwin.get()).w_view_width - (*curbuf.get()).b_p_wm as ::core::ffi::c_int;
        if curbuf.get() == cmdwin_buf.get() {
            textwidth -= 1 as ::core::ffi::c_int;
        }
        textwidth -= win_fdccol_count(curwin.get());
        textwidth -= (*curwin.get()).w_scwidth;
        if (*curwin.get()).w_onebuf_opt.wo_nu != 0 || (*curwin.get()).w_onebuf_opt.wo_rnu != 0 {
            textwidth -= 8 as ::core::ffi::c_int;
        }
    }
    textwidth = if textwidth > 0 as ::core::ffi::c_int {
        textwidth
    } else {
        0 as ::core::ffi::c_int
    };
    if ff as ::core::ffi::c_int != 0 && textwidth == 0 as ::core::ffi::c_int {
        textwidth = if ((*curwin.get()).w_view_width - 1 as ::core::ffi::c_int)
            < 79 as ::core::ffi::c_int
        {
            (*curwin.get()).w_view_width - 1 as ::core::ffi::c_int
        } else {
            79 as ::core::ffi::c_int
        };
    }
    return textwidth;
}
pub unsafe extern "C" fn op_format(mut oap: *mut oparg_T, mut keep_cursor: bool) {
    let mut old_line_count: linenr_T = (*curbuf.get()).b_ml.ml_line_count;
    (*curwin.get()).w_cursor = (*oap).cursor_start;
    if u_save(
        (*oap).start.lnum - 1 as linenr_T,
        (*oap).end.lnum + 1 as linenr_T,
    ) == FAIL
    {
        return;
    }
    (*curwin.get()).w_cursor = (*oap).start;
    if (*oap).is_VIsual {
        redraw_curbuf_later(UPD_INVERTED);
    }
    if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        (*curbuf.get()).b_op_start = (*oap).start;
    }
    if keep_cursor {
        saved_cursor.set((*oap).cursor_start);
    }
    format_lines((*oap).line_count, keep_cursor);
    if (*oap).end_adjusted as ::core::ffi::c_int != 0
        && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count
    {
        (*curwin.get()).w_cursor.lnum += 1;
    }
    beginline(BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
    old_line_count = (*curbuf.get()).b_ml.ml_line_count - old_line_count;
    msgmore(old_line_count as ::core::ffi::c_int);
    if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
    }
    if keep_cursor {
        (*curwin.get()).w_cursor = saved_cursor.get();
        (*saved_cursor.ptr()).lnum = 0 as ::core::ffi::c_int as linenr_T;
        check_cursor(curwin.get());
    }
    if (*oap).is_VIsual {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_old_cursor_lnum != 0 as linenr_T {
                if (*wp).w_old_cursor_lnum > (*wp).w_old_visual_lnum {
                    (*wp).w_old_cursor_lnum += old_line_count;
                } else {
                    (*wp).w_old_visual_lnum += old_line_count;
                }
            }
            wp = (*wp).w_next;
        }
    }
}
pub unsafe extern "C" fn op_formatexpr(mut oap: *mut oparg_T) {
    if (*oap).is_VIsual {
        redraw_curbuf_later(UPD_INVERTED);
    }
    if fex_format(
        (*oap).start.lnum,
        (*oap).line_count as ::core::ffi::c_long,
        NUL,
    ) != 0 as ::core::ffi::c_int
    {
        op_format(oap, false_0 != 0);
    }
}
pub unsafe extern "C" fn fex_format(
    mut lnum: linenr_T,
    mut count: ::core::ffi::c_long,
    mut c: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut use_sandbox: bool = was_set_insecurely(
        curwin.get(),
        kOptFormatexpr,
        OPT_LOCAL as ::core::ffi::c_int,
    );
    let save_sctx: sctx_T = current_sctx.get();
    set_vim_var_nr(VV_LNUM, lnum as varnumber_T);
    set_vim_var_nr(VV_COUNT, count as varnumber_T);
    set_vim_var_char(c);
    let mut fex: *mut ::core::ffi::c_char = xstrdup((*curbuf.get()).b_p_fex);
    current_sctx
        .set((*curbuf.get()).b_p_script_ctx[kBufOptFormatexpr as ::core::ffi::c_int as usize]);
    if use_sandbox {
        (*sandbox.ptr()) += 1;
    }
    let mut r: ::core::ffi::c_int = eval_to_number(fex, true_0 != 0) as ::core::ffi::c_int;
    if use_sandbox {
        (*sandbox.ptr()) -= 1;
    }
    set_vim_var_string(
        VV_CHAR,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ptrdiff_t,
    );
    xfree(fex as *mut ::core::ffi::c_void);
    current_sctx.set(save_sctx);
    return r;
}
pub unsafe extern "C" fn format_lines(mut line_count: linenr_T, mut avoid_fex: bool) {
    let mut is_not_par: bool = false;
    let mut next_is_not_par: bool = false;
    let mut is_end_par: bool = false;
    let mut prev_is_end_par: bool = false_0 != 0;
    let mut next_is_start_par: bool = false_0 != 0;
    let mut leader_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut next_leader_len: ::core::ffi::c_int = 0;
    let mut leader_flags: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut next_leader_flags: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut advance: bool = true_0 != 0;
    let mut second_indent: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut first_par_line: bool = true_0 != 0;
    let mut smd_save: ::core::ffi::c_int = 0;
    let mut count: ::core::ffi::c_long = 0;
    let mut need_set_indent: bool = true_0 != 0;
    let mut first_line: linenr_T = (*curwin.get()).w_cursor.lnum;
    let mut force_format: bool = false_0 != 0;
    let old_State: ::core::ffi::c_int = State.get();
    let max_len: ::core::ffi::c_int = comp_textwidth(true_0 != 0) * 3 as ::core::ffi::c_int;
    let do_comments: bool = has_format_option(FO_Q_COMS);
    let mut do_comments_list: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let do_second_indent: bool = has_format_option(FO_Q_SECOND);
    let do_number_indent: bool = has_format_option(FO_Q_NUMBER);
    let do_trail_white: bool = has_format_option(FO_WHITE_PAR);
    if (*curwin.get()).w_cursor.lnum > 1 as linenr_T {
        is_not_par = fmt_check_par(
            (*curwin.get()).w_cursor.lnum - 1 as linenr_T,
            &raw mut leader_len,
            &raw mut leader_flags,
            do_comments,
        ) != 0;
    } else {
        is_not_par = true_0 != 0;
    }
    next_is_not_par = fmt_check_par(
        (*curwin.get()).w_cursor.lnum,
        &raw mut next_leader_len,
        &raw mut next_leader_flags,
        do_comments,
    ) != 0;
    is_end_par =
        is_not_par as ::core::ffi::c_int != 0 || next_is_not_par as ::core::ffi::c_int != 0;
    if !is_end_par && do_trail_white as ::core::ffi::c_int != 0 {
        is_end_par = !ends_in_white((*curwin.get()).w_cursor.lnum - 1 as linenr_T);
    }
    (*curwin.get()).w_cursor.lnum -= 1;
    count = line_count as ::core::ffi::c_long;
    while count != 0 as ::core::ffi::c_long && !got_int.get() {
        if advance {
            (*curwin.get()).w_cursor.lnum += 1;
            prev_is_end_par = is_end_par;
            is_not_par = next_is_not_par;
            leader_len = next_leader_len;
            leader_flags = next_leader_flags;
        }
        if count == 1 as ::core::ffi::c_long
            || (*curwin.get()).w_cursor.lnum == (*curbuf.get()).b_ml.ml_line_count
        {
            next_is_not_par = true_0 != 0;
            next_leader_len = 0 as ::core::ffi::c_int;
            next_leader_flags = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            next_is_not_par = fmt_check_par(
                (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
                &raw mut next_leader_len,
                &raw mut next_leader_flags,
                do_comments,
            ) != 0;
            if do_number_indent {
                next_is_start_par =
                    get_number_indent((*curwin.get()).w_cursor.lnum + 1 as linenr_T)
                        > 0 as ::core::ffi::c_int;
            }
        }
        advance = true_0 != 0;
        is_end_par = is_not_par as ::core::ffi::c_int != 0
            || next_is_not_par as ::core::ffi::c_int != 0
            || next_is_start_par as ::core::ffi::c_int != 0;
        if !is_end_par && do_trail_white as ::core::ffi::c_int != 0 {
            is_end_par = !ends_in_white((*curwin.get()).w_cursor.lnum);
        }
        if is_not_par {
            if line_count < 0 as linenr_T {
                break;
            }
        } else {
            if first_par_line as ::core::ffi::c_int != 0
                && (do_second_indent as ::core::ffi::c_int != 0
                    || do_number_indent as ::core::ffi::c_int != 0)
                && prev_is_end_par as ::core::ffi::c_int != 0
                && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count
            {
                if do_second_indent as ::core::ffi::c_int != 0
                    && !(*ml_get((*curwin.get()).w_cursor.lnum + 1 as linenr_T)
                        as ::core::ffi::c_int
                        == NUL)
                {
                    if leader_len == 0 as ::core::ffi::c_int
                        && next_leader_len == 0 as ::core::ffi::c_int
                    {
                        second_indent =
                            get_indent_lnum((*curwin.get()).w_cursor.lnum + 1 as linenr_T);
                    } else {
                        second_indent = next_leader_len;
                        do_comments_list = 1 as ::core::ffi::c_int;
                    }
                } else if do_number_indent {
                    if leader_len == 0 as ::core::ffi::c_int
                        && next_leader_len == 0 as ::core::ffi::c_int
                    {
                        second_indent = get_number_indent((*curwin.get()).w_cursor.lnum);
                    } else {
                        second_indent = get_number_indent((*curwin.get()).w_cursor.lnum);
                        do_comments_list = 1 as ::core::ffi::c_int;
                    }
                }
            }
            if (*curwin.get()).w_cursor.lnum >= (*curbuf.get()).b_ml.ml_line_count
                || !same_leader(
                    (*curwin.get()).w_cursor.lnum,
                    leader_len,
                    leader_flags,
                    next_leader_len,
                    next_leader_flags,
                )
            {
                if next_leader_flags.is_null()
                    || strncmp(
                        next_leader_flags,
                        b"://\0".as_ptr() as *const ::core::ffi::c_char,
                        3 as size_t,
                    ) != 0 as ::core::ffi::c_int
                    || check_linecomment(get_cursor_line_ptr()) == MAXCOL as ::core::ffi::c_int
                {
                    is_end_par = true_0 != 0;
                }
            }
            if is_end_par as ::core::ffi::c_int != 0 || force_format as ::core::ffi::c_int != 0 {
                if need_set_indent {
                    let mut indent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    if (*curwin.get()).w_cursor.lnum == first_line {
                        indent = get_indent();
                    } else if (*curbuf.get()).b_p_lisp != 0 {
                        indent = get_lisp_indent();
                    } else if cindent_on() {
                        indent = if *(*curbuf.get()).b_p_inde as ::core::ffi::c_int != NUL {
                            get_expr_indent()
                        } else {
                            get_c_indent()
                        };
                    } else {
                        indent = get_indent();
                    }
                    set_indent(indent, SIN_CHANGED as ::core::ffi::c_int);
                }
                State.set(MODE_NORMAL);
                coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
                while (*curwin.get()).w_cursor.col != 0
                    && ascii_isspace(gchar_cursor()) as ::core::ffi::c_int != 0
                {
                    dec_cursor();
                }
                State.set(MODE_INSERT);
                smd_save = p_smd.get();
                p_smd.set(false_0);
                insertchar(
                    NUL,
                    INSCHAR_FORMAT as ::core::ffi::c_int
                        + (if do_comments as ::core::ffi::c_int != 0 {
                            INSCHAR_DO_COM as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        })
                        + (if do_comments as ::core::ffi::c_int != 0 && do_comments_list != 0 {
                            INSCHAR_COM_LIST as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        })
                        + (if avoid_fex as ::core::ffi::c_int != 0 {
                            INSCHAR_NO_FEX as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        }),
                    second_indent,
                );
                State.set(old_State);
                p_smd.set(smd_save);
                ui_cursor_shape();
                second_indent = -1 as ::core::ffi::c_int;
                need_set_indent = is_end_par;
                if is_end_par {
                    if line_count < 0 as linenr_T {
                        break;
                    }
                    first_par_line = true_0 != 0;
                }
                force_format = false_0 != 0;
            }
            if !is_end_par {
                advance = false_0 != 0;
                (*curwin.get()).w_cursor.lnum += 1;
                (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                if line_count < 0 as linenr_T && u_save_cursor() == FAIL {
                    break;
                }
                if next_leader_len > 0 as ::core::ffi::c_int {
                    del_bytes(next_leader_len as colnr_T, false_0 != 0, false_0 != 0);
                    mark_col_adjust(
                        (*curwin.get()).w_cursor.lnum,
                        0 as colnr_T,
                        0 as linenr_T,
                        -(next_leader_len as colnr_T),
                        0 as ::core::ffi::c_int,
                    );
                } else if second_indent > 0 as ::core::ffi::c_int {
                    let mut indent_0: ::core::ffi::c_int =
                        getwhitecols_curline() as ::core::ffi::c_int;
                    if indent_0 > 0 as ::core::ffi::c_int {
                        del_bytes(indent_0 as colnr_T, false_0 != 0, false_0 != 0);
                        mark_col_adjust(
                            (*curwin.get()).w_cursor.lnum,
                            0 as colnr_T,
                            0 as linenr_T,
                            -(indent_0 as colnr_T),
                            0 as ::core::ffi::c_int,
                        );
                    }
                }
                (*curwin.get()).w_cursor.lnum -= 1;
                if do_join(
                    2 as size_t,
                    true_0 != 0,
                    false_0 != 0,
                    false_0 != 0,
                    false_0 != 0,
                ) == FAIL
                {
                    beep_flush();
                    break;
                } else {
                    first_par_line = false_0 != 0;
                    force_format = get_cursor_line_len() > max_len;
                }
            }
        }
        line_breakcheck();
        count -= 1;
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
