//! `eval/deprecated.c`: the builtins upstream keeps only so old scripts keep
//! running.
//!
//! Four of them, and none does much of its own: `rpcstart()` and `termopen()`
//! check their arguments and hand over to `channel_job_start()` /
//! `jobstart()`, `rpcstop()` picks between `jobstop()` and closing a channel,
//! and `last_buffer_nr()` is a maximum over the buffer list.  What is worth
//! reading here is therefore the argument checking and the argv build — the
//! spawning itself belongs to `channel.rs`.
//!
//! # Safety
//!
//! Every function here is a `EvalFuncData` builtin: the evaluator calls it
//! with `argvars` pointing at the evaluated arguments followed by a
//! `VAR_UNKNOWN` terminator, and with `rettv` pointing at a cleared result.
//! Each declares its arity in `eval.lua`, and that arity is what says how
//! many of the two slots below are real.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::narrow::len_as_int;
use crate::types::Handle;
use core::ffi::CStr;

use crate::channel::channel_job_running;
use crate::channel::defs::ChannelPart;
use crate::channel::{channel_close_or_report, rpc_job_start};
use crate::eval::funcs::builtin::EvalFuncData;
use crate::eval::funcs::{f_jobstart, f_jobstop};
use crate::eval::typval::{CallFrame, list_iter, tv_dict_alloc};
use crate::eval::vars::emsg_static;
use crate::ex_cmds::check_secure;
use crate::message::{E_API_SPAWN_FAILED, E_INVARG};
use crate::semsg;
use crate::types::{BOOL_VAR_TRUE, TypVal, VAR_DICT, VAR_LIST, VAR_NUMBER, VAR_STRING, VarNumber};
use crate::winlayer::buffers;

/// `rpcstart(prog[, argv])`: start a job and speak RPC over its pipes.
///
/// Deprecated in favour of `jobstart(..., {'rpc': v:true})`.
pub fn f_rpcstart(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(0);

    if check_secure() {
        return;
    }

    // The arguments are optional: `rpcstart('prog')` gives one argument, so
    // the second slot is *absent* rather than a `VAR_UNKNOWN` terminator.
    let given = args.get(1);
    if args[0].v_type() != VAR_STRING || given.is_some_and(|a| a.v_type() != VAR_LIST) {
        // Wrong argument types.
        emsg_static(E_INVARG);
        return;
    }

    // The guard above leaves only a List, or nothing, here.
    let list = given.and_then(TypVal::list_ref);
    // Assert that all list items are strings.
    for (i, arg) in list_iter(list).enumerate() {
        if arg.li_tv.v_type() != VAR_STRING {
            semsg!(
                "E5010: List item {} of the second argument is not a string",
                len_as_int(i)
            );
            return;
        }
    }

    let Some(prog) = args[0].string_ref().filter(|prog| !prog.is_empty()) else {
        emsg_static(E_API_SPAWN_FAILED);
        return;
    };

    // The program name, then its arguments: Strings, checked above, the null
    // String reading as the empty one.
    let mut argv: Vec<&CStr> = vec![prog.as_cstr()];
    argv.extend(list_iter(list).map(|arg| arg.li_tv.string_cstr().unwrap_or(c"")));
    // The channel id, or the reason the spawn failed.
    result.write_number(rpc_job_start(&argv));
}

/// `rpcstop(id)`: stop a job, or close a channel that is not one.
pub fn f_rpcstop(args: &[TypVal], result: &mut TypVal, fptr: EvalFuncData) {
    result.write_number(0);

    if check_secure() {
        return;
    }

    if args[0].v_type() != VAR_NUMBER {
        // Wrong argument types.
        emsg_static(E_INVARG);
        return;
    }

    let id = args[0].number_or_zero().cast_unsigned();
    // If called with a job, stop it; otherwise close the channel.
    if channel_job_running(id) {
        f_jobstop(args, result, fptr);
    } else {
        let closed = channel_close_or_report(id, ChannelPart::Rpc).is_ok();
        result.write_number(VarNumber::from(closed));
    }
}

/// `last_buffer_nr()`: the highest buffer number in use.
///
/// Not the same answer as `bufnr("$")` once the highest-numbered buffer has
/// been wiped, which is the only reason it still exists.
pub fn f_last_buffer_nr(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut n = Handle::NONE;
    for buf in buffers() {
        n = n.max(buf.handle());
    }
    result.write_number(VarNumber::from(n.get()));
}

/// `termopen(cmd[, opts])`: `jobstart()` with `term` forced on.
pub fn f_termopen(args: &[TypVal], result: &mut TypVal, fptr: EvalFuncData) {
    if check_secure() {
        return;
    }

    // `jobstart()` reads its options from a dictionary, and this one always
    // has the `term` flag in it; with no options given, a dictionary is made
    // for the call and freed again on the way out.  The frame borrows its
    // values, so nothing in it is released.
    let made = (args.len() < 2).then(|| TypVal::dict(Some(tv_dict_alloc())));
    let mut frame = CallFrame::<2>::new();
    frame.push_borrowed(&args[0]);
    match args.get(1).or(made.as_ref()) {
        Some(opts) => frame.push_borrowed(opts),
        None => unreachable!("no options means a fresh dictionary"),
    }

    if frame.args()[1].v_type() != VAR_DICT {
        // Wrong argument types.
        semsg!("E475: Invalid argument: {}", "expected dictionary");
        return;
    }

    if let Some(dict) = frame.args()[1].dict_shared() {
        let _ = dict.edit().add_bool(b"term", BOOL_VAR_TRUE);
    }
    f_jobstart(frame.args(), result, fptr);
    drop(made);
}
