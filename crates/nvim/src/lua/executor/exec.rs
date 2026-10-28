//! Running Lua from Vimscript, and calling a `LuaRef` back.
//!
//! [`nlua_typval_eval`]/[`typval_call_lua`] are `luaeval()` and `v:lua`;
//! [`nlua_exec`] runs a chunk; [`nlua_call_ref_ctx`] is the callback path
//! every api-registered Lua function is invoked through, and
//! `nlua_call_pop_retval` is the shared conversion of whatever it left on
//! the stack, governed by the `LuaRetMode` the caller asked for.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use core::ffi::{CStr, c_int};
use std::ffi::CString;

use super::{
    FCERR_NONE, FCERR_OTHER, fast_pcall, kRetLuaref, kRetMulti, kRetNilBool, kRetObject,
    main_stack, nlua_error, nlua_pcall,
};
use crate::ex_cmds::check_secure;
use crate::lua::converter::{pop_object, pop_typval, push_object, push_typval};
use crate::lua::ffi::LUA_MULTRET;
use crate::lua::stack::{At, Kind, Stack};
use crate::message_fmt::msg_bytes_opt;
use crate::narrow::len_as_int;
use crate::types::{
    Array, Error, ErrorType, LuaRef, LuaRetMode, Object, TypVal, VAR_UNKNOWN, VarNumber,
    kErrorTypeException, kErrorTypeValidation,
};

/// `luaeval("expr")` becomes this chunk with the expression appended and a
/// `)` closed after it, so the expression is evaluated with `_A` bound to
/// the second argument.
const EVALHEADER: &[u8] = b"local _A=select(1,...) return (";
/// `v:lua.f(…)` becomes this, the name, and [`CALLSUFFIX`].
const CALLHEADER: &[u8] = b"return ";
const CALLSUFFIX: &[u8] = b"(...)";

/// The chunk name `luaeval()` errors carry.
const EVALNAME: &CStr = c"luaeval()";

/// `luaeval(str, arg)`.
pub fn nlua_typval_eval(expr: &[u8], arg: &TypVal, ret_tv: &mut TypVal) {
    let chunk = [EVALHEADER, expr, b")"].concat();
    nlua_typval_exec(
        &chunk,
        EVALNAME,
        core::slice::from_ref(arg),
        true,
        Some(ret_tv),
    );
}

/// `v:lua.name(...)`, `name` being the text after `v:lua.`.
pub(crate) fn typval_call_lua(name: &[u8], args: &[TypVal], result: &mut TypVal) {
    let chunk = [CALLHEADER, name, CALLSUFFIX].concat();
    nlua_typval_exec(&chunk, c"v:lua", args, false, Some(result));
}

/// The `customlist,v:lua.…` completion callback: the function `luaref`
/// called with the pattern, the whole line and the cursor column, answering
/// what it returned (or zero, once a failure has reported itself).
pub fn nlua_call_user_expand_func(
    luaref: LuaRef,
    pattern: &CStr,
    line: &CStr,
    col: c_int,
) -> TypVal {
    let mut stack = main_stack();
    stack.push_reference(luaref);
    stack.push_bytes(pattern.to_bytes());
    stack.push_bytes(line.to_bytes());
    stack.push_integer(col.try_into().unwrap_or_default());
    if nlua_pcall(&mut stack, 3, 1).is_err() {
        nlua_error(&mut stack, c"E5108: Lua function: %.*s");
        return TypVal::Number(0);
    }
    // A refusal has reported itself; the answer is then the zero
    // upstream left in the caller's slot.
    pop_typval(&mut stack).unwrap_or(TypVal::Number(0))
}

/// `require(module)` in the editor's state, through [`nlua_pcall`] so that a
/// failure carries its traceback.
///
/// Answers the error's text when it fails -- `None` for an error object that
/// is not a string -- and leaves the error on the stack, as upstream does.
pub(crate) fn nlua_require_module(module: &CStr) -> Result<(), Option<Vec<u8>>> {
    let mut stack = main_stack();
    stack.get_global(c"require");
    stack.push_bytes(module.to_bytes());
    if nlua_pcall(&mut stack, 1, 0).is_ok() {
        return Ok(());
    }
    // `lua_tolstring`'s C string: up to its first NUL.
    Err(stack
        .to_bytes(At::TOP)
        .map(|text| super::until_nul(text).to_vec()))
}

/// Run the Lua chunk `code` with `args`, answering its value converted as
/// `mode` says.
pub(crate) fn nlua_exec_cstr(code: &CStr, args: Array, mode: LuaRetMode) -> Result<Object, Error> {
    nlua_exec(code.to_bytes(), None, args, mode)
}

/// Load and run one chunk with `args` as its arguments, the chunk named
/// `name` in its messages.
///
/// `special` decides how a Vimscript value with no Lua image is pushed; a
/// `VAR_UNKNOWN` argument is `nil`, which is how a missing `luaeval()`
/// argument reaches the chunk.
pub(crate) fn nlua_typval_exec(
    chunk: &[u8],
    name: &CStr,
    args: &[TypVal],
    special: bool,
    ret_tv: Option<&mut TypVal>,
) {
    if check_secure() {
        if let Some(ret_tv) = ret_tv {
            ret_tv.write_number(0 as VarNumber);
        }
        return;
    }
    let mut stack = main_stack();
    if stack.load(chunk, Some(name)).is_err() {
        nlua_error(&mut stack, c"E5107: Lua: %.*s");
        return;
    }
    push_typval_args(&mut stack, args, special);
    if nlua_pcall(
        &mut stack,
        len_as_int(args.len()),
        c_int::from(ret_tv.is_some()),
    )
    .is_err()
    {
        nlua_error(&mut stack, c"E5108: Lua: %.*s");
        return;
    }
    if let Some(ret_tv) = ret_tv {
        ret_tv.overwrite(pop_typval(&mut stack).unwrap_or(TypVal::Number(0)));
    }
}

/// Push the arguments, with `VAR_UNKNOWN` standing for `nil`.
fn push_typval_args(stack: &mut Stack, args: &[TypVal], special: bool) {
    for arg in args {
        if arg.v_type() == VAR_UNKNOWN {
            stack.push_nil();
        } else {
            push_typval(stack, arg, special);
        }
    }
}

/// Run a `:lua` heredoc: `lines` joined by newlines, as the chunk `name`.
pub fn nlua_exec_lines(lines: &[CString], name: &CStr) {
    let mut code = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            code.push(b'\n');
        }
        code.extend_from_slice(line.to_bytes());
    }
    nlua_typval_exec(&code, name, &[], false, None);
}

/// Call a Lua function stored as a Vimscript Funcref.
pub fn typval_exec_lua_callable(lua_cb: LuaRef, argvars: &[TypVal], rettv: &mut TypVal) -> c_int {
    let mut stack = main_stack();
    stack.push_reference(lua_cb);
    push_typval_args(&mut stack, argvars, false);
    if nlua_pcall(&mut stack, len_as_int(argvars.len()), 1).is_err() {
        nlua_error(&mut stack, c"Lua callback: %.*s");
        return FCERR_OTHER.cast_signed();
    }
    rettv.overwrite(pop_typval(&mut stack).unwrap_or(TypVal::Number(0)));
    FCERR_NONE.cast_signed()
}

/// Run a chunk with api values as its arguments and its answer as one. The
/// chunk is named `chunkname` in its messages, `<nvim>` when that is absent
/// or empty.
///
/// The arguments are this call's: they are pushed and then released.
pub fn nlua_exec(
    code: &[u8],
    chunkname: Option<&CStr>,
    args: Array,
    mode: LuaRetMode,
) -> Result<Object, Error> {
    let mut stack = main_stack();
    let top = stack.top();
    let name = chunkname
        .filter(|name| !name.is_empty())
        .unwrap_or(c"<nvim>");
    if stack.load(code, Some(name)).is_err() {
        // The message stays on the stack, as upstream leaves it.
        return Err(lua_error_of(kErrorTypeValidation, &mut stack));
    }
    let argc = len_as_int(args.len());
    for item in args.iter() {
        push_object(&mut stack, item, false);
    }
    if nlua_pcall(&mut stack, argc, 1).is_err() {
        return Err(lua_error_of(kErrorTypeException, &mut stack));
    }
    nlua_call_pop_retval(&mut stack, mode, top)
}

/// The Lua error on top of the stack, which stays there.
fn lua_error_of(type_0: ErrorType, stack: &mut Stack) -> Error {
    let text = stack.to_bytes(At::TOP);
    Error::new(type_0, format_args!("Lua: {}", msg_bytes_opt(text)))
}

/// [`nlua_call_ref_ctx`] for a caller with nowhere to report to: a failing
/// callback shows its error rather than answering with one. `name`, when
/// given, is the first argument, as there.
pub fn nlua_call_ref_quiet(
    reference: LuaRef,
    name: Option<&CStr>,
    args: Array,
    mode: LuaRetMode,
) -> Object {
    // A reference is an index into the registry, read with a raw get: one
    // that names nothing pushes nil, which the protected call reports, as
    // `nlua_ref_is_function` relies on.
    nlua_call_ref_ctx(false, reference, name, args, mode, false).unwrap_or(Object::Nil)
}

/// [`nlua_call_ref_ctx`] outside a fast context, answering a failure to the
/// caller, who reports it.
pub(crate) fn nlua_call_ref_reporting(
    reference: LuaRef,
    name: Option<&CStr>,
    args: Array,
    mode: LuaRetMode,
) -> Result<Object, Error> {
    nlua_call_ref_ctx(false, reference, name, args, mode, true)
}

/// How many results `mode` wants off the call.
fn mode_ret(mode: LuaRetMode) -> c_int {
    if mode == kRetMulti { LUA_MULTRET } else { 1 }
}

/// Call the function `reference` refers to.
///
/// `name`, when given, is pushed as the *first* argument — that is how one
/// registered callback serves several event names. `fast` runs the call
/// through the luv path instead, which is the only one allowed inside a fast
/// callback and which reports rather than returns a failure. Without
/// `reports`, a failure is shown and answered as nil.
pub fn nlua_call_ref_ctx(
    fast: bool,
    reference: LuaRef,
    name: Option<&CStr>,
    args: Array,
    mode: LuaRetMode,
    reports: bool,
) -> Result<Object, Error> {
    let mut stack = main_stack();
    let top = stack.top();
    stack.push_reference(reference);
    let mut nargs = len_as_int(args.len());
    if let Some(name) = name {
        stack.push_bytes(name.to_bytes());
        nargs += 1;
    }
    for item in args.iter() {
        push_object(&mut stack, item, false);
    }

    if fast {
        if fast_pcall(&mut stack, nargs, mode_ret(mode), -1) < 0 {
            if !reports {
                return Ok(Object::Nil);
            }
            return Err(Error::exception(c"fast context failure"));
        }
    } else if nlua_pcall(&mut stack, nargs, mode_ret(mode)).is_err() {
        if !reports {
            // Nobody to report to: show it instead.
            nlua_error(&mut stack, c"Lua callback: %.*s");
            return Ok(Object::Nil);
        }
        return Err(lua_error_of(kErrorTypeException, &mut stack));
    }
    nlua_call_pop_retval(&mut stack, mode, top)
}

/// Convert whatever the call left on the stack, and pop it.
///
/// A `nil` answer is nil whatever the mode asked for — except `kRetMulti`,
/// where it is one element of the list. The results sit above `pretop`.
fn nlua_call_pop_retval(stack: &mut Stack, mode: LuaRetMode, pretop: At) -> Result<Object, Error> {
    if mode != kRetMulti && stack.kind(At::TOP) == Kind::Nil {
        stack.pop(1);
        return Ok(Object::Nil);
    }
    match mode {
        kRetNilBool => {
            let bool_value = stack.to_boolean(At::TOP);
            stack.pop(1);
            Ok(Object::boolean(bool_value))
        }
        kRetLuaref => {
            let reference = stack.reference(At::TOP);
            stack.pop(1);
            Ok(Object::luaref(reference))
        }
        kRetObject => pop_object(stack, false),
        kRetMulti => {
            let nres = stack.top().index() - pretop.index();
            // The values come off the stack topmost first, so they are
            // collected and then turned back into call order.
            let mut res: Vec<Object> = Vec::with_capacity(usize::try_from(nres).unwrap_or(0));
            for _ in 0..nres {
                res.push(pop_object(stack, false)?);
            }
            res.reverse();
            Ok(Object::array(Array::from(res)))
        }
        _ => unreachable!(),
    }
}
