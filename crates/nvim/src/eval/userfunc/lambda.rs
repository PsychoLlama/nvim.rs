//! Lambdas, closures and partials -- the anonymous half.
//!
//! `get_lambda_tv` parses `{x -> expr}` into a real `UserFunc` with a
//! generated `<lambda>N` name and, if it captured anything, a reference to
//! the funccall it was made in (`register_closure`).  `make_partial` is the
//! other way a callable carries state: a bound dictionary, bound arguments,
//! or both.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::eval::Parsed;
use crate::eval::typval::{DictRef, PartialRef};
use crate::memory::ThinCString;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use crate::strings::find_bytes;
use core::cell::{Cell, RefCell};
use core::ffi::c_int;
use std::rc::Rc;

use super::*;
use crate::types::{Failed, FuncBody, FuncProfile, Refcount, ScriptCtx};

/// Give `func` the funccall that is running as its scope, so that the locals
/// it closed over stay alive for as long as it does. Nothing happens when no
/// funccall is running.
pub(crate) fn register_closure(func: &Rc<UserFunc>) {
    if func.scoped.get() == current_fc_id() {
        return; // no change
    }
    funccal_unref(func.scoped.get(), func, false);
    let frame = current_fc();
    func.scoped.set(frame.as_ref().map(|frame| frame.id));
    if let Some(frame) = frame {
        frame.refcount.set(frame.refcount.get() + 1);
        frame.ufuncs.borrow_mut().push(Rc::downgrade(func));
    }
}

/// The name of the next lambda.
fn get_lambda_name() -> Vec<u8> {
    static LAMBDA_NO: GlobalCell<c_int> = GlobalCell::new(0);
    LAMBDA_NO.set(LAMBDA_NO.get() + 1);
    format!("<lambda>{}", LAMBDA_NO.get()).into_bytes()
}

/// A new function called `name`, with nothing in it yet: no body, no
/// flags, no counted holder. A script-local (mangled) name gets the
/// printable `<SNR>123_name` beside it.
pub(crate) fn alloc_ufunc(name: &[u8]) -> UserFunc {
    let name_exp = (name.first() == Some(&K_SPECIAL)).then(|| {
        let mut text = b"<SNR>".to_vec();
        text.extend_from_slice(name.get(3..).unwrap_or_default());
        ThinCString::from_vec(text)
    });
    UserFunc {
        name: ThinCString::from_bytes(name),
        name_exp,
        flags: Cell::new(FuncFlags::NONE),
        calls: Cell::new(0),
        cleared: Cell::new(false),
        refcount: Refcount::zero(),
        body: RefCell::new(Rc::new(FuncBody::default())),
        luaref: Cell::new(LUA_NOREF),
        script_ctx: Cell::new(ScriptCtx::default()),
        scoped: Cell::new(None),
        prof: RefCell::new(FuncProfile::default()),
    }
}

/// Parse a lambda expression at the cursor into a partial in `result`.
///
/// Answers [`Parsed::NotThis`] when it is a dictionary or a `{expr}` rather
/// than a lambda -- which is decided by whether an `->` follows a legal
/// argument list.
pub(crate) fn get_lambda_tv(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
) -> Result<Parsed, Failed> {
    let mut newargs: Vec<Box<[u8]>> = Vec::new();
    let mut varargs = false;
    // The enclosing lambda's capture flag, put back when this one is done.
    // Only an evaluating lambda starts its own: a skipped one leaves the
    // flag alone, so what it reads still counts for the enclosing lambda.
    let enclosing_uses_locals = LAMBDA_USES_LOCALS.get();
    let text = cursor.text();

    // First, check whether this is a lambda expression at all: an "->"
    // must follow a well-formed argument list.
    let mut look = Cursor::new(text);
    look.set_offset(cursor.offset() + 1);
    look.skip_white();
    let looks_like = get_function_args(&mut look, b'-', None, None, None, true);
    if looks_like.is_err() || look.byte() != b'>' {
        return Ok(Parsed::NotThis);
    }

    // Neither the function nor the partial escapes the arm that builds
    // them, which is why upstream's `assert(fp == NULL)` at its error label
    // holds.
    let names = evaluate.then_some(&mut newargs);
    let parsed = parse_lambda(cursor, names, &mut varargs, evaluate);
    if let Ok(lambda) = &parsed
        && evaluate
    {
        let body = &text[lambda.body.clone()];
        let args = core::mem::take(&mut newargs);
        let part = make_lambda(body, args, lambda.uses_locals);
        result.write_partial(Some(PartialRef::new(part)));
    }

    drop(newargs);
    if evaluate {
        LAMBDA_USES_LOCALS.set(enclosing_uses_locals);
    }
    parsed.map(|_| Parsed::Done)
}

/// A lambda's text, read: where its expression is, and whether it reads
/// the enclosing function's locals.
#[derive(Debug)]
struct LambdaText {
    body: core::ops::Range<usize>,
    uses_locals: bool,
}

/// Read a lambda from its `{` at the cursor to its `}`: the arguments (kept
/// in `names` when given), the `->` and the expression, skipped. Malformed
/// is an error: only a missing `}` is said here (E451), the rest by what
/// read it.
fn parse_lambda(
    cursor: &mut Cursor<'_>,
    names: Option<&mut Vec<Box<[u8]>>>,
    varargs: &mut bool,
    evaluate: bool,
) -> Result<LambdaText, Failed> {
    // Parse the arguments again, this time keeping them.
    cursor.bump(1);
    cursor.skip_white();
    let read = get_function_args(cursor, b'-', names, Some(varargs), None, false);
    if read.is_err() || cursor.byte() != b'>' {
        return Err(Failed);
    }

    // Set up a flag for checking local variables and arguments.
    if evaluate {
        LAMBDA_USES_LOCALS.set(Some(false));
    }

    // Get the start and the end of the expression.
    cursor.bump(1);
    cursor.skip_white();
    let start = cursor.offset();
    let mut skipped = TV_INITIAL_VALUE;
    let ret = eval1(cursor, &mut skipped, false);
    let end = cursor.offset();
    ret?;
    let uses_locals = evaluate && LAMBDA_USES_LOCALS.get() == Some(true);

    cursor.skip_white();
    if cursor.byte() != b'}' {
        let rest = msg_bytes(cursor.rest());
        semsg!("E451: Expected }}: {rest}");
        return Err(Failed);
    }
    cursor.bump(1);
    Ok(LambdaText {
        body: start..end,
        uses_locals,
    })
}

/// Define the function for a lambda whose expression is `body`, taking
/// `args`, and answer the partial that holds it.
fn make_lambda(body: &[u8], args: Vec<Box<[u8]>>, uses_locals: bool) -> Partial {
    let mut flags = FuncFlags::NONE;
    let name = get_lambda_name();
    let func = Rc::new(alloc_ufunc(&name));

    // The body is the expression with "return " in front of it.
    let mut line = Vec::with_capacity(b"return ".len() + body.len());
    line.extend_from_slice(b"return ");
    line.extend_from_slice(body);
    if find_bytes(body, b"a:").is_none() {
        // No a: variables are used for sure.
        flags |= FuncFlags::NOARGS;
    }

    // Fresh from `alloc_ufunc`, at zero: the one holder is the partial.
    func.retain();
    // A lambda's name is new, so the table cannot refuse it; were it
    // to, the partial still holds the function.
    let _ = add_func(func.clone());
    // Every lambda takes any number of arguments.
    *func.body.borrow_mut() = Rc::new(FuncBody {
        args,
        def_args: Vec::new(),
        lines: vec![Some(line.into_boxed_slice())],
        varargs: true,
    });
    if current_fc_id().is_some() && uses_locals {
        flags |= FuncFlags::CLOSURE;
        register_closure(&func);
    } else {
        func.scoped.set(None);
    }

    if prof_def_func() {
        func_do_profile(&func);
    }
    if SANDBOX.get() != 0 {
        flags |= FuncFlags::SANDBOX;
    }
    func.flags.set(flags);
    func.calls.set(0);
    let mut sctx = CURRENT_SCTX.get();
    sctx.sc_lnum += (sourcing_lnum() - 1).get();
    func.script_ctx.set(sctx);

    Partial {
        pt_func: Some(func),
        ..Partial::empty()
    }
}

/// Bind `selfdict` to the Funcref in `result`: `dict.Func` read out of
/// `dict`. Not for a partial that was bound explicitly (`pt_auto` clear).
pub(crate) fn set_selfdict(result: &mut TypVal, selfdict: &DictRef) {
    if let Some(pt) = result.partial_ref()
        && !pt.pt_auto
        && pt.pt_dict.is_some()
    {
        return;
    }
    make_partial(selfdict, result);
}

/// Turn `dict.Func` into a partial that binds `selfdict`, when `Func` was
/// declared with the `dict` attribute.
///
/// `result` holds the funcref just read and `selfdict` is the dictionary it
/// came out of.
pub fn make_partial(selfdict: &DictRef, result: &mut TypVal) {
    let held = result.partial_ref();
    let func = if let Some(func) = held.and_then(|held| held.pt_func.clone()) {
        Some(func)
    } else {
        let fname = if result.v_type() == VAR_FUNC || result.v_type() == VAR_STRING {
            result.text_or_name().map(|name| name.as_bytes())
        } else {
            held.and_then(|held| held.pt_name.as_ref())
                .map(|name| name.as_bytes())
        };
        match fname {
            // There is no point binding a dict to a NULL function, just
            // create a function reference.
            None => {
                result.write_func_name(None);
                None
            }
            // Translate "s:func" to the stored function name.
            Some(fname) => find_func(&fname_trans_sid(fname).0),
        }
    };

    if !func.is_some_and(|func| func.has_flag(FuncFlags::DICT)) {
        return;
    }
    let mut part = Partial {
        pt_dict: Some(selfdict.clone()),
        pt_auto: true,
        ..Partial::empty()
    };
    if result.v_type() == VAR_FUNC || result.v_type() == VAR_STRING {
        // Just a function: take over the function name and use selfdict.
        part.pt_name = if result.v_type() == VAR_STRING {
            result.take_string()
        } else {
            result.take_func_name()
        };
    } else {
        // Partial: copy the function name, use selfdict and copy the
        // arguments.  Neither can be taken over, because the partial may
        // be referenced elsewhere.
        let ret_pt = result
            .take_partial()
            .expect("a partial value with a function");
        if let Some(name) = &ret_pt.pt_name {
            func_ref_name(name);
            part.pt_name = Some(name.clone());
        } else if let Some(func) = &ret_pt.pt_func {
            func_ptr_ref(func);
            part.pt_func = Some(func.clone());
        }
        part.pt_argv = ret_pt.pt_argv.clone();
        drop(ret_pt);
    }
    result.write_partial(Some(PartialRef::new(part)));
}

/// Wrap a Lua reference in a `UserFunc`, so that Vimscript can call it by
/// name.  Answers that name.
///
/// The new function takes `luaref` over.
pub fn register_luafunc(luaref: LuaRef) -> ThinCString {
    let func = Rc::new(alloc_ufunc(&get_lambda_name()));
    // Fresh from `alloc_ufunc`, at zero: the one holder is the partial.
    func.retain();
    *func.body.borrow_mut() = Rc::new(FuncBody {
        varargs: true,
        ..FuncBody::default()
    });
    func.flags.set(FuncFlags::LUAREF);
    func.calls.set(0);
    func.script_ctx.set(CURRENT_SCTX.get());
    func.luaref.set(luaref);

    let name = func.name().clone();
    let _ = add_func(func);
    name
}
