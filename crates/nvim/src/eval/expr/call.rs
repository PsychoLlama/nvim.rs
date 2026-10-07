//! Calling something: a function name, a method, a lambda or a partial.
//!
//! All three entry points share one shape. The value already in `rettv` is
//! the *callee* (or, for `->`, the base the method is applied to); it is
//! moved into a local, `rettv` is blanked so the call can fill it, and the
//! local is cleared afterwards — after the call, so that a function may
//! delete the Funcref it is being reached through while its own arguments
//! are still being evaluated.

#![forbid(unsafe_code)]

use crate::eval::Parsed;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::memory::XString;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use core::ffi::CStr;

use crate::eval::typval::{DictRef, tv_clear};
use crate::eval::userfunc::{CallWith, deref_func_name_owned, get_func_tv, get_lambda_tv};
use crate::eval::vars::{check_vars, lua_partial};
use crate::eval::{
    Cursor, e_cannot_use_partial_here, e_empty_function_name, e_nowhitespace, eval7, get_name_len,
    is_luafunc, luafunc_name_end,
};
use crate::ex_eval::aborting;
use crate::memory::ThinCString;
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::types::{Failed, TypVal, VAR_FUNC, VAR_PARTIAL, VAR_STRING, VAR_UNKNOWN};

/// A freshly declared typval.
const UNSET_TV: TypVal = TV_INITIAL_VALUE;

/// Call a function by name, having parsed the name but not the arguments,
/// which the cursor is on.
pub(crate) fn eval_func(
    cursor: &mut Cursor<'_>,
    name: &[u8],
    result: &mut TypVal,
    evaluate: bool,
    basetv: Option<&mut TypVal>,
) -> Result<(), Failed> {
    if !evaluate {
        check_vars(name);
    }
    // A copy: the call may re-enter the evaluator and free the variable the
    // name was read out of.
    let (resolved, partial, found_var) = deref_func_name_owned(name, !evaluate);

    let with = CallWith {
        partial: partial.as_ref(),
        basetv,
        found_var,
        ..CallWith::at_cursor(evaluate)
    };
    let mut ret = get_func_tv(resolved.as_cstr(), None, result, cursor, with);
    drop((resolved, partial));

    // While skipping, a name that was never resolved still has to look
    // like a Funcref so the subscript handling can go on.
    if result.v_type() == VAR_UNKNOWN && !evaluate && cursor.byte() == b'(' {
        result.write_func_name(Some(ThinCString::empty()));
    }
    if evaluate && aborting() {
        if ret.is_ok() {
            tv_clear(result);
        }
        ret = Err(Failed);
    }
    ret
}

/// Call the value in `result` — a name, a Funcref or a partial — with the
/// cursor on the `(`, and leave the result in `result`.
///
/// `basetv` is the `expr` of `expr->method()`, passed as the first
/// argument; `lua_name` is where in the text the name of the `v:lua.`
/// function a partial stands for starts, running to the cursor.
pub(crate) fn call_func_rettv(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
    selfdict: Option<&DictRef>,
    basetv: Option<&mut TypVal>,
    lua_name: Option<usize>,
) -> Result<(), Failed> {
    // The callee moves out of `result` so the call can fill it. It is
    // cleared at the end rather than here: the arguments are evaluated
    // in between and may delete the Funcref they name.
    let mut functv = UNSET_TV;
    // A `v:lua.` name is not terminated: it runs to the cursor, and only
    // that much is called -- though a message quotes the rest of the text.
    let lua_text: Option<XString>;
    let mut name_len = None;
    let funcname: &CStr;

    if evaluate {
        functv = result.take();
        if functv.v_type() == VAR_PARTIAL {
            if is_luafunc(functv.partial_or_null()) {
                let start = lua_name.unwrap_or(cursor.offset());
                lua_text = Some(XString::from_bytes(&cursor.text()[start..]));
                name_len = Some(cursor.offset() - start);
                funcname = lua_text.as_ref().map_or(c"", XString::as_cstr);
            } else {
                funcname = functv.callable_name().unwrap_or(c"");
            }
        } else {
            // Not a partial, so the value holds a name: `VAR_FUNC`'s or
            // `VAR_STRING`'s. Anything else has no name and reports the
            // empty-name error just below.
            let name = match functv.v_type() {
                VAR_FUNC => functv.callable_name(),
                VAR_STRING => functv.string_cstr(),
                _ => None,
            };
            match name {
                Some(name) if !name.is_empty() => funcname = name,
                _ => {
                    emsg(gettext(e_empty_function_name));
                    tv_clear(&mut functv);
                    return Err(Failed);
                }
            }
        }
    } else {
        funcname = c"";
    }

    let partial = match &functv {
        TypVal::Partial(partial) => partial.as_ref(),
        _ => None,
    };
    let with = CallWith {
        partial,
        selfdict,
        basetv,
        ..CallWith::at_cursor(evaluate)
    };
    let ret = get_func_tv(funcname, name_len, result, cursor, with);

    if evaluate {
        tv_clear(&mut functv);
    }
    ret
}

/// `expr->{lambda}()`, with the cursor on the `-`.
pub(crate) fn eval_lambda(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
    verbose: bool,
) -> Result<(), Failed> {
    cursor.bump(2); // skip over the `->`
    let mut base = result.take();

    if get_lambda_tv(cursor, result, evaluate) != Ok(Parsed::Done) {
        // Upstream leaves `base` to the caller here; it is this frame's,
        // and goes with it.
        return Err(Failed);
    }
    let ret = if cursor.byte() != b'(' {
        if verbose {
            let mut after = Cursor::new(cursor.text());
            after.bump(cursor.offset());
            after.skip_white();
            if after.byte() == b'(' {
                emsg(gettext(e_nowhitespace));
            } else {
                semsg!("E107: Missing parentheses: lambda");
            }
        }
        tv_clear(result);
        Err(Failed)
    } else {
        call_func_rettv(cursor, result, evaluate, None, Some(&mut base), None)
    };

    if evaluate {
        tv_clear(&mut base);
    }
    ret
}

/// `expr->name()`, with the cursor on the `-`.
pub(crate) fn eval_method(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
    verbose: bool,
) -> Result<(), Failed> {
    let text = cursor.text();
    cursor.bump(2); // skip over the `->`
    let mut base = result.take();

    // Locate the method name.
    let start = cursor.offset();
    let (lua_name, alias, len) = if cursor.rest().starts_with(b"v:lua.") {
        let at = start + 6;
        cursor.bump(6 + luafunc_name_end(&text[at..]));
        cursor.skip_white(); // so trailing whitespace is detectable
        (Some(at), None, cursor.offset() - at)
    } else {
        let (scanned, alias) = get_name_len(cursor, evaluate, true);
        (None, alias, usize::try_from(scanned).unwrap_or(0))
    };
    // The name as the call will use it: the curly-brace expansion, or the
    // text, or -- for an indirect callee -- what that evaluated to.
    let mut name: Vec<u8> = match (&alias, lua_name) {
        (Some(alias), _) => alias.to_vec(),
        (None, Some(at)) => text[at..at + len].to_vec(),
        (None, None) => text[start..start + len].to_vec(),
    };

    let mut ret = Ok(());
    if len == 0 {
        if verbose {
            if lua_name.is_none() {
                emsg(gettext(c"E260: Missing name after ->"));
            } else {
                let name = msg_bytes(&text[start..]);
                semsg!("E15: Invalid expression: \"{name}\"");
            }
        }
        ret = Err(Failed);
    } else {
        cursor.skip_white();

        // No `(` immediately after, but one further on: this can be
        // "dict.Func()", "list[nr]" and so on. Anything where the `(`
        // is part of the expression itself is not handled.
        let paren = (cursor.byte() != b'(' && lua_name.is_none() && alias.is_none())
            .then(|| {
                cursor
                    .rest()
                    .iter()
                    .position(|&b| b == b'(')
                    .map(|at| cursor.offset() + at)
            })
            .flatten();
        if let Some(paren) = paren {
            // The callee alone is evaluated: the text up to the `(`.
            let mut callee_cursor = Cursor::new(&text[..paren]);
            callee_cursor.bump(start);
            let mut callee = UNSET_TV;
            if eval7(&mut callee_cursor, &mut callee, evaluate, false).is_err() {
                cursor.set_offset(start + len);
                ret = Err(Failed);
            } else {
                let end = callee_cursor.offset();
                cursor.set_offset(end);
                callee_cursor.skip_white();
                if callee_cursor.byte() != 0 {
                    if verbose {
                        // Quoted up to the `(`, where the callee's text ends.
                        let at = msg_bytes(&text[end..paren]);
                        semsg!("E488: Trailing characters: {at}");
                    }
                    ret = Err(Failed);
                } else if callee.func_name().is_some() {
                    name = callee
                        .callable_name()
                        .map_or(&[][..], CStr::to_bytes)
                        .to_vec();
                } else if callee.v_type() == VAR_PARTIAL && !callee.partial_or_null().is_null() {
                    let (dict, args) = callee.partial_binding();
                    if !args.is_empty() || dict.is_some() {
                        if verbose {
                            emsg(gettext(e_cannot_use_partial_here));
                        }
                        ret = Err(Failed);
                    } else {
                        name = callee
                            .callable_name()
                            .map_or(&[][..], CStr::to_bytes)
                            .to_vec();
                    }
                } else {
                    if verbose {
                        let name = msg_bytes(&text[start..paren]);
                        semsg!("E1085: Not a callable type: {name}");
                    }
                    ret = Err(Failed);
                }
            }
            tv_clear(&mut callee);
        }

        if ret.is_ok() {
            let mut basep = Some(&mut base);
            if cursor.byte() != b'(' {
                if verbose {
                    let shown = alias.as_ref().map_or(&text[start..], |alias| &alias[..]);
                    let name = msg_bytes(shown);
                    semsg!("E107: Missing parentheses: {name}");
                }
                ret = Err(Failed);
            } else if matches!(text[cursor.offset() - 1], b' ' | b'\t') {
                if verbose {
                    emsg(gettext(e_nowhitespace));
                }
                ret = Err(Failed);
            } else if lua_name.is_some() {
                if evaluate {
                    result.write_partial(lua_partial());
                }
                let base = basep.take();
                ret = call_func_rettv(cursor, result, evaluate, None, base, lua_name);
            } else {
                let base = basep.take();
                ret = eval_func(cursor, &name, result, evaluate, base);
            }
        }
    }

    // Clear the Funcref afterwards, so that deleting it while its own
    // arguments are being evaluated is possible (test55).
    if evaluate {
        tv_clear(&mut base);
    }
    ret
}
