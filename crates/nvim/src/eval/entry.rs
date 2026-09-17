//! What the rest of the editor calls the evaluator through.
//!
//! Every entry point here brackets one evaluation: it sets up an
//! `EvalArg`, runs `eval0`, converts the result to whatever the caller
//! wanted, and clears the typval on both the success and the error path.
//! The bracket is the whole content of the file — twenty-four times, with
//! the differences in what is counted up around it (`emsg_skip`,
//! `emsg_off`, `sandbox`, `textlock`, the funccal stack) and what the
//! answer is converted to.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::eval::Parsed;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::guard::{Lock, Suppress};
use crate::message_fmt::c_str;
use crate::semsg;
use crate::winlayer::Win;
use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of;
use core::ptr::null_mut;

use crate::api::private::helpers::cstr_to_string;
use crate::ascii::ascii_isdigit;
use crate::charset::skipwhite;
use crate::eval::EVALARG_EVALUATE;
use crate::eval::encode::encode_tv2string;
use crate::eval::typval::{
    NumBuf, list_join, list_last, list_len, list_set_lock, tv_clear, tv_dict_free_contents,
    tv_get_number_chk, tv_list_alloc,
};
use crate::eval::userfunc::{call_func, func_init, restore_funccal, save_funccal};
use crate::eval::vars::clear_local;
use crate::eval::vars::{evalvars_init, get_vim_var_dict, get_vim_var_partial, set_vim_var_list};
use crate::eval::{
    EVAL_EVALUATE, FUNCEXE_INIT, NL, Tv, check_luafunc_name, clear_evalarg, eval0,
    eval0_simple_funccal, eval1, may_call_simple_func, partial_name,
};
use crate::ex_eval::aborting;
use crate::garray::{ga_append, ga_init};
use crate::memory::{xfree, xmalloc, xstrdup};
use crate::message::state::{called_emsg, did_emsg};
use crate::option::was_set_insecurely;
use crate::options::{kOptFoldexpr, kOptFoldtext, kWinOptFoldexpr};
use crate::optionstr::LocalOptStr;
use crate::runtime::sourcing_a_script;
use crate::runtime::state::current_sctx;
use crate::types::{
    Dict, EvalArg, ExArg, Failed, FuncCallEntry, FuncExe, GArray, HashTab, NUL, Object,
    OptionSetFlags, Partial, SaveVEvent, ScriptCtx, String_0, TypVal, VAR_DICT, VAR_FUNC, VAR_LIST,
    VAR_NUMBER, VAR_PARTIAL, VAR_STRING, VAR_UNKNOWN, VarLock, VarNumber, Vv, ptrdiff_t, size_t,
    ssize_t, uint8_t,
};
use crate::winlayer::Live;
use ::libc::atol;

/// A freshly declared typval.
const UNSET_TV: TypVal = TV_INITIAL_VALUE;

/// A freshly declared `EvalArg`, before `fill_evalarg_from_eap`.
const UNSET_EVALARG: EvalArg = EvalArg {
    eval_flags: 0,
    eval_getline: None,
    eval_cookie: null_mut(),
    eval_tofree: null_mut(),
    next_cmd: None,
};

/// One expression's evaluation state, owned by the frame that declared it.
type Ev = Live<EvalArg>;

/// An empty growable array.
const UNSET_GA: GArray = GArray {
    ga_len: 0,
    ga_maxlen: 0,
    ga_itemsize: 0,
    ga_growsize: 0,
    ga_data: null_mut(),
};

/// Reserve `v:event` for the duration of one autocommand, saving whatever
/// a surrounding one had put there.
///
/// # Safety
/// `sve` must be valid.
pub unsafe fn get_v_event(sve: *mut SaveVEvent) -> *mut Dict {
    let v_event = get_vim_var_dict(Vv::Event);
    // SAFETY: the caller's promise about `sve`, and `v_event` as above.
    let (saved, live) = unsafe { (&mut (*sve).sve_hashtab, &mut (*v_event).dv_hashtab) };
    let did_save = live.ht_used > 0 as size_t;
    // SAFETY: the caller's promise about `sve`.
    unsafe { (*sve).sve_did_save = did_save };
    if did_save {
        // A plain move: the table owns its slots, so what the surrounding
        // autocommand put in `v:event` travels to `sve` intact and
        // `v:event` starts the inner one empty. `restore_v_event` moves it
        // back.
        *saved = core::mem::replace(live, HashTab::init());
    }
    v_event
}

/// Put back what `get_v_event` saved.
///
/// # Safety
/// `v_event` and `sve` must be a pair `get_v_event` produced.
pub unsafe fn restore_v_event(v_event: *mut Dict, sve: *mut SaveVEvent) {
    // SAFETY: the caller's promise -- the pair `get_v_event` produced.
    unsafe { tv_dict_free_contents(v_event) };
    // `tv_dict_free_contents` already left `v:event` with a fresh empty
    // table, so the not-saved case has nothing left to do.
    // SAFETY: as above.
    if unsafe { (*sve).sve_did_save } {
        // SAFETY: as above.
        let (saved, live) = unsafe { (&mut (*sve).sve_hashtab, &mut (*v_event).dv_hashtab) };
        // The move back. `sve` is left with a table that owns nothing,
        // which is what its `Default` is.
        *live = core::mem::take(saved);
    }
}

/// Bring up the evaluator: the `v:` variables and the function table.
pub fn eval_init() {
    evalvars_init();
    func_init();
}

/// Set up an `EvalArg` for an expression that is part of an Ex command.
///
/// The line-getter is carried over only while sourcing a script, which is
/// what lets an expression there run onto a following line.
///
/// # Safety
/// `evalarg` must be valid.
pub unsafe fn fill_evalarg_from_eap(evalarg: *mut EvalArg, excmd: Option<&mut ExArg>, skip: bool) {
    // SAFETY: the caller's promise -- `evalarg` outlives the call.
    let mut evalarg = unsafe { Ev::new(evalarg) };
    *evalarg = UNSET_EVALARG;
    evalarg.eval_flags = if skip { 0 } else { EVAL_EVALUATE as c_int };
    let Some(command) = excmd else {
        return;
    };
    if sourcing_a_script(command) != 0 {
        evalarg.eval_getline = command.ea_getline;
        evalarg.eval_cookie = command.cookie;
    }
}

/// Evaluate `arg` and answer its truth. `error` says whether the
/// evaluation itself failed, which is not the same as answering false.
///
/// # Safety
/// `arg` must be a NUL-terminated expression, `error` valid, `excmd` null or
/// valid.
pub unsafe fn eval_to_bool(
    arg: *mut c_char,
    error: *mut bool,
    mut excmd: Option<&mut ExArg>,
    skip: bool,
    use_simple_function: bool,
) -> bool {
    let mut tv = UNSET_TV;
    let mut retval = false;
    let mut evalarg = UNSET_EVALARG;
    unsafe { fill_evalarg_from_eap(&raw mut evalarg, excmd.as_deref_mut(), skip) };
    let skipping = skip.then(Suppress::emsg_skip);
    let r = if use_simple_function {
        unsafe { eval0_simple_funccal(arg, &mut tv, excmd.as_deref_mut(), &raw mut evalarg) }
    } else {
        unsafe { eval0(arg, &mut tv, excmd.as_deref_mut(), &raw mut evalarg) }
    };
    if r.is_err() {
        unsafe { *error = true };
    } else {
        unsafe { *error = false };
        if !skip {
            retval = match tv_get_number_chk(&tv) {
                Ok(n) => n != 0,
                Err(_) => {
                    // SAFETY: the caller's flag, as above.
                    unsafe { *error = true };
                    false
                }
            };
            clear_local(&mut tv);
        }
    }
    drop(skipping);
    unsafe { clear_evalarg(&raw mut evalarg, excmd) };
    retval
}

/// `eval1` with a fallback message: when the expression failed silently —
/// nothing aborted and nothing reported — say which expression it was.
///
/// # Safety
/// `arg` must point at the cursor into a NUL-terminated expression;
/// `result` valid; `excmd` null or valid.
pub(crate) unsafe fn eval1_emsg(
    arg: *mut *mut c_char,
    result: &mut TypVal,
    mut excmd: Option<&mut ExArg>,
) -> Result<(), Failed> {
    let start: *const c_char = unsafe { *arg };
    let did_emsg_before = did_emsg.get();
    let called_emsg_before = called_emsg.get();

    let mut evalarg = UNSET_EVALARG;
    let skip = excmd.as_deref().is_some_and(|command| command.skip);
    unsafe { fill_evalarg_from_eap(&raw mut evalarg, excmd.as_deref_mut(), skip) };
    let ret = unsafe { eval1(arg, result, &raw mut evalarg) };
    if ret.is_err()
        && !aborting()
        && did_emsg.get() == did_emsg_before
        && called_emsg.get() == called_emsg_before
    {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let start = unsafe { c_str(start) };
        semsg!("E15: Invalid expression: \"{start}\"");
    }
    unsafe { clear_evalarg(&raw mut evalarg, excmd) };
    ret
}

/// Is this typval usable as an expression argument at all? An unset value
/// and an empty String are not.
pub fn eval_expr_valid_arg(tv: &TypVal) -> bool {
    // SAFETY: the caller's promise -- the typval outlives the call, and it
    // is only read through here.
    let tv = unsafe { Tv::new(::core::ptr::from_ref(tv).cast_mut()) };
    if tv.v_type() == VAR_UNKNOWN {
        return false;
    }
    if tv.v_type() != VAR_STRING {
        return true;
    }
    // SAFETY: `VAR_STRING` says the value holds a string, and
    // a non-null one is NUL-terminated.
    let s = tv.string_or_null();
    !s.is_null() && unsafe { *s } as c_int != NUL
}

/// Call the partial in `expr`.
pub(crate) fn eval_expr_partial(
    expr: &TypVal,
    argv: &[TypVal],
    result: &mut TypVal,
) -> Result<(), Failed> {
    // SAFETY: the caller's promise -- a `VAR_PARTIAL`, so the value holds a
    // live partial or NULL.
    let partial = (*expr).partial_or_null();
    if partial.is_null() {
        return Err(Failed);
    }
    let s: *const c_char = unsafe { partial_name(partial) };
    if s.is_null() || unsafe { *s } as c_int == NUL {
        return Err(Failed);
    }
    let mut funcexe: FuncExe = FUNCEXE_INIT;
    funcexe.fe_evaluate = true;
    funcexe.fe_partial = partial;
    unsafe { call_func(s, -1, result, argv, &raw mut funcexe) }?;
    Ok(())
}

/// Call the function `expr` names.
pub(crate) fn eval_expr_func(
    expr: &TypVal,
    argv: &[TypVal],
    result: &mut TypVal,
) -> Result<(), Failed> {
    let mut buf = NumBuf::new();
    // SAFETY: the caller's promise -- `expr` outlives the call, and it is
    // only read through here; `VAR_FUNC` says `v_string` is its live
    // member, and `buf` outlives the string rendered into it.
    let expr_tv = expr;
    let s: *const c_char = if expr_tv.v_type() == VAR_FUNC {
        expr_tv.func_name_or_null() as *const c_char
    } else {
        buf.string_ptr_chk(expr)
    };
    if s.is_null() || unsafe { *s } as c_int == NUL {
        return Err(Failed);
    }
    let mut funcexe: FuncExe = FUNCEXE_INIT;
    funcexe.fe_evaluate = true;
    unsafe { call_func(s, -1, result, argv, &raw mut funcexe) }?;
    Ok(())
}

/// Evaluate `expr` as an expression *string*, which must consume all of it.
pub(crate) fn eval_expr_string(expr: &TypVal, result: &mut TypVal) -> Result<(), Failed> {
    let mut buf = NumBuf::new();
    let mut s = buf.string_ptr_chk(expr) as *mut c_char;
    if s.is_null() {
        return Err(Failed);
    }
    s = unsafe { skipwhite(s) };
    unsafe { eval1_emsg(&raw mut s, result, None) }?;
    if unsafe { *skipwhite(s) } as c_int != NUL {
        tv_clear(result);
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let s = unsafe { c_str(s) };
        semsg!("E15: Invalid expression: \"{s}\"");
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
///
/// # Safety
/// `expr` and `error` must be valid.
pub unsafe fn eval_expr_to_bool(expr: &TypVal, error: *mut bool) -> bool {
    let mut rettv = UNSET_TV;
    if eval_expr_typval(expr, false, &[], &mut rettv).is_err() {
        unsafe { *error = true };
        return false;
    }
    let res = match tv_get_number_chk(&rettv) {
        Ok(n) => n != 0,
        Err(_) => {
            // SAFETY: the caller's flag, as above.
            unsafe { *error = true };
            false
        }
    };
    clear_local(&mut rettv);
    res
}

/// Evaluate `arg` for its String, or only parse it when `skip`.
///
/// # Safety
/// As `eval_to_bool`.
pub unsafe fn eval_to_string_skip(arg: *mut c_char, excmd: &mut ExArg, skip: bool) -> *mut c_char {
    let mut numbuf = NumBuf::new();
    let mut tv = UNSET_TV;
    let mut evalarg = UNSET_EVALARG;
    unsafe { fill_evalarg_from_eap(&raw mut evalarg, Some(excmd), skip) };
    let skipping = skip.then(Suppress::emsg_skip);
    let retval = if unsafe { eval0(arg, &mut tv, Some(excmd), &raw mut evalarg) }.is_err() || skip {
        null_mut()
    } else {
        let s = unsafe { xstrdup(numbuf.string_ptr(&tv)) };
        clear_local(&mut tv);
        s
    };
    drop(skipping);
    unsafe { clear_evalarg(&raw mut evalarg, Some(excmd)) };
    retval
}

/// Step the cursor over an expression without evaluating it.
///
/// # Safety
/// `cursor` must point at the cursor into a NUL-terminated expression;
/// `evalarg` null or valid.
pub unsafe fn skip_expr(cursor: *mut *mut c_char, evalarg: *mut EvalArg) -> Result<(), Failed> {
    // SAFETY: the caller's promise -- a non-null `evalarg` outlives the
    // call.
    let ev = (!evalarg.is_null()).then(|| unsafe { Ev::new(evalarg) });
    let save_flags = ev.map_or(0, |e| e.eval_flags);
    if let Some(mut e) = ev {
        e.eval_flags &= !(EVAL_EVALUATE as c_int);
    }
    // SAFETY: the caller's promise -- `cursor` holds a cursor into a
    // NUL-terminated expression, and blanks stop at the terminator.
    unsafe { *cursor = skipwhite(*cursor) };
    let mut rettv = UNSET_TV;
    // Deliberately not handed `evalarg`: the flags were cleared on it
    // for the benefit of anything else looking, but this walk wants no
    // line getter either.
    // SAFETY: `cursor` is the caller's cursor and `rettv` is this frame's.
    let res = unsafe { eval1(cursor, &mut rettv, null_mut()) };
    if let Some(mut e) = ev {
        e.eval_flags = save_flags;
    }
    res
}

/// Render a typval as the String a caller of the evaluator expects: a List
/// joined with newlines when `join_list`, otherwise the `string()` form for
/// a container and the plain coercion for everything else.
///
/// # Safety
/// `tv` must be valid.
pub(crate) unsafe fn typval2string(tv: &mut TypVal, join_list: bool) -> *mut c_char {
    let mut numbuf = NumBuf::new();
    // SAFETY: the caller's promise -- the typval outlives the call, and
    // `VAR_LIST` says the value holds a List.
    let value = unsafe { Tv::new(tv) };
    if join_list && value.v_type() == VAR_LIST {
        let mut ga = UNSET_GA;
        // SAFETY: `ga` is this frame's.
        unsafe { ga_init(&raw mut ga, size_of::<c_char>() as c_int, 80) };
        let l = value.list_or_null();
        if !l.is_null() {
            // SAFETY: `l` is the typval's live List.
            let _ = unsafe { list_join(&raw mut ga, l.as_ref(), c"\n".as_ptr()) };
            // SAFETY: as above.
            if list_len(unsafe { l.as_ref() }) > 0 {
                // SAFETY: `ga` is this frame's.
                unsafe { ga_append(&raw mut ga, NL as uint8_t) };
            }
        }
        // SAFETY: `ga` is this frame's.
        unsafe { ga_append(&raw mut ga, NUL as uint8_t) };
        return ga.ga_data as *mut c_char;
    }
    if value.v_type() == VAR_LIST || value.v_type() == VAR_DICT {
        // SAFETY: the caller's typval.
        return unsafe { encode_tv2string(tv, null_mut()) };
    }
    // SAFETY: as above; `numbuf` outlives the string rendered into it.
    unsafe { xstrdup(numbuf.string_ptr(tv)) }
}

/// Evaluate `arg` for its String.
///
/// # Safety
/// As `eval_to_bool`.
pub unsafe fn eval_to_string_eap(
    arg: *mut c_char,
    join_list: bool,
    excmd: Option<&mut ExArg>,
    use_simple_function: bool,
) -> *mut c_char {
    let mut tv = UNSET_TV;
    let mut evalarg = UNSET_EVALARG;
    let skip = excmd.as_deref().is_some_and(|command| command.skip);
    unsafe { fill_evalarg_from_eap(&raw mut evalarg, excmd, skip) };
    // The `excmd` is read for the line getter above but deliberately not
    // handed on: this evaluation is not the Ex command's own.
    let r = if use_simple_function {
        unsafe { eval0_simple_funccal(arg, &mut tv, None, &raw mut evalarg) }
    } else {
        unsafe { eval0(arg, &mut tv, None, &raw mut evalarg) }
    };
    let retval = if r.is_err() {
        null_mut()
    } else {
        let s = unsafe { typval2string(&mut tv, join_list) };
        clear_local(&mut tv);
        s
    };
    unsafe { clear_evalarg(&raw mut evalarg, None) };
    retval
}

/// `eval_to_string_eap` with no Ex command around it.
///
/// # Safety
/// `arg` must be a NUL-terminated expression.
pub unsafe fn eval_to_string(
    arg: *mut c_char,
    join_list: bool,
    use_simple_function: bool,
) -> *mut c_char {
    unsafe { eval_to_string_eap(arg, join_list, None, use_simple_function) }
}

/// `eval_to_string` with the text locked and, optionally, the sandbox on,
/// and with the function-call stack saved across it.
///
/// # Safety
/// `arg` must be a NUL-terminated expression.
pub unsafe fn eval_to_string_safe(
    arg: *mut c_char,
    use_sandbox: bool,
    use_simple_function: bool,
) -> *mut c_char {
    let mut funccal_entry = FuncCallEntry {
        top_funccal: null_mut(),
        next: null_mut(),
    };
    unsafe { save_funccal(&raw mut funccal_entry) };
    let _sandboxed = use_sandbox.then(Lock::sandbox);
    let _locked = Lock::text();
    let retval = unsafe { eval_to_string(arg, false, use_simple_function) };
    unsafe { restore_funccal() };
    retval
}

/// Evaluate `expr` for its Number, silently. -1 for a failure, which is
/// not distinguishable from a result of -1.
///
/// # Safety
/// `expr` must be a NUL-terminated expression.
pub unsafe fn eval_to_number(expr: *mut c_char, use_simple_function: bool) -> VarNumber {
    let mut evalarg = EVALARG_EVALUATE;
    let mut rettv = UNSET_TV;
    let mut p = unsafe { skipwhite(expr) };
    let _no_emsg = Suppress::emsg();
    // Note the shortcut is handed the *unskipped* expression, unlike `eval1`.
    let simple = if use_simple_function {
        unsafe { may_call_simple_func(expr, &mut rettv) }
    } else {
        Ok(Parsed::NotThis)
    };
    let r = match simple {
        Ok(Parsed::NotThis) => unsafe { eval1(&raw mut p, &mut rettv, &raw mut evalarg) },
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

/// Evaluate `arg` into a heap typval the caller owns; null on failure.
///
/// # Safety
/// `arg` must be a NUL-terminated expression; `excmd` null or valid.
pub unsafe fn eval_expr(arg: *mut c_char, excmd: Option<&mut ExArg>) -> *mut TypVal {
    unsafe { eval_expr_ext(arg, excmd, false) }
}

/// As `eval_expr`, optionally taking the shortcut for an expression that is
/// nothing but one function call.
///
/// # Safety
/// As `eval_expr`.
pub unsafe fn eval_expr_ext(
    arg: *mut c_char,
    mut excmd: Option<&mut ExArg>,
    use_simple_function: bool,
) -> *mut TypVal {
    let mut tv = unsafe { xmalloc(size_of::<TypVal>()) } as *mut TypVal;
    let mut evalarg = UNSET_EVALARG;
    let skip = excmd.as_deref().is_some_and(|command| command.skip);
    unsafe { fill_evalarg_from_eap(&raw mut evalarg, excmd.as_deref_mut(), skip) };
    // `eval0_simple_funccal` falls through to `eval0` itself, so the two
    // arms are the whole of the choice: nothing here can be left undone.
    // The allocation holds no value yet, and the evaluator is handed a
    // borrow of it: give it one before anything reads the bytes.
    // SAFETY: the allocation just made.
    let slot = unsafe { (tv.write(TV_INITIAL_VALUE), &mut *tv) }.1;
    let r = if use_simple_function {
        unsafe { eval0_simple_funccal(arg, slot, excmd.as_deref_mut(), &raw mut evalarg) }
    } else {
        unsafe { eval0(arg, slot, excmd.as_deref_mut(), &raw mut evalarg) }
    };
    if r.is_err() {
        unsafe { xfree(tv as *mut c_void) };
        tv = null_mut();
    }
    unsafe { clear_evalarg(&raw mut evalarg, excmd) };
    tv
}

/// Call a Vimscript function by name with `argv` as its arguments.
///
/// # Safety
/// `func` must be NUL-terminated and `result` must be valid.
pub unsafe fn call_vim_function(
    func: *const c_char,
    argv: &[TypVal],
    result: &mut TypVal,
) -> Result<(), Failed> {
    let mut func = func;
    let mut len = unsafe { cstr::bytes_at(func) }.len() as c_int;
    let mut pt: *mut Partial = null_mut();
    let mut ret = Err(Failed);

    'fail: {
        // SAFETY: `len >= 6` promises six readable bytes.
        if len >= 6 && unsafe { cstr::starts_with(func, b"v:lua.") } {
            // SAFETY: the six bytes just compared are behind us, so what is
            // left is still inside the NUL-terminated name.
            func = unsafe { func.add(6) };
            // SAFETY: as above.
            len = unsafe { check_luafunc_name(func, false) };
            if len == 0 {
                break 'fail;
            }
            pt = get_vim_var_partial(Vv::Lua);
        }
        // SAFETY: the caller's promise about `result`.
        let rv = &mut *result;
        rv.write_empty(VAR_UNKNOWN);
        let mut funcexe: FuncExe = FUNCEXE_INIT;
        funcexe.fe_firstline = Win::current().w_cursor.lnum;
        funcexe.fe_lastline = Win::current().w_cursor.lnum;
        funcexe.fe_evaluate = true;
        funcexe.fe_partial = pt;
        ret = unsafe { call_func(func, len, rv, argv, &raw mut funcexe) };
    }

    if ret.is_err() {
        tv_clear(result);
    }
    ret
}

/// `call_vim_function`, answering an owned copy of the result's String.
///
/// # Safety
/// As `call_vim_function`.
pub unsafe fn call_func_retstr(func: *const c_char, argv: &[TypVal]) -> *mut c_void {
    let mut numbuf = NumBuf::new();
    let mut rettv = UNSET_TV;
    if unsafe { call_vim_function(func, argv, &mut rettv) }.is_err() {
        return null_mut();
    }
    let retval = unsafe { xstrdup(numbuf.string_ptr(&rettv)) };
    clear_local(&mut rettv);
    retval as *mut c_void
}

/// `call_vim_function`, answering the result's List — which the caller then
/// owns the reference to. Null for anything else.
///
/// # Safety
/// As `call_vim_function`.
pub unsafe fn call_func_retlist(func: *const c_char, argv: &[TypVal]) -> *mut c_void {
    let mut rettv = UNSET_TV;
    if unsafe { call_vim_function(func, argv, &mut rettv) }.is_err() {
        return null_mut();
    }
    if rettv.v_type() != VAR_LIST {
        clear_local(&mut rettv);
        return null_mut();
    }
    // The reference goes to the caller, which unreferences the list itself.
    let list = rettv.list_or_null();
    rettv.disown();
    list as *mut c_void
}

/// Run 'foldexpr' for the window's current line. `marker` comes back holding
/// the leading marker character (`>`, `<`, `=`, `a`, `s`) when there was
/// one.
///
/// # Safety
/// `window` and `marker` must be valid.
pub unsafe fn eval_foldexpr(window: Win, marker: *mut c_int) -> c_int {
    let mut evalarg = EVALARG_EVALUATE;
    let saved_sctx: ScriptCtx = current_sctx.get();
    let use_sandbox = was_set_insecurely(window, kOptFoldexpr, OptionSetFlags::LOCAL);
    // SAFETY: an option string is NUL-terminated.
    let arg = unsafe { skipwhite(window.w_onebuf_opt.wo_fde.value_ptr()) };
    current_sctx.set(window.w_onebuf_opt.wo_script_ctx[kWinOptFoldexpr as usize]);
    let retval: VarNumber = {
        let _no_emsg = Suppress::emsg();
        let _sandboxed = use_sandbox.then(Lock::sandbox);
        let _locked = Lock::text();
        // SAFETY: the caller's promise about `marker`.
        unsafe { *marker = NUL };

        let mut tv = UNSET_TV;
        let mut retval: VarNumber = 0;
        if unsafe { eval0_simple_funccal(arg, &mut tv, None, &raw mut evalarg) }.is_ok() {
            if tv.v_type() == VAR_NUMBER {
                retval = tv.number_or_zero();
            } else if tv.v_type() != VAR_STRING || tv.string_or_null().is_null() {
                retval = 0;
            } else {
                // SAFETY: `VAR_STRING` says `v_string` is the live member,
                // and a non-null one is NUL-terminated.
                let mut s = tv.string_or_null();
                let first = unsafe { *s };
                // A leading non-digit that is not a minus sign is the
                // fold marker; the rest is the level.
                if first as c_int != NUL
                    && !ascii_isdigit(first as c_int)
                    && first != b'-' as c_char
                {
                    // SAFETY: the caller's promise about `marker`; `first` is
                    // not the terminator, so the rest is inside the string.
                    unsafe { *marker = first as u8 as c_int };
                    s = unsafe { s.add(1) };
                }
                // SAFETY: `s` is inside the NUL-terminated string.
                retval = unsafe { atol(s) } as VarNumber;
            }
            clear_local(&mut tv);
        }
        retval
    };
    unsafe { clear_evalarg(&raw mut evalarg, None) };
    current_sctx.set(saved_sctx);
    retval as c_int
}

/// Run 'foldtext' for the window's current fold. A List comes back as an
/// Object so the caller can keep its per-chunk highlighting; anything else
/// is coerced to a String.
pub fn eval_foldtext(window: Win) -> Object {
    let mut evalarg = EVALARG_EVALUATE;
    let mut numbuf = NumBuf::new();
    /// The empty String an error answers with.
    fn empty_string() -> Object {
        Object::string(String_0::NULL)
    }

    let use_sandbox = was_set_insecurely(window, kOptFoldtext, OptionSetFlags::LOCAL);
    let arg = window.w_onebuf_opt.wo_fdt.value_ptr();
    let mut funccal_entry = FuncCallEntry {
        top_funccal: null_mut(),
        next: null_mut(),
    };
    unsafe { save_funccal(&raw mut funccal_entry) };
    let _sandboxed = use_sandbox.then(Lock::sandbox);
    let _locked = Lock::text();

    let mut tv = UNSET_TV;
    let retval = if unsafe { eval0_simple_funccal(arg, &mut tv, None, &raw mut evalarg) }.is_err() {
        empty_string()
    } else {
        let obj = if tv.v_type() == VAR_LIST {
            Object::from(&tv)
        } else {
            // SAFETY: `numbuf` holds the rendering, NUL-terminated.
            Object::string(unsafe { cstr_to_string(numbuf.string_ptr(&tv)) })
        };
        clear_local(&mut tv);
        obj
    };

    unsafe { clear_evalarg(&raw mut evalarg, None) };
    unsafe { restore_funccal() };
    retval
}

/// Fill `v:argv` from the process arguments. Every item is locked.
///
/// # Safety
/// `argv` must hold `argc` NUL-terminated strings.
pub unsafe fn set_argv_var(argv: *mut *mut c_char, argc: c_int) {
    let list = tv_list_alloc(argc as ptrdiff_t);
    let l = list.as_ptr();
    // SAFETY: `l` is that List.
    list_set_lock(unsafe { l.as_mut() }, VarLock::Fixed);
    for i in 0..argc {
        // SAFETY: the caller's promise -- `argc` NUL-terminated strings,
        // so slot `i` is one of them; -1 asks the callee to measure it.
        let arg = unsafe { *argv.offset(i as isize) } as *const c_char;
        // SAFETY: as above.
        unsafe { (*l).push_string(arg, -1 as ssize_t) };
        // SAFETY: the item just appended is the List's last.
        unsafe { (*list_last(l.as_mut())).li_lock = VarLock::Fixed };
    }
    set_vim_var_list(Vv::Argv, Some(list));
}

/// Render a typval for display, as `:echo` would. A null typval is the
/// "no such variable" text, which is what the debugger prints.
///
/// # Safety
/// `arg` must be null or valid.
pub unsafe fn typval_tostring(arg: Option<&TypVal>, quotes: bool) -> *mut c_char {
    let Some(value) = arg else {
        // SAFETY: the text is a NUL-terminated literal.
        return unsafe { xstrdup(c"(does not exist)".as_ptr()) };
    };
    if !quotes && value.v_type() == VAR_STRING {
        let s = value.string_or_null();
        let s = if s.is_null() {
            c"".as_ptr()
        } else {
            s as *const c_char
        };
        // SAFETY: `s` is NUL-terminated either way.
        return unsafe { xstrdup(s) };
    }
    // SAFETY: the caller's typval.
    unsafe { encode_tv2string(value, null_mut()) }
}
