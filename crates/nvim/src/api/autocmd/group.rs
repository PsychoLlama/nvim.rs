//! Augroups: named containers with an id.
//!
//! `nvim_create_augroup` is idempotent unless `clear` is set, which is the
//! whole reason plugins can re-source themselves; the two `del_augroup_*`
//! spellings differ only in how they name the group.
//! `get_augroup_from_object` is the shared "id, name, or absent" decoder
//! the create/clear/get paths all take their group from.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported};
use crate::winlayer::Live;
use core::ffi::{CStr, c_char};
use core::ptr;

/// "Invalid `name`: `n`", for a handle or id that names nothing.
///
/// # Safety
/// `err` must be the caller's error slot.
unsafe fn err_bad_number(err: *mut Error, name: &CStr, n: int64_t) {
    let none = ptr::null();
    // SAFETY: the caller's promise; `name` is a C string.
    unsafe { api_err_invalid(err, name.as_ptr(), none, n, false) };
}

/// "Invalid `name`: expected `want`", naming `got` when it says.
///
/// # Safety
/// `err` must be the caller's error slot, `want` a C string and `got` null
/// or a C string.
unsafe fn err_expected(err: *mut Error, name: &CStr, want: *const c_char, got: *const c_char) {
    // SAFETY: the caller's promise; `name` is a C string too.
    unsafe { api_err_exp(err, name.as_ptr(), want, got) };
}

/// "Invalid `name`: '`val`'", for a value the caller spelled wrong.
///
/// # Safety
/// `err` must be the caller's error slot and `val` null or a C string.
unsafe fn err_bad_value(err: *mut Error, name: &CStr, val: *const c_char) {
    // SAFETY: the caller's promise; `name` is a C string too.
    unsafe { api_err_invalid(err, name.as_ptr(), val, 0, true) };
}

pub unsafe fn nvim_create_augroup(
    channel_id: uint64_t,
    name: String_0,
    opts: *mut KeyDict_create_augroup,
) -> Result<Integer, Error> {
    // SAFETY: the dispatcher's keyset outlives this call.
    let opts = unsafe { Live::<KeyDict_create_augroup>::new(opts) };
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut augroup_name_0: *mut ::core::ffi::c_char = name.data();
    let mut clear_autocmds: bool = if opts.is_set__create_augroup_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_create_augroup__clear
        != 0 as ::core::ffi::c_ulonglong
    {
        opts.clear as ::core::ffi::c_int
    } else {
        1
    } != 0;
    let mut augroup: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let _sctx = api_set_sctx(channel_id);
    augroup = unsafe { augroup_add(augroup_name_0) };
    if augroup == AUGROUP_ERROR as ::core::ffi::c_int {
        // Unreachable: `augroup_add` only ever answers a positive id.
        // The guard restores on the way out regardless -- upstream's
        // `WITH_SCRIPT_CONTEXT` puts the restore *after* the block, so
        // this `return` skips it there.
        unsafe { api_set_error(err, kErrorTypeException, c"Failed to set augroup".as_ptr()) };
        return (-1 as Integer).reported(error);
    }
    if clear_autocmds {
        let mut event: event_T = EVENT_BUFADD;
        while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
            aucmd_del_for_event_and_group(event, augroup);
            event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
        }
    }
    (augroup as Integer).reported(error)
}

pub unsafe fn nvim_del_augroup_by_id(id: Integer) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    unsafe { try_enter(&raw mut tstate) };
    let mut name: *mut ::core::ffi::c_char = if id == 0 as Integer {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    } else {
        augroup_name(id as ::core::ffi::c_int)
    };
    unsafe { augroup_del(name, false) };
    unsafe { try_leave(&raw mut tstate, err) };
    ().reported(error)
}

pub unsafe fn nvim_del_augroup_by_name(name: String_0) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    unsafe { try_enter(&raw mut tstate) };
    unsafe { augroup_del(name.data(), false) };
    unsafe { try_leave(&raw mut tstate, err) };
    ().reported(error)
}

pub(crate) unsafe fn get_augroup_from_object(
    mut group: Object,
    mut err: *mut Error,
) -> ::core::ffi::c_int {
    let mut au_group: ::core::ffi::c_int = AUGROUP_ERROR as ::core::ffi::c_int;
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    match group.type_0 as ::core::ffi::c_uint {
        kObjectTypeNil => return AUGROUP_DEFAULT as ::core::ffi::c_int,
        kObjectTypeString => {
            au_group = unsafe { augroup_find(group.data.string.data()) };
            if !(au_group != AUGROUP_ERROR as ::core::ffi::c_int) {
                // SAFETY: `err` is this call's own error slot.
                unsafe { err_bad_value(err, c"group", group.data.string.data()) };
                return AUGROUP_ERROR as ::core::ffi::c_int;
            }
            return au_group;
        }
        kObjectTypeInteger => {
            au_group = unsafe { group.data.integer } as ::core::ffi::c_int;
            name = if au_group == 0 as ::core::ffi::c_int {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            } else {
                augroup_name(au_group)
            };
            if !unsafe { augroup_exists(name) } {
                // SAFETY: `err` is this call's own error slot.
                unsafe { err_bad_number(err, c"group", au_group as int64_t) };
                return AUGROUP_ERROR as ::core::ffi::c_int;
            }
            return au_group;
        }
        _ => {
            if true {
                let want = c"String or Integer".as_ptr();
                let got = api_typename(group.type_0);
                // SAFETY: `err` is this call's own error slot.
                unsafe { err_expected(err, c"group", want, got) };
                return AUGROUP_ERROR as ::core::ffi::c_int;
            }
        }
    }
    panic!("Reached end of non-void function without returning");
}
