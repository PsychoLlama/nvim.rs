//! Creating, deleting and clearing autocommands.
//!
//! `nvim_create_autocmd` is where an api-registered autocommand is born: it
//! resolves the event list, the pattern list and the group, then installs
//! either a command string or a `LuaRef` callback under a fresh id from the
//! parent's `NEXT_AUTOCMD_ID`.  `nvim_clear_autocmds` is the same
//! resolution driving `clear_autocmd` over every match instead.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api::private::validate::{
    err_bad_number, err_bad_value, err_conflict, err_expected, err_required,
};
use crate::cstr;
use crate::memory::ThinCString;
use crate::types::Failed;
use crate::types::Handle;

pub fn nvim_create_autocmd(
    channel_id: uint64_t,
    event: Object,
    opts: &mut KeyDict_create_autocmd,
) -> Result<Integer, Error> {
    let event_array: Array = unpack_string_or_array(Some(event), c"event", true)?;
    // The autocommands hold copies of the handler, so this frame's own
    // reference goes whether or not they were made.
    let mut handler_fn: Callback = Callback::None;
    let made = create_autocmd(channel_id, &event_array, opts, &mut handler_fn);
    callback_free(&mut handler_fn);
    made
}

/// [`nvim_create_autocmd`] once the events are unpacked: one autocommand
/// per (event, pattern), all under the one id it answers. `handler_fn` is
/// the caller's to release.
fn create_autocmd(
    channel_id: uint64_t,
    event_array: &Array,
    opts: &mut KeyDict_create_autocmd,
    handler_fn: &mut Callback,
) -> Result<Integer, Error> {
    let mut handler_cmd: Option<&[u8]> = None;
    if opts.callback.is_some() && opts.command.is_some() {
        return Err(err_conflict(c"callback", c"command"));
    }
    if let Some(given) = opts.callback.as_ref() {
        if let Some(luaref) = given.as_luaref() {
            if luaref == -2 {
                return Err(err_bad_value(c"callback", c"<no value>"));
            }
            if !nlua_ref_is_function(luaref) {
                return Err(err_bad_value(c"callback", c"<not a function>"));
            }
            *handler_fn = Callback::Lua(luaref);
            // The reference is the handler's now, so the keyset must not
            // release it: its value is taken out and the reference travels
            // with `handler_fn`.
            let taken = opts.callback.take();
            let _ = taken.and_then(Object::into_luaref);
        } else if let Some(name) = given.as_string() {
            *handler_fn = Callback::Funcref(::core::mem::ManuallyDrop::new(
                ThinCString::from_bytes(name.as_bytes()),
            ));
        } else {
            let want = c"Lua function or Vim function name";
            let got = api_typename(given.kind());
            return Err(err_expected(c"callback", want, Some(got)));
        }
    } else if let Some(command) = opts.command.as_ref() {
        handler_cmd = Some(command.as_bytes());
    } else {
        return Err(err_required(c"'command' or 'callback'"));
    }
    let au_group = get_augroup_from_object(opts.group.as_ref())?;
    let has_buf = opts.buf.is_some() || opts.buffer.is_some();
    let buf: BufferHandle = opts.buf.or(opts.buffer).unwrap_or(Handle::CURRENT);
    if opts.buf.is_some() && opts.buffer.is_some() {
        return Err(err_conflict(c"buf", c"buffer"));
    }
    if opts.pattern.is_some() && has_buf {
        return Err(err_conflict(c"pattern", c"buf"));
    }
    let patterns: Array =
        get_patterns_from_pattern_or_buf(opts.pattern.as_ref(), has_buf, buf, Some(c"*"))?;
    let desc: Option<&[u8]> = opts.desc.as_ref().map(|given| given.as_bytes());
    if event_array.is_empty() {
        return Err(err_required(c"event"));
    }
    let autocmd_id: int64_t = NEXT_AUTOCMD_ID.get();
    NEXT_AUTOCMD_ID.set(autocmd_id + 1);
    for event_str in event_array.iter() {
        let event_str = event_str
            .as_string()
            .expect("`unpack_string_or_array` answers Strings only");
        let Some(event_nr) = event_name2nr_str(event_str) else {
            return Err(err_bad_value(c"event", event_str.as_cstr()));
        };
        for pat in patterns.iter() {
            let pat = pat
                .as_string()
                .expect("`get_patterns_from_pattern_or_buf` answers Strings only");
            let sctx = api_set_sctx(channel_id);
            let registered = autocmd_register(
                autocmd_id,
                event_nr,
                pat.as_bytes(),
                au_group,
                opts.once.unwrap_or(false),
                opts.nested.unwrap_or(false),
                desc,
                handler_cmd,
                handler_fn,
            );
            drop(sctx);
            registered.map_err(|Failed| Error::exception(c"Failed to set autocmd"))?;
        }
    }
    Ok(autocmd_id)
}

pub fn nvim_del_autocmd(id: Integer) -> Result<(), Error> {
    if id <= 0 {
        return Err(err_bad_number(c"autocmd id", id));
    }
    if !autocmd_delete_id(id) {
        return Err(Error::exception(c"Failed to delete autocmd"));
    }
    Ok(())
}

pub fn nvim_clear_autocmds(opts: &mut KeyDict_clear_autocmds) -> Result<(), Error> {
    let event_array: Array = unpack_string_or_array(opts.event.take(), c"event", false)?;
    let has_buf: bool = opts.buf.is_some() || opts.buffer.is_some();
    let buf = (opts.buf.or(opts.buffer).unwrap_or(Handle::CURRENT)).get();
    if opts.buf.is_some() && opts.buffer.is_some() {
        return Err(err_conflict(c"buf", c"buffer"));
    }
    if opts.pattern.is_some() && has_buf {
        return Err(err_conflict(c"pattern", c"buf"));
    }
    let au_group: ::core::ffi::c_int = get_augroup_from_object(opts.group.as_ref())?;
    let patterns: Array = get_patterns_from_pattern_or_buf(
        opts.pattern.as_ref(),
        has_buf,
        Handle::new(buf),
        Some(c""),
    )?;
    if event_array.len() == 0 as size_t {
        for event in AutoEvent::all() {
            let mut pat_object_index: size_t = 0 as size_t;
            while pat_object_index < patterns.len() {
                let pat_object = &patterns[pat_object_index];
                let pat = pat_object
                    .as_string()
                    .expect("`get_patterns_from_pattern_or_buf` answers Strings only");
                clear_autocmd(event, pat.as_bytes(), au_group)?;
                pat_object_index = pat_object_index.wrapping_add(1);
            }
        }
    } else {
        let mut event_str_index: size_t = 0 as size_t;
        while event_str_index < event_array.len() {
            let event_str = &event_array[event_str_index];
            let event_str = event_str
                .as_string()
                .expect("`unpack_string_or_array` answers Strings only");
            let Some(event_nr) = event_name2nr_str(event_str) else {
                return Err(err_bad_value(c"event", event_str.as_cstr()));
            };
            let mut pat_object_index_0: size_t = 0 as size_t;
            while pat_object_index_0 < patterns.len() {
                let pat_object_0 = &patterns[pat_object_index_0];
                let pat_0 = pat_object_0
                    .as_string()
                    .expect("`get_patterns_from_pattern_or_buf` answers Strings only");
                clear_autocmd(event_nr, pat_0.as_bytes(), au_group)?;
                pat_object_index_0 = pat_object_index_0.wrapping_add(1);
            }
            event_str_index = event_str_index.wrapping_add(1);
        }
    };
    Ok(())
}

/// `nvim_clear_autocmds` for one event and pattern; the pattern reads as
/// far as a NUL, as the C string it was.
fn clear_autocmd(event: AutoEvent, pat: &[u8], au_group: ::core::ffi::c_int) -> Result<(), Error> {
    let pat = cstr::until_nul(pat);
    if do_autocmd_event(event, pat, false, false, b"", true, au_group).is_err() {
        return Err(Error::exception(c"Failed to clear autocmd"));
    }
    Ok(())
}
