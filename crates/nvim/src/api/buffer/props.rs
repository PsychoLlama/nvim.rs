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

use crate::winlayer::Buf;
pub unsafe fn nvim_buf_get_var(
    buf: Buffer,
    name: String_0,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return NIL.reported(error);
        }
        dict_get_value((*b).b_vars, name, arena, err).reported(error)
    }
}

pub unsafe fn nvim_buf_get_changedtick(buf: Buffer) -> Result<Integer, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let b: *const buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return (-1 as Integer).reported(error);
        }
        buf_get_changedtick(Buf::new(b.cast_mut())).reported(error)
    }
}

pub unsafe fn nvim_buf_get_keymap(
    buf: Buffer,
    mode: String_0,
    arena: *mut Arena,
) -> Result<Array, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            }
            .reported(error);
        }
        keymap_array(mode, Some(Buf::new(b)), arena).reported(error)
    }
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
    unsafe {
        modify_keymap(channel_id, buf, false, mode, lhs, rhs, opts, err);
    }
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
    unsafe {
        let mut rhs: String_0 =
            String_0::from_raw_parts(c"".as_ptr() as *mut ::core::ffi::c_char, 0 as size_t);
        modify_keymap(
            channel_id,
            buf,
            true,
            mode,
            lhs,
            rhs,
            ::core::ptr::null_mut::<KeyDict_keymap>(),
            err,
        );
    }
    ().reported(error)
}

pub unsafe fn nvim_buf_set_var(buf: Buffer, name: String_0, value: Object) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return ().reported(error);
        }
        dict_set_var(
            (*b).b_vars,
            name,
            value,
            false,
            false,
            ::core::ptr::null_mut::<Arena>(),
            err,
        );
    }
    ().reported(error)
}

pub unsafe fn nvim_buf_del_var(buf: Buffer, name: String_0) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return ().reported(error);
        }
        dict_set_var(
            (*b).b_vars,
            name,
            NIL,
            true,
            false,
            ::core::ptr::null_mut::<Arena>(),
            err,
        );
    }
    ().reported(error)
}

pub unsafe fn nvim_buf_get_name(buf: Buffer) -> Result<String_0, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut rv: String_0 =
            String_0::from_raw_parts(::core::ptr::null_mut::<::core::ffi::c_char>(), 0 as size_t);
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() || (*b).b_ffname.is_null() {
            return rv.reported(error);
        }
        cstr_as_string((*b).b_ffname).reported(error)
    }
}

pub unsafe fn nvim_buf_set_name(buf: Buffer, name: String_0) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
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
        try_enter(&raw mut tstate);
        let is_curbuf: bool = b == curbuf.get();
        let save_acd: ::core::ffi::c_int = p_acd.get();
        let redraw_off = (!is_curbuf).then(Suppress::redraw);
        if !is_curbuf {
            p_acd.set(0 as ::core::ffi::c_int);
        }
        let mut aco: aco_save_T = aco_save_T::default();
        aucmd_prepbuf(&raw mut aco, b);
        ren_ret = rename_buffer(name.data());
        aucmd_restbuf(&raw mut aco);
        drop(redraw_off);
        if !is_curbuf {
            p_acd.set(save_acd);
        }
        try_leave(&raw mut tstate, err);
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return ().reported(error);
        }
        if ren_ret == FAIL {
            api_set_error(
                err,
                kErrorTypeException,
                c"Failed to rename buffer".as_ptr(),
            );
        }
    }
    ().reported(error)
}

pub unsafe fn nvim_buf_is_loaded(buf: Buffer) -> Boolean {
    unsafe {
        let mut stub: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut b: *mut buf_T = find_buffer_by_handle(buf, &raw mut stub);
        api_clear_error(&raw mut stub);
        !b.is_null() && !(*b).b_ml.ml_mfp.is_null()
    }
}

pub unsafe fn nvim_buf_delete(buf: Buffer, opts: *mut KeyDict_buf_delete) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return ().reported(error);
        }
        let mut force: bool = (*opts).force;
        let mut unload: bool = (*opts).unload;
        let mut result: ::core::ffi::c_int = do_buffer(
            if unload as ::core::ffi::c_int != 0 {
                DOBUF_UNLOAD as ::core::ffi::c_int
            } else {
                DOBUF_WIPE as ::core::ffi::c_int
            },
            DOBUF_FIRST as ::core::ffi::c_int,
            FORWARD as ::core::ffi::c_int,
            (*b).handle as ::core::ffi::c_int,
            force as ::core::ffi::c_int,
        );
        if result == FAIL {
            api_set_error(
                err,
                kErrorTypeException,
                c"Failed to unload buffer.".as_ptr(),
            );
            return ().reported(error);
        }
    }
    ().reported(error)
}

pub unsafe fn nvim_buf_is_valid(buf: Buffer) -> Boolean {
    unsafe {
        let mut stub: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut ret: Boolean = !find_buffer_by_handle(buf, &raw mut stub).is_null();
        api_clear_error(&raw mut stub);
        ret
    }
}
