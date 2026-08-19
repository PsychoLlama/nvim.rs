//! The superseded Vimscript entry points.
//!
//! `nvim_exec`/`nvim_command_output` are `nvim_exec2` without its options
//! keyset, `nvim_execute_lua` is `nvim_exec_lua` under its old name, and
//! `nvim_call_atomic` is the batching call the RPC layer grew before
//! `nvim_exec_lua` made it unnecessary -- it dispatches an array of
//! [method, args] pairs and stops at the first error.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported, array_add};
use crate::api::vim::nvim_exec_lua;

pub unsafe fn nvim_exec(
    channel_id: uint64_t,
    src: String_0,
    output: Boolean,
) -> Result<String_0, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut opts: KeyDict_exec_opts = KeyDict_exec_opts { output: output };
        return exec_impl(channel_id, src, &raw mut opts, err).reported(error);
    }
}

pub unsafe fn nvim_command_output(
    channel_id: uint64_t,
    command: String_0,
) -> Result<String_0, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut opts: KeyDict_exec_opts = KeyDict_exec_opts { output: true };
        return exec_impl(channel_id, command, &raw mut opts, err).reported(error);
    }
}

pub unsafe fn nvim_execute_lua(
    code: String_0,
    args: Array,
    arena: *mut Arena,
) -> Result<Object, Error> {
    // The old name of `nvim_exec_lua`, and nothing else: the two had the same
    // transpiled body.
    unsafe { nvim_exec_lua(code, args, arena) }
}

pub unsafe fn nvim_call_atomic(
    channel_id: uint64_t,
    calls: Array,
    arena: *mut Arena,
) -> Result<Array, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut rv: Array = arena_array(arena, 2 as size_t);
        let mut results: Array = arena_array(arena, calls.size);
        let mut nested_error: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut i: size_t = 0;
        i = 0 as size_t;
        '_theend: {
            while i < calls.size {
                if kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                    != (*calls.items.add(i)).type_0 as ::core::ffi::c_uint
                {
                    api_err_exp(
                        err,
                        c"'calls' item".as_ptr(),
                        api_typename(kObjectTypeArray),
                        api_typename((*calls.items.add(i)).type_0),
                    );
                    break '_theend;
                }
                let mut call: Array = (*calls.items.add(i)).data.array;
                if !(call.size == 2 as size_t) {
                    api_err_exp(
                        err,
                        c"'calls' item".as_ptr(),
                        c"2-item Array".as_ptr(),
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                    break '_theend;
                } else if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                    != (*call.items.offset(0 as ::core::ffi::c_int as isize)).type_0
                        as ::core::ffi::c_uint
                {
                    api_err_exp(
                        err,
                        c"name".as_ptr(),
                        api_typename(kObjectTypeString),
                        api_typename((*call.items.offset(0 as ::core::ffi::c_int as isize)).type_0),
                    );
                    break '_theend;
                }
                let mut name: String_0 = (*call.items.offset(0 as ::core::ffi::c_int as isize))
                    .data
                    .string;
                if kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                    != (*call.items.offset(1 as ::core::ffi::c_int as isize)).type_0
                        as ::core::ffi::c_uint
                {
                    api_err_exp(
                        err,
                        c"call args".as_ptr(),
                        api_typename(kObjectTypeArray),
                        api_typename((*call.items.offset(1 as ::core::ffi::c_int as isize)).type_0),
                    );
                    break '_theend;
                }
                let mut args: Array = (*call.items.offset(1 as ::core::ffi::c_int as isize))
                    .data
                    .array;
                let mut handler: MsgpackRpcRequestHandler =
                    msgpack_rpc_get_handler_for(name.data(), name.len(), &raw mut nested_error);
                if nested_error.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                {
                    break;
                }
                let mut result: Object = handler.fn_0.expect("non-null function pointer")(
                    channel_id,
                    args,
                    arena,
                    &raw mut nested_error,
                );
                if nested_error.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                {
                    break;
                }
                array_add(&mut results, copy_object(result, arena));
                if handler.ret_alloc {
                    api_free_object(result);
                }
                i = i.wrapping_add(1);
            }
            array_add(&mut rv, Object::array(results));
            if nested_error.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                let mut errval: Array = arena_array(arena, 3 as size_t);
                array_add(&mut errval, Object::integer(i as Integer));
                array_add(&mut errval, Object::integer(nested_error.type_0 as Integer));
                array_add(
                    &mut errval,
                    Object::string(copy_string(cstr_as_string(nested_error.msg), arena)),
                );
                array_add(&mut rv, Object::array(errval));
            } else {
                array_add(&mut rv, Object::NIL);
            }
        }
        api_clear_error(&raw mut nested_error);
        return rv.reported(error);
    }
}
