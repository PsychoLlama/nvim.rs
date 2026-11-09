//! `nvim_open_term()`: a terminal whose input is a Lua callback.
//!
//! The buffer is given a `Terminal` whose write/resize/close hooks
//! (`term_write`, `term_resize`, `term_close`) forward to the caller
//! instead of to a process, which is what makes a channel-backed or
//! purely virtual terminal possible.  `nvim_chan_send` is the other
//! direction, and shares nothing but the channel.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api_error;
use crate::channel::defs::{ChannelStream, InternalState};
use crate::guard::Lock;
use crate::lua::executor::nlua_call_ref_quiet;
use crate::lua::executor::release_luaref;
use crate::winlayer::Win;

pub fn nvim_open_term(buf: BufferHandle, opts: &mut KeyDict_open_term) -> Result<Integer, Error> {
    let Some(buffer) = api_buf_ensure_loaded(buf)? else {
        return Ok(0 as Integer);
    };
    if cmdwin_buf.get() == Some(buffer.id()) {
        return Err(Error::exception(E_CMDWIN));
    }
    let mut may_read_buffer: bool = true;
    if !buffer.terminal.is_null() {
        if buf_terminal_running(buffer) {
            let handle = buffer.handle;
            return Err(api_error!(
                kErrorTypeException,
                "Terminal already connected to buffer {handle}"
            ));
        }
        buf_close_terminal(buffer);
        may_read_buffer = false;
    }
    // The channel takes the callback's reference over, so the keyset must
    // not release it too.
    let cb: LuaRef = opts.on_input.take().unwrap_or(LUA_NOREF);
    let id = alloc_channel(None, |_| {
        ChannelStream::Internal(InternalState { cb, closed: false })
    });
    let (view_width, view_height, col_off) = (
        Win::current().w_view_width,
        Win::current().w_view_height,
        Win::current().col_off(),
    );
    let topts: TerminalOptions = TerminalOptions {
        owner: id,
        width: u16::try_from((view_width - col_off).max(0)).unwrap_or(u16::MAX),
        height: u16::try_from(view_height).unwrap_or(u16::MAX),
        read_pause_cb: Some(term_read_pause),
        write_cb: Some(term_write),
        resize_cb: Some(term_resize),
        resume_cb: Some(term_resume),
        close_cb: Some(term_close),
        force_crlf: opts.force_crlf.unwrap_or(true),
    };
    let mut contents: Vec<u8> = Vec::new();
    if may_read_buffer {
        buffer.read_into(LineNr::ONE, buffer.b_ml.ml_line_count, &mut contents);
    }
    // Held across `TermOpen`, which may close the terminal.
    let held = ChannelRef::new(id);
    let term = terminal_alloc(buffer, topts);
    with_channel(id, |channel| channel.term = Some(term));
    let attached = || with_channel(id, |channel| channel.term.is_some()).unwrap_or(false);
    terminal_open(term, buffer, &attached);
    if let Some(term) = with_channel(id, |channel| channel.term).flatten() {
        terminal_check_size(term);
    }
    drop(held);
    if !contents.is_empty() {
        channel_send(id, contents).map_err(Error::validation)?;
    }
    Ok(id.cast_signed())
}

fn term_read_pause(_pause: bool, _owner: uint64_t) {}

/// The user typed into the terminal: hand it to the `on_input` callback.
fn term_write(bytes: &[u8], id: uint64_t) {
    let Some((cb, term)) = with_channel(id, |channel| {
        let cb = match channel.stream() {
            ChannelStream::Internal(internal) => internal.cb,
            _ => LUA_NOREF,
        };
        (cb, channel.term)
    }) else {
        return;
    };
    if cb == LUA_NOREF {
        return;
    }
    let args = vec![
        Object::integer(id.cast_signed()),
        Object::buffer(term.map_or(Handle::NONE, terminal_buf)),
        Object::string(ApiString::from_bytes(bytes)),
    ];
    let _locked = Lock::text();
    // The handler reports nothing, so it is given no error slot.
    nlua_call_ref_quiet(cb, Some(c"input"), args.into(), RET_NIL_BOOL);
}

fn term_resize(_width: uint16_t, _height: uint16_t, _owner: uint64_t) {}

fn term_resume(_owner: uint64_t) {}

/// The terminal went away: free it, the callback, and the channel's
/// reference.
fn term_close(id: uint64_t) {
    if let Some(term) = with_channel(id, |channel| channel.term).flatten()
        && terminal_destroy(term)
    {
        with_channel(id, |channel| channel.term = None);
    }
    let cb = with_channel(id, |channel| match channel.stream_mut().internal() {
        Some(internal) => core::mem::replace(&mut internal.cb, LUA_NOREF),
        None => LUA_NOREF,
    });
    if let Some(cb) = cb {
        release_luaref(cb);
    }
    channel_decref(id);
}

pub fn nvim_chan_send(chan: Integer, data: ApiString) -> Result<(), Error> {
    if data.is_empty() {
        return Ok(());
    }
    channel_send(chan.cast_unsigned(), data.as_bytes().to_vec()).map_err(Error::validation)?;
    Ok(())
}
