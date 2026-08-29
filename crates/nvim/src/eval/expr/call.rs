//! Calling something: a function name, a method, a lambda or a partial.
//!
//! All three entry points share one shape. The value already in `rettv` is
//! the *callee* (or, for `->`, the base the method is applied to); it is
//! moved into a local, `rettv` is blanked so the call can fill it, and the
//! local is cleared afterwards — after the call, so that a function may
//! delete the Funcref it is being reached through while its own arguments
//! are still being evaluated.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg;
use crate::winlayer::{Live, Win};
use core::ffi::{c_char, c_int, c_void};
use core::ptr::{null, null_mut};

use crate::ascii::ascii_iswhite;
use crate::charset::skipwhite;
use crate::eval::typval::{tv_clear, tv_dict_unref, tv_empty_string};
use crate::eval::userfunc::{
    deref_func_name, func_ptr_unref, func_unref, get_func_tv, get_lambda_tv,
};
use crate::eval::vars::{check_vars, get_vim_var_partial};
use crate::eval::{
    Cur, EVAL_EVALUATE, FUNCEXE_INIT, Tv, e_cannot_use_partial_here, e_empty_function_name,
    e_nowhitespace, eval7, get_name_len, is_luafunc, skip_luafunc_name,
};
use crate::ex_eval::aborting;
use crate::memory::{strnequal, xfree, xmemdupz, xstrdup};
use crate::message::emsg;
use crate::message_fmt::c_str;
use crate::os::cshim::gettext;
use crate::strings::vim_strchr;
use crate::types::{
    FAIL, NUL, OK, VAR_FUNC, VAR_PARTIAL, VAR_UNKNOWN, VarLock, Vv, dict_T, evalarg_T, funcexe_T,
    partial_T, size_t, typval_T, typval_vval_union,
};
use ::libc::strlen;

/// A freshly declared typval.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VarLock::Unlocked,
    vval: typval_vval_union { v_number: 0 },
};

/// Is this `evalarg` asking for the expression to actually be evaluated?
///
/// # Safety
/// `evalarg` must be null or valid.
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
    // SAFETY: the caller's promise -- `arg` is the cursor into the
    // expression and `rettv` is the result being built.
    let (cur, mut rv) = unsafe { (Cur::new(arg), Tv::new(rettv)) };
    let evaluate = flags & EVAL_EVALUATE as c_int != 0;
    let mut len = name_len;
    let mut found_var = false;
    if !evaluate {
        // SAFETY: `name` is a name of `len` bytes.
        unsafe { check_vars(name, len as size_t) };
    }
    let mut partial: *mut partial_T = null_mut();
    let (lenp, partialp) = (&raw mut len, &raw mut partial);
    let foundp = &raw mut found_var;
    // SAFETY: `name` is a name of `*lenp` bytes and the three out-parameters
    // are this frame's locals.
    let resolved = unsafe { deref_func_name(name, lenp, partialp, !evaluate, foundp) };
    // `get_func_tv` may re-enter the evaluator, so the name has to
    // outlive whatever `resolved` pointed into.
    // SAFETY: `deref_func_name` left `resolved` naming `len` readable bytes.
    let owned = unsafe { xmemdupz(resolved.cast(), len as size_t) } as *mut c_char;

    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_firstline = cur_win().w_cursor.lnum;
    funcexe.fe_lastline = cur_win().w_cursor.lnum;
    funcexe.fe_evaluate = evaluate;
    funcexe.fe_partial = partial;
    funcexe.fe_basetv = basetv;
    funcexe.fe_found_var = found_var;
    let exe = &raw mut funcexe;
    // SAFETY: `owned` is a NUL-terminated name of `len` bytes and `exe` is
    // this frame's local; the rest are the caller's own arguments.
    let mut ret = unsafe { get_func_tv(owned, len, rettv, arg, evalarg, exe) };
    // SAFETY: `owned` came from `xmemdupz` and nothing else freed it.
    unsafe { xfree(owned.cast()) };

    // While skipping, a name that was never resolved still has to look
    // like a Funcref so the subscript handling can go on.
    if rv.v_type == VAR_UNKNOWN && !evaluate && cur.byte() == b'(' {
        rv.vval.v_string = tv_empty_string.get() as *mut c_char;
        rv.v_type = VAR_FUNC;
    }
    if evaluate && aborting() {
        if ret == OK {
            // SAFETY: the caller's promise -- `rettv` is valid.
            unsafe { tv_clear(rettv) };
        }
        ret = FAIL;
    }
    ret
}

/// Call the value in `rettv` — a name, a Funcref or a partial — with the
/// cursor on the `(`, and leave the result in `rettv`.
///
/// `basetv` is the `expr` of `expr->method()`, passed as the first
/// argument; `lua_funcname` names the `v:lua.` function a partial stands
/// for. Both are null for a plain call.
///
/// # Safety
/// `arg` must point at the cursor into a NUL-terminated expression;
/// `rettv` must be valid; the rest null or valid.
pub(crate) unsafe fn call_func_rettv(
    arg: *mut *mut c_char,
    evalarg: *mut evalarg_T,
    rettv: *mut typval_T,
    evaluate: bool,
    selfdict: *mut dict_T,
    basetv: *mut typval_T,
    lua_funcname: *const c_char,
) -> c_int {
    // SAFETY: the caller's promise -- `arg` is the cursor into the
    // expression and `rettv` holds the callee.
    let (cur, mut rv) = unsafe { (Cur::new(arg), Tv::new(rettv)) };
    let mut pt: *mut partial_T = null_mut();
    // The callee moves out of `rettv` so the call can fill it. It is
    // cleared at the end rather than here: the arguments are evaluated
    // in between and may delete the Funcref they name.
    let mut functv = UNSET_TV;
    let mut is_lua = false;
    let funcname: *const c_char;

    if evaluate {
        functv = *rv;
        rv.v_type = VAR_UNKNOWN;
        if functv.v_type == VAR_PARTIAL {
            // SAFETY: the tag says the union holds a partial, which
            // `is_luafunc` and `partial_name` both take null or valid.
            pt = unsafe { functv.vval.v_partial };
            is_lua = unsafe { is_luafunc(pt) };
            funcname = if is_lua {
                lua_funcname
            } else {
                (unsafe { partial_name(pt) }) as *const c_char
            };
        } else {
            // SAFETY: the tag says the union holds a name, which is null or
            // NUL-terminated.
            funcname = unsafe { functv.vval.v_string };
            if funcname.is_null() || unsafe { *funcname } as c_int == NUL {
                emsg(gettext(e_empty_function_name));
                unsafe { tv_clear(&raw mut functv) };
                return FAIL;
            }
        }
    } else {
        funcname = c"".as_ptr();
    }

    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_firstline = cur_win().w_cursor.lnum;
    funcexe.fe_lastline = cur_win().w_cursor.lnum;
    funcexe.fe_evaluate = evaluate;
    funcexe.fe_partial = pt;
    funcexe.fe_selfdict = selfdict;
    funcexe.fe_basetv = basetv;
    // A `v:lua.` name is not NUL-terminated: it runs to the cursor.
    let namelen = if is_lua {
        // SAFETY: a `v:lua.` name starts inside the expression the cursor
        // is walking, so the two are in the same allocation.
        unsafe { cur.get().offset_from(funcname) as c_int }
    } else {
        -1
    };
    let exe = &raw mut funcexe;
    // SAFETY: `funcname` names the callee, `exe` is this frame's local and
    // the rest are the caller's own.
    let ret = unsafe { get_func_tv(funcname, namelen, rettv, arg, evalarg, exe) };

    if evaluate {
        // SAFETY: `functv` is this frame's own copy of the callee.
        unsafe { tv_clear(&raw mut functv) };
    }
    ret
}

/// `expr->{lambda}()`, with the cursor on the `-`.
///
/// # Safety
/// As `call_func_rettv`.
pub(crate) unsafe fn eval_lambda(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    verbose: bool,
) -> c_int {
    // SAFETY: the caller's promise -- `arg` is the cursor into the
    // expression, `rettv` holds the base and `evalarg` is null or valid.
    let (cur, mut rv) = unsafe { (Cur::new(arg), Tv::new(rettv)) };
    let evaluate = unsafe { evaluating(evalarg) };
    cur.bump(2); // skip over the `->`
    let mut base = *rv;
    rv.v_type = VAR_UNKNOWN;

    if unsafe { get_lambda_tv(arg, rettv, evalarg) } != OK {
        // `base` is not cleared: `get_lambda_tv` failing means the
        // caller still owns it. Upstream's.
        return FAIL;
    }
    let ret = if cur.byte() != b'(' {
        if verbose {
            // SAFETY: the cursor walks a NUL-terminated expression, and
            // both messages take a literal.
            if unsafe { *skipwhite(cur.get()) } == b'(' as c_char {
                emsg(gettext(e_nowhitespace));
            } else {
                let what = c"lambda".as_ptr();
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let what = unsafe { c_str(what) };
                semsg!("E107: Missing parentheses: {what}");
            }
        }
        unsafe { tv_clear(rettv) };
        FAIL
    } else {
        let basep = &raw mut base;
        // SAFETY: as above, with `base` this frame's own.
        unsafe { call_func_rettv(arg, evalarg, rettv, evaluate, null_mut(), basep, null()) }
    };

    if evaluate {
        unsafe { tv_clear(&raw mut base) };
    }
    ret
}

/// `expr->name()`, with the cursor on the `-`.
///
/// # Safety
/// As `call_func_rettv`.
pub(crate) unsafe fn eval_method(
    arg: *mut *mut c_char,
    rettv: *mut typval_T,
    evalarg: *mut evalarg_T,
    verbose: bool,
) -> c_int {
    // SAFETY: the caller's promise -- `arg` is the cursor into the
    // expression, `rettv` holds the base and `evalarg` is null or valid.
    // All three hold for every call below.
    let (cur, mut rv) = unsafe { (Cur::new(arg), Tv::new(rettv)) };
    let evaluate = unsafe { evaluating(evalarg) };
    cur.bump(2); // skip over the `->`
    let mut base = *rv;
    rv.v_type = VAR_UNKNOWN;

    // Locate the method name.
    let mut len: c_int;
    let mut name: *mut c_char = cur.get();
    let mut lua_funcname: *mut c_char = null_mut();
    let mut alias: *mut c_char = null_mut();
    if unsafe { strnequal(name, c"v:lua.".as_ptr(), 6 as size_t) } {
        lua_funcname = unsafe { name.add(6) };
        cur.set(unsafe { skip_luafunc_name(lua_funcname) } as *mut c_char);
        cur.skip(0); // so trailing whitespace is detectable
        len = unsafe { cur.get().offset_from(lua_funcname) } as c_int;
    } else {
        let aliasp = &raw mut alias;
        len = unsafe { get_name_len(cur.raw().cast(), aliasp, evaluate, true) };
        if !alias.is_null() {
            name = alias;
        }
    }

    let mut tofree: *mut c_char = null_mut();
    let mut ret = OK;
    if len <= 0 {
        if verbose {
            if lua_funcname.is_null() {
                emsg(gettext(c"E260: Missing name after ->"));
            } else {
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let name = unsafe { c_str(name) };
                semsg!("E15: Invalid expression: \"{name}\"");
            }
        }
        ret = FAIL;
    } else {
        cur.skip(0);

        // No `(` immediately after, but one further on: this can be
        // "dict.Func()", "list[nr]" and so on. Anything where the `(`
        // is part of the expression itself is not handled.
        let mut paren: *mut c_char = null_mut();
        let indirect = cur.byte() != b'(' && lua_funcname.is_null() && alias.is_null() && {
            paren = unsafe { vim_strchr(cur.get(), '(' as c_int) };
            !paren.is_null()
        };
        if indirect {
            cur.set(name);
            // The `(` is blanked so the callee alone is evaluated, and put
            // back at the end of the branch.
            unsafe { *paren = NUL as c_char };
            let mut callee = UNSET_TV;
            if unsafe { eval7(arg, &raw mut callee, evalarg, false) } == FAIL {
                cur.set(name.wrapping_offset(len as isize));
                ret = FAIL;
            } else if unsafe { *skipwhite(cur.get()) } as c_int != NUL {
                if verbose {
                    let at = cur.get();
                    // SAFETY: a message argument the caller holds as a NUL-terminated string.
                    let at = unsafe { c_str(at) };
                    semsg!("E488: Trailing characters: {at}");
                }
                ret = FAIL;
            } else if callee.v_type == VAR_FUNC && !unsafe { callee.vval.v_string }.is_null() {
                // Take the name over from the typval so `tv_clear`
                // below does not free what is about to be called.
                name = unsafe { callee.vval.v_string };
                callee.vval.v_string = null_mut();
                tofree = name;
                len = unsafe { strlen(name) } as c_int;
            } else if callee.v_type == VAR_PARTIAL && !unsafe { callee.vval.v_partial }.is_null() {
                // SAFETY: the tag says the union holds a live partial.
                let pt = unsafe { Live::new(callee.vval.v_partial) };
                if pt.pt_argc > 0 || !pt.pt_dict.is_null() {
                    if verbose {
                        emsg(gettext(e_cannot_use_partial_here));
                    }
                    ret = FAIL;
                } else {
                    name = unsafe { xstrdup(partial_name(pt.raw())) };
                    tofree = name;
                    // `xstrdup` aborts rather than answering null; the
                    // arm is upstream's and is kept.
                    if name.is_null() {
                        ret = FAIL;
                        name = cur.get();
                    } else {
                        len = unsafe { strlen(name) } as c_int;
                    }
                }
            } else {
                if verbose {
                    // SAFETY: a message argument the caller holds as a NUL-terminated string.
                    let name = unsafe { c_str(name) };
                    semsg!("E1085: Not a callable type: {name}");
                }
                ret = FAIL;
            }
            unsafe { tv_clear(&raw mut callee) };
            unsafe { *paren = b'(' as c_char };
        }

        if ret == OK {
            let basep = &raw mut base;
            if cur.byte() != b'(' {
                if verbose {
                    // SAFETY: a message argument the caller holds as a NUL-terminated string.
                    let name = unsafe { c_str(name) };
                    semsg!("E107: Missing parentheses: {name}");
                }
                ret = FAIL;
            } else if ascii_iswhite(unsafe { *cur.get().offset(-1) } as c_int) {
                if verbose {
                    emsg(gettext(e_nowhitespace));
                }
                ret = FAIL;
            } else if !lua_funcname.is_null() {
                if evaluate {
                    rv.v_type = VAR_PARTIAL;
                    // SAFETY: `v:lua` is a partial the editor owns.
                    let pt = unsafe { get_vim_var_partial(Vv::Lua) };
                    rv.vval.v_partial = pt;
                    unsafe { (*pt).pt_refcount.retain() };
                }
                let lua = lua_funcname;
                ret = unsafe {
                    call_func_rettv(arg, evalarg, rettv, evaluate, null_mut(), basep, lua)
                };
            } else {
                let flags = if evaluate { EVAL_EVALUATE as c_int } else { 0 };
                ret = unsafe { eval_func(arg, evalarg, name, len, rettv, flags, basep) };
            }
        }
    }

    // Clear the Funcref afterwards, so that deleting it while its own
    // arguments are being evaluated is possible (test55).
    if evaluate {
        unsafe { tv_clear(&raw mut base) };
    }
    // SAFETY: both are null or this call's own allocations.
    unsafe { xfree(tofree as *mut c_void) };
    if !alias.is_null() {
        unsafe { xfree(alias as *mut c_void) };
    }
    ret
}

/// The function name a partial stands for: its own, its `ufunc_T`'s, or the
/// empty string.
///
/// # Safety
/// `pt` must be null or valid.
pub(crate) unsafe fn partial_name(pt: *mut partial_T) -> *mut c_char {
    if !pt.is_null() {
        // SAFETY: the caller's promise, and `pt` is not null.
        let pt = unsafe { Live::new(pt) };
        if !pt.pt_name.is_null() {
            return pt.pt_name;
        }
        let func = pt.pt_func;
        if !func.is_null() {
            // SAFETY: `pt_func` is a live `ufunc_T` whose name is inline.
            return unsafe { &raw mut (*func).uf_name } as *mut c_char;
        }
    }
    c"".as_ptr() as *mut c_char
}

/// Release a partial and everything it bound.
///
/// # Safety
/// `pt` must be valid and unreferenced.
unsafe fn partial_free(pt: *mut partial_T) {
    // SAFETY: the caller's promise -- `pt` is a live, unreferenced partial.
    let live = unsafe { Live::new(pt) };
    for i in 0..live.pt_argc {
        // SAFETY: `pt_argv` holds `pt_argc` typvals this partial owns.
        unsafe { tv_clear(live.pt_argv.offset(i as isize)) };
    }
    unsafe { xfree(live.pt_argv as *mut c_void) };
    unsafe { tv_dict_unref(live.pt_dict) };
    if !live.pt_name.is_null() {
        unsafe { func_unref(live.pt_name) };
        unsafe { xfree(live.pt_name as *mut c_void) };
    } else {
        unsafe { func_ptr_unref(live.pt_func) };
    }
    unsafe { xfree(pt as *mut c_void) };
}

/// Drop one reference to a partial, freeing it at zero.
///
/// # Safety
/// `pt` must be null or valid.
pub(crate) unsafe fn partial_unref(pt: *mut partial_T) {
    if pt.is_null() {
        return;
    }
    // SAFETY: the caller's promise, and `pt` is not null.
    if unsafe { (*pt).pt_refcount.release() } <= 0 {
        unsafe { partial_free(pt) };
    }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
