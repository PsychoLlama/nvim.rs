//! `nvim_open_term()`: a terminal whose input is a Lua callback.
//!
//! The buffer is given a `Terminal` whose write/resize/close hooks
//! (`term_write`, `term_resize`, `term_close`) forward to the caller
//! instead of to a process, which is what makes a channel-backed or
//! purely virtual terminal possible.  `nvim_chan_send` is the other
//! direction, and shares nothing but the channel.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::api_error;
use crate::cstr;
use crate::guard::Lock;
use crate::lua::executor::nlua_call_ref_quiet;
use crate::winlayer::Win;

/// # Safety
///
/// `opts` must point at the `KeyDict_open_term` the dispatcher filled in,
/// live for the call.
pub unsafe fn nvim_open_term(
    buf: BufferHandle,
    opts: *mut KeyDict_open_term,
) -> Result<Integer, Error> {
    let mut slot = Error::none();
    let Some(buffer) = api_buf_ensure_loaded(buf)? else {
        return Ok(0 as Integer);
    };
    if cmdwin_buf.get() == Some(buffer.id()) {
        let msg = e_cmdwin.as_ptr();
        // SAFETY: the message the caller handed over, live for this call.
        slot = Error::from_message(kErrorTypeException, unsafe { cstr::at(msg) });
        return (0 as Integer).reported(slot);
    }
    let mut may_read_buffer: bool = true;
    if !buffer.terminal.is_null() {
        if unsafe { terminal_running(buffer.terminal) } {
            let handle = buffer.handle;
            slot = api_error!(
                kErrorTypeException,
                "Terminal already connected to buffer {handle}"
            );
            return (0 as Integer).reported(slot);
        }
        buf_close_terminal(buffer);
        may_read_buffer = false;
    }
    // The channel takes the callback's reference over, so the keyset must
    // not release it too.
    let cb: LuaRef = unsafe { (*opts).on_input.take() }.unwrap_or(LUA_NOREF);
    let chan: *mut Channel = unsafe { channel_alloc(kChannelStreamInternal) };
    unsafe { (*channel_internal(chan)).cb = cb };
    unsafe { (*channel_internal(chan)).closed = false };
    let (view_width, view_height, col_off) = (
        Win::current().w_view_width,
        Win::current().w_view_height,
        Win::current().col_off(),
    );
    let topts: TerminalOptions = TerminalOptions {
        data: chan as *mut ::core::ffi::c_void,
        width: (view_width - col_off).max(0) as uint16_t,
        height: view_height as uint16_t,
        read_pause_cb: Some(term_read_pause as unsafe fn(bool, *mut ::core::ffi::c_void) -> ()),
        write_cb: Some(
            term_write
                as unsafe fn(*const ::core::ffi::c_char, size_t, *mut ::core::ffi::c_void) -> (),
        ),
        resize_cb: Some(
            term_resize as unsafe fn(uint16_t, uint16_t, *mut ::core::ffi::c_void) -> (),
        ),
        resume_cb: Some(term_resume as unsafe fn(*mut ::core::ffi::c_void) -> ()),
        close_cb: Some(term_close as unsafe fn(*mut ::core::ffi::c_void) -> ()),
        force_crlf: unsafe { (*opts).force_crlf }.unwrap_or(true),
    };
    let mut contents: StringBuilder = StringBuilder {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    if may_read_buffer {
        unsafe { read_buffer_into(buffer, 1, buffer.b_ml.ml_line_count, &raw mut contents) };
    }
    unsafe { channel_incref(chan) };
    unsafe { (*chan).term = terminal_alloc(buffer, topts) };
    unsafe { terminal_open(&raw mut (*chan).term, buffer) };
    if !unsafe { (*chan).term }.is_null() {
        unsafe { terminal_check_size((*chan).term) };
    }
    unsafe { channel_decref(chan) };
    if contents.size > 0 as size_t {
        let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let (text, len, out) = (contents.items, contents.size, &raw mut error);
        // SAFETY: `chan` is the channel just made, `contents` this frame's
        // own lines, and `error` its own out-parameter.
        unsafe { channel_send((*chan).id, text, len, true, out) };
        if !error.is_null() {
            // SAFETY: `channel_send` left a NUL-terminated message there.
            slot = Error::from_message(kErrorTypeValidation, unsafe { cstr::at(error) });
        }
    }
    (unsafe { (*chan).id } as Integer).reported(slot)
}

fn term_read_pause(mut _pause: bool, mut _data: *mut ::core::ffi::c_void) {}

/// # Safety
///
/// `buf` must point at `size` readable bytes and `data` at the `Channel` this
/// terminal was opened for; libvterm's callback contract.
unsafe fn term_write(
    buf: *const ::core::ffi::c_char,
    size: size_t,
    data: *mut ::core::ffi::c_void,
) {
    let chan: *mut Channel = data as *mut Channel;
    let cb: LuaRef = unsafe { (*channel_internal(chan)).cb };
    if cb == LUA_NOREF {
        return;
    }
    let mut args: Array = Array::with_capacity(3);
    // SAFETY: `buf` holds `size` readable bytes, and `chan` is the
    // terminal's channel.
    unsafe {
        let text = String_0::from_bytes(core::slice::from_raw_parts(buf.cast::<u8>(), size));
        args.push(Object::integer((*chan).id as Integer));
        args.push(Object::buffer(terminal_buf((*chan).term)));
        args.push(Object::string(text));
    }
    let _locked = Lock::text();
    let (name, no_arena) = (c"input".as_ptr(), ::core::ptr::null_mut::<Arena>());

    // SAFETY: `cb` is a live Lua reference and `args` this frame's own; the
    // handler reports nothing, so it is given no error slot.
    unsafe { nlua_call_ref_quiet(cb, name, args, kRetNilBool, no_arena) };
}

fn term_resize(mut _width: uint16_t, mut _height: uint16_t, mut _data: *mut ::core::ffi::c_void) {}

fn term_resume(mut _data: *mut ::core::ffi::c_void) {}

/// # Safety
///
/// `data` must point at the `Channel` this terminal was opened for;
/// libvterm's callback contract.
unsafe fn term_close(data: *mut ::core::ffi::c_void) {
    let chan: *mut Channel = data as *mut Channel;
    unsafe { terminal_destroy(&raw mut (*chan).term) };
    unsafe { api_free_luaref((*channel_internal(chan)).cb) };
    unsafe { (*channel_internal(chan)).cb = LUA_NOREF as LuaRef };
    unsafe { channel_decref(chan) };
}

/// # Safety
///
/// `data` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`.
pub unsafe fn nvim_chan_send(chan: Integer, data: String_0) -> Result<(), Error> {
    let mut slot = Error::none();
    let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if data.is_empty() {
        return ().reported(slot);
    }
    let (id, text, len) = (chan as uint64_t, data.data(), data.len());
    // SAFETY: `data` is the caller's, and `error` this frame's own
    // out-parameter.
    unsafe { channel_send(id, text, len, false, &raw mut error) };
    if !error.is_null() {
        // SAFETY: `channel_send` left a NUL-terminated message there.
        slot = Error::from_message(kErrorTypeValidation, unsafe { cstr::at(error) });
    }
    ().reported(slot)
}
