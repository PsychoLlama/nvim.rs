//! The argument list: parsing it, checking it, filling `a:`.
//!
//! `get_function_args` reads the `(a, b = expr, ...)` of a definition once,
//! at definition time, keeping each default as unevaluated source; the
//! `get_func_arg*` pair reads the arguments of a *call*.  `add_nr_var`
//! seeds the three numeric `a:` entries (`a:0`, `a:firstline`,
//! `a:lastline`) directly into the funccall's embedded fixvar array.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::message_fmt::c_str;
use crate::semsg;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use super::*;
use crate::eval::Walk;
use crate::types::{Failed, NUL};

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
    // SAFETY: the caller's promise -- `arg` is NUL-terminated and writable,
    // and the walk stops at the first byte that is not an identifier one.
    let mut p = unsafe { Walk::new(arg) };
    while ascii_isident(c_int::from(p.byte())) {
        p.step(1);
    }
    let len = unsafe { p.since(arg) };
    // `isdigit()` is one of the ctype predicates the C standard fixes to
    // ASCII in every locale, so this really is the same test.
    let named = (len == 9 && unsafe { cstr::starts_with(arg, b"firstline") })
        || (len == 8 && unsafe { cstr::starts_with(arg, b"lastline") });
    if arg == p.raw() || unsafe { *arg as u8 }.is_ascii_digit() || named {
        if !skip {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg = unsafe { c_str(arg) };
            semsg!("E125: Illegal argument: {arg}");
        }
        return arg;
    }
    if !newargs.is_null() {
        // SAFETY: the caller's promise -- `newargs` is a `char *` garray,
        // which `ga_grow` has just made room in.
        unsafe { ga_grow(newargs, 1) };
        let c = p.chr();
        p.set(NUL as c_char);
        let arg_copy = unsafe { xstrdup(arg) };
        for &earlier in ga_strings(unsafe { &*newargs }) {
            if unsafe { cstr::eq(earlier, arg_copy) } {
                // SAFETY: `xstrdup` answered a NUL-terminated copy.
                let shown = unsafe { c_str(arg_copy) };
                semsg!("E853: Duplicate argument name: {shown}");
                unsafe { xfree(arg_copy as *mut c_void) };
                // Upstream leaves the name NUL-terminated here; the
                // caller stops on `p == arg` either way.
                return arg;
            }
        }
        unsafe { ga_push_string(newargs, arg_copy) };
        p.set(c);
    }
    p.raw()
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
) -> Result<(), Failed> {
    let mut mustend = false;
    let slot = size_of::<*mut c_char>() as c_int;
    // SAFETY: the caller's promise -- `*argp` is NUL-terminated and
    // writable, and the three out-parameters are null or writable. The walk
    // below never steps past the terminator.
    let mut p = unsafe { Walk::new(*argp) };
    if !newargs.is_null() {
        unsafe { ga_init(newargs, slot, 3) };
    }
    if !default_args.is_null() {
        unsafe { ga_init(default_args, slot, 3) };
    }
    if !varargs.is_null() {
        unsafe { *varargs = 0 };
    }

    // Isolate the arguments: "arg1, arg2, ...)".
    let mut any_default = false;
    let closed = 'parse: {
        while p.chr() != endchar {
            if p.byte() == b'.' && p.at(1) == b'.' && p.at(2) == b'.' {
                if !varargs.is_null() {
                    unsafe { *varargs = 1 };
                }
                p.step(3);
                mustend = true;
            } else {
                let arg = p.raw();
                p = unsafe { Walk::new(one_function_arg(arg, newargs, skip)) };
                if p.raw() == arg {
                    break;
                }
                if unsafe { *skipwhite(p.raw()) } == b'=' as c_char && !default_args.is_null() {
                    let mut rettv = TV_INITIAL_VALUE;
                    any_default = true;
                    let eq = unsafe { skipwhite(p.raw()).add(1) };
                    p = unsafe { Walk::new(skipwhite(eq)) };
                    let mut expr = p.raw();
                    // SAFETY: `&raw mut p` is this frame's own walk, which
                    // `eval1` advances in place.
                    let parsed =
                        unsafe { eval1((&raw mut p).cast(), &raw mut rettv, ptr::null_mut()) };
                    if parsed.is_ok() {
                        unsafe { ga_grow(default_args, 1) };
                        while p.raw() > expr && ascii_iswhite(c_int::from(p.behind(1))) {
                            p.step_back(1);
                        }
                        // The default is kept as source, so it is copied
                        // out from under a temporary terminator.
                        let c = p.chr();
                        p.set(NUL as c_char);
                        expr = unsafe { xstrdup(expr) };
                        unsafe { ga_push_string(default_args, expr) };
                        p.set(c);
                    } else {
                        mustend = true;
                    }
                } else if any_default {
                    let fmt = c"E989: Non-default argument follows default argument";
                    emsg(gettext(fmt));
                    mustend = true;
                }
                let comma_after_white = ascii_iswhite(c_int::from(p.byte()))
                    && unsafe { *skipwhite(p.raw()) } == b',' as c_char;
                if comma_after_white {
                    if !skip {
                        // SAFETY: `p` walks the caller's argument list.
                        let at = unsafe { c_str(p.raw()) };
                        semsg!("E1068: No white space allowed before ',': {at}");
                        break 'parse false;
                    }
                    p = unsafe { Walk::new(skipwhite(p.raw())) };
                }
                if p.byte() == b',' {
                    p.step(1);
                } else {
                    mustend = true;
                }
            }
            p = unsafe { Walk::new(skipwhite(p.raw())) };
            if mustend && p.chr() != endchar {
                if !skip {
                    let at = unsafe { *argp };
                    // SAFETY: a message argument the caller holds as a NUL-terminated string.
                    let at = unsafe { c_str(at) };
                    semsg!("E475: Invalid argument: {at}");
                }
                break;
            }
        }
        p.chr() == endchar
    };
    if closed {
        unsafe { *argp = p.raw().add(1) };
        return Ok(());
    }

    if !newargs.is_null() {
        unsafe { ga_clear_strings(newargs) };
    }
    if !default_args.is_null() {
        unsafe { ga_clear_strings(default_args) };
    }
    Err(Failed)
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
) -> Result<(), Failed> {
    // SAFETY: the caller's promise -- `*arg` is on the `(` of a
    // NUL-terminated argument list, and `argvars` has room past `*argcount`.
    let mut argp = unsafe { Walk::new(*arg) };
    let mut ret = Ok(());
    while unsafe { *argcount } < MAX_FUNC_ARGS - partial_argc {
        // skip the '(' or ','
        argp = unsafe { Walk::new(skipwhite(argp.raw().add(1))) };
        if matches!(argp.byte(), b')' | b',') || argp.byte() == NUL as u8 {
            break;
        }
        let slot = unsafe { argvars.offset(*argcount as isize) };
        // SAFETY: `&raw mut argp` is this frame's own walk, which `eval1`
        // advances in place.
        if unsafe { eval1((&raw mut argp).cast(), slot, evalarg) }.is_err() {
            ret = Err(Failed);
            break;
        }
        unsafe { *argcount += 1 };
        if argp.byte() != b',' {
            break;
        }
    }
    argp = unsafe { Walk::new(skipwhite(argp.raw())) };
    if argp.byte() == b')' {
        argp.step(1);
    } else {
        ret = Err(Failed);
    }
    unsafe { *arg = argp.raw() };
    ret
}

/// How many arguments `name` takes: required, optional, and whether it also
/// takes a `...`.  Answers `Err` when there is no such function.
///
/// # Safety
/// `name` is NUL-terminated and the three out-parameters are writable.
pub unsafe fn get_func_arity(
    name: *const c_char,
    required: *mut c_int,
    optional: *mut c_int,
    varargs: *mut bool,
) -> Result<(), Failed> {
    let argcount;
    let min_argcount;
    // SAFETY: the caller's promise -- `name` is NUL-terminated and the three
    // out-parameters are writable.
    let fdef = unsafe { find_internal_func(name) };
    if !fdef.is_null() {
        // SAFETY: `find_internal_func` answers a live table entry.
        argcount = unsafe { (*fdef).max_argc } as c_int;
        min_argcount = unsafe { (*fdef).min_argc } as c_int;
        unsafe { *varargs = false };
    } else {
        let mut fname_buf: [c_char; FLEN_FIXED as usize + 1] = [0; FLEN_FIXED as usize + 1];
        let mut tofree: *mut c_char = ptr::null_mut();
        let mut error = FCERR_NONE;
        let buf = fname_buf.as_mut_ptr();
        let (freep, errp) = (&raw mut tofree, &raw mut error);
        // SAFETY: `buf` has `FLEN_FIXED + 1` bytes and the two
        // out-parameters are this frame's locals.
        let fname = unsafe { fname_trans_sid(name, buf, freep, errp) };
        let ufunc = if error == FCERR_NONE {
            unsafe { find_func(fname) }
        } else {
            ptr::null_mut()
        };
        unsafe { xfree(tofree as *mut c_void) };
        if ufunc.is_null() {
            return Err(Failed);
        }
        // SAFETY: `find_func` answers a live function.
        let f = unsafe { Uf::new(ufunc) };
        argcount = f.uf_args.ga_len;
        min_argcount = f.uf_args.ga_len - f.uf_def_args.ga_len;
        unsafe { *varargs = f.uf_varargs != 0 };
    }
    unsafe { *required = min_argcount };
    unsafe { *optional = argcount - min_argcount };
    Ok(())
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
    // SAFETY: the caller's promise -- `v` is a `dictitem_T` with room for
    // `name` in its inline key, and `dp` is the dictionary it joins.
    let key = unsafe { (&raw mut (*v).di_key) as *mut c_char };
    unsafe { strcpy(key, name) };
    let mut item = unsafe { Live::new(v) };
    item.di_flags = DI_FLAGS_RO | DI_FLAGS_FIX;
    let _ = unsafe { hash_add(&raw mut (*dp).dv_hashtab, key) };
    item.di_tv.v_type = VAR_NUMBER;
    item.di_tv.v_lock = VarLock::Fixed;
    item.di_tv.vval.v_number = nr;
}

/// Whether `argcount` arguments can be given to `fp`: `FCERR_UNKNOWN` when
/// they can, one of `FCERR_TOOFEW`/`FCERR_TOOMANY` when they cannot.
///
/// # Safety
/// `fp` is a live function.
pub(crate) unsafe fn check_user_func_argcount(fp: *mut ufunc_T, argcount: c_int) -> c_int {
    // SAFETY: the caller's promise -- `fp` is a live function.
    let f = unsafe { Uf::new(fp) };
    let regular_args = f.uf_args.ga_len;
    if argcount < regular_args - f.uf_def_args.ga_len {
        FCERR_TOOFEW
    } else if f.uf_varargs == 0 && argcount > regular_args {
        FCERR_TOOMANY
    } else {
        FCERR_UNKNOWN
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
    if !basetv.is_null() {
        // Method call: base->Method()
        // SAFETY: the caller's promise -- `new_argvars` has room for
        // `*argcount + 1` values and the out-parameters are writable.
        let bytes = unsafe { size_of::<typval_T>().wrapping_mul(*argcount as size_t) };
        let (into, from) = unsafe { (new_argvars.add(1) as *mut c_void, *argvars) };
        unsafe { into.cast::<u8>().copy_from(from.cast(), bytes) };
        unsafe { *new_argvars = *basetv };
        unsafe { *argcount += 1 };
        unsafe { *argvars = new_argvars };
        unsafe { *argv_base = 1 };
    }
}
