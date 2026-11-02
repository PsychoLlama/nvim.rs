//! Attaching to a buffer, and running code with it current.
//!
//! `nvim_buf_attach` registers the update callbacks a channel or a Lua
//! table receives on every change, and `nvim_buf_detach` drops them.
//! `nvim_buf_call` is the other direction -- it makes a buffer current for
//! the duration of one callback -- and `api_buf_ensure_loaded` is the
//! load-on-demand every accessor in the family funnels through.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::narrow::len_as_integer;
use crate::types::DictBuilder;
use crate::winlayer::Buf;

pub(crate) fn api_buf_ensure_loaded(buffer: BufferHandle) -> Result<Option<Buf>, Error> {
    let Some(b) = find_buffer_by_handle(buffer)? else {
        return Ok(None);
    };
    if b.b_ml.ml_mfp.is_none() && !buf_ensure_loaded(b) {
        return Err(Error::exception(c"Failed to load buffer"));
    }
    Ok(Some(b))
}

pub fn nvim_buf_attach(
    channel_id: uint64_t,
    buf: BufferHandle,
    send_buffer: Boolean,
    opts: &mut KeyDict_buf_attach,
) -> Result<Boolean, Error> {
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
    let mut tstate: TryState = TryState::INIT;
    try_enter(&mut tstate);
    let mut aco: AcoSave = AcoSave::default();
    aucmd_prepbuf(&mut aco, b);
    let args: Array = Array::new();
    let res = nlua_call_ref_reporting(fun, None, args, kRetLuaref);
    aucmd_restbuf(&mut aco);
    // The bracket outranks the call's own failure, as it did when both went
    // through one slot.
    try_leave(&tstate)?;
    res
}

// `nvim__buf_stats` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__buf_stats(buf: BufferHandle) -> Result<ApiDict, Error> {
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(ApiDict::new());
    };
    let buffer = b;
    let mut rv = DictBuilder::with_capacity(7 as size_t);
    let d_flush_count = Object::integer(Integer::from(b.flush_count));
    rv.insert(c"flush_count", d_flush_count);
    let d_current_lnum = Object::integer(Integer::from(b.b_ml.cached_lnum()));
    rv.insert(c"current_lnum", d_current_lnum);
    let d_line_dirty = Object::boolean(b.b_ml.line_is_dirty());
    rv.insert(c"line_dirty", d_line_dirty);
    let d_dirty_bytes = Object::integer(len_as_integer(b.deleted_bytes));
    rv.insert(c"dirty_bytes", d_dirty_bytes);
    let d_dirty_bytes2 = Object::integer(len_as_integer(b.deleted_bytes2));
    rv.insert(c"dirty_bytes2", d_dirty_bytes2);
    let total = buf_meta_total(buffer, kMTMetaLines);
    let d_virt_blocks = Object::integer(Integer::from(total));
    rv.insert(c"virt_blocks", d_virt_blocks);
    let tip = buffer;
    if let Some(uhp) = tip
        .header(tip.b_u_curhead)
        .or_else(|| tip.header(tip.b_u_newhead))
    {
        let d_uhp_extmark_size = Object::integer(len_as_integer(uhp.extmarks.len()));
        rv.insert(c"uhp_extmark_size", d_uhp_extmark_size);
    }
    Ok(rv.into())
}
