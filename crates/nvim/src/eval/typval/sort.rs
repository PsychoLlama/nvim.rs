//! `sort()` and `uniq()`: the comparators and the two driver loops.
//!
//! [`item_compare`] is the default ordering and [`item_compare2`] the one
//! that calls a user function or dictionary method; each has the
//! `_keeping_zero` / `_not_keeping_zero` pair upstream hands to `qsort` so a
//! comparison error can stop the sort.  [`parse_sort_uniq_args`] reads the
//! optional `{how}` and `{dict}` arguments both builtins share.
//!
//! The four comparators keep `extern "C"` — `qsort` calls them — and the
//! sort keeps `qsort`.  A
//! `sort_by` is not a provable substitute here: a user comparison function can
//! answer inconsistently (or fail part-way), so which permutation of equal
//! items comes out is whatever the C library's sort did.  The `_not_keeping_zero`
//! pair exists to make ties total by original index, but only the
//! `_keeping_zero` pair reaches `uniq`.  `sortinfo` is a global for the same
//! reason: `qsort` has nowhere to put a context pointer.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::semsg_c;

/// Compare two list items by the ordering `sortinfo` selected: numeric, float,
/// or a string comparison of their `string()` forms.
///
/// With `keep_zero` clear, ties are broken by the items' original indexes,
/// which is what makes the sort stable.
pub(crate) unsafe extern "C" fn item_compare(
    s1: *const ::core::ffi::c_void,
    s2: *const ::core::ffi::c_void,
    keep_zero: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let si1 = s1 as *mut ListSortItem;
        let si2 = s2 as *mut ListSortItem;
        let tv1 = &raw mut (*(*si1).item).li_tv;
        let tv2 = &raw mut (*(*si2).item).li_tv;
        let info = sortinfo.get();

        // `cmp` on three-way-compared scalars: upstream's `a == b ? 0 : a > b
        // ? 1 : -1`, which for a NaN float answers -1 as this does.
        let sign = |greater: bool, equal: bool| {
            if equal {
                0
            } else if greater {
                1
            } else {
                -1
            }
        };

        let mut res;
        if (*info).item_compare_numbers {
            let v1 = tv_get_number(tv1);
            let v2 = tv_get_number(tv2);
            res = sign(v1 > v2, v1 == v2);
        } else if (*info).item_compare_float {
            let v1 = tv_get_float(tv1);
            let v2 = tv_get_float(tv2);
            res = sign(v1 > v2, v1 == v2);
        } else {
            // encode_tv2string() puts quotes around a string and allocates
            // memory.  Don't do that for string variables. Use a single quote
            // when comparing with a non-string to do what the docs promise.
            let mut tofree1 = ::core::ptr::null_mut();
            let mut tofree2 = ::core::ptr::null_mut();
            let mut p1;
            let mut p2;
            if (*tv1).v_type == VAR_STRING {
                if (*tv2).v_type != VAR_STRING || (*info).item_compare_numeric {
                    p1 = c"'".as_ptr().cast_mut();
                } else {
                    p1 = (*tv1).vval.v_string;
                }
            } else {
                p1 = encode_tv2string(tv1, ::core::ptr::null_mut());
                tofree1 = p1;
            }
            if (*tv2).v_type == VAR_STRING {
                if (*tv1).v_type != VAR_STRING || (*info).item_compare_numeric {
                    p2 = c"'".as_ptr().cast_mut();
                } else {
                    p2 = (*tv2).vval.v_string;
                }
            } else {
                p2 = encode_tv2string(tv2, ::core::ptr::null_mut());
                tofree2 = p2;
            }
            if p1.is_null() {
                p1 = c"".as_ptr().cast_mut();
            }
            if p2.is_null() {
                p2 = c"".as_ptr().cast_mut();
            }

            if !(*info).item_compare_numeric {
                res = if (*info).item_compare_lc {
                    strcoll(p1, p2)
                } else if (*info).item_compare_ic != 0 {
                    strcasecmp(p1, p2)
                } else {
                    strcmp(p1, p2)
                };
            } else {
                // `strtod` moves p1/p2 past the number; nothing reads them
                // after, which is why upstream passes them as the end pointers.
                let n1 = strtod(p1, &raw mut p1);
                let n2 = strtod(p2, &raw mut p2);
                res = sign(n1 > n2, n1 == n2);
            }

            xfree(tofree1.cast());
            xfree(tofree2.cast());
        }

        if res == 0 && !keep_zero {
            res = if (*si1).idx > (*si2).idx { 1 } else { -1 };
        }
        res
    }
}

/// [`item_compare`] answering 0 for equal items — `uniq`'s comparator.
pub(crate) unsafe extern "C" fn item_compare_keeping_zero(
    s1: *const ::core::ffi::c_void,
    s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe { item_compare(s1, s2, true) }
}

/// [`item_compare`] breaking ties by index — `sort`'s comparator.
pub(crate) unsafe extern "C" fn item_compare_not_keeping_zero(
    s1: *const ::core::ffi::c_void,
    s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe { item_compare(s1, s2, false) }
}

/// Compare two list items by calling the user function `sortinfo` holds.
///
/// A failed call sets `item_compare_func_err`, which makes every later
/// comparison answer 0 and the driver abandon the sort.
pub(crate) unsafe extern "C" fn item_compare2(
    s1: *const ::core::ffi::c_void,
    s2: *const ::core::ffi::c_void,
    keep_zero: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let info = sortinfo.get();
        let partial = (*info).item_compare_partial;

        // shortcut after failure in previous call; compare all items equal
        if (*info).item_compare_func_err {
            return 0;
        }

        let si1 = s1 as *mut ListSortItem;
        let si2 = s2 as *mut ListSortItem;
        let func_name = if partial.is_null() {
            (*info).item_compare_func
        } else {
            partial_name(partial)
        };

        // Copy the values.  This is needed to be able to set v_lock to
        // VAR_FIXED in the copy without changing the original list items.
        let mut argv = [TV_INITIAL_VALUE; 3];
        tv_copy(&raw mut (*(*si1).item).li_tv, &raw mut argv[0]);
        tv_copy(&raw mut (*(*si2).item).li_tv, &raw mut argv[1]);

        let mut rettv = TV_INITIAL_VALUE;
        let mut funcexe = FUNCEXE_INIT;
        funcexe.fe_evaluate = true;
        funcexe.fe_partial = partial;
        funcexe.fe_selfdict = (*info).item_compare_selfdict;
        let mut res = call_func(
            func_name,
            -1,
            &raw mut rettv,
            2,
            argv.as_mut_ptr(),
            &raw mut funcexe,
        );
        tv_clear(&raw mut argv[0]);
        tv_clear(&raw mut argv[1]);

        if res == FAIL {
            res = ITEM_COMPARE_FAIL;
            (*info).item_compare_func_err = true;
        } else {
            let n = tv_get_number_chk(&raw mut rettv, &raw mut (*info).item_compare_func_err);
            res = if n > 0 {
                1
            } else if n < 0 {
                -1
            } else {
                0
            };
        }
        if (*info).item_compare_func_err {
            res = ITEM_COMPARE_FAIL; // return value has wrong type
        }
        tv_clear(&raw mut rettv);

        if res == 0 && !keep_zero {
            res = if (*si1).idx > (*si2).idx { 1 } else { -1 };
        }
        res
    }
}

/// [`item_compare2`] answering 0 for equal items — `uniq`'s comparator.
pub(crate) unsafe extern "C" fn item_compare2_keeping_zero(
    s1: *const ::core::ffi::c_void,
    s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe { item_compare2(s1, s2, true) }
}

/// [`item_compare2`] breaking ties by index — `sort`'s comparator.
pub(crate) unsafe extern "C" fn item_compare2_not_keeping_zero(
    s1: *const ::core::ffi::c_void,
    s2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe { item_compare2(s1, s2, false) }
}

/// Which comparator `info` selects: the built-in ordering, or the user
/// function.
fn sorter(info: *const sortinfo_T, keep_zero: bool) -> ListSorter {
    let builtin =
        unsafe { (*info).item_compare_func.is_null() && (*info).item_compare_partial.is_null() };
    Some(match (builtin, keep_zero) {
        (true, true) => {
            item_compare_keeping_zero
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int
        }
        (true, false) => item_compare_not_keeping_zero,
        (false, true) => item_compare2_keeping_zero,
        (false, false) => item_compare2_not_keeping_zero,
    })
}

/// `sort()` over `l`, in place.
pub(crate) unsafe extern "C" fn do_sort(l: *mut list_T, info: *mut sortinfo_T) {
    unsafe {
        let len = tv_list_len(l);

        // Make an array with each entry pointing to an item in the List.
        let ptrs =
            xmalloc(len as usize * ::core::mem::size_of::<ListSortItem>()) as *mut ListSortItem;

        // f_sort(): ptrs will be the list to sort
        for (i, li) in tv_list_iter(l.as_ref()).enumerate() {
            (*ptrs.add(i)).item = li;
            (*ptrs.add(i)).idx = i as ::core::ffi::c_int;
        }

        (*info).item_compare_func_err = false;
        let item_compare_func = sorter(info, false);

        // Sort the array with item pointers.
        qsort(
            ptrs.cast(),
            len as size_t,
            ::core::mem::size_of::<ListSortItem>(),
            item_compare_func as __compar_fn_t,
        );

        if (*info).item_compare_func_err {
            emsg(gettext(c"E702: Sort compare function failed".as_ptr()));
        } else {
            // Clear the list and append the items in the sorted order.
            (*l).lv_first = ::core::ptr::null_mut();
            (*l).lv_last = ::core::ptr::null_mut();
            (*l).lv_idx_item = ::core::ptr::null_mut();
            (*l).lv_len = 0;
            for i in 0..len {
                tv_list_append(l, (*ptrs.offset(i as isize)).item);
            }
        }

        xfree(ptrs.cast());
    }
}

/// `uniq()` over `l`, in place: drop each item equal to the one before it.
pub(crate) unsafe extern "C" fn do_uniq(l: *mut list_T, info: *mut sortinfo_T) {
    unsafe {
        let len = tv_list_len(l);

        // Upstream allocates this array and never fills it — `uniq` walks the
        // list directly. Kept because it is what the C does; nothing reads it.
        let ptrs =
            xmalloc(len as usize * ::core::mem::size_of::<ListSortItem>()) as *mut ListSortItem;

        (*info).item_compare_func_err = false;
        let item_compare_func = sorter(info, true);

        let mut li = (*tv_list_first(l)).li_next;
        while !li.is_null() {
            let prev_li = (*li).li_prev;
            // Upstream hands the comparator the addresses of these two
            // `listitem_T *` locals, which it reads as `ListSortItem *`: the
            // `item` field lines up and `idx` is never touched, because only
            // the `_keeping_zero` comparators reach here.
            let equal = item_compare_func.expect("non-null function pointer")(
                (&raw const prev_li).cast(),
                (&raw const li).cast(),
            ) == 0;
            li = if equal {
                tv_list_item_remove(l, li)
            } else {
                (*li).li_next
            };
            if (*info).item_compare_func_err {
                emsg(gettext(c"E882: Uniq compare function failed".as_ptr()));
                break;
            }
        }

        xfree(ptrs.cast());
    }
}

/// Read `sort()`/`uniq()`'s optional `{how}` and `{dict}` arguments into
/// `info`.
pub(crate) unsafe extern "C" fn parse_sort_uniq_args(
    argvars: *mut typval_T,
    info: *mut sortinfo_T,
) -> ::core::ffi::c_int {
    unsafe {
        (*info).item_compare_ic = false_0;
        (*info).item_compare_lc = false;
        (*info).item_compare_numeric = false;
        (*info).item_compare_numbers = false;
        (*info).item_compare_float = false;
        (*info).item_compare_func = ::core::ptr::null();
        (*info).item_compare_partial = ::core::ptr::null_mut();
        (*info).item_compare_selfdict = ::core::ptr::null_mut();

        if (*argvars.add(1)).v_type == VAR_UNKNOWN {
            return OK;
        }

        // optional second argument: {func}
        if (*argvars.add(1)).v_type == VAR_FUNC {
            (*info).item_compare_func = (*argvars.add(1)).vval.v_string;
        } else if (*argvars.add(1)).v_type == VAR_PARTIAL {
            (*info).item_compare_partial = (*argvars.add(1)).vval.v_partial;
        } else {
            let mut error = false;
            let nr = tv_get_number_chk(argvars.add(1), &raw mut error) as ::core::ffi::c_int;
            if error {
                return FAIL; // type error; errmsg already given
            }
            if nr == 1 {
                (*info).item_compare_ic = true_0;
            } else if (*argvars.add(1)).v_type != VAR_NUMBER {
                (*info).item_compare_func = tv_get_string(argvars.add(1));
            } else if nr != 0 {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
                return FAIL;
            }

            let how = (*info).item_compare_func;
            if !how.is_null() {
                if *how as ::core::ffi::c_int == NUL {
                    // empty string means default sort
                    (*info).item_compare_func = ::core::ptr::null();
                } else if *how.add(1) as ::core::ffi::c_int == NUL {
                    // The five built-in orderings are one-character names;
                    // upstream spells each as a `strcmp` against a literal.
                    let mut builtin = true;
                    match *how as u8 {
                        b'n' => (*info).item_compare_numeric = true,
                        b'N' => (*info).item_compare_numbers = true,
                        b'f' => (*info).item_compare_float = true,
                        b'i' => (*info).item_compare_ic = true_0,
                        b'l' => (*info).item_compare_lc = true,
                        _ => builtin = false,
                    }
                    if builtin {
                        (*info).item_compare_func = ::core::ptr::null();
                    }
                }
            }
        }

        if (*argvars.add(2)).v_type != VAR_UNKNOWN {
            // optional third argument: {dict}
            if tv_check_for_dict_arg(argvars, 2) == FAIL {
                return FAIL;
            }
            (*info).item_compare_selfdict = (*argvars.add(2)).vval.v_dict;
        }

        OK
    }
}

/// The body `sort()` and `uniq()` share: check the argument, publish a
/// `sortinfo`, and run the driver.
///
/// `sortinfo` is saved and restored around the call because a user comparison
/// function can itself call `sort()`.
pub(crate) unsafe extern "C" fn do_sort_uniq(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    sort: bool,
) {
    unsafe {
        if (*argvars).v_type != VAR_LIST {
            semsg_c!(
                gettext(&raw const e_listarg as *const ::core::ffi::c_char),
                if sort {
                    c"sort()".as_ptr()
                } else {
                    c"uniq()".as_ptr()
                },
            );
            return;
        }

        let mut info = SORTINFO_INIT;
        let old_sortinfo = sortinfo.get();
        sortinfo.set(&raw mut info);

        let arg_errmsg = if sort {
            c"sort() argument".as_ptr()
        } else {
            c"uniq() argument".as_ptr()
        };
        let l = (*argvars).vval.v_list;
        if !value_check_lock(tv_list_locked(l), arg_errmsg, TV_TRANSLATE as size_t) {
            tv_list_set_ret(rettv, l);
            if tv_list_len(l) > 1 && parse_sort_uniq_args(argvars, &raw mut info) != FAIL {
                if sort {
                    do_sort(l, &raw mut info);
                } else {
                    do_uniq(l, &raw mut info);
                }
            }
        }

        sortinfo.set(old_sortinfo);
    }
}

/// `sort()`.
pub unsafe fn f_sort(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        do_sort_uniq(argvars, rettv, true);
    }
}

/// `uniq()`.
pub unsafe fn f_uniq(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        do_sort_uniq(argvars, rettv, false);
    }
}
