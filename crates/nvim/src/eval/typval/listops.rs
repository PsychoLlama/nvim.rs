//! Filling a list, copying one, and finding an item in it.
//!
//! The `tv_list_append_*` family is the C header's overload set — one
//! function per value kind, each allocating a `listitem_T` and linking it at
//! the tail.  [`tv_list_copy`] is `copy()`/`deepcopy()` over a list,
//! [`tv_list_extend`] and [`tv_list_concat`] the `extend()`/`+` pair, and
//! [`tv_list_find`] the index walk `list[n]` resolves through, which counts
//! from the tail for a negative index.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::semsg;
use crate::types::{FAIL, OK};

/// Link `ni` into `l` in front of `item`, or at the tail when `item` is NULL.
///
/// # Safety
/// `l` must point at a live list, `ni` at a fresh item on no list, and
/// `item` must be null or an item **of `l`** — the links are rewritten
/// around it, so an item from another list corrupts both.
pub unsafe fn tv_list_insert(l: *mut list_T, ni: *mut listitem_T, item: *mut listitem_T) {
    // SAFETY: the caller's promise: a live list.
    let mut list = unsafe { Ls::new(l) };
    if item.is_null() {
        // Append new item at end of list.
        unsafe { tv_list_append(l, ni) };
        return;
    }

    // Insert new item before existing item.
    unsafe { (*ni).li_prev = (*item).li_prev };
    unsafe { (*ni).li_next = item };
    match unsafe { (*item).li_prev.as_mut() } {
        Some(before) => {
            before.li_next = ni;
            // The cached index now names the wrong item.
            unsafe { (*l).lv_idx_item = ::core::ptr::null_mut() };
        }
        None => {
            list.lv_first = ni;
            // Everything shifted up by one, the cache included.
            list.lv_idx += 1;
        }
    }
    // SAFETY: the caller's promise: an item of `l`.
    let mut before = unsafe { Li::new(item) };
    before.li_prev = ni;
    list.lv_len += 1;
}

/// Insert a copy of `tv` into `l` in front of `item`.
///
/// # Safety
/// `l` must point at a live list, `tv` at a value that is safe to copy, and
/// `item` must be null or an item of `l`. The copy takes its own
/// references, so `tv` stays the caller's.
pub unsafe fn tv_list_insert_tv(l: *mut list_T, tv: *mut typval_T, item: *mut listitem_T) {
    let ni = tv_list_item_alloc();
    unsafe { tv_copy(tv, &raw mut (*ni).li_tv) };
    unsafe { tv_list_insert(l, ni, item) };
}

/// Link `item` onto `l`'s tail.
///
/// # Safety
/// `l` must point at a live list and `item` at a fresh item that is on no
/// list. The list takes it over.
pub unsafe fn tv_list_append(l: *mut list_T, item: *mut listitem_T) {
    // SAFETY: the caller's promise: a live list.
    let mut list = unsafe { Ls::new(l) };
    match unsafe { (*l).lv_last.as_mut() } {
        Some(last) => {
            last.li_next = item;
            unsafe { (*item).li_prev = (*l).lv_last };
        }
        None => {
            list.lv_first = item;
            unsafe { (*item).li_prev = ::core::ptr::null_mut() };
        }
    }
    list.lv_last = item;
    list.lv_len += 1;
    unsafe { (*item).li_next = ::core::ptr::null_mut() };
}

/// Append a copy of `tv` to `l`.
///
/// # Safety
/// `l` must point at a live list and `tv` at a value that is safe to copy;
/// `tv` stays the caller's.
pub unsafe fn tv_list_append_tv(l: *mut list_T, tv: *mut typval_T) {
    let li = tv_list_item_alloc();
    unsafe { tv_copy(tv, &raw mut (*li).li_tv) };
    unsafe { tv_list_append(l, li) };
}

/// Append `tv` to `l`, taking over whatever it owns.
///
/// Answers the appended item's value, so the caller can keep filling it in.
///
/// # Safety
/// `l` must point at a live list, and `tv` must be a value whose references
/// and allocations the caller is giving up — the list owns them now. The
/// returned pointer borrows the item and is invalidated by anything that
/// removes it.
pub unsafe fn tv_list_append_owned_tv(l: *mut list_T, tv: typval_T) -> *mut typval_T {
    let li = tv_list_item_alloc();
    unsafe { (*li).li_tv = tv };
    unsafe { tv_list_append(l, li) };
    li_tv(li)
}

/// Append `itemlist` to `l`, taking a reference to it.
///
/// # Safety
/// `l` must point at a live list, and `itemlist` is null or a live list. A
/// reference to `itemlist` is taken.
pub unsafe fn tv_list_append_list(l: *mut list_T, itemlist: *mut list_T) {
    unsafe { tv_list_append_owned_tv(l, typval_T::list(itemlist)) };
    unsafe { tv_list_ref(itemlist) };
}

/// Append `dict` to `l`, taking a reference to it.
///
/// # Safety
/// `l` must point at a live list, and `dict` is null or a live dictionary.
/// A reference to `dict` is taken.
pub unsafe fn tv_list_append_dict(l: *mut list_T, dict: *mut dict_T) {
    unsafe { tv_list_append_owned_tv(l, typval_T::dict(dict)) };
    if let Some(dict) = unsafe { dict.as_mut() } {
        dict.dv_refcount.retain();
    }
}

/// Append a copy of `str`'s first `len` bytes to `l`.
///
/// A negative `len` means the whole NUL-terminated string; a NULL `str`
/// appends a NULL string.
///
/// # Safety
/// `l` must point at a live list. `str` is null, or readable for `len`
/// bytes, or — when `len` is negative — NUL-terminated. The bytes are
/// copied, so `str` stays the caller's.
pub unsafe fn tv_list_append_string(l: *mut list_T, str: *const ::core::ffi::c_char, len: ssize_t) {
    let copied = if str.is_null() {
        ::core::ptr::null_mut()
    } else if len >= 0 {
        unsafe { xmemdupz(str.cast(), len as size_t) as *mut ::core::ffi::c_char }
    } else {
        unsafe { xstrdup(str) }
    };
    unsafe { tv_list_append_allocated_string(l, copied) };
}

/// Append `str` to `l`, taking ownership of the allocation.
///
/// # Safety
/// `l` must point at a live list, and `str` is null or an allocation from
/// the `xmalloc` family. **The list takes it over**; the caller must not
/// free it.
pub unsafe fn tv_list_append_allocated_string(l: *mut list_T, str: *mut ::core::ffi::c_char) {
    unsafe { tv_list_append_owned_tv(l, typval_T::string(str)) };
}

/// Append the number `n` to `l`.
///
/// # Safety
/// `l` must point at a live list.
pub unsafe fn tv_list_append_number(l: *mut list_T, n: varnumber_T) {
    unsafe { tv_list_append_owned_tv(l, typval_T::number(n)) };
}

/// Copy `orig`, deeply when `deep`, converting strings through `conv`.
///
/// `copyID` is the garbage collector's mark: non-zero records the copy on the
/// original *before* any item is added, so a list containing itself resolves
/// to the same copy.  Answers NULL when a deep copy of an item failed.
///
/// # Safety
/// `orig` is null or a live list and `conv` is null or a live converter. A
/// non-zero `copyID` must be one the caller reserved from `get_copyID`: it
/// is written onto `orig`, and a stale one makes an unrelated walk believe
/// this list is already visited.
pub unsafe fn tv_list_copy(
    conv: *const vimconv_T,
    orig: *mut list_T,
    deep: bool,
    copyID: ::core::ffi::c_int,
) -> *mut list_T {
    if orig.is_null() {
        return ::core::ptr::null_mut();
    }

    let copy = unsafe { tv_list_alloc(tv_list_len(orig) as ptrdiff_t) };
    unsafe { tv_list_ref(copy) };
    if copyID != 0 {
        // Do this before adding the items, because one of the items may
        // refer back to this list.
        // SAFETY: the caller's promise: a live list.
        let mut from = unsafe { Ls::new(orig) };
        from.lv_copyID = copyID;
        from.lv_copylist = copy;
    }
    for item in tv_list_iter(unsafe { orig.as_ref() }) {
        if got_int.get() {
            break;
        }
        let ni = tv_list_item_alloc();
        if deep {
            let from = li_tv(item);
            let to = li_tv(ni);
            if unsafe { var_item_copy(conv, from, to, deep, copyID) } == FAIL {
                // `tv_list_copy_error`: the partial copy goes too.
                unsafe { xfree(ni.cast()) };
                unsafe { tv_list_unref(copy) };
                return ::core::ptr::null_mut();
            }
        } else {
            unsafe { tv_copy(&raw mut (*item).li_tv, &raw mut (*ni).li_tv) };
        }
        unsafe { tv_list_append(copy, ni) };
    }
    copy
}

/// Insert copies of `l2`'s items into `l1` in front of `bef`.
///
/// # Safety
/// `l1` and `l2` must point at live lists, and `bef` must be null or an
/// item of `l1`. `l1` and `l2` may be the same list — the walk stops after
/// the original item count for exactly that case.
pub unsafe fn tv_list_extend(l1: *mut list_T, l2: *mut list_T, bef: *mut listitem_T) {
    let mut todo = unsafe { tv_list_len(l2) };
    let befbef = if bef.is_null() {
        ::core::ptr::null_mut()
    } else {
        unsafe { (*bef).li_prev }
    };
    let saved_next = if befbef.is_null() {
        ::core::ptr::null_mut()
    } else {
        unsafe { (*befbef).li_next }
    };
    // Quit once the original item count has been inserted, so that
    // extending a list with itself does not hang.  The walk is hand-rolled
    // rather than a `tv_list_iter`, because the item's own `li_next` is
    // relinked by the insertion above it.
    let mut item = unsafe { tv_list_first(l2) };
    while !item.is_null() && todo != 0 {
        todo -= 1;
        unsafe { tv_list_insert_tv(l1, &raw mut (*item).li_tv, bef) };
        item = if item == befbef {
            saved_next
        } else {
            unsafe { (*item).li_next }
        };
    }
}

/// `l1 + l2`: store a shallow copy of the two lists joined in `tv`.
///
/// # Safety
/// `l1` and `l2` are each null or a live list, and `tv` must point at a
/// writable `typval_T` holding no value yet.
pub unsafe fn tv_list_concat(
    l1: *mut list_T,
    l2: *mut list_T,
    tv: *mut typval_T,
) -> ::core::ffi::c_int {
    // SAFETY: the caller's promise: a writable typval.
    let mut val = unsafe { Tv::new(tv) };
    val.v_type = VAR_LIST;
    val.v_lock = VarLock::Unlocked;
    let l = if l1.is_null() && l2.is_null() {
        ::core::ptr::null_mut()
    } else if l1.is_null() {
        unsafe { tv_list_copy(::core::ptr::null(), l2, false, 0) }
    } else {
        let l = unsafe { tv_list_copy(::core::ptr::null(), l1, false, 0) };
        if !l.is_null() && !l2.is_null() {
            unsafe { tv_list_extend(l, l2, ::core::ptr::null_mut()) };
        }
        l
    };
    if l.is_null() && !(l1.is_null() && l2.is_null()) {
        return FAIL;
    }
    val.vval.v_list = l;
    OK
}

/// `remove()` over a list: move one item, or the range `[idx, end]`, into
/// `rettv`.
///
/// # Safety
/// `argvars` must point at at least three values, the first a `VAR_LIST`
/// and the third the `VAR_UNKNOWN` terminator when no range was given.
/// `rettv` must be writable and hold no value yet, and `arg_errmsg` must be
/// a NUL-terminated string.
pub unsafe fn tv_list_remove(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    arg_errmsg: *const ::core::ffi::c_char,
) {
    let l = unsafe { (*argvars).vval.v_list };
    if unsafe { value_check_lock(tv_list_locked(l), arg_errmsg, TV_TRANSLATE as size_t) } {
        return;
    }

    let mut error = false;
    let idx = unsafe { tv_get_number_chk(argvars.add(1), &raw mut error) };
    if error {
        // Type error: do nothing, errmsg already given.
        return;
    }
    let item = unsafe { tv_list_find(l, idx as ::core::ffi::c_int) };
    if item.is_null() {
        semsg!("E684: List index out of range: {}", idx);
        return;
    }

    if unsafe { (*argvars.add(2)).v_type } == VAR_UNKNOWN {
        // Remove one item, return its value.
        unsafe { tv_list_drop_items(l, item, item) };
        unsafe { *rettv = (*item).li_tv };
        unsafe { xfree(item.cast()) };
        return;
    }

    // Remove range of items, return list with values.
    let end = unsafe { tv_get_number_chk(argvars.add(2), &raw mut error) };
    if error {
        return;
    }
    let item2 = unsafe { tv_list_find(l, end as ::core::ffi::c_int) };
    if item2.is_null() {
        semsg!("E684: List index out of range: {}", end);
        return;
    }

    let mut cnt = 0;
    let mut li = item;
    while !li.is_null() {
        cnt += 1;
        if li == item2 {
            break;
        }
        li = unsafe { (*li).li_next };
    }
    if li.is_null() {
        // Didn't find "item2" after "item".
        emsg(gettext(e_invrange));
    } else {
        let tgt = unsafe { tv_list_alloc_ret(rettv, cnt as ptrdiff_t) };
        unsafe { tv_list_move_items(l, item, item2, tgt, cnt) };
    }
}

/// Whether `l1` and `l2` hold equal items in the same order.  An empty list and
/// a NULL one are equal.
///
/// # Safety
/// `l1` and `l2` are each null or a live list. Comparing values can
/// recurse, so a cycle must already have been ruled out by the caller's
/// `copyID` bookkeeping.
pub unsafe fn tv_list_equal(l1: *mut list_T, l2: *mut list_T, ic: bool) -> bool {
    if l1 == l2 {
        return true;
    }
    let len1 = unsafe { tv_list_len(l1) };
    if len1 != unsafe { tv_list_len(l2) } {
        return false;
    }
    if len1 == 0 {
        // empty and NULL list are considered equal
        return true;
    }
    if l1.is_null() || l2.is_null() {
        return false;
    }

    let mut item1 = unsafe { tv_list_first(l1) };
    let mut item2 = unsafe { tv_list_first(l2) };
    while !item1.is_null() && !item2.is_null() {
        if !unsafe { tv_equal(&raw mut (*item1).li_tv, &raw mut (*item2).li_tv, ic) } {
            return false;
        }
        item1 = unsafe { (*item1).li_next };
        item2 = unsafe { (*item2).li_next };
    }
    // The lengths matched, so both walks ended together.
    debug_assert!(item1.is_null() && item2.is_null());
    true
}

/// Reverse `l` in place.
///
/// # Safety
/// `l` is null or points at a live list. Every item's links are rewritten,
/// so nothing may be walking the list.
pub unsafe fn tv_list_reverse(l: *mut list_T) {
    if unsafe { tv_list_len(l) } <= 1 {
        return;
    }
    // SAFETY: the caller's promise: a live list.
    let mut list = unsafe { Ls::new(l) };
    unsafe { ::core::mem::swap(&mut (*l).lv_first, &mut (*l).lv_last) };
    let mut li = list.lv_first;
    while !li.is_null() {
        // SAFETY: an item of the list being reversed.
        unsafe { ::core::mem::swap(&mut (*li).li_next, &mut (*li).li_prev) };
        // `li_next` now holds what `li_prev` did, which is the direction
        // this walk goes.
        li = unsafe { (*li).li_next };
    }
    list.lv_idx = list.lv_len - list.lv_idx - 1;
}

/// The item at index `n` of `l`, counting from the tail when `n` is negative.
///
/// Caches the index it lands on in the list, and starts the next walk from
/// whichever of the head, the tail and that cache is nearest.
///
/// # Safety
/// `l` is null or points at a live list. The item borrows the list; the
/// index cache this writes into `l` is invalidated by any change to the
/// list's shape, which the mutating entry points here take care of.
pub unsafe fn tv_list_find(l: *mut list_T, n: ::core::ffi::c_int) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut();
    }

    let n = unsafe { tv_list_uidx(l, n) };
    if n == -1 {
        return ::core::ptr::null_mut();
    }

    // When there is a cached index may start search from there.
    // SAFETY: the caller's promise: a live list.
    let mut list = unsafe { Ls::new(l) };
    let (mut item, mut idx) = if !list.lv_idx_item.is_null() {
        if n < list.lv_idx / 2 {
            // Closest to the start of the list.
            (list.lv_first, 0)
        } else if n > (list.lv_idx + list.lv_len) / 2 {
            // Closest to the end of the list.
            (list.lv_last, list.lv_len - 1)
        } else {
            // Closest to the cached index.
            (list.lv_idx_item, list.lv_idx)
        }
    } else if n < list.lv_len / 2 {
        (list.lv_first, 0)
    } else {
        (list.lv_last, list.lv_len - 1)
    };

    while n > idx {
        // Search forward.
        item = unsafe { (*item).li_next };
        idx += 1;
    }
    while n < idx {
        // Search backward.
        item = unsafe { (*item).li_prev };
        idx -= 1;
    }
    debug_assert!(idx == n);

    // Cache the used index.
    list.lv_idx = idx;
    list.lv_idx_item = item;
    item
}

/// The number at index `n` of `l`.  Sets `*ret_error` when there is no such
/// item.
///
/// # Safety
/// `l` is null or points at a live list, and `ret_error` is null or points
/// at a writable `bool`.
pub unsafe fn tv_list_find_nr(
    l: *mut list_T,
    n: ::core::ffi::c_int,
    ret_error: *mut bool,
) -> varnumber_T {
    let li = unsafe { tv_list_find(l, n) };
    if li.is_null() {
        if let Some(ret_error) = unsafe { ret_error.as_mut() } {
            *ret_error = true;
        }
        return -1;
    }
    unsafe { tv_get_number_chk(&raw const (*li).li_tv, ret_error) }
}

/// The string at index `n` of `l`, or NULL with `E684` raised.
///
/// # Safety
/// `l` is null or points at a live list. The string borrows the item, so
/// it is only valid until the list changes; raising `E684` goes through the
/// editor's message state, so the caller must be on the main thread.
pub unsafe fn tv_list_find_str(
    l: *mut list_T,
    n: ::core::ffi::c_int,
    numbuf: &mut NumBuf,
) -> *const ::core::ffi::c_char {
    let li = unsafe { tv_list_find(l, n) };
    if li.is_null() {
        semsg!("E684: List index out of range: {}", n as int64_t);
        return ::core::ptr::null();
    }
    unsafe { numbuf.string(&raw const (*li).li_tv) }
}

/// [`tv_list_find`], clamping a negative index that fell off the front to 0.
///
/// `*idx` is updated to the index actually used.
///
/// # Safety
/// `l` is null or points at a live list, and `idx` must point at a writable
/// `c_int`, which is updated to the index actually used.
pub(crate) unsafe fn tv_list_find_index(
    l: *mut list_T,
    idx: *mut ::core::ffi::c_int,
) -> *mut listitem_T {
    let li = unsafe { tv_list_find(l, *idx) };
    if !li.is_null() {
        return li;
    }
    if unsafe { *idx } < 0 {
        unsafe { *idx = 0 };
        return unsafe { tv_list_find(l, *idx) };
    }
    li
}

/// The index of `item` in `l`, or -1 when it is not there.
///
/// # Safety
/// `l` is null or points at a live list. `item` is only compared, never
/// read, so it may be any pointer.
pub unsafe fn tv_list_idx_of_item(l: *const list_T, item: *const listitem_T) -> ::core::ffi::c_int {
    if l.is_null() {
        return -1;
    }
    for (idx, li) in tv_list_iter(unsafe { l.as_ref() }).enumerate() {
        if li.cast_const() == item {
            return idx as ::core::ffi::c_int;
        }
    }
    -1
}
