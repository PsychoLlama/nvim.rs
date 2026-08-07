//! `gq` and `gw`: reflowing whole paragraphs.
//!
//! [`format_lines`] is the engine -- walk the range, decide where each
//! paragraph ends, join it into one line and let `insertchar` re-wrap it --
//! and [`op_format`] the operator around it.  [`fex_format`] is the
//! 'formatexpr' escape hatch [`op_formatexpr`] tries first.

use super::*;
use crate::src::nvim::ascii::ascii_isspace;
use crate::src::nvim::change::del_bytes;
use crate::src::nvim::charset::getwhitecols_curline;
use crate::src::nvim::cursor::{
    check_cursor, coladvance, dec_cursor, gchar_cursor, get_cursor_line_len, get_cursor_line_ptr,
};
use crate::src::nvim::drawscreen::{UPD_INVERTED, redraw_curbuf_later};
use crate::src::nvim::edit::{beginline, insertchar};
use crate::src::nvim::eval::eval_to_number;
use crate::src::nvim::eval::vars::{set_vim_var_char, set_vim_var_nr, set_vim_var_string};
use crate::src::nvim::getchar::beep_flush;
use crate::src::nvim::indent::{
    get_expr_indent, get_indent, get_indent_lnum, get_lisp_indent, get_number_indent, set_indent,
};
use crate::src::nvim::indent_c::{cindent_on, get_c_indent};
use crate::src::nvim::main::{
    State, cmdmod, curbuf, current_sctx, curtab, curwin, firstwin, got_int, p_smd, sandbox,
    saved_cursor,
};
use crate::src::nvim::mark::mark_col_adjust;
use crate::src::nvim::memline::ml_get;
use crate::src::nvim::memory::{xfree, xstrdup};
use crate::src::nvim::message::msgmore;
use crate::src::nvim::ops::do_join;
use crate::src::nvim::option::was_set_insecurely;
use crate::src::nvim::options::kOptFormatexpr;
use crate::src::nvim::os::input::line_breakcheck;
use crate::src::nvim::os::libc::strncmp;
use crate::src::nvim::pos::MAXCOL;
use crate::src::nvim::search::check_linecomment;
use crate::src::nvim::state::{MODE_INSERT, MODE_NORMAL};
use crate::src::nvim::types::{
    CMOD_LOCKMARKS, INSCHAR_COM_LIST, INSCHAR_DO_COM, INSCHAR_FORMAT, INSCHAR_NO_FEX, VV_CHAR,
    VV_COUNT, VV_LNUM, colnr_T, linenr_T, oparg_T, ptrdiff_t, sctx_T, size_t, varnumber_T, win_T,
};
use crate::src::nvim::ui::ui_cursor_shape;
use crate::src::nvim::undo::{u_save, u_save_cursor};

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
        op_format(oap, false);
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
    let mut r: ::core::ffi::c_int = eval_to_number(fex, true) as ::core::ffi::c_int;
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
    let mut prev_is_end_par: bool = false;
    let mut next_is_start_par: bool = false;
    let mut leader_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut next_leader_len: ::core::ffi::c_int = 0;
    let mut leader_flags: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut next_leader_flags: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut advance: bool = true;
    let mut second_indent: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut first_par_line: bool = true;
    let mut smd_save: ::core::ffi::c_int = 0;
    let mut count: ::core::ffi::c_long = 0;
    let mut need_set_indent: bool = true;
    let mut first_line: linenr_T = (*curwin.get()).w_cursor.lnum;
    let mut force_format: bool = false;
    let old_State: ::core::ffi::c_int = State.get();
    let max_len: ::core::ffi::c_int = comp_textwidth(true) * 3 as ::core::ffi::c_int;
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
        is_not_par = true;
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
            next_is_not_par = true;
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
        advance = true;
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
                    || strncmp(next_leader_flags, c"://".as_ptr(), 3 as size_t)
                        != 0 as ::core::ffi::c_int
                    || check_linecomment(get_cursor_line_ptr()) == MAXCOL as ::core::ffi::c_int
                {
                    is_end_par = true;
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
                    first_par_line = true;
                }
                force_format = false;
            }
            if !is_end_par {
                advance = false;
                (*curwin.get()).w_cursor.lnum += 1;
                (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                if line_count < 0 as linenr_T && u_save_cursor() == FAIL {
                    break;
                }
                if next_leader_len > 0 as ::core::ffi::c_int {
                    del_bytes(next_leader_len as colnr_T, false, false);
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
                        del_bytes(indent_0 as colnr_T, false, false);
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
                if do_join(2 as size_t, true, false, false, false) == FAIL {
                    beep_flush();
                    break;
                } else {
                    first_par_line = false;
                    force_format = get_cursor_line_len() > max_len;
                }
            }
        }
        line_breakcheck();
        count -= 1;
    }
}
