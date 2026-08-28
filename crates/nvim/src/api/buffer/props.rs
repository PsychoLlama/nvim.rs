//! Everything a buffer *is*, rather than what it holds.
//!
//! The buffer-local variables, the buffer-local mappings, the name, the
//! change tick, and the validity/loaded/delete trio.  All of them are one
//! handle lookup plus a call into the layer that owns the property, so they
//! share nothing but that shape.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, NIL, Reported};
use crate::guard::Suppress;
use crate::types::{FAIL, OK};
use core::ffi::CStr;

use crate::winlayer::Buf;

/// An exception whose whole message is `why`.
///
/// # Safety
/// `err` must be the caller's error slot, and `why` must hold no `%`
/// directive: upstream passes it as the format itself.
unsafe fn err_exception(err: *mut Error, why: &CStr) {
    // SAFETY: the caller's promise.
    unsafe { api_set_error(err, kErrorTypeException, why.as_ptr()) };
}

pub unsafe fn nvim_buf_get_var(
    buf: Buffer,
    name: String_0,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut b: *mut buf_T = unsafe { find_buffer_by_handle(buf, err) };
    if b.is_null() {
        return NIL.reported(error);
    }
    unsafe { dict_get_value((*b).b_vars, name, arena, err) }.reported(error)
}

pub unsafe fn nvim_buf_get_changedtick(buf: Buffer) -> Result<Integer, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let b: *const buf_T = unsafe { find_buffer_by_handle(buf, err) };
    if b.is_null() {
        return (-1 as Integer).reported(error);
    }
    buf_get_changedtick(unsafe { Buf::new(b.cast_mut()) }).reported(error)
}

pub unsafe fn nvim_buf_get_keymap(
    buf: Buffer,
    mode: String_0,
    arena: *mut Arena,
) -> Result<Array, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut b: *mut buf_T = unsafe { find_buffer_by_handle(buf, err) };
    if b.is_null() {
        return Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        }
        .reported(error);
    }
    unsafe { keymap_array(mode, Some(Buf::new(b)), arena) }.reported(error)
}

pub unsafe fn nvim_buf_set_keymap(
    channel_id: uint64_t,
    buf: Buffer,
    mode: String_0,
    lhs: String_0,
    rhs: String_0,
    opts: *mut KeyDict_keymap,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe { modify_keymap(channel_id, buf, false, mode, lhs, rhs, opts, err) };
    ().reported(error)
}

pub unsafe fn nvim_buf_del_keymap(
    channel_id: uint64_t,
    buf: Buffer,
    mode: String_0,
    lhs: String_0,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut rhs: String_0 =
        String_0::from_raw_parts(c"".as_ptr() as *mut ::core::ffi::c_char, 0 as size_t);
    let no_opts = ::core::ptr::null_mut::<KeyDict_keymap>();
    // SAFETY: `err` is this call's own error slot; the mapping is deleted, so
    // it takes no options.
    unsafe { modify_keymap(channel_id, buf, true, mode, lhs, rhs, no_opts, err) };
    ().reported(error)
}

pub unsafe fn nvim_buf_set_var(buf: Buffer, name: String_0, value: Object) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut b: *mut buf_T = unsafe { find_buffer_by_handle(buf, err) };
    if b.is_null() {
        return ().reported(error);
    }
    // SAFETY: non-null, so the handle named a live buffer.
    let vars = unsafe { (*b).b_vars };
    let no_arena = ::core::ptr::null_mut::<Arena>();
    // SAFETY: `vars` is that buffer's variable dict, `err` our own slot.
    unsafe { dict_set_var(vars, name, value, false, false, no_arena, err) };
    ().reported(error)
}

pub unsafe fn nvim_buf_del_var(buf: Buffer, name: String_0) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut b: *mut buf_T = unsafe { find_buffer_by_handle(buf, err) };
    if b.is_null() {
        return ().reported(error);
    }
    // SAFETY: non-null, so the handle named a live buffer.
    let vars = unsafe { (*b).b_vars };
    let no_arena = ::core::ptr::null_mut::<Arena>();
    // SAFETY: `vars` is that buffer's variable dict, `err` our own slot.
    unsafe { dict_set_var(vars, name, NIL, true, false, no_arena, err) };
    ().reported(error)
}

pub unsafe fn nvim_buf_get_name(buf: Buffer) -> Result<String_0, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut rv: String_0 =
        String_0::from_raw_parts(::core::ptr::null_mut::<::core::ffi::c_char>(), 0 as size_t);
    let mut b: *mut buf_T = unsafe { find_buffer_by_handle(buf, err) };
    if b.is_null() || unsafe { (*b).b_ffname }.is_null() {
        return rv.reported(error);
    }
    unsafe { cstr_as_string((*b).b_ffname) }.reported(error)
}

pub unsafe fn nvim_buf_set_name(buf: Buffer, name: String_0) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut b: *mut buf_T = unsafe { find_buffer_by_handle(buf, err) };
    if b.is_null() {
        return ().reported(error);
    }
    let mut ren_ret: ::core::ffi::c_int = OK;
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
    let is_curbuf: bool = b == curbuf.get();
    let save_acd: ::core::ffi::c_int = p_acd.get();
    let redraw_off = (!is_curbuf).then(Suppress::redraw);
    if !is_curbuf {
        p_acd.set(0 as ::core::ffi::c_int);
    }
    let mut aco: aco_save_T = aco_save_T::default();
    unsafe { aucmd_prepbuf(&raw mut aco, b) };
    ren_ret = unsafe { rename_buffer(name.data()) };
    unsafe { aucmd_restbuf(&raw mut aco) };
    drop(redraw_off);
    if !is_curbuf {
        p_acd.set(save_acd);
    }
    unsafe { try_leave(&raw mut tstate, err) };
    if unsafe { (*err).type_0 } as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return ().reported(error);
    }
    if ren_ret == FAIL {
        let why = c"Failed to rename buffer";
        // SAFETY: `err` is this call's own error slot; the
        // message holds no `%` directive.
        unsafe { err_exception(err, why) };
    }
    ().reported(error)
}

pub unsafe fn nvim_buf_is_loaded(buf: Buffer) -> Boolean {
    let mut stub: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut b: *mut buf_T = unsafe { find_buffer_by_handle(buf, &raw mut stub) };
    unsafe { api_clear_error(&raw mut stub) };
    !b.is_null() && !unsafe { (*b).b_ml.ml_mfp }.is_null()
}

pub unsafe fn nvim_buf_delete(buf: Buffer, opts: *mut KeyDict_buf_delete) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut b: *mut buf_T = unsafe { find_buffer_by_handle(buf, err) };
    if unsafe { (*err).type_0 } as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return ().reported(error);
    }
    let mut force: bool = unsafe { (*opts).force };
    let mut unload: bool = unsafe { (*opts).unload };
    let mut result: ::core::ffi::c_int = do_buffer(
        if unload as ::core::ffi::c_int != 0 {
            DOBUF_UNLOAD as ::core::ffi::c_int
        } else {
            DOBUF_WIPE as ::core::ffi::c_int
        },
        DOBUF_FIRST as ::core::ffi::c_int,
        FORWARD as ::core::ffi::c_int,
        unsafe { (*b).handle } as ::core::ffi::c_int,
        force as ::core::ffi::c_int,
    );
    if result == FAIL {
        let why = c"Failed to unload buffer.";
        // SAFETY: `err` is this call's own error slot; the
        // message holds no `%` directive.
        unsafe { err_exception(err, why) };
        return ().reported(error);
    }
    ().reported(error)
}

pub unsafe fn nvim_buf_is_valid(buf: Buffer) -> Boolean {
    let mut stub: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut ret: Boolean = !unsafe { find_buffer_by_handle(buf, &raw mut stub) }.is_null();
    unsafe { api_clear_error(&raw mut stub) };
    ret
}
