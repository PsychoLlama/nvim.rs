//! The argument list: parsing it, checking it, filling `a:`.
//!
//! `get_function_args` reads the `(a, b = expr, ...)` of a definition once,
//! at definition time, keeping each default as unevaluated source; the
//! `get_func_arg*` pair reads the arguments of a *call*.  `add_nr_var`
//! seeds the three numeric `a:` entries (`a:0`, `a:firstline`,
//! `a:lastline`) directly into the funccall's embedded fixvar array.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::semsg;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use super::*;
use crate::eval::typval::DictEntry;
use crate::types::DictKey;
use crate::types::{Failed, NUL};

/// Read one argument name at the cursor and append a copy of it to
/// `newargs`, leaving the cursor after it.
///
/// Answers false, with the cursor where it was, when what is there cannot be
/// one: empty, starting with a digit, a duplicate of an earlier argument, or
/// one of the two names the `a:` scope already gives a meaning.
///
/// # Safety
/// `newargs`, when non-null, is a `char *` garray.
unsafe fn one_function_arg(cursor: &mut Cursor<'_>, newargs: *mut GArray, skip: bool) -> bool {
    let rest = cursor.rest();
    let len = rest
        .iter()
        .take_while(|&&b| ascii_isident(c_int::from(b)))
        .count();
    let name = &rest[..len];
    // `isdigit()` is one of the ctype predicates the C standard fixes to
    // ASCII in every locale, so this really is the same test.
    if len == 0 || name[0].is_ascii_digit() || name == b"firstline" || name == b"lastline" {
        if !skip {
            let arg = msg_bytes(rest);
            semsg!("E125: Illegal argument: {arg}");
        }
        return false;
    }
    if !newargs.is_null() {
        // SAFETY: the caller's promise -- `newargs` is a `char *` garray,
        // which `ga_grow` has just made room in.
        unsafe { ga_grow(newargs, 1) };
        for &earlier in ga_strings(unsafe { &*newargs }) {
            // SAFETY: every entry is a NUL-terminated copy.
            if unsafe { cstr::bytes_at(earlier) } == name {
                let shown = msg_bytes(name);
                semsg!("E853: Duplicate argument name: {shown}");
                return false;
            }
        }
        // SAFETY: as above.
        unsafe { ga_push_string(newargs, XString::from_bytes(name).into_raw()) };
    }
    cursor.bump(len);
    true
}

/// Parse a definition's argument list at the cursor, up to and including
/// `endchar`, and leave the cursor after it; on an error it stays put.
///
/// Fills `newargs` with the names, `default_args` with the *source* of each
/// `= expr` default (evaluated afresh on every call, not here) and `varargs`
/// with whether a `...` was seen.  Any of the three may be null, which is how
/// a caller that only wants to skip the list says so.
///
/// # Safety
/// The three out-parameters are null or writable.
pub(crate) unsafe fn get_function_args(
    cursor: &mut Cursor<'_>,
    endchar: u8,
    newargs: *mut GArray,
    varargs: *mut c_int,
    default_args: *mut GArray,
    skip: bool,
) -> Result<(), Failed> {
    let mut mustend = false;
    let slot = size_of::<*mut c_char>() as c_int;
    let start = cursor.offset();
    let text = cursor.text();
    // SAFETY: the caller's promise -- the three out-parameters are null or
    // writable.
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
        while cursor.byte() != endchar {
            if cursor.rest().starts_with(b"...") {
                if !varargs.is_null() {
                    unsafe { *varargs = 1 };
                }
                cursor.bump(3);
                mustend = true;
            } else {
                // SAFETY: the caller's promise about `newargs`.
                if !unsafe { one_function_arg(cursor, newargs, skip) } {
                    break;
                }
                let mut after = Cursor::new(text);
                after.set_offset(cursor.offset());
                after.skip_white();
                if after.byte() == b'=' && !default_args.is_null() {
                    let mut rettv = TV_INITIAL_VALUE;
                    any_default = true;
                    cursor.skip_white();
                    cursor.bump(1);
                    cursor.skip_white();
                    let expr = cursor.offset();
                    if eval1(cursor, &mut rettv, false).is_ok() {
                        // The default is kept as source, and the walk goes
                        // on from its end: the blanks are read again below.
                        let mut end = cursor.offset();
                        while end > expr && matches!(text[end - 1], b' ' | b'\t') {
                            end -= 1;
                        }
                        cursor.set_offset(end);
                        let copy = XString::from_bytes(&text[expr..end]).into_raw();
                        // SAFETY: the caller's promise about `default_args`.
                        unsafe { ga_grow(default_args, 1) };
                        unsafe { ga_push_string(default_args, copy) };
                    } else {
                        mustend = true;
                    }
                } else if any_default {
                    let fmt = c"E989: Non-default argument follows default argument";
                    emsg(gettext(fmt));
                    mustend = true;
                }
                let white = matches!(cursor.byte(), b' ' | b'\t');
                let mut after = Cursor::new(text);
                after.set_offset(cursor.offset());
                after.skip_white();
                if white && after.byte() == b',' {
                    if !skip {
                        let at = msg_bytes(cursor.rest());
                        semsg!("E1068: No white space allowed before ',': {at}");
                        break 'parse false;
                    }
                    cursor.skip_white();
                }
                if cursor.byte() == b',' {
                    cursor.bump(1);
                } else {
                    mustend = true;
                }
            }
            cursor.skip_white();
            if mustend && cursor.byte() != endchar {
                if !skip {
                    let at = msg_bytes(&text[start..]);
                    semsg!("E475: Invalid argument: {at}");
                }
                break;
            }
        }
        cursor.byte() == endchar
    };
    if closed {
        cursor.bump(1);
        return Ok(());
    }
    cursor.set_offset(start);

    if !newargs.is_null() {
        unsafe { ga_clear_strings(newargs) };
    }
    if !default_args.is_null() {
        unsafe { ga_clear_strings(default_args) };
    }
    Err(Failed)
}

/// Evaluate the arguments of a call, from the `(` at the cursor to its `)`.
///
/// Stops at `MAX_FUNC_ARGS` less whatever a partial has already bound.
pub(crate) fn get_func_arguments(
    cursor: &mut Cursor<'_>,
    evaluate: bool,
    partial_argc: c_int,
    args: &mut [TypVal],
    argcount: &mut usize,
) -> Result<(), Failed> {
    let mut ret = Ok(());
    let room = usize::try_from(MAX_FUNC_ARGS as c_int - partial_argc).unwrap_or(0);
    while *argcount < room {
        // skip the '(' or ','
        cursor.bump(1);
        cursor.skip_white();
        if matches!(cursor.byte(), b')' | b',') || cursor.byte() == NUL as u8 {
            break;
        }
        if eval1(cursor, &mut args[*argcount], evaluate).is_err() {
            ret = Err(Failed);
            break;
        }
        *argcount += 1;
        if cursor.byte() != b',' {
            break;
        }
    }
    cursor.skip_white();
    if cursor.byte() == b')' {
        cursor.bump(1);
    } else {
        ret = Err(Failed);
    }
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
        let arity = unsafe { (*fdef).arity };
        // An open-ended builtin takes as many as the evaluator will pass.
        argcount = arity.max().map_or(MAX_FUNC_ARGS, c_int::from);
        min_argcount = c_int::from(arity.min());
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
/// `v` is a `DictItem` whose key member has room for `name`, and `dp` is
/// the dictionary it is being linked into.  `v` must outlive `dp`.
pub(crate) unsafe fn add_nr_var(dp: *mut Dict, v: *mut DictItem, name: *mut c_char, nr: VarNumber) {
    // SAFETY: the caller's promise -- `v` is a `DictItem` with room for
    // `name` in its inline key, and `dp` is the dictionary it joins.
    // SAFETY: the caller's NUL-terminated name.
    unsafe { (*v).di_key = DictKey::new(cstr::bytes_at(name)) };
    let mut item = unsafe { Live::new(v) };
    item.di_flags = DI_FLAGS_RO | DI_FLAGS_FIX;
    let _ = unsafe { hash_add(&raw mut (*dp).dv_hashtab, DictEntry::new(v)) };
    item.di_lock = VarLock::Fixed;
    item.di_tv.write_number(nr);
}

/// Whether `argcount` arguments can be given to `func`: `FCERR_UNKNOWN` when
/// they can, one of `FCERR_TOOFEW`/`FCERR_TOOMANY` when they cannot.
///
/// # Safety
/// `func` is a live function.
pub(crate) unsafe fn check_user_func_argcount(func: *mut UserFunc, argcount: c_int) -> c_int {
    // SAFETY: the caller's promise -- `func` is a live function.
    let f = unsafe { Uf::new(func) };
    let regular_args = f.uf_args.ga_len;
    if argcount < regular_args - f.uf_def_args.ga_len {
        FCERR_TOOFEW
    } else if f.uf_varargs == 0 && argcount > regular_args {
        FCERR_TOOMANY
    } else {
        FCERR_UNKNOWN
    }
}
