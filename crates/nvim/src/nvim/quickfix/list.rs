//! The stack of lists, and the entries in one.
//!
//! A `qf_info_T` owns up to `LISTCOUNT` `qf_list_T`s and is shared by
//! reference between windows; [`qf_alloc_stack`], [`qf_resize_stack`] and
//! [`qf_free_all`] are its lifecycle, [`qf_new_list`] pushes a list and
//! [`qf_add_entry`] an entry. [`copy_loclist`] is what makes a location
//! list follow a window that was split, and [`qf_mark_adjust`] moves entry
//! line numbers when the buffer they point into is edited.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn qf_stack_empty(mut qi: *const qf_info_T) -> bool {
    unsafe {
        return qi.is_null() || (*qi).qf_listcount <= 0 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_list_empty(mut qfl: *mut qf_list_T) -> bool {
    unsafe {
        return qfl.is_null() || (*qfl).qf_count <= 0 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_list_has_valid_entries(mut qfl: *mut qf_list_T) -> bool {
    unsafe {
        return !qf_list_empty(qfl) && !(*qfl).qf_nonevalid;
    }
}

pub(crate) unsafe extern "C" fn qf_get_list(
    mut qi: *mut qf_info_T,
    mut idx: ::core::ffi::c_int,
) -> *mut qf_list_T {
    unsafe {
        return (*qi).qf_lists.offset(idx as isize);
    }
}

pub(crate) unsafe extern "C" fn qf_get_curlist(mut qi: *mut qf_info_T) -> *mut qf_list_T {
    unsafe {
        return qf_get_list(qi, (*qi).qf_curlist);
    }
}

pub(crate) unsafe extern "C" fn qf_pop_stack(mut qi: *mut qf_info_T, mut adjust: bool) {
    unsafe {
        qf_free((*qi).qf_lists.offset(0 as ::core::ffi::c_int as isize));
        let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while i < (*qi).qf_listcount {
            *(*qi)
                .qf_lists
                .offset((i - 1 as ::core::ffi::c_int) as isize) =
                *(*qi).qf_lists.offset(i as isize);
            i += 1;
        }
        memset(
            (*qi)
                .qf_lists
                .offset((*qi).qf_listcount as isize)
                .offset(-(1 as ::core::ffi::c_int as isize))
                as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<qf_list_T>(),
        );
        if adjust {
            (*qi).qf_listcount -= 1;
            if (*qi).qf_curlist == 0 as ::core::ffi::c_int {
                (*qi).qf_curlist = (*qi).qf_listcount - 1 as ::core::ffi::c_int;
            } else {
                (*qi).qf_curlist -= 1;
            }
        }
    }
}

pub(crate) unsafe extern "C" fn qf_new_list(
    mut qi: *mut qf_info_T,
    mut qf_title: *const ::core::ffi::c_char,
) {
    unsafe {
        while (*qi).qf_listcount > (*qi).qf_curlist + 1 as ::core::ffi::c_int {
            (*qi).qf_listcount -= 1;
            qf_free((*qi).qf_lists.offset((*qi).qf_listcount as isize));
        }
        if (*qi).qf_listcount == (*qi).qf_maxcount {
            qf_pop_stack(qi, false_0 != 0);
            (*qi).qf_curlist = (*qi).qf_listcount - 1 as ::core::ffi::c_int;
        } else {
            let c2rust_fresh21 = (*qi).qf_listcount;
            (*qi).qf_listcount = (*qi).qf_listcount + 1;
            (*qi).qf_curlist = c2rust_fresh21;
        }
        let mut qfl: *mut qf_list_T = qf_get_curlist(qi);
        memset(
            qfl as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<qf_list_T>(),
        );
        qf_store_title(qfl, qf_title);
        (*qfl).qfl_type = (*qi).qfl_type;
        last_qf_id.set((*last_qf_id.ptr()).wrapping_add(1));
        (*qfl).qf_id = last_qf_id.get();
        (*qfl).qf_has_user_data = false_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn locstack_queue_delreq(mut qi: *mut qf_info_T) {
    unsafe {
        let mut q: *mut qf_delq_T = xmalloc(::core::mem::size_of::<qf_delq_T>()) as *mut qf_delq_T;
        (*q).qi = qi;
        (*q).next = qf_delq_head.get() as *mut qf_delq_S;
        qf_delq_head.set(q);
    }
}

pub unsafe extern "C" fn qf_stack_get_bufnr() -> ::core::ffi::c_int {
    unsafe {
        '_c2rust_label: {
            if !(*ql_info.ptr()).is_null() {
            } else {
                __assert_fail(
                    b"ql_info != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1770 as ::core::ffi::c_uint,
                    b"int qf_stack_get_bufnr(void)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        return (*ql_info.get()).qf_bufnr;
    }
}

pub(crate) unsafe extern "C" fn wipe_qf_buffer(mut qi: *mut qf_info_T) {
    unsafe {
        if (*qi).qf_bufnr == INVALID_QFBUFNR {
            return;
        }
        let qfbuf: *mut buf_T = buflist_findnr((*qi).qf_bufnr);
        if !qfbuf.is_null() && (*qfbuf).b_nwindows == 0 as ::core::ffi::c_int {
            let mut buf_was_null: bool = false_0 != 0;
            if (*curwin.get()).w_buffer.is_null() {
                (*curwin.get()).w_buffer = curbuf.get();
                buf_was_null = true_0 != 0;
            }
            close_buffer(
                ::core::ptr::null_mut::<win_T>(),
                qfbuf,
                DOBUF_WIPE as ::core::ffi::c_int,
                false_0 != 0,
                false_0 != 0,
            );
            (*qi).qf_bufnr = INVALID_QFBUFNR;
            if buf_was_null {
                (*curwin.get()).w_buffer = ::core::ptr::null_mut::<buf_T>();
            }
        }
    }
}

pub(crate) unsafe extern "C" fn qf_free_list_stack_items(mut qi: *mut qf_info_T) {
    unsafe {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*qi).qf_listcount {
            qf_free(qf_get_list(qi, i));
            i += 1;
        }
    }
}

pub(crate) unsafe extern "C" fn qf_free_lists(mut qi: *mut qf_info_T) {
    unsafe {
        qf_free_list_stack_items(qi);
        xfree((*qi).qf_lists as *mut ::core::ffi::c_void);
        xfree(qi as *mut ::core::ffi::c_void);
    }
}

pub(crate) unsafe extern "C" fn ll_free_all(mut pqi: *mut *mut qf_info_T) {
    unsafe {
        let mut qi: *mut qf_info_T = *pqi;
        if qi.is_null() {
            return;
        }
        *pqi = ::core::ptr::null_mut::<qf_info_T>();
        if quickfix_busy.get() > 0 as ::core::ffi::c_int {
            locstack_queue_delreq(qi);
            return;
        }
        (*qi).qf_refcount -= 1;
        if (*qi).qf_refcount < 1 as ::core::ffi::c_int {
            wipe_qf_buffer(qi);
            qf_free_lists(qi);
        }
    }
}

pub unsafe fn qf_free_all(mut wp: *mut win_T) {
    unsafe {
        let mut qi: *mut qf_info_T = ql_info.get();
        if !wp.is_null() {
            ll_free_all(&raw mut (*wp).w_llist);
            ll_free_all(&raw mut (*wp).w_llist_ref);
        } else if !qi.is_null() {
            qf_free_list_stack_items(qi);
        }
    }
}

pub(crate) unsafe extern "C" fn incr_quickfix_busy() {
    unsafe {
        (*quickfix_busy.ptr()) += 1;
    }
}

pub(crate) unsafe extern "C" fn decr_quickfix_busy() {
    unsafe {
        (*quickfix_busy.ptr()) -= 1;
        if quickfix_busy.get() == 0 as ::core::ffi::c_int {
            while !(*qf_delq_head.ptr()).is_null() {
                let mut q: *mut qf_delq_T = qf_delq_head.get();
                qf_delq_head.set((*q).next as *mut qf_delq_T);
                ll_free_all(&raw mut (*q).qi);
                xfree(q as *mut ::core::ffi::c_void);
            }
        }
    }
}

pub(crate) unsafe extern "C" fn qf_add_entry(
    mut qfl: *mut qf_list_T,
    mut dir: *mut ::core::ffi::c_char,
    mut fname: *mut ::core::ffi::c_char,
    mut module: *mut ::core::ffi::c_char,
    mut bufnum: ::core::ffi::c_int,
    mut mesg: *mut ::core::ffi::c_char,
    mut lnum: linenr_T,
    mut end_lnum: linenr_T,
    mut col: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
    mut vis_col: ::core::ffi::c_char,
    mut pattern: *mut ::core::ffi::c_char,
    mut nr: ::core::ffi::c_int,
    mut type_0: ::core::ffi::c_char,
    mut user_data: *mut typval_T,
    mut valid: ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        let mut qfp: *mut qfline_T = xmalloc(::core::mem::size_of::<qfline_T>()) as *mut qfline_T;
        let mut fullname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if bufnum != 0 as ::core::ffi::c_int {
            buf = buflist_findnr(bufnum);
            (*qfp).qf_fnum = bufnum;
            if !buf.is_null() {
                (*buf).b_has_qf_entry |= if (*qfl).qfl_type as ::core::ffi::c_uint
                    == QFLT_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    BUF_HAS_QF_ENTRY
                } else {
                    BUF_HAS_LL_ENTRY
                };
            }
        } else {
            (*qfp).qf_fnum = qf_get_fnum(qfl, dir, fname);
            buf = buflist_findnr((*qfp).qf_fnum);
        }
        if !fname.is_null() {
            fullname = fix_fname(fname);
        }
        (*qfp).qf_fname = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if !buf.is_null() && !(*buf).b_ffname.is_null() && !fullname.is_null() {
            if path_fnamecmp(fullname, (*buf).b_ffname) != 0 as ::core::ffi::c_int {
                p = path_try_shorten_fname(fullname);
                if !p.is_null() {
                    (*qfp).qf_fname = xstrdup(p);
                }
            }
        }
        xfree(fullname as *mut ::core::ffi::c_void);
        (*qfp).qf_text = xstrdup(mesg);
        (*qfp).qf_lnum = lnum;
        (*qfp).qf_end_lnum = end_lnum;
        (*qfp).qf_col = col;
        (*qfp).qf_end_col = end_col;
        (*qfp).qf_viscol = vis_col;
        if user_data.is_null()
            || (*user_data).v_type as ::core::ffi::c_uint
                == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*qfp).qf_user_data.v_type = VAR_UNKNOWN;
        } else {
            tv_copy(user_data, &raw mut (*qfp).qf_user_data);
            (*qfl).qf_has_user_data = true_0 != 0;
        }
        if pattern.is_null() || *pattern as ::core::ffi::c_int == NUL {
            (*qfp).qf_pattern = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            (*qfp).qf_pattern = xstrdup(pattern);
        }
        if module.is_null() || *module as ::core::ffi::c_int == NUL {
            (*qfp).qf_module = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            (*qfp).qf_module = xstrdup(module);
        }
        (*qfp).qf_nr = nr;
        if type_0 as ::core::ffi::c_int != 1 as ::core::ffi::c_int
            && !vim_isprintc(type_0 as ::core::ffi::c_int)
        {
            type_0 = 0 as ::core::ffi::c_char;
        }
        (*qfp).qf_type = type_0;
        (*qfp).qf_valid = valid;
        let mut lastp: *mut *mut qfline_T = &raw mut (*qfl).qf_last;
        if qf_list_empty(qfl) {
            (*qfl).qf_start = qfp;
            (*qfl).qf_ptr = qfp;
            (*qfl).qf_index = 0 as ::core::ffi::c_int;
            (*qfp).qf_prev = ::core::ptr::null_mut::<qfline_T>();
        } else {
            '_c2rust_label: {
                if !(*lastp).is_null() {
                } else {
                    __assert_fail(
                    b"*lastp\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/quickfix.rs\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    1998 as ::core::ffi::c_uint,
                    b"int qf_add_entry(qf_list_T *, char *, char *, char *, int, char *, linenr_T, linenr_T, int, int, char, char *, int, char, typval_T *, char)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
                }
            };
            (*qfp).qf_prev = *lastp;
            (**lastp).qf_next = qfp;
        }
        (*qfp).qf_next = ::core::ptr::null_mut::<qfline_T>();
        (*qfp).qf_cleared = false_0 as ::core::ffi::c_char;
        *lastp = qfp;
        (*qfl).qf_count += 1;
        if (*qfl).qf_index == 0 as ::core::ffi::c_int && (*qfp).qf_valid as ::core::ffi::c_int != 0
        {
            (*qfl).qf_index = (*qfl).qf_count;
            (*qfl).qf_ptr = qfp;
        }
        return QF_OK as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn qf_resize_stack(mut n: ::core::ffi::c_int) {
    unsafe {
        '_c2rust_label: {
            if !(*ql_info.ptr()).is_null() {
            } else {
                __assert_fail(
                    b"ql_info != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2018 as ::core::ffi::c_uint,
                    b"void qf_resize_stack(int)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        qf_resize_stack_base(ql_info.get(), n);
    }
}

pub unsafe fn ll_resize_stack(mut wp: *mut win_T, mut n: ::core::ffi::c_int) {
    unsafe {
        if bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0 && !(*wp).w_llist_ref.is_null() {
            qf_sync_llw_to_win(wp);
        } else {
            qf_sync_win_to_llw(wp);
        }
        let mut qi: *mut qf_info_T = ll_get_or_alloc_list(wp);
        qf_resize_stack_base(qi, n);
    }
}

pub(crate) unsafe extern "C" fn qf_resize_stack_base(
    mut qi: *mut qf_info_T,
    mut n: ::core::ffi::c_int,
) {
    unsafe {
        let mut amount_to_rm: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut lsz: size_t = ::core::mem::size_of::<qf_list_T>();
        if n == (*qi).qf_maxcount {
            return;
        } else if n < (*qi).qf_maxcount && n < (*qi).qf_listcount {
            amount_to_rm = (*qi).qf_listcount - n;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < amount_to_rm {
                qf_pop_stack(qi, true_0 != 0);
                i += 1;
            }
        }
        let mut new: *mut qf_list_T = xrealloc(
            (*qi).qf_lists as *mut ::core::ffi::c_void,
            lsz.wrapping_mul(n as size_t),
        ) as *mut qf_list_T;
        if n > (*qi).qf_maxcount {
            memset(
                new.offset((*qi).qf_maxcount as isize) as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                lsz.wrapping_mul((n - (*qi).qf_maxcount) as size_t),
            );
        }
        (*qi).qf_lists = new;
        (*qi).qf_maxcount = n;
        qf_update_buffer(qi, ::core::ptr::null_mut::<qfline_T>());
    }
}

pub unsafe extern "C" fn qf_init_stack() {
    unsafe {
        ql_info.set(qf_alloc_stack(
            QFLT_QUICKFIX,
            p_chi.get() as ::core::ffi::c_int,
        ));
    }
}

pub(crate) unsafe extern "C" fn qf_sync_llw_to_win(mut llw: *mut win_T) {
    unsafe {
        let mut wp: *mut win_T = qf_find_win_with_loclist((*llw).w_llist_ref);
        if !wp.is_null() {
            (*wp).w_onebuf_opt.wo_lhi = (*llw).w_onebuf_opt.wo_lhi;
        }
    }
}

pub(crate) unsafe extern "C" fn qf_sync_win_to_llw(mut pwp: *mut win_T) {
    unsafe {
        let mut llw: *mut qf_info_T = (*pwp).w_llist;
        if !llw.is_null() {
            let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                firstwin.get()
            } else {
                (*curtab.get()).tp_firstwin
            };
            while !wp.is_null() {
                if (*wp).w_llist_ref == llw
                    && bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0
                {
                    (*wp).w_onebuf_opt.wo_lhi = (*pwp).w_onebuf_opt.wo_lhi;
                    return;
                }
                wp = (*wp).w_next;
            }
        }
    }
}

pub(crate) unsafe extern "C" fn qf_alloc_stack(
    mut qfltype: qfltype_T,
    mut n: ::core::ffi::c_int,
) -> *mut qf_info_T {
    unsafe {
        let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
        if qfltype as ::core::ffi::c_uint
            == QFLT_QUICKFIX as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            qi = ql_info_actual.ptr();
        } else {
            qi = xcalloc(1 as size_t, ::core::mem::size_of::<qf_info_T>()) as *mut qf_info_T;
            (*qi).qf_refcount += 1;
        }
        (*qi).qfl_type = qfltype;
        (*qi).qf_bufnr = INVALID_QFBUFNR;
        (*qi).qf_lists = qf_alloc_list_stack(n);
        (*qi).qf_maxcount = n;
        return qi;
    }
}

pub(crate) unsafe extern "C" fn qf_alloc_list_stack(mut n: ::core::ffi::c_int) -> *mut qf_list_T {
    unsafe {
        return xcalloc(n as size_t, ::core::mem::size_of::<qf_list_T>()) as *mut qf_list_T;
    }
}

pub(crate) unsafe extern "C" fn ll_get_or_alloc_list(mut wp: *mut win_T) -> *mut qf_info_T {
    unsafe {
        if bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0 && !(*wp).w_llist_ref.is_null() {
            return (*wp).w_llist_ref;
        }
        ll_free_all(&raw mut (*wp).w_llist_ref);
        if (*wp).w_llist.is_null() {
            (*wp).w_llist = qf_alloc_stack(
                QFLT_LOCATION,
                (*wp).w_onebuf_opt.wo_lhi as ::core::ffi::c_int,
            );
        }
        return (*wp).w_llist;
    }
}

pub(crate) unsafe extern "C" fn qf_cmd_get_stack(
    mut eap: *mut exarg_T,
    mut print_emsg: bool,
) -> *mut qf_info_T {
    unsafe {
        let mut qi: *mut qf_info_T = ql_info.get();
        '_c2rust_label: {
            if !qi.is_null() {
            } else {
                __assert_fail(
                    b"qi != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2156 as ::core::ffi::c_uint,
                    b"qf_info_T *qf_cmd_get_stack(exarg_T *, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if is_loclist_cmd((*eap).cmdidx as ::core::ffi::c_int) {
            qi = if bt_quickfix((*curwin.get()).w_buffer) as ::core::ffi::c_int != 0
                && !(*curwin.get()).w_llist_ref.is_null()
            {
                (*curwin.get()).w_llist_ref
            } else {
                (*curwin.get()).w_llist
            };
            if qi.is_null() {
                if print_emsg {
                    emsg(gettext(&raw const e_loclist as *const ::core::ffi::c_char));
                }
                return ::core::ptr::null_mut::<qf_info_T>();
            }
        }
        return qi;
    }
}

pub(crate) unsafe extern "C" fn qf_cmd_get_or_alloc_stack(
    mut eap: *const exarg_T,
    mut pwinp: *mut *mut win_T,
) -> *mut qf_info_T {
    unsafe {
        let mut qi: *mut qf_info_T = ql_info.get();
        if is_loclist_cmd((*eap).cmdidx as ::core::ffi::c_int) {
            qi = ll_get_or_alloc_list(curwin.get());
            *pwinp = curwin.get();
        }
        return qi;
    }
}

pub(crate) unsafe extern "C" fn copy_loclist_entries(
    mut from_qfl: *const qf_list_T,
    mut to_qfl: *mut qf_list_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut i: ::core::ffi::c_int = 0;
        let mut from_qfp: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
        i = 1 as ::core::ffi::c_int;
        from_qfp = (*from_qfl).qf_start;
        while !got_int.get() && i <= (*from_qfl).qf_count && !from_qfp.is_null() {
            if qf_add_entry(
                to_qfl,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                (*from_qfp).qf_module,
                0 as ::core::ffi::c_int,
                (*from_qfp).qf_text,
                (*from_qfp).qf_lnum,
                (*from_qfp).qf_end_lnum,
                (*from_qfp).qf_col,
                (*from_qfp).qf_end_col,
                (*from_qfp).qf_viscol,
                (*from_qfp).qf_pattern,
                (*from_qfp).qf_nr,
                0 as ::core::ffi::c_char,
                &raw mut (*from_qfp).qf_user_data,
                (*from_qfp).qf_valid,
            ) == QF_FAIL as ::core::ffi::c_int
            {
                return FAIL;
            }
            let prevp: *mut qfline_T = (*to_qfl).qf_last;
            (*prevp).qf_fnum = (*from_qfp).qf_fnum;
            (*prevp).qf_type = (*from_qfp).qf_type;
            if (*from_qfl).qf_ptr == from_qfp {
                (*to_qfl).qf_ptr = prevp;
            }
            i += 1;
            from_qfp = (*from_qfp).qf_next;
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn copy_loclist(
    mut from_qfl: *mut qf_list_T,
    mut to_qfl: *mut qf_list_T,
) -> ::core::ffi::c_int {
    unsafe {
        (*to_qfl).qfl_type = (*from_qfl).qfl_type;
        (*to_qfl).qf_nonevalid = (*from_qfl).qf_nonevalid;
        (*to_qfl).qf_has_user_data = (*from_qfl).qf_has_user_data;
        (*to_qfl).qf_count = 0 as ::core::ffi::c_int;
        (*to_qfl).qf_index = 0 as ::core::ffi::c_int;
        (*to_qfl).qf_start = ::core::ptr::null_mut::<qfline_T>();
        (*to_qfl).qf_last = ::core::ptr::null_mut::<qfline_T>();
        (*to_qfl).qf_ptr = ::core::ptr::null_mut::<qfline_T>();
        if !(*from_qfl).qf_title.is_null() {
            (*to_qfl).qf_title = xstrdup((*from_qfl).qf_title);
        } else {
            (*to_qfl).qf_title = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if !(*from_qfl).qf_ctx.is_null() {
            (*to_qfl).qf_ctx =
                xcalloc(1 as size_t, ::core::mem::size_of::<typval_T>()) as *mut typval_T;
            tv_copy((*from_qfl).qf_ctx, (*to_qfl).qf_ctx);
        } else {
            (*to_qfl).qf_ctx = ::core::ptr::null_mut::<typval_T>();
        }
        callback_copy(
            &raw mut (*to_qfl).qf_qftf_cb,
            &raw mut (*from_qfl).qf_qftf_cb,
        );
        if (*from_qfl).qf_count != 0 {
            if copy_loclist_entries(from_qfl, to_qfl) == FAIL {
                return FAIL;
            }
        }
        (*to_qfl).qf_index = (*from_qfl).qf_index;
        last_qf_id.set((*last_qf_id.ptr()).wrapping_add(1));
        (*to_qfl).qf_id = last_qf_id.get();
        (*to_qfl).qf_changedtick = 0 as ::core::ffi::c_int;
        if (*to_qfl).qf_nonevalid {
            (*to_qfl).qf_ptr = (*to_qfl).qf_start;
            (*to_qfl).qf_index = 1 as ::core::ffi::c_int;
        }
        return OK;
    }
}

pub unsafe fn copy_loclist_stack(mut from: *mut win_T, mut to: *mut win_T) {
    unsafe {
        let mut qi: *mut qf_info_T = if bt_quickfix((*from).w_buffer) as ::core::ffi::c_int != 0
            && !(*from).w_llist_ref.is_null()
        {
            (*from).w_llist_ref
        } else {
            (*from).w_llist
        };
        if qi.is_null() {
            return;
        }
        (*to).w_llist = qf_alloc_stack(
            QFLT_LOCATION,
            (*from).w_onebuf_opt.wo_lhi as ::core::ffi::c_int,
        );
        (*to).w_onebuf_opt.wo_lhi = (*(*to).w_llist).qf_maxcount as OptInt;
        (*(*to).w_llist).qf_listcount = (*qi).qf_listcount;
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while idx < (*qi).qf_listcount {
            (*(*to).w_llist).qf_curlist = idx;
            if copy_loclist(qf_get_list(qi, idx), qf_get_list((*to).w_llist, idx)) == FAIL {
                qf_free_all(to);
                return;
            }
            idx += 1;
        }
        (*(*to).w_llist).qf_curlist = (*qi).qf_curlist;
    }
}

pub(crate) unsafe extern "C" fn qf_free_items(mut qfl: *mut qf_list_T) {
    unsafe {
        let mut stop: bool = false_0 != 0;
        while (*qfl).qf_count != 0 && !(*qfl).qf_start.is_null() {
            let mut qfp: *mut qfline_T = (*qfl).qf_start;
            let mut qfpnext: *mut qfline_T = (*qfp).qf_next;
            if !stop {
                xfree((*qfp).qf_fname as *mut ::core::ffi::c_void);
                xfree((*qfp).qf_module as *mut ::core::ffi::c_void);
                xfree((*qfp).qf_text as *mut ::core::ffi::c_void);
                xfree((*qfp).qf_pattern as *mut ::core::ffi::c_void);
                tv_clear(&raw mut (*qfp).qf_user_data);
                stop = qfp == qfpnext;
                xfree(qfp as *mut ::core::ffi::c_void);
                if stop {
                    (*qfl).qf_count = 1 as ::core::ffi::c_int;
                } else {
                    (*qfl).qf_start = qfpnext;
                }
            }
            (*qfl).qf_count -= 1;
        }
        (*qfl).qf_start = ::core::ptr::null_mut::<qfline_T>();
        (*qfl).qf_ptr = ::core::ptr::null_mut::<qfline_T>();
        (*qfl).qf_index = 0 as ::core::ffi::c_int;
        (*qfl).qf_start = ::core::ptr::null_mut::<qfline_T>();
        (*qfl).qf_last = ::core::ptr::null_mut::<qfline_T>();
        (*qfl).qf_ptr = ::core::ptr::null_mut::<qfline_T>();
        (*qfl).qf_nonevalid = true_0 != 0;
        qf_clean_dir_stack(&raw mut (*qfl).qf_dir_stack);
        (*qfl).qf_directory = ::core::ptr::null_mut::<::core::ffi::c_char>();
        qf_clean_dir_stack(&raw mut (*qfl).qf_file_stack);
        (*qfl).qf_currfile = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*qfl).qf_multiline = false_0 != 0;
        (*qfl).qf_multiignore = false_0 != 0;
        (*qfl).qf_multiscan = false_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn qf_free(mut qfl: *mut qf_list_T) {
    unsafe {
        qf_free_items(qfl);
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*qfl).qf_title as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
        tv_free((*qfl).qf_ctx);
        (*qfl).qf_ctx = ::core::ptr::null_mut::<typval_T>();
        callback_free(&raw mut (*qfl).qf_qftf_cb);
        (*qfl).qf_id = 0 as ::core::ffi::c_uint;
        (*qfl).qf_changedtick = 0 as ::core::ffi::c_int;
    }
}

pub unsafe fn qf_mark_adjust(
    mut buf: *mut buf_T,
    mut wp: *mut win_T,
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut amount: linenr_T,
    mut amount_after: linenr_T,
) -> bool {
    unsafe {
        let mut qi: *mut qf_info_T = ql_info.get();
        '_c2rust_label: {
            if !qi.is_null() {
            } else {
                __assert_fail(
                b"qi != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                3686 as ::core::ffi::c_uint,
                b"_Bool qf_mark_adjust(buf_T *, win_T *, linenr_T, linenr_T, linenr_T, linenr_T)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
            }
        };
        let mut buf_has_flag: ::core::ffi::c_int = if wp.is_null() {
            BUF_HAS_QF_ENTRY
        } else {
            BUF_HAS_LL_ENTRY
        };
        if (*buf).b_has_qf_entry & buf_has_flag == 0 {
            return false_0 != 0;
        }
        if !wp.is_null() {
            if (*wp).w_llist.is_null() {
                return false_0 != 0;
            }
            qi = (*wp).w_llist;
        }
        let mut i: ::core::ffi::c_int = 0;
        let mut qfp: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
        let mut found_one: bool = false_0 != 0;
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while idx < (*qi).qf_listcount {
            let mut qfl: *mut qf_list_T = qf_get_list(qi, idx);
            if !qf_list_empty(qfl) {
                i = 1 as ::core::ffi::c_int;
                qfp = (*qfl).qf_start;
                while !got_int.get() && i <= (*qfl).qf_count && !qfp.is_null() {
                    if (*qfp).qf_fnum == (*buf).handle {
                        found_one = true_0 != 0;
                        if (*qfp).qf_lnum >= line1 && (*qfp).qf_lnum <= line2 {
                            if amount == MAXLNUM as ::core::ffi::c_int as linenr_T {
                                (*qfp).qf_cleared = true_0 as ::core::ffi::c_char;
                            } else {
                                (*qfp).qf_lnum += amount;
                            }
                        } else if amount_after != 0 && (*qfp).qf_lnum > line2 {
                            (*qfp).qf_lnum += amount_after;
                        }
                    }
                    i += 1;
                    qfp = (*qfp).qf_next;
                }
            }
            idx += 1;
        }
        return found_one;
    }
}

pub(crate) unsafe extern "C" fn qf_list_changed(mut qfl: *mut qf_list_T) {
    unsafe {
        (*qfl).qf_changedtick += 1;
    }
}

pub(crate) unsafe extern "C" fn qf_id2nr(
    qi: *const qf_info_T,
    qfid: ::core::ffi::c_uint,
) -> ::core::ffi::c_int {
    unsafe {
        let mut qf_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while qf_idx < (*qi).qf_listcount {
            if (*(*qi).qf_lists.offset(qf_idx as isize)).qf_id == qfid {
                return qf_idx;
            }
            qf_idx += 1;
        }
        return INVALID_QFIDX;
    }
}

pub(crate) unsafe extern "C" fn qf_restore_list(
    mut qi: *mut qf_info_T,
    mut save_qfid: ::core::ffi::c_uint,
) -> ::core::ffi::c_int {
    unsafe {
        if (*qf_get_curlist(qi)).qf_id == save_qfid {
            return OK;
        }
        let curlist: ::core::ffi::c_int = qf_id2nr(qi, save_qfid);
        if curlist < 0 as ::core::ffi::c_int {
            return FAIL;
        }
        (*qi).qf_curlist = curlist;
        return OK;
    }
}

pub(crate) unsafe extern "C" fn qf_free_stack(mut wp: *mut win_T, mut qi: *mut qf_info_T) {
    unsafe {
        let mut qfwin: *mut win_T = qf_find_win(qi);
        if !qfwin.is_null() {
            if (*qi).qf_curlist < (*qi).qf_listcount {
                qf_free(qf_get_curlist(qi));
            }
            qf_update_buffer(qi, ::core::ptr::null_mut::<qfline_T>());
        }
        if !wp.is_null()
            && (bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0
                && !(*wp).w_llist_ref.is_null())
        {
            let llwin: *mut win_T = qf_find_win_with_loclist(qi);
            if !llwin.is_null() {
                wp = llwin;
            }
        }
        qf_free_all(wp);
        if wp.is_null() {
            (*qi).qf_curlist = 0 as ::core::ffi::c_int;
            (*qi).qf_listcount = 0 as ::core::ffi::c_int;
        } else if !qfwin.is_null() {
            let mut new_ll: *mut qf_info_T = qf_alloc_stack(
                QFLT_LOCATION,
                (*wp).w_onebuf_opt.wo_lhi as ::core::ffi::c_int,
            );
            (*new_ll).qf_bufnr = (*(*qfwin).w_buffer).handle as ::core::ffi::c_int;
            ll_free_all(&raw mut (*qfwin).w_llist_ref);
            (*qfwin).w_llist_ref = new_ll;
            if wp != qfwin {
                win_set_loclist(wp, new_ll);
            }
        }
    }
}
