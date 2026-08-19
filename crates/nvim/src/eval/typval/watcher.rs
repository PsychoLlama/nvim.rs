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
use crate::types::NUL;

/// Free `watcher` and the callback and pattern it owns.
pub(crate) unsafe fn tv_dict_watcher_free(watcher: *mut DictWatcher) {
    unsafe {
        callback_free(&raw mut (*watcher).callback);
        xfree((*watcher).key_pattern.cast());
        xfree(watcher.cast());
    }
}

/// Register `callback` to fire when a key of `dict` matching `key_pattern`
/// changes.  A trailing `*` in the pattern matches a prefix.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_watcher_add(
    dict: *mut dict_T,
    key_pattern: *const ::core::ffi::c_char,
    key_pattern_len: size_t,
    callback: Callback,
) {
    unsafe {
        if dict.is_null() {
            return;
        }
        let watcher = xmalloc(::core::mem::size_of::<DictWatcher>()) as *mut DictWatcher;
        (*watcher).key_pattern = xmemdupz(key_pattern.cast(), key_pattern_len).cast();
        (*watcher).key_pattern_len = key_pattern_len;
        (*watcher).callback = callback;
        (*watcher).busy = false;
        (*watcher).needs_free = false;
        QUEUE_INSERT_TAIL(&raw mut (*dict).watchers, &raw mut (*watcher).node);
    }
}

/// Whether `cb1` and `cb2` name the same function.
pub unsafe fn tv_callback_equal(cb1: *const Callback, cb2: *const Callback) -> bool {
    unsafe {
        if (*cb1).type_0 != (*cb2).type_0 {
            return false;
        }
        match (*cb1).type_0 {
            kCallbackFuncref => strcmp((*cb1).data.funcref, (*cb2).data.funcref) == 0,
            kCallbackPartial => (*cb1).data.partial == (*cb2).data.partial,
            kCallbackLua => (*cb1).data.luaref == (*cb2).data.luaref,
            kCallbackNone => true,
            _ => abort(),
        }
    }
}

/// Drop whatever `callback` holds and leave it `kCallbackNone`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn callback_free(callback: *mut Callback) {
    unsafe {
        match (*callback).type_0 {
            kCallbackFuncref => {
                func_unref((*callback).data.funcref);
                xfree((*callback).data.funcref.cast());
            }
            kCallbackPartial => partial_unref((*callback).data.partial),
            kCallbackLua => {
                // NLUA_CLEAR_REF
                if (*callback).data.luaref != LUA_NOREF {
                    api_free_luaref((*callback).data.luaref);
                    (*callback).data.luaref = LUA_NOREF as LuaRef;
                }
            }
            _ => {}
        }
        (*callback).type_0 = kCallbackNone;
        (*callback).data.funcref = ::core::ptr::null_mut();
    }
}

/// Store `cb` in `tv` as a Vimscript value, taking a reference to it.
///
/// A Lua callback has no Vimscript form and comes out as `v:null`.
pub unsafe fn callback_put(cb: *mut Callback, tv: *mut typval_T) {
    unsafe {
        match (*cb).type_0 {
            kCallbackPartial => {
                (*tv).v_type = VAR_PARTIAL;
                (*tv).vval.v_partial = (*cb).data.partial;
                (*(*cb).data.partial).pt_refcount += 1;
            }
            kCallbackFuncref => {
                (*tv).v_type = VAR_FUNC;
                (*tv).vval.v_string = xstrdup((*cb).data.funcref);
                func_ref((*cb).data.funcref);
            }
            _ => {
                // kCallbackLua and kCallbackNone: no Vimscript representation.
                (*tv).v_type = VAR_SPECIAL;
                (*tv).vval.v_special = kSpecialVarNull;
            }
        }
    }
}

/// Copy `src` into `dest`, taking a reference to whatever it holds.
pub unsafe fn callback_copy(dest: *mut Callback, src: *mut Callback) {
    unsafe {
        (*dest).type_0 = (*src).type_0;
        match (*src).type_0 {
            kCallbackPartial => {
                (*dest).data.partial = (*src).data.partial;
                (*(*dest).data.partial).pt_refcount += 1;
            }
            kCallbackFuncref => {
                (*dest).data.funcref = xstrdup((*src).data.funcref);
                func_ref((*src).data.funcref);
            }
            kCallbackLua => (*dest).data.luaref = api_new_luaref((*src).data.luaref),
            _ => (*dest).data.funcref = ::core::ptr::null_mut(),
        }
    }
}

/// A freshly allocated description of `cb`, as `string()` prints it.
pub unsafe fn callback_to_string(cb: *mut Callback, arena: *mut Arena) -> *mut ::core::ffi::c_char {
    unsafe {
        if (*cb).type_0 == kCallbackLua {
            return nlua_funcref_str((*cb).data.luaref, arena);
        }

        let msglen: size_t = 100;
        let msg = xmallocz(msglen) as *mut ::core::ffi::c_char;
        match (*cb).type_0 {
            kCallbackFuncref => {
                snprintf(
                    msg,
                    msglen,
                    c"<vim function: %s>".as_ptr(),
                    (*cb).data.funcref,
                );
            }
            kCallbackPartial => {
                snprintf(
                    msg,
                    msglen,
                    c"<vim partial: %s>".as_ptr(),
                    (*(*cb).data.partial).pt_name,
                );
            }
            _ => *msg = NUL as ::core::ffi::c_char,
        }
        msg
    }
}

/// Unregister the watcher on `dict` with this exact pattern and callback.
///
/// A watcher removed while any watcher on the queue is mid-callback is only
/// marked `needs_free`; [`tv_dict_watcher_notify`] unlinks it when the walk
/// that is running finishes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_watcher_remove(
    dict: *mut dict_T,
    key_pattern: *const ::core::ffi::c_char,
    key_pattern_len: size_t,
    mut callback: Callback,
) -> bool {
    unsafe {
        if dict.is_null() {
            return false;
        }

        let mut watcher = ::core::ptr::null_mut::<DictWatcher>();
        let mut matched = false;
        let mut queue_is_busy = false;
        // QUEUE_FOREACH; `w` stays on the matching node when the walk breaks.
        let mut w = (*dict).watchers.next;
        while w != &raw mut (*dict).watchers {
            let next = (*w).next;
            watcher = tv_dict_watcher_node_data(w);
            if (*watcher).busy {
                queue_is_busy = true;
            }
            if tv_callback_equal(&raw mut (*watcher).callback, &raw mut callback)
                && (*watcher).key_pattern_len == key_pattern_len
                && memcmp(
                    (*watcher).key_pattern.cast(),
                    key_pattern.cast(),
                    key_pattern_len,
                ) == 0
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
            (*watcher).needs_free = true;
        } else {
            QUEUE_REMOVE(w);
            tv_dict_watcher_free(watcher);
        }
        true
    }
}

/// Whether `watcher`'s pattern matches `key`.  A trailing `*` makes it a
/// prefix match.
pub(crate) unsafe fn tv_dict_watcher_matches(
    watcher: *mut DictWatcher,
    key: *const ::core::ffi::c_char,
) -> bool {
    unsafe {
        let len = (*watcher).key_pattern_len;
        if len != 0 && *(*watcher).key_pattern.add(len - 1) as ::core::ffi::c_int == '*' as i32 {
            return strncmp(key, (*watcher).key_pattern, len - 1) == 0;
        }
        strcmp(key, (*watcher).key_pattern) == 0
    }
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
    unsafe {
        let mut argv = [TV_INITIAL_VALUE; 3];
        argv[0].v_type = VAR_DICT;
        argv[0].v_lock = VAR_UNLOCKED;
        argv[0].vval.v_dict = dict;
        argv[1].v_type = VAR_STRING;
        argv[1].v_lock = VAR_UNLOCKED;
        argv[1].vval.v_string = xstrdup(key);
        argv[2].v_type = VAR_DICT;
        argv[2].v_lock = VAR_UNLOCKED;
        argv[2].vval.v_dict = tv_dict_alloc();
        (*argv[2].vval.v_dict).dv_refcount += 1;

        // `tv_dict_item_alloc_len` copies exactly the length given and appends
        // the NUL itself, so a Rust `&str` is upstream's `S_LEN(…)`.
        let event = argv[2].vval.v_dict;
        let add = |name: &str, from: *mut typval_T| {
            let v = tv_dict_item_alloc_len(name.as_ptr().cast(), name.len());
            tv_copy(from, &raw mut (*v).di_tv);
            tv_dict_add(event, v);
        };
        if !newtv.is_null() {
            add("new", newtv);
        }
        if !oldtv.is_null() && (*oldtv).v_type != VAR_UNKNOWN {
            add("old", oldtv);
        }

        let mut any_needs_free = false;
        // Hold the dictionary across the callbacks: one of them may drop the
        // last other reference to it.
        (*dict).dv_refcount += 1;
        // QUEUE_FOREACH: the next node is read before the body, so a callback
        // that unlinks the current watcher does not strand the walk.
        let mut w = (*dict).watchers.next;
        while w != &raw mut (*dict).watchers {
            let next = (*w).next;
            let watcher = tv_dict_watcher_node_data(w);
            if !(*watcher).busy && tv_dict_watcher_matches(watcher, key) {
                let mut rettv = TV_INITIAL_VALUE;
                (*watcher).busy = true;
                callback_call(
                    &raw mut (*watcher).callback,
                    3,
                    argv.as_mut_ptr(),
                    &raw mut rettv,
                );
                (*watcher).busy = false;
                tv_clear(&raw mut rettv);
                if (*watcher).needs_free {
                    any_needs_free = true;
                }
            }
            w = next;
        }

        if any_needs_free {
            let mut w = (*dict).watchers.next;
            while w != &raw mut (*dict).watchers {
                let next = (*w).next;
                let watcher = tv_dict_watcher_node_data(w);
                if (*watcher).needs_free {
                    QUEUE_REMOVE(w);
                    tv_dict_watcher_free(watcher);
                }
                w = next;
            }
        }
        tv_dict_unref(dict);

        // From 1: `argv[0]` is the caller's dictionary, which it still owns.
        for tv in &mut argv[1..] {
            tv_clear(tv);
        }
    }
}
