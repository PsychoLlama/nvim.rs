//! Everything a buffer *is*, rather than what it holds.
//!
//! The buffer-local variables, the buffer-local mappings, the name, the
//! change tick, and the validity/loaded/delete trio.  All of them are one
//! handle lookup plus a call into the layer that owns the property, so they
//! share nothing but that shape.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::guard::Suppress;
use crate::types::Failed;

use crate::winlayer::Buf;

pub unsafe fn nvim_buf_get_var(
    buf: BufferHandle,
    name: String_0,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut error = Error::none();
    let Some(b) = find_buffer_by_handle(buf, &mut error) else {
        return Object::Nil.reported(error);
    };
    unsafe { dict_get_value(b.b_vars, name, arena, &mut error) }.reported(error)
}

pub fn nvim_buf_get_changedtick(buf: BufferHandle) -> Result<Integer, Error> {
    let mut error = Error::none();
    let Some(b) = find_buffer_by_handle(buf, &mut error) else {
        return (-1 as Integer).reported(error);
    };
    buf_get_changedtick(b).reported(error)
}

pub unsafe fn nvim_buf_get_keymap(
    buf: BufferHandle,
    mode: String_0,
    arena: *mut Arena,
) -> Result<Array, Error> {
    let mut error = Error::none();
    let Some(b) = find_buffer_by_handle(buf, &mut error) else {
        return Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        }
        .reported(error);
    };
    unsafe { keymap_array(mode, Some(b), arena) }.reported(error)
}

pub unsafe fn nvim_buf_set_keymap(
    channel_id: uint64_t,
    buf: BufferHandle,
    mode: String_0,
    lhs: String_0,
    rhs: String_0,
    opts: *mut KeyDict_keymap,
) -> Result<(), Error> {
    let mut error = Error::none();
    unsafe { modify_keymap(channel_id, buf, false, mode, lhs, rhs, opts, &mut error) };
    ().reported(error)
}

pub unsafe fn nvim_buf_del_keymap(
    channel_id: uint64_t,
    buf: BufferHandle,
    mode: String_0,
    lhs: String_0,
) -> Result<(), Error> {
    let mut error = Error::none();
    let rhs: String_0 =
        String_0::from_raw_parts(c"".as_ptr() as *mut ::core::ffi::c_char, 0 as size_t);
    let no_opts = ::core::ptr::null_mut::<KeyDict_keymap>();
    // SAFETY: `error` is this call's own error slot; the mapping is deleted, so
    // it takes no options.
    unsafe { modify_keymap(channel_id, buf, true, mode, lhs, rhs, no_opts, &mut error) };
    ().reported(error)
}

pub unsafe fn nvim_buf_set_var(
    buf: BufferHandle,
    name: String_0,
    value: Object,
) -> Result<(), Error> {
    let mut error = Error::none();
    let Some(b) = find_buffer_by_handle(buf, &mut error) else {
        return ().reported(error);
    };
    let vars = b.b_vars;
    let no_arena = ::core::ptr::null_mut::<Arena>();
    // SAFETY: `vars` is that buffer's variable dict, `error` our own slot.
    unsafe { dict_set_var(vars, name, value, false, false, no_arena, &mut error) };
    ().reported(error)
}

pub unsafe fn nvim_buf_del_var(buf: BufferHandle, name: String_0) -> Result<(), Error> {
    let mut error = Error::none();
    let Some(b) = find_buffer_by_handle(buf, &mut error) else {
        return ().reported(error);
    };
    let vars = b.b_vars;
    let no_arena = ::core::ptr::null_mut::<Arena>();
    // SAFETY: `vars` is that buffer's variable dict, `error` our own slot.
    unsafe { dict_set_var(vars, name, Object::Nil, true, false, no_arena, &mut error) };
    ().reported(error)
}

pub unsafe fn nvim_buf_get_name(buf: BufferHandle) -> Result<String_0, Error> {
    let mut error = Error::none();
    let rv: String_0 =
        String_0::from_raw_parts(::core::ptr::null_mut::<::core::ffi::c_char>(), 0 as size_t);
    let Some(b) = find_buffer_by_handle(buf, &mut error).filter(|b| !b.b_ffname.is_null()) else {
        return rv.reported(error);
    };
    unsafe { cstr_as_string(b.b_ffname) }.reported(error)
}

pub unsafe fn nvim_buf_set_name(buf: BufferHandle, name: String_0) -> Result<(), Error> {
    let mut error = Error::none();
    let Some(b) = find_buffer_by_handle(buf, &mut error) else {
        return ().reported(error);
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
    let is_curbuf: bool = b == Buf::current();
    let save_acd: ::core::ffi::c_int = p_acd.get();
    let redraw_off = (!is_curbuf).then(Suppress::redraw);
    if !is_curbuf {
        p_acd.set(0 as ::core::ffi::c_int);
    }
    let mut aco: AcoSave = AcoSave::default();
    unsafe { aucmd_prepbuf(&raw mut aco, b) };
    let ren_ret = unsafe { rename_buffer(name.data()) };
    unsafe { aucmd_restbuf(&raw mut aco) };
    drop(redraw_off);
    if !is_curbuf {
        p_acd.set(save_acd);
    }
    unsafe { try_leave(&raw mut tstate, &mut error) };
    if error.kind() as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return ().reported(error);
    }
    if ren_ret.is_err() {
        let why = c"Failed to rename buffer";
        error = Error::exception(why);
    }
    ().reported(error)
}

pub fn nvim_buf_is_loaded(buf: BufferHandle) -> Boolean {
    let mut stub: Error = Error::none();
    let b = find_buffer_by_handle(buf, &mut stub);
    stub.clear();
    b.is_some_and(|b| !b.b_ml.ml_mfp.is_null())
}

pub unsafe fn nvim_buf_delete(
    buf: BufferHandle,
    opts: *mut KeyDict_buf_delete,
) -> Result<(), Error> {
    let mut error = Error::none();
    let b = find_buffer_by_handle(buf, &mut error);
    if error.kind() as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return ().reported(error);
    }
    let b = b.expect("an unset error means the handle named a live buffer");
    let force: bool = unsafe { (*opts).force };
    let unload: bool = unsafe { (*opts).unload };
    let result: Result<(), Failed> = do_buffer(
        if unload as ::core::ffi::c_int != 0 {
            DOBUF_UNLOAD as ::core::ffi::c_int
        } else {
            DOBUF_WIPE as ::core::ffi::c_int
        },
        DOBUF_FIRST as ::core::ffi::c_int,
        FORWARD as ::core::ffi::c_int,
        b.handle as ::core::ffi::c_int,
        force as ::core::ffi::c_int,
    );
    if result.is_err() {
        let why = c"Failed to unload buffer.";
        error = Error::exception(why);
        return ().reported(error);
    }
    ().reported(error)
}

pub fn nvim_buf_is_valid(buf: BufferHandle) -> Boolean {
    let mut stub: Error = Error::none();
    let ret: Boolean = find_buffer_by_handle(buf, &mut stub).is_some();
    stub.clear();
    ret
}
