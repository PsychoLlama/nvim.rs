//! The argument list: parsing it, checking it, filling `a:`.
//!
//! `get_function_args` reads the `(a, b = expr, ...)` of a definition once,
//! at definition time, keeping each default as unevaluated source; the
//! `get_func_arg*` pair reads the arguments of a *call*.  `add_nr_var`
//! seeds the three numeric `a:` entries (`a:0`, `a:firstline`,
//! `a:lastline`) directly into the funccall's embedded fixvar array.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use super::*;
use crate::types::{FAIL, NUL, OK};

/// Read one argument name at `arg` and append a copy of it to `newargs`.
///
/// Answers the end of the name, or `arg` itself when what is there cannot be
/// one: empty, starting with a digit, a duplicate of an earlier argument, or
/// one of the two names the `a:` scope already gives a meaning.
///
/// # Safety
/// `arg` is a NUL-terminated, *writable* string -- the name is terminated in
/// place while it is copied.  `newargs`, when non-null, is a `char *` garray.
unsafe fn one_function_arg(arg: *mut c_char, newargs: *mut garray_T, skip: bool) -> *mut c_char {
    unsafe {
        let mut p = arg;
        while ascii_isident(*p as c_int) {
            p = p.add(1);
        }
        let len = p.offset_from(arg);
        // `isdigit()` is one of the ctype predicates the C standard fixes to
        // ASCII in every locale, so this really is the same test.
        if arg == p
            || (*arg as u8).is_ascii_digit()
            || (len == 9 && strncmp(arg, c"firstline".as_ptr(), 9) == 0)
            || (len == 8 && strncmp(arg, c"lastline".as_ptr(), 8) == 0)
        {
            if !skip {
                semsg_c!(gettext(c"E125: Illegal argument: %s".as_ptr()), arg);
            }
            return arg;
        }
        if !newargs.is_null() {
            ga_grow(newargs, 1);
            let c = *p;
            *p = NUL as c_char;
            let arg_copy = xstrdup(arg);
            for &earlier in ga_strings(&*newargs) {
                if strcmp(earlier, arg_copy) == 0 {
                    semsg_c!(
                        gettext(c"E853: Duplicate argument name: %s".as_ptr()),
                        arg_copy,
                    );
                    xfree(arg_copy as *mut c_void);
                    // Upstream leaves the name NUL-terminated here; the
                    // caller stops on `p == arg` either way.
                    return arg;
                }
            }
            ga_push_string(newargs, arg_copy);
            *p = c;
        }
        p
    }
}

/// Parse a definition's argument list, up to and including `endchar`.
///
/// Fills `newargs` with the names, `default_args` with the *source* of each
/// `= expr` default (evaluated afresh on every call, not here) and `varargs`
/// with whether a `...` was seen.  Any of the three may be null, which is how
/// a caller that only wants to skip the list says so.
///
/// # Safety
/// `*argp` is a NUL-terminated, writable string; the three out-parameters are
/// null or writable.
pub(crate) unsafe fn get_function_args(
    argp: *mut *mut c_char,
    endchar: c_char,
    newargs: *mut garray_T,
    varargs: *mut c_int,
    default_args: *mut garray_T,
    skip: bool,
) -> c_int {
    unsafe {
        let mut mustend = false;
        let mut p = *argp;
        if !newargs.is_null() {
            ga_init(newargs, size_of::<*mut c_char>() as c_int, 3);
        }
        if !default_args.is_null() {
            ga_init(default_args, size_of::<*mut c_char>() as c_int, 3);
        }
        if !varargs.is_null() {
            *varargs = 0;
        }

        // Isolate the arguments: "arg1, arg2, ...)".
        let mut any_default = false;
        let closed = 'parse: {
            while *p != endchar {
                if *p == b'.' as c_char
                    && *p.add(1) == b'.' as c_char
                    && *p.add(2) == b'.' as c_char
                {
                    if !varargs.is_null() {
                        *varargs = 1;
                    }
                    p = p.add(3);
                    mustend = true;
                } else {
                    let arg = p;
                    p = one_function_arg(p, newargs, skip);
                    if p == arg {
                        break;
                    }
                    if *skipwhite(p) == b'=' as c_char && !default_args.is_null() {
                        let mut rettv = TV_INITIAL_VALUE;
                        any_default = true;
                        p = skipwhite(skipwhite(p).add(1));
                        let mut expr = p;
                        if eval1(&raw mut p, &raw mut rettv, ptr::null_mut()) != FAIL {
                            ga_grow(default_args, 1);
                            while p > expr && ascii_iswhite(*p.sub(1) as c_int) {
                                p = p.sub(1);
                            }
                            // The default is kept as source, so it is copied
                            // out from under a temporary terminator.
                            let c = *p;
                            *p = NUL as c_char;
                            expr = xstrdup(expr);
                            ga_push_string(default_args, expr);
                            *p = c;
                        } else {
                            mustend = true;
                        }
                    } else if any_default {
                        emsg(gettext(
                            c"E989: Non-default argument follows default argument".as_ptr(),
                        ));
                        mustend = true;
                    }
                    if ascii_iswhite(*p as c_int) && *skipwhite(p) == b',' as c_char {
                        if !skip {
                            semsg_c!(
                                gettext(E_NO_WHITE_SPACE_ALLOWED_BEFORE_STR_STR.as_ptr()),
                                c",".as_ptr(),
                                p,
                            );
                            break 'parse false;
                        }
                        p = skipwhite(p);
                    }
                    if *p == b',' as c_char {
                        p = p.add(1);
                    } else {
                        mustend = true;
                    }
                }
                p = skipwhite(p);
                if mustend && *p != endchar {
                    if !skip {
                        semsg_c!(gettext(&raw const e_invarg2 as *const c_char), *argp);
                    }
                    break;
                }
            }
            *p == endchar
        };
        if closed {
            *argp = p.add(1);
            return OK;
        }

        if !newargs.is_null() {
            ga_clear_strings(newargs);
        }
        if !default_args.is_null() {
            ga_clear_strings(default_args);
        }
        FAIL
    }
}

/// Evaluate the arguments of a call, from the `(` at `*arg` to its `)`.
///
/// Stops at `MAX_FUNC_ARGS` less whatever a partial has already bound.
///
/// # Safety
/// `*arg` points at the `(`; `argvars` has room for `MAX_FUNC_ARGS` values
/// past `*argcount`.
pub(crate) unsafe fn get_func_arguments(
    arg: *mut *mut c_char,
    evalarg: *mut evalarg_T,
    partial_argc: c_int,
    argvars: *mut typval_T,
    argcount: *mut c_int,
) -> c_int {
    unsafe {
        let mut argp = *arg;
        let mut ret = OK;
        while *argcount < MAX_FUNC_ARGS - partial_argc {
            argp = skipwhite(argp.add(1)); // skip the '(' or ','
            if *argp == b')' as c_char || *argp == b',' as c_char || *argp == NUL as c_char {
                break;
            }
            if eval1(&raw mut argp, argvars.offset(*argcount as isize), evalarg) == FAIL {
                ret = FAIL;
                break;
            }
            *argcount += 1;
            if *argp != b',' as c_char {
                break;
            }
        }
        argp = skipwhite(argp);
        if *argp == b')' as c_char {
            argp = argp.add(1);
        } else {
            ret = FAIL;
        }
        *arg = argp;
        ret
    }
}

/// How many arguments `name` takes: required, optional, and whether it also
/// takes a `...`.  Answers `FAIL` when there is no such function.
///
/// # Safety
/// `name` is NUL-terminated and the three out-parameters are writable.
pub unsafe fn get_func_arity(
    name: *const c_char,
    required: *mut c_int,
    optional: *mut c_int,
    varargs: *mut bool,
) -> c_int {
    unsafe {
        let argcount;
        let min_argcount;
        let fdef = find_internal_func(name);
        if !fdef.is_null() {
            argcount = (*fdef).max_argc as c_int;
            min_argcount = (*fdef).min_argc as c_int;
            *varargs = false;
        } else {
            let mut fname_buf: [c_char; FLEN_FIXED as usize + 1] = [0; FLEN_FIXED as usize + 1];
            let mut tofree: *mut c_char = ptr::null_mut();
            let mut error = FCERR_NONE;
            let fname = fname_trans_sid(
                name,
                fname_buf.as_mut_ptr(),
                &raw mut tofree,
                &raw mut error,
            );
            let ufunc = if error == FCERR_NONE {
                find_func(fname)
            } else {
                ptr::null_mut()
            };
            xfree(tofree as *mut c_void);
            if ufunc.is_null() {
                return FAIL;
            }
            argcount = (*ufunc).uf_args.ga_len;
            min_argcount = (*ufunc).uf_args.ga_len - (*ufunc).uf_def_args.ga_len;
            *varargs = (*ufunc).uf_varargs != 0;
        }
        *required = min_argcount;
        *optional = argcount - min_argcount;
        OK
    }
}

/// Add one of `a:`'s fixed numbers, into a slot of the funccall's own
/// `fc_fixvar` array rather than an allocation.
///
/// # Safety
/// `v` is a `dictitem_T` whose key member has room for `name`, and `dp` is
/// the dictionary it is being linked into.  `v` must outlive `dp`.
pub(crate) unsafe fn add_nr_var(
    dp: *mut dict_T,
    v: *mut dictitem_T,
    name: *mut c_char,
    nr: varnumber_T,
) {
    unsafe {
        strcpy((&raw mut (*v).di_key) as *mut c_char, name);
        (*v).di_flags = DI_FLAGS_RO | DI_FLAGS_FIX;
        hash_add(
            &raw mut (*dp).dv_hashtab,
            (&raw mut (*v).di_key) as *mut c_char,
        );
        (*v).di_tv.v_type = VAR_NUMBER;
        (*v).di_tv.v_lock = VAR_FIXED;
        (*v).di_tv.vval.v_number = nr;
    }
}

/// Whether `argcount` arguments can be given to `fp`: `FCERR_UNKNOWN` when
/// they can, one of `FCERR_TOOFEW`/`FCERR_TOOMANY` when they cannot.
///
/// # Safety
/// `fp` is a live function.
pub(crate) unsafe fn check_user_func_argcount(fp: *mut ufunc_T, argcount: c_int) -> c_int {
    unsafe {
        let regular_args = (*fp).uf_args.ga_len;
        if argcount < regular_args - (*fp).uf_def_args.ga_len {
            FCERR_TOOFEW
        } else if (*fp).uf_varargs == 0 && argcount > regular_args {
            FCERR_TOOMANY
        } else {
            FCERR_UNKNOWN
        }
    }
}

/// Put `basetv` in front of the argument list, which is what makes
/// `base->Method(a)` a call of `Method(base, a)`.
///
/// The arguments move into `new_argvars`, the caller's own array, because the
/// one they came from has no room at the front.
///
/// # Safety
/// `new_argvars` has room for `*argcount + 1` values, and the four
/// out-parameters are writable.
pub(crate) unsafe fn argv_add_base(
    basetv: *mut typval_T,
    argvars: *mut *mut typval_T,
    argcount: *mut c_int,
    new_argvars: *mut typval_T,
    argv_base: *mut c_int,
) {
    unsafe {
        if !basetv.is_null() {
            // Method call: base->Method()
            memmove(
                new_argvars.add(1) as *mut c_void,
                *argvars as *const c_void,
                size_of::<typval_T>().wrapping_mul(*argcount as size_t),
            );
            *new_argvars = *basetv;
            *argcount += 1;
            *argvars = new_argvars;
            *argv_base = 1;
        }
    }
}
