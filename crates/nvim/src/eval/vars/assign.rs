//! `:let` -- parsing the targets and performing the assignment.
//!
//! [`ex_let`] splits the command, [`ex_let_vars`] deals with the
//! `[a, b; rest]` unpack, and the four `ex_let_*` below it are one per kind
//! of target: a variable, an environment variable, an option and a register.
//! The last three implement the compound operators themselves and never
//! reach `set_var_lval`.
//!
//! Every target is read out of the command line by offset: the line is not
//! written to, so a name is measured where it stands rather than cut there.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr::{self, byte_at};
use crate::guard::Suppress;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use crate::types::CmdIdx;
use core::ffi::{CStr, c_int};
use std::ffi::CString;

use super::{
    ScopeLister, clear_local, e_double_semicolon_in_list_of_variables, e_letunexp, emsg_static,
    heredoc_get, kGRegExprSrc, list_arg_vars, list_buf_vars, list_func_vars, list_glob_vars,
    list_script_vars, list_tab_vars, list_vim_vars, list_win_vars, tv_to_optval,
};
use crate::charset::skip;
use crate::eval::typval::{NumBuf, TV_INITIAL_VALUE, list_len, tv_copy, tv_list_alloc};
use crate::eval::{
    FNE_CHECK_START, FNE_INCL_BR, env_name_len, eval_isnamec1, eval0_in_cmd_until, get_lval,
    num_divide, num_modulus, option_var_end, set_var_lval,
};
use crate::ex_cmds::check_secure;
use crate::ex_docmd::ends_excmd;
use crate::message::{e_invarg, e_listreq, emsg, internal_error};
use crate::option::{
    boolean_optval, get_option_value, get_tty_option, is_option_hidden, is_tty_option,
    optval_concat, optval_free, set_option_value_handle_tty_named,
};
use crate::os::cshim::gettext_owned;
use crate::os::env::{vim_getenv_owned, vim_setenv_named};
use crate::register::{get_reg_contents_owned, write_reg_contents_bytes};
use crate::types::{ExArg, Failed, OptInt, OptVal, TypVal, VAR_LIST, VarNumber};

/// The compound assignment operators, as they appear before the `=`.
const OPERATORS: &[u8] = b"+-*/%.";

/// The arithmetic ones, which an environment variable and a register refuse.
const ARITHMETIC: &[u8] = b"+-*/%";

/// One `:let` target parser, dispatched on the sigil the target starts with.
type LetTarget = fn(&[u8], &mut TypVal, bool, Option<&[u8]>, Option<u8>) -> Option<usize>;

/// Whether the operator is an arithmetic one, which an environment variable
/// and a register both refuse with E734.
fn is_arithmetic(op: Option<u8>) -> bool {
    op.is_some_and(|c| ARITHMETIC.contains(&c))
}

/// Report E734 for the operator `op`.
fn wrong_type(op: Option<u8>) {
    let op = char::from(op.unwrap_or(b'='));
    semsg!("E734: Wrong variable type for {op}=");
}

/// Past the white space at `text[at]`.
fn skip_white(text: &[u8], at: usize) -> usize {
    at + skip::white(text.get(at..).unwrap_or_default())
}

/// Whether what follows the target at `text[at]` is one of the characters
/// that may. The end of the text is not one of them.
fn ends_target(endchars: Option<&[u8]>, text: &[u8], at: usize) -> bool {
    endchars.is_none_or(|set| {
        let byte = byte_at(text, skip_white(text, at));
        byte != 0 && set.contains(&byte)
    })
}

/// What [`skip_var_list`] found.
#[derive(Clone, Copy)]
pub(crate) struct VarList {
    /// Where the target, or the `[...]` of them, ends.
    pub(crate) end: usize,
    /// How many targets a `[...]` holds; 0 for a single one.
    pub(crate) count: c_int,
    /// Whether a `[...]` has a `; rest` target.
    pub(crate) semicolon: bool,
}

/// `:let`, `:const` and (with no `=`) the listing forms.
pub fn ex_let(excmd: &mut ExArg) {
    let is_const = excmd.cmdidx == CmdIdx::r#const;
    let arg = excmd.line.arg;
    let mut first = true;

    // The targets' text, measured once: nothing below writes into the line
    // before it is read again (the here-document cuts only after them).
    let text_len = excmd.line.rest_of(arg).len();
    let Some(targets) = skip_var_list(&excmd.line.tail(arg)[..text_len], false) else {
        return;
    };
    let line = &excmd.line;
    let mut expr = line.skip_white(arg + targets.end);
    let concat = line.starts_with(expr, b"..=");
    let lead = line.byte_at(expr);
    let has_assign =
        lead == b'=' || (lead != 0 && OPERATORS.contains(&lead) && line.byte_at(expr + 1) == b'=');

    if !has_assign && !concat {
        // ":let" with no "=": list variables.
        let head = line.byte_at(arg);
        let mut end = arg;
        if head == b'[' {
            emsg_static(e_invarg);
        } else if ends_excmd(c_int::from(head)) == 0 {
            // ":let var1 var2"
            end = arg + list_arg_vars(line.rest_of(arg), excmd.skip, &mut first);
        } else if !excmd.skip {
            // ":let" on its own.
            const SCOPES: [ScopeLister; 7] = [
                list_glob_vars,
                list_buf_vars,
                list_win_vars,
                list_tab_vars,
                list_script_vars,
                list_func_vars,
                list_vim_vars,
            ];
            for lister in SCOPES {
                lister(&mut first);
            }
        }
        excmd.line.next = excmd.line.check_next(end);
        return;
    }

    let mut rettv = TV_INITIAL_VALUE;
    if lead == b'=' && line.byte_at(expr + 1) == b'<' && line.byte_at(expr + 2) == b'<' {
        // A here-document.
        if let Some(list) = heredoc_get(excmd, expr + 3, false) {
            rettv.write_list(Some(list));
            if !excmd.skip {
                let text = excmd.line.rest_of(arg);
                let _ = ex_let_vars(text, &mut rettv, false, targets, is_const, Some(b'='));
            }
            clear_local(&mut rettv);
        }
        return;
    }

    // The operator, if any, and the expression past it.
    let mut op = b'=';
    if lead == b'=' {
        expr += 1;
    } else {
        if OPERATORS.contains(&lead) {
            // "+=", "-=", "*=", "/=", "%=" or ".="
            op = lead;
            if lead == b'.' && line.byte_at(expr + 1) == b'.' {
                // "..=" -- one character longer than the rest.
                expr += 1;
            }
        }
        expr += 2;
    }
    let expr = line.skip_white(expr);

    let skipping = excmd.skip.then(Suppress::emsg_skip);
    let evaluate = !excmd.skip;
    let eval_res = eval0_in_cmd_until(excmd, expr, arg + text_len, &mut rettv, evaluate);
    drop(skipping);

    if evaluate && eval_res.is_ok() {
        let text = &excmd.line.tail(arg)[..text_len];
        let _ = ex_let_vars(text, &mut rettv, false, targets, is_const, Some(op));
    }
    if eval_res.is_ok() {
        clear_local(&mut rettv);
    }
}

/// Assign `tv` to the target or targets `text` starts with: one name, or
/// the `[v1, v2]` / `[v1, v2; rest]` unpack of a List, as [`skip_var_list`]
/// counted them.
///
/// `op` is the operator -- `+`, `-`, `*`, `/`, `%`, `.` or `=` -- and a
/// single target must be followed by it; `None` (`:for`) assigns plainly
/// and lets anything follow.
pub(crate) fn ex_let_vars(
    text: &[u8],
    tv: &mut TypVal,
    copy: bool,
    targets: VarList,
    is_const: bool,
    op: Option<u8>,
) -> Result<(), Failed> {
    if byte_at(text, 0) != b'[' {
        // ":let var = expr" or ":for var in list"
        let endchars = op.map(|op| [op]);
        let endchars = endchars.as_ref().map(<[u8; 1]>::as_slice);
        return ex_let_one(text, tv, copy, is_const, endchars, op)
            .map(drop)
            .ok_or(Failed);
    }

    // ":let [v1, v2] = list" or ":for [v1, v2] in listlist"
    if tv.v_type() != VAR_LIST {
        emsg_static(e_listreq);
        return Err(Failed);
    }
    let len = list_len(tv.list_ref());
    let semicolon = c_int::from(targets.semicolon);
    if semicolon == 0 && targets.count < len {
        emsg_static(c"E687: Less targets than List items");
        return Err(Failed);
    }
    if targets.count - semicolon > len {
        emsg_static(c"E688: More targets than List items");
        return Err(Failed);
    }
    // `:let [] = v:_null_list` fails with E688 or earlier before it can get
    // here, so the list is there.
    let list = tv.list_handle().ok_or(Failed)?;

    // An index, not an address: each target's subscripts run the
    // evaluator, which may edit the very list being unpacked. Each item is
    // copied out before its target is resolved.
    let mut at: usize = 0;
    let mut pos: usize = 0;
    while byte_at(text, pos) != b']' {
        // Skip the whitespace after the '[', ',' or ';'.
        let next = skip_white(text, pos + 1);
        let mut item = TV_INITIAL_VALUE;
        let Some(source) = list.items().get(at) else {
            // The list lost items while an earlier target was resolved.
            emsg_static(c"E688: More targets than List items");
            return Err(Failed);
        };
        tv_copy(&source.li_tv, &mut item);
        let end = ex_let_one(&text[next..], &mut item, false, is_const, Some(b",;]"), op);
        clear_local(&mut item);
        let Some(end) = end else {
            return Err(Failed);
        };
        at += 1;

        pos = skip_white(text, next + end);
        let sep = byte_at(text, pos);
        if sep == b';' {
            // The rest of the list, which may be empty, goes to the
            // variable after the ';', as a list of its own.
            let rest = list.items().get(at..).unwrap_or_default();
            let mut rest_list = tv_list_alloc(rest.len().try_into().unwrap_or(0));
            for item in rest {
                rest_list.push_copy(&item.li_tv);
            }
            let mut ltv = TypVal::list(Some(rest_list));
            let rest_at = skip_white(text, pos + 1);
            let end = ex_let_one(&text[rest_at..], &mut ltv, false, is_const, Some(b"]"), op);
            clear_local(&mut ltv);
            end.ok_or(Failed)?;
            break;
        } else if sep != b',' && sep != b']' {
            internal_error(c"ex_let_vars()");
            return Err(Failed);
        }
    }
    Ok(())
}

/// Skip an assignable variable, or the `[var, var]` list of them, at the
/// start of `text`. `None` after an error, which `silent` keeps from being
/// reported.
pub(crate) fn skip_var_list(text: &[u8], silent: bool) -> Option<VarList> {
    let mut list = VarList {
        end: 0,
        count: 0,
        semicolon: false,
    };
    if byte_at(text, 0) != b'[' {
        list.end = skip_var_one(text, 0);
        return Some(list);
    }
    // "[var, var]": find the matching ']'.
    let mut p = 0;
    let invalid = |at: usize| {
        if !silent {
            let rest = msg_bytes(text.get(at..).unwrap_or_default());
            semsg!("E475: Invalid argument: {rest}");
        }
    };
    loop {
        // Skip the whitespace after the '[', ';' or ','.
        p = skip_white(text, p + 1);
        let s = skip_var_one(text, p);
        if s == p {
            invalid(p);
            return None;
        }
        list.count += 1;

        p = skip_white(text, s);
        match byte_at(text, p) {
            b']' => {
                list.end = p + 1;
                return Some(list);
            }
            b';' if list.semicolon => {
                if !silent {
                    emsg_static(e_double_semicolon_in_list_of_variables);
                }
                return None;
            }
            b';' => list.semicolon = true,
            b',' => {}
            _ => {
                invalid(p);
                return None;
            }
        }
    }
}

/// Past the one assignable name at `text[at]`, including `@r`, `$VAR`,
/// `&option`, `d.key` and `l[idx]`.
fn skip_var_one(text: &[u8], at: usize) -> usize {
    let sigil = byte_at(text, at);
    if sigil == b'@' && byte_at(text, at + 1) != 0 {
        return at + 2;
    }
    let name = if sigil == b'$' || sigil == b'&' {
        at + 1
    } else {
        at
    };
    let rest = text.get(name..).unwrap_or_default();
    name + crate::eval::name_end(rest, FNE_INCL_BR | FNE_CHECK_START).end
}

/// `:let $VAR = …`, `text` starting at the `$`. Answers where the name
/// ends, or `None`.
fn ex_let_env(
    text: &[u8],
    tv: &mut TypVal,
    is_const: bool,
    endchars: Option<&[u8]>,
    op: Option<u8>,
) -> Option<usize> {
    let mut numbuf = NumBuf::new();
    if is_const {
        emsg_static(c"E996: Cannot lock an environment variable");
        return None;
    }

    // Find the end of the name.
    let len = env_name_len(&text[1..]);
    let end = 1 + len;
    if len == 0 {
        let text = msg_bytes(text);
        semsg!("E475: Invalid argument: {text}");
    } else if is_arithmetic(op) {
        wrong_type(op);
    } else if !ends_target(endchars, text, end) {
        emsg_static(e_letunexp);
    } else if !check_secure() {
        let name = &text[1..end];
        let value = numbuf.string_chk(tv)?;
        let joined;
        let mut value: &CStr = value;
        if op == Some(b'.')
            && let Some(old) = cstr::with_terminated(name, vim_getenv_owned)
        {
            joined = join(&old, value.to_bytes());
            value = &joined;
        }
        cstr::with_terminated(name, |name| vim_setenv_named(name, value));
        return Some(end);
    }
    None
}

/// `head` and `tail` as one C string.
fn join(head: &[u8], tail: &[u8]) -> CString {
    let mut joined = Vec::with_capacity(head.len() + tail.len() + 1);
    joined.extend_from_slice(head);
    joined.extend_from_slice(tail);
    cstr::owned(&joined)
}

/// `:let &opt = …`, `text` starting at the `&`. Answers where the name
/// ends, or `None`.
///
/// The compound operators are implemented here rather than through
/// `eexe_mod_op`, because an option's value is an `OptVal` and not a
/// `TypVal`: the current value is read, combined, and set back.
fn ex_let_option(
    text: &[u8],
    tv: &mut TypVal,
    is_const: bool,
    endchars: Option<&[u8]>,
    op: Option<u8>,
) -> Option<usize> {
    if is_const {
        emsg_static(c"E996: Cannot lock an option");
        return None;
    }

    // Find the end of the name.
    let option = option_var_end(text);
    let (opt_idx, opt_flags) = (option.index, option.flags);
    let Some(end) = option.end.filter(|&end| ends_target(endchars, text, end)) else {
        emsg_static(e_letunexp);
        return None;
    };
    // The name proper starts after the scope, if there is one.
    let name = &text[option.start..end];

    let is_tty_opt = is_tty_option(name);
    let hidden = is_option_hidden(opt_idx);
    let curval = if is_tty_opt {
        get_tty_option(name)
    } else {
        get_option_value(opt_idx, opt_flags)
    };
    let mut newval = OptVal::Nil;
    let mut arg_end = None;

    'theend: {
        if curval.is_nil() {
            let name = msg_bytes(name);
            semsg!("E355: Unknown option: {name}");
            break 'theend;
        }
        let compound = op.is_some_and(|c| c != b'=');
        let is_string = matches!(curval, OptVal::String(_));
        if compound && op.is_some_and(|c| (c == b'.') != is_string) {
            wrong_type(op);
            break 'theend;
        }

        let error;
        (newval, error) = tv_to_optval(tv, opt_idx, name);
        if error {
            break 'theend;
        }
        // The current and the new value must have the same type.
        debug_assert!(curval.kind() == newval.kind());

        if compound && !hidden {
            // A Number or Boolean `OptVal` as a number. Only those two
            // variants get this far: the `if` just below is the guard that
            // keeps a String or a Nil out, and both calls are inside it.
            let as_int = |v: OptVal| -> OptInt {
                match v {
                    OptVal::Number(number) => number,
                    // The tri-state word itself, as upstream's union read
                    // of the `boolean` arm answered.
                    OptVal::Boolean(_) => OptInt::from(v.tristate().expect("the arm is Boolean")),
                    OptVal::Nil | OptVal::String(_) => {
                        unreachable!("guarded to a Number or a Boolean")
                    }
                }
            };
            if matches!(curval, OptVal::Number(_) | OptVal::Boolean(_)) {
                let cur_n = as_int(curval);
                let new_n = as_int(newval);
                let new_n = match op.unwrap_or(b'=') {
                    b'+' => cur_n + new_n,
                    b'-' => cur_n - new_n,
                    b'*' => cur_n * new_n,
                    b'/' => num_divide(VarNumber::from(cur_n), VarNumber::from(new_n)),
                    b'%' => num_modulus(VarNumber::from(cur_n), VarNumber::from(new_n)),
                    // No other operator reaches here: `.` was refused
                    // above for a non-String option.
                    _ => new_n,
                };
                newval = if matches!(curval, OptVal::Number(_)) {
                    OptVal::Number(new_n)
                } else {
                    boolean_optval(tristate_from_int(new_n))
                };
            } else if let Some(joined) = optval_concat(&curval, &newval) {
                optval_free(newval);
                newval = joined;
            }
        }

        let err = set_option_value_handle_tty_named(name, opt_idx, newval, opt_flags);
        arg_end = Some(end);
        if let Err(err) = err {
            emsg(&gettext_owned(err.as_cstr()));
        }
    }

    optval_free(curval);
    optval_free(newval);
    arg_end
}

/// Upstream's `TRISTATE_FROM_INT`: anything positive is true, zero is false,
/// and a negative number is "unset".
pub(crate) fn tristate_from_int(n: OptInt) -> Option<bool> {
    if n == 0 {
        Some(false)
    } else if n >= 1 {
        Some(true)
    } else {
        None
    }
}

/// `:let @r = …`, `text` starting at the `@`. Answers where the register
/// name ends, or `None`.
fn ex_let_register(
    text: &[u8],
    tv: &mut TypVal,
    is_const: bool,
    endchars: Option<&[u8]>,
    op: Option<u8>,
) -> Option<usize> {
    let mut numbuf = NumBuf::new();
    if is_const {
        emsg_static(c"E996: Cannot lock a register");
        return None;
    }
    if is_arithmetic(op) {
        wrong_type(op);
        return None;
    }
    // The register name is one byte.
    let past = 2;
    if !ends_target(endchars, text, past) {
        emsg_static(e_letunexp);
        return None;
    }

    // A bare "@" is the unnamed register.
    let regname = match byte_at(text, 1) {
        b'@' => c_int::from(b'"'),
        // Sign-extended, as the C's `*arg` is: a register name is ASCII, but
        // the byte is what upstream passes on.
        c => c_int::from(c.cast_signed()),
    };
    let value = numbuf.string_chk(tv)?.to_bytes();
    let joined;
    let mut value = value;
    if op == Some(b'.')
        && let Some(old) = get_reg_contents_owned(regname, kGRegExprSrc.cast_signed())
    {
        joined = join(&old, value);
        value = joined.to_bytes();
    }
    write_reg_contents_bytes(regname, value, false);
    Some(past)
}

/// One assignment target, dispatched on what `text` starts with. Answers
/// where it ends, or `None` on an error.
fn ex_let_one(
    text: &[u8],
    tv: &mut TypVal,
    copy: bool,
    is_const: bool,
    endchars: Option<&[u8]>,
    op: Option<u8>,
) -> Option<usize> {
    let sigil = byte_at(text, 0);
    // The three sigils each have a parser of their own, which reads the
    // sigil back off `text`.
    let target: Option<LetTarget> = match sigil {
        b'$' => Some(ex_let_env),
        b'&' => Some(ex_let_option),
        b'@' => Some(ex_let_register),
        _ => None,
    };
    if let Some(target) = target {
        return target(text, tv, is_const, endchars, op);
    }
    if !eval_isnamec1(c_int::from(sigil)) && sigil != b'{' {
        let text = msg_bytes(text);
        semsg!("E475: Invalid argument: {text}");
        return None;
    }

    // A variable, a List or Dict item, or a Blob byte.
    let (mut lval, end) = get_lval(text, Some(tv), false, false, 0, FNE_CHECK_START);
    let end = end.filter(|_| lval.has_name())?;
    if !ends_target(endchars, text, end) {
        emsg_static(e_letunexp);
        return None;
    }
    set_var_lval(&mut lval, tv, copy, is_const, op);
    Some(end)
}
