//! Writing a list from Vimscript.
//!
//! [`set_errorlist`] is `setqflist()`: a list of dictionaries goes through
//! [`qf_add_entries`] and [`qf_add_entry_from_dict`], and a `what`
//! dictionary through [`qf_set_properties`] and the `qf_setprop_*`
//! helpers.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn qf_setprop_qftf(
    mut qfl: *mut qf_list_T,
    mut di: *mut dictitem_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut cb: Callback = Callback {
            data: C2Rust_Unnamed_6 {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        };
        if check_secure() {
            return FAIL;
        }
        callback_free(&raw mut (*qfl).qf_qftf_cb);
        if callback_from_typval(&raw mut cb, &raw mut (*di).di_tv) {
            (*qfl).qf_qftf_cb = cb;
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn qf_add_entry_from_dict(
    mut qfl: *mut qf_list_T,
    mut d: *mut dict_T,
    mut first_entry: bool,
    mut valid_entry: *mut bool,
) -> ::core::ffi::c_int {
    unsafe {
        static did_bufnr_emsg: GlobalCell<bool> = GlobalCell::new(false);
        if first_entry {
            did_bufnr_emsg.set(false_0 != 0);
        }
        let filename: *mut ::core::ffi::c_char = tv_dict_get_string(
            d,
            b"filename\0".as_ptr() as *const ::core::ffi::c_char,
            true_0 != 0,
        );
        let module: *mut ::core::ffi::c_char = tv_dict_get_string(
            d,
            b"module\0".as_ptr() as *const ::core::ffi::c_char,
            true_0 != 0,
        );
        let mut bufnum: ::core::ffi::c_int =
            tv_dict_get_number(d, b"bufnr\0".as_ptr() as *const ::core::ffi::c_char)
                as ::core::ffi::c_int;
        let lnum: linenr_T =
            tv_dict_get_number(d, b"lnum\0".as_ptr() as *const ::core::ffi::c_char) as linenr_T;
        let end_lnum: linenr_T =
            tv_dict_get_number(d, b"end_lnum\0".as_ptr() as *const ::core::ffi::c_char) as linenr_T;
        let col: ::core::ffi::c_int =
            tv_dict_get_number(d, b"col\0".as_ptr() as *const ::core::ffi::c_char)
                as ::core::ffi::c_int;
        let end_col: ::core::ffi::c_int =
            tv_dict_get_number(d, b"end_col\0".as_ptr() as *const ::core::ffi::c_char)
                as ::core::ffi::c_int;
        let vcol: ::core::ffi::c_char =
            tv_dict_get_number(d, b"vcol\0".as_ptr() as *const ::core::ffi::c_char)
                as ::core::ffi::c_char;
        let nr: ::core::ffi::c_int =
            tv_dict_get_number(d, b"nr\0".as_ptr() as *const ::core::ffi::c_char)
                as ::core::ffi::c_int;
        let type_0: *const ::core::ffi::c_char = tv_dict_get_string(
            d,
            b"type\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        let pattern: *mut ::core::ffi::c_char = tv_dict_get_string(
            d,
            b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
            true_0 != 0,
        );
        let mut text: *mut ::core::ffi::c_char = tv_dict_get_string(
            d,
            b"text\0".as_ptr() as *const ::core::ffi::c_char,
            true_0 != 0,
        );
        if text.is_null() {
            text = xcalloc(1 as size_t, 1 as size_t) as *mut ::core::ffi::c_char;
        }
        let mut user_data: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        tv_dict_get_tv(
            d,
            b"user_data\0".as_ptr() as *const ::core::ffi::c_char,
            &raw mut user_data,
        );
        let mut valid: bool = true_0 != 0;
        if filename.is_null() && bufnum == 0 as ::core::ffi::c_int
            || lnum == 0 as linenr_T && pattern.is_null()
        {
            valid = false_0 != 0;
        }
        if bufnum != 0 as ::core::ffi::c_int && buflist_findnr(bufnum).is_null() {
            if !did_bufnr_emsg.get() {
                did_bufnr_emsg.set(true_0 != 0);
                semsg(
                    gettext(b"E92: Buffer %d not found\0".as_ptr() as *const ::core::ffi::c_char),
                    bufnum,
                );
            }
            valid = false_0 != 0;
            bufnum = 0 as ::core::ffi::c_int;
        }
        if !tv_dict_find(
            d,
            b"valid\0".as_ptr() as *const ::core::ffi::c_char,
            -1 as ptrdiff_t,
        )
        .is_null()
        {
            valid = tv_dict_get_bool(
                d,
                b"valid\0".as_ptr() as *const ::core::ffi::c_char,
                false_0,
            ) != 0;
        }
        qf_add_entry(
            qfl,
            &NewEntry {
                fname: filename,
                module,
                bufnum,
                lnum,
                end_lnum,
                col,
                end_col,
                vis_col: vcol,
                pattern,
                nr,
                kind: (if type_0.is_null() {
                    NUL
                } else {
                    *type_0 as ::core::ffi::c_int
                }) as ::core::ffi::c_char,
                user_data: &raw mut user_data,
                valid,
                ..NewEntry::new(text)
            },
        );
        xfree(filename as *mut ::core::ffi::c_void);
        xfree(module as *mut ::core::ffi::c_void);
        xfree(pattern as *mut ::core::ffi::c_void);
        xfree(text as *mut ::core::ffi::c_void);
        tv_clear(&raw mut user_data);
        if valid {
            *valid_entry = true_0 != 0;
        }
        return QF_OK as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn entry_is_closer_to_target(
    mut entry: *mut qfline_T,
    mut other_entry: *mut qfline_T,
    mut target_fnum: ::core::ffi::c_int,
    mut target_lnum: ::core::ffi::c_int,
    mut target_col: ::core::ffi::c_int,
) -> bool {
    unsafe {
        if target_fnum == 0 {
            return false_0 != 0;
        }
        let mut is_target_file: bool = (*entry).qf_fnum != 0 && (*entry).qf_fnum == target_fnum;
        let mut other_is_target_file: bool =
            (*other_entry).qf_fnum != 0 && (*other_entry).qf_fnum == target_fnum;
        if !is_target_file && other_is_target_file as ::core::ffi::c_int != 0 {
            return false_0 != 0;
        } else if is_target_file as ::core::ffi::c_int != 0 && !other_is_target_file {
            return true_0 != 0;
        }
        if target_lnum == 0 {
            return false_0 != 0;
        }
        let mut line_distance: ::core::ffi::c_int = if (*entry).qf_lnum != 0 {
            abs((*entry).qf_lnum as ::core::ffi::c_int - target_lnum)
        } else {
            INT_MAX
        };
        let mut other_line_distance: ::core::ffi::c_int = if (*other_entry).qf_lnum != 0 {
            abs((*other_entry).qf_lnum as ::core::ffi::c_int - target_lnum)
        } else {
            INT_MAX
        };
        if line_distance > other_line_distance {
            return false_0 != 0;
        } else if line_distance < other_line_distance {
            return true_0 != 0;
        }
        if target_col == 0 {
            return false_0 != 0;
        }
        let mut column_distance: ::core::ffi::c_int = if (*entry).qf_col != 0 {
            abs((*entry).qf_col - target_col)
        } else {
            INT_MAX
        };
        let mut other_column_distance: ::core::ffi::c_int = if (*other_entry).qf_col != 0 {
            abs((*other_entry).qf_col - target_col)
        } else {
            INT_MAX
        };
        if column_distance > other_column_distance {
            return false_0 != 0;
        } else if column_distance < other_column_distance {
            return true_0 != 0;
        }
        return false_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn qf_add_entries(
    mut qi: *mut qf_info_T,
    mut qf_idx: ::core::ffi::c_int,
    mut list: *mut list_T,
    mut title: *mut ::core::ffi::c_char,
    mut action: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut qfl: *mut qf_list_T = qf_get_list(qi, qf_idx);
        let mut old_last: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
        let mut retval: ::core::ffi::c_int = OK;
        let mut valid_entry: bool = false_0 != 0;
        let mut prev_fnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut prev_lnum: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut prev_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if !(*qfl).qf_ptr.is_null() {
            prev_fnum = (*(*qfl).qf_ptr).qf_fnum;
            prev_lnum = (*(*qfl).qf_ptr).qf_lnum as ::core::ffi::c_int;
            prev_col = (*(*qfl).qf_ptr).qf_col;
        }
        let mut select_first_entry: bool = false_0 != 0;
        let mut select_nearest_entry: bool = false_0 != 0;
        if action == ' ' as ::core::ffi::c_int || qf_idx == (*qi).qf_listcount {
            select_first_entry = true_0 != 0;
            qf_new_list(qi, title);
            qf_idx = (*qi).qf_curlist;
            qfl = qf_get_list(qi, qf_idx);
        } else if action == 'a' as ::core::ffi::c_int {
            if qf_list_empty(qfl) {
                select_first_entry = true_0 != 0;
            } else {
                old_last = (*qfl).qf_last;
            }
        } else if action == 'r' as ::core::ffi::c_int {
            select_first_entry = true_0 != 0;
            qf_free_items(qfl);
            qf_store_title(qfl, title);
        } else if action == 'u' as ::core::ffi::c_int {
            select_nearest_entry = true_0 != 0;
            qf_free_items(qfl);
            qf_store_title(qfl, title);
        }
        let mut entry_to_select: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
        let mut entry_to_select_index: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let l_: *const list_T = list;
        if !l_.is_null() {
            let mut li: *const listitem_T = (*l_).lv_first;
            while !li.is_null() {
                if (*li).li_tv.v_type as ::core::ffi::c_uint
                    == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let d: *mut dict_T = (*li).li_tv.vval.v_dict;
                    if !d.is_null() {
                        retval = qf_add_entry_from_dict(
                            qfl,
                            d,
                            li == tv_list_first(list) as *const listitem_T,
                            &raw mut valid_entry,
                        );
                        if retval == QF_FAIL as ::core::ffi::c_int {
                            break;
                        }
                        let mut entry: *mut qfline_T = (*qfl).qf_last;
                        if select_first_entry as ::core::ffi::c_int != 0
                            && entry_to_select.is_null()
                            || select_nearest_entry as ::core::ffi::c_int != 0
                                && (entry_to_select.is_null()
                                    || entry_is_closer_to_target(
                                        entry,
                                        entry_to_select,
                                        prev_fnum,
                                        prev_lnum,
                                        prev_col,
                                    ) as ::core::ffi::c_int
                                        != 0)
                        {
                            entry_to_select = entry;
                            entry_to_select_index = (*qfl).qf_count;
                        }
                    }
                }
                li = (*li).li_next;
            }
        }
        if valid_entry {
            (*qfl).qf_nonevalid = false_0 != 0;
        } else if (*qfl).qf_index == 0 as ::core::ffi::c_int {
            (*qfl).qf_nonevalid = true_0 != 0;
        }
        if !entry_to_select.is_null() {
            (*qfl).qf_ptr = entry_to_select;
            (*qfl).qf_index = entry_to_select_index;
        }
        qf_update_buffer(qi, old_last);
        return retval;
    }
}

pub(crate) unsafe extern "C" fn qf_setprop_get_qfidx(
    mut qi: *const qf_info_T,
    mut what: *const dict_T,
    mut action: ::core::ffi::c_int,
    mut newlist: *mut bool,
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
                }
                if (action == ' ' as ::core::ffi::c_int || action == 'a' as ::core::ffi::c_int)
                    && qf_idx == (*qi).qf_listcount
                {
                    *newlist = true_0 != 0;
                    qf_idx = if qf_stack_empty(qi) as ::core::ffi::c_int != 0 {
                        0 as ::core::ffi::c_int
                    } else {
                        (*qi).qf_listcount - 1 as ::core::ffi::c_int
                    };
                } else if qf_idx < 0 as ::core::ffi::c_int || qf_idx >= (*qi).qf_listcount {
                    return INVALID_QFIDX;
                } else if action != ' ' as ::core::ffi::c_int {
                    *newlist = false_0 != 0;
                }
            } else if (*di).di_tv.v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                && strequal(
                    (*di).di_tv.vval.v_string,
                    b"$\0".as_ptr() as *const ::core::ffi::c_char,
                ) as ::core::ffi::c_int
                    != 0
            {
                if !qf_stack_empty(qi) {
                    qf_idx = (*qi).qf_listcount - 1 as ::core::ffi::c_int;
                } else if *newlist {
                    qf_idx = 0 as ::core::ffi::c_int;
                } else {
                    return INVALID_QFIDX;
                }
            } else {
                return INVALID_QFIDX;
            }
        }
        if !*newlist && {
            di = tv_dict_find(
                what,
                b"id\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as usize)
                    as ptrdiff_t,
            );
            !di.is_null()
        } {
            if (*di).di_tv.v_type as ::core::ffi::c_uint
                != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return INVALID_QFIDX;
            }
            return qf_id2nr(qi, (*di).di_tv.vval.v_number as ::core::ffi::c_uint);
        }
        return qf_idx;
    }
}

pub(crate) unsafe extern "C" fn qf_setprop_title(
    mut qi: *mut qf_info_T,
    mut qf_idx: ::core::ffi::c_int,
    mut what: *const dict_T,
    mut di: *const dictitem_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut qfl: *mut qf_list_T = qf_get_list(qi, qf_idx);
        if (*di).di_tv.v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return FAIL;
        }
        xfree((*qfl).qf_title as *mut ::core::ffi::c_void);
        (*qfl).qf_title = tv_dict_get_string(
            what,
            b"title\0".as_ptr() as *const ::core::ffi::c_char,
            true_0 != 0,
        );
        if qf_idx == (*qi).qf_curlist {
            qf_update_win_titlevar(qi);
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn qf_setprop_items(
    mut qi: *mut qf_info_T,
    mut qf_idx: ::core::ffi::c_int,
    mut di: *mut dictitem_T,
    mut action: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if (*di).di_tv.v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return FAIL;
        }
        let mut title_save: *mut ::core::ffi::c_char = xstrdup((*qf_get_list(qi, qf_idx)).qf_title);
        let retval: ::core::ffi::c_int = qf_add_entries(
            qi,
            qf_idx,
            (*di).di_tv.vval.v_list,
            title_save,
            if action == ' ' as ::core::ffi::c_int {
                'a' as ::core::ffi::c_int
            } else {
                action
            },
        );
        xfree(title_save as *mut ::core::ffi::c_void);
        return retval;
    }
}

pub(crate) unsafe extern "C" fn qf_setprop_items_from_lines(
    mut qi: *mut qf_info_T,
    mut qf_idx: ::core::ffi::c_int,
    mut what: *const dict_T,
    mut di: *mut dictitem_T,
    mut action: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut errorformat: *mut ::core::ffi::c_char = p_efm.get();
        let mut efm_di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
        let mut retval: ::core::ffi::c_int = FAIL;
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
        if (*di).di_tv.v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*di).di_tv.vval.v_list.is_null()
        {
            return FAIL;
        }
        if action == 'r' as ::core::ffi::c_int || action == 'u' as ::core::ffi::c_int {
            qf_free_items(qf_get_list(qi, qf_idx));
        }
        if qf_init_ext(
            qi,
            qf_idx,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<buf_T>(),
            &raw mut (*di).di_tv,
            errorformat,
            false_0 != 0,
            0 as linenr_T,
            0 as linenr_T,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ) >= 0 as ::core::ffi::c_int
        {
            retval = OK;
        }
        return retval;
    }
}

pub(crate) unsafe extern "C" fn qf_setprop_context(
    mut qfl: *mut qf_list_T,
    mut di: *mut dictitem_T,
) -> ::core::ffi::c_int {
    unsafe {
        tv_free((*qfl).qf_ctx);
        let mut ctx: *mut typval_T =
            xcalloc(1 as size_t, ::core::mem::size_of::<typval_T>()) as *mut typval_T;
        tv_copy(&raw mut (*di).di_tv, ctx);
        (*qfl).qf_ctx = ctx;
        return OK;
    }
}

pub(crate) unsafe extern "C" fn qf_setprop_curidx(
    mut qi: *mut qf_info_T,
    mut qfl: *mut qf_list_T,
    mut di: *const dictitem_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut newidx: ::core::ffi::c_int = 0;
        if (*di).di_tv.v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            && !(*di).di_tv.vval.v_string.is_null()
            && strcmp(
                (*di).di_tv.vval.v_string,
                b"$\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
        {
            newidx = (*qfl).qf_count;
        } else {
            let mut denote: bool = false_0 != 0;
            newidx =
                tv_get_number_chk(&raw const (*di).di_tv, &raw mut denote) as ::core::ffi::c_int;
            if denote {
                return FAIL;
            }
        }
        if newidx < 1 as ::core::ffi::c_int {
            return FAIL;
        }
        newidx = if newidx < (*qfl).qf_count {
            newidx
        } else {
            (*qfl).qf_count
        };
        let old_qfidx: ::core::ffi::c_int = (*qfl).qf_index;
        let qf_ptr: *mut qfline_T = get_nth_entry(qfl, newidx, &raw mut newidx);
        if qf_ptr.is_null() {
            return FAIL;
        }
        (*qfl).qf_ptr = qf_ptr;
        (*qfl).qf_index = newidx;
        if (*qf_get_curlist(qi)).qf_id == (*qfl).qf_id {
            qf_win_pos_update(qi, old_qfidx);
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn qf_set_properties(
    mut qi: *mut qf_info_T,
    mut what: *const dict_T,
    mut action: ::core::ffi::c_int,
    mut title: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut newlist: bool =
            action == ' ' as ::core::ffi::c_int || qf_stack_empty(qi) as ::core::ffi::c_int != 0;
        let mut qf_idx: ::core::ffi::c_int =
            qf_setprop_get_qfidx(qi, what, action, &raw mut newlist);
        if qf_idx == INVALID_QFIDX {
            return FAIL;
        }
        if newlist {
            (*qi).qf_curlist = qf_idx;
            qf_new_list(qi, title);
            qf_idx = (*qi).qf_curlist;
        }
        let mut qfl: *mut qf_list_T = qf_get_list(qi, qf_idx);
        let mut di: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
        let mut retval: ::core::ffi::c_int = FAIL;
        di = tv_dict_find(
            what,
            b"title\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !di.is_null() {
            retval = qf_setprop_title(qi, qf_idx, what, di);
        }
        di = tv_dict_find(
            what,
            b"items\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !di.is_null() {
            retval = qf_setprop_items(qi, qf_idx, di, action);
        }
        di = tv_dict_find(
            what,
            b"lines\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !di.is_null() {
            retval = qf_setprop_items_from_lines(qi, qf_idx, what, di, action);
        }
        di = tv_dict_find(
            what,
            b"context\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !di.is_null() {
            retval = qf_setprop_context(qfl, di);
        }
        di = tv_dict_find(
            what,
            b"idx\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !di.is_null() {
            retval = qf_setprop_curidx(qi, qfl, di);
        }
        di = tv_dict_find(
            what,
            b"quickfixtextfunc\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !di.is_null() {
            retval = qf_setprop_qftf(qfl, di);
        }
        if newlist as ::core::ffi::c_int != 0 || retval == OK {
            qf_list_changed(qfl);
        }
        if newlist {
            qf_update_buffer(qi, ::core::ptr::null_mut::<qfline_T>());
        }
        return retval;
    }
}

pub unsafe fn set_errorlist(
    mut wp: *mut win_T,
    mut list: *mut list_T,
    mut action: ::core::ffi::c_int,
    mut title: *mut ::core::ffi::c_char,
    mut what: *mut dict_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut qi: *mut qf_info_T = ::core::ptr::null_mut::<qf_info_T>();
        if !wp.is_null() {
            qi = ll_get_or_alloc_list(wp);
        } else {
            qi = ql_info.get();
        }
        '_c2rust_label: {
            if !qi.is_null() {
            } else {
                __assert_fail(
                    b"qi != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    7120 as ::core::ffi::c_uint,
                    b"int set_errorlist(win_T *, list_T *, int, char *, dict_T *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if action == 'f' as ::core::ffi::c_int {
            qf_free_stack(wp, qi);
            return OK;
        }
        if !list.is_null() && tv_list_len(list) != 0 as ::core::ffi::c_int && !what.is_null() {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                gettext(
                    b"cannot have both a list and a \"what\" argument\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
            );
            return FAIL;
        }
        incr_quickfix_busy();
        let mut retval: ::core::ffi::c_int = OK;
        if !what.is_null() {
            retval = qf_set_properties(qi, what, action, title);
        } else {
            retval = qf_add_entries(qi, (*qi).qf_curlist, list, title, action);
            if retval == OK {
                qf_list_changed(qf_get_curlist(qi));
            }
        }
        decr_quickfix_busy();
        return retval;
    }
}
