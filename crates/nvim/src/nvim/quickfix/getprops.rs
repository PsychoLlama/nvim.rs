//! Reading a list from Vimscript.
//!
//! [`qf_get_properties`] is `getqflist({what})`: [`qf_getprop_keys2flags`]
//! turns the requested keys into a flag set and one `qf_getprop_*` helper
//! answers each. [`get_errorlist`] is the plain, no-argument form, whose
//! entries [`get_qfline_items`] builds.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn get_qfline_items(
    mut qfp: *mut qfline_T,
    mut list: *mut list_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut bufnum: ::core::ffi::c_int = (*qfp).qf_fnum;
        if bufnum != 0 as ::core::ffi::c_int && buflist_findnr(bufnum).is_null() {
            bufnum = 0 as ::core::ffi::c_int;
        }
        let dict: *mut dict_T = tv_dict_alloc();
        tv_list_append_dict(list, dict);
        let mut buf: [::core::ffi::c_char; 2] = [0; 2];
        buf[0 as ::core::ffi::c_int as usize] = (*qfp).qf_type;
        buf[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        if tv_dict_add_nr(
            dict,
            b"bufnr\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            bufnum as varnumber_T,
        ) == FAIL
            || tv_dict_add_nr(
                dict,
                b"lnum\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                (*qfp).qf_lnum as varnumber_T,
            ) == FAIL
            || tv_dict_add_nr(
                dict,
                b"end_lnum\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                (*qfp).qf_end_lnum as varnumber_T,
            ) == FAIL
            || tv_dict_add_nr(
                dict,
                b"col\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
                (*qfp).qf_col as varnumber_T,
            ) == FAIL
            || tv_dict_add_nr(
                dict,
                b"end_col\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                (*qfp).qf_end_col as varnumber_T,
            ) == FAIL
            || tv_dict_add_nr(
                dict,
                b"vcol\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                (*qfp).qf_viscol as varnumber_T,
            ) == FAIL
            || tv_dict_add_nr(
                dict,
                b"nr\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
                (*qfp).qf_nr as varnumber_T,
            ) == FAIL
            || tv_dict_add_str(
                dict,
                b"module\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
                if (*qfp).qf_module.is_null() {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    (*qfp).qf_module as *const ::core::ffi::c_char
                },
            ) == FAIL
            || tv_dict_add_str(
                dict,
                b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                if (*qfp).qf_pattern.is_null() {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    (*qfp).qf_pattern as *const ::core::ffi::c_char
                },
            ) == FAIL
            || tv_dict_add_str(
                dict,
                b"text\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                if (*qfp).qf_text.is_null() {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    (*qfp).qf_text as *const ::core::ffi::c_char
                },
            ) == FAIL
            || tv_dict_add_str(
                dict,
                b"type\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                &raw mut buf as *mut ::core::ffi::c_char,
            ) == FAIL
            || (*qfp).qf_user_data.v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && tv_dict_add_tv(
                    dict,
                    b"user_data\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
                    &raw mut (*qfp).qf_user_data,
                ) == FAIL
            || tv_dict_add_nr(
                dict,
                b"valid\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                (*qfp).qf_valid as varnumber_T,
            ) == FAIL
        {
            abort();
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn get_errorlist(
    mut qi_arg: *mut qf_info_T,
    mut wp: *mut win_T,
    mut qf_idx: ::core::ffi::c_int,
    mut eidx: ::core::ffi::c_int,
    mut list: *mut list_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut qi: *mut qf_info_T = qi_arg;
        if qi.is_null() {
            qi = ql_info.get();
            if !wp.is_null() {
                qi = if bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0
                    && !(*wp).w_llist_ref.is_null()
                {
                    (*wp).w_llist_ref
                } else {
                    (*wp).w_llist
                };
            }
            if qi.is_null() {
                return FAIL;
            }
        }
        if eidx < 0 as ::core::ffi::c_int {
            return OK;
        }
        if qf_idx == INVALID_QFIDX {
            qf_idx = (*qi).qf_curlist;
        }
        if qf_idx >= (*qi).qf_listcount {
            return FAIL;
        }
        let mut qfl: *mut qf_list_T = qf_get_list(qi, qf_idx);
        if qf_list_empty(qfl) {
            return FAIL;
        }
        let mut qfp: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
        let mut i: ::core::ffi::c_int = 0;
        i = 1 as ::core::ffi::c_int;
        qfp = (*qfl).qf_start;
        while !got_int.get() && i <= (*qfl).qf_count && !qfp.is_null() {
            if eidx > 0 as ::core::ffi::c_int {
                if eidx == i {
                    return get_qfline_items(qfp, list);
                }
            } else if get_qfline_items(qfp, list) == FAIL {
                return FAIL;
            }
            i += 1;
            qfp = (*qfp).qf_next;
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn qf_get_list_from_lines(
    mut what: *mut dict_T,
    mut di: *mut dictitem_T,
    mut retdict: *mut dict_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut status: ::core::ffi::c_int = FAIL;
        if (*di).di_tv.v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*di).di_tv.vval.v_list.is_null()
        {
            return FAIL;
        }
        let mut errorformat: *mut ::core::ffi::c_char = p_efm.get();
        let mut efm_di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
        efm_di = tv_dict_find(
            what,
            b"efm\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !efm_di.is_null() {
            if (*efm_di).di_tv.v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*efm_di).di_tv.vval.v_string.is_null()
            {
                return FAIL;
            }
            errorformat = (*efm_di).di_tv.vval.v_string;
        }
        let mut l: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        let qi: *mut qf_info_T = qf_alloc_stack(QFLT_INTERNAL, 1 as ::core::ffi::c_int);
        if qf_init_ext(
            qi,
            0 as ::core::ffi::c_int,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<buf_T>(),
            &raw mut (*di).di_tv,
            errorformat,
            true_0 != 0,
            0 as linenr_T,
            0 as linenr_T,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ) > 0 as ::core::ffi::c_int
        {
            get_errorlist(
                qi,
                ::core::ptr::null_mut::<win_T>(),
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                l,
            );
            qf_free((*qi).qf_lists.offset(0 as ::core::ffi::c_int as isize));
        }
        qf_free_lists(qi);
        tv_dict_add_list(
            retdict,
            b"items\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            l,
        );
        status = OK;
        return status;
    }
}

pub(crate) unsafe extern "C" fn qf_winid(mut qi: *mut qf_info_T) -> ::core::ffi::c_int {
    unsafe {
        if qi.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        let mut win: *mut win_T = qf_find_win(qi);
        if !win.is_null() {
            return (*win).handle as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn qf_getprop_qfbufnr(
    mut qi: *const qf_info_T,
    mut retdict: *mut dict_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut bufnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if !qi.is_null() && !buflist_findnr((*qi).qf_bufnr).is_null() {
            bufnum = (*qi).qf_bufnr;
        }
        return tv_dict_add_nr(
            retdict,
            b"qfbufnr\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            bufnum as varnumber_T,
        );
    }
}

pub(crate) unsafe extern "C" fn qf_getprop_keys2flags(
    mut what: *const dict_T,
    mut loclist: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut flags: ::core::ffi::c_int = QF_GETLIST_NONE as ::core::ffi::c_int;
        if !tv_dict_find(
            what,
            b"all\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        )
        .is_null()
        {
            flags |= QF_GETLIST_ALL as ::core::ffi::c_int;
            if !loclist {
                flags &= !(QF_GETLIST_FILEWINID as ::core::ffi::c_int);
            }
        }
        if !tv_dict_find(
            what,
            b"title\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        )
        .is_null()
        {
            flags |= QF_GETLIST_TITLE as ::core::ffi::c_int;
        }
        if !tv_dict_find(
            what,
            b"nr\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        )
        .is_null()
        {
            flags |= QF_GETLIST_NR as ::core::ffi::c_int;
        }
        if !tv_dict_find(
            what,
            b"winid\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        )
        .is_null()
        {
            flags |= QF_GETLIST_WINID as ::core::ffi::c_int;
        }
        if !tv_dict_find(
            what,
            b"context\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        )
        .is_null()
        {
            flags |= QF_GETLIST_CONTEXT as ::core::ffi::c_int;
        }
        if !tv_dict_find(
            what,
            b"id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        )
        .is_null()
        {
            flags |= QF_GETLIST_ID as ::core::ffi::c_int;
        }
        if !tv_dict_find(
            what,
            b"items\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        )
        .is_null()
        {
            flags |= QF_GETLIST_ITEMS as ::core::ffi::c_int;
        }
        if !tv_dict_find(
            what,
            b"idx\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        )
        .is_null()
        {
            flags |= QF_GETLIST_IDX as ::core::ffi::c_int;
        }
        if !tv_dict_find(
            what,
            b"size\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        )
        .is_null()
        {
            flags |= QF_GETLIST_SIZE as ::core::ffi::c_int;
        }
        if !tv_dict_find(
            what,
            b"changedtick\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        )
        .is_null()
        {
            flags |= QF_GETLIST_TICK as ::core::ffi::c_int;
        }
        if loclist as ::core::ffi::c_int != 0
            && !tv_dict_find(
                what,
                b"filewinid\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
            )
            .is_null()
        {
            flags |= QF_GETLIST_FILEWINID as ::core::ffi::c_int;
        }
        if !tv_dict_find(
            what,
            b"qfbufnr\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        )
        .is_null()
        {
            flags |= QF_GETLIST_QFBUFNR as ::core::ffi::c_int;
        }
        if !tv_dict_find(
            what,
            b"quickfixtextfunc\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        )
        .is_null()
        {
            flags |= QF_GETLIST_QFTF as ::core::ffi::c_int;
        }
        return flags;
    }
}

pub(crate) unsafe extern "C" fn qf_getprop_qfidx(
    mut qi: *mut qf_info_T,
    mut what: *mut dict_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
        let mut qf_idx: ::core::ffi::c_int = (*qi).qf_curlist;
        di = tv_dict_find(
            what,
            b"nr\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !di.is_null() {
            if (*di).di_tv.v_type as ::core::ffi::c_uint
                == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if (*di).di_tv.vval.v_number != 0 as varnumber_T {
                    qf_idx =
                        (*di).di_tv.vval.v_number as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
                    if qf_idx < 0 as ::core::ffi::c_int || qf_idx >= (*qi).qf_listcount {
                        qf_idx = INVALID_QFIDX;
                    }
                }
            } else if (*di).di_tv.v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                && strequal(
                    (*di).di_tv.vval.v_string,
                    b"$\0".as_ptr() as *const ::core::ffi::c_char,
                ) as ::core::ffi::c_int
                    != 0
            {
                qf_idx = (*qi).qf_listcount - 1 as ::core::ffi::c_int;
            } else {
                qf_idx = INVALID_QFIDX;
            }
        }
        di = tv_dict_find(
            what,
            b"id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !di.is_null() {
            if (*di).di_tv.v_type as ::core::ffi::c_uint
                == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if (*di).di_tv.vval.v_number != 0 as varnumber_T {
                    qf_idx = qf_id2nr(qi, (*di).di_tv.vval.v_number as ::core::ffi::c_uint);
                }
            } else {
                qf_idx = INVALID_QFIDX;
            }
        }
        return qf_idx;
    }
}

pub(crate) unsafe extern "C" fn qf_getprop_defaults(
    mut qi: *mut qf_info_T,
    mut flags: ::core::ffi::c_int,
    mut locstack: ::core::ffi::c_int,
    mut retdict: *mut dict_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut status: ::core::ffi::c_int = OK;
        if flags & QF_GETLIST_TITLE as ::core::ffi::c_int != 0 {
            status = tv_dict_add_str(
                retdict,
                b"title\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                b"\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if status == OK && flags & QF_GETLIST_ITEMS as ::core::ffi::c_int != 0 {
            let mut l: *mut list_T =
                tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
            status = tv_dict_add_list(
                retdict,
                b"items\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                l,
            );
        }
        if status == OK && flags & QF_GETLIST_NR as ::core::ffi::c_int != 0 {
            status = tv_dict_add_nr(
                retdict,
                b"nr\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
                0 as varnumber_T,
            );
        }
        if status == OK && flags & QF_GETLIST_WINID as ::core::ffi::c_int != 0 {
            status = tv_dict_add_nr(
                retdict,
                b"winid\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                qf_winid(qi) as varnumber_T,
            );
        }
        if status == OK && flags & QF_GETLIST_CONTEXT as ::core::ffi::c_int != 0 {
            status = tv_dict_add_str(
                retdict,
                b"context\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                b"\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if status == OK && flags & QF_GETLIST_ID as ::core::ffi::c_int != 0 {
            status = tv_dict_add_nr(
                retdict,
                b"id\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
                0 as varnumber_T,
            );
        }
        if status == OK && flags & QF_GETLIST_IDX as ::core::ffi::c_int != 0 {
            status = tv_dict_add_nr(
                retdict,
                b"idx\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
                0 as varnumber_T,
            );
        }
        if status == OK && flags & QF_GETLIST_SIZE as ::core::ffi::c_int != 0 {
            status = tv_dict_add_nr(
                retdict,
                b"size\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                0 as varnumber_T,
            );
        }
        if status == OK && flags & QF_GETLIST_TICK as ::core::ffi::c_int != 0 {
            status = tv_dict_add_nr(
                retdict,
                b"changedtick\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
                0 as varnumber_T,
            );
        }
        if status == OK && locstack != 0 && flags & QF_GETLIST_FILEWINID as ::core::ffi::c_int != 0
        {
            status = tv_dict_add_nr(
                retdict,
                b"filewinid\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
                0 as varnumber_T,
            );
        }
        if status == OK && flags & QF_GETLIST_QFBUFNR as ::core::ffi::c_int != 0 {
            status = qf_getprop_qfbufnr(qi, retdict);
        }
        if status == OK && flags & QF_GETLIST_QFTF as ::core::ffi::c_int != 0 {
            status = tv_dict_add_str(
                retdict,
                b"quickfixtextfunc\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as size_t),
                b"\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        return status;
    }
}

pub(crate) unsafe extern "C" fn qf_getprop_title(
    mut qfl: *mut qf_list_T,
    mut retdict: *mut dict_T,
) -> ::core::ffi::c_int {
    unsafe {
        return tv_dict_add_str(
            retdict,
            b"title\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            (*qfl).qf_title,
        );
    }
}

pub(crate) unsafe extern "C" fn qf_getprop_filewinid(
    mut wp: *const win_T,
    mut qi: *const qf_info_T,
    mut retdict: *mut dict_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut winid: handle_T = 0 as handle_T;
        if !wp.is_null()
            && (bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0
                && !(*wp).w_llist_ref.is_null())
        {
            let mut ll_wp: *mut win_T = qf_find_win_with_loclist(qi);
            if !ll_wp.is_null() {
                winid = (*ll_wp).handle;
            }
        }
        return tv_dict_add_nr(
            retdict,
            b"filewinid\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            winid as varnumber_T,
        );
    }
}

pub(crate) unsafe extern "C" fn qf_getprop_items(
    mut qi: *mut qf_info_T,
    mut qf_idx: ::core::ffi::c_int,
    mut eidx: ::core::ffi::c_int,
    mut retdict: *mut dict_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut l: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        get_errorlist(qi, ::core::ptr::null_mut::<win_T>(), qf_idx, eidx, l);
        tv_dict_add_list(
            retdict,
            b"items\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            l,
        );
        return OK;
    }
}

pub(crate) unsafe extern "C" fn qf_getprop_ctx(
    mut qfl: *mut qf_list_T,
    mut retdict: *mut dict_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut status: ::core::ffi::c_int = 0;
        if !(*qfl).qf_ctx.is_null() {
            let mut di: *mut dictitem_T = tv_dict_item_alloc_len(
                b"context\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            );
            tv_copy((*qfl).qf_ctx, &raw mut (*di).di_tv);
            status = tv_dict_add(retdict, di);
            if status == FAIL {
                tv_dict_item_free(di);
            }
        } else {
            status = tv_dict_add_str(
                retdict,
                b"context\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                b"\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        return status;
    }
}

pub(crate) unsafe extern "C" fn qf_getprop_idx(
    mut qfl: *mut qf_list_T,
    mut eidx: ::core::ffi::c_int,
    mut retdict: *mut dict_T,
) -> ::core::ffi::c_int {
    unsafe {
        if eidx == 0 as ::core::ffi::c_int {
            eidx = (*qfl).qf_index;
            if qf_list_empty(qfl) {
                eidx = 0 as ::core::ffi::c_int;
            }
        }
        return tv_dict_add_nr(
            retdict,
            b"idx\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
            eidx as varnumber_T,
        );
    }
}

pub(crate) unsafe extern "C" fn qf_getprop_qftf(
    mut qfl: *mut qf_list_T,
    mut retdict: *mut dict_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut status: ::core::ffi::c_int = 0;
        if (*qfl).qf_qftf_cb.type_0 as ::core::ffi::c_uint
            != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut tv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            callback_put(&raw mut (*qfl).qf_qftf_cb, &raw mut tv);
            status = tv_dict_add_tv(
                retdict,
                b"quickfixtextfunc\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as size_t),
                &raw mut tv,
            );
            tv_clear(&raw mut tv);
        } else {
            status = tv_dict_add_str(
                retdict,
                b"quickfixtextfunc\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as size_t),
                b"\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        return status;
    }
}

pub(crate) unsafe extern "C" fn qf_get_properties(
    mut wp: *mut win_T,
    mut what: *mut dict_T,
    mut retdict: *mut dict_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut qi: *mut qf_info_T = ql_info.get();
        '_c2rust_label: {
            if !qi.is_null() {
            } else {
                __assert_fail(
                    b"qi != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    6512 as ::core::ffi::c_uint,
                    b"int qf_get_properties(win_T *, dict_T *, dict_T *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
        let mut status: ::core::ffi::c_int = OK;
        let mut qf_idx: ::core::ffi::c_int = INVALID_QFIDX;
        di = tv_dict_find(
            what,
            b"lines\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !di.is_null() {
            return qf_get_list_from_lines(what, di, retdict);
        }
        if !wp.is_null() {
            qi = if bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0
                && !(*wp).w_llist_ref.is_null()
            {
                (*wp).w_llist_ref
            } else {
                (*wp).w_llist
            };
        }
        let flags: ::core::ffi::c_int = qf_getprop_keys2flags(what, !wp.is_null());
        if !qf_stack_empty(qi) {
            qf_idx = qf_getprop_qfidx(qi, what);
        }
        if qf_stack_empty(qi) as ::core::ffi::c_int != 0 || qf_idx == INVALID_QFIDX {
            return qf_getprop_defaults(qi, flags, !wp.is_null() as ::core::ffi::c_int, retdict);
        }
        let mut qfl: *mut qf_list_T = qf_get_list(qi, qf_idx);
        let mut eidx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        di = tv_dict_find(
            what,
            b"idx\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !di.is_null() {
            if (*di).di_tv.v_type as ::core::ffi::c_uint
                != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return FAIL;
            }
            eidx = (*di).di_tv.vval.v_number as ::core::ffi::c_int;
        }
        if flags & QF_GETLIST_TITLE as ::core::ffi::c_int != 0 {
            status = qf_getprop_title(qfl, retdict);
        }
        if status == OK && flags & QF_GETLIST_NR as ::core::ffi::c_int != 0 {
            status = tv_dict_add_nr(
                retdict,
                b"nr\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
                (qf_idx + 1 as ::core::ffi::c_int) as varnumber_T,
            );
        }
        if status == OK && flags & QF_GETLIST_WINID as ::core::ffi::c_int != 0 {
            status = tv_dict_add_nr(
                retdict,
                b"winid\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                qf_winid(qi) as varnumber_T,
            );
        }
        if status == OK && flags & QF_GETLIST_ITEMS as ::core::ffi::c_int != 0 {
            status = qf_getprop_items(qi, qf_idx, eidx, retdict);
        }
        if status == OK && flags & QF_GETLIST_CONTEXT as ::core::ffi::c_int != 0 {
            status = qf_getprop_ctx(qfl, retdict);
        }
        if status == OK && flags & QF_GETLIST_ID as ::core::ffi::c_int != 0 {
            status = tv_dict_add_nr(
                retdict,
                b"id\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
                (*qfl).qf_id as varnumber_T,
            );
        }
        if status == OK && flags & QF_GETLIST_IDX as ::core::ffi::c_int != 0 {
            status = qf_getprop_idx(qfl, eidx, retdict);
        }
        if status == OK && flags & QF_GETLIST_SIZE as ::core::ffi::c_int != 0 {
            status = tv_dict_add_nr(
                retdict,
                b"size\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                (*qfl).qf_count as varnumber_T,
            );
        }
        if status == OK && flags & QF_GETLIST_TICK as ::core::ffi::c_int != 0 {
            status = tv_dict_add_nr(
                retdict,
                b"changedtick\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
                (*qfl).qf_changedtick as varnumber_T,
            );
        }
        if status == OK && !wp.is_null() && flags & QF_GETLIST_FILEWINID as ::core::ffi::c_int != 0
        {
            status = qf_getprop_filewinid(wp, qi, retdict);
        }
        if status == OK && flags & QF_GETLIST_QFBUFNR as ::core::ffi::c_int != 0 {
            status = qf_getprop_qfbufnr(qi, retdict);
        }
        if status == OK && flags & QF_GETLIST_QFTF as ::core::ffi::c_int != 0 {
            status = qf_getprop_qftf(qfl, retdict);
        }
        return status;
    }
}
