//! Attaching to a buffer, and running code with it current.
//!
//! `nvim_buf_attach` registers the update callbacks a channel or a Lua
//! table receives on every change, and `nvim_buf_detach` drops them.
//! `nvim_buf_call` is the other direction -- it makes a buffer current for
//! the duration of one callback -- and `api_buf_ensure_loaded` is the
//! load-on-demand every accessor in the family funnels through.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, NIL, Reported, dict_put, has_key};
use crate::winlayer::Buf;

pub unsafe fn api_buf_ensure_loaded(mut buf: Buffer, mut err: *mut Error) -> *mut buf_T {
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return ::core::ptr::null_mut::<buf_T>();
        }
        if (*b).b_ml.ml_mfp.is_null() && !buf_ensure_loaded(b) {
            api_set_error(err, kErrorTypeException, c"Failed to load buffer".as_ptr());
            return ::core::ptr::null_mut::<buf_T>();
        }
        b
    }
}

pub unsafe fn nvim_buf_attach(
    channel_id: uint64_t,
    buf: Buffer,
    send_buffer: Boolean,
    opts: *mut KeyDict_buf_attach,
) -> Result<Boolean, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return false.reported(error);
        }
        let mut cb: BufUpdateCallbacks = BUF_UPDATE_CALLBACKS_INIT;
        if channel_id == LUA_INTERNAL_CALL {
            if has_key(
                (*opts).is_set__buf_attach_,
                KEYSET_OPTIDX_buf_attach__on_lines,
            ) {
                cb.on_lines = (*opts).on_lines;
                (*opts).on_lines = LUA_NOREF as LuaRef;
            }
            if has_key(
                (*opts).is_set__buf_attach_,
                KEYSET_OPTIDX_buf_attach__on_bytes,
            ) {
                cb.on_bytes = (*opts).on_bytes;
                (*opts).on_bytes = LUA_NOREF as LuaRef;
            }
            if has_key(
                (*opts).is_set__buf_attach_,
                KEYSET_OPTIDX_buf_attach__on_changedtick,
            ) {
                cb.on_changedtick = (*opts).on_changedtick;
                (*opts).on_changedtick = LUA_NOREF as LuaRef;
            }
            if has_key(
                (*opts).is_set__buf_attach_,
                KEYSET_OPTIDX_buf_attach__on_detach,
            ) {
                cb.on_detach = (*opts).on_detach;
                (*opts).on_detach = LUA_NOREF as LuaRef;
            }
            if has_key(
                (*opts).is_set__buf_attach_,
                KEYSET_OPTIDX_buf_attach__on_reload,
            ) {
                cb.on_reload = (*opts).on_reload;
                (*opts).on_reload = LUA_NOREF as LuaRef;
            }
            cb.utf_sizes = (*opts).utf_sizes;
            cb.preview = (*opts).preview;
        }
        buf_updates_register(b, channel_id, cb, send_buffer).reported(error)
    }
}

pub unsafe fn nvim_buf_detach(channel_id: uint64_t, buf: Buffer) -> Result<Boolean, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return false.reported(error);
        }
        buf_updates_unregister(b, channel_id);
        true.reported(error)
    }
}

pub unsafe fn nvim_buf_call(buf: Buffer, fun: LuaRef) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return NIL.reported(error);
        }
        let mut res: Object = NIL;
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
        res.reported(error)
    }
}

pub unsafe fn nvim__buf_stats(buf: Buffer, arena: *mut Arena) -> Result<Dict, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return Dict {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<KeyValuePair>(),
            }
            .reported(error);
        }
        let mut rv: Dict = arena_dict(arena, 7 as size_t);
        dict_put(
            &mut rv,
            c"flush_count",
            Object::integer((*b).flush_count as Integer),
        );
        dict_put(
            &mut rv,
            c"current_lnum",
            Object::integer((*b).b_ml.ml_line_lnum as Integer),
        );
        dict_put(
            &mut rv,
            c"line_dirty",
            Object::boolean((*b).b_ml.ml_flags & 0x2 as ::core::ffi::c_int != 0),
        );
        dict_put(
            &mut rv,
            c"dirty_bytes",
            Object::integer((*b).deleted_bytes as Integer),
        );
        dict_put(
            &mut rv,
            c"dirty_bytes2",
            Object::integer((*b).deleted_bytes2 as Integer),
        );
        dict_put(
            &mut rv,
            c"virt_blocks",
            Object::integer(buf_meta_total(b, kMTMetaLines) as Integer),
        );
        // SAFETY: a live buffer, as above.
        let tip = Buf::new(b);
        if let Some(uhp) = tip
            .header(tip.b_u_curhead)
            .or_else(|| tip.header(tip.b_u_newhead))
        {
            dict_put(
                &mut rv,
                c"uhp_extmark_size",
                Object::integer(uhp.uh_extmark.size as Integer),
            );
        }
        rv.reported(error)
    }
}
