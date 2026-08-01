//! `'tagfunc'`, the user-supplied tag lookup.
//!
//! [`find_tagfunc_tags`] calls the option's callback instead of reading any
//! tags file and validates what comes back — a list of dictionaries with at
//! least `name`, `filename` and `cmd`.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn did_set_tagfunc(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    unsafe {
        let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
        let mut retval: ::core::ffi::c_int = 0;
        if (*args).os_flags & OPT_LOCAL as ::core::ffi::c_int != 0 {
            retval =
                option_set_callback_func((*args).os_newval.string.data, &raw mut (*buf).b_tfu_cb);
        } else {
            retval = option_set_callback_func((*args).os_newval.string.data, tfu_cb.ptr());
            if retval == OK && (*args).os_flags & OPT_GLOBAL as ::core::ffi::c_int == 0 {
                set_buflocal_tfu_callback(buf);
            }
        }
        return if retval == FAIL {
            &raw const e_invarg as *const ::core::ffi::c_char
        } else {
            ::core::ptr::null::<::core::ffi::c_char>()
        };
    }
}

pub unsafe extern "C" fn set_ref_in_tagfunc(mut copyID: ::core::ffi::c_int) -> bool {
    unsafe {
        return set_ref_in_callback(
            tfu_cb.ptr(),
            copyID,
            ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        );
    }
}

pub unsafe extern "C" fn set_buflocal_tfu_callback(mut buf: *mut buf_T) {
    unsafe {
        callback_free(&raw mut (*buf).b_tfu_cb);
        if (*tfu_cb.ptr()).type_0 as ::core::ffi::c_uint
            != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            callback_copy(&raw mut (*buf).b_tfu_cb, tfu_cb.ptr());
        }
    }
}

pub(crate) unsafe extern "C" fn find_tagfunc_tags(
    mut pat: *mut ::core::ffi::c_char,
    mut ga: *mut garray_T,
    mut match_count: *mut ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut buf_ffname: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut ntags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut args: [typval_T; 4] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 4];
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut flagString: [::core::ffi::c_char; 4] = [0; 4];
        let mut tag: *mut taggy_T = ::core::ptr::null_mut::<taggy_T>();
        if (*curwin.get()).w_tagstacklen > 0 as ::core::ffi::c_int {
            if (*curwin.get()).w_tagstackidx == (*curwin.get()).w_tagstacklen {
                tag = (&raw mut (*curwin.get()).w_tagstack as *mut taggy_T)
                    .offset(((*curwin.get()).w_tagstackidx - 1 as ::core::ffi::c_int) as isize);
            } else {
                tag = (&raw mut (*curwin.get()).w_tagstack as *mut taggy_T)
                    .offset((*curwin.get()).w_tagstackidx as isize);
            }
        }
        if *(*curbuf.get()).b_p_tfu as ::core::ffi::c_int == NUL
            || (*curbuf.get()).b_tfu_cb.type_0 as ::core::ffi::c_uint
                == kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return FAIL;
        }
        args[0 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
        args[0 as ::core::ffi::c_int as usize].vval.v_string = pat;
        args[1 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
        args[1 as ::core::ffi::c_int as usize].vval.v_string =
            &raw mut flagString as *mut ::core::ffi::c_char;
        let d: *mut dict_T = tv_dict_alloc_lock(VAR_FIXED);
        if flags & TAG_INS_COMP as ::core::ffi::c_int == 0
            && !tag.is_null()
            && !(*tag).user_data.is_null()
        {
            tv_dict_add_str(
                d,
                b"user_data\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
                (*tag).user_data,
            );
        }
        if !buf_ffname.is_null() {
            tv_dict_add_str(
                d,
                b"buf_ffname\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 11]>().wrapping_sub(1 as size_t),
                buf_ffname,
            );
        }
        (*d).dv_refcount += 1;
        args[2 as ::core::ffi::c_int as usize].v_type = VAR_DICT;
        args[2 as ::core::ffi::c_int as usize].vval.v_dict = d;
        args[3 as ::core::ffi::c_int as usize].v_type = VAR_UNKNOWN;
        vim_snprintf(
            &raw mut flagString as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4]>(),
            b"%s%s%s\0".as_ptr() as *const ::core::ffi::c_char,
            if g_tag_at_cursor.get() as ::core::ffi::c_int != 0 {
                b"c\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
            if flags & TAG_INS_COMP as ::core::ffi::c_int != 0 {
                b"i\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
            if flags & TAG_REGEXP as ::core::ffi::c_int != 0 {
                b"r\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
        );
        let mut save_pos: pos_T = (*curwin.get()).w_cursor;
        let mut result: ::core::ffi::c_int = callback_call(
            &raw mut (*curbuf.get()).b_tfu_cb,
            3 as ::core::ffi::c_int,
            &raw mut args as *mut typval_T,
            &raw mut rettv,
        ) as ::core::ffi::c_int;
        (*curwin.get()).w_cursor = save_pos;
        check_cursor(curwin.get());
        (*d).dv_refcount -= 1;
        if result == FAIL {
            return FAIL;
        }
        if rettv.v_type as ::core::ffi::c_uint
            == VAR_SPECIAL as ::core::ffi::c_int as ::core::ffi::c_uint
            && rettv.vval.v_special as ::core::ffi::c_uint
                == kSpecialVarNull as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_clear(&raw mut rettv);
            return NOTDONE;
        }
        if rettv.v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            || rettv.vval.v_list.is_null()
        {
            tv_clear(&raw mut rettv);
            emsg(gettext(
                (e_invalid_return_value_from_tagfunc.ptr() as *const _)
                    as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
        let mut taglist: *mut list_T = rettv.vval.v_list;
        let l_: *const list_T = taglist;
        if !l_.is_null() {
            let mut li: *const listitem_T = (*l_).lv_first;
            while !li.is_null() {
                let mut res_name: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                let mut res_fname: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                let mut res_cmd: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                let mut res_kind: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                let mut has_extra: bool = false;
                let mut name_only: ::core::ffi::c_int = flags & TAG_NAMES as ::core::ffi::c_int;
                if (*li).li_tv.v_type as ::core::ffi::c_uint
                    != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    emsg(gettext(
                        (e_invalid_return_value_from_tagfunc.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    ));
                    break;
                } else {
                    let mut len: size_t = 2 as size_t;
                    res_name = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    res_fname = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    res_cmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    res_kind = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    let dihi_ht_: *mut hashtab_T = &raw mut (*(*li).li_tv.vval.v_dict).dv_hashtab;
                    let mut dihi_todo_: size_t = (*dihi_ht_).ht_used;
                    let mut dihi_: *mut hashitem_T = (*dihi_ht_).ht_array;
                    while dihi_todo_ != 0 {
                        if !((*dihi_).hi_key.is_null()
                            || (*dihi_).hi_key
                                == &raw const hash_removed as *mut ::core::ffi::c_char)
                        {
                            dihi_todo_ = dihi_todo_.wrapping_sub(1);
                            let di: *mut dictitem_T = (*dihi_)
                                .hi_key
                                .offset(-(17 as ::core::ffi::c_ulong as isize))
                                as *mut dictitem_T;
                            let mut dict_key: *const ::core::ffi::c_char =
                                &raw mut (*di).di_key as *mut ::core::ffi::c_char;
                            let mut tv: *mut typval_T = &raw mut (*di).di_tv;
                            if !((*tv).v_type as ::core::ffi::c_uint
                                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                                || (*tv).vval.v_string.is_null())
                            {
                                len = len.wrapping_add(
                                    strlen((*tv).vval.v_string).wrapping_add(1 as size_t),
                                );
                                if strcmp(
                                    dict_key,
                                    b"name\0".as_ptr() as *const ::core::ffi::c_char,
                                ) == 0
                                {
                                    res_name = (*tv).vval.v_string;
                                } else if strcmp(
                                    dict_key,
                                    b"filename\0".as_ptr() as *const ::core::ffi::c_char,
                                ) == 0
                                {
                                    res_fname = (*tv).vval.v_string;
                                } else if strcmp(
                                    dict_key,
                                    b"cmd\0".as_ptr() as *const ::core::ffi::c_char,
                                ) == 0
                                {
                                    res_cmd = (*tv).vval.v_string;
                                } else {
                                    has_extra = true;
                                    if strcmp(
                                        dict_key,
                                        b"kind\0".as_ptr() as *const ::core::ffi::c_char,
                                    ) == 0
                                    {
                                        res_kind = (*tv).vval.v_string;
                                    } else {
                                        len = len.wrapping_add(
                                            strlen(dict_key).wrapping_add(1 as size_t),
                                        );
                                    }
                                }
                            }
                        }
                        dihi_ = dihi_.offset(1);
                    }
                    if has_extra {
                        len = len.wrapping_add(2 as size_t);
                    }
                    if res_name.is_null() || res_fname.is_null() || res_cmd.is_null() {
                        emsg(gettext(
                            (e_invalid_return_value_from_tagfunc.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        ));
                        break;
                    } else {
                        let mfp: *mut ::core::ffi::c_char = (if name_only != 0 {
                            xstrdup(res_name) as *mut ::core::ffi::c_void
                        } else {
                            xmalloc(len.wrapping_add(2 as size_t))
                        })
                            as *mut ::core::ffi::c_char;
                        if name_only == 0 {
                            let mut p: *mut ::core::ffi::c_char = mfp;
                            let c2rust_fresh7 = p;
                            p = p.offset(1);
                            *c2rust_fresh7 = (MT_GL_OTH as ::core::ffi::c_int
                                + 1 as ::core::ffi::c_int)
                                as ::core::ffi::c_char;
                            let c2rust_fresh8 = p;
                            p = p.offset(1);
                            *c2rust_fresh8 = 0x2 as ::core::ffi::c_char;
                            strcpy(p, res_name);
                            p = p.offset(strlen(p) as isize);
                            let c2rust_fresh9 = p;
                            p = p.offset(1);
                            *c2rust_fresh9 = '\t' as ::core::ffi::c_char;
                            strcpy(p, res_fname);
                            p = p.offset(strlen(p) as isize);
                            let c2rust_fresh10 = p;
                            p = p.offset(1);
                            *c2rust_fresh10 = '\t' as ::core::ffi::c_char;
                            strcpy(p, res_cmd);
                            p = p.offset(strlen(p) as isize);
                            if has_extra {
                                strcpy(
                                    p,
                                    b";\"\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                );
                                p = p.offset(strlen(p) as isize);
                                if !res_kind.is_null() {
                                    let c2rust_fresh11 = p;
                                    p = p.offset(1);
                                    *c2rust_fresh11 = '\t' as ::core::ffi::c_char;
                                    strcpy(p, res_kind);
                                    p = p.offset(strlen(p) as isize);
                                }
                                let dihi_ht__0: *mut hashtab_T =
                                    &raw mut (*(*li).li_tv.vval.v_dict).dv_hashtab;
                                let mut dihi_todo__0: size_t = (*dihi_ht__0).ht_used;
                                let mut dihi__0: *mut hashitem_T = (*dihi_ht__0).ht_array;
                                while dihi_todo__0 != 0 {
                                    if !((*dihi__0).hi_key.is_null()
                                        || (*dihi__0).hi_key
                                            == &raw const hash_removed as *mut ::core::ffi::c_char)
                                    {
                                        dihi_todo__0 = dihi_todo__0.wrapping_sub(1);
                                        let di_0: *mut dictitem_T = (*dihi__0)
                                            .hi_key
                                            .offset(-(17 as ::core::ffi::c_ulong as isize))
                                            as *mut dictitem_T;
                                        let mut dict_key_0: *const ::core::ffi::c_char =
                                            &raw mut (*di_0).di_key as *mut ::core::ffi::c_char;
                                        let mut tv_0: *mut typval_T = &raw mut (*di_0).di_tv;
                                        if !((*tv_0).v_type as ::core::ffi::c_uint
                                            != VAR_STRING as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            || (*tv_0).vval.v_string.is_null())
                                        {
                                            if strcmp(
                                                dict_key_0,
                                                b"name\0".as_ptr() as *const ::core::ffi::c_char,
                                            ) != 0
                                            {
                                                if strcmp(
                                                    dict_key_0,
                                                    b"filename\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ) != 0
                                                {
                                                    if strcmp(
                                                        dict_key_0,
                                                        b"cmd\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    ) != 0
                                                    {
                                                        if strcmp(
                                                            dict_key_0,
                                                            b"kind\0".as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                        ) != 0
                                                        {
                                                            let c2rust_fresh12 = p;
                                                            p = p.offset(1);
                                                            *c2rust_fresh12 =
                                                                '\t' as ::core::ffi::c_char;
                                                            strcpy(
                                                                p,
                                                                dict_key_0
                                                                    as *mut ::core::ffi::c_char,
                                                            );
                                                            p = p.offset(strlen(p) as isize);
                                                            strcpy(
                                                                p,
                                                                b":\0".as_ptr()
                                                                    as *const ::core::ffi::c_char
                                                                    as *mut ::core::ffi::c_char,
                                                            );
                                                            p = p.offset(strlen(p) as isize);
                                                            strcpy(p, (*tv_0).vval.v_string);
                                                            p = p.offset(strlen(p) as isize);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    dihi__0 = dihi__0.offset(1);
                                }
                            }
                        }
                        ga_grow(ga, 1 as ::core::ffi::c_int);
                        let c2rust_fresh13 = (*ga).ga_len;
                        (*ga).ga_len = (*ga).ga_len + 1;
                        let c2rust_lvalue_ptr = &raw mut *((*ga).ga_data
                            as *mut *mut ::core::ffi::c_char)
                            .offset(c2rust_fresh13 as isize);
                        *c2rust_lvalue_ptr = mfp;
                        ntags += 1;
                        result = 1 as ::core::ffi::c_int;
                        li = (*li).li_next;
                    }
                }
            }
        }
        tv_clear(&raw mut rettv);
        *match_count = ntags;
        return result;
    }
}
