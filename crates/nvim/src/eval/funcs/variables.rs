//! Variables themselves: the dictionary watchers, `islocked()` and `id()`.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::frame;
use super::wrappers::{arg_string, arg_string_chk};
use super::{DI_FLAGS_LOCK, FNE_CHECK_START, GLV_NO_AUTOLOAD, GLV_READ_ONLY, dummy_ap};
use crate::eval::typval::{
    NumBuf, callback_free, kCallbackNone, tv_dict_watcher_add, tv_dict_watcher_remove, tv_islocked,
};
use crate::eval::vars::find_var;
use crate::eval::{callback_from_typval, clear_lval, get_lval};
use crate::ex_cmds::check_secure;
use crate::main::{e_dictkey, e_invarg2, e_trailing_arg};
use crate::memory::xmalloc;
use crate::os::cshim::{gettext, gettext_ptr};
use crate::semsg;
use crate::semsg_c;
use crate::strings::vim_vsnprintf_typval;
use crate::types::{
    Callback, Callback_data, EvalFuncData, NUL, VAR_DICT, VAR_FUNC, VAR_NUMBER, VAR_STRING,
    typval_T, varnumber_T,
};
use ::libc::strlen;
use core::ffi::{c_char, c_int};
use core::ptr;

/// An unset callback, the shape `callback_from_typval` fills in.
const NO_CALLBACK: Callback = Callback {
    data: Callback_data {
        funcref: ptr::null_mut(),
    },
    type_0: kCallbackNone,
};

/// `dictwatcheradd({dict}, {pattern}, {callback})`.
pub unsafe fn f_dictwatcheradd(argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, _rettv) = frame!(argvars, _rettv);
    // SAFETY throughout: every callee below is a C entry point taking live typvals from
    // the frame; the callback is handed to the watcher, which takes it over.
    if check_secure() {
        return;
    }
    if args.ty(0) != VAR_DICT {
        semsg!("E475: Invalid argument: dict");
        return;
    }
    if unsafe { args.get(0).vval.v_dict }.is_null() {
        // The C spells the name through the read-only-variable message's
        // `%.*s`, with the length `strlen` gives it; the text is fixed.
        semsg!("E46: Cannot change read-only variable \"dictwatcheradd() argument\"");
        return;
    }
    if args.ty(1) != VAR_STRING && args.ty(1) != VAR_NUMBER {
        semsg!("E475: Invalid argument: key");
        return;
    }
    let key_pattern = arg_string_chk(&mut numbuf, args.get(1));
    if key_pattern.is_null() {
        return;
    }
    let key_pattern_len = unsafe { strlen(key_pattern) };
    let mut callback = NO_CALLBACK;
    if !unsafe { callback_from_typval(&raw mut callback, args.ptr(2)) } {
        semsg!("E475: Invalid argument: funcref");
        return;
    }
    // SAFETY: the tag checked above says the union holds a Dict pointer;
    // the watcher takes the callback over.
    let d = unsafe { args.get(0).vval.v_dict };
    unsafe { tv_dict_watcher_add(d, key_pattern, key_pattern_len, callback) };
}

/// `dictwatcherdel({dict}, {pattern}, {callback})`.
pub unsafe fn f_dictwatcherdel(argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, _rettv) = frame!(argvars, _rettv);
    // SAFETY throughout: as `f_dictwatcheradd`; the callback built here is only used to
    // identify a watcher and is freed before returning.
    if check_secure() {
        return;
    }
    if args.ty(0) != VAR_DICT {
        semsg!("E475: Invalid argument: dict");
        return;
    }
    if args.ty(2) != VAR_FUNC && args.ty(2) != VAR_STRING {
        semsg!("E475: Invalid argument: funcref");
        return;
    }
    let key_pattern = arg_string_chk(&mut numbuf, args.get(1));
    if key_pattern.is_null() {
        return;
    }
    let mut callback = NO_CALLBACK;
    if !unsafe { callback_from_typval(&raw mut callback, args.ptr(2)) } {
        return;
    }
    // SAFETY: as `f_dictwatcheradd`; the callback only identifies a
    // watcher here and is freed below.
    let d = unsafe { args.get(0).vval.v_dict };
    let len = unsafe { strlen(key_pattern) };
    if !unsafe { tv_dict_watcher_remove(d, key_pattern, len, &callback) } {
        semsg!("Couldn't find a watcher matching key and callback");
    }
    unsafe { callback_free(&raw mut callback) };
}

/// `islocked({expr})` — 1 when the variable the name resolves to is locked,
/// 0 when it is not, -1 when there is no such variable.
pub unsafe fn f_islocked(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = -1;
    // SAFETY: `get_lval` clears `lv` before writing to it, and every pointer
    // read below comes back from it; `clear_lval` runs on every path.
    let mut lv = unsafe { core::mem::zeroed() };
    let name = arg_string(&mut numbuf, args.get(0)) as *mut c_char;
    let out = &raw mut lv;
    let flags = (GLV_NO_AUTOLOAD | GLV_READ_ONLY) as c_int;
    let nul = ptr::null_mut();
    let start = FNE_CHECK_START;
    let end = unsafe { get_lval(name, nul, out, false, false, flags, start) };
    if !end.is_null() && !lv.ll_name.is_null() {
        if unsafe { *end } as c_int != NUL {
            // Both texts interpolate the unconsumed remainder of the
            // caller's expression, so they keep the variadic call.
            let fmt = if lv.ll_name_len == 0 {
                e_invarg2.as_ptr()
            } else {
                e_trailing_arg.as_ptr()
            };
            unsafe { semsg_c!(gettext_ptr(fmt), end) };
        } else if lv.ll_tv.is_null() {
            let di = unsafe { find_var(lv.ll_name, lv.ll_name_len, ptr::null_mut(), true) };
            if !di.is_null() {
                let locked = unsafe { (*di).di_flags } as c_int & DI_FLAGS_LOCK as c_int != 0
                    || unsafe { tv_islocked(&raw mut (*di).di_tv) };
                rettv.vval.v_number = locked as varnumber_T;
            }
        } else if lv.ll_range {
            semsg!("E786: Range not allowed");
        } else if !lv.ll_newkey.is_null() {
            unsafe { semsg_c!(gettext(e_dictkey), lv.ll_newkey) };
        } else if !lv.ll_list.is_null() {
            rettv.vval.v_number = unsafe { tv_islocked(&raw mut (*lv.ll_li).li_tv) } as varnumber_T;
        } else {
            rettv.vval.v_number = unsafe { tv_islocked(&raw mut (*lv.ll_di).di_tv) } as varnumber_T;
        }
    }
    unsafe { clear_lval(&raw mut lv) };
}

/// `id({expr})` — a string unique to the container `expr` refers to.
///
/// The address is formatted by `vim_vsnprintf_typval`'s `%p`, which reads
/// its operand from the typval array rather than from a `va_list`; the
/// `va_list` handed in is a zeroed placeholder that is never read.
pub unsafe fn f_id(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY throughout: the measuring call writes nothing; the second is handed a
    // buffer of exactly the size it reported plus the terminator.
    let base = args.ptr(0);
    let fmt = c"%p".as_ptr();
    let nul = ptr::null_mut();
    let ap = unsafe { (*dummy_ap.ptr()).clone() };
    let len = unsafe { vim_vsnprintf_typval(nul, 0, fmt, ap, base) };
    rettv.v_type = VAR_STRING;
    rettv.vval.v_string = unsafe { xmalloc(len as usize + 1) } as *mut c_char;
    let out = unsafe { rettv.vval.v_string };
    let cap = len as usize + 1;
    let ap = unsafe { (*dummy_ap.ptr()).clone() };
    unsafe { vim_vsnprintf_typval(out, cap, fmt, ap, base) };
}
