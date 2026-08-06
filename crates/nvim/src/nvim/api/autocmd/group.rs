//! Augroups: named containers with an id.
//!
//! `nvim_create_augroup` is idempotent unless `clear` is set, which is the
//! whole reason plugins can re-source themselves; the two `del_augroup_*`
//! spellings differ only in how they name the group.
//! `get_augroup_from_object` is the shared "id, name, or absent" decoder
//! the create/clear/get paths all take their group from.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn nvim_create_augroup(
    mut channel_id: uint64_t,
    mut name: String_0,
    mut opts: *mut KeyDict_create_augroup,
    mut err: *mut Error,
) -> Integer {
    unsafe {
        let mut augroup_name_0: *mut ::core::ffi::c_char = name.data;
        let mut clear_autocmds: bool = if (*opts).is_set__create_augroup_
            as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_create_augroup__clear
            != 0 as ::core::ffi::c_ulonglong
        {
            (*opts).clear as ::core::ffi::c_int
        } else {
            true_0
        } != 0;
        let mut augroup: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let save_current_sctx: sctx_T = api_set_sctx(channel_id);
        augroup = augroup_add(augroup_name_0);
        if augroup == AUGROUP_ERROR as ::core::ffi::c_int {
            api_set_error(err, kErrorTypeException, c"Failed to set augroup".as_ptr());
            return -1 as Integer;
        }
        if clear_autocmds {
            let mut event: event_T = EVENT_BUFADD;
            while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
                aucmd_del_for_event_and_group(event, augroup);
                event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
            }
        }
        current_sctx.set(save_current_sctx);
        return augroup as Integer;
    }
}

pub unsafe extern "C" fn nvim_del_augroup_by_id(mut id: Integer, mut err: *mut Error) {
    unsafe {
        let mut tstate: TryState = TryState {
            current_exception: ::core::ptr::null_mut::<except_T>(),
            private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
            msg_list: ::core::ptr::null::<*const msglist_T>(),
            got_int: 0,
            did_throw: false,
            need_rethrow: 0,
            did_emsg: 0,
        };
        try_enter(&raw mut tstate);
        let mut name: *mut ::core::ffi::c_char = if id == 0 as Integer {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        } else {
            augroup_name(id as ::core::ffi::c_int)
        };
        augroup_del(name, false);
        try_leave(&raw mut tstate, err);
    }
}

pub unsafe extern "C" fn nvim_del_augroup_by_name(mut name: String_0, mut err: *mut Error) {
    unsafe {
        let mut tstate: TryState = TryState {
            current_exception: ::core::ptr::null_mut::<except_T>(),
            private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
            msg_list: ::core::ptr::null::<*const msglist_T>(),
            got_int: 0,
            did_throw: false,
            need_rethrow: 0,
            did_emsg: 0,
        };
        try_enter(&raw mut tstate);
        augroup_del(name.data, false);
        try_leave(&raw mut tstate, err);
    }
}

pub(crate) unsafe extern "C" fn get_augroup_from_object(
    mut group: Object,
    mut err: *mut Error,
) -> ::core::ffi::c_int {
    unsafe {
        let mut au_group: ::core::ffi::c_int = AUGROUP_ERROR as ::core::ffi::c_int;
        let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        match group.type_0 as ::core::ffi::c_uint {
            kObjectTypeNil => return AUGROUP_DEFAULT as ::core::ffi::c_int,
            kObjectTypeString => {
                au_group = augroup_find(group.data.string.data);
                if !(au_group != AUGROUP_ERROR as ::core::ffi::c_int) {
                    api_err_invalid(
                        err,
                        c"group".as_ptr(),
                        group.data.string.data,
                        0 as int64_t,
                        true,
                    );
                    return AUGROUP_ERROR as ::core::ffi::c_int;
                }
                return au_group;
            }
            kObjectTypeInteger => {
                au_group = group.data.integer as ::core::ffi::c_int;
                name = if au_group == 0 as ::core::ffi::c_int {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                } else {
                    augroup_name(au_group)
                };
                if !augroup_exists(name) {
                    api_err_invalid(
                        err,
                        c"group".as_ptr(),
                        ::core::ptr::null::<::core::ffi::c_char>(),
                        au_group as int64_t,
                        false,
                    );
                    return AUGROUP_ERROR as ::core::ffi::c_int;
                }
                return au_group;
            }
            _ => {
                if true {
                    api_err_exp(
                        err,
                        c"group".as_ptr(),
                        c"String or Integer".as_ptr(),
                        api_typename(group.type_0),
                    );
                    return AUGROUP_ERROR as ::core::ffi::c_int;
                }
            }
        }
        panic!("Reached end of non-void function without returning");
    }
}
