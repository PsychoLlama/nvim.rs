//! Ranges over a list: slicing, assigning through a slice, flattening, joining.
//!
//! [`tv_list_check_range_index_one`] and [`tv_list_check_range_index_two`]
//! are the bounds arithmetic `l[i:j]` shares with `l[i:j] = x`;
//! [`tv_list_slice_or_index`] is the subscript itself.
//! [`tv_list_join`] and [`list_join_inner`] are `join()`, which makes two
//! passes so the result buffer is sized once, and [`f_list2str`] is the
//! codepoint-list-to-string builtin.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::message::emsg_ptr;
use crate::semsg;
use crate::types::Failed;
use crate::types::NUL;

/// Resolve the first index of `l[n1:n2]`, clamping a negative one that fell
/// off the front and raising `E684` when there is no such item.
///
/// `*n1` is updated to the index actually used.
pub unsafe fn tv_list_check_range_index_one(
    l: *mut list_T,
    n1: *mut ::core::ffi::c_int,
    quiet: bool,
) -> *mut listitem_T {
    let li = unsafe { tv_list_find_index(l, n1) };
    if li.is_null() && !quiet {
        // SAFETY: the caller's index cell.
        let at = int64_t::from(unsafe { *n1 });
        semsg!("E684: List index out of range: {at}");
    }
    li
}

/// Resolve the second index of `l[n1:n2]` against the item `li1` the first one
/// landed on, normalising both to non-negative indexes.
pub unsafe fn tv_list_check_range_index_two(
    l: *mut list_T,
    n1: *mut ::core::ffi::c_int,
    li1: *const listitem_T,
    n2: *mut ::core::ffi::c_int,
    quiet: bool,
) -> Result<(), Failed> {
    if unsafe { *n2 } < 0 {
        let ni = unsafe { tv_list_find(l, *n2) };
        if ni.is_null() {
            if !quiet {
                // SAFETY: the caller's index cell.
                let at = int64_t::from(unsafe { *n2 });
                semsg!("E684: List index out of range: {at}");
            }
            return Err(Failed);
        }
        unsafe { *n2 = tv_list_idx_of_item(l, ni) };
    }
    if unsafe { *n1 } < 0 {
        unsafe { *n1 = tv_list_idx_of_item(l, li1) };
    }
    if unsafe { *n2 } < unsafe { *n1 } {
        if !quiet {
            // SAFETY: the caller's index cell.
            let at = int64_t::from(unsafe { *n2 });
            semsg!("E684: List index out of range: {at}");
        }
        return Err(Failed);
    }
    Ok(())
}

/// `dest[idx1:idx2] = src`, or `dest[idx1:idx2] op= src` when `op` is given.
///
/// `empty_idx2` means the range had no upper bound (`dest[idx1:]`).
pub unsafe fn tv_list_assign_range(
    dest: *mut list_T,
    src: *mut list_T,
    idx1_arg: ::core::ffi::c_int,
    idx2: ::core::ffi::c_int,
    empty_idx2: bool,
    op: *const ::core::ffi::c_char,
    varname: *const ::core::ffi::c_char,
) -> Result<(), Failed> {
    let mut idx1 = idx1_arg;
    let first_li = unsafe { tv_list_find_index(dest, &raw mut idx1) };

    // Check whether any of the list items is locked before making any
    // changes.
    let mut idx = idx1;
    let mut dest_li = first_li;
    let mut src_li = unsafe { tv_list_first(src) };
    while !src_li.is_null() && !dest_li.is_null() {
        if unsafe { value_check_lock((*dest_li).li_tv.v_lock, varname, TV_CSTRING as size_t) } {
            return Err(Failed);
        }
        src_li = unsafe { (*src_li).li_next };
        if src_li.is_null() || (!empty_idx2 && idx2 == idx) {
            break;
        }
        dest_li = unsafe { (*dest_li).li_next };
        idx += 1;
    }

    // Assign the List values to the list items.
    idx = idx1;
    dest_li = first_li;
    src_li = unsafe { tv_list_first(src) };
    while !src_li.is_null() {
        debug_assert!(!dest_li.is_null());
        if !op.is_null() && unsafe { *op } as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
            let _ = unsafe { eexe_mod_op(&raw mut (*dest_li).li_tv, &raw mut (*src_li).li_tv, op) };
        } else {
            unsafe { tv_clear(&raw mut (*dest_li).li_tv) };
            unsafe { tv_copy(&raw mut (*src_li).li_tv, &raw mut (*dest_li).li_tv) };
        }
        src_li = unsafe { (*src_li).li_next };
        if src_li.is_null() || (!empty_idx2 && idx2 == idx) {
            break;
        }
        if unsafe { (*dest_li).li_next }.is_null() {
            // Need to add an empty item.
            unsafe { tv_list_append_number(dest, 0) };
            // "dest_li" may have become invalid after append, don't use it.
            dest_li = unsafe { tv_list_last(dest) }; // Valid again.
        } else {
            dest_li = unsafe { (*dest_li).li_next };
        }
        idx += 1;
    }

    if !src_li.is_null() {
        let msg = tr(c"E710: List value has more items than target");
        unsafe { emsg_ptr(msg) };
        return Err(Failed);
    }
    let short = if empty_idx2 {
        !dest_li.is_null() && !unsafe { (*dest_li).li_next }.is_null()
    } else {
        idx != idx2
    };
    if short {
        emsg(gettext(c"E711: List value has not enough items"));
        return Err(Failed);
    }
    Ok(())
}

/// `flatten()`: splice the items of any nested list into `list` in place,
/// starting at `first` and going `maxdepth` levels down.
pub unsafe fn tv_list_flatten(
    list: *mut list_T,
    first: *mut listitem_T,
    maxitems: int64_t,
    maxdepth: int64_t,
) {
    if maxdepth == 0 {
        return;
    }

    let mut item = if first.is_null() {
        // SAFETY: the caller's promise: a live list.
        let flat = unsafe { Ls::new(list) };
        flat.lv_first
    } else {
        first
    };
    let mut done = 0;
    while !item.is_null() && done < maxitems {
        // The link is read before the body, which unlinks and frees `item`.
        // SAFETY: an item of the list being flattened.
        let flat = unsafe { Li::new(item) };
        let next = flat.li_next;

        fast_breakcheck();
        if got_int.get() {
            return;
        }
        if flat.li_tv.v_type == VAR_LIST {
            let itemlist = flat.list();

            unsafe { tv_list_drop_items(list, item, item) };
            unsafe { tv_list_extend(list, itemlist, next) };

            if maxdepth > 0 {
                let spliced_first = if flat.li_prev.is_null() {
                    // SAFETY: the caller's promise: a live list.
                    unsafe { Ls::new(list) }.lv_first
                } else {
                    unsafe { (*(*item).li_prev).li_next }
                };
                let n = int64_t::from(unsafe { (*itemlist).lv_len });
                unsafe { tv_list_flatten(list, spliced_first, n, maxdepth - 1) };
            }
            unsafe { tv_clear(&raw mut (*item).li_tv) };
            unsafe { xfree(item.cast()) };
        }

        done += 1;
        item = next;
    }
}

/// A fresh list holding copies of `ol[n1..=n2]`.
pub(crate) unsafe fn tv_list_slice(
    ol: *mut list_T,
    mut n1: varnumber_T,
    n2: varnumber_T,
) -> *mut list_T {
    let l = unsafe { tv_list_alloc((n2 - n1 + 1) as ptrdiff_t) };
    let mut item = unsafe { tv_list_find(ol, n1 as ::core::ffi::c_int) };
    while n1 <= n2 {
        unsafe { tv_list_append_tv(l, &raw mut (*item).li_tv) };
        item = unsafe { (*item).li_next };
        n1 += 1;
    }
    l
}

/// `list[n1]` or `list[n1 : n2]`, whichever `range` says.
///
/// `rettv` holds the list being subscripted on the way in.  An index out of
/// range is an error; a *range* out of range is merely empty.
pub unsafe fn tv_list_slice_or_index(
    _list: *mut list_T,
    range: bool,
    n1_arg: varnumber_T,
    n2_arg: varnumber_T,
    exclusive: bool,
    rettv: *mut typval_T,
    verbose: bool,
) -> Result<(), Failed> {
    let len = unsafe { tv_list_len((*rettv).vval.v_list) };
    let mut n1 = n1_arg;
    let mut n2 = n2_arg;

    if n1 < 0 {
        n1 += varnumber_T::from(len);
    }
    if n1 < 0 || n1 >= varnumber_T::from(len) {
        // For a range we allow invalid values and return an empty list.
        // A list index out of range is an error.
        if !range {
            if verbose {
                semsg!("E684: List index out of range: {}", n1_arg);
            }
            return Err(Failed);
        }
        n1 = varnumber_T::from(len);
    }

    if range {
        if n2 < 0 {
            n2 += varnumber_T::from(len);
        } else if n2 >= varnumber_T::from(len) {
            n2 = varnumber_T::from(len - if exclusive { 0 } else { 1 });
        }
        if exclusive {
            n2 -= 1;
        }
        if n2 < 0 || n2 + 1 < n1 {
            n2 = -1;
        }
        let l = unsafe { tv_list_slice((*rettv).vval.v_list, n1, n2) };
        unsafe { tv_clear(rettv) };
        unsafe { tv_list_set_ret(rettv, l) };
    } else {
        // copy the item to "var1" to avoid that freeing the list makes it
        // invalid.
        let mut var1 = TV_INITIAL_VALUE;
        let li = unsafe { tv_list_find((*rettv).vval.v_list, n1 as ::core::ffi::c_int) };
        unsafe { tv_copy(&raw mut (*li).li_tv, &raw mut var1) };
        unsafe { tv_clear(rettv) };
        unsafe { *rettv = var1 };
    }
    Ok(())
}

/// `join()`'s two passes: stringify every item into `join_gap`, then
/// concatenate them into `gap` with `sep` between.
///
/// Splitting it in two is what lets `gap` be grown to its final size once.
pub(crate) unsafe fn list_join_inner(
    gap: *mut garray_T,
    l: *mut list_T,
    sep: *const ::core::ffi::c_char,
    join_gap: *mut garray_T,
) -> Result<(), Failed> {
    let mut sumlen: size_t = 0;
    let mut first = true;

    // Stringify each item in the list.
    for item in tv_list_iter(unsafe { l.as_ref() }) {
        if got_int.get() {
            break;
        }
        let mut s = String_0::NULL;
        let data = unsafe { encode_tv2echo(&raw mut (*item).li_tv, s.len_mut()) };
        s.set_data(data);
        if s.data().is_null() {
            return Err(Failed);
        }

        sumlen += s.len();

        let p = unsafe { ga_append_via_ptr(join_gap, ::core::mem::size_of::<Join>()) } as *mut Join;
        // SAFETY: the entry `ga_append_via_ptr` just made room for.
        let mut joined = unsafe { Live::<Join>::new(p) };
        joined.s = s;
        joined.tofree = s.data();

        line_breakcheck();
    }

    // Allocate result buffer with its total size, avoid re-allocation and
    // multiple copy operations.  Add 2 for a tailing ']' and NUL.
    let seplen = unsafe { strlen(sep) };
    // SAFETY: the caller's stack garray.
    let ga = unsafe { Ga::new(join_gap) };
    if ga.ga_len >= 2 {
        sumlen += seplen * (ga.ga_len - 1) as size_t;
    }
    unsafe { ga_grow(gap, sumlen as ::core::ffi::c_int + 2) };

    let mut i = 0;
    while i < ga.ga_len && !got_int.get() {
        if first {
            first = false;
        } else {
            unsafe { ga_concat_len(gap, sep, seplen) };
        }
        let p = unsafe { (ga.ga_data as *const Join).offset(i as isize) };
        if !unsafe { (*p).s }.data().is_null() {
            unsafe { ga_concat_len(gap, (*p).s.data(), (*p).s.len()) };
        }
        line_breakcheck();
        i += 1;
    }

    Ok(())
}

/// `join()`: append `l`'s items to `gap`, separated by `sep`.
pub unsafe fn tv_list_join(
    gap: *mut garray_T,
    l: *mut list_T,
    sep: *const ::core::ffi::c_char,
) -> Result<(), Failed> {
    if unsafe { tv_list_len(l) } == 0 {
        return Ok(());
    }

    let mut join_ga = GARRAY_EMPTY;
    let itemsize = ::core::mem::size_of::<Join>() as ::core::ffi::c_int;
    let growsize = unsafe { tv_list_len(l) };
    unsafe { ga_init(&raw mut join_ga, itemsize, growsize) };
    let retval = unsafe { list_join_inner(gap, l, sep, &raw mut join_ga) };

    // GA_DEEP_CLEAR with FREE_JOIN_TOFREE.
    if !join_ga.ga_data.is_null() {
        for i in 0..join_ga.ga_len {
            let joined = join_ga.ga_data as *mut Join;
            unsafe { xfree((*joined.offset(i as isize)).tofree.cast()) };
        }
    }
    unsafe { ga_clear(&raw mut join_ga) };

    retval
}

/// `join()` the builtin.
pub unsafe fn f_join(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    if unsafe { (*argvars).v_type } != VAR_LIST {
        emsg(gettext(e_listreq));
        return;
    }
    let sep = if unsafe { (*argvars.add(1)).v_type } == VAR_UNKNOWN {
        c" ".as_ptr()
    } else {
        unsafe { numbuf.string_chk(argvars.add(1)) }
    };

    unsafe { (*rettv).v_type = VAR_STRING };
    if sep.is_null() {
        unsafe { (*rettv).vval.v_string = ::core::ptr::null_mut() };
        return;
    }

    let mut ga = GARRAY_EMPTY;
    let itemsize = ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int;
    unsafe { ga_init(&raw mut ga, itemsize, 80) };
    let _ = unsafe { tv_list_join(&raw mut ga, (*argvars).vval.v_list, sep) };
    unsafe { ga_append(&raw mut ga, NUL as uint8_t) };
    unsafe { (*rettv).vval.v_string = ga.ga_data as *mut ::core::ffi::c_char };
}

/// `list2str()`: a list of codepoints as a string.
pub unsafe fn f_list2str(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe { (*rettv).v_type = VAR_STRING };
    unsafe { (*rettv).vval.v_string = ::core::ptr::null_mut() };
    // SAFETY: the builtin's argument array.
    let args = unsafe { Tv::new(argvars) };
    if args.v_type != VAR_LIST {
        emsg(gettext(e_invarg));
        return;
    }
    let l = args.list();
    if l.is_null() {
        return;
    }

    let mut ga = GARRAY_EMPTY;
    unsafe { ga_init(&raw mut ga, 1, 80) };
    let mut buf: [::core::ffi::c_char; 22] = [0; 22];
    for li in tv_list_iter(unsafe { l.as_ref() }) {
        let n = unsafe { tv_get_number(&raw const (*li).li_tv) };
        let buflen = unsafe { utf_char2bytes(n as ::core::ffi::c_int, buf.as_mut_ptr()) } as size_t;
        buf[buflen as usize] = '\0' as ::core::ffi::c_char;
        unsafe { ga_concat_len(&raw mut ga, buf.as_mut_ptr(), buflen) };
    }
    unsafe { ga_append(&raw mut ga, NUL as uint8_t) };
    unsafe { (*rettv).vval.v_string = ga.ga_data as *mut ::core::ffi::c_char };
}
