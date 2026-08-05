//! `:let` -- parsing the targets and performing the assignment.
//!
//! `ex_let` splits the command, `ex_let_vars` deals with the `[a, b; rest]`
//! unpack, and the four `ex_let_*` below it are one per kind of target: a
//! variable, an environment variable, an option and a register.  The last
//! three implement the compound operators themselves and never reach
//! `set_var_lval`.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe fn ex_let(mut eap: *mut exarg_T) {
    unsafe {
        let is_const: bool = (*eap).cmdidx as ::core::ffi::c_int == CMD_const as ::core::ffi::c_int;
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        let mut expr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut var_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut semicolon: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut op: [::core::ffi::c_char; 2] = [0; 2];
        let mut argend: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut first: ::core::ffi::c_int = true_0;
        argend = skip_var_list(arg, &raw mut var_count, &raw mut semicolon, false_0 != 0);
        if argend.is_null() {
            return;
        }
        expr = skipwhite(argend);
        let mut concat: bool = strncmp(
            expr,
            b"..=\0".as_ptr() as *const ::core::ffi::c_char,
            3 as size_t,
        ) == 0 as ::core::ffi::c_int;
        let mut has_assign: bool = *expr as ::core::ffi::c_int == '=' as ::core::ffi::c_int
            || !vim_strchr(
                b"+-*/%.\0".as_ptr() as *const ::core::ffi::c_char,
                *expr as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
                && *expr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '=' as ::core::ffi::c_int;
        if !has_assign && !concat {
            if *arg as ::core::ffi::c_int == '[' as ::core::ffi::c_int {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            } else if ends_excmd(*arg as ::core::ffi::c_int) == 0 {
                arg = list_arg_vars(eap, arg, &raw mut first) as *mut ::core::ffi::c_char;
            } else if (*eap).skip == 0 {
                list_glob_vars(&raw mut first);
                list_buf_vars(&raw mut first);
                list_win_vars(&raw mut first);
                list_tab_vars(&raw mut first);
                list_script_vars(&raw mut first);
                list_func_vars(&raw mut first);
                list_vim_vars(&raw mut first);
            }
            (*eap).nextcmd = check_nextcmd(arg);
            return;
        }
        if *expr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '=' as ::core::ffi::c_int
            && *expr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '<' as ::core::ffi::c_int
            && *expr.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '<' as ::core::ffi::c_int
        {
            let mut l: *mut list_T = heredoc_get(
                eap,
                expr.offset(3 as ::core::ffi::c_int as isize),
                false_0 != 0,
            );
            if !l.is_null() {
                tv_list_set_ret(&raw mut rettv, l);
                if (*eap).skip == 0 {
                    op[0 as ::core::ffi::c_int as usize] = '=' as ::core::ffi::c_char;
                    op[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
                    ex_let_vars(
                        (*eap).arg,
                        &raw mut rettv,
                        false_0,
                        semicolon,
                        var_count,
                        is_const as ::core::ffi::c_int,
                        &raw mut op as *mut ::core::ffi::c_char,
                    );
                }
                tv_clear(&raw mut rettv);
            }
            return;
        }
        rettv.v_type = VAR_UNKNOWN;
        op[0 as ::core::ffi::c_int as usize] = '=' as ::core::ffi::c_char;
        op[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        if *expr as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
            if !vim_strchr(
                b"+-*/%.\0".as_ptr() as *const ::core::ffi::c_char,
                *expr as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
            {
                op[0 as ::core::ffi::c_int as usize] = *expr;
                if *expr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int
                    && *expr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '.' as ::core::ffi::c_int
                {
                    expr = expr.offset(1);
                }
            }
            expr = expr.offset(2 as ::core::ffi::c_int as isize);
        } else {
            expr = expr.offset(1 as ::core::ffi::c_int as isize);
        }
        expr = skipwhite(expr);
        if (*eap).skip != 0 {
            (*emsg_skip.ptr()) += 1;
        }
        let mut evalarg: evalarg_T = evalarg_T {
            eval_flags: 0,
            eval_getline: None,
            eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        fill_evalarg_from_eap(&raw mut evalarg, eap, (*eap).skip != 0);
        let mut eval_res: ::core::ffi::c_int = eval0(expr, &raw mut rettv, eap, &raw mut evalarg);
        if (*eap).skip != 0 {
            (*emsg_skip.ptr()) -= 1;
        }
        clear_evalarg(&raw mut evalarg, eap);
        if (*eap).skip == 0 && eval_res != FAIL {
            ex_let_vars(
                (*eap).arg,
                &raw mut rettv,
                false_0,
                semicolon,
                var_count,
                is_const as ::core::ffi::c_int,
                &raw mut op as *mut ::core::ffi::c_char,
            );
        }
        if eval_res != FAIL {
            tv_clear(&raw mut rettv);
        }
    }
}

pub unsafe extern "C" fn ex_let_vars(
    mut arg_start: *mut ::core::ffi::c_char,
    mut tv: *mut typval_T,
    mut copy: ::core::ffi::c_int,
    mut semicolon: ::core::ffi::c_int,
    mut var_count: ::core::ffi::c_int,
    mut is_const: ::core::ffi::c_int,
    mut op: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = arg_start;
        let mut ltv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if *arg as ::core::ffi::c_int != '[' as ::core::ffi::c_int {
            if ex_let_one(arg, tv, copy != 0, is_const != 0, op, op).is_null() {
                return FAIL;
            }
            return OK;
        }
        if (*tv).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
            return FAIL;
        }
        let l: *mut list_T = (*tv).vval.v_list;
        let len: ::core::ffi::c_int = tv_list_len(l);
        if semicolon == 0 as ::core::ffi::c_int && var_count < len {
            emsg(gettext(
                b"E687: Less targets than List items\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return FAIL;
        }
        if var_count - semicolon > len {
            emsg(gettext(
                b"E688: More targets than List items\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return FAIL;
        }
        '_c2rust_label: {
            if !l.is_null() {
            } else {
                __assert_fail(
                    b"l != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/vars.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1043 as ::core::ffi::c_uint,
                    b"int ex_let_vars(char *, typval_T *, int, int, int, int, char *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut item: *mut listitem_T = tv_list_first(l);
        let mut rest_len: size_t = tv_list_len(l) as size_t;
        while *arg as ::core::ffi::c_int != ']' as ::core::ffi::c_int {
            arg = skipwhite(arg.offset(1 as ::core::ffi::c_int as isize));
            arg = ex_let_one(
                arg,
                &raw mut (*item).li_tv,
                true_0 != 0,
                is_const != 0,
                b",;]\0".as_ptr() as *const ::core::ffi::c_char,
                op,
            );
            if arg.is_null() {
                return FAIL;
            }
            rest_len = rest_len.wrapping_sub(1);
            item = (*item).li_next;
            arg = skipwhite(arg);
            if *arg as ::core::ffi::c_int == ';' as ::core::ffi::c_int {
                let rest_list: *mut list_T = tv_list_alloc(rest_len as ptrdiff_t);
                while !item.is_null() {
                    tv_list_append_tv(rest_list, &raw mut (*item).li_tv);
                    item = (*item).li_next;
                }
                ltv.v_type = VAR_LIST;
                ltv.v_lock = VAR_UNLOCKED;
                ltv.vval.v_list = rest_list;
                tv_list_ref(rest_list);
                arg = ex_let_one(
                    skipwhite(arg.offset(1 as ::core::ffi::c_int as isize)),
                    &raw mut ltv,
                    false_0 != 0,
                    is_const != 0,
                    b"]\0".as_ptr() as *const ::core::ffi::c_char,
                    op,
                );
                tv_clear(&raw mut ltv);
                if arg.is_null() {
                    return FAIL;
                }
                break;
            } else if *arg as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                && *arg as ::core::ffi::c_int != ']' as ::core::ffi::c_int
            {
                internal_error(b"ex_let_vars()\0".as_ptr() as *const ::core::ffi::c_char);
                return FAIL;
            }
        }
        return OK;
    }
}

pub unsafe extern "C" fn skip_var_list(
    mut arg: *const ::core::ffi::c_char,
    mut var_count: *mut ::core::ffi::c_int,
    mut semicolon: *mut ::core::ffi::c_int,
    mut silent: bool,
) -> *const ::core::ffi::c_char {
    unsafe {
        if *arg as ::core::ffi::c_int == '[' as ::core::ffi::c_int {
            let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
            let mut p: *const ::core::ffi::c_char = arg;
            loop {
                p = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
                s = skip_var_one(p);
                if s == p {
                    if !silent {
                        semsg(
                            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                            p,
                        );
                    }
                    return ::core::ptr::null::<::core::ffi::c_char>();
                }
                *var_count += 1;
                p = skipwhite(s);
                if *p as ::core::ffi::c_int == ']' as ::core::ffi::c_int {
                    break;
                }
                if *p as ::core::ffi::c_int == ';' as ::core::ffi::c_int {
                    if *semicolon == 1 as ::core::ffi::c_int {
                        if !silent {
                            emsg(gettext(
                                (e_double_semicolon_in_list_of_variables.ptr() as *const _)
                                    as *const ::core::ffi::c_char,
                            ));
                        }
                        return ::core::ptr::null::<::core::ffi::c_char>();
                    }
                    *semicolon = 1 as ::core::ffi::c_int;
                } else if *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int {
                    if !silent {
                        semsg(
                            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                            p,
                        );
                    }
                    return ::core::ptr::null::<::core::ffi::c_char>();
                }
            }
            return p.offset(1 as ::core::ffi::c_int as isize);
        }
        return skip_var_one(arg);
    }
}

unsafe extern "C" fn skip_var_one(
    mut arg: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        if *arg as ::core::ffi::c_int == '@' as ::core::ffi::c_int
            && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            return arg.offset(2 as ::core::ffi::c_int as isize);
        }
        return find_name_end(
            if *arg as ::core::ffi::c_int == '$' as ::core::ffi::c_int
                || *arg as ::core::ffi::c_int == '&' as ::core::ffi::c_int
            {
                arg.offset(1 as ::core::ffi::c_int as isize)
            } else {
                arg
            },
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            FNE_INCL_BR | FNE_CHECK_START,
        );
    }
}

unsafe extern "C" fn ex_let_env(
    mut arg: *mut ::core::ffi::c_char,
    tv: *mut typval_T,
    is_const: bool,
    endchars: *const ::core::ffi::c_char,
    op: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if is_const {
            emsg(gettext(
                b"E996: Cannot lock an environment variable\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut arg_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        arg = arg.offset(1);
        let mut name: *mut ::core::ffi::c_char = arg;
        let mut len: ::core::ffi::c_int =
            get_env_len(&raw mut arg as *mut *const ::core::ffi::c_char);
        if len == 0 as ::core::ffi::c_int {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                name.offset(-(1 as ::core::ffi::c_int as isize)),
            );
        } else if !op.is_null()
            && !vim_strchr(
                b"+-*/%\0".as_ptr() as *const ::core::ffi::c_char,
                *op as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
        {
            semsg(
                gettext(&raw const e_letwrong as *const ::core::ffi::c_char),
                op,
            );
        } else if !endchars.is_null()
            && vim_strchr(endchars, *skipwhite(arg) as uint8_t as ::core::ffi::c_int).is_null()
        {
            emsg(gettext(e_letunexp.get()));
        } else if !check_secure() {
            let mut tofree: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let c1: ::core::ffi::c_char = *name.offset(len as isize);
            *name.offset(len as isize) = NUL as ::core::ffi::c_char;
            let mut p: *const ::core::ffi::c_char = tv_get_string_chk(tv);
            if !p.is_null()
                && !op.is_null()
                && *op as ::core::ffi::c_int == '.' as ::core::ffi::c_int
            {
                let mut s: *mut ::core::ffi::c_char = vim_getenv(name);
                if !s.is_null() {
                    tofree = concat_str(s, p);
                    p = tofree;
                    xfree(s as *mut ::core::ffi::c_void);
                }
            }
            if !p.is_null() {
                vim_setenv_ext(name, p);
                arg_end = arg;
            }
            *name.offset(len as isize) = c1;
            xfree(tofree as *mut ::core::ffi::c_void);
        }
        return arg_end;
    }
}

unsafe extern "C" fn ex_let_option(
    mut arg: *mut ::core::ffi::c_char,
    tv: *mut typval_T,
    is_const: bool,
    endchars: *const ::core::ffi::c_char,
    op: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut error: bool = false;
        let mut is_num: bool = false;
        let mut is_string: bool = false;
        let mut err: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if is_const {
            emsg(gettext(
                b"E996: Cannot lock an option\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut arg_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut opt_idx: OptIndex = kOptAleph;
        let mut opt_flags: ::core::ffi::c_int = 0;
        let p: *mut ::core::ffi::c_char = find_option_var_end(
            &raw mut arg as *mut *const ::core::ffi::c_char,
            &raw mut opt_idx,
            &raw mut opt_flags,
        ) as *mut ::core::ffi::c_char;
        if p.is_null()
            || !endchars.is_null()
                && vim_strchr(endchars, *skipwhite(p) as uint8_t as ::core::ffi::c_int).is_null()
        {
            emsg(gettext(e_letunexp.get()));
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let c1: ::core::ffi::c_char = *p;
        *p = NUL as ::core::ffi::c_char;
        let mut is_tty_opt: bool = is_tty_option(arg);
        let mut hidden: bool = is_option_hidden(opt_idx);
        let mut curval: OptVal = if is_tty_opt as ::core::ffi::c_int != 0 {
            get_tty_option(arg)
        } else {
            get_option_value(opt_idx, opt_flags)
        };
        let mut newval: OptVal = OptVal {
            type_0: kOptValTypeNil,
            data: OptValData { boolean: kFalse },
        };
        if curval.type_0 as ::core::ffi::c_int == kOptValTypeNil as ::core::ffi::c_int {
            semsg(
                gettext(&raw const e_unknown_option2 as *const ::core::ffi::c_char),
                arg,
            );
        } else if !op.is_null()
            && *op as ::core::ffi::c_int != '=' as ::core::ffi::c_int
            && (curval.type_0 as ::core::ffi::c_int != kOptValTypeString as ::core::ffi::c_int
                && *op as ::core::ffi::c_int == '.' as ::core::ffi::c_int
                || curval.type_0 as ::core::ffi::c_int == kOptValTypeString as ::core::ffi::c_int
                    && *op as ::core::ffi::c_int != '.' as ::core::ffi::c_int)
        {
            semsg(
                gettext(&raw const e_letwrong as *const ::core::ffi::c_char),
                op,
            );
        } else {
            error = false;
            newval = tv_to_optval(tv, opt_idx, arg, &raw mut error);
            if !error {
                '_c2rust_label: {
                    if curval.type_0 as ::core::ffi::c_int == newval.type_0 as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                        b"curval.type == newval.type\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/eval/vars.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        1383 as ::core::ffi::c_uint,
                        b"char *ex_let_option(char *, typval_T *const, const _Bool, const char *const, const char *const)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                    }
                };
                is_num = curval.type_0 as ::core::ffi::c_int
                    == kOptValTypeNumber as ::core::ffi::c_int
                    || curval.type_0 as ::core::ffi::c_int
                        == kOptValTypeBoolean as ::core::ffi::c_int;
                is_string =
                    curval.type_0 as ::core::ffi::c_int == kOptValTypeString as ::core::ffi::c_int;
                if !op.is_null() && *op as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
                    if !hidden && is_num as ::core::ffi::c_int != 0 {
                        let mut cur_n: OptInt = if curval.type_0 as ::core::ffi::c_int
                            == kOptValTypeNumber as ::core::ffi::c_int
                        {
                            curval.data.number
                        } else {
                            curval.data.boolean as OptInt
                        };
                        let mut new_n: OptInt = if newval.type_0 as ::core::ffi::c_int
                            == kOptValTypeNumber as ::core::ffi::c_int
                        {
                            newval.data.number
                        } else {
                            newval.data.boolean as OptInt
                        };
                        match *op as ::core::ffi::c_int {
                            43 => {
                                new_n = cur_n + new_n;
                            }
                            45 => {
                                new_n = cur_n - new_n;
                            }
                            42 => {
                                new_n = cur_n * new_n;
                            }
                            47 => {
                                new_n = num_divide(cur_n as varnumber_T, new_n as varnumber_T)
                                    as OptInt;
                            }
                            37 => {
                                new_n = num_modulus(cur_n as varnumber_T, new_n as varnumber_T)
                                    as OptInt;
                            }
                            _ => {}
                        }
                        if curval.type_0 as ::core::ffi::c_int
                            == kOptValTypeNumber as ::core::ffi::c_int
                        {
                            newval = OptVal {
                                type_0: kOptValTypeNumber,
                                data: OptValData { number: new_n },
                            };
                        } else {
                            newval = OptVal {
                                type_0: kOptValTypeBoolean,
                                data: OptValData {
                                    boolean: (if new_n == 0 as OptInt {
                                        kFalse as ::core::ffi::c_int
                                    } else if new_n >= 1 as OptInt {
                                        kTrue as ::core::ffi::c_int
                                    } else {
                                        kNone as ::core::ffi::c_int
                                    }) as TriState,
                                },
                            };
                        }
                    } else if !hidden && is_string as ::core::ffi::c_int != 0 {
                        let mut curval_data: *const ::core::ffi::c_char = curval.data.string.data;
                        let mut newval_data: *const ::core::ffi::c_char = newval.data.string.data;
                        if !curval_data.is_null() && !newval_data.is_null() {
                            let mut newval_old: OptVal = newval;
                            newval = OptVal {
                                type_0: kOptValTypeString,
                                data: OptValData {
                                    string: cstr_as_string(concat_str(curval_data, newval_data)),
                                },
                            };
                            optval_free(newval_old);
                        }
                    }
                }
                err = set_option_value_handle_tty(arg, opt_idx, newval, opt_flags);
                arg_end = p;
                if !err.is_null() {
                    emsg(gettext(err));
                }
            }
        }
        *p = c1;
        optval_free(curval);
        optval_free(newval);
        return arg_end;
    }
}

unsafe extern "C" fn ex_let_register(
    mut arg: *mut ::core::ffi::c_char,
    tv: *mut typval_T,
    is_const: bool,
    endchars: *const ::core::ffi::c_char,
    op: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if is_const {
            emsg(gettext(
                b"E996: Cannot lock a register\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut arg_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        arg = arg.offset(1);
        if !op.is_null()
            && !vim_strchr(
                b"+-*/%\0".as_ptr() as *const ::core::ffi::c_char,
                *op as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
        {
            semsg(
                gettext(&raw const e_letwrong as *const ::core::ffi::c_char),
                op,
            );
        } else if !endchars.is_null()
            && vim_strchr(
                endchars,
                *skipwhite(arg.offset(1 as ::core::ffi::c_int as isize)) as uint8_t
                    as ::core::ffi::c_int,
            )
            .is_null()
        {
            emsg(gettext(e_letunexp.get()));
        } else {
            let mut ptofree: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut p: *const ::core::ffi::c_char = tv_get_string_chk(tv);
            if !p.is_null()
                && !op.is_null()
                && *op as ::core::ffi::c_int == '.' as ::core::ffi::c_int
            {
                let mut s: *mut ::core::ffi::c_char = get_reg_contents(
                    if *arg as ::core::ffi::c_int == '@' as ::core::ffi::c_int {
                        '"' as ::core::ffi::c_int
                    } else {
                        *arg as ::core::ffi::c_int
                    },
                    kGRegExprSrc as ::core::ffi::c_int,
                ) as *mut ::core::ffi::c_char;
                if !s.is_null() {
                    ptofree = concat_str(s, p);
                    p = ptofree;
                    xfree(s as *mut ::core::ffi::c_void);
                }
            }
            if !p.is_null() {
                write_reg_contents(
                    if *arg as ::core::ffi::c_int == '@' as ::core::ffi::c_int {
                        '"' as ::core::ffi::c_int
                    } else {
                        *arg as ::core::ffi::c_int
                    },
                    p,
                    strlen(p) as ssize_t,
                    false_0,
                );
                arg_end = arg.offset(1 as ::core::ffi::c_int as isize);
            }
            xfree(ptofree as *mut ::core::ffi::c_void);
        }
        return arg_end;
    }
}

unsafe extern "C" fn ex_let_one(
    mut arg: *mut ::core::ffi::c_char,
    tv: *mut typval_T,
    copy: bool,
    is_const: bool,
    endchars: *const ::core::ffi::c_char,
    op: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut arg_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if *arg as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
            return ex_let_env(arg, tv, is_const, endchars, op);
        } else if *arg as ::core::ffi::c_int == '&' as ::core::ffi::c_int {
            return ex_let_option(arg, tv, is_const, endchars, op);
        } else if *arg as ::core::ffi::c_int == '@' as ::core::ffi::c_int {
            return ex_let_register(arg, tv, is_const, endchars, op);
        } else if eval_isnamec1(*arg as ::core::ffi::c_int) as ::core::ffi::c_int != 0
            || *arg as ::core::ffi::c_int == '{' as ::core::ffi::c_int
        {
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
            let p: *mut ::core::ffi::c_char = get_lval(
                arg,
                tv,
                &raw mut lv,
                false_0 != 0,
                false_0 != 0,
                0 as ::core::ffi::c_int,
                FNE_CHECK_START,
            );
            if !p.is_null() && !lv.ll_name.is_null() {
                if !endchars.is_null()
                    && vim_strchr(endchars, *skipwhite(p) as uint8_t as ::core::ffi::c_int)
                        .is_null()
                {
                    emsg(gettext(e_letunexp.get()));
                } else {
                    set_var_lval(&raw mut lv, p, tv, copy, is_const, op);
                    arg_end = p;
                }
            }
            clear_lval(&raw mut lv);
        } else {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                arg,
            );
        }
        return arg_end;
    }
}
