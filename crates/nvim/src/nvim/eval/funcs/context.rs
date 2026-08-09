//! The editor context stack: the `ctx*()` family.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::frame;
use super::{CONTEXT_INIT, kCtxBufs, kCtxFuncs, kCtxGVars, kCtxJumps, kCtxRegs, kCtxSFuncs};
use crate::semsg;
use crate::semsg_c;
use crate::src::nvim::api::private::converter::{object_to_vim, vim_to_object};
use crate::src::nvim::api::private::helpers::api_clear_error;
use crate::src::nvim::context::{
    ctx_free, ctx_from_dict, ctx_get, ctx_restore, ctx_save, ctx_size, ctx_to_dict, kCtxAll,
};
use crate::src::nvim::eval::typval::tv_list_first;
use crate::src::nvim::main::did_emsg;
use crate::src::nvim::memory::{ARENA_EMPTY, arena_finish, arena_mem_free};
use crate::src::nvim::types::{
    Context, Error, EvalFuncData, VAR_DICT, VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN,
    kErrorTypeNone, kObjectTypeDict, object, object_data, typval_T, varnumber_T,
};
use core::ffi::{CStr, c_int};
use core::ptr;

/// A cleared API error, the shape every `api_*` out-parameter starts in.
const NO_ERROR: Error = Error {
    type_0: kErrorTypeNone,
    msg: ptr::null_mut(),
};

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
///
/// # Safety
/// Reads the context stack, which is only touched from the main thread.
unsafe fn context_at(index: usize) -> Option<*mut Context> {
    // SAFETY: the caller's obligation.
    let ctx = unsafe { ctx_get(index) };
    if ctx.is_null() {
        semsg!("E475: Invalid value for argument index: out of bounds");
        return None;
    }
    Some(ctx)
}

/// `ctxget([{index}])` — the context at `index` as a Dictionary.
pub unsafe extern "C" fn f_ctxget(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arena and the error are owned here and freed on the way
    // out; `object_to_vim` copies what it keeps out of the arena's dict.
    unsafe {
        let Some(index) = context_index(args.ptr(0), "expected nothing or a Number as an argument")
        else {
            return;
        };
        let Some(ctx) = context_at(index) else {
            return;
        };
        let mut arena = ARENA_EMPTY;
        let ctx_dict = ctx_to_dict(ctx, &raw mut arena);
        let mut err = NO_ERROR;
        object_to_vim(
            object {
                type_0: kObjectTypeDict,
                data: object_data { dict: ctx_dict },
            },
            rettv,
            &raw mut err,
        );
        arena_mem_free(arena_finish(&raw mut arena));
        api_clear_error(&raw mut err);
    }
}

/// `ctxpop()` — restore and drop the context on top of the stack.
pub unsafe extern "C" fn f_ctxpop(
    _argvars: *mut typval_T,
    _rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: restores from the context stack; main thread only.
    if !unsafe { ctx_restore(ptr::null_mut(), kCtxAll.get()) } {
        semsg!("Context stack is empty");
    }
}

/// `ctxpush([{types}])` — push a context holding the named parts of the
/// editor state, or all of them when no list is given.
pub unsafe extern "C" fn f_ctxpush(
    argvars: *mut typval_T,
    _rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, _rettv) = frame!(argvars, _rettv);
    // SAFETY: walks the argument list, whose items live for the call.
    unsafe {
        let types = match args.ty(0) {
            VAR_LIST => {
                let mut types: c_int = 0;
                let mut li = tv_list_first(args.get(0).vval.v_list);
                while !li.is_null() {
                    let tv = &(*li).li_tv;
                    // An unrecognised name is silently ignored, as is a
                    // non-String item.
                    // A null `v_string` is the empty string, which matches
                    // no name; `strequal` answered the same for it.
                    if tv.v_type == VAR_STRING && !tv.vval.v_string.is_null() {
                        types |= match CStr::from_ptr(tv.vval.v_string).to_bytes() {
                            b"regs" => kCtxRegs as c_int,
                            b"jumps" => kCtxJumps as c_int,
                            b"bufs" => kCtxBufs as c_int,
                            b"gvars" => kCtxGVars as c_int,
                            b"sfuncs" => kCtxSFuncs as c_int,
                            b"funcs" => kCtxFuncs as c_int,
                            _ => 0,
                        };
                    }
                    li = (*li).li_next;
                }
                types
            }
            VAR_UNKNOWN => kCtxAll.get(),
            _ => {
                semsg!("E475: Invalid argument: expected nothing or a List as an argument");
                return;
            }
        };
        ctx_save(ptr::null_mut(), types);
    }
}

/// `ctxset({context} [, {index}])` — replace the context at `index`.
pub unsafe extern "C" fn f_ctxset(
    argvars: *mut typval_T,
    _rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, _rettv) = frame!(argvars, _rettv);
    // SAFETY: the arena, the error and the scratch context are owned here;
    // `tmp` is either installed in place of `ctx` or freed.
    unsafe {
        if args.ty(0) != VAR_DICT {
            semsg!("E475: Invalid argument: expected dictionary as first argument");
            return;
        }
        let Some(index) = context_index(
            args.ptr(1),
            "expected nothing or a Number as second argument",
        ) else {
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
        let dict = vim_to_object(args.ptr(0), &raw mut arena, true).data.dict;
        let mut tmp = CONTEXT_INIT;
        let mut err = NO_ERROR;
        ctx_from_dict(dict, &raw mut tmp, &raw mut err);
        if err.type_0 != kErrorTypeNone {
            // The message is whatever the API layer produced, so it keeps
            // the variadic call rather than assuming UTF-8.
            semsg_c!(c"%s".as_ptr(), err.msg);
            ctx_free(&raw mut tmp);
        } else {
            ctx_free(ctx);
            *ctx = tmp;
        }
        arena_mem_free(arena_finish(&raw mut arena));
        api_clear_error(&raw mut err);
        did_emsg.set(save_did_emsg);
    }
}

/// `ctxsize()` — how many contexts are on the stack.
pub unsafe extern "C" fn f_ctxsize(
    _argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (_args, rettv) = frame!(_argvars, rettv);
    rettv.v_type = VAR_NUMBER;
    // SAFETY: reads the context stack's length; main thread only.
    rettv.vval.v_number = unsafe { ctx_size() } as varnumber_T;
}
