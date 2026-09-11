//! `nvim_exec_autocmds()`: firing an event by hand.
//!
//! It resolves the same (events, pattern, group, buffer) tuple the other
//! entry points do, then calls `apply_autocmds_group` once per event with
//! the caller's `data` published as `v:event` for the duration.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::api::private::validate::{err_bad_number, err_bad_value, err_conflict, err_expected};
use crate::types::OptionSetFlags;
use crate::winlayer::Buf;
use crate::winlayer::Live;

/// # Safety
///
/// `event` must be a well-formed API object the caller owns for the call.
/// `opts` must point at the `KeyDict_exec_autocmds` the dispatcher filled in,
/// live for the call. `arena` must point at a live arena, which the memory
/// this answers with is taken from and must outlive.
pub unsafe fn nvim_exec_autocmds(
    event: Object,
    opts: *mut KeyDict_exec_autocmds,
) -> Result<(), Error> {
    // SAFETY: the dispatcher's keyset outlives this call.
    let opts = unsafe { Live::<KeyDict_exec_autocmds>::new(opts) };
    let mut error = Error::none();
    let mut au_group: ::core::ffi::c_int = AUGROUP_ALL as ::core::ffi::c_int;
    let mut buffer: Option<Buf> = Buf::current_or_none();
    // The event data is handed on by address; the keyset's own `Option` has
    // no `Object` to point at, so the value is copied into this frame first.
    let mut event_data: Object;
    let mut data: *mut Object = ::core::ptr::null_mut::<Object>();
    let event_array: Array = unsafe {
        unpack_string_or_array(
            Some(event),
            c"event".as_ptr() as *mut ::core::ffi::c_char,
            true,
        )
    }?;
    let name: *mut ::core::ffi::c_char;
    match opts.group.as_ref().unwrap_or(&Object::Nil) {
        Object::Nil => {}
        Object::String(group) => {
            au_group = unsafe { augroup_find(group.data()) };
            if !(au_group != AUGROUP_ERROR as ::core::ffi::c_int) {
                // SAFETY: the value the keyset carried, live for this call.
                error = err_bad_value(c"group", group.as_cstr());
                return ().reported(error);
            }
        }
        Object::Integer(group) => {
            let group = *group;
            au_group = group as ::core::ffi::c_int;
            name = if au_group == 0 as ::core::ffi::c_int {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            } else {
                augroup_name(au_group)
            };
            if !unsafe { augroup_exists(name) } {
                error = err_bad_number(c"group", au_group as int64_t);
                return ().reported(error);
            }
        }
        _ => {
            if true {
                let want = c"String or Integer";
                let got = api_typename(opts.group.as_ref().unwrap_or(&Object::Nil).kind());
                error = err_expected(c"group", want, Some(got));
                return ().reported(error);
            }
        }
    }
    let has_buf: bool = opts.buf.is_some() || opts.buffer.is_some();
    let buf: BufferHandle = opts.buf.or(opts.buffer).unwrap_or(0);
    if opts.buf.is_some() && opts.buffer.is_some() {
        error = err_conflict(c"buf", c"buffer");
        return ().reported(error);
    }
    if has_buf {
        if opts.pattern.is_some() {
            error = err_conflict(c"pattern", c"buf");
            return ().reported(error);
        }
        buffer = find_buffer_by_handle(buf)?;
    }
    let patterns: Array = unsafe {
        get_patterns_from_pattern_or_buf(
            opts.pattern.as_ref(),
            has_buf,
            buf,
            c"".as_ptr() as *mut ::core::ffi::c_char,
        )
    }?;
    if let Some(given) = opts.data.as_ref() {
        event_data = given.clone();
        data = &raw mut event_data;
    }
    let modeline: bool = opts.modeline.unwrap_or(true);
    let mut did_aucmd: bool = false;
    let mut event_str_index: size_t = 0 as size_t;
    while event_str_index < event_array.len() {
        let event_str = &event_array[event_str_index];
        let event_str = event_str
            .as_string()
            .expect("`unpack_string_or_array` answers Strings only");
        let Some(event_nr) = event_name2nr_str(event_str) else {
            // SAFETY: the value the keyset carried, live for this call.
            error = err_bad_value(c"event", event_str.as_cstr());
            return ().reported(error);
        };
        let mut pat_index: size_t = 0 as size_t;
        while pat_index < patterns.len() {
            let pat = &patterns[pat_index];
            let fname: *mut ::core::ffi::c_char = if !has_buf {
                pat.as_string()
                    .expect("`get_patterns_from_pattern_or_buf` answers Strings only")
                    .data()
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
                        buffer,
                        ::core::ptr::null_mut::<ExArg>(),
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
