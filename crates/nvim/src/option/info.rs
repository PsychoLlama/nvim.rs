//! What `nvim_get_option_info` reports.
//!
//! Thirteen keys per option, in a dictionary the caller owns. The keys
//! and their order are API surface — the oracle byte-compares them — so the
//! push order below is deliberate and must not be sorted.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::winlayer::{Buf, Win};
use core::ffi::c_char;

use crate::api::private::helpers::cstr_to_string;
use crate::options::*;
use crate::types::{
    ApiDict, Error, Integer, Object, OptIndex, OptionSetFlags, ScriptCtx, String_0, size_t,
};

use crate::api::private::validate::err_bad_value;

use super::{
    find_option_len, get_option, kOptFlagComma, kOptFlagFlagList, kOptFlagNoDup, kOptScopeBuf,
    kOptScopeWin, option_default, option_get_type, option_has_scope, option_is_global_local,
    option_last_set, option_was_set, optval_as_object, optval_type_name,
};

/// Append `key: value` to the dictionary.
///
fn push(dict: &mut ApiDict, key: &'static core::ffi::CStr, value: Object) {
    dict.insert(key, value);
}

/// A `String` value naming one of the option table's static strings.
///
/// Every `*const c_char` this module hands over comes from the generated
/// table or from a `c"..."` literal, so it is a live NUL-terminated string
/// for the whole run — which is the whole of `cstr_to_string`'s promise, and
/// why it is paid once here rather than at each of the four keys below.
fn name_value(name: *const c_char) -> Object {
    // SAFETY: a static NUL-terminated string.
    Object::string(unsafe { cstr_to_string(name) })
}

/// A `Boolean` value.
fn bool_value(b: bool) -> Object {
    Object::Boolean(b)
}

/// An `Integer` value.
fn int_value(n: Integer) -> Object {
    Object::Integer(n)
}

/// The info dictionary for one option, looked up by name.
pub(crate) fn get_vimoption(
    name: String_0,
    opt_flags: OptionSetFlags,
    buffer: Buf,
    win: Win,
) -> Result<ApiDict, Error> {
    // SAFETY: the caller's pointers are live.
    let opt_idx: OptIndex = find_option_len(name.as_bytes());
    if opt_idx == kOptInvalid {
        // SAFETY: the keyset's name is NUL-terminated.
        let name = name.as_cstr();
        return Err(err_bad_value(c"option (not found)", name));
    }
    Ok(vimoption2dict(opt_idx, opt_flags, buffer, win))
}

/// Every option's info dictionary, keyed by full name.
pub(crate) fn get_all_vimoptions() -> ApiDict {
    // SAFETY: the arena is live, and it is asked for exactly `kOptCount`
    // pairs before any is pushed.
    let mut retval = ApiDict::with_capacity(kOptCount as size_t);
    for opt_idx in kOptAleph..kOptCount {
        let (scope, buf, win) = (OptionSetFlags::GLOBAL, Buf::current(), Win::current());
        let opt_dict = vimoption2dict(opt_idx, scope, buf, win);
        // SAFETY: the option table's names are static C strings.
        let key = unsafe { crate::cstr::bytes_at(get_option(opt_idx).fullname) };
        retval.insert(key, Object::dict(opt_dict));
    }
    retval
}

/// Which script last set the option, for the scope `opt_flags` names. A
/// buffer-local and a window-local answer can both apply — a window-local
/// one wins — and `:set` (neither flag) falls back to the global context
/// when the local one was never set.
fn last_set(opt_idx: OptIndex, opt_flags: OptionSetFlags, buffer: Buf, win: Win) -> ScriptCtx {
    let opt = get_option(opt_idx);
    if opt_flags == OptionSetFlags::GLOBAL {
        return option_last_set(opt_idx);
    }
    let mut script_ctx = ScriptCtx::NONE;
    if option_has_scope(opt_idx, kOptScopeBuf) {
        let idx = opt.scope_idx[kOptScopeBuf as usize].cast_unsigned();
        script_ctx = buffer.b_p_script_ctx[idx];
    }
    if option_has_scope(opt_idx, kOptScopeWin) {
        let idx = opt.scope_idx[kOptScopeWin as usize].cast_unsigned();
        script_ctx = win.w_onebuf_opt.wo_script_ctx[idx];
    }
    if opt_flags != OptionSetFlags::LOCAL && script_ctx.sc_sid == 0 {
        script_ctx = option_last_set(opt_idx);
    }
    script_ctx
}

/// The thirteen keys `nvim_get_option_info` reports for one option.
pub(crate) fn vimoption2dict(
    opt_idx: OptIndex,
    opt_flags: OptionSetFlags,
    buffer: Buf,
    win: Win,
) -> ApiDict {
    let opt = get_option(opt_idx);
    // SAFETY: the caller's pointers are live, and the dictionary is asked
    // for exactly the thirteen slots pushed below.
    let mut dict = ApiDict::with_capacity(13 as size_t);

    // An option in more than one scope reports the narrowest.
    let scope = if option_has_scope(opt_idx, kOptScopeBuf) {
        c"buf"
    } else if option_has_scope(opt_idx, kOptScopeWin) {
        c"win"
    } else {
        c"global"
    };
    let script_ctx = last_set(opt_idx, opt_flags, buffer, win);
    let type_name = optval_type_name(option_get_type(opt_idx));

    // The thirteen keys, in the order the API reports them. Building the
    // values first and pushing afterwards is the same sequence: an array
    // evaluates left to right, and `push` only writes into the dictionary.
    let entries = [
        (c"name", name_value(opt.fullname)),
        (c"shortname", name_value(opt.shortname)),
        (c"scope", name_value(scope.as_ptr())),
        (c"global_local", bool_value(option_is_global_local(opt_idx))),
        (c"commalist", bool_value(opt.flags & kOptFlagComma != 0)),
        (c"flaglist", bool_value(opt.flags & kOptFlagFlagList != 0)),
        (c"was_set", bool_value(option_was_set(opt_idx))),
        (c"last_set_sid", int_value(Integer::from(script_ctx.sc_sid))),
        (
            c"last_set_linenr",
            int_value(Integer::from(script_ctx.sc_lnum)),
        ),
        (
            c"last_set_chan",
            int_value(script_ctx.sc_chan.cast_signed()),
        ),
        (c"type", name_value(type_name.as_ptr())),
        (c"default", optval_as_object(option_default(opt_idx))),
        (
            c"allows_duplicates",
            bool_value(opt.flags & kOptFlagNoDup == 0),
        ),
    ];
    for (key, value) in entries {
        push(&mut dict, key, value);
    }

    dict
}
