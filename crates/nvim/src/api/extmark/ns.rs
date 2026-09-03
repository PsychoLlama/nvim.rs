//! Namespaces: the id space every extmark and decoration lives in.
//!
//! `nvim_create_namespace` interns a name in the `namespace_ids` table and
//! hands back a monotonic id; `nvim_get_namespaces` renders the table back,
//! and `ns_initialized`/`describe_ns` are the validity and lookup helpers the
//! rest of the family funnels through.  `nvim__ns_set`/`nvim__ns_get` carry
//! the per-window visibility a namespace can be given.
//!
//! The table is a [`SlotTable`](crate::registry::SlotTable) rather than a
//! `HashMap`: `nvim_get_namespaces` answers a dict in the table's own order
//! and `describe_ns` answers the *first* name an id was created under, both
//! of which the api observes (F-P21-9), and khash -- which this was -- is
//! insertion-ordered. The two `set_*` shims that remain are the ones for
//! `w_ns_set`, a khash still embedded in `win_T`.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{Reported, array_add, dict_put_str, has_key, set_key};
use crate::api::private::validate::err_bad_number;
use crate::registry::interned_key;
use crate::winlayer::{Win, tab_windows};
use core::ptr;

#[inline]
unsafe fn set_del_uint32_t(mut set: *mut Set_uint32_t, mut key: uint32_t) -> uint32_t {
    unsafe { mh_delete_uint32_t(set, &raw mut key) };
    key
}

#[inline]
unsafe fn set_put_uint32_t(
    mut set: *mut Set_uint32_t,
    mut key: uint32_t,
    mut key_alloc: *mut *mut uint32_t,
) -> bool {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = unsafe { mh_put_uint32_t(set, key, &raw mut status) };
    if !key_alloc.is_null() {
        unsafe { *key_alloc = (*set).keys.offset(k as isize) };
    }
    status as ::core::ffi::c_uint != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint
}

/// The id `name` is registered under, if any.
pub(crate) fn namespace_id_for(name: &[u8]) -> Option<handle_T> {
    namespace_ids.with(|ids| ids.get(&interned_key(name)))
}

/// Whether namespace `ns_id` is window-local rather than visible everywhere.
pub(crate) fn ns_is_local(ns_id: uint32_t) -> bool {
    namespace_localscope.with(|scoped| scoped.contains(&ns_id))
}

/// Whether `win`'s buffer holds any extmark in namespace `ns_id`.
///
/// `b_extmark_ns` is declared as the untyped map header, so upstream casts it
/// to the `uint32_t -> uint32_t` instantiation before asking; the four call
/// sites below did that inline.
fn buffer_uses_ns(win: Win, ns_id: uint32_t) -> bool {
    // SAFETY: a live window has a live buffer, and the map is that buffer's
    // own field.
    unsafe {
        let map = &raw mut (*(*win.raw()).w_buffer).b_extmark_ns as *mut Map_uint32_t_uint32_t;
        set_has_uint32_t(&raw mut (*map).set, ns_id)
    }
}

/// Whether namespace `ns_id` is one of the ones `win` shows.
fn window_shows_ns(win: Win, ns_id: uint32_t) -> bool {
    // SAFETY: a live window, and the set is its own field.
    unsafe { set_has_uint32_t(&raw mut (*win.raw()).w_ns_set, ns_id) }
}

/// Show namespace `ns_id` in `win`.
fn show_ns_in_window(win: Win, ns_id: uint32_t) {
    // SAFETY: `w_ns_set` is the live window's own field.
    let set = unsafe { &raw mut (*win.raw()).w_ns_set };
    // SAFETY: `set` is that live set.
    unsafe { set_put_uint32_t(set, ns_id, ptr::null_mut()) };
}

/// Stop showing namespace `ns_id` in `win`.
fn hide_ns_in_window(win: Win, ns_id: uint32_t) {
    // SAFETY: a live window, and the set is its own field.
    unsafe { set_del_uint32_t(&raw mut (*win.raw()).w_ns_set, ns_id) };
}

/// # Safety
/// `name` must be a live api string.
pub unsafe fn nvim_create_namespace(name: String_0) -> Integer {
    // SAFETY: the caller's api string.
    let bytes = unsafe { name.as_bytes() };
    if let Some(id) = namespace_id_for(bytes)
        && id > 0
    {
        return id as Integer;
    }
    let id = next_namespace_id.get();
    next_namespace_id.set(id + 1);
    // The nameless namespaces -- one per anonymous `nvim_create_namespace("")`
    // -- are never interned, so each call answers a fresh id.
    if !bytes.is_empty() {
        namespace_ids.with_mut(|ids| ids.insert(interned_key(bytes), id));
    }
    id as Integer
}

/// # Safety
/// `arena` is null or this call's own arena.
pub unsafe fn nvim_get_namespaces(arena: *mut Arena) -> Dict {
    namespace_ids.with(|ids| {
        let mut retval: Dict = arena_dict(arena, ids.len() as size_t);
        for (name, id) in ids.entries() {
            // SAFETY: `ns_key` terminated the key, and `cstr_as_string`
            // re-measures it -- so a name with an interior NUL is answered
            // truncated, as it was when the key was a `String_0`.
            let key = unsafe { cstr_as_string(name.as_ptr().cast::<::core::ffi::c_char>()) };
            let value = Object::integer(*id as Integer);
            // SAFETY: `retval` is this call's own dict.
            unsafe { dict_put_str(&mut retval, key, value) };
        }
        retval
    })
}

/// The name namespace `ns_id` was created under, or `unknown`.
///
/// The answer points into the table's own key, which is a `Box` the table
/// never moves and -- names are interned, never removed -- never frees.
pub fn describe_ns(ns_id: NS, unknown: *const ::core::ffi::c_char) -> *const ::core::ffi::c_char {
    namespace_ids.with(|ids| {
        for (name, id) in ids.entries() {
            // `ns_key`'s terminator is the whole of an empty name's key.
            if *id == ns_id && name.len() > 1 {
                return name.as_ptr().cast::<::core::ffi::c_char>();
            }
        }
        unknown
    })
}

pub fn ns_initialized(mut ns: uint32_t) -> bool {
    if ns < 1 as uint32_t {
        return false;
    }
    ns < next_namespace_id.get() as uint32_t
}

pub unsafe fn nvim__ns_set(ns_id: Integer, opts: *mut KeyDict_ns_opts) -> Result<(), Error> {
    let mut error = Error::none();
    if !ns_initialized(ns_id as uint32_t) {
        error = err_bad_number(c"ns_id", ns_id);
        return ().reported(error);
    }
    let mut set_scoped: bool = true;
    if has_key(
        unsafe { (*opts).is_set__ns_opts_ },
        KEYSET_OPTIDX_ns_opts__wins,
    ) {
        if unsafe { (*opts).wins.size } == 0 as size_t {
            set_scoped = false;
        }
        let mut windows: IdSet<*mut win_T> = id_set();
        let mut i: size_t = 0 as size_t;
        while i < unsafe { (*opts).wins.size } {
            // A `wins` element that is neither a window handle nor a plain
            // integer takes -1, which no window carries, so the lookup below
            // refuses it -- the transpile read its bytes as an integer.
            let item = unsafe { *(*opts).wins.items.add(i) };
            let mut win: Integer = item.as_handle().or_else(|| item.as_integer()).unwrap_or(-1);
            let mut wp: *mut win_T = unsafe { find_window_by_handle(win as Window, &mut error) };
            if wp.is_null() {
                return ().reported(error);
            }
            windows.insert(wp);
            i = i.wrapping_add(1);
        }
        for win in tab_windows() {
            let wp_0 = win.raw();
            if windows.contains(&wp_0) && !window_shows_ns(win, ns_id as uint32_t) {
                show_ns_in_window(win, ns_id as uint32_t);
                if buffer_uses_ns(win, ns_id as uint32_t) {
                    changed_window_setting(win);
                }
            }
            if window_shows_ns(win, ns_id as uint32_t) && !windows.contains(&wp_0) {
                hide_ns_in_window(win, ns_id as uint32_t);
                if buffer_uses_ns(win, ns_id as uint32_t) {
                    changed_window_setting(win);
                }
            }
        }
    }
    if set_scoped && !ns_is_local(ns_id as uint32_t) {
        namespace_localscope.with_mut(|scoped| scoped.insert(ns_id as uint32_t));
        for win in tab_windows() {
            if buffer_uses_ns(win, ns_id as uint32_t) {
                changed_window_setting(win);
            }
        }
    } else if !set_scoped && ns_is_local(ns_id as uint32_t) {
        namespace_localscope.with_mut(|scoped| scoped.remove(&(ns_id as uint32_t)));
        for win in tab_windows() {
            if buffer_uses_ns(win, ns_id as uint32_t) {
                changed_window_setting(win);
            }
        }
    }
    ().reported(error)
}

pub unsafe fn nvim__ns_get(ns_id: Integer, arena: *mut Arena) -> Result<KeyDict_ns_opts, Error> {
    let mut error = Error::none();
    let mut opts: KeyDict_ns_opts = KEYDICT_INIT;
    let mut windows: Array = ARRAY_DICT_INIT;
    opts.is_set__ns_opts_ = set_key(opts.is_set__ns_opts_, KEYSET_OPTIDX_ns_opts__wins);
    opts.wins = windows;
    if !ns_initialized(ns_id as uint32_t) {
        error = err_bad_number(c"ns_id", ns_id);
        return opts.reported(error);
    }
    if !ns_is_local(ns_id as uint32_t) {
        return opts.reported(error);
    }
    let mut count: size_t = 0 as size_t;
    for win in tab_windows() {
        if window_shows_ns(win, ns_id as uint32_t) {
            count = count.wrapping_add(1);
        }
    }
    windows = arena_array(arena, count);
    for win in tab_windows() {
        if window_shows_ns(win, ns_id as uint32_t) {
            if windows.size == windows.capacity {
                windows.capacity = if windows.capacity != 0 {
                    windows.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                let old = windows.items as *mut ::core::ffi::c_void;
                let bytes = ::core::mem::size_of::<Object>().wrapping_mul(windows.capacity);
                // SAFETY: `old` is null or this array's own allocation.
                windows.items = unsafe { xrealloc(old, bytes) } as *mut Object;
            };
            let handle = Object::integer(win.handle() as Integer);
            // SAFETY: `windows` is this call's own array.
            unsafe { array_add(&mut windows, handle) };
        }
    }
    opts.is_set__ns_opts_ = set_key(opts.is_set__ns_opts_, KEYSET_OPTIDX_ns_opts__wins);
    opts.wins = windows;
    opts.reported(error)
}
