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
use crate::types::Refcount;

/// Safe: it takes nothing and reads nothing.  `xmalloc` either answers an
/// allocation or aborts, so the only obligation left is the ambient one every
/// allocation in the editor carries — being on the main thread — and *using*
/// what comes back is the caller's business, not this call's.
pub(crate) fn tv_list_item_alloc() -> *mut listitem_T {
    unsafe { xmalloc(::core::mem::size_of::<listitem_T>()) as *mut listitem_T }
}

/// Remove `item` from `l`, clear its value and free it.
///
/// Answers the item that followed it, or NULL when it was the last one.
pub unsafe fn tv_list_item_remove(l: *mut list_T, item: *mut listitem_T) -> *mut listitem_T {
    let next_item = unsafe { (*item).li_next };
    unsafe { tv_list_drop_items(l, item, item) };
    unsafe { tv_clear(&raw mut (*item).li_tv) };
    unsafe { xfree(item.cast()) };
    next_item
}

/// Push `lw` onto `l`'s watcher chain.
pub unsafe fn tv_list_watch_add(l: *mut list_T, lw: *mut listwatch_T) {
    unsafe { (*lw).lw_next = (*l).lv_watch };
    unsafe { (*l).lv_watch = lw };
}

/// Unlink `lwrem` from `l`'s watcher chain.
pub unsafe fn tv_list_watch_remove(l: *mut list_T, lwrem: *mut listwatch_T) {
    // `lwp` trails `lw` by one link so the match can be spliced out.
    let mut lwp = lv_watch(l);
    let mut lw = unsafe { (*l).lv_watch };
    while !lw.is_null() {
        if lw == lwrem {
            unsafe { *lwp = (*lw).lw_next };
            break;
        }
        // SAFETY: an entry of `l`'s watcher chain.
        let mut watch = unsafe { Lw::new(lw) };
        lwp = &raw mut watch.lw_next;
        lw = watch.lw_next;
    }
}

/// Advance any watcher standing on `item` to the item after it.
///
/// This is what keeps a `:for` loop walking a list whose current item is
/// removed underneath it.
pub(crate) unsafe fn tv_list_watch_fix(l: *mut list_T, item: *const listitem_T) {
    let mut lw = unsafe { (*l).lv_watch };
    while !lw.is_null() {
        // SAFETY: an entry of `l`'s watcher chain.
        let watch = unsafe { Lw::new(lw) };
        if watch.lw_item.cast_const() == item {
            unsafe { (*lw).lw_item = (*item).li_next };
        }
        lw = watch.lw_next;
    }
}

/// Allocate an empty list.  The caller owns the reference count.
///
/// `len` is upstream's hint for a future array-backed list; nothing reads it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_alloc(_len: ptrdiff_t) -> *mut list_T {
    let list = unsafe { xcalloc(1, ::core::mem::size_of::<list_T>()) } as *mut list_T;

    // Prepend the list to the list of lists for garbage collection.
    if let Some(first) = unsafe { gc_first_list.get().as_mut() } {
        first.lv_used_prev = list;
    }
    unsafe { (*list).lv_used_prev = ::core::ptr::null_mut() };
    unsafe { (*list).lv_used_next = gc_first_list.get() };
    gc_first_list.set(list);
    unsafe { (*list).lua_table_ref = LUA_NOREF as LuaRef };
    list
}

/// Initialise a stack-allocated ten-item list, all items zeroed and linked.
///
/// The list is `VarLock::Fixed` and carries `DO_NOT_FREE_CNT`, so nothing frees it.
pub unsafe fn tv_list_init_static10(sl: *mut staticList10_T) {
    // No `Live<staticList10_T>` here: the list this builds points at the item
    // array in the *same* struct, and a `DerefMut` that reborrows the whole
    // struct pops those interior pointers under Stacked and Tree Borrows.
    unsafe { sl.write_bytes(0, 1) };
    let l = unsafe { &raw mut (*sl).sl_list };
    let items = unsafe { &raw mut (*sl).sl_items }.cast::<listitem_T>();

    unsafe { (*l).lv_first = items };
    unsafe { (*l).lv_last = items.add(SL_SIZE - 1) };
    unsafe { (*l).lv_refcount = Refcount::new(DO_NOT_FREE_CNT as ::core::ffi::c_int) };
    unsafe { tv_list_set_lock(l, VarLock::Fixed) };
    unsafe { (*l).lv_len = 10 };

    unsafe { (*items).li_prev = ::core::ptr::null_mut() };
    unsafe { (*items).li_next = items.add(1) };
    unsafe { (*items.add(SL_SIZE - 1)).li_prev = items.add(SL_SIZE - 2) };
    unsafe { (*items.add(SL_SIZE - 1)).li_next = ::core::ptr::null_mut() };

    for i in 1..SL_SIZE - 1 {
        let li = unsafe { items.add(i) };
        unsafe { (*li).li_prev = li.sub(1) };
        unsafe { (*li).li_next = li.add(1) };
    }
}

/// Initialise a stack-allocated empty list that nothing may free.
pub unsafe fn tv_list_init_static(l: *mut list_T) {
    unsafe { l.write_bytes(0, 1) };
    unsafe { (*l).lv_refcount = Refcount::new(DO_NOT_FREE_CNT as ::core::ffi::c_int) };
}

/// Free every item in `l`, leaving the list itself allocated and empty.
pub unsafe fn tv_list_free_contents(l: *mut list_T) {
    // Unlink each item before clearing it: `tv_clear` can re-enter.
    // SAFETY: the caller's promise: a live list.
    let mut list = unsafe { Ls::new(l) };
    let mut item = list.lv_first;
    while !item.is_null() {
        unsafe { (*l).lv_first = (*item).li_next };
        unsafe { tv_clear(&raw mut (*item).li_tv) };
        unsafe { xfree(item.cast()) };
        item = list.lv_first;
    }
    list.lv_len = 0;
    unsafe { (*l).lv_idx_item = ::core::ptr::null_mut() };
    unsafe { (*l).lv_last = ::core::ptr::null_mut() };
    debug_assert!(list.lv_watch.is_null());
}

/// Unlink `l` from the garbage collector's chain and free the `list_T` itself.
pub unsafe fn tv_list_free_list(l: *mut list_T) {
    // Remove the list from the list of lists for garbage collection.
    // SAFETY: the caller's promise: a live list.
    let mut list = unsafe { Ls::new(l) };
    match unsafe { (*l).lv_used_prev.as_mut() } {
        Some(prev) => prev.lv_used_next = list.lv_used_next,
        None => gc_first_list.set(list.lv_used_next),
    }
    if let Some(next) = unsafe { (*l).lv_used_next.as_mut() } {
        next.lv_used_prev = list.lv_used_prev;
    }

    // NLUA_CLEAR_REF
    if list.lua_table_ref != LUA_NOREF {
        unsafe { api_free_luaref((*l).lua_table_ref) };
        list.lua_table_ref = LUA_NOREF as LuaRef;
    }
    unsafe { xfree(l.cast()) };
}

/// Free `l` and everything in it.  A no-op while `free_unref_items()` is
/// walking, which frees the whole graph itself.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_list_free(l: *mut list_T) {
    if tv_in_free_unref_items.get() {
        return;
    }
    unsafe { tv_list_free_contents(l) };
    unsafe { tv_list_free_list(l) };
}

/// Drop a reference to `l`, freeing it when the last one goes.
pub unsafe fn tv_list_unref(l: *mut list_T) {
    if let Some(list) = unsafe { l.as_mut() }
        && list.lv_refcount.release() <= 0
    {
        unsafe { tv_list_free(l) };
    }
}

/// Unlink the items `item..=item2` from `l` without freeing them.
pub unsafe fn tv_list_drop_items(l: *mut list_T, item: *mut listitem_T, item2: *mut listitem_T) {
    // Notify watchers.
    let mut ip = item;
    // SAFETY: the caller's promise: an item of `l`.
    let last = unsafe { Li::new(item2) };
    while ip != last.li_next {
        unsafe { (*l).lv_len -= 1 };
        unsafe { tv_list_watch_fix(l, ip) };
        ip = unsafe { (*ip).li_next };
    }

    // SAFETY: the caller's promise: an item of `l`.
    let first = unsafe { Li::new(item) };
    match unsafe { (*item2).li_next.as_mut() } {
        Some(after) => after.li_prev = first.li_prev,
        None => unsafe { (*l).lv_last = (*item).li_prev },
    }
    match unsafe { (*item).li_prev.as_mut() } {
        Some(before) => before.li_next = last.li_next,
        None => unsafe { (*l).lv_first = (*item2).li_next },
    }
    unsafe { (*l).lv_idx_item = ::core::ptr::null_mut() };
}

/// Unlink the items `item..=item2` from `l` and free them.
pub unsafe fn tv_list_remove_items(l: *mut list_T, item: *mut listitem_T, item2: *mut listitem_T) {
    unsafe { tv_list_drop_items(l, item, item2) };
    let mut li = item;
    loop {
        unsafe { tv_clear(&raw mut (*li).li_tv) };
        // Read the link before the free, not after.
        let nli = unsafe { (*li).li_next };
        unsafe { xfree(li.cast()) };
        if li == item2 {
            break;
        }
        li = nli;
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
    unsafe { tv_list_drop_items(l, item, item2) };
    unsafe { (*item).li_prev = (*tgt_l).lv_last };
    unsafe { (*item2).li_next = ::core::ptr::null_mut() };
    // SAFETY: the caller's promise: a live target list.
    let mut tgt = unsafe { Ls::new(tgt_l) };
    match unsafe { (*tgt_l).lv_last.as_mut() } {
        Some(last) => last.li_next = item,
        None => tgt.lv_first = item,
    }
    tgt.lv_last = item2;
    tgt.lv_len += cnt;
}

/// Allocate an empty list and store it in `ret_tv` as the return value.
pub unsafe fn tv_list_alloc_ret(ret_tv: *mut typval_T, len: ptrdiff_t) -> *mut list_T {
    let l = unsafe { tv_list_alloc(len) };
    unsafe { tv_list_set_ret(ret_tv, l) };
    unsafe { (*ret_tv).v_lock = VarLock::Unlocked };
    l
}
