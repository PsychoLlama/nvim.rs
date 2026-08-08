//! TAB, CR and the two shift keys -- the characters that change an
//! indent.
//!
//! `ins_tab` is TAB, and it is long because 'expandtab', 'softtabstop',
//! 'vartabstop' and 'varsofttabstop' each give the key a different meaning,
//! and because inserting spaces where a tab was requires rebuilding the
//! white space run around the cursor rather than inserting at it.
//! `ins_eol` is CR/NL: `open_line` does the work, but the replace stack and
//! 'formatoptions' have to be told first.  `ins_shift` is `i_CTRL-T` and
//! `i_CTRL-D`, which add or remove one 'shiftwidth' from the current
//! line.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn ins_shift(
    mut c: ::core::ffi::c_int,
    mut lastc: ::core::ffi::c_int,
) {
    unsafe {
        if stop_arrow() == FAIL {
            return;
        }
        AppendCharToRedobuff(c);
        if c == Ctrl_D
            && (lastc == '0' as ::core::ffi::c_int || lastc == '^' as ::core::ffi::c_int)
            && (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
        {
            (*curwin.get()).w_cursor.col -= 1;
            del_char(false_0 != 0);
            if State.get() & REPLACE_FLAG != 0 {
                replace_pop_ins();
            }
            if lastc == '^' as ::core::ffi::c_int {
                old_indent.set(get_indent());
            }
            change_indent(
                INDENT_SET as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                true_0,
                true_0 != 0,
            );
        } else {
            change_indent(
                if c == Ctrl_D {
                    INDENT_DEC as ::core::ffi::c_int
                } else {
                    INDENT_INC as ::core::ffi::c_int
                },
                0 as ::core::ffi::c_int,
                true_0,
                true_0 != 0,
            );
        }
        if did_ai.get() as ::core::ffi::c_int != 0
            && *skipwhite(get_cursor_line_ptr()) as ::core::ffi::c_int != NUL
        {
            did_ai.set(false_0 != 0);
        }
        did_si.set(false_0 != 0);
        can_si.set(false_0 != 0);
        can_si_back.set(false_0 != 0);
        can_cindent.set(false_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn ins_tab() -> bool {
    unsafe {
        let mut temp: ::core::ffi::c_int = 0;
        if Insstart_blank_vcol.get() == MAXCOL as ::core::ffi::c_int
            && (*curwin.get()).w_cursor.lnum == (*Insstart.ptr()).lnum
        {
            Insstart_blank_vcol.set(get_nolist_virtcol());
        }
        if echeck_abbr(TAB + ABBR_OFF) {
            return false_0 != 0;
        }
        let mut ind: bool = inindent(0 as ::core::ffi::c_int);
        if ind {
            can_cindent.set(false_0 != 0);
        }
        if (*curbuf.get()).b_p_et == 0
            && !(p_sta.get() != 0
                && ind as ::core::ffi::c_int != 0
                && (tabstop_count((*curbuf.get()).b_p_vts_array) > 1 as ::core::ffi::c_int
                    || tabstop_count((*curbuf.get()).b_p_vts_array) == 1 as ::core::ffi::c_int
                        && tabstop_first((*curbuf.get()).b_p_vts_array)
                            != get_sw_value(curbuf.get())
                    || tabstop_count((*curbuf.get()).b_p_vts_array) == 0 as ::core::ffi::c_int
                        && (*curbuf.get()).b_p_ts != get_sw_value(curbuf.get()) as OptInt))
            && tabstop_count((*curbuf.get()).b_p_vsts_array) == 0 as ::core::ffi::c_int
            && get_sts_value() == 0 as ::core::ffi::c_int
        {
            return true_0 != 0;
        }
        if stop_arrow() == FAIL {
            return true_0 != 0;
        }
        did_ai.set(false_0 != 0);
        did_si.set(false_0 != 0);
        can_si.set(false_0 != 0);
        can_si_back.set(false_0 != 0);
        AppendToRedobuff(b"\t\0".as_ptr() as *const ::core::ffi::c_char);
        if p_sta.get() != 0 && ind as ::core::ffi::c_int != 0 {
            temp = get_sw_value(curbuf.get());
            temp -= get_nolist_virtcol() % temp;
        } else if tabstop_count((*curbuf.get()).b_p_vsts_array) > 0 as ::core::ffi::c_int
            || (*curbuf.get()).b_p_sts != 0 as OptInt
        {
            temp = tabstop_padding(
                get_nolist_virtcol(),
                get_sts_value() as OptInt,
                (*curbuf.get()).b_p_vsts_array,
            );
        } else {
            temp = tabstop_padding(
                get_nolist_virtcol(),
                (*curbuf.get()).b_p_ts,
                (*curbuf.get()).b_p_vts_array,
            );
        }
        ins_char(' ' as ::core::ffi::c_int);
        loop {
            temp -= 1;
            if temp <= 0 as ::core::ffi::c_int {
                break;
            }
            if State.get() & VREPLACE_FLAG != 0 {
                ins_char(' ' as ::core::ffi::c_int);
            } else {
                ins_str(
                    b" \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as size_t),
                );
                if State.get() & REPLACE_FLAG != 0 {
                    replace_push_nul();
                }
            }
        }
        if (*curbuf.get()).b_p_et == 0
            && (tabstop_count((*curbuf.get()).b_p_vsts_array) > 0 as ::core::ffi::c_int
                || get_sts_value() > 0 as ::core::ffi::c_int
                || p_sta.get() != 0 && ind as ::core::ffi::c_int != 0)
        {
            let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut saved_line: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut pos: pos_T = pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            };
            let mut cursor: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
            let mut want_vcol: colnr_T = 0;
            let mut vcol: colnr_T = 0;
            let mut change_col: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
            let mut save_list: ::core::ffi::c_int = (*curwin.get()).w_onebuf_opt.wo_list;
            if State.get() & VREPLACE_FLAG != 0 {
                pos = (*curwin.get()).w_cursor;
                cursor = &raw mut pos;
                saved_line = xstrnsave(get_cursor_line_ptr(), get_cursor_line_len() as size_t);
                ptr = saved_line.offset(pos.col as isize);
            } else {
                ptr = get_cursor_pos_ptr();
                cursor = &raw mut (*curwin.get()).w_cursor;
            }
            if vim_strchr(p_cpo.get(), CPO_LISTWM).is_null() {
                (*curwin.get()).w_onebuf_opt.wo_list = false_0;
            }
            let mut fpos: pos_T = (*curwin.get()).w_cursor;
            while fpos.col > 0 as ::core::ffi::c_int
                && ascii_iswhite(
                    *ptr.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                ) as ::core::ffi::c_int
                    != 0
            {
                fpos.col -= 1;
                ptr = ptr.offset(-1);
            }
            if State.get() & REPLACE_FLAG != 0
                && fpos.lnum == (*Insstart.ptr()).lnum
                && fpos.col < (*Insstart.ptr()).col
            {
                ptr = ptr.offset(((*Insstart.ptr()).col - fpos.col) as isize);
                fpos.col = (*Insstart.ptr()).col;
            }
            getvcol(
                curwin.get(),
                &raw mut fpos,
                &raw mut vcol,
                ::core::ptr::null_mut::<colnr_T>(),
                ::core::ptr::null_mut::<colnr_T>(),
            );
            getvcol(
                curwin.get(),
                cursor,
                &raw mut want_vcol,
                ::core::ptr::null_mut::<colnr_T>(),
                ::core::ptr::null_mut::<colnr_T>(),
            );
            let mut tab: *mut ::core::ffi::c_char =
                b"\t\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            let mut tab_v: int32_t = *tab as uint8_t as int32_t;
            let mut csarg: CharsizeArg = CharsizeArg::default();
            let mut cstype: CharsizeKind =
                init_charsize_arg(&mut csarg, curwin.get(), 0 as linenr_T, tab);
            while ascii_iswhite(*ptr as ::core::ffi::c_int) {
                let mut i: ::core::ffi::c_int =
                    win_charsize(cstype, vcol as ::core::ffi::c_int, tab, tab_v, &mut csarg).width;
                if vcol as ::core::ffi::c_int + i > want_vcol {
                    break;
                }
                if *ptr as ::core::ffi::c_int != TAB {
                    *ptr = TAB as ::core::ffi::c_char;
                    if change_col < 0 as ::core::ffi::c_int {
                        change_col = fpos.col as ::core::ffi::c_int;
                        if fpos.lnum == (*Insstart.ptr()).lnum && fpos.col < (*Insstart.ptr()).col {
                            (*Insstart.ptr()).col = fpos.col;
                        }
                    }
                }
                fpos.col += 1;
                ptr = ptr.offset(1);
                vcol += i;
            }
            if change_col >= 0 as ::core::ffi::c_int {
                let mut repl_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                cstype = init_charsize_arg(&mut csarg, curwin.get(), 0 as linenr_T, ptr);
                while vcol < want_vcol && *ptr as ::core::ffi::c_int == ' ' as ::core::ffi::c_int {
                    vcol += win_charsize(
                        cstype,
                        vcol as ::core::ffi::c_int,
                        ptr,
                        ' ' as ::core::ffi::c_int as uint8_t as int32_t,
                        &mut csarg,
                    )
                    .width;
                    ptr = ptr.offset(1);
                    repl_off += 1;
                }
                if vcol > want_vcol {
                    ptr = ptr.offset(-1);
                    repl_off -= 1;
                }
                fpos.col += repl_off;
                let mut i_0: ::core::ffi::c_int =
                    (*cursor).col as ::core::ffi::c_int - fpos.col as ::core::ffi::c_int;
                if i_0 > 0 as ::core::ffi::c_int {
                    if State.get() & VREPLACE_FLAG == 0 {
                        let newp_len: colnr_T =
                            (*curbuf.get()).b_ml.ml_line_textlen - i_0 as colnr_T;
                        let mut newp: *mut ::core::ffi::c_char =
                            xmalloc(newp_len as size_t) as *mut ::core::ffi::c_char;
                        let mut col: ptrdiff_t = ptr.offset_from((*curbuf.get()).b_ml.ml_line_ptr);
                        if col > 0 as ptrdiff_t {
                            memmove(
                                newp as *mut ::core::ffi::c_void,
                                ptr.offset(-(col as isize)) as *const ::core::ffi::c_void,
                                col as size_t,
                            );
                        }
                        memmove(
                            newp.offset(col as isize) as *mut ::core::ffi::c_void,
                            ptr.offset(i_0 as isize) as *const ::core::ffi::c_void,
                            (newp_len as ptrdiff_t - col) as size_t,
                        );
                        if (*curbuf.get()).b_ml.ml_flags & (ML_LINE_DIRTY | ML_ALLOCATED) != 0 {
                            xfree((*curbuf.get()).b_ml.ml_line_ptr as *mut ::core::ffi::c_void);
                        }
                        (*curbuf.get()).b_ml.ml_line_ptr = newp;
                        (*curbuf.get()).b_ml.ml_line_textlen = newp_len;
                        (*curbuf.get()).b_ml.ml_flags =
                            ((*curbuf.get()).b_ml.ml_flags | ML_LINE_DIRTY) & !ML_EMPTY;
                        inserted_bytes(
                            fpos.lnum,
                            change_col as colnr_T,
                            (*cursor).col as ::core::ffi::c_int - change_col,
                            fpos.col as ::core::ffi::c_int - change_col,
                        );
                    } else {
                        memmove(
                            ptr as *mut ::core::ffi::c_void,
                            ptr.offset(i_0 as isize) as *const ::core::ffi::c_void,
                            strlen(ptr.offset(i_0 as isize)).wrapping_add(1 as size_t),
                        );
                    }
                    if State.get() & REPLACE_FLAG != 0 && State.get() & VREPLACE_FLAG == 0 {
                        temp = i_0;
                        loop {
                            temp -= 1;
                            if temp < 0 as ::core::ffi::c_int {
                                break;
                            }
                            replace_join(repl_off);
                        }
                    }
                }
                (*cursor).col -= i_0;
                if State.get() & VREPLACE_FLAG != 0 {
                    backspace_until_column(change_col);
                    ins_bytes_len(
                        saved_line.offset(change_col as isize),
                        ((*cursor).col as ::core::ffi::c_int - change_col) as size_t,
                    );
                }
            }
            if State.get() & VREPLACE_FLAG != 0 {
                xfree(saved_line as *mut ::core::ffi::c_void);
            }
            (*curwin.get()).w_onebuf_opt.wo_list = save_list;
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn ins_eol(mut c: ::core::ffi::c_int) -> bool {
    unsafe {
        if echeck_abbr(c + ABBR_OFF) {
            return true_0 != 0;
        }
        if stop_arrow() == FAIL {
            return false_0 != 0;
        }
        undisplay_dollar();
        if State.get() & REPLACE_FLAG != 0 && State.get() & VREPLACE_FLAG == 0 {
            replace_push_nul();
        }
        if virtual_active(curwin.get()) as ::core::ffi::c_int != 0
            && (*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int
        {
            coladvance(curwin.get(), getviscol());
        }
        if revins_on.get() {
            (*curwin.get()).w_cursor.col += get_cursor_pos_len();
        }
        AppendToRedobuff(NL_STR.as_ptr());
        let mut i: bool = open_line(
            FORWARD as ::core::ffi::c_int,
            if has_format_option(FO_RET_COMS) as ::core::ffi::c_int != 0 {
                OPENLINE_DO_COM as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
            old_indent.get(),
            ::core::ptr::null_mut::<bool>(),
        );
        old_indent.set(0 as ::core::ffi::c_int);
        can_cindent.set(true_0 != 0);
        foldOpenCursor();
        return i;
    }
}
