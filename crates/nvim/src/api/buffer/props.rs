//! Everything a buffer *is*, rather than what it holds.
//!
//! The buffer-local variables, the buffer-local mappings, the name, the
//! change tick, and the validity/loaded/delete trio.  All of them are one
//! handle lookup plus a call into the layer that owns the property, so they
//! share nothing but that shape.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::guard::Suppress;
use crate::types::Failed;

use crate::winlayer::Buf;

/// # Safety
///
/// `name` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`. `arena` must point at a live arena, which the memory this
/// answers with is taken from and must outlive.
pub unsafe fn nvim_buf_get_var(
    buf: BufferHandle,
    name: String_0,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut error = Error::none();
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(Object::Nil);
    };
    unsafe { dict_get_value(b.b_vars, name, arena, &mut error) }.reported(error)
}

pub fn nvim_buf_get_changedtick(buf: BufferHandle) -> Result<Integer, Error> {
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(-1 as Integer);
    };
    Ok(buf_get_changedtick(b))
}

/// # Safety
///
/// `mode` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`. `arena` must point at a live arena, which the memory this
/// answers with is taken from and must outlive.
pub unsafe fn nvim_buf_get_keymap(
    buf: BufferHandle,
    mode: String_0,
    arena: *mut Arena,
) -> Result<Array, Error> {
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        });
    };
    Ok(unsafe { keymap_array(mode, Some(b), arena) })
}

/// # Safety
///
/// `mode` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`. `lhs` must be a well-formed API string: `size` readable
/// bytes with a NUL at `data[size]`. `rhs` must be a well-formed API string:
/// `size` readable bytes with a NUL at `data[size]`. `opts` must point at the
/// `KeyDict_keymap` the dispatcher filled in, live for the call.
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

/// # Safety
///
/// `mode` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`. `lhs` must be a well-formed API string: `size` readable
/// bytes with a NUL at `data[size]`.
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

/// # Safety
///
/// `name` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`. `value` must be a well-formed API object the caller owns
/// for the call.
pub unsafe fn nvim_buf_set_var(
    buf: BufferHandle,
    name: String_0,
    value: Object,
) -> Result<(), Error> {
    let mut error = Error::none();
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(());
    };
    let vars = b.b_vars;
    let no_arena = ::core::ptr::null_mut::<Arena>();
    // SAFETY: `vars` is that buffer's variable dict, `error` our own slot.
    unsafe { dict_set_var(vars, name, value, false, false, no_arena, &mut error) };
    ().reported(error)
}

/// # Safety
///
/// `name` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`.
pub unsafe fn nvim_buf_del_var(buf: BufferHandle, name: String_0) -> Result<(), Error> {
    let mut error = Error::none();
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(());
    };
    let vars = b.b_vars;
    let no_arena = ::core::ptr::null_mut::<Arena>();
    // SAFETY: `vars` is that buffer's variable dict, `error` our own slot.
    unsafe { dict_set_var(vars, name, Object::Nil, true, false, no_arena, &mut error) };
    ().reported(error)
}

pub fn nvim_buf_get_name(buf: BufferHandle) -> Result<String_0, Error> {
    let rv: String_0 =
        String_0::from_raw_parts(::core::ptr::null_mut::<::core::ffi::c_char>(), 0 as size_t);
    let Some(b) = find_buffer_by_handle(buf)?.filter(|b| !b.b_ffname.is_null()) else {
        return Ok(rv);
    };
    Ok(unsafe { cstr_as_string(b.b_ffname) })
}

/// # Safety
///
/// `name` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`.
pub unsafe fn nvim_buf_set_name(buf: BufferHandle, name: String_0) -> Result<(), Error> {
    let mut error = Error::none();
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(());
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
    error.absorb(unsafe { try_leave(&raw mut tstate) });
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
    // A handle that names nothing is not an error here, only a `false`.
    let b = find_buffer_by_handle(buf).unwrap_or_default();
    b.is_some_and(|b| !b.b_ml.ml_mfp.is_null())
}

/// # Safety
///
/// `opts` must point at the `KeyDict_buf_delete` the dispatcher filled in,
/// live for the call.
pub unsafe fn nvim_buf_delete(
    buf: BufferHandle,
    opts: *mut KeyDict_buf_delete,
) -> Result<(), Error> {
    let mut error = Error::none();
    let b = find_buffer_by_handle(buf)?.expect("a resolved handle names a live buffer");
    let force: bool = unsafe { (*opts).force };
    let unload: bool = unsafe { (*opts).unload };
    let result: Result<(), Failed> = do_buffer(
        if ::core::ffi::c_int::from(unload) != 0 {
            DOBUF_UNLOAD.cast_signed()
        } else {
            DOBUF_WIPE.cast_signed()
        },
        DOBUF_FIRST.cast_signed(),
        FORWARD as ::core::ffi::c_int,
        b.handle as ::core::ffi::c_int,
        ::core::ffi::c_int::from(force),
    );
    if result.is_err() {
        let why = c"Failed to unload buffer.";
        error = Error::exception(why);
        return ().reported(error);
    }
    ().reported(error)
}

pub fn nvim_buf_is_valid(buf: BufferHandle) -> Boolean {
    // A handle that names nothing is not an error here, only a `false`.
    find_buffer_by_handle(buf).unwrap_or_default().is_some()
}
