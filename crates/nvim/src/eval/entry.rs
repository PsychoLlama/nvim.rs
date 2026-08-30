//! What the rest of the editor calls the evaluator through.
//!
//! Every entry point here brackets one evaluation: it sets up an
//! `evalarg_T`, runs `eval0`, converts the result to whatever the caller
//! wanted, and clears the typval on both the success and the error path.
//! The bracket is the whole content of the file — twenty-four times, with
//! the differences in what is counted up around it (`emsg_skip`,
//! `emsg_off`, `sandbox`, `textlock`, the funccal stack) and what the
//! answer is converted to.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::eval::Parsed;
use crate::guard::{Lock, Suppress};
use crate::message_fmt::c_str;
use crate::semsg;
use crate::winlayer::Win;
use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of;
use core::ptr::null_mut;

use crate::api::private::converter::vim_to_object;
use crate::api::private::helpers::cstr_to_string;
use crate::ascii::ascii_isdigit;
use crate::charset::skipwhite;
use crate::eval::EVALARG_EVALUATE;
use crate::eval::encode::encode_tv2string;
use crate::eval::typval::{
    NumBuf, tv_clear, tv_dict_free_contents, tv_get_number_chk, tv_get_string_buf_chk,
    tv_list_alloc, tv_list_append_string, tv_list_join, tv_list_last, tv_list_len,
    tv_list_set_lock,
};
use crate::eval::userfunc::{call_func, func_init, restore_funccal, save_funccal};
use crate::eval::vars::clear_local;
use crate::eval::vars::{evalvars_init, get_vim_var_dict, get_vim_var_partial, set_vim_var_list};
use crate::eval::window::cur_win;
use crate::eval::{
    EVAL_EVALUATE, FUNCEXE_INIT, NL, Tv, check_luafunc_name, clear_evalarg, eval0,
    eval0_simple_funccal, eval1, may_call_simple_func, partial_name,
};
use crate::ex_eval::aborting;
use crate::garray::{ga_append, ga_init};
use crate::hashtab::hash_init;
use crate::main::{called_emsg, current_sctx, did_emsg};
use crate::memory::{xfree, xmalloc, xstrdup};
use crate::option::was_set_insecurely;
use crate::options::{kOptFoldexpr, kOptFoldtext, kWinOptFoldexpr};
use crate::runtime::sourcing_a_script;
use crate::types::{
    Arena, Failed, NUL, Object, OptionSetFlags, String_0, VAR_DICT, VAR_FUNC, VAR_LIST, VAR_NUMBER,
    VAR_PARTIAL, VAR_STRING, VAR_UNKNOWN, VarLock, Vv, dict_T, evalarg_T, exarg_T, funccal_entry_T,
    funcexe_T, garray_T, hashtab_T, kObjectTypeString, list_T, object_data, partial_T, ptrdiff_t,
    save_v_event_T, sctx_T, size_t, ssize_t, typval_T, typval_vval_union, uint8_t, varnumber_T,
    win_T,
};
use crate::winlayer::{Ea, Live};
use ::libc::atol;

/// A freshly declared typval.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VarLock::Unlocked,
    vval: typval_vval_union { v_number: 0 },
};

/// A freshly declared `evalarg_T`, before `fill_evalarg_from_eap`.
const UNSET_EVALARG: evalarg_T = evalarg_T {
    eval_flags: 0,
    eval_getline: None,
    eval_cookie: null_mut(),
    eval_tofree: null_mut(),
};

/// The scratch a Number or a Float is rendered into. `NUMBUFLEN` in the C.
const NUMBUFLEN: usize = 65;

/// One expression's evaluation state, owned by the frame that declared it.
type Ev = Live<evalarg_T>;

/// An empty growable array.
const UNSET_GA: garray_T = garray_T {
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
pub unsafe fn get_v_event(sve: *mut save_v_event_T) -> *mut dict_T {
    // SAFETY: `v:event` is a live dictionary from startup to exit.
    let v_event = unsafe { get_vim_var_dict(Vv::Event) };
    // Neither pointee may be reached through a `Live`: both hold a
    // `hashtab_T`, which points at its own inline array, and `DerefMut`
    // borrows the whole struct — see `winlayer::live`'s note. Their fields
    // are named by address instead.
    // SAFETY: the caller's promise about `sve`, and `v_event` as above.
    let saved: *mut hashtab_T = unsafe { &raw mut (*sve).sve_hashtab };
    // SAFETY: as above.
    let live: *mut hashtab_T = unsafe { &raw mut (*v_event).dv_hashtab };
    // SAFETY: as above.
    let did_save = unsafe { (*live).ht_used } > 0 as size_t;
    // SAFETY: as above.
    unsafe { (*sve).sve_did_save = did_save };
    if did_save {
        // A hashtab that has not outgrown its inline array holds
        // `ht_array` pointing *into itself*, so this is a move and not
        // a copy: the bytes go to `sve` and `hash_init` immediately
        // makes the source a fresh empty table. `restore_v_event` puts
        // them back at the address they came from, which is what makes
        // the self-reference valid again.
        // SAFETY: both name a whole `hashtab_T`, and the move is the one
        // the comment above describes.
        unsafe { saved.write(live.read()) };
        // SAFETY: `live` is `v:event`'s own hashtab.
        unsafe { hash_init(live) };
    }
    v_event
}

/// Put back what `get_v_event` saved.
///
/// # Safety
/// `v_event` and `sve` must be a pair `get_v_event` produced.
pub unsafe fn restore_v_event(v_event: *mut dict_T, sve: *mut save_v_event_T) {
    // SAFETY: the caller's promise -- the pair `get_v_event` produced.
    unsafe { tv_dict_free_contents(v_event) };
    // Named by address, not through a `Live`: as [`get_v_event`].
    // SAFETY: as above.
    let saved: *mut hashtab_T = unsafe { &raw mut (*sve).sve_hashtab };
    // SAFETY: as above.
    let live: *mut hashtab_T = unsafe { &raw mut (*v_event).dv_hashtab };
    // SAFETY: as above.
    if unsafe { (*sve).sve_did_save } {
        // The move back, to the address [`get_v_event`] took it from.
        // SAFETY: both name a whole `hashtab_T`.
        unsafe { live.write(saved.read()) };
    } else {
        // SAFETY: `live` is `v:event`'s own hashtab.
        unsafe { hash_init(live) };
    }
}

/// Bring up the evaluator: the `v:` variables and the function table.
///
/// # Safety
/// Called once, during startup.
pub unsafe fn eval_init() {
    unsafe { evalvars_init() };
    func_init();
}

/// Set up an `evalarg_T` for an expression that is part of an Ex command.
///
/// The line-getter is carried over only while sourcing a script, which is
/// what lets an expression there run onto a following line.
///
/// # Safety
/// `evalarg` must be valid; `eap` null or valid.
pub unsafe fn fill_evalarg_from_eap(evalarg: *mut evalarg_T, eap: *mut exarg_T, skip: bool) {
    // SAFETY: the caller's promise -- `evalarg` outlives the call.
    let mut evalarg = unsafe { Ev::new(evalarg) };
    *evalarg = UNSET_EVALARG;
    evalarg.eval_flags = if skip { 0 } else { EVAL_EVALUATE as c_int };
    if eap.is_null() {
        return;
    }
    // SAFETY: the caller's promise -- a non-null `eap` is the live Ex
    // command being run.
    let eap = unsafe { Ea::new(eap) };
    if unsafe { sourcing_a_script(eap.raw()) } != 0 {
        evalarg.eval_getline = eap.ea_getline;
        evalarg.eval_cookie = eap.cookie;
    }
}

/// Evaluate `arg` and answer its truth. `error` says whether the
/// evaluation itself failed, which is not the same as answering false.
///
/// # Safety
/// `arg` must be a NUL-terminated expression, `error` valid, `eap` null or
/// valid.
pub unsafe fn eval_to_bool(
    arg: *mut c_char,
    error: *mut bool,
    eap: *mut exarg_T,
    skip: bool,
    use_simple_function: bool,
) -> bool {
    let mut tv = UNSET_TV;
    let mut retval = false;
    let mut evalarg = UNSET_EVALARG;
    unsafe { fill_evalarg_from_eap(&raw mut evalarg, eap, skip) };
    let skipping = skip.then(Suppress::emsg_skip);
    let r = if use_simple_function {
        unsafe { eval0_simple_funccal(arg, &raw mut tv, eap, &raw mut evalarg) }
    } else {
        unsafe { eval0(arg, &raw mut tv, eap, &raw mut evalarg) }
    };
    if r.is_err() {
        unsafe { *error = true };
    } else {
        unsafe { *error = false };
        if !skip {
            retval = unsafe { tv_get_number_chk(&raw mut tv, error) } != 0;
            clear_local(&mut tv);
        }
    }
    drop(skipping);
    unsafe { clear_evalarg(&raw mut evalarg, eap) };
    retval
}

/// `eval1` with a fallback message: when the expression failed silently —
/// nothing aborted and nothing reported — say which expression it was.
///
/// # Safety
/// `arg` must point at the cursor into a NUL-terminated expression;
/// `rettv` valid; `eap` null or valid.
pub(crate) unsafe fn eval1_emsg(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    eap: *mut exarg_T,
) -> Result<(), Failed> {
    let start: *const c_char = unsafe { *arg };
    let did_emsg_before = did_emsg.get();
    let called_emsg_before = called_emsg.get();

    let mut evalarg = UNSET_EVALARG;
    unsafe { fill_evalarg_from_eap(&raw mut evalarg, eap, !eap.is_null() && (*eap).skip != 0) };
    let ret = unsafe { eval1(arg, rettv, &raw mut evalarg) };
    if ret.is_err()
        && !aborting()
        && did_emsg.get() == did_emsg_before
        && called_emsg.get() == called_emsg_before
    {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let start = unsafe { c_str(start) };
        semsg!("E15: Invalid expression: \"{start}\"");
    }
    unsafe { clear_evalarg(&raw mut evalarg, eap) };
    ret
}

/// Is this typval usable as an expression argument at all? An unset value
/// and an empty String are not.
///
/// # Safety
/// `tv` must be valid.
pub unsafe fn eval_expr_valid_arg(tv: *const typval_T) -> bool {
    // SAFETY: the caller's promise -- the typval outlives the call, and it
    // is only read through here.
    let tv = unsafe { Tv::new(tv.cast_mut()) };
    if tv.v_type == VAR_UNKNOWN {
        return false;
    }
    if tv.v_type != VAR_STRING {
        return true;
    }
    // SAFETY: `VAR_STRING` says `v_string` is the union's live member, and
    // a non-null one is NUL-terminated.
    let s = unsafe { tv.vval.v_string };
    !s.is_null() && unsafe { *s } as c_int != NUL
}

/// Call the partial in `expr`.
///
/// # Safety
/// `expr` must be a valid `VAR_PARTIAL`; `argv` must hold `argc` typvals.
pub(crate) unsafe fn eval_expr_partial(
    expr: *const typval_T,
    argv: *mut typval_T,
    argc: c_int,
    rettv: *mut typval_T,
) -> Result<(), Failed> {
    // SAFETY: the caller's promise -- a `VAR_PARTIAL`, so `v_partial` is
    // the union's live member.
    let partial = unsafe { (*expr).vval.v_partial };
    if partial.is_null() {
        return Err(Failed);
    }
    let s: *const c_char = unsafe { partial_name(partial) };
    if s.is_null() || unsafe { *s } as c_int == NUL {
        return Err(Failed);
    }
    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_evaluate = true;
    funcexe.fe_partial = partial;
    unsafe { call_func(s, -1, rettv, argc, argv, &raw mut funcexe) }?;
    Ok(())
}

/// Call the function `expr` names.
///
/// # Safety
/// As `eval_expr_partial`.
pub(crate) unsafe fn eval_expr_func(
    expr: *const typval_T,
    argv: *mut typval_T,
    argc: c_int,
    rettv: *mut typval_T,
) -> Result<(), Failed> {
    let mut buf: [c_char; NUMBUFLEN] = [0; NUMBUFLEN];
    // SAFETY: the caller's promise -- `expr` outlives the call, and it is
    // only read through here; `VAR_FUNC` says `v_string` is its live
    // member, and `buf` outlives the string rendered into it.
    let expr_tv = unsafe { Tv::new(expr.cast_mut()) };
    let s: *const c_char = if expr_tv.v_type == VAR_FUNC {
        unsafe { expr_tv.vval.v_string as *const c_char }
    } else {
        unsafe { tv_get_string_buf_chk(expr, buf.as_mut_ptr()) }
    };
    if s.is_null() || unsafe { *s } as c_int == NUL {
        return Err(Failed);
    }
    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_evaluate = true;
    unsafe { call_func(s, -1, rettv, argc, argv, &raw mut funcexe) }?;
    Ok(())
}

/// Evaluate `expr` as an expression *string*, which must consume all of it.
///
/// # Safety
/// `expr` and `rettv` must be valid.
pub(crate) unsafe fn eval_expr_string(
    expr: *const typval_T,
    rettv: *mut typval_T,
) -> Result<(), Failed> {
    let mut buf: [c_char; NUMBUFLEN] = [0; NUMBUFLEN];
    let mut s = unsafe { tv_get_string_buf_chk(expr, buf.as_mut_ptr()) } as *mut c_char;
    if s.is_null() {
        return Err(Failed);
    }
    s = unsafe { skipwhite(s) };
    unsafe { eval1_emsg(&raw mut s, rettv, null_mut()) }?;
    if unsafe { *skipwhite(s) } as c_int != NUL {
        unsafe { tv_clear(rettv) };
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let s = unsafe { c_str(s) };
        semsg!("E15: Invalid expression: \"{s}\"");
        return Err(Failed);
    }
    Ok(())
}

/// Evaluate whatever `expr` holds — a partial, a Funcref, a function name
/// or an expression string — with `argc` arguments.
///
/// # Safety
/// `expr` and `rettv` must be valid; `argv` must hold `argc` typvals.
pub unsafe fn eval_expr_typval(
    expr: *const typval_T,
    want_func: bool,
    argv: *mut typval_T,
    argc: c_int,
    rettv: *mut typval_T,
) -> Result<(), Failed> {
    // SAFETY: the caller's promise -- `expr` outlives the call and is only
    // read through here; each arm restates the same promise.
    let ty = unsafe { Tv::new(expr.cast_mut()) };
    if ty.v_type == VAR_PARTIAL {
        return unsafe { eval_expr_partial(expr, argv, argc, rettv) };
    }
    if ty.v_type == VAR_FUNC || want_func {
        return unsafe { eval_expr_func(expr, argv, argc, rettv) };
    }
    unsafe { eval_expr_string(expr, rettv) }
}

/// `eval_expr_typval` with no arguments, answering the result's truth.
///
/// # Safety
/// `expr` and `error` must be valid.
pub unsafe fn eval_expr_to_bool(expr: *const typval_T, error: *mut bool) -> bool {
    let mut argv = UNSET_TV;
    let mut rettv = UNSET_TV;
    if unsafe { eval_expr_typval(expr, false, &raw mut argv, 0, &raw mut rettv) }.is_err() {
        unsafe { *error = true };
        return false;
    }
    let res = unsafe { tv_get_number_chk(&raw mut rettv, error) } != 0;
    clear_local(&mut rettv);
    res
}

/// Evaluate `arg` for its String, or only parse it when `skip`.
///
/// # Safety
/// As `eval_to_bool`.
pub unsafe fn eval_to_string_skip(arg: *mut c_char, eap: *mut exarg_T, skip: bool) -> *mut c_char {
    let mut numbuf = NumBuf::new();
    let mut tv = UNSET_TV;
    let mut evalarg = UNSET_EVALARG;
    unsafe { fill_evalarg_from_eap(&raw mut evalarg, eap, skip) };
    let skipping = skip.then(Suppress::emsg_skip);
    let retval = if unsafe { eval0(arg, &raw mut tv, eap, &raw mut evalarg) }.is_err() || skip {
        null_mut()
    } else {
        let s = unsafe { xstrdup(numbuf.string(&raw mut tv)) };
        clear_local(&mut tv);
        s
    };
    drop(skipping);
    unsafe { clear_evalarg(&raw mut evalarg, eap) };
    retval
}

/// Step the cursor over an expression without evaluating it.
///
/// # Safety
/// `pp` must point at the cursor into a NUL-terminated expression;
/// `evalarg` null or valid.
pub unsafe fn skip_expr(pp: *mut *mut c_char, evalarg: *mut evalarg_T) -> Result<(), Failed> {
    // SAFETY: the caller's promise -- a non-null `evalarg` outlives the
    // call.
    let ev = (!evalarg.is_null()).then(|| unsafe { Ev::new(evalarg) });
    let save_flags = ev.map_or(0, |e| e.eval_flags);
    if let Some(mut e) = ev {
        e.eval_flags &= !(EVAL_EVALUATE as c_int);
    }
    // SAFETY: the caller's promise -- `pp` holds a cursor into a
    // NUL-terminated expression, and blanks stop at the terminator.
    unsafe { *pp = skipwhite(*pp) };
    let mut rettv = UNSET_TV;
    // Deliberately not handed `evalarg`: the flags were cleared on it
    // for the benefit of anything else looking, but this walk wants no
    // line getter either.
    // SAFETY: `pp` is the caller's cursor and `rettv` is this frame's.
    let res = unsafe { eval1(pp, &raw mut rettv, null_mut()) };
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
pub(crate) unsafe fn typval2string(tv: *mut typval_T, join_list: bool) -> *mut c_char {
    let mut numbuf = NumBuf::new();
    // SAFETY: the caller's promise -- the typval outlives the call, and
    // `VAR_LIST` says `v_list` is the union's live member.
    let value = unsafe { Tv::new(tv) };
    if join_list && value.v_type == VAR_LIST {
        let mut ga = UNSET_GA;
        // SAFETY: `ga` is this frame's.
        unsafe { ga_init(&raw mut ga, size_of::<c_char>() as c_int, 80) };
        // SAFETY: as above.
        let l = unsafe { value.vval.v_list };
        if !l.is_null() {
            // SAFETY: `l` is the typval's live List.
            let _ = unsafe { tv_list_join(&raw mut ga, l, c"\n".as_ptr()) };
            // SAFETY: as above.
            if unsafe { tv_list_len(l) } > 0 {
                // SAFETY: `ga` is this frame's.
                unsafe { ga_append(&raw mut ga, NL as uint8_t) };
            }
        }
        // SAFETY: `ga` is this frame's.
        unsafe { ga_append(&raw mut ga, NUL as uint8_t) };
        return ga.ga_data as *mut c_char;
    }
    if value.v_type == VAR_LIST || value.v_type == VAR_DICT {
        // SAFETY: the caller's typval.
        return unsafe { encode_tv2string(tv, null_mut()) };
    }
    // SAFETY: as above; `numbuf` outlives the string rendered into it.
    unsafe { xstrdup(numbuf.string(tv)) }
}

/// Evaluate `arg` for its String.
///
/// # Safety
/// As `eval_to_bool`.
pub unsafe fn eval_to_string_eap(
    arg: *mut c_char,
    join_list: bool,
    eap: *mut exarg_T,
    use_simple_function: bool,
) -> *mut c_char {
    let mut tv = UNSET_TV;
    let mut evalarg = UNSET_EVALARG;
    unsafe { fill_evalarg_from_eap(&raw mut evalarg, eap, !eap.is_null() && (*eap).skip != 0) };
    // The `eap` is read for the line getter above but deliberately not
    // handed on: this evaluation is not the Ex command's own.
    let r = if use_simple_function {
        unsafe { eval0_simple_funccal(arg, &raw mut tv, null_mut(), &raw mut evalarg) }
    } else {
        unsafe { eval0(arg, &raw mut tv, null_mut(), &raw mut evalarg) }
    };
    let retval = if r.is_err() {
        null_mut()
    } else {
        let s = unsafe { typval2string(&raw mut tv, join_list) };
        clear_local(&mut tv);
        s
    };
    unsafe { clear_evalarg(&raw mut evalarg, null_mut()) };
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
    unsafe { eval_to_string_eap(arg, join_list, null_mut(), use_simple_function) }
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
    let mut funccal_entry = funccal_entry_T {
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
pub unsafe fn eval_to_number(expr: *mut c_char, use_simple_function: bool) -> varnumber_T {
    let mut evalarg = EVALARG_EVALUATE;
    let mut rettv = UNSET_TV;
    let mut p = unsafe { skipwhite(expr) };
    let _no_emsg = Suppress::emsg();
    // Note the shortcut is handed the *unskipped* expression, unlike `eval1`.
    let simple = if use_simple_function {
        unsafe { may_call_simple_func(expr, &raw mut rettv) }
    } else {
        Ok(Parsed::NotThis)
    };
    let r = match simple {
        Ok(Parsed::NotThis) => unsafe { eval1(&raw mut p, &raw mut rettv, &raw mut evalarg) },
        other => other.map(|_| ()),
    };

    if r.is_err() {
        -1
    } else {
        let n = unsafe { tv_get_number_chk(&raw mut rettv, null_mut()) };
        clear_local(&mut rettv);
        n
    }
}

/// Evaluate `arg` into a heap typval the caller owns; null on failure.
///
/// # Safety
/// `arg` must be a NUL-terminated expression; `eap` null or valid.
pub unsafe fn eval_expr(arg: *mut c_char, eap: *mut exarg_T) -> *mut typval_T {
    unsafe { eval_expr_ext(arg, eap, false) }
}

/// As `eval_expr`, optionally taking the shortcut for an expression that is
/// nothing but one function call.
///
/// # Safety
/// As `eval_expr`.
pub unsafe fn eval_expr_ext(
    arg: *mut c_char,
    eap: *mut exarg_T,
    use_simple_function: bool,
) -> *mut typval_T {
    let mut tv = unsafe { xmalloc(size_of::<typval_T>()) } as *mut typval_T;
    let mut evalarg = UNSET_EVALARG;
    unsafe { fill_evalarg_from_eap(&raw mut evalarg, eap, !eap.is_null() && (*eap).skip != 0) };
    // `eval0_simple_funccal` falls through to `eval0` itself, so the two
    // arms are the whole of the choice: nothing here can be left undone.
    let r = if use_simple_function {
        unsafe { eval0_simple_funccal(arg, tv, eap, &raw mut evalarg) }
    } else {
        unsafe { eval0(arg, tv, eap, &raw mut evalarg) }
    };
    if r.is_err() {
        unsafe { xfree(tv as *mut c_void) };
        tv = null_mut();
    }
    unsafe { clear_evalarg(&raw mut evalarg, eap) };
    tv
}

/// Call a Vimscript function by name with `argc` typvals.
///
/// # Safety
/// `func` must be NUL-terminated, `argv` must hold `argc` typvals and
/// `rettv` must be valid.
pub unsafe fn call_vim_function(
    func: *const c_char,
    argc: c_int,
    argv: *mut typval_T,
    rettv: *mut typval_T,
) -> Result<(), Failed> {
    let mut func = func;
    let mut len = unsafe { cstr::bytes_at(func) }.len() as c_int;
    let mut pt: *mut partial_T = null_mut();
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
            // SAFETY: `v:lua` holds a live partial from startup to exit.
            pt = unsafe { get_vim_var_partial(Vv::Lua) };
        }
        // SAFETY: the caller's promise about `rettv`.
        unsafe { (*rettv).v_type = VAR_UNKNOWN };
        let mut funcexe: funcexe_T = FUNCEXE_INIT;
        funcexe.fe_firstline = cur_win().w_cursor.lnum;
        funcexe.fe_lastline = cur_win().w_cursor.lnum;
        funcexe.fe_evaluate = true;
        funcexe.fe_partial = pt;
        ret = unsafe { call_func(func, len, rettv, argc, argv, &raw mut funcexe) };
    }

    if ret.is_err() {
        unsafe { tv_clear(rettv) };
    }
    ret
}

/// `call_vim_function`, answering an owned copy of the result's String.
///
/// # Safety
/// As `call_vim_function`.
pub unsafe fn call_func_retstr(
    func: *const c_char,
    argc: c_int,
    argv: *mut typval_T,
) -> *mut c_void {
    let mut numbuf = NumBuf::new();
    let mut rettv = UNSET_TV;
    if unsafe { call_vim_function(func, argc, argv, &raw mut rettv) }.is_err() {
        return null_mut();
    }
    let retval = unsafe { xstrdup(numbuf.string(&raw mut rettv)) };
    clear_local(&mut rettv);
    retval as *mut c_void
}

/// `call_vim_function`, answering the result's List — which the caller then
/// owns the reference to. Null for anything else.
///
/// # Safety
/// As `call_vim_function`.
pub unsafe fn call_func_retlist(
    func: *const c_char,
    argc: c_int,
    argv: *mut typval_T,
) -> *mut c_void {
    let mut rettv = UNSET_TV;
    if unsafe { call_vim_function(func, argc, argv, &raw mut rettv) }.is_err() {
        return null_mut();
    }
    if rettv.v_type != VAR_LIST {
        clear_local(&mut rettv);
        return null_mut();
    }
    unsafe { rettv.vval.v_list as *mut c_void }
}

/// Run 'foldexpr' for the window's current line. `cp` comes back holding
/// the leading marker character (`>`, `<`, `=`, `a`, `s`) when there was
/// one.
///
/// # Safety
/// `wp` and `cp` must be valid.
pub unsafe fn eval_foldexpr(wp: *mut win_T, cp: *mut c_int) -> c_int {
    let mut evalarg = EVALARG_EVALUATE;
    let saved_sctx: sctx_T = current_sctx.get();
    // SAFETY: the caller's promise -- a live window.
    let use_sandbox = unsafe { was_set_insecurely(wp, kOptFoldexpr, OptionSetFlags::LOCAL) };
    // SAFETY: as above; the window outlives this call.
    let wp = unsafe { Win::new(wp) };
    // SAFETY: an option string is NUL-terminated.
    let arg = unsafe { skipwhite(wp.w_onebuf_opt.wo_fde) };
    current_sctx.set(wp.w_onebuf_opt.wo_script_ctx[kWinOptFoldexpr as usize]);
    let retval: varnumber_T = {
        let _no_emsg = Suppress::emsg();
        let _sandboxed = use_sandbox.then(Lock::sandbox);
        let _locked = Lock::text();
        // SAFETY: the caller's promise about `cp`.
        unsafe { *cp = NUL };

        let mut tv = UNSET_TV;
        let mut retval: varnumber_T = 0;
        if unsafe { eval0_simple_funccal(arg, &raw mut tv, null_mut(), &raw mut evalarg) }.is_ok() {
            if tv.v_type == VAR_NUMBER {
                retval = unsafe { tv.vval.v_number };
            } else if tv.v_type != VAR_STRING || unsafe { tv.vval.v_string }.is_null() {
                retval = 0;
            } else {
                // SAFETY: `VAR_STRING` says `v_string` is the live member,
                // and a non-null one is NUL-terminated.
                let mut s = unsafe { tv.vval.v_string };
                let first = unsafe { *s };
                // A leading non-digit that is not a minus sign is the
                // fold marker; the rest is the level.
                if first as c_int != NUL
                    && !ascii_isdigit(first as c_int)
                    && first != b'-' as c_char
                {
                    // SAFETY: the caller's promise about `cp`; `first` is
                    // not the terminator, so the rest is inside the string.
                    unsafe { *cp = first as u8 as c_int };
                    s = unsafe { s.add(1) };
                }
                // SAFETY: `s` is inside the NUL-terminated string.
                retval = unsafe { atol(s) } as varnumber_T;
            }
            clear_local(&mut tv);
        }
        retval
    };
    unsafe { clear_evalarg(&raw mut evalarg, null_mut()) };
    current_sctx.set(saved_sctx);
    retval as c_int
}

/// Run 'foldtext' for the window's current fold. A List comes back as an
/// Object so the caller can keep its per-chunk highlighting; anything else
/// is coerced to a String.
///
/// # Safety
/// `wp` must be valid.
pub unsafe fn eval_foldtext(wp: *mut win_T) -> Object {
    let mut evalarg = EVALARG_EVALUATE;
    let mut numbuf = NumBuf::new();
    /// The empty String an error answers with.
    fn empty_string() -> Object {
        Object {
            type_0: kObjectTypeString,
            data: object_data {
                string: String_0::from_raw_parts(null_mut(), 0 as size_t),
            },
        }
    }

    // SAFETY: the caller's promise -- a live window.
    let use_sandbox = unsafe { was_set_insecurely(wp, kOptFoldtext, OptionSetFlags::LOCAL) };
    // SAFETY: as above; the window outlives this call.
    let arg = unsafe { Win::new(wp) }.w_onebuf_opt.wo_fdt;
    let mut funccal_entry = funccal_entry_T {
        top_funccal: null_mut(),
        next: null_mut(),
    };
    unsafe { save_funccal(&raw mut funccal_entry) };
    let _sandboxed = use_sandbox.then(Lock::sandbox);
    let _locked = Lock::text();

    let mut tv = UNSET_TV;
    let retval = if unsafe { eval0_simple_funccal(arg, &raw mut tv, null_mut(), &raw mut evalarg) }
        .is_err()
    {
        empty_string()
    } else {
        let obj = if tv.v_type == VAR_LIST {
            unsafe { vim_to_object(&raw mut tv, null_mut::<Arena>(), false) }
        } else {
            Object {
                type_0: kObjectTypeString,
                data: object_data {
                    string: unsafe { cstr_to_string(numbuf.string(&raw mut tv)) },
                },
            }
        };
        clear_local(&mut tv);
        obj
    };

    unsafe { clear_evalarg(&raw mut evalarg, null_mut()) };
    unsafe { restore_funccal() };
    retval
}

/// Fill `v:argv` from the process arguments. Every item is locked.
///
/// # Safety
/// `argv` must hold `argc` NUL-terminated strings.
pub unsafe fn set_argv_var(argv: *mut *mut c_char, argc: c_int) {
    // SAFETY: the List is fresh and this frame's until `v:argv` takes it.
    let l: *mut list_T = unsafe { tv_list_alloc(argc as ptrdiff_t) };
    // SAFETY: `l` is that List.
    unsafe { tv_list_set_lock(l, VarLock::Fixed) };
    for i in 0..argc {
        // SAFETY: the caller's promise -- `argc` NUL-terminated strings,
        // so slot `i` is one of them; -1 asks the callee to measure it.
        let arg = unsafe { *argv.offset(i as isize) } as *const c_char;
        // SAFETY: as above.
        unsafe { tv_list_append_string(l, arg, -1 as ssize_t) };
        // SAFETY: the item just appended is the List's last.
        unsafe { (*tv_list_last(l)).li_tv.v_lock = VarLock::Fixed };
    }
    // SAFETY: `v:argv` takes the List over.
    unsafe { set_vim_var_list(Vv::Argv, l) };
}

/// Render a typval for display, as `:echo` would. A null typval is the
/// "no such variable" text, which is what the debugger prints.
///
/// # Safety
/// `arg` must be null or valid.
pub unsafe fn typval_tostring(arg: *mut typval_T, quotes: bool) -> *mut c_char {
    if arg.is_null() {
        // SAFETY: the text is a NUL-terminated literal.
        return unsafe { xstrdup(c"(does not exist)".as_ptr()) };
    }
    // SAFETY: the caller's promise -- a non-null typval outlives the call.
    let value = unsafe { Tv::new(arg) };
    if !quotes && value.v_type == VAR_STRING {
        // SAFETY: `VAR_STRING` says `v_string` is the union's live member,
        // and a non-null one is NUL-terminated.
        let s = unsafe { value.vval.v_string };
        let s = if s.is_null() {
            c"".as_ptr()
        } else {
            s as *const c_char
        };
        // SAFETY: `s` is NUL-terminated either way.
        return unsafe { xstrdup(s) };
    }
    // SAFETY: the caller's typval.
    unsafe { encode_tv2string(arg, null_mut()) }
}

/// Blank a typval in place.
///
/// # Safety
/// `tv` must be null or valid.
#[inline]
pub(crate) unsafe fn tv_init(tv: *mut typval_T) {
    if !tv.is_null() {
        unsafe { tv.cast::<u8>().write_bytes(0, size_of::<typval_T>()) };
    }
}
