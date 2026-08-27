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

use core::ffi::{c_char, c_int};
use core::slice;

use crate::channel::{channel_close, channel_create_event, channel_job_start};
use crate::eval::find_job;
use crate::eval::funcs::{f_jobstart, f_jobstop};
use crate::eval::typval::{
    NumBuf, kCallbackNone, tv_dict_add_bool, tv_dict_alloc, tv_dict_free, tv_list_len,
};
use crate::eval::vars::emsg_static;
use crate::ex_cmds::check_secure;
use crate::main::{e_api_spawn_failed, e_invarg, e_invarg2};
use crate::memory::{xmalloc, xstrdup};
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::semsg_c;
use crate::types::channel::kChannelStdinPipe;
use crate::types::{
    Callback, Callback_data, CallbackReader, ChannelPart, EvalFuncData, VAR_DICT, VAR_LIST,
    VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, garray_T, kBoolVarTrue, list_T, listitem_T, typval_T,
    uint64_t, varnumber_T,
};
use crate::winlayer::buffers;

pub const kChannelPartRpc: ChannelPart = 3;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0,
    ga_maxlen: 0,
    ga_itemsize: 0,
    ga_growsize: 1,
    ga_data: core::ptr::null_mut(),
};

/// `CALLBACK_NONE`: no callback at all.
const CALLBACK_NONE: Callback = Callback {
    data: Callback_data {
        funcref: core::ptr::null_mut(),
    },
    type_0: kCallbackNone,
};

/// `CALLBACK_READER_INIT`: a stream nobody is listening to.
const CALLBACK_READER_INIT: CallbackReader = CallbackReader {
    cb: CALLBACK_NONE,
    self_0: core::ptr::null_mut(),
    buffer: GA_EMPTY_INIT_VALUE,
    eof: false,
    buffered: false,
    fwd_err: false,
    type_0: core::ptr::null(),
};

/// The evaluator's argument vector: the declared arguments plus the
/// `VAR_UNKNOWN` that ends them.
///
/// Every builtin here declares at most two, so two slots are always readable
/// — the second being `VAR_UNKNOWN` when the caller passed one argument.
///
/// # Safety
/// `argvars` must be a builtin's own argument vector.
#[inline(always)]
unsafe fn args<'a>(argvars: *mut typval_T) -> &'a mut [typval_T] {
    unsafe { slice::from_raw_parts_mut(argvars, 2) }
}

/// The items of `list`, front to back.  A NULL list is an empty one.
///
/// # Safety
/// `list` must be live, and nothing may change it while the iterator is
/// alive.
unsafe fn items(list: *const list_T) -> impl Iterator<Item = *const listitem_T> {
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
///
/// # Safety
/// As the module doc; arity 1..2.
pub unsafe fn f_rpcstart(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    // SAFETY: the caller's promise about `rettv`.
    let rettv = unsafe { &mut *rettv };
    rettv.v_type = VAR_NUMBER;
    rettv.vval.v_number = 0;

    // SAFETY: `check_secure` only reads the option and reports.
    if check_secure() {
        return;
    }

    // SAFETY: the caller's promise about `argvars`.
    let argv = unsafe { args(argvars) };
    if argv[0].v_type != VAR_STRING || (argv[1].v_type != VAR_LIST && argv[1].v_type != VAR_UNKNOWN)
    {
        // Wrong argument types.
        emsg_static(&e_invarg);
        return;
    }

    let mut args_list: *mut list_T = core::ptr::null_mut();
    let mut argsl = 0;
    if argv[1].v_type == VAR_LIST {
        // SAFETY: a `VAR_LIST` holds a live list or NULL.
        args_list = unsafe { argv[1].vval.v_list };
        argsl = unsafe { tv_list_len(args_list) };
        // Assert that all list items are strings.
        for (i, arg) in unsafe { items(args_list) }.enumerate() {
            // SAFETY: `arg` is one of the list's items.
            if unsafe { (*arg).li_tv.v_type } != VAR_STRING {
                let msg = c"E5010: List item %d of the second argument is not a string";
                // SAFETY: the message is a NUL-terminated literal whose
                // format takes one `int`.
                unsafe { semsg_c!(gettext(msg.as_ptr()), i as c_int) };
                return;
            }
        }
    }

    // SAFETY: a `VAR_STRING` holds a NUL-terminated string or NULL.
    let prog = unsafe { argv[0].vval.v_string };
    if prog.is_null() || unsafe { *prog } == 0 {
        emsg_static(&e_api_spawn_failed);
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
            &raw mut rettv.vval.v_number,
        )
    };
    if !chan.is_null() {
        // SAFETY: `chan` is the channel just created.
        unsafe { channel_create_event(chan, core::ptr::null()) };
    }
}

/// `rpcstop(id)`: stop a job, or close a channel that is not one.
///
/// # Safety
/// As the module doc; arity 1.
pub unsafe fn f_rpcstop(argvars: *mut typval_T, rettv: *mut typval_T, fptr: EvalFuncData) {
    // SAFETY: the caller's promise about `rettv`.
    let ret = unsafe { &mut *rettv };
    ret.v_type = VAR_NUMBER;
    ret.vval.v_number = 0;

    // SAFETY: `check_secure` only reads the option and reports.
    if check_secure() {
        return;
    }

    // SAFETY: the caller's promise about `argvars`.
    let argv = unsafe { args(argvars) };
    if argv[0].v_type != VAR_NUMBER {
        // Wrong argument types.
        emsg_static(&e_invarg);
        return;
    }

    // SAFETY: a `VAR_NUMBER` holds its number inline.
    let id = unsafe { argv[0].vval.v_number } as uint64_t;
    // If called with a job, stop it; otherwise close the channel.
    // SAFETY: `find_job` only looks the id up.
    if !unsafe { find_job(id, false) }.is_null() {
        // SAFETY: the arguments are this call's own.
        unsafe { f_jobstop(argvars, rettv, fptr) };
    } else {
        let mut error: *const c_char = core::ptr::null();
        // SAFETY: `error` is written whenever the close fails.
        let closed = unsafe { channel_close(id, kChannelPartRpc, &raw mut error) };
        ret.vval.v_number = closed as varnumber_T;
        if !closed {
            // SAFETY: the failed close named its reason.
            unsafe { emsg(error) };
        }
    }
}

/// `last_buffer_nr()`: the highest buffer number in use.
///
/// Not the same answer as `bufnr("$")` once the highest-numbered buffer has
/// been wiped, which is the only reason it still exists.
///
/// # Safety
/// As the module doc; arity 0.
pub unsafe fn f_last_buffer_nr(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut n = 0;
    for buf in buffers() {
        n = n.max(buf.handle());
    }
    // SAFETY: the caller's promise about `rettv`.
    unsafe { (*rettv).vval.v_number = n as varnumber_T };
}

/// `termopen(cmd[, opts])`: `jobstart()` with `term` forced on.
///
/// # Safety
/// As the module doc; arity 1..2.
pub unsafe fn f_termopen(argvars: *mut typval_T, rettv: *mut typval_T, fptr: EvalFuncData) {
    // SAFETY: `check_secure` only reads the option and reports.
    if check_secure() {
        return;
    }

    // SAFETY: the caller's promise about `argvars`.
    let argv = unsafe { args(argvars) };
    // With no options at all, borrow a dictionary for the one flag this adds
    // and free it again on the way out.
    let must_free = argv[1].v_type == VAR_UNKNOWN;
    if must_free {
        argv[1].v_type = VAR_DICT;
        // SAFETY: `tv_dict_alloc` never answers NULL.
        argv[1].vval.v_dict = unsafe { tv_dict_alloc() };
    }

    if argv[1].v_type != VAR_DICT {
        // Wrong argument types.
        // SAFETY: `e_invarg2` takes one string.
        unsafe { semsg_c!(gettext(e_invarg2.as_ptr()), c"expected dictionary".as_ptr(),) };
        return;
    }

    // SAFETY: `argv[1]` holds a live dictionary, either the caller's or the
    // one allocated above; `f_jobstart` takes the whole argument vector.
    let dict = unsafe { argv[1].vval.v_dict };
    // SAFETY: as above -- `dict` is that dictionary.
    unsafe { tv_dict_add_bool(dict, c"term".as_ptr(), 4, kBoolVarTrue) };
    // SAFETY: as above -- the whole argument vector goes to `jobstart()`.
    unsafe { f_jobstart(argvars, rettv, fptr) };
    if must_free {
        // SAFETY: the dictionary was borrowed for this call only.
        unsafe { tv_dict_free(dict) };
    }
}
