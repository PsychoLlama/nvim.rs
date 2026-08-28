//! `nvim_exec_autocmds()`: firing an event by hand.
//!
//! It resolves the same (events, pattern, group, buffer) tuple the other
//! entry points do, then calls `apply_autocmds_group` once per event with
//! the caller's `data` published as `v:event` for the duration.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported, has_key};
use crate::types::OptionSetFlags;
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

/// "Invalid `name`: expected `want`", naming `got` when it says.
///
/// # Safety
/// `err` must be the caller's error slot, `want` a C string and `got` null
/// or a C string.
unsafe fn err_expected(err: *mut Error, name: &CStr, want: *const c_char, got: *const c_char) {
    // SAFETY: the caller's promise; `name` is a C string too.
    unsafe { api_err_exp(err, name.as_ptr(), want, got) };
}

pub unsafe fn nvim_exec_autocmds(
    event: Object,
    opts: *mut KeyDict_exec_autocmds,
    arena: *mut Arena,
) -> Result<(), Error> {
    // SAFETY: the dispatcher's keyset outlives this call.
    let opts = unsafe { Live::<KeyDict_exec_autocmds>::new(opts) };
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut au_group: ::core::ffi::c_int = AUGROUP_ALL as ::core::ffi::c_int;
    let mut modeline: bool = true;
    let mut b: *mut buf_T = curbuf.get();
    let mut data: *mut Object = ::core::ptr::null_mut::<Object>();
    let mut event_array: Array = unsafe {
        unpack_string_or_array(
            event,
            c"event".as_ptr() as *mut ::core::ffi::c_char,
            true,
            arena,
            err,
        )
    };
    if unsafe { (*err).type_0 } as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return ().reported(error);
    }
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    match opts.group.type_0 as ::core::ffi::c_uint {
        kObjectTypeNil => {}
        kObjectTypeString => {
            au_group = unsafe { augroup_find(opts.group.data.string.data()) };
            if !(au_group != AUGROUP_ERROR as ::core::ffi::c_int) {
                // SAFETY: `err` is this call's own error slot.
                unsafe { err_bad_value(err, c"group", opts.group.data.string.data()) };
                return ().reported(error);
            }
        }
        kObjectTypeInteger => {
            // SAFETY: the type tag says this arm's union field is the live one.
            au_group = unsafe { opts.group.data.integer } as ::core::ffi::c_int;
            name = if au_group == 0 as ::core::ffi::c_int {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            } else {
                augroup_name(au_group)
            };
            if !unsafe { augroup_exists(name) } {
                // SAFETY: `err` is this call's own error slot.
                unsafe { err_bad_number(err, c"group", au_group as int64_t) };
                return ().reported(error);
            }
        }
        _ => {
            if true {
                let want = c"String or Integer".as_ptr();
                let got = api_typename(opts.group.type_0);
                // SAFETY: `err` is this call's own error slot.
                unsafe { err_expected(err, c"group", want, got) };
                return ().reported(error);
            }
        }
    }
    let mut has_buf: bool = has_key(
        opts.is_set__exec_autocmds_,
        KEYSET_OPTIDX_exec_autocmds__buf,
    ) || has_key(
        opts.is_set__exec_autocmds_,
        KEYSET_OPTIDX_exec_autocmds__buffer,
    );
    let mut buf: Buffer = if has_key(
        opts.is_set__exec_autocmds_,
        KEYSET_OPTIDX_exec_autocmds__buf,
    ) {
        opts.buf
    } else {
        opts.buffer
    };
    if !(!(has_key(opts.is_set__exec_autocmds_, 1 as ::core::ffi::c_int))
        || !(has_key(opts.is_set__exec_autocmds_, 4 as ::core::ffi::c_int)))
    {
        unsafe { api_err_conflict(err, c"buf".as_ptr(), c"buffer".as_ptr()) };
        return ().reported(error);
    }
    if has_buf {
        if has_key(opts.is_set__exec_autocmds_, 5 as ::core::ffi::c_int) {
            unsafe { api_err_conflict(err, c"pattern".as_ptr(), c"buf".as_ptr()) };
            return ().reported(error);
        }
        b = unsafe { find_buffer_by_handle(buf, err) };
        if unsafe { (*err).type_0 } as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return ().reported(error);
        }
    }
    let mut patterns: Array = unsafe {
        get_patterns_from_pattern_or_buf(
            opts.pattern,
            has_buf,
            buf,
            c"".as_ptr() as *mut ::core::ffi::c_char,
            arena,
            err,
        )
    };
    if unsafe { (*err).type_0 } as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return ().reported(error);
    }
    if has_key(
        opts.is_set__exec_autocmds_,
        KEYSET_OPTIDX_exec_autocmds__data,
    ) {
        data = unsafe { &raw mut (*opts.raw()).data };
    }
    modeline = if has_key(
        opts.is_set__exec_autocmds_,
        KEYSET_OPTIDX_exec_autocmds__modeline,
    ) {
        opts.modeline as ::core::ffi::c_int
    } else {
        1
    } != 0;
    let mut did_aucmd: bool = false;
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
        let mut pat_index: size_t = 0 as size_t;
        while pat_index < patterns.size {
            let mut pat: Object = unsafe { *patterns.items.add(pat_index) };
            let mut fname: *mut ::core::ffi::c_char = if !has_buf {
                unsafe { pat.data.string }.data()
            } else {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            };
            did_aucmd = did_aucmd as ::core::ffi::c_int
                | unsafe {
                    apply_autocmds_group(
                        event_nr,
                        fname,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        true,
                        au_group,
                        b,
                        ::core::ptr::null_mut::<exarg_T>(),
                        data,
                    )
                } as ::core::ffi::c_int
                != 0;
            pat_index = pat_index.wrapping_add(1);
        }
        event_str_index = event_str_index.wrapping_add(1);
    }
    if did_aucmd as ::core::ffi::c_int != 0 && modeline as ::core::ffi::c_int != 0 {
        do_modelines(OptionSetFlags::NONE);
    }
    ().reported(error)
}
