//! `nvim_exec_autocmds()`: firing an event by hand.
//!
//! It resolves the same (events, pattern, group, buffer) tuple the other
//! entry points do, then calls `apply_autocmds_group` once per event with
//! the caller's `data` published as `v:event` for the duration.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api::private::validate::{err_bad_number, err_bad_value, err_conflict, err_expected};
use crate::memory::XString;
use crate::narrow::number_as_int;
use crate::option::defs::OptionSetFlags;
use crate::types::Handle;
use crate::winlayer::Buf;

pub fn nvim_exec_autocmds(event: Object, opts: &mut KeyDict_exec_autocmds) -> Result<(), Error> {
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
                return Err(err_bad_value(c"group", group.as_cstr()));
            }
        }
        Object::Integer(group) => {
            let group = *group;
            au_group = number_as_int(group);
            // No name is the empty one, as a null `char *` was.
            let name = if au_group == 0 as ::core::ffi::c_int {
                None
            } else {
                augroup_name(au_group)
            };
            if !augroup_exists(name.as_deref().unwrap_or_default()) {
                return Err(err_bad_number(c"group", int64_t::from(au_group)));
            }
        }
        _ => {
            if true {
                let want = c"String or Integer";
                let got = api_typename(opts.group.as_ref().unwrap_or(&Object::Nil).kind());
                return Err(err_expected(c"group", want, Some(got)));
            }
        }
    }
    let has_buf: bool = opts.buf.is_some() || opts.buffer.is_some();
    let buf: BufferHandle = opts.buf.or(opts.buffer).unwrap_or(Handle::CURRENT);
    if opts.buf.is_some() && opts.buffer.is_some() {
        return Err(err_conflict(c"buf", c"buffer"));
    }
    if has_buf {
        if opts.pattern.is_some() {
            return Err(err_conflict(c"pattern", c"buf"));
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
            return Err(err_bad_value(c"event", event_str.as_cstr()));
        };
        let mut pat_index: size_t = 0 as size_t;
        while pat_index < patterns.len() {
            let pat = &patterns[pat_index];
            let pattern = if !has_buf {
                Some(XString::from_cstr(
                    pat.as_string()
                        .expect("`get_patterns_from_pattern_or_buf` answers Strings only")
                        .as_cstr(),
                ))
            } else {
                None
            };
            did_aucmd = ::core::ffi::c_int::from(did_aucmd)
                | ::core::ffi::c_int::from(apply_autocmds_group(
                    event_nr,
                    pattern,
                    None,
                    true,
                    au_group,
                    buffer,
                    None,
                    event_data.as_ref(),
                ))
                != 0;
            pat_index = pat_index.wrapping_add(1);
        }
        event_str_index = event_str_index.wrapping_add(1);
    }
    if ::core::ffi::c_int::from(did_aucmd) != 0 && ::core::ffi::c_int::from(modeline) != 0 {
        do_modelines(OptionSetFlags::NONE);
    }
    Ok(())
}
