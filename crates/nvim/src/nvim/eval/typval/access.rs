//! The one-line accessors every other module reaches a `typval_T` through.
//!
//! Upstream declares these `static inline` in `typval.h`, so they are the part
//! of this file that is compiled into its callers rather than called; they keep
//! the `#[inline]` the transpile gave them for the same reason.  The four
//! `QUEUE_*` helpers are the intrusive-list macros `dv_watchers` is threaded
//! on.
//!
//! Every accessor takes the raw pointer its callers already hold — 500-odd call
//! sites across the tree pass `*mut list_T`/`*mut dict_T` around, and the
//! `typval_T` family's layout is frozen by the LuaJIT unit specs.  What they
//! buy the rest of the family is that *nothing else* has to spell a field walk:
//! the children below reach a list through `tv_list_first`/`tv_list_last`/
//! `tv_list_len`, never through `(*l).lv_first`.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

/// True when an intrusive queue head has no entries.
#[inline(always)]
pub unsafe fn QUEUE_EMPTY(q: *const QUEUE) -> bool {
    unsafe { q == (*q).next }
}

/// Make `q` an empty queue head, pointing at itself both ways.
#[inline(always)]
pub unsafe fn QUEUE_INIT(q: *mut QUEUE) {
    unsafe {
        (*q).next = q;
        (*q).prev = q;
    }
}

/// Splice `q` in as the last entry of the queue headed by `h`.
#[inline(always)]
pub(crate) unsafe extern "C" fn QUEUE_INSERT_TAIL(h: *mut QUEUE, q: *mut QUEUE) {
    unsafe {
        (*q).next = h;
        (*q).prev = (*h).prev;
        (*(*q).prev).next = q;
        (*h).prev = q;
    }
}

/// Unlink `q` from whatever queue it is on.
#[inline(always)]
pub(crate) unsafe extern "C" fn QUEUE_REMOVE(q: *mut QUEUE) {
    unsafe {
        (*(*q).prev).next = (*q).next;
        (*(*q).next).prev = (*q).prev;
    }
}

/// Increase the reference count of `l`; does nothing for a NULL list.
#[inline(always)]
pub unsafe fn tv_list_ref(l: *mut list_T) {
    if let Some(l) = unsafe { l.as_mut() } {
        l.lv_refcount += 1;
    }
}

/// Store `l` in `tv` as the return value, taking a reference to it.
#[inline(always)]
pub unsafe fn tv_list_set_ret(tv: *mut typval_T, l: *mut list_T) {
    unsafe {
        (*tv).v_type = VAR_LIST;
        (*tv).vval.v_list = l;
        tv_list_ref(l);
    }
}

/// Lock status of `l`; a NULL list reads as `VAR_FIXED`.
#[inline]
pub unsafe fn tv_list_locked(l: *const list_T) -> VarLockStatus {
    unsafe { l.as_ref() }.map_or(VAR_FIXED, |l| l.lv_lock)
}

/// Set the lock status of `l`.  A NULL list may only be "set" to `VAR_FIXED`.
#[inline]
pub unsafe fn tv_list_set_lock(l: *mut list_T, lock: VarLockStatus) {
    match unsafe { l.as_mut() } {
        Some(l) => l.lv_lock = lock,
        None => debug_assert!(lock == VAR_FIXED),
    }
}

/// Set the copyID of `l`.  Does not expect a NULL list, be careful.
#[inline]
pub unsafe fn tv_list_set_copyid(l: *mut list_T, copyid: ::core::ffi::c_int) {
    unsafe {
        (*l).lv_copyID = copyid;
    }
}

/// Number of items in `l`; a NULL list is empty.
#[inline]
pub unsafe fn tv_list_len(l: *const list_T) -> ::core::ffi::c_int {
    unsafe { l.as_ref() }.map_or(0, |l| l.lv_len)
}

/// The copyID of `l`.  Does not expect a NULL list, be careful.
#[inline]
pub unsafe fn tv_list_copyid(l: *const list_T) -> ::core::ffi::c_int {
    unsafe { (*l).lv_copyID }
}

/// Normalise a possibly negative list index against `l`'s length.
///
/// Returns an index in `0..tv_list_len(l)`, or -1 when it is out of range.
#[inline]
pub unsafe fn tv_list_uidx(l: *const list_T, n: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        // A negative index counts back from the end.
        let n = if n < 0 { n + tv_list_len(l) } else { n };
        if n < 0 || n >= tv_list_len(l) { -1 } else { n }
    }
}

/// First item of `l`, or NULL when it is empty or NULL.
#[inline]
pub unsafe fn tv_list_first(l: *const list_T) -> *mut listitem_T {
    unsafe { l.as_ref() }.map_or(::core::ptr::null_mut(), |l| l.lv_first)
}

/// Last item of `l`, or NULL when it is empty or NULL.
#[inline]
pub unsafe fn tv_list_last(l: *const list_T) -> *mut listitem_T {
    unsafe { l.as_ref() }.map_or(::core::ptr::null_mut(), |l| l.lv_last)
}

/// Store `d` in `tv` as the return value, taking a reference to it.
#[inline(always)]
pub unsafe fn tv_dict_set_ret(tv: *mut typval_T, d: *mut dict_T) {
    unsafe {
        (*tv).v_type = VAR_DICT;
        (*tv).vval.v_dict = d;
        if let Some(d) = d.as_mut() {
            d.dv_refcount += 1;
        }
    }
}

/// Number of items in `d`; a NULL dictionary is empty.
#[inline]
pub unsafe fn tv_dict_len(d: *const dict_T) -> ::core::ffi::c_long {
    unsafe { d.as_ref() }.map_or(0, |d| d.dv_hashtab.ht_used as ::core::ffi::c_long)
}

/// Whether at least one watcher is registered on `d`.
#[inline]
pub unsafe fn tv_dict_is_watched(d: *const dict_T) -> bool {
    unsafe {
        d.as_ref()
            .is_some_and(|d| !QUEUE_EMPTY(&raw const d.watchers))
    }
}

/// Store `b` in `tv` as the return value, taking a reference to it.
#[inline(always)]
pub unsafe fn tv_blob_set_ret(tv: *mut typval_T, b: *mut blob_T) {
    unsafe {
        (*tv).v_type = VAR_BLOB;
        (*tv).vval.v_blob = b;
        if let Some(b) = b.as_mut() {
            b.bv_refcount += 1;
        }
    }
}

/// Length of `b`'s data in bytes; a NULL blob is empty.
#[inline]
pub unsafe fn tv_blob_len(b: *const blob_T) -> ::core::ffi::c_int {
    unsafe { b.as_ref() }.map_or(0, |b| b.bv_ga.ga_len)
}

/// The byte at `idx` in `b`.  `b` must be non-NULL and `idx` in range.
#[inline(always)]
pub unsafe fn tv_blob_get(b: *const blob_T, idx: ::core::ffi::c_int) -> uint8_t {
    unsafe { *(*b).bv_ga.ga_data.cast::<uint8_t>().offset(idx as isize) }
}

/// Store `c` at `idx` in `blob`.  `blob` must be non-NULL and `idx` in range.
#[inline(always)]
pub unsafe fn tv_blob_set(blob: *mut blob_T, idx: ::core::ffi::c_int, c: uint8_t) {
    unsafe {
        *(*blob).bv_ga.ga_data.cast::<uint8_t>().offset(idx as isize) = c;
    }
}

/// The `DictWatcher` a queue node is embedded in (upstream's `QUEUE_DATA`).
///
/// Upstream spells this out as a function rather than the macro purely so it
/// can carry `FUNC_ATTR_NO_SANITIZE_ADDRESS`: ASan does not follow the pointer
/// arithmetic back out of the struct.
#[inline(always)]
pub unsafe fn tv_dict_watcher_node_data(q: *mut QUEUE) -> *mut DictWatcher {
    unsafe {
        q.cast::<::core::ffi::c_char>()
            .sub(::core::mem::offset_of!(DictWatcher, node))
            .cast::<DictWatcher>()
    }
}

/// Whether `tv` holds a function: either `VAR_FUNC` or `VAR_PARTIAL`.
#[inline(always)]
pub fn tv_is_func(tv: typval_T) -> bool {
    tv.v_type == VAR_FUNC || tv.v_type == VAR_PARTIAL
}
