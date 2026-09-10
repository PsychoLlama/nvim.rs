//! Augroups: named containers with an id.
//!
//! `nvim_create_augroup` is idempotent unless `clear` is set, which is the
//! whole reason plugins can re-source themselves; the two `del_augroup_*`
//! spellings differ only in how they name the group.
//! `get_augroup_from_object` is the shared "id, name, or absent" decoder
//! the create/clear/get paths all take their group from.

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
use crate::api::private::helpers::Reported;
use crate::api::private::validate::{err_bad_number, err_bad_value, err_expected};
use crate::narrow::number_as_int;
use crate::winlayer::Live;

/// # Safety
///
/// `name` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`. `opts` must point at the `KeyDict_create_augroup` the
/// dispatcher filled in, live for the call.
pub unsafe fn nvim_create_augroup(
    channel_id: uint64_t,
    name: String_0,
    opts: *mut KeyDict_create_augroup,
) -> Result<Integer, Error> {
    // SAFETY: the dispatcher's keyset outlives this call.
    let opts = unsafe { Live::<KeyDict_create_augroup>::new(opts) };
    let mut error = Error::none();
    let augroup_name_0: *mut ::core::ffi::c_char = name.data();
    let clear_autocmds: bool = if opts.is_set__create_augroup_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_create_augroup__clear
        != 0 as ::core::ffi::c_ulonglong
    {
        ::core::ffi::c_int::from(opts.clear)
    } else {
        1
    } != 0;
    let _sctx = api_set_sctx(channel_id);
    let augroup: ::core::ffi::c_int = unsafe { augroup_add(augroup_name_0) };
    if augroup == AUGROUP_ERROR as ::core::ffi::c_int {
        // Unreachable: `augroup_add` only ever answers a positive id.
        // The guard restores on the way out regardless -- upstream's
        // `WITH_SCRIPT_CONTEXT` puts the restore *after* the block, so
        // this `return` skips it there.
        error = Error::exception(c"Failed to set augroup");
        return (-1 as Integer).reported(error);
    }
    if clear_autocmds {
        for event in AutoEvent::all() {
            aucmd_del_for_event_and_group(event, augroup);
        }
    }
    Integer::from(augroup).reported(error)
}

pub fn nvim_del_augroup_by_id(id: Integer) -> Result<(), Error> {
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<Exception>(),
        private_msg_list: ::core::ptr::null_mut::<MsgList>(),
        msg_list: ::core::ptr::null::<*const MsgList>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    unsafe { try_enter(&raw mut tstate) };
    let name: *mut ::core::ffi::c_char = if id == 0 as Integer {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    } else {
        augroup_name(number_as_int(id))
    };
    unsafe { augroup_del(name, false) };
    unsafe { try_leave(&raw mut tstate) }?;
    Ok(())
}

/// # Safety
///
/// `name` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`.
pub unsafe fn nvim_del_augroup_by_name(name: String_0) -> Result<(), Error> {
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<Exception>(),
        private_msg_list: ::core::ptr::null_mut::<MsgList>(),
        msg_list: ::core::ptr::null::<*const MsgList>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    unsafe { try_enter(&raw mut tstate) };
    unsafe { augroup_del(name.data(), false) };
    unsafe { try_leave(&raw mut tstate) }?;
    Ok(())
}

/// # Safety
///
/// `group` must be a well-formed API object the caller owns for the call.
pub(crate) unsafe fn get_augroup_from_object(group: Object, err: &mut Error) -> ::core::ffi::c_int {
    let au_group: ::core::ffi::c_int;
    let name: *mut ::core::ffi::c_char;
    match group {
        Object::Nil => return AUGROUP_DEFAULT as ::core::ffi::c_int,
        Object::String(s) => {
            au_group = unsafe { augroup_find(s.data()) };
            if !(au_group != AUGROUP_ERROR as ::core::ffi::c_int) {
                // SAFETY: the string's bytes outlive this call.
                let name = unsafe { s.as_cstr() };
                *err = err_bad_value(c"group", name);
                return AUGROUP_ERROR as ::core::ffi::c_int;
            }
            return au_group;
        }
        Object::Integer(n) => {
            au_group = number_as_int(n);
            name = if au_group == 0 as ::core::ffi::c_int {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            } else {
                augroup_name(au_group)
            };
            if !unsafe { augroup_exists(name) } {
                *err = err_bad_number(c"group", int64_t::from(au_group));
                return AUGROUP_ERROR as ::core::ffi::c_int;
            }
            return au_group;
        }
        _ => {
            if true {
                let want = c"String or Integer";
                let got = api_typename(group.kind());
                *err = err_expected(c"group", want, Some(got));
                return AUGROUP_ERROR as ::core::ffi::c_int;
            }
        }
    }
    panic!("Reached end of non-void function without returning");
}
