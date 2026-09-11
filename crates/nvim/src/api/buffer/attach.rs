//! Attaching to a buffer, and running code with it current.
//!
//! `nvim_buf_attach` registers the update callbacks a channel or a Lua
//! table receives on every change, and `nvim_buf_detach` drops them.
//! `nvim_buf_call` is the other direction -- it makes a buffer current for
//! the duration of one callback -- and `api_buf_ensure_loaded` is the
//! load-on-demand every accessor in the family funnels through.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::winlayer::Buf;
use crate::winlayer::Live;

pub(crate) fn api_buf_ensure_loaded(buffer: BufferHandle) -> Result<Option<Buf>, Error> {
    let Some(b) = find_buffer_by_handle(buffer)? else {
        return Ok(None);
    };
    if b.b_ml.ml_mfp.is_null() && !buf_ensure_loaded(b) {
        return Err(Error::exception(c"Failed to load buffer"));
    }
    Ok(Some(b))
}

/// # Safety
///
/// `opts` must point at the `KeyDict_buf_attach` the dispatcher filled in,
/// live for the call.
pub unsafe fn nvim_buf_attach(
    channel_id: uint64_t,
    buf: BufferHandle,
    send_buffer: Boolean,
    opts: *mut KeyDict_buf_attach,
) -> Result<Boolean, Error> {
    // SAFETY: the dispatcher's keyset outlives this call.
    let mut opts = unsafe { Live::<KeyDict_buf_attach>::new(opts) };
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(false);
    };
    let mut cb: BufUpdateCallbacks = BUF_UPDATE_CALLBACKS_INIT;
    if channel_id == LUA_INTERNAL_CALL {
        // The callbacks move into the registration, so the keyset gives up
        // each reference it hands over.
        if let Some(reference) = opts.on_lines.take() {
            cb.on_lines = reference;
        }
        if let Some(reference) = opts.on_bytes.take() {
            cb.on_bytes = reference;
        }
        if let Some(reference) = opts.on_changedtick.take() {
            cb.on_changedtick = reference;
        }
        if let Some(reference) = opts.on_detach.take() {
            cb.on_detach = reference;
        }
        if let Some(reference) = opts.on_reload.take() {
            cb.on_reload = reference;
        }
        cb.utf_sizes = opts.utf_sizes.unwrap_or(false);
        cb.preview = opts.preview.unwrap_or(false);
    }
    Ok(buf_updates_register(b, channel_id, cb, send_buffer))
}

pub fn nvim_buf_detach(channel_id: uint64_t, buf: BufferHandle) -> Result<Boolean, Error> {
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(false);
    };
    buf_updates_unregister(b, channel_id);
    Ok(true)
}

pub fn nvim_buf_call(buf: BufferHandle, fun: LuaRef) -> Result<Object, Error> {
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(Object::Nil);
    };
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<Exception>(),
        private_msg_list: ::core::ptr::null_mut::<MsgList>(),
        msg_list: ::core::ptr::null::<*const MsgList>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    unsafe { try_enter(&raw mut tstate) };
    let mut aco: AcoSave = AcoSave::default();
    unsafe { aucmd_prepbuf(&raw mut aco, b) };
    let args: Array = Array::EMPTY;
    let res = unsafe {
        nlua_call_ref(
            fun,
            ::core::ptr::null::<::core::ffi::c_char>(),
            args,
            kRetLuaref,
            ::core::ptr::null_mut::<Arena>(),
        )
    };
    unsafe { aucmd_restbuf(&raw mut aco) };
    // The bracket outranks the call's own failure, as it did when both went
    // through one slot.
    unsafe { try_leave(&raw mut tstate) }?;
    res
}

/// # Safety
///
/// `arena` must point at a live arena, which the memory this answers with is
/// taken from and must outlive.
// `nvim__buf_stats` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub unsafe fn nvim__buf_stats(buf: BufferHandle) -> Result<ApiDict, Error> {
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(ApiDict::EMPTY);
    };
    let buffer = b;
    let mut rv: ApiDict = ApiDict::with_capacity(7 as size_t);
    // SAFETY: a live pointer the code around it already holds.
    let d_flush_count = Object::integer(b.flush_count as Integer);
    // SAFETY: the collection is this call's own.
    rv.insert(c"flush_count", d_flush_count);
    // SAFETY: a live pointer the code around it already holds.
    let d_current_lnum = Object::integer(b.b_ml.cached_lnum() as Integer);
    // SAFETY: the collection is this call's own.
    rv.insert(c"current_lnum", d_current_lnum);
    // SAFETY: a live pointer the code around it already holds.
    let d_line_dirty = Object::boolean(b.b_ml.line_is_dirty());
    // SAFETY: the collection is this call's own.
    rv.insert(c"line_dirty", d_line_dirty);
    // SAFETY: a live pointer the code around it already holds.
    let d_dirty_bytes = Object::integer(b.deleted_bytes as Integer);
    // SAFETY: the collection is this call's own.
    rv.insert(c"dirty_bytes", d_dirty_bytes);
    // SAFETY: a live pointer the code around it already holds.
    let d_dirty_bytes2 = Object::integer(b.deleted_bytes2 as Integer);
    // SAFETY: the collection is this call's own.
    rv.insert(c"dirty_bytes2", d_dirty_bytes2);
    let total = buf_meta_total(buffer, kMTMetaLines);
    let d_virt_blocks = Object::integer(total as Integer);
    // SAFETY: the collection is this call's own.
    rv.insert(c"virt_blocks", d_virt_blocks);
    let tip = buffer;
    if let Some(uhp) = tip
        .header(tip.b_u_curhead)
        .or_else(|| tip.header(tip.b_u_newhead))
    {
        let d_uhp_extmark_size = Object::integer(uhp.uh_extmark.size as Integer);
        // SAFETY: the collection is this call's own.
        rv.insert(c"uhp_extmark_size", d_uhp_extmark_size);
    }
    Ok(rv)
}
