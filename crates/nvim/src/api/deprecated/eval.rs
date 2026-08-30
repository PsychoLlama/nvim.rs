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
use crate::api::private::validate::err_expected_ptr;
use crate::api::vim::nvim_exec_lua;
use crate::api::vimscript::exec_impl;

pub unsafe fn nvim_exec(
    channel_id: uint64_t,
    src: String_0,
    output: Boolean,
) -> Result<String_0, Error> {
    let mut error = ERROR_INIT;
    let mut opts = KeyDict_exec_opts { output };
    // SAFETY: `src` is the caller's and `opts`/`error` are this frame's.
    unsafe { exec_impl(channel_id, src, &raw mut opts, &mut error) }.reported(error)
}

pub unsafe fn nvim_command_output(
    channel_id: uint64_t,
    command: String_0,
) -> Result<String_0, Error> {
    let mut error = ERROR_INIT;
    let mut opts = KeyDict_exec_opts { output: true };
    // SAFETY: as `nvim_exec`.
    unsafe { exec_impl(channel_id, command, &raw mut opts, &mut error) }.reported(error)
}

pub unsafe fn nvim_execute_lua(
    code: String_0,
    args: Array,
    arena: *mut Arena,
) -> Result<Object, Error> {
    // The old name of `nvim_exec_lua`, and nothing else: the two had the same
    // transpiled body.
    // SAFETY: every argument is the caller's, live for the call.
    unsafe { nvim_exec_lua(code, args, arena) }
}

pub unsafe fn nvim_call_atomic(
    channel_id: uint64_t,
    calls: Array,
    arena: *mut Arena,
) -> Result<Array, Error> {
    let mut error = ERROR_INIT;
    // "results" and the error report, and one result per call.
    let mut rv: Array = arena_array(arena, 2 as size_t);
    let mut results: Array = arena_array(arena, calls.size);
    let mut nested_error = ERROR_INIT;
    let mut i: size_t = 0;
    // A call that is not a well-formed [name, args] pair is the caller's
    // mistake rather than a failed call, so it is reported through `err` and
    // answers nothing at all -- not even the results already collected.
    '_theend: {
        while i < calls.size {
            // SAFETY: `i` is below `size`, so the item is inside `items`.
            let item = unsafe { *calls.items.add(i) };
            let Some(call) = item.as_array() else {
                let (want, got) = (api_typename(kObjectTypeArray), api_typename(item.type_0));
                // SAFETY: `err` is this frame's slot and the names are
                // `api_typename`'s own statics.
                error = unsafe { err_expected_ptr(c"'calls' item".as_ptr(), want, Some(got)) };
                break '_theend;
            };
            if call.size != 2 as size_t {
                let want = c"2-item Array";
                // SAFETY: as above.
                error = unsafe { err_expected_ptr(c"'calls' item".as_ptr(), want, None) };
                break '_theend;
            }
            // SAFETY: the pair has both of its items.
            let (head, tail) = unsafe { (*call.items, *call.items.add(1)) };
            let Some(name) = head.as_string() else {
                let (want, got) = (api_typename(kObjectTypeString), api_typename(head.type_0));
                // SAFETY: as above.
                error = unsafe { err_expected_ptr(c"name".as_ptr(), want, Some(got)) };
                break '_theend;
            };
            let Some(args) = tail.as_array() else {
                let (want, got) = (api_typename(kObjectTypeArray), api_typename(tail.type_0));
                // SAFETY: as above.
                error = unsafe { err_expected_ptr(c"call args".as_ptr(), want, Some(got)) };
                break '_theend;
            };

            // A call that *fails*, on the other hand, stops the batch and is
            // reported alongside the results that did come back.
            // SAFETY: `name` names its own bytes and `nested_error` is this
            // frame's slot.
            let handler: MsgpackRpcRequestHandler =
                unsafe { msgpack_rpc_get_handler_for(name.data(), name.len(), &mut nested_error) };
            if nested_error.is_set() {
                break;
            }
            let dispatch = handler.fn_0.expect("non-null function pointer");
            // SAFETY: the handler is the generated wrapper for `name`, which
            // reads `args` and reports through the slot it is given.
            let result = unsafe { dispatch(channel_id, args, arena, &mut nested_error) };
            if nested_error.is_set() {
                break;
            }
            // SAFETY: `results` was sized for one item per call, and the
            // copy is the arena's rather than the handler's.
            unsafe { array_add(&mut results, copy_object(result, arena)) };
            if handler.ret_alloc {
                // SAFETY: the handler allocated its answer, so freeing it is
                // this call's job.
                unsafe { api_free_object(result) };
            }
            i = i.wrapping_add(1);
        }
        // SAFETY: `rv` was sized for exactly these two pushes.
        unsafe { array_add(&mut rv, Object::array(results)) };
        if nested_error.is_set() {
            let mut errval: Array = arena_array(arena, 3 as size_t);
            // SAFETY: `errval` was sized for these three, and the message is
            // `nested_error`'s own NUL-terminated string.
            unsafe {
                array_add(&mut errval, Object::integer(i as Integer));
                array_add(&mut errval, Object::integer(nested_error.kind() as Integer));
                let why = nested_error.message_or_empty().as_ptr();
                let msg = copy_string(cstr_as_string(why), arena);
                array_add(&mut errval, Object::string(msg));
                array_add(&mut rv, Object::array(errval));
            }
        } else {
            // SAFETY: as above.
            unsafe { array_add(&mut rv, Object::NIL) };
        }
    }
    // SAFETY: `nested_error` is this frame's slot.
    nested_error.clear();
    rv.reported(error)
}
