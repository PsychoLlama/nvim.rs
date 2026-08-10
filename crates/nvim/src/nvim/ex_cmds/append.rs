//! Adding lines -- `:append`, `:insert`, `:change` and `:z`.
//!
//! `ex_append` reads lines from the command line's input stream until a lone
//! `.`, honouring 'autoindent' (`append_indent`) and the `:change` variant that
//! deletes the range first.  `:z` is the paging command: print a window of
//! lines around a position, with the `+`/`-`/`=`/`.`/`^` forms picking which
//! window and `:z#` numbering it.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

static append_indent: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);

pub unsafe fn ex_append(mut eap: *mut exarg_T) {
    unsafe {
        let mut theline: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut did_undo: bool = false_0 != 0;
        let mut lnum: linenr_T = (*eap).line2;
        let mut indent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut empty: bool = (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0;
        if (*eap).forceit != 0 {
            (*curbuf.get()).b_p_ai = ((*curbuf.get()).b_p_ai == 0) as ::core::ffi::c_int;
        }
        if (*eap).cmdidx as ::core::ffi::c_int != CMD_change as ::core::ffi::c_int
            && (*curbuf.get()).b_p_ai != 0
            && lnum > 0 as linenr_T
        {
            append_indent.set(get_indent_lnum(lnum));
        }
        if (*eap).cmdidx as ::core::ffi::c_int != CMD_append as ::core::ffi::c_int {
            lnum -= 1;
        }
        if empty as ::core::ffi::c_int != 0 && lnum == 1 as linenr_T {
            lnum = 0 as ::core::ffi::c_int as linenr_T;
        }
        State.set(MODE_INSERT);
        if (*curbuf.get()).b_p_iminsert == B_IMODE_LMAP as OptInt {
            (*State.ptr()) |= MODE_LANGMAP;
        }
        loop {
            msg_scroll.set(true_0);
            need_wait_return.set(false_0 != 0);
            if (*curbuf.get()).b_p_ai != 0 {
                if append_indent.get() >= 0 as ::core::ffi::c_int {
                    indent = append_indent.get();
                    append_indent.set(-1 as ::core::ffi::c_int);
                } else if lnum > 0 as linenr_T {
                    indent = get_indent_lnum(lnum);
                }
            }
            if *(*eap).arg as ::core::ffi::c_int == '|' as ::core::ffi::c_int {
                theline = xstrdup((*eap).arg.offset(1 as ::core::ffi::c_int as isize));
                *(*eap).arg = NUL as ::core::ffi::c_char;
            } else if (*eap).ea_getline.is_none() {
                if (*eap).nextcmd.is_null() {
                    break;
                }
                p = vim_strchr((*eap).nextcmd, NL);
                if p.is_null() {
                    p = (*eap).nextcmd.add(strlen((*eap).nextcmd));
                }
                theline = xmemdupz(
                    (*eap).nextcmd as *const ::core::ffi::c_void,
                    p.offset_from((*eap).nextcmd) as size_t,
                ) as *mut ::core::ffi::c_char;
                if *p as ::core::ffi::c_int != NUL {
                    p = p.offset(1);
                } else {
                    p = ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                (*eap).nextcmd = p;
            } else {
                let mut save_State: ::core::ffi::c_int = State.get();
                State.set(MODE_CMDLINE);
                theline = (*eap).ea_getline.expect("non-null function pointer")(
                    if (*(*eap).cstack).cs_looplevel > 0 as ::core::ffi::c_int {
                        -1 as ::core::ffi::c_int
                    } else {
                        NUL
                    },
                    (*eap).cookie,
                    indent,
                    true_0 != 0,
                );
                State.set(save_State);
            }
            lines_left.set(Rows.get() - 1 as ::core::ffi::c_int);
            if theline.is_null() {
                break;
            }
            let mut vcol: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            p = theline;
            while indent > vcol {
                if *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int {
                    vcol += 1;
                } else {
                    if *p as ::core::ffi::c_int != TAB {
                        break;
                    }
                    vcol += 8 as ::core::ffi::c_int - vcol % 8 as ::core::ffi::c_int;
                }
                p = p.offset(1);
            }
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                || !did_undo
                    && u_save(
                        lnum,
                        lnum + 1 as linenr_T
                            + (if empty as ::core::ffi::c_int != 0 {
                                1 as linenr_T
                            } else {
                                0 as linenr_T
                            }),
                    ) == FAIL
            {
                xfree(theline as *mut ::core::ffi::c_void);
                break;
            } else {
                if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
                    *theline.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
                }
                did_undo = true_0 != 0;
                ml_append(lnum, theline, 0 as colnr_T, false_0 != 0);
                if empty {
                    appended_lines(lnum, 1 as linenr_T);
                } else {
                    appended_lines_mark(lnum, 1 as ::core::ffi::c_int);
                }
                xfree(theline as *mut ::core::ffi::c_void);
                lnum += 1;
                if empty {
                    ml_delete(2 as linenr_T);
                    empty = false_0 != 0;
                }
            }
        }
        State.set(MODE_NORMAL);
        ui_cursor_shape();
        if (*eap).forceit != 0 {
            (*curbuf.get()).b_p_ai = ((*curbuf.get()).b_p_ai == 0) as ::core::ffi::c_int;
        }
        if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            (*curbuf.get()).b_op_start.lnum = if (*eap).line2 < (*curbuf.get()).b_ml.ml_line_count {
                (*eap).line2 + 1 as linenr_T
            } else {
                (*curbuf.get()).b_ml.ml_line_count
            };
            if (*eap).cmdidx as ::core::ffi::c_int != CMD_append as ::core::ffi::c_int {
                (*curbuf.get()).b_op_start.lnum -= 1;
            }
            (*curbuf.get()).b_op_end.lnum = if (*eap).line2 < lnum {
                lnum
            } else {
                (*curbuf.get()).b_op_start.lnum
            };
            (*curbuf.get()).b_op_end.col = 0 as ::core::ffi::c_int as colnr_T;
            (*curbuf.get()).b_op_start.col = (*curbuf.get()).b_op_end.col;
        }
        (*curwin.get()).w_cursor.lnum = lnum;
        check_cursor_lnum(curwin.get());
        beginline(BL_SOL as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int);
        need_wait_return.set(false_0 != 0);
        ex_no_reprint.set(true_0 != 0);
    }
}

pub unsafe fn ex_change(mut eap: *mut exarg_T) {
    unsafe {
        let mut lnum: linenr_T = 0;
        if (*eap).line2 >= (*eap).line1
            && u_save((*eap).line1 - 1 as linenr_T, (*eap).line2 + 1 as linenr_T) == FAIL
        {
            return;
        }
        if if (*eap).forceit != 0 {
            ((*curbuf.get()).b_p_ai == 0) as ::core::ffi::c_int
        } else {
            (*curbuf.get()).b_p_ai
        } != 0
        {
            append_indent.set(get_indent_lnum((*eap).line1));
        }
        lnum = (*eap).line2;
        while lnum >= (*eap).line1 {
            if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
                break;
            }
            ml_delete((*eap).line1);
            lnum -= 1;
        }
        check_cursor_lnum(curwin.get());
        deleted_lines_mark(
            (*eap).line1,
            (*eap).line2 as ::core::ffi::c_int - lnum as ::core::ffi::c_int,
        );
        (*eap).line2 = (*eap).line1;
        ex_append(eap);
    }
}

pub unsafe fn ex_z(mut eap: *mut exarg_T) {
    unsafe {
        let mut bigness: int64_t = 0;
        let mut minus: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut start: linenr_T = 0;
        let mut end: linenr_T = 0;
        let mut curs: linenr_T = 0;
        let mut lnum: linenr_T = (*eap).line2;
        if (*eap).forceit != 0 {
            bigness = (Rows.get() - 1 as ::core::ffi::c_int) as int64_t;
        } else if firstwin.get() == lastwin.get() {
            bigness = ((*curwin.get()).w_onebuf_opt.wo_scr * 2 as OptInt) as int64_t;
        } else {
            bigness = ((*curwin.get()).w_view_height - 3 as ::core::ffi::c_int) as int64_t;
        }
        bigness = if bigness > 1 as int64_t {
            bigness
        } else {
            1 as int64_t
        };
        let mut x: *mut ::core::ffi::c_char = (*eap).arg;
        let mut kind: *mut ::core::ffi::c_char = x;
        if *kind as ::core::ffi::c_int == '-' as ::core::ffi::c_int
            || *kind as ::core::ffi::c_int == '+' as ::core::ffi::c_int
            || *kind as ::core::ffi::c_int == '=' as ::core::ffi::c_int
            || *kind as ::core::ffi::c_int == '^' as ::core::ffi::c_int
            || *kind as ::core::ffi::c_int == '.' as ::core::ffi::c_int
        {
            x = x.offset(1);
        }
        while *x as ::core::ffi::c_int == '-' as ::core::ffi::c_int
            || *x as ::core::ffi::c_int == '+' as ::core::ffi::c_int
        {
            x = x.offset(1);
        }
        if *x as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            if !ascii_isdigit(*x as ::core::ffi::c_int) {
                emsg(gettext(
                    (e_non_numeric_argument_to_z.ptr() as *const _) as *const ::core::ffi::c_char,
                ));
                return;
            }
            bigness = atol(x) as int64_t;
            if bigness > (2 as linenr_T * (*curbuf.get()).b_ml.ml_line_count) as int64_t
                || bigness < 0 as int64_t
            {
                bigness = (2 as linenr_T * (*curbuf.get()).b_ml.ml_line_count) as int64_t;
            }
            p_window.set(bigness as ::core::ffi::c_int as OptInt);
            if *kind as ::core::ffi::c_int == '=' as ::core::ffi::c_int {
                bigness += 2 as int64_t;
            }
        }
        if *kind as ::core::ffi::c_int == '-' as ::core::ffi::c_int
            || *kind as ::core::ffi::c_int == '+' as ::core::ffi::c_int
        {
            x = kind.offset(1 as ::core::ffi::c_int as isize);
            while *x as ::core::ffi::c_int == *kind as ::core::ffi::c_int {
                x = x.offset(1);
            }
        }
        match *kind as ::core::ffi::c_int {
            45 => {
                start =
                    lnum - bigness as linenr_T * x.offset_from(kind) as linenr_T + 1 as linenr_T;
                end = start + bigness as linenr_T - 1 as linenr_T;
                curs = end;
            }
            61 => {
                start =
                    lnum - (bigness as linenr_T + 1 as linenr_T) / 2 as linenr_T + 1 as linenr_T;
                end = lnum + (bigness as linenr_T + 1 as linenr_T) / 2 as linenr_T - 1 as linenr_T;
                curs = lnum;
                minus = 1 as ::core::ffi::c_int;
            }
            94 => {
                start = lnum - bigness as linenr_T * 2 as linenr_T;
                end = lnum - bigness as linenr_T;
                curs = lnum - bigness as linenr_T;
            }
            46 => {
                start =
                    lnum - (bigness as linenr_T + 1 as linenr_T) / 2 as linenr_T + 1 as linenr_T;
                end = lnum + (bigness as linenr_T + 1 as linenr_T) / 2 as linenr_T - 1 as linenr_T;
                curs = end;
            }
            _ => {
                start = lnum;
                if *kind as ::core::ffi::c_int == '+' as ::core::ffi::c_int {
                    start = (start as ::core::ffi::c_int
                        + (bigness as linenr_T * (x.offset_from(kind) - 1_isize) as linenr_T
                            + 1 as linenr_T) as ::core::ffi::c_int)
                        as linenr_T;
                } else if (*eap).addr_count == 0 as ::core::ffi::c_int {
                    start += 1;
                }
                end = start + bigness as linenr_T - 1 as linenr_T;
                curs = end;
            }
        }
        start = if start > 1 as linenr_T {
            start
        } else {
            1 as linenr_T
        };
        end = if end < (*curbuf.get()).b_ml.ml_line_count {
            end
        } else {
            (*curbuf.get()).b_ml.ml_line_count
        };
        curs = if (if curs > 1 as linenr_T {
            curs
        } else {
            1 as linenr_T
        }) < (*curbuf.get()).b_ml.ml_line_count
        {
            if curs > 1 as linenr_T {
                curs
            } else {
                1 as linenr_T
            }
        } else {
            (*curbuf.get()).b_ml.ml_line_count
        };
        let mut i: linenr_T = start;
        while i <= end {
            if minus != 0 && i == lnum {
                msg_putchar('\n' as ::core::ffi::c_int);
                let mut j: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                while j < Columns.get() {
                    msg_putchar('-' as ::core::ffi::c_int);
                    j += 1;
                }
            }
            print_line(
                i,
                (*eap).flags & EXFLAG_NR != 0,
                (*eap).flags & EXFLAG_LIST != 0,
                i == start,
            );
            if minus != 0 && i == lnum {
                msg_putchar('\n' as ::core::ffi::c_int);
                let mut j_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                while j_0 < Columns.get() {
                    msg_putchar('-' as ::core::ffi::c_int);
                    j_0 += 1;
                }
            }
            i += 1;
        }
        if (*curwin.get()).w_cursor.lnum != curs {
            (*curwin.get()).w_cursor.lnum = curs;
            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        }
        ex_no_reprint.set(true_0 != 0);
    }
}
