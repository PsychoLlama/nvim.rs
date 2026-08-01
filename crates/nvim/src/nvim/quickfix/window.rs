//! The quickfix window.
//!
//! [`ex_copen`]/[`ex_cwindow`] open it, [`qf_open_new_cwindow`] creates it
//! with the right options ([`qf_set_cwindow_options`]), and
//! [`qf_fill_buffer`] writes one buffer line per entry —
//! [`call_qftf_func`] first, if `'quickfixtextfunc'` is set.
//! [`qf_update_buffer`] keeps the buffer in step with the list.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe fn ex_cwindow(mut eap: *mut exarg_T) {
    unsafe {
        let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
        qi = qf_cmd_get_stack(eap, true_0 != 0);
        if qi.is_null() {
            return;
        }
        let mut qfl: *mut qf_list_T = qf_get_curlist(qi);
        let mut win: *mut win_T = qf_find_win(qi);
        if qf_stack_empty(qi) as ::core::ffi::c_int != 0
            || (*qfl).qf_nonevalid as ::core::ffi::c_int != 0
            || qf_list_empty(qfl) as ::core::ffi::c_int != 0
        {
            if !win.is_null() {
                ex_cclose(eap);
            }
        } else if win.is_null() {
            ex_copen(eap);
        }
    }
}

pub unsafe fn ex_cclose(mut eap: *mut exarg_T) {
    unsafe {
        let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
        qi = qf_cmd_get_stack(eap, false_0 != 0);
        if qi.is_null() {
            return;
        }
        let mut win: *mut win_T = qf_find_win(qi);
        if !win.is_null() {
            win_close(win, false_0 != 0, false_0 != 0);
        }
    }
}

pub(crate) unsafe extern "C" fn qf_goto_cwindow(
    mut qi: *const qf_info_T,
    mut resize: bool,
    mut sz: ::core::ffi::c_int,
    mut vertsplit: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let win: *mut win_T = qf_find_win(qi);
        if win.is_null() {
            return FAIL;
        }
        win_goto(win);
        if resize {
            if vertsplit {
                if sz != (*win).w_width {
                    win_setwidth(sz);
                }
            } else if sz != (*win).w_height
                && (*win).w_height
                    + (*win).w_hsep_height
                    + (*win).w_status_height
                    + tabline_height()
                    < cmdline_row.get()
            {
                win_setheight(sz);
            }
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn qf_set_cwindow_options() {
    unsafe {
        set_option_value_give_err(
            kOptSwapfile,
            OptVal {
                type_0: kOptValTypeBoolean,
                data: OptValData { boolean: kFalse },
            },
            OPT_LOCAL as ::core::ffi::c_int,
        );
        set_option_value_give_err(
            kOptBuftype,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: String_0 {
                        data: b"quickfix\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 9]>()
                            .wrapping_sub(1 as size_t),
                    },
                },
            },
            OPT_LOCAL as ::core::ffi::c_int,
        );
        set_option_value_give_err(
            kOptBufhidden,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: String_0 {
                        data: b"hide\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                            .wrapping_sub(1 as size_t),
                    },
                },
            },
            OPT_LOCAL as ::core::ffi::c_int,
        );
        (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
        (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
        (*curwin.get()).w_onebuf_opt.wo_diff = false_0;
        set_option_value_give_err(
            kOptFoldmethod,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: String_0 {
                        data: b"manual\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                            .wrapping_sub(1 as size_t),
                    },
                },
            },
            OPT_LOCAL as ::core::ffi::c_int,
        );
    }
}

pub(crate) unsafe extern "C" fn qf_open_new_cwindow(
    mut qi: *mut qf_info_T,
    mut height: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut oldwin: *mut win_T = curwin.get();
        let prevtab: *const tabpage_T = curtab.get();
        let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let qf_buf: *const buf_T = qf_find_buf(qi);
        let win: *mut win_T = curwin.get();
        if (*cmdmod.ptr()).cmod_split == 0 as ::core::ffi::c_int {
            flags = if (*qi).qfl_type as ::core::ffi::c_uint
                == QFLT_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                WSP_BOT as ::core::ffi::c_int
            } else {
                WSP_BELOW as ::core::ffi::c_int
            };
        }
        flags |= WSP_NEWLOC as ::core::ffi::c_int;
        if (*qi).qfl_type as ::core::ffi::c_uint
            == QFLT_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            flags |= WSP_QUICKFIX as ::core::ffi::c_int;
        }
        if win_split(height, flags) == FAIL {
            return FAIL;
        }
        (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
        (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
        if (*qi).qfl_type as ::core::ffi::c_uint
            == QFLT_LOCATION as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*curwin.get()).w_llist_ref = qi;
            (*qi).qf_refcount += 1;
        }
        if oldwin != curwin.get() {
            oldwin = ::core::ptr::null_mut::<win_T>();
        }
        if !qf_buf.is_null() {
            if do_ecmd(
                (*qf_buf).handle as ::core::ffi::c_int,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<exarg_T>(),
                ECMD_ONE as ::core::ffi::c_int as linenr_T,
                ECMD_HIDE as ::core::ffi::c_int
                    + ECMD_OLDBUF as ::core::ffi::c_int
                    + ECMD_NOWINENTER as ::core::ffi::c_int,
                oldwin,
            ) == FAIL
            {
                return FAIL;
            }
        } else {
            if do_ecmd(
                0 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<exarg_T>(),
                ECMD_ONE as ::core::ffi::c_int as linenr_T,
                ECMD_HIDE as ::core::ffi::c_int + ECMD_NOWINENTER as ::core::ffi::c_int,
                oldwin,
            ) == FAIL
            {
                return FAIL;
            }
            (*qi).qf_bufnr = (*curbuf.get()).handle as ::core::ffi::c_int;
        }
        if !bt_quickfix(curbuf.get()) {
            qf_set_cwindow_options();
        }
        if curtab.get() == prevtab as *mut tabpage_T && (*curwin.get()).w_width == Columns.get() {
            win_setheight(height);
        }
        (*curwin.get()).w_onebuf_opt.wo_wfh = true_0;
        if win_valid(win) {
            prevwin.set(win);
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn qf_set_title_var(mut qfl: *mut qf_list_T) {
    unsafe {
        if !(*qfl).qf_title.is_null() {
            set_internal_string_var(
                b"w:quickfix_title\0".as_ptr() as *const ::core::ffi::c_char,
                (*qfl).qf_title,
            );
        }
    }
}

pub unsafe fn ex_copen(mut eap: *mut exarg_T) {
    unsafe {
        let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
        qi = qf_cmd_get_stack(eap, true_0 != 0);
        if qi.is_null() {
            return;
        }
        incr_quickfix_busy();
        let mut height: ::core::ffi::c_int = 0;
        if (*eap).addr_count != 0 as ::core::ffi::c_int {
            height = (*eap).line2 as ::core::ffi::c_int;
        } else {
            height = QF_WINHEIGHT as ::core::ffi::c_int;
        }
        reset_VIsual_and_resel();
        let mut status: ::core::ffi::c_int = FAIL;
        if (*cmdmod.ptr()).cmod_tab == 0 as ::core::ffi::c_int {
            status = qf_goto_cwindow(
                qi,
                (*eap).addr_count != 0 as ::core::ffi::c_int,
                height,
                (*cmdmod.ptr()).cmod_split & WSP_VERT as ::core::ffi::c_int != 0,
            );
        }
        if status == FAIL {
            if qf_open_new_cwindow(qi, height) == FAIL {
                decr_quickfix_busy();
                return;
            }
        }
        let mut qfl: *mut qf_list_T = qf_get_curlist(qi);
        qf_set_title_var(qfl);
        let mut lnum: ::core::ffi::c_int = (*qfl).qf_index;
        qf_fill_buffer(
            qfl,
            curbuf.get(),
            ::core::ptr::null_mut::<qfline_T>(),
            (*curwin.get()).handle as ::core::ffi::c_int,
        );
        decr_quickfix_busy();
        (*curwin.get()).w_cursor.lnum = lnum as linenr_T;
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        check_cursor(curwin.get());
        update_topline(curwin.get());
    }
}

pub(crate) unsafe extern "C" fn qf_win_goto(mut win: *mut win_T, mut lnum: linenr_T) {
    unsafe {
        let mut old_curwin: *mut win_T = curwin.get();
        curwin.set(win);
        curbuf.set((*win).w_buffer);
        (*curwin.get()).w_cursor.lnum = lnum;
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
        (*curwin.get()).w_curswant = 0 as ::core::ffi::c_int as colnr_T;
        update_topline(curwin.get());
        redraw_later(curwin.get(), UPD_VALID);
        (*curwin.get()).w_redr_status = true_0 != 0;
        curwin.set(old_curwin);
        curbuf.set((*curwin.get()).w_buffer);
    }
}

pub unsafe fn ex_cbottom(mut eap: *mut exarg_T) {
    unsafe {
        let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
        qi = qf_cmd_get_stack(eap, true_0 != 0);
        if qi.is_null() {
            return;
        }
        let mut win: *mut win_T = qf_find_win(qi);
        if !win.is_null() && (*win).w_cursor.lnum != (*(*win).w_buffer).b_ml.ml_line_count {
            qf_win_goto(win, (*(*win).w_buffer).b_ml.ml_line_count);
        }
    }
}

pub unsafe fn qf_current_entry(mut wp: *mut win_T) -> linenr_T {
    unsafe {
        let mut qi: *mut qf_info_T = ql_info.get();
        '_c2rust_label: {
            if !qi.is_null() {
            } else {
                __assert_fail(
                    b"qi != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    4056 as ::core::ffi::c_uint,
                    b"linenr_T qf_current_entry(win_T *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0 && !(*wp).w_llist_ref.is_null() {
            qi = (*wp).w_llist_ref;
        }
        return (*qf_get_curlist(qi)).qf_index as linenr_T;
    }
}

pub(crate) unsafe extern "C" fn qf_win_pos_update(
    mut qi: *mut qf_info_T,
    mut old_qf_index: ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut qf_index: ::core::ffi::c_int = (*qf_get_curlist(qi)).qf_index;
        let mut win: *mut win_T = qf_find_win(qi);
        if !win.is_null()
            && qf_index as linenr_T <= (*(*win).w_buffer).b_ml.ml_line_count
            && old_qf_index != qf_index
        {
            (*win).w_redraw_top = (if old_qf_index < qf_index {
                old_qf_index
            } else {
                qf_index
            }) as linenr_T;
            (*win).w_redraw_bot = (if old_qf_index > qf_index {
                old_qf_index
            } else {
                qf_index
            }) as linenr_T;
            qf_win_goto(win, qf_index as linenr_T);
        }
        return !win.is_null();
    }
}

pub(crate) unsafe extern "C" fn is_qf_win(
    mut win: *const win_T,
    mut qi: *const qf_info_T,
) -> ::core::ffi::c_int {
    unsafe {
        if buf_valid((*win).w_buffer) as ::core::ffi::c_int != 0
            && bt_quickfix((*win).w_buffer) as ::core::ffi::c_int != 0
        {
            if (*qi).qfl_type as ::core::ffi::c_uint
                == QFLT_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*win).w_llist_ref.is_null()
                || (*qi).qfl_type as ::core::ffi::c_uint
                    == QFLT_LOCATION as ::core::ffi::c_int as ::core::ffi::c_uint
                    && (*win).w_llist_ref == qi as *mut qf_info_T
            {
                return true_0;
            }
        }
        return false_0;
    }
}

pub(crate) unsafe extern "C" fn qf_find_win(mut qi: *const qf_info_T) -> *mut win_T {
    unsafe {
        let mut win: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !win.is_null() {
            if is_qf_win(win, qi) != 0 {
                return win;
            }
            win = (*win).w_next;
        }
        return ::core::ptr::null_mut::<win_T>();
    }
}

pub(crate) unsafe extern "C" fn qf_find_buf(mut qi: *mut qf_info_T) -> *mut buf_T {
    unsafe {
        if (*qi).qf_bufnr != INVALID_QFBUFNR {
            let qfbuf: *mut buf_T = buflist_findnr((*qi).qf_bufnr);
            if !qfbuf.is_null() {
                return qfbuf;
            }
            (*qi).qf_bufnr = INVALID_QFBUFNR;
        }
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut win: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !win.is_null() {
                if is_qf_win(win, qi) != 0 {
                    return (*win).w_buffer;
                }
                win = (*win).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        return ::core::ptr::null_mut::<buf_T>();
    }
}

pub unsafe extern "C" fn did_set_quickfixtextfunc(
    mut _args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    unsafe {
        if option_set_callback_func(p_qftf.get(), qftf_cb.ptr()) == FAIL {
            return &raw const e_invarg as *const ::core::ffi::c_char;
        }
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
}

pub(crate) unsafe extern "C" fn qf_update_win_titlevar(mut qi: *mut qf_info_T) {
    unsafe {
        let qfl: *mut qf_list_T = qf_get_curlist(qi);
        let save_curwin: *mut win_T = curwin.get();
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut win: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !win.is_null() {
                if is_qf_win(win, qi) != 0 {
                    curwin.set(win);
                    qf_set_title_var(qfl);
                }
                win = (*win).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        curwin.set(save_curwin);
    }
}

pub(crate) unsafe extern "C" fn qf_update_buffer(
    mut qi: *mut qf_info_T,
    mut old_last: *mut qfline_T,
) {
    unsafe {
        let mut buf: *mut buf_T = qf_find_buf(qi);
        if buf.is_null() {
            return;
        }
        let mut old_line_count: linenr_T = (*buf).b_ml.ml_line_count;
        let mut old_endcol: colnr_T = ml_get_buf_len(buf, old_line_count);
        let mut old_bytecount: bcount_t =
            get_region_bytecount(buf, 1 as linenr_T, old_line_count, 0 as colnr_T, old_endcol);
        let mut qf_winid_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
        if (*qi).qfl_type as ::core::ffi::c_uint
            == QFLT_LOCATION as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*curwin.get()).w_llist == qi {
                win = curwin.get();
            } else {
                win = qf_find_win_with_loclist(qi);
                if win.is_null() {
                    win = qf_find_win(qi);
                }
                if win.is_null() {
                    return;
                }
            }
            qf_winid_0 = (*win).handle;
        }
        incr_quickfix_busy();
        let mut aco: aco_save_T = aco_save_T::default();
        if old_last.is_null() {
            aucmd_prepbuf(&raw mut aco, buf);
        }
        qf_update_win_titlevar(qi);
        qf_fill_buffer(qf_get_curlist(qi), buf, old_last, qf_winid_0);
        let mut new_line_count: linenr_T = (*buf).b_ml.ml_line_count;
        let mut new_endcol: colnr_T = ml_get_buf_len(buf, new_line_count);
        let mut new_byte_count: bcount_t = 0 as bcount_t;
        let mut delta: linenr_T = new_line_count - old_line_count;
        if old_last.is_null() {
            new_byte_count =
                get_region_bytecount(buf, 1 as linenr_T, new_line_count, 0 as colnr_T, new_endcol);
            extmark_splice(
                buf,
                0 as ::core::ffi::c_int,
                0 as colnr_T,
                old_line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                0 as colnr_T,
                old_bytecount,
                new_line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                new_endcol,
                new_byte_count,
                kExtmarkNoUndo,
            );
            changed_lines(
                buf,
                1 as linenr_T,
                0 as colnr_T,
                if old_line_count > 0 as linenr_T {
                    old_line_count + 1 as linenr_T
                } else {
                    1 as linenr_T
                },
                delta,
                true_0 != 0,
            );
        } else if delta > 0 as linenr_T {
            let mut start_lnum: linenr_T = old_line_count + 1 as linenr_T;
            new_byte_count =
                get_region_bytecount(buf, start_lnum, new_line_count, 0 as colnr_T, new_endcol);
            extmark_splice(
                buf,
                old_line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                old_endcol,
                0 as ::core::ffi::c_int,
                0 as colnr_T,
                0 as bcount_t,
                delta as ::core::ffi::c_int,
                new_endcol,
                new_byte_count,
                kExtmarkNoUndo,
            );
            changed_lines(
                buf,
                start_lnum,
                0 as colnr_T,
                start_lnum,
                delta,
                true_0 != 0,
            );
        }
        (*buf).b_changed = false_0;
        if old_last.is_null() {
            qf_win_pos_update(qi, 0 as ::core::ffi::c_int);
            aucmd_restbuf(&raw mut aco);
        }
        win = qf_find_win(qi);
        if !win.is_null() && old_line_count < (*win).w_botline {
            redraw_buf_later(buf, UPD_NOT_VALID);
        }
        decr_quickfix_busy();
    }
}

pub(crate) unsafe extern "C" fn qf_buf_add_line(
    mut _qfl: *mut qf_list_T,
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut qfp: *const qfline_T,
    mut dirname: *mut ::core::ffi::c_char,
    mut qftf_str: *mut ::core::ffi::c_char,
    mut first_bufline: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut gap: *mut garray_T = qfga_get();
        if !qftf_str.is_null() && *qftf_str as ::core::ffi::c_int != NUL {
            ga_concat(gap, qftf_str);
        } else {
            let mut errbuf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
            if !(*qfp).qf_module.is_null() {
                ga_concat(gap, (*qfp).qf_module);
            } else if (*qfp).qf_fnum != 0 as ::core::ffi::c_int
                && {
                    errbuf = buflist_findnr((*qfp).qf_fnum);
                    !errbuf.is_null()
                }
                && !(*errbuf).b_fname.is_null()
            {
                if (*qfp).qf_type as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
                    ga_concat(gap, path_tail((*errbuf).b_fname));
                } else {
                    if first_bufline as ::core::ffi::c_int != 0
                        && ((*errbuf).b_sfname.is_null()
                            || path_is_absolute((*errbuf).b_sfname) as ::core::ffi::c_int != 0)
                    {
                        if *dirname as ::core::ffi::c_int == NUL {
                            os_dirname(dirname, MAXPATHL as size_t);
                        }
                        shorten_buf_fname(errbuf, dirname, false_0);
                    }
                    ga_concat(
                        gap,
                        if (*qfp).qf_fname.is_null() {
                            (*errbuf).b_fname
                        } else {
                            (*qfp).qf_fname
                        },
                    );
                }
            }
            ga_append(gap, '|' as uint8_t);
            if (*qfp).qf_lnum > 0 as linenr_T {
                qf_range_text(gap, qfp);
                ga_concat(
                    gap,
                    qf_types((*qfp).qf_type as ::core::ffi::c_int, (*qfp).qf_nr),
                );
            } else if !(*qfp).qf_pattern.is_null() {
                qf_fmt_text(gap, (*qfp).qf_pattern);
            }
            ga_append(gap, '|' as uint8_t);
            ga_append(gap, ' ' as uint8_t);
            qf_fmt_text(
                gap,
                if (*gap).ga_len > 3 as ::core::ffi::c_int {
                    skipwhite((*qfp).qf_text)
                } else {
                    (*qfp).qf_text
                },
            );
        }
        ga_append(gap, NUL as uint8_t);
        if ml_append_buf(
            buf,
            lnum,
            (*gap).ga_data as *mut ::core::ffi::c_char,
            (*gap).ga_len as colnr_T,
            false_0 != 0,
        ) == FAIL
        {
            return FAIL;
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn call_qftf_func(
    mut qfl: *mut qf_list_T,
    mut qf_winid_0: ::core::ffi::c_int,
    mut start_idx: ::core::ffi::c_int,
    mut end_idx: ::core::ffi::c_int,
) -> *mut list_T {
    unsafe {
        let mut cb: *mut Callback = qftf_cb.ptr();
        let mut qftf_list: *mut list_T = ::core::ptr::null_mut::<list_T>();
        static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        if recursive.get() {
            return ::core::ptr::null_mut::<list_T>();
        }
        recursive.set(true_0 != 0);
        if (*qfl).qf_qftf_cb.type_0 as ::core::ffi::c_uint
            != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            cb = &raw mut (*qfl).qf_qftf_cb;
        }
        if (*cb).type_0 as ::core::ffi::c_uint
            != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut args: [typval_T; 1] = [typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            }; 1];
            let mut rettv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            let dict: *mut dict_T = tv_dict_alloc_lock(VAR_FIXED);
            tv_dict_add_nr(
                dict,
                b"quickfix\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                ((*qfl).qfl_type as ::core::ffi::c_uint
                    == QFLT_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_uint)
                    as ::core::ffi::c_int as varnumber_T,
            );
            tv_dict_add_nr(
                dict,
                b"winid\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                qf_winid_0 as varnumber_T,
            );
            tv_dict_add_nr(
                dict,
                b"id\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
                (*qfl).qf_id as varnumber_T,
            );
            tv_dict_add_nr(
                dict,
                b"start_idx\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
                start_idx as varnumber_T,
            );
            tv_dict_add_nr(
                dict,
                b"end_idx\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                end_idx as varnumber_T,
            );
            (*dict).dv_refcount += 1;
            args[0 as ::core::ffi::c_int as usize].v_type = VAR_DICT;
            args[0 as ::core::ffi::c_int as usize].vval.v_dict = dict;
            (*textlock.ptr()) += 1;
            if callback_call(
                cb,
                1 as ::core::ffi::c_int,
                &raw mut args as *mut typval_T,
                &raw mut rettv,
            ) {
                if rettv.v_type as ::core::ffi::c_uint
                    == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    qftf_list = rettv.vval.v_list;
                    tv_list_ref(qftf_list);
                }
                tv_clear(&raw mut rettv);
            }
            (*textlock.ptr()) -= 1;
            tv_dict_unref(dict);
        }
        recursive.set(false_0 != 0);
        return qftf_list;
    }
}

pub(crate) unsafe extern "C" fn qf_fill_buffer(
    mut qfl: *mut qf_list_T,
    mut buf: *mut buf_T,
    mut old_last: *mut qfline_T,
    mut qf_winid_0: ::core::ffi::c_int,
) {
    unsafe {
        let old_KeyTyped: bool = KeyTyped.get();
        if old_last.is_null() {
            if buf != curbuf.get() {
                internal_error(b"qf_fill_buffer()\0".as_ptr() as *const ::core::ffi::c_char);
                return;
            }
            while (*curbuf.get()).b_ml.ml_flags & ML_EMPTY == 0 as ::core::ffi::c_int {
                if ml_delete(1 as linenr_T) == FAIL {
                    internal_error(b"qf_fill_buffer()\0".as_ptr() as *const ::core::ffi::c_char);
                    return;
                }
            }
            let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
            while !tp.is_null() {
                let mut wp: *mut win_T = if tp == curtab.get() {
                    firstwin.get()
                } else {
                    (*tp).tp_firstwin
                };
                while !wp.is_null() {
                    if (*wp).w_buffer == curbuf.get() {
                        (*wp).w_skipcol = 0 as ::core::ffi::c_int as colnr_T;
                    }
                    wp = (*wp).w_next;
                }
                tp = (*tp).tp_next as *mut tabpage_T;
            }
            u_clearallandblockfree(curbuf.get());
        }
        if !qfl.is_null() && !(*qfl).qf_start.is_null() {
            let mut dirname: [::core::ffi::c_char; 4096] = [0; 4096];
            *(&raw mut dirname as *mut ::core::ffi::c_char) = NUL as ::core::ffi::c_char;
            let mut lnum: linenr_T = 0;
            let mut qfp: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
            if old_last.is_null() {
                qfp = (*qfl).qf_start;
                lnum = 0 as ::core::ffi::c_int as linenr_T;
            } else {
                qfp = if !(*old_last).qf_next.is_null() {
                    (*old_last).qf_next
                } else {
                    old_last
                };
                lnum = (*buf).b_ml.ml_line_count;
            }
            let mut qftf_list: *mut list_T = call_qftf_func(
                qfl,
                qf_winid_0,
                lnum as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
                (*qfl).qf_count,
            );
            let mut qftf_li: *mut listitem_T = tv_list_first(qftf_list);
            let mut prev_bufnr: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
            let mut invalid_val: bool = false_0 != 0;
            while lnum < (*qfl).qf_count as linenr_T {
                let mut qftf_str: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                if !qftf_li.is_null() && !invalid_val {
                    qftf_str =
                        tv_get_string_chk(&raw mut (*qftf_li).li_tv) as *mut ::core::ffi::c_char;
                    if qftf_str.is_null() {
                        invalid_val = true_0 != 0;
                    }
                }
                if qf_buf_add_line(
                    qfl,
                    buf,
                    lnum,
                    qfp,
                    &raw mut dirname as *mut ::core::ffi::c_char,
                    qftf_str,
                    prev_bufnr != (*qfp).qf_fnum,
                ) == FAIL
                {
                    break;
                }
                prev_bufnr = (*qfp).qf_fnum;
                lnum += 1;
                qfp = (*qfp).qf_next;
                if qfp.is_null() {
                    break;
                }
                if !qftf_li.is_null() {
                    qftf_li = (*qftf_li).li_next;
                }
            }
            if old_last.is_null() {
                ml_delete(lnum + 1 as linenr_T);
            }
            qfga_clear();
        }
        check_lnums(true_0 != 0);
        if old_last.is_null() {
            (*curbuf.get()).b_ro_locked += 1;
            set_option_value_give_err(
                kOptFiletype,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: String_0 {
                            data: b"qf\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                .wrapping_sub(1 as size_t),
                        },
                    },
                },
                OPT_LOCAL as ::core::ffi::c_int,
            );
            (*curbuf.get()).b_p_ma = false_0;
            (*curbuf.get()).b_keep_filetype = true_0 != 0;
            apply_autocmds(
                EVENT_BUFREADPOST,
                b"quickfix\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
            apply_autocmds(
                EVENT_BUFWINENTER,
                b"quickfix\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
            (*curbuf.get()).b_keep_filetype = false_0 != 0;
            (*curbuf.get()).b_ro_locked -= 1;
            redraw_curbuf_later(UPD_NOT_VALID);
        }
        KeyTyped.set(old_KeyTyped);
    }
}
