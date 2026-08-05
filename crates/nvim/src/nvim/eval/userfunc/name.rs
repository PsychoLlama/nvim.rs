//! Turning what the user wrote into the name a `ufunc_T` is stored under.
//!
//! `trans_function_name` is the whole of it: it resolves `s:`/`<SID>` to
//! the `<SNR>N_` mangling, evaluates a curly-brace name, follows a
//! dictionary subscript to a numbered function, and rejects the spellings
//! that are not names at all.  `fname_trans_sid` and `cat_func_name` are
//! the two smaller manglings around it, and `builtin_function` is what
//! decides a name belongs to the builtin table instead.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn deref_func_name(
    mut name: *const ::core::ffi::c_char,
    mut lenp: *mut ::core::ffi::c_int,
    partialp: *mut *mut partial_T,
    mut no_autoload: bool,
    mut found_var: *mut bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if !partialp.is_null() {
            *partialp = ::core::ptr::null_mut::<partial_T>();
        }
        let v: *mut dictitem_T = find_var(
            name,
            *lenp as size_t,
            ::core::ptr::null_mut::<*mut hashtab_T>(),
            no_autoload,
        );
        if v.is_null() {
            return name as *mut ::core::ffi::c_char;
        }
        let tv: *mut typval_T = &raw mut (*v).di_tv;
        if !found_var.is_null() {
            *found_var = true_0 != 0;
        }
        if (*tv).v_type as ::core::ffi::c_uint
            == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*tv).vval.v_string.is_null() {
                *lenp = 0 as ::core::ffi::c_int;
                return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            *lenp = strlen((*tv).vval.v_string) as ::core::ffi::c_int;
            return (*tv).vval.v_string;
        }
        if (*tv).v_type as ::core::ffi::c_uint
            == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let pt: *mut partial_T = (*tv).vval.v_partial;
            if pt.is_null() {
                *lenp = 0 as ::core::ffi::c_int;
                return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            if !partialp.is_null() {
                *partialp = pt;
            }
            let mut s: *mut ::core::ffi::c_char = partial_name(pt);
            *lenp = strlen(s) as ::core::ffi::c_int;
            return s;
        }
        return name as *mut ::core::ffi::c_char;
    }
}

pub unsafe extern "C" fn emsg_funcname(
    mut errmsg: *const ::core::ffi::c_char,
    mut name: *const ::core::ffi::c_char,
) {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = name as *mut ::core::ffi::c_char;
        if *name.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            == K_SPECIAL
            && *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            && *name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            p = concat_str(
                b"<SNR>\0".as_ptr() as *const ::core::ffi::c_char,
                name.offset(3 as ::core::ffi::c_int as isize),
            );
        }
        semsg(gettext(errmsg), p);
        if p != name as *mut ::core::ffi::c_char {
            xfree(p as *mut ::core::ffi::c_void);
        }
    }
}

pub const FLEN_FIXED: ::core::ffi::c_int = 40 as ::core::ffi::c_int;

#[inline(always)]
unsafe extern "C" fn eval_fname_sid(name: *const ::core::ffi::c_char) -> bool {
    unsafe {
        return *name as ::core::ffi::c_int == 's' as ::core::ffi::c_int
            || (if (*name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                < 'a' as ::core::ffi::c_int
                || *name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    > 'z' as ::core::ffi::c_int
            {
                *name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            } else {
                *name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
            }) == 'I' as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn fname_trans_sid(
    name: *const ::core::ffi::c_char,
    fname_buf: *mut ::core::ffi::c_char,
    tofree: *mut *mut ::core::ffi::c_char,
    error: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut script_name: *const ::core::ffi::c_char =
            name.offset(eval_fname_script(name) as isize);
        if script_name == name {
            return name as *mut ::core::ffi::c_char;
        }
        *fname_buf.offset(0 as ::core::ffi::c_int as isize) = K_SPECIAL as ::core::ffi::c_char;
        *fname_buf.offset(1 as ::core::ffi::c_int as isize) = KS_EXTRA as ::core::ffi::c_char;
        *fname_buf.offset(2 as ::core::ffi::c_int as isize) =
            KE_SNR as ::core::ffi::c_int as ::core::ffi::c_char;
        let mut fname_buflen: size_t = 3 as size_t;
        if !eval_fname_sid(name) {
            *fname_buf.offset(fname_buflen as isize) = NUL as ::core::ffi::c_char;
        } else if (*current_sctx.ptr()).sc_sid <= 0 as ::core::ffi::c_int {
            *error = FCERR_SCRIPT as ::core::ffi::c_int;
        } else {
            fname_buflen = fname_buflen.wrapping_add(snprintf(
                fname_buf.offset(fname_buflen as isize),
                ((FLEN_FIXED + 1 as ::core::ffi::c_int) as size_t).wrapping_sub(fname_buflen),
                b"%d_\0".as_ptr() as *const ::core::ffi::c_char,
                (*current_sctx.ptr()).sc_sid,
            ) as size_t);
        }
        let mut fnamelen: size_t = fname_buflen.wrapping_add(strlen(script_name));
        let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if fnamelen < FLEN_FIXED as size_t {
            strcpy(
                fname_buf.offset(fname_buflen as isize),
                script_name as *mut ::core::ffi::c_char,
            );
            fname = fname_buf;
        } else {
            fname = xmalloc(fnamelen.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
            *tofree = fname;
            snprintf(
                fname,
                fnamelen.wrapping_add(1 as size_t),
                b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                fname_buf,
                script_name,
            );
        }
        return fname;
    }
}

pub unsafe extern "C" fn find_func(mut name: *const ::core::ffi::c_char) -> *mut ufunc_T {
    unsafe {
        let mut hi: *mut hashitem_T = hash_find(func_hashtab.ptr(), name);
        if !((*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            return (*hi).hi_key.offset(-(240 as ::core::ffi::c_ulong as isize)) as *mut ufunc_T;
        }
        return ::core::ptr::null_mut::<ufunc_T>();
    }
}

unsafe extern "C" fn func_is_global(mut ufunc: *const ufunc_T) -> bool {
    unsafe {
        return *(&raw const (*ufunc).uf_name as *const ::core::ffi::c_char)
            .offset(0 as ::core::ffi::c_int as isize) as uint8_t
            as ::core::ffi::c_int
            != K_SPECIAL;
    }
}

pub(crate) unsafe extern "C" fn cat_func_name(
    mut buf: *mut ::core::ffi::c_char,
    mut bufsize: size_t,
    mut fp: *const ufunc_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut len: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut uflen: size_t = (*fp).uf_namelen;
        '_c2rust_label: {
            if uflen > 0 as size_t {
            } else {
                __assert_fail(
                    b"uflen > 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/userfunc.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    736 as ::core::ffi::c_uint,
                    b"int cat_func_name(char *, size_t, const ufunc_T *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if !func_is_global(fp) && uflen > 3 as size_t {
            len = snprintf(
                buf,
                bufsize,
                b"<SNR>%s\0".as_ptr() as *const ::core::ffi::c_char,
                (&raw const (*fp).uf_name as *const ::core::ffi::c_char)
                    .offset(3 as ::core::ffi::c_int as isize),
            );
        } else {
            len = snprintf(
                buf,
                bufsize,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const (*fp).uf_name as *const ::core::ffi::c_char,
            );
        }
        '_c2rust_label_0: {
            if len > 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"len > 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/userfunc.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    744 as ::core::ffi::c_uint,
                    b"int cat_func_name(char *, size_t, const ufunc_T *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        return if len >= bufsize as ::core::ffi::c_int {
            bufsize as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        } else {
            len
        };
    }
}

pub(crate) unsafe extern "C" fn func_name_refcount(mut name: *const ::core::ffi::c_char) -> bool {
    unsafe {
        return *(*__ctype_b_loc()).offset(*name as uint8_t as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
            || *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '<' as ::core::ffi::c_int
                && *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'l' as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn builtin_function(
    mut name: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> bool {
    unsafe {
        if !(*name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
            >= 'a' as ::core::ffi::c_uint
            && *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
                <= 'z' as ::core::ffi::c_uint)
            || *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == ':' as ::core::ffi::c_int
        {
            return false_0 != 0;
        }
        let mut p: *const ::core::ffi::c_char = (if len == -1 as ::core::ffi::c_int {
            strchr(name, AUTOLOAD_CHAR) as *mut ::core::ffi::c_void
        } else {
            memchr(
                name as *const ::core::ffi::c_void,
                AUTOLOAD_CHAR,
                len as size_t,
            )
        }) as *const ::core::ffi::c_char;
        return p.is_null();
    }
}

pub unsafe extern "C" fn printable_func_name(mut fp: *mut ufunc_T) -> *mut ::core::ffi::c_char {
    unsafe {
        return if !(*fp).uf_name_exp.is_null() {
            (*fp).uf_name_exp
        } else {
            &raw mut (*fp).uf_name as *mut ::core::ffi::c_char
        };
    }
}

pub unsafe extern "C" fn trans_function_name(
    mut pp: *mut *mut ::core::ffi::c_char,
    mut skip: bool,
    mut flags: ::core::ffi::c_int,
    mut fdp: *mut funcdict_T,
    mut partial: *mut *mut partial_T,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut sid_buflen: size_t = 0;
        let mut sid_buf: [::core::ffi::c_char; 20] = [0; 20];
        let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut len: ::core::ffi::c_int = 0;
        let mut lv: lval_T = lval_T {
            ll_name: ::core::ptr::null::<::core::ffi::c_char>(),
            ll_name_len: 0,
            ll_exp_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ll_tv: ::core::ptr::null_mut::<typval_T>(),
            ll_li: ::core::ptr::null_mut::<listitem_T>(),
            ll_list: ::core::ptr::null_mut::<list_T>(),
            ll_range: false,
            ll_empty2: false,
            ll_n1: 0,
            ll_n2: 0,
            ll_dict: ::core::ptr::null_mut::<dict_T>(),
            ll_di: ::core::ptr::null_mut::<dictitem_T>(),
            ll_newkey: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ll_blob: ::core::ptr::null_mut::<blob_T>(),
        };
        if !fdp.is_null() {
            memset(
                fdp as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<funcdict_T>(),
            );
        }
        let mut start: *const ::core::ffi::c_char = *pp;
        if *(*pp).offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            == K_SPECIAL
            && *(*pp).offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
                == KS_EXTRA
            && *(*pp).offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == KE_SNR as ::core::ffi::c_int
        {
            *pp = (*pp).offset(3 as ::core::ffi::c_int as isize);
            len = get_id_len(pp as *mut *const ::core::ffi::c_char) + 3 as ::core::ffi::c_int;
            return xmemdupz(start as *const ::core::ffi::c_void, len as size_t)
                as *mut ::core::ffi::c_char;
        }
        let mut lead: ::core::ffi::c_int = eval_fname_script(start);
        if lead > 2 as ::core::ffi::c_int {
            start = start.offset(lead as isize);
        }
        let mut end: *const ::core::ffi::c_char = get_lval(
            start as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<typval_T>(),
            &raw mut lv,
            false_0 != 0,
            skip,
            flags | GLV_READ_ONLY as ::core::ffi::c_int,
            if lead > 2 as ::core::ffi::c_int {
                0 as ::core::ffi::c_int
            } else {
                FNE_CHECK_START
            },
        );
        '_theend: {
            if end == start {
                if !skip {
                    emsg(gettext(
                        b"E129: Function name required\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                }
            } else if end.is_null()
                || !lv.ll_tv.is_null()
                    && (lead > 2 as ::core::ffi::c_int || lv.ll_range as ::core::ffi::c_int != 0)
            {
                if !aborting() {
                    if !end.is_null() {
                        semsg(
                            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                            start,
                        );
                    }
                } else {
                    *pp = find_name_end(
                        start,
                        ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
                        ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
                        FNE_INCL_BR,
                    ) as *mut ::core::ffi::c_char;
                }
            } else if !lv.ll_tv.is_null() {
                if !fdp.is_null() {
                    (*fdp).fd_dict = lv.ll_dict;
                    (*fdp).fd_newkey = lv.ll_newkey;
                    lv.ll_newkey = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    (*fdp).fd_di = lv.ll_di;
                }
                if (*lv.ll_tv).v_type as ::core::ffi::c_uint
                    == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
                    && !(*lv.ll_tv).vval.v_string.is_null()
                {
                    name = xstrdup((*lv.ll_tv).vval.v_string);
                    *pp = end as *mut ::core::ffi::c_char;
                } else if (*lv.ll_tv).v_type as ::core::ffi::c_uint
                    == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
                    && !(*lv.ll_tv).vval.v_partial.is_null()
                {
                    if is_luafunc((*lv.ll_tv).vval.v_partial) as ::core::ffi::c_int != 0
                        && *end as ::core::ffi::c_int == '.' as ::core::ffi::c_int
                    {
                        len = check_luafunc_name(
                            end.offset(1 as ::core::ffi::c_int as isize),
                            true_0 != 0,
                        );
                        if len == 0 as ::core::ffi::c_int {
                            semsg(
                                &raw const e_invexpr2 as *const ::core::ffi::c_char,
                                b"v:lua\0".as_ptr() as *const ::core::ffi::c_char,
                            );
                            break '_theend;
                        } else {
                            name = xmallocz(len as size_t) as *mut ::core::ffi::c_char;
                            memcpy(
                                name as *mut ::core::ffi::c_void,
                                end.offset(1 as ::core::ffi::c_int as isize)
                                    as *const ::core::ffi::c_void,
                                len as size_t,
                            );
                            *pp = (end as *mut ::core::ffi::c_char)
                                .offset(1 as ::core::ffi::c_int as isize)
                                .offset(len as isize);
                        }
                    } else {
                        name = xstrdup(partial_name((*lv.ll_tv).vval.v_partial));
                        *pp = end as *mut ::core::ffi::c_char;
                    }
                    if !partial.is_null() {
                        *partial = (*lv.ll_tv).vval.v_partial;
                    }
                } else {
                    if !skip
                        && flags & TFN_QUIET as ::core::ffi::c_int == 0
                        && (fdp.is_null() || lv.ll_dict.is_null() || (*fdp).fd_newkey.is_null())
                    {
                        emsg(gettext(e_funcref.get()));
                    } else {
                        *pp = end as *mut ::core::ffi::c_char;
                    }
                    name = ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
            } else if lv.ll_name.is_null() {
                *pp = end as *mut ::core::ffi::c_char;
            } else {
                if !lv.ll_exp_name.is_null() {
                    len = strlen(lv.ll_exp_name) as ::core::ffi::c_int;
                    name = deref_func_name(
                        lv.ll_exp_name,
                        &raw mut len,
                        partial,
                        flags & TFN_NO_AUTOLOAD as ::core::ffi::c_int != 0,
                        ::core::ptr::null_mut::<bool>(),
                    );
                    if name == lv.ll_exp_name {
                        name = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    }
                } else if flags & TFN_NO_DEREF as ::core::ffi::c_int == 0 {
                    len = end.offset_from(*pp) as ::core::ffi::c_int;
                    name = deref_func_name(
                        *pp,
                        &raw mut len,
                        partial,
                        flags & TFN_NO_AUTOLOAD as ::core::ffi::c_int != 0,
                        ::core::ptr::null_mut::<bool>(),
                    );
                    if name == *pp {
                        name = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    }
                }
                if !name.is_null() {
                    name = xstrdup(name);
                    *pp = end as *mut ::core::ffi::c_char;
                    if strncmp(
                        name,
                        b"<SNR>\0".as_ptr() as *const ::core::ffi::c_char,
                        5 as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        *name.offset(0 as ::core::ffi::c_int as isize) =
                            K_SPECIAL as ::core::ffi::c_char;
                        *name.offset(1 as ::core::ffi::c_int as isize) =
                            KS_EXTRA as ::core::ffi::c_char;
                        *name.offset(2 as ::core::ffi::c_int as isize) =
                            KE_SNR as ::core::ffi::c_int as ::core::ffi::c_char;
                        memmove(
                            name.offset(3 as ::core::ffi::c_int as isize)
                                as *mut ::core::ffi::c_void,
                            name.offset(5 as ::core::ffi::c_int as isize)
                                as *const ::core::ffi::c_void,
                            strlen(name.offset(5 as ::core::ffi::c_int as isize))
                                .wrapping_add(1 as size_t),
                        );
                    }
                } else {
                    if !lv.ll_exp_name.is_null() {
                        len = strlen(lv.ll_exp_name) as ::core::ffi::c_int;
                        if lead <= 2 as ::core::ffi::c_int
                            && lv.ll_name == lv.ll_exp_name as *const ::core::ffi::c_char
                            && lv.ll_name_len >= 2 as size_t
                            && memcmp(
                                lv.ll_name as *const ::core::ffi::c_void,
                                b"s:\0".as_ptr() as *const ::core::ffi::c_char
                                    as *const ::core::ffi::c_void,
                                2 as size_t,
                            ) == 0 as ::core::ffi::c_int
                        {
                            lv.ll_name = lv.ll_name.offset(2 as ::core::ffi::c_int as isize);
                            lv.ll_name_len = lv.ll_name_len.wrapping_sub(2 as size_t);
                            len -= 2 as ::core::ffi::c_int;
                            lead = 2 as ::core::ffi::c_int;
                        }
                    } else {
                        if lead == 2 as ::core::ffi::c_int
                            || *lv.ll_name.offset(0 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == 'g' as ::core::ffi::c_int
                                && *lv.ll_name.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == ':' as ::core::ffi::c_int
                        {
                            lv.ll_name = lv.ll_name.offset(2 as ::core::ffi::c_int as isize);
                            lv.ll_name_len = lv.ll_name_len.wrapping_sub(2 as size_t);
                        }
                        len = end.offset_from(lv.ll_name) as ::core::ffi::c_int;
                    }
                    sid_buflen = 0 as size_t;
                    sid_buf = [0; 20];
                    if skip {
                        lead = 0 as ::core::ffi::c_int;
                    } else if lead > 0 as ::core::ffi::c_int {
                        lead = 3 as ::core::ffi::c_int;
                        if !lv.ll_exp_name.is_null()
                            && eval_fname_sid(lv.ll_exp_name) as ::core::ffi::c_int != 0
                            || eval_fname_sid(*pp) as ::core::ffi::c_int != 0
                        {
                            if (*current_sctx.ptr()).sc_sid <= 0 as ::core::ffi::c_int {
                                emsg(gettext(&raw const e_usingsid as *const ::core::ffi::c_char));
                                break '_theend;
                            } else {
                                sid_buflen = snprintf(
                                    &raw mut sid_buf as *mut ::core::ffi::c_char,
                                    ::core::mem::size_of::<[::core::ffi::c_char; 20]>(),
                                    b"%d_\0".as_ptr() as *const ::core::ffi::c_char,
                                    (*current_sctx.ptr()).sc_sid,
                                ) as size_t;
                                lead += sid_buflen as ::core::ffi::c_int;
                            }
                        }
                    } else if flags & TFN_INT as ::core::ffi::c_int == 0
                        && builtin_function(lv.ll_name, lv.ll_name_len as ::core::ffi::c_int)
                            as ::core::ffi::c_int
                            != 0
                    {
                        semsg(
                            gettext(
                                b"E128: Function name must start with a capital or \"s:\": %s\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                            ),
                            start,
                        );
                        break '_theend;
                    }
                    if !skip
                        && flags & TFN_QUIET as ::core::ffi::c_int == 0
                        && flags & TFN_NO_DEREF as ::core::ffi::c_int == 0
                    {
                        let mut cp: *mut ::core::ffi::c_char = xmemrchr(
                            lv.ll_name as *const ::core::ffi::c_void,
                            ':' as uint8_t,
                            lv.ll_name_len,
                        )
                            as *mut ::core::ffi::c_char;
                        // Upstream also asks `cp < end`.  `cp` points into
                        // `lv.ll_name`, which for a curly-brace name is a
                        // fresh allocation while `end` points into the
                        // command line: that compares two unrelated objects
                        // and answers whatever the allocator happened to do.
                        // `xmemrchr` is already bounded by `ll_name_len`, so
                        // every colon it finds is inside the name and the
                        // extra test adds nothing but the coin flip
                        // (O-B14-12).
                        if !cp.is_null() {
                            semsg(
                                gettext(
                                    b"E884: Function name cannot contain a colon: %s\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                ),
                                start,
                            );
                            break '_theend;
                        }
                    }
                    name = xmalloc(
                        (len as size_t)
                            .wrapping_add(lead as size_t)
                            .wrapping_add(1 as size_t),
                    ) as *mut ::core::ffi::c_char;
                    if !skip && lead > 0 as ::core::ffi::c_int {
                        *name.offset(0 as ::core::ffi::c_int as isize) =
                            K_SPECIAL as ::core::ffi::c_char;
                        *name.offset(1 as ::core::ffi::c_int as isize) =
                            KS_EXTRA as ::core::ffi::c_char;
                        *name.offset(2 as ::core::ffi::c_int as isize) =
                            KE_SNR as ::core::ffi::c_int as ::core::ffi::c_char;
                        if sid_buflen > 0 as size_t {
                            memcpy(
                                name.offset(3 as ::core::ffi::c_int as isize)
                                    as *mut ::core::ffi::c_void,
                                &raw mut sid_buf as *mut ::core::ffi::c_char
                                    as *const ::core::ffi::c_void,
                                sid_buflen,
                            );
                        }
                    }
                    memmove(
                        name.offset(lead as isize) as *mut ::core::ffi::c_void,
                        lv.ll_name as *const ::core::ffi::c_void,
                        len as size_t,
                    );
                    *name.offset((lead + len) as isize) = NUL as ::core::ffi::c_char;
                    *pp = end as *mut ::core::ffi::c_char;
                }
            }
        }
        clear_lval(&raw mut lv);
        return name;
    }
}

pub unsafe extern "C" fn get_scriptlocal_funcname(
    mut funcname: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if funcname.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if strncmp(
            funcname,
            b"s:\0".as_ptr() as *const ::core::ffi::c_char,
            2 as size_t,
        ) != 0 as ::core::ffi::c_int
            && strncmp(
                funcname,
                b"<SID>\0".as_ptr() as *const ::core::ffi::c_char,
                5 as size_t,
            ) != 0 as ::core::ffi::c_int
        {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if !((*current_sctx.ptr()).sc_sid > 0 as ::core::ffi::c_int
            && (*current_sctx.ptr()).sc_sid <= (*script_items.ptr()).ga_len)
        {
            emsg(gettext(&raw const e_usingsid as *const ::core::ffi::c_char));
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut sid_buf: [::core::ffi::c_char; 25] = [0; 25];
        let mut sid_buflen: size_t = snprintf(
            &raw mut sid_buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 25]>(),
            b"<SNR>%d_\0".as_ptr() as *const ::core::ffi::c_char,
            (*current_sctx.ptr()).sc_sid,
        ) as size_t;
        let off: ::core::ffi::c_int =
            if *funcname as ::core::ffi::c_int == 's' as ::core::ffi::c_int {
                2 as ::core::ffi::c_int
            } else {
                5 as ::core::ffi::c_int
            };
        let mut newnamesize: size_t = sid_buflen
            .wrapping_add(strlen(funcname.offset(off as isize)))
            .wrapping_add(1 as size_t);
        let mut newname: *mut ::core::ffi::c_char =
            xmalloc(newnamesize) as *mut ::core::ffi::c_char;
        snprintf(
            newname,
            newnamesize,
            b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
            &raw mut sid_buf as *mut ::core::ffi::c_char,
            funcname.offset(off as isize),
        );
        return newname;
    }
}

pub unsafe extern "C" fn save_function_name(
    mut name: *mut *mut ::core::ffi::c_char,
    mut skip: bool,
    mut flags: ::core::ffi::c_int,
    mut fudi: *mut funcdict_T,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = *name;
        let mut saved: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if strncmp(
            p,
            b"<lambda>\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            p = p.offset(8 as ::core::ffi::c_int as isize);
            getdigits(&raw mut p, false_0 != 0, 0 as intmax_t);
            saved = xmemdupz(
                *name as *const ::core::ffi::c_void,
                p.offset_from(*name) as size_t,
            ) as *mut ::core::ffi::c_char;
            if !fudi.is_null() {
                memset(
                    fudi as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    ::core::mem::size_of::<funcdict_T>(),
                );
            }
        } else {
            saved = trans_function_name(
                &raw mut p,
                skip,
                flags,
                fudi,
                ::core::ptr::null_mut::<*mut partial_T>(),
            );
        }
        *name = p;
        return saved;
    }
}

pub unsafe extern "C" fn eval_fname_script(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '<' as ::core::ffi::c_int
            && (mb_strnicmp(
                p.offset(1 as ::core::ffi::c_int as isize),
                b"SID>\0".as_ptr() as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
                || mb_strnicmp(
                    p.offset(1 as ::core::ffi::c_int as isize),
                    b"SNR>\0".as_ptr() as *const ::core::ffi::c_char,
                    4 as size_t,
                ) == 0 as ::core::ffi::c_int)
        {
            return 5 as ::core::ffi::c_int;
        }
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 's' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == ':' as ::core::ffi::c_int
        {
            return 2 as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
}
