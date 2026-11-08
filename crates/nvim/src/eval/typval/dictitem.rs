//! A dictionary's items: the slots that name them, the pair that allocates
//! and frees one, and reading a value back out.
//!
//! The table's slots name the items ([`DictEntry`]), where upstream's named
//! the item's *key* -- an item was over-allocated so its key sat in a
//! flexible tail, and the item was the key pointer minus a constant. That is
//! why an item could not own its key. Reading the key out of the item costs
//! the probe nothing -- the probe compares the same bytes the slot used to
//! hold -- and so leaves the hash, the probe sequence, the resize thresholds
//! and the slot every key lands in exactly as they were. That matters: slot order is what `keys()`, `values()` and
//! `items()` show.
//!
//! The table does not free its items. Which of them it owns is
//! `DI_FLAGS_ALLOC` -- four kinds of item are embedded in something bigger
//! (a funccall's fixed variables, a scope's own entry, `b:changedtick`, a
//! `v:` row) and outlive every table they are in. An item off a table is a
//! `Box<DictItem>` ([`DictItem::boxed`]); [`Dict::add_item`] takes one over,
//! and a removal hands back what the table held
//! ([`RemovedItem`](super::RemovedItem)), to be dropped once the borrow of
//! the dictionary has ended.
//!
//! [`Dict::find`] is the hashtable lookup every getter goes through, and the
//! `dict_get_*` family coerces what it finds to one type, answering a
//! caller-supplied default when the key is absent or the wrong kind.  Each
//! has a free form over `Option<&Dict>`, because `v:_null_dict` is a real
//! value and not an error state.  [`dict_to_env`] builds the
//! `environ`-shaped array a job's environment is passed as.  The `*2items`
//! half and [`f_items`] / [`f_keys`] / [`f_values`] are the builtins that
//! turn a container into a list.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::deref::deref_to_field;
use crate::eval::vars::var_wrong_func_name_named;
use crate::hashtab::HashTab;
use crate::mbyte::cluster_len;
use crate::memory::ThinCString;
use crate::memory::handoff::owned_cstr;
use crate::os::libc::uint8_t;
use crate::semsg;
use crate::types::Failed;
use core::ffi::CStr;
use core::mem::ManuallyDrop;

/// A dictionary's hash table.
pub type DictTab = HashTab<DictEntry>;

/// One slot of a dictionary's hash table.
pub type ItemSlot = Slot<DictEntry>;

/// The dictionary already has an entry under that key — or, in a scope
/// dictionary, the key would shadow a builtin function — so nothing was
/// stored and the value the caller passed is still the caller's to free.
///
/// The `tv_dict_add_*` family answers the anonymous [`Failed`]; this is the
/// name that failure has for a caller who is filling a dictionary, and the
/// `From` below is what lets one `?` into the other.
#[derive(Clone, Copy, PartialEq, Eq, Debug, thiserror::Error)]
#[error("the key was already in the dictionary")]
pub struct KeyTaken;

impl From<Failed> for KeyTaken {
    fn from(_: Failed) -> Self {
        KeyTaken
    }
}

impl DictItem {
    /// A fresh item holding a copy of `key`, and `VAR_UNKNOWN`.
    ///
    /// The item owns its key: a short one -- which is nearly every key --
    /// lives in the item, a long one in its own allocation.  Upstream
    /// over-allocated the item so the key sat in a flexible tail, because its
    /// hash table's slot pointed at the key and found the item by subtracting
    /// an offset; the slot names the item now.
    ///
    /// The item is `DI_FLAGS_ALLOC`: a dictionary that takes it over
    /// ([`Dict::add_item`]) owns it, and gives it back on removal.
    pub fn boxed(key: &[u8]) -> Box<DictItem> {
        Box::new(DictItem {
            di_tv: TypVal::Unknown,
            di_lock: VarLock::Unlocked,
            di_flags: u8::try_from(DI_FLAGS_ALLOC).expect("a flag byte"),
            di_key: DictKey::new(key),
        })
    }
}

/// A fresh item holding a copy of `item`'s key and value.
pub fn tv_dict_item_copy(item: &DictItem) -> Box<DictItem> {
    let mut copy = DictItem::boxed(item.key());
    tv_copy(&item.di_tv, &mut copy.di_tv);
    copy
}

/// Remove the item under `key` from `dict` and release it; E685 when there
/// is none.
///
/// A handle, because releasing a value can reach the dictionary it was in:
/// the release happens once the removal's borrow has ended.
pub fn tv_dict_item_remove(dict: &DictRef, key: &[u8]) {
    let removed = dict.edit().remove_key(key);
    if removed.is_none() {
        let arg0 = "tv_dict_item_remove()";
        semsg!("E685: Internal error: {arg0}");
    }
    drop(removed);
}

/// `items()` over a blob: a list of `[index, byte]` pairs.
pub(crate) fn tv_blob2items(args: &[TypVal], result: &mut TypVal) {
    let bytes = blob_bytes(args[0].blob_ref());
    tv_list_alloc_ret(result, ptrdiff_t::try_from(bytes.len()).unwrap_or(-1));
    let Some(list) = result.list_mut() else {
        return;
    };
    for (at, &byte) in bytes.iter().enumerate() {
        let mut pair = tv_list_alloc(2);
        pair.push_number(VarNumber::try_from(at).expect("a short blob"));
        pair.push_number(VarNumber::from(byte));
        list.push_list(Some(pair));
    }
}

/// `items()` over a dictionary: a list of `[key, value]` pairs.
pub(crate) fn tv_dict2items(args: &[TypVal], result: &mut TypVal) {
    tv_dict2list(args, result, kDict2ListItems);
}

/// `items()` over a list: a list of `[index, value]` pairs.
pub(crate) fn tv_list2items(args: &[TypVal], result: &mut TypVal) {
    let source = args[0].list_ref();
    tv_list_alloc_ret(
        result,
        ptrdiff_t::try_from(list_len(source)).unwrap_or(ptrdiff_t::MAX),
    );
    if source.is_none() {
        return;
    }
    let Some(list) = result.list_mut() else {
        return;
    };
    for (idx, item) in list_iter(source).enumerate() {
        let mut pair = tv_list_alloc(2);
        pair.push_number(VarNumber::try_from(idx).expect("a short list"));
        pair.push_copy(&item.li_tv);
        list.push_list(Some(pair));
    }
}

/// `items()` over a string: a list of `[index, character]` pairs.
pub(crate) fn tv_string2items(args: &[TypVal], result: &mut TypVal) {
    tv_list_alloc_ret(result, kListLenMayKnow as ptrdiff_t);
    let Some(list) = result.list_mut() else {
        return;
    };
    // A null string behaves like an empty one.
    let text = args[0].string_bytes();

    let mut idx: VarNumber = 0;
    let mut offset = 0;
    while offset < text.len() {
        let len = cluster_len(&text[offset..]);
        let mut pair = tv_list_alloc(2);
        pair.push_number(idx);
        pair.push(TypVal::string_from(&text[offset..offset + len]));
        list.push_list(Some(pair));
        offset += len;
        idx += 1;
    }
}

impl Dict {
    /// Take the item under `key` out of the table, answering what the table
    /// held -- `None` when there is no such key. Dropping the answer is the
    /// release; do it once the borrow of the dictionary has ended.
    pub(crate) fn remove_key(&mut self, key: &[u8]) -> Option<RemovedItem> {
        let slot = self.slot_of(key)?;
        Some(self.remove_at(slot))
    }

    /// Whether the dictionary has `key`.
    #[inline]
    pub fn has_key(&self, key: &[u8]) -> bool {
        self.find(key).is_some()
    }
}

/// The item `d[key]`, or `None` when there is none -- including when `d` is
/// `v:_null_dict`, which holds nothing.
#[inline]
pub fn dict_find<'a>(d: Option<&'a Dict>, key: &[u8]) -> Option<&'a DictItem> {
    d?.find(key)
}

/// [`dict_find`] with the item writable.
#[inline]
pub fn dict_find_mut<'a>(d: Option<&'a mut Dict>, key: &[u8]) -> Option<&'a mut DictItem> {
    d?.find_mut(key)
}

/// Whether `d` has `key`; a NULL dictionary has none.
#[inline]
pub fn dict_has_key(d: Option<&Dict>, key: &[u8]) -> bool {
    dict_find(d, key).is_some()
}

/// Copy `d[key]` into `result`.  `Err` when there is no such key.
///
/// `result` must hold no value yet: it is overwritten, not cleared.
pub fn dict_get_tv(d: Option<&Dict>, key: &[u8], result: &mut TypVal) -> Result<(), Failed> {
    let di = dict_find(d, key).ok_or(Failed)?;
    tv_copy(&di.di_tv, result);
    Ok(())
}

/// `d[key]` as a number, or 0 when there is no such key.
pub fn dict_get_number(d: Option<&Dict>, key: &[u8]) -> VarNumber {
    dict_get_number_def(d, key, 0)
}

/// `d[key]` as a number, or `def` when there is no such key.
pub fn dict_get_number_def(d: Option<&Dict>, key: &[u8], def: ::core::ffi::c_int) -> VarNumber {
    match dict_find(d, key) {
        Some(di) => tv_get_number(&di.di_tv),
        None => VarNumber::from(def),
    }
}

/// `d[key]` as a boolean, or `def` when there is no such key.
pub fn dict_get_bool(d: Option<&Dict>, key: &[u8], def: ::core::ffi::c_int) -> VarNumber {
    match dict_find(d, key) {
        Some(di) => tv_get_bool(&di.di_tv),
        None => VarNumber::from(def),
    }
}

/// `denv` as a NULL-terminated `environ`-shaped array of `KEY=VALUE` strings.
///
/// Every string, and the array itself, is freshly allocated from the
/// allocator `xfree` releases; **the caller owns the lot** and has to free
/// it. Every value of `denv` must have a string form.
pub fn dict_to_env(denv: &Dict) -> *mut *mut ::core::ffi::c_char {
    let mut numbuf = NumBuf::new();
    // + 1 for NULL
    let mut env = Vec::with_capacity(denv.len() + 1);
    for var in denv.items() {
        let value = numbuf.string(&var.di_tv);
        env.push(owned_cstr([var.key(), b"=", value.to_bytes()].concat()));
    }
    // must be null terminated
    env.push(::core::ptr::null_mut());
    Box::into_raw(env.into_boxed_slice()).cast::<*mut ::core::ffi::c_char>()
}

/// `d[key]` as a fresh allocation **the caller owns**, NULL for a missing
/// key.
///
/// The `save` half of the C's `tv_dict_get_string`; the borrowing half is
/// [`dict_get_string_buf`], which renders into the caller's own [`NumBuf`]
/// rather than a process-wide one.
pub fn dict_get_string_alloc(d: Option<&Dict>, key: &[u8]) -> Option<ThinCString> {
    let mut numbuf = NumBuf::new();
    dict_get_string_buf(d, key, &mut numbuf).map(ThinCString::from_cstr)
}

/// `d[key]` as a string, formatting a number into `numbuf`.
///
/// The answer points into `numbuf` or borrows the item, so it lives no
/// longer than whichever of the two the value came from.
pub fn dict_get_string_buf<'a>(
    d: Option<&'a Dict>,
    key: &[u8],
    numbuf: &'a mut NumBuf,
) -> Option<&'a CStr> {
    dict_find(d, key).map(|di| numbuf.string(&di.di_tv))
}

/// [`dict_get_string_buf`] answering `def` for a missing key, and `None`
/// with an error raised for a value that has no string form.
pub fn dict_get_string_buf_chk<'a>(
    d: Option<&'a Dict>,
    key: &[u8],
    numbuf: &'a mut NumBuf,
    def: Option<&'a CStr>,
) -> Option<&'a CStr> {
    match dict_find(d, key) {
        Some(di) => numbuf.string_chk(&di.di_tv).ok(),
        None => def,
    }
}

/// Why [`dict_get_callback`] could not read a callback.
#[derive(Debug, thiserror::Error)]
pub enum CallbackRefused {
    /// The value is neither a function nor a function name.
    #[error("{}", crate::tr!("E6000: Argument is not a function or function name"))]
    NotAFunction,
    /// A function name `callback_from_typval` would not take (one that
    /// starts with a digit, a Lua table it could not register): upstream
    /// refuses it without a word, and so does [`Self::report`].
    #[error("unusable callback")]
    Unusable,
}

impl CallbackRefused {
    /// Say the refusal, where there is one to say.
    pub fn report(self) {
        if let CallbackRefused::NotAFunction = self {
            crate::message_fmt::emsg_text(self.to_string());
        }
    }
}

/// `d[key]` as a callback, bound to `d` as its `self` dictionary.
///
/// A missing key — or a NULL dictionary — answers `Ok` with `result` left as
/// `kCallbackNone`.  `result` must hold no callback yet -- it is overwritten,
/// not freed -- and on `Ok` the caller owns whatever it now holds.
pub fn dict_get_callback(
    d: Option<&DictRef>,
    key: &[u8],
    result: &mut Callback,
) -> Result<(), CallbackRefused> {
    *result = Callback::None;
    // A NULL dictionary has no such key, which is the missing-key answer.
    let Some(d) = d else { return Ok(()) };
    let mut tv = TV_INITIAL_VALUE;
    match d.find(key) {
        None => return Ok(()),
        Some(di) if !di.di_tv.is_func() && di.di_tv.v_type() != VAR_STRING => {
            return Err(CallbackRefused::NotAFunction);
        }
        Some(di) => tv_copy(&di.di_tv, &mut tv),
    }

    // The borrow of the item is over, so the dictionary can be named again:
    // the partial the value becomes takes a *reference* to it, and that is
    // the only thing `set_selfdict` does to it.
    set_selfdict(&mut tv, d);
    let res = callback_from_typval(result, &tv);
    tv_clear(&mut tv);
    if res.is_ok() {
        Ok(())
    } else {
        Err(CallbackRefused::Unusable)
    }
}

/// Whether storing `item` in `d` would shadow a builtin function.
///
/// Only the global scope and a function's local scope are guarded, and both
/// are read through their globals, so this is the editor's own thread's to
/// call.
///
/// **The item, not its key as a `&CStr`.** Every insertion runs this, and
/// building a `&CStr` out of a `DictKey` *validates* it — a scan of the key
/// on the hot path, which is the 2.5 % `evalbench` p30-9 §7 already paid
/// once. [`DictKey::as_ptr`] is the read that costs nothing, and the guards
/// above it rule the call out before the key is touched at all.
#[inline]
pub(crate) fn dict_wrong_func_name(d: &Dict, item: &DictItem) -> bool {
    item.di_tv.is_func()
        && (is_globvar_dict(d) || is_funccal_local_dict(d))
        && var_wrong_func_name_named(item.key(), true)
}

/// The shared body of `keys()`, `values()` and `items()` over a dictionary.
pub(crate) fn tv_dict2list(args: &[TypVal], result: &mut TypVal, what: DictListType) {
    if tv_check_for_dict_arg(args, 0).is_err() {
        tv_list_alloc_ret(result, 0);
        return;
    }

    let d = args[0].dict_ref();
    tv_list_alloc_ret(
        result,
        ptrdiff_t::try_from(dict_len(d)).unwrap_or(ptrdiff_t::MAX),
    );
    // NULL dict behaves like an empty dict
    let Some(d) = d else { return };
    let Some(list) = result.list_mut() else {
        return;
    };

    for di in d.items() {
        let mut tv_item = TV_INITIAL_VALUE;

        match what {
            kDict2ListKeys => {
                tv_item.write_string(Some(ThinCString::from_bytes(di.di_key.bytes())));
            }
            kDict2ListValues => {
                tv_copy(&di.di_tv, &mut tv_item);
            }
            kDict2ListItems => {
                // items()
                let mut pair = tv_list_alloc(2);
                pair.push(TypVal::string(Some(ThinCString::from_bytes(di.key()))));
                pair.push_copy(&di.di_tv);
                tv_item.write_list(Some(pair));
            }
            _ => {}
        }

        list.push(tv_item);
    }
}

/// `items()`: index/value pairs of a string, list, blob or dictionary.
pub fn f_items(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    match args[0].v_type() {
        VAR_STRING => tv_string2items(args, result),
        VAR_LIST => tv_list2items(args, result),
        VAR_BLOB => tv_blob2items(args, result),
        VAR_DICT => tv_dict2items(args, result),
        _ => {
            semsg!(
                "E1225: List, Dictionary, Blob or String required for argument {}",
                1 as ::core::ffi::c_int
            );
        }
    }
}

/// `keys()`: the keys of a dictionary.
pub fn f_keys(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    tv_dict2list(args, result, kDict2ListKeys);
}

/// `values()`: the values of a dictionary.
pub fn f_values(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    tv_dict2list(args, result, kDict2ListValues);
}

/// `has_key()`: whether a dictionary has a key.
pub fn f_has_key(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    if tv_check_for_dict_arg(args, 0).is_err() {
        return;
    }
    let found = dict_has_key(args[0].dict_ref(), numbuf.bytes(&args[1]));
    result.write_number(VarNumber::from(found));
}

impl NumBuf {
    /// `d[key]` as a string, `None` for a missing key. The borrowing half of
    /// the C's `tv_dict_get_string`; [`dict_get_string_alloc`] is the other
    /// one.
    pub fn dict_string<'a>(&'a mut self, d: Option<&'a Dict>, key: &[u8]) -> Option<&'a CStr> {
        dict_get_string_buf(d, key, self)
    }
}

impl Dict {
    /// The item under `key`, or `None` when there is none.
    ///
    /// The key is bytes: the table hashes and compares exactly those, and a
    /// key that reaches the table has no NUL among them. **The answer borrows
    /// the dictionary, not the slot**: an item is its own allocation, so a
    /// rehash leaves it where it was, and only a removal -- which needs the
    /// exclusive borrow this rules out -- ends it.
    #[inline]
    pub fn find(&self, key: &[u8]) -> Option<&DictItem> {
        self.slot_of(key).and_then(|slot| self.item_at(slot))
    }

    /// [`Dict::find`] with the item writable.
    #[inline]
    pub fn find_mut(&mut self, key: &[u8]) -> Option<&mut DictItem> {
        self.slot_of(key).and_then(|slot| self.item_at_mut(slot))
    }
}

/// What [`Dict::remove_at`] took out of a table: an allocated item, or the
/// value of one embedded in a structure that keeps the item itself.
/// Dropping it releases the value.
pub(crate) enum RemovedItem {
    /// An item the dictionary owned.
    Allocated(Box<DictItem>),
    /// The value of an embedded item, which stays where it is, emptied.
    Embedded(TypVal),
}

impl RemovedItem {
    /// The value the item held.
    pub(crate) fn value_mut(&mut self) -> &mut TypVal {
        match self {
            RemovedItem::Allocated(item) => &mut item.di_tv,
            RemovedItem::Embedded(value) => value,
        }
    }
}

/// A walk over a dictionary's occupied slots that holds no borrow of it
/// between steps: upstream's `TV_DICT_ITER`.
///
/// The live-item count is taken at the start, as the macro does, which is
/// what lets a body remove entries as it goes -- with the table locked
/// ([`Dict::lock_table`]), since an unlocked removal may rehash and renumber
/// the slots under the walk.
pub(crate) struct DictCursor {
    slot: usize,
    todo: usize,
}

impl DictCursor {
    /// A walk from the first slot of `dict`.
    pub(crate) fn new(dict: &Dict) -> DictCursor {
        DictCursor {
            slot: 0,
            todo: dict.dv_hashtab.ht_used,
        }
    }

    /// The next occupied slot of `dict`, or `None` at the end.
    ///
    /// The walk also ends at the last slot: a body that ran user code may
    /// have removed entries from an unlocked table, which can shrink it.
    pub(crate) fn next(&mut self, dict: &Dict) -> Option<usize> {
        let slots = dict.dv_hashtab.slots();
        while self.todo != 0 && self.slot < slots.len() {
            let slot = self.slot;
            self.slot += 1;
            if slots[slot].is_kept() {
                self.todo -= 1;
                return Some(slot);
            }
        }
        None
    }
}

/// The [`DictItem`] a scope dictionary is reached through, whose key is the
/// empty string and whose value is the scope dictionary itself.
///
/// It is `release_var_scope` that gives up the reference -- a buffer, a window
/// and a tab page each embed one of these and are ordinary Rust values, so
/// without [`ManuallyDrop`](core::mem::ManuallyDrop) freeing one would
/// release the scope twice.  The item is wrapped rather than duplicated: its
/// key is the empty string, which is inline, so nothing is leaked by not
/// dropping it.
///
/// `#[repr(transparent)]`: the scopes hand their own entry around as a bare
/// `*mut DictItem`, and the collector names its value by adding
/// `offset_of!(DictItem, di_tv)` to the field's offset in a buffer, a window
/// or a tab page.  Both are casts through this wrapper.
#[repr(transparent)]
pub struct ScopeDictItem(pub ManuallyDrop<DictItem>);

impl ScopeDictItem {
    /// A scope's entry before its dictionary exists.
    pub const EMPTY: Self = ScopeDictItem(ManuallyDrop::new(DictItem {
        di_tv: TypVal::empty(VAR_UNKNOWN),
        di_lock: VarLock::Unlocked,
        di_flags: 0,
        di_key: DictKey::EMPTY,
    }));

    /// The item itself, which is what a hashtab slot names.
    pub fn item(&mut self) -> *mut DictItem {
        &raw mut *self.0
    }

    /// The scope dictionary's address, for `vim.b`/`vim.w`/`vim.t`; null
    /// before the scope exists.
    pub fn dict_ptr(&self) -> *mut Dict {
        self.di_tv
            .dict_shared()
            .map_or(::core::ptr::null_mut(), DictRef::as_ptr)
    }
}

deref_to_field!(ScopeDictItem => mut DictItem, 0);

/// A dictionary item's key: its own bytes, NUL-terminated.
///
/// Short keys -- which in practice is nearly all of them, and *every* key of
/// the four kinds of item that are embedded in something bigger rather than
/// allocated by the dictionary -- live in the item.  Upstream got that by
/// over-allocating the item and letting the hash table's slot point into the
/// tail, so the item could be recovered by subtracting an offset from the
/// slot's key pointer; the slot names the item directly now
/// ([`SlotEntry`](crate::hashtab::SlotEntry)), and the key is the item's own.
///
/// The bytes are always NUL-terminated, whichever arm holds them, because
/// the probe compares NUL-terminated keys and always has.
///
/// `#[repr(u8)]` for one reason: it puts a real tag byte in the value and
/// fixes `Inline`'s discriminant at zero, so **all-zero storage is a valid
/// empty key** -- and dropping one is a no-op.  That is what lets a
/// `DictItem` sit in `xcalloc`'d memory and be filled in field by field: a
/// funccall's twelve fixed variables, a buffer's `b:changedtick`, a scope's
/// own entry.  It costs nothing: the tag lands in the tail padding either
/// way (asserted below).
#[derive(Clone)]
#[repr(u8)]
pub enum DictKey {
    /// Up to [`DictKey::INLINE_MAX`] bytes plus the NUL, in the item itself.
    ///
    /// First, and with a zero length, so that all-zero storage is a valid
    /// empty key: a `FuncCall`'s twelve fixed variables arrive `xcalloc`'d.
    Inline {
        len: uint8_t,
        bytes: [uint8_t; DictKey::INLINE_CAP],
    },
    /// A longer key, NUL included.
    Heap(Box<[uint8_t]>),
}

impl DictKey {
    /// How many bytes the inline arm holds, NUL included.  Sized so that the
    /// enum is no larger than the boxed arm forces it to be, and so that
    /// every embedded item's key fits: the longest is a funccall's
    /// twenty-character short name.
    pub const INLINE_CAP: usize = 22;
    /// The longest key that stays in the item.
    pub const INLINE_MAX: usize = Self::INLINE_CAP - 1;

    /// The empty key, which is what an uninitialised item carries.
    pub const EMPTY: Self = DictKey::Inline {
        len: 0,
        bytes: [0; Self::INLINE_CAP],
    };

    /// A key holding a copy of `bytes`, NUL-terminated.
    pub fn new(bytes: &[uint8_t]) -> Self {
        if bytes.len() <= Self::INLINE_MAX {
            let mut inline = [0; Self::INLINE_CAP];
            inline[..bytes.len()].copy_from_slice(bytes);
            DictKey::Inline {
                len: uint8_t::try_from(bytes.len()).expect("an inline key is at most 21 bytes"),
                bytes: inline,
            }
        } else {
            let mut heap = Vec::with_capacity(bytes.len() + 1);
            heap.extend_from_slice(bytes);
            heap.push(0);
            DictKey::Heap(heap.into_boxed_slice())
        }
    }

    /// The key's bytes, without the terminating NUL.
    pub fn bytes(&self) -> &[uint8_t] {
        match self {
            DictKey::Inline { len, bytes } => &bytes[..usize::from(*len)],
            DictKey::Heap(boxed) => &boxed[..boxed.len() - 1],
        }
    }

    /// How many bytes the key is, without the NUL.
    pub fn len(&self) -> usize {
        self.bytes().len()
    }

    /// Whether the key is the empty string, which is a key like any other:
    /// `{'': 1}` is a dictionary of one entry.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The key's first byte, NUL-terminated: upstream's `di_key`.
    ///
    /// This is what every hash-table probe reads, so it is a match and a
    /// field address and nothing else.  Going through
    /// [`as_c_str`](Self::as_c_str) instead cost `evalbench` two and a half
    /// per cent: `CStr::from_bytes_with_nul` *validates*, which is a scan of
    /// the key on every lookup.
    pub fn as_ptr(&self) -> *const ::core::ffi::c_char {
        match self {
            DictKey::Inline { bytes, .. } => bytes.as_ptr().cast(),
            DictKey::Heap(boxed) => boxed.as_ptr().cast(),
        }
    }

    /// The key as a C string, for a caller that wants one rather than a
    /// pointer.  Validating, and therefore linear: not for a probe.
    pub fn as_c_str(&self) -> &::core::ffi::CStr {
        let with_nul = match self {
            DictKey::Inline { len, bytes } => &bytes[..usize::from(*len) + 1],
            DictKey::Heap(boxed) => boxed,
        };
        ::core::ffi::CStr::from_bytes_with_nul(with_nul).expect("a key is NUL-terminated once")
    }
    /// A key of `len` bytes, every one of them NUL, and a pointer the
    /// caller may write those `len` bytes through.
    ///
    /// For the decoders that learn a key's length from a header and then
    /// receive its bytes in chunks: the storage is the key's own from the
    /// start, so a message that ends early leaves a short key rather than a
    /// dangling one.
    pub fn zeroed(len: usize) -> Self {
        if len <= Self::INLINE_MAX {
            DictKey::Inline {
                len: uint8_t::try_from(len).expect("an inline key is at most 21 bytes"),
                bytes: [0; Self::INLINE_CAP],
            }
        } else {
            DictKey::Heap(vec![0; len + 1].into_boxed_slice())
        }
    }

    /// The key's bytes, writable, without the terminating NUL. Pairs with
    /// [`zeroed`](Self::zeroed); the NUL past them is not writable through
    /// this.
    pub fn bytes_mut(&mut self) -> &mut [uint8_t] {
        match self {
            DictKey::Inline { len, bytes } => &mut bytes[..usize::from(*len)],
            DictKey::Heap(boxed) => {
                let end = boxed.len() - 1;
                &mut boxed[..end]
            }
        }
    }
}

impl PartialEq for DictKey {
    /// Byte equality, whichever arm each side is in.
    fn eq(&self, other: &Self) -> bool {
        self.bytes() == other.bytes()
    }
}

impl Eq for DictKey {}

impl ::core::fmt::Debug for DictKey {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        ::core::fmt::Debug::fmt(&self.bytes().escape_ascii().to_string(), f)
    }
}

impl From<&::core::ffi::CStr> for DictKey {
    fn from(key: &::core::ffi::CStr) -> Self {
        Self::new(key.to_bytes())
    }
}

impl From<&[uint8_t]> for DictKey {
    fn from(key: &[uint8_t]) -> Self {
        Self::new(key)
    }
}

impl From<&str> for DictKey {
    fn from(key: &str) -> Self {
        Self::new(key.as_bytes())
    }
}

#[cfg(not(randomized_layout))]
const _: () = {
    assert!(::core::mem::size_of::<DictKey>() == 24);
    assert!(::core::mem::size_of::<DictItem>() == 48);
};

impl DictItem {
    /// The item's key, without the terminating NUL.
    ///
    /// The cheap spelling, and the one that shares its name with
    /// `DictItemRef::key`: both answer bytes in constant time, so a caller
    /// reaching for "the key" gets the same thing on either type.
    pub fn key(&self) -> &[uint8_t] {
        self.di_key.bytes()
    }

    /// The item's key as a C string, for a caller that wants a *string*.
    ///
    /// **Not for a hot path.** Building one validates the key (see
    /// [`DictKey::as_c_str`]); a caller that wants a pointer to pass on
    /// wants [`DictKey::as_ptr`] on [`di_key`](Self::di_key) instead.
    pub fn key_cstr(&self) -> &::core::ffi::CStr {
        self.di_key.as_c_str()
    }
}

/// One entry of a [`Dict`](crate::types::Dict), owning its key.
///
/// `di_lock` is the *slot's* lock: `:lockvar` on a variable locks the item it
/// lives in, not the value it currently holds.
///
/// This is the only item shape there is.  The four kinds that are embedded
/// in something bigger -- a funccall's fixed variables, a scope dictionary's
/// own entry ([`ScopeDictItem`]), `b:changedtick` and a `v:` row -- were
/// separate structs only because each spelled the flexible key member out at
/// a different length; an owned key makes them all this.
pub struct DictItem {
    pub di_tv: TypVal,
    pub di_lock: VarLock,
    pub di_flags: uint8_t,
    pub di_key: DictKey,
}
