//! The editor context stack: the `ctx*()` family.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::{CONTEXT_INIT, kCtxBufs, kCtxFuncs, kCtxGVars, kCtxJumps, kCtxRegs, kCtxSFuncs};
use crate::context::{
    ctx_free, ctx_from_dict, ctx_get, ctx_restore, ctx_save, ctx_size, ctx_to_dict, kCtxAll,
};
use crate::eval::typval::tv_list_iter;
use crate::memory::{ARENA_EMPTY, arena_finish, arena_mem_free};
use crate::message::state::did_emsg;
use crate::message_fmt::c_str;
use crate::semsg;
use crate::types::{
    Context, Error, EvalFuncData, Object, TypVal, VAR_DICT, VAR_LIST, VAR_NUMBER, VAR_STRING,
    VAR_UNKNOWN, VarNumber,
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
fn context_index(tv: Option<&TypVal>, what: &str) -> Option<usize> {
    match tv {
        None => Some(0),
        Some(tv) if tv.v_type() == VAR_NUMBER => Some(tv.number_or_zero() as usize),
        Some(_) => {
            semsg!("E475: Invalid argument: {what}");
            None
        }
    }
}

/// Resolve a context by index, reporting the out-of-bounds message.
fn context_at(index: usize) -> Option<*mut Context> {
    let ctx = ctx_get(index);
    if ctx.is_null() {
        semsg!("E475: Invalid value for argument index: out of bounds");
        return None;
    }
    Some(ctx)
}

/// `ctxget([{index}])` — the context at `index` as a Dictionary.
pub fn f_ctxget(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: the arena and the error are owned here and freed on the way
    // out; `object_to_vim` copies what it keeps out of the arena's dict.
    let Some(index) = context_index(args.first(), "expected nothing or a Number as an argument")
    else {
        return;
    };
    let Some(ctx) = context_at(index) else {
        return;
    };
    let ctx_dict = unsafe { ctx_to_dict(ctx) };
    let mut err = NO_ERROR;
    *result = TypVal::from(Object::dict(ctx_dict));
    err.clear();
}

/// `ctxpop()` — restore and drop the context on top of the stack.
pub fn f_ctxpop(_args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: restores from the context stack; main thread only.
    if !unsafe { ctx_restore(ptr::null_mut(), kCtxAll.get()) } {
        semsg!("Context stack is empty");
    }
}

/// `ctxpush([{types}])` — push a context holding the named parts of the
/// editor state, or all of them when no list is given.
pub fn f_ctxpush(args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    let _rettv = _result;
    // SAFETY throughout: walks the argument list, whose items live for the call.
    let types = match args.first().map_or(VAR_UNKNOWN, TypVal::v_type) {
        VAR_LIST => {
            let mut types: c_int = 0;
            for li in tv_list_iter(unsafe { args[0].list_or_null().as_ref() }) {
                let tv = &li.li_tv;
                // An unrecognised name is silently ignored, as is a
                // non-String item.
                // A null `v_string` is the empty string, which matches
                // no name; `strequal` answered the same for it.
                if tv.v_type() == VAR_STRING && !tv.string_or_null().is_null() {
                    types |= match unsafe { CStr::from_ptr(tv.string_or_null()) }.to_bytes() {
                        b"regs" => kCtxRegs as c_int,
                        b"jumps" => kCtxJumps as c_int,
                        b"bufs" => kCtxBufs as c_int,
                        b"gvars" => kCtxGVars as c_int,
                        b"sfuncs" => kCtxSFuncs as c_int,
                        b"funcs" => kCtxFuncs as c_int,
                        _ => 0,
                    };
                }
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
pub fn f_ctxset(args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    let _rettv = _result;
    // SAFETY throughout: the arena, the error and the scratch context are owned here;
    // `tmp` is either installed in place of `ctx` or freed.
    if args[0].v_type() != VAR_DICT {
        semsg!("E475: Invalid argument: expected dictionary as first argument");
        return;
    }
    let msg = "expected nothing or a Number as second argument";
    let Some(index) = context_index(args.get(1), msg) else {
        return;
    };
    let Some(ctx) = context_at(index) else {
        return;
    };
    // The conversion reports its problems through `did_emsg`; the caller's
    // flag is restored whatever happens here.
    let save_did_emsg = did_emsg.get();
    did_emsg.set(0);
    let mut arena = ARENA_EMPTY;
    let dict = Object::from(&args[0])
        .into_dict()
        .expect("a VAR_DICT converts to a Dict object");
    let mut tmp = CONTEXT_INIT;
    if let Err(e) = unsafe { ctx_from_dict(dict, &raw mut tmp) } {
        // The message is whatever the API layer produced, so it keeps
        // the variadic call rather than assuming UTF-8.
        // SAFETY: the refusal owns its NUL-terminated message.
        let msg = unsafe { c_str(e.message_or_empty().as_ptr()) };
        semsg!("{msg}");
        unsafe { ctx_free(&raw mut tmp) };
    } else {
        unsafe { ctx_free(ctx) };
        unsafe { *ctx = tmp };
    }
    unsafe { arena_mem_free(arena_finish(&raw mut arena)) };
    did_emsg.set(save_did_emsg);
}

/// `ctxsize()` — how many contexts are on the stack.
pub fn f_ctxsize(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(ctx_size() as VarNumber);
}
