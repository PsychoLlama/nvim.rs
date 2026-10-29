//! `vim.call()` and the rpc entry points.
//!
//! [`call`] invokes a *Vimscript* function from Lua: it converts up to
//! `MAX_FUNC_ARGS` Lua values to `TypVal`s, calls through `call_func_with`,
//! and converts the result back. [`rpcrequest`] and [`rpcnotify`] are
//! `vim.rpcrequest()` and `vim.rpcnotify()`, which differ only in whether
//! they wait.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int};

use super::{LUA_INTERNAL_CALL, MAX_FUNC_ARGS, nlua_is_deferred_safe, viml_func_is_fast};
use crate::api::private::helpers::{api_set_sctx, try_enter, try_leave};
use crate::api_error;
use crate::eval::typval::{CallFrame, TV_INITIAL_VALUE, tv_clear};
use crate::eval::userfunc::{CallWith, call_func_with};
use crate::ex_eval::state::{did_throw, force_abort, suppress_errthrow};
use crate::ex_getln::TRY_STATE_INIT;
use crate::lua::converter::{pop_object, pop_typval, push_object, push_typval};
use crate::lua::stack::{At, Raise, Stack};
use crate::message::e_fast_api_disabled;
use crate::message::state::did_emsg;
use crate::msgpack_rpc::channel::{rpc_call_with, rpc_send_event};
use crate::narrow::len_as_int;
use crate::types::{Array, Error, kErrorTypeException, kErrorTypeValidation};

/// How much of a rejected function's name the "not allowed in a fast event"
/// message quotes.
const NAME_LIMIT: usize = 100;

/// The arguments `vim.call()` converts, plus the one slot `CallFrame` keeps
/// spare.
const FRAME_SLOTS: usize = MAX_FUNC_ARGS as usize + 1;

/// `e_fast_api_disabled` with `what` in its slot: the refusal a function
/// that may not run in a fast event raises, as `luaL_error` formats it.
pub(crate) fn fast_api_refusal(what: &[u8]) -> Vec<u8> {
    let template = e_fast_api_disabled.to_bytes();
    let at = template
        .windows(2)
        .position(|w| w == b"%s")
        .expect("the message has a slot for the name");
    [&template[..at], what, &template[at + 2..]].concat()
}

/// `E5560: Vimscript function "<name>" must not be called in a fast event
/// context`.
///
/// Upstream formats the quoted part into a buffer sized for at most
/// `NAME_LIMIT` bytes of the name plus the quotes and the terminator -- one
/// byte short, so a name over the limit is cut to `NAME_LIMIT + 1` bytes
/// and loses its closing quote. Kept, byte for byte.
fn fast_call_refusal(name: &[u8]) -> Vec<u8> {
    const PREFIX: &[u8] = b"Vimscript function \"";
    let mut quoted = [PREFIX, name, b"\""].concat();
    quoted.truncate(PREFIX.len() + name.len().min(NAME_LIMIT) + 1);
    fast_api_refusal(&quoted)
}

/// Raise `err` as it is: its message, with no position in front.
fn raise_error(stack: &mut Stack, err: &Error) -> Raise {
    stack.push_bytes(err.message_or_empty().to_bytes());
    Raise::top_value()
}

/// `vim.call(name, ...)`: call a Vimscript function.
///
/// Every argument is converted into a frame of typvals, and every one
/// converted is cleared again, whichever way the call went.
pub(crate) fn call(stack: &mut Stack) -> Result<c_int, Raise> {
    // The name stays in slot 1, read once: the view the arguments are
    // converted in owns only the slots above it.
    let mut args = stack.above(At::arg(1));
    let (name, name_len) = args.check_c_string_below(At::arg(1))?;
    if !nlua_is_deferred_safe() {
        // `%s` stops at a NUL, and so does the name the message quotes.
        let name = name.to_bytes();
        if !viml_func_is_fast(name) {
            return Err(Raise::message(fast_call_refusal(name)));
        }
    }

    let nargs = usize::try_from(args.top().index() - 1).unwrap_or(0);
    if nargs > MAX_FUNC_ARGS as usize {
        return Err(Raise::message("Function called with too many arguments"));
    }

    // A frame rather than a plain array: it releases the `nargs` slots it
    // was given and no more, where an array's drop glue walks all
    // twenty-one on every call.
    let mut vim_args = CallFrame::<FRAME_SLOTS>::new();
    match convert_and_call(&mut args, nargs, (name, name_len), &mut vim_args) {
        Ok(()) => Ok(1),
        Err(err) => Err(raise_error(&mut args, &err)),
    }
}

/// [`call`]'s body once the arguments are counted: convert them into
/// `vim_args`, call the function `name` (its C string and full length),
/// and push its answer.
fn convert_and_call(
    stack: &mut Stack,
    nargs: usize,
    (name, name_len): (&CStr, usize),
    vim_args: &mut CallFrame<FRAME_SLOTS>,
) -> Result<(), Error> {
    for i in 0..nargs {
        stack.push_value(At::arg(len_as_int(i) + 2));
        let Some(arg) = pop_typval(stack) else {
            return Err(api_error!(
                kErrorTypeException,
                "error converting argument {}",
                i + 1
            ));
        };
        *vim_args.claim() = arg;
    }

    // Start the call from a clean exception state: a Lua caller has no
    // `:try` around it and must not inherit one.
    force_abort.set(false);
    suppress_errthrow.set(false);
    did_throw.set(false);
    did_emsg.set(0);

    let mut rettv = TV_INITIAL_VALUE;
    let mut err = Error::none();
    let sctx = api_set_sctx(LUA_INTERNAL_CALL);
    let mut tstate = TRY_STATE_INIT;
    try_enter(&mut tstate);
    let _ = call_func_with(
        name,
        Some(name_len),
        &mut rettv,
        vim_args.args(),
        CallWith::at_cursor(true),
    );
    err.absorb(try_leave(&tstate));
    drop(sctx);

    if !err.is_set() {
        push_typval(stack, &rettv, false);
    }
    tv_clear(&mut rettv);
    if err.is_set() { Err(err) } else { Ok(()) }
}

/// `vim.rpcrequest(channel, method, ...)`.
pub(crate) fn rpcrequest(stack: &mut Stack) -> Result<c_int, Raise> {
    if !nlua_is_deferred_safe() {
        return Err(Raise::message(fast_api_refusal(b"rpcrequest")));
    }
    rpc(stack, true)
}

/// `vim.rpcnotify(channel, method, ...)`.
pub(crate) fn rpcnotify(stack: &mut Stack) -> Result<c_int, Raise> {
    rpc(stack, false)
}

/// The shared body of the two rpc entry points. `request` waits for the
/// answer and pushes it; a notification pushes nothing.
fn rpc(stack: &mut Stack, request: bool) -> Result<c_int, Raise> {
    // A negative id wraps, as the C cast does.
    let chan_id = i64::try_from(stack.check_integer(At::arg(1))?)
        .unwrap_or_default()
        .cast_unsigned();
    // The method name stays in slot 2, read once: the arguments are
    // converted, and the answer pushed, in a view above it.
    let mut args = stack.above(At::arg(2));
    let (name, _) = args.check_c_string_below(At::arg(2))?;
    let nargs = args.top().index() - 2;

    let mut err = Error::none();
    let mut converted = Array::with_capacity(usize::try_from(nargs).unwrap_or(0));
    let mut converts = true;
    for i in 0..nargs {
        args.push_value(At::arg(i + 3));
        match pop_object(&mut args, false) {
            Ok(value) => converted.push(value),
            Err(e) => {
                err = e;
                converts = false;
                break;
            }
        }
    }

    if converts {
        if request {
            let answered = rpc_call_with(chan_id, name, converted, |result| {
                push_object(&mut args, &result, false);
            });
            if let Err(e) = answered {
                err = e;
            }
        } else if !rpc_send_event(chan_id, name, converted) {
            err = api_error!(kErrorTypeValidation, "Invalid channel: {chan_id}");
        }
    }

    if err.is_set() {
        return Err(raise_error(&mut args, &err));
    }
    Ok(if request { 1 } else { 0 })
}
