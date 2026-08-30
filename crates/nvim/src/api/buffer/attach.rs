//! Attaching to a buffer, and running code with it current.
//!
//! `nvim_buf_attach` registers the update callbacks a channel or a Lua
//! table receives on every change, and `nvim_buf_detach` drops them.
//! `nvim_buf_call` is the other direction -- it makes a buffer current for
//! the duration of one callback -- and `api_buf_ensure_loaded` is the
//! load-on-demand every accessor in the family funnels through.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{NIL, Reported, dict_put, has_key};
use crate::winlayer::{Buf, Live};

pub unsafe fn api_buf_ensure_loaded(mut buf: Buffer, err: &mut Error) -> *mut buf_T {
    let mut b: *mut buf_T = unsafe { find_buffer_by_handle(buf, err) };
    if b.is_null() {
        return ::core::ptr::null_mut::<buf_T>();
    }
    if unsafe { (*b).b_ml.ml_mfp }.is_null() && !buf_ensure_loaded(unsafe { Buf::new(b) }) {
        *err = Error::exception(c"Failed to load buffer");
        return ::core::ptr::null_mut::<buf_T>();
    }
    b
}

pub unsafe fn nvim_buf_attach(
    channel_id: uint64_t,
    buf: Buffer,
    send_buffer: Boolean,
    opts: *mut KeyDict_buf_attach,
) -> Result<Boolean, Error> {
    // SAFETY: the dispatcher's keyset outlives this call.
    let mut opts = unsafe { Live::<KeyDict_buf_attach>::new(opts) };
    let mut error = Error::none();
    let mut b: *mut buf_T = unsafe { find_buffer_by_handle(buf, &mut error) };
    if b.is_null() {
        return false.reported(error);
    }
    let mut cb: BufUpdateCallbacks = BUF_UPDATE_CALLBACKS_INIT;
    if channel_id == LUA_INTERNAL_CALL {
        if has_key(opts.is_set__buf_attach_, KEYSET_OPTIDX_buf_attach__on_lines) {
            cb.on_lines = opts.on_lines;
            opts.on_lines = LUA_NOREF as LuaRef;
        }
        if has_key(opts.is_set__buf_attach_, KEYSET_OPTIDX_buf_attach__on_bytes) {
            cb.on_bytes = opts.on_bytes;
            opts.on_bytes = LUA_NOREF as LuaRef;
        }
        if has_key(
            opts.is_set__buf_attach_,
            KEYSET_OPTIDX_buf_attach__on_changedtick,
        ) {
            cb.on_changedtick = opts.on_changedtick;
            opts.on_changedtick = LUA_NOREF as LuaRef;
        }
        if has_key(
            opts.is_set__buf_attach_,
            KEYSET_OPTIDX_buf_attach__on_detach,
        ) {
            cb.on_detach = opts.on_detach;
            opts.on_detach = LUA_NOREF as LuaRef;
        }
        if has_key(
            opts.is_set__buf_attach_,
            KEYSET_OPTIDX_buf_attach__on_reload,
        ) {
            cb.on_reload = opts.on_reload;
            opts.on_reload = LUA_NOREF as LuaRef;
        }
        cb.utf_sizes = opts.utf_sizes;
        cb.preview = opts.preview;
    }
    unsafe { buf_updates_register(b, channel_id, cb, send_buffer) }.reported(error)
}

pub unsafe fn nvim_buf_detach(channel_id: uint64_t, buf: Buffer) -> Result<Boolean, Error> {
    let mut error = Error::none();
    let mut b: *mut buf_T = unsafe { find_buffer_by_handle(buf, &mut error) };
    if b.is_null() {
        return false.reported(error);
    }
    unsafe { buf_updates_unregister(b, channel_id) };
    true.reported(error)
}

pub unsafe fn nvim_buf_call(buf: Buffer, fun: LuaRef) -> Result<Object, Error> {
    let mut error = Error::none();
    let mut b: *mut buf_T = unsafe { find_buffer_by_handle(buf, &mut error) };
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
    unsafe { try_enter(&raw mut tstate) };
    let mut aco: aco_save_T = aco_save_T::default();
    unsafe { aucmd_prepbuf(&raw mut aco, b) };
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    res = unsafe {
        nlua_call_ref(
            fun,
            ::core::ptr::null::<::core::ffi::c_char>(),
            args,
            kRetLuaref,
            ::core::ptr::null_mut::<Arena>(),
            &mut error,
        )
    };
    unsafe { aucmd_restbuf(&raw mut aco) };
    unsafe { try_leave(&raw mut tstate, &mut error) };
    res.reported(error)
}

pub unsafe fn nvim__buf_stats(buf: Buffer, arena: *mut Arena) -> Result<Dict, Error> {
    let mut error = Error::none();
    let mut b: *mut buf_T = unsafe { find_buffer_by_handle(buf, &mut error) };
    if b.is_null() {
        return Dict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        }
        .reported(error);
    }
    let mut rv: Dict = arena_dict(arena, 7 as size_t);
    // SAFETY: a live pointer the code around it already holds.
    let d_flush_count = unsafe { Object::integer((*b).flush_count as Integer) };
    // SAFETY: the collection is this call's own.
    unsafe { dict_put(&mut rv, c"flush_count", d_flush_count) };
    // SAFETY: a live pointer the code around it already holds.
    let d_current_lnum = unsafe { Object::integer((*b).b_ml.cached_lnum() as Integer) };
    // SAFETY: the collection is this call's own.
    unsafe { dict_put(&mut rv, c"current_lnum", d_current_lnum) };
    // SAFETY: a live pointer the code around it already holds.
    let d_line_dirty = unsafe { Object::boolean((*b).b_ml.line_is_dirty()) };
    // SAFETY: the collection is this call's own.
    unsafe { dict_put(&mut rv, c"line_dirty", d_line_dirty) };
    // SAFETY: a live pointer the code around it already holds.
    let d_dirty_bytes = unsafe { Object::integer((*b).deleted_bytes as Integer) };
    // SAFETY: the collection is this call's own.
    unsafe { dict_put(&mut rv, c"dirty_bytes", d_dirty_bytes) };
    // SAFETY: a live pointer the code around it already holds.
    let d_dirty_bytes2 = unsafe { Object::integer((*b).deleted_bytes2 as Integer) };
    // SAFETY: the collection is this call's own.
    unsafe { dict_put(&mut rv, c"dirty_bytes2", d_dirty_bytes2) };
    // SAFETY: a live buffer, as above.
    let total = unsafe { buf_meta_total(Buf::new(b), kMTMetaLines) };
    let d_virt_blocks = Object::integer(total as Integer);
    // SAFETY: the collection is this call's own.
    unsafe { dict_put(&mut rv, c"virt_blocks", d_virt_blocks) };
    // SAFETY: a live buffer, as above.
    let tip = unsafe { Buf::new(b) };
    if let Some(uhp) = tip
        .header(tip.b_u_curhead)
        .or_else(|| tip.header(tip.b_u_newhead))
    {
        let d_uhp_extmark_size = Object::integer(uhp.uh_extmark.size as Integer);
        // SAFETY: the collection is this call's own.
        unsafe { dict_put(&mut rv, c"uhp_extmark_size", d_uhp_extmark_size) };
    }
    rv.reported(error)
}
