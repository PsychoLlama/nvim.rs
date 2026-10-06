//! Variables themselves: the dictionary watchers, `islocked()` and `id()`.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::{DI_FLAGS_LOCK, FNE_CHECK_START, GLV_NO_AUTOLOAD, GLV_READ_ONLY, dummy_ap};
use crate::eval::typval::{NumBuf, callback_free, tv_islocked};
use crate::eval::vars::with_var;
use crate::eval::{Target, callback_from_typval, get_lval};
use crate::ex_cmds::check_secure;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use crate::strings::vim_vsnprintf_typval;
use crate::types::{
    Callback, EvalFuncData, TypVal, VAR_DICT, VAR_FUNC, VAR_NUMBER, VAR_STRING, VarNumber,
};
use core::ffi::c_int;
use core::ptr;

/// An unset callback, the shape `callback_from_typval` fills in.
const NO_CALLBACK: Callback = Callback::None;

/// `dictwatcheradd({dict}, {pattern}, {callback})`.
pub fn f_dictwatcheradd(args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let _rettv = _result;
    // SAFETY throughout: every callee below is a C entry point taking live typvals from
    // the frame; the callback is handed to the watcher, which takes it over.
    if check_secure() {
        return;
    }
    if args[0].v_type() != VAR_DICT {
        semsg!("E475: Invalid argument: dict");
        return;
    }
    if args[0].dict_or_null().is_null() {
        // The C spells the name through the read-only-variable message's
        // `%.*s`, with the length `strlen` gives it; the text is fixed.
        semsg!("E46: Cannot change read-only variable \"dictwatcheradd() argument\"");
        return;
    }
    if args[1].v_type() != VAR_STRING && args[1].v_type() != VAR_NUMBER {
        semsg!("E475: Invalid argument: key");
        return;
    }
    let Some(key_pattern) = numbuf.bytes_chk(&args[1]) else {
        return;
    };
    let mut callback = NO_CALLBACK;
    if !callback_from_typval(&mut callback, &args[2]) {
        semsg!("E475: Invalid argument: funcref");
        return;
    }
    // SAFETY: the kind checked above says the value holds a Dict pointer;
    // the watcher takes the callback over.
    let d = args[0].dict_or_null();
    unsafe { (*d).watcher_add(key_pattern, callback) };
}

/// `dictwatcherdel({dict}, {pattern}, {callback})`.
pub fn f_dictwatcherdel(args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let _rettv = _result;
    // SAFETY throughout: as `f_dictwatcheradd`; the callback built here is only used to
    // identify a watcher and is freed before returning.
    if check_secure() {
        return;
    }
    if args[0].v_type() != VAR_DICT {
        semsg!("E475: Invalid argument: dict");
        return;
    }
    if args[2].v_type() != VAR_FUNC && args[2].v_type() != VAR_STRING {
        semsg!("E475: Invalid argument: funcref");
        return;
    }
    let Some(key_pattern) = numbuf.bytes_chk(&args[1]) else {
        return;
    };
    let mut callback = NO_CALLBACK;
    if !callback_from_typval(&mut callback, &args[2]) {
        return;
    }
    // SAFETY: as `f_dictwatcheradd`; the callback only identifies a
    // watcher here and is freed below.
    // `v:_null_dict` is a `VAR_DICT` holding nothing, and has no watchers:
    // upstream's own entry point tested the pointer, and this is that test.
    let d = args[0].dict_or_null();
    // SAFETY: the argument's own dictionary, or none.
    let removed = !d.is_null() && unsafe { (*d).watcher_remove(key_pattern, &callback) };
    if !removed {
        semsg!("Couldn't find a watcher matching key and callback");
    }
    callback_free(&mut callback);
}

/// `islocked({expr})` — 1 when the variable the name resolves to is locked,
/// 0 when it is not, -1 when there is no such variable.
pub fn f_islocked(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_number(-1);
    let text = numbuf.string(&args[0]).to_bytes();
    let flags = (GLV_NO_AUTOLOAD | GLV_READ_ONLY) as c_int;
    let (mut lval, end) = get_lval(text, None, false, false, flags, FNE_CHECK_START);
    let Some(end) = end.filter(|_| lval.has_name()) else {
        return;
    };
    if end < text.len() {
        // The unconsumed remainder of the expression.
        let rest = msg_bytes(&text[end..]);
        if lval.name().is_empty() {
            semsg!("E475: Invalid argument: {rest}");
        } else {
            semsg!("E488: Trailing characters: {rest}");
        }
        return;
    }
    let locked = match &lval.target {
        Target::Variable | Target::Blob { .. } => with_var(lval.name(), true, |item| {
            item.di_flags & DI_FLAGS_LOCK as u8 != 0 || tv_islocked(item.di_lock, &item.di_tv)
        }),
        Target::Slot { span, .. } if span.range => {
            semsg!("E786: Range not allowed");
            None
        }
        Target::NewKey { key, .. } => {
            let key = msg_bytes(key);
            semsg!("E716: Key not present in Dictionary: \"{key}\"");
            None
        }
        Target::Slot { .. } => lval.with_slot(|tv, lock| tv_islocked(*lock, tv)),
    };
    if let Some(locked) = locked {
        result.write_number(VarNumber::from(locked));
    }
}

/// `id({expr})` — a string unique to the container `expr` refers to.
///
/// The address is formatted by `vim_vsnprintf_typval`'s `%p`, which reads
/// its operand from the typval array rather than from a `va_list`; the
/// `va_list` handed in is a zeroed placeholder that is never read.
pub fn f_id(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: the measuring call writes nothing; the second is handed a
    // buffer of exactly the size it reported plus the terminator.
    let base = Some(&args[..1]);
    let fmt = c"%p".as_ptr();
    let nul = ptr::null_mut();
    let ap = unsafe { (*dummy_ap.ptr()).clone() };
    let len = unsafe { vim_vsnprintf_typval(nul, 0, fmt, ap, base) };
    let mut out = vec![0u8; len as usize + 1];
    let cap = out.len();
    let ap = unsafe { (*dummy_ap.ptr()).clone() };
    unsafe { vim_vsnprintf_typval(out.as_mut_ptr().cast(), cap, fmt, ap, base) };
    out.truncate(len as usize);
    result.write_string(Some(out.into()));
}
