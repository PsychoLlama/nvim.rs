//! `nvim_open_term()`: a terminal whose input is a Lua callback.
//!
//! The buffer is given a `Terminal` whose write/resize/close hooks
//! (`term_write`, `term_resize`, `term_close`) forward to the caller
//! instead of to a process, which is what makes a channel-backed or
//! purely virtual terminal possible.  `nvim_chan_send` is the other
//! direction, and shares nothing but the channel.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, NIL, Reported, array_add, has_key};
use crate::guard::Lock;
use crate::winlayer::Buf;

pub unsafe fn nvim_open_term(buf: Buffer, opts: *mut KeyDict_open_term) -> Result<Integer, Error> {
    let mut slot = ERROR_INIT;
    let err = &raw mut slot;
    let mut b: *mut buf_T = unsafe { api_buf_ensure_loaded(buf, err) };
    if b.is_null() {
        return (0 as Integer).reported(slot);
    }
    if b == cmdwin_buf.get() {
        unsafe {
            api_set_error(
                err,
                kErrorTypeException,
                c"%s".as_ptr(),
                &raw const e_cmdwin as *const ::core::ffi::c_char,
            )
        };
        return (0 as Integer).reported(slot);
    }
    let mut may_read_buffer: bool = true;
    if !unsafe { (*b).terminal }.is_null() {
        if unsafe { terminal_running((*b).terminal) } {
            unsafe {
                api_set_error(
                    err,
                    kErrorTypeException,
                    c"Terminal already connected to buffer %d".as_ptr(),
                    (*b).handle,
                )
            };
            return (0 as Integer).reported(slot);
        }
        buf_close_terminal(unsafe { Buf::new(b) });
        may_read_buffer = false;
    }
    let mut cb: LuaRef = LUA_NOREF;
    if has_key(
        unsafe { (*opts).is_set__open_term_ },
        KEYSET_OPTIDX_open_term__on_input,
    ) {
        cb = unsafe { (*opts).on_input };
        unsafe { (*opts).on_input = LUA_NOREF as LuaRef };
    }
    let mut chan: *mut Channel = unsafe { channel_alloc(kChannelStreamInternal) };
    unsafe { (*channel_internal(chan)).cb = cb };
    unsafe { (*channel_internal(chan)).closed = false };
    // SAFETY: `curwin` names a live window for the editor's whole run.
    let (view_width, view_height, col_off) = unsafe {
        (
            (*curwin.get()).w_view_width,
            (*curwin.get()).w_view_height,
            win_col_off(curwin.get()),
        )
    };
    let mut topts: TerminalOptions = TerminalOptions {
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
        force_crlf: if has_key(
            unsafe { (*opts).is_set__open_term_ },
            KEYSET_OPTIDX_open_term__force_crlf,
        ) {
            unsafe { (*opts).force_crlf as ::core::ffi::c_int }
        } else {
            1
        } != 0,
    };
    let mut contents: StringBuilder = StringBuilder {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    if may_read_buffer {
        unsafe { read_buffer_into(Buf::new(b), 1, (*b).b_ml.ml_line_count, &raw mut contents) };
    }
    unsafe { channel_incref(chan) };
    unsafe { (*chan).term = terminal_alloc(b, topts) };
    unsafe { terminal_open(&raw mut (*chan).term, b) };
    if !unsafe { (*chan).term }.is_null() {
        unsafe { terminal_check_size((*chan).term) };
    }
    unsafe { channel_decref(chan) };
    if contents.size > 0 as size_t {
        let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        unsafe {
            channel_send(
                (*chan).id,
                contents.items,
                contents.size,
                true,
                &raw mut error,
            )
        };
        if !error.is_null() {
            unsafe { api_set_error(err, kErrorTypeValidation, c"%s".as_ptr(), error) };
        }
    }
    (unsafe { (*chan).id } as Integer).reported(slot)
}

fn term_read_pause(mut _pause: bool, mut _data: *mut ::core::ffi::c_void) {}

unsafe fn term_write(
    mut buf: *const ::core::ffi::c_char,
    mut size: size_t,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut chan: *mut Channel = data as *mut Channel;
    let mut cb: LuaRef = unsafe { (*channel_internal(chan)).cb };
    if cb == LUA_NOREF {
        return;
    }
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 3] = [NIL; 3];
    args.capacity = 3 as size_t;
    args.items = &raw mut args__items as *mut Object;
    unsafe { array_add(&mut args, Object::integer((*chan).id as Integer)) };
    unsafe { array_add(&mut args, Object::buffer(terminal_buf((*chan).term))) };
    unsafe {
        array_add(
            &mut args,
            Object::string(String_0::from_raw_parts(
                buf as *mut ::core::ffi::c_char,
                size,
            )),
        )
    };
    let _locked = Lock::text();
    unsafe {
        nlua_call_ref(
            cb,
            c"input".as_ptr(),
            args,
            kRetNilBool,
            ::core::ptr::null_mut::<Arena>(),
            ::core::ptr::null_mut::<Error>(),
        )
    };
}

fn term_resize(mut _width: uint16_t, mut _height: uint16_t, mut _data: *mut ::core::ffi::c_void) {}

fn term_resume(mut _data: *mut ::core::ffi::c_void) {}

unsafe fn term_close(mut data: *mut ::core::ffi::c_void) {
    let mut chan: *mut Channel = data as *mut Channel;
    unsafe { terminal_destroy(&raw mut (*chan).term) };
    unsafe { api_free_luaref((*channel_internal(chan)).cb) };
    unsafe { (*channel_internal(chan)).cb = LUA_NOREF as LuaRef };
    unsafe { channel_decref(chan) };
}

pub unsafe fn nvim_chan_send(chan: Integer, data: String_0) -> Result<(), Error> {
    let mut slot = ERROR_INIT;
    let err = &raw mut slot;
    let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if data.is_empty() {
        return ().reported(slot);
    }
    unsafe {
        channel_send(
            chan as uint64_t,
            data.data(),
            data.len(),
            false,
            &raw mut error,
        )
    };
    if !error.is_null() {
        unsafe { api_set_error(err, kErrorTypeValidation, c"%s".as_ptr(), error) };
    }
    ().reported(slot)
}
