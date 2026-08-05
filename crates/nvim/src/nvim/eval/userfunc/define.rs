//! `:function` itself -- defining, and the header a listing prints.
//!
//! `ex_function` decides which of the four things the command is (define,
//! list one, list a pattern, list everything), builds the `ufunc_T` and
//! installs it in the table.  `list_func_head` prints the `function
//! Name(a, b = 1, ...) dict abort range` line, which is the same text in a
//! listing and in a `:verbose` report.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn function_list_modified(
    prev_ht_changed: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if prev_ht_changed != (*func_hashtab.ptr()).ht_changed {
            emsg(gettext(E_FUNCTION_LIST_WAS_MODIFIED.as_ptr()));
            return true_0;
        }
        return false_0;
    }
}

pub(crate) unsafe extern "C" fn list_func_head(
    mut fp: *mut ufunc_T,
    mut indent: bool,
    mut force: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let prev_ht_changed: ::core::ffi::c_int = (*func_hashtab.ptr()).ht_changed;
        msg_start();
        if function_list_modified(prev_ht_changed) != 0 {
            return FAIL;
        }
        if indent {
            msg_puts(b"   \0".as_ptr() as *const ::core::ffi::c_char);
        }
        msg_puts(if force as ::core::ffi::c_int != 0 {
            b"function! \0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"function \0".as_ptr() as *const ::core::ffi::c_char
        });
        if !(*fp).uf_name_exp.is_null() {
            msg_puts((*fp).uf_name_exp);
        } else {
            msg_puts(&raw mut (*fp).uf_name as *mut ::core::ffi::c_char);
        }
        msg_putchar('(' as ::core::ffi::c_int);
        let mut j: ::core::ffi::c_int = 0;
        j = 0 as ::core::ffi::c_int;
        while j < (*fp).uf_args.ga_len {
            if j != 0 {
                msg_puts(b", \0".as_ptr() as *const ::core::ffi::c_char);
            }
            msg_puts(*((*fp).uf_args.ga_data as *mut *mut ::core::ffi::c_char).offset(j as isize));
            if j >= (*fp).uf_args.ga_len - (*fp).uf_def_args.ga_len {
                msg_puts(b" = \0".as_ptr() as *const ::core::ffi::c_char);
                msg_puts(
                    *((*fp).uf_def_args.ga_data as *mut *mut ::core::ffi::c_char)
                        .offset((j - (*fp).uf_args.ga_len + (*fp).uf_def_args.ga_len) as isize),
                );
            }
            j += 1;
        }
        if (*fp).uf_varargs != 0 {
            if j != 0 {
                msg_puts(b", \0".as_ptr() as *const ::core::ffi::c_char);
            }
            msg_puts(b"...\0".as_ptr() as *const ::core::ffi::c_char);
        }
        msg_putchar(')' as ::core::ffi::c_int);
        if (*fp).uf_flags & FC_ABORT != 0 {
            msg_puts(b" abort\0".as_ptr() as *const ::core::ffi::c_char);
        }
        if (*fp).uf_flags & FC_RANGE != 0 {
            msg_puts(b" range\0".as_ptr() as *const ::core::ffi::c_char);
        }
        if (*fp).uf_flags & FC_DICT != 0 {
            msg_puts(b" dict\0".as_ptr() as *const ::core::ffi::c_char);
        }
        if (*fp).uf_flags & FC_CLOSURE != 0 {
            msg_puts(b" closure\0".as_ptr() as *const ::core::ffi::c_char);
        }
        msg_clr_eos();
        if p_verbose.get() > 0 as OptInt {
            last_set_msg((*fp).uf_script_ctx);
        }
        return OK;
    }
}

pub unsafe fn ex_function(mut eap: *mut exarg_T) {
    unsafe {
        let mut sourcing_lnum_top: linenr_T = 0;
        let mut namelen: size_t = 0;
        let mut line_to_free: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut line_arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut newargs: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        let mut default_args: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        let mut newlines: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        let mut varargs: ::core::ffi::c_int = false_0;
        let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut fp: *mut ufunc_T = ::core::ptr::null_mut::<ufunc_T>();
        let mut free_fp: bool = false_0 != 0;
        let mut overwrite: bool = false_0 != 0;
        let mut fudi: funcdict_T = funcdict_T {
            fd_dict: ::core::ptr::null_mut::<dict_T>(),
            fd_newkey: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            fd_di: ::core::ptr::null_mut::<dictitem_T>(),
        };
        static func_nr: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        let mut ht: *mut hashtab_T = ::core::ptr::null_mut::<hashtab_T>();
        let mut show_block: bool = false_0 != 0;
        if ends_excmd(*(*eap).arg as ::core::ffi::c_int) != 0 {
            if (*eap).skip == 0 {
                list_functions(::core::ptr::null_mut::<regmatch_T>());
            }
            (*eap).nextcmd = check_nextcmd((*eap).arg);
            return;
        }
        if *(*eap).arg as ::core::ffi::c_int == '/' as ::core::ffi::c_int {
            let mut p: *mut ::core::ffi::c_char = list_functions_matching_pat(eap);
            (*eap).nextcmd = check_nextcmd(p);
            return;
        }
        let mut p_0: *mut ::core::ffi::c_char = (*eap).arg;
        let mut name: *mut ::core::ffi::c_char = save_function_name(
            &raw mut p_0,
            (*eap).skip != 0,
            TFN_NO_AUTOLOAD as ::core::ffi::c_int,
            &raw mut fudi,
        );
        let mut paren: ::core::ffi::c_int =
            !vim_strchr(p_0, '(' as ::core::ffi::c_int).is_null() as ::core::ffi::c_int;
        if name.is_null() && (fudi.fd_dict.is_null() || paren == 0) && (*eap).skip == 0 {
            if !aborting() {
                if !fudi.fd_newkey.is_null() {
                    semsg(
                        gettext(&raw const e_dictkey as *const ::core::ffi::c_char),
                        fudi.fd_newkey,
                    );
                }
                xfree(fudi.fd_newkey as *mut ::core::ffi::c_void);
                return;
            }
            (*eap).skip = true_0;
        }
        let saved_did_emsg: ::core::ffi::c_int = did_emsg.get();
        did_emsg.set(false_0);
        '_ret_free: {
            if paren == 0 {
                fp = list_one_function(eap, name, p_0);
            } else {
                p_0 = skipwhite(p_0);
                if *p_0 as ::core::ffi::c_int != '(' as ::core::ffi::c_int {
                    if (*eap).skip == 0 {
                        semsg(
                            gettext(
                                b"E124: Missing '(': %s\0".as_ptr() as *const ::core::ffi::c_char
                            ),
                            (*eap).arg,
                        );
                        break '_ret_free;
                    } else if !vim_strchr(p_0, '(' as ::core::ffi::c_int).is_null() {
                        p_0 = vim_strchr(p_0, '(' as ::core::ffi::c_int);
                    }
                }
                p_0 = skipwhite(p_0.offset(1 as ::core::ffi::c_int as isize));
                ga_init(
                    &raw mut newargs,
                    ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
                    3 as ::core::ffi::c_int,
                );
                ga_init(
                    &raw mut newlines,
                    ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
                    3 as ::core::ffi::c_int,
                );
                if (*eap).skip == 0 {
                    if !name.is_null() {
                        arg = name;
                    } else {
                        arg = fudi.fd_newkey;
                    }
                    if !arg.is_null() && (fudi.fd_di.is_null() || !tv_is_func((*fudi.fd_di).di_tv))
                    {
                        let mut name_base: *mut ::core::ffi::c_char = arg;
                        if arg != fudi.fd_newkey {
                            if *arg as uint8_t as ::core::ffi::c_int == K_SPECIAL {
                                name_base = vim_strchr(arg, '_' as ::core::ffi::c_int);
                                if name_base.is_null() {
                                    name_base = arg.offset(3 as ::core::ffi::c_int as isize);
                                } else {
                                    name_base = name_base.offset(1);
                                }
                            }
                            let mut i: ::core::ffi::c_int = 0;
                            i = 0 as ::core::ffi::c_int;
                            while *name_base.offset(i as isize) as ::core::ffi::c_int != NUL
                                && (if i == 0 as ::core::ffi::c_int {
                                    eval_isnamec1(
                                        *name_base.offset(i as isize) as ::core::ffi::c_int
                                    ) as ::core::ffi::c_int
                                } else {
                                    eval_isnamec(*name_base.offset(i as isize) as ::core::ffi::c_int)
                                        as ::core::ffi::c_int
                                }) != 0
                            {
                                i += 1;
                            }
                            if *name_base.offset(i as isize) as ::core::ffi::c_int != NUL {
                                emsg_funcname(
                                    &raw const e_invarg2 as *const ::core::ffi::c_char,
                                    arg,
                                );
                                break '_ret_free;
                            }
                        }
                    }
                    if !fudi.fd_dict.is_null()
                        && (*fudi.fd_dict).dv_scope as ::core::ffi::c_uint
                            == VAR_DEF_SCOPE as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        emsg(gettext(
                            b"E862: Cannot use g: here\0".as_ptr() as *const ::core::ffi::c_char
                        ));
                        break '_ret_free;
                    }
                }
                '_errret_keep: {
                    if get_function_args(
                        &raw mut p_0,
                        ')' as ::core::ffi::c_char,
                        &raw mut newargs,
                        &raw mut varargs,
                        &raw mut default_args,
                        (*eap).skip != 0,
                    ) != FAIL
                    {
                        if KeyTyped.get() as ::core::ffi::c_int != 0
                            && ui_has(kUICmdline) as ::core::ffi::c_int != 0
                        {
                            show_block = true_0 != 0;
                            ui_ext_cmdline_block_append(0 as size_t, (*eap).cmd);
                        }
                        '_erret: {
                            loop {
                                p_0 = skipwhite(p_0);
                                if strncmp(
                                    p_0,
                                    b"range\0".as_ptr() as *const ::core::ffi::c_char,
                                    5 as size_t,
                                ) == 0 as ::core::ffi::c_int
                                {
                                    flags |= FC_RANGE;
                                    p_0 = p_0.offset(5 as ::core::ffi::c_int as isize);
                                } else if strncmp(
                                    p_0,
                                    b"dict\0".as_ptr() as *const ::core::ffi::c_char,
                                    4 as size_t,
                                ) == 0 as ::core::ffi::c_int
                                {
                                    flags |= FC_DICT;
                                    p_0 = p_0.offset(4 as ::core::ffi::c_int as isize);
                                } else if strncmp(
                                    p_0,
                                    b"abort\0".as_ptr() as *const ::core::ffi::c_char,
                                    5 as size_t,
                                ) == 0 as ::core::ffi::c_int
                                {
                                    flags |= FC_ABORT;
                                    p_0 = p_0.offset(5 as ::core::ffi::c_int as isize);
                                } else {
                                    if strncmp(
                                        p_0,
                                        b"closure\0".as_ptr() as *const ::core::ffi::c_char,
                                        7 as size_t,
                                    ) != 0 as ::core::ffi::c_int
                                    {
                                        break;
                                    }
                                    flags |= FC_CLOSURE;
                                    p_0 = p_0.offset(7 as ::core::ffi::c_int as isize);
                                    if !(*current_funccal.ptr()).is_null() {
                                        continue;
                                    }
                                    emsg_funcname(
                                        b"E932: Closure function should not be at top level: %s\0"
                                            .as_ptr()
                                            as *const ::core::ffi::c_char,
                                        if name.is_null() {
                                            b"\0".as_ptr() as *const ::core::ffi::c_char
                                        } else {
                                            name as *const ::core::ffi::c_char
                                        },
                                    );
                                    break '_erret;
                                }
                            }
                            if *p_0 as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
                                line_arg = p_0.offset(1 as ::core::ffi::c_int as isize);
                            } else if *p_0 as ::core::ffi::c_int != NUL
                                && *p_0 as ::core::ffi::c_int != '"' as ::core::ffi::c_int
                                && (*eap).skip == 0
                                && did_emsg.get() == 0
                            {
                                semsg(
                                    gettext(
                                        &raw const e_trailing_arg as *const ::core::ffi::c_char,
                                    ),
                                    p_0,
                                );
                            }
                            if KeyTyped.get() {
                                if (*eap).skip == 0 && (*eap).forceit == 0 {
                                    if !fudi.fd_dict.is_null() && fudi.fd_newkey.is_null() {
                                        emsg(gettext(E_FUNCDICT.as_ptr()));
                                    } else if !name.is_null() && !find_func(name).is_null() {
                                        emsg_funcname(E_FUNCEXTS.as_ptr(), name);
                                    }
                                }
                                if (*eap).skip == 0 && did_emsg.get() != 0 {
                                    break '_erret;
                                } else {
                                    if !ui_has(kUICmdline) {
                                        msg_putchar('\n' as ::core::ffi::c_int);
                                    }
                                    cmdline_row.set(msg_row.get());
                                }
                            }
                            sourcing_lnum_top = (*((*exestack.ptr()).ga_data as *mut estack_T)
                                .offset(
                                    ((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize,
                                ))
                            .es_lnum;
                            if !(get_function_body(
                                eap,
                                &raw mut newlines,
                                line_arg,
                                &raw mut line_to_free,
                                show_block,
                            ) == FAIL
                                || (*eap).skip != 0)
                            {
                                namelen = 0 as size_t;
                                if fudi.fd_dict.is_null() {
                                    let mut v: *mut dictitem_T =
                                        find_var(name, strlen(name), &raw mut ht, false);
                                    if !v.is_null()
                                        && (*v).di_tv.v_type as ::core::ffi::c_uint
                                            == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        emsg_funcname(
                                            b"E707: Function name conflicts with variable: %s\0"
                                                .as_ptr()
                                                as *const ::core::ffi::c_char,
                                            name,
                                        );
                                        break '_erret;
                                    } else {
                                        fp = find_func(name);
                                        if !fp.is_null() {
                                            if (*eap).forceit == 0
                                                && ((*fp).uf_script_ctx.sc_sid
                                                    != (*current_sctx.ptr()).sc_sid
                                                    || (*fp).uf_script_ctx.sc_seq
                                                        == (*current_sctx.ptr()).sc_seq)
                                            {
                                                emsg_funcname(E_FUNCEXTS.as_ptr(), name);
                                                break '_errret_keep;
                                            } else if (*fp).uf_calls > 0 as ::core::ffi::c_int {
                                                emsg_funcname(
                                                b"E127: Cannot redefine function %s: It is in use\0"
                                                    .as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                name,
                                            );
                                                break '_errret_keep;
                                            } else if (*fp).uf_refcount > 1 as ::core::ffi::c_int {
                                                (*fp).uf_refcount -= 1;
                                                (*fp).uf_flags |= FC_REMOVED;
                                                fp = ::core::ptr::null_mut::<ufunc_T>();
                                                overwrite = true_0 != 0;
                                            } else {
                                                let mut exp_name: *mut ::core::ffi::c_char =
                                                    (*fp).uf_name_exp;
                                                let mut ptr_: *mut *mut ::core::ffi::c_void =
                                                    &raw mut name as *mut *mut ::core::ffi::c_void;
                                                xfree(*ptr_);
                                                *ptr_ = NULL;
                                                let _ = *ptr_;
                                                (*fp).uf_name_exp =
                                                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                                                func_clear_items(fp);
                                                (*fp).uf_name_exp = exp_name;
                                                (*fp).uf_profiling = false_0;
                                                (*fp).uf_prof_initialized = false_0;
                                            }
                                        }
                                    }
                                } else {
                                    let mut numbuf: [::core::ffi::c_char; 65] = [0; 65];
                                    fp = ::core::ptr::null_mut::<ufunc_T>();
                                    if fudi.fd_newkey.is_null() && (*eap).forceit == 0 {
                                        emsg(gettext(E_FUNCDICT.as_ptr()));
                                        break '_erret;
                                    } else {
                                        if fudi.fd_di.is_null() {
                                            if value_check_lock(
                                                (*fudi.fd_dict).dv_lock,
                                                (*eap).arg,
                                                TV_CSTRING as size_t,
                                            ) {
                                                break '_erret;
                                            }
                                        } else if value_check_lock(
                                            (*fudi.fd_di).di_tv.v_lock,
                                            (*eap).arg,
                                            TV_CSTRING as size_t,
                                        ) {
                                            break '_erret;
                                        }
                                        xfree(name as *mut ::core::ffi::c_void);
                                        (*func_nr.ptr()) += 1;
                                        namelen = snprintf(
                                            &raw mut numbuf as *mut ::core::ffi::c_char,
                                            ::core::mem::size_of::<[::core::ffi::c_char; 65]>(),
                                            b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                                            func_nr.get(),
                                        )
                                            as size_t;
                                        name = xmemdupz(
                                            &raw mut numbuf as *mut ::core::ffi::c_char
                                                as *const ::core::ffi::c_void,
                                            namelen,
                                        )
                                            as *mut ::core::ffi::c_char;
                                    }
                                }
                                if fp.is_null() {
                                    if fudi.fd_dict.is_null()
                                        && !vim_strchr(name, AUTOLOAD_CHAR).is_null()
                                    {
                                        let mut j: ::core::ffi::c_int = FAIL;
                                        if !(*((*exestack.ptr()).ga_data as *mut estack_T).offset(
                                            ((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int)
                                                as isize,
                                        ))
                                        .es_name
                                        .is_null()
                                        {
                                            let mut scriptname: *mut ::core::ffi::c_char =
                                                autoload_name(name, strlen(name));
                                            p_0 = vim_strchr(scriptname, '/' as ::core::ffi::c_int);
                                            let mut plen: ::core::ffi::c_int =
                                                strlen(p_0) as ::core::ffi::c_int;
                                            let mut slen: ::core::ffi::c_int = strlen(
                                                (*((*exestack.ptr()).ga_data as *mut estack_T)
                                                    .offset(
                                                        ((*exestack.ptr()).ga_len
                                                            - 1 as ::core::ffi::c_int)
                                                            as isize,
                                                    ))
                                                .es_name,
                                            )
                                                as ::core::ffi::c_int;
                                            if slen > plen
                                                && path_fnamecmp(
                                                    p_0,
                                                    (*((*exestack.ptr()).ga_data as *mut estack_T)
                                                        .offset(
                                                            ((*exestack.ptr()).ga_len
                                                                - 1 as ::core::ffi::c_int)
                                                                as isize,
                                                        ))
                                                    .es_name
                                                    .offset(slen as isize)
                                                    .offset(-(plen as isize)),
                                                ) == 0 as ::core::ffi::c_int
                                            {
                                                j = OK;
                                            }
                                            xfree(scriptname as *mut ::core::ffi::c_void);
                                        }
                                        if j == FAIL {
                                            semsg(
                                            gettext(
                                                b"E746: Function name does not match script file name: %s\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            name,
                                        );
                                            break '_erret;
                                        }
                                    }
                                    if namelen == 0 as size_t {
                                        namelen = strlen(name);
                                    }
                                    fp = alloc_ufunc(name, namelen);
                                    if !fudi.fd_dict.is_null() {
                                        if fudi.fd_di.is_null() {
                                            fudi.fd_di = tv_dict_item_alloc(fudi.fd_newkey);
                                            if tv_dict_add(fudi.fd_dict, fudi.fd_di) == FAIL {
                                                xfree(fudi.fd_di as *mut ::core::ffi::c_void);
                                                let mut ptr__0: *mut *mut ::core::ffi::c_void =
                                                    &raw mut fp as *mut *mut ::core::ffi::c_void;
                                                xfree(*ptr__0);
                                                *ptr__0 = NULL;
                                                let _ = *ptr__0;
                                                break '_erret;
                                            }
                                        } else {
                                            tv_clear(&raw mut (*fudi.fd_di).di_tv);
                                        }
                                        (*fudi.fd_di).di_tv.v_type = VAR_FUNC;
                                        (*fudi.fd_di).di_tv.vval.v_string =
                                            xmemdupz(name as *const ::core::ffi::c_void, namelen)
                                                as *mut ::core::ffi::c_char;
                                        flags |= FC_DICT;
                                    }
                                    if overwrite {
                                        let mut hi: *mut hashitem_T =
                                            hash_find(func_hashtab.ptr(), name);
                                        (*hi).hi_key =
                                            &raw mut (*fp).uf_name as *mut ::core::ffi::c_char;
                                    } else if hash_add(
                                        func_hashtab.ptr(),
                                        &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
                                    ) == FAIL
                                    {
                                        free_fp = true_0 != 0;
                                        break '_erret;
                                    }
                                    (*fp).uf_refcount = 1 as ::core::ffi::c_int;
                                }
                                (*fp).uf_args = newargs;
                                (*fp).uf_def_args = default_args;
                                (*fp).uf_lines = newlines;
                                if flags & FC_CLOSURE != 0 as ::core::ffi::c_int {
                                    register_closure(fp);
                                } else {
                                    (*fp).uf_scoped = ::core::ptr::null_mut::<funccall_T>();
                                }
                                if prof_def_func() {
                                    func_do_profile(fp);
                                }
                                (*fp).uf_varargs = varargs;
                                if sandbox.get() != 0 {
                                    flags |= FC_SANDBOX;
                                }
                                (*fp).uf_flags = flags;
                                (*fp).uf_calls = 0 as ::core::ffi::c_int;
                                (*fp).uf_script_ctx = current_sctx.get();
                                (*fp).uf_script_ctx.sc_lnum += sourcing_lnum_top;
                                nlua_set_sctx(&raw mut (*fp).uf_script_ctx);
                                break '_ret_free;
                            }
                        }
                        if !fp.is_null() {
                            ga_init(
                                &raw mut (*fp).uf_args,
                                ::core::mem::size_of::<*mut ::core::ffi::c_char>()
                                    as ::core::ffi::c_int,
                                1 as ::core::ffi::c_int,
                            );
                            ga_init(
                                &raw mut (*fp).uf_def_args,
                                ::core::mem::size_of::<*mut ::core::ffi::c_char>()
                                    as ::core::ffi::c_int,
                                1 as ::core::ffi::c_int,
                            );
                        }
                    }
                    if !fp.is_null() {
                        let mut ptr__1: *mut *mut ::core::ffi::c_void =
                            &raw mut (*fp).uf_name_exp as *mut *mut ::core::ffi::c_void;
                        xfree(*ptr__1);
                        *ptr__1 = NULL;
                        let _ = *ptr__1;
                    }
                    if free_fp {
                        let mut ptr__2: *mut *mut ::core::ffi::c_void =
                            &raw mut fp as *mut *mut ::core::ffi::c_void;
                        xfree(*ptr__2);
                        *ptr__2 = NULL;
                        let _ = *ptr__2;
                    }
                }
                ga_clear_strings(&raw mut newargs);
                ga_clear_strings(&raw mut default_args);
                ga_clear_strings(&raw mut newlines);
            }
        }
        xfree(line_to_free as *mut ::core::ffi::c_void);
        xfree(fudi.fd_newkey as *mut ::core::ffi::c_void);
        xfree(name as *mut ::core::ffi::c_void);
        (*did_emsg.ptr()) |= saved_did_emsg;
        if show_block {
            ui_ext_cmdline_block_leave();
        }
    }
}
