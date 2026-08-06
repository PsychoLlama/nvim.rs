//! The superseded Vimscript entry points.
//!
//! `nvim_exec`/`nvim_command_output` are `nvim_exec2` without its options
//! keyset, `nvim_execute_lua` is `nvim_exec_lua` under its old name, and
//! `nvim_call_atomic` is the batching call the RPC layer grew before
//! `nvim_exec_lua` made it unnecessary -- it dispatches an array of
//! [method, args] pairs and stops at the first error.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn nvim_exec(
    mut channel_id: uint64_t,
    mut src: String_0,
    mut output: Boolean,
    mut err: *mut Error,
) -> String_0 {
    unsafe {
        let mut opts: KeyDict_exec_opts = KeyDict_exec_opts { output: output };
        return exec_impl(channel_id, src, &raw mut opts, err);
    }
}

pub unsafe extern "C" fn nvim_command_output(
    mut channel_id: uint64_t,
    mut command: String_0,
    mut err: *mut Error,
) -> String_0 {
    unsafe {
        let mut opts: KeyDict_exec_opts = KeyDict_exec_opts { output: true };
        return exec_impl(channel_id, command, &raw mut opts, err);
    }
}

pub unsafe extern "C" fn nvim_execute_lua(
    mut code: String_0,
    mut args: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    unsafe {
        return nlua_exec(
            code,
            ::core::ptr::null::<::core::ffi::c_char>(),
            args,
            kRetObject,
            arena,
            err,
        );
    }
}

pub unsafe extern "C" fn nvim_call_atomic(
    mut channel_id: uint64_t,
    mut calls: Array,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
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
                    != (*calls.items.offset(i as isize)).type_0 as ::core::ffi::c_uint
                {
                    api_err_exp(
                        err,
                        c"'calls' item".as_ptr(),
                        api_typename(kObjectTypeArray),
                        api_typename((*calls.items.offset(i as isize)).type_0),
                    );
                    break '_theend;
                }
                let mut call: Array = (*calls.items.offset(i as isize)).data.array;
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
                    msgpack_rpc_get_handler_for(name.data, name.size, &raw mut nested_error);
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
                let c2rust_fresh0 = results.size;
                results.size = results.size.wrapping_add(1);
                *results.items.offset(c2rust_fresh0 as isize) = copy_object(result, arena);
                if handler.ret_alloc {
                    api_free_object(result);
                }
                i = i.wrapping_add(1);
            }
            let c2rust_fresh1 = rv.size;
            rv.size = rv.size.wrapping_add(1);
            *rv.items.offset(c2rust_fresh1 as isize) = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed { array: results },
            };
            if nested_error.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                let mut errval: Array = arena_array(arena, 3 as size_t);
                let c2rust_fresh2 = errval.size;
                errval.size = errval.size.wrapping_add(1);
                *errval.items.offset(c2rust_fresh2 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: i as Integer,
                    },
                };
                let c2rust_fresh3 = errval.size;
                errval.size = errval.size.wrapping_add(1);
                *errval.items.offset(c2rust_fresh3 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: nested_error.type_0 as Integer,
                    },
                };
                let c2rust_fresh4 = errval.size;
                errval.size = errval.size.wrapping_add(1);
                *errval.items.offset(c2rust_fresh4 as isize) = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: copy_string(cstr_as_string(nested_error.msg), arena),
                    },
                };
                let c2rust_fresh5 = rv.size;
                rv.size = rv.size.wrapping_add(1);
                *rv.items.offset(c2rust_fresh5 as isize) = object {
                    type_0: kObjectTypeArray,
                    data: C2Rust_Unnamed { array: errval },
                };
            } else {
                let c2rust_fresh6 = rv.size;
                rv.size = rv.size.wrapping_add(1);
                *rv.items.offset(c2rust_fresh6 as isize) = object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                };
            }
        }
        api_clear_error(&raw mut nested_error);
        return rv;
    }
}
