//! What the rest of the editor calls the evaluator through.
//!
//! Every entry point here brackets one evaluation: it takes the
//! expression's text as bytes (or a command's line and an offset into it),
//! runs `eval0`, converts the result to whatever the caller wanted, and
//! clears the typval on both the success and the error path. What differs
//! is what is counted up around it (`emsg_skip`, `emsg_off`, `sandbox`,
//! `textlock`, the funccal stack) and what the answer is converted to.
//!
//! The text is the caller's, borrowed for the evaluation. A caller whose
//! text lives where the expression can reach it -- an option value, a
//! mapping, a register -- hands over a copy.

#![forbid(unsafe_code)]

use crate::cstr;
use crate::eval::Parsed;
use crate::eval::list::cstr_of;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::guard::{Lock, Suppress};
use crate::memory::XString;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use crate::winlayer::Win;
use core::ffi::{CStr, c_int};

use crate::charset::skip;
use crate::eval::encode::tv2string_bytes;
use crate::eval::typval::{
    NumBuf, list_items_mut, list_join, list_len, list_set_lock, tv_clear, tv_get_number_chk,
    tv_list_alloc,
};
use crate::eval::userfunc::{CallStackAside, CallWith, call_func_with, func_init};
use crate::eval::vars::{clear_local, evalvars_init, lua_partial, set_vim_var_list};
use crate::eval::{
    Cursor, check_luafunc_name, eval0, eval0_in_cmd, eval0_simple_funccal, eval1,
    may_call_simple_func,
};
use crate::ex_eval::aborting;
use crate::message::state::{called_emsg, did_emsg};
use crate::option::was_set_insecurely;
use crate::options::{kOptFoldexpr, kOptFoldtext, kWinOptFoldexpr};
use crate::optionstr::OptString;
use crate::runtime::state::current_sctx;
use crate::types::{
    ExArg, Failed, NUL, Object, OptionSetFlags, ScriptCtx, String_0, TypVal, VAR_DICT, VAR_FUNC,
    VAR_LIST, VAR_NUMBER, VAR_PARTIAL, VAR_STRING, VAR_UNKNOWN, VarLock, VarNumber, Vv, ptrdiff_t,
};

/// A freshly declared typval.
const UNSET_TV: TypVal = TV_INITIAL_VALUE;

/// Bring up the evaluator: the `v:` variables and the function table.
pub fn eval_init() {
    evalvars_init();
    func_init();
}

/// A number's truth, with an error for a value that has no number.
fn truth(tv: &mut TypVal) -> Result<bool, Failed> {
    let answer = tv_get_number_chk(tv).map(|n| n != 0).map_err(|_| Failed);
    clear_local(tv);
    answer
}

/// Evaluate `text` and answer its truth. An `Err` is an evaluation that
/// failed, which is not the same as answering false.
pub(crate) fn eval_to_bool(text: &[u8], use_simple_function: bool) -> Result<bool, Failed> {
    let mut tv = UNSET_TV;
    if use_simple_function {
        eval0_simple_funccal(text, &mut tv)?;
    } else {
        eval0(text, &mut tv, true).0?;
    }
    truth(&mut tv)
}

/// The truth of the command's argument -- `:if`, `:elseif`, `:while` -- or,
/// when `skip`, only its parse (and then false). The command after it is
/// left in `line.next`.
pub(crate) fn eval_cmd_bool(excmd: &mut ExArg, skip: bool) -> Result<bool, Failed> {
    let mut tv = UNSET_TV;
    let _skipping = skip.then(Suppress::emsg_skip);
    let at = excmd.line.arg;
    eval0_in_cmd(excmd, at, &mut tv, !skip)?;
    if skip { Ok(false) } else { truth(&mut tv) }
}

/// `eval1` with a fallback message: when the expression failed silently —
/// nothing aborted and nothing reported — say which expression it was.
pub(crate) fn eval1_emsg(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
) -> Result<(), Failed> {
    let start = cursor.offset();
    let did_emsg_before = did_emsg.get();
    let called_emsg_before = called_emsg.get();
    let ret = eval1(cursor, result, evaluate);
    if ret.is_err()
        && !aborting()
        && did_emsg.get() == did_emsg_before
        && called_emsg.get() == called_emsg_before
    {
        let start = msg_bytes(&cursor.text()[start..]);
        semsg!("E15: Invalid expression: \"{start}\"");
    }
    ret
}

/// Is this typval usable as an expression argument at all? An unset value
/// and an empty String are not.
pub fn eval_expr_valid_arg(tv: &TypVal) -> bool {
    match tv.v_type() {
        VAR_UNKNOWN => false,
        VAR_STRING => tv.string_cstr().is_some_and(|text| !text.is_empty()),
        _ => true,
    }
}

/// Call the partial in `expr`.
pub(crate) fn eval_expr_partial(
    expr: &TypVal,
    argv: &[TypVal],
    result: &mut TypVal,
) -> Result<(), Failed> {
    let TypVal::Partial(partial) = expr else {
        return Err(Failed);
    };
    let partial = partial.as_ref().ok_or(Failed)?;
    let name = expr.callable_name().ok_or(Failed)?;
    let with = CallWith {
        partial: Some(partial),
        ..CallWith::new(true)
    };
    call_func_with(name, None, result, argv, with)
}

/// Call the function `expr` names.
pub(crate) fn eval_expr_func(
    expr: &TypVal,
    argv: &[TypVal],
    result: &mut TypVal,
) -> Result<(), Failed> {
    let mut buf = NumBuf::new();
    let name = if expr.v_type() == VAR_FUNC {
        expr.callable_name()
    } else {
        buf.string_chk(expr)
    };
    match name {
        Some(name) if !name.is_empty() => {
            call_func_with(name, None, result, argv, CallWith::new(true))
        }
        _ => Err(Failed),
    }
}

/// Evaluate `expr` as an expression *string*, which must consume all of it.
pub(crate) fn eval_expr_string(expr: &TypVal, result: &mut TypVal) -> Result<(), Failed> {
    let mut buf = NumBuf::new();
    let text = buf.string_chk(expr).ok_or(Failed)?;
    let mut cursor = Cursor::new(text.to_bytes());
    cursor.skip_white();
    eval1_emsg(&mut cursor, result, true)?;
    let end = cursor.offset();
    cursor.skip_white();
    if cursor.byte() != 0 {
        tv_clear(result);
        let rest = msg_bytes(&cursor.text()[end..]);
        semsg!("E15: Invalid expression: \"{rest}\"");
        return Err(Failed);
    }
    Ok(())
}

/// Evaluate whatever `expr` holds — a partial, a Funcref, a function name
/// or an expression string — with `argv` as its arguments.
pub fn eval_expr_typval(
    expr: &TypVal,
    want_func: bool,
    argv: &[TypVal],
    result: &mut TypVal,
) -> Result<(), Failed> {
    // SAFETY: the caller's promise -- `expr` outlives the call and is only
    // read through here; each arm restates the same promise.
    let ty = expr;
    if ty.v_type() == VAR_PARTIAL {
        return eval_expr_partial(expr, argv, result);
    }
    if ty.v_type() == VAR_FUNC || want_func {
        return eval_expr_func(expr, argv, result);
    }
    eval_expr_string(expr, result)
}

/// `eval_expr_typval` with no arguments, answering the result's truth.
pub(crate) fn eval_expr_to_bool(expr: &TypVal) -> Result<bool, Failed> {
    let mut rettv = UNSET_TV;
    eval_expr_typval(expr, false, &[], &mut rettv)?;
    truth(&mut rettv)
}

/// The command's argument evaluated for its String -- `:throw` -- or, when
/// `skip`, only parsed. The command after it is left in `line.next`.
pub(crate) fn eval_cmd_string(excmd: &mut ExArg, skip: bool) -> Option<XString> {
    let mut numbuf = NumBuf::new();
    let mut tv = UNSET_TV;
    let _skipping = skip.then(Suppress::emsg_skip);
    let at = excmd.line.arg;
    if eval0_in_cmd(excmd, at, &mut tv, !skip).is_err() || skip {
        return None;
    }
    let value = XString::from_cstr(numbuf.string(&tv));
    clear_local(&mut tv);
    Some(value)
}

/// How much of `text` an expression takes up, white space before it
/// included, parsing it without evaluating anything. A `` `=expr` `` in a
/// command's argument is stepped over this way.
pub(crate) fn skip_expr(text: &[u8]) -> usize {
    let mut cursor = Cursor::new(text);
    cursor.skip_white();
    let mut skipped = UNSET_TV;
    let _ = eval1(&mut cursor, &mut skipped, false);
    cursor.offset()
}

/// Render a typval as the String a caller of the evaluator expects: a List
/// joined with newlines when `join_list`, otherwise the `string()` form for
/// a container and the plain coercion for everything else.
pub(crate) fn typval2string(tv: &TypVal, join_list: bool) -> XString {
    let mut numbuf = NumBuf::new();
    if join_list && tv.v_type() == VAR_LIST {
        let mut text = XString::new();
        let l = tv.list_ref();
        let _ = list_join(&mut text, l, c"\n");
        if list_len(l) > 0 {
            text.push_byte(b'\n');
        }
        return text;
    }
    if tv.v_type() == VAR_LIST || tv.v_type() == VAR_DICT {
        return XString::from_bytes(&tv2string_bytes(tv));
    }
    XString::from_cstr(numbuf.string(tv))
}

/// Evaluate `text` for its String: a List joined with newlines when
/// `join_list`. `None` when the evaluation failed.
pub fn eval_to_string(text: &[u8], join_list: bool, use_simple_function: bool) -> Option<XString> {
    let mut tv = UNSET_TV;
    let r = if use_simple_function {
        eval0_simple_funccal(text, &mut tv)
    } else {
        eval0(text, &mut tv, true).0
    };
    r.ok()?;
    let value = typval2string(&tv, join_list);
    clear_local(&mut tv);
    Some(value)
}

/// `eval_to_string` with the text locked and, optionally, the sandbox on,
/// and with the function-call stack saved across it.
pub(crate) fn eval_to_string_safe(
    text: &[u8],
    use_sandbox: bool,
    use_simple_function: bool,
) -> Option<XString> {
    let call_stack_aside = CallStackAside::new();
    let _sandboxed = use_sandbox.then(Lock::sandbox);
    let _locked = Lock::text();
    let value = eval_to_string(text, false, use_simple_function);
    drop(call_stack_aside);
    value
}

/// Evaluate `text` for its Number, silently. -1 for a failure, which is
/// not distinguishable from a result of -1.
pub fn eval_to_number(text: &[u8], use_simple_function: bool) -> VarNumber {
    let mut rettv = UNSET_TV;
    let _no_emsg = Suppress::emsg();
    // Note the shortcut is handed the *unskipped* expression, unlike `eval1`.
    let simple = if use_simple_function {
        may_call_simple_func(text, &mut rettv)
    } else {
        Ok(Parsed::NotThis)
    };
    let r = match simple {
        Ok(Parsed::NotThis) => {
            let mut cursor = Cursor::new(text);
            cursor.skip_white();
            eval1(&mut cursor, &mut rettv, true)
        }
        other => other.map(|_| ()),
    };

    if r.is_err() {
        -1
    } else {
        let n = tv_get_number_chk(&rettv).unwrap_or(-1);
        clear_local(&mut rettv);
        n
    }
}

/// Evaluate the command's argument as an expression, leaving the command
/// after it in `line.next`. `None` when it failed.
pub(crate) fn eval_cmd_arg(excmd: &mut ExArg) -> Option<TypVal> {
    let mut tv = UNSET_TV;
    let at = excmd.line.arg;
    let evaluate = !excmd.skip;
    eval0_in_cmd(excmd, at, &mut tv, evaluate).ok()?;
    Some(tv)
}

/// Evaluate `text` into a value the caller owns; `None` on failure.
pub(crate) fn eval_expr(text: &[u8]) -> Option<TypVal> {
    eval_expr_ext(text, false)
}

/// As `eval_expr`, optionally taking the shortcut for an expression that is
/// nothing but one function call.
pub(crate) fn eval_expr_ext(text: &[u8], use_simple_function: bool) -> Option<TypVal> {
    let mut tv = UNSET_TV;
    let r = if use_simple_function {
        eval0_simple_funccal(text, &mut tv)
    } else {
        eval0(text, &mut tv, true).0
    };
    r.ok()?;
    Some(tv)
}

/// Call a Vimscript function by name with `argv` as its arguments.
pub(crate) fn call_vim_function(
    func: &CStr,
    argv: &[TypVal],
    result: &mut TypVal,
) -> Result<(), Failed> {
    let mut ret = Err(Failed);
    'fail: {
        let (name, len, lua) = if let Some(lua_name) = func.to_bytes().strip_prefix(b"v:lua.") {
            let len = check_luafunc_name(lua_name, false);
            if len == 0 {
                break 'fail;
            }
            (cstr::suffix(func, 6), Some(len), lua_partial())
        } else {
            (func, None, None)
        };
        result.write_empty(VAR_UNKNOWN);
        let with = CallWith {
            partial: lua.as_ref(),
            ..CallWith::at_cursor(true)
        };
        ret = call_func_with(name, len, result, argv, with);
    }
    if ret.is_err() {
        tv_clear(result);
    }
    ret
}

/// [`call_vim_function`] of the function named `func`, answering a copy of
/// its result as a String (a Number spelled out); `None` when the call
/// failed.
pub(crate) fn call_func_retstr(func: &CStr, argv: &[TypVal]) -> Option<XString> {
    let mut numbuf = NumBuf::new();
    let mut rettv = UNSET_TV;
    call_vim_function(func, argv, &mut rettv).ok()?;
    let retval = XString::from_cstr(cstr_of(&rettv, &mut numbuf));
    clear_local(&mut rettv);
    Some(retval)
}

/// [`call_vim_function`] of the function named `func`, answering its result
/// when that is a List; `None` for anything else.
pub(crate) fn call_func_retlist(func: &CStr, argv: &[TypVal]) -> Option<TypVal> {
    let mut rettv = UNSET_TV;
    call_vim_function(func, argv, &mut rettv).ok()?;
    if rettv.v_type() != VAR_LIST {
        clear_local(&mut rettv);
        return None;
    }
    Some(rettv)
}

/// The number `text` starts with, as C's `atol` reads it: white space, a
/// sign and digits, saturating rather than overflowing.
fn leading_long(text: &[u8]) -> VarNumber {
    let skip = text
        .iter()
        .take_while(|&&b| matches!(b, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r'))
        .count();
    let mut rest = &text[skip..];
    let negative = rest.first() == Some(&b'-');
    if matches!(rest.first(), Some(b'-' | b'+')) {
        rest = &rest[1..];
    }
    let mut n: i128 = 0;
    for &b in rest.iter().take_while(|b| b.is_ascii_digit()) {
        n = (n * 10 + i128::from(b - b'0')).min(i128::from(VarNumber::MAX) + 1);
    }
    let n = if negative { -n } else { n };
    n.clamp(i128::from(VarNumber::MIN), i128::from(VarNumber::MAX)) as VarNumber
}

/// Run 'foldexpr' for the window's current line: the level, and the leading
/// marker character (`>`, `<`, `=`, `a`, `s`) when there was one, else NUL.
pub(crate) fn eval_foldexpr(window: Win) -> (c_int, c_int) {
    let saved_sctx: ScriptCtx = current_sctx.get();
    let use_sandbox = was_set_insecurely(window, kOptFoldexpr, OptionSetFlags::LOCAL);
    // A copy: the expression may set 'foldexpr' and free the option's text.
    let expr = window.w_onebuf_opt.wo_fde.get();
    let text = &expr[skip::white(&expr)..];
    current_sctx.set(window.w_onebuf_opt.wo_script_ctx[kWinOptFoldexpr as usize]);
    let mut marker = NUL;
    let retval: VarNumber = {
        let _no_emsg = Suppress::emsg();
        let _sandboxed = use_sandbox.then(Lock::sandbox);
        let _locked = Lock::text();

        let mut tv = UNSET_TV;
        let mut retval: VarNumber = 0;
        if eval0_simple_funccal(text, &mut tv).is_ok() {
            if tv.v_type() == VAR_NUMBER {
                retval = tv.number_or_zero();
            } else if let Some(answer) = tv.string_cstr() {
                let mut digits = answer.to_bytes();
                // A leading non-digit that is not a minus sign is the
                // fold marker; the rest is the level.
                if let Some(&first) = digits.first()
                    && !first.is_ascii_digit()
                    && first != b'-'
                {
                    marker = c_int::from(first);
                    digits = &digits[1..];
                }
                retval = leading_long(digits);
            }
            clear_local(&mut tv);
        }
        retval
    };
    current_sctx.set(saved_sctx);
    (retval as c_int, marker)
}

/// Run 'foldtext' for the window's current fold. A List comes back as an
/// Object so the caller can keep its per-chunk highlighting; anything else
/// is coerced to a String.
pub fn eval_foldtext(window: Win) -> Object {
    let mut numbuf = NumBuf::new();
    let use_sandbox = was_set_insecurely(window, kOptFoldtext, OptionSetFlags::LOCAL);
    // A copy: the expression may set 'foldtext' and free the option's text.
    let expr = window.w_onebuf_opt.wo_fdt.get();
    let call_stack_aside = CallStackAside::new();
    let _sandboxed = use_sandbox.then(Lock::sandbox);
    let _locked = Lock::text();

    let mut tv = UNSET_TV;
    let retval = if eval0_simple_funccal(&expr, &mut tv).is_err() {
        Object::string(String_0::NULL)
    } else {
        let obj = if tv.v_type() == VAR_LIST {
            Object::from(&tv)
        } else {
            Object::string(String_0::from_bytes(numbuf.string(&tv).to_bytes()))
        };
        clear_local(&mut tv);
        obj
    };
    drop(call_stack_aside);
    retval
}

/// Fill `v:argv` from the process arguments. Every item is locked.
pub(crate) fn set_argv_var(args: &[&CStr]) {
    let mut list = tv_list_alloc(args.len() as ptrdiff_t);
    for arg in args {
        list.push(TypVal::string_from(arg.to_bytes()));
    }
    for item in list_items_mut(Some(&mut list)) {
        item.li_lock = VarLock::Fixed;
    }
    list_set_lock(Some(&mut list), VarLock::Fixed);
    set_vim_var_list(Vv::Argv, Some(list));
}

/// Render a typval for display, as `:echo` would. A null typval is the
/// "no such variable" text, which is what the debugger prints.
pub(crate) fn typval_tostring(arg: Option<&TypVal>, quotes: bool) -> XString {
    let Some(value) = arg else {
        return XString::from_cstr(c"(does not exist)");
    };
    if !quotes && value.v_type() == VAR_STRING {
        return XString::from_cstr(value.string_cstr().unwrap_or(c""));
    }
    XString::from_bytes(&tv2string_bytes(value))
}
