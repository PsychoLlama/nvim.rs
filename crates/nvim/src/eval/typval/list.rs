//! Allocating, freeing and unlinking a `list_T` and its `listitem_T`s.
//!
//! [`tv_list_alloc`] and [`tv_list_free`] are the reference-counted pair,
//! [`tv_list_unref`] the one every caller actually uses.  The `listwatch_T`
//! half ([`tv_list_watch_add`], [`tv_list_watch_fix`]) is how a `:for` loop
//! survives having the item it is standing on removed underneath it, and
//! [`tv_list_drop_items`] / [`tv_list_move_items`] are the two ways items
//! leave a list.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;

pub(crate) unsafe fn tv_list_item_alloc() -> *mut listitem_T {
    unsafe { xmalloc(::core::mem::size_of::<listitem_T>()) as *mut listitem_T }
}

/// Remove `item` from `l`, clear its value and free it.
///
/// Answers the item that followed it, or NULL when it was the last one.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_item_remove(
    l: *mut list_T,
    item: *mut listitem_T,
) -> *mut listitem_T {
    unsafe {
        let next_item = (*item).li_next;
        tv_list_drop_items(l, item, item);
        tv_clear(&raw mut (*item).li_tv);
        xfree(item.cast());
        next_item
    }
}

/// Push `lw` onto `l`'s watcher chain.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_watch_add(l: *mut list_T, lw: *mut listwatch_T) {
    unsafe {
        (*lw).lw_next = (*l).lv_watch;
        (*l).lv_watch = lw;
    }
}

/// Unlink `lwrem` from `l`'s watcher chain.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_watch_remove(l: *mut list_T, lwrem: *mut listwatch_T) {
    unsafe {
        // `lwp` trails `lw` by one link so the match can be spliced out.
        let mut lwp = &raw mut (*l).lv_watch;
        let mut lw = (*l).lv_watch;
        while !lw.is_null() {
            if lw == lwrem {
                *lwp = (*lw).lw_next;
                break;
            }
            lwp = &raw mut (*lw).lw_next;
            lw = (*lw).lw_next;
        }
    }
}

/// Advance any watcher standing on `item` to the item after it.
///
/// This is what keeps a `:for` loop walking a list whose current item is
/// removed underneath it.
pub(crate) unsafe fn tv_list_watch_fix(l: *mut list_T, item: *const listitem_T) {
    unsafe {
        let mut lw = (*l).lv_watch;
        while !lw.is_null() {
            if (*lw).lw_item.cast_const() == item {
                (*lw).lw_item = (*item).li_next;
            }
            lw = (*lw).lw_next;
        }
    }
}

/// Allocate an empty list.  The caller owns the reference count.
///
/// `len` is upstream's hint for a future array-backed list; nothing reads it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_alloc(_len: ptrdiff_t) -> *mut list_T {
    unsafe {
        let list = xcalloc(1, ::core::mem::size_of::<list_T>()) as *mut list_T;

        // Prepend the list to the list of lists for garbage collection.
        if let Some(first) = gc_first_list.get().as_mut() {
            first.lv_used_prev = list;
        }
        (*list).lv_used_prev = ::core::ptr::null_mut();
        (*list).lv_used_next = gc_first_list.get();
        gc_first_list.set(list);
        (*list).lua_table_ref = LUA_NOREF as LuaRef;
        list
    }
}

/// Initialise a stack-allocated ten-item list, all items zeroed and linked.
///
/// The list is `VAR_FIXED` and carries `DO_NOT_FREE_CNT`, so nothing frees it.
pub unsafe fn tv_list_init_static10(sl: *mut staticList10_T) {
    unsafe {
        let l = &raw mut (*sl).sl_list;
        let items = (&raw mut (*sl).sl_items).cast::<listitem_T>();

        sl.write_bytes(0, 1);
        (*l).lv_first = items;
        (*l).lv_last = items.add(SL_SIZE - 1);
        (*l).lv_refcount = DO_NOT_FREE_CNT as ::core::ffi::c_int;
        tv_list_set_lock(l, VAR_FIXED);
        (*sl).sl_list.lv_len = 10;

        (*items).li_prev = ::core::ptr::null_mut();
        (*items).li_next = items.add(1);
        (*items.add(SL_SIZE - 1)).li_prev = items.add(SL_SIZE - 2);
        (*items.add(SL_SIZE - 1)).li_next = ::core::ptr::null_mut();

        for i in 1..SL_SIZE - 1 {
            let li = items.add(i);
            (*li).li_prev = li.sub(1);
            (*li).li_next = li.add(1);
        }
    }
}

/// Initialise a stack-allocated empty list that nothing may free.
pub unsafe fn tv_list_init_static(l: *mut list_T) {
    unsafe {
        l.write_bytes(0, 1);
        (*l).lv_refcount = DO_NOT_FREE_CNT as ::core::ffi::c_int;
    }
}

/// Free every item in `l`, leaving the list itself allocated and empty.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_free_contents(l: *mut list_T) {
    unsafe {
        // Unlink each item before clearing it: `tv_clear` can re-enter.
        let mut item = (*l).lv_first;
        while !item.is_null() {
            (*l).lv_first = (*item).li_next;
            tv_clear(&raw mut (*item).li_tv);
            xfree(item.cast());
            item = (*l).lv_first;
        }
        (*l).lv_len = 0;
        (*l).lv_idx_item = ::core::ptr::null_mut();
        (*l).lv_last = ::core::ptr::null_mut();
        debug_assert!((*l).lv_watch.is_null());
    }
}

/// Unlink `l` from the garbage collector's chain and free the `list_T` itself.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_free_list(l: *mut list_T) {
    unsafe {
        // Remove the list from the list of lists for garbage collection.
        match (*l).lv_used_prev.as_mut() {
            Some(prev) => prev.lv_used_next = (*l).lv_used_next,
            None => gc_first_list.set((*l).lv_used_next),
        }
        if let Some(next) = (*l).lv_used_next.as_mut() {
            next.lv_used_prev = (*l).lv_used_prev;
        }

        // NLUA_CLEAR_REF
        if (*l).lua_table_ref != LUA_NOREF {
            api_free_luaref((*l).lua_table_ref);
            (*l).lua_table_ref = LUA_NOREF as LuaRef;
        }
        xfree(l.cast());
    }
}

/// Free `l` and everything in it.  A no-op while `free_unref_items()` is
/// walking, which frees the whole graph itself.
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

/// Drop a reference to `l`, freeing it when the last one goes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_unref(l: *mut list_T) {
    unsafe {
        if let Some(list) = l.as_mut() {
            list.lv_refcount -= 1;
            if list.lv_refcount <= 0 {
                tv_list_free(l);
            }
        }
    }
}

/// Unlink the items `item..=item2` from `l` without freeing them.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_drop_items(
    l: *mut list_T,
    item: *mut listitem_T,
    item2: *mut listitem_T,
) {
    unsafe {
        // Notify watchers.
        let mut ip = item;
        while ip != (*item2).li_next {
            (*l).lv_len -= 1;
            tv_list_watch_fix(l, ip);
            ip = (*ip).li_next;
        }

        match (*item2).li_next.as_mut() {
            Some(after) => after.li_prev = (*item).li_prev,
            None => (*l).lv_last = (*item).li_prev,
        }
        match (*item).li_prev.as_mut() {
            Some(before) => before.li_next = (*item2).li_next,
            None => (*l).lv_first = (*item2).li_next,
        }
        (*l).lv_idx_item = ::core::ptr::null_mut();
    }
}

/// Unlink the items `item..=item2` from `l` and free them.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_remove_items(
    l: *mut list_T,
    item: *mut listitem_T,
    item2: *mut listitem_T,
) {
    unsafe {
        tv_list_drop_items(l, item, item2);
        let mut li = item;
        loop {
            tv_clear(&raw mut (*li).li_tv);
            // Read the link before the free, not after.
            let nli = (*li).li_next;
            xfree(li.cast());
            if li == item2 {
                break;
            }
            li = nli;
        }
    }
}

/// Move the items `item..=item2` (`cnt` of them) from `l` onto `tgt_l`'s tail.
pub unsafe fn tv_list_move_items(
    l: *mut list_T,
    item: *mut listitem_T,
    item2: *mut listitem_T,
    tgt_l: *mut list_T,
    cnt: ::core::ffi::c_int,
) {
    unsafe {
        tv_list_drop_items(l, item, item2);
        (*item).li_prev = (*tgt_l).lv_last;
        (*item2).li_next = ::core::ptr::null_mut();
        match (*tgt_l).lv_last.as_mut() {
            Some(last) => last.li_next = item,
            None => (*tgt_l).lv_first = item,
        }
        (*tgt_l).lv_last = item2;
        (*tgt_l).lv_len += cnt;
    }
}

/// Allocate an empty list and store it in `ret_tv` as the return value.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_alloc_ret(ret_tv: *mut typval_T, len: ptrdiff_t) -> *mut list_T {
    unsafe {
        let l = tv_list_alloc(len);
        tv_list_set_ret(ret_tv, l);
        (*ret_tv).v_lock = VAR_UNLOCKED;
        l
    }
}
