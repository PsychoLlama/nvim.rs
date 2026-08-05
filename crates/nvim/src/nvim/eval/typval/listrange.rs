//! Ranges over a list: slicing, assigning through a slice, flattening, joining.
//!
//! [`tv_list_check_range_index_one`] and [`tv_list_check_range_index_two`]
//! are the bounds arithmetic `l[i:j]` shares with `l[i:j] = x`;
//! [`tv_list_slice_or_index`] is the subscript itself.
//! [`tv_list_join`] and [`list_join_inner`] are `join()`, which makes two
//! passes so the result buffer is sized once, and [`f_list2str`] is the
//! codepoint-list-to-string builtin.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn tv_list_check_range_index_one(
    l: *mut list_T,
    n1: *mut ::core::ffi::c_int,
    quiet: bool,
) -> *mut listitem_T {
    unsafe {
        let mut li: *mut listitem_T = tv_list_find_index(l, n1);
        if !li.is_null() {
            return li;
        }
        if !quiet {
            semsg(
                gettext(&raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char),
                *n1 as int64_t,
            );
        }
        return ::core::ptr::null_mut::<listitem_T>();
    }
}

pub unsafe extern "C" fn tv_list_check_range_index_two(
    l: *mut list_T,
    n1: *mut ::core::ffi::c_int,
    li1: *const listitem_T,
    n2: *mut ::core::ffi::c_int,
    quiet: bool,
) -> ::core::ffi::c_int {
    unsafe {
        if *n2 < 0 as ::core::ffi::c_int {
            let mut ni: *mut listitem_T = tv_list_find(l, *n2);
            if ni.is_null() {
                if !quiet {
                    semsg(
                        gettext(
                            &raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char,
                        ),
                        *n2 as int64_t,
                    );
                }
                return FAIL;
            }
            *n2 = tv_list_idx_of_item(l, ni);
        }
        if *n1 < 0 as ::core::ffi::c_int {
            *n1 = tv_list_idx_of_item(l, li1);
        }
        if *n2 < *n1 {
            if !quiet {
                semsg(
                    gettext(&raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char),
                    *n2 as int64_t,
                );
            }
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn tv_list_assign_range(
    dest: *mut list_T,
    src: *mut list_T,
    idx1_arg: ::core::ffi::c_int,
    idx2: ::core::ffi::c_int,
    empty_idx2: bool,
    op: *const ::core::ffi::c_char,
    varname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut idx1: ::core::ffi::c_int = idx1_arg;
        let first_li: *mut listitem_T = tv_list_find_index(dest, &raw mut idx1);
        let mut src_li: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
        let mut idx: ::core::ffi::c_int = idx1;
        let mut dest_li: *mut listitem_T = first_li;
        src_li = tv_list_first(src);
        while !src_li.is_null() && !dest_li.is_null() {
            if value_check_lock((*dest_li).li_tv.v_lock, varname, TV_CSTRING as size_t) {
                return FAIL;
            }
            src_li = (*src_li).li_next;
            if src_li.is_null() || !empty_idx2 && idx2 == idx {
                break;
            }
            dest_li = (*dest_li).li_next;
            idx += 1;
        }
        idx = idx1;
        dest_li = first_li;
        src_li = tv_list_first(src);
        while !src_li.is_null() {
            '_c2rust_label: {
                if !dest_li.is_null() {
                } else {
                    __assert_fail(
                    b"dest_li != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/typval.rs\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    710 as ::core::ffi::c_uint,
                    b"int tv_list_assign_range(list_T *const, list_T *const, const int, const int, const _Bool, const char *const, const char *const)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
                }
            };
            if !op.is_null() && *op as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
                eexe_mod_op(&raw mut (*dest_li).li_tv, &raw mut (*src_li).li_tv, op);
            } else {
                tv_clear(&raw mut (*dest_li).li_tv);
                tv_copy(&raw mut (*src_li).li_tv, &raw mut (*dest_li).li_tv);
            }
            src_li = (*src_li).li_next;
            if src_li.is_null() || !empty_idx2 && idx2 == idx {
                break;
            }
            if (*dest_li).li_next.is_null() {
                tv_list_append_number(dest, 0 as varnumber_T);
                dest_li = tv_list_last(dest);
            } else {
                dest_li = (*dest_li).li_next;
            }
            idx += 1;
        }
        if !src_li.is_null() {
            emsg(gettext(
                b"E710: List value has more items than target\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
        if if empty_idx2 as ::core::ffi::c_int != 0 {
            (!dest_li.is_null() && !(*dest_li).li_next.is_null()) as ::core::ffi::c_int
        } else {
            (idx != idx2) as ::core::ffi::c_int
        } != 0
        {
            emsg(gettext(
                b"E711: List value has not enough items\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn tv_list_flatten(
    mut list: *mut list_T,
    mut first: *mut listitem_T,
    mut maxitems: int64_t,
    mut maxdepth: int64_t,
) {
    unsafe {
        let mut item: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
        let mut done: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if maxdepth == 0 as int64_t {
            return;
        }
        if first.is_null() {
            item = (*list).lv_first;
        } else {
            item = first;
        }
        while !item.is_null() && (done as int64_t) < maxitems {
            let mut next: *mut listitem_T = (*item).li_next;
            fast_breakcheck();
            if got_int.get() {
                return;
            }
            if (*item).li_tv.v_type as ::core::ffi::c_uint
                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut itemlist: *mut list_T = (*item).li_tv.vval.v_list;
                tv_list_drop_items(list, item, item);
                tv_list_extend(list, itemlist, next);
                if maxdepth > 0 as int64_t {
                    tv_list_flatten(
                        list,
                        if (*item).li_prev.is_null() {
                            (*list).lv_first
                        } else {
                            (*(*item).li_prev).li_next
                        },
                        (*itemlist).lv_len as int64_t,
                        maxdepth - 1 as int64_t,
                    );
                }
                tv_clear(&raw mut (*item).li_tv);
                xfree(item as *mut ::core::ffi::c_void);
            }
            done += 1;
            item = next;
        }
    }
}

pub(crate) unsafe extern "C" fn tv_list_slice(
    mut ol: *mut list_T,
    mut n1: varnumber_T,
    mut n2: varnumber_T,
) -> *mut list_T {
    unsafe {
        let mut l: *mut list_T = tv_list_alloc(n2 as ptrdiff_t - n1 as ptrdiff_t + 1 as ptrdiff_t);
        let mut item: *mut listitem_T = tv_list_find(ol, n1 as ::core::ffi::c_int);
        while n1 <= n2 {
            tv_list_append_tv(l, &raw mut (*item).li_tv);
            item = (*item).li_next;
            n1 += 1;
        }
        return l;
    }
}

pub unsafe extern "C" fn tv_list_slice_or_index(
    mut _list: *mut list_T,
    mut range: bool,
    mut n1_arg: varnumber_T,
    mut n2_arg: varnumber_T,
    mut exclusive: bool,
    mut rettv: *mut typval_T,
    mut verbose: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut len: ::core::ffi::c_int = tv_list_len((*rettv).vval.v_list);
        let mut n1: varnumber_T = n1_arg;
        let mut n2: varnumber_T = n2_arg;
        if n1 < 0 as varnumber_T {
            n1 = len as varnumber_T + n1;
        }
        if n1 < 0 as varnumber_T || n1 >= len as varnumber_T {
            if !range {
                if verbose {
                    semsg(
                        gettext(
                            &raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char,
                        ),
                        n1_arg,
                    );
                }
                return FAIL;
            }
            n1 = len as varnumber_T;
        }
        if range {
            if n2 < 0 as varnumber_T {
                n2 = len as varnumber_T + n2;
            } else if n2 >= len as varnumber_T {
                n2 = (len
                    - (if exclusive as ::core::ffi::c_int != 0 {
                        0 as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                    })) as varnumber_T;
            }
            if exclusive {
                n2 -= 1;
            }
            if n2 < 0 as varnumber_T || (n2 + 1 as varnumber_T) < n1 {
                n2 = -1 as varnumber_T;
            }
            let mut l: *mut list_T = tv_list_slice((*rettv).vval.v_list, n1, n2);
            tv_clear(rettv);
            tv_list_set_ret(rettv, l);
        } else {
            let mut var1: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            tv_copy(
                &raw mut (*(tv_list_find
                    as unsafe extern "C" fn(*mut list_T, ::core::ffi::c_int) -> *mut listitem_T)(
                    (*rettv).vval.v_list,
                    n1 as ::core::ffi::c_int,
                ))
                .li_tv,
                &raw mut var1,
            );
            tv_clear(rettv);
            *rettv = var1;
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn list_join_inner(
    gap: *mut garray_T,
    l: *mut list_T,
    sep: *const ::core::ffi::c_char,
    join_gap: *mut garray_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut sumlen: size_t = 0 as size_t;
        let mut first: bool = true_0 != 0;
        let l_: *mut list_T = l;
        if !l_.is_null() {
            let mut item: *mut listitem_T = (*l_).lv_first;
            while !item.is_null() {
                if got_int.get() {
                    break;
                }
                let mut s: String_0 = String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                };
                s.data = encode_tv2echo(&raw mut (*item).li_tv, &raw mut s.size);
                if s.data.is_null() {
                    return 0 as ::core::ffi::c_int;
                }
                sumlen = sumlen.wrapping_add(s.size);
                let p: *mut Join =
                    ga_append_via_ptr(join_gap, ::core::mem::size_of::<Join>()) as *mut Join;
                (*p).s = s;
                (*p).tofree = s.data;
                line_breakcheck();
                item = (*item).li_next;
            }
        }
        let mut seplen: size_t = strlen(sep);
        if (*join_gap).ga_len >= 2 as ::core::ffi::c_int {
            sumlen = sumlen.wrapping_add(
                seplen.wrapping_mul(((*join_gap).ga_len - 1 as ::core::ffi::c_int) as size_t),
            );
        }
        ga_grow(gap, sumlen as ::core::ffi::c_int + 2 as ::core::ffi::c_int);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*join_gap).ga_len && !got_int.get() {
            if first {
                first = false_0 != 0;
            } else {
                ga_concat_len(gap, sep, seplen);
            }
            let p_0: *const Join = ((*join_gap).ga_data as *const Join).offset(i as isize);
            if !(*p_0).s.data.is_null() {
                ga_concat_len(gap, (*p_0).s.data, (*p_0).s.size);
            }
            line_breakcheck();
            i += 1;
        }
        return OK;
    }
}

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
        let mut join_ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        let mut retval: ::core::ffi::c_int = 0;
        ga_init(
            &raw mut join_ga,
            ::core::mem::size_of::<Join>() as ::core::ffi::c_int,
            tv_list_len(l),
        );
        retval = list_join_inner(gap, l, sep, &raw mut join_ga);
        let mut _gap: *mut garray_T = &raw mut join_ga;
        if !(*_gap).ga_data.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*_gap).ga_len {
                let mut _item: *mut Join = ((*_gap).ga_data as *mut Join).offset(i as isize);
                xfree((*_item).tofree as *mut ::core::ffi::c_void);
                i += 1;
            }
        }
        ga_clear(_gap);
        return retval;
    }
}

pub unsafe extern "C" fn f_join(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
            return;
        }
        let sep: *const ::core::ffi::c_char = if (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            b" \0".as_ptr() as *const ::core::ffi::c_char
        } else {
            tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize))
        };
        (*rettv).v_type = VAR_STRING;
        if !sep.is_null() {
            let mut ga: garray_T = garray_T {
                ga_len: 0,
                ga_maxlen: 0,
                ga_itemsize: 0,
                ga_growsize: 0,
                ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            };
            ga_init(
                &raw mut ga,
                ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
                80 as ::core::ffi::c_int,
            );
            tv_list_join(
                &raw mut ga,
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_list,
                sep,
            );
            ga_append(&raw mut ga, NUL as uint8_t);
            (*rettv).vval.v_string = ga.ga_data as *mut ::core::ffi::c_char;
        } else {
            (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        };
    }
}

pub unsafe extern "C" fn f_list2str(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return;
        }
        let l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list;
        if l.is_null() {
            return;
        }
        ga_init(
            &raw mut ga,
            1 as ::core::ffi::c_int,
            80 as ::core::ffi::c_int,
        );
        let mut buf: [::core::ffi::c_char; 22] = [0; 22];
        let l_: *const list_T = l;
        if !l_.is_null() {
            let mut li: *const listitem_T = (*l_).lv_first;
            while !li.is_null() {
                let n: varnumber_T = tv_get_number(&raw const (*li).li_tv);
                let buflen: size_t = utf_char2bytes(
                    n as ::core::ffi::c_int,
                    &raw mut buf as *mut ::core::ffi::c_char,
                ) as size_t;
                buf[buflen as usize] = '\0' as ::core::ffi::c_char;
                ga_concat_len(
                    &raw mut ga,
                    &raw mut buf as *mut ::core::ffi::c_char,
                    buflen,
                );
                li = (*li).li_next;
            }
        }
        ga_append(&raw mut ga, NUL as uint8_t);
        (*rettv).vval.v_string = ga.ga_data as *mut ::core::ffi::c_char;
    }
}
