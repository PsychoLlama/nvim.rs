//! Resolving a name to the `dictitem_T` that holds it.
//!
//! `find_var_ht_dict` picks the scope from the name's prefix, `find_var_in_ht`
//! finds the entry in it (and is where a bare `g:`/`b:`/`l:` becomes the
//! scope's own dictionary item), and `eval_variable` is the whole path an
//! expression takes.  `get_user_var_name` walks the same scopes backwards,
//! for completion.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

static varnamebuf: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());

static varnamebuflen: GlobalCell<size_t> = GlobalCell::new(0 as size_t);

pub unsafe extern "C" fn cat_prefix_varname(
    mut prefix: ::core::ffi::c_int,
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut len: size_t = strlen(name).wrapping_add(3 as size_t);
        if len > varnamebuflen.get() {
            xfree(varnamebuf.get() as *mut ::core::ffi::c_void);
            len = len.wrapping_add(10 as size_t);
            varnamebuf.set(xmalloc(len) as *mut ::core::ffi::c_char);
            varnamebuflen.set(len);
        }
        *varnamebuf.get() = prefix as ::core::ffi::c_char;
        *(*varnamebuf.ptr()).offset(1 as ::core::ffi::c_int as isize) = ':' as ::core::ffi::c_char;
        strcpy(
            (*varnamebuf.ptr()).offset(2 as ::core::ffi::c_int as isize),
            name as *mut ::core::ffi::c_char,
        );
        return varnamebuf.get();
    }
}

pub unsafe extern "C" fn get_user_var_name(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        static gdone: GlobalCell<size_t> = GlobalCell::new(0);
        static bdone: GlobalCell<size_t> = GlobalCell::new(0);
        static wdone: GlobalCell<size_t> = GlobalCell::new(0);
        static tdone: GlobalCell<size_t> = GlobalCell::new(0);
        static vidx: GlobalCell<size_t> = GlobalCell::new(0);
        static hi: GlobalCell<*mut hashitem_T> =
            GlobalCell::new(::core::ptr::null_mut::<hashitem_T>());
        if idx == 0 as ::core::ffi::c_int {
            vidx.set(0 as size_t);
            wdone.set(vidx.get());
            bdone.set(wdone.get());
            gdone.set(bdone.get());
            tdone.set(0 as size_t);
        }
        if gdone.get() < (*globvardict.ptr()).dv_hashtab.ht_used {
            let c2rust_fresh0 = gdone.get();
            gdone.set((*gdone.ptr()).wrapping_add(1));
            if c2rust_fresh0 == 0 as size_t {
                hi.set((*globvardict.ptr()).dv_hashtab.ht_array);
            } else {
                hi.set((*hi.ptr()).offset(1));
            }
            while (*hi.get()).hi_key.is_null()
                || (*hi.get()).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
            {
                hi.set((*hi.ptr()).offset(1));
            }
            if strncmp(
                b"g:\0".as_ptr() as *const ::core::ffi::c_char,
                (*xp).xp_pattern,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                return cat_prefix_varname('g' as ::core::ffi::c_int, (*hi.get()).hi_key);
            }
            return (*hi.get()).hi_key;
        }
        let mut ht: *const hashtab_T =
            &raw mut (*(*(*(prevwin_curwin as unsafe extern "C" fn() -> *mut win_T)()).w_buffer)
                .b_vars)
                .dv_hashtab;
        if bdone.get() < (*ht).ht_used {
            let c2rust_fresh1 = bdone.get();
            bdone.set((*bdone.ptr()).wrapping_add(1));
            if c2rust_fresh1 == 0 as size_t {
                hi.set((*ht).ht_array);
            } else {
                hi.set((*hi.ptr()).offset(1));
            }
            while (*hi.get()).hi_key.is_null()
                || (*hi.get()).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
            {
                hi.set((*hi.ptr()).offset(1));
            }
            return cat_prefix_varname('b' as ::core::ffi::c_int, (*hi.get()).hi_key);
        }
        ht = &raw mut (*(*(prevwin_curwin as unsafe extern "C" fn() -> *mut win_T)()).w_vars)
            .dv_hashtab;
        if wdone.get() < (*ht).ht_used {
            let c2rust_fresh2 = wdone.get();
            wdone.set((*wdone.ptr()).wrapping_add(1));
            if c2rust_fresh2 == 0 as size_t {
                hi.set((*ht).ht_array);
            } else {
                hi.set((*hi.ptr()).offset(1));
            }
            while (*hi.get()).hi_key.is_null()
                || (*hi.get()).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
            {
                hi.set((*hi.ptr()).offset(1));
            }
            return cat_prefix_varname('w' as ::core::ffi::c_int, (*hi.get()).hi_key);
        }
        ht = &raw mut (*(*curtab.get()).tp_vars).dv_hashtab;
        if tdone.get() < (*ht).ht_used {
            let c2rust_fresh3 = tdone.get();
            tdone.set((*tdone.ptr()).wrapping_add(1));
            if c2rust_fresh3 == 0 as size_t {
                hi.set((*ht).ht_array);
            } else {
                hi.set((*hi.ptr()).offset(1));
            }
            while (*hi.get()).hi_key.is_null()
                || (*hi.get()).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
            {
                hi.set((*hi.ptr()).offset(1));
            }
            return cat_prefix_varname('t' as ::core::ffi::c_int, (*hi.get()).hi_key);
        }
        if vidx.get()
            < ::core::mem::size_of::<[vimvar; 106]>()
                .wrapping_div(::core::mem::size_of::<vimvar>())
                .wrapping_div(
                    (::core::mem::size_of::<[vimvar; 106]>()
                        .wrapping_rem(::core::mem::size_of::<vimvar>())
                        == 0) as ::core::ffi::c_int as usize,
                )
        {
            let c2rust_fresh4 = vidx.get();
            vidx.set((*vidx.ptr()).wrapping_add(1));
            return cat_prefix_varname(
                'v' as ::core::ffi::c_int,
                get_vim_var_name(c2rust_fresh4 as VimVarIndex),
            );
        }
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            varnamebuf.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        varnamebuflen.set(0 as size_t);
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub unsafe extern "C" fn eval_variable(
    mut name: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut rettv: *mut typval_T,
    mut dip: *mut *mut dictitem_T,
    mut verbose: bool,
    mut no_autoload: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut ret: ::core::ffi::c_int = OK;
        let mut tv: *mut typval_T = ::core::ptr::null_mut::<typval_T>();
        let mut v: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
        v = find_var(
            name,
            len as size_t,
            ::core::ptr::null_mut::<*mut hashtab_T>(),
            no_autoload as ::core::ffi::c_int,
        );
        if !v.is_null() {
            tv = &raw mut (*v).di_tv;
            if !dip.is_null() {
                *dip = v;
            }
        }
        if tv.is_null() {
            if !rettv.is_null() && verbose as ::core::ffi::c_int != 0 {
                semsg(
                    gettext(
                        b"E121: Undefined variable: %.*s\0".as_ptr() as *const ::core::ffi::c_char
                    ),
                    len,
                    name,
                );
            }
            ret = FAIL;
        } else if !rettv.is_null() {
            tv_copy(tv, rettv);
        }
        return ret;
    }
}

pub unsafe extern "C" fn check_vars(mut name: *const ::core::ffi::c_char, mut len: size_t) {
    unsafe {
        if (*eval_lavars_used.ptr()).is_null() {
            return;
        }
        let mut varname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut ht: *mut hashtab_T = find_var_ht(name, len, &raw mut varname);
        if ht == get_funccal_local_ht() || ht == get_funccal_args_ht() {
            if !find_var(name, len, ::core::ptr::null_mut::<*mut hashtab_T>(), true_0).is_null() {
                *eval_lavars_used.get() = true_0 != 0;
            }
        }
    }
}

pub unsafe extern "C" fn find_var(
    name: *const ::core::ffi::c_char,
    name_len: size_t,
    mut htp: *mut *mut hashtab_T,
    mut no_autoload: ::core::ffi::c_int,
) -> *mut dictitem_T {
    unsafe {
        let mut varname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let ht: *mut hashtab_T = find_var_ht(name, name_len, &raw mut varname);
        if !htp.is_null() {
            *htp = ht;
        }
        if ht.is_null() {
            return ::core::ptr::null_mut::<dictitem_T>();
        }
        let ret: *mut dictitem_T = find_var_in_ht(
            ht,
            *name as ::core::ffi::c_int,
            varname,
            name_len.wrapping_sub(varname.offset_from(name) as size_t),
            (no_autoload != 0 || !htp.is_null()) as ::core::ffi::c_int,
        );
        if !ret.is_null() {
            return ret;
        }
        return find_var_in_scoped_ht(
            name,
            name_len,
            (no_autoload != 0 || !htp.is_null()) as ::core::ffi::c_int,
        );
    }
}

pub unsafe extern "C" fn find_var_in_ht(
    ht: *mut hashtab_T,
    mut htname: ::core::ffi::c_int,
    varname: *const ::core::ffi::c_char,
    varname_len: size_t,
    mut no_autoload: ::core::ffi::c_int,
) -> *mut dictitem_T {
    unsafe {
        if varname_len == 0 as size_t {
            match htname {
                115 => {
                    return &raw mut (*(**((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
                        .offset(
                            ((*current_sctx.ptr()).sc_sid as ::core::ffi::c_int
                                - 1 as ::core::ffi::c_int) as isize,
                        ))
                    .sn_vars)
                        .sv_var as *mut dictitem_T;
                }
                103 => return globvars_var.ptr() as *mut dictitem_T,
                118 => return vimvars_var.ptr() as *mut dictitem_T,
                98 => return &raw mut (*curbuf.get()).b_bufvar as *mut dictitem_T,
                119 => return &raw mut (*curwin.get()).w_winvar as *mut dictitem_T,
                116 => return &raw mut (*curtab.get()).tp_winvar as *mut dictitem_T,
                108 => return get_funccal_local_var(),
                97 => return get_funccal_args_var(),
                _ => {}
            }
            return ::core::ptr::null_mut::<dictitem_T>();
        }
        let mut hi: *mut hashitem_T = hash_find_len(ht, varname, varname_len);
        if (*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
        {
            if ht == get_globvar_ht() && no_autoload == 0 {
                if !script_autoload(varname, varname_len, false_0 != 0)
                    || aborting() as ::core::ffi::c_int != 0
                {
                    return ::core::ptr::null_mut::<dictitem_T>();
                }
                hi = hash_find_len(ht, varname, varname_len);
            }
            if (*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
            {
                return ::core::ptr::null_mut::<dictitem_T>();
            }
        }
        return (*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize)) as *mut dictitem_T;
    }
}

pub(crate) unsafe extern "C" fn find_var_ht_dict(
    mut name: *const ::core::ffi::c_char,
    name_len: size_t,
    mut varname: *mut *const ::core::ffi::c_char,
    mut d: *mut *mut dict_T,
) -> *mut hashtab_T {
    unsafe {
        *d = ::core::ptr::null_mut::<dict_T>();
        if name_len == 0 as size_t {
            return ::core::ptr::null_mut::<hashtab_T>();
        }
        if name_len == 1 as size_t
            || *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != ':' as ::core::ffi::c_int
        {
            if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == ':' as ::core::ffi::c_int
                || *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == AUTOLOAD_CHAR
            {
                return ::core::ptr::null_mut::<hashtab_T>();
            }
            *varname = name;
            let mut hi: *mut hashitem_T = hash_find_len(compat_hashtab.ptr(), name, name_len);
            if !((*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                return compat_hashtab.ptr();
            }
            *d = get_funccal_local_dict();
            if (*d).is_null() {
                *d = get_globvar_dict();
            }
        } else {
            *varname = name.offset(2 as ::core::ffi::c_int as isize);
            if *name as ::core::ffi::c_int == 'g' as ::core::ffi::c_int {
                *d = get_globvar_dict();
            } else if name_len > 2 as size_t
                && (!memchr(
                    name.offset(2 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    ':' as ::core::ffi::c_int,
                    name_len.wrapping_sub(2 as size_t),
                )
                .is_null()
                    || !memchr(
                        name.offset(2 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                        AUTOLOAD_CHAR,
                        name_len.wrapping_sub(2 as size_t),
                    )
                    .is_null())
            {
                return ::core::ptr::null_mut::<hashtab_T>();
            }
            if *name as ::core::ffi::c_int == 'b' as ::core::ffi::c_int {
                *d = (*curbuf.get()).b_vars;
            } else if *name as ::core::ffi::c_int == 'w' as ::core::ffi::c_int {
                *d = (*curwin.get()).w_vars;
            } else if *name as ::core::ffi::c_int == 't' as ::core::ffi::c_int {
                *d = (*curtab.get()).tp_vars;
            } else if *name as ::core::ffi::c_int == 'v' as ::core::ffi::c_int {
                *d = get_vimvar_dict();
            } else if *name as ::core::ffi::c_int == 'a' as ::core::ffi::c_int {
                *d = get_funccal_args_dict();
            } else if *name as ::core::ffi::c_int == 'l' as ::core::ffi::c_int {
                *d = get_funccal_local_dict();
            } else if *name as ::core::ffi::c_int == 's' as ::core::ffi::c_int
                && ((*current_sctx.ptr()).sc_sid > 0 as ::core::ffi::c_int
                    || (*current_sctx.ptr()).sc_sid == SID_STR
                    || (*current_sctx.ptr()).sc_sid == SID_LUA)
                && (*current_sctx.ptr()).sc_sid <= (*script_items.ptr()).ga_len
            {
                nlua_set_sctx(current_sctx.ptr());
                if (*current_sctx.ptr()).sc_sid == SID_STR
                    || (*current_sctx.ptr()).sc_sid == SID_LUA
                {
                    new_script_item(
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        &raw mut (*current_sctx.ptr()).sc_sid,
                    );
                }
                *d = &raw mut (*(**((*script_items.ptr()).ga_data as *mut *mut scriptitem_T)
                    .offset(
                        ((*current_sctx.ptr()).sc_sid as ::core::ffi::c_int
                            - 1 as ::core::ffi::c_int) as isize,
                    ))
                .sn_vars)
                    .sv_dict;
            }
        }
        return if !(*d).is_null() {
            &raw mut (**d).dv_hashtab
        } else {
            ::core::ptr::null_mut::<hashtab_T>()
        };
    }
}

pub unsafe extern "C" fn find_var_ht(
    mut name: *const ::core::ffi::c_char,
    name_len: size_t,
    mut varname: *mut *const ::core::ffi::c_char,
) -> *mut hashtab_T {
    unsafe {
        let mut d: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
        return find_var_ht_dict(name, name_len, varname, &raw mut d);
    }
}

pub unsafe extern "C" fn get_var_value(
    name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut v: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
        v = find_var(
            name,
            strlen(name),
            ::core::ptr::null_mut::<*mut hashtab_T>(),
            false_0,
        );
        if v.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        return tv_get_string(&raw mut (*v).di_tv) as *mut ::core::ffi::c_char;
    }
}

pub unsafe extern "C" fn var_exists(mut var: *const ::core::ffi::c_char) -> bool {
    unsafe {
        let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut n: bool = false_0 != 0;
        let mut name: *const ::core::ffi::c_char = var;
        let len: ::core::ffi::c_int =
            get_name_len(&raw mut var, &raw mut tofree, true_0 != 0, false_0 != 0);
        if len > 0 as ::core::ffi::c_int {
            let mut tv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            if !tofree.is_null() {
                name = tofree;
            }
            n = eval_variable(
                name,
                len,
                &raw mut tv,
                ::core::ptr::null_mut::<*mut dictitem_T>(),
                false_0 != 0,
                true_0 != 0,
            ) == OK;
            if n {
                n = handle_subscript(
                    &raw mut var,
                    &raw mut tv,
                    EVALARG_EVALUATE.ptr(),
                    false_0 != 0,
                ) == OK;
                if n {
                    tv_clear(&raw mut tv);
                }
            }
        }
        if *var as ::core::ffi::c_int != NUL {
            n = false_0 != 0;
        }
        xfree(tofree as *mut ::core::ffi::c_void);
        return n;
    }
}
