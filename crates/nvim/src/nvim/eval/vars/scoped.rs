//! `b:`, `w:` and `t:` from somewhere else.
//!
//! `get_var_from` and `setwinvar` switch to the requested buffer, window or
//! tabpage, do the lookup there and switch back; the `f_*` entries below
//! them are the Vimscript builtins that call them.  The `&option` spelling
//! of a name lands in `tv_to_optval`/`optval_as_tv` instead.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn get_var_from(
    mut varname: *const ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    mut deftv: *mut typval_T,
    mut htname: ::core::ffi::c_int,
    mut tp: *mut tabpage_T,
    mut win: *mut win_T,
    mut buf: *mut buf_T,
) {
    unsafe {
        let mut done: bool = false_0 != 0;
        let do_change_curbuf: bool = !buf.is_null() && htname == 'b' as ::core::ffi::c_int;
        (*emsg_off.ptr()) += 1;
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if !varname.is_null()
            && !tp.is_null()
            && !win.is_null()
            && (htname != 'b' as ::core::ffi::c_int || !buf.is_null())
        {
            let need_switch_win: bool =
                !(tp == curtab.get() && win == curwin.get()) && !do_change_curbuf;
            let mut switchwin: switchwin_T = switchwin_T {
                sw_curwin: ::core::ptr::null_mut::<win_T>(),
                sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
                sw_same_win: false,
                sw_visual_active: false,
            };
            if !need_switch_win || switch_win(&raw mut switchwin, win, tp, true_0 != 0) == OK {
                if *varname as ::core::ffi::c_int == '&' as ::core::ffi::c_int
                    && htname != 't' as ::core::ffi::c_int
                {
                    let save_curbuf: *mut buf_T = curbuf.get();
                    if do_change_curbuf {
                        curbuf.set(buf);
                    }
                    if *varname.offset(1 as ::core::ffi::c_int as isize) == NUL {
                        let mut opts: *mut dict_T = get_winbuf_options(
                            (htname == 'b' as ::core::ffi::c_int) as ::core::ffi::c_int,
                        );
                        if !opts.is_null() {
                            tv_dict_set_ret(rettv, opts);
                            done = true_0 != 0;
                        }
                    } else if eval_option(&raw mut varname, rettv, true_0 != 0) == OK {
                        done = true_0 != 0;
                    }
                    curbuf.set(save_curbuf);
                } else if *varname == NUL {
                    let mut v: *const ScopeDictDictItem = ::core::ptr::null::<ScopeDictDictItem>();
                    if htname == 'b' as ::core::ffi::c_int {
                        v = &raw mut (*buf).b_bufvar;
                    } else if htname == 'w' as ::core::ffi::c_int {
                        v = &raw mut (*win).w_winvar;
                    } else {
                        v = &raw mut (*tp).tp_winvar;
                    }
                    tv_copy(&raw const (*v).di_tv, rettv);
                    done = true_0 != 0;
                } else {
                    let mut ht: *mut hashtab_T = ::core::ptr::null_mut::<hashtab_T>();
                    if htname == 'b' as ::core::ffi::c_int {
                        ht = &raw mut (*(*buf).b_vars).dv_hashtab;
                    } else if htname == 'w' as ::core::ffi::c_int {
                        ht = &raw mut (*(*win).w_vars).dv_hashtab;
                    } else {
                        ht = &raw mut (*(*tp).tp_vars).dv_hashtab;
                    }
                    let v_0: *const dictitem_T =
                        find_var_in_ht(ht, htname, varname, strlen(varname), false_0);
                    if !v_0.is_null() {
                        tv_copy(&raw const (*v_0).di_tv, rettv);
                        done = true_0 != 0;
                    }
                }
            }
            if need_switch_win {
                restore_win(&raw mut switchwin, true_0 != 0);
            }
        }
        if !done
            && (*deftv).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_copy(deftv, rettv);
        }
        (*emsg_off.ptr()) -= 1;
    }
}

unsafe extern "C" fn getwinvar(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut off: ::core::ffi::c_int,
) {
    unsafe {
        let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
        if off == 1 as ::core::ffi::c_int {
            tp = find_tabpage(tv_get_number_chk(
                argvars.offset(0 as ::core::ffi::c_int as isize),
                ::core::ptr::null_mut::<bool>(),
            ) as ::core::ffi::c_int);
        } else {
            tp = curtab.get();
        }
        let win: *mut win_T = find_win_by_nr(argvars.offset(off as isize), tp);
        let varname: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset((off + 1 as ::core::ffi::c_int) as isize));
        get_var_from(
            varname,
            rettv,
            argvars.offset((off + 2 as ::core::ffi::c_int) as isize),
            'w' as ::core::ffi::c_int,
            tp,
            win,
            ::core::ptr::null_mut::<buf_T>(),
        );
    }
}

pub(crate) unsafe extern "C" fn tv_to_optval(
    mut tv: *mut typval_T,
    mut opt_idx: OptIndex,
    mut option: *const ::core::ffi::c_char,
    mut error: *mut bool,
) -> OptVal {
    unsafe {
        let mut value: OptVal = OptVal {
            type_0: kOptValTypeNil,
            data: OptValData { boolean: kFalse },
        };
        let mut nbuf: [::core::ffi::c_char; 65] = [0; 65];
        let mut err: bool = false_0 != 0;
        let is_tty_opt: bool = is_tty_option(option);
        let option_has_bool: bool =
            !is_tty_opt && option_has_type(opt_idx, kOptValTypeBoolean) as ::core::ffi::c_int != 0;
        let option_has_num: bool =
            !is_tty_opt && option_has_type(opt_idx, kOptValTypeNumber) as ::core::ffi::c_int != 0;
        let option_has_str: bool = is_tty_opt as ::core::ffi::c_int != 0
            || option_has_type(opt_idx, kOptValTypeString) as ::core::ffi::c_int != 0;
        if !is_tty_opt
            && (*get_option(opt_idx)).flags & kOptFlagFunc as ::core::ffi::c_int as uint32_t != 0
            && tv_is_func(*tv) as ::core::ffi::c_int != 0
        {
            let mut strval: *mut ::core::ffi::c_char =
                encode_tv2string(tv, ::core::ptr::null_mut::<size_t>());
            err = strval.is_null();
            value = OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(strval),
                },
            };
        } else if option_has_bool as ::core::ffi::c_int != 0
            || option_has_num as ::core::ffi::c_int != 0
        {
            let mut n: varnumber_T = if option_has_num as ::core::ffi::c_int != 0 {
                tv_get_number_chk(tv, &raw mut err)
            } else {
                tv_get_bool_chk(tv, &raw mut err)
            };
            if !err
                && (*tv).v_type as ::core::ffi::c_uint
                    == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                && n == 0 as varnumber_T
            {
                let mut idx: ::core::ffi::c_uint = 0;
                idx = 0 as ::core::ffi::c_uint;
                while !(*tv).vval.v_string.is_null()
                    && *(*tv).vval.v_string.offset(idx as isize) as ::core::ffi::c_int
                        == '0' as ::core::ffi::c_int
                {
                    idx = idx.wrapping_add(1);
                }
                if idx == 0 as ::core::ffi::c_uint
                    || *(*tv).vval.v_string.offset(idx as isize) != NUL
                {
                    err = true_0 != 0;
                    semsg(
                        gettext(b"E521: Number required: &%s = '%s'\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        option,
                        if (*tv).vval.v_string.is_null() {
                            b"\0".as_ptr() as *const ::core::ffi::c_char
                        } else {
                            (*tv).vval.v_string as *const ::core::ffi::c_char
                        },
                    );
                }
            }
            value = if option_has_num as ::core::ffi::c_int != 0 {
                OptVal {
                    type_0: kOptValTypeNumber,
                    data: OptValData { number: n },
                }
            } else {
                OptVal {
                    type_0: kOptValTypeBoolean,
                    data: OptValData {
                        boolean: (if n == 0 as varnumber_T {
                            kFalse as ::core::ffi::c_int
                        } else if n >= 1 as varnumber_T {
                            kTrue as ::core::ffi::c_int
                        } else {
                            kNone as ::core::ffi::c_int
                        }) as TriState,
                    },
                }
            };
        } else if option_has_str {
            if (*tv).v_type as ::core::ffi::c_uint
                != VAR_BOOL as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*tv).v_type as ::core::ffi::c_uint
                    != VAR_SPECIAL as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut strval_0: *const ::core::ffi::c_char =
                    tv_get_string_buf_chk(tv, &raw mut nbuf as *mut ::core::ffi::c_char);
                err = strval_0.is_null();
                value = OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: cstr_to_string(strval_0),
                    },
                };
            } else if !is_tty_opt {
                err = true_0 != 0;
                emsg(gettext(
                    &raw const e_string_required as *const ::core::ffi::c_char,
                ));
            }
        } else {
            abort();
        }
        if !error.is_null() {
            *error = err;
        }
        return value;
    }
}

pub unsafe extern "C" fn optval_as_tv(mut value: OptVal, mut numbool: bool) -> typval_T {
    unsafe {
        let mut rettv: typval_T = typval_T {
            v_type: VAR_SPECIAL,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_special: kSpecialVarNull,
            },
        };
        match value.type_0 as ::core::ffi::c_int {
            0 => {
                if numbool {
                    rettv.v_type = VAR_NUMBER;
                    rettv.vval.v_number = value.data.boolean as varnumber_T;
                } else if value.data.boolean as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
                    rettv.v_type = VAR_BOOL;
                    rettv.vval.v_bool =
                        (value.data.boolean as ::core::ffi::c_int == kTrue as ::core::ffi::c_int)
                            as ::core::ffi::c_int as BoolVarValue;
                }
            }
            1 => {
                rettv.v_type = VAR_NUMBER;
                rettv.vval.v_number = value.data.number as varnumber_T;
            }
            2 => {
                rettv.v_type = VAR_STRING;
                rettv.vval.v_string = value.data.string.data;
            }
            -1 | _ => {}
        }
        return rettv;
    }
}

unsafe extern "C" fn set_option_from_tv(
    mut varname: *const ::core::ffi::c_char,
    mut varp: *mut typval_T,
) {
    unsafe {
        let mut opt_idx: OptIndex = find_option(varname);
        if opt_idx as ::core::ffi::c_int == kOptInvalid as ::core::ffi::c_int {
            semsg(
                gettext(&raw const e_unknown_option2 as *const ::core::ffi::c_char),
                varname,
            );
            return;
        }
        let mut error: bool = false_0 != 0;
        let mut value: OptVal = tv_to_optval(varp, opt_idx, varname, &raw mut error);
        if !error {
            let mut errmsg: *const ::core::ffi::c_char = set_option_value_handle_tty(
                varname,
                opt_idx,
                value,
                OPT_LOCAL as ::core::ffi::c_int,
            );
            if !errmsg.is_null() {
                emsg(errmsg);
            }
        }
        optval_free(value);
    }
}

unsafe extern "C" fn setwinvar(mut argvars: *mut typval_T, mut off: ::core::ffi::c_int) {
    unsafe {
        if check_secure() {
            return;
        }
        let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
        if off == 1 as ::core::ffi::c_int {
            tp = find_tabpage(tv_get_number_chk(
                argvars.offset(0 as ::core::ffi::c_int as isize),
                ::core::ptr::null_mut::<bool>(),
            ) as ::core::ffi::c_int);
        } else {
            tp = curtab.get();
        }
        let win: *mut win_T = find_win_by_nr(argvars.offset(off as isize), tp);
        let mut varname: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset((off + 1 as ::core::ffi::c_int) as isize));
        let mut varp: *mut typval_T = argvars.offset((off + 2 as ::core::ffi::c_int) as isize);
        if win.is_null() || varname.is_null() {
            return;
        }
        let mut need_switch_win: bool = !(tp == curtab.get() && win == curwin.get());
        let mut switchwin: switchwin_T = switchwin_T {
            sw_curwin: ::core::ptr::null_mut::<win_T>(),
            sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
            sw_same_win: false,
            sw_visual_active: false,
        };
        if !need_switch_win || switch_win(&raw mut switchwin, win, tp, true_0 != 0) == OK {
            if *varname as ::core::ffi::c_int == '&' as ::core::ffi::c_int {
                set_option_from_tv(varname.offset(1 as ::core::ffi::c_int as isize), varp);
            } else {
                let varname_len: size_t = strlen(varname);
                let winvarname: *mut ::core::ffi::c_char =
                    xmalloc(varname_len.wrapping_add(3 as size_t)) as *mut ::core::ffi::c_char;
                memcpy(
                    winvarname as *mut ::core::ffi::c_void,
                    b"w:\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                    2 as size_t,
                );
                memcpy(
                    winvarname.offset(2 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                    varname as *const ::core::ffi::c_void,
                    varname_len.wrapping_add(1 as size_t),
                );
                set_var(
                    winvarname,
                    varname_len.wrapping_add(2 as size_t),
                    varp,
                    true_0 != 0,
                );
                xfree(winvarname as *mut ::core::ffi::c_void);
            }
        }
        if need_switch_win {
            restore_win(&raw mut switchwin, true_0 != 0);
        }
    }
}

pub unsafe extern "C" fn f_gettabvar(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let varname: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize));
        let tp: *mut tabpage_T = find_tabpage(tv_get_number_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            ::core::ptr::null_mut::<bool>(),
        ) as ::core::ffi::c_int);
        let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
        if !tp.is_null() {
            win = if tp == curtab.get() || (*tp).tp_firstwin.is_null() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
        }
        get_var_from(
            varname,
            rettv,
            argvars.offset(2 as ::core::ffi::c_int as isize),
            't' as ::core::ffi::c_int,
            tp,
            win,
            ::core::ptr::null_mut::<buf_T>(),
        );
    }
}

pub unsafe extern "C" fn f_gettabwinvar(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        getwinvar(argvars, rettv, 1 as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn f_getwinvar(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        getwinvar(argvars, rettv, 0 as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn f_getbufvar(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let varname: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize));
        let buf: *mut buf_T = tv_get_buf_from_arg(argvars.offset(0 as ::core::ffi::c_int as isize));
        get_var_from(
            varname,
            rettv,
            argvars.offset(2 as ::core::ffi::c_int as isize),
            'b' as ::core::ffi::c_int,
            curtab.get(),
            curwin.get(),
            buf,
        );
    }
}

pub unsafe extern "C" fn f_settabvar(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        if check_secure() {
            return;
        }
        let tp: *mut tabpage_T = find_tabpage(tv_get_number_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            ::core::ptr::null_mut::<bool>(),
        ) as ::core::ffi::c_int);
        let varname: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize));
        let varp: *mut typval_T = argvars.offset(2 as ::core::ffi::c_int as isize);
        if varname.is_null() || tp.is_null() {
            return;
        }
        let save_curtab: *mut tabpage_T = curtab.get();
        let save_lu_tp: *mut tabpage_T = lastused_tabpage.get();
        goto_tabpage_tp(tp, false_0 != 0, false_0 != 0);
        let varname_len: size_t = strlen(varname);
        let tabvarname: *mut ::core::ffi::c_char =
            xmalloc(varname_len.wrapping_add(3 as size_t)) as *mut ::core::ffi::c_char;
        memcpy(
            tabvarname as *mut ::core::ffi::c_void,
            b"t:\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            2 as size_t,
        );
        memcpy(
            tabvarname.offset(2 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            varname as *const ::core::ffi::c_void,
            varname_len.wrapping_add(1 as size_t),
        );
        set_var(
            tabvarname,
            varname_len.wrapping_add(2 as size_t),
            varp,
            true_0 != 0,
        );
        xfree(tabvarname as *mut ::core::ffi::c_void);
        if valid_tabpage(save_curtab) {
            goto_tabpage_tp(save_curtab, false_0 != 0, false_0 != 0);
            if valid_tabpage(save_lu_tp) {
                lastused_tabpage.set(save_lu_tp);
            }
        }
    }
}

pub unsafe extern "C" fn f_settabwinvar(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        setwinvar(argvars, 1 as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn f_setwinvar(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        setwinvar(argvars, 0 as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn f_setbufvar(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        if check_secure() as ::core::ffi::c_int != 0
            || !tv_check_str_or_nr(argvars.offset(0 as ::core::ffi::c_int as isize))
        {
            return;
        }
        let mut varname: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize));
        let buf: *mut buf_T = tv_get_buf(argvars.offset(0 as ::core::ffi::c_int as isize), false_0);
        let mut varp: *mut typval_T = argvars.offset(2 as ::core::ffi::c_int as isize);
        if buf.is_null() || varname.is_null() {
            return;
        }
        if *varname as ::core::ffi::c_int == '&' as ::core::ffi::c_int {
            let mut aco: aco_save_T = aco_save_T::default();
            aucmd_prepbuf(&raw mut aco, buf);
            set_option_from_tv(varname.offset(1 as ::core::ffi::c_int as isize), varp);
            aucmd_restbuf(&raw mut aco);
        } else {
            let varname_len: size_t = strlen(varname);
            let bufvarname: *mut ::core::ffi::c_char =
                xmalloc(varname_len.wrapping_add(3 as size_t)) as *mut ::core::ffi::c_char;
            let save_curbuf: *mut buf_T = curbuf.get();
            curbuf.set(buf);
            memcpy(
                bufvarname as *mut ::core::ffi::c_void,
                b"b:\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                2 as size_t,
            );
            memcpy(
                bufvarname.offset(2 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                varname as *const ::core::ffi::c_void,
                varname_len.wrapping_add(1 as size_t),
            );
            set_var(
                bufvarname,
                varname_len.wrapping_add(2 as size_t),
                varp,
                true_0 != 0,
            );
            xfree(bufvarname as *mut ::core::ffi::c_void);
            curbuf.set(save_curbuf);
        };
    }
}
