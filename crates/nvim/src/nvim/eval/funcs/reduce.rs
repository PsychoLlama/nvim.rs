//! Folding a sequence down to one value: `reduce()`, `max()`, `min()`.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

unsafe extern "C" fn max_min(tv: *const typval_T, rettv: *mut typval_T, domax: bool) {
    let mut error: bool = false_0 != 0;
    (*rettv).vval.v_number = 0 as varnumber_T;
    let mut n: varnumber_T = if domax as ::core::ffi::c_int != 0 {
        VARNUMBER_MIN as varnumber_T
    } else {
        VARNUMBER_MAX as varnumber_T
    };
    if (*tv).v_type as ::core::ffi::c_uint == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tv_list_len((*tv).vval.v_list) == 0 as ::core::ffi::c_int {
            return;
        }
        let l_: *const list_T = (*tv).vval.v_list;
        if !l_.is_null() {
            let mut li: *const listitem_T = (*l_).lv_first;
            while !li.is_null() {
                let i: varnumber_T = tv_get_number_chk(&raw const (*li).li_tv, &raw mut error);
                if error {
                    return;
                }
                if if domax as ::core::ffi::c_int != 0 {
                    (i > n) as ::core::ffi::c_int
                } else {
                    (i < n) as ::core::ffi::c_int
                } != 0
                {
                    n = i;
                }
                li = (*li).li_next;
            }
        }
    } else if (*tv).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tv_dict_len((*tv).vval.v_dict) == 0 as ::core::ffi::c_long {
            return;
        }
        let dihi_ht_: *mut hashtab_T = &raw mut (*(*tv).vval.v_dict).dv_hashtab;
        let mut dihi_todo_: size_t = (*dihi_ht_).ht_used;
        let mut dihi_: *mut hashitem_T = (*dihi_ht_).ht_array;
        while dihi_todo_ != 0 {
            if !((*dihi_).hi_key.is_null()
                || (*dihi_).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                dihi_todo_ = dihi_todo_.wrapping_sub(1);
                let di: *mut dictitem_T = (*dihi_)
                    .hi_key
                    .offset(-(17 as ::core::ffi::c_ulong as isize))
                    as *mut dictitem_T;
                let i_0: varnumber_T = tv_get_number_chk(&raw mut (*di).di_tv, &raw mut error);
                if error {
                    return;
                }
                if if domax as ::core::ffi::c_int != 0 {
                    (i_0 > n) as ::core::ffi::c_int
                } else {
                    (i_0 < n) as ::core::ffi::c_int
                } != 0
                {
                    n = i_0;
                }
            }
            dihi_ = dihi_.offset(1);
        }
    } else {
        semsg(
            gettext(&raw const e_listdictarg as *const ::core::ffi::c_char),
            if domax as ::core::ffi::c_int != 0 {
                b"max()\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"min()\0".as_ptr() as *const ::core::ffi::c_char
            },
        );
        return;
    }
    (*rettv).vval.v_number = n;
}
pub unsafe extern "C" fn f_max(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    max_min(argvars, rettv, true_0 != 0);
}
pub unsafe extern "C" fn f_min(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    max_min(argvars, rettv, false_0 != 0);
}
unsafe extern "C" fn reduce_list(
    mut argvars: *mut typval_T,
    mut expr: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    let l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_list;
    let called_emsg_start: ::core::ffi::c_int = called_emsg.get();
    let mut initial: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut li: *const listitem_T = ::core::ptr::null::<listitem_T>();
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tv_list_len(l) == 0 as ::core::ffi::c_int {
            semsg(
                gettext(
                    &raw const e_reduce_of_an_empty_str_with_no_initial_value
                        as *const ::core::ffi::c_char,
                ),
                b"List\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
        let first: *const listitem_T = tv_list_first(l);
        initial = (*first).li_tv;
        li = (*first).li_next;
    } else {
        initial = *argvars.offset(2 as ::core::ffi::c_int as isize);
        li = tv_list_first(l);
    }
    tv_copy(&raw mut initial, rettv);
    if l.is_null() {
        return;
    }
    let prev_locked: VarLockStatus = tv_list_locked(l);
    tv_list_set_lock(l, VAR_FIXED);
    while !li.is_null() {
        let mut argv: [typval_T; 3] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 3];
        argv[0 as ::core::ffi::c_int as usize] = *rettv;
        argv[1 as ::core::ffi::c_int as usize] = (*li).li_tv;
        (*rettv).v_type = VAR_UNKNOWN;
        let r: ::core::ffi::c_int = eval_expr_typval(
            expr,
            true_0 != 0,
            &raw mut argv as *mut typval_T,
            2 as ::core::ffi::c_int,
            rettv,
        );
        tv_clear((&raw mut argv as *mut typval_T).offset(0 as ::core::ffi::c_int as isize));
        if r == FAIL || called_emsg.get() != called_emsg_start {
            break;
        }
        li = (*li).li_next;
    }
    tv_list_set_lock(l, prev_locked);
}
unsafe extern "C" fn reduce_string(
    mut argvars: *mut typval_T,
    mut expr: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    let mut p: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut len: ::core::ffi::c_int = 0;
    let called_emsg_start: ::core::ffi::c_int = called_emsg.get();
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if *p as ::core::ffi::c_int == NUL {
            semsg(
                gettext(
                    &raw const e_reduce_of_an_empty_str_with_no_initial_value
                        as *const ::core::ffi::c_char,
                ),
                b"String\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
        len = utfc_ptr2len(p);
        *rettv = typval_T {
            v_type: VAR_STRING,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_string: xmemdupz(p as *const ::core::ffi::c_void, len as size_t)
                    as *mut ::core::ffi::c_char,
            },
        };
        p = p.offset(len as isize);
    } else if tv_check_for_string_arg(argvars, 2 as ::core::ffi::c_int) == FAIL {
        return;
    } else {
        tv_copy(argvars.offset(2 as ::core::ffi::c_int as isize), rettv);
    }
    while *p as ::core::ffi::c_int != NUL {
        let mut argv: [typval_T; 3] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 3];
        argv[0 as ::core::ffi::c_int as usize] = *rettv;
        len = utfc_ptr2len(p);
        argv[1 as ::core::ffi::c_int as usize] = typval_T {
            v_type: VAR_STRING,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_string: xmemdupz(p as *const ::core::ffi::c_void, len as size_t)
                    as *mut ::core::ffi::c_char,
            },
        };
        let r: ::core::ffi::c_int = eval_expr_typval(
            expr,
            true_0 != 0,
            &raw mut argv as *mut typval_T,
            2 as ::core::ffi::c_int,
            rettv,
        );
        tv_clear((&raw mut argv as *mut typval_T).offset(0 as ::core::ffi::c_int as isize));
        tv_clear((&raw mut argv as *mut typval_T).offset(1 as ::core::ffi::c_int as isize));
        if r == FAIL || called_emsg.get() != called_emsg_start {
            break;
        }
        p = p.offset(len as isize);
    }
}
unsafe extern "C" fn reduce_blob(
    mut argvars: *mut typval_T,
    mut expr: *mut typval_T,
    mut rettv: *mut typval_T,
) {
    let b: *const blob_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_blob;
    let called_emsg_start: ::core::ffi::c_int = called_emsg.get();
    let mut initial: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut i: ::core::ffi::c_int = 0;
    if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tv_blob_len(b) == 0 as ::core::ffi::c_int {
            semsg(
                gettext(
                    &raw const e_reduce_of_an_empty_str_with_no_initial_value
                        as *const ::core::ffi::c_char,
                ),
                b"Blob\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
        initial = typval_T {
            v_type: VAR_NUMBER,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_number: tv_blob_get(b, 0 as ::core::ffi::c_int) as varnumber_T,
            },
        };
        i = 1 as ::core::ffi::c_int;
    } else if tv_check_for_number_arg(argvars, 2 as ::core::ffi::c_int) == FAIL {
        return;
    } else {
        initial = *argvars.offset(2 as ::core::ffi::c_int as isize);
        i = 0 as ::core::ffi::c_int;
    }
    tv_copy(&raw mut initial, rettv);
    while i < tv_blob_len(b) {
        let mut argv: [typval_T; 3] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 3];
        argv[0 as ::core::ffi::c_int as usize] = *rettv;
        argv[1 as ::core::ffi::c_int as usize] = typval_T {
            v_type: VAR_NUMBER,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_number: tv_blob_get(b, i) as varnumber_T,
            },
        };
        let r: ::core::ffi::c_int = eval_expr_typval(
            expr,
            true_0 != 0,
            &raw mut argv as *mut typval_T,
            2 as ::core::ffi::c_int,
            rettv,
        );
        if r == FAIL || called_emsg.get() != called_emsg_start {
            return;
        }
        i += 1;
    }
}
pub unsafe extern "C" fn f_reduce(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        emsg(gettext(
            (e_string_list_or_blob_required.ptr() as *const _) as *const ::core::ffi::c_char,
        ));
        return;
    }
    let mut func_name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        func_name = (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_string;
    } else if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        func_name = partial_name(
            (*argvars.offset(1 as ::core::ffi::c_int as isize))
                .vval
                .v_partial,
        );
    } else {
        func_name = tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
    }
    if func_name.is_null() || *func_name as ::core::ffi::c_int == NUL {
        emsg(gettext(
            (e_missing_function_argument.ptr() as *const _) as *const ::core::ffi::c_char,
        ));
        return;
    }
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        reduce_list(
            argvars,
            argvars.offset(1 as ::core::ffi::c_int as isize),
            rettv,
        );
    } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        reduce_string(
            argvars,
            argvars.offset(1 as ::core::ffi::c_int as isize),
            rettv,
        );
    } else {
        reduce_blob(
            argvars,
            argvars.offset(1 as ::core::ffi::c_int as isize),
            rettv,
        );
    };
}
