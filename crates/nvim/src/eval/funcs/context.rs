//! The editor context stack: the `ctx*()` family.
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::context::Context;
use crate::context::kCtxBufs;
use crate::context::kCtxFuncs;
use crate::context::kCtxGVars;
use crate::context::kCtxJumps;
use crate::context::kCtxRegs;
use crate::context::kCtxSFuncs;
use crate::context::{
    CTX_ALL, ctx_from_dict, ctx_restore, ctx_save, ctx_size, ctx_to_dict, with_ctx,
};
use crate::eval::funcs::builtin::EvalFuncData;
use crate::eval::typval::list_iter;
use crate::message::state::did_emsg;
use crate::message_fmt::msg_cstr;
use crate::narrow::len_as_integer;
use crate::semsg;
use crate::types::{Error, Failed, Object, TypVal, VAR_DICT, VAR_LIST, VAR_NUMBER, VAR_UNKNOWN};
use core::ffi::c_int;

/// A cleared API error, the shape every `api_*` out-parameter starts in.
const NO_ERROR: Error = Error::none();

/// The `{index}` argument the `ctxget`/`ctxset` pair share: absent means 0,
/// a Number is taken as-is, anything else is rejected with `what`.
fn context_index(tv: Option<&TypVal>, what: &str) -> Result<usize, Failed> {
    match tv {
        None => Ok(0),
        Some(tv) if tv.v_type() == VAR_NUMBER => {
            Ok(usize::try_from(tv.number_or_zero()).unwrap_or(usize::MAX))
        }
        Some(_) => {
            semsg!("E475: Invalid argument: {what}");
            Err(Failed)
        }
    }
}

/// Report a context index past the bottom of the stack.
fn out_of_bounds() {
    semsg!("E475: Invalid value for argument index: out of bounds");
}

/// `ctxget([{index}])` — the context at `index` as a Dictionary.
pub fn f_ctxget(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let Ok(index) = context_index(args.first(), "expected nothing or a Number as an argument")
    else {
        return;
    };
    let Some(ctx_dict) = with_ctx(index, |ctx| ctx_to_dict(ctx)) else {
        out_of_bounds();
        return;
    };
    let mut err = NO_ERROR;
    *result = TypVal::from(Object::dict(ctx_dict));
    err.clear();
}

/// `ctxpop()` — restore and drop the context on top of the stack.
pub fn f_ctxpop(_args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    if !ctx_restore(None, CTX_ALL) {
        semsg!("Context stack is empty");
    }
}

/// `ctxpush([{types}])` — push a context holding the named parts of the
/// editor state, or all of them when no list is given.
pub fn f_ctxpush(args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    let types = match args.first().map_or(VAR_UNKNOWN, TypVal::v_type) {
        VAR_LIST => {
            let mut types: c_int = 0;
            for li in list_iter(args[0].list_ref()) {
                let tv = &li.li_tv;
                // An unrecognised name is silently ignored, as is a
                // non-String item.
                // A null `v_string` is the empty string, which matches
                // no name; `strequal` answered the same for it.
                if let Some(name) = tv.string_ref() {
                    types |= match name.as_bytes() {
                        b"regs" => kCtxRegs,
                        b"jumps" => kCtxJumps,
                        b"bufs" => kCtxBufs,
                        b"gvars" => kCtxGVars,
                        b"sfuncs" => kCtxSFuncs,
                        b"funcs" => kCtxFuncs,
                        _ => 0,
                    };
                }
            }
            types
        }
        VAR_UNKNOWN => CTX_ALL,
        _ => {
            semsg!("E475: Invalid argument: expected nothing or a List as an argument");
            return;
        }
    };
    ctx_save(None, types);
}

/// `ctxset({context} [, {index}])` — replace the context at `index`.
pub fn f_ctxset(args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    if args[0].v_type() != VAR_DICT {
        semsg!("E475: Invalid argument: expected dictionary as first argument");
        return;
    }
    let msg = "expected nothing or a Number as second argument";
    let Ok(index) = context_index(args.get(1), msg) else {
        return;
    };
    if index >= ctx_size() {
        out_of_bounds();
        return;
    }
    // The conversion reports its problems through `did_emsg`; the caller's
    // flag is restored whatever happens here.
    let save_did_emsg = did_emsg.get();
    did_emsg.set(0);
    let dict = Object::from(&args[0])
        .into_dict()
        .expect("a VAR_DICT converts to a Dict object");
    let mut read = Context::default();
    match ctx_from_dict(dict, &mut read) {
        Err(e) => {
            // The message is whatever the API layer produced.
            let msg = msg_cstr(e.message_or_empty());
            semsg!("{msg}");
        }
        // Replacing the context releases the one it replaces.
        Ok(_) => {
            let _ = with_ctx(index, |ctx| *ctx = read);
        }
    }
    did_emsg.set(save_did_emsg);
}

/// `ctxsize()` — how many contexts are on the stack.
pub(crate) fn f_ctxsize(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(len_as_integer(ctx_size()));
}
