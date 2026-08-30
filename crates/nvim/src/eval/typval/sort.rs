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
use crate::cstr;
use crate::message_fmt::c_str;
use crate::semsg;
use crate::types::{Failed, NUL};

/// Compare two list items by the ordering `sortinfo` selected: numeric, float,
/// or a string comparison of their `string()` forms.
///
/// With `keep_zero` clear, ties are broken by the items' original indexes,
/// which is what makes the sort stable.
pub(crate) unsafe fn item_compare(
    s1: *const ::core::ffi::c_void,
    s2: *const ::core::ffi::c_void,
    keep_zero: bool,
) -> ::core::ffi::c_int {
    let si1 = s1 as *mut ListSortItem;
    let si2 = s2 as *mut ListSortItem;
    let tv1 = unsafe { &raw mut (*(*si1).item).li_tv };
    let tv2 = unsafe { &raw mut (*(*si2).item).li_tv };
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
    // SAFETY: the `sortinfo_T` the sort set up.
    let sort_info = unsafe { Si::new(info) };
    if sort_info.item_compare_numbers {
        let v1 = unsafe { tv_get_number(tv1) };
        let v2 = unsafe { tv_get_number(tv2) };
        res = sign(v1 > v2, v1 == v2);
    } else if sort_info.item_compare_float {
        let v1 = unsafe { tv_get_float(tv1) };
        let v2 = unsafe { tv_get_float(tv2) };
        res = sign(v1 > v2, v1 == v2);
    } else {
        // encode_tv2string() puts quotes around a string and allocates
        // memory.  Don't do that for string variables. Use a single quote
        // when comparing with a non-string to do what the docs promise.
        let mut tofree1 = ::core::ptr::null_mut();
        let mut tofree2 = ::core::ptr::null_mut();
        let mut p1;
        let mut p2;
        // SAFETY: the two items' values, live while their lists are.
        let (a, b) = unsafe { (Tv::new(tv1), Tv::new(tv2)) };
        if a.v_type == VAR_STRING {
            if b.v_type != VAR_STRING || sort_info.item_compare_numeric {
                p1 = c"'".as_ptr().cast_mut();
            } else {
                p1 = a.string();
            }
        } else {
            p1 = unsafe { encode_tv2string(tv1, ::core::ptr::null_mut()) };
            tofree1 = p1;
        }
        if b.v_type == VAR_STRING {
            if a.v_type != VAR_STRING || sort_info.item_compare_numeric {
                p2 = c"'".as_ptr().cast_mut();
            } else {
                p2 = b.string();
            }
        } else {
            p2 = unsafe { encode_tv2string(tv2, ::core::ptr::null_mut()) };
            tofree2 = p2;
        }
        if p1.is_null() {
            p1 = c"".as_ptr().cast_mut();
        }
        if p2.is_null() {
            p2 = c"".as_ptr().cast_mut();
        }

        if !sort_info.item_compare_numeric {
            res = if sort_info.item_compare_lc {
                unsafe { strcoll(p1, p2) }
            } else if sort_info.item_compare_ic != 0 {
                unsafe { strcasecmp(p1, p2) }
            } else {
                unsafe { cstr::cmp(p1, p2) as ::core::ffi::c_int }
            };
        } else {
            // `strtod` moves p1/p2 past the number; nothing reads them
            // after, which is why upstream passes them as the end pointers.
            let n1 = unsafe { strtod(p1, &raw mut p1) };
            let n2 = unsafe { strtod(p2, &raw mut p2) };
            res = sign(n1 > n2, n1 == n2);
        }

        unsafe { xfree(tofree1.cast()) };
        unsafe { xfree(tofree2.cast()) };
    }

    if res == 0 && !keep_zero {
        res = if unsafe { (*si1).idx } > unsafe { (*si2).idx } {
            1
        } else {
            -1
        };
    }
    res
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
pub(crate) unsafe fn item_compare2(
    s1: *const ::core::ffi::c_void,
    s2: *const ::core::ffi::c_void,
    keep_zero: bool,
) -> ::core::ffi::c_int {
    let info = sortinfo.get();
    // SAFETY: the `sortinfo_T` the sort set up.
    let mut sort_info = unsafe { Si::new(info) };
    let partial = sort_info.item_compare_partial;

    // shortcut after failure in previous call; compare all items equal
    if sort_info.item_compare_func_err {
        return 0;
    }

    let si1 = s1 as *mut ListSortItem;
    let si2 = s2 as *mut ListSortItem;
    let func_name = if partial.is_null() {
        sort_info.item_compare_func
    } else {
        unsafe { partial_name(partial) }
    };

    // Copy the values.  This is needed to be able to set v_lock to
    // VarLock::Fixed in the copy without changing the original list items.
    let mut argv = [TV_INITIAL_VALUE; 3];
    unsafe { tv_copy(&raw mut (*(*si1).item).li_tv, &raw mut argv[0]) };
    unsafe { tv_copy(&raw mut (*(*si2).item).li_tv, &raw mut argv[1]) };

    let mut rettv = TV_INITIAL_VALUE;
    let mut funcexe = FUNCEXE_INIT;
    funcexe.fe_evaluate = true;
    funcexe.fe_partial = partial;
    funcexe.fe_selfdict = sort_info.item_compare_selfdict;
    let argp = argv.as_mut_ptr();
    let called = unsafe { call_func(func_name, -1, &raw mut rettv, 2, argp, &raw mut funcexe) };
    unsafe { tv_clear(&raw mut argv[0]) };
    unsafe { tv_clear(&raw mut argv[1]) };

    let mut res;
    if called.is_err() {
        res = ITEM_COMPARE_FAIL;
        sort_info.item_compare_func_err = true;
    } else {
        let n =
            unsafe { tv_get_number_chk(&raw mut rettv, &raw mut (*info).item_compare_func_err) };
        res = if n > 0 {
            1
        } else if n < 0 {
            -1
        } else {
            0
        };
    }
    if sort_info.item_compare_func_err {
        res = ITEM_COMPARE_FAIL; // return value has wrong type
    }
    unsafe { tv_clear(&raw mut rettv) };

    if res == 0 && !keep_zero {
        res = if unsafe { (*si1).idx } > unsafe { (*si2).idx } {
            1
        } else {
            -1
        };
    }
    res
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

/// The record a comparator reads for one list item, at its position in the
/// list.  Only a `_not_keeping_zero` comparator reads `idx`.
fn sort_item(item: *mut listitem_T, idx: ::core::ffi::c_int) -> ListSortItem {
    ListSortItem { item, idx }
}

/// `sort()` over `l`, in place.
pub(crate) unsafe fn do_sort(l: *mut list_T, info: *mut sortinfo_T) {
    let len = unsafe { tv_list_len(l) };

    // Make an array with each entry pointing to an item in the List.
    let ptrs = unsafe { xmalloc(len as usize * ::core::mem::size_of::<ListSortItem>()) }
        as *mut ListSortItem;

    // f_sort(): ptrs will be the list to sort
    for (i, li) in tv_list_iter(unsafe { l.as_ref() }).enumerate() {
        unsafe { *ptrs.add(i) = sort_item(li, i as ::core::ffi::c_int) };
    }

    // SAFETY: the caller's `sortinfo_T`.
    let mut sort_info = unsafe { Si::new(info) };
    sort_info.item_compare_func_err = false;
    let item_compare_func = sorter(info, false);

    // Sort the array with item pointers.
    let itemsize = ::core::mem::size_of::<ListSortItem>();
    let cmp = item_compare_func as __compar_fn_t;
    unsafe { qsort(ptrs.cast(), len as size_t, itemsize, cmp) };

    if sort_info.item_compare_func_err {
        emsg(gettext(c"E702: Sort compare function failed"));
    } else {
        // Clear the list and append the items in the sorted order.
        unsafe { (*l).lv_first = ::core::ptr::null_mut() };
        unsafe { (*l).lv_last = ::core::ptr::null_mut() };
        unsafe { (*l).lv_idx_item = ::core::ptr::null_mut() };
        unsafe { (*l).lv_len = 0 };
        for i in 0..len {
            unsafe { tv_list_append(l, (*ptrs.offset(i as isize)).item) };
        }
    }

    unsafe { xfree(ptrs.cast()) };
}

/// `uniq()` over `l`, in place: drop each item equal to the one before it.
pub(crate) unsafe fn do_uniq(l: *mut list_T, info: *mut sortinfo_T) {
    let len = unsafe { tv_list_len(l) };

    // Upstream allocates this array and never fills it — `uniq` walks the
    // list directly. Kept because it is what the C does; nothing reads it.
    let ptrs = unsafe { xmalloc(len as usize * ::core::mem::size_of::<ListSortItem>()) }
        as *mut ListSortItem;

    // SAFETY: the caller's `sortinfo_T`.
    let mut sort_info = unsafe { Si::new(info) };
    sort_info.item_compare_func_err = false;
    let compare = sorter(info, true).expect("non-null function pointer");

    let mut li = unsafe { (*tv_list_first(l)).li_next };
    while !li.is_null() {
        // Upstream hands the comparator the addresses of two bare
        // `listitem_T *` locals and lets it read them as `ListSortItem *`,
        // relying on `item` sitting at offset 0 and on `idx` never being
        // touched (only the `_keeping_zero` comparators reach here).  That
        // pun reads eight bytes past a pointer-sized local unless
        // `ListSortItem` happens to have C's field order, so it is
        // out of bounds under `-Zrandomize-layout` -- which is what made
        // `uniq()` compare garbage there.  Building the two records costs
        // the same and promises nothing about the layout, so
        // `ListSortItem` stays free to be reordered.  The indexes are only
        // read by the `_not_keeping_zero` comparators, which never reach
        // here; they are still filled in list order so that would work.
        let prev = sort_item(unsafe { (*li).li_prev }, 0);
        let cur = sort_item(li, 1);
        let equal = unsafe { compare((&raw const prev).cast(), (&raw const cur).cast()) } == 0;
        li = if equal {
            unsafe { tv_list_item_remove(l, li) }
        } else {
            unsafe { (*li).li_next }
        };
        if sort_info.item_compare_func_err {
            emsg(gettext(c"E882: Uniq compare function failed"));
            break;
        }
    }

    unsafe { xfree(ptrs.cast()) };
}

/// Read `sort()`/`uniq()`'s optional `{how}` and `{dict}` arguments into
/// `info`.
///
/// A `{how}` given as a Number has no string of its own, so the caller lends
/// `how` for it: `info.item_compare_func` may borrow it, and the sort
/// reads that field long after this returns.
pub(crate) unsafe fn parse_sort_uniq_args(
    argvars: *mut typval_T,
    info: *mut sortinfo_T,
    how: &mut NumBuf,
) -> Result<(), Failed> {
    // SAFETY: the caller's stack `sortinfo_T`.
    let mut sort_info = unsafe { Si::new(info) };
    sort_info.item_compare_ic = 0;
    sort_info.item_compare_lc = false;
    sort_info.item_compare_numeric = false;
    sort_info.item_compare_numbers = false;
    sort_info.item_compare_float = false;
    sort_info.item_compare_func = ::core::ptr::null();
    sort_info.item_compare_partial = ::core::ptr::null_mut();
    sort_info.item_compare_selfdict = ::core::ptr::null_mut();

    // SAFETY: the builtin's argument array, which has at least two slots.
    let arg1 = unsafe { Tv::new(argvars.add(1)) };
    if arg1.v_type == VAR_UNKNOWN {
        return Ok(());
    }

    // optional second argument: {func}
    if arg1.v_type == VAR_FUNC {
        sort_info.item_compare_func = arg1.string();
    } else if arg1.v_type == VAR_PARTIAL {
        sort_info.item_compare_partial = arg1.partial();
    } else {
        let mut error = false;
        let nr = unsafe { tv_get_number_chk(argvars.add(1), &raw mut error) } as ::core::ffi::c_int;
        if error {
            return Err(Failed); // type error; errmsg already given
        }
        if nr == 1 {
            sort_info.item_compare_ic = 1;
        } else if arg1.v_type != VAR_NUMBER {
            let name = unsafe { how.string(argvars.add(1)) };
            sort_info.item_compare_func = name;
        } else if nr != 0 {
            emsg(gettext(e_invarg));
            return Err(Failed);
        }

        let how = sort_info.item_compare_func;
        if !how.is_null() {
            if unsafe { *how } as ::core::ffi::c_int == NUL {
                // empty string means default sort
                sort_info.item_compare_func = ::core::ptr::null();
            } else if unsafe { *how.add(1) } as ::core::ffi::c_int == NUL {
                // The five built-in orderings are one-character names;
                // upstream spells each as a `strcmp` against a literal.
                let mut builtin = true;
                match unsafe { *how } as u8 {
                    b'n' => sort_info.item_compare_numeric = true,
                    b'N' => sort_info.item_compare_numbers = true,
                    b'f' => sort_info.item_compare_float = true,
                    b'i' => sort_info.item_compare_ic = 1,
                    b'l' => sort_info.item_compare_lc = true,
                    _ => builtin = false,
                }
                if builtin {
                    unsafe { (*info).item_compare_func = ::core::ptr::null() };
                }
            }
        }
    }

    if unsafe { (*argvars.add(2)).v_type } != VAR_UNKNOWN {
        // optional third argument: {dict}
        if unsafe { tv_check_for_dict_arg(argvars, 2) }.is_err() {
            return Err(Failed);
        }
        unsafe { (*info).item_compare_selfdict = (*argvars.add(2)).vval.v_dict };
    }

    Ok(())
}

/// The body `sort()` and `uniq()` share: check the argument, publish a
/// `sortinfo`, and run the driver.
///
/// `sortinfo` is saved and restored around the call because a user comparison
/// function can itself call `sort()`.
pub(crate) unsafe fn do_sort_uniq(argvars: *mut typval_T, rettv: *mut typval_T, sort: bool) {
    let mut how = NumBuf::new();
    // SAFETY: the builtin's argument array.
    let args = unsafe { Tv::new(argvars) };
    if args.v_type != VAR_LIST {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg0 = unsafe {
            c_str(if sort {
                c"sort()".as_ptr()
            } else {
                c"uniq()".as_ptr()
            })
        };
        semsg!("E686: Argument of {arg0} must be a List");
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
    let l = args.list();
    if !unsafe { value_check_lock(tv_list_locked(l), arg_errmsg, TV_TRANSLATE as size_t) } {
        unsafe { tv_list_set_ret(rettv, l) };
        if unsafe { tv_list_len(l) } > 1
            && unsafe { parse_sort_uniq_args(argvars, &raw mut info, &mut how) }.is_ok()
        {
            if sort {
                unsafe { do_sort(l, &raw mut info) };
            } else {
                unsafe { do_uniq(l, &raw mut info) };
            }
        }
    }

    sortinfo.set(old_sortinfo);
}

/// `sort()`.
pub unsafe fn f_sort(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe { do_sort_uniq(argvars, rettv, true) };
}

/// `uniq()`.
pub unsafe fn f_uniq(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe { do_sort_uniq(argvars, rettv, false) };
}
