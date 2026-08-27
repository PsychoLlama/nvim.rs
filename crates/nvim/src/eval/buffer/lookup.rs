//! Resolving a buffer argument — number, name, `#`, `%` — and the questions
//! about one: `bufnr()`, `bufname()`, `bufwinid()`, ...

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::eval::typval::NumBuf;
use crate::guard::Suppress;
use crate::types::{VAR_NUMBER, VAR_STRING};

/// The buffer `avar` names, by number or by exact name.
///
/// # Safety
/// `avar` must point at a live typval.
pub unsafe fn find_buffer(avar: *mut typval_T) -> *mut buf_T {
    // SAFETY: the caller's obligation; under `VAR_STRING` the union's live arm
    // is `v_string`, a NUL-terminated string or NULL.
    match unsafe { (*avar).v_type } {
        VAR_NUMBER => find_buf(number_as_int(unsafe { (*avar).vval.v_number }))
            .map_or(ptr::null_mut(), |mut b| b.raw()),
        VAR_STRING if !unsafe { (*avar).vval.v_string }.is_null() => {
            let name = unsafe { (*avar).vval.v_string };
            if let Some(mut found) = unsafe { buflist_findname_exp(name) } {
                return found.raw();
            }
            // A buffer with no file of its own — a URL, or a scratch
            // buffer — is not in the name index, so it is matched
            // literally instead.
            buffers()
                .find(|b| {
                    !b.b_fname.is_null()
                        && (unsafe { path_with_url(b.b_fname) } != 0 || buf_is_nofilename(Some(*b)))
                        && unsafe { strcmp(b.b_fname, name) } == 0
                })
                .map_or(ptr::null_mut(), Buf::raw)
        }
        _ => ptr::null_mut(),
    }
}

/// `bufadd({name})` — the number of the buffer, creating it if need be.
pub unsafe fn f_bufadd(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments are live typvals and `tv_get_string` hands back a
    // NUL-terminated string.
    let name = unsafe { numbuf.string(args.ptr(0)) } as *mut c_char;
    // An empty name asks for an unnamed buffer.
    let name = if unsafe { *name } == 0 {
        ptr::null_mut()
    } else {
        name
    };
    rettv.vval.v_number = varnumber_T::from(unsafe { buflist_add(name, 0) });
}

/// `bufexists({buf})`.
pub unsafe fn f_bufexists(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments are live typvals.
    let buf = unsafe { find_buffer(args.ptr(0)) };
    rettv.vval.v_number = varnumber_T::from(!buf.is_null());
}

/// `buflisted({buf})`.
pub unsafe fn f_buflisted(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments are live typvals, and the resolver answers a live
    // buffer or NULL.
    let listed = unsafe { Buf::from_raw(find_buffer(args.ptr(0))) }.is_some_and(|b| b.b_p_bl != 0);
    rettv.vval.v_number = varnumber_T::from(listed);
}

/// `bufload({buf})` — read the file in if the buffer is not loaded yet.
pub unsafe fn f_bufload(argvars: *mut typval_T, unused: *mut typval_T, _fptr: EvalFuncData) {
    let (args, _) = frame!(argvars, unused);
    // SAFETY: the arguments are live typvals, and the resolver answers a live
    // buffer or NULL.
    let buf = unsafe { get_buf_arg(args.ptr(0)) };
    if buf.is_null() {
        return;
    }
    // A swap file found while loading must not leave the standing
    // "read-only" answer behind unless that is what it already was.
    if swap_exists_action.get() != SEA_READONLY {
        swap_exists_action.set(SEA_NONE);
    }
    buf_ensure_loaded(unsafe { Buf::new(buf) });
}

/// `bufloaded({buf})`.
pub unsafe fn f_bufloaded(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments are live typvals, and the resolver answers a live
    // buffer or NULL.
    let loaded = unsafe { Buf::from_raw(find_buffer(args.ptr(0))) }
        .is_some_and(|b| !b.b_ml.ml_mfp.is_null());
    rettv.vval.v_number = varnumber_T::from(loaded);
}

/// `bufname([{buf}])` — the buffer's short name, empty when it has none.
pub unsafe fn f_bufname(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_STRING;
    rettv.vval.v_string = ptr::null_mut();
    // SAFETY: the arguments are live typvals; `curbuf` is set and the resolver
    // answers a live buffer or NULL.
    let buf = if args.has(0) {
        unsafe { Buf::from_raw(tv_get_buf_from_arg(args.ptr(0))) }
    } else {
        Some(unsafe { Buf::current() })
    };
    if let Some(buf) = buf
        && !buf.b_fname.is_null()
    {
        rettv.vval.v_string = unsafe { xstrdup(buf.b_fname) };
    }
}

/// `bufnr([{buf} [, {create}]])` — -1 when there is no such buffer and it was
/// not asked to be created.
pub unsafe fn f_bufnr(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = -1;
    // SAFETY: the arguments are live typvals and `curbuf` is set.
    let mut buf: *mut buf_T = if !args.has(0) {
        curbuf.get()
    } else {
        if !unsafe { tv_check_str_or_nr(args.ptr(0)) } {
            return;
        }
        // The lookup itself must not report "no such buffer": a second
        // argument asks for the buffer to be created instead.
        let _no_emsg = Suppress::emsg();
        unsafe { tv_get_buf(args.ptr(0), 0) }
    };
    let mut error = false;
    if buf.is_null()
        && args.has(1)
        && unsafe { tv_get_number_chk(args.ptr(1), &raw mut error) } != 0
        && !error
    {
        let name = unsafe { numbuf.string_chk(args.ptr(0)) };
        if !name.is_null() {
            buf = unsafe { buflist_new(name as *mut c_char, ptr::null_mut(), 1, 0) };
        }
    }
    if let Some(buf) = unsafe { Buf::from_raw(buf) } {
        rettv.vval.v_number = varnumber_T::from(buf.handle);
    }
}

/// `bufwinid()` and `bufwinnr()`: the first window of the current tab page
/// showing the buffer, as an id or as a `winnr()` ordinal.
///
/// # Safety
/// The arguments and `rettv` must be live typvals.
unsafe fn buf_win_common(args: Args<'_>, rettv: &mut typval_T, get_nr: bool) {
    // SAFETY: the caller's obligation.
    let buf = unsafe { tv_get_buf_from_arg(args.ptr(0)) };
    if buf.is_null() {
        rettv.vval.v_number = -1;
        return;
    }
    // SAFETY: `curtab` is set from startup to exit.
    let tp = unsafe { TabPage::current() };
    // `bufwinnr()` skips a window the numbering has no number for;
    // `bufwinid()` still answers its id.
    let mut winnr = 0;
    let found = windows_in_tab(tp).find(|wp| {
        winnr += c_int::from(wp.has_winnr(tp));
        ptr::eq(wp.w_buffer, buf) && (!get_nr || wp.has_winnr(tp))
    });
    rettv.vval.v_number = match found {
        Some(wp) => varnumber_T::from(if get_nr { winnr } else { wp.handle }),
        None => -1,
    };
}

/// `bufwinid({buf})`.
pub unsafe fn f_bufwinid(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals.
    unsafe { buf_win_common(args, rettv, false) };
}

/// `bufwinnr({buf})`.
pub unsafe fn f_bufwinnr(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the arguments and `rettv` are live typvals.
    unsafe { buf_win_common(args, rettv, true) };
}
