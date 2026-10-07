//! The Vimscript builtins that work over a whole container.
//!
//! Carved by what the builtin does to it:
//!
//! | child | what |
//! | --- | --- |
//! | [`filtermap`] | `filter()`, `map()`, `mapnew()`, `foreach()` -- and, one level down, the four per-container walks |
//! | [`count`] | `count()` and `add()` |
//! | [`extend`] | `extend()`, `extendnew()`, `insert()` |
//!
//! What stays here is `remove()` and `reverse()` -- the two that only take
//! something out of a container or turn it around -- plus the two shared
//! error texts and the safe layer the whole family is written against.
//!
//! # The safe layer
//!
//! [`ListArg`], [`DictArg`] and [`BlobArg`] (and their item types) each
//! borrow the *handle* the argument holds -- never the container -- and
//! reach the container through it one statement at a time (`edit()`), so
//! the builtins are ordinary safe Rust.  [`Container::of`] is the one place
//! the `TypVal` union is read, under the `v_type` that names the live arm.
//!
//! # Re-entrancy
//!
//! Nothing here caches anything across a call that can run Vimscript, and
//! the four walks run one per item.  An [`Item`] is a list and an *index*,
//! and a [`DictItemRef`] a dictionary and a *slot*, so each re-derives its
//! item after every callback rather than holding an address the callback
//! could have invalidated; and an item's value crosses into the evaluator
//! as a copy in `v:val`, made before the callback runs.
//!
//! Original: `src/nvim/eval/list.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::eval::typval::CallFrame;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::strings::reversed_text;
use core::ffi::{CStr, c_int};

use crate::eval::typval::{
    BlobRef, DictCursor, DictRef, ListRef, LockName, NumBuf, blob_copy, blob_remove, dict_copy,
    dict_extend, index_of, list_copy, list_extend, list_index, list_remove,
    tv_check_for_string_or_list_or_blob_arg, tv_clear, tv_copy, tv_dict_alloc_ret,
    tv_dict_item_remove, tv_dict_remove, tv_equal, tv_get_number_chk, tv_list_alloc_ret,
    value_check_lock,
};
use crate::eval::vars::{
    clear_vimvar, prepare_vimvar, restore_vimvar, set_vim_var_bytes, set_vim_var_nr,
    set_vim_var_tv, set_vim_var_type, var_check_fixed_named, var_check_ro_named, with_vim_var,
};
use crate::eval::{eval_expr_typval, get_copy_id};
use crate::ex_docmd::do_cmdline_cmd;
use crate::mbyte::{cluster_len, strnicmp_in};
use crate::memory::ThinCString;
use crate::message::e_listdictblobarg;
use crate::message::emsg;
use crate::message_fmt::{emsg_text, msg_cstr};
use crate::os::cshim::gettext;
use crate::tr_c;
use crate::types::{
    Blob, Dict, DictItem, EvalFuncData, List, ListItem, TypVal, VAR_BLOB, VAR_DICT, VAR_LIST,
    VAR_STRING, VarLock, VarNumber, VarType, Vv, int64_t, ptrdiff_t, uint8_t,
};

// The carve of the transpiled module; see each child's docs.
mod count;
mod extend;
mod filtermap;

pub use self::count::{f_add, f_count};
pub use self::extend::{f_extend, f_extendnew, f_insert};
pub use self::filtermap::{f_filter, f_foreach, f_map, f_mapnew};

static e_argument_of_str_must_be_list_string_or_dictionary: &CStr =
    c"E706: Argument of %s must be a List, String or Dictionary";
static e_argument_of_str_must_be_list_string_dictionary_or_blob: &CStr =
    c"E1250: Argument of %s must be a List, String, Dictionary or Blob";

/// A cleared `TypVal`, the `{ .v_type = VAR_UNKNOWN }` every walk starts
/// its per-item result from.
pub(crate) const UNKNOWN_TV: TypVal = TV_INITIAL_VALUE;

// ---------------------------------------------------------------------
// The container a typval holds
// ---------------------------------------------------------------------

/// Which container a `TypVal` holds, read from the arm its `v_type` says
/// is live.  Everything else -- Number, Float, Funcref, ... -- is
/// [`Container::Other`], which is what the family's type errors report.
#[derive(Clone, Copy)]
pub(crate) enum Container<'a> {
    List(ListArg<'a>),
    Dict(DictArg<'a>),
    Blob(BlobArg<'a>),
    /// A String's text, or `None` for `v:_null_string`.
    Str(Option<&'a CStr>),
    Other,
}

impl<'a> Container<'a> {
    /// Read `tv`'s live union arm.
    #[inline(always)]
    pub(crate) fn of(tv: &'a TypVal) -> Self {
        match tv.v_type() {
            VAR_LIST => Self::List(ListArg(tv.list_shared())),
            VAR_DICT => Self::Dict(DictArg(tv.dict_shared())),
            VAR_BLOB => Self::Blob(BlobArg(tv.blob_shared())),
            VAR_STRING => Self::Str(tv.string_ref().map(ThinCString::as_cstr)),
            _ => Self::Other,
        }
    }
}

// ---------------------------------------------------------------------
// Lists
// ---------------------------------------------------------------------

/// A `List` the evaluator handed us: the argument's handle, or `None` for
/// `v:_null_list`.
///
/// A **borrow** of the handle -- the argument's own reference is what keeps
/// the list alive for the call, and this takes none of its own. The list is
/// reached through it one statement at a time, because a callback between
/// two of them can reach it too.
///
/// NULL is not an error state: `v:_null_list` reaches every builtin here, and
/// every helper reads it as an empty, `VarLock::Fixed` list.
#[derive(Clone, Copy)]
pub(crate) struct ListArg<'a>(Option<&'a ListRef>);

impl<'a> ListArg<'a> {
    /// A borrow of a list handle; `None` for `v:_null_list`.
    #[inline(always)]
    pub(crate) fn of(list: Option<&'a ListRef>) -> ListArg<'a> {
        ListArg(list)
    }

    /// The list, writable, for the one statement that asked.
    #[inline(always)]
    fn get(self) -> Option<&'a mut List> {
        self.0.map(ListRef::edit)
    }

    #[inline(always)]
    pub(crate) fn is_null(self) -> bool {
        self.0.is_none()
    }

    /// The lock status; a NULL list reads as `VarLock::Fixed`.
    #[inline(always)]
    pub(crate) fn locked(self) -> VarLock {
        self.0.map_or(VarLock::Fixed, |l| l.lv_lock)
    }

    /// Set the lock status.  A NULL list already reads as `VarLock::Fixed`.
    #[inline(always)]
    pub(crate) fn set_lock(self, lock: VarLock) {
        if let Some(list) = self.get() {
            list.lv_lock = lock;
        }
    }

    /// Number of items; a NULL list is empty.
    #[inline(always)]
    pub(crate) fn len(self) -> c_int {
        index_of(self.count())
    }

    /// How many items, as the array's own count.
    #[inline(always)]
    fn count(self) -> usize {
        self.0.map_or(0, |l| l.lv_items.len())
    }

    #[inline(always)]
    pub(crate) fn first(self) -> Option<Item<'a>> {
        self.at(0)
    }

    /// The item at `at`, or None when the list is shorter than that.
    #[inline(always)]
    fn at(self, at: usize) -> Option<Item<'a>> {
        (at < self.count()).then_some(Item { list: self, at })
    }

    /// The item at `n`, which may count back from the end.
    #[inline(always)]
    pub(crate) fn find(self, n: c_int) -> Option<Item<'a>> {
        let at = list_index(self.0.map(|l| &**l), n)?;
        Some(Item { list: self, at })
    }

    /// Reverse the items. A NULL list is empty, so there is nothing to do.
    #[inline(always)]
    pub(crate) fn reverse(self) {
        if let Some(list) = self.get() {
            list.reverse();
        }
    }

    /// Store the list in `result`, which takes a reference of its own.
    #[inline(always)]
    pub(crate) fn set_ret(self, result: &mut TypVal) {
        result.write_list(self.0.cloned());
    }

    /// Append a copy of `tv`.
    ///
    /// The copy is made before the list is borrowed: `add(l, l)` hands the
    /// list itself as `tv`, and copying it writes the list's count.
    ///
    /// A NULL list reads as `VarLock::Fixed`, so every caller has already
    /// been refused by [`check_lock`] before it gets here; the guard is what
    /// makes that a *fact* rather than a claim about callers this type
    /// cannot see.
    #[inline(always)]
    pub(crate) fn append_tv(self, tv: &TypVal) {
        if let Some(list) = self.0 {
            let copy = tv.clone();
            list.edit().push(copy);
        }
    }

    /// Append `tv`, taking ownership of it.
    #[inline(always)]
    pub(crate) fn append_owned(self, tv: TypVal) {
        self.get().expect("a live list").push(tv);
    }

    /// Insert a copy of `tv` before `before`, or at the end when it is None.
    /// `before` is an item of this very list -- [`find`](Self::find) is the
    /// only thing that produces one -- and a NULL list has no items, so it
    /// is `None` and there is nothing to insert into. The copy comes first,
    /// as [`append_tv`](Self::append_tv)'s does.
    #[inline(always)]
    pub(crate) fn insert_tv(self, tv: &TypVal, before: Option<Item<'_>>) {
        if let Some(list) = self.0 {
            let copy = tv.clone();
            list.edit().insert(copy, before.map(|i| i.at));
        }
    }

    /// Splice copies of `other`'s items in before `before`.
    #[inline(always)]
    pub(crate) fn extend_with(self, other: ListArg<'_>, before: Option<Item<'_>>) {
        if let Some(dest) = self.0 {
            // `before` is an item of this list.
            list_extend(dest, other.0, before.map(|i| i.at));
        }
    }

    /// Remove `item` and answer the one that followed it.
    #[inline(always)]
    pub(crate) fn remove_item(self, item: Item<'a>) -> Option<Item<'a>> {
        // This is what shifts any `:for` cursor parked on it; the item is
        // released once the borrow has ended.
        let taken = self.get()?.take_range(item.at, item.at);
        drop(taken);
        self.at(item.at)
    }

    /// A shallow copy, for `extendnew()`.  `None` when the copy failed.
    #[inline(always)]
    pub(crate) fn copy(self) -> Option<ListRef> {
        // No conversion, and a fresh copyID.
        list_copy(None, self.0?, false, get_copy_id())
    }
}

/// Allocate a fresh list into `result`, for `mapnew()`.
#[inline(always)]
pub(crate) fn list_alloc_ret(result: &mut TypVal) -> ListArg<'_> {
    // `kListLenUnknown`: no idea how long.  Declared here rather than at
    // module level, where `ffigen` would emit it into the unit cdefs.
    const LEN_UNKNOWN: ptrdiff_t = -1;
    tv_list_alloc_ret(result, LEN_UNKNOWN);
    ListArg(result.list_shared())
}

/// One item of a list: the list, and *where in it*.
///
/// An index and not an address, because a callback between two steps of a
/// walk may insert or remove items and move every one of them.  An `Item`
/// that named a slot that has since gone panics rather than reading a stale
/// one -- the walks here re-derive it through [`ListArg::at`] each step,
/// which answers `None` instead.
#[derive(Clone, Copy)]
pub(crate) struct Item<'a> {
    list: ListArg<'a>,
    at: usize,
}

impl<'a> Item<'a> {
    /// The item itself, for the one statement that asked.
    #[inline(always)]
    fn get(self) -> &'a mut ListItem {
        let list = self.list.get().expect("an item of a live list");
        &mut list.items_mut()[self.at]
    }

    /// Copy the item's value into `v:val`, before a callback that may
    /// remove this very item runs.
    #[inline(always)]
    pub(crate) fn set_val(self) {
        set_vim_var_tv(Vv::Val, &mut self.get().li_tv);
    }

    #[inline(always)]
    pub(crate) fn lock(self) -> VarLock {
        self.get().li_lock
    }

    /// The item after this one, looked up *now*: a callback runs between two
    /// of these and may have edited the list, so a walk that remembered a
    /// slot from before it would read a stale one.
    #[inline(always)]
    pub(crate) fn next(self) -> Option<Self> {
        self.list.at(self.at + 1)
    }

    /// Replace the item's value with `newtv`, clearing what was there.
    #[inline(always)]
    pub(crate) fn set_tv(self, newtv: TypVal) {
        // Taken out first, and released once the borrow has ended: a value
        // can name the list it was in.
        let old = {
            let item = self.get();
            // Upstream unlocks the value it is about to store; with the
            // lock on the slot, what it unlocks is this item.
            item.li_lock = VarLock::Unlocked;
            ::core::mem::replace(&mut item.li_tv, newtv)
        };
        drop(old);
    }

    /// Whether the item's value equals `needle`, `ic` ignoring case.
    #[inline(always)]
    pub(crate) fn equals(self, needle: &TypVal, ic: bool) -> bool {
        let list = self.list.0.expect("an item of a live list");
        equal(&list.items()[self.at].li_tv, needle, ic)
    }
}

// ---------------------------------------------------------------------
// Dicts
// ---------------------------------------------------------------------

/// A `Dict` the evaluator handed us: the argument's handle, or `None` for
/// `v:_null_dict`.
///
/// `filter()`, `map()` and `foreach()` run a callback per item, and that
/// callback reaches this same dictionary through whatever named it -- so
/// nothing here holds a borrow of the dictionary across a step, and
/// [`DictArg::items`] is a *slot cursor* rather than an iterator that
/// borrows the table.
#[derive(Clone, Copy)]
pub(crate) struct DictArg<'a>(Option<&'a DictRef>);

impl<'a> DictArg<'a> {
    /// A borrow of a dictionary handle; `None` for `v:_null_dict`.
    #[inline(always)]
    pub(crate) fn of(dict: Option<&'a DictRef>) -> DictArg<'a> {
        DictArg(dict)
    }

    #[inline(always)]
    pub(crate) fn is_null(self) -> bool {
        self.0.is_none()
    }

    #[inline(always)]
    pub(crate) fn lock(self) -> VarLock {
        self.0.map_or(VarLock::Fixed, |d| d.dv_lock)
    }

    #[inline(always)]
    pub(crate) fn set_lock(self, lock: VarLock) {
        if let Some(dict) = self.0 {
            dict.edit().dv_lock = lock;
        }
    }

    /// Forbid the hashtab any rehashing for the duration of a walk, so the
    /// slots [`DictArg::items`] steps over cannot move under it.
    #[inline(always)]
    pub(crate) fn hash_lock(self) {
        self.0.expect("a live dict").edit().lock_table();
    }

    #[inline(always)]
    pub(crate) fn hash_unlock(self) {
        self.0.expect("a live dict").edit().unlock_table();
    }

    /// The dict's items, in hashtab order -- upstream's `TV_DICT_ITER`.
    ///
    /// It holds what the macro holds and no more: the slot cursor and the
    /// count of live items still to come.  That is what makes it safe to
    /// drive across a callback: the walk is under [`DictArg::hash_lock`], so
    /// a removal only leaves a tombstone in a slot already passed.
    #[inline(always)]
    pub(crate) fn items(self) -> impl Iterator<Item = DictItemRef<'a>> {
        let mut cursor = self.0.map(|dict| DictCursor::new(dict));
        core::iter::from_fn(move || {
            let dict = self.0?;
            let slot = cursor.as_mut()?.next(dict)?;
            Some(DictItemRef(dict, slot))
        })
    }

    /// Add a copy of `tv` under `key`; false when the key was already there.
    #[inline(always)]
    pub(crate) fn add_tv(self, key: &[u8], tv: &TypVal) -> bool {
        self.0.expect("a live dict").edit().add_tv(key, tv).is_ok()
    }

    #[inline(always)]
    pub(crate) fn remove_item(self, item: DictItemRef<'_>) {
        // The key is copied out of the item the removal frees.
        let key = crate::types::DictKey::new(item.key());
        tv_dict_item_remove(self.0.expect("a live dict"), key.bytes());
    }

    /// Merge `other`'s keys in under `action` (`"keep"`/`"force"`/`"error"`).
    #[inline(always)]
    pub(crate) fn extend_with(self, other: DictArg<'_>, action: &CStr) {
        // The mode is the action's first byte, which is how upstream tells
        // `"keep"` from `"force"` from `"error"`. The two may be the same
        // dictionary.
        let mode = action.to_bytes()[0];
        dict_extend(
            self.0.expect("a live dict"),
            other.0.expect("a live dict"),
            mode,
        );
    }

    /// A shallow copy, for `extendnew()`.  `None` when the copy failed.
    #[inline(always)]
    pub(crate) fn copy(self) -> Option<DictRef> {
        // No conversion, and a fresh copyID.
        dict_copy(None, self.0?, false, get_copy_id())
    }

    /// Allocate a fresh dict into `result`, for `mapnew()`.
    #[inline(always)]
    pub(crate) fn alloc_ret(result: &mut TypVal) -> DictArg<'_> {
        tv_dict_alloc_ret(result);
        DictArg(result.dict_shared())
    }
}

/// One entry of a dict: the dictionary, and the slot it is in.
///
/// A slot and not an address, for [`Item`]'s reason: it is re-read after
/// every callback. The walks run under the table's lock, so the slot does
/// not move.
#[derive(Clone, Copy)]
pub(crate) struct DictItemRef<'a>(&'a DictRef, usize);

impl<'a> DictItemRef<'a> {
    /// The entry itself, for the one statement that asked.
    #[inline(always)]
    fn get(self) -> &'a mut DictItem {
        self.0
            .edit()
            .item_at_mut(self.1)
            .expect("an entry of the walk's dict")
    }

    /// The entry, read-only.
    #[inline(always)]
    fn read(self) -> &'a DictItem {
        let dict: &'a Dict = self.0;
        dict.item_at(self.1).expect("an entry of the walk's dict")
    }

    /// The key: the item's own bytes, without the terminator.
    #[inline(always)]
    pub(crate) fn key(self) -> &'a [u8] {
        self.read().key()
    }

    /// Copy the value into `v:val`; see [`Item::set_val`].
    #[inline(always)]
    pub(crate) fn set_val(self) {
        set_vim_var_tv(Vv::Val, &mut self.get().di_tv);
    }

    #[inline(always)]
    pub(crate) fn lock(self) -> VarLock {
        self.read().di_lock
    }

    /// `DI_FLAGS_*`: read-only, fixed, allocated, ...
    #[inline(always)]
    pub(crate) fn flags(self) -> c_int {
        c_int::from(self.read().di_flags)
    }

    /// Replace the value with `newtv`, clearing what was there.
    #[inline(always)]
    pub(crate) fn set_tv(self, newtv: TypVal) {
        // As `Item::set_tv`: the old value goes once the borrow has ended,
        // and the lock unlocked here is the slot's.
        let old = {
            let item = self.get();
            item.di_lock = VarLock::Unlocked;
            ::core::mem::replace(&mut item.di_tv, newtv)
        };
        drop(old);
    }

    /// Whether the value equals `needle`, `ic` ignoring case.
    #[inline(always)]
    pub(crate) fn equals(self, needle: &TypVal, ic: bool) -> bool {
        equal(&self.read().di_tv, needle, ic)
    }
}

// ---------------------------------------------------------------------
// Blobs
// ---------------------------------------------------------------------

/// A `Blob` the evaluator handed us: the argument's handle, or `None` for
/// `v:_null_blob`.
///
/// A borrow of the handle, for [`DictArg`]'s reason: `filter()`/`map()`/
/// `foreach()` run a callback per byte and the callback reaches this same
/// blob, so every method here re-derives the borrow of the blob.
#[derive(Clone, Copy)]
pub(crate) struct BlobArg<'a>(Option<&'a BlobRef>);

impl<'a> BlobArg<'a> {
    /// The blob, writable, for the one statement that asked.
    #[inline(always)]
    fn get(self) -> Option<&'a mut Blob> {
        self.0.map(BlobRef::edit)
    }

    #[inline(always)]
    pub(crate) fn is_null(self) -> bool {
        self.0.is_none()
    }

    #[inline(always)]
    pub(crate) fn lock(self) -> VarLock {
        self.0.map_or(VarLock::Fixed, |b| b.bv_lock)
    }

    #[inline(always)]
    pub(crate) fn set_lock(self, lock: VarLock) {
        if let Some(b) = self.get() {
            b.bv_lock = lock;
        }
    }

    /// Length in bytes; a NULL blob is empty.
    #[inline(always)]
    pub(crate) fn len(self) -> c_int {
        self.0.map_or(0, |b| b.len_int())
    }

    #[inline(always)]
    pub(crate) fn byte(self, idx: c_int) -> uint8_t {
        self.0.expect("a live blob").byte(idx)
    }

    #[inline(always)]
    pub(crate) fn set_byte(self, idx: c_int, byte: uint8_t) {
        self.get().expect("a live blob").set_byte(idx, byte);
    }

    /// Drop the byte at `idx`, closing the gap -- `filter()`'s removal.
    #[inline(always)]
    pub(crate) fn remove_byte(self, idx: c_int) {
        let at = usize::try_from(idx).expect("a byte of the blob");
        self.get().expect("a live blob").drain(at, at);
    }

    /// Insert `byte` before `idx`, which may be the blob's length.
    #[inline(always)]
    pub(crate) fn insert_byte(self, idx: c_int, byte: uint8_t) {
        let blob = self.get().expect("a live blob");
        let idx = usize::try_from(idx).expect("a byte of the blob");
        let len = blob.len();
        // `claim` grows the array *and* declares the byte live, so the
        // shuffle below runs over the whole new length.
        blob.claim(1);
        let bytes = blob.bytes_mut();
        bytes.copy_within(idx..len, idx + 1);
        bytes[idx] = byte;
    }

    /// Append `byte`, growing the blob -- `add()`'s one-item form.
    #[inline(always)]
    pub(crate) fn push(self, byte: uint8_t) {
        self.get().expect("a live blob").push(byte);
    }

    /// Store the blob in `result`, taking a reference to it.
    #[inline(always)]
    pub(crate) fn set_ret(self, result: &mut TypVal) {
        result.write_blob(self.0.cloned());
    }

    /// Copy the blob into `result` and answer the copy, for `mapnew()`.
    #[inline(always)]
    pub(crate) fn copy_to(self, result: &mut TypVal) -> BlobArg<'_> {
        blob_copy(self.0.map(|b| &**b), result);
        BlobArg(result.blob_shared())
    }
}

// ---------------------------------------------------------------------
// Values, errors, and the evaluator
// ---------------------------------------------------------------------

/// Copy `from` into `to`, taking a reference to whatever it holds.
#[inline(always)]
pub(crate) fn copy_tv(from: &TypVal, to: &mut TypVal) {
    tv_copy(from, to);
}

/// Release whatever `tv` holds and leave it `VAR_UNKNOWN`.
#[inline(always)]
pub(crate) fn clear_tv(tv: &mut TypVal) {
    tv_clear(tv);
}

/// `tv` as a Number, setting `error` (and reporting one) if it is not.
#[inline(always)]
pub(crate) fn number_of(tv: &TypVal, error: &mut bool) -> VarNumber {
    tv_get_number_chk(tv).unwrap_or_else(|_| {
        *error = true;
        0
    })
}

/// `tv`'s Number, for the arms whose `v_type` has already been checked.
///
/// `VAR_BOOL` answers too: upstream reads `v_number` for both, the boolean
/// living in the same word, and the callers accept either tag.
#[inline(always)]
pub(crate) fn number_arm(tv: &TypVal) -> VarNumber {
    match (tv.as_number(), tv.as_bool()) {
        (Some(n), _) => n,
        (_, Some(b)) => VarNumber::from(b),
        _ => 0,
    }
}

/// Whether `a` and `b` are equal, `ic` ignoring case in strings.
#[inline(always)]
fn equal(a: &TypVal, b: &TypVal, ic: bool) -> bool {
    tv_equal(a, b, ic)
}

/// The bytes of a String `tv`; empty for `v:_null_string`, and for anything
/// that is not a String at all.
#[inline(always)]
pub(crate) fn string_bytes(tv: &TypVal) -> &[u8] {
    tv.string_bytes()
}

/// `tv` as a NUL-terminated string, coercing what can be coerced.
///
/// A Number has no string of its own, so the caller lends `buf` for it to be
/// spelled into; the answer borrows `buf` or the value, whichever it came
/// from, and lives no longer than either.
#[inline(always)]
pub(crate) fn cstr_of<'a>(tv: &'a TypVal, buf: &'a mut NumBuf) -> &'a CStr {
    buf.string(tv)
}

/// `tv` as a NUL-terminated string, or None -- having reported the error --
/// for a type that has no string form. As [`cstr_of`], the caller lends the
/// scratch a Number is spelled into.
#[inline(always)]
pub(crate) fn cstr_of_chk<'a>(tv: &'a TypVal, buf: &'a mut NumBuf) -> Option<&'a CStr> {
    buf.string_chk(tv)
}

/// A `VAR_STRING` owning a fresh copy of `bytes`, NUL-terminated.
#[inline(always)]
pub(crate) fn string_tv(bytes: &[u8]) -> TypVal {
    TypVal::string_from(bytes)
}

/// Whether `lock` forbids a change, reporting `E741`/`E742` naming `what`,
/// translated.
#[inline(always)]
pub(crate) fn check_lock(lock: VarLock, what: &'static CStr) -> bool {
    value_check_lock(lock, LockName::Translate(what))
}

/// Whether `flags` says the variable is read-only, reporting `E46` if so.
#[inline(always)]
pub(crate) fn check_ro(flags: c_int, what: &'static CStr) -> bool {
    // Translated only when there is something to report: the lookup is not
    // free, and this runs on every write.
    flags & (crate::eval::typval::DI_FLAGS_RO | crate::eval::typval::DI_FLAGS_RO_SBX) as c_int != 0
        && var_check_ro_named(flags, gettext(what).to_bytes())
}

/// Whether `flags` says the variable is fixed, reporting `E795` if so.
#[inline(always)]
pub(crate) fn check_fixed(flags: c_int, what: &'static CStr) -> bool {
    flags & crate::eval::typval::DI_FLAGS_FIX as c_int != 0
        && var_check_fixed_named(flags, gettext(what).to_bytes())
}

/// Report `msg`, one of `main.rs`'s shared error texts, translated.
#[inline(always)]
pub(crate) fn err(msg: &'static CStr) {
    emsg(gettext(msg));
}

/// Report `msg` -- a shared error text with one `%s` -- naming `what`.
#[inline(always)]
pub(crate) fn err_str(msg: &'static CStr, what: &CStr) {
    // The message is translated, the name is not -- upstream's
    // `semsg(_(msg), what)`.
    emsg_text(tr_c!(msg, msg_cstr(what)));
}

/// Report `msg` -- a shared error text with one `%ld` -- naming `n`.
#[inline(always)]
pub(crate) fn err_nr(msg: &'static CStr, n: int64_t) {
    emsg_text(tr_c!(msg, n));
}

/// `E1250`: what `filter()`/`map()`/`mapnew()`/`foreach()` say about an
/// argument that is none of the four containers.
#[inline(always)]
pub(crate) fn err_not_container(func_name: &CStr) {
    err_str(
        e_argument_of_str_must_be_list_string_dictionary_or_blob,
        func_name,
    );
}

/// `E706`: what `count()` says about the same.
#[inline(always)]
pub(crate) fn err_not_countable(func_name: &CStr) {
    err_str(
        e_argument_of_str_must_be_list_string_or_dictionary,
        func_name,
    );
}

/// Name the `v:` variable `idx` in the argument frame `filter_map_one`
/// calls with.
///
/// A *borrowed* slot: the value is the `v:` variable's, which keeps it, so
/// the frame releases nothing.
#[inline(always)]
pub(crate) fn push_vim_var(argv: &mut CallFrame<2>, idx: Vv) {
    with_vim_var(idx, |value| argv.push_borrowed(value));
}

/// Release whatever the `v:` variable `idx` holds.
#[inline(always)]
pub(crate) fn clear_vim_var(idx: Vv) {
    clear_vimvar(idx);
}

/// Copy `tv` into `v:val`, for the walks over values of their own making.
#[inline(always)]
pub(crate) fn set_val(tv: &mut TypVal) {
    set_vim_var_tv(Vv::Val, tv);
}

/// Set `v:key` to the Number `n`.  Its type is set separately, once per
/// walk, because `set_vim_var_nr` does not set one.
#[inline(always)]
pub(crate) fn set_key_nr(n: VarNumber) {
    set_vim_var_nr(Vv::Key, n);
}

/// Set `v:key` to the string `key`.
#[inline(always)]
pub(crate) fn set_key_string(key: &[u8]) {
    // A slice rather than a `strlen`: this runs once per item of every
    // `filter()` and `map()` over a dictionary, and a `DictKey` already
    // knows how long it is.
    set_vim_var_bytes(Vv::Key, key);
}

/// Declare `v:key`'s type for a walk that will set Numbers into it.
#[inline(always)]
pub(crate) fn set_key_type(v_type: VarType) {
    set_vim_var_type(Vv::Key, v_type);
}

/// Save the `v:` variable `idx` across a walk.
#[inline(always)]
pub(crate) fn save_vim_var(idx: Vv) -> TypVal {
    let mut save = UNKNOWN_TV;
    prepare_vimvar(idx, &mut save);
    save
}

/// Put back what [`save_vim_var`] took.
#[inline(always)]
pub(crate) fn restore_vim_var(idx: Vv, save: &mut TypVal) {
    restore_vimvar(idx, save);
}

/// Evaluate `expr` -- a Funcref, a partial or an expression string -- with
/// `v:key` and `v:val` as its two arguments, into `newtv`.  This is where the
/// family re-enters the evaluator, and so where anything may happen to the
/// container being walked.
#[inline(always)]
pub(crate) fn eval_expr(expr: &TypVal, argv: &CallFrame<2>, newtv: &mut TypVal) -> bool {
    eval_expr_typval(expr, false, argv.args(), newtv).is_ok()
}

/// Run `cmd` as an Ex command line -- `foreach()`'s String arm, which is not
/// limited to an expression. `v:_null_string` runs nothing.
#[inline(always)]
pub(crate) fn run_cmd(cmd: Option<&CStr>) {
    let _ = do_cmdline_cmd(cmd.unwrap_or(c""));
}

/// The length in bytes of the character `s` starts with, combining
/// characters included.
#[inline(always)]
pub(crate) fn char_len(s: &[u8]) -> usize {
    cluster_len(s)
}

/// Whether `hay` starts with `needle`, case-insensitively and multibyte
/// aware, comparing `needle.len()` bytes of each.
#[inline(always)]
pub(crate) fn starts_with_ic(hay: &[u8], needle: &[u8]) -> bool {
    // A `hay` shorter than `needle` ends where its string did.
    strnicmp_in(&hay[..hay.len().min(needle.len())], needle) == 0
}

// ---------------------------------------------------------------------
// The two builtins that stay here
// ---------------------------------------------------------------------

/// `remove(container, idx [, end])`: take items out and answer them.
///
/// Each container type has its own `tv_*_remove` in `typval.rs`, which is
/// where the index arithmetic and the `end` argument live.
pub fn f_remove(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let arg_errmsg = c"remove() argument";
    // Each takes the vector itself, and is reached only for the type it
    // handles; none runs user code, so the container is borrowed for the
    // call.
    match Container::of(&args[0]) {
        Container::Dict(_) => tv_dict_remove(args, result, arg_errmsg),
        Container::Blob(b) => blob_remove(b.get(), args, result, arg_errmsg),
        Container::List(l) => list_remove(l.get(), args, result, arg_errmsg),
        _ => err_str(e_listdictblobarg, c"remove()"),
    }
}

/// `reverse(container)`: turn a List, Blob or String around.
///
/// The List and the Blob are reversed in place; the String is rebuilt,
/// character by character, by `reverse_text`.
pub fn f_reverse(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // The check reports E1252 for a type that cannot be reversed.
    if tv_check_for_string_or_list_or_blob_arg(args, 0).is_err() {
        return;
    }
    match Container::of(&args[0]) {
        Container::Blob(b) => {
            let len = b.len();
            for i in 0..len / 2 {
                let tmp = b.byte(i);
                b.set_byte(i, b.byte(len - i - 1));
                b.set_byte(len - i - 1, tmp);
            }
            b.set_ret(result);
        }
        Container::Str(_) => {
            let reversed = args[0]
                .string_ref()
                .map(|text| ThinCString::from_vec(reversed_text(text.as_bytes())));
            result.write_string(reversed);
        }
        Container::List(l) => {
            if !check_lock(l.locked(), c"reverse() argument") {
                l.reverse();
                l.set_ret(result);
            }
        }
        Container::Dict(_) | Container::Other => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::typval::tv_list_alloc;
    use crate::global_cell::editor_state_lock;

    /// `add(l, l)` and `insert(l, l)`: the value stored is the list itself.
    /// The copy -- which writes the list's count -- is taken before the list
    /// is borrowed to store it.
    #[test]
    fn adding_a_list_to_itself_stores_a_reference_to_it() {
        let _serial = editor_state_lock();
        let list = tv_list_alloc(2);
        let tv = TypVal::list(Some(list.clone()));
        let arg = ListArg::of(tv.list_shared());
        arg.append_tv(&tv);
        arg.insert_tv(&tv, arg.first());
        assert_eq!(list.len(), 2);
        // The handle, the value, and the two items.
        assert_eq!(list.lv_refcount.get(), 4);
        assert!(
            list.items()
                .iter()
                .all(|item| { item.li_tv.list_shared().is_some_and(|l| l.ptr_eq(&list)) })
        );
        // Break the cycle so that the last handle frees it.
        list.remove_range(0, 1);
        assert_eq!(list.lv_refcount.get(), 2);
    }
}
