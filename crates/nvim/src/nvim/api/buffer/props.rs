//! Everything a buffer *is*, rather than what it holds.
//!
//! The buffer-local variables, the buffer-local mappings, the name, the
//! change tick, and the validity/loaded/delete trio.  All of them are one
//! handle lookup plus a call into the layer that owns the property, so they
//! share nothing but that shape.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn nvim_buf_get_var(
    mut buf: Buffer,
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
        }
        return dict_get_value((*b).b_vars, name, arena, err);
    }
}

pub unsafe extern "C" fn nvim_buf_get_changedtick(mut buf: Buffer, mut err: *mut Error) -> Integer {
    unsafe {
        let b: *const buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return -1 as Integer;
        }
        return buf_get_changedtick(b);
    }
}

pub unsafe extern "C" fn nvim_buf_get_keymap(
    mut buf: Buffer,
    mut mode: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            };
        }
        return keymap_array(mode, b, arena);
    }
}

pub unsafe extern "C" fn nvim_buf_set_keymap(
    mut channel_id: uint64_t,
    mut buf: Buffer,
    mut mode: String_0,
    mut lhs: String_0,
    mut rhs: String_0,
    mut opts: *mut KeyDict_keymap,
    mut err: *mut Error,
) {
    unsafe {
        modify_keymap(channel_id, buf, false_0 != 0, mode, lhs, rhs, opts, err);
    }
}

pub unsafe extern "C" fn nvim_buf_del_keymap(
    mut channel_id: uint64_t,
    mut buf: Buffer,
    mut mode: String_0,
    mut lhs: String_0,
    mut err: *mut Error,
) {
    unsafe {
        let mut rhs: String_0 = String_0 {
            data: c"".as_ptr() as *mut ::core::ffi::c_char,
            size: 0 as size_t,
        };
        modify_keymap(
            channel_id,
            buf,
            true_0 != 0,
            mode,
            lhs,
            rhs,
            ::core::ptr::null_mut::<KeyDict_keymap>(),
            err,
        );
    }
}

pub unsafe extern "C" fn nvim_buf_set_var(
    mut buf: Buffer,
    mut name: String_0,
    mut value: Object,
    mut err: *mut Error,
) {
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return;
        }
        dict_set_var(
            (*b).b_vars,
            name,
            value,
            false_0 != 0,
            false_0 != 0,
            ::core::ptr::null_mut::<Arena>(),
            err,
        );
    }
}

pub unsafe extern "C" fn nvim_buf_del_var(
    mut buf: Buffer,
    mut name: String_0,
    mut err: *mut Error,
) {
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return;
        }
        dict_set_var(
            (*b).b_vars,
            name,
            object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            true_0 != 0,
            false_0 != 0,
            ::core::ptr::null_mut::<Arena>(),
            err,
        );
    }
}

pub unsafe extern "C" fn nvim_buf_get_name(mut buf: Buffer, mut err: *mut Error) -> String_0 {
    unsafe {
        let mut rv: String_0 = String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0 as size_t,
        };
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() || (*b).b_ffname.is_null() {
            return rv;
        }
        return cstr_as_string((*b).b_ffname);
    }
}

pub unsafe extern "C" fn nvim_buf_set_name(
    mut buf: Buffer,
    mut name: String_0,
    mut err: *mut Error,
) {
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return;
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
        if !is_curbuf {
            (*RedrawingDisabled.ptr()) += 1;
            p_acd.set(0 as ::core::ffi::c_int);
        }
        let mut aco: aco_save_T = aco_save_T::default();
        aucmd_prepbuf(&raw mut aco, b);
        ren_ret = rename_buffer(name.data);
        aucmd_restbuf(&raw mut aco);
        if !is_curbuf {
            (*RedrawingDisabled.ptr()) -= 1;
            p_acd.set(save_acd);
        }
        try_leave(&raw mut tstate, err);
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return;
        }
        if ren_ret == FAIL {
            api_set_error(
                err,
                kErrorTypeException,
                c"Failed to rename buffer".as_ptr(),
            );
        }
    }
}

pub unsafe extern "C" fn nvim_buf_is_loaded(mut buf: Buffer) -> Boolean {
    unsafe {
        let mut stub: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut b: *mut buf_T = find_buffer_by_handle(buf, &raw mut stub);
        api_clear_error(&raw mut stub);
        return !b.is_null() && !(*b).b_ml.ml_mfp.is_null();
    }
}

pub unsafe extern "C" fn nvim_buf_delete(
    mut buf: Buffer,
    mut opts: *mut KeyDict_buf_delete,
    mut err: *mut Error,
) {
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return;
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
            return;
        }
    }
}

pub unsafe extern "C" fn nvim_buf_is_valid(mut buf: Buffer) -> Boolean {
    unsafe {
        let mut stub: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut ret: Boolean = !find_buffer_by_handle(buf, &raw mut stub).is_null();
        api_clear_error(&raw mut stub);
        return ret;
    }
}
