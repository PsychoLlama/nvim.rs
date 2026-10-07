//! The argument list: parsing it and checking it.
//!
//! `get_function_args` reads the `(a, b = expr, ...)` of a definition once,
//! at definition time, keeping each default as unevaluated source; the
//! `get_func_arg*` pair reads the arguments of a *call*.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::semsg;
use core::ffi::c_int;

use super::*;
use crate::types::Failed;

/// Read one argument name at the cursor and append a copy of it to
/// `newargs`, leaving the cursor after it.
///
/// Answers false, with the cursor where it was, when what is there cannot be
/// one: empty, starting with a digit, a duplicate of an earlier argument, or
/// one of the two names the `a:` scope already gives a meaning.
fn one_function_arg(
    cursor: &mut Cursor<'_>,
    newargs: Option<&mut Vec<Box<[u8]>>>,
    skip: bool,
) -> bool {
    let rest = cursor.rest();
    let len = rest
        .iter()
        .take_while(|&&b| ascii_isident(c_int::from(b)))
        .count();
    let name = &rest[..len];
    // `isdigit()` is one of the ctype predicates the C standard fixes to
    // ASCII in every locale, so this really is the same test.
    if len == 0 || name[0].is_ascii_digit() || name == b"firstline" || name == b"lastline" {
        if !skip {
            let arg = msg_bytes(rest);
            semsg!("E125: Illegal argument: {arg}");
        }
        return false;
    }
    if let Some(newargs) = newargs {
        if newargs.iter().any(|earlier| **earlier == *name) {
            let shown = msg_bytes(name);
            semsg!("E853: Duplicate argument name: {shown}");
            return false;
        }
        newargs.push(name.into());
    }
    cursor.bump(len);
    true
}

/// Parse a definition's argument list at the cursor, up to and including
/// `endchar`, and leave the cursor after it; on an error it stays put.
///
/// Fills `newargs` with the names, `default_args` with the *source* of each
/// `= expr` default (evaluated afresh on every call, not here) and `varargs`
/// with whether a `...` was seen.  Any of the three may be `None`, which is
/// how a caller that only wants to skip the list says so.
pub(crate) fn get_function_args(
    cursor: &mut Cursor<'_>,
    endchar: u8,
    mut newargs: Option<&mut Vec<Box<[u8]>>>,
    mut varargs: Option<&mut bool>,
    mut default_args: Option<&mut Vec<Box<[u8]>>>,
    skip: bool,
) -> Result<(), Failed> {
    let mut mustend = false;
    let start = cursor.offset();
    let text = cursor.text();
    if let Some(newargs) = newargs.as_deref_mut() {
        newargs.clear();
    }
    if let Some(default_args) = default_args.as_deref_mut() {
        default_args.clear();
    }
    if let Some(varargs) = varargs.as_deref_mut() {
        *varargs = false;
    }

    // Isolate the arguments: "arg1, arg2, ...)".
    let mut any_default = false;
    let closed = 'parse: {
        while cursor.byte() != endchar {
            if cursor.rest().starts_with(b"...") {
                if let Some(varargs) = varargs.as_deref_mut() {
                    *varargs = true;
                }
                cursor.bump(3);
                mustend = true;
            } else {
                if !one_function_arg(cursor, newargs.as_deref_mut(), skip) {
                    break;
                }
                let mut after = Cursor::new(text);
                after.set_offset(cursor.offset());
                after.skip_white();
                if after.byte() == b'=' && default_args.is_some() {
                    let mut rettv = TV_INITIAL_VALUE;
                    any_default = true;
                    cursor.skip_white();
                    cursor.bump(1);
                    cursor.skip_white();
                    let expr = cursor.offset();
                    if eval1(cursor, &mut rettv, false).is_ok() {
                        // The default is kept as source, and the walk goes
                        // on from its end: the blanks are read again below.
                        let mut end = cursor.offset();
                        while end > expr && matches!(text[end - 1], b' ' | b'\t') {
                            end -= 1;
                        }
                        cursor.set_offset(end);
                        if let Some(default_args) = default_args.as_deref_mut() {
                            default_args.push(text[expr..end].into());
                        }
                    } else {
                        mustend = true;
                    }
                } else if any_default {
                    let fmt = c"E989: Non-default argument follows default argument";
                    emsg(gettext(fmt));
                    mustend = true;
                }
                let white = matches!(cursor.byte(), b' ' | b'\t');
                let mut after = Cursor::new(text);
                after.set_offset(cursor.offset());
                after.skip_white();
                if white && after.byte() == b',' {
                    if !skip {
                        let at = msg_bytes(cursor.rest());
                        semsg!("E1068: No white space allowed before ',': {at}");
                        break 'parse false;
                    }
                    cursor.skip_white();
                }
                if cursor.byte() == b',' {
                    cursor.bump(1);
                } else {
                    mustend = true;
                }
            }
            cursor.skip_white();
            if mustend && cursor.byte() != endchar {
                if !skip {
                    let at = msg_bytes(&text[start..]);
                    semsg!("E475: Invalid argument: {at}");
                }
                break;
            }
        }
        cursor.byte() == endchar
    };
    if closed {
        cursor.bump(1);
        return Ok(());
    }
    cursor.set_offset(start);

    if let Some(newargs) = newargs {
        newargs.clear();
    }
    if let Some(default_args) = default_args {
        default_args.clear();
    }
    Err(Failed)
}

/// Evaluate the arguments of a call, from the `(` at the cursor to its `)`.
///
/// Stops at `MAX_FUNC_ARGS` less whatever a partial has already bound.
pub(crate) fn get_func_arguments(
    cursor: &mut Cursor<'_>,
    evaluate: bool,
    partial_argc: c_int,
    args: &mut [TypVal],
    argcount: &mut usize,
) -> Result<(), Failed> {
    let mut ret = Ok(());
    let room = usize::try_from(MAX_FUNC_ARGS - partial_argc).unwrap_or(0);
    while *argcount < room {
        // skip the '(' or ','
        cursor.bump(1);
        cursor.skip_white();
        if matches!(cursor.byte(), b')' | b',') || cursor.byte() == 0 {
            break;
        }
        if eval1(cursor, &mut args[*argcount], evaluate).is_err() {
            ret = Err(Failed);
            break;
        }
        *argcount += 1;
        if cursor.byte() != b',' {
            break;
        }
    }
    cursor.skip_white();
    if cursor.byte() == b')' {
        cursor.bump(1);
    } else {
        ret = Err(Failed);
    }
    ret
}

/// How many arguments `name` takes: required, optional, and whether it also
/// takes a `...`. `None` when there is no such function.
pub(crate) fn get_func_arity(name: &[u8]) -> Option<(c_int, c_int, bool)> {
    if let Some(fdef) = find_builtin(name) {
        // An open-ended builtin takes as many as the evaluator will pass.
        let argcount = fdef.arity.max().map_or(MAX_FUNC_ARGS, c_int::from);
        let min_argcount = c_int::from(fdef.arity.min());
        return Some((min_argcount, argcount - min_argcount, false));
    }
    let (fname, error) = fname_trans_sid(name);
    if error != FCERR_NONE {
        return None;
    }
    let body = find_func(&fname)?.body();
    let declared = c_int::try_from(body.args.len()).unwrap_or(c_int::MAX);
    let defaults = c_int::try_from(body.def_args.len()).unwrap_or(c_int::MAX);
    let min_argcount = declared - defaults;
    Some((min_argcount, declared - min_argcount, body.varargs))
}

/// Whether `argcount` arguments can be given to `func`: `FCERR_UNKNOWN` when
/// they can, one of `FCERR_TOOFEW`/`FCERR_TOOMANY` when they cannot.
pub(crate) fn check_user_func_argcount(func: &UserFunc, argcount: c_int) -> c_int {
    let body = func.body();
    let regular_args = c_int::try_from(body.args.len()).unwrap_or(c_int::MAX);
    let defaults = c_int::try_from(body.def_args.len()).unwrap_or(c_int::MAX);
    if argcount < regular_args - defaults {
        FCERR_TOOFEW
    } else if !body.varargs && argcount > regular_args {
        FCERR_TOOMANY
    } else {
        FCERR_UNKNOWN
    }
}
