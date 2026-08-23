//! Variables themselves: the dictionary watchers, `islocked()` and `id()`.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::frame;
use super::{DI_FLAGS_LOCK, FNE_CHECK_START, GLV_NO_AUTOLOAD, GLV_READ_ONLY, dummy_ap};
use crate::eval::typval::{
    callback_free, kCallbackNone, tv_dict_watcher_add, tv_dict_watcher_remove, tv_get_string,
    tv_get_string_chk, tv_islocked,
};
use crate::eval::vars::find_var;
use crate::eval::{callback_from_typval, clear_lval, get_lval};
use crate::ex_cmds::check_secure;
use crate::main::{e_dictkey, e_invarg2, e_trailing_arg};
use crate::memory::xmalloc;
use crate::os::cshim::gettext;
use crate::strings::vim_vsnprintf_typval;
use crate::types::{
    Callback, Callback_data, EvalFuncData, NUL, VAR_DICT, VAR_FUNC, VAR_NUMBER, VAR_STRING,
    typval_T, varnumber_T,
};
use crate::{semsg, semsg_c};
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
    let (args, _rettv) = frame!(argvars, _rettv);
    // SAFETY: every callee below is a C entry point taking live typvals from
    // the frame; the callback is handed to the watcher, which takes it over.
    unsafe {
        if check_secure() {
            return;
        }
        if args.ty(0) != VAR_DICT {
            semsg!("E475: Invalid argument: dict");
            return;
        }
        if args.get(0).vval.v_dict.is_null() {
            // The C spells the name through the read-only-variable message's
            // `%.*s`, with the length `strlen` gives it; the text is fixed.
            semsg!("E46: Cannot change read-only variable \"dictwatcheradd() argument\"");
            return;
        }
        if args.ty(1) != VAR_STRING && args.ty(1) != VAR_NUMBER {
            semsg!("E475: Invalid argument: key");
            return;
        }
        let key_pattern = tv_get_string_chk(args.ptr(1));
        if key_pattern.is_null() {
            return;
        }
        let key_pattern_len = strlen(key_pattern);
        let mut callback = NO_CALLBACK;
        if !callback_from_typval(&raw mut callback, args.ptr(2)) {
            semsg!("E475: Invalid argument: funcref");
            return;
        }
        tv_dict_watcher_add(
            args.get(0).vval.v_dict,
            key_pattern,
            key_pattern_len,
            callback,
        );
    }
}

/// `dictwatcherdel({dict}, {pattern}, {callback})`.
pub unsafe fn f_dictwatcherdel(argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, _rettv) = frame!(argvars, _rettv);
    // SAFETY: as `f_dictwatcheradd`; the callback built here is only used to
    // identify a watcher and is freed before returning.
    unsafe {
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
        let key_pattern = tv_get_string_chk(args.ptr(1));
        if key_pattern.is_null() {
            return;
        }
        let mut callback = NO_CALLBACK;
        if !callback_from_typval(&raw mut callback, args.ptr(2)) {
            return;
        }
        if !tv_dict_watcher_remove(
            args.get(0).vval.v_dict,
            key_pattern,
            strlen(key_pattern),
            &callback,
        ) {
            semsg!("Couldn't find a watcher matching key and callback");
        }
        callback_free(&raw mut callback);
    }
}

/// `islocked({expr})` — 1 when the variable the name resolves to is locked,
/// 0 when it is not, -1 when there is no such variable.
pub unsafe fn f_islocked(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = -1;
    // SAFETY: `get_lval` clears `lv` before writing to it, and every pointer
    // read below comes back from it; `clear_lval` runs on every path.
    unsafe {
        let mut lv = core::mem::zeroed();
        let end = get_lval(
            tv_get_string(args.ptr(0)) as *mut c_char,
            ptr::null_mut(),
            &raw mut lv,
            false,
            false,
            (GLV_NO_AUTOLOAD | GLV_READ_ONLY) as c_int,
            FNE_CHECK_START,
        );
        if !end.is_null() && !lv.ll_name.is_null() {
            if *end as c_int != NUL {
                // Both texts interpolate the unconsumed remainder of the
                // caller's expression, so they keep the variadic call.
                let fmt = if lv.ll_name_len == 0 {
                    e_invarg2.as_ptr()
                } else {
                    e_trailing_arg.as_ptr()
                };
                semsg_c!(gettext(fmt), end);
            } else if lv.ll_tv.is_null() {
                let di = find_var(lv.ll_name, lv.ll_name_len, ptr::null_mut(), true);
                if !di.is_null() {
                    let locked = (*di).di_flags as c_int & DI_FLAGS_LOCK as c_int != 0
                        || tv_islocked(&raw mut (*di).di_tv);
                    rettv.vval.v_number = locked as varnumber_T;
                }
            } else if lv.ll_range {
                semsg!("E786: Range not allowed");
            } else if !lv.ll_newkey.is_null() {
                semsg_c!(gettext(e_dictkey.as_ptr()), lv.ll_newkey);
            } else if !lv.ll_list.is_null() {
                rettv.vval.v_number = tv_islocked(&raw mut (*lv.ll_li).li_tv) as varnumber_T;
            } else {
                rettv.vval.v_number = tv_islocked(&raw mut (*lv.ll_di).di_tv) as varnumber_T;
            }
        }
        clear_lval(&raw mut lv);
    }
}

/// `id({expr})` — a string unique to the container `expr` refers to.
///
/// The address is formatted by `vim_vsnprintf_typval`'s `%p`, which reads
/// its operand from the typval array rather than from a `va_list`; the
/// `va_list` handed in is a zeroed placeholder that is never read.
pub unsafe fn f_id(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the measuring call writes nothing; the second is handed a
    // buffer of exactly the size it reported plus the terminator.
    unsafe {
        let base = args.ptr(0);
        let len = vim_vsnprintf_typval(
            ptr::null_mut(),
            0,
            c"%p".as_ptr(),
            (*dummy_ap.ptr()).clone(),
            base,
        );
        rettv.v_type = VAR_STRING;
        rettv.vval.v_string = xmalloc(len as usize + 1) as *mut c_char;
        vim_vsnprintf_typval(
            rettv.vval.v_string,
            len as usize + 1,
            c"%p".as_ptr(),
            (*dummy_ap.ptr()).clone(),
            base,
        );
    }
}
