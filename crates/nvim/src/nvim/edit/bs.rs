//! Backspace and delete: `i_BS`, `i_CTRL-W`, `i_CTRL-U` and `i_DEL`.
//!
//! `ins_bs` is one function for all three backwards forms, told apart by its
//! `mode` argument (`BACKSPACE_CHAR`, `BACKSPACE_WORD`,
//! `BACKSPACE_WORD_NOT_SPACE`, `BACKSPACE_LINE`).  Most of its length is the
//! set of things it is *not allowed* to delete: 'backspace' decides whether
//! it may cross the start of the insert, an auto-indent, or a line break;
//! Replace and Virtual Replace mode restore from the replace stack instead
//! of deleting; a prompt buffer's prompt is off limits; and joining two
//! lines has to reproduce what `J` would have done to the indent.
//! `ins_del` is `<Del>`, which is the forward case and much shorter.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn ins_del() {
    unsafe {
        if stop_arrow() == FAIL {
            return;
        }
        if gchar_cursor() == NUL {
            let temp: ::core::ffi::c_int = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
            if !can_bs(BS_EOL)
                || do_join(
                    2 as size_t,
                    false_0 != 0,
                    true_0 != 0,
                    false_0 != 0,
                    false_0 != 0,
                ) == FAIL
            {
                vim_beep(kOptBoFlagBackspace as ::core::ffi::c_int as ::core::ffi::c_uint);
            } else {
                (*curwin.get()).w_cursor.col = temp as colnr_T;
                if State.get() & VREPLACE_FLAG != 0
                    && orig_line_count.get() > (*curbuf.get()).b_ml.ml_line_count
                {
                    orig_line_count.set((*curbuf.get()).b_ml.ml_line_count);
                }
            }
        } else if del_char(false_0 != 0) == FAIL {
            vim_beep(kOptBoFlagBackspace as ::core::ffi::c_int as ::core::ffi::c_uint);
        }
        did_ai.set(false_0 != 0);
        did_si.set(false_0 != 0);
        can_si.set(false_0 != 0);
        can_si_back.set(false_0 != 0);
        AppendCharToRedobuff(K_DEL);
    }
}

pub(crate) unsafe extern "C" fn ins_bs(
    mut c: ::core::ffi::c_int,
    mut mode: ::core::ffi::c_int,
    mut inserted_space_p: *mut ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut cc: ::core::ffi::c_int = 0;
        let mut temp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut did_backspace: bool = false_0 != 0;
        let mut call_fix_indent: bool = false_0 != 0;
        if buf_is_empty(curbuf.get()) as ::core::ffi::c_int != 0
            || !revins_on.get()
                && ((*curwin.get()).w_cursor.lnum == 1 as linenr_T
                    && (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int
                    || !can_bs(BS_START)
                        && (arrow_used.get() as ::core::ffi::c_int != 0
                            && !bt_prompt(curbuf.get())
                            || (*curwin.get()).w_cursor.lnum == (*Insstart_orig.ptr()).lnum
                                && (*curwin.get()).w_cursor.col <= (*Insstart_orig.ptr()).col)
                    || !can_bs(BS_INDENT)
                        && !arrow_used.get()
                        && ai_col.get() > 0 as ::core::ffi::c_int
                        && (*curwin.get()).w_cursor.col <= ai_col.get()
                    || !can_bs(BS_EOL) && (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int)
        {
            vim_beep(kOptBoFlagBackspace as ::core::ffi::c_int as ::core::ffi::c_uint);
            return false_0 != 0;
        }
        if stop_arrow() == FAIL {
            return false_0 != 0;
        }
        let mut in_indent: bool = inindent(0 as ::core::ffi::c_int);
        if in_indent {
            can_cindent.set(false_0 != 0);
        }
        end_comment_pending.set(NUL);
        if revins_on.get() {
            inc_cursor();
        }
        if (*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int {
            if mode == BACKSPACE_CHAR {
                (*curwin.get()).w_cursor.coladd -= 1;
                return true_0 != 0;
            }
            if mode == BACKSPACE_WORD {
                (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
                return true_0 != 0;
            }
            (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
        }
        if (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int {
            let mut lnum: linenr_T = (*Insstart.ptr()).lnum;
            if (*curwin.get()).w_cursor.lnum == lnum || revins_on.get() as ::core::ffi::c_int != 0 {
                if u_save(
                    (*curwin.get()).w_cursor.lnum - 2 as linenr_T,
                    (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
                ) == FAIL
                {
                    return false_0 != 0;
                }
                (*Insstart.ptr()).lnum -= 1;
                (*Insstart.ptr()).col = ml_get_len((*Insstart.ptr()).lnum);
            }
            cc = -1 as ::core::ffi::c_int;
            if State.get() & REPLACE_FLAG != 0 {
                cc = replace_pop_if_nul();
            }
            if State.get() & REPLACE_FLAG != 0 && (*curwin.get()).w_cursor.lnum <= lnum {
                dec_cursor();
            } else {
                if State.get() & VREPLACE_FLAG == 0
                    || (*curwin.get()).w_cursor.lnum > orig_line_count.get()
                {
                    temp = gchar_cursor();
                    (*curwin.get()).w_cursor.lnum -= 1;
                    if has_format_option(FO_AUTO) as ::core::ffi::c_int != 0
                        && has_format_option(FO_WHITE_PAR) as ::core::ffi::c_int != 0
                    {
                        let mut ptr: *const ::core::ffi::c_char =
                            ml_get_buf(curbuf.get(), (*curwin.get()).w_cursor.lnum);
                        let mut len: ::core::ffi::c_int = get_cursor_line_len();
                        if len > 0 as ::core::ffi::c_int
                            && *ptr.offset((len - 1 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int
                                == ' ' as ::core::ffi::c_int
                        {
                            let mut newp: *mut ::core::ffi::c_char = xmemdupz(
                                ptr as *const ::core::ffi::c_void,
                                (len - 1 as ::core::ffi::c_int) as size_t,
                            )
                                as *mut ::core::ffi::c_char;
                            if (*curbuf.get()).b_ml.ml_flags & (ML_LINE_DIRTY | ML_ALLOCATED) != 0 {
                                xfree((*curbuf.get()).b_ml.ml_line_ptr as *mut ::core::ffi::c_void);
                            }
                            (*curbuf.get()).b_ml.ml_line_ptr = newp;
                            (*curbuf.get()).b_ml.ml_line_textlen -= 1;
                            (*curbuf.get()).b_ml.ml_flags |= ML_LINE_DIRTY;
                        }
                    }
                    do_join(
                        2 as size_t,
                        false_0 != 0,
                        false_0 != 0,
                        false_0 != 0,
                        false_0 != 0,
                    );
                    if temp == NUL && gchar_cursor() != NUL {
                        inc_cursor();
                    }
                } else {
                    dec_cursor();
                }
                if State.get() & REPLACE_FLAG != 0 {
                    let mut oldState: ::core::ffi::c_int = State.get();
                    State.set(MODE_NORMAL);
                    while cc > 0 as ::core::ffi::c_int {
                        let mut save_col: colnr_T = (*curwin.get()).w_cursor.col;
                        mb_replace_pop_ins();
                        (*curwin.get()).w_cursor.col = save_col;
                        cc = replace_pop_if_nul();
                    }
                    replace_pop_ins();
                    State.set(oldState);
                }
            }
            did_ai.set(false_0 != 0);
        } else {
            if revins_on.get() {
                dec_cursor();
            }
            let mut mincol: colnr_T = 0 as colnr_T;
            if mode == BACKSPACE_LINE
                && ((*curbuf.get()).b_p_ai != 0 || cindent_on() as ::core::ffi::c_int != 0)
                && !revins_on.get()
            {
                let mut save_col_0: colnr_T = (*curwin.get()).w_cursor.col;
                beginline(BL_WHITE);
                if (*curwin.get()).w_cursor.col < save_col_0 {
                    mincol = (*curwin.get()).w_cursor.col;
                    call_fix_indent = true_0 != 0;
                }
                (*curwin.get()).w_cursor.col = save_col_0;
            }
            if mode == BACKSPACE_CHAR
                && (p_sta.get() != 0 && in_indent as ::core::ffi::c_int != 0
                    || (get_sts_value() != 0 as ::core::ffi::c_int
                        || tabstop_count((*curbuf.get()).b_p_vsts_array) != 0)
                        && (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
                        && (*get_cursor_pos_ptr().offset(-(1 as ::core::ffi::c_int as isize))
                            as ::core::ffi::c_int
                            == TAB
                            || *get_cursor_pos_ptr().offset(-(1 as ::core::ffi::c_int as isize))
                                as ::core::ffi::c_int
                                == ' ' as ::core::ffi::c_int
                                && (*inserted_space_p == 0
                                    || arrow_used.get() as ::core::ffi::c_int != 0)))
            {
                *inserted_space_p = false_0;
                let use_ts: bool = (*curwin.get()).w_onebuf_opt.wo_list == 0
                    || (*curwin.get()).w_p_lcs_chars.tab1 != 0;
                let line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
                let cursor_ptr: *mut ::core::ffi::c_char =
                    line.offset((*curwin.get()).w_cursor.col as isize);
                let mut vcol: colnr_T = 0 as colnr_T;
                let mut space_vcol: colnr_T = 0 as colnr_T;
                let mut sci: StrCharInfo = utf_ptr2StrCharInfo(line);
                let mut space_sci: StrCharInfo = sci;
                let mut prev_space: bool = false_0 != 0;
                while sci.ptr < cursor_ptr {
                    let mut cur_space: bool = ascii_iswhite(sci.chr.value as ::core::ffi::c_int);
                    if !prev_space && cur_space as ::core::ffi::c_int != 0 {
                        space_sci = sci;
                        space_vcol = vcol;
                    }
                    vcol += charsize_nowrap(curbuf.get(), sci.ptr, use_ts, vcol, sci.chr.value);
                    sci = utfc_next(sci);
                    prev_space = cur_space;
                }
                let mut want_vcol: colnr_T = if vcol > 0 as ::core::ffi::c_int {
                    vcol - 1 as colnr_T
                } else {
                    0 as colnr_T
                };
                if p_sta.get() != 0 && in_indent as ::core::ffi::c_int != 0 {
                    want_vcol -= want_vcol as ::core::ffi::c_int % get_sw_value(curbuf.get());
                } else {
                    want_vcol =
                        tabstop_start(want_vcol, get_sts_value(), (*curbuf.get()).b_p_vsts_array);
                }
                loop {
                    let mut size: ::core::ffi::c_int = charsize_nowrap(
                        curbuf.get(),
                        space_sci.ptr,
                        use_ts,
                        space_vcol,
                        space_sci.chr.value,
                    );
                    if space_vcol as ::core::ffi::c_int + size > want_vcol {
                        break;
                    }
                    space_vcol += size;
                    space_sci = utfc_next(space_sci);
                }
                let want_col: colnr_T = space_sci.ptr.offset_from(line) as colnr_T;
                while (*curwin.get()).w_cursor.col > want_col {
                    dec_cursor();
                    if State.get() & REPLACE_FLAG != 0 {
                        if (*curwin.get()).w_cursor.lnum != (*Insstart.ptr()).lnum
                            || (*curwin.get()).w_cursor.col >= (*Insstart.ptr()).col
                        {
                            replace_do_bs(-1 as ::core::ffi::c_int);
                        }
                    } else {
                        del_char(false_0 != 0);
                    }
                }
                while space_vcol < want_vcol {
                    if (*curwin.get()).w_cursor.lnum == (*Insstart_orig.ptr()).lnum
                        && (*curwin.get()).w_cursor.col < (*Insstart_orig.ptr()).col
                    {
                        (*Insstart_orig.ptr()).col = (*curwin.get()).w_cursor.col;
                    }
                    if State.get() & VREPLACE_FLAG != 0 {
                        ins_char(' ' as ::core::ffi::c_int);
                    } else {
                        ins_str(
                            b" \0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                .wrapping_sub(1 as size_t),
                        );
                        if State.get() & REPLACE_FLAG != 0 {
                            replace_push_nul();
                        }
                    }
                    space_vcol += 1;
                }
            } else {
                let mut cclass: ::core::ffi::c_int = mb_get_class(get_cursor_pos_ptr());
                loop {
                    if !revins_on.get() {
                        dec_cursor();
                    }
                    cc = gchar_cursor();
                    let mut prev_cclass: ::core::ffi::c_int = cclass;
                    cclass = mb_get_class(get_cursor_pos_ptr());
                    if mode == BACKSPACE_WORD && !ascii_isspace(cc) {
                        mode = BACKSPACE_WORD_NOT_SPACE;
                        temp = vim_iswordc(cc) as ::core::ffi::c_int;
                    } else if mode == BACKSPACE_WORD_NOT_SPACE
                        && (ascii_isspace(cc) as ::core::ffi::c_int != 0
                            || vim_iswordc(cc) as ::core::ffi::c_int != temp
                            || prev_cclass != cclass)
                    {
                        if !revins_on.get() {
                            inc_cursor();
                        } else if State.get() & REPLACE_FLAG != 0 {
                            dec_cursor();
                        }
                        break;
                    }
                    if State.get() & REPLACE_FLAG != 0 {
                        replace_do_bs(-1 as ::core::ffi::c_int);
                    } else {
                        let mut has_composing: bool = false_0 != 0;
                        if p_deco.get() != 0 {
                            let mut p0: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
                            has_composing = utf_composinglike(
                                p0,
                                p0.offset(utf_ptr2len(p0) as isize),
                                ::core::ptr::null_mut::<GraphemeState>(),
                            );
                        }
                        del_char(false_0 != 0);
                        if has_composing {
                            inc_cursor();
                        }
                        if revins_chars.get() != 0 {
                            (*revins_chars.ptr()) -= 1;
                            (*revins_legal.ptr()) += 1;
                        }
                        if revins_on.get() as ::core::ffi::c_int != 0 && gchar_cursor() == NUL {
                            break;
                        }
                    }
                    if mode == BACKSPACE_CHAR {
                        break;
                    }
                    if !(revins_on.get() as ::core::ffi::c_int != 0
                        || (*curwin.get()).w_cursor.col > mincol
                            && (can_bs(BS_NOSTOP) as ::core::ffi::c_int != 0
                                || ((*curwin.get()).w_cursor.lnum != (*Insstart_orig.ptr()).lnum
                                    || (*curwin.get()).w_cursor.col != (*Insstart_orig.ptr()).col)))
                    {
                        break;
                    }
                }
            }
            did_backspace = true_0 != 0;
        }
        did_si.set(false_0 != 0);
        can_si.set(false_0 != 0);
        can_si_back.set(false_0 != 0);
        if (*curwin.get()).w_cursor.col <= 1 as ::core::ffi::c_int {
            did_ai.set(false_0 != 0);
        }
        if call_fix_indent {
            fix_indent();
        }
        AppendCharToRedobuff(c);
        if (*curwin.get()).w_cursor.lnum == (*Insstart_orig.ptr()).lnum
            && (*curwin.get()).w_cursor.col < (*Insstart_orig.ptr()).col
        {
            (*Insstart_orig.ptr()).col = (*curwin.get()).w_cursor.col;
        }
        if !vim_strchr(p_cpo.get(), CPO_BACKSPACE).is_null()
            && dollar_vcol.get() == -1 as ::core::ffi::c_int
        {
            dollar_vcol.set((*curwin.get()).w_virtcol);
        }
        if did_backspace {
            foldOpenCursor();
        }
        return did_backspace;
    }
}
