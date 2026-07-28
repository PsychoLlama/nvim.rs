//! Time: the `timer_*()` family, `wait()` and the `reltime()` clock.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

unsafe extern "C" fn dummy_timer_due_cb(
    mut tw: *mut TimeWatcher,
    mut _data: *mut ::core::ffi::c_void,
) {
    if (*main_loop.ptr()).closing {
        time_watcher_stop(tw);
        time_watcher_close(
            tw,
            Some(
                dummy_timer_close_cb
                    as unsafe extern "C" fn(*mut TimeWatcher, *mut ::core::ffi::c_void) -> (),
            ),
        );
    }
}
unsafe extern "C" fn dummy_timer_close_cb(
    mut tw: *mut TimeWatcher,
    mut _data: *mut ::core::ffi::c_void,
) {
    xfree(tw as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn f_wait(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = -1 as varnumber_T;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        semsg(
            gettext(&raw const e_invargval as *const ::core::ffi::c_char),
            b"1\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(2 as ::core::ffi::c_int as isize))
                .vval
                .v_number
                <= 0 as varnumber_T
    {
        semsg(
            gettext(&raw const e_invargval as *const ::core::ffi::c_char),
            b"3\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut timeout: ::core::ffi::c_int = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_number as ::core::ffi::c_int;
    let mut expr: typval_T = *argvars.offset(1 as ::core::ffi::c_int as isize);
    let mut interval: ::core::ffi::c_int = if (*argvars.offset(2 as ::core::ffi::c_int as isize))
        .v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*argvars.offset(2 as ::core::ffi::c_int as isize))
            .vval
            .v_number as ::core::ffi::c_int
    } else {
        200 as ::core::ffi::c_int
    };
    let mut tw: *mut TimeWatcher =
        xmalloc(::core::mem::size_of::<TimeWatcher>()) as *mut TimeWatcher;
    time_watcher_init(main_loop.ptr(), tw, NULL_0);
    (*tw).events = ::core::ptr::null_mut::<MultiQueue>();
    time_watcher_start(
        tw,
        Some(
            dummy_timer_due_cb
                as unsafe extern "C" fn(*mut TimeWatcher, *mut ::core::ffi::c_void) -> (),
        ),
        interval as uint64_t,
        interval as uint64_t,
    );
    let mut argv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut exprval: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut error: bool = false_0 != 0;
    let called_emsg_before: ::core::ffi::c_int = called_emsg.get();
    ui_flush();
    process_events_until(
        main_loop.ptr(),
        (*main_loop.ptr()).events,
        timeout as int64_t,
        || {
            eval_expr_typval(
                &raw mut expr,
                false,
                &raw mut argv,
                0 as ::core::ffi::c_int,
                &raw mut exprval,
            ) != 1 as ::core::ffi::c_int
                || tv_get_number_chk(&raw mut exprval, &raw mut error) != 0
                || called_emsg.get() > called_emsg_before
                || error
                || got_int.get()
        },
    );
    if called_emsg.get() > called_emsg_before || error as ::core::ffi::c_int != 0 {
        (*rettv).vval.v_number = -3 as varnumber_T;
    } else if got_int.get() {
        got_int.set(false_0 != 0);
        vgetc();
        (*rettv).vval.v_number = -2 as varnumber_T;
    } else if tv_get_number_chk(&raw mut exprval, &raw mut error) != 0 {
        (*rettv).vval.v_number = 0 as varnumber_T;
    }
    time_watcher_stop(tw);
    time_watcher_close(
        tw,
        Some(
            dummy_timer_close_cb
                as unsafe extern "C" fn(*mut TimeWatcher, *mut ::core::ffi::c_void) -> (),
        ),
    );
}
pub unsafe extern "C" fn f_localtime(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = time(::core::ptr::null_mut::<time_t>()) as varnumber_T;
}
unsafe extern "C" fn list2proftime(
    mut arg: *mut typval_T,
    mut tm: *mut proftime_T,
) -> ::core::ffi::c_int {
    if (*arg).v_type as ::core::ffi::c_uint != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        || tv_list_len((*arg).vval.v_list) != 2 as ::core::ffi::c_int
    {
        return FAIL;
    }
    let mut error: bool = false_0 != 0;
    let mut n1: varnumber_T =
        tv_list_find_nr((*arg).vval.v_list, 0 as ::core::ffi::c_int, &raw mut error);
    let mut n2: varnumber_T =
        tv_list_find_nr((*arg).vval.v_list, 1 as ::core::ffi::c_int, &raw mut error);
    if error {
        return FAIL;
    }
    let mut u: C2Rust_Unnamed_47 = C2Rust_Unnamed_47 {
        split: C2Rust_Unnamed_48 {
            low: n2 as int32_t,
            high: n1 as int32_t,
        },
    };
    *tm = u.prof;
    return OK;
}
pub unsafe extern "C" fn f_reltime(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut res: proftime_T = 0;
    let mut start: proftime_T = 0;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        res = profile_start();
    } else if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if list2proftime(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut res,
        ) == FAIL
        {
            return;
        }
        res = profile_end(res);
    } else {
        if list2proftime(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut start,
        ) == FAIL
            || list2proftime(
                argvars.offset(1 as ::core::ffi::c_int as isize),
                &raw mut res,
            ) == FAIL
        {
            return;
        }
        res = profile_sub(res, start);
    }
    let mut u: C2Rust_Unnamed_51 = C2Rust_Unnamed_51 { prof: res };
    tv_list_alloc_ret(rettv, 2 as ptrdiff_t);
    tv_list_append_number((*rettv).vval.v_list, u.split.high as varnumber_T);
    tv_list_append_number((*rettv).vval.v_list, u.split.low as varnumber_T);
}
pub unsafe extern "C" fn f_reltimestr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut tm: proftime_T = 0;
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if list2proftime(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut tm,
    ) == OK
    {
        (*rettv).vval.v_string = xstrdup(profile_msg(tm));
    }
}
pub unsafe extern "C" fn f_reltimefloat(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut tm: proftime_T = 0;
    (*rettv).v_type = VAR_FLOAT;
    (*rettv).vval.v_float = 0 as ::core::ffi::c_int as float_T;
    if list2proftime(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut tm,
    ) == OK
    {
        (*rettv).vval.v_float =
            (profile_signed(tm) as ::core::ffi::c_double / 1000000000.0f64) as float_T;
    }
}
pub unsafe extern "C" fn f_timer_info(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
    if tv_check_for_opt_number_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut timer: *mut timer_T = find_timer_by_nr(tv_get_number(
            argvars.offset(0 as ::core::ffi::c_int as isize),
        ));
        if !timer.is_null() && (!(*timer).stopped || (*timer).refcount > 1 as ::core::ffi::c_int) {
            add_timer_info(rettv, timer);
        }
    } else {
        add_timer_info_all(rettv);
    };
}
pub unsafe extern "C" fn f_timer_pause(
    mut argvars: *mut typval_T,
    mut _unused: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(
            &raw const e_number_exp as *const ::core::ffi::c_char,
        ));
        return;
    }
    let mut paused: ::core::ffi::c_int =
        (tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) != 0)
            as ::core::ffi::c_int;
    let mut timer: *mut timer_T = find_timer_by_nr(tv_get_number(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    ));
    if !timer.is_null() {
        if !(*timer).paused && paused != 0 {
            time_watcher_stop(&raw mut (*timer).tw);
        } else if (*timer).paused as ::core::ffi::c_int != 0 && paused == 0 {
            time_watcher_start(
                &raw mut (*timer).tw,
                Some(
                    timer_due_cb
                        as unsafe extern "C" fn(*mut TimeWatcher, *mut ::core::ffi::c_void) -> (),
                ),
                (*timer).timeout as uint64_t,
                (*timer).timeout as uint64_t,
            );
        }
        (*timer).paused = paused != 0;
    }
}
pub unsafe extern "C" fn f_timer_start(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut repeat: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    (*rettv).vval.v_number = -1 as varnumber_T;
    if check_secure() {
        return;
    }
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tv_check_for_nonnull_dict_arg(argvars, 2 as ::core::ffi::c_int) == FAIL {
            return;
        }
        let mut dict: *mut dict_T = (*argvars.offset(2 as ::core::ffi::c_int as isize))
            .vval
            .v_dict;
        let di: *mut dictitem_T = tv_dict_find(
            dict,
            b"repeat\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !di.is_null() {
            repeat = tv_get_number(&raw mut (*di).di_tv) as ::core::ffi::c_int;
            if repeat == 0 as ::core::ffi::c_int {
                repeat = 1 as ::core::ffi::c_int;
            }
        }
    }
    let mut callback: Callback = Callback {
        data: C2Rust_Unnamed_22 {
            funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        type_0: kCallbackNone,
    };
    if !callback_from_typval(
        &raw mut callback,
        argvars.offset(1 as ::core::ffi::c_int as isize),
    ) {
        return;
    }
    (*rettv).vval.v_number = timer_start(
        tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)),
        repeat,
        &raw mut callback,
    ) as varnumber_T;
}
pub unsafe extern "C" fn f_timer_stop(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if tv_check_for_number_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
        return;
    }
    let mut timer: *mut timer_T = find_timer_by_nr(tv_get_number(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    ));
    if timer.is_null() {
        return;
    }
    timer_stop(timer);
}
pub unsafe extern "C" fn f_timer_stopall(
    mut _argvars: *mut typval_T,
    mut _unused: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    timer_stop_all();
}
