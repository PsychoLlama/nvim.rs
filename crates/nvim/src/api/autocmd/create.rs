//! Creating, deleting and clearing autocommands.
//!
//! `nvim_create_autocmd` is where an api-registered autocommand is born: it
//! resolves the event list, the pattern list and the group, then installs
//! either a command string or a `LuaRef` callback under a fresh id from the
//! parent's `next_autocmd_id`.  `nvim_clear_autocmds` is the same
//! resolution driving `clear_autocmd` over every match instead.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{Reported, has_key};
use crate::api::private::validate::{
    err_bad_number, err_bad_value, err_conflict, err_expected, err_required,
};
use crate::cstr;
use crate::types::Failed;
use crate::winlayer::Live;

pub unsafe fn nvim_create_autocmd(
    channel_id: uint64_t,
    event: Object,
    opts: *mut KeyDict_create_autocmd,
    arena: *mut Arena,
) -> Result<Integer, Error> {
    // SAFETY: the dispatcher's keyset outlives this call.
    let opts = unsafe { Live::<KeyDict_create_autocmd>::new(opts) };
    let mut error = Error::none();
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
    let mut handler_fn: Callback = Callback::None;
    let mut event_array: Array = unsafe {
        unpack_string_or_array(
            event,
            c"event".as_ptr() as *mut ::core::ffi::c_char,
            true,
            arena,
            &mut error,
        )
    };
    '_cleanup: {
        if error.kind() as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            if !(!(has_key(opts.is_set__create_autocmd_, 9 as ::core::ffi::c_int))
                || !(has_key(opts.is_set__create_autocmd_, 7 as ::core::ffi::c_int)))
            {
                error = err_conflict(c"callback", c"command");
            } else {
                if has_key(
                    opts.is_set__create_autocmd_,
                    KEYSET_OPTIDX_create_autocmd__callback,
                ) {
                    let callback: *mut Object = unsafe { &raw mut (*opts.raw()).callback };
                    // SAFETY: the pointer the caller handed this call.
                    match unsafe { *callback } {
                        Object::LuaRef(luaref) => {
                            if !(luaref != -2 as ::core::ffi::c_int) {
                                error = err_bad_value(c"callback", c"<no value>");
                                break '_cleanup;
                            } else if !unsafe { nlua_ref_is_function(luaref) } {
                                let bad = c"<not a function>".as_ptr();
                                // SAFETY: the value the keyset carried, live for this call.
                                error = err_bad_value(c"callback", unsafe { cstr::at(bad) });
                                break '_cleanup;
                            } else {
                                handler_fn = Callback::Lua(luaref);
                                // The reference is the handler's now, so the
                                // keyset must not free it a second time.
                                // SAFETY: the pointer the caller handed this call.
                                unsafe { *callback = Object::LuaRef(LUA_NOREF as LuaRef) };
                            }
                        }
                        Object::String(name) => {
                            handler_fn = Callback::Funcref(unsafe { string_to_cstr(name) });
                        }
                        other => {
                            if true {
                                let want = c"Lua function or Vim function name";
                                let got = api_typename(other.kind());
                                error = err_expected(c"callback", want, Some(got));
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
                    error = err_required(c"'command' or 'callback'");
                    break '_cleanup;
                }
                au_group = unsafe { get_augroup_from_object(opts.group, &mut error) };
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
                        error = err_conflict(c"buf", c"buffer");
                    } else if !(!(has_key(opts.is_set__create_autocmd_, 8 as ::core::ffi::c_int))
                        || !has_buf)
                    {
                        error = err_conflict(c"pattern", c"buf");
                    } else {
                        patterns = unsafe {
                            get_patterns_from_pattern_or_buf(
                                opts.pattern,
                                has_buf,
                                buf,
                                c"*".as_ptr() as *mut ::core::ffi::c_char,
                                arena,
                                &mut error,
                            )
                        };
                        if error.kind() as ::core::ffi::c_int
                            == kErrorTypeNone as ::core::ffi::c_int
                        {
                            if has_key(
                                opts.is_set__create_autocmd_,
                                KEYSET_OPTIDX_create_autocmd__desc,
                            ) {
                                desc = opts.desc.data();
                            }
                            if !(event_array.size > 0 as size_t) {
                                error = err_required(c"event");
                            } else {
                                autocmd_id = next_autocmd_id.get();
                                next_autocmd_id.set(autocmd_id + 1);
                                let mut event_str_index: size_t = 0 as size_t;
                                loop {
                                    if event_str_index >= event_array.size {
                                        break '_cleanup;
                                    }
                                    let event_str: Object =
                                        unsafe { *event_array.items.add(event_str_index) };
                                    let event_str = event_str
                                        .as_string()
                                        .expect("`unpack_string_or_array` answers Strings only");
                                    let Some(event_nr) = (unsafe { event_name2nr_str(event_str) })
                                    else {
                                        let bad = event_str.data();
                                        // SAFETY: the value the keyset carried, live for this call.
                                        error = err_bad_value(c"event", unsafe { cstr::at(bad) });
                                        break '_cleanup;
                                    };
                                    {
                                        let mut retval: Result<(), Failed> = Err(Failed);
                                        let mut pat_index: size_t = 0 as size_t;
                                        while pat_index < patterns.size {
                                            let pat: Object =
                                                unsafe { *patterns.items.add(pat_index) };
                                            let pat = pat.as_string().expect(
                                                "`get_patterns_from_pattern_or_buf` answers \
                                                 Strings only",
                                            );
                                            let sctx = api_set_sctx(channel_id);
                                            retval = unsafe {
                                                autocmd_register(
                                                    autocmd_id,
                                                    event_nr,
                                                    pat.data(),
                                                    pat.len() as ::core::ffi::c_int,
                                                    au_group,
                                                    opts.once,
                                                    opts.nested,
                                                    desc,
                                                    handler_cmd,
                                                    &raw mut handler_fn,
                                                )
                                            };
                                            drop(sctx);
                                            if retval.is_err() {
                                                let why = c"Failed to set autocmd";
                                                error = Error::exception(why);
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
    let mut error = Error::none();
    if !(id > 0 as Integer) {
        error = err_bad_number(c"autocmd id", id);
        return ().reported(error);
    }
    if !autocmd_delete_id(id as int64_t) {
        let why = c"Failed to delete autocmd";
        error = Error::exception(why);
    }
    ().reported(error)
}

pub unsafe fn nvim_clear_autocmds(
    opts: *mut KeyDict_clear_autocmds,
    arena: *mut Arena,
) -> Result<(), Error> {
    // SAFETY: the dispatcher's keyset outlives this call.
    let opts = unsafe { Live::<KeyDict_clear_autocmds>::new(opts) };
    let mut error = Error::none();
    let mut event_array: Array = unsafe {
        unpack_string_or_array(
            opts.event,
            c"event".as_ptr() as *mut ::core::ffi::c_char,
            false,
            arena,
            &mut error,
        )
    };
    if error.kind() as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
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
        error = err_conflict(c"buf", c"buffer");
        return ().reported(error);
    }
    if !(!(has_key(opts.is_set__clear_autocmds_, 5 as ::core::ffi::c_int)) || !has_buf) {
        error = err_conflict(c"pattern", c"buf");
        return ().reported(error);
    }
    let mut au_group: ::core::ffi::c_int =
        unsafe { get_augroup_from_object(opts.group, &mut error) };
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
            &mut error,
        )
    };
    if error.kind() as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return ().reported(error);
    }
    if event_array.size == 0 as size_t {
        for event in AutoEvent::all() {
            let mut pat_object_index: size_t = 0 as size_t;
            while pat_object_index < patterns.size {
                let pat_object: Object = unsafe { *patterns.items.add(pat_object_index) };
                let mut pat: *mut ::core::ffi::c_char = pat_object
                    .as_string()
                    .expect("`get_patterns_from_pattern_or_buf` answers Strings only")
                    .data();
                if !unsafe { clear_autocmd(event, pat, au_group, &mut error) } {
                    return ().reported(error);
                }
                pat_object_index = pat_object_index.wrapping_add(1);
            }
        }
    } else {
        let mut event_str_index: size_t = 0 as size_t;
        while event_str_index < event_array.size {
            let event_str: Object = unsafe { *event_array.items.add(event_str_index) };
            let event_str = event_str
                .as_string()
                .expect("`unpack_string_or_array` answers Strings only");
            let Some(event_nr) = (unsafe { event_name2nr_str(event_str) }) else {
                // SAFETY: the value the keyset carried, live for this call.
                error = err_bad_value(c"event", unsafe { event_str.as_cstr() });
                return ().reported(error);
            };
            let mut pat_object_index_0: size_t = 0 as size_t;
            while pat_object_index_0 < patterns.size {
                let pat_object_0: Object = unsafe { *patterns.items.add(pat_object_index_0) };
                let mut pat_0: *mut ::core::ffi::c_char = pat_object_0
                    .as_string()
                    .expect("`get_patterns_from_pattern_or_buf` answers Strings only")
                    .data();
                if !unsafe { clear_autocmd(event_nr, pat_0, au_group, &mut error) } {
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
    mut event: AutoEvent,
    mut pat: *mut ::core::ffi::c_char,
    mut au_group: ::core::ffi::c_int,
    err: &mut Error,
) -> bool {
    if unsafe { do_autocmd_event(event, pat, false, 0, c"".as_ptr(), true, au_group) }.is_err() {
        let why = c"Failed to clear autocmd";
        *err = Error::exception(why);
        return false;
    }
    true
}
