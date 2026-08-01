//! The Vimscript function bridges, and the garbage collector.
//!
//! [`f_getqflist`]/[`f_setqflist`] and their location-list twins unpack
//! their arguments and call into `getprops`/`setprops`.
//! [`set_ref_in_quickfix`] is the other half: every list's context and
//! every entry's user data is a `typval_T` the collector has to see.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn mark_quickfix_user_data(
    mut qi: *mut qf_info_T,
    mut copyID: ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut abort_0: bool = false_0 != 0;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*qi).qf_maxcount && !abort_0 {
            let mut qfl: *mut qf_list_T = (*qi).qf_lists.offset(i as isize);
            if (*qfl).qf_has_user_data {
                let mut qfp: *mut qfline_T = ::core::ptr::null_mut::<qfline_T>();
                let mut j: ::core::ffi::c_int = 0;
                j = 1 as ::core::ffi::c_int;
                qfp = (*qfl).qf_start;
                while !got_int.get() && j <= (*qfl).qf_count && !qfp.is_null() {
                    let mut user_data: *mut typval_T = &raw mut (*qfp).qf_user_data;
                    if !user_data.is_null()
                        && (*user_data).v_type as ::core::ffi::c_uint
                            != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                        && (*user_data).v_type as ::core::ffi::c_uint
                            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                        && (*user_data).v_type as ::core::ffi::c_uint
                            != VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        abort_0 = abort_0 as ::core::ffi::c_int != 0
                            || set_ref_in_item(
                                user_data,
                                copyID,
                                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                                ::core::ptr::null_mut::<*mut list_stack_T>(),
                            ) as ::core::ffi::c_int
                                != 0;
                    }
                    j += 1;
                    qfp = (*qfp).qf_next;
                }
            }
            i += 1;
        }
        return abort_0;
    }
}

pub(crate) unsafe extern "C" fn mark_quickfix_ctx(
    mut qi: *mut qf_info_T,
    mut copyID: ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut abort_0: bool = false_0 != 0;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*qi).qf_maxcount && !abort_0 {
            let mut ctx: *mut typval_T = (*(*qi).qf_lists.offset(i as isize)).qf_ctx;
            if !ctx.is_null()
                && (*ctx).v_type as ::core::ffi::c_uint
                    != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*ctx).v_type as ::core::ffi::c_uint
                    != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*ctx).v_type as ::core::ffi::c_uint
                    != VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                abort_0 = set_ref_in_item(
                    ctx,
                    copyID,
                    ::core::ptr::null_mut::<*mut ht_stack_T>(),
                    ::core::ptr::null_mut::<*mut list_stack_T>(),
                );
            }
            let mut cb: *mut Callback = &raw mut (*(*qi).qf_lists.offset(i as isize)).qf_qftf_cb;
            abort_0 = abort_0 as ::core::ffi::c_int != 0
                || set_ref_in_callback(
                    cb,
                    copyID,
                    ::core::ptr::null_mut::<*mut ht_stack_T>(),
                    ::core::ptr::null_mut::<*mut list_stack_T>(),
                ) as ::core::ffi::c_int
                    != 0;
            i += 1;
        }
        return abort_0;
    }
}

pub unsafe extern "C" fn set_ref_in_quickfix(mut copyID: ::core::ffi::c_int) -> bool {
    unsafe {
        '_c2rust_label: {
            if !(*ql_info.ptr()).is_null() {
            } else {
                __assert_fail(
                    b"ql_info != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/quickfix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    7196 as ::core::ffi::c_uint,
                    b"_Bool set_ref_in_quickfix(int)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if mark_quickfix_ctx(ql_info.get(), copyID) as ::core::ffi::c_int != 0
            || mark_quickfix_user_data(ql_info.get(), copyID) as ::core::ffi::c_int != 0
            || set_ref_in_callback(
                qftf_cb.ptr(),
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as ::core::ffi::c_int
                != 0
        {
            return true_0 != 0;
        }
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut win: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !win.is_null() {
                if !(*win).w_llist.is_null() {
                    if mark_quickfix_ctx((*win).w_llist, copyID) as ::core::ffi::c_int != 0
                        || mark_quickfix_user_data((*win).w_llist, copyID) as ::core::ffi::c_int
                            != 0
                    {
                        return true_0 != 0;
                    }
                }
                if bt_quickfix((*win).w_buffer) as ::core::ffi::c_int != 0
                    && !(*win).w_llist_ref.is_null()
                    && (*(*win).w_llist_ref).qf_refcount == 1 as ::core::ffi::c_int
                {
                    if mark_quickfix_ctx((*win).w_llist_ref, copyID) as ::core::ffi::c_int != 0
                        || mark_quickfix_user_data((*win).w_llist_ref, copyID) as ::core::ffi::c_int
                            != 0
                    {
                        return true_0 != 0;
                    }
                }
                win = (*win).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        return false_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn get_qf_loc_list(
    mut is_qf: bool,
    mut wp: *mut win_T,
    mut what_arg: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    unsafe {
        if (*what_arg).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_list_alloc_ret(rettv, kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
            if is_qf as ::core::ffi::c_int != 0 || !wp.is_null() {
                get_errorlist(
                    ::core::ptr::null_mut::<qf_info_T>(),
                    wp,
                    -1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    (*rettv).vval.v_list,
                );
            }
        } else {
            tv_dict_alloc_ret(rettv);
            if is_qf as ::core::ffi::c_int != 0 || !wp.is_null() {
                if (*what_arg).v_type as ::core::ffi::c_uint
                    == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut d: *mut dict_T = (*what_arg).vval.v_dict;
                    if !d.is_null() {
                        qf_get_properties(wp, d, (*rettv).vval.v_dict);
                    }
                } else {
                    emsg(gettext(&raw const e_dictreq as *const ::core::ffi::c_char));
                }
            }
        };
    }
}

pub unsafe extern "C" fn f_getloclist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut wp: *mut win_T =
            find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
        get_qf_loc_list(
            false_0 != 0,
            wp,
            argvars.offset(1 as ::core::ffi::c_int as isize),
            rettv,
        );
    }
}

pub unsafe extern "C" fn f_getqflist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        get_qf_loc_list(
            true_0 != 0,
            ::core::ptr::null_mut::<win_T>(),
            argvars.offset(0 as ::core::ffi::c_int as isize),
            rettv,
        );
    }
}

pub(crate) unsafe extern "C" fn set_qf_ll_list(
    mut wp: *mut win_T,
    mut args: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    unsafe {
        let mut act: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut what_arg: *mut typval_T = ::core::ptr::null_mut::<typval_T>();
        static e_invact: GlobalCell<*const ::core::ffi::c_char> =
            GlobalCell::new(b"E927: Invalid action: '%s'\0".as_ptr() as *const ::core::ffi::c_char);
        let mut title: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut action: ::core::ffi::c_char = ' ' as ::core::ffi::c_char;
        static recursive: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        (*rettv).vval.v_number = -1 as varnumber_T;
        let mut what: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
        let mut list_arg: *mut typval_T = args.offset(0 as ::core::ffi::c_int as isize);
        if (*list_arg).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
            return;
        } else if recursive.get() != 0 as ::core::ffi::c_int {
            emsg(gettext(
                &raw const e_au_recursive as *const ::core::ffi::c_char,
            ));
            return;
        }
        let mut action_arg: *mut typval_T = args.offset(1 as ::core::ffi::c_int as isize);
        if (*action_arg).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*action_arg).v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                emsg(gettext(
                    &raw const e_string_required as *const ::core::ffi::c_char,
                ));
                return;
            }
            act = tv_get_string_chk(action_arg);
            if (*act as ::core::ffi::c_int == 'a' as ::core::ffi::c_int
                || *act as ::core::ffi::c_int == 'r' as ::core::ffi::c_int
                || *act as ::core::ffi::c_int == 'u' as ::core::ffi::c_int
                || *act as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                || *act as ::core::ffi::c_int == 'f' as ::core::ffi::c_int)
                && *act.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
            {
                action = *act;
            } else {
                semsg(gettext(e_invact.get()), act);
                return;
            }
            what_arg = args.offset(2 as ::core::ffi::c_int as isize);
            if (*what_arg).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if (*what_arg).v_type as ::core::ffi::c_uint
                    == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    title = tv_get_string_chk(what_arg);
                    if title.is_null() {
                        return;
                    }
                } else if (*what_arg).v_type as ::core::ffi::c_uint
                    == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
                    && !(*what_arg).vval.v_dict.is_null()
                {
                    what = (*what_arg).vval.v_dict;
                } else {
                    emsg(gettext(&raw const e_dictreq as *const ::core::ffi::c_char));
                    return;
                }
            }
        }
        if title.is_null() {
            title = if !wp.is_null() {
                b":setloclist()\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b":setqflist()\0".as_ptr() as *const ::core::ffi::c_char
            };
        }
        (*recursive.ptr()) += 1;
        let l: *mut list_T = (*list_arg).vval.v_list;
        if set_errorlist(
            wp,
            l,
            action as ::core::ffi::c_int,
            title as *mut ::core::ffi::c_char,
            what,
        ) == OK
        {
            (*rettv).vval.v_number = 0 as varnumber_T;
        }
        (*recursive.ptr()) -= 1;
    }
}

pub unsafe extern "C" fn f_setloclist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number = -1 as varnumber_T;
        let mut win: *mut win_T =
            find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
        if !win.is_null() {
            set_qf_ll_list(win, argvars.offset(1 as ::core::ffi::c_int as isize), rettv);
        }
    }
}

pub unsafe extern "C" fn f_setqflist(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        set_qf_ll_list(::core::ptr::null_mut::<win_T>(), argvars, rettv);
    }
}
