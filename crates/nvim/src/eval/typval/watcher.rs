//! `dict_T` watchers and the `Callback` values they hold.
//!
//! [`tv_dict_watcher_add`] threads a `DictWatcher` onto `dv_watchers` and
//! [`tv_dict_watcher_notify`] fires every watcher whose pattern matches a
//! key that just changed, building the `{old, new}` dictionary each callback
//! is handed.  The `callback_*` half is `Callback`'s own lifetime — funcref,
//! partial and LuaRef each reference-counted differently.
//!
//! The three walks over `dv_watchers` are written out rather than folded into
//! a shared iterator: each is upstream's `QUEUE_FOREACH`, which caches the
//! next node *before* running the body precisely so the body may unlink the
//! current one, and a callback fired from the middle of one can re-enter and
//! edit the queue.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::types::NUL;

/// Free `watcher` and the callback and pattern it owns.
pub(crate) unsafe fn tv_dict_watcher_free(watcher: *mut DictWatcher) {
    unsafe { callback_free(&raw mut (*watcher).callback) };
    unsafe { xfree((*watcher).key_pattern.cast()) };
    unsafe { xfree(watcher.cast()) };
}

/// Register `callback` to fire when a key of `dict` matching `key_pattern`
/// changes.  A trailing `*` in the pattern matches a prefix.
pub unsafe fn tv_dict_watcher_add(
    dict: *mut dict_T,
    key_pattern: *const ::core::ffi::c_char,
    key_pattern_len: size_t,
    callback: Callback,
) {
    if dict.is_null() {
        return;
    }
    let watcher = unsafe { xmalloc(::core::mem::size_of::<DictWatcher>()) } as *mut DictWatcher;
    unsafe { (*watcher).key_pattern = xmemdupz(key_pattern.cast(), key_pattern_len).cast() };
    // SAFETY: freshly allocated just above.
    let mut w = unsafe { Dw::new(watcher) };
    w.key_pattern_len = key_pattern_len;
    w.callback = callback;
    w.busy = false;
    w.needs_free = false;
    unsafe { queue_insert_tail(&raw mut (*dict).watchers, &raw mut (*watcher).node) };
}

/// Whether `cb1` and `cb2` name the same function.
pub unsafe fn tv_callback_equal(cb1: *const Callback, cb2: *const Callback) -> bool {
    // SAFETY: the caller's callbacks, live for the comparison.
    match unsafe { (&*cb1, &*cb2) } {
        (Callback::None, Callback::None) => true,
        // SAFETY: a funcref names its own NUL-terminated bytes.
        (Callback::Funcref(a), Callback::Funcref(b)) => unsafe { cstr::eq(*a, *b) },
        (Callback::Partial(a), Callback::Partial(b)) => a == b,
        (Callback::Lua(a), Callback::Lua(b)) => a == b,
        _ => false,
    }
}

/// Drop whatever `callback` holds and leave it `kCallbackNone`.
pub unsafe fn callback_free(callback: *mut Callback) {
    // SAFETY: the caller's promise: a live callback, whose payload it owns.
    match unsafe { &*callback } {
        Callback::Funcref(name) => {
            // SAFETY: a funcref owns its NUL-terminated name.
            unsafe { func_unref(*name) };
            unsafe { xfree(name.cast()) };
        }
        Callback::Partial(partial) => unsafe { partial_unref(*partial) },
        // NLUA_CLEAR_REF
        Callback::Lua(reference) => {
            if *reference != LUA_NOREF {
                // SAFETY: a registry index, not a pointer.
                unsafe { api_free_luaref(*reference) };
            }
        }
        Callback::None => {}
    }
    // SAFETY: as above.
    unsafe { *callback = Callback::None };
}

/// Store `cb` in `tv` as a Vimscript value, taking a reference to it.
///
/// A Lua callback has no Vimscript form and comes out as `v:null`.
pub unsafe fn callback_put(cb: *mut Callback, tv: *mut typval_T) {
    // SAFETY: the caller's promise: a live typval.
    let mut value = unsafe { Tv::new(tv) };
    // SAFETY: as above, and a live callback whose payload it owns.
    match unsafe { &*cb } {
        Callback::Partial(partial) => {
            value.v_type = VAR_PARTIAL;
            value.vval.v_partial = *partial;
            // SAFETY: the partial the callback holds; the reference the
            // typval is about to hold is what this counts.
            unsafe { (**partial).pt_refcount.retain() };
        }
        Callback::Funcref(name) => {
            value.v_type = VAR_FUNC;
            // SAFETY: a funcref names its own NUL-terminated bytes.
            unsafe {
                value.vval.v_string = xstrdup(*name);
                func_ref(*name);
            }
        }
        // A Lua callback and no callback at all have no Vimscript form.
        Callback::Lua(_) | Callback::None => {
            value.v_type = VAR_SPECIAL;
            value.vval.v_special = kSpecialVarNull;
        }
    }
}

/// Copy `src` into `dest`, taking a reference to whatever it holds.
pub unsafe fn callback_copy(dest: *mut Callback, src: *mut Callback) {
    // SAFETY: the caller's callbacks; `dest` need not hold a value yet, and
    // a `Callback` has no destructor to run over what was there.
    let copy = match unsafe { &*src } {
        Callback::Partial(partial) => {
            // SAFETY: the partial the source holds; the destination becomes
            // a second owner of it.
            unsafe { (**partial).pt_refcount.retain() };
            Callback::Partial(*partial)
        }
        Callback::Funcref(name) => {
            // SAFETY: a funcref names its own NUL-terminated bytes.
            unsafe {
                func_ref(*name);
                Callback::Funcref(xstrdup(*name))
            }
        }
        // SAFETY: a registry index, not a pointer.
        Callback::Lua(reference) => Callback::Lua(unsafe { api_new_luaref(*reference) }),
        Callback::None => Callback::None,
    };
    // SAFETY: as above.
    unsafe { *dest = copy };
}

/// A freshly allocated description of `cb`, as `string()` prints it.
pub unsafe fn callback_to_string(cb: *mut Callback, arena: *mut Arena) -> *mut ::core::ffi::c_char {
    // SAFETY: the caller's promise: a live callback.
    // SAFETY: the caller's promise: a live callback.
    let callback = unsafe { &*cb };
    if let Callback::Lua(reference) = callback {
        // SAFETY: a registry index, and the caller's arena.
        return unsafe { nlua_funcref_str(*reference, arena) };
    }

    let msglen: size_t = 100;
    let msg = unsafe { xmallocz(msglen) } as *mut ::core::ffi::c_char;
    // SAFETY: `msg` is `msglen` writable bytes, and each name below is a
    // NUL-terminated string the callback owns.
    match callback {
        Callback::Funcref(name) => unsafe {
            snprintf(msg, msglen, c"<vim function: %s>".as_ptr(), *name);
        },
        Callback::Partial(partial) => unsafe {
            let name = (**partial).pt_name;
            snprintf(msg, msglen, c"<vim partial: %s>".as_ptr(), name);
        },
        _ => unsafe { *msg = NUL as ::core::ffi::c_char },
    }
    msg
}

/// Unregister the watcher on `dict` with this exact pattern and callback.
///
/// A watcher removed while any watcher on the queue is mid-callback is only
/// marked `needs_free`; [`tv_dict_watcher_notify`] unlinks it when the walk
/// that is running finishes.
///
/// `callback` is only compared against the registered ones — it stays the
/// caller's to free, which is why it arrives borrowed. Contrast
/// [`tv_dict_watcher_add`], which takes its callback over.
pub unsafe fn tv_dict_watcher_remove(
    dict: *mut dict_T,
    key_pattern: *const ::core::ffi::c_char,
    key_pattern_len: size_t,
    callback: &Callback,
) -> bool {
    if dict.is_null() {
        return false;
    }

    let mut watcher = ::core::ptr::null_mut::<DictWatcher>();
    let mut matched = false;
    let mut queue_is_busy = false;
    // QUEUE_FOREACH; `w` stays on the matching node when the walk breaks.
    // SAFETY: the caller's promise: a live dictionary.
    let d = unsafe { Dt::new(dict) };
    let mut w = d.watchers.next;
    while w != dv_watchers(dict) {
        let next = unsafe { (*w).next };
        watcher = unsafe { tv_dict_watcher_node_data(w) };
        // SAFETY: an entry of the dictionary's watcher queue.
        let wd = unsafe { Dw::new(watcher) };
        if wd.busy {
            queue_is_busy = true;
        }
        if unsafe { tv_callback_equal(&raw const (*watcher).callback, callback) }
            && wd.key_pattern_len == key_pattern_len
            && {
                let n = key_pattern_len;
                unsafe { cstr::slice_at(wd.key_pattern, n) == cstr::slice_at(key_pattern, n) }
            }
        {
            matched = true;
            break;
        }
        w = next;
    }

    if !matched {
        return false;
    }

    if queue_is_busy {
        unsafe { (*watcher).needs_free = true };
    } else {
        unsafe { queue_remove(w) };
        unsafe { tv_dict_watcher_free(watcher) };
    }
    true
}

/// Whether `watcher`'s pattern matches `key`.  A trailing `*` makes it a
/// prefix match.
pub(crate) unsafe fn tv_dict_watcher_matches(
    watcher: *mut DictWatcher,
    key: *const ::core::ffi::c_char,
) -> bool {
    let len = unsafe { (*watcher).key_pattern_len };
    if len != 0
        && unsafe { *(*watcher).key_pattern.add(len - 1) } as ::core::ffi::c_int == '*' as i32
    {
        return unsafe { cstr::prefix_eq(key, (*watcher).key_pattern, len - 1) };
    }
    unsafe { cstr::eq(key, (*watcher).key_pattern) }
}

/// Fire every watcher of `dict` that matches `key`, handing each
/// `(dict, key, {old, new})`.
///
/// A callback may add or remove watchers, and may re-enter this function; the
/// `busy` flag is what stops a watcher firing inside its own callback, and the
/// second walk is the deferred deletion the first one could not do.
pub unsafe fn tv_dict_watcher_notify(
    dict: *mut dict_T,
    key: *const ::core::ffi::c_char,
    newtv: *mut typval_T,
    oldtv: *mut typval_T,
) {
    let mut argv = [TV_INITIAL_VALUE; 3];
    argv[0].v_type = VAR_DICT;
    argv[0].v_lock = VarLock::Unlocked;
    argv[0].vval.v_dict = dict;
    argv[1].v_type = VAR_STRING;
    argv[1].v_lock = VarLock::Unlocked;
    argv[1].vval.v_string = unsafe { xstrdup(key) };
    argv[2].v_type = VAR_DICT;
    argv[2].v_lock = VarLock::Unlocked;
    argv[2].vval.v_dict = unsafe { tv_dict_alloc() };
    unsafe { (*argv[2].dict_or_null()).dv_refcount.retain() };

    // `tv_dict_item_alloc_len` copies exactly the length given and appends
    // the NUL itself, so a Rust `&str` is upstream's `S_LEN(…)`.
    let event = argv[2].dict_or_null();
    let add = |name: &str, from: *mut typval_T| {
        let v = unsafe { tv_dict_item_alloc_len(name.as_ptr().cast(), name.len()) };
        unsafe { tv_copy(from, &raw mut (*v).di_tv) };
        let _ = unsafe { tv_dict_add(event, v) };
    };
    if !newtv.is_null() {
        add("new", newtv);
    }
    if !oldtv.is_null() && unsafe { (*oldtv).v_type } != VAR_UNKNOWN {
        add("old", oldtv);
    }

    let mut any_needs_free = false;
    // Hold the dictionary across the callbacks: one of them may drop the
    // last other reference to it.
    // SAFETY: the caller's promise: a live dictionary.
    let mut d = unsafe { Dt::new(dict) };
    d.dv_refcount.retain();
    // The queue head is a *field of* the dictionary, so its address is taken
    // from the raw pointer rather than through the handle: a `DerefMut`
    // reborrow ends with the expression that asked for it, and a raw pointer
    // outliving one is exactly what Stacked and Tree Borrows reject.
    let head = dv_watchers(dict);
    // QUEUE_FOREACH: the next node is read before the body, so a callback
    // that unlinks the current watcher does not strand the walk.
    let mut w = d.watchers.next;
    while w != head {
        let next = unsafe { (*w).next };
        let watcher = unsafe { tv_dict_watcher_node_data(w) };
        if !unsafe { (*watcher).busy } && unsafe { tv_dict_watcher_matches(watcher, key) } {
            let mut rettv = TV_INITIAL_VALUE;
            // SAFETY: an entry of the dictionary's watcher queue.
            let mut wd = unsafe { Dw::new(watcher) };
            wd.busy = true;
            let cb = wd.field_ptr(::core::mem::offset_of!(DictWatcher, callback));
            let argp = argv.as_mut_ptr();
            unsafe { callback_call(cb, 3, argp, &raw mut rettv) };
            wd.busy = false;
            unsafe { tv_clear(&raw mut rettv) };
            if wd.needs_free {
                any_needs_free = true;
            }
        }
        w = next;
    }

    if any_needs_free {
        let mut w = d.watchers.next;
        while w != head {
            let next = unsafe { (*w).next };
            let watcher = unsafe { tv_dict_watcher_node_data(w) };
            // SAFETY: an entry of the dictionary's watcher queue.
            if unsafe { Dw::new(watcher) }.needs_free {
                unsafe { queue_remove(w) };
                unsafe { tv_dict_watcher_free(watcher) };
            }
            w = next;
        }
    }
    unsafe { tv_dict_unref(dict) };

    // From 1: `argv[0]` is the caller's dictionary, which it still owns.
    for tv in &mut argv[1..] {
        unsafe { tv_clear(tv) };
    }
}
