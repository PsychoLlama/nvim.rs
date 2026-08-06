//! Attaching to a buffer, and running code with it current.
//!
//! `nvim_buf_attach` registers the update callbacks a channel or a Lua
//! table receives on every change, and `nvim_buf_detach` drops them.
//! `nvim_buf_call` is the other direction -- it makes a buffer current for
//! the duration of one callback -- and `api_buf_ensure_loaded` is the
//! load-on-demand every accessor in the family funnels through.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn api_buf_ensure_loaded(mut buf: Buffer, mut err: *mut Error) -> *mut buf_T {
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return ::core::ptr::null_mut::<buf_T>();
        }
        if (*b).b_ml.ml_mfp.is_null() && !buf_ensure_loaded(b) {
            api_set_error(
                err,
                kErrorTypeException,
                b"Failed to load buffer\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return ::core::ptr::null_mut::<buf_T>();
        }
        return b;
    }
}

pub unsafe extern "C" fn nvim_buf_attach(
    mut channel_id: uint64_t,
    mut buf: Buffer,
    mut send_buffer: Boolean,
    mut opts: *mut KeyDict_buf_attach,
    mut err: *mut Error,
) -> Boolean {
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return false_0 != 0;
        }
        let mut cb: BufUpdateCallbacks = BUF_UPDATE_CALLBACKS_INIT;
        if channel_id == LUA_INTERNAL_CALL {
            if (*opts).is_set__buf_attach_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_buf_attach__on_lines
                != 0 as ::core::ffi::c_ulonglong
            {
                cb.on_lines = (*opts).on_lines;
                (*opts).on_lines = LUA_NOREF as LuaRef;
            }
            if (*opts).is_set__buf_attach_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_buf_attach__on_bytes
                != 0 as ::core::ffi::c_ulonglong
            {
                cb.on_bytes = (*opts).on_bytes;
                (*opts).on_bytes = LUA_NOREF as LuaRef;
            }
            if (*opts).is_set__buf_attach_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_buf_attach__on_changedtick
                != 0 as ::core::ffi::c_ulonglong
            {
                cb.on_changedtick = (*opts).on_changedtick;
                (*opts).on_changedtick = LUA_NOREF as LuaRef;
            }
            if (*opts).is_set__buf_attach_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_buf_attach__on_detach
                != 0 as ::core::ffi::c_ulonglong
            {
                cb.on_detach = (*opts).on_detach;
                (*opts).on_detach = LUA_NOREF as LuaRef;
            }
            if (*opts).is_set__buf_attach_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_buf_attach__on_reload
                != 0 as ::core::ffi::c_ulonglong
            {
                cb.on_reload = (*opts).on_reload;
                (*opts).on_reload = LUA_NOREF as LuaRef;
            }
            cb.utf_sizes = (*opts).utf_sizes as bool;
            cb.preview = (*opts).preview as bool;
        }
        return buf_updates_register(b, channel_id, cb, send_buffer as bool);
    }
}

pub unsafe extern "C" fn nvim_buf_detach(
    mut channel_id: uint64_t,
    mut buf: Buffer,
    mut err: *mut Error,
) -> Boolean {
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return false_0 != 0;
        }
        buf_updates_unregister(b, channel_id);
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn nvim_buf_call(
    mut buf: Buffer,
    mut fun: LuaRef,
    mut err: *mut Error,
) -> Object {
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
        }
        let mut res: Object = object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
        let mut tstate: TryState = TryState {
            current_exception: ::core::ptr::null_mut::<except_T>(),
            private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
            msg_list: ::core::ptr::null::<*const msglist_T>(),
            got_int: 0,
            did_throw: false,
            need_rethrow: 0,
            did_emsg: 0,
        };
        try_enter(&raw mut tstate);
        let mut aco: aco_save_T = aco_save_T::default();
        aucmd_prepbuf(&raw mut aco, b);
        let mut args: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        res = nlua_call_ref(
            fun,
            ::core::ptr::null::<::core::ffi::c_char>(),
            args,
            kRetLuaref,
            ::core::ptr::null_mut::<Arena>(),
            err,
        );
        aucmd_restbuf(&raw mut aco);
        try_leave(&raw mut tstate, err);
        return res;
    }
}

pub unsafe extern "C" fn nvim__buf_stats(
    mut buf: Buffer,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return Dict {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<KeyValuePair>(),
            };
        }
        let mut rv: Dict = arena_dict(arena, 7 as size_t);
        let c2rust_fresh4 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh4 as isize) = key_value_pair {
            key: cstr_as_string(b"flush_count\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (*b).flush_count as Integer,
                },
            },
        };
        let c2rust_fresh5 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh5 as isize) = key_value_pair {
            key: cstr_as_string(b"current_lnum\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (*b).b_ml.ml_line_lnum as Integer,
                },
            },
        };
        let c2rust_fresh6 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh6 as isize) = key_value_pair {
            key: cstr_as_string(b"line_dirty\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: (*b).b_ml.ml_flags & 0x2 as ::core::ffi::c_int != 0,
                },
            },
        };
        let c2rust_fresh7 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh7 as isize) = key_value_pair {
            key: cstr_as_string(b"dirty_bytes\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (*b).deleted_bytes as Integer,
                },
            },
        };
        let c2rust_fresh8 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh8 as isize) = key_value_pair {
            key: cstr_as_string(b"dirty_bytes2\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (*b).deleted_bytes2 as Integer,
                },
            },
        };
        let c2rust_fresh9 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh9 as isize) = key_value_pair {
            key: cstr_as_string(b"virt_blocks\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: buf_meta_total(b, kMTMetaLines) as Integer,
                },
            },
        };
        let mut uhp: *mut u_header_T = ::core::ptr::null_mut::<u_header_T>();
        if !(*b).b_u_curhead.is_null() {
            uhp = (*b).b_u_curhead;
        } else if !(*b).b_u_newhead.is_null() {
            uhp = (*b).b_u_newhead;
        }
        if !uhp.is_null() {
            let c2rust_fresh10 = rv.size;
            rv.size = rv.size.wrapping_add(1);
            *rv.items.offset(c2rust_fresh10 as isize) = key_value_pair {
                key: cstr_as_string(b"uhp_extmark_size\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: (*uhp).uh_extmark.size as Integer,
                    },
                },
            };
        }
        return rv;
    }
}
