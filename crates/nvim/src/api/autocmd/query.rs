//! `nvim_get_autocmds()`: the whole autocommand table as data.
//!
//! Three filters -- over events, over groups and over patterns -- each of
//! which may be given as a string, an array or not at all, and then every
//! matching command rendered into a Dict of its own.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::api::private::validate::{err_bad_number, err_bad_value, err_conflict, err_expected};
use crate::api_error;
use crate::winlayer::Live;

/// The keyset this call was handed, with checked field access.
type Opts = Live<KeyDict_get_autocmds>;

/// How many patterns, or buffers, one call may filter on.
const MAX_PATTERNS: usize = 256;
/// How many events there are, which is the width of the event filter.
const EVENT_COUNT: usize = 145;
/// The longest `<buffer=N>` a buffer-local pattern normalises to.
const BUFLOCAL_PAT_LEN: usize = 25;
/// How many keys one command's Dict can grow to.
const DICT_KEYS: size_t = 12;

/// # Safety
///
/// `opts` must point at the `KeyDict_get_autocmds` the dispatcher filled in,
/// live for the call.
pub unsafe fn nvim_get_autocmds(opts: *mut KeyDict_get_autocmds) -> Result<Array, Error> {
    // SAFETY: the dispatcher's keyset outlives this call.
    let opts = unsafe { Live::<KeyDict_get_autocmds>::new(opts) };
    let group = group_filter(&opts)?;
    let id = opts.id.map_or(-1, |id| id as ::core::ffi::c_int);
    let events = event_filter(&opts)?;

    let has_buf = opts.buf.is_some() || opts.buffer.is_some();
    if opts.buf.is_some() && opts.buffer.is_some() {
        return Err(err_conflict(c"buf", c"buffer"));
    }
    if opts.pattern.is_some() && has_buf {
        return Err(err_conflict(c"pattern", c"buf"));
    }
    // The `<buffer=N>` strings the pattern filter below borrows from.
    let buffers = buffer_patterns(&opts, has_buf)?;
    let patterns = pattern_filters(&opts, &buffers)?;

    let mut autocmd_list = Array::EMPTY;
    for event in AutoEvent::all() {
        if events.is_some_and(|wanted| !wanted[event.index()]) {
            continue;
        }
        let acs: *mut AutoCmdVec = au_get_autocmds_for_event(event);
        // SAFETY: `au_get_autocmds_for_event` answers the table's own vector
        // for this event, which nothing below edits.
        for i in 0..unsafe { (*acs).size } {
            // SAFETY: `i` is below `(*acs).size`.
            let ac: *mut AutoCmd = unsafe { (*acs).items.add(i) };
            // SAFETY: as above.
            let ap: *mut AutoPat = unsafe { (*ac).pat };
            if ap.is_null() {
                continue;
            }
            // SAFETY: the command and pattern rows the vector holds, live
            // for as long as the table is not edited.
            let (ac, ap) = unsafe { (&*ac, &*ap) };
            if id != -1 && ac.id != int64_t::from(id) {
                continue;
            }
            if group != 0 && ap.group != group {
                continue;
            }
            if !patterns.is_empty() && !matches_pattern(ap, &patterns) {
                continue;
            }
            // SAFETY: `arena` is the caller's.
            autocmd_list.push(Object::dict(unsafe { autocmd_dict(event, ac, ap) }));
        }
    }
    Ok(autocmd_list)
}

/// The `group` key: a name or an id, and `0` for "every group".
fn group_filter(opts: &Opts) -> Result<::core::ffi::c_int, Error> {
    match opts.group.as_ref().unwrap_or(&Object::Nil) {
        Object::Nil => Ok(0),
        Object::String(group_name) => {
            // SAFETY: the value the keyset carried, live for this call.
            let group = unsafe { augroup_find(group_name.data()) };
            if group < 0 {
                // SAFETY: as above.
                return Err(err_bad_value(c"group", group_name.as_cstr()));
            }
            Ok(group)
        }
        Object::Integer(group_id) => {
            let group = *group_id as ::core::ffi::c_int;
            let name = if group == 0 {
                ::core::ptr::null_mut()
            } else {
                augroup_name(group)
            };
            // SAFETY: `augroup_name` answers a C string or null.
            if !unsafe { augroup_exists(name) } {
                return Err(err_bad_number(c"group", *group_id));
            }
            Ok(group)
        }
        other => {
            let want = c"String or Integer";
            Err(err_expected(
                c"group",
                want,
                Some(api_typename(other.kind())),
            ))
        }
    }
}

/// The `event` key: which events to report, or `None` for all of them.
fn event_filter(opts: &Opts) -> Result<Option<[bool; EVENT_COUNT]>, Error> {
    let Some(given) = opts.event.as_ref() else {
        return Ok(None);
    };
    let mut wanted = [false; EVENT_COUNT];
    if let Some(event_name) = given.as_string() {
        let Some(event) = event_name2nr_str(event_name) else {
            // SAFETY: the value the keyset carried, live for this call.
            return Err(err_bad_value(c"event", event_name.as_cstr()));
        };
        wanted[event.index()] = true;
        return Ok(Some(wanted));
    }
    let Some(given) = given.as_array() else {
        return Err(err_expected(c"event", c"String or Array", None));
    };
    for item in given.iter() {
        let Some(event_name) = item.as_string() else {
            let want = api_typename(kObjectTypeString);
            let got = api_typename(item.kind());
            return Err(err_expected(c"event item", want, Some(got)));
        };
        let Some(event) = event_name2nr_str(event_name) else {
            // SAFETY: the value the keyset carried, live for this call.
            return Err(err_bad_value(c"event", event_name.as_cstr()));
        };
        wanted[event.index()] = true;
    }
    Ok(Some(wanted))
}

/// The `buf`/`buffer` key, as the `<buffer=N>` patterns it stands for.
fn buffer_patterns(opts: &Opts, has_buf: bool) -> Result<Array, Error> {
    let Some(given) = opts.buf.as_ref().or(opts.buffer.as_ref()) else {
        return Ok(Array::EMPTY);
    };
    let buflocal = |handle: Integer| -> Result<Object, Error> {
        let b = find_buffer_by_handle(handle as BufferHandle)?;
        let pat = format!("<buffer={}>", b.map_or(0, |b| b.handle));
        Ok(Object::string(String_0::from_bytes(pat.as_bytes())))
    };
    if let Object::Integer(handle) | Object::Buffer(handle) = given {
        let mut buffers = Array::with_capacity(1);
        buffers.push(buflocal(*handle)?);
        return Ok(buffers);
    }
    let Some(handles) = given.as_array() else {
        if !has_buf {
            return Ok(Array::EMPTY);
        }
        let want = c"Integer or Array";
        let got = api_typename(given.kind());
        return Err(err_expected(c"buffer", want, Some(got)));
    };
    if handles.len() > MAX_PATTERNS {
        let max = MAX_PATTERNS;
        return Err(api_error!(
            kErrorTypeValidation,
            "Too many buffers (maximum of {max})"
        ));
    }
    let mut buffers = Array::with_capacity(handles.len());
    for item in handles.iter() {
        let (Object::Integer(handle) | Object::Buffer(handle)) = item else {
            let got = api_typename(item.kind());
            return Err(err_expected(c"buffer", c"Integer", Some(got)));
        };
        buffers.push(buflocal(*handle)?);
    }
    Ok(buffers)
}

/// The patterns a command's own is compared against: the `pattern` key's
/// globs, plus one `<buffer=N>` per buffer the `buf` key named.
///
/// Each borrows the string it came from, so the answer lives only as long as
/// `opts` and `buffers` do.
fn pattern_filters<'a>(
    opts: &'a Opts,
    buffers: &'a Array,
) -> Result<Vec<&'a ::core::ffi::CStr>, Error> {
    let mut filters: Vec<&::core::ffi::CStr> = Vec::new();
    if let Some(given) = opts.pattern.as_ref() {
        if let Some(pattern) = given.as_string() {
            filters.push(pattern.as_cstr());
        } else if let Some(patterns) = given.as_array() {
            if patterns.len() > MAX_PATTERNS {
                let max = MAX_PATTERNS;
                return Err(api_error!(
                    kErrorTypeValidation,
                    "Too many patterns (maximum of {max})"
                ));
            }
            for item in patterns.iter() {
                let Some(pattern) = item.as_string() else {
                    let want = api_typename(kObjectTypeString);
                    let got = api_typename(item.kind());
                    return Err(err_expected(c"pattern", want, Some(got)));
                };
                filters.push(pattern.as_cstr());
            }
        } else {
            let want = c"String or Array";
            let got = api_typename(given.kind());
            return Err(err_expected(c"pattern", want, Some(got)));
        }
    }
    for buffer in buffers.iter() {
        let pattern = buffer
            .as_string()
            .expect("`buffers` was filled with `<buffer=N>` Strings");
        filters.push(pattern.as_cstr());
    }
    Ok(filters)
}

/// Whether `ap`'s pattern is one of `filters`.
///
/// A buffer-local filter is normalised first: `<buffer>` and `<buffer=abuf>`
/// both mean one particular buffer, and the stored pattern spells it out.
///
fn matches_pattern(ap: &AutoPat, filters: &[&::core::ffi::CStr]) -> bool {
    filters.iter().any(|filter| {
        let patlen = filter.count_bytes() as ::core::ffi::c_int;
        let mut normalized = [0 as ::core::ffi::c_char; BUFLOCAL_PAT_LEN];
        // SAFETY: `filter` is a C string of `patlen` bytes, and `normalized`
        // is this frame's buffer, which is what the normalised form fits in.
        let pat = unsafe {
            if aupat_is_buflocal(filter.as_ptr(), patlen) {
                let dest = (&raw mut normalized).cast::<::core::ffi::c_char>();
                let nr = aupat_get_buflocal_nr(filter.as_ptr(), patlen);
                aupat_normalize_buflocal_pat(dest, filter.as_ptr(), patlen, nr);
                dest.cast_const()
            } else {
                filter.as_ptr()
            }
        };
        // SAFETY: the row's own pattern, and a C string either way.
        unsafe { strequal(ap.pat, pat) }
    })
}

/// One command, as the Dict `nvim_get_autocmds` reports it.
///
/// # Safety
/// A `Partial` handler's pointer must name a live partial.
unsafe fn autocmd_dict(event: AutoEvent, ac: &AutoCmd, ap: &AutoPat) -> ApiDict {
    // Every C string read below is either a row's own or a static name, so
    // each `cstr_to_string` copies out of a live one.
    let mut info = ApiDict::with_capacity(DICT_KEYS);
    if ap.group != AUGROUP_DEFAULT {
        info.insert(c"group", Object::integer(ap.group as Integer));
        // SAFETY: `augroup_name` answers a C string for a live group.
        let name = unsafe { cstr_to_string(augroup_name(ap.group)) };
        info.insert(c"group_name", Object::string(name));
    }
    if ac.id > 0 {
        info.insert(c"id", Object::integer(ac.id));
    }
    if !ac.desc.is_null() {
        // SAFETY: the row's own description.
        info.insert(c"desc", Object::string(unsafe { cstr_to_string(ac.desc) }));
    }
    if ac.handler_cmd.is_null() {
        info.insert(c"command", Object::string(String_0::NULL));
        match &ac.handler_fn {
            Callback::Lua(luaref) => {
                if nlua_ref_is_function(*luaref) {
                    info.insert(c"callback", Object::luaref(api_new_luaref(*luaref)));
                }
            }
            handler @ (Callback::Funcref(_) | Callback::Partial(_)) => {
                // SAFETY: the caller's promise about a partial, and `arena`.
                let name = unsafe { cstr_to_string(callback_to_string(handler)) };
                info.insert(c"callback", Object::string(name));
            }
            // A row with neither a command nor a handler cannot exist.
            // SAFETY: `abort` only ever ends the process.
            Callback::None => unsafe { abort() },
        }
    } else {
        // SAFETY: the row's own command.
        let command = unsafe { cstr_to_string(ac.handler_cmd) };
        info.insert(c"command", Object::string(command));
    }
    // SAFETY: the pattern row's own string.
    info.insert(
        c"pattern",
        Object::string(unsafe { cstr_to_string(ap.pat) }),
    );
    // SAFETY: `event_nr2name` answers a static C string.
    let event = unsafe { cstr_to_string(event_nr2name(event)) };
    info.insert(c"event", Object::string(event));
    info.insert(c"once", Object::boolean(ac.once));
    if ap.buflocal_nr == 0 {
        info.insert(c"buflocal", Object::boolean(false));
    } else {
        info.insert(c"buflocal", Object::boolean(true));
        info.insert(c"buf", Object::integer(ap.buflocal_nr as Integer));
        info.insert(c"buffer", Object::integer(ap.buflocal_nr as Integer));
    }
    info
}
