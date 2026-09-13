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
//! `DI_FLAGS_ALLOC`, and [`tv_dict_item_free`] is what acts on it -- four
//! kinds of item are embedded in something bigger (a funccall's fixed
//! variables, a scope's own entry, `b:changedtick`, a `v:` row) and outlive
//! every table they are in.  The `tv_dict_item_*` family keeps its raw
//! pointers for the reason `tv_list_free` does: an item off a table is
//! owned by nobody, so no borrow describes it.
//!
//! [`Dict::find`] is the hashtable lookup every getter goes through, and the
//! `dict_get_*` family coerces what it finds to one type, answering a
//! caller-supplied default when the key is absent or the wrong kind.  Each
//! has a free form over `Option<&Dict>`, because `v:_null_dict` is a real
//! value and not an error state.  [`dict_to_env`] builds the
//! `environ`-shaped array a job's environment is passed as.  The `*2items`
//! half and [`f_items`] / [`f_keys`] / [`f_values`] are the builtins that
//! turn a container into a list.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use ::core::ffi::CStr;

use super::*;
use crate::cstr;
use crate::hashtab::removed_sentinel;
use crate::message::emsg_ptr;
use crate::semsg;
use crate::types::NUL;
use crate::types::{DictKey, Failed, HashTab, SlotEntry};

/// What a dictionary's hash table holds in an occupied slot: the item.
///
/// `Copy`, and a raw pointer, for the same reason the slots of every other
/// table are: the table names its items, and which of them it is responsible
/// for freeing is the item's own `DI_FLAGS_ALLOC`.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct DictEntry(*mut DictItem);

impl DictEntry {
    /// The entry for `item`.
    pub fn new(item: *mut DictItem) -> Self {
        DictEntry(item)
    }

    /// The item this entry names. Meaningless for an empty or removed slot.
    pub fn item(self) -> *mut DictItem {
        self.0
    }
}

// SAFETY: `EMPTY` is the null pointer and `is_empty` is the null test; the
// tombstone is the hash table's own private sentinel address, which no item
// can be allocated at and which is never dereferenced; `key` reads the key
// of an item the caller has promised is live, and a `DictItem` owns its key
// for as long as it is alive.
unsafe impl SlotEntry for DictEntry {
    const EMPTY: Self = DictEntry(::core::ptr::null_mut());

    fn is_empty(self) -> bool {
        self.0.is_null()
    }

    fn is_removed(self) -> bool {
        self.0 == removed_sentinel().cast::<DictItem>()
    }

    fn removed() -> Self {
        DictEntry(removed_sentinel().cast::<DictItem>())
    }

    /// # Safety
    ///
    /// As the trait's: a live entry, whose item is still alive.
    unsafe fn key(self) -> *const ::core::ffi::c_char {
        // SAFETY: the caller's promise -- a live item, which owns its key.
        unsafe { (*self.0).di_key.as_ptr() }
    }
}

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

/// Allocate a `DictItem` holding a copy of `key`'s first `key_len` bytes.
///
/// The item owns its key: a short one -- which is nearly every key -- lives
/// in the item, a long one in its own allocation.  Upstream over-allocated
/// the item so the key sat in a flexible tail, because its hash table's slot
/// pointed at the key and found the item by subtracting an offset; the slot
/// names the item now.
///
/// # Safety
/// `key` must be readable for `key_len` bytes; it need not be
/// NUL-terminated, since the key terminates itself.
///
/// The item comes back owned by the caller, holding `VAR_UNKNOWN`. It has
/// to reach either [`Dict::add_item`] (which takes it on `Ok` and leaves
/// it on `Err`) or [`tv_dict_item_free`]; nothing else frees it.
pub unsafe fn tv_dict_item_alloc_len(
    key: *const ::core::ffi::c_char,
    key_len: size_t,
) -> *mut DictItem {
    // SAFETY: the caller's promise -- `key_len` readable bytes.
    let bytes = unsafe { ::core::slice::from_raw_parts(key.cast::<u8>(), key_len) };
    Box::into_raw(Box::new(DictItem {
        di_tv: TypVal::Unknown,
        di_lock: VarLock::Unlocked,
        di_flags: DI_FLAGS_ALLOC as uint8_t,
        di_key: DictKey::new(bytes),
    }))
}

/// [`tv_dict_item_alloc_len`] for a NUL-terminated key.
///
/// # Safety
/// `key` must be a NUL-terminated string. Otherwise as
/// [`tv_dict_item_alloc_len`], including the caller's ownership of the
/// result.
pub unsafe fn tv_dict_item_alloc(key: *const ::core::ffi::c_char) -> *mut DictItem {
    unsafe { tv_dict_item_alloc_len(key, cstr::bytes_at(key).len()) }
}

/// Clear `item`'s value and free it, if it was allocated (rather than
/// embedded in a `FuncCall`'s fixed-variable array, a scope dictionary, a
/// buffer's `b:changedtick` or a `v:` row).
///
/// # Safety
/// `item` must be a live item that is **not** in any hashtab -- remove it
/// first, or the hashtab is left pointing at freed memory. An item with
/// `DI_FLAGS_ALLOC` is dangling afterwards; an embedded one is merely
/// emptied, keeping its key: the storage is not this call's to release.
pub unsafe fn tv_dict_item_free(item: *mut DictItem) {
    if unsafe { (*item).di_flags } as ::core::ffi::c_uint & DI_FLAGS_ALLOC != 0 {
        // The value and the key go with the item.
        // SAFETY: the caller's live item, which this allocated.
        drop(unsafe { Box::from_raw(item) });
    } else {
        unsafe { tv_clear(&mut (*item).di_tv) };
    }
}

/// A fresh item holding a copy of `di`'s key and value.
///
/// # Safety
/// `di` must be a live item. The copy is the caller's, with the same
/// obligation as [`tv_dict_item_alloc_len`]'s result.
pub unsafe fn tv_dict_item_copy(di: *mut DictItem) -> *mut DictItem {
    let new_di = unsafe { tv_dict_item_alloc(tv_dict_item_key(di)) };
    unsafe { tv_copy(&(*di).di_tv, &mut (*new_di).di_tv) };
    new_di
}

/// Remove `item` from `dict` and free it.
///
/// # Safety
/// `item` must be an item of `dict`, and both must be live. `item` is
/// freed, so the caller must not hold it afterwards.
pub unsafe fn tv_dict_item_remove(dict: *mut Dict, item: *mut DictItem) {
    let hi = unsafe { hash_find(&raw mut (*dict).dv_hashtab, tv_dict_item_key(item)) };
    if hi.is_kept() {
        unsafe { hash_remove(&raw mut (*dict).dv_hashtab, hi) };
    } else {
        let arg0 = "tv_dict_item_remove()";
        semsg!("E685: Internal error: {arg0}");
    }
    unsafe { tv_dict_item_free(item) };
}

/// `items()` over a blob: a list of `[index, byte]` pairs.
pub(crate) fn tv_blob2items(args: &[TypVal], result: &mut TypVal) {
    let bytes = blob_bytes(args[0].blob_ref());
    tv_list_alloc_ret(result, ptrdiff_t::try_from(bytes.len()).unwrap_or(-1));
    for (at, &byte) in bytes.iter().enumerate() {
        let pair = tv_list_alloc(2);
        let into = pair.as_ptr();
        // SAFETY: the list stored in the return slot, and the fresh pair.
        unsafe { (*result.list_or_null()).push_list(Some(pair)) };
        unsafe { (*into).push_number(VarNumber::try_from(at).expect("a short blob")) };
        unsafe { (*into).push_number(VarNumber::from(byte)) };
    }
}

/// `items()` over a dictionary: a list of `[key, value]` pairs.
pub(crate) fn tv_dict2items(args: &[TypVal], result: &mut TypVal) {
    tv_dict2list(args, result, kDict2ListItems);
}

/// `items()` over a list: a list of `[index, value]` pairs.
pub(crate) fn tv_list2items(args: &[TypVal], result: &mut TypVal) {
    let l = args[0].list_or_null();
    tv_list_alloc_ret(result, list_len(unsafe { l.as_ref() }) as ptrdiff_t);
    if l.is_null() {
        return;
    }
    for (idx, li) in list_iter(unsafe { l.as_ref() }).enumerate() {
        let l2 = tv_list_alloc(2);
        let at = l2.as_ptr();
        unsafe { (*(*result).list_or_null()).push_list(Some(l2)) };
        unsafe { (*at).push_number(idx as VarNumber) };
        unsafe { (*at).push_copy(&li.li_tv) };
    }
}

/// `items()` over a string: a list of `[index, character]` pairs.
pub(crate) fn tv_string2items(args: &[TypVal], result: &mut TypVal) {
    let mut p = args[0].string_or_null().cast_const();

    tv_list_alloc_ret(result, kListLenMayKnow as ptrdiff_t);
    if p.is_null() {
        return; // null string behaves like an empty string
    }

    let mut idx: VarNumber = 0;
    while unsafe { *p } as ::core::ffi::c_int != NUL {
        let len = unsafe { utfc_ptr2len(p) };
        if len == 0 {
            break;
        }
        let l2 = tv_list_alloc(2);
        let at = l2.as_ptr();
        unsafe { (*(*result).list_or_null()).push_list(Some(l2)) };
        unsafe { (*at).push_number(idx) };
        unsafe { (*at).push_string(p, len as ssize_t) };
        p = unsafe { p.offset(len as isize) };
        idx += 1;
    }
}

impl Dict {
    /// The item under `key`, or `None` when there is none.
    ///
    /// The key is bytes, not a NUL-terminated string: the table hashes and
    /// compares exactly the bytes it is given, and the two spellings the C
    /// had (`hash_find` and `hash_find_len`) agree on every key a dictionary
    /// can hold, since a key that reaches the table is NUL-terminated and so
    /// has no NUL among its bytes.  A caller holding a `&CStr` says
    /// `key.to_bytes()`; one holding a `c"..."` literal pays nothing for it.
    ///
    /// **The answer borrows the dictionary, not the slot.** An item is its
    /// own allocation, so a rehash moves the slot and leaves the item where
    /// it was; what invalidates the borrow is the item being *removed*,
    /// which needs the exclusive borrow this one rules out.
    #[inline]
    pub fn find(&self, key: &[u8]) -> Option<&DictItem> {
        let at = self.find_ptr(key);
        // SAFETY: a non-null answer is one of this dictionary's own items,
        // which the borrow of the dictionary keeps alive.
        (!at.is_null()).then(|| unsafe { &*at })
    }

    /// [`Dict::find`] with the item writable.
    #[inline]
    pub fn find_mut(&mut self, key: &[u8]) -> Option<&mut DictItem> {
        let at = self.find_ptr(key);
        // SAFETY: a non-null answer is one of this dictionary's own items;
        // no two slots name the same one, and the exclusive borrow of the
        // dictionary keeps the set of them fixed.
        (!at.is_null()).then(|| unsafe { &mut *at })
    }

    /// The lookup both borrow forms are built on: the item under `key`, or
    /// null.
    ///
    /// **The answer is not derived from the borrow.** The table's slot holds
    /// a `*mut DictItem` that came from the item's own allocation, and this
    /// copies it out, so writing through it is sound where casting a shared
    /// borrow's address would not be.
    ///
    /// It is the escape hatch for the two bodies that must hold an item
    /// *while* they reach the dictionary again -- `extend()`'s overwrite
    /// branch, which re-enters through `value_check_lock`, and `remove()`,
    /// which takes the value out and then unlinks the item. Everything else
    /// wants [`Dict::find`].
    #[inline]
    pub(crate) fn find_ptr(&self, key: &[u8]) -> *mut DictItem {
        // SAFETY: a live table of this dictionary's own, and `key` is a
        // slice, so it is readable for its length. The two spellings the C
        // had agree here: a key that reaches the table is NUL-terminated, so
        // it has no NUL among its bytes, which is the only input the
        // length-taking hash treats differently.
        let hi = unsafe {
            hash_find_len(
                &raw const self.dv_hashtab,
                key.as_ptr().cast::<::core::ffi::c_char>(),
                key.len(),
            )
        };
        if hi.is_kept() {
            tv_dict_hi2di(hi)
        } else {
            ::core::ptr::null_mut()
        }
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
/// Every string, and the array itself, is freshly allocated; **the caller
/// owns the lot** and has to free it. Every value of `denv` must have a
/// string form.
pub fn dict_to_env(denv: &Dict) -> *mut *mut ::core::ffi::c_char {
    let mut numbuf = NumBuf::new();
    let env_size = denv.len();

    // + 1 for NULL
    let env =
        unsafe { xmalloc((env_size + 1) * ::core::mem::size_of::<*mut ::core::ffi::c_char>()) }
            as *mut *mut ::core::ffi::c_char;

    for (i, var) in denv.items().enumerate() {
        let key = var.key();
        let str = numbuf.string_ptr(&var.di_tv);
        debug_assert!(!str.is_null());
        // SAFETY: a non-null answer is a NUL-terminated string.
        let len = key.count_bytes() + unsafe { cstr::bytes_at(str) }.len() + c"=".count_bytes() + 1;
        // SAFETY: `i` is below `env_size`, and the format spends two
        // NUL-terminated strings into `len` writable bytes.
        unsafe {
            *env.add(i) = xmalloc(len) as *mut ::core::ffi::c_char;
            snprintf(*env.add(i), len, c"%s=%s".as_ptr(), key.as_ptr(), str);
        }
    }

    // must be null terminated
    // SAFETY: the slot past the last entry, which the allocation has room for.
    unsafe { *env.add(env_size) = ::core::ptr::null_mut() };
    env
}

/// `d[key]` as a fresh allocation **the caller owns**, NULL for a missing
/// key.
///
/// The `save` half of the C's `tv_dict_get_string`; the borrowing half is
/// [`dict_get_string_buf`], which renders into the caller's own [`NumBuf`]
/// rather than a process-wide one.
pub fn dict_get_string_alloc(d: Option<&Dict>, key: &[u8]) -> *mut ::core::ffi::c_char {
    let mut numbuf = NumBuf::new();
    let s = dict_get_string_buf(d, key, &mut numbuf);
    if s.is_null() {
        return ::core::ptr::null_mut();
    }
    // SAFETY: a non-null answer is a NUL-terminated string.
    unsafe { xstrdup(s) }
}

/// `d[key]` as a string, formatting a number into `numbuf`.
///
/// The answer points into `numbuf` or borrows the item, so it lives no
/// longer than whichever of the two the value came from.
pub fn dict_get_string_buf(
    d: Option<&Dict>,
    key: &[u8],
    numbuf: &mut NumBuf,
) -> *const ::core::ffi::c_char {
    match dict_find(d, key) {
        Some(di) => numbuf.string_ptr(&di.di_tv),
        None => ::core::ptr::null(),
    }
}

/// [`dict_get_string_buf`] answering `def` for a missing key, and NULL with
/// an error raised for a value that has no string form.
///
/// `def` is returned as-is, so its lifetime is the caller's problem.
pub fn dict_get_string_buf_chk(
    d: Option<&Dict>,
    key: &[u8],
    numbuf: &mut NumBuf,
    def: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    match dict_find(d, key) {
        Some(di) => numbuf.string_ptr_chk(&di.di_tv),
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
            let msg = tr(c"E6000: Argument is not a function or function name");
            // SAFETY: a NUL-terminated message from the translation table.
            unsafe { emsg_ptr(msg) };
            return false;
        }
        Some(di) => tv_copy(&di.di_tv, &mut tv),
    }

    // The borrow of the item is over, so the dictionary can be named again:
    // the partial the value becomes takes a *reference* to it, and that is
    // the only thing `set_selfdict` does to it.
    // SAFETY: the caller's dictionary and a value this frame owns.
    unsafe { set_selfdict(&mut tv, &raw mut *d) };
    // SAFETY: a callback slot the caller may not read until this answers.
    let res = unsafe { callback_from_typval(result, &tv) };
    tv_clear(&mut tv);
    res
}

/// Whether storing `tv` under `name` in `d` would shadow a builtin function.
///
/// Only the global scope and a function's local scope are guarded, and both
/// are read through their globals, so this is the editor's own thread's to
/// call.
pub fn dict_wrong_func_name(d: &Dict, tv: &TypVal, name: &CStr) -> bool {
    let at = &raw const *d;
    (at == get_globvar_dict().cast_const() || dv_hashtab(at.cast_mut()) == get_funccal_local_ht())
        && tv.is_func()
        // SAFETY: `name` is NUL-terminated, which is what a `&CStr` is.
        && unsafe { var_wrong_func_name(name.as_ptr(), true) }
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

    for di in d.items() {
        let mut tv_item = TV_INITIAL_VALUE;

        match what {
            kDict2ListKeys => {
                // SAFETY: the item's own NUL-terminated key.
                tv_item.write_string(unsafe { xstrdup(di.key().as_ptr()) });
            }
            kDict2ListValues => {
                tv_copy(&di.di_tv, &mut tv_item);
            }
            kDict2ListItems => {
                // items()
                let sub_l = tv_list_alloc(2);
                let at = sub_l.as_ptr();
                tv_item.write_list(Some(sub_l));
                // SAFETY: the pair just allocated, and the item's own key.
                unsafe { (*at).push_string(di.key().as_ptr(), -1) };
                // SAFETY: as above.
                unsafe { (*at).push_copy(&di.di_tv) };
            }
            _ => {}
        }

        // SAFETY: the list this call put in the return slot.
        unsafe { (*result.list_or_null()).push(tv_item) };
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
    let key = numbuf.string_ptr(&args[1]);
    // SAFETY: a non-null answer is a NUL-terminated string.
    let found = dict_has_key(args[0].dict_ref(), unsafe { cstr::bytes_at(key) });
    result.write_number(VarNumber::from(found));
}

impl NumBuf {
    /// `d[key]` as a string, NULL for a missing key. The borrowing half of
    /// the C's `tv_dict_get_string`; [`dict_get_string_alloc`] is the other
    /// one.
    pub fn dict_string(&mut self, d: Option<&Dict>, key: &[u8]) -> *const ::core::ffi::c_char {
        dict_get_string_buf(d, key, self)
    }
}
