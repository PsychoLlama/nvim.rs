//! What `nvim_get_option_info` reports.
//!
//! Thirteen keys per option, built into an arena the caller owns. The keys
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
use core::ptr;

use crate::api::private::helpers::{arena_dict, cstr_as_string};
use crate::options::*;
use crate::types::{
    ApiDict, Arena, Error, Integer, KeyValuePair, Object, OptIndex, OptionSetFlags, ScriptCtx,
    String_0, key_value_pair, size_t,
};

use crate::api::private::validate::err_bad_value;

use super::{
    find_option_len, get_option, kOptFlagComma, kOptFlagFlagList, kOptFlagNoDup, kOptScopeBuf,
    kOptScopeWin, option_default, option_get_type, option_has_scope, option_is_global_local,
    option_last_set, option_was_set, optval_as_object, optval_type_name,
};

/// Append `key: value` to an arena-backed dictionary.
///
/// # Safety
///
/// `dict` must have been allocated with room for one more pair.
unsafe fn push(dict: &mut ApiDict, key: &'static core::ffi::CStr, value: Object) {
    // SAFETY: the caller reserved the capacity.
    unsafe {
        *dict.items.add(dict.size) = key_value_pair {
            key: cstr_as_string(key.as_ptr().cast_mut()),
            value,
        }
    };
    dict.size += 1;
}

/// A `String` value naming one of the option table's static strings.
///
/// Every `*const c_char` this module hands over comes from the generated
/// table or from a `c"..."` literal, so it is a live NUL-terminated string
/// for the whole run — which is the whole of `cstr_as_string`'s promise, and
/// why it is paid once here rather than at each of the four keys below.
fn name_value(name: *const c_char) -> Object {
    // SAFETY: a static NUL-terminated string.
    Object::String(unsafe { cstr_as_string(name) })
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
///
/// # Safety
///
/// `name` must be a valid string; `buffer`, `win` and `arena` must be live.
pub(crate) unsafe fn get_vimoption(
    name: String_0,
    opt_flags: OptionSetFlags,
    buffer: Buf,
    win: Win,
    arena: *mut Arena,
    err: &mut Error,
) -> ApiDict {
    // SAFETY: the caller's pointers are live.
    let opt_idx: OptIndex = find_option_len(unsafe { name.as_bytes() });
    if opt_idx == kOptInvalid {
        // SAFETY: the keyset's name is NUL-terminated.
        let name = unsafe { name.as_cstr() };
        *err = err_bad_value(c"option (not found)", name);
        return ApiDict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ptr::null_mut::<KeyValuePair>(),
        };
    }
    unsafe { vimoption2dict(opt_idx, opt_flags, buffer, win, arena) }
}

/// Every option's info dictionary, keyed by full name.
///
/// # Safety
///
/// `arena` must be live.
pub(crate) unsafe fn get_all_vimoptions(arena: *mut Arena) -> ApiDict {
    // SAFETY: the arena is live, and it is asked for exactly `kOptCount`
    // pairs before any is pushed.
    let mut retval = arena_dict(arena, kOptCount as size_t);
    for opt_idx in kOptAleph..kOptCount {
        let (scope, buf, win) = (
            OptionSetFlags::GLOBAL,
            Buf::current_raw(),
            Win::current_raw(),
        );
        // SAFETY: the caller's arena, and `curbuf`/`curwin` are live.
        let opt_dict =
            unsafe { vimoption2dict(opt_idx, scope, Buf::new(buf), Win::new(win), arena) };
        let pair = key_value_pair {
            // SAFETY: the option table's names are static C strings.
            key: unsafe { cstr_as_string(get_option(opt_idx).fullname) },
            value: Object::Dict(opt_dict),
        };
        // SAFETY: the dictionary was asked for exactly `kOptCount` pairs.
        unsafe { *retval.items.add(retval.size) = pair };
        retval.size += 1;
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
///
/// # Safety
///
/// `buffer`, `win` and `arena` must be live.
pub(crate) unsafe fn vimoption2dict(
    opt_idx: OptIndex,
    opt_flags: OptionSetFlags,
    buffer: Buf,
    win: Win,
    arena: *mut Arena,
) -> ApiDict {
    let opt = get_option(opt_idx);
    // SAFETY: the caller's pointers are live, and the dictionary is asked
    // for exactly the thirteen slots pushed below.
    let mut dict = arena_dict(arena, 13 as size_t);

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
    // SAFETY: the dictionary was asked for exactly these thirteen slots.
    for (key, value) in entries {
        unsafe { push(&mut dict, key, value) };
    }

    dict
}
