//! What `nvim_get_option_info` reports.
//!
//! Thirteen keys per option, built into an arena the caller owns. The keys
//! and their order are API surface — the oracle byte-compares them — so the
//! push order below is deliberate and must not be sorted.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::ptr;

use crate::api::private::helpers::{arena_dict, cstr_as_string};
use crate::api::private::validate::api_err_invalid;
use crate::main::{curbuf, curwin};
use crate::options::*;
use crate::types::{
    Arena, Dict, Error, Integer, KeyValuePair, Object, OptIndex, String_0, buf_T, int64_t,
    kObjectTypeBoolean, kObjectTypeDict, kObjectTypeInteger, kObjectTypeString, key_value_pair,
    object, object_data, scid_T, sctx_T, size_t, vimoption_T, win_T,
};

use super::{
    OPT_GLOBAL, OPT_LOCAL, find_option_len, get_opt_idx, kOptFlagComma, kOptFlagFlagList,
    kOptFlagNoDup, kOptFlagWasSet, kOptScopeBuf, kOptScopeWin, option_get_type, option_has_scope,
    option_is_global_local, optval_as_object, optval_type_name,
};

/// Append `key: value` to an arena-backed dictionary.
///
/// # Safety
///
/// `dict` must have been allocated with room for one more pair.
unsafe fn push(dict: &mut Dict, key: &'static core::ffi::CStr, value: Object) {
    // SAFETY: the caller reserved the capacity.
    unsafe {
        *dict.items.add(dict.size) = key_value_pair {
            key: cstr_as_string(key.as_ptr().cast_mut()),
            value,
        };
    }
    dict.size += 1;
}

/// A `String` value, from a NUL-terminated C string the arena does not own.
///
/// # Safety
///
/// `s` must be NUL-terminated or null.
unsafe fn str_value(s: *const c_char) -> Object {
    object {
        type_0: kObjectTypeString,
        // SAFETY: the caller's string is NUL-terminated.
        data: object_data {
            string: unsafe { cstr_as_string(s.cast_mut()) },
        },
    }
}

/// A `Boolean` value.
fn bool_value(b: bool) -> Object {
    object {
        type_0: kObjectTypeBoolean,
        data: object_data { boolean: b },
    }
}

/// An `Integer` value.
fn int_value(n: Integer) -> Object {
    object {
        type_0: kObjectTypeInteger,
        data: object_data { integer: n },
    }
}

/// The info dictionary for one option, looked up by name.
///
/// # Safety
///
/// `name` must be a valid string; `buf`, `win` and `arena` must be live.
pub unsafe fn get_vimoption(
    name: String_0,
    opt_flags: c_int,
    buf: *mut buf_T,
    win: *mut win_T,
    arena: *mut Arena,
    err: *mut Error,
) -> Dict {
    // SAFETY: the caller's pointers are live.
    unsafe {
        let opt_idx: OptIndex = find_option_len(name.data, name.size);
        if opt_idx == kOptInvalid {
            api_err_invalid(err, c"option (not found)".as_ptr(), name.data, 0, true);
            return Dict {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ptr::null_mut::<KeyValuePair>(),
            };
        }
        let opt = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
        vimoption2dict(opt, opt_flags, buf, win, arena)
    }
}

/// Every option's info dictionary, keyed by full name.
///
/// # Safety
///
/// `arena` must be live.
pub unsafe fn get_all_vimoptions(arena: *mut Arena) -> Dict {
    // SAFETY: the arena is live, and it is asked for exactly `kOptCount`
    // pairs before any is pushed.
    unsafe {
        let mut retval = arena_dict(arena, kOptCount as size_t);
        for opt_idx in kOptAleph..kOptCount {
            let opt = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
            let opt_dict = vimoption2dict(opt, OPT_GLOBAL, curbuf.get(), curwin.get(), arena);
            *retval.items.add(retval.size) = key_value_pair {
                key: cstr_as_string((*opt).fullname),
                value: object {
                    type_0: kObjectTypeDict,
                    data: object_data { dict: opt_dict },
                },
            };
            retval.size += 1;
        }
        retval
    }
}

/// Which script last set the option, for the scope `opt_flags` names. A
/// buffer-local and a window-local answer can both apply — a window-local
/// one wins — and `:set` (neither flag) falls back to the global context
/// when the local one was never set.
///
/// # Safety
///
/// `opt` must point into the option table; `buf` and `win` must be live
/// unless `opt_flags` is exactly `OPT_GLOBAL`.
unsafe fn last_set(
    opt: *mut vimoption_T,
    opt_idx: OptIndex,
    opt_flags: c_int,
    buf: *mut buf_T,
    win: *mut win_T,
) -> sctx_T {
    // SAFETY: the caller's pointers are live for the scopes reached below.
    unsafe {
        if opt_flags == OPT_GLOBAL {
            return (*opt).script_ctx;
        }
        let mut script_ctx = sctx_T {
            sc_sid: 0 as scid_T,
            sc_seq: 0,
            sc_lnum: 0,
            sc_chan: 0,
        };
        if option_has_scope(opt_idx, kOptScopeBuf) {
            script_ctx = (*buf).b_p_script_ctx[(*opt).scope_idx[kOptScopeBuf as usize] as usize];
        }
        if option_has_scope(opt_idx, kOptScopeWin) {
            script_ctx =
                (*win).w_onebuf_opt.wo_script_ctx[(*opt).scope_idx[kOptScopeWin as usize] as usize];
        }
        if opt_flags != OPT_LOCAL && script_ctx.sc_sid == 0 {
            script_ctx = (*opt).script_ctx;
        }
        script_ctx
    }
}

/// The thirteen keys `nvim_get_option_info` reports for one option.
///
/// # Safety
///
/// `opt` must point into the option table; `buf`, `win` and `arena` must be
/// live.
pub(crate) unsafe fn vimoption2dict(
    opt: *mut vimoption_T,
    opt_flags: c_int,
    buf: *mut buf_T,
    win: *mut win_T,
    arena: *mut Arena,
) -> Dict {
    // SAFETY: the caller's pointers are live, and the dictionary is asked
    // for exactly the thirteen slots pushed below.
    unsafe {
        let opt_idx = get_opt_idx(opt);
        let mut dict = arena_dict(arena, 13 as size_t);

        push(&mut dict, c"name", str_value((*opt).fullname));
        push(&mut dict, c"shortname", str_value((*opt).shortname));

        // An option in more than one scope reports the narrowest.
        let scope = if option_has_scope(opt_idx, kOptScopeBuf) {
            c"buf"
        } else if option_has_scope(opt_idx, kOptScopeWin) {
            c"win"
        } else {
            c"global"
        };
        push(&mut dict, c"scope", str_value(scope.as_ptr()));

        push(
            &mut dict,
            c"global_local",
            bool_value(option_is_global_local(opt_idx)),
        );
        push(
            &mut dict,
            c"commalist",
            bool_value((*opt).flags & kOptFlagComma != 0),
        );
        push(
            &mut dict,
            c"flaglist",
            bool_value((*opt).flags & kOptFlagFlagList != 0),
        );
        push(
            &mut dict,
            c"was_set",
            bool_value((*opt).flags & kOptFlagWasSet != 0),
        );

        let script_ctx = last_set(opt, opt_idx, opt_flags, buf, win);
        push(
            &mut dict,
            c"last_set_sid",
            int_value(script_ctx.sc_sid as Integer),
        );
        push(
            &mut dict,
            c"last_set_linenr",
            int_value(script_ctx.sc_lnum as Integer),
        );
        push(
            &mut dict,
            c"last_set_chan",
            int_value(script_ctx.sc_chan as int64_t),
        );

        push(
            &mut dict,
            c"type",
            str_value(optval_type_name(option_get_type(opt_idx)).as_ptr()),
        );
        push(&mut dict, c"default", optval_as_object((*opt).def_val));
        push(
            &mut dict,
            c"allows_duplicates",
            bool_value((*opt).flags & kOptFlagNoDup == 0),
        );

        dict
    }
}
