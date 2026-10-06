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

use super::*;
use crate::cstr;
use crate::mbyte::cluster_len;
use crate::memory::ThinCString;
use crate::message::emsg;
use crate::semsg;
use crate::snprintf;
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

/// A fresh item holding a copy of `di`'s key and value.
///
/// # Safety
/// `di` must be a live item. The copy is the caller's, with the same
/// obligation as [`tv_dict_item_alloc_len`]'s result.
pub unsafe fn tv_dict_item_copy(di: *mut DictItem) -> *mut DictItem {
    let new_di = unsafe { tv_dict_item_alloc((*di).di_key.as_ptr()) };
    unsafe { tv_copy(&(*di).di_tv, &mut (*new_di).di_tv) };
    new_di
}

/// Remove `item` from `dict` and free it.
///
/// # Safety
/// `item` must be an item of `dict`, and both must be live. `item` is
/// freed, so the caller must not hold it afterwards.
pub unsafe fn tv_dict_item_remove(dict: *mut Dict, item: *mut DictItem) {
    let hi = unsafe { hash_find(&raw mut (*dict).dv_hashtab, (*item).di_key.as_ptr()) };
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
    tv_list_alloc_ret(result, kListLenMayKnow as ptrdiff_t);
    // A null string behaves like an empty one.
    let text = args[0].string_bytes();

    let mut idx: VarNumber = 0;
    let mut offset = 0;
    while offset < text.len() {
        let len = cluster_len(&text[offset..]);
        let l2 = tv_list_alloc(2);
        let at = l2.as_ptr();
        unsafe { (*(*result).list_or_null()).push_list(Some(l2)) };
        unsafe { (*at).push_number(idx) };
        unsafe { (*at).push(TypVal::string_from(&text[offset..offset + len])) };
        offset += len;
        idx += 1;
    }
}

impl Dict {
    /// Remove the item under `key`, freeing it and its value. False when
    /// there is none.
    pub(crate) fn remove_key(&mut self, key: &[u8]) -> bool {
        let item = self.find_ptr(key);
        if item.is_null() {
            return false;
        }
        // SAFETY: an item of this dictionary, which the exclusive borrow
        // keeps in it until the removal.
        unsafe { tv_dict_item_remove(self, item) };
        true
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
        let key = &var.di_key;
        let str = numbuf.string(&var.di_tv);
        let len = key.len() + str.count_bytes() + c"=".count_bytes() + 1;
        // SAFETY: `i` is below `env_size`, and the format spends two
        // NUL-terminated strings into `len` writable bytes.
        unsafe {
            *env.add(i) = xmalloc(len) as *mut ::core::ffi::c_char;
            snprintf!(
                *env.add(i),
                len,
                c"%s=%s".as_ptr(),
                key.as_ptr(),
                str.as_ptr()
            );
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
    // SAFETY: a callback slot the caller may not read until this answers.
    let res = unsafe { callback_from_typval(result, &tv) };
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
pub fn dict_wrong_func_name(d: &Dict, item: &DictItem) -> bool {
    let at = &raw const *d;
    (at == get_globvar_dict().cast_const() || dv_hashtab(at.cast_mut()) == get_funccal_local_ht())
        && item.di_tv.is_func()
        // SAFETY: a key is NUL-terminated, which is what `as_ptr` answers.
        && unsafe { var_wrong_func_name(item.di_key.as_ptr(), true) }
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
                tv_item.write_string(Some(ThinCString::from_bytes(di.di_key.bytes())));
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
                unsafe { (*at).push_string(di.di_key.as_ptr(), -1) };
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
