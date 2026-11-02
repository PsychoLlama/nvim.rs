//! The superseded Vimscript entry points.
//!
//! `nvim_exec`/`nvim_command_output` are `nvim_exec2` without its options
//! keyset, `nvim_execute_lua` is `nvim_exec_lua` under its old name, and
//! `nvim_call_atomic` is the batching call the RPC layer grew before
//! `nvim_exec_lua` made it unnecessary -- it dispatches an array of
//! [method, args] pairs and stops at the first error.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api::private::validate::err_expected;
use crate::api::vim::nvim_exec_lua;
use crate::api::vimscript::exec_impl;

pub fn nvim_exec(
    channel_id: uint64_t,
    src: ApiString,
    output: Boolean,
) -> Result<ApiString, Error> {
    let mut opts = KeyDict_exec_opts {
        output: Some(output),
    };
    exec_impl(channel_id, src, &mut opts)
}

pub fn nvim_command_output(channel_id: uint64_t, command: ApiString) -> Result<ApiString, Error> {
    let mut opts = KeyDict_exec_opts { output: Some(true) };
    exec_impl(channel_id, command, &mut opts)
}

/// The old name of [`nvim_exec_lua`], and nothing else: the two had the same
/// transpiled body.
pub fn nvim_execute_lua(code: ApiString, args: Array) -> Result<Object, Error> {
    nvim_exec_lua(code, args)
}

/// Run each `[name, args]` pair of `calls` through the dispatcher, stopping
/// at the first that fails.
pub fn nvim_call_atomic(channel_id: uint64_t, calls: Array) -> Result<Array, Error> {
    // "results" and the error report, and one result per call.
    let mut rv = Vec::with_capacity(2 as size_t);
    let mut results = Vec::with_capacity(calls.len());
    let mut nested_error: Option<Error> = None;
    let mut i: size_t = 0;
    // A call that is not a well-formed [name, args] pair is the caller's
    // mistake rather than a failed call, so it refuses the whole batch and
    // answers nothing at all -- not even the results already collected.
    while i < calls.len() {
        let item = &calls[i];
        let Some(call) = item.as_array() else {
            let (want, got) = (api_typename(kObjectTypeArray), api_typename(item.kind()));
            return Err(err_expected(c"'calls' item", want, Some(got)));
        };
        if call.len() != 2 as size_t {
            let want = c"2-item Array";
            return Err(err_expected(c"'calls' item", want, None));
        }
        // The length check above says the pair has both of its items.
        let (head, tail) = (&call[0], &call[1]);
        let Some(name) = head.as_string() else {
            let (want, got) = (api_typename(kObjectTypeString), api_typename(head.kind()));
            return Err(err_expected(c"name", want, Some(got)));
        };
        let Some(args) = tail.as_array() else {
            let (want, got) = (api_typename(kObjectTypeArray), api_typename(tail.kind()));
            return Err(err_expected(c"call args", want, Some(got)));
        };

        // A call that *fails*, on the other hand, stops the batch and is
        // reported alongside the results that did come back.
        let handler: MsgpackRpcRequestHandler = match msgpack_rpc_get_handler_for(name.as_bytes()) {
            Ok(handler) => handler,
            Err(e) => {
                nested_error = Some(e);
                break;
            }
        };
        let dispatch = handler.fn_0.expect("non-null function pointer");
        let result = match dispatch(channel_id, args.clone()) {
            Ok(rv) => rv,
            Err(e) => {
                nested_error = Some(e);
                break;
            }
        };
        // `results` was sized for one item per call.
        results.push(result);
        i = i.wrapping_add(1);
    }
    rv.push(Object::array(results));
    if let Some(nested_error) = nested_error {
        let mut errval = Vec::with_capacity(3 as size_t);
        let failed_at = Integer::try_from(i).expect("a call index fits an Integer");
        errval.push(Object::integer(failed_at));
        errval.push(Object::integer(nested_error.kind().into()));
        let msg = ApiString::from_cstr(nested_error.message_or_empty());
        errval.push(Object::string(msg));
        rv.push(Object::array(errval));
    } else {
        rv.push(Object::Nil);
    }
    Ok(rv.into())
}
