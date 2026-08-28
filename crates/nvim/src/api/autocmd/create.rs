//! Creating, deleting and clearing autocommands.
//!
//! `nvim_create_autocmd` is where an api-registered autocommand is born: it
//! resolves the event list, the pattern list and the group, then installs
//! either a command string or a `LuaRef` callback under a fresh id from the
//! parent's `next_autocmd_id`.  `nvim_clear_autocmds` is the same
//! resolution driving `clear_autocmd` over every match instead.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported, has_key};
use crate::types::{FAIL, kObjectTypeLuaRef, kObjectTypeString};
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

/// "Invalid `name`: '`val`'", for a value the caller spelled wrong.
///
/// # Safety
/// `err` must be the caller's error slot and `val` null or a C string.
unsafe fn err_bad_value(err: *mut Error, name: &CStr, val: *const c_char) {
    // SAFETY: the caller's promise; `name` is a C string too.
    unsafe { api_err_invalid(err, name.as_ptr(), val, 0, true) };
}

/// An exception whose whole message is `why`.
///
/// # Safety
/// `err` must be the caller's error slot, and `why` must hold no `%`
/// directive: upstream passes it as the format itself.
unsafe fn err_exception(err: *mut Error, why: &CStr) {
    // SAFETY: the caller's promise.
    unsafe { api_set_error(err, kErrorTypeException, why.as_ptr()) };
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

pub unsafe fn nvim_create_autocmd(
    channel_id: uint64_t,
    event: Object,
    opts: *mut KeyDict_create_autocmd,
    arena: *mut Arena,
) -> Result<Integer, Error> {
    // SAFETY: the dispatcher's keyset outlives this call.
    let opts = unsafe { Live::<KeyDict_create_autocmd>::new(opts) };
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut au_group: ::core::ffi::c_int = 0;
    let mut has_buf: bool = false;
    let mut buf: Buffer = 0;
    let mut patterns: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut autocmd_id: int64_t = -1 as int64_t;
    let mut desc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut handler_cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut handler_fn: Callback = Callback {
        data: Callback_data {
            funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        type_0: kCallbackNone,
    };
    let mut event_array: Array = unsafe {
        unpack_string_or_array(
            event,
            c"event".as_ptr() as *mut ::core::ffi::c_char,
            true,
            arena,
            err,
        )
    };
    '_cleanup: {
        if unsafe { (*err).type_0 } as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            if !(!(has_key(opts.is_set__create_autocmd_, 9 as ::core::ffi::c_int))
                || !(has_key(opts.is_set__create_autocmd_, 7 as ::core::ffi::c_int)))
            {
                unsafe { api_err_conflict(err, c"callback".as_ptr(), c"command".as_ptr()) };
            } else {
                if has_key(
                    opts.is_set__create_autocmd_,
                    KEYSET_OPTIDX_create_autocmd__callback,
                ) {
                    let callback: *mut Object = unsafe { &raw mut (*opts.raw()).callback };
                    match unsafe { (*callback).type_0 } as ::core::ffi::c_uint {
                        kObjectTypeLuaRef => {
                            if !(unsafe { (*callback).data.luaref } != -2 as ::core::ffi::c_int) {
                                // SAFETY: `err` is this call's own error slot.
                                unsafe { err_bad_value(err, c"callback", c"<no value>".as_ptr()) };
                                break '_cleanup;
                            } else if !unsafe { nlua_ref_is_function((*callback).data.luaref) } {
                                // SAFETY: `err` is this call's own error slot.
                                let bad = c"<not a function>".as_ptr();
                                unsafe { err_bad_value(err, c"callback", bad) };
                                break '_cleanup;
                            } else {
                                handler_fn.type_0 = kCallbackLua;
                                handler_fn.data.luaref = unsafe { (*callback).data.luaref };
                                unsafe { (*callback).data.luaref = LUA_NOREF as LuaRef };
                            }
                        }
                        kObjectTypeString => {
                            handler_fn.type_0 = kCallbackFuncref;
                            handler_fn.data.funcref =
                                unsafe { string_to_cstr((*callback).data.string) };
                        }
                        _ => {
                            if true {
                                let want = c"Lua function or Vim function name".as_ptr();
                                // SAFETY: the pointer the caller handed this call.
                                let got = unsafe { api_typename((*callback).type_0) };
                                // SAFETY: `err` is this call's own error slot.
                                unsafe { err_expected(err, c"callback", want, got) };
                                break '_cleanup;
                            }
                        }
                    }
                } else if has_key(
                    opts.is_set__create_autocmd_,
                    KEYSET_OPTIDX_create_autocmd__command,
                ) {
                    handler_cmd = unsafe { string_to_cstr(opts.command) };
                } else if true {
                    unsafe { api_err_required(err, c"'command' or 'callback'".as_ptr()) };
                    break '_cleanup;
                }
                au_group = unsafe { get_augroup_from_object(opts.group, err) };
                if au_group != AUGROUP_ERROR as ::core::ffi::c_int {
                    has_buf = has_key(
                        opts.is_set__create_autocmd_,
                        KEYSET_OPTIDX_create_autocmd__buf,
                    ) || has_key(
                        opts.is_set__create_autocmd_,
                        KEYSET_OPTIDX_create_autocmd__buffer,
                    );
                    buf = if has_key(
                        opts.is_set__create_autocmd_,
                        KEYSET_OPTIDX_create_autocmd__buf,
                    ) {
                        opts.buf
                    } else {
                        opts.buffer
                    };
                    if !(!(has_key(opts.is_set__create_autocmd_, 1 as ::core::ffi::c_int))
                        || !(has_key(opts.is_set__create_autocmd_, 5 as ::core::ffi::c_int)))
                    {
                        unsafe { api_err_conflict(err, c"buf".as_ptr(), c"buffer".as_ptr()) };
                    } else if !(!(has_key(opts.is_set__create_autocmd_, 8 as ::core::ffi::c_int))
                        || !has_buf)
                    {
                        unsafe { api_err_conflict(err, c"pattern".as_ptr(), c"buf".as_ptr()) };
                    } else {
                        patterns = unsafe {
                            get_patterns_from_pattern_or_buf(
                                opts.pattern,
                                has_buf,
                                buf,
                                c"*".as_ptr() as *mut ::core::ffi::c_char,
                                arena,
                                err,
                            )
                        };
                        if unsafe { (*err).type_0 } as ::core::ffi::c_int
                            == kErrorTypeNone as ::core::ffi::c_int
                        {
                            if has_key(
                                opts.is_set__create_autocmd_,
                                KEYSET_OPTIDX_create_autocmd__desc,
                            ) {
                                desc = opts.desc.data();
                            }
                            if !(event_array.size > 0 as size_t) {
                                unsafe { api_err_required(err, c"event".as_ptr()) };
                            } else {
                                autocmd_id = next_autocmd_id.get();
                                next_autocmd_id.set(autocmd_id + 1);
                                let mut event_str_index: size_t = 0 as size_t;
                                loop {
                                    if event_str_index >= event_array.size {
                                        break '_cleanup;
                                    }
                                    let mut event_str: Object =
                                        unsafe { *event_array.items.add(event_str_index) };
                                    let mut event_nr: event_T =
                                        unsafe { event_name2nr_str(event_str.data.string) };
                                    if !((event_nr as ::core::ffi::c_uint)
                                        < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint)
                                    {
                                        // SAFETY: the type tag said this is
                                        // the union's string arm.
                                        let bad = unsafe { event_str.data.string }.data();
                                        // SAFETY: `err` is this call's own error slot.
                                        unsafe { err_bad_value(err, c"event", bad) };
                                        break '_cleanup;
                                    } else {
                                        let mut retval: ::core::ffi::c_int = 0;
                                        let mut pat_index: size_t = 0 as size_t;
                                        while pat_index < patterns.size {
                                            let mut pat: Object =
                                                unsafe { *patterns.items.add(pat_index) };
                                            let sctx = api_set_sctx(channel_id);
                                            retval = unsafe {
                                                autocmd_register(
                                                    autocmd_id,
                                                    event_nr,
                                                    pat.data.string.data(),
                                                    pat.data.string.len() as ::core::ffi::c_int,
                                                    au_group,
                                                    opts.once,
                                                    opts.nested,
                                                    desc,
                                                    handler_cmd,
                                                    &raw mut handler_fn,
                                                )
                                            };
                                            drop(sctx);
                                            if retval == 0 as ::core::ffi::c_int {
                                                let why = c"Failed to set autocmd";
                                                // SAFETY: `err` is this call's own error slot; the
                                                // message holds no `%` directive.
                                                unsafe { err_exception(err, why) };
                                                break '_cleanup;
                                            } else {
                                                pat_index = pat_index.wrapping_add(1);
                                            }
                                        }
                                        event_str_index = event_str_index.wrapping_add(1);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if !handler_cmd.is_null() {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut handler_cmd as *mut *mut ::core::ffi::c_void;
        unsafe { xfree(*ptr_) };
        unsafe { *ptr_ = NULL_0 };
        let _ = unsafe { *ptr_ };
    } else {
        unsafe { callback_free(&raw mut handler_fn) };
    }
    (autocmd_id as Integer).reported(error)
}

pub unsafe fn nvim_del_autocmd(id: Integer) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    if !(id > 0 as Integer) {
        // SAFETY: `err` is this call's own error slot.
        unsafe { err_bad_number(err, c"autocmd id", id) };
        return ().reported(error);
    }
    if !autocmd_delete_id(id as int64_t) {
        let why = c"Failed to delete autocmd";
        // SAFETY: `err` is this call's own error slot; the
        // message holds no `%` directive.
        unsafe { err_exception(err, why) };
    }
    ().reported(error)
}

pub unsafe fn nvim_clear_autocmds(
    opts: *mut KeyDict_clear_autocmds,
    arena: *mut Arena,
) -> Result<(), Error> {
    // SAFETY: the dispatcher's keyset outlives this call.
    let opts = unsafe { Live::<KeyDict_clear_autocmds>::new(opts) };
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut event_array: Array = unsafe {
        unpack_string_or_array(
            opts.event,
            c"event".as_ptr() as *mut ::core::ffi::c_char,
            false,
            arena,
            err,
        )
    };
    if unsafe { (*err).type_0 } as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return ().reported(error);
    }
    let mut has_buf: bool = has_key(
        opts.is_set__clear_autocmds_,
        KEYSET_OPTIDX_clear_autocmds__buf,
    ) || has_key(
        opts.is_set__clear_autocmds_,
        KEYSET_OPTIDX_clear_autocmds__buffer,
    );
    let mut buf: ::core::ffi::c_int = if opts.is_set__clear_autocmds_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_clear_autocmds__buf
        != 0 as ::core::ffi::c_ulonglong
    {
        opts.buf as ::core::ffi::c_int
    } else {
        opts.buffer as ::core::ffi::c_int
    };
    if !(!(has_key(opts.is_set__clear_autocmds_, 1 as ::core::ffi::c_int))
        || !(has_key(opts.is_set__clear_autocmds_, 4 as ::core::ffi::c_int)))
    {
        unsafe { api_err_conflict(err, c"buf".as_ptr(), c"buffer".as_ptr()) };
        return ().reported(error);
    }
    if !(!(has_key(opts.is_set__clear_autocmds_, 5 as ::core::ffi::c_int)) || !has_buf) {
        unsafe { api_err_conflict(err, c"pattern".as_ptr(), c"buf".as_ptr()) };
        return ().reported(error);
    }
    let mut au_group: ::core::ffi::c_int = unsafe { get_augroup_from_object(opts.group, err) };
    if au_group == AUGROUP_ERROR as ::core::ffi::c_int {
        return ().reported(error);
    }
    let mut patterns: Array = unsafe {
        get_patterns_from_pattern_or_buf(
            opts.pattern,
            has_buf,
            buf as Buffer,
            c"".as_ptr() as *mut ::core::ffi::c_char,
            arena,
            err,
        )
    };
    if unsafe { (*err).type_0 } as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return ().reported(error);
    }
    if event_array.size == 0 as size_t {
        let mut event: event_T = EVENT_BUFADD;
        while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
            let mut pat_object_index: size_t = 0 as size_t;
            while pat_object_index < patterns.size {
                let mut pat_object: Object = unsafe { *patterns.items.add(pat_object_index) };
                let mut pat: *mut ::core::ffi::c_char = unsafe { pat_object.data.string }.data();
                if !unsafe { clear_autocmd(event, pat, au_group, err) } {
                    return ().reported(error);
                }
                pat_object_index = pat_object_index.wrapping_add(1);
            }
            event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
        }
    } else {
        let mut event_str_index: size_t = 0 as size_t;
        while event_str_index < event_array.size {
            let mut event_str: Object = unsafe { *event_array.items.add(event_str_index) };
            let mut event_nr: event_T = unsafe { event_name2nr_str(event_str.data.string) };
            if !((event_nr as ::core::ffi::c_uint)
                < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                // SAFETY: `err` is this call's own error slot.
                unsafe { err_bad_value(err, c"event", event_str.data.string.data()) };
                return ().reported(error);
            }
            let mut pat_object_index_0: size_t = 0 as size_t;
            while pat_object_index_0 < patterns.size {
                let mut pat_object_0: Object = unsafe { *patterns.items.add(pat_object_index_0) };
                let mut pat_0: *mut ::core::ffi::c_char =
                    unsafe { pat_object_0.data.string }.data();
                if !unsafe { clear_autocmd(event_nr, pat_0, au_group, err) } {
                    return ().reported(error);
                }
                pat_object_index_0 = pat_object_index_0.wrapping_add(1);
            }
            event_str_index = event_str_index.wrapping_add(1);
        }
    };
    ().reported(error)
}

unsafe fn clear_autocmd(
    mut event: event_T,
    mut pat: *mut ::core::ffi::c_char,
    mut au_group: ::core::ffi::c_int,
    mut err: *mut Error,
) -> bool {
    if unsafe { do_autocmd_event(event, pat, false, 0, c"".as_ptr(), true, au_group) } == FAIL {
        let why = c"Failed to clear autocmd";
        // SAFETY: `err` is this call's own error slot; the
        // message holds no `%` directive.
        unsafe { err_exception(err, why) };
        return false;
    }
    true
}
