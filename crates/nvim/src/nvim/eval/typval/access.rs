//! The one-line accessors every other module reaches a `typval_T` through.
//!
//! Upstream declares these `static inline` in `typval.h`, so they are the
//! part of this file that is compiled into its callers rather than called;
//! they keep their `#[inline]` here for the same reason.  The four `QUEUE_*`
//! helpers are the intrusive-list macros `dv_watchers` is threaded on.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

#[inline(always)]
pub unsafe fn QUEUE_EMPTY(q: *const QUEUE) -> ::core::ffi::c_int {
    unsafe {
        return (q == (*q).next as *const QUEUE) as ::core::ffi::c_int;
    }
}

#[inline(always)]
pub unsafe fn QUEUE_INIT(q: *mut QUEUE) {
    unsafe {
        (*q).next = q as *mut queue;
        (*q).prev = q as *mut queue;
    }
}

#[inline(always)]
pub(crate) unsafe extern "C" fn QUEUE_INSERT_TAIL(h: *mut QUEUE, q: *mut QUEUE) {
    unsafe {
        (*q).next = h as *mut queue;
        (*q).prev = (*h).prev;
        (*(*q).prev).next = q as *mut queue;
        (*h).prev = q as *mut queue;
    }
}

#[inline(always)]
pub(crate) unsafe extern "C" fn QUEUE_REMOVE(q: *mut QUEUE) {
    unsafe {
        (*(*q).prev).next = (*q).next;
        (*(*q).next).prev = (*q).prev;
    }
}

#[inline(always)]
pub unsafe fn tv_list_ref(l: *mut list_T) {
    unsafe {
        if l.is_null() {
            return;
        }
        (*l).lv_refcount += 1;
    }
}

#[inline(always)]
pub unsafe fn tv_list_set_ret(tv: *mut typval_T, l: *mut list_T) {
    unsafe {
        (*tv).v_type = VAR_LIST;
        (*tv).vval.v_list = l;
        tv_list_ref(l);
    }
}

#[inline]
pub unsafe fn tv_list_locked(l: *const list_T) -> VarLockStatus {
    unsafe {
        if l.is_null() {
            return VAR_FIXED;
        }
        return (*l).lv_lock;
    }
}

#[inline]
pub unsafe fn tv_list_set_lock(l: *mut list_T, lock: VarLockStatus) {
    unsafe {
        if l.is_null() {
            '_c2rust_label: {
                if lock as ::core::ffi::c_uint
                    == VAR_FIXED as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                } else {
                    __assert_fail(
                        b"lock == VAR_FIXED\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/eval/typval.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        76 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            return;
        }
        (*l).lv_lock = lock;
    }
}

#[inline]
pub unsafe fn tv_list_set_copyid(l: *mut list_T, copyid: ::core::ffi::c_int) {
    unsafe {
        (*l).lv_copyID = copyid;
    }
}

#[inline]
pub unsafe fn tv_list_len(l: *const list_T) -> ::core::ffi::c_int {
    unsafe {
        if l.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        return (*l).lv_len;
    }
}

#[inline]
pub unsafe fn tv_list_copyid(l: *const list_T) -> ::core::ffi::c_int {
    unsafe {
        return (*l).lv_copyID;
    }
}

#[inline]
pub unsafe fn tv_list_uidx(l: *const list_T, mut n: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        if n < 0 as ::core::ffi::c_int {
            n += tv_list_len(l);
        }
        if n < 0 as ::core::ffi::c_int || n >= tv_list_len(l) {
            return -1 as ::core::ffi::c_int;
        }
        return n;
    }
}

#[inline]
pub unsafe fn tv_list_first(l: *const list_T) -> *mut listitem_T {
    unsafe {
        if l.is_null() {
            return ::core::ptr::null_mut::<listitem_T>();
        }
        return (*l).lv_first;
    }
}

#[inline]
pub unsafe fn tv_list_last(l: *const list_T) -> *mut listitem_T {
    unsafe {
        if l.is_null() {
            return ::core::ptr::null_mut::<listitem_T>();
        }
        return (*l).lv_last;
    }
}

#[inline(always)]
pub unsafe fn tv_dict_set_ret(tv: *mut typval_T, d: *mut dict_T) {
    unsafe {
        (*tv).v_type = VAR_DICT;
        (*tv).vval.v_dict = d;
        if !d.is_null() {
            (*d).dv_refcount += 1;
        }
    }
}

#[inline]
pub unsafe fn tv_dict_len(d: *const dict_T) -> ::core::ffi::c_long {
    unsafe {
        if d.is_null() {
            return 0 as ::core::ffi::c_long;
        }
        return (*d).dv_hashtab.ht_used as ::core::ffi::c_long;
    }
}

#[inline]
pub unsafe fn tv_dict_is_watched(d: *const dict_T) -> bool {
    unsafe {
        return !d.is_null() && QUEUE_EMPTY(&raw const (*d).watchers) == 0;
    }
}

#[inline(always)]
pub unsafe fn tv_blob_set_ret(tv: *mut typval_T, b: *mut blob_T) {
    unsafe {
        (*tv).v_type = VAR_BLOB;
        (*tv).vval.v_blob = b;
        if !b.is_null() {
            (*b).bv_refcount += 1;
        }
    }
}

#[inline]
pub unsafe fn tv_blob_len(b: *const blob_T) -> ::core::ffi::c_int {
    unsafe {
        if b.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        return (*b).bv_ga.ga_len;
    }
}

#[inline(always)]
pub unsafe fn tv_blob_get(b: *const blob_T, mut idx: ::core::ffi::c_int) -> uint8_t {
    unsafe {
        return *((*b).bv_ga.ga_data as *mut uint8_t).offset(idx as isize);
    }
}

#[inline(always)]
pub unsafe fn tv_blob_set(blob: *mut blob_T, mut idx: ::core::ffi::c_int, mut c: uint8_t) {
    unsafe {
        *((*blob).bv_ga.ga_data as *mut uint8_t).offset(idx as isize) = c;
    }
}

#[inline(always)]
pub unsafe fn tv_dict_watcher_node_data(mut q: *mut QUEUE) -> *mut DictWatcher {
    unsafe {
        return (q as *mut ::core::ffi::c_char).offset(-(32 as ::core::ffi::c_ulong as isize))
            as *mut DictWatcher;
    }
}

#[inline(always)]
pub unsafe fn tv_is_func(tv: typval_T) -> bool {
    return tv.v_type as ::core::ffi::c_uint
        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
        || tv.v_type as ::core::ffi::c_uint
            == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint;
}
