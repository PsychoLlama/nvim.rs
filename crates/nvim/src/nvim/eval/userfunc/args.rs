//! The argument list: parsing it, checking it, filling `a:`.
//!
//! `get_function_args` reads the `(a, b = expr, ...)` of a definition once,
//! at definition time, keeping each default as unevaluated source; the
//! `get_func_arg*` pair reads the arguments of a *call*.  `add_nr_var`
//! seeds the three numeric `a:` entries (`a:0`, `a:firstline`,
//! `a:lastline`) directly into the funccall's embedded fixvar array.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn one_function_arg(
    mut arg: *mut ::core::ffi::c_char,
    mut newargs: *mut garray_T,
    mut skip: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = arg;
        while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
            || *p as ::core::ffi::c_int == '_' as ::core::ffi::c_int
        {
            p = p.offset(1);
        }
        if arg == p
            || *(*__ctype_b_loc()).offset(*arg as uint8_t as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                != 0
            || p.offset_from(arg) == 9 as isize
                && strncmp(
                    arg,
                    b"firstline\0".as_ptr() as *const ::core::ffi::c_char,
                    9 as size_t,
                ) == 0 as ::core::ffi::c_int
            || p.offset_from(arg) == 8 as isize
                && strncmp(
                    arg,
                    b"lastline\0".as_ptr() as *const ::core::ffi::c_char,
                    8 as size_t,
                ) == 0 as ::core::ffi::c_int
        {
            if !skip {
                semsg(
                    gettext(b"E125: Illegal argument: %s\0".as_ptr() as *const ::core::ffi::c_char),
                    arg,
                );
            }
            return arg;
        }
        if !newargs.is_null() {
            ga_grow(newargs, 1 as ::core::ffi::c_int);
            let mut c: uint8_t = *p as uint8_t;
            *p = NUL as ::core::ffi::c_char;
            let mut arg_copy: *mut ::core::ffi::c_char = xstrdup(arg);
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*newargs).ga_len {
                if strcmp(
                    *((*newargs).ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize),
                    arg_copy,
                ) == 0 as ::core::ffi::c_int
                {
                    semsg(
                        gettext(b"E853: Duplicate argument name: %s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        arg_copy,
                    );
                    xfree(arg_copy as *mut ::core::ffi::c_void);
                    return arg;
                }
                i += 1;
            }
            *((*newargs).ga_data as *mut *mut ::core::ffi::c_char)
                .offset((*newargs).ga_len as isize) = arg_copy;
            (*newargs).ga_len += 1;
            *p = c as ::core::ffi::c_char;
        }
        return p;
    }
}

pub(crate) unsafe extern "C" fn get_function_args(
    mut argp: *mut *mut ::core::ffi::c_char,
    mut endchar: ::core::ffi::c_char,
    mut newargs: *mut garray_T,
    mut varargs: *mut ::core::ffi::c_int,
    mut default_args: *mut garray_T,
    mut skip: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut mustend: bool = false_0 != 0;
        let mut arg: *mut ::core::ffi::c_char = *argp;
        let mut p: *mut ::core::ffi::c_char = arg;
        if !newargs.is_null() {
            ga_init(
                newargs,
                ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
                3 as ::core::ffi::c_int,
            );
        }
        if !default_args.is_null() {
            ga_init(
                default_args,
                ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
                3 as ::core::ffi::c_int,
            );
        }
        if !varargs.is_null() {
            *varargs = false_0;
        }
        let mut any_default: bool = false_0 != 0;
        '_err_ret: {
            while *p as ::core::ffi::c_int != endchar as ::core::ffi::c_int {
                if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int
                    && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '.' as ::core::ffi::c_int
                    && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '.' as ::core::ffi::c_int
                {
                    if !varargs.is_null() {
                        *varargs = true_0;
                    }
                    p = p.offset(3 as ::core::ffi::c_int as isize);
                    mustend = true_0 != 0;
                } else {
                    arg = p;
                    p = one_function_arg(p, newargs, skip);
                    if p == arg {
                        break;
                    }
                    if *skipwhite(p) as ::core::ffi::c_int == '=' as ::core::ffi::c_int
                        && !default_args.is_null()
                    {
                        let mut rettv: typval_T = typval_T {
                            v_type: VAR_UNKNOWN,
                            v_lock: VAR_UNLOCKED,
                            vval: typval_vval_union { v_number: 0 },
                        };
                        any_default = true_0 != 0;
                        p = skipwhite(p).offset(1 as ::core::ffi::c_int as isize);
                        p = skipwhite(p);
                        let mut expr: *mut ::core::ffi::c_char = p;
                        if eval1(
                            &raw mut p,
                            &raw mut rettv,
                            ::core::ptr::null_mut::<evalarg_T>(),
                        ) != FAIL
                        {
                            ga_grow(default_args, 1 as ::core::ffi::c_int);
                            while p > expr
                                && ascii_iswhite(*p.offset(-1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0
                            {
                                p = p.offset(-1);
                            }
                            let mut c: uint8_t = *p as uint8_t;
                            *p = NUL as ::core::ffi::c_char;
                            expr = xstrdup(expr);
                            *((*default_args).ga_data as *mut *mut ::core::ffi::c_char)
                                .offset((*default_args).ga_len as isize) = expr;
                            (*default_args).ga_len += 1;
                            *p = c as ::core::ffi::c_char;
                        } else {
                            mustend = true_0 != 0;
                        }
                    } else if any_default {
                        emsg(gettext(
                            b"E989: Non-default argument follows default argument\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ));
                        mustend = true_0 != 0;
                    }
                    if ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                        && *skipwhite(p) as ::core::ffi::c_int == ',' as ::core::ffi::c_int
                    {
                        if !skip {
                            semsg(
                                gettext(E_NO_WHITE_SPACE_ALLOWED_BEFORE_STR_STR.as_ptr()),
                                b",\0".as_ptr() as *const ::core::ffi::c_char,
                                p,
                            );
                            break '_err_ret;
                        } else {
                            p = skipwhite(p);
                        }
                    }
                    if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
                        p = p.offset(1);
                    } else {
                        mustend = true_0 != 0;
                    }
                }
                p = skipwhite(p);
                if !(mustend as ::core::ffi::c_int != 0
                    && *p as ::core::ffi::c_int != endchar as ::core::ffi::c_int)
                {
                    continue;
                }
                if !skip {
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        *argp,
                    );
                }
                break;
            }
            if *p as ::core::ffi::c_int == endchar as ::core::ffi::c_int {
                p = p.offset(1);
                *argp = p;
                return OK;
            }
        }
        if !newargs.is_null() {
            ga_clear_strings(newargs);
        }
        if !default_args.is_null() {
            ga_clear_strings(default_args);
        }
        return FAIL;
    }
}

pub(crate) unsafe extern "C" fn get_func_arguments(
    mut arg: *mut *mut ::core::ffi::c_char,
    evalarg: *mut evalarg_T,
    mut partial_argc: ::core::ffi::c_int,
    mut argvars: *mut typval_T,
    mut argcount: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut argp: *mut ::core::ffi::c_char = *arg;
        let mut ret: ::core::ffi::c_int = OK;
        while *argcount < MAX_FUNC_ARGS as ::core::ffi::c_int - partial_argc {
            argp = skipwhite(argp.offset(1 as ::core::ffi::c_int as isize));
            if *argp as ::core::ffi::c_int == ')' as ::core::ffi::c_int
                || *argp as ::core::ffi::c_int == ',' as ::core::ffi::c_int
                || *argp as ::core::ffi::c_int == NUL
            {
                break;
            }
            if eval1(&raw mut argp, argvars.offset(*argcount as isize), evalarg) == FAIL {
                ret = FAIL;
                break;
            } else {
                *argcount += 1;
                if *argp as ::core::ffi::c_int != ',' as ::core::ffi::c_int {
                    break;
                }
            }
        }
        argp = skipwhite(argp);
        if *argp as ::core::ffi::c_int == ')' as ::core::ffi::c_int {
            argp = argp.offset(1);
        } else {
            ret = FAIL;
        }
        *arg = argp;
        return ret;
    }
}

pub unsafe extern "C" fn get_func_arity(
    mut name: *const ::core::ffi::c_char,
    mut required: *mut ::core::ffi::c_int,
    mut optional: *mut ::core::ffi::c_int,
    mut varargs: *mut bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut argcount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut min_argcount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut fdef: *const EvalFuncDef = find_internal_func(name);
        if !fdef.is_null() {
            argcount = (*fdef).max_argc as ::core::ffi::c_int;
            min_argcount = (*fdef).min_argc as ::core::ffi::c_int;
            *varargs = false_0 != 0;
        } else {
            let mut fname_buf: [::core::ffi::c_char; 41] = [0; 41];
            let mut tofree: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut error: ::core::ffi::c_int = FCERR_NONE as ::core::ffi::c_int;
            let mut fname: *mut ::core::ffi::c_char = fname_trans_sid(
                name,
                &raw mut fname_buf as *mut ::core::ffi::c_char,
                &raw mut tofree,
                &raw mut error,
            );
            let mut ufunc: *mut ufunc_T = ::core::ptr::null_mut::<ufunc_T>();
            if error == FCERR_NONE as ::core::ffi::c_int {
                ufunc = find_func(fname);
            }
            xfree(tofree as *mut ::core::ffi::c_void);
            if ufunc.is_null() {
                return FAIL;
            }
            argcount = (*ufunc).uf_args.ga_len;
            min_argcount = (*ufunc).uf_args.ga_len - (*ufunc).uf_def_args.ga_len;
            *varargs = (*ufunc).uf_varargs != 0;
        }
        *required = min_argcount;
        *optional = argcount - min_argcount;
        return OK;
    }
}

pub(crate) unsafe extern "C" fn add_nr_var(
    mut dp: *mut dict_T,
    mut v: *mut dictitem_T,
    mut name: *mut ::core::ffi::c_char,
    mut nr: varnumber_T,
) {
    unsafe {
        strcpy(&raw mut (*v).di_key as *mut ::core::ffi::c_char, name);
        (*v).di_flags =
            (DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int) as uint8_t;
        hash_add(
            &raw mut (*dp).dv_hashtab,
            &raw mut (*v).di_key as *mut ::core::ffi::c_char,
        );
        (*v).di_tv.v_type = VAR_NUMBER;
        (*v).di_tv.v_lock = VAR_FIXED;
        (*v).di_tv.vval.v_number = nr;
    }
}

pub(crate) unsafe extern "C" fn check_user_func_argcount(
    mut fp: *mut ufunc_T,
    mut argcount: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let regular_args: ::core::ffi::c_int = (*fp).uf_args.ga_len;
        if argcount < regular_args - (*fp).uf_def_args.ga_len {
            return FCERR_TOOFEW as ::core::ffi::c_int;
        } else if (*fp).uf_varargs == 0 && argcount > regular_args {
            return FCERR_TOOMANY as ::core::ffi::c_int;
        }
        return FCERR_UNKNOWN as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn argv_add_base(
    basetv: *mut typval_T,
    argvars: *mut *mut typval_T,
    argcount: *mut ::core::ffi::c_int,
    new_argvars: *mut typval_T,
    argv_base: *mut ::core::ffi::c_int,
) {
    unsafe {
        if !basetv.is_null() {
            memmove(
                new_argvars.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                *argvars as *const ::core::ffi::c_void,
                ::core::mem::size_of::<typval_T>().wrapping_mul(*argcount as size_t),
            );
            *new_argvars.offset(0 as ::core::ffi::c_int as isize) = *basetv;
            *argcount += 1;
            *argvars = new_argvars;
            *argv_base = 1 as ::core::ffi::c_int;
        }
    }
}
