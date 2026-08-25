//! The recursive-descent evaluator, one function per precedence level.
//!
//! `eval0` is the entry; each level parses its own operators and hands the
//! rest down, so `eval1` is `? :`, `eval2` is `||`, `eval3` is `&&`, `eval4`
//! the comparisons, `eval5` `+`/`-`/`..`, `eval6` `*`/`/`/`%` and `eval7` an
//! operand with its subscripts.
//!
//! Every level takes `arg` by pointer and leaves it on the first byte it did
//! not consume, and every one of them returns `OK`/`FAIL` rather than a
//! `bool` because a hundred still-transpiled call sites test it that way.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int, c_ushort};
use core::ptr::{null_mut, write_bytes};

use crate::charset::{skipdigits, skipwhite};
use crate::eval::expr::arith::{
    eval_addblob, eval_addlist, eval_addsub_number, eval_concat_str, eval_multdiv_number,
};
use crate::eval::typval::{
    tv_check_num, tv_check_str, tv_clear, tv_empty_string, tv_get_number_chk, tv2bool,
};
use crate::eval::userfunc::{
    call_simple_func, call_simple_luafunc, deref_func_name, get_func_tv, get_lambda_tv,
};
use crate::eval::vars::{check_vars, eval_variable, get_vim_var_partial};
use crate::eval::{
    _ISalnum, EVAL_EVALUATE, EXPR_EQUAL, EXPR_GEQUAL, EXPR_GREATER, EXPR_IS, EXPR_ISNOT,
    EXPR_MATCH, EXPR_NEQUAL, EXPR_NOMATCH, EXPR_SEQUAL, EXPR_SMALLER, EXPR_UNKNOWN, FUNCEXE_INIT,
    NOTDONE, e_expression_too_recursive_str, eval_dict, eval_env_var, eval_interp_string,
    eval_list, eval_lit_dict, eval_lit_string, eval_number, eval_option, eval_string, get_name_len,
    handle_subscript, kGRegExprSrc, skip_luafunc_name, to_name_end, typval_compare,
};
use crate::ex_docmd::{check_nextcmd, ends_excmd};
use crate::ex_eval::aborting;
use crate::global_cell::GlobalCell;
use crate::main::{called_emsg, curwin, did_emsg, e_invexpr2, e_trailing_arg, p_ic};
use crate::memory::{strnequal, xfree, xmemdupz};
use crate::message::emsg;
use crate::os::cshim::{__ctype_b_loc, gettext, strncmp, strstr};
use crate::register::get_reg_contents;
use crate::types::{
    FAIL, NUL, OK, VAR_BLOB, VAR_BOOL, VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_NUMBER, VAR_PARTIAL,
    VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, Vv, dictitem_T, evalarg_T, exarg_T, exprtype_T, float_T,
    funcexe_T, kBoolVarFalse, kBoolVarTrue, partial_T, size_t, typval_T, typval_vval_union,
    varnumber_T,
};

/// A freshly declared typval, which is what every level starts a second
/// operand as.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VAR_UNLOCKED,
    vval: typval_vval_union { v_number: 0 },
};

/// The `evalarg_T` a level substitutes when the caller supplied none. It
/// exists only to carry the "do not evaluate" flag across a short circuit.
const BORROWED_EVALARG: evalarg_T = evalarg_T {
    eval_flags: 0,
    eval_getline: None,
    eval_cookie: null_mut(),
    eval_tofree: null_mut(),
};

/// `isalnum` in the process locale, which is what decides whether `is` and
/// `isnot` stand as whole words. nvim calls `setlocale(LC_ALL, "")` at
/// startup, so this is deliberately not the ASCII test.
fn isalnum_locale(c: u8) -> bool {
    // SAFETY: `__ctype_b_loc` yields a table valid over the whole byte range.
    unsafe { *(*__ctype_b_loc()).offset(c as isize) & _ISalnum as c_ushort != 0 }
}

/// The caller's flags with evaluation switched off unless `on`.
fn flags_evaluating(orig: c_int, on: bool) -> c_int {
    if on {
        orig
    } else {
        orig & !(EVAL_EVALUATE as c_int)
    }
}

/// Is this `evalarg` asking for the expression to actually be evaluated?
///
/// # Safety
/// `evalarg` must be null or a valid `evalarg_T`.
unsafe fn evaluating(evalarg: *const evalarg_T) -> bool {
    !evalarg.is_null() && unsafe { (*evalarg).eval_flags } & EVAL_EVALUATE as c_int != 0
}

/// Call a function by name, having parsed the name but not the arguments.
///
/// # Safety
/// `arg` must point at the cursor, positioned on the `(`.
pub(crate) unsafe fn eval_func(
    arg: *mut *mut c_char,
    evalarg: *mut evalarg_T,
    name: *mut c_char,
    name_len: c_int,
    rettv: *mut typval_T,
    flags: c_int,
    basetv: *mut typval_T,
) -> c_int {
    unsafe {
        let evaluate = flags & EVAL_EVALUATE as c_int != 0;
        let mut len = name_len;
        let mut found_var = false;
        if !evaluate {
            check_vars(name, len as size_t);
        }
        let mut partial: *mut partial_T = null_mut();
        let resolved = deref_func_name(
            name,
            &raw mut len,
            &raw mut partial,
            !evaluate,
            &raw mut found_var,
        );
        // `get_func_tv` may re-enter the evaluator, so the name has to
        // outlive whatever `resolved` pointed into.
        let owned = xmemdupz(resolved.cast(), len as size_t) as *mut c_char;

        let mut funcexe: funcexe_T = FUNCEXE_INIT;
        funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
        funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
        funcexe.fe_evaluate = evaluate;
        funcexe.fe_partial = partial;
        funcexe.fe_basetv = basetv;
        funcexe.fe_found_var = found_var;
        let mut ret = get_func_tv(owned, len, rettv, arg, evalarg, &raw mut funcexe);
        xfree(owned.cast());

        // While skipping, a name that was never resolved still has to look
        // like a Funcref so the subscript handling can go on.
        if (*rettv).v_type == VAR_UNKNOWN && !evaluate && **arg == b'(' as c_char {
            (*rettv).vval.v_string = tv_empty_string.get() as *mut c_char;
            (*rettv).v_type = VAR_FUNC;
        }
        if evaluate && aborting() {
            if ret == OK {
                tv_clear(rettv);
            }
            ret = FAIL;
        }
        ret
    }
}

/// Release the line `evalarg` took ownership of while reading a
/// continuation, handing it back to the Ex command line when there is one.
///
/// # Safety
/// `evalarg` may be null; `eap` may be null.
pub(crate) unsafe fn clear_evalarg(evalarg: *mut evalarg_T, eap: *mut exarg_T) {
    unsafe {
        if evalarg.is_null() || (*evalarg).eval_tofree.is_null() {
            return;
        }
        if eap.is_null() {
            xfree((*evalarg).eval_tofree.cast());
        } else {
            xfree((*eap).cmdline_tofree.cast());
            (*eap).cmdline_tofree = *(*eap).cmdlinep;
            *(*eap).cmdlinep = (*evalarg).eval_tofree;
        }
        (*evalarg).eval_tofree = null_mut();
    }
}

/// Evaluate a whole expression, which must be all that is left of the line.
///
/// # Safety
/// `arg` must be a NUL-terminated expression; `eap` may be null.
pub unsafe fn eval0(
    arg: *mut c_char,
    rettv: *mut typval_T,
    eap: *mut exarg_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    unsafe {
        let did_emsg_before = did_emsg.get();
        let called_emsg_before = called_emsg.get();
        let mut p = skipwhite(arg);
        let ret = eval1(&raw mut p, rettv, evalarg);
        // Anything left over is an error, but only once the expression
        // itself parsed.
        let end_error = ret != FAIL && ends_excmd(*p as c_int) == 0;

        if ret == FAIL || end_error {
            if ret != FAIL {
                tv_clear(rettv);
            }
            // Stay quiet if something already reported, or if we are
            // unwinding from an exception.
            if !aborting()
                && did_emsg.get() == did_emsg_before
                && called_emsg.get() == called_emsg_before
            {
                if end_error {
                    semsg_c!(gettext((&raw const e_trailing_arg).cast()), p);
                } else {
                    semsg_c!(gettext((&raw const e_invexpr2).cast()), arg);
                }
            }
            if !eap.is_null() && !p.is_null() {
                let nextcmd = check_nextcmd(p);
                if !nextcmd.is_null() && *nextcmd != b'|' as c_char {
                    (*eap).nextcmd = nextcmd;
                }
            }
            return FAIL;
        }
        if !eap.is_null() {
            (*eap).nextcmd = check_nextcmd(p);
        }
        ret
    }
}

/// Shortcut for a whole expression that is nothing but one call: `Foo()`.
///
/// Answers `NOTDONE` when the expression is anything else.
///
/// # Safety
/// `arg` must be a NUL-terminated expression.
pub(crate) unsafe fn may_call_simple_func(arg: *const c_char, rettv: *mut typval_T) -> c_int {
    unsafe {
        let parens = strstr(arg, c"()".as_ptr());
        if parens.is_null() || *skipwhite(parens.add(2)) as c_int != NUL {
            return NOTDONE;
        }
        if strnequal(arg, c"v:lua.".as_ptr(), 6) {
            let p = arg.add(6);
            if p != parens && skip_luafunc_name(p) == parens {
                return call_simple_luafunc(p, parens.offset_from(p) as size_t, rettv);
            }
        } else {
            // A script-local name arrives as `<SNR>123_name`.
            let p = if strncmp(arg, c"<SNR>".as_ptr(), 5) == 0 {
                skipdigits(arg.add(5)) as *const c_char
            } else {
                arg
            };
            if to_name_end(p, true) == parens {
                return call_simple_func(arg, parens.offset_from(arg) as size_t, rettv);
            }
        }
        NOTDONE
    }
}

/// `eval0` with the single-call shortcut tried first.
///
/// # Safety
/// As `eval0`.
pub(crate) unsafe fn eval0_simple_funccal(
    arg: *mut c_char,
    rettv: *mut typval_T,
    eap: *mut exarg_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    unsafe {
        let r = may_call_simple_func(arg, rettv);
        if r == NOTDONE {
            eval0(arg, rettv, eap, evalarg)
        } else {
            r
        }
    }
}

/// `? :` and `??`, the lowest-precedence level.
///
/// # Safety
/// `arg` must point at the cursor into a NUL-terminated expression.
pub(crate) unsafe fn eval1(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    unsafe {
        write_bytes(rettv, 0, 1);
        if eval2(arg, rettv, evalarg) == FAIL {
            return FAIL;
        }
        let mut p = *arg;
        if *p != b'?' as c_char {
            return OK;
        }
        let op_falsy = *p.add(1) == b'?' as c_char;

        let mut local_evalarg = BORROWED_EVALARG;
        let used = if evalarg.is_null() {
            &raw mut local_evalarg
        } else {
            evalarg
        };
        let orig_flags = (*used).eval_flags;
        let evaluate = (*used).eval_flags & EVAL_EVALUATE as c_int != 0;

        let mut result = false;
        if evaluate {
            let mut error = false;
            if op_falsy {
                result = tv2bool(rettv);
            } else {
                result = tv_get_number_chk(rettv, &raw mut error) != 0;
            }
            // `??` keeps the left operand when it is truthy; `? :` never
            // does, and neither keeps it after an error.
            if error || !op_falsy || !result {
                tv_clear(rettv);
            }
            if error {
                return FAIL;
            }
        }

        if op_falsy {
            *arg = (*arg).add(1);
        }
        *arg = skipwhite((*arg).add(1));
        (*used).eval_flags = flags_evaluating(orig_flags, if op_falsy { !result } else { result });

        let mut var2 = UNSET_TV;
        if eval1(arg, &raw mut var2, used) == FAIL {
            (*used).eval_flags = orig_flags;
            return FAIL;
        }
        if !op_falsy || !result {
            *rettv = var2;
        }

        if !op_falsy {
            p = *arg;
            if *p != b':' as c_char {
                emsg(gettext(c"E109: Missing ':' after '?'".as_ptr()));
                if evaluate && result {
                    tv_clear(rettv);
                }
                (*used).eval_flags = orig_flags;
                return FAIL;
            }
            *arg = skipwhite((*arg).add(1));
            (*used).eval_flags = flags_evaluating(orig_flags, !result);
            if eval1(arg, &raw mut var2, used) == FAIL {
                if evaluate && result {
                    tv_clear(rettv);
                }
                (*used).eval_flags = orig_flags;
                return FAIL;
            }
            if evaluate && !result {
                *rettv = var2;
            }
        }

        if evalarg.is_null() {
            clear_evalarg(&raw mut local_evalarg, null_mut());
        } else {
            (*evalarg).eval_flags = orig_flags;
        }
        OK
    }
}

/// `||` and `&&`, which differ only in what settles the answer early.
///
/// `stop_at` is the result that makes the remaining operands irrelevant:
/// true for `||`, false for `&&`. Everything else — the initial value, when
/// to keep evaluating, and when to fold the operand in — follows from it.
///
/// # Safety
/// As `eval1`.
unsafe fn eval_logical(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    operand: unsafe fn(*mut *mut c_char, *mut typval_T, *mut evalarg_T) -> c_int,
    op: u8,
    stop_at: bool,
) -> c_int {
    unsafe {
        if operand(arg, rettv, evalarg) == FAIL {
            return FAIL;
        }
        let mut p = *arg;
        let is_op = |p: *const c_char| *p == op as c_char && *p.add(1) == op as c_char;
        if !is_op(p) {
            return OK;
        }

        let mut local_evalarg = BORROWED_EVALARG;
        let used = if evalarg.is_null() {
            &raw mut local_evalarg
        } else {
            evalarg
        };
        let orig_flags = (*used).eval_flags;
        let evaluate = (*used).eval_flags & EVAL_EVALUATE as c_int != 0;

        let mut result = !stop_at;
        if evaluate {
            let mut error = false;
            result = tv_get_number_chk(rettv, &raw mut error) != 0;
            tv_clear(rettv);
            if error {
                return FAIL;
            }
        }

        while is_op(p) {
            *arg = skipwhite((*arg).add(2));
            (*used).eval_flags = flags_evaluating(orig_flags, result != stop_at);
            let mut var2 = UNSET_TV;
            if operand(arg, &raw mut var2, used) == FAIL {
                return FAIL;
            }
            if evaluate && result != stop_at {
                let mut error = false;
                result = tv_get_number_chk(&raw mut var2, &raw mut error) != 0;
                tv_clear(&raw mut var2);
                if error {
                    return FAIL;
                }
            }
            if evaluate {
                (*rettv).v_type = VAR_NUMBER;
                (*rettv).vval.v_number = varnumber_T::from(result);
            }
            p = *arg;
        }

        if evalarg.is_null() {
            clear_evalarg(&raw mut local_evalarg, null_mut());
        } else {
            (*evalarg).eval_flags = orig_flags;
        }
        OK
    }
}

/// `||`.
///
/// # Safety
/// As `eval1`.
pub(crate) unsafe fn eval2(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    unsafe { eval_logical(arg, rettv, evalarg, eval3, b'|', true) }
}

/// `&&`.
///
/// # Safety
/// As `eval1`.
pub(crate) unsafe fn eval3(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    unsafe { eval_logical(arg, rettv, evalarg, eval4, b'&', false) }
}

/// Recognise a comparison operator, answering it and how many bytes it took.
///
/// The second byte is read *inside* each arm, never before the `match`:
/// `eval5` may well have left the cursor on the terminating NUL, and the
/// first byte matching an operator character is the only thing that proves
/// there is a second one.
///
/// # Safety
/// `p` must point into a NUL-terminated expression.
unsafe fn comparison_at(p: *const c_char) -> (exprtype_T, c_int) {
    unsafe {
        let next = || *p.add(1) as u8;
        match *p as u8 {
            b'=' => match next() {
                b'=' => (EXPR_EQUAL, 2),
                b'~' => (EXPR_MATCH, 2),
                _ => (EXPR_UNKNOWN, 2),
            },
            b'!' => match next() {
                b'=' => (EXPR_NEQUAL, 2),
                b'~' => (EXPR_NOMATCH, 2),
                _ => (EXPR_UNKNOWN, 2),
            },
            b'>' if next() == b'=' => (EXPR_GEQUAL, 2),
            b'>' => (EXPR_GREATER, 1),
            b'<' if next() == b'=' => (EXPR_SEQUAL, 2),
            b'<' => (EXPR_SMALLER, 1),
            b'i' if next() == b's' => {
                let len = if *p.add(2) as u8 == b'n'
                    && *p.add(3) as u8 == b'o'
                    && *p.add(4) as u8 == b't'
                {
                    5
                } else {
                    2
                };
                // `isnothing` is a name, not `isnot` followed by `hing`.
                let after = *p.add(len as usize) as u8;
                if !isalnum_locale(after) && after != b'_' {
                    (if len == 2 { EXPR_IS } else { EXPR_ISNOT }, len)
                } else {
                    (EXPR_UNKNOWN, 2)
                }
            }
            _ => (EXPR_UNKNOWN, 2),
        }
    }
}

/// The comparison operators.
///
/// # Safety
/// As `eval1`.
pub(crate) unsafe fn eval4(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    unsafe {
        if eval5(arg, rettv, evalarg) == FAIL {
            return FAIL;
        }
        let p = *arg;
        let (op, mut len) = comparison_at(p);
        if op == EXPR_UNKNOWN {
            return OK;
        }

        // A trailing `?` or `#` overrides 'ignorecase' for this comparison.
        let ic = match *p.add(len as usize) as u8 {
            b'?' => {
                len += 1;
                true
            }
            b'#' => {
                len += 1;
                false
            }
            _ => p_ic.get() != 0,
        };

        *arg = skipwhite(p.add(len as usize));
        let mut var2 = UNSET_TV;
        if eval5(arg, &raw mut var2, evalarg) == FAIL {
            tv_clear(rettv);
            return FAIL;
        }
        if evaluating(evalarg) {
            let ret = typval_compare(rettv, &raw mut var2, op, ic);
            tv_clear(&raw mut var2);
            return ret;
        }
        OK
    }
}

/// `+`, `-` and the two spellings of string concatenation.
///
/// # Safety
/// As `eval1`.
pub(crate) unsafe fn eval5(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    unsafe {
        if eval6(arg, rettv, evalarg, false) == FAIL {
            return FAIL;
        }
        loop {
            let op = **arg as u8;
            let concat = op == b'.';
            if op != b'+' && op != b'-' && !concat {
                return OK;
            }
            let evaluate = evaluating(evalarg);

            // Reject an operand of the wrong type before consuming the
            // operator — but not for the two cases that have their own
            // handling: `+` on a List or Blob, and anything on a Float.
            let container_plus =
                op == b'+' && ((*rettv).v_type == VAR_LIST || (*rettv).v_type == VAR_BLOB);
            let float_arith = op != b'.' && (*rettv).v_type == VAR_FLOAT;
            if !container_plus && !float_arith && evaluate {
                let ok = if concat {
                    tv_check_str(rettv)
                } else {
                    tv_check_num(rettv)
                };
                if !ok {
                    tv_clear(rettv);
                    return FAIL;
                }
            }

            // `..` is two bytes, `.` one.
            if concat && *(*arg).add(1) == b'.' as c_char {
                *arg = (*arg).add(1);
            }
            *arg = skipwhite((*arg).add(1));

            let mut var2 = UNSET_TV;
            if eval6(arg, &raw mut var2, evalarg, concat) == FAIL {
                tv_clear(rettv);
                return FAIL;
            }
            if evaluate {
                let ok = if concat {
                    eval_concat_str(rettv, &raw mut var2)
                } else if op == b'+' && (*rettv).v_type == VAR_BLOB && var2.v_type == VAR_BLOB {
                    eval_addblob(rettv, &raw mut var2);
                    true
                } else if op == b'+' && (*rettv).v_type == VAR_LIST && var2.v_type == VAR_LIST {
                    eval_addlist(rettv, &raw mut var2)
                } else {
                    eval_addsub_number(rettv, &raw mut var2, op)
                };
                if !ok {
                    return FAIL;
                }
                tv_clear(&raw mut var2);
            }
        }
    }
}

/// `*`, `/` and `%`.
///
/// # Safety
/// As `eval1`.
pub(crate) unsafe fn eval6(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    want_string: bool,
) -> c_int {
    unsafe {
        if eval7(arg, rettv, evalarg, want_string) == FAIL {
            return FAIL;
        }
        loop {
            let op = **arg as u8;
            if op != b'*' && op != b'/' && op != b'%' {
                return OK;
            }
            let evaluate = evaluating(evalarg);
            *arg = skipwhite((*arg).add(1));
            let mut var2 = UNSET_TV;
            if eval7(arg, &raw mut var2, evalarg, false) == FAIL {
                return FAIL;
            }
            if evaluate && !eval_multdiv_number(rettv, &raw mut var2, op) {
                return FAIL;
            }
        }
    }
}

/// An operand, with the `!`/`-`/`+` prefixes and any subscripts.
///
/// # Safety
/// As `eval1`.
pub(crate) unsafe fn eval7(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    want_string: bool,
) -> c_int {
    /// How deep `eval7` is into itself. The guard is what stops a
    /// self-referential expression from exhausting the C stack.
    static RECURSE: GlobalCell<c_int> = GlobalCell::new(0);
    const MAX_RECURSE: c_int = 1000;

    unsafe {
        let evaluate = evaluating(evalarg);
        let mut ret = OK;
        (*rettv).v_type = VAR_UNKNOWN;

        // The prefixes are collected now and applied last, so that `-1` is
        // parsed as a negated literal but `!x[0]` negates the subscript.
        let start_leader: *const c_char = *arg;
        while matches!(**arg as u8, b'!' | b'-' | b'+') {
            *arg = skipwhite((*arg).add(1));
        }
        let mut end_leader: *const c_char = *arg;

        if RECURSE.get() == MAX_RECURSE {
            semsg_c!(gettext(e_expression_too_recursive_str.as_ptr()), *arg);
            return FAIL;
        }
        RECURSE.set(RECURSE.get() + 1);

        match **arg as u8 {
            b'0'..=b'9' => {
                ret = eval_number(arg, rettv, evaluate, want_string);
                // A number applies its prefixes here, where `-` still means
                // arithmetic negation rather than "negate what follows".
                if ret == OK && evaluate && end_leader > start_leader {
                    ret = eval7_leader(rettv, true, start_leader, &raw mut end_leader);
                }
            }
            b'"' => ret = eval_string(arg, rettv, evaluate, false),
            b'\'' => ret = eval_lit_string(arg, rettv, evaluate, false),
            b'[' => ret = eval_list(arg, rettv, evalarg),
            b'#' => ret = eval_lit_dict(arg, rettv, evalarg),
            b'{' => {
                // A `{` is a lambda if it parses as one and a Dict if not.
                ret = get_lambda_tv(arg, rettv, evalarg);
                if ret == NOTDONE {
                    ret = eval_dict(arg, rettv, evalarg, false);
                }
            }
            b'&' => ret = eval_option(arg as *mut *const c_char, rettv, evaluate),
            b'$' => {
                ret = if matches!(*(*arg).add(1) as u8, b'"' | b'\'') {
                    eval_interp_string(arg, rettv, evaluate)
                } else {
                    eval_env_var(arg, rettv, evaluate)
                };
            }
            b'@' => {
                *arg = (*arg).add(1);
                if evaluate {
                    (*rettv).v_type = VAR_STRING;
                    (*rettv).vval.v_string =
                        get_reg_contents(**arg as c_int, kGRegExprSrc as c_int) as *mut c_char;
                }
                // `@` at the very end of the line names no register.
                if **arg as c_int != NUL {
                    *arg = (*arg).add(1);
                }
            }
            b'(' => {
                *arg = skipwhite((*arg).add(1));
                ret = eval1(arg, rettv, evalarg);
                if **arg == b')' as c_char {
                    *arg = (*arg).add(1);
                } else if ret == OK {
                    emsg(gettext(c"E110: Missing ')'".as_ptr()));
                    tv_clear(rettv);
                    ret = FAIL;
                }
            }
            _ => ret = NOTDONE,
        }

        if ret == NOTDONE {
            // Not a literal: it must be a name, and then either a call or a
            // variable.
            let mut alias: *mut c_char = null_mut();
            let start = *arg;
            let len = get_name_len(arg as *mut *const c_char, &raw mut alias, evaluate, true);
            let name = if alias.is_null() { start } else { alias };
            if len <= 0 {
                ret = FAIL;
            } else {
                let flags = if evalarg.is_null() {
                    0
                } else {
                    (*evalarg).eval_flags
                };
                if *skipwhite(*arg) == b'(' as c_char {
                    *arg = skipwhite(*arg);
                    ret = eval_func(arg, evalarg, name, len, rettv, flags, null_mut());
                } else if evaluate {
                    ret =
                        eval_variable(name, len, rettv, null_mut::<*mut dictitem_T>(), true, false);
                } else {
                    check_vars(name, len as size_t);
                    // While skipping, `v:lua.x` still has to come out as
                    // something callable.
                    if (*rettv).v_type == VAR_UNKNOWN && strnequal(name, c"v:lua.".as_ptr(), 6) {
                        (*rettv).v_type = VAR_PARTIAL;
                        (*rettv).vval.v_partial = get_vim_var_partial(Vv::Lua);
                        (*(*rettv).vval.v_partial).pt_refcount += 1;
                    }
                    ret = OK;
                }
            }
            xfree(alias.cast());
        }

        *arg = skipwhite(*arg);
        if ret == OK {
            ret = handle_subscript(arg as *mut *const c_char, rettv, evalarg, true);
        }
        if ret == OK && evaluate && end_leader > start_leader {
            ret = eval7_leader(rettv, false, start_leader, &raw mut end_leader);
        }
        RECURSE.set(RECURSE.get() - 1);
        ret
    }
}

/// Apply the `!`/`-`/`+` prefixes an operand was preceded by, rightmost
/// first.
///
/// `numeric_only` stops at the first `!`, which is how a numeric literal
/// takes its sign without taking a logical negation that belongs to the
/// whole subscripted operand; the caller is told where it stopped through
/// `end_leaderp`.
///
/// # Safety
/// `start_leader` and `*end_leaderp` must bound the run of prefixes.
pub(crate) unsafe fn eval7_leader(
    rettv: *mut typval_T,
    numeric_only: bool,
    start_leader: *const c_char,
    end_leaderp: *mut *const c_char,
) -> c_int {
    unsafe {
        let mut end_leader = *end_leaderp;
        let mut ret = OK;
        let mut error = false;
        let mut val: varnumber_T = 0;
        let mut f: float_T = 0.0;

        if (*rettv).v_type == VAR_FLOAT {
            f = (*rettv).vval.v_float;
        } else {
            val = tv_get_number_chk(rettv, &raw mut error);
        }

        if error {
            tv_clear(rettv);
            ret = FAIL;
        } else {
            while end_leader > start_leader {
                end_leader = end_leader.sub(1);
                match *end_leader as u8 {
                    b'!' => {
                        if numeric_only {
                            end_leader = end_leader.add(1);
                            break;
                        }
                        if (*rettv).v_type == VAR_FLOAT {
                            // Negating a Float leaves the value in `val` and
                            // the tag saying so, which is what makes a second
                            // `!` see a Number. The tag is overwritten below,
                            // so `!1.5` still answers a Number.
                            (*rettv).v_type = VAR_BOOL;
                            val = varnumber_T::from(if f == 0.0 {
                                kBoolVarTrue
                            } else {
                                kBoolVarFalse
                            });
                        } else {
                            val = varnumber_T::from(val == 0);
                        }
                    }
                    // Vimscript arithmetic wraps, so negating VARNUMBER_MIN
                    // is itself rather than an abort.
                    b'-' => {
                        if (*rettv).v_type == VAR_FLOAT {
                            f = -f;
                        } else {
                            val = val.wrapping_neg();
                        }
                    }
                    // A `+` prefix does nothing at all.
                    _ => {}
                }
            }
            if (*rettv).v_type == VAR_FLOAT {
                tv_clear(rettv);
                (*rettv).vval.v_float = f;
            } else {
                tv_clear(rettv);
                (*rettv).v_type = VAR_NUMBER;
                (*rettv).vval.v_number = val;
            }
        }

        *end_leaderp = end_leader;
        ret
    }
}
