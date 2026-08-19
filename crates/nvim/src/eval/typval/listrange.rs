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
use crate::semsg_c;
use crate::types::{FAIL, NUL, OK};

/// Resolve the first index of `l[n1:n2]`, clamping a negative one that fell
/// off the front and raising `E684` when there is no such item.
///
/// `*n1` is updated to the index actually used.
pub unsafe fn tv_list_check_range_index_one(
    l: *mut list_T,
    n1: *mut ::core::ffi::c_int,
    quiet: bool,
) -> *mut listitem_T {
    unsafe {
        let li = tv_list_find_index(l, n1);
        if li.is_null() && !quiet {
            semsg_c!(
                gettext(&raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char),
                int64_t::from(*n1),
            );
        }
        li
    }
}

/// Resolve the second index of `l[n1:n2]` against the item `li1` the first one
/// landed on, normalising both to non-negative indexes.
pub unsafe fn tv_list_check_range_index_two(
    l: *mut list_T,
    n1: *mut ::core::ffi::c_int,
    li1: *const listitem_T,
    n2: *mut ::core::ffi::c_int,
    quiet: bool,
) -> ::core::ffi::c_int {
    unsafe {
        if *n2 < 0 {
            let ni = tv_list_find(l, *n2);
            if ni.is_null() {
                if !quiet {
                    semsg_c!(
                        gettext(
                            &raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char,
                        ),
                        int64_t::from(*n2),
                    );
                }
                return FAIL;
            }
            *n2 = tv_list_idx_of_item(l, ni);
        }
        if *n1 < 0 {
            *n1 = tv_list_idx_of_item(l, li1);
        }
        if *n2 < *n1 {
            if !quiet {
                semsg_c!(
                    gettext(&raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char),
                    int64_t::from(*n2),
                );
            }
            return FAIL;
        }
        OK
    }
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
) -> ::core::ffi::c_int {
    unsafe {
        let mut idx1 = idx1_arg;
        let first_li = tv_list_find_index(dest, &raw mut idx1);

        // Check whether any of the list items is locked before making any
        // changes.
        let mut idx = idx1;
        let mut dest_li = first_li;
        let mut src_li = tv_list_first(src);
        while !src_li.is_null() && !dest_li.is_null() {
            if value_check_lock((*dest_li).li_tv.v_lock, varname, TV_CSTRING as size_t) {
                return FAIL;
            }
            src_li = (*src_li).li_next;
            if src_li.is_null() || (!empty_idx2 && idx2 == idx) {
                break;
            }
            dest_li = (*dest_li).li_next;
            idx += 1;
        }

        // Assign the List values to the list items.
        idx = idx1;
        dest_li = first_li;
        src_li = tv_list_first(src);
        while !src_li.is_null() {
            debug_assert!(!dest_li.is_null());
            if !op.is_null() && *op as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
                eexe_mod_op(&raw mut (*dest_li).li_tv, &raw mut (*src_li).li_tv, op);
            } else {
                tv_clear(&raw mut (*dest_li).li_tv);
                tv_copy(&raw mut (*src_li).li_tv, &raw mut (*dest_li).li_tv);
            }
            src_li = (*src_li).li_next;
            if src_li.is_null() || (!empty_idx2 && idx2 == idx) {
                break;
            }
            if (*dest_li).li_next.is_null() {
                // Need to add an empty item.
                tv_list_append_number(dest, 0);
                // "dest_li" may have become invalid after append, don't use it.
                dest_li = tv_list_last(dest); // Valid again.
            } else {
                dest_li = (*dest_li).li_next;
            }
            idx += 1;
        }

        if !src_li.is_null() {
            emsg(gettext(
                c"E710: List value has more items than target".as_ptr(),
            ));
            return FAIL;
        }
        let short = if empty_idx2 {
            !dest_li.is_null() && !(*dest_li).li_next.is_null()
        } else {
            idx != idx2
        };
        if short {
            emsg(gettext(c"E711: List value has not enough items".as_ptr()));
            return FAIL;
        }
        OK
    }
}

/// `flatten()`: splice the items of any nested list into `list` in place,
/// starting at `first` and going `maxdepth` levels down.
pub unsafe fn tv_list_flatten(
    list: *mut list_T,
    first: *mut listitem_T,
    maxitems: int64_t,
    maxdepth: int64_t,
) {
    unsafe {
        if maxdepth == 0 {
            return;
        }

        let mut item = if first.is_null() {
            (*list).lv_first
        } else {
            first
        };
        let mut done = 0;
        while !item.is_null() && done < maxitems {
            // The link is read before the body, which unlinks and frees `item`.
            let next = (*item).li_next;

            fast_breakcheck();
            if got_int.get() {
                return;
            }
            if (*item).li_tv.v_type == VAR_LIST {
                let itemlist = (*item).li_tv.vval.v_list;

                tv_list_drop_items(list, item, item);
                tv_list_extend(list, itemlist, next);

                if maxdepth > 0 {
                    let spliced_first = if (*item).li_prev.is_null() {
                        (*list).lv_first
                    } else {
                        (*(*item).li_prev).li_next
                    };
                    tv_list_flatten(
                        list,
                        spliced_first,
                        int64_t::from((*itemlist).lv_len),
                        maxdepth - 1,
                    );
                }
                tv_clear(&raw mut (*item).li_tv);
                xfree(item.cast());
            }

            done += 1;
            item = next;
        }
    }
}

/// A fresh list holding copies of `ol[n1..=n2]`.
pub(crate) unsafe fn tv_list_slice(
    ol: *mut list_T,
    mut n1: varnumber_T,
    n2: varnumber_T,
) -> *mut list_T {
    unsafe {
        let l = tv_list_alloc((n2 - n1 + 1) as ptrdiff_t);
        let mut item = tv_list_find(ol, n1 as ::core::ffi::c_int);
        while n1 <= n2 {
            tv_list_append_tv(l, &raw mut (*item).li_tv);
            item = (*item).li_next;
            n1 += 1;
        }
        l
    }
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
) -> ::core::ffi::c_int {
    unsafe {
        let len = tv_list_len((*rettv).vval.v_list);
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
                    semsg_c!(
                        gettext(
                            &raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char,
                        ),
                        n1_arg,
                    );
                }
                return FAIL;
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
            let l = tv_list_slice((*rettv).vval.v_list, n1, n2);
            tv_clear(rettv);
            tv_list_set_ret(rettv, l);
        } else {
            // copy the item to "var1" to avoid that freeing the list makes it
            // invalid.
            let mut var1 = TV_INITIAL_VALUE;
            let li = tv_list_find((*rettv).vval.v_list, n1 as ::core::ffi::c_int);
            tv_copy(&raw mut (*li).li_tv, &raw mut var1);
            tv_clear(rettv);
            *rettv = var1;
        }
        OK
    }
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
) -> ::core::ffi::c_int {
    unsafe {
        let mut sumlen: size_t = 0;
        let mut first = true;

        // Stringify each item in the list.
        for item in tv_list_iter(l.as_ref()) {
            if got_int.get() {
                break;
            }
            let mut s = String_0::NULL;
            let data = encode_tv2echo(&raw mut (*item).li_tv, s.len_mut());
            s.set_data(data);
            if s.data().is_null() {
                return FAIL;
            }

            sumlen += s.len();

            let p = ga_append_via_ptr(join_gap, ::core::mem::size_of::<Join>()) as *mut Join;
            (*p).s = s;
            (*p).tofree = s.data();

            line_breakcheck();
        }

        // Allocate result buffer with its total size, avoid re-allocation and
        // multiple copy operations.  Add 2 for a tailing ']' and NUL.
        let seplen = strlen(sep);
        if (*join_gap).ga_len >= 2 {
            sumlen += seplen * ((*join_gap).ga_len - 1) as size_t;
        }
        ga_grow(gap, sumlen as ::core::ffi::c_int + 2);

        let mut i = 0;
        while i < (*join_gap).ga_len && !got_int.get() {
            if first {
                first = false;
            } else {
                ga_concat_len(gap, sep, seplen);
            }
            let p = ((*join_gap).ga_data as *const Join).offset(i as isize);
            if !(*p).s.data().is_null() {
                ga_concat_len(gap, (*p).s.data(), (*p).s.len());
            }
            line_breakcheck();
            i += 1;
        }

        OK
    }
}

/// `join()`: append `l`'s items to `gap`, separated by `sep`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_join(
    gap: *mut garray_T,
    l: *mut list_T,
    sep: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        if tv_list_len(l) == 0 {
            return OK;
        }

        let mut join_ga = GARRAY_EMPTY;
        ga_init(
            &raw mut join_ga,
            ::core::mem::size_of::<Join>() as ::core::ffi::c_int,
            tv_list_len(l),
        );
        let retval = list_join_inner(gap, l, sep, &raw mut join_ga);

        // GA_DEEP_CLEAR with FREE_JOIN_TOFREE.
        if !join_ga.ga_data.is_null() {
            for i in 0..join_ga.ga_len {
                xfree(
                    (*(join_ga.ga_data as *mut Join).offset(i as isize))
                        .tofree
                        .cast(),
                );
            }
        }
        ga_clear(&raw mut join_ga);

        retval
    }
}

/// `join()` the builtin.
pub unsafe fn f_join(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        if (*argvars).v_type != VAR_LIST {
            emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
            return;
        }
        let sep = if (*argvars.add(1)).v_type == VAR_UNKNOWN {
            c" ".as_ptr()
        } else {
            tv_get_string_chk(argvars.add(1))
        };

        (*rettv).v_type = VAR_STRING;
        if sep.is_null() {
            (*rettv).vval.v_string = ::core::ptr::null_mut();
            return;
        }

        let mut ga = GARRAY_EMPTY;
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
            80,
        );
        tv_list_join(&raw mut ga, (*argvars).vval.v_list, sep);
        ga_append(&raw mut ga, NUL as uint8_t);
        (*rettv).vval.v_string = ga.ga_data as *mut ::core::ffi::c_char;
    }
}

/// `list2str()`: a list of codepoints as a string.
pub unsafe fn f_list2str(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = ::core::ptr::null_mut();
        if (*argvars).v_type != VAR_LIST {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return;
        }
        let l = (*argvars).vval.v_list;
        if l.is_null() {
            return;
        }

        let mut ga = GARRAY_EMPTY;
        ga_init(&raw mut ga, 1, 80);
        let mut buf: [::core::ffi::c_char; 22] = [0; 22];
        for li in tv_list_iter(l.as_ref()) {
            let n = tv_get_number(&raw const (*li).li_tv);
            let buflen = utf_char2bytes(n as ::core::ffi::c_int, buf.as_mut_ptr()) as size_t;
            buf[buflen as usize] = '\0' as ::core::ffi::c_char;
            ga_concat_len(&raw mut ga, buf.as_mut_ptr(), buflen);
        }
        ga_append(&raw mut ga, NUL as uint8_t);
        (*rettv).vval.v_string = ga.ga_data as *mut ::core::ffi::c_char;
    }
}
