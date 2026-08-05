//! `dict_T` watchers and the `Callback` values they hold.
//!
//! [`tv_dict_watcher_add`] threads a `DictWatcher` onto `dv_watchers` and
//! [`tv_dict_watcher_notify`] fires every watcher whose pattern matches a
//! key that just changed, building the `{old, new}` dictionary each callback
//! is handed.  The `callback_*` half is `Callback`'s own lifetime — funcref,
//! partial and LuaRef each reference-counted differently.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn tv_dict_watcher_free(mut watcher: *mut DictWatcher) {
    unsafe {
        callback_free(&raw mut (*watcher).callback);
        xfree((*watcher).key_pattern as *mut ::core::ffi::c_void);
        xfree(watcher as *mut ::core::ffi::c_void);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_watcher_add(
    dict: *mut dict_T,
    key_pattern: *const ::core::ffi::c_char,
    key_pattern_len: size_t,
    mut callback: Callback,
) {
    unsafe {
        if dict.is_null() {
            return;
        }
        let watcher: *mut DictWatcher =
            xmalloc(::core::mem::size_of::<DictWatcher>()) as *mut DictWatcher;
        (*watcher).key_pattern =
            xmemdupz(key_pattern as *const ::core::ffi::c_void, key_pattern_len)
                as *mut ::core::ffi::c_char;
        (*watcher).key_pattern_len = key_pattern_len;
        (*watcher).callback = callback;
        (*watcher).busy = false_0 != 0;
        (*watcher).needs_free = false_0 != 0;
        QUEUE_INSERT_TAIL(&raw mut (*dict).watchers, &raw mut (*watcher).node);
    }
}

pub unsafe extern "C" fn tv_callback_equal(
    mut cb1: *const Callback,
    mut cb2: *const Callback,
) -> bool {
    unsafe {
        if (*cb1).type_0 as ::core::ffi::c_uint != (*cb2).type_0 as ::core::ffi::c_uint {
            return false_0 != 0;
        }
        match (*cb1).type_0 as ::core::ffi::c_uint {
            1 => {
                return strcmp((*cb1).data.funcref, (*cb2).data.funcref) == 0 as ::core::ffi::c_int;
            }
            2 => return (*cb1).data.partial == (*cb2).data.partial,
            3 => return (*cb1).data.luaref == (*cb2).data.luaref,
            0 => return true_0 != 0,
            _ => {}
        }
        abort();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn callback_free(mut callback: *mut Callback) {
    unsafe {
        match (*callback).type_0 as ::core::ffi::c_uint {
            1 => {
                func_unref((*callback).data.funcref);
                xfree((*callback).data.funcref as *mut ::core::ffi::c_void);
            }
            2 => {
                partial_unref((*callback).data.partial);
            }
            3 => {
                if (*callback).data.luaref != LUA_NOREF {
                    api_free_luaref((*callback).data.luaref);
                    (*callback).data.luaref = LUA_NOREF as LuaRef;
                }
            }
            0 | _ => {}
        }
        (*callback).type_0 = kCallbackNone;
        (*callback).data.funcref = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub unsafe extern "C" fn callback_put(mut cb: *mut Callback, mut tv: *mut typval_T) {
    unsafe {
        match (*cb).type_0 as ::core::ffi::c_uint {
            2 => {
                (*tv).v_type = VAR_PARTIAL;
                (*tv).vval.v_partial = (*cb).data.partial;
                (*(*cb).data.partial).pt_refcount += 1;
            }
            1 => {
                (*tv).v_type = VAR_FUNC;
                (*tv).vval.v_string = xstrdup((*cb).data.funcref);
                func_ref((*cb).data.funcref);
            }
            3 | _ => {
                (*tv).v_type = VAR_SPECIAL;
                (*tv).vval.v_special = kSpecialVarNull;
            }
        };
    }
}

pub unsafe extern "C" fn callback_copy(mut dest: *mut Callback, mut src: *mut Callback) {
    unsafe {
        (*dest).type_0 = (*src).type_0;
        match (*src).type_0 as ::core::ffi::c_uint {
            2 => {
                (*dest).data.partial = (*src).data.partial;
                (*(*dest).data.partial).pt_refcount += 1;
            }
            1 => {
                (*dest).data.funcref = xstrdup((*src).data.funcref);
                func_ref((*src).data.funcref);
            }
            3 => {
                (*dest).data.luaref = api_new_luaref((*src).data.luaref);
            }
            _ => {
                (*dest).data.funcref = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
        };
    }
}

pub unsafe extern "C" fn callback_to_string(
    mut cb: *mut Callback,
    mut arena: *mut Arena,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if (*cb).type_0 as ::core::ffi::c_uint
            == kCallbackLua as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return nlua_funcref_str((*cb).data.luaref, arena);
        }
        let msglen: size_t = 100 as size_t;
        let mut msg: *mut ::core::ffi::c_char = xmallocz(msglen) as *mut ::core::ffi::c_char;
        match (*cb).type_0 as ::core::ffi::c_uint {
            1 => {
                snprintf(
                    msg,
                    msglen,
                    b"<vim function: %s>\0".as_ptr() as *const ::core::ffi::c_char,
                    (*cb).data.funcref,
                );
            }
            2 => {
                snprintf(
                    msg,
                    msglen,
                    b"<vim partial: %s>\0".as_ptr() as *const ::core::ffi::c_char,
                    (*(*cb).data.partial).pt_name,
                );
            }
            _ => {
                *msg = NUL as ::core::ffi::c_char;
            }
        }
        return msg;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_watcher_remove(
    dict: *mut dict_T,
    key_pattern: *const ::core::ffi::c_char,
    key_pattern_len: size_t,
    mut callback: Callback,
) -> bool {
    unsafe {
        if dict.is_null() {
            return false_0 != 0;
        }
        let mut w: *mut QUEUE = ::core::ptr::null_mut::<QUEUE>();
        let mut watcher: *mut DictWatcher = ::core::ptr::null_mut::<DictWatcher>();
        let mut matched: bool = false_0 != 0;
        let mut queue_is_busy: bool = false_0 != 0;
        w = (*dict).watchers.next as *mut QUEUE;
        while w != &raw mut (*dict).watchers {
            let mut next: *mut QUEUE = (*w).next as *mut QUEUE;
            watcher = tv_dict_watcher_node_data(w);
            if (*watcher).busy {
                queue_is_busy = true;
            }
            if tv_callback_equal(&raw mut (*watcher).callback, &raw mut callback)
                as ::core::ffi::c_int
                != 0
                && (*watcher).key_pattern_len == key_pattern_len
                && memcmp(
                    (*watcher).key_pattern as *const ::core::ffi::c_void,
                    key_pattern as *const ::core::ffi::c_void,
                    key_pattern_len,
                ) == 0 as ::core::ffi::c_int
            {
                matched = true;
                break;
            } else {
                w = next;
            }
        }
        if !matched {
            return false_0 != 0;
        }
        if queue_is_busy {
            (*watcher).needs_free = true_0 != 0;
        } else {
            QUEUE_REMOVE(w);
            tv_dict_watcher_free(watcher);
        }
        return true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn tv_dict_watcher_matches(
    mut watcher: *mut DictWatcher,
    key: *const ::core::ffi::c_char,
) -> bool {
    unsafe {
        let len: size_t = (*watcher).key_pattern_len;
        if len != 0
            && *(*watcher)
                .key_pattern
                .offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                == '*' as ::core::ffi::c_int
        {
            return strncmp(key, (*watcher).key_pattern, len.wrapping_sub(1 as size_t))
                == 0 as ::core::ffi::c_int;
        }
        return strcmp(key, (*watcher).key_pattern) == 0 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn tv_dict_watcher_notify(
    dict: *mut dict_T,
    key: *const ::core::ffi::c_char,
    newtv: *mut typval_T,
    oldtv: *mut typval_T,
) {
    unsafe {
        let mut argv: [typval_T; 3] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 3];
        argv[0 as ::core::ffi::c_int as usize].v_type = VAR_DICT;
        argv[0 as ::core::ffi::c_int as usize].v_lock = VAR_UNLOCKED;
        argv[0 as ::core::ffi::c_int as usize].vval.v_dict = dict;
        argv[1 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
        argv[1 as ::core::ffi::c_int as usize].v_lock = VAR_UNLOCKED;
        argv[1 as ::core::ffi::c_int as usize].vval.v_string = xstrdup(key);
        argv[2 as ::core::ffi::c_int as usize].v_type = VAR_DICT;
        argv[2 as ::core::ffi::c_int as usize].v_lock = VAR_UNLOCKED;
        argv[2 as ::core::ffi::c_int as usize].vval.v_dict = tv_dict_alloc();
        (*argv[2 as ::core::ffi::c_int as usize].vval.v_dict).dv_refcount += 1;
        if !newtv.is_null() {
            let v: *mut dictitem_T = tv_dict_item_alloc_len(
                b"new\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
            );
            tv_copy(newtv, &raw mut (*v).di_tv);
            tv_dict_add(argv[2 as ::core::ffi::c_int as usize].vval.v_dict, v);
        }
        if !oldtv.is_null()
            && (*oldtv).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let v_0: *mut dictitem_T = tv_dict_item_alloc_len(
                b"old\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
            );
            tv_copy(oldtv, &raw mut (*v_0).di_tv);
            tv_dict_add(argv[2 as ::core::ffi::c_int as usize].vval.v_dict, v_0);
        }
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut any_needs_free: bool = false_0 != 0;
        (*dict).dv_refcount += 1;
        let mut w: *mut QUEUE = ::core::ptr::null_mut::<QUEUE>();
        w = (*dict).watchers.next as *mut QUEUE;
        while w != &raw mut (*dict).watchers {
            let mut next: *mut QUEUE = (*w).next as *mut QUEUE;
            let mut watcher: *mut DictWatcher = tv_dict_watcher_node_data(w);
            if !(*watcher).busy && tv_dict_watcher_matches(watcher, key) as ::core::ffi::c_int != 0
            {
                rettv = typval_T {
                    v_type: VAR_UNKNOWN,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union { v_number: 0 },
                };
                (*watcher).busy = true;
                callback_call(
                    &raw mut (*watcher).callback,
                    3 as ::core::ffi::c_int,
                    &raw mut argv as *mut typval_T,
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
            w = (*dict).watchers.next as *mut QUEUE;
            while w != &raw mut (*dict).watchers {
                let mut next_0: *mut QUEUE = (*w).next as *mut QUEUE;
                let mut watcher_0: *mut DictWatcher = tv_dict_watcher_node_data(w);
                if (*watcher_0).needs_free {
                    QUEUE_REMOVE(w);
                    tv_dict_watcher_free(watcher_0);
                }
                w = next_0;
            }
        }
        tv_dict_unref(dict);
        let mut i: size_t = 1 as size_t;
        while i < ::core::mem::size_of::<[typval_T; 3]>()
            .wrapping_div(::core::mem::size_of::<typval_T>())
            .wrapping_div(
                (::core::mem::size_of::<[typval_T; 3]>()
                    .wrapping_rem(::core::mem::size_of::<typval_T>())
                    == 0) as ::core::ffi::c_int as usize,
            )
        {
            tv_clear((&raw mut argv as *mut typval_T).offset(i as isize));
            i = i.wrapping_add(1);
        }
    }
}
