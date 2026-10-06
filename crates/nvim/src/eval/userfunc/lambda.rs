//! Lambdas, closures and partials -- the anonymous half.
//!
//! `get_lambda_tv` parses `{x -> expr}` into a real `UserFunc` with a
//! generated `<lambda>N` name and, if it captured anything, a reference to
//! the funccall it was made in (`register_closure`).  `make_partial` is the
//! other way a callable carries state: a bound dictionary, bound arguments,
//! or both.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::eval::Parsed;
use crate::eval::typval::PartialRef;
use crate::memory::handoff::owned_cstr;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use crate::snprintf;
use crate::strings::find_bytes;
use core::ffi::{c_char, c_int, c_void};
use core::mem::offset_of;
use core::ptr;

use super::*;
use crate::types::{Failed, Refcount};

/// Give `func` the funccall that is running as its scope, so that the locals it
/// closed over stay alive for as long as it does.
///
/// # Safety
/// `func` is a live function and a funccall is running.
pub(crate) unsafe fn register_closure(func: *mut UserFunc) {
    // SAFETY: the caller's promise -- `func` is a live function.
    let mut f = unsafe { Uf::new(func) };
    if f.uf_scoped == current_fc() {
        return; // no change
    }
    unsafe { funccal_unref(f.uf_scoped, func, false) };
    let fc = current_fc();
    f.uf_scoped = fc;
    unsafe { (*fc).fc_refcount.retain() };
    unsafe { ga_grow(&raw mut (*fc).fc_ufuncs, 1) };
    let ufuncs = unsafe { &raw mut (*fc).fc_ufuncs };
    unsafe { *((*ufuncs).ga_data as *mut *mut UserFunc).offset((*ufuncs).ga_len as isize) = func };
    unsafe { (*ufuncs).ga_len += 1 };
}

/// `"<lambda>"` plus `NUMBUFLEN`, the widest a `VarNumber` prints.
const LAMBDA_NAME_LEN: usize = 8 + 65;

/// The name of the next lambda, rendered through `into` — the caller's
/// scratch buffer, so that two names can be alive at once. Upstream answers
/// one static buffer.
fn get_lambda_name(into: &mut [c_char; LAMBDA_NAME_LEN]) -> String_0 {
    static lambda_no: GlobalCell<c_int> = GlobalCell::new(0);
    lambda_no.set(lambda_no.get() + 1);
    let buf = into.as_mut_ptr();
    let nr = lambda_no.get();
    // SAFETY: `buf` is the caller's array of `LAMBDA_NAME_LEN` bytes.
    let n = unsafe { snprintf!(buf, LAMBDA_NAME_LEN, c"<lambda>%d".as_ptr(), nr) };
    let len = if n < 1 {
        0
    } else {
        n.min(LAMBDA_NAME_LEN as c_int - 1) as size_t
    };
    // SAFETY: `snprintf` wrote `len` bytes of the caller's array, which the
    // answer copies.
    unsafe { String_0::from_raw_bytes(buf, len) }
}

/// Allocate a `UserFunc` for a function called `name`, whose name lives in the
/// flexible member at the end of the allocation.
///
/// # Safety
/// `name` has `namelen` readable bytes.
pub(crate) unsafe fn alloc_ufunc(name: *const c_char, namelen: size_t) -> *mut UserFunc {
    let fp = unsafe { xcalloc(1, offset_of!(UserFunc, uf_name) + namelen + 1) } as *mut UserFunc;
    // SAFETY: the allocation ends in `namelen + 1` bytes for the name.
    let into = uf_name_ptr(fp) as *mut c_void;
    unsafe { xmemcpyz(into, name as *const c_void, namelen) };
    unsafe { (*fp).uf_namelen = namelen };

    if unsafe { *name } as u8 as c_int == K_SPECIAL {
        // A script-local name is stored mangled; keep the printable
        // "<SNR>123_name" beside it.
        let len = namelen + 3;
        // SAFETY: `fp` is the allocation just made, whose inline name has
        // `namelen + 1` bytes; the printable form gets three more.
        let into = unsafe { xmalloc(len) } as *mut c_char;
        unsafe { (*fp).uf_name_exp = into };
        let tail = unsafe { uf_name_ptr(fp).add(3) };
        unsafe { snprintf!(into, len, c"<SNR>%s".as_ptr(), tail) };
    }
    fp
}

/// Parse a lambda expression at the cursor into a partial in `result`.
///
/// Answers [`Parsed::NotThis`] when it is a dictionary or a `{expr}` rather
/// than a lambda -- which is decided by whether an `->` follows a legal
/// argument list.
pub(crate) fn get_lambda_tv(
    cursor: &mut Cursor<'_>,
    result: &mut TypVal,
    evaluate: bool,
) -> Result<Parsed, Failed> {
    let mut lambda_buf = [0 as c_char; LAMBDA_NAME_LEN];
    let mut newargs = GArray::EMPTY;
    let mut varargs = 0;
    // The enclosing lambda's capture flag, put back when this one is done.
    // Only an evaluating lambda starts its own: a skipped one leaves the
    // flag alone, so what it reads still counts for the enclosing lambda.
    let enclosing_uses_locals = LAMBDA_USES_LOCALS.get();
    let mut uses_locals = false;
    let text = cursor.text();

    // First, check whether this is a lambda expression at all: an "->"
    // must follow a well-formed argument list.
    let mut look = Cursor::new(text);
    look.set_offset(cursor.offset() + 1);
    look.skip_white();
    let none = ptr::null_mut();
    // SAFETY: nothing is asked back.
    let looks_like =
        unsafe { get_function_args(&mut look, b'-', none, ptr::null_mut(), none, true) };
    if looks_like.is_err() || look.byte() != b'>' {
        return Ok(Parsed::NotThis);
    }

    // Neither `fp` nor `pt` escapes the arm that builds them, which is
    // why upstream's `assert(fp == NULL)` at its error label holds.
    let parsed = 'errret: {
        // Parse the arguments again, this time keeping them.
        let pnewargs = if evaluate {
            &raw mut newargs
        } else {
            ptr::null_mut()
        };
        cursor.bump(1);
        cursor.skip_white();
        // SAFETY: `newargs` and `varargs` are this frame's locals.
        let read =
            unsafe { get_function_args(cursor, b'-', pnewargs, &raw mut varargs, none, false) };
        if read.is_err() || cursor.byte() != b'>' {
            break 'errret false;
        }

        // Set up a flag for checking local variables and arguments.
        if evaluate {
            LAMBDA_USES_LOCALS.set(Some(false));
        }

        // Get the start and the end of the expression.
        cursor.bump(1);
        cursor.skip_white();
        let start = cursor.offset();
        let mut skipped = TV_INITIAL_VALUE;
        let ret = eval1(cursor, &mut skipped, false);
        let end = cursor.offset();
        if ret.is_err() {
            break 'errret false;
        }
        if evaluate {
            uses_locals = LAMBDA_USES_LOCALS.get() == Some(true);
        }

        cursor.skip_white();
        if cursor.byte() != b'}' {
            let rest = msg_bytes(cursor.rest());
            semsg!("E451: Expected }}: {rest}");
            break 'errret false;
        }
        cursor.bump(1);

        if evaluate {
            let mut flags = FuncFlags::NONE;
            let name = get_lambda_name(&mut lambda_buf);
            let fp = unsafe { alloc_ufunc(name.data(), name.len()) };
            let pt = unsafe { xcalloc(1, size_of::<Partial>()) } as *mut Partial;
            // SAFETY: both are this call's own allocations.
            let (mut f, mut part) = unsafe { (Uf::new(fp), Live::new(pt)) };

            // The body is the expression with "return " in front of it.
            let body = &text[start..end];
            let mut line = Vec::with_capacity(b"return ".len() + body.len());
            line.extend_from_slice(b"return ");
            line.extend_from_slice(body);
            if find_bytes(body, b"a:").is_none() {
                // No a: variables are used for sure.
                flags |= FuncFlags::NOARGS;
            }
            let mut newlines = GArray::EMPTY;
            unsafe { ga_init(&raw mut newlines, size_of::<*mut c_char>() as c_int, 1) };
            unsafe { ga_grow(&raw mut newlines, 1) };
            unsafe { *(newlines.ga_data as *mut *mut c_char) = owned_cstr(line) };
            newlines.ga_len = 1;

            f.uf_refcount = Refcount::ONE;
            let _ = unsafe { func_table().add(uf_name_ptr(fp)) };
            f.uf_args = newargs;
            let slot = size_of::<*mut c_char>() as c_int;
            unsafe { ga_init(&raw mut (*fp).uf_def_args, slot, 1) };
            f.uf_lines = newlines;
            if !current_fc().is_null() && uses_locals {
                flags |= FuncFlags::CLOSURE;
                unsafe { register_closure(fp) };
            } else {
                f.uf_scoped = ptr::null_mut();
            }

            if prof_def_func() {
                unsafe { func_do_profile(fp) };
            }
            if sandbox.get() != 0 {
                flags |= FuncFlags::SANDBOX;
            }
            f.uf_varargs = 1;
            f.uf_flags = flags;
            f.uf_calls = 0;
            f.uf_script_ctx = current_sctx.get();
            f.uf_script_ctx.sc_lnum += sourcing_lnum() - newlines.ga_len as LineNr;

            part.pt_func = fp;
            part.pt_refcount = Refcount::ONE;
            // SAFETY: the count just set is the one the slot takes over.
            result.write_partial(unsafe { PartialRef::owning(pt) });
        }
        true
    };

    if !parsed {
        unsafe { ga_clear_strings(&raw mut newargs) };
    }
    if evaluate {
        LAMBDA_USES_LOCALS.set(enclosing_uses_locals);
    }
    if parsed {
        Ok(Parsed::Done)
    } else {
        Err(Failed)
    }
}

/// Bind `selfdict` to the Funcref in `result`: `dict.Func` read out of
/// `dict`. Not for a partial that was bound explicitly (`pt_auto` clear).
pub(crate) fn set_selfdict(result: &mut TypVal, selfdict: &mut Dict) {
    if let Some(pt) = result.as_partial()
        && !pt.is_null()
    {
        // SAFETY: a partial value holds a reference to a live partial.
        let pt = unsafe { &*pt };
        if !pt.pt_auto && !pt.pt_dict.is_null() {
            return;
        }
    }
    // SAFETY: `result` holds the funcref just read out of `selfdict`, which
    // the borrow keeps live while the partial takes its own reference.
    unsafe { make_partial(selfdict, result) };
}

/// Turn `dict.Func` into a partial that binds `selfdict`, when `Func` was
/// declared with the `dict` attribute.
///
/// # Safety
/// `result` holds the funcref just read and `selfdict` the dictionary it came
/// out of.
pub unsafe fn make_partial(selfdict: *mut Dict, result: &mut TypVal) {
    // SAFETY: the caller's promise -- `result` holds the funcref just read.
    let mut rv = unsafe { Tv::new(result) };
    let mut fp: *mut UserFunc = ptr::null_mut();
    let mut fname_buf: [c_char; FLEN_FIXED as usize + 1] = [0; FLEN_FIXED as usize + 1];
    let mut error = 0;

    // SAFETY: the tag says which union member holds the callable, and a
    // partial in it is null or live.
    let held = rv.partial_or_null();
    if rv.v_type() == VAR_PARTIAL && !held.is_null() && !unsafe { (*held).pt_func }.is_null() {
        fp = unsafe { (*held).pt_func };
    } else {
        let mut fname = if rv.v_type() == VAR_FUNC || rv.v_type() == VAR_STRING {
            rv.string_or_func_name()
        } else if held.is_null() {
            ptr::null_mut()
        } else {
            unsafe { (*held).pt_name }
        };
        if fname.is_null() {
            // There is no point binding a dict to a NULL function, just
            // create a function reference.
            rv.write_func_name(ptr::null_mut());
        } else {
            // Translate "s:func" to the stored function name.
            let mut tofree: *mut c_char = ptr::null_mut();
            let buf = fname_buf.as_mut_ptr();
            let (freep, errp) = (&raw mut tofree, &raw mut error);
            fname = unsafe { fname_trans_sid(fname, buf, freep, errp) };
            fp = unsafe { find_func(fname) };
            unsafe { xfree(tofree as *mut c_void) };
        }
    }

    if fp.is_null() || !unsafe { (*fp).uf_flags }.has(FuncFlags::DICT) {
        return;
    }
    let pt = unsafe { xcalloc(1, size_of::<Partial>()) } as *mut Partial;
    // SAFETY: a fresh partial of this call's own, and `selfdict` is the
    // dictionary the funcref came out of.
    let mut part = unsafe { Live::new(pt) };
    part.pt_refcount = Refcount::ONE;
    part.pt_dict = selfdict;
    unsafe { (*selfdict).dv_refcount.retain() };
    part.pt_auto = true;
    if rv.v_type() == VAR_FUNC || rv.v_type() == VAR_STRING {
        // Just a function: take over the function name and use selfdict.
        part.pt_name = rv.string_or_func_name();
    } else {
        // Partial: copy the function name, use selfdict and copy the
        // arguments.  Neither can be taken over, because the partial may
        // be referenced elsewhere.
        // SAFETY: the kind says the value holds a live partial.
        let ret_pt = unsafe { Live::new(rv.partial_or_null()) };
        if !ret_pt.pt_name.is_null() {
            part.pt_name = unsafe { xstrdup(ret_pt.pt_name) };
            unsafe { func_ref(part.pt_name) };
        } else {
            part.pt_func = ret_pt.pt_func;
            unsafe { func_ptr_ref(part.pt_func) };
        }
        if ret_pt.pt_argc > 0 {
            let arg_size = size_of::<TypVal>().wrapping_mul(ret_pt.pt_argc as size_t);
            part.pt_argv = unsafe { xmalloc(arg_size) } as *mut TypVal;
            part.pt_argc = ret_pt.pt_argc;
            let (from, into) = (ret_pt.pt_argv, part.pt_argv);
            for i in 0..part.pt_argc {
                unsafe { tv_copy(&*from.offset(i as isize), &mut *into.offset(i as isize)) };
            }
        }
        unsafe { partial_unref(ret_pt.raw()) };
    }
    // SAFETY: the count set above is the one the slot takes over.
    rv.write_partial(unsafe { PartialRef::owning(pt) });
}

/// Wrap a Lua reference in a `UserFunc`, so that Vimscript can call it by
/// name.  Answers that name.
///
/// # Safety
/// `ref_0` is a live Lua reference the new function takes over.
pub unsafe fn register_luafunc(ref_0: LuaRef) -> *mut c_char {
    let mut lambda_buf = [0 as c_char; LAMBDA_NAME_LEN];
    let name = get_lambda_name(&mut lambda_buf);
    let fp = unsafe { alloc_ufunc(name.data(), name.len()) };
    // SAFETY: `fp` is the allocation just made.
    let mut f = unsafe { Uf::new(fp) };
    f.uf_refcount = Refcount::ONE;
    f.uf_varargs = 1;
    f.uf_flags = FuncFlags::LUAREF;
    f.uf_calls = 0;
    f.uf_script_ctx = current_sctx.get();
    f.uf_luaref = ref_0;

    let _ = unsafe { func_table().add(uf_name_ptr(fp)) };
    uf_name_ptr(fp)
}
