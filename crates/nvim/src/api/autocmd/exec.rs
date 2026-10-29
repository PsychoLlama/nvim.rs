//! `nvim_exec_autocmds()`: firing an event by hand.
//!
//! It resolves the same (events, pattern, group, buffer) tuple the other
//! entry points do, then calls `apply_autocmds_group` once per event with
//! the caller's `data` published as `v:event` for the duration.

#![forbid(unsafe_code)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::api::private::validate::{err_bad_number, err_bad_value, err_conflict, err_expected};
use crate::types::OptionSetFlags;
use crate::winlayer::Buf;

pub fn nvim_exec_autocmds(event: Object, opts: &mut KeyDict_exec_autocmds) -> Result<(), Error> {
    let mut error = Error::none();
    let mut au_group: ::core::ffi::c_int = AUGROUP_ALL as ::core::ffi::c_int;
    let mut buffer: Option<Buf> = Buf::current_or_none();
    // The event data is handed on by address; the keyset's own `Option` has
    // no `Object` to point at, so the value is copied into this frame first.
    let event_array: Array = unpack_string_or_array(Some(event), c"event", true)?;
    match opts.group.as_ref().unwrap_or(&Object::Nil) {
        Object::Nil => {}
        Object::String(group) => {
            au_group = augroup_find(crate::cstr::until_nul(group.as_bytes()));
            if !(au_group != AUGROUP_ERROR as ::core::ffi::c_int) {
                error = err_bad_value(c"group", group.as_cstr());
                return ().reported(error);
            }
        }
        Object::Integer(group) => {
            let group = *group;
            au_group = group as ::core::ffi::c_int;
            // No name is the empty one, as a null `char *` was.
            let name = if au_group == 0 as ::core::ffi::c_int {
                None
            } else {
                augroup_name(au_group)
            };
            if !augroup_exists(name.as_deref().unwrap_or_default()) {
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
    let patterns: Array =
        get_patterns_from_pattern_or_buf(opts.pattern.as_ref(), has_buf, buf, Some(c""))?;
    let event_data: Option<Object> = opts.data.as_ref().cloned();
    let modeline: bool = opts.modeline.unwrap_or(true);
    let mut did_aucmd: bool = false;
    let mut event_str_index: size_t = 0 as size_t;
    while event_str_index < event_array.len() {
        let event_str = &event_array[event_str_index];
        let event_str = event_str
            .as_string()
            .expect("`unpack_string_or_array` answers Strings only");
        let Some(event_nr) = event_name2nr_str(event_str) else {
            error = err_bad_value(c"event", event_str.as_cstr());
            return ().reported(error);
        };
        let mut pat_index: size_t = 0 as size_t;
        while pat_index < patterns.len() {
            let pat = &patterns[pat_index];
            let fname = if !has_buf {
                Some(
                    pat.as_string()
                        .expect("`get_patterns_from_pattern_or_buf` answers Strings only")
                        .as_cstr(),
                )
            } else {
                None
            };
            did_aucmd = did_aucmd as ::core::ffi::c_int
                | apply_autocmds_group(
                    event_nr,
                    fname,
                    None,
                    true,
                    au_group,
                    buffer,
                    None,
                    event_data.as_ref(),
                ) as ::core::ffi::c_int
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
