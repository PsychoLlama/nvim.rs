//! Resolving a buffer argument — number, name, `#`, `%` — and the questions
//! about one: `bufnr()`, `bufname()`, `bufwinid()`, ...

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::cstr;
use crate::eval::typval::NumBuf;
use crate::guard::Suppress;
use crate::types::{VAR_NUMBER, VAR_STRING};

/// The buffer `avar` names, by number or by exact name.
///
/// # Safety
/// `avar` must point at a live typval.
pub unsafe fn find_buffer(avar: *mut TypVal) -> Option<Buf> {
    // SAFETY: the caller's obligation; under `VAR_STRING` the union's live arm
    // is `v_string`, a NUL-terminated string or NULL.
    match unsafe { (*avar).v_type } {
        VAR_NUMBER => find_buf(number_as_int(unsafe { (*avar).number_or_zero() })),
        VAR_STRING if !unsafe { (*avar).string_or_null() }.is_null() => {
            let name = unsafe { (*avar).string_or_null() };
            if let Some(found) = unsafe { buflist_findname_exp(name) } {
                return Some(found);
            }
            // A buffer with no file of its own — a URL, or a scratch
            // buffer — is not in the name index, so it is matched
            // literally instead.
            buffers().find(|b| {
                !b.b_fname.is_null()
                    && (unsafe { path_with_url(cstr::at(b.b_fname)) } != 0
                        || buf_is_nofilename(Some(*b)))
                    && unsafe { cstr::eq(b.b_fname, name) }
            })
        }
        _ => None,
    }
}

/// `bufadd({name})` — the number of the buffer, creating it if need be.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_bufadd(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, result) = frame!(args, result);
    // SAFETY: the arguments are live typvals and `tv_get_string` hands back a
    // NUL-terminated string.
    let name = unsafe { numbuf.string(args.ptr(0)) } as *mut c_char;
    // An empty name asks for an unnamed buffer.
    let name = if unsafe { *name } == 0 {
        ptr::null_mut()
    } else {
        name
    };
    result.write_number(VarNumber::from(unsafe { buflist_add(name, 0) }));
}

/// `bufexists({buf})`.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_bufexists(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    // SAFETY: the arguments are live typvals.
    let buf = unsafe { find_buffer(args.ptr(0)) };
    result.write_number(VarNumber::from(buf.is_some()));
}

/// `buflisted({buf})`.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_buflisted(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    // SAFETY: the arguments are live typvals, and the resolver answers a live
    // buffer or NULL.
    let listed = unsafe { find_buffer(args.ptr(0)) }.is_some_and(|b| b.b_p_bl != 0);
    result.write_number(VarNumber::from(listed));
}

/// `bufload({buf})` — read the file in if the buffer is not loaded yet.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `unused` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_bufload(args: *mut TypVal, unused: *mut TypVal, _fptr: EvalFuncData) {
    let (args, _) = frame!(args, unused);
    // SAFETY: the arguments are live typvals, and the resolver answers a live
    // buffer or NULL.
    let buf = unsafe { get_buf_arg(args.ptr(0)) };
    if buf.is_none() {
        return;
    }
    // A swap file found while loading must not leave the standing
    // "read-only" answer behind unless that is what it already was.
    if swap_exists_action.get() != SEA_READONLY {
        swap_exists_action.set(SEA_NONE);
    }
    buf_ensure_loaded(buf.expect("a live handle"));
}

/// `bufloaded({buf})`.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_bufloaded(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    // SAFETY: the arguments are live typvals, and the resolver answers a live
    // buffer or NULL.
    let loaded = unsafe { find_buffer(args.ptr(0)) }.is_some_and(|b| !b.b_ml.ml_mfp.is_null());
    result.write_number(VarNumber::from(loaded));
}

/// `bufname([{buf}])` — the buffer's short name, empty when it has none.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_bufname(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    result.write_string(ptr::null_mut());
    // SAFETY: the arguments are live typvals; `curbuf` is set and the resolver
    // answers a live buffer or NULL.
    let buf = if args.has(0) {
        unsafe { tv_get_buf_from_arg(args.ptr(0)) }
    } else {
        Some(Buf::current())
    };
    if let Some(buf) = buf
        && !buf.b_fname.is_null()
    {
        result.write_string(unsafe { xstrdup(buf.b_fname) });
    }
}

/// `bufnr([{buf} [, {create}]])` — -1 when there is no such buffer and it was
/// not asked to be created.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_bufnr(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, result) = frame!(args, result);
    result.write_number(-1);
    // SAFETY: the arguments are live typvals and `curbuf` is set.
    let mut buf: *mut Buffer = if !args.has(0) {
        Buf::current_raw()
    } else {
        if !unsafe { tv_check_str_or_nr(args.ptr(0)) } {
            return;
        }
        // The lookup itself must not report "no such buffer": a second
        // argument asks for the buffer to be created instead.
        let _no_emsg = Suppress::emsg();
        arg_buf(args, 0, 0).map_or(ptr::null_mut(), Buf::raw)
    };
    let mut error = false;
    if buf.is_null()
        && args.has(1)
        && unsafe { tv_get_number_chk(args.ptr(1), &raw mut error) } != 0
        && !error
    {
        let name = unsafe { numbuf.string_chk(args.ptr(0)) };
        if !name.is_null() {
            buf = unsafe {
                buflist_new(name as *mut c_char, ptr::null_mut(), 1, 0)
                    .map_or(ptr::null_mut(), Buf::raw)
            };
        }
    }
    if let Some(buf) = unsafe { Buf::from_raw(buf) } {
        result.write_number(VarNumber::from(buf.handle));
    }
}

/// `bufwinid()` and `bufwinnr()`: the first window of the current tab page
/// showing the buffer, as an id or as a `winnr()` ordinal.
fn buf_win_common(args: Args<'_>, result: &mut TypVal, get_nr: bool) {
    let buf = arg_buf_chk(args, 0);
    if buf.is_none() {
        result.write_number(-1);
        return;
    }
    let tp = TabPage::current();
    // `bufwinnr()` skips a window the numbering has no number for;
    // `bufwinid()` still answers its id.
    let mut winnr = 0;
    let found = windows_in_tab(tp).find(|wp| {
        winnr += c_int::from(wp.has_winnr(tp));
        Some(wp.buffer()) == buf && (!get_nr || wp.has_winnr(tp))
    });
    result.write_number(match found {
        Some(wp) => VarNumber::from(if get_nr { winnr } else { wp.handle }),
        None => -1,
    });
}

/// `bufwinid({buf})`.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_bufwinid(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    buf_win_common(args, result, false);
}

/// `bufwinnr({buf})`.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and
/// `result` its live return value: the contract the two builtin
/// dispatchers keep.
pub unsafe fn f_bufwinnr(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    buf_win_common(args, result, true);
}
