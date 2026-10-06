//! The recursive-descent evaluator, one function per precedence level.
//!
//! `eval0` is the entry; each level parses its own operators and hands the
//! rest down, so `eval1` is `? :`, `eval2` is `||`, `eval3` is `&&`, `eval4`
//! the comparisons, `eval5` `+`/`-`/`..`, `eval6` `*`/`/`/`%` and `eval7` an
//! operand with its subscripts.
//!
//! Every level takes the [`Cursor`] and leaves it on the first byte it did
//! not consume, and a flag saying whether to evaluate or only to parse: a
//! short-circuited operand, a skipped `:if` branch and the look-ahead that
//! tells a Dict from a curly-brace name all parse without running anything.

#![forbid(unsafe_code)]

use crate::charset::skip;
use crate::eval::Cursor;
use crate::eval::char_len_at;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use core::ffi::c_int;

use crate::eval::expr::arith::{
    eval_addblob, eval_addlist, eval_addsub_number, eval_concat_str, eval_multdiv_number,
};
use crate::eval::typval::{tv_check_num, tv_check_str, tv_clear, tv_get_number_chk, tv2bool};
use crate::eval::userfunc::{call_simple_func, call_simple_luafunc, get_lambda_tv};
use crate::eval::vars::{check_vars_named, eval_variable, lua_partial};
use crate::eval::{
    EXPR_UNKNOWN, Parsed, comparison_at, eval_dict, eval_env_var, eval_func, eval_interp_string,
    eval_isnamec, eval_isnamec1, eval_list, eval_lit_dict, eval_lit_string, eval_number,
    eval_option, eval_string, get_name_len, handle_subscript, kGRegExprSrc, luafunc_name_end,
    typval_compare,
};
use crate::ex_docmd::ends_excmd;
use crate::ex_eval::aborting;
use crate::global_cell::GlobalCell;
use crate::guard::Depth;
use crate::message::emsg;
use crate::message::state::{called_emsg, did_emsg};
use crate::option::vars::p_ic;
use crate::os::cshim::gettext;
use crate::register::get_reg_contents;
use crate::strings::find_bytes;
use crate::types::{
    ExArg, Failed, Float, NUL, TypVal, VAR_BLOB, VAR_BOOL, VAR_FLOAT, VAR_LIST, VAR_STRING,
    VAR_UNKNOWN, VarNumber, kBoolVarFalse, kBoolVarTrue,
};

/// A freshly declared typval, which is what every level starts a second
/// operand as.
const UNSET_TV: TypVal = TV_INITIAL_VALUE;

/// Evaluate a whole expression, which must be all that is left of `text`.
///
/// Answers how far the parse got as well: the offset into `text` of the
/// first byte the expression did not consume, which is where the command
/// after it (`| cmd`) is looked for.
pub fn eval0(text: &[u8], result: &mut TypVal, evaluate: bool) -> (Result<(), Failed>, usize) {
    let did_emsg_before = did_emsg.get();
    let called_emsg_before = called_emsg.get();
    let mut cursor = Cursor::new(text);
    cursor.skip_white();
    let ret = eval1(&mut cursor, result, evaluate);
    // Anything left over is an error, but only once the expression
    // itself parsed.
    let end_error = ret.is_ok() && ends_excmd(c_int::from(cursor.byte())) == 0;
    if ret.is_err() || end_error {
        if ret.is_ok() {
            tv_clear(result);
        }
        // Stay quiet if something already reported, or if we are
        // unwinding from an exception.
        if !aborting()
            && did_emsg.get() == did_emsg_before
            && called_emsg.get() == called_emsg_before
        {
            if end_error {
                let rest = msg_bytes(cursor.rest());
                semsg!("E488: Trailing characters: {rest}");
            } else {
                let whole = msg_bytes(text);
                semsg!("E15: Invalid expression: \"{whole}\"");
            }
        }
        return (Err(Failed), cursor.offset());
    }
    (ret, cursor.offset())
}

/// [`eval0`] over the command's own line from `at`, leaving the command
/// after the expression in `line.next`.
///
/// After a failure the next command is still found, so that `:if 1 | ...
/// endif` keeps its `| endif`, unless what follows the `|` is another `|`.
pub(crate) fn eval0_in_cmd(
    excmd: &mut ExArg,
    at: usize,
    result: &mut TypVal,
    evaluate: bool,
) -> Result<(), Failed> {
    let end = excmd.line.end_of(at);
    eval0_in_cmd_until(excmd, at, end, result, evaluate)
}

/// [`eval0_in_cmd`] for a caller that already measured where the string
/// `at` is in ends: `end`, the offset of its terminator.
pub(crate) fn eval0_in_cmd_until(
    excmd: &mut ExArg,
    at: usize,
    end: usize,
    result: &mut TypVal,
    evaluate: bool,
) -> Result<(), Failed> {
    let text = &excmd.line.tail(at)[..end - at];
    let (ret, consumed) = eval0(text, result, evaluate);
    let next = excmd.line.check_next(at + consumed);
    match (&ret, next) {
        (Ok(()), _) => excmd.line.next = next,
        (Err(_), Some(next)) if excmd.line.byte_at(next) != b'|' => excmd.line.next = Some(next),
        (Err(_), _) => {}
    }
    ret
}

/// Where the plain name at the start of `text` ends; 0 when it does not
/// start one. With `use_namespace`, a single leading `x:` from `bgstvw` is
/// part of the name rather than its end.
fn plain_name_end(text: &[u8], use_namespace: bool) -> usize {
    let Some(&first) = text.first() else {
        return 0;
    };
    if !eval_isnamec1(c_int::from(first)) {
        return 0;
    }
    let mut p = 1;
    while let Some(&c) = text.get(p) {
        if c == NUL as u8 || !eval_isnamec(c_int::from(c)) {
            break;
        }
        // A `:` continues the name only as the one namespace letter.
        if c == b':' && !(use_namespace && p == 1 && b"bgstvw".contains(&first)) {
            break;
        }
        p += char_len_at(text, p);
    }
    p
}

/// Shortcut for a whole expression that is nothing but one call: `Foo()`.
///
/// Answers [`Parsed::NotThis`] when the expression is anything else.
pub(crate) fn may_call_simple_func(text: &[u8], result: &mut TypVal) -> Result<Parsed, Failed> {
    let Some(parens) = find_bytes(text, b"()") else {
        return Ok(Parsed::NotThis);
    };
    let after = &text[parens + 2..];
    if skip::white(after) != after.len() {
        return Ok(Parsed::NotThis);
    }
    if let Some(lua) = text.strip_prefix(b"v:lua.") {
        let p = 6;
        if p != parens && p + luafunc_name_end(lua) == parens {
            return Parsed::done(call_simple_luafunc(&text[p..parens], result));
        }
    } else {
        // A script-local name arrives as `<SNR>123_name`.
        let p = if text.starts_with(b"<SNR>") {
            5 + skip::digits(&text[5..])
        } else {
            0
        };
        if p + plain_name_end(&text[p..], true) == parens {
            return call_simple_func(&text[..parens], result);
        }
    }
    Ok(Parsed::NotThis)
}

/// `eval0` with the single-call shortcut tried first. No offset comes back:
/// the shortcut parses nothing, so its callers are the ones that do not ask.
pub(crate) fn eval0_simple_funccal(text: &[u8], result: &mut TypVal) -> Result<(), Failed> {
    match may_call_simple_func(text, result)? {
        Parsed::NotThis => eval0(text, result, true).0,
        Parsed::Done => Ok(()),
    }
}

/// `? :` and `??`, the lowest-precedence level.
pub(crate) fn eval1(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
) -> Result<(), Failed> {
    result.overwrite(UNSET_TV);
    eval2(cursor, result, evaluate)?;
    if cursor.byte() != b'?' {
        return Ok(());
    }
    let op_falsy = cursor.at(1) == b'?';

    let mut truthy = false;
    if evaluate {
        let mut error = false;
        truthy = if op_falsy {
            tv2bool(result)
        } else {
            tv_get_number_chk(result).map_or_else(
                |_| {
                    error = true;
                    false
                },
                |n| n != 0,
            )
        };
        // `??` keeps the left operand when it is truthy; `? :` never
        // does, and neither keeps it after an error.
        if error || !op_falsy || !truthy {
            tv_clear(result);
        }
        if error {
            return Err(Failed);
        }
    }

    // `??` is two bytes, `?` one, and white space follows either.
    cursor.bump(if op_falsy { 2 } else { 1 });
    cursor.skip_white();
    let first = evaluate && if op_falsy { !truthy } else { truthy };

    let mut var2 = UNSET_TV;
    eval1(cursor, &mut var2, first)?;
    if !op_falsy || !truthy {
        *result = var2.take();
    }

    if !op_falsy {
        if cursor.byte() != b':' {
            emsg(gettext(c"E109: Missing ':' after '?'"));
            if evaluate && truthy {
                tv_clear(result);
            }
            return Err(Failed);
        }
        cursor.bump(1);
        cursor.skip_white();
        if eval1(cursor, &mut var2, evaluate && !truthy).is_err() {
            if evaluate && truthy {
                tv_clear(result);
            }
            return Err(Failed);
        }
        if evaluate && !truthy {
            *result = var2.take();
        }
    }
    Ok(())
}

/// One level of the descent: parse an operand at the cursor into the typval,
/// evaluating it when the flag says so.
type Level = fn(&mut Cursor<'_>, &mut TypVal, bool) -> Result<(), Failed>;

/// `||` and `&&`, which differ only in what settles the answer early.
///
/// `stop_at` is the result that makes the remaining operands irrelevant:
/// true for `||`, false for `&&`. Everything else — the initial value, when
/// to keep evaluating, and when to fold the operand in — follows from it.
fn eval_logical(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
    operand: Level,
    op: u8,
    stop_at: bool,
) -> Result<(), Failed> {
    operand(cursor, result, evaluate)?;
    let is_op = |cursor: &Cursor<'_>| cursor.byte() == op && cursor.at(1) == op;
    if !is_op(cursor) {
        return Ok(());
    }

    let mut truthy = !stop_at;
    if evaluate {
        let read = tv_get_number_chk(result);
        tv_clear(result);
        let Ok(n) = read else {
            return Err(Failed);
        };
        truthy = n != 0;
    }

    while is_op(cursor) {
        cursor.bump(2);
        cursor.skip_white();
        let mut var2 = UNSET_TV;
        operand(cursor, &mut var2, evaluate && truthy != stop_at)?;
        if evaluate && truthy != stop_at {
            let read = tv_get_number_chk(&var2);
            tv_clear(&mut var2);
            let Ok(n) = read else {
                return Err(Failed);
            };
            truthy = n != 0;
        }
        if evaluate {
            result.write_number(VarNumber::from(truthy));
        }
    }
    Ok(())
}

/// `||`.
pub(crate) fn eval2(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
) -> Result<(), Failed> {
    eval_logical(cursor, result, evaluate, eval3, b'|', true)
}

/// `&&`.
pub(crate) fn eval3(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
) -> Result<(), Failed> {
    eval_logical(cursor, result, evaluate, eval4, b'&', false)
}

/// The comparison operators.
pub(crate) fn eval4(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
) -> Result<(), Failed> {
    eval5(cursor, result, evaluate)?;
    let (op, len) = comparison_at(|i| cursor.at(i));
    if op == EXPR_UNKNOWN {
        return Ok(());
    }
    let mut len = usize::try_from(len).unwrap_or(0);

    // A trailing `?` or `#` overrides 'ignorecase' for this comparison.
    let ic = match cursor.at(len) {
        b'?' => {
            len += 1;
            true
        }
        b'#' => {
            len += 1;
            false
        }
        _ => p_ic(),
    };

    cursor.bump(len);
    cursor.skip_white();
    let mut var2 = UNSET_TV;
    if eval5(cursor, &mut var2, evaluate).is_err() {
        tv_clear(result);
        return Err(Failed);
    }
    if evaluate {
        let ret = typval_compare(result, &mut var2, op, ic);
        tv_clear(&mut var2);
        return ret;
    }
    Ok(())
}

/// `+`, `-` and the two spellings of string concatenation.
pub(crate) fn eval5(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
) -> Result<(), Failed> {
    eval6(cursor, result, evaluate, false)?;
    loop {
        let op = cursor.byte();
        let concat = op == b'.';
        if op != b'+' && op != b'-' && !concat {
            return Ok(());
        }

        // Reject an operand of the wrong type before consuming the
        // operator — but not for the two cases that have their own
        // handling: `+` on a List or Blob, and anything on a Float.
        let container_plus =
            op == b'+' && (result.v_type() == VAR_LIST || result.v_type() == VAR_BLOB);
        let float_arith = op != b'.' && result.v_type() == VAR_FLOAT;
        if !container_plus && !float_arith && evaluate {
            let ok = if concat {
                tv_check_str(result)
            } else {
                tv_check_num(result)
            };
            if !ok {
                tv_clear(result);
                return Err(Failed);
            }
        }

        // `..` is two bytes, `.` one.
        cursor.bump(if concat && cursor.at(1) == b'.' { 2 } else { 1 });
        cursor.skip_white();

        let mut var2 = UNSET_TV;
        if eval6(cursor, &mut var2, evaluate, concat).is_err() {
            tv_clear(result);
            return Err(Failed);
        }
        if evaluate {
            let (blob2, list2) = (var2.v_type() == VAR_BLOB, var2.v_type() == VAR_LIST);
            let ok = if concat {
                eval_concat_str(result, &mut var2)
            } else if op == b'+' && result.v_type() == VAR_BLOB && blob2 {
                eval_addblob(result, &var2);
                true
            } else if op == b'+' && result.v_type() == VAR_LIST && list2 {
                eval_addlist(result, &mut var2)
            } else {
                eval_addsub_number(result, &mut var2, op)
            };
            if !ok {
                return Err(Failed);
            }
            tv_clear(&mut var2);
        }
    }
}

/// `*`, `/` and `%`.
pub(crate) fn eval6(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
    want_string: bool,
) -> Result<(), Failed> {
    eval7(cursor, result, evaluate, want_string)?;
    loop {
        let op = cursor.byte();
        if op != b'*' && op != b'/' && op != b'%' {
            return Ok(());
        }
        cursor.bump(1);
        cursor.skip_white();
        let mut var2 = UNSET_TV;
        eval7(cursor, &mut var2, evaluate, false)?;
        if evaluate && !eval_multdiv_number(result, &mut var2, op) {
            return Err(Failed);
        }
    }
}

/// An operand, with the `!`/`-`/`+` prefixes and any subscripts.
pub(crate) fn eval7(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
    want_string: bool,
) -> Result<(), Failed> {
    /// How deep `eval7` is into itself. The guard is what stops a
    /// self-referential expression from exhausting the C stack.
    static RECURSE: GlobalCell<c_int> = GlobalCell::new(0);
    const MAX_RECURSE: c_int = 1000;

    let text = cursor.text();
    let mut ret = Ok(Parsed::Done);
    result.write_empty(VAR_UNKNOWN);

    // The prefixes are collected now and applied last, so that `-1` is
    // parsed as a negated literal but `!x[0]` negates the subscript.
    let start_leader = cursor.offset();
    while matches!(cursor.byte(), b'!' | b'-' | b'+') {
        cursor.bump(1);
        cursor.skip_white();
    }
    let mut end_leader = cursor.offset();

    if RECURSE.get() == MAX_RECURSE {
        let at = msg_bytes(cursor.rest());
        semsg!("E1169: Expression too recursive: {at}");
        return Err(Failed);
    }
    // The un-bump is the guard's, so that an early exit cannot skip it.
    let _depth = Depth::of(&RECURSE);

    match cursor.byte() {
        b'0'..=b'9' => {
            ret = Parsed::done(eval_number(cursor, result, evaluate, want_string));
            // A number applies its prefixes here, where `-` still means
            // arithmetic negation rather than "negate what follows".
            if ret.is_ok() && evaluate && end_leader > start_leader {
                let (applied, left) = eval7_leader(result, true, &text[start_leader..end_leader]);
                end_leader = start_leader + left;
                ret = Parsed::done(applied);
            }
        }
        b'"' => ret = Parsed::done(eval_string(cursor, result, evaluate, false)),
        b'\'' => ret = Parsed::done(eval_lit_string(cursor, result, evaluate, false)),
        b'[' => ret = Parsed::done(eval_list(cursor, result, evaluate)),
        b'#' => ret = eval_lit_dict(cursor, result, evaluate),
        b'{' => {
            // A `{` is a lambda if it parses as one and a Dict if not.
            ret = get_lambda_tv(cursor, result, evaluate);
            if ret == Ok(Parsed::NotThis) {
                ret = eval_dict(cursor, result, evaluate, false);
            }
        }
        b'&' => ret = Parsed::done(eval_option(cursor, Some(result), evaluate)),
        b'$' => {
            ret = Parsed::done(if matches!(cursor.at(1), b'"' | b'\'') {
                eval_interp_string(cursor, result, evaluate)
            } else {
                eval_env_var(cursor, result, evaluate)
            });
        }
        b'@' => {
            cursor.bump(1);
            if evaluate {
                result.write_empty(VAR_STRING);
                // Sign-extended, as the C is: `**arg` is a `char`.
                let name = c_int::from(cursor.byte().cast_signed());
                let text = get_reg_contents(name, kGRegExprSrc as c_int);
                result.write_string_raw(text.cast());
            }
            // `@` at the very end of the line names no register.
            if cursor.byte() != NUL as u8 {
                cursor.bump(1);
            }
        }
        b'(' => {
            cursor.bump(1);
            cursor.skip_white();
            ret = Parsed::done(eval1(cursor, result, evaluate));
            if cursor.byte() == b')' {
                cursor.bump(1);
            } else if ret.is_ok() {
                emsg(gettext(c"E110: Missing ')'"));
                tv_clear(result);
                ret = Err(Failed);
            }
        }
        _ => ret = Ok(Parsed::NotThis),
    }

    if ret == Ok(Parsed::NotThis) {
        // Not a literal: it must be a name, and then either a call or a
        // variable.
        let start = cursor.offset();
        let (len, alias) = get_name_len(cursor, evaluate, true);
        if len <= 0 {
            ret = Err(Failed);
        } else {
            let name = alias
                .as_deref()
                .unwrap_or(&text[start..start + len as usize]);
            // A name may be followed by white space and then its arguments,
            // which the name scan has already stepped over.
            if cursor.byte() == b'(' {
                ret = Parsed::done(eval_func(cursor, name, result, evaluate, None));
            } else if evaluate {
                ret = Parsed::done(eval_variable(name, Some(&mut *result), true, false));
            } else {
                check_vars_named(name);
                // While skipping, `v:lua.x` still has to come out as
                // something callable. The name is `v:lua`: the `.x` is the
                // text after it, which is what is looked at.
                let after = alias.as_deref().unwrap_or(&text[start..]);
                if result.v_type() == VAR_UNKNOWN && after.starts_with(b"v:lua.") {
                    result.write_partial(lua_partial());
                }
                ret = Ok(Parsed::Done);
            }
        }
    }

    cursor.skip_white();
    if ret.is_ok() {
        ret = Parsed::done(handle_subscript(cursor, result, evaluate, true));
    }
    if ret.is_ok() && evaluate && end_leader > start_leader {
        ret = Parsed::done(eval7_leader(result, false, &text[start_leader..end_leader]).0);
    }
    ret.map(|_| ())
}

/// Apply the `!`/`-`/`+` prefixes an operand was preceded by, rightmost
/// first. `leaders` is the run of them, white space and all.
///
/// `numeric_only` stops at the first `!`, which is how a numeric literal
/// takes its sign without taking a logical negation that belongs to the
/// whole subscripted operand; the answer says how much of the run is left
/// for the caller to apply later.
pub(crate) fn eval7_leader(
    result: &mut TypVal,
    numeric_only: bool,
    leaders: &[u8],
) -> (Result<(), Failed>, usize) {
    let mut end_leader = leaders.len();
    let mut val: VarNumber = 0;
    let mut f: Float = 0.0;

    if result.v_type() == VAR_FLOAT {
        f = result.float_or_zero();
    } else {
        match tv_get_number_chk(result) {
            Ok(n) => val = n,
            Err(_) => {
                tv_clear(result);
                return (Err(Failed), end_leader);
            }
        }
    }

    while end_leader > 0 {
        end_leader -= 1;
        match leaders[end_leader] {
            b'!' => {
                if numeric_only {
                    end_leader += 1;
                    break;
                }
                if result.v_type() == VAR_FLOAT {
                    // Negating a Float leaves the value in `val` and the tag
                    // saying so, which is what makes a second `!` see a
                    // Number. The tag is overwritten below, so `!1.5` still
                    // answers a Number.
                    result.write_empty(VAR_BOOL);
                    val = VarNumber::from(if f == 0.0 {
                        kBoolVarTrue
                    } else {
                        kBoolVarFalse
                    });
                } else {
                    val = VarNumber::from(val == 0);
                }
            }
            // Vimscript arithmetic wraps, so negating VARNUMBER_MIN is
            // itself rather than an abort.
            b'-' => {
                if result.v_type() == VAR_FLOAT {
                    f = -f;
                } else {
                    val = val.wrapping_neg();
                }
            }
            // A `+` prefix does nothing at all.
            _ => {}
        }
    }
    let float = result.v_type() == VAR_FLOAT;
    tv_clear(result);
    if float {
        result.write_float(f);
    } else {
        result.write_number(val);
    }
    (Ok(()), end_leader)
}
