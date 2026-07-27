//! The `argc()`, `argidx()`, `arglistid()` and `argv()` builtins.

use super::*;

pub unsafe extern "C" fn f_argc(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0)).v_type as c_uint == VAR_UNKNOWN as c_int as c_uint {
        (*rettv).vval.v_number = argcount() as varnumber_T;
    } else if (*argvars.offset(0)).v_type as c_uint == VAR_NUMBER as c_int as c_uint
        && tv_get_number(argvars.offset(0)) == -1 as varnumber_T
    {
        (*rettv).vval.v_number = alist_count(global_alist.ptr()) as varnumber_T;
    } else {
        let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(0));
        if !wp.is_null() {
            (*rettv).vval.v_number = (*(*wp).w_alist).al_ga.ga_len as varnumber_T;
        } else {
            (*rettv).vval.v_number = -1 as varnumber_T;
        }
    };
}
pub unsafe extern "C" fn f_argidx(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = (*curwin.get()).w_arg_idx as varnumber_T;
}
pub unsafe extern "C" fn f_arglistid(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = -1 as varnumber_T;
    let mut wp: *mut win_T = find_tabwin(argvars.offset(0), argvars.offset(1));
    if !wp.is_null() {
        (*rettv).vval.v_number = (*(*wp).w_alist).id as varnumber_T;
    }
}
unsafe extern "C" fn get_arglist_as_rettv(
    mut arglist: *mut aentry_T,
    mut argcount: c_int,
    mut rettv: *mut typval_T,
) {
    tv_list_alloc_ret(rettv, argcount as ptrdiff_t);
    if !arglist.is_null() {
        let mut idx: c_int = 0;
        while idx < argcount {
            tv_list_append_string(
                (*rettv).vval.v_list,
                alist_name(arglist.offset(idx as isize)),
                -1 as ssize_t,
            );
            idx += 1;
        }
    }
}
pub unsafe extern "C" fn f_argv(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut arglist: *mut aentry_T = ptr::null_mut();
    let mut count: c_int = -1;
    if (*argvars.offset(0)).v_type as c_uint == VAR_UNKNOWN as c_int as c_uint {
        get_arglist_as_rettv(
            (*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T,
            argcount(),
            rettv,
        );
        return;
    }
    if (*argvars.offset(1)).v_type as c_uint == VAR_UNKNOWN as c_int as c_uint {
        arglist = (*(*curwin.get()).w_alist).al_ga.ga_data as *mut aentry_T;
        count = argcount();
    } else if (*argvars.offset(1)).v_type as c_uint == VAR_NUMBER as c_int as c_uint
        && tv_get_number(argvars.offset(1)) == -1 as varnumber_T
    {
        arglist = (*global_alist.ptr()).al_ga.ga_data as *mut aentry_T;
        count = alist_count(global_alist.ptr());
    } else {
        let mut wp: *mut win_T = find_win_by_nr_or_id(argvars.offset(1));
        if !wp.is_null() {
            arglist = (*(*wp).w_alist).al_ga.ga_data as *mut aentry_T;
            count = (*(*wp).w_alist).al_ga.ga_len;
        }
    }
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ptr::null_mut();
    let mut idx: c_int = tv_get_number_chk(argvars.offset(0), ptr::null_mut()) as c_int;
    if !arglist.is_null() && idx >= 0 && idx < count {
        (*rettv).vval.v_string = xstrdup(alist_name(arglist.offset(idx as isize)));
    } else if idx == -1 {
        get_arglist_as_rettv(arglist, count, rettv);
    }
}
