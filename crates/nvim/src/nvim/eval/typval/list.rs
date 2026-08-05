//! Allocating, freeing and unlinking a `list_T` and its `listitem_T`s.
//!
//! [`tv_list_alloc`] and [`tv_list_free`] are the reference-counted pair,
//! [`tv_list_unref`] the one every caller actually uses.  The `listwatch_T`
//! half ([`tv_list_watch_add`], [`tv_list_watch_fix`]) is how a `:for` loop
//! survives having the item it is standing on removed underneath it, and
//! [`tv_list_drop_items`] / [`tv_list_move_items`] are the two ways items
//! leave a list.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn tv_list_item_alloc() -> *mut listitem_T {
    unsafe {
        return xmalloc(::core::mem::size_of::<listitem_T>()) as *mut listitem_T;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_item_remove(
    l: *mut list_T,
    item: *mut listitem_T,
) -> *mut listitem_T {
    unsafe {
        let next_item: *mut listitem_T = (*item).li_next;
        tv_list_drop_items(l, item, item);
        tv_clear(&raw mut (*item).li_tv);
        xfree(item as *mut ::core::ffi::c_void);
        return next_item;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_watch_add(l: *mut list_T, lw: *mut listwatch_T) {
    unsafe {
        (*lw).lw_next = (*l).lv_watch;
        (*l).lv_watch = lw;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_watch_remove(l: *mut list_T, lwrem: *mut listwatch_T) {
    unsafe {
        let mut lwp: *mut *mut listwatch_T = &raw mut (*l).lv_watch;
        let mut lw: *mut listwatch_T = (*l).lv_watch;
        while !lw.is_null() {
            if lw == lwrem {
                *lwp = (*lw).lw_next;
                break;
            } else {
                lwp = &raw mut (*lw).lw_next;
                lw = (*lw).lw_next;
            }
        }
    }
}

pub(crate) unsafe extern "C" fn tv_list_watch_fix(l: *mut list_T, item: *const listitem_T) {
    unsafe {
        let mut lw: *mut listwatch_T = (*l).lv_watch;
        while !lw.is_null() {
            if (*lw).lw_item == item as *mut listitem_T {
                (*lw).lw_item = (*item).li_next;
            }
            lw = (*lw).lw_next;
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_alloc(_len: ptrdiff_t) -> *mut list_T {
    unsafe {
        let list: *mut list_T =
            xcalloc(1 as size_t, ::core::mem::size_of::<list_T>()) as *mut list_T;
        if !(*gc_first_list.ptr()).is_null() {
            (*gc_first_list.get()).lv_used_prev = list;
        }
        (*list).lv_used_prev = ::core::ptr::null_mut::<list_T>();
        (*list).lv_used_next = gc_first_list.get();
        gc_first_list.set(list);
        (*list).lua_table_ref = LUA_NOREF as LuaRef;
        return list;
    }
}

pub unsafe extern "C" fn tv_list_init_static10(sl: *mut staticList10_T) {
    unsafe {
        let l: *mut list_T = &raw mut (*sl).sl_list;
        memset(
            sl as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<staticList10_T>(),
        );
        (*l).lv_first =
            (&raw mut (*sl).sl_items as *mut listitem_T).offset(0 as ::core::ffi::c_int as isize);
        (*l).lv_last = (&raw mut (*sl).sl_items as *mut listitem_T)
            .offset(SL_SIZE.wrapping_sub(1 as usize) as isize);
        (*l).lv_refcount = DO_NOT_FREE_CNT as ::core::ffi::c_int;
        tv_list_set_lock(l, VAR_FIXED);
        (*sl).sl_list.lv_len = 10 as ::core::ffi::c_int;
        (*sl).sl_items[0 as ::core::ffi::c_int as usize].li_prev =
            ::core::ptr::null_mut::<listitem_T>();
        (*sl).sl_items[0 as ::core::ffi::c_int as usize].li_next =
            (&raw mut (*sl).sl_items as *mut listitem_T).offset(1 as ::core::ffi::c_int as isize);
        (*sl).sl_items[SL_SIZE.wrapping_sub(1 as usize) as usize].li_prev =
            (&raw mut (*sl).sl_items as *mut listitem_T)
                .offset(SL_SIZE.wrapping_sub(2 as usize) as isize);
        (*sl).sl_items[SL_SIZE.wrapping_sub(1 as usize) as usize].li_next =
            ::core::ptr::null_mut::<listitem_T>();
        let mut i: size_t = 1 as size_t;
        while i < SL_SIZE.wrapping_sub(1 as usize) {
            let li: *mut listitem_T =
                (&raw mut (*sl).sl_items as *mut listitem_T).offset(i as isize);
            (*li).li_prev = li.offset(-(1 as ::core::ffi::c_int as isize));
            (*li).li_next = li.offset(1 as ::core::ffi::c_int as isize);
            i = i.wrapping_add(1);
        }
    }
}

pub unsafe extern "C" fn tv_list_init_static(l: *mut list_T) {
    unsafe {
        memset(
            l as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<list_T>(),
        );
        (*l).lv_refcount = DO_NOT_FREE_CNT as ::core::ffi::c_int;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_free_contents(l: *mut list_T) {
    unsafe {
        let mut item: *mut listitem_T = (*l).lv_first;
        while !item.is_null() {
            (*l).lv_first = (*item).li_next;
            tv_clear(&raw mut (*item).li_tv);
            xfree(item as *mut ::core::ffi::c_void);
            item = (*l).lv_first;
        }
        (*l).lv_len = 0 as ::core::ffi::c_int;
        (*l).lv_idx_item = ::core::ptr::null_mut::<listitem_T>();
        (*l).lv_last = ::core::ptr::null_mut::<listitem_T>();
        '_c2rust_label: {
            if (*l).lv_watch.is_null() {
            } else {
                __assert_fail(
                    b"l->lv_watch == NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    282 as ::core::ffi::c_uint,
                    b"void tv_list_free_contents(list_T *const)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_free_list(l: *mut list_T) {
    unsafe {
        if (*l).lv_used_prev.is_null() {
            gc_first_list.set((*l).lv_used_next);
        } else {
            (*(*l).lv_used_prev).lv_used_next = (*l).lv_used_next;
        }
        if !(*l).lv_used_next.is_null() {
            (*(*l).lv_used_next).lv_used_prev = (*l).lv_used_prev;
        }
        if (*l).lua_table_ref != LUA_NOREF {
            api_free_luaref((*l).lua_table_ref);
            (*l).lua_table_ref = LUA_NOREF as LuaRef;
        }
        xfree(l as *mut ::core::ffi::c_void);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_free(l: *mut list_T) {
    unsafe {
        if tv_in_free_unref_items.get() {
            return;
        }
        tv_list_free_contents(l);
        tv_list_free_list(l);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_unref(l: *mut list_T) {
    unsafe {
        if !l.is_null() && {
            (*l).lv_refcount -= 1;
            (*l).lv_refcount <= 0 as ::core::ffi::c_int
        } {
            tv_list_free(l);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_drop_items(
    l: *mut list_T,
    item: *mut listitem_T,
    item2: *mut listitem_T,
) {
    unsafe {
        let mut ip: *mut listitem_T = item;
        while ip != (*item2).li_next {
            (*l).lv_len -= 1;
            tv_list_watch_fix(l, ip);
            ip = (*ip).li_next;
        }
        if (*item2).li_next.is_null() {
            (*l).lv_last = (*item).li_prev;
        } else {
            (*(*item2).li_next).li_prev = (*item).li_prev;
        }
        if (*item).li_prev.is_null() {
            (*l).lv_first = (*item2).li_next;
        } else {
            (*(*item).li_prev).li_next = (*item2).li_next;
        }
        (*l).lv_idx_item = ::core::ptr::null_mut::<listitem_T>();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_remove_items(
    l: *mut list_T,
    item: *mut listitem_T,
    item2: *mut listitem_T,
) {
    unsafe {
        tv_list_drop_items(l, item, item2);
        let mut li: *mut listitem_T = item;
        loop {
            tv_clear(&raw mut (*li).li_tv);
            let nli: *mut listitem_T = (*li).li_next;
            xfree(li as *mut ::core::ffi::c_void);
            if li == item2 {
                break;
            }
            li = nli;
        }
    }
}

pub unsafe extern "C" fn tv_list_move_items(
    l: *mut list_T,
    item: *mut listitem_T,
    item2: *mut listitem_T,
    tgt_l: *mut list_T,
    cnt: ::core::ffi::c_int,
) {
    unsafe {
        tv_list_drop_items(l, item, item2);
        (*item).li_prev = (*tgt_l).lv_last;
        (*item2).li_next = ::core::ptr::null_mut::<listitem_T>();
        if (*tgt_l).lv_last.is_null() {
            (*tgt_l).lv_first = item;
        } else {
            (*(*tgt_l).lv_last).li_next = item;
        }
        (*tgt_l).lv_last = item2;
        (*tgt_l).lv_len += cnt;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_alloc_ret(ret_tv: *mut typval_T, len: ptrdiff_t) -> *mut list_T {
    unsafe {
        let l: *mut list_T = tv_list_alloc(len);
        tv_list_set_ret(ret_tv, l);
        (*ret_tv).v_lock = VAR_UNLOCKED;
        return l;
    }
}
