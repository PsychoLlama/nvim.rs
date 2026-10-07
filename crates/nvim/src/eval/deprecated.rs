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
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use core::ffi::{CStr, c_int};

use crate::channel::{channel_close_or_report, rpc_job_start};
use crate::eval::funcs::{f_jobstart, f_jobstop};
use crate::eval::job_is_running;
use crate::eval::typval::{CallFrame, list_iter, tv_dict_alloc};
use crate::eval::vars::emsg_static;
use crate::ex_cmds::check_secure;
use crate::message::{e_api_spawn_failed, e_invarg};
use crate::semsg;
use crate::types::{
    ChannelPart, EvalFuncData, TypVal, VAR_DICT, VAR_LIST, VAR_NUMBER, VAR_STRING, VarNumber,
    kBoolVarTrue, uint64_t,
};
use crate::winlayer::buffers;

pub const kChannelPartRpc: ChannelPart = 3;

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
        emsg_static(e_invarg);
        return;
    }

    // The guard above leaves only a List, or nothing, here.
    let list = given.and_then(TypVal::list_ref);
    // Assert that all list items are strings.
    for (i, arg) in list_iter(list).enumerate() {
        if arg.li_tv.v_type() != VAR_STRING {
            semsg!(
                "E5010: List item {} of the second argument is not a string",
                i as c_int
            );
            return;
        }
    }

    let Some(prog) = args[0].string_ref().filter(|prog| !prog.is_empty()) else {
        emsg_static(e_api_spawn_failed);
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
        emsg_static(e_invarg);
        return;
    }

    let id = args[0].number_or_zero() as uint64_t;
    // If called with a job, stop it; otherwise close the channel.
    if job_is_running(id) {
        f_jobstop(args, result, fptr);
    } else {
        let closed = channel_close_or_report(id, kChannelPartRpc);
        result.write_number(VarNumber::from(closed));
    }
}

/// `last_buffer_nr()`: the highest buffer number in use.
///
/// Not the same answer as `bufnr("$")` once the highest-numbered buffer has
/// been wiped, which is the only reason it still exists.
pub fn f_last_buffer_nr(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut n = 0;
    for buf in buffers() {
        n = n.max(buf.handle());
    }
    result.write_number(n as VarNumber);
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
        let _ = dict.edit().add_bool(b"term", kBoolVarTrue);
    }
    f_jobstart(frame.args(), result, fptr);
    drop(made);
}
