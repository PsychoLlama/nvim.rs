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

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use core::ffi::{c_char, c_int};
use core::slice;

use crate::channel::{channel_close, channel_create_event, channel_job_start};
use crate::eval::find_job;
use crate::eval::funcs::{f_jobstart, f_jobstop};
use crate::eval::typval::{
    CallFrame, NumBuf, tv_dict_add_bool, tv_dict_alloc, tv_dict_free, tv_list_len,
};
use crate::eval::vars::emsg_static;
use crate::ex_cmds::check_secure;
use crate::memory::{xmalloc, xstrdup};
use crate::message::emsg_ptr;
use crate::message::{e_api_spawn_failed, e_invarg};
use crate::semsg;
use crate::types::channel::kChannelStdinPipe;
use crate::types::{
    Callback, CallbackReader, ChannelPart, EvalFuncData, GArray, List, ListItem, TypVal, VAR_DICT,
    VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, VarNumber, kBoolVarTrue, uint64_t,
};
use crate::winlayer::buffers;

pub const kChannelPartRpc: ChannelPart = 3;
pub const GA_EMPTY_INIT_VALUE: GArray = GArray {
    ga_len: 0,
    ga_maxlen: 0,
    ga_itemsize: 0,
    ga_growsize: 1,
    ga_data: core::ptr::null_mut(),
};

/// `CALLBACK_NONE`: no callback at all.
const CALLBACK_NONE: Callback = Callback::None;

/// `CALLBACK_READER_INIT`: a stream nobody is listening to.
const CALLBACK_READER_INIT: CallbackReader = CallbackReader::none();

/// The items of `list`, front to back.  A NULL list is an empty one.
///
/// # Safety
/// `list` must be live, and nothing may change it while the iterator is
/// alive.
unsafe fn items(list: *const List) -> impl Iterator<Item = *const ListItem> {
    let mut li = if list.is_null() {
        core::ptr::null()
    } else {
        unsafe { (*list).lv_first }
    };
    core::iter::from_fn(move || {
        let cur = li;
        if cur.is_null() {
            return None;
        }
        li = unsafe { (*cur).li_next };
        Some(cur)
    })
}

/// `rpcstart(prog[, argv])`: start a job and speak RPC over its pipes.
///
/// Deprecated in favour of `jobstart(..., {'rpc': v:true})`.
pub fn f_rpcstart(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    // SAFETY: the caller's promise about `result`.
    let result = &mut *result;
    result.write_number(0);

    // SAFETY: `check_secure` only reads the option and reports.
    if check_secure() {
        return;
    }

    // SAFETY: the caller's promise about `args`.
    let argv = args;
    if argv[0].v_type() != VAR_STRING
        || (argv[1].v_type() != VAR_LIST && argv[1].v_type() != VAR_UNKNOWN)
    {
        // Wrong argument types.
        emsg_static(e_invarg);
        return;
    }

    let mut args_list: *mut List = core::ptr::null_mut();
    let mut argsl = 0;
    if argv[1].v_type() == VAR_LIST {
        // SAFETY: a `VAR_LIST` holds a live list or NULL.
        args_list = argv[1].list_or_null();
        argsl = unsafe { tv_list_len(args_list) };
        // Assert that all list items are strings.
        for (i, arg) in unsafe { items(args_list) }.enumerate() {
            // SAFETY: `arg` is one of the list's items.
            if unsafe { (*arg).li_tv.v_type() } != VAR_STRING {
                semsg!(
                    "E5010: List item {} of the second argument is not a string",
                    i as c_int
                );
                return;
            }
        }
    }

    // SAFETY: a `VAR_STRING` holds a NUL-terminated string or NULL.
    let prog = argv[0].string_or_null();
    if prog.is_null() || unsafe { *prog } == 0 {
        emsg_static(e_api_spawn_failed);
        return;
    }

    // The program name, its arguments, and the NULL the vector ends with.
    let argvl = argsl as usize + 2;
    // SAFETY: `xmalloc` never answers NULL, and `argvl` slots are written
    // below before anything reads them.
    let raw = unsafe { xmalloc(size_of::<*mut c_char>() * argvl) }.cast::<*mut c_char>();
    // SAFETY: as above -- `argvl` slots were allocated.
    let child_argv = unsafe { slice::from_raw_parts_mut(raw, argvl) };
    // SAFETY: `prog` is a live NUL-terminated string.
    child_argv[0] = unsafe { xstrdup(prog) };
    let mut i = 1;
    // SAFETY: the list is unchanged since it was counted, so it still has
    // `argsl` items and they all fit.
    for arg in unsafe { items(args_list) } {
        child_argv[i] = unsafe { xstrdup(numbuf.string(&raw const (*arg).li_tv)) };
        i += 1;
    }
    child_argv[i] = core::ptr::null_mut();

    // The channel id, or the reason the spawn failed; written on every path.
    let mut status: VarNumber = 0;
    // SAFETY: `channel_job_start` takes over the vector.
    let chan = unsafe {
        channel_job_start(
            child_argv.as_mut_ptr(),
            core::ptr::null(),
            CALLBACK_READER_INIT,
            CALLBACK_READER_INIT,
            CALLBACK_NONE,
            false,
            true,
            false,
            false,
            kChannelStdinPipe,
            core::ptr::null(),
            0,
            0,
            core::ptr::null_mut(),
            &raw mut status,
        )
    };
    result.write_number(status);
    if !chan.is_null() {
        // SAFETY: `chan` is the channel just created.
        unsafe { channel_create_event(chan, core::ptr::null()) };
    }
}

/// `rpcstop(id)`: stop a job, or close a channel that is not one.
pub fn f_rpcstop(args: &[TypVal], result: &mut TypVal, fptr: EvalFuncData) {
    // SAFETY: the caller's promise about `result`.
    let ret = &mut *result;
    ret.write_number(0);

    // SAFETY: `check_secure` only reads the option and reports.
    if check_secure() {
        return;
    }

    // SAFETY: the caller's promise about `args`.
    let argv = args;
    if argv[0].v_type() != VAR_NUMBER {
        // Wrong argument types.
        emsg_static(e_invarg);
        return;
    }

    // SAFETY: a `VAR_NUMBER` holds its number inline.
    let id = argv[0].number_or_zero() as uint64_t;
    // If called with a job, stop it; otherwise close the channel.
    // SAFETY: `find_job` only looks the id up.
    if !unsafe { find_job(id, false) }.is_null() {
        // SAFETY: the arguments are this call's own.
        f_jobstop(args, result, fptr);
    } else {
        let mut error: *const c_char = core::ptr::null();
        // SAFETY: `error` is written whenever the close fails.
        let closed = unsafe { channel_close(id, kChannelPartRpc, &raw mut error) };
        ret.write_number(closed as VarNumber);
        if !closed {
            // SAFETY: the failed close named its reason.
            unsafe { emsg_ptr(error) };
        }
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
    // SAFETY: the caller's promise about `result`.
    result.write_number(n as VarNumber);
}

/// `termopen(cmd[, opts])`: `jobstart()` with `term` forced on.
pub fn f_termopen(args: &[TypVal], result: &mut TypVal, fptr: EvalFuncData) {
    if check_secure() {
        return;
    }

    // `jobstart()` reads its options from a dictionary, and this one always
    // has the `term` flag in it; with no options given, a dictionary is
    // borrowed for the call and freed again on the way out.  The frame
    // borrows the caller's values, so nothing in it is released.
    let borrowed = args.len() < 2;
    let mut frame = CallFrame::<2>::new();
    frame.push_borrowed(&args[0]);
    match args.get(1) {
        Some(opts) => frame.push_borrowed(opts),
        // SAFETY: `tv_dict_alloc` never answers NULL; the dictionary is
        // freed by hand below.
        None => frame.push_naming(TypVal::Dict(unsafe { tv_dict_alloc() })),
    }

    if frame.args()[1].v_type() != VAR_DICT {
        // Wrong argument types.
        semsg!("E475: Invalid argument: {}", "expected dictionary");
        return;
    }

    let dict = frame.args()[1].dict_or_null();
    // SAFETY: `dict` is the dictionary the frame's second slot names.
    let _ = unsafe { tv_dict_add_bool(dict, c"term".as_ptr(), 4, kBoolVarTrue) };
    f_jobstart(frame.args(), result, fptr);
    if borrowed {
        // SAFETY: the dictionary was borrowed for this call only.
        unsafe { tv_dict_free(dict) };
    }
}
