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
//! [`List`], [`Dict`] and [`Blob`] (and their item types) each wrap one raw
//! pointer and have one unsafe accessor -- `get` -- that turns it into a
//! borrow; everything else is a field read or a one-line forwarder, so the
//! builtins are ordinary safe Rust.  [`Container::of`] is the one place the
//! `typval_T` union is read, under the `v_type` that names the live arm.
//!
//! # Re-entrancy
//!
//! Nothing here caches anything across a call that can run Vimscript, and
//! the four walks run one per item.  [`Item::next`] re-reads `li_next`
//! *after* the callback has run, exactly where upstream reads it;
//! [`Dict::items`] holds only the two locals upstream's `TV_DICT_ITER`
//! holds; and an item's value crosses into the evaluator as a [`TvRef`]
//! rather than a borrow that would have to survive the call.
//!
//! Original: `src/nvim/eval/list.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};
use core::marker::PhantomData;
use core::mem::offset_of;
use core::slice;

use crate::cstr;
use crate::eval::typval::{
    NumBuf, tv_blob_copy, tv_blob_remove, tv_blob_set_ret, tv_check_for_string_or_list_or_blob_arg,
    tv_clear, tv_copy, tv_dict_add_tv, tv_dict_alloc_ret, tv_dict_copy, tv_dict_extend,
    tv_dict_item_remove, tv_dict_remove, tv_dict_unref, tv_equal, tv_get_number_chk,
    tv_get_string_buf, tv_get_string_buf_chk, tv_list_alloc_ret, tv_list_append_owned_tv,
    tv_list_append_tv, tv_list_copy, tv_list_extend, tv_list_find, tv_list_insert_tv,
    tv_list_item_remove, tv_list_remove, tv_list_reverse, tv_list_set_ret, tv_list_unref,
    value_check_lock,
};
use crate::eval::vars::{
    get_vim_var_tv, prepare_vimvar, restore_vimvar, set_vim_var_nr, set_vim_var_string,
    set_vim_var_type, var_check_fixed, var_check_ro,
};
use crate::eval::{eval_expr_typval, get_copy_id};
use crate::ex_docmd::do_cmdline_cmd;
use crate::garray::ga_grow;
use crate::hashtab::{hash_lock, hash_unlock};
use crate::main::e_listdictblobarg;
use crate::mbyte::{mb_strnicmp, utfc_ptr2len};
use crate::memory::xmemdupz;
use crate::message::emsg;
use crate::message_fmt::{emsg_text, msg_cstr};
use crate::os::cshim::gettext;
use crate::strings::reverse_text;
use crate::tr_c;
use crate::types::{
    EvalFuncData, VAR_BLOB, VAR_DICT, VAR_LIST, VAR_STRING, VAR_UNKNOWN, VarLock, VarType, Vv,
    blob_T, dict_T, dictitem_T, hashitem_T, int64_t, list_T, listitem_T, ptrdiff_t, size_t,
    typval_T, typval_vval_union, uint8_t, varnumber_T, vimconv_T,
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

/// A cleared `typval_T`, the `{ .v_type = VAR_UNKNOWN }` every walk starts
/// its per-item result from.
pub(crate) const UNKNOWN_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VarLock::Unlocked,
    vval: typval_vval_union { v_number: 0 },
};

// ---------------------------------------------------------------------
// The argument vector, and a value held as a pointer
// ---------------------------------------------------------------------

/// A builtin's argument vector is the tree's own [`Args`], shared with every
/// other `f_*` family, and [`frame`] is how a builtin opens onto it.
pub(crate) use crate::eval::funcs::args::{Args, frame};

/// A live `typval_T` held as a pointer rather than a borrow.
///
/// The walks hand a container's item straight to a callback that may remove
/// or free it, and a Rust reference would have to stay valid for the whole of
/// that call.  The lifetime says how long the value lives.
#[derive(Clone, Copy)]
pub(crate) struct TvRef<'a>(*mut typval_T, PhantomData<&'a mut typval_T>);

impl<'a> TvRef<'a> {
    /// A borrow the caller already holds, as a pointer.
    #[inline(always)]
    pub(crate) fn of(tv: &'a mut typval_T) -> Self {
        Self(&raw mut *tv, PhantomData)
    }
}

// ---------------------------------------------------------------------
// The container a typval holds
// ---------------------------------------------------------------------

/// Which container a `typval_T` holds, read from the arm its `v_type` says
/// is live.  Everything else -- Number, Float, Funcref, ... -- is
/// [`Container::Other`], which is what the family's type errors report.
#[derive(Clone, Copy)]
pub(crate) enum Container {
    List(List),
    Dict(Dict),
    Blob(Blob),
    /// A String's bytes, NUL-terminated, or NULL for `v:_null_string`.
    Str(*const c_char),
    Other,
}

impl Container {
    /// Read `tv`'s live union arm.
    #[inline(always)]
    pub(crate) fn of(tv: &typval_T) -> Self {
        match tv.v_type {
            // SAFETY: `v_type` is what says which arm of `vval` is live.
            VAR_LIST => Self::List(List(unsafe { tv.vval.v_list })),
            VAR_DICT => Self::Dict(Dict(unsafe { tv.vval.v_dict })),
            VAR_BLOB => Self::Blob(Blob(unsafe { tv.vval.v_blob })),
            VAR_STRING => Self::Str(unsafe { tv.vval.v_string }),
            _ => Self::Other,
        }
    }
}

// ---------------------------------------------------------------------
// Lists
// ---------------------------------------------------------------------

/// A `list_T` the evaluator handed us: live, or NULL.
///
/// NULL is not an error state: `v:_null_list` reaches every builtin here, and
/// every helper reads it as an empty, `VarLock::Fixed` list.
#[derive(Clone, Copy)]
pub(crate) struct List(*mut list_T);

impl List {
    /// The list itself, or None when it is NULL.  The one unsafe step.
    #[inline(always)]
    fn get<'a>(self) -> Option<&'a mut list_T> {
        // SAFETY: the evaluator handed us a live list, or NULL.
        unsafe { self.0.as_mut() }
    }

    #[inline(always)]
    pub(crate) fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// The list itself, for the `typval_T` `extendnew()` builds by hand.
    #[inline(always)]
    pub(crate) fn raw(self) -> *mut list_T {
        self.0
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
        self.get().map_or(0, |l| l.lv_len)
    }

    #[inline(always)]
    pub(crate) fn first(self) -> Option<Item> {
        Item::new(self.get().map_or(core::ptr::null_mut(), |l| l.lv_first))
    }

    /// The item at `n`, which may count back from the end.
    #[inline(always)]
    pub(crate) fn find(self, n: c_int) -> Option<Item> {
        // SAFETY: live or NULL, which is what `tv_list_find` takes.
        Item::new(unsafe { tv_list_find(self.0, n) })
    }

    #[inline(always)]
    pub(crate) fn reverse(self) {
        // SAFETY: live or NULL.
        unsafe { tv_list_reverse(self.0) };
    }

    /// Store the list in `rettv`, taking a reference to it.
    #[inline(always)]
    pub(crate) fn set_ret(self, rettv: &mut typval_T) {
        // SAFETY: live or NULL, and `rettv` is a cleared result slot.
        unsafe { tv_list_set_ret(rettv, self.0) };
    }

    /// Append a copy of `tv`.
    #[inline(always)]
    pub(crate) fn append_tv(self, tv: &mut typval_T) {
        // SAFETY: live or NULL, and `tv` is a live value.
        unsafe { tv_list_append_tv(self.0, tv) };
    }

    /// Append `tv`, taking ownership of it.
    #[inline(always)]
    pub(crate) fn append_owned(self, tv: typval_T) {
        // SAFETY: live, and the caller gives up `tv`.
        unsafe { tv_list_append_owned_tv(self.0, tv) };
    }

    /// Insert a copy of `tv` before `before`, or at the end when it is None.
    #[inline(always)]
    pub(crate) fn insert_tv(self, tv: &mut typval_T, before: Option<Item>) {
        // SAFETY: live, `tv` is a live value, and `before` is an item of this
        // very list -- `find` is the only thing that produces one.
        unsafe { tv_list_insert_tv(self.0, tv, Item::raw(before)) };
    }

    /// Splice copies of `other`'s items in before `before`.
    #[inline(always)]
    pub(crate) fn extend_with(self, other: List, before: Option<Item>) {
        // SAFETY: both live or NULL, and `before` is an item of this list.
        unsafe { tv_list_extend(self.0, other.0, Item::raw(before)) };
    }

    /// Remove `item` and answer the one that followed it.
    #[inline(always)]
    pub(crate) fn remove_item(self, item: Item) -> Option<Item> {
        // SAFETY: live, and `item` is an item of this list.  Upstream's
        // `tv_list_item_remove` is what fixes up any watcher parked on it.
        Item::new(unsafe { tv_list_item_remove(self.0, item.0) })
    }

    /// A shallow copy, for `extendnew()`.  NULL when the copy failed.
    #[inline(always)]
    pub(crate) fn copy(self) -> List {
        // SAFETY: live or NULL; no conversion, and a fresh copyID.
        Self(unsafe { tv_list_copy(core::ptr::null::<vimconv_T>(), self.0, false, get_copy_id()) })
    }

    #[inline(always)]
    pub(crate) fn unref(self) {
        // SAFETY: live or NULL, and the caller gives up its reference.
        unsafe { tv_list_unref(self.0) };
    }
}

/// Allocate a fresh list into `rettv`, for `mapnew()`.
#[inline(always)]
pub(crate) fn list_alloc_ret(rettv: &mut typval_T) -> List {
    // `kListLenUnknown`: no idea how long.  Declared here rather than at
    // module level, where `ffigen` would emit it into the unit cdefs.
    const LEN_UNKNOWN: ptrdiff_t = -1;
    // SAFETY: `rettv` is a cleared result slot.
    List(unsafe { tv_list_alloc_ret(rettv, LEN_UNKNOWN) })
}

/// One item of a list.  Never NULL -- absence is `Option<Item>`.
#[derive(Clone, Copy)]
pub(crate) struct Item(*mut listitem_T);

impl Item {
    /// The item itself.  The one unsafe step.
    #[inline(always)]
    fn get<'a>(self) -> &'a mut listitem_T {
        // SAFETY: an `Item` is only ever made from a live list's own item.
        unsafe { &mut *self.0 }
    }

    #[inline(always)]
    fn new(li: *mut listitem_T) -> Option<Self> {
        (!li.is_null()).then_some(Self(li))
    }

    #[inline(always)]
    fn raw(item: Option<Self>) -> *mut listitem_T {
        item.map_or(core::ptr::null_mut(), |i| i.0)
    }

    /// The item's value.  A [`TvRef`] and not a borrow: it is handed to a
    /// callback that may remove this very item.
    #[inline(always)]
    pub(crate) fn tv<'a>(self) -> TvRef<'a> {
        TvRef::of(&mut self.get().li_tv)
    }

    #[inline(always)]
    pub(crate) fn lock(self) -> VarLock {
        self.get().li_tv.v_lock
    }

    /// The item after this one, read *now*: a callback runs between two of
    /// these reads and may have removed items, so a walk that remembered the
    /// pointer from before it would follow a freed one.
    #[inline(always)]
    pub(crate) fn next(self) -> Option<Self> {
        Self::new(self.get().li_next)
    }

    /// Replace the item's value with `newtv`, clearing what was there.
    #[inline(always)]
    pub(crate) fn set_tv(self, mut newtv: typval_T) {
        newtv.v_lock = VarLock::Unlocked;
        let li = self.get();
        clear_tv(&mut li.li_tv);
        li.li_tv = newtv;
    }

    /// Whether the item's value equals `needle`, `ic` ignoring case.
    #[inline(always)]
    pub(crate) fn equals(self, needle: &mut typval_T, ic: bool) -> bool {
        equal(&mut self.get().li_tv, needle, ic)
    }
}

// ---------------------------------------------------------------------
// Dicts
// ---------------------------------------------------------------------

/// A `dict_T` the evaluator handed us: live, or NULL for `v:_null_dict`.
#[derive(Clone, Copy)]
pub(crate) struct Dict(*mut dict_T);

impl Dict {
    /// The dict itself, or None when it is NULL.  The one unsafe step.
    #[inline(always)]
    fn get<'a>(self) -> Option<&'a mut dict_T> {
        // SAFETY: the evaluator handed us a live dict, or NULL.
        unsafe { self.0.as_mut() }
    }

    #[inline(always)]
    pub(crate) fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// The dict itself, for the `typval_T` `extendnew()` builds by hand.
    #[inline(always)]
    pub(crate) fn raw(self) -> *mut dict_T {
        self.0
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
    /// It holds what the macro holds and no more: the slot cursor and the
    /// count of live items still to come.  That is what makes it safe to
    /// drive across a callback: the walk is under [`Dict::hash_lock`], so a
    /// removal only leaves a tombstone in a slot already passed.
    #[inline(always)]
    pub(crate) fn items(self) -> impl Iterator<Item = DictItem> {
        let (mut slot, mut todo) = self.get().map_or((core::ptr::null_mut(), 0), |d| {
            (d.dv_hashtab.slot_ptr(), d.dv_hashtab.ht_used)
        });
        core::iter::from_fn(move || {
            while todo != 0 {
                let hi: *mut hashitem_T = slot;
                // SAFETY: `todo` live items remain, so `hi` is in the array
                // and the slot after it is one to step to.
                let key = unsafe {
                    slot = slot.add(1);
                    (*hi).is_kept().then(|| (*hi).hi_key)
                };
                if let Some(key) = key {
                    todo -= 1;
                    // SAFETY-free: a live slot's key is a `dictitem_T`'s
                    // `di_key`, and stepping a pointer back is not a read.
                    return Some(DictItem(
                        key.wrapping_byte_sub(offset_of!(dictitem_T, di_key)).cast(),
                    ));
                }
            }
            None
        })
    }

    /// Add a copy of `tv` under `key`; false when the key was already there.
    #[inline(always)]
    pub(crate) fn add_tv(self, key: *mut c_char, tv: &mut typval_T) -> bool {
        // SAFETY: a live dict, `key` the NUL-terminated key of one of its own
        // items, and `tv` a live value.
        unsafe { tv_dict_add_tv(self.0, key, cstr::bytes_at(key).len(), tv) }.is_ok()
    }

    #[inline(always)]
    pub(crate) fn remove_item(self, item: DictItem) {
        // SAFETY: a live dict and one of its own items.
        unsafe { tv_dict_item_remove(self.0, item.0) };
    }

    /// Merge `other`'s keys in under `action` (`"keep"`/`"force"`/`"error"`).
    #[inline(always)]
    pub(crate) fn extend_with(self, other: Dict, action: &CStr) {
        // SAFETY: both live, and `action` is NUL-terminated.
        unsafe { tv_dict_extend(self.0, other.0, action.as_ptr()) };
    }

    /// A shallow copy, for `extendnew()`.  NULL when the copy failed.
    #[inline(always)]
    pub(crate) fn copy(self) -> Dict {
        // SAFETY: live; no conversion, and a fresh copyID.
        Self(unsafe { tv_dict_copy(core::ptr::null::<vimconv_T>(), self.0, false, get_copy_id()) })
    }

    #[inline(always)]
    pub(crate) fn unref(self) {
        // SAFETY: live or NULL, and the caller gives up its reference.
        unsafe { tv_dict_unref(self.0) };
    }

    /// Allocate a fresh dict into `rettv`, for `mapnew()`.
    #[inline(always)]
    pub(crate) fn alloc_ret(rettv: &mut typval_T) -> Dict {
        // SAFETY: `rettv` is a cleared result slot.
        unsafe { tv_dict_alloc_ret(rettv) };
        // SAFETY: which just made a dict the live arm.
        Self(unsafe { rettv.vval.v_dict })
    }
}

/// One entry of a dict.  Never NULL.
#[derive(Clone, Copy)]
pub(crate) struct DictItem(*mut dictitem_T);

impl DictItem {
    /// The entry itself.  The one unsafe step.
    #[inline(always)]
    fn get<'a>(self) -> &'a mut dictitem_T {
        // SAFETY: a `DictItem` is only ever made from a live dict's own slot.
        unsafe { &mut *self.0 }
    }

    /// The key, NUL-terminated.  `di_key` is a flexible array member, so its
    /// bytes live past the end of the struct.
    #[inline(always)]
    pub(crate) fn key(self) -> *mut c_char {
        (&raw mut self.get().di_key).cast()
    }

    /// The value; see [`Item::tv`] for why it is not a borrow.
    #[inline(always)]
    pub(crate) fn tv<'a>(self) -> TvRef<'a> {
        TvRef::of(&mut self.get().di_tv)
    }

    #[inline(always)]
    pub(crate) fn lock(self) -> VarLock {
        self.get().di_tv.v_lock
    }

    /// `DI_FLAGS_*`: read-only, fixed, allocated, ...
    #[inline(always)]
    pub(crate) fn flags(self) -> c_int {
        self.get().di_flags as c_int
    }

    /// Replace the value with `newtv`, clearing what was there.
    #[inline(always)]
    pub(crate) fn set_tv(self, mut newtv: typval_T) {
        newtv.v_lock = VarLock::Unlocked;
        let di = self.get();
        clear_tv(&mut di.di_tv);
        di.di_tv = newtv;
    }

    /// Whether the value equals `needle`, `ic` ignoring case.
    #[inline(always)]
    pub(crate) fn equals(self, needle: &mut typval_T, ic: bool) -> bool {
        equal(&mut self.get().di_tv, needle, ic)
    }
}

// ---------------------------------------------------------------------
// Blobs
// ---------------------------------------------------------------------

/// A `blob_T` the evaluator handed us: live, or NULL for `v:_null_blob`.
#[derive(Clone, Copy)]
pub(crate) struct Blob(*mut blob_T);

impl Blob {
    /// The blob itself, or None when it is NULL.  The one unsafe step.
    #[inline(always)]
    fn get<'a>(self) -> Option<&'a mut blob_T> {
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

    /// Store the blob in `rettv`, taking a reference to it.
    #[inline(always)]
    pub(crate) fn set_ret(self, rettv: &mut typval_T) {
        // SAFETY: live or NULL, and `rettv` is a cleared result slot.
        unsafe { tv_blob_set_ret(rettv, self.0) };
    }

    /// Copy the blob into `rettv` and answer the copy, for `mapnew()`.
    #[inline(always)]
    pub(crate) fn copy_to(self, rettv: &mut typval_T) -> Blob {
        // SAFETY: a live blob and a cleared result slot.
        unsafe { tv_blob_copy(self.0, rettv) };
        // SAFETY: which just made a blob the live arm.
        Self(unsafe { rettv.vval.v_blob })
    }
}

// ---------------------------------------------------------------------
// Values, errors, and the evaluator
// ---------------------------------------------------------------------

/// Copy `from` into `to`, taking a reference to whatever it holds.
#[inline(always)]
pub(crate) fn copy_tv(from: &typval_T, to: &mut typval_T) {
    // SAFETY: two live typvals.
    unsafe { tv_copy(from, to) };
}

/// Release whatever `tv` holds and leave it `VAR_UNKNOWN`.
#[inline(always)]
pub(crate) fn clear_tv(tv: &mut typval_T) {
    // SAFETY: a live typval.
    unsafe { tv_clear(tv) };
}

/// `tv` as a Number, setting `error` (and reporting one) if it is not.
#[inline(always)]
pub(crate) fn number_of(tv: &mut typval_T, error: &mut bool) -> varnumber_T {
    // SAFETY: a live typval.
    unsafe { tv_get_number_chk(tv, error) }
}

/// `tv`'s Number, for the arms whose `v_type` has already been checked.
#[inline(always)]
pub(crate) fn number_arm(tv: &typval_T) -> varnumber_T {
    // SAFETY: only reached where `v_type` is `VAR_NUMBER` or `VAR_BOOL`, both
    // of which make `v_number` the live arm.
    unsafe { tv.vval.v_number }
}

/// Whether `a` and `b` are equal, `ic` ignoring case in strings.
#[inline(always)]
fn equal(a: &mut typval_T, b: &mut typval_T, ic: bool) -> bool {
    // SAFETY: two live typvals; `tv_equal` only reads them.
    unsafe { tv_equal(a, b, ic) }
}

/// The bytes of a String `tv`; empty for `v:_null_string`, and for anything
/// that is not a String at all.
#[inline(always)]
pub(crate) fn string_bytes<'a>(tv: &typval_T) -> &'a [u8] {
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
pub(crate) fn cstr_of<'a>(tv: &mut typval_T, buf: &'a mut NumBuf) -> &'a CStr {
    // SAFETY: the scratch is the promised length and the answer is
    // NUL-terminated, never NULL.
    unsafe { CStr::from_ptr(tv_get_string_buf(tv, buf.as_mut_ptr())) }
}

/// `tv` as a NUL-terminated string, or None -- having reported the error --
/// for a type that has no string form. As [`cstr_of`], the caller lends the
/// scratch a Number is spelled into.
#[inline(always)]
pub(crate) fn cstr_of_chk<'a>(tv: &mut typval_T, buf: &'a mut NumBuf) -> Option<&'a CStr> {
    // SAFETY: as `cstr_of`; the answer may also be NULL.
    unsafe { cstr::at_opt(tv_get_string_buf_chk(tv, buf.as_mut_ptr())) }
}

/// A `VAR_STRING` owning a fresh copy of `bytes`, NUL-terminated.
#[inline(always)]
pub(crate) fn string_tv(bytes: &[u8]) -> typval_T {
    typval_T {
        v_type: VAR_STRING,
        v_lock: VarLock::Unlocked,
        vval: typval_vval_union {
            // SAFETY: `xmemdupz` reads `bytes` and appends the NUL itself.
            v_string: unsafe { xmemdupz(bytes.as_ptr().cast(), bytes.len()).cast() },
        },
    }
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

/// The value of the `v:` variable `idx`, copied out shallowly the way
/// `filter_map_one` builds its argument vector.
#[inline(always)]
pub(crate) fn vim_var_value(idx: Vv) -> typval_T {
    // SAFETY: `idx` names a `v:` variable, whose slot is always live.
    unsafe { *get_vim_var_tv(idx) }
}

/// Release whatever the `v:` variable `idx` holds.
#[inline(always)]
pub(crate) fn clear_vim_var(idx: Vv) {
    // SAFETY: as `vim_var_value`.
    unsafe { tv_clear(get_vim_var_tv(idx)) };
}

/// Copy `tv` into the `v:` variable `idx`.
#[inline(always)]
pub(crate) fn set_vim_var_tv(idx: Vv, tv: TvRef) {
    // SAFETY: as `vim_var_value`, and `tv` is a live value.
    unsafe { tv_copy(tv.0, get_vim_var_tv(idx)) };
}

/// Set `v:key` to the Number `n`.  Its type is set separately, once per
/// walk, because `set_vim_var_nr` does not set one.
#[inline(always)]
pub(crate) fn set_key_nr(n: varnumber_T) {
    // SAFETY: `Vv::Key` names a `v:` variable.
    unsafe { set_vim_var_nr(Vv::Key, n) };
}

/// Set `v:key` to the NUL-terminated string `s`.
#[inline(always)]
pub(crate) fn set_key_string(s: *mut c_char) {
    // SAFETY: `Vv::Key` names a `v:` variable and `s` is a dict key, which is
    // NUL-terminated -- what a length of -1 promises.
    unsafe { set_vim_var_string(Vv::Key, s, -1 as ptrdiff_t) };
}

/// Declare `v:key`'s type for a walk that will set Numbers into it.
#[inline(always)]
pub(crate) fn set_key_type(v_type: VarType) {
    // SAFETY: `Vv::Key` names a `v:` variable.
    unsafe { set_vim_var_type(Vv::Key, v_type) };
}

/// Save the `v:` variable `idx` across a walk.
#[inline(always)]
pub(crate) fn save_vim_var(idx: Vv) -> typval_T {
    let mut save = UNKNOWN_TV;
    // SAFETY: `idx` names a `v:` variable.
    unsafe { prepare_vimvar(idx, &raw mut save) };
    save
}

/// Put back what [`save_vim_var`] took.
#[inline(always)]
pub(crate) fn restore_vim_var(idx: Vv, save: &mut typval_T) {
    // SAFETY: `save` came from `save_vim_var` for that same variable.
    unsafe { restore_vimvar(idx, save) };
}

/// Evaluate `expr` -- a Funcref, a partial or an expression string -- with
/// `v:key` and `v:val` as its two arguments, into `newtv`.  This is where the
/// family re-enters the evaluator, and so where anything may happen to the
/// container being walked.
#[inline(always)]
pub(crate) fn eval_expr(
    expr: &mut typval_T,
    argv: &mut [typval_T; 3],
    newtv: &mut typval_T,
) -> bool {
    // SAFETY: three live typvals, and `argv` holds the two the count names.
    unsafe { eval_expr_typval(expr, false, argv.as_mut_ptr(), 2, newtv) }.is_ok()
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
///
/// # Safety
/// `argvars` is the evaluator's own argument vector, arity 2..3, and `rettv`
/// a cleared result.
pub unsafe fn f_remove(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let arg_errmsg = c"remove() argument".as_ptr();
    // SAFETY: the caller's contract.
    let mut args = unsafe { Args::new(argvars) };
    match Container::of(args.get_mut(0)) {
        // SAFETY: as above -- these three take the vector itself, and each is
        // reached only for the type it handles.
        Container::Dict(_) => unsafe { tv_dict_remove(argvars, rettv, arg_errmsg) },
        Container::Blob(_) => unsafe { tv_blob_remove(argvars, rettv, arg_errmsg) },
        Container::List(_) => unsafe { tv_list_remove(argvars, rettv, arg_errmsg) },
        _ => err_str(e_listdictblobarg, c"remove()"),
    }
}

/// `reverse(container)`: turn a List, Blob or String around.
///
/// The List and the Blob are reversed in place; the String is rebuilt,
/// character by character, by `reverse_text`.
///
/// # Safety
/// `argvars` is the evaluator's own argument vector, arity 1, and `rettv` a
/// cleared result.
pub unsafe fn f_reverse(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the caller's contract; the check reports E1252 for a type
    // that cannot be reversed.
    if unsafe { tv_check_for_string_or_list_or_blob_arg(argvars, 0) }.is_err() {
        return;
    }
    // SAFETY: the caller's contract.
    let (mut args, rettv) = frame!(argvars, rettv);
    match Container::of(args.get_mut(0)) {
        Container::Blob(b) => {
            let len = b.len();
            for i in 0..len / 2 {
                let tmp = b.byte(i);
                b.set_byte(i, b.byte(len - i - 1));
                b.set_byte(len - i - 1, tmp);
            }
            b.set_ret(rettv);
        }
        Container::Str(s) => {
            rettv.v_type = VAR_STRING;
            rettv.vval.v_string = if s.is_null() {
                core::ptr::null_mut()
            } else {
                // SAFETY: a live NUL-terminated string; `reverse_text`
                // allocates the answer.
                unsafe { reverse_text(s as *mut c_char) }
            };
        }
        Container::List(l) => {
            if !check_lock(l.locked(), c"reverse() argument") {
                l.reverse();
                l.set_ret(rettv);
            }
        }
        Container::Dict(_) | Container::Other => {}
    }
}
