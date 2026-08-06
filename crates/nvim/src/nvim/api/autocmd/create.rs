//! Creating, deleting and clearing autocommands.
//!
//! `nvim_create_autocmd` is where an api-registered autocommand is born: it
//! resolves the event list, the pattern list and the group, then installs
//! either a command string or a `LuaRef` callback under a fresh id from the
//! parent's `next_autocmd_id`.  `nvim_clear_autocmds` is the same
//! resolution driving `clear_autocmd` over every match instead.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn nvim_create_autocmd(
    mut channel_id: uint64_t,
    mut event: Object,
    mut opts: *mut KeyDict_create_autocmd,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Integer {
    unsafe {
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
        let mut handler_cmd: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut handler_fn: Callback = Callback {
            data: C2Rust_Unnamed_5 {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        };
        let mut event_array: Array = unpack_string_or_array(
            event,
            b"event\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            true_0 != 0,
            arena,
            err,
        );
        '_cleanup: {
            if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
                if !(!((*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << 9 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_ulonglong)
                    || !((*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << 7 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_ulonglong))
                {
                    api_err_conflict(
                        err,
                        b"callback\0".as_ptr() as *const ::core::ffi::c_char,
                        b"command\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                } else {
                    if (*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_create_autocmd__callback
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        let mut callback: *mut Object = &raw mut (*opts).callback;
                        match (*callback).type_0 as ::core::ffi::c_uint {
                            7 => {
                                if !((*callback).data.luaref != -2 as ::core::ffi::c_int) {
                                    api_err_invalid(
                                        err,
                                        b"callback\0".as_ptr() as *const ::core::ffi::c_char,
                                        b"<no value>\0".as_ptr() as *const ::core::ffi::c_char,
                                        0 as int64_t,
                                        true_0 != 0,
                                    );
                                    break '_cleanup;
                                } else if !nlua_ref_is_function((*callback).data.luaref) {
                                    api_err_invalid(
                                        err,
                                        b"callback\0".as_ptr() as *const ::core::ffi::c_char,
                                        b"<not a function>\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        0 as int64_t,
                                        true_0 != 0,
                                    );
                                    break '_cleanup;
                                } else {
                                    handler_fn.type_0 = kCallbackLua;
                                    handler_fn.data.luaref = (*callback).data.luaref;
                                    (*callback).data.luaref = LUA_NOREF as LuaRef;
                                }
                            }
                            4 => {
                                handler_fn.type_0 = kCallbackFuncref;
                                handler_fn.data.funcref = string_to_cstr((*callback).data.string);
                            }
                            _ => {
                                if true {
                                    api_err_exp(
                                        err,
                                        b"callback\0".as_ptr() as *const ::core::ffi::c_char,
                                        b"Lua function or Vim function name\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        api_typename((*callback).type_0),
                                    );
                                    break '_cleanup;
                                }
                            }
                        }
                    } else if (*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_create_autocmd__command
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        handler_cmd = string_to_cstr((*opts).command);
                    } else if true {
                        api_err_required(
                            err,
                            b"'command' or 'callback'\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                        break '_cleanup;
                    }
                    au_group = get_augroup_from_object((*opts).group, err);
                    if au_group != AUGROUP_ERROR as ::core::ffi::c_int {
                        has_buf = (*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_create_autocmd__buf
                            != 0 as ::core::ffi::c_ulonglong
                            || (*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                                & (1 as ::core::ffi::c_ulonglong)
                                    << KEYSET_OPTIDX_create_autocmd__buffer
                                != 0 as ::core::ffi::c_ulonglong;
                        buf = if (*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_create_autocmd__buf
                            != 0 as ::core::ffi::c_ulonglong
                        {
                            (*opts).buf
                        } else {
                            (*opts).buffer
                        };
                        if !(!((*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
                            != 0 as ::core::ffi::c_ulonglong)
                            || !((*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                                & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                                != 0 as ::core::ffi::c_ulonglong))
                        {
                            api_err_conflict(
                                err,
                                b"buf\0".as_ptr() as *const ::core::ffi::c_char,
                                b"buffer\0".as_ptr() as *const ::core::ffi::c_char,
                            );
                        } else if !(!((*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << 8 as ::core::ffi::c_int
                            != 0 as ::core::ffi::c_ulonglong)
                            || !has_buf)
                        {
                            api_err_conflict(
                                err,
                                b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                                b"buf\0".as_ptr() as *const ::core::ffi::c_char,
                            );
                        } else {
                            patterns = get_patterns_from_pattern_or_buf(
                                (*opts).pattern,
                                has_buf,
                                buf,
                                b"*\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                arena,
                                err,
                            );
                            if (*err).type_0 as ::core::ffi::c_int
                                == kErrorTypeNone as ::core::ffi::c_int
                            {
                                if (*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                                    & (1 as ::core::ffi::c_ulonglong)
                                        << KEYSET_OPTIDX_create_autocmd__desc
                                    != 0 as ::core::ffi::c_ulonglong
                                {
                                    desc = (*opts).desc.data;
                                }
                                if !(event_array.size > 0 as size_t) {
                                    api_err_required(
                                        err,
                                        b"event\0".as_ptr() as *const ::core::ffi::c_char,
                                    );
                                } else {
                                    let c2rust_fresh18 = next_autocmd_id.get();
                                    next_autocmd_id.set(next_autocmd_id.get() + 1);
                                    autocmd_id = c2rust_fresh18;
                                    let mut event_str_index: size_t = 0 as size_t;
                                    loop {
                                        if event_str_index >= event_array.size {
                                            break '_cleanup;
                                        }
                                        let mut event_str: Object =
                                            *event_array.items.offset(event_str_index as isize);
                                        let mut event_nr: event_T =
                                            event_name2nr_str(event_str.data.string);
                                        if !((event_nr as ::core::ffi::c_uint)
                                            < NUM_EVENTS as ::core::ffi::c_int
                                                as ::core::ffi::c_uint)
                                        {
                                            api_err_invalid(
                                                err,
                                                b"event\0".as_ptr() as *const ::core::ffi::c_char,
                                                event_str.data.string.data,
                                                0 as int64_t,
                                                true,
                                            );
                                            break '_cleanup;
                                        } else {
                                            let mut retval: ::core::ffi::c_int = 0;
                                            let mut pat_index: size_t = 0 as size_t;
                                            while pat_index < patterns.size {
                                                let mut pat: Object =
                                                    *patterns.items.offset(pat_index as isize);
                                                let save_current_sctx: sctx_T =
                                                    api_set_sctx(channel_id);
                                                retval = autocmd_register(
                                                    autocmd_id,
                                                    event_nr,
                                                    pat.data.string.data,
                                                    pat.data.string.size as ::core::ffi::c_int,
                                                    au_group,
                                                    (*opts).once as bool,
                                                    (*opts).nested as bool,
                                                    desc,
                                                    handler_cmd,
                                                    &raw mut handler_fn,
                                                );
                                                current_sctx.set(save_current_sctx);
                                                if retval == 0 as ::core::ffi::c_int {
                                                    api_set_error(
                                                        err,
                                                        kErrorTypeException,
                                                        b"Failed to set autocmd\0".as_ptr()
                                                            as *const ::core::ffi::c_char,
                                                    );
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
            xfree(*ptr_);
            *ptr_ = NULL_0;
            let _ = *ptr_;
        } else {
            callback_free(&raw mut handler_fn);
        }
        return autocmd_id as Integer;
    }
}

pub unsafe extern "C" fn nvim_del_autocmd(mut id: Integer, mut err: *mut Error) {
    unsafe {
        if !(id > 0 as Integer) {
            api_err_invalid(
                err,
                b"autocmd id\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<::core::ffi::c_char>(),
                id as int64_t,
                false_0 != 0,
            );
            return;
        }
        if !autocmd_delete_id(id as int64_t) {
            api_set_error(
                err,
                kErrorTypeException,
                b"Failed to delete autocmd\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    }
}

pub unsafe extern "C" fn nvim_clear_autocmds(
    mut opts: *mut KeyDict_clear_autocmds,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    unsafe {
        let mut event_array: Array = unpack_string_or_array(
            (*opts).event,
            b"event\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            false_0 != 0,
            arena,
            err,
        );
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return;
        }
        let mut has_buf: bool = (*opts).is_set__clear_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_clear_autocmds__buf
            != 0 as ::core::ffi::c_ulonglong
            || (*opts).is_set__clear_autocmds_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_clear_autocmds__buffer
                != 0 as ::core::ffi::c_ulonglong;
        let mut buf: ::core::ffi::c_int = if (*opts).is_set__clear_autocmds_
            as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_clear_autocmds__buf
            != 0 as ::core::ffi::c_ulonglong
        {
            (*opts).buf as ::core::ffi::c_int
        } else {
            (*opts).buffer as ::core::ffi::c_int
        };
        if !(!((*opts).is_set__clear_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong)
            || !((*opts).is_set__clear_autocmds_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 4 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_ulonglong))
        {
            api_err_conflict(
                err,
                b"buf\0".as_ptr() as *const ::core::ffi::c_char,
                b"buffer\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
        if !(!((*opts).is_set__clear_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong)
            || !has_buf)
        {
            api_err_conflict(
                err,
                b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                b"buf\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
        let mut au_group: ::core::ffi::c_int = get_augroup_from_object((*opts).group, err);
        if au_group == AUGROUP_ERROR as ::core::ffi::c_int {
            return;
        }
        let mut patterns: Array = get_patterns_from_pattern_or_buf(
            (*opts).pattern,
            has_buf,
            buf as Buffer,
            b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            arena,
            err,
        );
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return;
        }
        if event_array.size == 0 as size_t {
            let mut event: event_T = EVENT_BUFADD;
            while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
                let mut pat_object_index: size_t = 0 as size_t;
                while pat_object_index < patterns.size {
                    let mut pat_object: Object = *patterns.items.offset(pat_object_index as isize);
                    let mut pat: *mut ::core::ffi::c_char = pat_object.data.string.data;
                    if !clear_autocmd(event, pat, au_group, err) {
                        return;
                    }
                    pat_object_index = pat_object_index.wrapping_add(1);
                }
                event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
            }
        } else {
            let mut event_str_index: size_t = 0 as size_t;
            while event_str_index < event_array.size {
                let mut event_str: Object = *event_array.items.offset(event_str_index as isize);
                let mut event_nr: event_T = event_name2nr_str(event_str.data.string);
                if !((event_nr as ::core::ffi::c_uint)
                    < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    api_err_invalid(
                        err,
                        b"event\0".as_ptr() as *const ::core::ffi::c_char,
                        event_str.data.string.data,
                        0 as int64_t,
                        true,
                    );
                    return;
                }
                let mut pat_object_index_0: size_t = 0 as size_t;
                while pat_object_index_0 < patterns.size {
                    let mut pat_object_0: Object =
                        *patterns.items.offset(pat_object_index_0 as isize);
                    let mut pat_0: *mut ::core::ffi::c_char = pat_object_0.data.string.data;
                    if !clear_autocmd(event_nr, pat_0, au_group, err) {
                        return;
                    }
                    pat_object_index_0 = pat_object_index_0.wrapping_add(1);
                }
                event_str_index = event_str_index.wrapping_add(1);
            }
        };
    }
}

unsafe extern "C" fn clear_autocmd(
    mut event: event_T,
    mut pat: *mut ::core::ffi::c_char,
    mut au_group: ::core::ffi::c_int,
    mut err: *mut Error,
) -> bool {
    unsafe {
        if do_autocmd_event(
            event,
            pat,
            false_0 != 0,
            false_0,
            b"\0".as_ptr() as *const ::core::ffi::c_char,
            true_0 != 0,
            au_group,
        ) == FAIL
        {
            api_set_error(
                err,
                kErrorTypeException,
                b"Failed to clear autocmd\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return false_0 != 0;
        }
        return true_0 != 0;
    }
}
