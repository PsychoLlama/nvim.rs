//! `Dict` watchers and the `Callback` values they hold.
//!
//! [`Dict::watcher_add`] threads a `DictWatcher` onto `dv_watchers` and
//! [`dict_watcher_notify`] fires every watcher whose pattern matches a
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
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use ::core::ffi::CStr;

use super::*;
use crate::cstr;

/// Free `watcher` and the callback and pattern it owns.
///
/// # Safety
///
/// `watcher` must point at a live dictionary watcher, unaliased for the call.
pub(crate) unsafe fn tv_dict_watcher_free(watcher: *mut DictWatcher) {
    unsafe { callback_free(&raw mut (*watcher).callback) };
    unsafe { xfree((*watcher).key_pattern.cast()) };
    unsafe { xfree(watcher.cast()) };
}

impl Dict {
    /// Register `callback` to fire when a key matching `key_pattern`
    /// changes.  A trailing `*` in the pattern matches a prefix.
    ///
    /// The callback is taken over; the pattern is copied.
    pub fn watcher_add(&mut self, key_pattern: &[u8], callback: Callback) {
        // SAFETY: room for one watcher, filled in field by field below.
        let watcher =
            unsafe { xmalloc(::core::mem::size_of::<DictWatcher>()) }.cast::<DictWatcher>();
        // SAFETY: the allocation just made, and `key_pattern` is a slice.
        unsafe {
            (*watcher).key_pattern =
                xmemdupz(key_pattern.as_ptr().cast(), key_pattern.len()).cast();
        };
        // SAFETY: freshly allocated just above.
        let mut w = unsafe { Dw::new(watcher) };
        w.key_pattern_len = key_pattern.len();
        w.callback = callback;
        w.busy = false;
        w.needs_free = false;
        // SAFETY: this dictionary's own queue head, and a node on no queue.
        unsafe { queue_insert_tail(&raw mut self.watchers, &raw mut (*watcher).node) };
    }

    /// Unregister the watcher with this exact pattern and callback.
    ///
    /// A watcher removed while any watcher on the queue is mid-callback is
    /// only marked `needs_free`; [`dict_watcher_notify`] unlinks it when the
    /// walk that is running finishes.
    ///
    /// `callback` is only compared against the registered ones — it stays
    /// the caller's to free, which is why it arrives borrowed. Contrast
    /// [`Dict::watcher_add`], which takes its callback over.
    pub fn watcher_remove(&mut self, key_pattern: &[u8], callback: &Callback) -> bool {
        let head = &raw mut self.watchers;
        let mut watcher = ::core::ptr::null_mut::<DictWatcher>();
        let mut matched = false;
        let mut queue_is_busy = false;
        // QUEUE_FOREACH; `w` stays on the matching node when the walk breaks.
        let mut w = self.watchers.next;
        while w != head {
            // SAFETY: an entry of this dictionary's watcher queue.
            let next = unsafe { (*w).next };
            // SAFETY: as above.
            watcher = unsafe { tv_dict_watcher_node_data(w) };
            // SAFETY: as above.
            let wd = unsafe { Dw::new(watcher) };
            if wd.busy {
                queue_is_busy = true;
            }
            // SAFETY: the watcher's own callback and pattern.
            if unsafe { tv_callback_equal(&raw const (*watcher).callback, callback) }
                && wd.key_pattern_len == key_pattern.len()
                // SAFETY: the watcher's own pattern, that many bytes of it.
                && unsafe { cstr::slice_at(wd.key_pattern, key_pattern.len()) } == key_pattern
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
            // SAFETY: the watcher the walk stopped on.
            unsafe { (*watcher).needs_free = true };
        } else {
            // SAFETY: as above, and the node it is on.
            unsafe { queue_remove(w) };
            // SAFETY: as above, now off the queue.
            unsafe { tv_dict_watcher_free(watcher) };
        }
        true
    }
}

/// Whether `cb1` and `cb2` name the same function.
///
/// # Safety
///
/// `cb1` must point at an initialized callback. `cb2` must point at an
/// initialized callback.
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
///
/// # Safety
///
/// `callback` must point at an initialized callback, unaliased for the call.
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
///
/// # Safety
///
/// `cb` must point at an initialized callback, unaliased for the call. `tv`
/// must point at an initialized typval, unaliased for the call.
pub unsafe fn callback_put(cb: *mut Callback, tv: &mut TypVal) {
    // SAFETY: the caller's promise: a live typval.
    let mut value = unsafe { Tv::new(tv) };
    // SAFETY: as above, and a live callback whose payload it owns.
    match unsafe { &*cb } {
        Callback::Partial(partial) => {
            // SAFETY: the partial the callback holds; the typval takes a
            // reference of its own.
            value.write_partial(unsafe { PartialRef::retained(*partial) });
        }
        Callback::Funcref(name) => {
            // SAFETY: a funcref names its own NUL-terminated bytes.
            unsafe {
                value.write_func_name(xstrdup(*name));
                func_ref(*name);
            }
        }
        // A Lua callback and no callback at all have no Vimscript form.
        Callback::Lua(_) | Callback::None => {
            value.write_special(kSpecialVarNull);
        }
    }
}

/// Copy `src` into `dest`, taking a reference to whatever it holds.
///
/// # Safety
///
/// `dest` must point at an initialized callback, unaliased for the call.
/// `src` must point at an initialized callback, unaliased for the call.
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
        Callback::Lua(reference) => Callback::Lua(api_new_luaref(*reference)),
        Callback::None => Callback::None,
    };
    // SAFETY: as above.
    unsafe { *dest = copy };
}

/// A freshly allocated description of `cb`, as `string()` prints it.
///
/// # Safety
///
/// A `Partial` callback's pointer must name a live partial.
pub unsafe fn callback_to_string(callback: &Callback) -> *mut ::core::ffi::c_char {
    if let Callback::Lua(reference) = callback {
        // SAFETY: a registry index.
        return unsafe { nlua_funcref_str(*reference) };
    }

    let msglen: size_t = 100;
    let msg = unsafe { xmallocz(msglen) }.cast::<::core::ffi::c_char>();
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
        // Anything else leaves the message an empty string.
        _ => unsafe { *msg = 0 },
    }
    msg
}

/// Whether `watcher`'s pattern matches `key`.  A trailing `*` makes it a
/// prefix match.
///
/// # Safety
///
/// `watcher` must point at a live entry of some dictionary's watcher chain
/// and `key` at a NUL-terminated key, both live for the call.
pub(crate) unsafe fn tv_dict_watcher_matches(
    watcher: *mut DictWatcher,
    key: *const ::core::ffi::c_char,
) -> bool {
    let len = unsafe { (*watcher).key_pattern_len };
    if len != 0
        && ::core::ffi::c_int::from(unsafe { *(*watcher).key_pattern.add(len - 1) }) == '*' as i32
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
///
/// **The dictionary stays a pointer.** Every callback fired here runs
/// arbitrary Vimscript or Lua, which reaches this same dictionary through
/// whatever named it in the first place -- a scope, a variable, the
/// argument the callback is handed -- so no borrow of it may be live across
/// the walk. The reference the callbacks run under is the retain below.
///
/// # Safety
///
/// `dict` must point at a live dictionary.
pub unsafe fn dict_watcher_notify(
    dict: *mut Dict,
    key: &CStr,
    newtv: Option<&TypVal>,
    oldtv: Option<&TypVal>,
) {
    // Slot 0 names the caller's dictionary, which the retain below holds
    // across the callbacks; the other two are this frame's own.
    let mut argv = CallFrame::<3>::new();
    // SAFETY: the caller's dictionary, live for the call. The slot *names*
    // it: the reference the callbacks run under is the retain below, and
    // `tv_dict_unref` at the bottom is what gives it back.
    argv.push_naming(TypVal::dict(unsafe { DictRef::owning(dict) }));
    // SAFETY: the key's own NUL-terminated bytes, copied into the frame.
    argv.push_owned(TypVal::String(unsafe { xstrdup(key.as_ptr()) }));
    argv.push_owned(TypVal::dict(Some(tv_dict_alloc())));
    let event = argv.args()[2].dict_or_null();

    // A `&[u8]` is upstream's `S_LEN(…)`: the key is copied at exactly the
    // length given, and the item appends the NUL itself.
    let add = |name: &[u8], from: &TypVal| {
        // SAFETY: the dictionary this call just allocated into the frame.
        let _ = unsafe { (*event).add_tv(name, from) };
    };
    if let Some(newtv) = newtv {
        add(b"new", newtv);
    }
    if let Some(oldtv) = oldtv
        && oldtv.v_type() != VAR_UNKNOWN
    {
        add(b"old", oldtv);
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
        if !unsafe { (*watcher).busy } && unsafe { tv_dict_watcher_matches(watcher, key.as_ptr()) }
        {
            let mut rettv = TV_INITIAL_VALUE;
            // SAFETY: an entry of the dictionary's watcher queue.
            let mut wd = unsafe { Dw::new(watcher) };
            wd.busy = true;
            let cb = wd.field_ptr(::core::mem::offset_of!(DictWatcher, callback));
            unsafe { callback_call(cb, argv.args(), &mut rettv) };
            wd.busy = false;
            tv_clear(&mut rettv);
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
}
