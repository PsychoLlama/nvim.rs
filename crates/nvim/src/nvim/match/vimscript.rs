//! The `match*()` Vimscript functions.
//!
//! `matchadd()`/`matchaddpos()`/`matchdelete()`/`clearmatches()` are thin
//! wrappers over the list operations in the parent; `getmatches()` and
//! `setmatches()` are the dictionary round trip that lets a match list be
//! saved and restored, including the `pos1`..`pos8` keys a position match
//! is described by.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn matchadd_dict_arg(
    mut tv: *mut typval_T,
    mut conceal_char: *mut *const ::core::ffi::c_char,
    mut win: *mut *mut win_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
        if (*tv).v_type as ::core::ffi::c_uint
            != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(&raw const e_dictreq as *const ::core::ffi::c_char));
            return FAIL;
        }
        di = tv_dict_find(
            (*tv).vval.v_dict,
            b"conceal\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !di.is_null() {
            *conceal_char = tv_get_string(&raw mut (*di).di_tv);
        }
        di = tv_dict_find(
            (*tv).vval.v_dict,
            b"window\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if di.is_null() {
            return OK;
        }
        *win = find_win_by_nr_or_id(&raw mut (*di).di_tv);
        if (*win).is_null() {
            emsg(gettext(
                &raw const e_invalwindow as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn f_clearmatches(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut win: *mut win_T = get_optional_window(argvars, 0 as ::core::ffi::c_int);
        if !win.is_null() {
            clear_matches(win);
        }
    }
}

pub unsafe extern "C" fn f_getmatches(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut win: *mut win_T = get_optional_window(argvars, 0 as ::core::ffi::c_int);
        tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        if win.is_null() {
            return;
        }
        let mut cur: *mut matchitem_T = (*win).w_match_head;
        while !cur.is_null() {
            let mut dict: *mut dict_T = tv_dict_alloc();
            if (*cur).mit_match.regprog.is_null() {
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < (*cur).mit_pos_count {
                    let mut llpos: *mut llpos_T = ::core::ptr::null_mut::<llpos_T>();
                    let mut buf: [::core::ffi::c_char; 30] = [0; 30];
                    llpos = (*cur).mit_pos_array.offset(i as isize);
                    if (*llpos).lnum == 0 as linenr_T {
                        break;
                    }
                    let l: *mut list_T = tv_list_alloc(
                        (1 as ::core::ffi::c_int
                            + (if (*llpos).col > 0 as ::core::ffi::c_int {
                                2 as ::core::ffi::c_int
                            } else {
                                0 as ::core::ffi::c_int
                            })) as ptrdiff_t,
                    );
                    tv_list_append_number(l, (*llpos).lnum as varnumber_T);
                    if (*llpos).col > 0 as ::core::ffi::c_int {
                        tv_list_append_number(l, (*llpos).col as varnumber_T);
                        tv_list_append_number(l, (*llpos).len as varnumber_T);
                    }
                    let mut len: ::core::ffi::c_int = snprintf(
                        &raw mut buf as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 30]>(),
                        b"pos%d\0".as_ptr() as *const ::core::ffi::c_char,
                        i + 1 as ::core::ffi::c_int,
                    );
                    '_c2rust_label: {
                        if (len as size_t) < ::core::mem::size_of::<[::core::ffi::c_char; 30]>() {
                        } else {
                            __assert_fail(
                                b"(size_t)len < sizeof(buf)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/match.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                898 as ::core::ffi::c_uint,
                                __ASSERT_FUNCTION.as_ptr(),
                            );
                        }
                    };
                    tv_dict_add_list(
                        dict,
                        &raw mut buf as *mut ::core::ffi::c_char,
                        len as size_t,
                        l,
                    );
                    i += 1;
                }
            } else {
                tv_dict_add_str(
                    dict,
                    b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                    (*cur).mit_pattern,
                );
            }
            tv_dict_add_str(
                dict,
                b"group\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                syn_id2name((*cur).mit_hlg_id),
            );
            tv_dict_add_nr(
                dict,
                b"priority\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                (*cur).mit_priority as varnumber_T,
            );
            tv_dict_add_nr(
                dict,
                b"id\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
                (*cur).mit_id as varnumber_T,
            );
            if (*cur).mit_conceal_char != 0 {
                let mut buf_0: [::core::ffi::c_char; 7] = [0; 7];
                buf_0[utf_char2bytes(
                    (*cur).mit_conceal_char,
                    &raw mut buf_0 as *mut ::core::ffi::c_char,
                ) as usize] = NUL as ::core::ffi::c_char;
                tv_dict_add_str(
                    dict,
                    b"conceal\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                    &raw mut buf_0 as *mut ::core::ffi::c_char,
                );
            }
            tv_list_append_dict((*rettv).vval.v_list, dict);
            cur = (*cur).mit_next;
        }
    }
}

pub unsafe extern "C" fn f_setmatches(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut d: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
        let mut s: *mut list_T = ::core::ptr::null_mut::<list_T>();
        let mut win: *mut win_T = get_optional_window(argvars, 1 as ::core::ffi::c_int);
        (*rettv).vval.v_number = -1 as varnumber_T;
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
            return;
        }
        if win.is_null() {
            return;
        }
        let l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list;
        let mut li_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let l_: *const list_T = l;
        if !l_.is_null() {
            let mut li: *const listitem_T = (*l_).lv_first;
            while !li.is_null() {
                if (*li).li_tv.v_type as ::core::ffi::c_uint
                    != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                    || {
                        d = (*li).li_tv.vval.v_dict;
                        d.is_null()
                    }
                {
                    semsg(
                        gettext(
                            b"E474: List item %d is either not a dictionary or an empty one\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        li_idx,
                    );
                    return;
                }
                if !(!tv_dict_find(
                    d,
                    b"group\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
                        as ptrdiff_t,
                )
                .is_null()
                    && (!tv_dict_find(
                        d,
                        b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize)
                            as ptrdiff_t,
                    )
                    .is_null()
                        || !tv_dict_find(
                            d,
                            b"pos1\0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                                .wrapping_sub(1 as usize) as ptrdiff_t,
                        )
                        .is_null())
                    && !tv_dict_find(
                        d,
                        b"priority\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as usize)
                            as ptrdiff_t,
                    )
                    .is_null()
                    && !tv_dict_find(
                        d,
                        b"id\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as usize)
                            as ptrdiff_t,
                    )
                    .is_null())
                {
                    semsg(
                        gettext(
                            b"E474: List item %d is missing one of the required keys\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        li_idx,
                    );
                    return;
                }
                li_idx += 1;
                li = (*li).li_next;
            }
        }
        clear_matches(win);
        let mut match_add_failed: bool = false_0 != 0;
        let l__0: *const list_T = l;
        if !l__0.is_null() {
            let mut li_0: *const listitem_T = (*l__0).lv_first;
            while !li_0.is_null() {
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                d = (*li_0).li_tv.vval.v_dict;
                let di: *mut dictitem_T = tv_dict_find(
                    d,
                    b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize)
                        as ptrdiff_t,
                );
                if di.is_null() {
                    if s.is_null() {
                        s = tv_list_alloc(9 as ptrdiff_t);
                    }
                    i = 1 as ::core::ffi::c_int;
                    while i < 9 as ::core::ffi::c_int {
                        let mut buf: [::core::ffi::c_char; 30] = [0; 30];
                        snprintf(
                            &raw mut buf as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 30]>(),
                            b"pos%d\0".as_ptr() as *const ::core::ffi::c_char,
                            i,
                        );
                        let pos_di: *mut dictitem_T = tv_dict_find(
                            d,
                            &raw mut buf as *mut ::core::ffi::c_char,
                            -1 as ptrdiff_t,
                        );
                        if pos_di.is_null() {
                            break;
                        }
                        if (*pos_di).di_tv.v_type as ::core::ffi::c_uint
                            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            return;
                        }
                        tv_list_append_tv(s, &raw mut (*pos_di).di_tv);
                        tv_list_ref(s);
                        i += 1;
                    }
                }
                let mut group_buf: [::core::ffi::c_char; 65] = [0; 65];
                let group: *const ::core::ffi::c_char = tv_dict_get_string_buf(
                    d,
                    b"group\0".as_ptr() as *const ::core::ffi::c_char,
                    &raw mut group_buf as *mut ::core::ffi::c_char,
                );
                let priority: ::core::ffi::c_int =
                    tv_dict_get_number(d, b"priority\0".as_ptr() as *const ::core::ffi::c_char)
                        as ::core::ffi::c_int;
                let id: ::core::ffi::c_int =
                    tv_dict_get_number(d, b"id\0".as_ptr() as *const ::core::ffi::c_char)
                        as ::core::ffi::c_int;
                let conceal_di: *mut dictitem_T = tv_dict_find(
                    d,
                    b"conceal\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize)
                        as ptrdiff_t,
                );
                let conceal: *const ::core::ffi::c_char = if !conceal_di.is_null() {
                    tv_get_string(&raw mut (*conceal_di).di_tv)
                } else {
                    ::core::ptr::null::<::core::ffi::c_char>()
                };
                if i == 0 as ::core::ffi::c_int {
                    if match_add(
                        win,
                        group,
                        tv_dict_get_string(
                            d,
                            b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                            false,
                        ),
                        priority,
                        id,
                        ::core::ptr::null_mut::<list_T>(),
                        conceal,
                    ) != id
                    {
                        match_add_failed = true;
                    }
                } else {
                    if match_add(
                        win,
                        group,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                        priority,
                        id,
                        s,
                        conceal,
                    ) != id
                    {
                        match_add_failed = true;
                    }
                    tv_list_unref(s);
                    s = ::core::ptr::null_mut::<list_T>();
                }
                li_0 = (*li_0).li_next;
            }
        }
        if !match_add_failed {
            (*rettv).vval.v_number = 0 as varnumber_T;
        }
    }
}

pub unsafe extern "C" fn f_matchadd(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut grpbuf: [::core::ffi::c_char; 65] = [0; 65];
        let mut patbuf: [::core::ffi::c_char; 65] = [0; 65];
        let grp: *const ::core::ffi::c_char = tv_get_string_buf_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut grpbuf as *mut ::core::ffi::c_char,
        );
        let pat: *const ::core::ffi::c_char = tv_get_string_buf_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut patbuf as *mut ::core::ffi::c_char,
        );
        let mut prio: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
        let mut id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut error: bool = false_0 != 0;
        let mut conceal_char: *const ::core::ffi::c_char =
            ::core::ptr::null::<::core::ffi::c_char>();
        let mut win: *mut win_T = curwin.get();
        (*rettv).vval.v_number = -1 as varnumber_T;
        if grp.is_null() || pat.is_null() {
            return;
        }
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            prio = tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) as ::core::ffi::c_int;
            if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                id = tv_get_number_chk(
                    argvars.offset(3 as ::core::ffi::c_int as isize),
                    &raw mut error,
                ) as ::core::ffi::c_int;
                if (*argvars.offset(4 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                    && matchadd_dict_arg(
                        argvars.offset(4 as ::core::ffi::c_int as isize),
                        &raw mut conceal_char,
                        &raw mut win,
                    ) == FAIL
                {
                    return;
                }
            }
        }
        if error {
            return;
        }
        if id >= 1 as ::core::ffi::c_int && id <= 3 as ::core::ffi::c_int {
            semsg(
                gettext(b"E798: ID is reserved for \":match\": %d\0".as_ptr()
                    as *const ::core::ffi::c_char),
                id,
            );
            return;
        }
        (*rettv).vval.v_number = match_add(
            win,
            grp,
            pat,
            prio,
            id,
            ::core::ptr::null_mut::<list_T>(),
            conceal_char,
        ) as varnumber_T;
    }
}

pub unsafe extern "C" fn f_matchaddpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number = -1 as varnumber_T;
        let mut buf: [::core::ffi::c_char; 65] = [0; 65];
        let group: *const ::core::ffi::c_char = tv_get_string_buf_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        if group.is_null() {
            return;
        }
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            semsg(
                gettext(&raw const e_listarg as *const ::core::ffi::c_char),
                b"matchaddpos()\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
        let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
        l = (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_list;
        if tv_list_len(l) == 0 as ::core::ffi::c_int {
            return;
        }
        let mut error: bool = false_0 != 0;
        let mut prio: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
        let mut id: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut conceal_char: *const ::core::ffi::c_char =
            ::core::ptr::null::<::core::ffi::c_char>();
        let mut win: *mut win_T = curwin.get();
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            prio = tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) as ::core::ffi::c_int;
            if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                id = tv_get_number_chk(
                    argvars.offset(3 as ::core::ffi::c_int as isize),
                    &raw mut error,
                ) as ::core::ffi::c_int;
                if (*argvars.offset(4 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                    && matchadd_dict_arg(
                        argvars.offset(4 as ::core::ffi::c_int as isize),
                        &raw mut conceal_char,
                        &raw mut win,
                    ) == FAIL
                {
                    return;
                }
            }
        }
        if error as ::core::ffi::c_int == true_0 {
            return;
        }
        if id == 1 as ::core::ffi::c_int || id == 2 as ::core::ffi::c_int {
            semsg(
                gettext(b"E798: ID is reserved for \"match\": %d\0".as_ptr()
                    as *const ::core::ffi::c_char),
                id,
            );
            return;
        }
        (*rettv).vval.v_number = match_add(
            win,
            group,
            ::core::ptr::null::<::core::ffi::c_char>(),
            prio,
            id,
            l,
            conceal_char,
        ) as varnumber_T;
    }
}

pub unsafe extern "C" fn f_matcharg(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let id: ::core::ffi::c_int =
            tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
        tv_list_alloc_ret(
            rettv,
            (if id >= 1 as ::core::ffi::c_int && id <= 3 as ::core::ffi::c_int {
                2 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as ptrdiff_t,
        );
        if id >= 1 as ::core::ffi::c_int && id <= 3 as ::core::ffi::c_int {
            let m: *mut matchitem_T = get_match(curwin.get(), id);
            if !m.is_null() {
                tv_list_append_string(
                    (*rettv).vval.v_list,
                    syn_id2name((*m).mit_hlg_id),
                    -1 as ssize_t,
                );
                tv_list_append_string((*rettv).vval.v_list, (*m).mit_pattern, -1 as ssize_t);
            } else {
                tv_list_append_string(
                    (*rettv).vval.v_list,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    0 as ssize_t,
                );
                tv_list_append_string(
                    (*rettv).vval.v_list,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    0 as ssize_t,
                );
            }
        }
    }
}

pub unsafe extern "C" fn f_matchdelete(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut win: *mut win_T = get_optional_window(argvars, 1 as ::core::ffi::c_int);
        if win.is_null() {
            (*rettv).vval.v_number = -1 as varnumber_T;
        } else {
            (*rettv).vval.v_number = match_delete(
                win,
                tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize))
                    as ::core::ffi::c_int,
                true_0 != 0,
            ) as varnumber_T;
        };
    }
}
