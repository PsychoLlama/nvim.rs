//! `sort()` and `uniq()`: the comparators and the two driver loops.
//!
//! [`item_compare`] is the default ordering and [`item_compare2`] the one
//! that calls a user function or dictionary method; each has the
//! `_keeping_zero` / `_not_keeping_zero` pair upstream hands to `qsort` so a
//! comparison error can stop the sort.  [`parse_sort_uniq_args`] reads the
//! optional `{how}` and `{dict}` arguments both builtins share.
//!
//! The four comparators keep `extern "C"` — `qsort_r` calls them — and the
//! sort keeps the C library's.  A
//! `sort_by` is not a provable substitute here: a user comparison function can
//! answer inconsistently (or fail part-way), so which permutation of equal
//! items comes out is whatever the C library's sort did, and Rust's sorts may
//! panic on an order that is not total.  The `_not_keeping_zero`
//! pair exists to make ties total by original index, but only the
//! `_keeping_zero` pair reaches `uniq`.  The sort's [`SortInfo`] travels as
//! `qsort_r`'s context argument; upstream's `qsort` had nowhere to put it and
//! published it in a global, saved and restored around a nested `sort()`.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::cstr;
use crate::message_fmt::c_str;
use crate::semsg;
use crate::types::{Failed, ListWatch, NUL};
use ::core::ffi::c_void;
use ::libc::qsort_r;

/// Compare two list items by the ordering `info` selected: numeric, float,
/// or a string comparison of their `string()` forms.
///
/// With `keep_zero` clear, ties are broken by the items' original indexes,
/// which is what makes the sort stable.
///
/// # Safety
///
/// `s1` and `s2` must point at the two `ListSortItem`s of the array
/// `do_sort`/`do_uniq` handed to `qsort_r`, live for the comparison, and
/// `info` at the `SortInfo` that sort set up.
pub(crate) unsafe fn item_compare(
    s1: *const c_void,
    s2: *const c_void,
    info: &SortInfo,
    keep_zero: bool,
) -> ::core::ffi::c_int {
    let si1 = s1 as *mut ListSortItem;
    let si2 = s2 as *mut ListSortItem;
    let tv1 = unsafe { &raw mut (*(*si1).item).li_tv };
    let tv2 = unsafe { &raw mut (*(*si2).item).li_tv };

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
    if info.item_compare_numbers {
        let v1 = unsafe { tv_get_number(&*tv1) };
        let v2 = unsafe { tv_get_number(&*tv2) };
        res = sign(v1 > v2, v1 == v2);
    } else if info.item_compare_float {
        let v1 = unsafe { tv_get_float(&*tv1) };
        let v2 = unsafe { tv_get_float(&*tv2) };
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
        if a.v_type() == VAR_STRING {
            if b.v_type() != VAR_STRING || info.item_compare_numeric {
                p1 = c"'".as_ptr().cast_mut();
            } else {
                p1 = a
                    .string_ref()
                    .map_or(::core::ptr::null_mut(), |s| s.as_ptr().cast_mut());
            }
        } else {
            p1 = unsafe { encode_tv2string(&*tv1, ::core::ptr::null_mut()) };
            tofree1 = p1;
        }
        if b.v_type() == VAR_STRING {
            if a.v_type() != VAR_STRING || info.item_compare_numeric {
                p2 = c"'".as_ptr().cast_mut();
            } else {
                p2 = b
                    .string_ref()
                    .map_or(::core::ptr::null_mut(), |s| s.as_ptr().cast_mut());
            }
        } else {
            p2 = unsafe { encode_tv2string(&*tv2, ::core::ptr::null_mut()) };
            tofree2 = p2;
        }
        if p1.is_null() {
            p1 = c"".as_ptr().cast_mut();
        }
        if p2.is_null() {
            p2 = c"".as_ptr().cast_mut();
        }

        if !info.item_compare_numeric {
            res = if info.item_compare_lc {
                unsafe { strcoll(p1, p2) }
            } else if info.item_compare_ic != 0 {
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
///
/// # Safety
///
/// As [`item_compare`].
pub(crate) unsafe extern "C" fn item_compare_keeping_zero(
    s1: *const c_void,
    s2: *const c_void,
    info: *mut c_void,
) -> ::core::ffi::c_int {
    unsafe { item_compare(s1, s2, &*info.cast::<SortInfo>(), true) }
}

/// [`item_compare`] breaking ties by index — `sort`'s comparator.
///
/// # Safety
///
/// As [`item_compare`].
pub(crate) unsafe extern "C" fn item_compare_not_keeping_zero(
    s1: *const c_void,
    s2: *const c_void,
    info: *mut c_void,
) -> ::core::ffi::c_int {
    unsafe { item_compare(s1, s2, &*info.cast::<SortInfo>(), false) }
}

/// Compare two list items by calling the user function `info` holds.
///
/// A failed call sets `item_compare_func_err`, which makes every later
/// comparison answer 0 and the driver abandon the sort.
///
/// # Safety
///
/// `s1` and `s2` must point at the two `ListSortItem`s of the array
/// `do_sort`/`do_uniq` handed to `qsort_r`, live for the comparison, and
/// `info` at the `SortInfo` that sort set up.
pub(crate) unsafe fn item_compare2(
    s1: *const c_void,
    s2: *const c_void,
    info: &mut SortInfo,
    keep_zero: bool,
) -> ::core::ffi::c_int {
    let partial = info.item_compare_partial;

    // shortcut after failure in previous call; compare all items equal
    if info.item_compare_func_err {
        return 0;
    }

    let si1 = s1 as *mut ListSortItem;
    let si2 = s2 as *mut ListSortItem;
    let func_name = if partial.is_null() {
        info.item_compare_func
    } else {
        unsafe { partial_name(partial) }
    };

    // Copy the values.  This is needed to be able to set v_lock to
    // VarLock::Fixed in the copy without changing the original list items.
    let mut argv = [TV_INITIAL_VALUE; 2];
    unsafe { tv_copy(&(*(*si1).item).li_tv, &mut argv[0]) };
    unsafe { tv_copy(&(*(*si2).item).li_tv, &mut argv[1]) };

    let mut rettv = TV_INITIAL_VALUE;
    let mut funcexe = FUNCEXE_INIT;
    funcexe.fe_evaluate = true;
    funcexe.fe_partial = partial;
    funcexe.fe_selfdict = info.item_compare_selfdict;
    let called = unsafe { call_func(func_name, -1, &mut rettv, &argv, &raw mut funcexe) };
    drop(argv);

    let mut res;
    if called.is_err() {
        res = ITEM_COMPARE_FAIL;
        info.item_compare_func_err = true;
    } else {
        let n = tv_get_number_chk(&rettv).unwrap_or_else(|_| {
            // SAFETY: the sort's own record, live for the comparison.
            info.item_compare_func_err = true;
            0
        });
        res = if n > 0 {
            1
        } else if n < 0 {
            -1
        } else {
            0
        };
    }
    if info.item_compare_func_err {
        res = ITEM_COMPARE_FAIL; // return value has wrong type
    }
    tv_clear(&mut rettv);

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
///
/// # Safety
///
/// As [`item_compare2`].
pub(crate) unsafe extern "C" fn item_compare2_keeping_zero(
    s1: *const c_void,
    s2: *const c_void,
    info: *mut c_void,
) -> ::core::ffi::c_int {
    unsafe { item_compare2(s1, s2, &mut *info.cast::<SortInfo>(), true) }
}

/// [`item_compare2`] breaking ties by index — `sort`'s comparator.
///
/// # Safety
///
/// As [`item_compare2`].
pub(crate) unsafe extern "C" fn item_compare2_not_keeping_zero(
    s1: *const c_void,
    s2: *const c_void,
    info: *mut c_void,
) -> ::core::ffi::c_int {
    unsafe { item_compare2(s1, s2, &mut *info.cast::<SortInfo>(), false) }
}

/// Which comparator `info` selects: the built-in ordering, or the user
/// function.
fn sorter(info: &SortInfo, keep_zero: bool) -> ListSorter {
    let builtin = info.item_compare_func.is_null() && info.item_compare_partial.is_null();
    Some(match (builtin, keep_zero) {
        (true, true) => {
            item_compare_keeping_zero
                as unsafe extern "C" fn(
                    *const c_void,
                    *const c_void,
                    *mut c_void,
                ) -> ::core::ffi::c_int
        }
        (true, false) => item_compare_not_keeping_zero,
        (false, true) => item_compare2_keeping_zero,
        (false, false) => item_compare2_not_keeping_zero,
    })
}

/// The record a comparator reads for one list item, at its position in the
/// list.  Only a `_not_keeping_zero` comparator reads `idx`.
fn sort_item(item: *mut ListItem, idx: ::core::ffi::c_int) -> ListSortItem {
    ListSortItem { item, idx }
}

/// `sort()` over `l`, in place.
///
/// The items are **taken out of the list** for the duration: `qsort` permutes
/// an array of pointers into them, and a user comparator that re-enters the
/// evaluator must not be able to move them.  Upstream sorted the links in
/// place and left a comparator that edited the list holding freed items;
/// what a comparator sees here is an empty list instead, and anything it
/// appends is dropped when the sorted items go back (upstream leaked them).
///
/// # Safety
///
/// `l` must point at a live list, unaliased for the call.
pub(crate) unsafe fn do_sort(l: *mut List, info: &mut SortInfo) {
    // SAFETY: the caller's promise: a live list.
    let mut taken = ::core::mem::take(unsafe { &mut (*l).lv_items });
    let len = taken.len();

    // Make an array with each entry pointing to an item.  Every pointer is
    // an offset from the *one* derivation of the array below, so nothing
    // here retags an element out from under the comparator.
    let base = taken.as_mut_ptr();
    let mut ptrs: Vec<ListSortItem> = (0..len)
        // SAFETY: `i` is inside the array `base` names.
        .map(|i| sort_item(unsafe { base.add(i) }, index_of(i)))
        .collect();

    info.item_compare_func_err = false;
    let item_compare_func = sorter(info, false);

    // Sort the array with item pointers.
    let itemsize = ::core::mem::size_of::<ListSortItem>();
    // SAFETY: `ptrs` holds `len` records of exactly `itemsize`, and the
    // comparator reads two of them and the caller's `info`.
    unsafe {
        qsort_r(
            ptrs.as_mut_ptr().cast(),
            len as size_t,
            itemsize,
            item_compare_func,
            ::core::ptr::from_mut(info).cast(),
        );
    };

    if info.item_compare_func_err {
        emsg(gettext(c"E702: Sort compare function failed"));
        // The list is left as it was.
        unsafe { (*l).lv_items = taken };
        return;
    }

    // Put the items back in the sorted order.  Each pointer in `ptrs` names
    // a different item of `taken`, so every item moves out exactly once.
    let mut sorted: Vec<ListItem> = Vec::with_capacity(len);
    for entry in &ptrs {
        // SAFETY: a distinct item of `taken`, moved out once.
        sorted.push(unsafe { entry.item.read() });
    }
    // Every item has moved out; the array is only its allocation now.
    // SAFETY: `len` items were read out of it, and none is left to drop.
    unsafe { taken.set_len(0) };
    // A cursor stands on an item, not on a place, so it goes where the item
    // went.  Only paid for when something is actually walking the list.
    // SAFETY: the caller's promise: a live list.
    let list = unsafe { &mut *l };
    if !list.lv_watch.is_null() {
        let mut moved = vec![ListWatch::ENDED; len];
        for (dest, entry) in ptrs.iter().enumerate() {
            // SAFETY: every entry points at an item of `taken`, whose base
            // is `base`.
            let from = unsafe { entry.item.offset_from(base) };
            moved[from.cast_unsigned()] = index_of(dest);
        }
        watch_permute(list, &moved);
    }
    // Anything the comparator appended is dropped with the empty list it
    // went into.
    unsafe { (*l).lv_items = sorted };
}

/// `uniq()` over `l`, in place: drop each item equal to the one before it.
///
/// # Safety
///
/// `l` must point at a live list, unaliased for the call.
pub(crate) unsafe fn do_uniq(l: *mut List, info: &mut SortInfo) {
    info.item_compare_func_err = false;
    let compare = sorter(info, true).expect("non-null function pointer");

    let mut at = 1;
    // Re-read the length every step: the comparator runs a user function,
    // which may edit the list.
    // SAFETY: the caller's promise: a live list.
    while at < list_items(unsafe { l.as_ref() }).len() {
        // Upstream hands the comparator the addresses of two bare
        // `ListItem *` locals and lets it read them as `ListSortItem *`,
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
        // SAFETY: two items of the list, read out afresh each step.
        let items = list_items_mut(unsafe { l.as_mut() });
        let prev = sort_item(&raw mut items[at - 1], 0);
        let cur = sort_item(&raw mut items[at], 1);
        // SAFETY: the two records just built.
        let (prev, cur) = ((&raw const prev).cast(), (&raw const cur).cast());
        let equal = unsafe { compare(prev, cur, ::core::ptr::from_mut(info).cast()) } == 0;
        if equal {
            // SAFETY: a live list and an index of it.
            unsafe { (*l).remove_range(at, at) };
        } else {
            at += 1;
        }
        if info.item_compare_func_err {
            emsg(gettext(c"E882: Uniq compare function failed"));
            break;
        }
    }
}

/// Read `sort()`/`uniq()`'s optional `{how}` and `{dict}` arguments into
/// `info`.
///
/// A `{how}` given as a Number has no string of its own, so the caller lends
/// `how` for it: `info.item_compare_func` may borrow it, and the sort
/// reads that field long after this returns.
pub(crate) fn parse_sort_uniq_args(
    args: &[TypVal],
    info: &mut SortInfo,
    how: &mut NumBuf,
) -> Result<(), Failed> {
    info.item_compare_ic = 0;
    info.item_compare_lc = false;
    info.item_compare_numeric = false;
    info.item_compare_numbers = false;
    info.item_compare_float = false;
    info.item_compare_func = ::core::ptr::null();
    info.item_compare_partial = ::core::ptr::null_mut();
    info.item_compare_selfdict = ::core::ptr::null_mut();

    let Some(arg1) = args.get(1) else {
        return Ok(());
    };

    // optional second argument: {func}
    if arg1.v_type() == VAR_FUNC {
        info.item_compare_func = arg1
            .func_name()
            .map_or(::core::ptr::null(), |name| name.as_ptr());
    } else if arg1.v_type() == VAR_PARTIAL {
        info.item_compare_partial = arg1.partial_or_null();
    } else {
        let Ok(nr) = tv_get_number_chk(&args[1]) else {
            return Err(Failed); // type error; errmsg already given
        };
        let nr = nr as ::core::ffi::c_int;
        if nr == 1 {
            info.item_compare_ic = 1;
        } else if arg1.v_type() != VAR_NUMBER {
            info.item_compare_func = how.string(&args[1]).as_ptr();
        } else if nr != 0 {
            emsg(gettext(e_invarg));
            return Err(Failed);
        }

        let how = info.item_compare_func;
        // SAFETY: a String argument's text or `how`'s, both NUL-terminated
        // and live for the call; only the first two bytes are read, the
        // second only past a non-NUL first.
        let (first, second) = (!how.is_null())
            .then(|| unsafe {
                let first = *how as u8;
                (first, if first == 0 { 0 } else { *how.add(1) as u8 })
            })
            .unzip();
        if let (Some(first), Some(second)) = (first, second) {
            if first == NUL as u8 {
                // empty string means default sort
                info.item_compare_func = ::core::ptr::null();
            } else if second == NUL as u8 {
                // The five built-in orderings are one-character names;
                // upstream spells each as a `strcmp` against a literal.
                let mut builtin = true;
                match first {
                    b'n' => info.item_compare_numeric = true,
                    b'N' => info.item_compare_numbers = true,
                    b'f' => info.item_compare_float = true,
                    b'i' => info.item_compare_ic = 1,
                    b'l' => info.item_compare_lc = true,
                    _ => builtin = false,
                }
                if builtin {
                    info.item_compare_func = ::core::ptr::null();
                }
            }
        }
    }

    if args.len() > 2 {
        // optional third argument: {dict}
        tv_check_for_dict_arg(args, 2)?;
        info.item_compare_selfdict = args[2].dict_or_null();
    }

    Ok(())
}

/// The body `sort()` and `uniq()` share: check the argument, read the
/// ordering, and run the driver.
///
/// The ordering is this call's local: a user comparison function can itself
/// call `sort()`, which gets its own.
pub(crate) fn do_sort_uniq(args: &[TypVal], result: &mut TypVal, sort: bool) {
    let mut how = NumBuf::new();
    // SAFETY: the builtin's argument array.
    let first = unsafe { Tv::new(core::ptr::from_ref(&args[0]).cast_mut()) };
    if first.v_type() != VAR_LIST {
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

    let arg_errmsg = if sort {
        c"sort() argument".as_ptr()
    } else {
        c"uniq() argument".as_ptr()
    };
    let l = first.list_or_null();
    if !unsafe { value_check_lock(list_locked(l.as_ref()), arg_errmsg, TV_TRANSLATE as size_t) } {
        // SAFETY: the argument's own list, whose reference the answer
        // takes a second one of.
        result.write_list(unsafe { ListRef::retained(l) });
        if list_len(unsafe { l.as_ref() }) > 1
            && parse_sort_uniq_args(args, &mut info, &mut how).is_ok()
        {
            if sort {
                unsafe { do_sort(l, &mut info) };
            } else {
                unsafe { do_uniq(l, &mut info) };
            }
        }
    }
}

/// `sort()`.
pub fn f_sort(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    do_sort_uniq(args, result, true);
}

/// `uniq()`.
pub fn f_uniq(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    do_sort_uniq(args, result, false);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::list::string_tv;
    use crate::global_cell::editor_state_lock;
    use crate::types::EvalFuncData;

    use V::{F, N, Str};

    /// One item of a test list, as the case wrote it.
    #[derive(Clone, Debug, PartialEq)]
    enum V {
        N(VarNumber),
        Str(String),
        F(f64),
    }

    /// A string item.
    #[allow(non_snake_case)]
    fn S(text: &str) -> V {
        Str(text.to_owned())
    }

    impl V {
        fn tv(&self) -> TypVal {
            match self {
                N(n) => TypVal::Number(*n),
                Str(s) => string_tv(s.as_bytes()),
                F(f) => TypVal::Float(*f),
            }
        }

        fn of(tv: &TypVal) -> V {
            match tv {
                TypVal::Number(n) => N(*n),
                TypVal::Float(f) => F(*f),
                _ => {
                    let text = tv.string_ref().map(|s| s.to_bytes().to_vec());
                    Str(String::from_utf8(text.unwrap_or_default()).expect("ASCII"))
                }
            }
        }
    }

    fn list_of(items: &[V]) -> ListRef {
        let mut l = tv_list_alloc(ptrdiff_t::try_from(items.len()).expect("short"));
        for v in items {
            let mut tv = v.tv();
            l.push_copy(&tv);
            tv_clear(&mut tv);
        }
        l
    }

    fn contents(l: &ListRef) -> Vec<V> {
        l.items().iter().map(|li| V::of(&li.li_tv)).collect()
    }

    /// `sort(l, how)` or `uniq(l, how)` in place, through the builtin.
    fn run(l: &ListRef, how: Option<V>, sort: bool) {
        let mut argv = vec![TypVal::list(Some(l.clone()))];
        if let Some(how) = how {
            argv.push(how.tv());
        }
        let mut result = TypVal::Number(0);
        if sort {
            f_sort(&argv, &mut result, EvalFuncData::None);
        } else {
            f_uniq(&argv, &mut result, EvalFuncData::None);
        }
        assert_eq!(
            result.list_or_null(),
            l.as_ptr(),
            "the answer is the list itself"
        );
        tv_clear(&mut result);
        for tv in &mut argv {
            tv_clear(tv);
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "qsort_r is a foreign function")]
    fn sorting_numbers_by_the_default_and_the_numeric_orders() {
        let _serial = editor_state_lock();
        let l = list_of(&[N(10), N(9), N(-2), N(100)]);
        // The default compares string forms.
        run(&l, None, true);
        assert_eq!(contents(&l), [N(-2), N(10), N(100), N(9)]);
        run(&l, Some(S("N")), true);
        assert_eq!(contents(&l), [N(-2), N(9), N(10), N(100)]);
        let l = list_of(&[S("10"), S("9"), N(3), S("x")]);
        run(&l, Some(S("N")), true);
        assert_eq!(contents(&l), [S("x"), N(3), S("9"), S("10")]);
        // An `n` sort orders numbers and floats; every string is `'`.
        run(&l, Some(S("n")), true);
        assert_eq!(contents(&l), [S("x"), S("9"), S("10"), N(3)]);
    }

    #[test]
    #[cfg_attr(miri, ignore = "qsort_r is a foreign function")]
    fn sorting_floats_and_ignoring_case() {
        let _serial = editor_state_lock();
        let l = list_of(&[F(2.5), N(1), F(-0.5), F(1e10)]);
        run(&l, Some(S("f")), true);
        assert_eq!(contents(&l), [F(-0.5), N(1), F(2.5), F(1e10)]);
        let l = list_of(&[S("b"), S("A"), S("a"), S("B")]);
        run(&l, Some(S("i")), true);
        assert_eq!(contents(&l), [S("A"), S("a"), S("b"), S("B")]);
        run(&l, Some(N(1)), true);
        assert_eq!(contents(&l), [S("A"), S("a"), S("b"), S("B")]);
        run(&l, None, true);
        assert_eq!(contents(&l), [S("A"), S("B"), S("a"), S("b")]);
    }

    /// A `:for` cursor on a list being sorted stands on an *item*: it goes
    /// where the item went.
    #[test]
    #[cfg_attr(miri, ignore = "qsort_r is a foreign function")]
    fn a_watcher_follows_its_item_through_a_sort() {
        let _serial = editor_state_lock();
        let mut l = list_of(&[N(3), N(0), N(2), N(1)]);
        let mut on_two = Box::new(ListWatch {
            lw_index: 2,
            lw_next: ::core::ptr::null_mut(),
        });
        let mut on_three = Box::new(ListWatch {
            lw_index: 0,
            lw_next: ::core::ptr::null_mut(),
        });
        let (two, three) = (&raw mut *on_two, &raw mut *on_three);
        // SAFETY: both boxes outlive their registration, ended below.
        unsafe {
            l.watch_add(two);
            l.watch_add(three);
        }
        run(&l, Some(S("N")), true);
        assert_eq!(contents(&l), [N(0), N(1), N(2), N(3)]);
        // SAFETY: the two watchers, still registered.
        assert_eq!(unsafe { ((*two).lw_index, (*three).lw_index) }, (2, 3));
        // An ended cursor stays ended.
        // SAFETY: as above.
        unsafe { (*three).lw_index = ListWatch::ENDED };
        run(&l, None, true);
        // SAFETY: as above.
        assert_eq!(unsafe { (*three).lw_index }, ListWatch::ENDED);
        // SAFETY: as above.
        unsafe {
            l.watch_remove(two);
            l.watch_remove(three);
        }
        drop(on_two);
        drop(on_three);
    }

    #[test]
    fn uniq_drops_each_item_equal_to_the_one_before() {
        let _serial = editor_state_lock();
        let l = list_of(&[N(1), N(1), N(2), N(1), N(1), N(3), N(3)]);
        run(&l, Some(S("N")), false);
        assert_eq!(contents(&l), [N(1), N(2), N(1), N(3)]);
        // Strings by their bytes.
        let l = list_of(&[S("a"), S("a"), S("A"), S("b"), S("b")]);
        run(&l, None, false);
        assert_eq!(contents(&l), [S("a"), S("A"), S("b")]);
        // By number, 1 and '1' and '01' are one.  (A Float here would be
        // E805, which needs an execution stack to report against.)
        let l = list_of(&[N(1), S("1"), S("1"), S("01")]);
        run(&l, Some(S("N")), false);
        assert_eq!(contents(&l), [N(1)]);
        let l = list_of(&[F(1.0), N(1), F(1.5)]);
        run(&l, Some(S("f")), false);
        assert_eq!(contents(&l), [F(1.0), F(1.5)]);
    }

    /// The default ordering compares `string()` forms, so a number and the
    /// string of its digits differ: one is quoted.
    ///
    /// That comparison is `encode_tv2string(&TypVal)`, whose walk
    /// (`encode_typval_read`) casts the shared borrow to `*mut` and hands
    /// each hook `tv.as_mut()` -- a `&mut` retag of a `SharedReadOnly`
    /// pointer, which Stacked Borrows rejects before anything is written.
    /// Every `string()` of a non-string value through that entry point
    /// does the same.
    #[test]
    #[cfg_attr(
        miri,
        ignore = "UB in current code: encode_typval_read retags a &TypVal as &mut \
                  (Stacked Borrows: Unique retag of a SharedReadOnly tag)"
    )]
    fn uniq_by_string_form_tells_a_number_from_its_digits() {
        let _serial = editor_state_lock();
        let l = list_of(&[N(1), N(1), S("1"), S("1"), S("01")]);
        run(&l, None, false);
        assert_eq!(contents(&l), [N(1), S("1"), S("01")]);
    }

    #[test]
    #[cfg_attr(miri, ignore = "strcasecmp is a foreign function")]
    fn uniq_ignoring_case() {
        let _serial = editor_state_lock();
        let l = list_of(&[S("a"), S("A"), S("b"), S("B"), S("a")]);
        run(&l, Some(S("i")), false);
        assert_eq!(contents(&l), [S("a"), S("b"), S("a")]);
    }

    /// A watcher on an item `uniq` removes lands on the next one, as for any
    /// removal.
    #[test]
    fn a_watcher_on_a_duplicate_lands_on_what_followed_it() {
        let _serial = editor_state_lock();
        let mut l = list_of(&[N(1), N(1), N(2), N(2), N(3)]);
        let mut lw = Box::new(ListWatch {
            lw_index: 3,
            lw_next: ::core::ptr::null_mut(),
        });
        let w = &raw mut *lw;
        // SAFETY: the box outlives its registration, ended below.
        unsafe { l.watch_add(w) };
        run(&l, Some(S("N")), false);
        assert_eq!(contents(&l), [N(1), N(2), N(3)]);
        // SAFETY: the watcher, still registered.
        assert_eq!(unsafe { (*w).lw_index }, 2);
        // SAFETY: as above.
        unsafe { l.watch_remove(w) };
        drop(lw);
    }

    /// A list of one, or none, is answered without parsing `{how}`: even a
    /// nonsense ordering is no error then.
    #[test]
    fn a_short_list_is_answered_untouched() {
        let _serial = editor_state_lock();
        for items in [vec![], vec![N(5)]] {
            let l = list_of(&items);
            run(&l, Some(S("no-such-function")), true);
            run(&l, Some(S("no-such-function")), false);
            assert_eq!(contents(&l), items);
        }
    }
}
