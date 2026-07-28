//! `system()` and `systemlist()`: the argument vector and the captured output.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn tv_to_argv(
    mut cmd_tv: *mut typval_T,
    mut cmd: *mut *const c_char,
    mut executable: *mut bool,
) -> *mut *mut c_char {
    if (*cmd_tv).v_type as c_uint == VAR_STRING as c_int as c_uint {
        let mut cmd_str: *const c_char = tv_get_string(cmd_tv);
        if !cmd.is_null() {
            *cmd = cmd_str;
        }
        return shell_build_argv(cmd_str, ::core::ptr::null::<c_char>());
    }
    if (*cmd_tv).v_type as c_uint != VAR_LIST as c_int as c_uint {
        semsg(
            gettext(&raw const e_invarg2 as *const c_char),
            b"expected String or List\0".as_ptr() as *const c_char,
        );
        return ::core::ptr::null_mut::<*mut c_char>();
    }
    let mut argl: *mut list_T = (*cmd_tv).vval.v_list;
    let mut argc: c_int = tv_list_len(argl);
    if argc == 0 {
        emsg(gettext(&raw const e_invarg as *const c_char));
        return ::core::ptr::null_mut::<*mut c_char>();
    }
    let mut arg0: *const c_char = tv_get_string_chk(&raw mut (*tv_list_first(argl)).li_tv);
    let mut exe_resolved: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if arg0.is_null() || !os_can_exe(arg0, &raw mut exe_resolved, true_0 != 0) {
        if !arg0.is_null() && !executable.is_null() {
            let mut buf: [c_char; 1025] = [0; 1025];
            snprintf(
                &raw mut buf as *mut c_char,
                ::core::mem::size_of::<[c_char; 1025]>(),
                b"'%s' is not executable\0".as_ptr() as *const c_char,
                arg0,
            );
            semsg(
                gettext(&raw const e_invargNval as *const c_char),
                b"cmd\0".as_ptr() as *const c_char,
                &raw mut buf as *mut c_char,
            );
            *executable = false_0 != 0;
        }
        return ::core::ptr::null_mut::<*mut c_char>();
    }
    if !cmd.is_null() {
        *cmd = exe_resolved;
    }
    let mut i: c_int = 0 as c_int;
    let mut argv: *mut *mut c_char = xcalloc(
        (argc as size_t).wrapping_add(1 as size_t),
        ::core::mem::size_of::<*mut c_char>(),
    ) as *mut *mut c_char;
    let l_: *const list_T = argl;
    if !l_.is_null() {
        let mut arg: *const listitem_T = (*l_).lv_first;
        while !arg.is_null() {
            let mut a: *const c_char = tv_get_string_chk(&raw const (*arg).li_tv);
            if a.is_null() {
                shell_free_argv(argv);
                xfree(exe_resolved as *mut c_void);
                return ::core::ptr::null_mut::<*mut c_char>();
            }
            let c2rust_fresh11 = i;
            i = i + 1;
            let c2rust_lvalue_ptr = &raw mut *argv.offset(c2rust_fresh11 as isize);
            *c2rust_lvalue_ptr = xstrdup(a);
            arg = (*arg).li_next;
        }
    }
    xfree(*argv.offset(0 as c_int as isize) as *mut c_void);
    *argv.offset(0 as c_int as isize) = exe_resolved;
    return argv;
}

pub(crate) unsafe extern "C" fn string_to_list(
    mut str: *const c_char,
    mut len: size_t,
    keepempty: bool,
) -> *mut list_T {
    if !keepempty && *str.offset(len.wrapping_sub(1 as size_t) as isize) as c_int == NL {
        len = len.wrapping_sub(1);
    }
    let list: *mut list_T = tv_list_alloc(kListLenMayKnow as c_int as ptrdiff_t);
    encode_list_write(list as *mut c_void, str, len);
    return list;
}

pub(crate) unsafe extern "C" fn get_system_output_as_rettv(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut retlist: bool,
) {
    let mut wait_time: proftime_T = 0;
    let mut profiling: bool = do_profiling.get() == PROF_YES;
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<c_char>();
    if check_secure() {
        return;
    }
    let mut input_len: ptrdiff_t = 0;
    let mut input: *mut c_char = save_tv_as_string(
        argvars.offset(1 as c_int as isize),
        &raw mut input_len,
        false_0 != 0,
        false_0 != 0,
    );
    if input_len < 0 as ptrdiff_t {
        '_c2rust_label: {
            if input.is_null() {
            } else {
                __assert_fail(
                    b"input == NULL\0".as_ptr() as *const c_char,
                    b"src/nvim/eval.rs\0".as_ptr() as *const c_char,
                    4731 as c_uint,
                    b"void get_system_output_as_rettv(typval_T *, typval_T *, _Bool)\0".as_ptr()
                        as *const c_char,
                );
            }
        };
        return;
    }
    let mut executable: bool = true_0 != 0;
    let mut argv: *mut *mut c_char = tv_to_argv(
        argvars.offset(0 as c_int as isize),
        ::core::ptr::null_mut::<*const c_char>(),
        &raw mut executable,
    );
    if argv.is_null() {
        if !executable {
            set_vim_var_nr(VV_SHELL_ERROR, -1 as varnumber_T);
        }
        xfree(input as *mut c_void);
        return;
    }
    if p_verbose.get() > 3 as OptInt {
        let mut cmdstr: *mut c_char = shell_argv_to_str(argv);
        verbose_enter_scroll();
        smsg(
            0 as c_int,
            gettext(b"Executing command: \"%s\"\0".as_ptr() as *const c_char),
            cmdstr,
        );
        msg_puts(b"\n\n\0".as_ptr() as *const c_char);
        verbose_leave_scroll();
        xfree(cmdstr as *mut c_void);
    }
    if profiling {
        wait_time = prof_child_enter();
    }
    let mut nread: size_t = 0 as size_t;
    let mut res: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut status: c_int = os_system(
        argv,
        input,
        input_len as size_t,
        &raw mut res,
        &raw mut nread,
    );
    if profiling {
        prof_child_exit(wait_time);
    }
    xfree(input as *mut c_void);
    set_vim_var_nr(VV_SHELL_ERROR, status as varnumber_T);
    if res.is_null() {
        if retlist {
            tv_list_alloc_ret(rettv, 0 as ptrdiff_t);
        } else {
            (*rettv).vval.v_string = xstrdup(b"\0".as_ptr() as *const c_char);
        }
        return;
    }
    if retlist {
        let mut keepempty: c_int = 0 as c_int;
        if (*argvars.offset(1 as c_int as isize)).v_type as c_uint != VAR_UNKNOWN as c_int as c_uint
            && (*argvars.offset(2 as c_int as isize)).v_type as c_uint
                != VAR_UNKNOWN as c_int as c_uint
        {
            keepempty = tv_get_number(argvars.offset(2 as c_int as isize)) as c_int;
        }
        (*rettv).vval.v_list = string_to_list(res, nread, keepempty != 0);
        tv_list_ref((*rettv).vval.v_list);
        (*rettv).v_type = VAR_LIST;
        xfree(res as *mut c_void);
    } else {
        memchrsub(res as *mut c_void, NUL as c_char, 1 as c_char, nread);
        (*rettv).vval.v_string = res;
    };
}

pub unsafe extern "C" fn f_system(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    get_system_output_as_rettv(argvars, rettv, false_0 != 0);
}

pub unsafe extern "C" fn f_systemlist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    get_system_output_as_rettv(argvars, rettv, true_0 != 0);
}

pub unsafe extern "C" fn save_tv_as_string(
    mut tv: *mut typval_T,
    len: *mut ptrdiff_t,
    mut endnl: bool,
    mut crlf: bool,
) -> *mut c_char {
    *len = 0 as ptrdiff_t;
    if (*tv).v_type as c_uint == VAR_UNKNOWN as c_int as c_uint {
        return ::core::ptr::null_mut::<c_char>();
    }
    if (*tv).v_type as c_uint != VAR_LIST as c_int as c_uint
        && (*tv).v_type as c_uint != VAR_NUMBER as c_int as c_uint
    {
        let mut ret: *const c_char = tv_get_string_chk(tv);
        if !ret.is_null() {
            *len = strlen(ret) as ptrdiff_t;
            return xmemdupz(ret as *const c_void, *len as size_t) as *mut c_char;
        } else {
            *len = -1 as ptrdiff_t;
            return ::core::ptr::null_mut::<c_char>();
        }
    }
    if (*tv).v_type as c_uint == VAR_NUMBER as c_int as c_uint {
        let mut buf: *mut buf_T = buflist_findnr((*tv).vval.v_number as c_int);
        if !buf.is_null() {
            let mut lnum: linenr_T = 1 as linenr_T;
            while lnum <= (*buf).b_ml.ml_line_count {
                let mut p: *mut c_char = ml_get_buf(buf, lnum);
                while *p as c_int != NUL {
                    *len += 1 as ptrdiff_t;
                    p = p.offset(1);
                }
                *len += 1 as ptrdiff_t;
                lnum += 1;
            }
        } else {
            semsg(
                gettext(&raw const e_nobufnr as *const c_char),
                (*tv).vval.v_number,
            );
            *len = -1 as ptrdiff_t;
            return ::core::ptr::null_mut::<c_char>();
        }
        if *len == 0 as ptrdiff_t {
            return ::core::ptr::null_mut::<c_char>();
        }
        let mut ret_0: *mut c_char =
            xmalloc((*len as size_t).wrapping_add(1 as size_t)) as *mut c_char;
        let mut end: *mut c_char = ret_0;
        let mut lnum_0: linenr_T = 1 as linenr_T;
        while lnum_0 <= (*buf).b_ml.ml_line_count {
            let mut p_0: *mut c_char = ml_get_buf(buf, lnum_0);
            while *p_0 as c_int != NUL {
                let c2rust_fresh12 = end;
                end = end.offset(1);
                *c2rust_fresh12 = (if *p_0 as c_int == '\n' as c_int {
                    NUL
                } else {
                    *p_0 as c_int
                }) as c_char;
                p_0 = p_0.offset(1);
            }
            let c2rust_fresh13 = end;
            end = end.offset(1);
            *c2rust_fresh13 = '\n' as c_char;
            lnum_0 += 1;
        }
        *end = NUL as c_char;
        *len = end.offset_from(ret_0) as ptrdiff_t;
        return ret_0;
    }
    '_c2rust_label: {
        if (*tv).v_type as c_uint == VAR_LIST as c_int as c_uint {
        } else {
            __assert_fail(
                b"tv->v_type == VAR_LIST\0".as_ptr() as *const c_char,
                b"src/nvim/eval.rs\0".as_ptr() as *const c_char,
                5197 as c_uint,
                b"char *save_tv_as_string(typval_T *, ptrdiff_t *const, _Bool, _Bool)\0".as_ptr()
                    as *const c_char,
            );
        }
    };
    let mut list: *mut list_T = (*tv).vval.v_list;
    let l_: *const list_T = list;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            *len += strlen(tv_get_string(&raw const (*li).li_tv)) as ptrdiff_t
                + (if crlf as c_int != 0 {
                    2 as c_int
                } else {
                    1 as c_int
                }) as ptrdiff_t;
            li = (*li).li_next;
        }
    }
    if *len == 0 as ptrdiff_t {
        return ::core::ptr::null_mut::<c_char>();
    }
    let mut ret_1: *mut c_char = xmalloc((*len as size_t).wrapping_add(
        (if endnl as c_int != 0 {
            if crlf as c_int != 0 {
                2 as c_int
            } else {
                1 as c_int
            }
        } else {
            0 as c_int
        }) as size_t,
    )) as *mut c_char;
    let mut end_0: *mut c_char = ret_1;
    let l__0: *const list_T = list;
    if !l__0.is_null() {
        let mut li_0: *const listitem_T = (*l__0).lv_first;
        while !li_0.is_null() {
            let mut s: *const c_char = tv_get_string(&raw const (*li_0).li_tv);
            while *s as c_int != '\0' as c_int {
                let c2rust_fresh14 = end_0;
                end_0 = end_0.offset(1);
                *c2rust_fresh14 = (if *s as c_int == '\n' as c_int {
                    '\0' as c_int
                } else {
                    *s as c_int
                }) as c_char;
                s = s.offset(1);
            }
            if endnl as c_int != 0 || !(*li_0).li_next.is_null() {
                if crlf {
                    let c2rust_fresh15 = end_0;
                    end_0 = end_0.offset(1);
                    *c2rust_fresh15 = '\r' as c_char;
                }
                let c2rust_fresh16 = end_0;
                end_0 = end_0.offset(1);
                *c2rust_fresh16 = '\n' as c_char;
            }
            li_0 = (*li_0).li_next;
        }
    }
    *end_0 = NUL as c_char;
    *len = end_0.offset_from(ret_1) as ptrdiff_t;
    return ret_1;
}
