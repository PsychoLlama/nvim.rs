//! Printing functions back, and `:delfunction`.
//!
//! `list_functions` walks the whole table, `list_functions_matching_pat`
//! the subset a `/pattern/` matches, and `list_one_function` prints one
//! with its numbered body lines.  `ex_delfunction` is here because it is
//! the same argument parse in reverse; `function_exists` and
//! `get_user_func_name` answer `exists('*x')` and completion.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn list_functions(mut regmatch: *mut regmatch_T) {
    unsafe {
        let prev_ht_changed: ::core::ffi::c_int = (*func_hashtab.ptr()).ht_changed;
        let mut todo: size_t = (*func_hashtab.ptr()).ht_used;
        let ht_array: *const hashitem_T = (*func_hashtab.ptr()).ht_array;
        msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
        let mut hi: *const hashitem_T = ht_array;
        while todo > 0 as size_t && !got_int.get() {
            if !((*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                let mut fp: *mut ufunc_T =
                    (*hi).hi_key.offset(-(240 as ::core::ffi::c_ulong as isize)) as *mut ufunc_T;
                todo = todo.wrapping_sub(1);
                if if regmatch.is_null() {
                    (!message_filtered(&raw mut (*fp).uf_name as *mut ::core::ffi::c_char)
                        && !func_name_refcount(&raw mut (*fp).uf_name as *mut ::core::ffi::c_char))
                        as ::core::ffi::c_int
                } else {
                    (*(*__ctype_b_loc()).offset(
                        *(&raw mut (*fp).uf_name as *mut ::core::ffi::c_char) as uint8_t
                            as ::core::ffi::c_int as isize,
                    ) as ::core::ffi::c_int
                        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort
                            as ::core::ffi::c_int
                        == 0
                        && vim_regexec(
                            regmatch,
                            &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
                            0 as colnr_T,
                        ) as ::core::ffi::c_int
                            != 0) as ::core::ffi::c_int
                } != 0
                {
                    if list_func_head(fp, false_0 != 0, false_0 != 0) == FAIL {
                        return;
                    }
                    if function_list_modified(prev_ht_changed) != 0 {
                        return;
                    }
                }
            }
            hi = hi.offset(1);
        }
    }
}

pub(crate) unsafe extern "C" fn list_functions_matching_pat(
    mut eap: *mut exarg_T,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = skip_regexp(
            (*eap).arg.offset(1 as ::core::ffi::c_int as isize),
            '/' as ::core::ffi::c_int,
            true_0,
        );
        if (*eap).skip == 0 {
            let mut regmatch: regmatch_T = regmatch_T {
                regprog: ::core::ptr::null_mut::<regprog_T>(),
                startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
                rm_matchcol: 0,
                rm_ic: false,
            };
            let mut c: ::core::ffi::c_char = *p;
            *p = NUL as ::core::ffi::c_char;
            regmatch.regprog = vim_regcomp(
                (*eap).arg.offset(1 as ::core::ffi::c_int as isize),
                RE_MAGIC,
            );
            *p = c;
            if !regmatch.regprog.is_null() {
                regmatch.rm_ic = p_ic.get() != 0;
                list_functions(&raw mut regmatch);
                vim_regfree(regmatch.regprog);
            }
        }
        if *p as ::core::ffi::c_int == '/' as ::core::ffi::c_int {
            p = p.offset(1);
        }
        return p;
    }
}

pub(crate) unsafe extern "C" fn list_one_function(
    mut eap: *mut exarg_T,
    mut name: *mut ::core::ffi::c_char,
    mut p: *mut ::core::ffi::c_char,
) -> *mut ufunc_T {
    unsafe {
        if ends_excmd(*skipwhite(p) as ::core::ffi::c_int) == 0 {
            semsg(
                gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                p,
            );
            return ::core::ptr::null_mut::<ufunc_T>();
        }
        (*eap).nextcmd = check_nextcmd(p);
        if !(*eap).nextcmd.is_null() {
            *p = NUL as ::core::ffi::c_char;
        }
        if (*eap).skip != 0 || got_int.get() as ::core::ffi::c_int != 0 {
            return ::core::ptr::null_mut::<ufunc_T>();
        }
        let mut fp: *mut ufunc_T = find_func(name);
        if fp.is_null() {
            emsg_funcname(
                b"E123: Undefined function: %s\0".as_ptr() as *const ::core::ffi::c_char,
                name,
            );
            return ::core::ptr::null_mut::<ufunc_T>();
        }
        let prev_ht_changed: ::core::ffi::c_int = (*func_hashtab.ptr()).ht_changed;
        msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
        if list_func_head(fp, (*eap).forceit == 0, (*eap).forceit != 0) != OK {
            return fp;
        }
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while j < (*fp).uf_lines.ga_len && !got_int.get() {
            if !(*((*fp).uf_lines.ga_data as *mut *mut ::core::ffi::c_char).offset(j as isize))
                .is_null()
            {
                msg_putchar('\n' as ::core::ffi::c_int);
                if (*eap).forceit == 0 {
                    msg_outnum(j + 1 as ::core::ffi::c_int);
                    if j < 9 as ::core::ffi::c_int {
                        msg_putchar(' ' as ::core::ffi::c_int);
                    }
                    if j < 99 as ::core::ffi::c_int {
                        msg_putchar(' ' as ::core::ffi::c_int);
                    }
                    if function_list_modified(prev_ht_changed) != 0 {
                        break;
                    }
                }
                msg_prt_line(
                    *((*fp).uf_lines.ga_data as *mut *mut ::core::ffi::c_char).offset(j as isize),
                    false_0 != 0,
                );
                line_breakcheck();
            }
            j += 1;
        }
        if !got_int.get() {
            msg_putchar('\n' as ::core::ffi::c_int);
            if function_list_modified(prev_ht_changed) == 0 {
                msg_puts(if (*eap).forceit != 0 {
                    b"endfunction\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"   endfunction\0".as_ptr() as *const ::core::ffi::c_char
                });
            }
        }
        return fp;
    }
}

pub unsafe extern "C" fn translated_function_exists(mut name: *const ::core::ffi::c_char) -> bool {
    unsafe {
        if builtin_function(name, -1 as ::core::ffi::c_int) {
            return !find_internal_func(name).is_null();
        }
        return !find_func(name).is_null();
    }
}

pub unsafe extern "C" fn function_exists(
    name: *const ::core::ffi::c_char,
    mut no_deref: bool,
) -> bool {
    unsafe {
        let mut nm: *const ::core::ffi::c_char = name;
        let mut n: bool = false_0 != 0;
        let mut flag: ::core::ffi::c_int = TFN_INT as ::core::ffi::c_int
            | TFN_QUIET as ::core::ffi::c_int
            | TFN_NO_AUTOLOAD as ::core::ffi::c_int;
        if no_deref {
            flag |= TFN_NO_DEREF as ::core::ffi::c_int;
        }
        let p: *mut ::core::ffi::c_char = trans_function_name(
            &raw mut nm as *mut *mut ::core::ffi::c_char,
            false_0 != 0,
            flag,
            ::core::ptr::null_mut::<funcdict_T>(),
            ::core::ptr::null_mut::<*mut partial_T>(),
        );
        nm = skipwhite(nm);
        if !p.is_null()
            && (*nm as ::core::ffi::c_int == NUL
                || *nm as ::core::ffi::c_int == '(' as ::core::ffi::c_int)
        {
            n = translated_function_exists(p);
        }
        xfree(p as *mut ::core::ffi::c_void);
        return n;
    }
}

pub unsafe extern "C" fn get_user_func_name(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        static done: GlobalCell<size_t> = GlobalCell::new(0);
        static changed: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
        static hi: GlobalCell<*mut hashitem_T> =
            GlobalCell::new(::core::ptr::null_mut::<hashitem_T>());
        if idx == 0 as ::core::ffi::c_int {
            done.set(0 as size_t);
            hi.set((*func_hashtab.ptr()).ht_array);
            changed.set((*func_hashtab.ptr()).ht_changed);
        }
        '_c2rust_label: {
            if !(*hi.ptr()).is_null() {
            } else {
                __assert_fail(
                    b"hi\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/userfunc.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3083 as ::core::ffi::c_uint,
                    b"char *get_user_func_name(expand_T *, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if changed.get() == (*func_hashtab.ptr()).ht_changed
            && done.get() < (*func_hashtab.ptr()).ht_used
        {
            let c2rust_fresh16 = done.get();
            done.set((*done.ptr()).wrapping_add(1));
            if c2rust_fresh16 > 0 as size_t {
                hi.set((*hi.ptr()).offset(1));
            }
            while (*hi.get()).hi_key.is_null()
                || (*hi.get()).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
            {
                hi.set((*hi.ptr()).offset(1));
            }
            let mut fp: *mut ufunc_T = (*hi.get())
                .hi_key
                .offset(-(240 as ::core::ffi::c_ulong as isize))
                as *mut ufunc_T;
            if (*fp).uf_flags & FC_DICT != 0
                || strncmp(
                    &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
                    b"<lambda>\0".as_ptr() as *const ::core::ffi::c_char,
                    8 as size_t,
                ) == 0 as ::core::ffi::c_int
            {
                return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            if (*fp).uf_namelen.wrapping_add(4 as size_t) >= IOSIZE as size_t {
                return &raw mut (*fp).uf_name as *mut ::core::ffi::c_char;
            }
            let mut len: ::core::ffi::c_int = cat_func_name(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                fp,
            );
            if (*xp).xp_context != EXPAND_USER_FUNC as ::core::ffi::c_int {
                xstrlcpy(
                    (IObuff.ptr() as *mut ::core::ffi::c_char).offset(len as isize),
                    b"(\0".as_ptr() as *const ::core::ffi::c_char,
                    (IOSIZE as size_t).wrapping_sub(len as size_t),
                );
                if (*fp).uf_varargs == 0 && (*fp).uf_args.ga_len <= 0 as ::core::ffi::c_int {
                    len += 1;
                    xstrlcpy(
                        (IObuff.ptr() as *mut ::core::ffi::c_char).offset(len as isize),
                        b")\0".as_ptr() as *const ::core::ffi::c_char,
                        (IOSIZE as size_t).wrapping_sub(len as size_t),
                    );
                }
            }
            return IObuff.ptr() as *mut ::core::ffi::c_char;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub unsafe fn ex_delfunction(mut eap: *mut exarg_T) {
    unsafe {
        let mut fp: *mut ufunc_T = ::core::ptr::null_mut::<ufunc_T>();
        let mut fudi: funcdict_T = funcdict_T {
            fd_dict: ::core::ptr::null_mut::<dict_T>(),
            fd_newkey: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            fd_di: ::core::ptr::null_mut::<dictitem_T>(),
        };
        let mut p: *mut ::core::ffi::c_char = (*eap).arg;
        let mut name: *mut ::core::ffi::c_char = trans_function_name(
            &raw mut p,
            (*eap).skip != 0,
            0 as ::core::ffi::c_int,
            &raw mut fudi,
            ::core::ptr::null_mut::<*mut partial_T>(),
        );
        xfree(fudi.fd_newkey as *mut ::core::ffi::c_void);
        if name.is_null() {
            if !fudi.fd_dict.is_null() && (*eap).skip == 0 {
                emsg(gettext(E_FUNCREF.as_ptr()));
            }
            return;
        }
        if ends_excmd(*skipwhite(p) as ::core::ffi::c_int) == 0 {
            xfree(name as *mut ::core::ffi::c_void);
            semsg(
                gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                p,
            );
            return;
        }
        (*eap).nextcmd = check_nextcmd(p);
        if !(*eap).nextcmd.is_null() {
            *p = NUL as ::core::ffi::c_char;
        }
        if *(*__ctype_b_loc()).offset(*name as uint8_t as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
            && fudi.fd_dict.is_null()
        {
            if (*eap).skip == 0 {
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    (*eap).arg,
                );
            }
            xfree(name as *mut ::core::ffi::c_void);
            return;
        }
        if (*eap).skip == 0 {
            fp = find_func(name);
        }
        xfree(name as *mut ::core::ffi::c_void);
        if (*eap).skip == 0 {
            if fp.is_null() {
                if (*eap).forceit == 0 {
                    semsg(gettext(E_NOFUNC.as_ptr()), (*eap).arg);
                }
                return;
            }
            if (*fp).uf_calls > 0 as ::core::ffi::c_int {
                semsg(
                    gettext(b"E131: Cannot delete function %s: It is in use\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    (*eap).arg,
                );
                return;
            }
            if (*fp).uf_refcount > 2 as ::core::ffi::c_int {
                semsg(
                    gettext(
                        b"Cannot delete function %s: It is being used internally\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                    (*eap).arg,
                );
                return;
            }
            if !fudi.fd_dict.is_null() {
                tv_dict_item_remove(fudi.fd_dict, fudi.fd_di);
            } else if (*fp).uf_refcount
                > (if func_name_refcount(&raw mut (*fp).uf_name as *mut ::core::ffi::c_char)
                    as ::core::ffi::c_int
                    != 0
                {
                    0 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                })
            {
                if func_remove(fp) {
                    (*fp).uf_refcount -= 1;
                }
                (*fp).uf_flags |= FC_DELETED;
            } else {
                func_clear_free(fp, false_0 != 0);
            }
        }
    }
}
