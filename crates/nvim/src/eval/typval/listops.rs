//! Filling a list, copying one, and finding an item in it.
//!
//! The `tv_list_append_*` family is the C header's overload set — one
//! function per value kind, each pushing an item onto the tail.
//! [`list_copy`] is `copy()`/`deepcopy()` over a list,
//! [`list_extend`] and [`list_concat`] the `extend()`/`+` pair, and
//! [`list_find`] the subscript `list[n]` resolves through, which counts
//! from the tail for a negative index and is a bounds check and an array
//! index — the list owns its items, so there is nothing to walk.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::eval::collect::var_item_copy_with;
use crate::memory::ThinCString;
use crate::semsg;
use crate::types::Failed;
use core::ffi::CStr;

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

    /// Append a copy of `text`; `None` appends a NULL string.
    pub fn push_bytes(&mut self, text: Option<&[u8]>) {
        self.push(TypVal::string(text.map(ThinCString::from_bytes)));
    }

    /// Append a copy of the NUL-terminated `text`; `None` appends a NULL
    /// string.
    pub fn push_str(&mut self, text: Option<&CStr>) {
        self.push(TypVal::string(text.map(ThinCString::from_cstr)));
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
/// to the same copy.  Answers `None` when a deep copy of an item failed.
/// A non-zero `copy_id` must be one the caller reserved from `get_copyID`:
/// a stale one makes an unrelated walk believe this list is already visited.
///
/// A deep copy re-enters through `var_item_copy`, and a list that holds
/// itself is read again from in there -- through the mark this call has just
/// written onto it -- so the walk is by index over the handle, each item read
/// through a fresh borrow.
pub fn list_copy(
    conv: Option<&VimConv>,
    orig: &ListRef,
    deep: bool,
    copy_id: ::core::ffi::c_int,
) -> Option<ListRef> {
    let mut copy = tv_list_alloc(ptrdiff_t::try_from(orig.len()).unwrap_or(-1));
    if copy_id != 0 {
        // Do this before adding the items, because one of the items may
        // refer back to this list.
        orig.remember_copy(copy_id, &copy);
    }
    // The count is taken once, as upstream's walk over the original links
    // effectively did.
    let len = orig.len();
    for at in 0..len {
        if got_int.get() {
            break;
        }
        let mut value = TV_INITIAL_VALUE;
        let from = &orig.items()[at].li_tv;
        if deep {
            if var_item_copy_with(conv, from, &mut value, deep, copy_id).is_err() {
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

/// Insert copies of `src`'s items into `dest` at `bef`, where the two may be
/// the same list: `extend(l, l)`, `l += l`, and `flatten()` over a list
/// holding itself.
///
/// Each copy is made before the list it goes into is borrowed, and that
/// borrow lasts one statement: a copy of an item naming either list takes a
/// reference to it. For the same list, the items are all copied first, the
/// count taken before -- so the walk copies what was there and does not run
/// away, which is what upstream's `befbef`/`saved_next` bookkeeping bought.
///
/// A NULL `src` is the empty list -- `extend(l, v:_null_list)` -- and
/// extends nothing. `bef` must be `None` or an index of `dest`.
pub fn list_extend(dest: &ListRef, src: Option<&ListRef>, bef: InsertAt) {
    let Some(src) = src else {
        return;
    };
    if dest.ptr_eq(src) {
        let copies: Vec<TypVal> = (0..src.len())
            .map(|at| src.items()[at].li_tv.clone())
            .collect();
        dest.edit().lv_items.reserve(copies.len());
        for (i, copy) in copies.into_iter().enumerate() {
            dest.edit()
                .insert_item(ListItem::new(copy), bef.map(|at| at + i));
        }
    } else {
        let count = src.len();
        dest.edit().lv_items.reserve(count);
        for i in 0..count {
            // The insertion point walks along with the items already put in.
            let copy = src.items()[i].li_tv.clone();
            dest.edit()
                .insert_item(ListItem::new(copy), bef.map(|at| at + i));
        }
    }
}

/// `l1 + l2`: store a shallow copy of the two lists joined in `tv`.
///
/// `tv` holds no value yet: it is overwritten, not cleared. The two may be
/// the same list (`l + l`), which the copy makes harmless.
pub fn list_concat(
    l1: Option<&ListRef>,
    l2: Option<&ListRef>,
    tv: &mut TypVal,
) -> Result<(), Failed> {
    tv.write_empty(VAR_LIST);
    let l = match (l1, l2) {
        (None, None) => None,
        (None, Some(l2)) => list_copy(None, l2, false, 0),
        (Some(l1), l2) => {
            let l = list_copy(None, l1, false, 0);
            if let Some(l) = &l {
                // The copy is a list of this call's own, so it is never `l2`.
                list_extend(l, l2, None);
            }
            l
        }
    };
    if l.is_none() && !(l1.is_none() && l2.is_none()) {
        return Err(Failed);
    }
    tv.write_list(l);
    Ok(())
}

/// [`list_concat`] of two List values: `tv1 + tv2` into `tv`.
pub(crate) fn list_concat_values(
    tv1: &TypVal,
    tv2: &TypVal,
    tv: &mut TypVal,
) -> Result<(), Failed> {
    list_concat(tv1.list_shared(), tv2.list_shared(), tv)
}

/// `remove()` over a list: move one item, or the range `[idx, end]`, into
/// `result`, which holds no value yet. `args` holds at least two values,
/// the first a `VAR_LIST`; `arg_errmsg` names the argument in a lock error,
/// translated.
pub fn list_remove(
    list: Option<&mut List>,
    args: &[TypVal],
    result: &mut TypVal,
    arg_errmsg: &'static CStr,
) {
    let lock = list.as_deref().map_or(VarLock::Fixed, List::lock);
    if value_check_lock_named(lock, gettext(arg_errmsg).to_bytes()) {
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
        if self.is_watched() {
            let mirrored: Vec<::core::ffi::c_int> = (0..len).rev().map(index_of).collect();
            watch_permute(self, &mirrored);
        }
    }
}

/// The item at index `n` of `l`, counting from the tail when `n` is
/// negative, or `None` when there is no such item.
pub fn list_find(l: Option<&mut List>, n: ::core::ffi::c_int) -> Option<&mut ListItem> {
    let l = l?;
    let at = list_index(Some(l), n)?;
    Some(&mut l.items_mut()[at])
}

/// First item of `l`, or `None` when it is empty or NULL.
#[inline]
pub fn list_first(l: Option<&mut List>) -> Option<&mut ListItem> {
    list_find(l, 0)
}

/// Last item of `l`, or `None` when it is empty or NULL.
#[inline]
pub fn list_last(l: Option<&mut List>) -> Option<&mut ListItem> {
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

/// The string at index `n` of `l`, or `None` with `E684` raised.
///
/// The string borrows the item, so it is only valid until the list changes;
/// raising `E684` goes through the editor's message state, so the caller
/// must be on the main thread.
pub fn list_find_str<'a>(
    l: Option<&'a List>,
    n: ::core::ffi::c_int,
    numbuf: &'a mut NumBuf,
) -> Option<&'a CStr> {
    let Some(at) = list_index(l, n) else {
        semsg!("E684: List index out of range: {}", int64_t::from(n));
        return None;
    };
    Some(numbuf.string(&list_items(l)[at].li_tv))
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
