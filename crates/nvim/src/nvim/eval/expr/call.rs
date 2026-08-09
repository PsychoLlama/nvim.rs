//! Calling something: a function name, a method, a lambda or a partial.
//!
//! All three entry points share one shape. The value already in `rettv` is
//! the *callee* (or, for `->`, the base the method is applied to); it is
//! moved into a local, `rettv` is blanked so the call can fill it, and the
//! local is cleared afterwards — after the call, so that a function may
//! delete the Funcref it is being reached through while its own arguments
//! are still being evaluated.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int, c_void};
use core::ptr::{null, null_mut};

use crate::src::nvim::ascii::ascii_iswhite;
use crate::src::nvim::charset::skipwhite;
use crate::src::nvim::eval::typval::{tv_clear, tv_dict_unref};
use crate::src::nvim::eval::userfunc::{func_ptr_unref, func_unref, get_func_tv, get_lambda_tv};
use crate::src::nvim::eval::vars::get_vim_var_partial;
use crate::src::nvim::eval::{
    EVAL_EVALUATE, FAIL, FUNCEXE_INIT, NUL, OK, e_cannot_use_partial_here, e_empty_function_name,
    e_nowhitespace, eval_func, eval7, get_name_len, is_luafunc, skip_luafunc_name,
};
use crate::src::nvim::main::{
    curwin, e_invexpr2, e_missingparen, e_not_callable_type_str, e_trailing_arg,
};
use crate::src::nvim::memory::{strnequal, xfree, xstrdup};
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::{gettext, strlen};
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{
    VAR_FUNC, VAR_PARTIAL, VAR_UNKNOWN, VAR_UNLOCKED, VV_LUA, dict_T, evalarg_T, funcexe_T,
    partial_T, size_t, typval_T, typval_vval_union,
};

/// A freshly declared typval.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VAR_UNLOCKED,
    vval: typval_vval_union { v_number: 0 },
};

/// Is this `evalarg` asking for the expression to actually be evaluated?
///
/// # Safety
/// `evalarg` must be null or valid.
unsafe fn evaluating(evalarg: *const evalarg_T) -> bool {
    !evalarg.is_null() && unsafe { (*evalarg).eval_flags } & EVAL_EVALUATE as c_int != 0
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
    unsafe {
        let mut pt: *mut partial_T = null_mut();
        // The callee moves out of `rettv` so the call can fill it. It is
        // cleared at the end rather than here: the arguments are evaluated
        // in between and may delete the Funcref they name.
        let mut functv = UNSET_TV;
        let mut is_lua = false;
        let funcname: *const c_char;

        if evaluate {
            functv = *rettv;
            (*rettv).v_type = VAR_UNKNOWN;
            if functv.v_type == VAR_PARTIAL {
                pt = functv.vval.v_partial;
                is_lua = is_luafunc(pt);
                funcname = if is_lua {
                    lua_funcname
                } else {
                    partial_name(pt) as *const c_char
                };
            } else {
                funcname = functv.vval.v_string;
                if funcname.is_null() || *funcname as c_int == NUL {
                    emsg(gettext(e_empty_function_name.as_ptr()));
                    tv_clear(&raw mut functv);
                    return FAIL;
                }
            }
        } else {
            funcname = c"".as_ptr();
        }

        let mut funcexe: funcexe_T = FUNCEXE_INIT;
        funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
        funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
        funcexe.fe_evaluate = evaluate;
        funcexe.fe_partial = pt;
        funcexe.fe_selfdict = selfdict;
        funcexe.fe_basetv = basetv;
        // A `v:lua.` name is not NUL-terminated: it runs to the cursor.
        let namelen = if is_lua {
            (*arg).offset_from(funcname) as c_int
        } else {
            -1
        };
        let ret = get_func_tv(funcname, namelen, rettv, arg, evalarg, &raw mut funcexe);

        if evaluate {
            tv_clear(&raw mut functv);
        }
        ret
    }
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
    unsafe {
        let evaluate = evaluating(evalarg);
        *arg = (*arg).add(2); // skip over the `->`
        let mut base = *rettv;
        (*rettv).v_type = VAR_UNKNOWN;

        if get_lambda_tv(arg, rettv, evalarg) != OK {
            // `base` is not cleared: `get_lambda_tv` failing means the
            // caller still owns it. Upstream's.
            return FAIL;
        }
        let ret = if **arg != b'(' as c_char {
            if verbose {
                if *skipwhite(*arg) == b'(' as c_char {
                    emsg(gettext(e_nowhitespace.as_ptr()));
                } else {
                    semsg_c!(gettext(e_missingparen.ptr().cast()), c"lambda".as_ptr());
                }
            }
            tv_clear(rettv);
            FAIL
        } else {
            call_func_rettv(
                arg,
                evalarg,
                rettv,
                evaluate,
                null_mut(),
                &raw mut base,
                null(),
            )
        };

        if evaluate {
            tv_clear(&raw mut base);
        }
        ret
    }
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
    unsafe {
        let evaluate = evaluating(evalarg);
        *arg = (*arg).add(2); // skip over the `->`
        let mut base = *rettv;
        (*rettv).v_type = VAR_UNKNOWN;

        // Locate the method name.
        let mut len: c_int;
        let mut name: *mut c_char = *arg;
        let mut lua_funcname: *mut c_char = null_mut();
        let mut alias: *mut c_char = null_mut();
        if strnequal(name, c"v:lua.".as_ptr(), 6 as size_t) {
            lua_funcname = name.add(6);
            *arg = skip_luafunc_name(lua_funcname) as *mut c_char;
            *arg = skipwhite(*arg); // so trailing whitespace is detectable
            len = (*arg).offset_from(lua_funcname) as c_int;
        } else {
            len = get_name_len(arg as *mut *const c_char, &raw mut alias, evaluate, true);
            if !alias.is_null() {
                name = alias;
            }
        }

        let mut tofree: *mut c_char = null_mut();
        let mut ret = OK;
        if len <= 0 {
            if verbose {
                if lua_funcname.is_null() {
                    emsg(gettext(c"E260: Missing name after ->".as_ptr()));
                } else {
                    semsg_c!(gettext(e_invexpr2.ptr().cast()), name);
                }
            }
            ret = FAIL;
        } else {
            *arg = skipwhite(*arg);

            // No `(` immediately after, but one further on: this can be
            // "dict.Func()", "list[nr]" and so on. Anything where the `(`
            // is part of the expression itself is not handled.
            let mut paren: *mut c_char = null_mut();
            let indirect =
                **arg != b'(' as c_char && lua_funcname.is_null() && alias.is_null() && {
                    paren = vim_strchr(*arg, '(' as c_int);
                    !paren.is_null()
                };
            if indirect {
                *arg = name;
                *paren = NUL as c_char;
                let mut callee = UNSET_TV;
                if eval7(arg, &raw mut callee, evalarg, false) == FAIL {
                    *arg = name.offset(len as isize);
                    ret = FAIL;
                } else if *skipwhite(*arg) as c_int != NUL {
                    if verbose {
                        semsg_c!(gettext(e_trailing_arg.ptr().cast()), *arg);
                    }
                    ret = FAIL;
                } else if callee.v_type == VAR_FUNC && !callee.vval.v_string.is_null() {
                    // Take the name over from the typval so `tv_clear`
                    // below does not free what is about to be called.
                    name = callee.vval.v_string;
                    callee.vval.v_string = null_mut();
                    tofree = name;
                    len = strlen(name) as c_int;
                } else if callee.v_type == VAR_PARTIAL && !callee.vval.v_partial.is_null() {
                    if (*callee.vval.v_partial).pt_argc > 0
                        || !(*callee.vval.v_partial).pt_dict.is_null()
                    {
                        if verbose {
                            emsg(gettext(e_cannot_use_partial_here.as_ptr()));
                        }
                        ret = FAIL;
                    } else {
                        name = xstrdup(partial_name(callee.vval.v_partial));
                        tofree = name;
                        // `xstrdup` aborts rather than answering null; the
                        // arm is upstream's and is kept.
                        if name.is_null() {
                            ret = FAIL;
                            name = *arg;
                        } else {
                            len = strlen(name) as c_int;
                        }
                    }
                } else {
                    if verbose {
                        semsg_c!(gettext(e_not_callable_type_str.ptr().cast()), name);
                    }
                    ret = FAIL;
                }
                tv_clear(&raw mut callee);
                *paren = b'(' as c_char;
            }

            if ret == OK {
                if **arg != b'(' as c_char {
                    if verbose {
                        semsg_c!(gettext(e_missingparen.ptr().cast()), name);
                    }
                    ret = FAIL;
                } else if ascii_iswhite(*(*arg).offset(-1) as c_int) {
                    if verbose {
                        emsg(gettext(e_nowhitespace.as_ptr()));
                    }
                    ret = FAIL;
                } else if !lua_funcname.is_null() {
                    if evaluate {
                        (*rettv).v_type = VAR_PARTIAL;
                        (*rettv).vval.v_partial = get_vim_var_partial(VV_LUA);
                        (*(*rettv).vval.v_partial).pt_refcount += 1;
                    }
                    ret = call_func_rettv(
                        arg,
                        evalarg,
                        rettv,
                        evaluate,
                        null_mut(),
                        &raw mut base,
                        lua_funcname,
                    );
                } else {
                    let flags = if evaluate { EVAL_EVALUATE as c_int } else { 0 };
                    ret = eval_func(arg, evalarg, name, len, rettv, flags, &raw mut base);
                }
            }
        }

        // Clear the Funcref afterwards, so that deleting it while its own
        // arguments are being evaluated is possible (test55).
        if evaluate {
            tv_clear(&raw mut base);
        }
        xfree(tofree as *mut c_void);
        if !alias.is_null() {
            xfree(alias as *mut c_void);
        }
        ret
    }
}

/// The function name a partial stands for: its own, its `ufunc_T`'s, or the
/// empty string.
///
/// # Safety
/// `pt` must be null or valid.
pub unsafe fn partial_name(pt: *mut partial_T) -> *mut c_char {
    unsafe {
        if !pt.is_null() {
            if !(*pt).pt_name.is_null() {
                return (*pt).pt_name;
            }
            if !(*pt).pt_func.is_null() {
                return &raw mut (*(*pt).pt_func).uf_name as *mut c_char;
            }
        }
        c"".as_ptr() as *mut c_char
    }
}

/// Release a partial and everything it bound.
///
/// # Safety
/// `pt` must be valid and unreferenced.
unsafe fn partial_free(pt: *mut partial_T) {
    unsafe {
        for i in 0..(*pt).pt_argc {
            tv_clear((*pt).pt_argv.offset(i as isize));
        }
        xfree((*pt).pt_argv as *mut c_void);
        tv_dict_unref((*pt).pt_dict);
        if !(*pt).pt_name.is_null() {
            func_unref((*pt).pt_name);
            xfree((*pt).pt_name as *mut c_void);
        } else {
            func_ptr_unref((*pt).pt_func);
        }
        xfree(pt as *mut c_void);
    }
}

/// Drop one reference to a partial, freeing it at zero.
///
/// # Safety
/// `pt` must be null or valid.
pub unsafe fn partial_unref(pt: *mut partial_T) {
    unsafe {
        if pt.is_null() {
            return;
        }
        (*pt).pt_refcount -= 1;
        if (*pt).pt_refcount <= 0 {
            partial_free(pt);
        }
    }
}
