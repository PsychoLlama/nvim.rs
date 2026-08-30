//! The editor context stack: the `ctx*()` family.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::frame;
use super::{CONTEXT_INIT, kCtxBufs, kCtxFuncs, kCtxGVars, kCtxJumps, kCtxRegs, kCtxSFuncs};
use crate::api::private::converter::{object_to_vim, vim_to_object};
use crate::context::{
    ctx_free, ctx_from_dict, ctx_get, ctx_restore, ctx_save, ctx_size, ctx_to_dict, kCtxAll,
};
use crate::eval::typval::tv_list_first;
use crate::main::did_emsg;
use crate::memory::{ARENA_EMPTY, arena_finish, arena_mem_free};
use crate::message_fmt::c_str;
use crate::semsg;
use crate::types::{
    Context, Error, EvalFuncData, VAR_DICT, VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN,
    kObjectTypeDict, object, object_data, typval_T, varnumber_T,
};
use core::ffi::{CStr, c_int};
use core::ptr;

/// A cleared API error, the shape every `api_*` out-parameter starts in.
const NO_ERROR: Error = Error::none();

/// The `{index}` argument the `ctxget`/`ctxset` pair share: absent means 0,
/// a Number is taken as-is, anything else is rejected with `what`.
///
/// # Safety
/// `tv` is a live typval from the call frame.
unsafe fn context_index(tv: *const typval_T, what: &str) -> Option<usize> {
    // SAFETY: the caller's obligation.
    let tv = unsafe { &*tv };
    if tv.v_type == VAR_NUMBER {
        // SAFETY: the tag says the union holds a Number.
        Some(unsafe { tv.vval.v_number } as usize)
    } else if tv.v_type == VAR_UNKNOWN {
        Some(0)
    } else {
        semsg!("E475: Invalid argument: {what}");
        None
    }
}

/// Resolve a context by index, reporting the out-of-bounds message.
fn context_at(index: usize) -> Option<*mut Context> {
    // SAFETY: the caller's obligation.
    let ctx = unsafe { ctx_get(index) };
    if ctx.is_null() {
        semsg!("E475: Invalid value for argument index: out of bounds");
        return None;
    }
    Some(ctx)
}

/// `ctxget([{index}])` — the context at `index` as a Dictionary.
pub unsafe fn f_ctxget(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY throughout: the arena and the error are owned here and freed on the way
    // out; `object_to_vim` copies what it keeps out of the arena's dict.
    let arg = args.ptr(0);
    let Some(index) =
        (unsafe { context_index(arg, "expected nothing or a Number as an argument") })
    else {
        return;
    };
    let Some(ctx) = context_at(index) else {
        return;
    };
    let mut arena = ARENA_EMPTY;
    let ctx_dict = unsafe { ctx_to_dict(ctx, &raw mut arena) };
    let mut err = NO_ERROR;
    let dict = object {
        type_0: kObjectTypeDict,
        data: object_data { dict: ctx_dict },
    };
    unsafe { object_to_vim(dict, rettv, &raw mut err) };
    unsafe { arena_mem_free(arena_finish(&raw mut arena)) };
    err.clear();
}

/// `ctxpop()` — restore and drop the context on top of the stack.
pub unsafe fn f_ctxpop(_argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: restores from the context stack; main thread only.
    if !unsafe { ctx_restore(ptr::null_mut(), kCtxAll.get()) } {
        semsg!("Context stack is empty");
    }
}

/// `ctxpush([{types}])` — push a context holding the named parts of the
/// editor state, or all of them when no list is given.
pub unsafe fn f_ctxpush(argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, _rettv) = frame!(argvars, _rettv);
    // SAFETY throughout: walks the argument list, whose items live for the call.
    let types = match args.ty(0) {
        VAR_LIST => {
            let mut types: c_int = 0;
            let mut li = unsafe { tv_list_first(args.get(0).vval.v_list) };
            while !li.is_null() {
                let tv = unsafe { &(*li).li_tv };
                // An unrecognised name is silently ignored, as is a
                // non-String item.
                // A null `v_string` is the empty string, which matches
                // no name; `strequal` answered the same for it.
                if tv.v_type == VAR_STRING && !unsafe { tv.vval.v_string }.is_null() {
                    types |= match unsafe { CStr::from_ptr(tv.vval.v_string) }.to_bytes() {
                        b"regs" => kCtxRegs as c_int,
                        b"jumps" => kCtxJumps as c_int,
                        b"bufs" => kCtxBufs as c_int,
                        b"gvars" => kCtxGVars as c_int,
                        b"sfuncs" => kCtxSFuncs as c_int,
                        b"funcs" => kCtxFuncs as c_int,
                        _ => 0,
                    };
                }
                li = unsafe { (*li).li_next };
            }
            types
        }
        VAR_UNKNOWN => kCtxAll.get(),
        _ => {
            semsg!("E475: Invalid argument: expected nothing or a List as an argument");
            return;
        }
    };
    unsafe { ctx_save(ptr::null_mut(), types) };
}

/// `ctxset({context} [, {index}])` — replace the context at `index`.
pub unsafe fn f_ctxset(argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, _rettv) = frame!(argvars, _rettv);
    // SAFETY throughout: the arena, the error and the scratch context are owned here;
    // `tmp` is either installed in place of `ctx` or freed.
    if args.ty(0) != VAR_DICT {
        semsg!("E475: Invalid argument: expected dictionary as first argument");
        return;
    }
    let arg = args.ptr(1);
    let msg = "expected nothing or a Number as second argument";
    let Some(index) = (unsafe { context_index(arg, msg) }) else {
        return;
    };
    let Some(ctx) = context_at(index) else {
        return;
    };
    // `vim_to_object` reports conversion problems through `did_emsg`;
    // the caller's flag is restored whatever happens here.
    let save_did_emsg = did_emsg.get();
    did_emsg.set(0);
    let mut arena = ARENA_EMPTY;
    let dict = unsafe { vim_to_object(args.ptr(0), &raw mut arena, true).data.dict };
    let mut tmp = CONTEXT_INIT;
    let mut err = NO_ERROR;
    unsafe { ctx_from_dict(dict, &raw mut tmp, &raw mut err) };
    if err.is_set() {
        // The message is whatever the API layer produced, so it keeps
        // the variadic call rather than assuming UTF-8.
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let msg = unsafe { c_str(err.message_or_empty().as_ptr()) };
        semsg!("{msg}");
        unsafe { ctx_free(&raw mut tmp) };
    } else {
        unsafe { ctx_free(ctx) };
        unsafe { *ctx = tmp };
    }
    unsafe { arena_mem_free(arena_finish(&raw mut arena)) };
    err.clear();
    did_emsg.set(save_did_emsg);
}

/// `ctxsize()` — how many contexts are on the stack.
pub unsafe fn f_ctxsize(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (_args, rettv) = frame!(_argvars, rettv);
    rettv.v_type = VAR_NUMBER;
    // SAFETY: reads the context stack's length; main thread only.
    rettv.vval.v_number = unsafe { ctx_size() } as varnumber_T;
}
