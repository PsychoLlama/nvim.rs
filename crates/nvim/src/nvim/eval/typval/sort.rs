//! `sort()` and `uniq()`: the comparators and the two driver loops.
//!
//! [`item_compare`] is the default ordering and [`item_compare2`] the one
//! that calls a user function or dictionary method; each has the
//! `_keeping_zero` / `_not_keeping_zero` pair upstream hands to `qsort` so a
//! comparison error can stop the sort.  [`parse_sort_uniq_args`] reads the
//! optional `{how}` and `{dict}` arguments both builtins share.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn item_compare(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
    mut keep_zero: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut tofree1: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut tofree2: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut p1: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut p2: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let si1: *mut ListSortItem = s1 as *mut ListSortItem;
        let si2: *mut ListSortItem = s2 as *mut ListSortItem;
        let tv1: *mut typval_T = &raw mut (*(*si1).item).li_tv;
        let tv2: *mut typval_T = &raw mut (*(*si2).item).li_tv;
        let mut res: ::core::ffi::c_int = 0;
        if (*sortinfo.get()).item_compare_numbers {
            let v1: varnumber_T = tv_get_number(tv1);
            let v2: varnumber_T = tv_get_number(tv2);
            res = if v1 == v2 {
                0 as ::core::ffi::c_int
            } else if v1 > v2 {
                1 as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            };
        } else if (*sortinfo.get()).item_compare_float {
            let v1_0: float_T = tv_get_float(tv1);
            let v2_0: float_T = tv_get_float(tv2);
            res = if v1_0 == v2_0 {
                0 as ::core::ffi::c_int
            } else if v1_0 > v2_0 {
                1 as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            };
        } else {
            tofree1 = ::core::ptr::null_mut::<::core::ffi::c_char>();
            tofree2 = ::core::ptr::null_mut::<::core::ffi::c_char>();
            p1 = ::core::ptr::null_mut::<::core::ffi::c_char>();
            p2 = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if (*tv1).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if (*tv2).v_type as ::core::ffi::c_uint
                    != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                    || (*sortinfo.get()).item_compare_numeric as ::core::ffi::c_int != 0
                {
                    p1 = b"'\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    p1 = (*tv1).vval.v_string;
                }
            } else {
                p1 = encode_tv2string(tv1, ::core::ptr::null_mut::<size_t>());
                tofree1 = p1;
            }
            if (*tv2).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if (*tv1).v_type as ::core::ffi::c_uint
                    != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                    || (*sortinfo.get()).item_compare_numeric as ::core::ffi::c_int != 0
                {
                    p2 = b"'\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                } else {
                    p2 = (*tv2).vval.v_string;
                }
            } else {
                p2 = encode_tv2string(tv2, ::core::ptr::null_mut::<size_t>());
                tofree2 = p2;
            }
            if p1.is_null() {
                p1 = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            if p2.is_null() {
                p2 = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            if !(*sortinfo.get()).item_compare_numeric {
                if (*sortinfo.get()).item_compare_lc {
                    res = strcoll(p1, p2);
                } else {
                    res = if (*sortinfo.get()).item_compare_ic != 0 {
                        strcasecmp(p1, p2)
                    } else {
                        strcmp(p1, p2)
                    };
                }
            } else {
                let mut n1: ::core::ffi::c_double = strtod(p1, &raw mut p1);
                let mut n2: ::core::ffi::c_double = strtod(p2, &raw mut p2);
                res = if n1 == n2 {
                    0 as ::core::ffi::c_int
                } else if n1 > n2 {
                    1 as ::core::ffi::c_int
                } else {
                    -1 as ::core::ffi::c_int
                };
            }
            xfree(tofree1 as *mut ::core::ffi::c_void);
            xfree(tofree2 as *mut ::core::ffi::c_void);
        }
        if res == 0 as ::core::ffi::c_int && !keep_zero {
            res = if (*si1).idx > (*si2).idx {
                1 as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            };
        }
        return res;
    }
}

pub(crate) unsafe extern "C" fn item_compare_keeping_zero(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        return item_compare(s1, s2, true_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn item_compare_not_keeping_zero(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        return item_compare(s1, s2, false_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn item_compare2(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
    mut keep_zero: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut argv: [typval_T; 3] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 3];
        let mut func_name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut partial: *mut partial_T = (*sortinfo.get()).item_compare_partial;
        if (*sortinfo.get()).item_compare_func_err {
            return 0 as ::core::ffi::c_int;
        }
        let mut si1: *mut ListSortItem = s1 as *mut ListSortItem;
        let mut si2: *mut ListSortItem = s2 as *mut ListSortItem;
        if partial.is_null() {
            func_name = (*sortinfo.get()).item_compare_func;
        } else {
            func_name = partial_name(partial);
        }
        tv_copy(
            &raw mut (*(*si1).item).li_tv,
            (&raw mut argv as *mut typval_T).offset(0 as ::core::ffi::c_int as isize),
        );
        tv_copy(
            &raw mut (*(*si2).item).li_tv,
            (&raw mut argv as *mut typval_T).offset(1 as ::core::ffi::c_int as isize),
        );
        rettv.v_type = VAR_UNKNOWN;
        let mut funcexe: funcexe_T = FUNCEXE_INIT;
        funcexe.fe_evaluate = true_0 != 0;
        funcexe.fe_partial = partial;
        funcexe.fe_selfdict = (*sortinfo.get()).item_compare_selfdict;
        let mut res: ::core::ffi::c_int = call_func(
            func_name,
            -1 as ::core::ffi::c_int,
            &raw mut rettv,
            2 as ::core::ffi::c_int,
            &raw mut argv as *mut typval_T,
            &raw mut funcexe,
        );
        tv_clear((&raw mut argv as *mut typval_T).offset(0 as ::core::ffi::c_int as isize));
        tv_clear((&raw mut argv as *mut typval_T).offset(1 as ::core::ffi::c_int as isize));
        if res == FAIL {
            res = ITEM_COMPARE_FAIL;
            (*sortinfo.get()).item_compare_func_err = true_0 != 0;
        } else {
            let mut n: varnumber_T = tv_get_number_chk(
                &raw mut rettv,
                &raw mut (*sortinfo.get()).item_compare_func_err,
            );
            res = if n > 0 as varnumber_T {
                1 as ::core::ffi::c_int
            } else if n < 0 as varnumber_T {
                -1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            };
        }
        if (*sortinfo.get()).item_compare_func_err {
            res = ITEM_COMPARE_FAIL;
        }
        tv_clear(&raw mut rettv);
        if res == 0 as ::core::ffi::c_int && !keep_zero {
            res = if (*si1).idx > (*si2).idx {
                1 as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            };
        }
        return res;
    }
}

pub(crate) unsafe extern "C" fn item_compare2_keeping_zero(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        return item_compare2(s1, s2, true_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn item_compare2_not_keeping_zero(
    mut s1: *const ::core::ffi::c_void,
    mut s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        return item_compare2(s1, s2, false_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn do_sort(mut l: *mut list_T, mut info: *mut sortinfo_T) {
    unsafe {
        let len: ::core::ffi::c_int = tv_list_len(l);
        let mut ptrs: *mut ListSortItem = xmalloc(
            (len as ::core::ffi::c_uint as usize)
                .wrapping_mul(::core::mem::size_of::<ListSortItem>()),
        ) as *mut ListSortItem;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let l_: *mut list_T = l;
        if !l_.is_null() {
            let mut li: *mut listitem_T = (*l_).lv_first;
            while !li.is_null() {
                (*ptrs.offset(i as isize)).item = li;
                (*ptrs.offset(i as isize)).idx = i;
                i += 1;
                li = (*li).li_next;
            }
        }
        (*info).item_compare_func_err = false_0 != 0;
        let mut item_compare_func: ListSorter =
            if (*info).item_compare_func.is_null() && (*info).item_compare_partial.is_null() {
                Some(
                    item_compare_not_keeping_zero
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_void,
                            *const ::core::ffi::c_void,
                        ) -> ::core::ffi::c_int,
                )
            } else {
                Some(
                    item_compare2_not_keeping_zero
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_void,
                            *const ::core::ffi::c_void,
                        ) -> ::core::ffi::c_int,
                )
            };
        qsort(
            ptrs as *mut ::core::ffi::c_void,
            len as size_t,
            ::core::mem::size_of::<ListSortItem>(),
            item_compare_func as __compar_fn_t,
        );
        if !(*info).item_compare_func_err {
            (*l).lv_first = ::core::ptr::null_mut::<listitem_T>();
            (*l).lv_last = ::core::ptr::null_mut::<listitem_T>();
            (*l).lv_idx_item = ::core::ptr::null_mut::<listitem_T>();
            (*l).lv_len = 0 as ::core::ffi::c_int;
            i = 0 as ::core::ffi::c_int;
            while i < len {
                tv_list_append(l, (*ptrs.offset(i as isize)).item);
                i += 1;
            }
        }
        if (*info).item_compare_func_err {
            emsg(gettext(
                b"E702: Sort compare function failed\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
        xfree(ptrs as *mut ::core::ffi::c_void);
    }
}

pub(crate) unsafe extern "C" fn do_uniq(mut l: *mut list_T, mut info: *mut sortinfo_T) {
    unsafe {
        let len: ::core::ffi::c_int = tv_list_len(l);
        let mut ptrs: *mut ListSortItem = xmalloc(
            (len as ::core::ffi::c_uint as usize)
                .wrapping_mul(::core::mem::size_of::<ListSortItem>()),
        ) as *mut ListSortItem;
        (*info).item_compare_func_err = false_0 != 0;
        let mut item_compare_func: ListSorter =
            if (*info).item_compare_func.is_null() && (*info).item_compare_partial.is_null() {
                Some(
                    item_compare_keeping_zero
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_void,
                            *const ::core::ffi::c_void,
                        ) -> ::core::ffi::c_int,
                )
            } else {
                Some(
                    item_compare2_keeping_zero
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_void,
                            *const ::core::ffi::c_void,
                        ) -> ::core::ffi::c_int,
                )
            };
        let mut li: *mut listitem_T = (*tv_list_first(l)).li_next;
        while !li.is_null() {
            let prev_li: *mut listitem_T = (*li).li_prev;
            if item_compare_func.expect("non-null function pointer")(
                &raw const prev_li as *const ::core::ffi::c_void,
                &raw mut li as *const ::core::ffi::c_void,
            ) == 0 as ::core::ffi::c_int
            {
                li = tv_list_item_remove(l, li);
            } else {
                li = (*li).li_next;
            }
            if !(*info).item_compare_func_err {
                continue;
            }
            emsg(gettext(
                b"E882: Uniq compare function failed\0".as_ptr() as *const ::core::ffi::c_char
            ));
            break;
        }
        xfree(ptrs as *mut ::core::ffi::c_void);
    }
}

pub(crate) unsafe extern "C" fn parse_sort_uniq_args(
    mut argvars: *mut typval_T,
    mut info: *mut sortinfo_T,
) -> ::core::ffi::c_int {
    unsafe {
        (*info).item_compare_ic = false_0;
        (*info).item_compare_lc = false_0 != 0;
        (*info).item_compare_numeric = false_0 != 0;
        (*info).item_compare_numbers = false_0 != 0;
        (*info).item_compare_float = false_0 != 0;
        (*info).item_compare_func = ::core::ptr::null::<::core::ffi::c_char>();
        (*info).item_compare_partial = ::core::ptr::null_mut::<partial_T>();
        (*info).item_compare_selfdict = ::core::ptr::null_mut::<dict_T>();
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return OK;
        }
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*info).item_compare_func = (*argvars.offset(1 as ::core::ffi::c_int as isize))
                .vval
                .v_string;
        } else if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*info).item_compare_partial = (*argvars.offset(1 as ::core::ffi::c_int as isize))
                .vval
                .v_partial;
        } else {
            let mut error: bool = false_0 != 0;
            let mut nr: ::core::ffi::c_int = tv_get_number_chk(
                argvars.offset(1 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) as ::core::ffi::c_int;
            if error {
                return FAIL;
            }
            if nr == 1 as ::core::ffi::c_int {
                (*info).item_compare_ic = true_0;
            } else if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type
                as ::core::ffi::c_uint
                != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*info).item_compare_func =
                    tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
            } else if nr != 0 as ::core::ffi::c_int {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
                return FAIL;
            }
            if !(*info).item_compare_func.is_null() {
                if *(*info).item_compare_func as ::core::ffi::c_int == NUL {
                    (*info).item_compare_func = ::core::ptr::null::<::core::ffi::c_char>();
                } else if strcmp(
                    (*info).item_compare_func,
                    b"n\0".as_ptr() as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    (*info).item_compare_func = ::core::ptr::null::<::core::ffi::c_char>();
                    (*info).item_compare_numeric = true_0 != 0;
                } else if strcmp(
                    (*info).item_compare_func,
                    b"N\0".as_ptr() as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    (*info).item_compare_func = ::core::ptr::null::<::core::ffi::c_char>();
                    (*info).item_compare_numbers = true_0 != 0;
                } else if strcmp(
                    (*info).item_compare_func,
                    b"f\0".as_ptr() as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    (*info).item_compare_func = ::core::ptr::null::<::core::ffi::c_char>();
                    (*info).item_compare_float = true_0 != 0;
                } else if strcmp(
                    (*info).item_compare_func,
                    b"i\0".as_ptr() as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    (*info).item_compare_func = ::core::ptr::null::<::core::ffi::c_char>();
                    (*info).item_compare_ic = true_0;
                } else if strcmp(
                    (*info).item_compare_func,
                    b"l\0".as_ptr() as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    (*info).item_compare_func = ::core::ptr::null::<::core::ffi::c_char>();
                    (*info).item_compare_lc = true_0 != 0;
                }
            }
        }
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if tv_check_for_dict_arg(argvars, 2 as ::core::ffi::c_int) == FAIL {
                return FAIL;
            }
            (*info).item_compare_selfdict = (*argvars.offset(2 as ::core::ffi::c_int as isize))
                .vval
                .v_dict;
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn do_sort_uniq(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut sort: bool,
) {
    unsafe {
        let mut len: ::core::ffi::c_int = 0;
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            semsg(
                gettext(&raw const e_listarg as *const ::core::ffi::c_char),
                if sort as ::core::ffi::c_int != 0 {
                    b"sort()\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"uniq()\0".as_ptr() as *const ::core::ffi::c_char
                },
            );
            return;
        }
        let mut info: sortinfo_T = sortinfo_T {
            item_compare_ic: 0,
            item_compare_lc: false,
            item_compare_numeric: false,
            item_compare_numbers: false,
            item_compare_float: false,
            item_compare_func: ::core::ptr::null::<::core::ffi::c_char>(),
            item_compare_partial: ::core::ptr::null_mut::<partial_T>(),
            item_compare_selfdict: ::core::ptr::null_mut::<dict_T>(),
            item_compare_func_err: false,
        };
        let mut old_sortinfo: *mut sortinfo_T = sortinfo.get();
        sortinfo.set(&raw mut info);
        let arg_errmsg: *const ::core::ffi::c_char = if sort as ::core::ffi::c_int != 0 {
            b"sort() argument\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"uniq() argument\0".as_ptr() as *const ::core::ffi::c_char
        };
        let l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list;
        if !value_check_lock(tv_list_locked(l), arg_errmsg, TV_TRANSLATE as size_t) {
            tv_list_set_ret(rettv, l);
            len = tv_list_len(l);
            if len > 1 as ::core::ffi::c_int {
                if parse_sort_uniq_args(argvars, &raw mut info) != FAIL {
                    if sort {
                        do_sort(l, &raw mut info);
                    } else {
                        do_uniq(l, &raw mut info);
                    }
                }
            }
        }
        sortinfo.set(old_sortinfo);
    }
}

pub unsafe extern "C" fn f_sort(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        do_sort_uniq(argvars, rettv, true_0 != 0);
    }
}

pub unsafe extern "C" fn f_uniq(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        do_sort_uniq(argvars, rettv, false_0 != 0);
    }
}
