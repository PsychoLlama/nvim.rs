//! Filling a list, copying one, and finding an item in it.
//!
//! The `tv_list_append_*` family is the C header's overload set — one
//! function per value kind, each pushing an item onto the tail.
//! [`tv_list_copy`] is `copy()`/`deepcopy()` over a list,
//! [`tv_list_extend`] and [`tv_list_concat`] the `extend()`/`+` pair, and
//! [`tv_list_find`] the subscript `list[n]` resolves through, which counts
//! from the tail for a negative index and is a bounds check and an array
//! index — the list owns its items, so there is nothing to walk.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::semsg;
use crate::types::Failed;

/// Where an insertion goes: in front of the item at this index, or at the
/// tail when it is `None`.
///
/// Upstream spelled the tail as a NULL `ListItem *`; an index is what
/// survives the items being owned by the list.
pub type InsertAt = Option<usize>;

/// Insert `item` into `l` at `at`.
///
/// # Safety
/// `l` must point at a live list, unaliased for the call, and `at` must be
/// `None` or an index into it.
unsafe fn insert_item(l: *mut List, item: ListItem, at: InsertAt) {
    // SAFETY: the caller's promise: a live, unaliased list.
    let items = unsafe { &mut (*l).lv_items };
    let at = match at {
        Some(at) => {
            items.insert(at, item);
            at
        }
        None => {
            items.push(item);
            items.len() - 1
        }
    };
    // SAFETY: as above; every cursor at or past the new item moves up one.
    unsafe { tv_list_watch_shift(l, index_of(at), 1) };
}

/// Insert a copy of `tv` into `l` at `at`.
///
/// # Safety
/// `l` must point at a live list, `tv` be a value that is safe to copy, and
/// `at` be `None` or an index of `l`. The copy takes its own references, so
/// `tv` stays the caller's.
pub unsafe fn tv_list_insert_tv(l: *mut List, tv: &TypVal, at: InsertAt) {
    let mut copy = TV_INITIAL_VALUE;
    // SAFETY: the caller's promise: a value that is safe to copy, into this
    // frame's own slot.
    unsafe { tv_copy(tv, &mut copy) };
    unsafe { insert_item(l, ListItem::new(copy), at) };
}

/// Append a copy of `tv` to `l`.
///
/// # Safety
/// `l` must point at a live list and `tv` be a value that is safe to copy;
/// `tv` stays the caller's.
pub unsafe fn tv_list_append_tv(l: *mut List, tv: &TypVal) {
    unsafe { tv_list_insert_tv(l, tv, None) };
}

/// Append `tv` to `l`, taking over whatever it owns.
///
/// Answers the appended item's value, so the caller can keep filling it in.
///
/// # Safety
/// `l` must point at a live list, and `tv` must be a value whose references
/// and allocations the caller is giving up — the list owns them now. The
/// returned pointer borrows the item store and is invalidated by anything
/// that edits the list.
pub unsafe fn tv_list_append_owned_tv(l: *mut List, tv: TypVal) -> *mut TypVal {
    unsafe { insert_item(l, ListItem::new(tv), None) };
    // SAFETY: the item just pushed, which is the last one.
    let items = unsafe { &mut (*l).lv_items };
    &raw mut items.last_mut().expect("the item just appended").li_tv
}

/// Append `itemlist` to `l`, which takes the handle over.
///
/// # Safety
/// `l` must point at a live list.
pub unsafe fn tv_list_append_list(l: *mut List, itemlist: Option<ListRef>) {
    unsafe { tv_list_append_owned_tv(l, TypVal::list(itemlist)) };
}

/// Append `dict` to `l`, which takes the handle over.
///
/// # Safety
/// `l` must point at a live list.
pub unsafe fn tv_list_append_dict(l: *mut List, dict: Option<DictRef>) {
    unsafe { tv_list_append_owned_tv(l, TypVal::dict(dict)) };
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
pub unsafe fn tv_list_append_string(l: *mut List, str: *const ::core::ffi::c_char, len: ssize_t) {
    let copied = if str.is_null() {
        ::core::ptr::null_mut()
    } else if len >= 0 {
        unsafe { xmemdupz(str.cast(), len.cast_unsigned()).cast::<::core::ffi::c_char>() }
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
pub unsafe fn tv_list_append_allocated_string(l: *mut List, str: *mut ::core::ffi::c_char) {
    unsafe { tv_list_append_owned_tv(l, TypVal::String(str)) };
}

/// Append the number `n` to `l`.
///
/// # Safety
/// `l` must point at a live list.
pub unsafe fn tv_list_append_number(l: *mut List, n: VarNumber) {
    unsafe { tv_list_append_owned_tv(l, TypVal::Number(n)) };
}

/// Copy `orig`, deeply when `deep`, converting strings through `conv`.
///
/// `copy_id` is the garbage collector's mark: non-zero records the copy on the
/// original *before* any item is added, so a list containing itself resolves
/// to the same copy.  Answers NULL when a deep copy of an item failed.
///
/// # Safety
/// `orig` is null or a live list and `conv` is null or a live converter. A
/// non-zero `copy_id` must be one the caller reserved from `get_copyID`: it
/// is written onto `orig`, and a stale one makes an unrelated walk believe
/// this list is already visited.
pub unsafe fn tv_list_copy(
    conv: *const VimConv,
    orig: *mut List,
    deep: bool,
    copy_id: ::core::ffi::c_int,
) -> Option<ListRef> {
    if orig.is_null() {
        return None;
    }

    let copy = tv_list_alloc(ptrdiff_t::try_from(unsafe { tv_list_len(orig) }).unwrap_or(-1));
    // A borrow of the list the handle owns, for the items to go into.
    let into = copy.as_ptr();
    if copy_id != 0 {
        // Do this before adding the items, because one of the items may
        // refer back to this list.
        // SAFETY: the caller's promise: a live list.
        let mut from = unsafe { Ls::new(orig) };
        from.lv_copy_id = copy_id;
        from.lv_copylist = into;
    }
    // By index: a deep copy runs `var_item_copy`, which can re-enter and
    // grow the very list being copied.  The count is taken once, as
    // upstream's walk over the original links effectively did.
    // SAFETY: the caller's promise: a live list.
    let len = unsafe { tv_list_items(orig) }.len();
    for at in 0..len {
        if got_int.get() {
            break;
        }
        let mut value = TV_INITIAL_VALUE;
        // SAFETY: `at` is inside the list, whose items nothing has moved.
        let from = &unsafe { tv_list_items(orig) }[at].li_tv;
        if deep {
            if unsafe { var_item_copy(conv, from, &mut value, deep, copy_id) }.is_err() {
                // `tv_list_copy_error`: the partial copy goes with the
                // handle, which is the only reference to it.
                return None;
            }
        } else {
            unsafe { tv_copy(from, &mut value) };
        }
        unsafe { tv_list_append_owned_tv(into, value) };
    }
    Some(copy)
}

/// Insert copies of `l2`'s items into `l1` at `bef`.
///
/// # Safety
/// `l1` and `l2` must point at live lists, and `bef` must be `None` or an
/// index of `l1`. `l1` and `l2` may be the same list — the walk stops after
/// the original item count for exactly that case.
pub unsafe fn tv_list_extend(l1: *mut List, l2: *mut List, bef: InsertAt) {
    // The count is read once, so that extending a list with itself copies
    // what was there and does not hang.
    let todo = unsafe { tv_list_items(l2) }.len();
    // SAFETY: the caller's promise: a live list.
    unsafe { &mut (*l1).lv_items }.reserve(todo);
    // Extending a list with *itself* moves its own items along as the
    // copies go in: the `i`th original item has `i` copies in front of it
    // by the time its turn comes, so it now sits at `2 * i` -- unless it
    // started before the insertion point, where nothing has moved.  That is
    // upstream's `befbef`/`saved_next` bookkeeping, arithmetic instead of
    // links.
    let itself = ::core::ptr::eq(l1, l2);
    let bef_at = bef.unwrap_or(usize::MAX);
    for i in 0..todo {
        let src = if itself && i >= bef_at { i + i } else { i };
        // SAFETY: `src` is inside `l2` as it now stands.
        let from = &unsafe { tv_list_items(l2) }[src].li_tv;
        // The copy is made *before* anything is inserted, which is what
        // lets `from` borrow the array being written to.
        // SAFETY: the caller's promise about `l1` and `bef`; the insertion
        // point walks along with the items already put in.
        unsafe { tv_list_insert_tv(l1, from, bef.map(|at| at + i)) };
    }
}

/// `l1 + l2`: store a shallow copy of the two lists joined in `tv`.
///
/// # Safety
/// `l1` and `l2` are each null or a live list, and `tv` must point at a
/// writable `TypVal` holding no value yet.
pub unsafe fn tv_list_concat(l1: *mut List, l2: *mut List, tv: &mut TypVal) -> Result<(), Failed> {
    // SAFETY: the caller's promise: a writable typval.
    let mut val = unsafe { Tv::new(tv) };
    val.write_empty(VAR_LIST);
    let l = if l1.is_null() && l2.is_null() {
        None
    } else if l1.is_null() {
        unsafe { tv_list_copy(::core::ptr::null(), l2, false, 0) }
    } else {
        let l = unsafe { tv_list_copy(::core::ptr::null(), l1, false, 0) };
        if let Some(ref l) = l
            && !l2.is_null()
        {
            unsafe { tv_list_extend(l.as_ptr(), l2, None) };
        }
        l
    };
    if l.is_none() && !(l1.is_null() && l2.is_null()) {
        return Err(Failed);
    }
    val.write_list(l);
    Ok(())
}

/// `remove()` over a list: move one item, or the range `[idx, end]`, into
/// `result`.
///
/// # Safety
/// `args` must hold at least two values, the first a `VAR_LIST`.
/// `result` must be writable and hold no value yet, and `arg_errmsg` must be
/// a NUL-terminated string.
pub unsafe fn tv_list_remove(
    args: &[TypVal],
    result: &mut TypVal,
    arg_errmsg: *const ::core::ffi::c_char,
) {
    let l = args[0].list_or_null();
    let translate = size_t::try_from(TV_TRANSLATE).unwrap_or(size_t::MAX);
    if unsafe { value_check_lock(tv_list_locked(l), arg_errmsg, translate) } {
        return;
    }

    let mut error = false;
    let idx = unsafe { tv_get_number_chk(&args[1], &raw mut error) };
    if error {
        // Type error: do nothing, errmsg already given.
        return;
    }
    let at = ::core::ffi::c_int::try_from(idx).ok();
    let Some(first) = at.and_then(|n| unsafe { tv_list_index(l, n) }) else {
        semsg!("E684: List index out of range: {}", idx);
        return;
    };

    if args.len() <= 2 {
        // Remove one item, return its value.
        // SAFETY: a live list and an index of it.
        let mut taken = unsafe { tv_list_take_range(l, first, first) };
        *result = taken[0].li_tv.take();
        return;
    }

    // Remove range of items, return list with values.
    let end = unsafe { tv_get_number_chk(&args[2], &raw mut error) };
    if error {
        return;
    }
    let at = ::core::ffi::c_int::try_from(end).ok();
    let Some(last) = at.and_then(|n| unsafe { tv_list_index(l, n) }) else {
        semsg!("E684: List index out of range: {}", end);
        return;
    };
    if last < first {
        // Didn't find "item2" after "item".
        emsg(gettext(e_invrange));
        return;
    }
    let cnt = last - first + 1;
    // SAFETY: `result` is the caller's return slot.
    let tgt = tv_list_alloc_ret(result, ptrdiff_t::try_from(cnt).unwrap_or(-1));
    // SAFETY: a live list, a run of its items, and a fresh target list.
    unsafe { tv_list_move_range(l, first, last, tgt) };
}

/// Whether `l1` and `l2` hold equal items in the same order.  An empty list and
/// a NULL one are equal.
///
/// # Safety
/// `l1` and `l2` are each null or a live list. Comparing values can
/// recurse, so a cycle must already have been ruled out by the caller's
/// `copy_id` bookkeeping.
pub unsafe fn tv_list_equal(l1: *mut List, l2: *mut List, ic: bool) -> bool {
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

    // By index rather than by two zipped borrows: `tv_equal` can run a
    // user's `==` on a Blob or a Float and re-enter, and neither array may
    // be borrowed across that.
    // SAFETY: the caller's promise: two live lists of the same length.
    for at in 0..unsafe { tv_list_items(l1) }.len() {
        // SAFETY: as above -- `at` is inside both lists.
        let (a, b) = unsafe { (&tv_list_items(l1)[at], &tv_list_items(l2)[at]) };
        if !unsafe { tv_equal(&a.li_tv, &b.li_tv, ic) } {
            return false;
        }
    }
    true
}

/// Reverse `l` in place.
///
/// # Safety
/// `l` is null or points at a live list. Every item moves, so nothing may be
/// walking the list.
pub unsafe fn tv_list_reverse(l: *mut List) {
    // SAFETY: the caller's promise: a live list.
    let list = unsafe { &mut *l };
    let len = list.lv_items.len();
    if len <= 1 {
        return;
    }
    list.lv_items.reverse();
    // A cursor follows the item it stands on, which has been mirrored.
    if !list.lv_watch.is_null() {
        let mirrored: Vec<::core::ffi::c_int> = (0..len).rev().map(index_of).collect();
        tv_list_watch_permute(list, &mirrored);
    }
}

/// The item at index `n` of `l`, counting from the tail when `n` is
/// negative, or NULL when there is no such item.
///
/// # Safety
/// `l` is null or points at a live list. The item borrows the list's item
/// store and is invalidated by any edit to it.
pub unsafe fn tv_list_find(l: *mut List, n: ::core::ffi::c_int) -> *mut ListItem {
    let Some(at) = (unsafe { tv_list_index(l, n) }) else {
        return ::core::ptr::null_mut();
    };
    // SAFETY: an index this call just bounds-checked.
    let items = unsafe { tv_list_items_mut(l) };
    &raw mut items[at]
}

/// First item of `l`, or NULL when it is empty or NULL.
///
/// # Safety
/// As [`tv_list_find`].
#[inline]
pub unsafe fn tv_list_first(l: *mut List) -> *mut ListItem {
    unsafe { tv_list_find(l, 0) }
}

/// Last item of `l`, or NULL when it is empty or NULL.
///
/// # Safety
/// As [`tv_list_find`].
#[inline]
pub unsafe fn tv_list_last(l: *mut List) -> *mut ListItem {
    unsafe { tv_list_find(l, -1) }
}

/// [`tv_list_find`] as an index: `n` normalised against `l`'s length, or
/// `None` when it names no item.
///
/// # Safety
/// `l` is null or points at a live list.
#[inline]
pub(crate) unsafe fn tv_list_index(l: *const List, n: ::core::ffi::c_int) -> Option<usize> {
    usize::try_from(unsafe { tv_list_uidx(l, n) }).ok()
}

/// The number at index `n` of `l`.  Sets `*ret_error` when there is no such
/// item.
///
/// # Safety
/// `l` is null or points at a live list, and `ret_error` is null or points
/// at a writable `bool`.
pub unsafe fn tv_list_find_nr(
    l: *mut List,
    n: ::core::ffi::c_int,
    ret_error: *mut bool,
) -> VarNumber {
    let Some(at) = (unsafe { tv_list_index(l, n) }) else {
        if let Some(ret_error) = unsafe { ret_error.as_mut() } {
            *ret_error = true;
        }
        return -1;
    };
    // SAFETY: an index just bounds-checked, and the caller's error cell.
    unsafe { tv_get_number_chk(&tv_list_items(l)[at].li_tv, ret_error) }
}

/// The string at index `n` of `l`, or NULL with `E684` raised.
///
/// # Safety
/// `l` is null or points at a live list. The string borrows the item, so
/// it is only valid until the list changes; raising `E684` goes through the
/// editor's message state, so the caller must be on the main thread.
pub unsafe fn tv_list_find_str(
    l: *mut List,
    n: ::core::ffi::c_int,
    numbuf: &mut NumBuf,
) -> *const ::core::ffi::c_char {
    let Some(at) = (unsafe { tv_list_index(l, n) }) else {
        semsg!("E684: List index out of range: {}", int64_t::from(n));
        return ::core::ptr::null();
    };
    // SAFETY: an index just bounds-checked.
    unsafe { numbuf.string(&tv_list_items(l)[at].li_tv) }
}

/// [`tv_list_index`], clamping a negative index that fell off the front to 0.
///
/// `*idx` is updated to the index actually used.
///
/// # Safety
/// `l` is null or points at a live list, and `idx` must point at a writable
/// `c_int`, which is updated to the index actually used.
pub(crate) unsafe fn tv_list_find_index(
    l: *mut List,
    idx: *mut ::core::ffi::c_int,
) -> Option<usize> {
    if let Some(at) = unsafe { tv_list_index(l, *idx) } {
        return Some(at);
    }
    if unsafe { *idx } < 0 {
        unsafe { *idx = 0 };
        return unsafe { tv_list_index(l, 0) };
    }
    None
}
