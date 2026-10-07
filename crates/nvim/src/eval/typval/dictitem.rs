//! A dictionary's items: the slots that name them, the pair that allocates
//! and frees one, and reading a value back out.
//!
//! The table's slots name the items ([`DictEntry`]), where upstream's named
//! the item's *key* -- an item was over-allocated so its key sat in a
//! flexible tail, and the item was the key pointer minus a constant. That is
//! why an item could not own its key. Reading the key out of the entry costs
//! the probe nothing -- [`DictEntry::key`] answers the same NUL-terminated
//! bytes the slot used to hold -- and so leaves the hash, the probe
//! sequence, the resize thresholds and the slot every key lands in exactly
//! as they were. That matters: slot order is what `keys()`, `values()` and
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
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::eval::vars::var_wrong_func_name_named;
use crate::mbyte::cluster_len;
use crate::memory::ThinCString;
use crate::memory::handoff::owned_cstr;
use crate::message::emsg;
use crate::semsg;
use crate::types::{DictKey, Failed, HashTab};
use core::ffi::CStr;

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
            di_flags: DI_FLAGS_ALLOC as uint8_t,
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
    /// The item under `key` as a pointer, or null: the escape hatch for the
    /// bodies that hold an item *while* they reach the dictionary again.
    /// Safe to compute; it is the dereference that needs a promise.
    #[inline]
    pub(crate) fn find_ptr(&self, key: &[u8]) -> *mut DictItem {
        self.slot_of(key).map_or(::core::ptr::null_mut(), |slot| {
            tv_dict_hi2di(self.dv_hashtab.slot(slot))
        })
    }

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
        Some(di) => numbuf.string_chk(&di.di_tv),
        None => def,
    }
}

/// `d[key]` as a callback, bound to `d` as its `self` dictionary.
///
/// A missing key — or a NULL dictionary — answers true with `result` left as
/// `kCallbackNone`; a value that is neither a function nor a string answers
/// false with `E6000` raised.  `result` must hold no callback yet -- it is overwritten, not
/// freed -- and on `true` the caller owns whatever it now holds.
pub fn dict_get_callback(d: Option<&mut Dict>, key: &[u8], result: &mut Callback) -> bool {
    *result = Callback::None;
    // A NULL dictionary has no such key, which is the missing-key answer.
    let Some(d) = d else { return true };
    let mut tv = TV_INITIAL_VALUE;
    match d.find(key) {
        None => return true,
        Some(di) if !di.di_tv.is_func() && di.di_tv.v_type() != VAR_STRING => {
            let msg = gettext(c"E6000: Argument is not a function or function name");
            emsg(msg);
            return false;
        }
        Some(di) => tv_copy(&di.di_tv, &mut tv),
    }

    // The borrow of the item is over, so the dictionary can be named again:
    // the partial the value becomes takes a *reference* to it, and that is
    // the only thing `set_selfdict` does to it.
    set_selfdict(&mut tv, d);
    let res = callback_from_typval(result, &tv);
    tv_clear(&mut tv);
    res
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
    let at = ::core::ptr::from_ref(d);
    (at == get_globvar_dict().cast_const()
        || ::core::ptr::eq(&d.dv_hashtab, get_funccal_local_ht().cast_const()))
        && item.di_tv.is_func()
        && var_wrong_func_name_named(item.key(), true)
}

/// The shared body of `keys()`, `values()` and `items()` over a dictionary.
pub(crate) fn tv_dict2list(args: &[TypVal], result: &mut TypVal, what: DictListType) {
    if tv_check_for_dict_arg(args, 0).is_err() {
        tv_list_alloc_ret(result, 0);
        return;
    }

    let d = args[0].dict_ref();
    tv_list_alloc_ret(result, dict_len(d) as ptrdiff_t);
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
    pub(crate) fn next(&mut self, dict: &Dict) -> Option<usize> {
        while self.todo != 0 {
            let slot = self.slot;
            self.slot += 1;
            if dict.dv_hashtab.slot(slot).is_kept() {
                self.todo -= 1;
                return Some(slot);
            }
        }
        None
    }
}
