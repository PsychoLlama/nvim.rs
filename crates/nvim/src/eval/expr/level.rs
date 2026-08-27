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
use crate::winlayer::{Ea, Live};
use core::ffi::{c_char, c_int};
use core::ptr::{null_mut, write_bytes};

use crate::charset::{skipdigits, skipwhite};
use crate::eval::expr::arith::{
    eval_addblob, eval_addlist, eval_addsub_number, eval_concat_str, eval_multdiv_number,
};
use crate::eval::typval::{tv_check_num, tv_check_str, tv_clear, tv_get_number_chk, tv2bool};
use crate::eval::userfunc::{call_simple_func, call_simple_luafunc, get_lambda_tv};
use crate::eval::vars::{check_vars, eval_variable, get_vim_var_partial};
use crate::eval::{
    EVAL_EVALUATE, EXPR_UNKNOWN, NOTDONE, Tv, comparison_at, e_expression_too_recursive_str,
    eval_dict, eval_env_var, eval_func, eval_interp_string, eval_list, eval_lit_dict,
    eval_lit_string, eval_number, eval_option, eval_string, get_name_len, handle_subscript,
    kGRegExprSrc, skip_luafunc_name, to_name_end, typval_compare,
};
use crate::ex_docmd::{check_nextcmd, ends_excmd};
use crate::ex_eval::aborting;
use crate::global_cell::GlobalCell;
use crate::main::{called_emsg, did_emsg, e_invexpr2, e_trailing_arg, p_ic};
use crate::memory::{strnequal, xfree};
use crate::message::emsg;
use crate::os::cshim::{gettext, strncmp, strstr};
use crate::register::get_reg_contents;
use crate::types::{
    FAIL, NUL, OK, VAR_BLOB, VAR_BOOL, VAR_FLOAT, VAR_LIST, VAR_NUMBER, VAR_PARTIAL, VAR_STRING,
    VAR_UNKNOWN, VarLock, Vv, dictitem_T, evalarg_T, exarg_T, float_T, kBoolVarFalse, kBoolVarTrue,
    size_t, typval_T, typval_vval_union, varnumber_T,
};

/// A freshly declared typval, which is what every level starts a second
/// operand as.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VarLock::Unlocked,
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

/// The `evalarg_T` that says "evaluate, and read no continuation lines".
///
/// A value, not a cell as the C has it: the levels below write `eval_flags`
/// while they short-circuit and restore it afterwards, so a caller sharing
/// one with a nested evaluation would be lending it a mutable scratch. Each
/// declares its own — `let mut evalarg = EVALARG_EVALUATE;`.
pub(crate) const EVALARG_EVALUATE: evalarg_T = evalarg_T {
    eval_flags: EVAL_EVALUATE as c_int,
    ..BORROWED_EVALARG
};

/// The cursor an expression is parsed through: the `*mut *mut c_char` every
/// level takes, reads bytes at and leaves on the first byte it did not
/// consume.
///
/// The same shape as [`Live<T>`](crate::winlayer::Live), for the same reason
/// — **construction is the one unsafe step, and every byte read after it is
/// ordinary checked code**. A `&mut *argp` instead would be `noalias` to
/// LLVM, and an operand may re-enter the evaluator through a function call,
/// an autocommand or Lua while this same cursor is still live up the stack.
#[derive(Clone, Copy)]
pub(crate) struct Cur(*mut *mut c_char);

impl Cur {
    /// # Safety
    /// `argp` must point at a live `*mut c_char` walking a NUL-terminated
    /// expression, and both must stay valid for as long as the cursor is.
    pub(crate) const unsafe fn new(argp: *mut *mut c_char) -> Self {
        Self(argp)
    }

    /// Where the cursor stands.
    pub(crate) fn get(self) -> *mut c_char {
        // SAFETY: the constructor's promise.
        unsafe { *self.0 }
    }

    /// Move it to `p`.
    pub(crate) fn set(self, p: *mut c_char) {
        // SAFETY: the constructor's promise.
        unsafe { *self.0 = p };
    }

    /// The byte `i` past the cursor.
    ///
    /// Reading past the terminating NUL would be out of bounds, so a caller
    /// asking for `i > 0` has already seen a non-NUL at every offset below
    /// it — which is why the levels below read the second byte of an
    /// operator only inside the arm the first byte selected.
    pub(crate) fn at(self, i: usize) -> u8 {
        // SAFETY: the constructor's promise, plus the caller's: the walk has
        // not stepped past the NUL.
        unsafe { *self.get().add(i) as u8 }
    }

    /// The byte under the cursor.
    pub(crate) fn byte(self) -> u8 {
        self.at(0)
    }

    /// Step it `n` bytes on.
    pub(crate) fn bump(self, n: usize) {
        self.set(self.get().wrapping_add(n));
    }

    /// Step it `n` bytes on and then past the white space, which is how a
    /// level consumes an operator it has recognised.
    pub(crate) fn skip(self, n: usize) {
        // SAFETY: the constructor's promise -- `skipwhite` stops at the NUL.
        self.set(unsafe { skipwhite(self.get().wrapping_add(n)) });
    }

    /// The pointer back, for the callees that still take one.
    pub(crate) fn raw(self) -> *mut *mut c_char {
        self.0
    }
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

/// Release the line `evalarg` took ownership of while reading a
/// continuation, handing it back to the Ex command line when there is one.
///
/// # Safety
/// `evalarg` may be null; `eap` may be null.
pub(crate) unsafe fn clear_evalarg(evalarg: *mut evalarg_T, eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- `evalarg` is null or valid.
    if evalarg.is_null() || unsafe { (*evalarg).eval_tofree }.is_null() {
        return;
    }
    // SAFETY: as above, and `evalarg` is not null.
    let mut ev = unsafe { Live::new(evalarg) };
    if eap.is_null() {
        // SAFETY: `eval_tofree` is the line this `evalarg` owns.
        unsafe { xfree(ev.eval_tofree.cast()) };
    } else {
        // SAFETY: the caller's promise -- `eap` is not null here, and its
        // `cmdlinep` names the command line being run.
        let mut ea = unsafe { Ea::new(eap) };
        // SAFETY: `cmdline_tofree` is the line the command owns.
        unsafe { xfree(ea.cmdline_tofree.cast()) };
        // SAFETY: as above -- `cmdlinep` is a live `*mut c_char`.
        ea.cmdline_tofree = unsafe { *ea.cmdlinep };
        unsafe { *ea.cmdlinep = ev.eval_tofree };
    }
    ev.eval_tofree = null_mut();
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
    let did_emsg_before = did_emsg.get();
    let called_emsg_before = called_emsg.get();
    // SAFETY: the caller's promise -- `arg` is a NUL-terminated expression,
    // so `p` walks it and `&raw mut p` is this frame's own cursor.
    let mut p = unsafe { skipwhite(arg) };
    let ret = unsafe { eval1(&raw mut p, rettv, evalarg) };
    // Anything left over is an error, but only once the expression
    // itself parsed.
    // SAFETY: `eval1` left `p` inside the expression.
    let end_error = ret != FAIL && ends_excmd(unsafe { *p } as c_int) == 0;

    if ret == FAIL || end_error {
        if ret != FAIL {
            // SAFETY: the caller's promise -- `rettv` is valid.
            unsafe { tv_clear(rettv) };
        }
        // Stay quiet if something already reported, or if we are
        // unwinding from an exception.
        if !aborting()
            && did_emsg.get() == did_emsg_before
            && called_emsg.get() == called_emsg_before
        {
            let (fmt, subject) = if end_error {
                ((&raw const e_trailing_arg).cast(), p)
            } else {
                ((&raw const e_invexpr2).cast(), arg)
            };
            // SAFETY: both messages take one NUL-terminated string, and
            // `p` and `arg` are tails of the expression.
            unsafe { semsg_c!(gettext(fmt), subject) };
        }
        if !eap.is_null() && !p.is_null() {
            // SAFETY: `p` is inside the expression; `eap` is not null.
            let nextcmd = unsafe { check_nextcmd(p) };
            if !nextcmd.is_null() && unsafe { *nextcmd } != b'|' as c_char {
                unsafe { (*eap).nextcmd = nextcmd };
            }
        }
        return FAIL;
    }
    if !eap.is_null() {
        // SAFETY: as above.
        unsafe { (*eap).nextcmd = check_nextcmd(p) };
    }
    ret
}

/// Shortcut for a whole expression that is nothing but one call: `Foo()`.
///
/// Answers `NOTDONE` when the expression is anything else.
///
/// # Safety
/// `arg` must be a NUL-terminated expression.
pub(crate) unsafe fn may_call_simple_func(arg: *const c_char, rettv: *mut typval_T) -> c_int {
    // SAFETY: the caller's promise -- `arg` is a NUL-terminated expression,
    // so `parens` is inside it and the two bytes of `()` precede its tail.
    let parens = unsafe { strstr(arg, c"()".as_ptr()) };
    if parens.is_null() || unsafe { *skipwhite(parens.add(2)) } as c_int != NUL {
        return NOTDONE;
    }
    // SAFETY: as above, for every walk of `arg` below.
    if unsafe { strnequal(arg, c"v:lua.".as_ptr(), 6) } {
        let p = unsafe { arg.add(6) };
        if p != parens && unsafe { skip_luafunc_name(p) } == parens {
            let len = unsafe { parens.offset_from(p) } as size_t;
            return unsafe { call_simple_luafunc(p, len, rettv) };
        }
    } else {
        // A script-local name arrives as `<SNR>123_name`.
        let snr = unsafe { strncmp(arg, c"<SNR>".as_ptr(), 5) } == 0;
        let p = if snr {
            (unsafe { skipdigits(arg.add(5)) }) as *const c_char
        } else {
            arg
        };
        if unsafe { to_name_end(p, true) } == parens {
            let len = unsafe { parens.offset_from(arg) } as size_t;
            return unsafe { call_simple_func(arg, len, rettv) };
        }
    }
    NOTDONE
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
    // SAFETY: the caller's promise, handed straight on to both.
    let r = unsafe { may_call_simple_func(arg, rettv) };
    if r == NOTDONE {
        unsafe { eval0(arg, rettv, eap, evalarg) }
    } else {
        r
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
    // SAFETY: the caller's promise -- `arg` is the cursor into the
    // expression, `rettv` is the result being built and is written whole
    // before anything reads it.
    let (cur, mut rv) = unsafe { (Cur::new(arg), Tv::new(rettv)) };
    unsafe { write_bytes(rettv, 0, 1) };
    if unsafe { eval2(arg, rettv, evalarg) } == FAIL {
        return FAIL;
    }
    if cur.byte() != b'?' {
        return OK;
    }
    let op_falsy = cur.at(1) == b'?';

    let mut local_evalarg = BORROWED_EVALARG;
    // SAFETY: the caller's promise -- `evalarg` is null or valid; when it is
    // null the substitute is this frame's own.
    let borrowed = if evalarg.is_null() {
        &raw mut local_evalarg
    } else {
        evalarg
    };
    let mut used = unsafe { Live::new(borrowed) };
    let orig_flags = used.eval_flags;
    let evaluate = orig_flags & EVAL_EVALUATE as c_int != 0;

    let mut result = false;
    if evaluate {
        let mut error = false;
        // SAFETY: `rettv` is the operand `eval2` just parsed.
        result = if op_falsy {
            unsafe { tv2bool(rettv) }
        } else {
            let n = unsafe { tv_get_number_chk(rettv, &raw mut error) };
            n != 0
        };
        // `??` keeps the left operand when it is truthy; `? :` never
        // does, and neither keeps it after an error.
        if error || !op_falsy || !result {
            // SAFETY: as above.
            unsafe { tv_clear(rettv) };
        }
        if error {
            return FAIL;
        }
    }

    // `??` is two bytes, `?` one, and white space follows either.
    cur.skip(if op_falsy { 2 } else { 1 });
    used.eval_flags = flags_evaluating(orig_flags, if op_falsy { !result } else { result });

    let mut var2 = UNSET_TV;
    // SAFETY: `cur` is still the caller's cursor, `var2` is this frame's own
    // and `used` is the `evalarg` settled above.
    if unsafe { eval1(arg, &raw mut var2, used.raw()) } == FAIL {
        used.eval_flags = orig_flags;
        return FAIL;
    }
    if !op_falsy || !result {
        *rv = var2;
    }

    if !op_falsy {
        if cur.byte() != b':' {
            // SAFETY: a literal message, and `rettv` is the caller's.
            unsafe { emsg(gettext(c"E109: Missing ':' after '?'".as_ptr())) };
            if evaluate && result {
                unsafe { tv_clear(rettv) };
            }
            used.eval_flags = orig_flags;
            return FAIL;
        }
        cur.skip(1);
        used.eval_flags = flags_evaluating(orig_flags, !result);
        // SAFETY: as the first branch.
        if unsafe { eval1(arg, &raw mut var2, used.raw()) } == FAIL {
            if evaluate && result {
                unsafe { tv_clear(rettv) };
            }
            used.eval_flags = orig_flags;
            return FAIL;
        }
        if evaluate && !result {
            *rv = var2;
        }
    }

    if evalarg.is_null() {
        // SAFETY: the substitute is this frame's own, and there is no `eap`.
        unsafe { clear_evalarg(&raw mut local_evalarg, null_mut()) };
    } else {
        used.eval_flags = orig_flags;
    }
    OK
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
    // SAFETY: the caller's promise -- `arg` is the cursor into the
    // expression and `rettv` is the result being built.
    let (cur, mut rv) = unsafe { (Cur::new(arg), Tv::new(rettv)) };
    if unsafe { operand(arg, rettv, evalarg) } == FAIL {
        return FAIL;
    }
    // The second byte is read only once the first matched, which is what
    // proves the cursor is not on the terminating NUL.
    let is_op = |cur: Cur| cur.byte() == op && cur.at(1) == op;
    if !is_op(cur) {
        return OK;
    }

    let mut local_evalarg = BORROWED_EVALARG;
    // SAFETY: the caller's promise -- `evalarg` is null or valid; when it is
    // null the substitute is this frame's own.
    let borrowed = if evalarg.is_null() {
        &raw mut local_evalarg
    } else {
        evalarg
    };
    let mut used = unsafe { Live::new(borrowed) };
    let orig_flags = used.eval_flags;
    let evaluate = orig_flags & EVAL_EVALUATE as c_int != 0;

    let mut result = !stop_at;
    if evaluate {
        let mut error = false;
        // SAFETY: `rettv` is the operand just parsed.
        result = unsafe { tv_get_number_chk(rettv, &raw mut error) } != 0;
        unsafe { tv_clear(rettv) };
        if error {
            return FAIL;
        }
    }

    while is_op(cur) {
        cur.skip(2);
        used.eval_flags = flags_evaluating(orig_flags, result != stop_at);
        let mut var2 = UNSET_TV;
        // SAFETY: `arg` is still the caller's cursor and `var2` this
        // frame's own.
        if unsafe { operand(arg, &raw mut var2, used.raw()) } == FAIL {
            return FAIL;
        }
        if evaluate && result != stop_at {
            let mut error = false;
            // SAFETY: `var2` is the operand just parsed.
            result = unsafe { tv_get_number_chk(&raw mut var2, &raw mut error) } != 0;
            unsafe { tv_clear(&raw mut var2) };
            if error {
                return FAIL;
            }
        }
        if evaluate {
            rv.v_type = VAR_NUMBER;
            rv.vval.v_number = varnumber_T::from(result);
        }
    }

    if evalarg.is_null() {
        // SAFETY: the substitute is this frame's own, and there is no `eap`.
        unsafe { clear_evalarg(&raw mut local_evalarg, null_mut()) };
    } else {
        used.eval_flags = orig_flags;
    }
    OK
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

/// The comparison operators.
///
/// # Safety
/// As `eval1`.
pub(crate) unsafe fn eval4(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
) -> c_int {
    // SAFETY: the caller's promise -- `arg` is the cursor into the
    // expression, and `rettv`/`evalarg` are the caller's own.
    let cur = unsafe { Cur::new(arg) };
    if unsafe { eval5(arg, rettv, evalarg) } == FAIL {
        return FAIL;
    }
    let (op, mut len) = comparison_at(cur);
    if op == EXPR_UNKNOWN {
        return OK;
    }

    // A trailing `?` or `#` overrides 'ignorecase' for this comparison.
    let ic = match cur.at(len as usize) {
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

    cur.skip(len as usize);
    let mut var2 = UNSET_TV;
    // SAFETY: as above, with `var2` this frame's own.
    if unsafe { eval5(arg, &raw mut var2, evalarg) } == FAIL {
        unsafe { tv_clear(rettv) };
        return FAIL;
    }
    if unsafe { evaluating(evalarg) } {
        // SAFETY: both operands are typvals the levels just parsed.
        let ret = unsafe { typval_compare(rettv, &raw mut var2, op, ic) };
        unsafe { tv_clear(&raw mut var2) };
        return ret;
    }
    OK
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
    // SAFETY: the caller's promise -- `arg` is the cursor into the
    // expression, `rettv` the result being built and `evalarg` null or
    // valid. All three hold for every call below.
    let (cur, rv) = unsafe { (Cur::new(arg), Tv::new(rettv)) };
    if unsafe { eval6(arg, rettv, evalarg, false) } == FAIL {
        return FAIL;
    }
    loop {
        let op = cur.byte();
        let concat = op == b'.';
        if op != b'+' && op != b'-' && !concat {
            return OK;
        }
        let evaluate = unsafe { evaluating(evalarg) };

        // Reject an operand of the wrong type before consuming the
        // operator — but not for the two cases that have their own
        // handling: `+` on a List or Blob, and anything on a Float.
        let container_plus = op == b'+' && (rv.v_type == VAR_LIST || rv.v_type == VAR_BLOB);
        let float_arith = op != b'.' && rv.v_type == VAR_FLOAT;
        if !container_plus && !float_arith && evaluate {
            let ok = if concat {
                unsafe { tv_check_str(rettv) }
            } else {
                unsafe { tv_check_num(rettv) }
            };
            if !ok {
                unsafe { tv_clear(rettv) };
                return FAIL;
            }
        }

        // `..` is two bytes, `.` one.
        cur.skip(if concat && cur.at(1) == b'.' { 2 } else { 1 });

        let mut var2 = UNSET_TV;
        if unsafe { eval6(arg, &raw mut var2, evalarg, concat) } == FAIL {
            unsafe { tv_clear(rettv) };
            return FAIL;
        }
        if evaluate {
            let (blob2, list2) = (var2.v_type == VAR_BLOB, var2.v_type == VAR_LIST);
            let two = &raw mut var2;
            let ok = if concat {
                unsafe { eval_concat_str(rettv, two) }
            } else if op == b'+' && rv.v_type == VAR_BLOB && blob2 {
                unsafe { eval_addblob(rettv, two) };
                true
            } else if op == b'+' && rv.v_type == VAR_LIST && list2 {
                unsafe { eval_addlist(rettv, two) }
            } else {
                unsafe { eval_addsub_number(rettv, two, op) }
            };
            if !ok {
                return FAIL;
            }
            unsafe { tv_clear(two) };
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
    // SAFETY: the caller's promise -- `arg` is the cursor into the
    // expression, `rettv` the result being built and `evalarg` null or
    // valid. All three hold for every call below.
    let cur = unsafe { Cur::new(arg) };
    if unsafe { eval7(arg, rettv, evalarg, want_string) } == FAIL {
        return FAIL;
    }
    loop {
        let op = cur.byte();
        if op != b'*' && op != b'/' && op != b'%' {
            return OK;
        }
        let evaluate = unsafe { evaluating(evalarg) };
        cur.skip(1);
        let mut var2 = UNSET_TV;
        if unsafe { eval7(arg, &raw mut var2, evalarg, false) } == FAIL {
            return FAIL;
        }
        if evaluate && unsafe { !eval_multdiv_number(rettv, &raw mut var2, op) } {
            return FAIL;
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

    // SAFETY: the caller's promise -- `arg` is the cursor into the
    // expression, `rettv` the result being built and `evalarg` null or
    // valid. All three hold for every call below.
    let (cur, mut rv) = unsafe { (Cur::new(arg), Tv::new(rettv)) };
    let evaluate = unsafe { evaluating(evalarg) };
    let mut ret = OK;
    rv.v_type = VAR_UNKNOWN;

    // The prefixes are collected now and applied last, so that `-1` is
    // parsed as a negated literal but `!x[0]` negates the subscript.
    let start_leader: *const c_char = cur.get();
    while matches!(cur.byte(), b'!' | b'-' | b'+') {
        cur.skip(1);
    }
    let mut end_leader: *const c_char = cur.get();

    if RECURSE.get() == MAX_RECURSE {
        let at = cur.get();
        unsafe { semsg_c!(gettext(e_expression_too_recursive_str.as_ptr()), at) };
        return FAIL;
    }
    RECURSE.set(RECURSE.get() + 1);

    match cur.byte() {
        b'0'..=b'9' => {
            ret = unsafe { eval_number(arg, rettv, evaluate, want_string) };
            // A number applies its prefixes here, where `-` still means
            // arithmetic negation rather than "negate what follows".
            if ret == OK && evaluate && end_leader > start_leader {
                let endp = &raw mut end_leader;
                ret = unsafe { eval7_leader(rettv, true, start_leader, endp) };
            }
        }
        b'"' => ret = unsafe { eval_string(arg, rettv, evaluate, false) },
        b'\'' => ret = unsafe { eval_lit_string(arg, rettv, evaluate, false) },
        b'[' => ret = unsafe { eval_list(arg, rettv, evalarg) },
        b'#' => ret = unsafe { eval_lit_dict(arg, rettv, evalarg) },
        b'{' => {
            // A `{` is a lambda if it parses as one and a Dict if not.
            ret = unsafe { get_lambda_tv(arg, rettv, evalarg) };
            if ret == NOTDONE {
                ret = unsafe { eval_dict(arg, rettv, evalarg, false) };
            }
        }
        b'&' => ret = unsafe { eval_option(arg as *mut *const c_char, rettv, evaluate) },
        b'$' => {
            ret = if matches!(cur.at(1), b'"' | b'\'') {
                unsafe { eval_interp_string(arg, rettv, evaluate) }
            } else {
                unsafe { eval_env_var(arg, rettv, evaluate) }
            };
        }
        b'@' => {
            cur.bump(1);
            if evaluate {
                rv.v_type = VAR_STRING;
                // Sign-extended, as the C is: `**arg` is a `char`.
                let name = cur.byte() as c_char as c_int;
                // SAFETY: `get_reg_contents` reads only the register name.
                let text = unsafe { get_reg_contents(name, kGRegExprSrc as c_int) };
                rv.vval.v_string = text as *mut c_char;
            }
            // `@` at the very end of the line names no register.
            if cur.byte() != NUL as u8 {
                cur.bump(1);
            }
        }
        b'(' => {
            cur.skip(1);
            ret = unsafe { eval1(arg, rettv, evalarg) };
            if cur.byte() == b')' {
                cur.bump(1);
            } else if ret == OK {
                unsafe { emsg(gettext(c"E110: Missing ')'".as_ptr())) };
                unsafe { tv_clear(rettv) };
                ret = FAIL;
            }
        }
        _ => ret = NOTDONE,
    }

    if ret == NOTDONE {
        // Not a literal: it must be a name, and then either a call or a
        // variable.
        let mut alias: *mut c_char = null_mut();
        let start = cur.get();
        let aliasp = &raw mut alias;
        let len = unsafe { get_name_len(cur.raw().cast(), aliasp, evaluate, true) };
        let name = if alias.is_null() { start } else { alias };
        if len <= 0 {
            ret = FAIL;
        } else {
            // SAFETY: `evalarg` is null or valid.
            let flags = if evalarg.is_null() {
                0
            } else {
                unsafe { (*evalarg).eval_flags }
            };
            // A name may be followed by white space and then its arguments.
            let call = unsafe { *skipwhite(cur.get()) } == b'(' as c_char;
            if call {
                cur.skip(0);
                ret = unsafe { eval_func(arg, evalarg, name, len, rettv, flags, null_mut()) };
            } else if evaluate {
                let none = null_mut::<*mut dictitem_T>();
                ret = unsafe { eval_variable(name, len, rettv, none, true, false) };
            } else {
                unsafe { check_vars(name, len as size_t) };
                // While skipping, `v:lua.x` still has to come out as
                // something callable.
                let lua = unsafe { strnequal(name, c"v:lua.".as_ptr(), 6) };
                if rv.v_type == VAR_UNKNOWN && lua {
                    rv.v_type = VAR_PARTIAL;
                    let partial = unsafe { get_vim_var_partial(Vv::Lua) };
                    rv.vval.v_partial = partial;
                    // SAFETY: `get_vim_var_partial` answers a live partial.
                    unsafe { (*partial).pt_refcount.retain() };
                }
                ret = OK;
            }
        }
        // SAFETY: `alias` is null or the buffer `get_name_len` allocated.
        unsafe { xfree(alias.cast()) };
    }

    cur.skip(0);
    if ret == OK {
        ret = unsafe { handle_subscript(cur.raw().cast(), rettv, evalarg, true) };
    }
    if ret == OK && evaluate && end_leader > start_leader {
        let endp = &raw mut end_leader;
        ret = unsafe { eval7_leader(rettv, false, start_leader, endp) };
    }
    RECURSE.set(RECURSE.get() - 1);
    ret
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
    // SAFETY: the caller's promise -- `rettv` is the operand just parsed,
    // and `end_leaderp` is the caller's own cursor over the prefixes.
    let mut rv = unsafe { Tv::new(rettv) };
    let mut end_leader = unsafe { *end_leaderp };
    let mut ret = OK;
    let mut error = false;
    let mut val: varnumber_T = 0;
    let mut f: float_T = 0.0;

    if rv.v_type == VAR_FLOAT {
        // SAFETY: the tag says the union holds a Float.
        f = unsafe { rv.vval.v_float };
    } else {
        val = unsafe { tv_get_number_chk(rettv, &raw mut error) };
    }

    if error {
        unsafe { tv_clear(rettv) };
        ret = FAIL;
    } else {
        while end_leader > start_leader {
            end_leader = end_leader.wrapping_sub(1);
            // SAFETY: `end_leader` is inside the run of prefixes the caller
            // bounded, which it has not yet walked past.
            match unsafe { *end_leader } as u8 {
                b'!' => {
                    if numeric_only {
                        end_leader = end_leader.wrapping_add(1);
                        break;
                    }
                    if rv.v_type == VAR_FLOAT {
                        // Negating a Float leaves the value in `val` and
                        // the tag saying so, which is what makes a second
                        // `!` see a Number. The tag is overwritten below,
                        // so `!1.5` still answers a Number.
                        rv.v_type = VAR_BOOL;
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
                    if rv.v_type == VAR_FLOAT {
                        f = -f;
                    } else {
                        val = val.wrapping_neg();
                    }
                }
                // A `+` prefix does nothing at all.
                _ => {}
            }
        }
        let float = rv.v_type == VAR_FLOAT;
        unsafe { tv_clear(rettv) };
        if float {
            rv.vval.v_float = f;
        } else {
            rv.v_type = VAR_NUMBER;
            rv.vval.v_number = val;
        }
    }

    unsafe { *end_leaderp = end_leader };
    ret
}
