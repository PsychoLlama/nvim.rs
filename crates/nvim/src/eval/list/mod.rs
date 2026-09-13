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
//! [`ListArg`], [`DictArg`] and [`BlobArg`] (and their item types) each wrap one raw
//! pointer and have one unsafe accessor -- `get` -- that turns it into a
//! borrow; everything else is a field read or a one-line forwarder, so the
//! builtins are ordinary safe Rust.  [`Container::of`] is the one place the
//! `TypVal` union is read, under the `v_type` that names the live arm.
//!
//! # Re-entrancy
//!
//! Nothing here caches anything across a call that can run Vimscript, and
//! the four walks run one per item.  An [`Item`] is a list and an *index*,
//! so it re-derives the slot after every callback rather than holding an
//! address the callback could have invalidated; [`DictArg::items`] holds
//! only the two locals upstream's `TV_DICT_ITER` holds; and an item's value
//! crosses into the evaluator as a [`TvRef`] rather than a borrow that would
//! have to survive the call.
//!
//! Original: `src/nvim/eval/list.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::eval::typval::CallFrame;
use crate::eval::typval::TV_INITIAL_VALUE;
use core::ffi::{CStr, c_char, c_int};
use core::marker::PhantomData;
use core::slice;

use crate::cstr;
use crate::eval::typval::{
    BlobRef, DictRef, ListRef, NumBuf, blob_copy, blob_remove, dict_copy, dict_extend, index_of,
    list_copy, list_extend, list_index, list_items_mut, list_remove, tv_blob_set_ret,
    tv_check_for_string_or_list_or_blob_arg, tv_clear, tv_copy, tv_dict_alloc_ret,
    tv_dict_item_remove, tv_dict_remove, tv_equal, tv_get_number_chk, tv_list_alloc_ret,
    value_check_lock,
};
use crate::eval::vars::{
    get_vim_var_tv, prepare_vimvar, restore_vimvar, set_vim_var_nr, set_vim_var_string,
    set_vim_var_type, var_check_fixed, var_check_ro,
};
use crate::eval::{eval_expr_typval, get_copy_id};
use crate::ex_docmd::do_cmdline_cmd;
use crate::garray::ga_grow;
use crate::global_cell::GlobalCell;
use crate::hashtab::{hash_lock, hash_unlock};
use crate::mbyte::{mb_strnicmp, utfc_ptr2len};
use crate::memory::xmemdupz;
use crate::message::e_listdictblobarg;
use crate::message::emsg;
use crate::message_fmt::{emsg_text, msg_cstr};
use crate::os::cshim::gettext;
use crate::strings::reverse_text;
use crate::tr_c;
use crate::types::{
    Blob, Dict, DictItem, EvalFuncData, List, ListItem, TypVal, VAR_BLOB, VAR_DICT, VAR_LIST,
    VAR_STRING, VarLock, VarNumber, VarType, VimConv, Vv, int64_t, ptrdiff_t, size_t, uint8_t,
};

// The carve of the transpiled module; see each child's docs.
mod count;
mod extend;
mod filtermap;

pub use self::count::{f_add, f_count};
pub use self::extend::{f_extend, f_extendnew, f_insert};
pub use self::filtermap::{f_filter, f_foreach, f_map, f_mapnew};

/// `TV_TRANSLATE`: the `name_len` that tells `value_check_lock` and friends
/// to run the name through `gettext` and measure it themselves.
const TV_TRANSLATE: size_t = size_t::MAX;

static e_argument_of_str_must_be_list_string_or_dictionary: &CStr =
    c"E706: Argument of %s must be a List, String or Dictionary";
static e_argument_of_str_must_be_list_string_dictionary_or_blob: &CStr =
    c"E1250: Argument of %s must be a List, String, Dictionary or Blob";

/// A cleared `TypVal`, the `{ .v_type = VAR_UNKNOWN }` every walk starts
/// its per-item result from.
pub(crate) const UNKNOWN_TV: TypVal = TV_INITIAL_VALUE;

// ---------------------------------------------------------------------
// A value held as a pointer
// ---------------------------------------------------------------------

// TV_CSTRING (SIZE_MAX - 1): c2rust dropped the initializer expression and
// left 0, which is a valid pointer-sentinel value and would corrupt any
// caller comparing against it (the unit tests do, via FFI).
pub static kTVCstring: GlobalCell<size_t> = GlobalCell::new(18446744073709551614);

/// A live `TypVal` held as a pointer rather than a borrow.
///
/// The walks hand a container's item straight to a callback that may remove
/// or free it, and a Rust reference would have to stay valid for the whole of
/// that call.  The lifetime says how long the value lives.
#[derive(Clone, Copy)]
pub(crate) struct TvRef<'a>(*mut TypVal, PhantomData<&'a mut TypVal>);

impl<'a> TvRef<'a> {
    /// A borrow the caller already holds, as a pointer.
    #[inline(always)]
    pub(crate) fn of(tv: &'a mut TypVal) -> Self {
        Self(&raw mut *tv, PhantomData)
    }
}

// ---------------------------------------------------------------------
// The container a typval holds
// ---------------------------------------------------------------------

/// Which container a `TypVal` holds, read from the arm its `v_type` says
/// is live.  Everything else -- Number, Float, Funcref, ... -- is
/// [`Container::Other`], which is what the family's type errors report.
#[derive(Clone, Copy)]
pub(crate) enum Container {
    List(ListArg),
    Dict(DictArg),
    Blob(BlobArg),
    /// A String's bytes, NUL-terminated, or NULL for `v:_null_string`.
    Str(*const c_char),
    Other,
}

impl Container {
    /// Read `tv`'s live union arm.
    #[inline(always)]
    pub(crate) fn of(tv: &TypVal) -> Self {
        match tv.v_type() {
            // SAFETY: `v_type` is what says which arm of `vval` is live.
            VAR_LIST => Self::List(ListArg(tv.list_or_null())),
            VAR_DICT => Self::Dict(DictArg(tv.dict_or_null())),
            VAR_BLOB => Self::Blob(BlobArg(tv.blob_or_null())),
            VAR_STRING => Self::Str(tv.string_or_null()),
            _ => Self::Other,
        }
    }
}

// ---------------------------------------------------------------------
// Lists
// ---------------------------------------------------------------------

/// A `List` the evaluator handed us: live, or NULL.
///
/// A **borrow** -- the argument's own reference is what keeps it alive for
/// the call, and this holds none of its own; the owning handle is
/// [`ListRef`](crate::types::ListRef).
///
/// NULL is not an error state: `v:_null_list` reaches every builtin here, and
/// every helper reads it as an empty, `VarLock::Fixed` list.
#[derive(Clone, Copy)]
pub(crate) struct ListArg(*mut List);

impl ListArg {
    /// A borrow of a list something else holds a reference to; null for
    /// `v:_null_list`.
    #[inline(always)]
    pub(crate) fn of(l: *mut List) -> ListArg {
        ListArg(l)
    }

    /// The list itself, or None when it is NULL.  The one unsafe step.
    #[inline(always)]
    fn get<'a>(self) -> Option<&'a mut List> {
        // SAFETY: the evaluator handed us a live list, or NULL.
        unsafe { self.0.as_mut() }
    }

    #[inline(always)]
    pub(crate) fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// The lock status; a NULL list reads as `VarLock::Fixed`.
    #[inline(always)]
    pub(crate) fn locked(self) -> VarLock {
        self.get().map_or(VarLock::Fixed, |l| l.lv_lock)
    }

    /// Set the lock status.  A NULL list already reads as `VarLock::Fixed`.
    #[inline(always)]
    pub(crate) fn set_lock(self, lock: VarLock) {
        if let Some(l) = self.get() {
            l.lv_lock = lock;
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
        self.get().map_or(0, |l| l.lv_items.len())
    }

    #[inline(always)]
    pub(crate) fn first(self) -> Option<Item> {
        self.at(0)
    }

    /// The item at `at`, or None when the list is shorter than that.
    #[inline(always)]
    fn at(self, at: usize) -> Option<Item> {
        (at < self.count()).then_some(Item { list: self, at })
    }

    /// The item at `n`, which may count back from the end.
    #[inline(always)]
    pub(crate) fn find(self, n: c_int) -> Option<Item> {
        // SAFETY: live or NULL, which is what `list_index` takes.
        let at = list_index(unsafe { self.0.as_ref() }, n)?;
        Some(Item { list: self, at })
    }

    #[inline(always)]
    pub(crate) fn reverse(self) {
        // SAFETY: live or NULL.
        unsafe { (*self.0).reverse() };
    }

    /// Store the list in `result`, which takes a reference of its own.
    #[inline(always)]
    pub(crate) fn set_ret(self, result: &mut TypVal) {
        // SAFETY: live or NULL; the answer takes a reference of its own.
        result.write_list(unsafe { ListRef::retained(self.0) });
    }

    /// Append a copy of `tv`.
    #[inline(always)]
    pub(crate) fn append_tv(self, tv: &TypVal) {
        // SAFETY: live or NULL, and `tv` is a live value.
        unsafe { (*self.0).push_copy(tv) };
    }

    /// Append `tv`, taking ownership of it.
    #[inline(always)]
    pub(crate) fn append_owned(self, tv: TypVal) {
        // SAFETY: live, and the caller gives up `tv`.
        unsafe { (*self.0).push(tv) };
    }

    /// Insert a copy of `tv` before `before`, or at the end when it is None.
    #[inline(always)]
    pub(crate) fn insert_tv(self, tv: &TypVal, before: Option<Item>) {
        // SAFETY: live, `tv` is a live value, and `before` is an item of this
        // very list -- `find` is the only thing that produces one.
        unsafe { (*self.0).insert_copy(tv, before.map(|i| i.at)) };
    }

    /// Splice copies of `other`'s items in before `before`.
    #[inline(always)]
    pub(crate) fn extend_with(self, other: ListArg, before: Option<Item>) {
        // SAFETY: both live or NULL, and `before` is an item of this list.
        unsafe { list_extend(self.0, other.0, before.map(|i| i.at)) };
    }

    /// Remove `item` and answer the one that followed it.
    #[inline(always)]
    pub(crate) fn remove_item(self, item: Item) -> Option<Item> {
        // SAFETY: live, and `item` is an item of this list.  This is what
        // shifts any `:for` cursor parked on it.
        unsafe { (*self.0).remove_at(item.at) };
        self.at(item.at)
    }

    /// A shallow copy, for `extendnew()`.  `None` when the copy failed.
    #[inline(always)]
    pub(crate) fn copy(self) -> Option<ListRef> {
        // SAFETY: live or NULL; the copy takes a reference of its own for
        // the walk, no conversion, and a fresh copyID.
        unsafe {
            list_copy(
                core::ptr::null::<VimConv>(),
                ListRef::retained(self.0),
                false,
                get_copy_id(),
            )
        }
    }
}

/// Allocate a fresh list into `result`, for `mapnew()`.
#[inline(always)]
pub(crate) fn list_alloc_ret(result: &mut TypVal) -> ListArg {
    // `kListLenUnknown`: no idea how long.  Declared here rather than at
    // module level, where `ffigen` would emit it into the unit cdefs.
    const LEN_UNKNOWN: ptrdiff_t = -1;
    // SAFETY: `result` is a cleared result slot.
    ListArg(&raw mut *tv_list_alloc_ret(result, LEN_UNKNOWN))
}

/// One item of a list: the list, and *where in it*.
///
/// An index and not an address, because a callback between two steps of a
/// walk may insert or remove items and move every one of them.  An `Item`
/// that named a slot that has since gone panics rather than reading a stale
/// one -- the walks here re-derive it through [`ListArg::at`] each step,
/// which answers `None` instead.
#[derive(Clone, Copy)]
pub(crate) struct Item {
    list: ListArg,
    at: usize,
}

impl Item {
    /// The item itself.  The one unsafe step.
    #[inline(always)]
    fn get<'a>(self) -> &'a mut ListItem {
        // SAFETY: an `Item` is only ever made from a live list.
        let items = list_items_mut(unsafe { self.list.0.as_mut() });
        &mut items[self.at]
    }

    /// The item's value.  A [`TvRef`] and not a borrow: it is handed to a
    /// callback that may remove this very item.
    #[inline(always)]
    pub(crate) fn tv<'a>(self) -> TvRef<'a> {
        TvRef::of(&mut self.get().li_tv)
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
        let li = self.get();
        clear_tv(&mut li.li_tv);
        li.li_tv = newtv;
        // Upstream unlocks the value it is about to store; with the lock on
        // the slot, what it unlocks is this item.
        li.li_lock = VarLock::Unlocked;
    }

    /// Whether the item's value equals `needle`, `ic` ignoring case.
    #[inline(always)]
    pub(crate) fn equals(self, needle: &TypVal, ic: bool) -> bool {
        equal(&self.get().li_tv, needle, ic)
    }
}

// ---------------------------------------------------------------------
// Dicts
// ---------------------------------------------------------------------

/// A `Dict` the evaluator handed us: live, or NULL for `v:_null_dict`.
///
/// **Not a borrow wearing a handle's name, unlike its list and blob
/// siblings.** `filter()`, `map()` and `foreach()` run a callback per item,
/// and that callback reaches this same dictionary through whatever named it
/// — so nothing here may hold a `&mut Dict` across a step, and
/// [`DictArg::items`] is a *slot index* rather than an iterator that borrows
/// the table.
#[derive(Clone, Copy)]
pub(crate) struct DictArg(*mut Dict);

impl DictArg {
    /// A borrow of a dictionary something else holds a reference to; null
    /// for `v:_null_dict`.
    #[inline(always)]
    pub(crate) fn of(d: *mut Dict) -> DictArg {
        DictArg(d)
    }

    /// The dict itself, or None when it is NULL.  The one unsafe step.
    #[inline(always)]
    fn get<'a>(self) -> Option<&'a mut Dict> {
        // SAFETY: the evaluator handed us a live dict, or NULL.
        unsafe { self.0.as_mut() }
    }

    #[inline(always)]
    pub(crate) fn is_null(self) -> bool {
        self.0.is_null()
    }

    #[inline(always)]
    pub(crate) fn lock(self) -> VarLock {
        self.get().map_or(VarLock::Fixed, |d| d.dv_lock)
    }

    #[inline(always)]
    pub(crate) fn set_lock(self, lock: VarLock) {
        if let Some(d) = self.get() {
            d.dv_lock = lock;
        }
    }

    /// Forbid the hashtab any rehashing for the duration of a walk, so the
    /// array [`Dict::items`] steps over cannot move under it.
    #[inline(always)]
    pub(crate) fn hash_lock(self) {
        // SAFETY: a live dict.
        unsafe { hash_lock(&raw mut (*self.0).dv_hashtab) };
    }

    #[inline(always)]
    pub(crate) fn hash_unlock(self) {
        // SAFETY: a live dict, locked by `hash_lock`.
        unsafe { hash_unlock(&raw mut (*self.0).dv_hashtab) };
    }

    /// The dict's items, in hashtab order -- upstream's `TV_DICT_ITER`.
    ///
    /// It holds what the macro holds and no more: the slot cursor -- an
    /// *index*, since the small run lives inside the table -- and the count
    /// of live items still to come.  That is what makes it safe to drive
    /// across a callback: the walk is under [`Dict::hash_lock`], so a
    /// removal only leaves a tombstone in a slot already passed.
    #[inline(always)]
    pub(crate) fn items(self) -> impl Iterator<Item = DictItemRef> {
        // The cursor is derived from the raw dict pointer, not from a
        // borrow of it: a body mutates the table through that same pointer.
        let ht = (!self.is_null()).then(|| unsafe { &raw const (*self.0).dv_hashtab });
        // SAFETY: the dict is live for the walk, or NULL and never read.
        let mut todo = ht.map_or(0, |ht| unsafe { (*ht).ht_used });
        let mut idx = 0usize;
        core::iter::from_fn(move || {
            let ht = ht?;
            while todo != 0 {
                // SAFETY: the dict is live for the walk and `todo` live
                // items remain, so `idx` is one of its slots.
                let hi = unsafe { (*ht).slot(idx) };
                idx += 1;
                if hi.is_kept() {
                    todo -= 1;
                    return Some(DictItemRef(hi.hi_key.item()));
                }
            }
            None
        })
    }

    /// Add a copy of `tv` under `key`; false when the key was already there.
    #[inline(always)]
    pub(crate) fn add_tv(self, key: &[u8], tv: &TypVal) -> bool {
        // SAFETY: a live dict, and `tv` a live value of another one.
        unsafe { (*self.0).add_tv(key, tv) }.is_ok()
    }

    #[inline(always)]
    pub(crate) fn remove_item(self, item: DictItemRef) {
        // SAFETY: a live dict and one of its own items.
        unsafe { tv_dict_item_remove(self.0, item.0) };
    }

    /// Merge `other`'s keys in under `action` (`"keep"`/`"force"`/`"error"`).
    #[inline(always)]
    pub(crate) fn extend_with(self, other: DictArg, action: &CStr) {
        // The mode is the action's first byte, which is how upstream tells
        // `"keep"` from `"force"` from `"error"`.
        let mode = action.to_bytes()[0];
        // SAFETY: both live, and the two may be the same dictionary --
        // which is the whole reason `dict_extend` takes pointers.
        unsafe { dict_extend(self.0, other.0, mode) };
    }

    /// A shallow copy, for `extendnew()`.  `None` when the copy failed.
    #[inline(always)]
    pub(crate) fn copy(self) -> Option<DictRef> {
        // SAFETY: live; no conversion, and a fresh copyID.
        unsafe { dict_copy(core::ptr::null::<VimConv>(), self.0, false, get_copy_id()) }
    }

    /// Allocate a fresh dict into `result`, for `mapnew()`.
    #[inline(always)]
    pub(crate) fn alloc_ret(result: &mut TypVal) -> DictArg {
        tv_dict_alloc_ret(result);
        Self(result.dict_or_null())
    }
}

/// One entry of a dict.  Never NULL.
#[derive(Clone, Copy)]
pub(crate) struct DictItemRef(*mut DictItem);

impl DictItemRef {
    /// The entry itself.  The one unsafe step.
    #[inline(always)]
    fn get<'a>(self) -> &'a mut DictItem {
        // SAFETY: a `DictItemRef` is only ever made from a live dict's own slot.
        unsafe { &mut *self.0 }
    }

    /// The key: the item's own bytes, without the terminator.
    #[inline(always)]
    pub(crate) fn key(self) -> &'static [u8] {
        self.get().key_bytes()
    }

    /// The value; see [`Item::tv`] for why it is not a borrow.
    #[inline(always)]
    pub(crate) fn tv<'a>(self) -> TvRef<'a> {
        TvRef::of(&mut self.get().di_tv)
    }

    #[inline(always)]
    pub(crate) fn lock(self) -> VarLock {
        self.get().di_lock
    }

    /// `DI_FLAGS_*`: read-only, fixed, allocated, ...
    #[inline(always)]
    pub(crate) fn flags(self) -> c_int {
        self.get().di_flags as c_int
    }

    /// Replace the value with `newtv`, clearing what was there.
    #[inline(always)]
    pub(crate) fn set_tv(self, newtv: TypVal) {
        let di = self.get();
        clear_tv(&mut di.di_tv);
        di.di_tv = newtv;
        // As `Li::set_tv`: the lock unlocked here is the slot's.
        di.di_lock = VarLock::Unlocked;
    }

    /// Whether the value equals `needle`, `ic` ignoring case.
    #[inline(always)]
    pub(crate) fn equals(self, needle: &TypVal, ic: bool) -> bool {
        equal(&self.get().di_tv, needle, ic)
    }
}

// ---------------------------------------------------------------------
// Blobs
// ---------------------------------------------------------------------

/// A `Blob` the evaluator handed us: live, or NULL for `v:_null_blob`.
#[derive(Clone, Copy)]
pub(crate) struct BlobArg(*mut Blob);

impl BlobArg {
    /// The blob itself, or None when it is NULL.  The one unsafe step.
    #[inline(always)]
    fn get<'a>(self) -> Option<&'a mut Blob> {
        // SAFETY: the evaluator handed us a live blob, or NULL.
        unsafe { self.0.as_mut() }
    }

    #[inline(always)]
    pub(crate) fn is_null(self) -> bool {
        self.0.is_null()
    }

    #[inline(always)]
    pub(crate) fn lock(self) -> VarLock {
        self.get().map_or(VarLock::Fixed, |b| b.bv_lock)
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
        self.get().map_or(0, |b| b.bv_ga.ga_len)
    }

    /// The first `len` bytes of storage: no more than the length plus what
    /// [`Blob::grow`] has just reserved.
    #[inline(always)]
    fn bytes<'a>(self, len: usize) -> &'a mut [uint8_t] {
        let data = self
            .get()
            .map_or(core::ptr::null_mut(), |b| b.bv_ga.ga_data);
        if len == 0 {
            return &mut [];
        }
        // SAFETY: a non-empty blob's `ga_data` holds `ga_maxlen` writable
        // bytes, and the caller stays inside them.
        unsafe { slice::from_raw_parts_mut(data.cast(), len) }
    }

    /// Make room for `n` more bytes without changing the length.
    #[inline(always)]
    fn grow(self, n: c_int) {
        // SAFETY: a live blob.
        unsafe { ga_grow(&raw mut (*self.0).bv_ga, n) };
    }

    #[inline(always)]
    fn set_len(self, len: c_int) {
        if let Some(b) = self.get() {
            b.bv_ga.ga_len = len;
        }
    }

    #[inline(always)]
    pub(crate) fn byte(self, idx: c_int) -> uint8_t {
        self.bytes(self.len() as usize)[idx as usize]
    }

    #[inline(always)]
    pub(crate) fn set_byte(self, idx: c_int, byte: uint8_t) {
        self.bytes(self.len() as usize)[idx as usize] = byte;
    }

    /// Drop the byte at `idx`, closing the gap -- `filter()`'s removal.
    #[inline(always)]
    pub(crate) fn remove_byte(self, idx: c_int) {
        let len = self.len() as usize;
        self.bytes(len)
            .copy_within(idx as usize + 1.., idx as usize);
        self.set_len(len as c_int - 1);
    }

    /// Insert `byte` before `idx`, which may be the blob's length.
    #[inline(always)]
    pub(crate) fn insert_byte(self, idx: c_int, byte: uint8_t) {
        let (len, idx) = (self.len() as usize, idx as usize);
        self.grow(1);
        let bytes = self.bytes(len + 1);
        bytes.copy_within(idx..len, idx + 1);
        bytes[idx] = byte;
        self.set_len(len as c_int + 1);
    }

    /// Append `byte`, growing the blob -- `add()`'s one-item form.
    #[inline(always)]
    pub(crate) fn push(self, byte: uint8_t) {
        self.insert_byte(self.len(), byte);
    }

    /// Store the blob in `result`, taking a reference to it.
    #[inline(always)]
    pub(crate) fn set_ret(self, result: &mut TypVal) {
        // SAFETY: live or NULL; the slot takes a reference of its own.
        tv_blob_set_ret(result, unsafe { BlobRef::retained(self.0) });
    }

    /// Copy the blob into `result` and answer the copy, for `mapnew()`.
    #[inline(always)]
    pub(crate) fn copy_to(self, result: &mut TypVal) -> BlobArg {
        // SAFETY: the argument's own blob, borrowed for the copy.
        blob_copy(unsafe { self.0.as_ref() }, result);
        Self(result.blob_or_null())
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
pub(crate) fn string_bytes<'a>(tv: &TypVal) -> &'a [u8] {
    match Container::of(tv) {
        // SAFETY: a `VAR_STRING`'s `v_string` is NUL-terminated.
        Container::Str(s) if !s.is_null() => unsafe { CStr::from_ptr(s) }.to_bytes(),
        _ => b"",
    }
}

/// `tv` as a NUL-terminated string, coercing what can be coerced.
///
/// A Number has no string of its own, so the caller lends `buf` for it to be
/// spelled into; the answer borrows `buf` or the value, whichever it came
/// from, and lives no longer than either.
#[inline(always)]
pub(crate) fn cstr_of<'a>(tv: &TypVal, buf: &'a mut NumBuf) -> &'a CStr {
    // SAFETY: the scratch is the promised length and the answer is
    // NUL-terminated, never NULL.
    unsafe { CStr::from_ptr(buf.string_ptr(tv)) }
}

/// `tv` as a NUL-terminated string, or None -- having reported the error --
/// for a type that has no string form. As [`cstr_of`], the caller lends the
/// scratch a Number is spelled into.
#[inline(always)]
pub(crate) fn cstr_of_chk<'a>(tv: &TypVal, buf: &'a mut NumBuf) -> Option<&'a CStr> {
    // SAFETY: as `cstr_of`; the answer may also be NULL.
    unsafe { cstr::at_opt(buf.string_ptr_chk(tv)) }
}

/// A `VAR_STRING` owning a fresh copy of `bytes`, NUL-terminated.
#[inline(always)]
pub(crate) fn string_tv(bytes: &[u8]) -> TypVal {
    TypVal::String(unsafe { xmemdupz(bytes.as_ptr().cast(), bytes.len()).cast() })
}

/// Whether `lock` forbids a change, reporting `E741`/`E742` naming `what`.
#[inline(always)]
pub(crate) fn check_lock(lock: VarLock, what: &CStr) -> bool {
    // SAFETY: `what` is NUL-terminated, and `TV_TRANSLATE` asks for it to be
    // translated and measured.
    unsafe { value_check_lock(lock, what.as_ptr(), TV_TRANSLATE) }
}

/// Whether `flags` says the variable is read-only, reporting `E46` if so.
#[inline(always)]
pub(crate) fn check_ro(flags: c_int, what: &CStr) -> bool {
    // SAFETY: as `check_lock`.
    unsafe { var_check_ro(flags, what.as_ptr(), TV_TRANSLATE) }
}

/// Whether `flags` says the variable is fixed, reporting `E795` if so.
#[inline(always)]
pub(crate) fn check_fixed(flags: c_int, what: &CStr) -> bool {
    // SAFETY: as `check_lock`.
    unsafe { var_check_fixed(flags, what.as_ptr(), TV_TRANSLATE) }
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

/// The `v:` variable `idx`, as the argument slot `filter_map_one` puts it
/// in.
///
/// A *borrowed* slot: the value is the `v:` variable's, which keeps it, so
/// the frame it goes into must release nothing.
#[inline(always)]
pub(crate) fn vim_var_value(idx: Vv) -> &'static TypVal {
    // SAFETY: `idx` names a `v:` variable, whose slot is always live, and
    // the answer names it rather than owning it.
    unsafe { &*get_vim_var_tv(idx) }
}

/// Release whatever the `v:` variable `idx` holds.
#[inline(always)]
pub(crate) fn clear_vim_var(idx: Vv) {
    // SAFETY: as `vim_var_value`.
    unsafe { tv_clear(&mut *get_vim_var_tv(idx)) };
}

/// Copy `tv` into the `v:` variable `idx`.
#[inline(always)]
pub(crate) fn set_vim_var_tv(idx: Vv, tv: TvRef) {
    // SAFETY: as `vim_var_value`, and `tv` is a live value.
    unsafe { tv_copy(&*tv.0, &mut *get_vim_var_tv(idx)) };
}

/// Set `v:key` to the Number `n`.  Its type is set separately, once per
/// walk, because `set_vim_var_nr` does not set one.
#[inline(always)]
pub(crate) fn set_key_nr(n: VarNumber) {
    set_vim_var_nr(Vv::Key, n);
}

/// Set `v:key` to the NUL-terminated string `s`.
#[inline(always)]
pub(crate) fn set_key_string(key: &[u8]) {
    // The length is spelled out rather than left to a `strlen`: this runs
    // once per item of every `filter()` and `map()` over a dictionary, and
    // a `DictKey` already knows how long it is.
    // SAFETY: `Vv::Key` names a `v:` variable, and `key` is a slice, so it
    // is readable for its own length.
    unsafe {
        set_vim_var_string(
            Vv::Key,
            key.as_ptr().cast::<c_char>(),
            index_of(key.len()) as ptrdiff_t,
        );
    };
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
/// limited to an expression.
#[inline(always)]
pub(crate) fn run_cmd(cmd: *const c_char) {
    // SAFETY: `cmd` is the NUL-terminated string of a `VAR_STRING` typval.
    let _ = unsafe { do_cmdline_cmd(cmd) };
}

/// The length in bytes of the character `s` starts with, combining
/// characters included.
#[inline(always)]
pub(crate) fn char_len(s: &[u8]) -> usize {
    // SAFETY: `s` is the tail of a NUL-terminated string, so the terminator
    // is right after it and `utfc_ptr2len` stops there.
    unsafe { utfc_ptr2len(s.as_ptr().cast()) as usize }
}

/// Whether `hay` starts with `needle`, case-insensitively and multibyte
/// aware, comparing `needle.len()` bytes of each.
#[inline(always)]
pub(crate) fn starts_with_ic(hay: &[u8], needle: &[u8]) -> bool {
    // SAFETY: both are tails of NUL-terminated strings, which is what stops
    // the comparison at the end of a `hay` shorter than `needle`.
    unsafe { mb_strnicmp(hay.as_ptr().cast(), needle.as_ptr().cast(), needle.len()) == 0 }
}

// ---------------------------------------------------------------------
// The two builtins that stay here
// ---------------------------------------------------------------------

/// `remove(container, idx [, end])`: take items out and answer them.
///
/// Each container type has its own `tv_*_remove` in `typval.rs`, which is
/// where the index arithmetic and the `end` argument live.
pub fn f_remove(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let arg_errmsg = c"remove() argument".as_ptr();
    match Container::of(&args[0]) {
        // SAFETY: as above -- these three take the vector itself, and each is
        // reached only for the type it handles.
        Container::Dict(_) => unsafe { tv_dict_remove(args, result, arg_errmsg) },
        // SAFETY: the blob the first argument holds, borrowed for the call.
        Container::Blob(b) => unsafe { blob_remove(b.0.as_mut(), args, result, arg_errmsg) },
        // SAFETY: the list the first argument holds, borrowed for the call.
        Container::List(l) => unsafe { list_remove(l.0.as_mut(), args, result, arg_errmsg) },
        _ => err_str(e_listdictblobarg, c"remove()"),
    }
}

/// `reverse(container)`: turn a List, Blob or String around.
///
/// The List and the Blob are reversed in place; the String is rebuilt,
/// character by character, by `reverse_text`.
pub fn f_reverse(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: the caller's contract; the check reports E1252 for a type
    // that cannot be reversed.
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
        Container::Str(s) => {
            result.write_string(if s.is_null() {
                core::ptr::null_mut()
            } else {
                // SAFETY: a live NUL-terminated string; `reverse_text`
                // allocates the answer.
                unsafe { reverse_text(s as *mut c_char) }
            });
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
