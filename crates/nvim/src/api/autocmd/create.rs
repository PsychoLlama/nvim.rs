//! Creating, deleting and clearing autocommands.
//!
//! `nvim_create_autocmd` is where an api-registered autocommand is born: it
//! resolves the event list, the pattern list and the group, then installs
//! either a command string or a `LuaRef` callback under a fresh id from the
//! parent's `next_autocmd_id`.  `nvim_clear_autocmds` is the same
//! resolution driving `clear_autocmd` over every match instead.

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
use crate::api::private::validate::{
    err_bad_number, err_bad_value, err_conflict, err_expected, err_required,
};
use crate::cstr;
use crate::narrow::len_as_int;
use crate::types::Failed;
use crate::winlayer::Live;

/// # Safety
///
/// `event` must be a well-formed API object the caller owns for the call.
/// `opts` must point at the `KeyDict_create_autocmd` the dispatcher filled
/// in, live for the call. `arena` must point at a live arena, which the
/// memory this answers with is taken from and must outlive.
pub unsafe fn nvim_create_autocmd(
    channel_id: uint64_t,
    event: Object,
    opts: *mut KeyDict_create_autocmd,
    arena: *mut Arena,
) -> Result<Integer, Error> {
    // SAFETY: the dispatcher's keyset outlives this call.
    let opts = unsafe { Live::<KeyDict_create_autocmd>::new(opts) };
    let mut error = Error::none();
    let au_group: ::core::ffi::c_int;
    let has_buf: bool;
    let buf: BufferHandle;
    let patterns: Array;
    let mut autocmd_id: int64_t = -1 as int64_t;
    let mut desc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut handler_cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut handler_fn: Callback = Callback::None;
    let event_array: Array = unsafe {
        unpack_string_or_array(
            event,
            c"event".as_ptr() as *mut ::core::ffi::c_char,
            true,
            arena,
        )
    }?;
    '_cleanup: {
        {
            if opts.callback.is_some() && opts.command.is_some() {
                error = err_conflict(c"callback", c"command");
            } else {
                if let Some(given) = opts.callback {
                    let callback: *mut Option<Object> = unsafe { &raw mut (*opts.raw()).callback };
                    match given {
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
                                unsafe { *callback = Some(Object::LuaRef(LUA_NOREF as LuaRef)) };
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
                } else if let Some(command) = opts.command {
                    handler_cmd = unsafe { string_to_cstr(command) };
                } else if true {
                    error = err_required(c"'command' or 'callback'");
                    break '_cleanup;
                }
                au_group =
                    match unsafe { get_augroup_from_object(opts.group.unwrap_or(Object::Nil)) } {
                        Ok(au_group) => au_group,
                        Err(e) => {
                            error = e;
                            AUGROUP_ERROR as ::core::ffi::c_int
                        }
                    };
                if au_group != AUGROUP_ERROR as ::core::ffi::c_int {
                    has_buf = opts.buf.is_some() || opts.buffer.is_some();
                    buf = opts.buf.or(opts.buffer).unwrap_or(0);
                    if opts.buf.is_some() && opts.buffer.is_some() {
                        error = err_conflict(c"buf", c"buffer");
                    } else if opts.pattern.is_some() && has_buf {
                        error = err_conflict(c"pattern", c"buf");
                    } else {
                        patterns = match unsafe {
                            get_patterns_from_pattern_or_buf(
                                opts.pattern.unwrap_or(Object::Nil),
                                has_buf,
                                buf,
                                c"*".as_ptr() as *mut ::core::ffi::c_char,
                                arena,
                            )
                        } {
                            Ok(patterns) => patterns,
                            Err(e) => {
                                error = e;
                                break '_cleanup;
                            }
                        };
                        {
                            if let Some(given) = opts.desc {
                                desc = given.data();
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
                                        let mut retval: Result<(), Failed>;
                                        let mut pat_index: size_t = 0 as size_t;
                                        while pat_index < patterns.size {
                                            let pat: Object =
                                                unsafe { *patterns.items.add(pat_index) };
                                            let pat = pat.as_string().expect(
                                                "`get_patterns_from_pattern_or_buf` answers \
                                                 Strings only",
                                            );
                                            let sctx = api_set_sctx(channel_id);
                                            let patlen = len_as_int(pat.len());
                                            retval = unsafe {
                                                autocmd_register(
                                                    autocmd_id,
                                                    event_nr,
                                                    pat.data(),
                                                    patlen,
                                                    au_group,
                                                    opts.once.unwrap_or(false),
                                                    opts.nested.unwrap_or(false),
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
        let ptr_: *mut *mut ::core::ffi::c_void =
            (&raw mut handler_cmd).cast::<*mut ::core::ffi::c_void>();
        unsafe { xfree(*ptr_) };
        unsafe { *ptr_ = NULL_0 };
        let _ = unsafe { *ptr_ };
    } else {
        unsafe { callback_free(&raw mut handler_fn) };
    }
    (autocmd_id as Integer).reported(error)
}

pub fn nvim_del_autocmd(id: Integer) -> Result<(), Error> {
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

/// # Safety
///
/// `opts` must point at the `KeyDict_clear_autocmds` the dispatcher filled
/// in, live for the call. `arena` must point at a live arena, which the
/// memory this answers with is taken from and must outlive.
pub unsafe fn nvim_clear_autocmds(
    opts: *mut KeyDict_clear_autocmds,
    arena: *mut Arena,
) -> Result<(), Error> {
    // SAFETY: the dispatcher's keyset outlives this call.
    let opts = unsafe { Live::<KeyDict_clear_autocmds>::new(opts) };
    let mut error = Error::none();
    let event_array: Array = unsafe {
        unpack_string_or_array(
            opts.event.unwrap_or(Object::Nil),
            c"event".as_ptr() as *mut ::core::ffi::c_char,
            false,
            arena,
        )
    }?;
    let has_buf: bool = opts.buf.is_some() || opts.buffer.is_some();
    let buf = opts.buf.or(opts.buffer).unwrap_or(0) as ::core::ffi::c_int;
    if opts.buf.is_some() && opts.buffer.is_some() {
        error = err_conflict(c"buf", c"buffer");
        return ().reported(error);
    }
    if opts.pattern.is_some() && has_buf {
        error = err_conflict(c"pattern", c"buf");
        return ().reported(error);
    }
    let group = opts.group.unwrap_or(Object::Nil);
    let au_group: ::core::ffi::c_int = unsafe { get_augroup_from_object(group) }?;
    let patterns: Array = unsafe {
        get_patterns_from_pattern_or_buf(
            opts.pattern.unwrap_or(Object::Nil),
            has_buf,
            buf as BufferHandle,
            c"".as_ptr() as *mut ::core::ffi::c_char,
            arena,
        )
    }?;
    if event_array.size == 0 as size_t {
        for event in AutoEvent::all() {
            let mut pat_object_index: size_t = 0 as size_t;
            while pat_object_index < patterns.size {
                let pat_object: Object = unsafe { *patterns.items.add(pat_object_index) };
                let pat: *mut ::core::ffi::c_char = pat_object
                    .as_string()
                    .expect("`get_patterns_from_pattern_or_buf` answers Strings only")
                    .data();
                unsafe { clear_autocmd(event, pat, au_group) }?;
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
                let pat_0: *mut ::core::ffi::c_char = pat_object_0
                    .as_string()
                    .expect("`get_patterns_from_pattern_or_buf` answers Strings only")
                    .data();
                unsafe { clear_autocmd(event_nr, pat_0, au_group) }?;
                pat_object_index_0 = pat_object_index_0.wrapping_add(1);
            }
            event_str_index = event_str_index.wrapping_add(1);
        }
    };
    ().reported(error)
}

/// # Safety
///
/// `pat` must point at a NUL-terminated string, unaliased for the call.
unsafe fn clear_autocmd(
    event: AutoEvent,
    pat: *mut ::core::ffi::c_char,
    au_group: ::core::ffi::c_int,
) -> Result<(), Error> {
    if unsafe { do_autocmd_event(event, pat, false, 0, c"".as_ptr(), true, au_group) }.is_err() {
        return Err(Error::exception(c"Failed to clear autocmd"));
    }
    Ok(())
}
