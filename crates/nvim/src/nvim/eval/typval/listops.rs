//! Filling a list, copying one, and finding an item in it.
//!
//! The `tv_list_append_*` family is the C header's overload set — one
//! function per value kind, each allocating a `listitem_T` and linking it at
//! the tail.  [`tv_list_copy`] is `copy()`/`deepcopy()` over a list,
//! [`tv_list_extend`] and [`tv_list_concat`] the `extend()`/`+` pair, and
//! [`tv_list_find`] the index walk `list[n]` resolves through, which counts
//! from the tail for a negative index.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_insert(
    l: *mut list_T,
    ni: *mut listitem_T,
    item: *mut listitem_T,
) {
    unsafe {
        if item.is_null() {
            tv_list_append(l, ni);
        } else {
            (*ni).li_prev = (*item).li_prev;
            (*ni).li_next = item;
            if (*item).li_prev.is_null() {
                (*l).lv_first = ni;
                (*l).lv_idx += 1;
            } else {
                (*(*item).li_prev).li_next = ni;
                (*l).lv_idx_item = ::core::ptr::null_mut::<listitem_T>();
            }
            (*item).li_prev = ni;
            (*l).lv_len += 1;
        };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_insert_tv(
    l: *mut list_T,
    tv: *mut typval_T,
    item: *mut listitem_T,
) {
    unsafe {
        let ni: *mut listitem_T = tv_list_item_alloc();
        tv_copy(tv, &raw mut (*ni).li_tv);
        tv_list_insert(l, ni, item);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_append(l: *mut list_T, item: *mut listitem_T) {
    unsafe {
        if (*l).lv_last.is_null() {
            (*l).lv_first = item;
            (*l).lv_last = item;
            (*item).li_prev = ::core::ptr::null_mut::<listitem_T>();
        } else {
            (*(*l).lv_last).li_next = item;
            (*item).li_prev = (*l).lv_last;
            (*l).lv_last = item;
        }
        (*l).lv_len += 1;
        (*item).li_next = ::core::ptr::null_mut::<listitem_T>();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_append_tv(l: *mut list_T, tv: *mut typval_T) {
    unsafe {
        let li: *mut listitem_T = tv_list_item_alloc();
        tv_copy(tv, &raw mut (*li).li_tv);
        tv_list_append(l, li);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_append_owned_tv(
    l: *mut list_T,
    mut tv: typval_T,
) -> *mut typval_T {
    unsafe {
        let li: *mut listitem_T = tv_list_item_alloc();
        (*li).li_tv = tv;
        tv_list_append(l, li);
        return &raw mut (*li).li_tv;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_append_list(l: *mut list_T, itemlist: *mut list_T) {
    unsafe {
        tv_list_append_owned_tv(
            l,
            typval_T {
                v_type: VAR_LIST,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_list: itemlist },
            },
        );
        tv_list_ref(itemlist);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_append_dict(l: *mut list_T, dict: *mut dict_T) {
    unsafe {
        tv_list_append_owned_tv(
            l,
            typval_T {
                v_type: VAR_DICT,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_dict: dict },
            },
        );
        if !dict.is_null() {
            (*dict).dv_refcount += 1;
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_append_string(
    l: *mut list_T,
    str: *const ::core::ffi::c_char,
    len: ssize_t,
) {
    unsafe {
        tv_list_append_owned_tv(
            l,
            typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union {
                    v_string: (if str.is_null() {
                        NULL_0
                    } else if len >= 0 as ssize_t {
                        xmemdupz(str as *const ::core::ffi::c_void, len as size_t)
                    } else {
                        xstrdup(str) as *mut ::core::ffi::c_void
                    }) as *mut ::core::ffi::c_char,
                },
            },
        );
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_append_allocated_string(
    l: *mut list_T,
    str: *mut ::core::ffi::c_char,
) {
    unsafe {
        tv_list_append_owned_tv(
            l,
            typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_string: str },
            },
        );
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_append_number(l: *mut list_T, n: varnumber_T) {
    unsafe {
        tv_list_append_owned_tv(
            l,
            typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: n },
            },
        );
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_copy(
    conv: *const vimconv_T,
    orig: *mut list_T,
    deep: bool,
    copyID: ::core::ffi::c_int,
) -> *mut list_T {
    unsafe {
        if orig.is_null() {
            return ::core::ptr::null_mut::<list_T>();
        }
        let mut copy: *mut list_T = tv_list_alloc(tv_list_len(orig) as ptrdiff_t);
        tv_list_ref(copy);
        if copyID != 0 as ::core::ffi::c_int {
            (*orig).lv_copyID = copyID;
            (*orig).lv_copylist = copy;
        }
        let l_: *mut list_T = orig;
        's_99: {
            if !l_.is_null() {
                let mut item: *mut listitem_T = (*l_).lv_first;
                loop {
                    if item.is_null() {
                        break 's_99;
                    }
                    if got_int.get() {
                        break 's_99;
                    }
                    let ni: *mut listitem_T = tv_list_item_alloc();
                    if deep {
                        if var_item_copy(
                            conv,
                            &raw mut (*item).li_tv,
                            &raw mut (*ni).li_tv,
                            deep,
                            copyID,
                        ) == 0 as ::core::ffi::c_int
                        {
                            xfree(ni as *mut ::core::ffi::c_void);
                            break;
                        }
                    } else {
                        tv_copy(&raw mut (*item).li_tv, &raw mut (*ni).li_tv);
                    }
                    tv_list_append(copy, ni);
                    item = (*item).li_next;
                }
                tv_list_unref(copy);
                return ::core::ptr::null_mut::<list_T>();
            }
        }
        return copy;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_extend(l1: *mut list_T, l2: *mut list_T, bef: *mut listitem_T) {
    unsafe {
        let mut todo: ::core::ffi::c_int = tv_list_len(l2);
        let befbef: *mut listitem_T = if bef.is_null() {
            ::core::ptr::null_mut::<listitem_T>()
        } else {
            (*bef).li_prev
        };
        let saved_next: *mut listitem_T = if befbef.is_null() {
            ::core::ptr::null_mut::<listitem_T>()
        } else {
            (*befbef).li_next
        };
        let mut item: *mut listitem_T = tv_list_first(l2);
        while !item.is_null() && {
            let c2rust_fresh8 = todo;
            todo = todo - 1;
            c2rust_fresh8 != 0
        } {
            tv_list_insert_tv(l1, &raw mut (*item).li_tv, bef);
            item = if item == befbef {
                saved_next
            } else {
                (*item).li_next
            };
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_concat(
    l1: *mut list_T,
    l2: *mut list_T,
    tv: *mut typval_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
        (*tv).v_type = VAR_LIST;
        (*tv).v_lock = VAR_UNLOCKED;
        if l1.is_null() && l2.is_null() {
            l = ::core::ptr::null_mut::<list_T>();
        } else if l1.is_null() {
            l = tv_list_copy(
                ::core::ptr::null::<vimconv_T>(),
                l2,
                false_0 != 0,
                0 as ::core::ffi::c_int,
            );
        } else {
            l = tv_list_copy(
                ::core::ptr::null::<vimconv_T>(),
                l1,
                false_0 != 0,
                0 as ::core::ffi::c_int,
            );
            if !l.is_null() && !l2.is_null() {
                tv_list_extend(l, l2, ::core::ptr::null_mut::<listitem_T>());
            }
        }
        if l.is_null() && !(l1.is_null() && l2.is_null()) {
            return FAIL;
        }
        (*tv).vval.v_list = l;
        return OK;
    }
}

pub unsafe extern "C" fn tv_list_remove(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut arg_errmsg: *const ::core::ffi::c_char,
) {
    unsafe {
        let mut l: *mut list_T = ::core::ptr::null_mut::<list_T>();
        let mut error: bool = false_0 != 0;
        l = (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_list;
        if value_check_lock(tv_list_locked(l), arg_errmsg, TV_TRANSLATE as size_t) {
            return;
        }
        let mut idx: int64_t = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        );
        let mut item: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
        if !error {
            item = tv_list_find(l, idx as ::core::ffi::c_int);
            if item.is_null() {
                semsg(
                    gettext(&raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char),
                    idx,
                );
            } else if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type
                as ::core::ffi::c_uint
                == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                tv_list_drop_items(l, item, item);
                *rettv = (*item).li_tv;
                xfree(item as *mut ::core::ffi::c_void);
            } else {
                let mut item2: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
                let mut end: int64_t = tv_get_number_chk(
                    argvars.offset(2 as ::core::ffi::c_int as isize),
                    &raw mut error,
                );
                if !error {
                    item2 = tv_list_find(l, end as ::core::ffi::c_int);
                    if item2.is_null() {
                        semsg(
                            gettext(
                                &raw const e_list_index_out_of_range_nr
                                    as *const ::core::ffi::c_char,
                            ),
                            end,
                        );
                    } else {
                        let mut cnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut li: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
                        li = item;
                        while !li.is_null() {
                            cnt += 1;
                            if li == item2 {
                                break;
                            }
                            li = (*li).li_next;
                        }
                        if li.is_null() {
                            emsg(gettext(&raw const e_invrange as *const ::core::ffi::c_char));
                        } else {
                            tv_list_move_items(
                                l,
                                item,
                                item2,
                                tv_list_alloc_ret(rettv, cnt as ptrdiff_t),
                                cnt,
                            );
                        }
                    }
                }
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_equal(l1: *mut list_T, l2: *mut list_T, ic: bool) -> bool {
    unsafe {
        if l1 == l2 {
            return true_0 != 0;
        }
        if tv_list_len(l1) != tv_list_len(l2) {
            return false_0 != 0;
        }
        if tv_list_len(l1) == 0 as ::core::ffi::c_int {
            return true_0 != 0;
        }
        if l1.is_null() || l2.is_null() {
            return false_0 != 0;
        }
        let mut item1: *mut listitem_T = tv_list_first(l1);
        let mut item2: *mut listitem_T = tv_list_first(l2);
        while !item1.is_null() && !item2.is_null() {
            if !tv_equal(&raw mut (*item1).li_tv, &raw mut (*item2).li_tv, ic) {
                return false_0 != 0;
            }
            item1 = (*item1).li_next;
            item2 = (*item2).li_next;
        }
        '_c2rust_label: {
            if item1.is_null() && item2.is_null() {
            } else {
                __assert_fail(
                    b"item1 == NULL && item2 == NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1568 as ::core::ffi::c_uint,
                    b"_Bool tv_list_equal(list_T *const, list_T *const, const _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn tv_list_reverse(l: *mut list_T) {
    unsafe {
        if tv_list_len(l) <= 1 as ::core::ffi::c_int {
            return;
        }
        let mut tmp: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
        tmp = (*l).lv_first;
        (*l).lv_first = (*l).lv_last;
        (*l).lv_last = tmp;
        let mut li: *mut listitem_T = (*l).lv_first;
        while !li.is_null() {
            tmp = (*li).li_next;
            (*li).li_next = (*li).li_prev;
            (*li).li_prev = tmp;
            li = (*li).li_next;
        }
        (*l).lv_idx = (*l).lv_len - (*l).lv_idx - 1 as ::core::ffi::c_int;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_find(
    l: *mut list_T,
    mut n: ::core::ffi::c_int,
) -> *mut listitem_T {
    unsafe {
        if l.is_null() {
            return ::core::ptr::null_mut::<listitem_T>();
        }
        n = tv_list_uidx(l, n);
        if n == -1 as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<listitem_T>();
        }
        let mut idx: ::core::ffi::c_int = 0;
        let mut item: *mut listitem_T = ::core::ptr::null_mut::<listitem_T>();
        if !(*l).lv_idx_item.is_null() {
            if n < (*l).lv_idx / 2 as ::core::ffi::c_int {
                item = (*l).lv_first;
                idx = 0 as ::core::ffi::c_int;
            } else if n > ((*l).lv_idx + (*l).lv_len) / 2 as ::core::ffi::c_int {
                item = (*l).lv_last;
                idx = (*l).lv_len - 1 as ::core::ffi::c_int;
            } else {
                item = (*l).lv_idx_item;
                idx = (*l).lv_idx;
            }
        } else if n < (*l).lv_len / 2 as ::core::ffi::c_int {
            item = (*l).lv_first;
            idx = 0 as ::core::ffi::c_int;
        } else {
            item = (*l).lv_last;
            idx = (*l).lv_len - 1 as ::core::ffi::c_int;
        }
        while n > idx {
            item = (*item).li_next;
            idx += 1;
        }
        while n < idx {
            item = (*item).li_prev;
            idx -= 1;
        }
        '_c2rust_label: {
            if idx == n {
            } else {
                __assert_fail(
                    b"idx == n\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1661 as ::core::ffi::c_uint,
                    b"listitem_T *tv_list_find(list_T *const, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        (*l).lv_idx = idx;
        (*l).lv_idx_item = item;
        return item;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_find_nr(
    l: *mut list_T,
    n: ::core::ffi::c_int,
    ret_error: *mut bool,
) -> varnumber_T {
    unsafe {
        let li: *const listitem_T = tv_list_find(l, n);
        if li.is_null() {
            if !ret_error.is_null() {
                *ret_error = true_0 != 0;
            }
            return -1 as varnumber_T;
        }
        return tv_get_number_chk(&raw const (*li).li_tv, ret_error);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_find_str(
    l: *mut list_T,
    n: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    unsafe {
        let li: *const listitem_T = tv_list_find(l, n);
        if li.is_null() {
            semsg(
                gettext(&raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char),
                n as int64_t,
            );
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        return tv_get_string(&raw const (*li).li_tv);
    }
}

pub(crate) unsafe extern "C" fn tv_list_find_index(
    l: *mut list_T,
    idx: *mut ::core::ffi::c_int,
) -> *mut listitem_T {
    unsafe {
        let mut li: *mut listitem_T = tv_list_find(l, *idx);
        if !li.is_null() {
            return li;
        }
        if *idx < 0 as ::core::ffi::c_int {
            *idx = 0 as ::core::ffi::c_int;
            li = tv_list_find(l, *idx);
        }
        return li;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_idx_of_item(
    l: *const list_T,
    item: *const listitem_T,
) -> ::core::ffi::c_int {
    unsafe {
        if l.is_null() {
            return -1 as ::core::ffi::c_int;
        }
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let l_: *const list_T = l;
        if !l_.is_null() {
            let mut li: *const listitem_T = (*l_).lv_first;
            while !li.is_null() {
                if li == item {
                    return idx;
                }
                idx += 1;
                li = (*li).li_next;
            }
        }
        return -1 as ::core::ffi::c_int;
    }
}
