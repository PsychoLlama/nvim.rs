//! Filling a list, copying one, and finding an item in it.
//!
//! The `tv_list_append_*` family is the C header's overload set — one
//! function per value kind, each pushing an item onto the tail.
//! [`list_copy`] is `copy()`/`deepcopy()` over a list,
//! [`list_extend`] and [`list_concat`] the `extend()`/`+` pair, and
//! [`list_find`] the subscript `list[n]` resolves through, which counts
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

/// Filling a [`List`]: the `append` overload set, as methods.
impl List {
    /// Insert `item` at `at`, which must be `None` or an index of the list.
    fn insert_item(&mut self, item: ListItem, at: InsertAt) {
        let at = match at {
            Some(at) => {
                self.lv_items.insert(at, item);
                at
            }
            None => {
                self.lv_items.push(item);
                self.lv_items.len() - 1
            }
        };
        // Every cursor at or past the new item moves up one.
        watch_shift(self, index_of(at), 1);
    }

    /// Insert a copy of `tv` at `at`.
    ///
    /// The copy takes its own references, so `tv` stays the caller's -- and
    /// it is made *before* the array is touched, which is what lets `tv`
    /// borrow an item of the very list being inserted into.
    pub fn insert_copy(&mut self, tv: &TypVal, at: InsertAt) {
        let mut copy = TV_INITIAL_VALUE;
        tv_copy(tv, &mut copy);
        self.insert_item(ListItem::new(copy), at);
    }

    /// Append a copy of `tv`.
    pub fn push_copy(&mut self, tv: &TypVal) {
        self.insert_copy(tv, None);
    }

    /// Append `tv`, taking over whatever it owns.
    ///
    /// Answers the appended item's value, so the caller can keep filling it
    /// in.
    pub fn push(&mut self, tv: TypVal) -> &mut TypVal {
        self.insert_item(ListItem::new(tv), None);
        &mut self
            .lv_items
            .last_mut()
            .expect("the item just appended")
            .li_tv
    }

    /// Append `itemlist`, which the list takes the handle over.
    pub fn push_list(&mut self, itemlist: Option<ListRef>) {
        self.push(TypVal::list(itemlist));
    }

    /// Append `dict`, which the list takes the handle over.
    pub fn push_dict(&mut self, dict: Option<DictRef>) {
        self.push(TypVal::dict(dict));
    }

    /// Append a copy of `str`'s first `len` bytes.
    ///
    /// A negative `len` means the whole NUL-terminated string; a NULL `str`
    /// appends a NULL string.
    ///
    /// # Safety
    /// `str` is null, or readable for `len` bytes, or -- when `len` is
    /// negative -- NUL-terminated. The bytes are copied, so `str` stays the
    /// caller's.
    pub unsafe fn push_string(&mut self, str: *const ::core::ffi::c_char, len: ssize_t) {
        let copied = if str.is_null() {
            ::core::ptr::null_mut()
        } else if len >= 0 {
            // SAFETY: the caller's promise: `len` readable bytes.
            unsafe { xmemdupz(str.cast(), len.cast_unsigned()).cast::<::core::ffi::c_char>() }
        } else {
            // SAFETY: the caller's promise: NUL-terminated.
            unsafe { xstrdup(str) }
        };
        // SAFETY: the copy just made is this list's now.
        unsafe { self.push_allocated_string(copied) };
    }

    /// Append `str`, taking ownership of the allocation.
    ///
    /// # Safety
    /// `str` is null or an allocation from the `xmalloc` family. **The list
    /// takes it over**; the caller must not free it.
    pub unsafe fn push_allocated_string(&mut self, str: *mut ::core::ffi::c_char) {
        self.push(TypVal::String(str));
    }

    /// Append the number `n`.
    pub fn push_number(&mut self, n: VarNumber) {
        self.push(TypVal::Number(n));
    }
}

/// Copy `orig`, deeply when `deep`, converting strings through `conv`.
///
/// `copy_id` is the garbage collector's mark: non-zero records the copy on the
/// original *before* any item is added, so a list containing itself resolves
/// to the same copy.  Answers NULL when a deep copy of an item failed.
///
/// A **counted** handle rather than a borrow: `var_item_copy` re-enters the
/// evaluator, which can grow -- or release the last reference to -- the very
/// list being copied. The handle keeps it alive for the walk, and every item
/// read is a fresh, short-lived borrow through it.
///
/// # Safety
/// `conv` is null or a live converter. A non-zero `copy_id` must be one the
/// caller reserved from `get_copyID`: it is written onto `orig`, and a stale
/// one makes an unrelated walk believe this list is already visited.
pub unsafe fn list_copy(
    conv: *const VimConv,
    orig: Option<ListRef>,
    deep: bool,
    copy_id: ::core::ffi::c_int,
) -> Option<ListRef> {
    let mut orig = orig?;

    let mut copy = tv_list_alloc(ptrdiff_t::try_from(orig.len()).unwrap_or(-1));
    if copy_id != 0 {
        // Do this before adding the items, because one of the items may
        // refer back to this list.
        orig.lv_copy_id = copy_id;
        orig.lv_copylist = copy.as_ptr();
    }
    // By index, re-derived each step: a deep copy runs `var_item_copy`,
    // which can re-enter and grow the very list being copied.  The count is
    // taken once, as upstream's walk over the original links effectively
    // did.
    let len = orig.len();
    for at in 0..len {
        if got_int.get() {
            break;
        }
        let mut value = TV_INITIAL_VALUE;
        let from = &orig.items()[at].li_tv;
        if deep {
            // SAFETY: the caller's promise about `conv` and `copy_id`.
            if unsafe { var_item_copy(conv, from, &mut value, deep, copy_id) }.is_err() {
                // `tv_list_copy_error`: the partial copy goes with the
                // handle, which is the only reference to it.
                return None;
            }
        } else {
            tv_copy(from, &mut value);
        }
        copy.push(value);
    }
    Some(copy)
}

/// Splicing one list into another: the `extend()` half, and the aliasing
/// case it has to answer.
impl List {
    /// Insert copies of `src`'s items at `bef`, where `src` is a **different**
    /// list.
    ///
    /// The two borrows are what says so.  For `extend(l, l)` -- and for
    /// `l += l`, and for `flatten()` over a list holding itself -- the
    /// caller branches to [`List::extend_from_self`] instead: two live
    /// borrows of one list would be undefined where the pointer this
    /// replaced was merely delicate.
    pub(crate) fn extend_from(&mut self, src: &List, bef: InsertAt) {
        self.lv_items.reserve(src.len());
        for i in 0..src.len() {
            // The copy is made *before* anything is inserted, and the
            // insertion point walks along with the items already put in.
            self.insert_copy(&src.items()[i].li_tv, bef.map(|at| at + i));
        }
    }

    /// Insert copies of this list's *own* items at `bef`: `extend(l, l)`.
    ///
    /// The count is read once, so the walk copies what was there and does
    /// not run away.  The `i`th original item has `i` copies in front of it
    /// by the time its turn comes, so it now sits at `2 * i` -- unless it
    /// started before the insertion point, where nothing has moved.  That is
    /// upstream's `befbef`/`saved_next` bookkeeping, arithmetic instead of
    /// links.
    pub(crate) fn extend_from_self(&mut self, bef: InsertAt) {
        let todo = self.len();
        self.lv_items.reserve(todo);
        let bef_at = bef.unwrap_or(usize::MAX);
        for i in 0..todo {
            let src = if i >= bef_at { i + i } else { i };
            // The copy is made before the insert, which is what lets it read
            // the array it is about to write.
            let mut copy = TV_INITIAL_VALUE;
            tv_copy(&self.items()[src].li_tv, &mut copy);
            self.insert_item(ListItem::new(copy), bef.map(|at| at + i));
        }
    }
}

/// Insert copies of `src`'s items into `dest` at `bef`, where the two may be
/// the same list.
///
/// The branch is the whole function: `dest` and `src` are named by raw
/// pointers precisely because one borrow cannot answer both, and each arm
/// takes only the borrows it needs.
///
/// # Safety
/// `dest` and `src` must point at live lists -- possibly the same one --
/// with no other borrow of either live for the call, and `bef` must be
/// `None` or an index of `dest`.
pub unsafe fn list_extend(dest: *mut List, src: *const List, bef: InsertAt) {
    if ::core::ptr::eq(dest.cast_const(), src) {
        // SAFETY: the caller's promise: a live list.
        unsafe { &mut *dest }.extend_from_self(bef);
    } else {
        // SAFETY: as above, and the test above says the two are disjoint.
        unsafe { (*dest).extend_from(&*src, bef) };
    }
}

/// `l1 + l2`: store a shallow copy of the two lists joined in `tv`.
///
/// `tv` holds no value yet: it is overwritten, not cleared.
///
/// # Safety
/// `l1` and `l2` are each null or a live list the caller holds a reference
/// to; they may be the same list (`l + l`), which the copy makes harmless.
pub unsafe fn list_concat(l1: *mut List, l2: *mut List, tv: &mut TypVal) -> Result<(), Failed> {
    tv.write_empty(VAR_LIST);
    // SAFETY: the caller's promise: live lists, or null.
    let (held1, mut held2) = unsafe { (ListRef::retained(l1), ListRef::retained(l2)) };
    let l = if held1.is_none() && held2.is_none() {
        None
    } else if held1.is_none() {
        // SAFETY: no conversion, and the fresh-copy marker is 0.
        unsafe { list_copy(::core::ptr::null(), held2, false, 0) }
    } else {
        // SAFETY: as above.
        let mut l = unsafe { list_copy(::core::ptr::null(), held1, false, 0) };
        if let Some(ref mut l) = l
            && let Some(src) = held2.as_deref_mut()
        {
            // The copy is a list of this call's own, so it is never `src`.
            l.extend_from(src, None);
        }
        l
    };
    if l.is_none() && !(l1.is_null() && l2.is_null()) {
        return Err(Failed);
    }
    tv.write_list(l);
    Ok(())
}

/// `remove()` over a list: move one item, or the range `[idx, end]`, into
/// `result`.
///
/// # Safety
/// `args` must hold at least two values, the first a `VAR_LIST`.
/// `result` must be writable and hold no value yet, and `arg_errmsg` must be
/// a NUL-terminated string.
pub unsafe fn list_remove(
    list: Option<&mut List>,
    args: &[TypVal],
    result: &mut TypVal,
    arg_errmsg: *const ::core::ffi::c_char,
) {
    let translate = size_t::try_from(TV_TRANSLATE).unwrap_or(size_t::MAX);
    let lock = list.as_deref().map_or(VarLock::Fixed, List::lock);
    // SAFETY: the caller's promise: a NUL-terminated message.
    if unsafe { value_check_lock(lock, arg_errmsg, translate) } {
        return;
    }
    let Some(list) = list else { return };

    let Ok(idx) = tv_get_number_chk(&args[1]) else {
        // Type error: do nothing, errmsg already given.
        return;
    };
    let at = ::core::ffi::c_int::try_from(idx).ok();
    let Some(first) = at.and_then(|n| list_index(Some(list), n)) else {
        semsg!("E684: List index out of range: {}", idx);
        return;
    };

    if args.len() <= 2 {
        // Remove one item, return its value.
        let mut taken = list.take_range(first, first);
        *result = taken[0].li_tv.take();
        return;
    }

    // Remove range of items, return list with values.
    let Ok(end) = tv_get_number_chk(&args[2]) else {
        return;
    };
    let at = ::core::ffi::c_int::try_from(end).ok();
    let Some(last) = at.and_then(|n| list_index(Some(list), n)) else {
        semsg!("E684: List index out of range: {}", end);
        return;
    };
    if last < first {
        // Didn't find "item2" after "item".
        emsg(gettext(e_invrange));
        return;
    }
    let cnt = last - first + 1;
    let tgt = tv_list_alloc_ret(result, ptrdiff_t::try_from(cnt).unwrap_or(-1));
    // The target is a list of this call's own, so it is never `list`.
    list.move_range_to(first, last, tgt);
}

/// Whether `l1` and `l2` hold equal items in the same order.  An empty list and
/// a NULL one are equal.
///
/// Comparing values can recurse, so a cycle must already have been ruled out
/// by the caller's `copy_id` bookkeeping.
pub fn list_equal(l1: Option<&List>, l2: Option<&List>, ic: bool) -> bool {
    if l1.map(::core::ptr::from_ref) == l2.map(::core::ptr::from_ref) {
        return true;
    }
    let (len1, len2) = (list_len(l1), list_len(l2));
    if len1 != len2 {
        return false;
    }
    if len1 == 0 {
        // empty and NULL list are considered equal
        return true;
    }
    let (Some(l1), Some(l2)) = (l1, l2) else {
        return false;
    };

    // By index rather than by two zipped borrows: `tv_equal` can run a
    // user's `==` on a Blob or a Float and re-enter, and neither array may
    // be borrowed across that.
    for at in 0..l1.len() {
        if !tv_equal(&l1.items()[at].li_tv, &l2.items()[at].li_tv, ic) {
            return false;
        }
    }
    true
}

impl List {
    /// Reverse the list in place.  Every item moves, so nothing may be
    /// walking it.
    pub fn reverse(&mut self) {
        let len = self.len();
        if len <= 1 {
            return;
        }
        self.lv_items.reverse();
        // A cursor follows the item it stands on, which has been mirrored.
        if !self.lv_watch.is_null() {
            let mirrored: Vec<::core::ffi::c_int> = (0..len).rev().map(index_of).collect();
            watch_permute(self, &mirrored);
        }
    }
}

/// The item at index `n` of `l`, counting from the tail when `n` is
/// negative, or NULL when there is no such item.
///
/// The answer **borrows the list's item store** and is invalidated by any
/// edit to it.
pub fn list_find(l: Option<&mut List>, n: ::core::ffi::c_int) -> *mut ListItem {
    let Some(l) = l else {
        return ::core::ptr::null_mut();
    };
    let Some(at) = list_index(Some(l), n) else {
        return ::core::ptr::null_mut();
    };
    &raw mut l.items_mut()[at]
}

/// First item of `l`, or NULL when it is empty or NULL.
#[inline]
pub fn list_first(l: Option<&mut List>) -> *mut ListItem {
    list_find(l, 0)
}

/// Last item of `l`, or NULL when it is empty or NULL.
#[inline]
pub fn list_last(l: Option<&mut List>) -> *mut ListItem {
    list_find(l, -1)
}

/// [`list_find`] as an index: `n` normalised against `l`'s length, or `None`
/// when it names no item.
#[inline]
pub(crate) fn list_index(l: Option<&List>, n: ::core::ffi::c_int) -> Option<usize> {
    usize::try_from(list_uidx(l, n)).ok()
}

/// The number at index `n` of `l`.  Sets `*ret_error` when there is no such
/// item.
pub fn list_find_nr(
    l: Option<&List>,
    n: ::core::ffi::c_int,
    ret_error: Option<&mut bool>,
) -> VarNumber {
    let Some(at) = list_index(l, n) else {
        if let Some(ret_error) = ret_error {
            *ret_error = true;
        }
        return -1;
    };
    let item = &list_items(l)[at].li_tv;
    match (tv_get_number_chk(item), ret_error) {
        (Ok(n), _) => n,
        (Err(_), Some(flag)) => {
            *flag = true;
            0
        }
        (Err(_), None) => -1,
    }
}

/// The string at index `n` of `l`, or NULL with `E684` raised.
///
/// The string borrows the item, so it is only valid until the list changes;
/// raising `E684` goes through the editor's message state, so the caller
/// must be on the main thread.
pub fn list_find_str(
    l: Option<&List>,
    n: ::core::ffi::c_int,
    numbuf: &mut NumBuf,
) -> *const ::core::ffi::c_char {
    let Some(at) = list_index(l, n) else {
        semsg!("E684: List index out of range: {}", int64_t::from(n));
        return ::core::ptr::null();
    };
    numbuf.string_ptr(&list_items(l)[at].li_tv)
}

/// [`list_index`], clamping a negative index that fell off the front to 0.
///
/// `*idx` is updated to the index actually used.
pub(crate) fn list_find_index(l: Option<&List>, idx: &mut ::core::ffi::c_int) -> Option<usize> {
    if let Some(at) = list_index(l, *idx) {
        return Some(at);
    }
    if *idx < 0 {
        *idx = 0;
        return list_index(l, 0);
    }
    None
}
