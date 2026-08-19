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

use crate::semsg_c;
use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of;
use core::ptr::null_mut;

use crate::api::private::converter::vim_to_object;
use crate::api::private::helpers::cstr_to_string;
use crate::ascii::ascii_isdigit;
use crate::charset::skipwhite;
use crate::eval::encode::encode_tv2string;
use crate::eval::typval::{
    tv_clear, tv_dict_free_contents, tv_get_number_chk, tv_get_string, tv_get_string_buf_chk,
    tv_list_alloc, tv_list_append_string, tv_list_join, tv_list_last, tv_list_len,
    tv_list_set_lock,
};
use crate::eval::userfunc::{call_func, func_init, restore_funccal, save_funccal};
use crate::eval::vars::{evalvars_init, get_vim_var_dict, get_vim_var_partial, set_vim_var_list};
use crate::eval::{
    EVAL_EVALUATE, FUNCEXE_INIT, NL, NOTDONE, check_luafunc_name, clear_evalarg, eval0,
    eval0_simple_funccal, eval1, kWinOptFoldexpr, may_call_simple_func, partial_name,
};
use crate::ex_eval::aborting;
use crate::garray::{ga_append, ga_init};
use crate::hashtab::hash_init;
use crate::main::{
    EVALARG_EVALUATE, called_emsg, current_sctx, curwin, did_emsg, e_invexpr2, emsg_off, emsg_skip,
    sandbox, textlock,
};
use crate::memory::{xfree, xmalloc, xstrdup};
use crate::option::was_set_insecurely;
use crate::options::{kOptFoldexpr, kOptFoldtext};
use crate::os::cshim::gettext;
use crate::runtime::sourcing_a_script;
use crate::types::{
    Arena, FAIL, NUL, OK, Object, OptionSetFlags, String_0, VAR_DICT, VAR_FIXED, VAR_FUNC,
    VAR_LIST, VAR_NUMBER, VAR_PARTIAL, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, Vv, dict_T,
    evalarg_T, exarg_T, funccal_entry_T, funcexe_T, garray_T, kObjectTypeString, list_T,
    object_data, partial_T, ptrdiff_t, save_v_event_T, sctx_T, size_t, ssize_t, typval_T,
    typval_vval_union, uint8_t, varnumber_T, win_T,
};
use ::libc::{atol, memcmp, memset, strlen};

/// A freshly declared typval.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VAR_UNLOCKED,
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
    unsafe {
        let v_event = get_vim_var_dict(Vv::Event);
        (*sve).sve_did_save = (*v_event).dv_hashtab.ht_used > 0 as size_t;
        if (*sve).sve_did_save {
            (*sve).sve_hashtab = (*v_event).dv_hashtab;
            hash_init(&raw mut (*v_event).dv_hashtab);
        }
        v_event
    }
}

/// Put back what `get_v_event` saved.
///
/// # Safety
/// `v_event` and `sve` must be a pair `get_v_event` produced.
pub unsafe fn restore_v_event(v_event: *mut dict_T, sve: *mut save_v_event_T) {
    unsafe {
        tv_dict_free_contents(v_event);
        if (*sve).sve_did_save {
            (*v_event).dv_hashtab = (*sve).sve_hashtab;
        } else {
            hash_init(&raw mut (*v_event).dv_hashtab);
        }
    }
}

/// Bring up the evaluator: the `v:` variables and the function table.
///
/// # Safety
/// Called once, during startup.
pub unsafe fn eval_init() {
    unsafe {
        evalvars_init();
        func_init();
    }
}

/// Set up an `evalarg_T` for an expression that is part of an Ex command.
///
/// The line-getter is carried over only while sourcing a script, which is
/// what lets an expression there run onto a following line.
///
/// # Safety
/// `evalarg` must be valid; `eap` null or valid.
pub unsafe fn fill_evalarg_from_eap(evalarg: *mut evalarg_T, eap: *mut exarg_T, skip: bool) {
    unsafe {
        *evalarg = UNSET_EVALARG;
        (*evalarg).eval_flags = if skip { 0 } else { EVAL_EVALUATE as c_int };
        if eap.is_null() {
            return;
        }
        if sourcing_a_script(eap) != 0 {
            (*evalarg).eval_getline = (*eap).ea_getline;
            (*evalarg).eval_cookie = (*eap).cookie;
        }
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
    unsafe {
        let mut tv = UNSET_TV;
        let mut retval = false;
        let mut evalarg = UNSET_EVALARG;
        fill_evalarg_from_eap(&raw mut evalarg, eap, skip);
        if skip {
            *emsg_skip.ptr() += 1;
        }
        let r = if use_simple_function {
            eval0_simple_funccal(arg, &raw mut tv, eap, &raw mut evalarg)
        } else {
            eval0(arg, &raw mut tv, eap, &raw mut evalarg)
        };
        if r == FAIL {
            *error = true;
        } else {
            *error = false;
            if !skip {
                retval = tv_get_number_chk(&raw mut tv, error) != 0;
                tv_clear(&raw mut tv);
            }
        }
        if skip {
            *emsg_skip.ptr() -= 1;
        }
        clear_evalarg(&raw mut evalarg, eap);
        retval
    }
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
) -> c_int {
    unsafe {
        let start: *const c_char = *arg;
        let did_emsg_before = did_emsg.get();
        let called_emsg_before = called_emsg.get();

        let mut evalarg = UNSET_EVALARG;
        fill_evalarg_from_eap(&raw mut evalarg, eap, !eap.is_null() && (*eap).skip != 0);
        let ret = eval1(arg, rettv, &raw mut evalarg);
        if ret == FAIL
            && !aborting()
            && did_emsg.get() == did_emsg_before
            && called_emsg.get() == called_emsg_before
        {
            semsg_c!(gettext(e_invexpr2.as_ptr()), start);
        }
        clear_evalarg(&raw mut evalarg, eap);
        ret
    }
}

/// Is this typval usable as an expression argument at all? An unset value
/// and an empty String are not.
///
/// # Safety
/// `tv` must be valid.
pub unsafe fn eval_expr_valid_arg(tv: *const typval_T) -> bool {
    unsafe {
        (*tv).v_type != VAR_UNKNOWN
            && ((*tv).v_type != VAR_STRING
                || (!(*tv).vval.v_string.is_null() && *(*tv).vval.v_string as c_int != NUL))
    }
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
) -> c_int {
    unsafe {
        let partial = (*expr).vval.v_partial;
        if partial.is_null() {
            return FAIL;
        }
        let s: *const c_char = partial_name(partial);
        if s.is_null() || *s as c_int == NUL {
            return FAIL;
        }
        let mut funcexe: funcexe_T = FUNCEXE_INIT;
        funcexe.fe_evaluate = true;
        funcexe.fe_partial = partial;
        if call_func(s, -1, rettv, argc, argv, &raw mut funcexe) == FAIL {
            return FAIL;
        }
        OK
    }
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
) -> c_int {
    unsafe {
        let mut buf: [c_char; NUMBUFLEN] = [0; NUMBUFLEN];
        let s: *const c_char = if (*expr).v_type == VAR_FUNC {
            (*expr).vval.v_string as *const c_char
        } else {
            tv_get_string_buf_chk(expr, buf.as_mut_ptr())
        };
        if s.is_null() || *s as c_int == NUL {
            return FAIL;
        }
        let mut funcexe: funcexe_T = FUNCEXE_INIT;
        funcexe.fe_evaluate = true;
        if call_func(s, -1, rettv, argc, argv, &raw mut funcexe) == FAIL {
            return FAIL;
        }
        OK
    }
}

/// Evaluate `expr` as an expression *string*, which must consume all of it.
///
/// # Safety
/// `expr` and `rettv` must be valid.
pub(crate) unsafe fn eval_expr_string(expr: *const typval_T, rettv: *mut typval_T) -> c_int {
    unsafe {
        let mut buf: [c_char; NUMBUFLEN] = [0; NUMBUFLEN];
        let mut s = tv_get_string_buf_chk(expr, buf.as_mut_ptr()) as *mut c_char;
        if s.is_null() {
            return FAIL;
        }
        s = skipwhite(s);
        if eval1_emsg(&raw mut s, rettv, null_mut()) == FAIL {
            return FAIL;
        }
        if *skipwhite(s) as c_int != NUL {
            tv_clear(rettv);
            semsg_c!(gettext(e_invexpr2.as_ptr()), s);
            return FAIL;
        }
        OK
    }
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
) -> c_int {
    unsafe {
        if (*expr).v_type == VAR_PARTIAL {
            return eval_expr_partial(expr, argv, argc, rettv);
        }
        if (*expr).v_type == VAR_FUNC || want_func {
            return eval_expr_func(expr, argv, argc, rettv);
        }
        eval_expr_string(expr, rettv)
    }
}

/// `eval_expr_typval` with no arguments, answering the result's truth.
///
/// # Safety
/// `expr` and `error` must be valid.
pub unsafe fn eval_expr_to_bool(expr: *const typval_T, error: *mut bool) -> bool {
    unsafe {
        let mut argv = UNSET_TV;
        let mut rettv = UNSET_TV;
        if eval_expr_typval(expr, false, &raw mut argv, 0, &raw mut rettv) == FAIL {
            *error = true;
            return false;
        }
        let res = tv_get_number_chk(&raw mut rettv, error) != 0;
        tv_clear(&raw mut rettv);
        res
    }
}

/// Evaluate `arg` for its String, or only parse it when `skip`.
///
/// # Safety
/// As `eval_to_bool`.
pub unsafe fn eval_to_string_skip(arg: *mut c_char, eap: *mut exarg_T, skip: bool) -> *mut c_char {
    unsafe {
        let mut tv = UNSET_TV;
        let mut evalarg = UNSET_EVALARG;
        fill_evalarg_from_eap(&raw mut evalarg, eap, skip);
        if skip {
            *emsg_skip.ptr() += 1;
        }
        let retval = if eval0(arg, &raw mut tv, eap, &raw mut evalarg) == FAIL || skip {
            null_mut()
        } else {
            let s = xstrdup(tv_get_string(&raw mut tv));
            tv_clear(&raw mut tv);
            s
        };
        if skip {
            *emsg_skip.ptr() -= 1;
        }
        clear_evalarg(&raw mut evalarg, eap);
        retval
    }
}

/// Step the cursor over an expression without evaluating it.
///
/// # Safety
/// `pp` must point at the cursor into a NUL-terminated expression;
/// `evalarg` null or valid.
pub unsafe fn skip_expr(pp: *mut *mut c_char, evalarg: *mut evalarg_T) -> c_int {
    unsafe {
        let save_flags = if evalarg.is_null() {
            0
        } else {
            (*evalarg).eval_flags
        };
        if !evalarg.is_null() {
            (*evalarg).eval_flags &= !(EVAL_EVALUATE as c_int);
        }
        *pp = skipwhite(*pp);
        let mut rettv = UNSET_TV;
        // Deliberately not handed `evalarg`: the flags were cleared on it
        // for the benefit of anything else looking, but this walk wants no
        // line getter either.
        let res = eval1(pp, &raw mut rettv, null_mut());
        if !evalarg.is_null() {
            (*evalarg).eval_flags = save_flags;
        }
        res
    }
}

/// Render a typval as the String a caller of the evaluator expects: a List
/// joined with newlines when `join_list`, otherwise the `string()` form for
/// a container and the plain coercion for everything else.
///
/// # Safety
/// `tv` must be valid.
pub(crate) unsafe fn typval2string(tv: *mut typval_T, join_list: bool) -> *mut c_char {
    unsafe {
        if join_list && (*tv).v_type == VAR_LIST {
            let mut ga = UNSET_GA;
            ga_init(&raw mut ga, size_of::<c_char>() as c_int, 80);
            if !(*tv).vval.v_list.is_null() {
                tv_list_join(&raw mut ga, (*tv).vval.v_list, c"\n".as_ptr());
                if tv_list_len((*tv).vval.v_list) > 0 {
                    ga_append(&raw mut ga, NL as uint8_t);
                }
            }
            ga_append(&raw mut ga, NUL as uint8_t);
            return ga.ga_data as *mut c_char;
        }
        if (*tv).v_type == VAR_LIST || (*tv).v_type == VAR_DICT {
            return encode_tv2string(tv, null_mut());
        }
        xstrdup(tv_get_string(tv))
    }
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
    unsafe {
        let mut tv = UNSET_TV;
        let mut evalarg = UNSET_EVALARG;
        fill_evalarg_from_eap(&raw mut evalarg, eap, !eap.is_null() && (*eap).skip != 0);
        // The `eap` is read for the line getter above but deliberately not
        // handed on: this evaluation is not the Ex command's own.
        let r = if use_simple_function {
            eval0_simple_funccal(arg, &raw mut tv, null_mut(), &raw mut evalarg)
        } else {
            eval0(arg, &raw mut tv, null_mut(), &raw mut evalarg)
        };
        let retval = if r == FAIL {
            null_mut()
        } else {
            let s = typval2string(&raw mut tv, join_list);
            tv_clear(&raw mut tv);
            s
        };
        clear_evalarg(&raw mut evalarg, null_mut());
        retval
    }
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
    unsafe {
        let mut funccal_entry = funccal_entry_T {
            top_funccal: null_mut(),
            next: null_mut(),
        };
        save_funccal(&raw mut funccal_entry);
        if use_sandbox {
            *sandbox.ptr() += 1;
        }
        *textlock.ptr() += 1;
        let retval = eval_to_string(arg, false, use_simple_function);
        if use_sandbox {
            *sandbox.ptr() -= 1;
        }
        *textlock.ptr() -= 1;
        restore_funccal();
        retval
    }
}

/// Evaluate `expr` for its Number, silently. -1 for a failure, which is
/// not distinguishable from a result of -1.
///
/// # Safety
/// `expr` must be a NUL-terminated expression.
pub unsafe fn eval_to_number(expr: *mut c_char, use_simple_function: bool) -> varnumber_T {
    unsafe {
        let mut rettv = UNSET_TV;
        let mut p = skipwhite(expr);
        *emsg_off.ptr() += 1;
        let mut r = NOTDONE;
        if use_simple_function {
            // Note it is handed the *unskipped* expression, unlike `eval1`.
            r = may_call_simple_func(expr, &raw mut rettv);
        }
        if r == NOTDONE {
            r = eval1(&raw mut p, &raw mut rettv, EVALARG_EVALUATE.ptr());
        }
        let retval = if r == FAIL {
            -1
        } else {
            let n = tv_get_number_chk(&raw mut rettv, null_mut());
            tv_clear(&raw mut rettv);
            n
        };
        *emsg_off.ptr() -= 1;
        retval
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
    unsafe {
        let mut tv = xmalloc(size_of::<typval_T>()) as *mut typval_T;
        let mut evalarg = UNSET_EVALARG;
        fill_evalarg_from_eap(&raw mut evalarg, eap, !eap.is_null() && (*eap).skip != 0);
        let mut r = NOTDONE;
        if use_simple_function {
            r = eval0_simple_funccal(arg, tv, eap, &raw mut evalarg);
        }
        if r == NOTDONE {
            r = eval0(arg, tv, eap, &raw mut evalarg);
        }
        if r == FAIL {
            xfree(tv as *mut c_void);
            tv = null_mut();
        }
        clear_evalarg(&raw mut evalarg, eap);
        tv
    }
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
) -> c_int {
    unsafe {
        let mut func = func;
        let mut len = strlen(func) as c_int;
        let mut pt: *mut partial_T = null_mut();
        let mut ret = FAIL;

        'fail: {
            if len >= 6 && memcmp(func.cast(), c"v:lua.".as_ptr().cast(), 6 as size_t) == 0 {
                func = func.add(6);
                len = check_luafunc_name(func, false);
                if len == 0 {
                    break 'fail;
                }
                pt = get_vim_var_partial(Vv::Lua);
            }
            (*rettv).v_type = VAR_UNKNOWN;
            let mut funcexe: funcexe_T = FUNCEXE_INIT;
            funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
            funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
            funcexe.fe_evaluate = true;
            funcexe.fe_partial = pt;
            ret = call_func(func, len, rettv, argc, argv, &raw mut funcexe);
        }

        if ret == FAIL {
            tv_clear(rettv);
        }
        ret
    }
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
    unsafe {
        let mut rettv = UNSET_TV;
        if call_vim_function(func, argc, argv, &raw mut rettv) == FAIL {
            return null_mut();
        }
        let retval = xstrdup(tv_get_string(&raw mut rettv));
        tv_clear(&raw mut rettv);
        retval as *mut c_void
    }
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
    unsafe {
        let mut rettv = UNSET_TV;
        if call_vim_function(func, argc, argv, &raw mut rettv) == FAIL {
            return null_mut();
        }
        if rettv.v_type != VAR_LIST {
            tv_clear(&raw mut rettv);
            return null_mut();
        }
        rettv.vval.v_list as *mut c_void
    }
}

/// Run 'foldexpr' for the window's current line. `cp` comes back holding
/// the leading marker character (`>`, `<`, `=`, `a`, `s`) when there was
/// one.
///
/// # Safety
/// `wp` and `cp` must be valid.
pub unsafe fn eval_foldexpr(wp: *mut win_T, cp: *mut c_int) -> c_int {
    unsafe {
        let saved_sctx: sctx_T = current_sctx.get();
        let use_sandbox = was_set_insecurely(wp, kOptFoldexpr, OptionSetFlags::LOCAL);
        let arg = skipwhite((*wp).w_onebuf_opt.wo_fde);
        current_sctx.set((*wp).w_onebuf_opt.wo_script_ctx[kWinOptFoldexpr as usize]);
        *emsg_off.ptr() += 1;
        if use_sandbox {
            *sandbox.ptr() += 1;
        }
        *textlock.ptr() += 1;
        *cp = NUL;

        let mut tv = UNSET_TV;
        let mut retval: varnumber_T = 0;
        if eval0_simple_funccal(arg, &raw mut tv, null_mut(), EVALARG_EVALUATE.ptr()) != FAIL {
            if tv.v_type == VAR_NUMBER {
                retval = tv.vval.v_number;
            } else if tv.v_type != VAR_STRING || tv.vval.v_string.is_null() {
                retval = 0;
            } else {
                let mut s = tv.vval.v_string;
                // A leading non-digit that is not a minus sign is the fold
                // marker; the rest is the level.
                if *s as c_int != NUL && !ascii_isdigit(*s as c_int) && *s != b'-' as c_char {
                    *cp = *s as u8 as c_int;
                    s = s.add(1);
                }
                retval = atol(s) as varnumber_T;
            }
            tv_clear(&raw mut tv);
        }

        *emsg_off.ptr() -= 1;
        if use_sandbox {
            *sandbox.ptr() -= 1;
        }
        *textlock.ptr() -= 1;
        clear_evalarg(EVALARG_EVALUATE.ptr(), null_mut());
        current_sctx.set(saved_sctx);
        retval as c_int
    }
}

/// Run 'foldtext' for the window's current fold. A List comes back as an
/// Object so the caller can keep its per-chunk highlighting; anything else
/// is coerced to a String.
///
/// # Safety
/// `wp` must be valid.
pub unsafe fn eval_foldtext(wp: *mut win_T) -> Object {
    unsafe {
        /// The empty String an error answers with.
        fn empty_string() -> Object {
            Object {
                type_0: kObjectTypeString,
                data: object_data {
                    string: String_0 {
                        data: null_mut(),
                        size: 0 as size_t,
                    },
                },
            }
        }

        let use_sandbox = was_set_insecurely(wp, kOptFoldtext, OptionSetFlags::LOCAL);
        let arg = (*wp).w_onebuf_opt.wo_fdt;
        let mut funccal_entry = funccal_entry_T {
            top_funccal: null_mut(),
            next: null_mut(),
        };
        save_funccal(&raw mut funccal_entry);
        if use_sandbox {
            *sandbox.ptr() += 1;
        }
        *textlock.ptr() += 1;

        let mut tv = UNSET_TV;
        let retval =
            if eval0_simple_funccal(arg, &raw mut tv, null_mut(), EVALARG_EVALUATE.ptr()) == FAIL {
                empty_string()
            } else {
                let obj = if tv.v_type == VAR_LIST {
                    vim_to_object(&raw mut tv, null_mut::<Arena>(), false)
                } else {
                    Object {
                        type_0: kObjectTypeString,
                        data: object_data {
                            string: cstr_to_string(tv_get_string(&raw mut tv)),
                        },
                    }
                };
                tv_clear(&raw mut tv);
                obj
            };

        clear_evalarg(EVALARG_EVALUATE.ptr(), null_mut());
        if use_sandbox {
            *sandbox.ptr() -= 1;
        }
        *textlock.ptr() -= 1;
        restore_funccal();
        retval
    }
}

/// Fill `v:argv` from the process arguments. Every item is locked.
///
/// # Safety
/// `argv` must hold `argc` NUL-terminated strings.
pub unsafe fn set_argv_var(argv: *mut *mut c_char, argc: c_int) {
    unsafe {
        let l: *mut list_T = tv_list_alloc(argc as ptrdiff_t);
        tv_list_set_lock(l, VAR_FIXED);
        for i in 0..argc {
            tv_list_append_string(l, *argv.offset(i as isize) as *const c_char, -1 as ssize_t);
            (*tv_list_last(l)).li_tv.v_lock = VAR_FIXED;
        }
        set_vim_var_list(Vv::Argv, l);
    }
}

/// Render a typval for display, as `:echo` would. A null typval is the
/// "no such variable" text, which is what the debugger prints.
///
/// # Safety
/// `arg` must be null or valid.
pub unsafe fn typval_tostring(arg: *mut typval_T, quotes: bool) -> *mut c_char {
    unsafe {
        if arg.is_null() {
            return xstrdup(c"(does not exist)".as_ptr());
        }
        if !quotes && (*arg).v_type == VAR_STRING {
            return xstrdup(if (*arg).vval.v_string.is_null() {
                c"".as_ptr()
            } else {
                (*arg).vval.v_string as *const c_char
            });
        }
        encode_tv2string(arg, null_mut())
    }
}

/// Blank a typval in place.
///
/// # Safety
/// `tv` must be null or valid.
#[inline]
pub(crate) unsafe fn tv_init(tv: *mut typval_T) {
    unsafe {
        if !tv.is_null() {
            memset(tv as *mut c_void, 0, size_of::<typval_T>());
        }
    }
}
