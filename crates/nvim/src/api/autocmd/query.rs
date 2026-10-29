//! `nvim_get_autocmds()`: the whole autocommand table as data.
//!
//! Three filters -- over events, over groups and over patterns -- each of
//! which may be given as a string, an array or not at all, and then every
//! matching command rendered into a Dict of its own.

#![forbid(unsafe_code)]

use super::*;
use crate::api::private::validate::{err_bad_number, err_bad_value, err_conflict, err_expected};
use crate::api_error;

/// The keyset this call was handed, with checked field access.
type Opts = KeyDict_get_autocmds;

/// How many patterns, or buffers, one call may filter on.
const MAX_PATTERNS: usize = 256;
/// How many events there are, which is the width of the event filter.
const EVENT_COUNT: usize = 145;
/// How many keys one command's Dict can grow to.
const DICT_KEYS: size_t = 12;

pub fn nvim_get_autocmds(opts: &mut KeyDict_get_autocmds) -> Result<Array, Error> {
    let group = group_filter(opts)?;
    let id = opts.id.map_or(-1, |id| id as ::core::ffi::c_int);
    let events = event_filter(opts)?;

    let has_buf = opts.buf.is_some() || opts.buffer.is_some();
    if opts.buf.is_some() && opts.buffer.is_some() {
        return Err(err_conflict(c"buf", c"buffer"));
    }
    if opts.pattern.is_some() && has_buf {
        return Err(err_conflict(c"pattern", c"buf"));
    }
    // The `<buffer=N>` strings the pattern filter below borrows from.
    let buffers = buffer_patterns(opts, has_buf)?;
    let patterns = pattern_filters(opts, &buffers)?;

    let mut autocmd_list = Array::EMPTY;
    for event in AutoEvent::all() {
        if events.is_some_and(|wanted| !wanted[event.index()]) {
            continue;
        }
        // Building a row runs no user code, so it is done with the table
        // held; the order is the table's, which is the order they fire in.
        with_autocmds_for_event(event, |cmds| {
            for (ac, ap) in cmds.live() {
                if id != -1 && ac.id != int64_t::from(id) {
                    continue;
                }
                if group != 0 && ap.group != group {
                    continue;
                }
                if !patterns.is_empty() && !matches_pattern(ap, &patterns) {
                    continue;
                }
                autocmd_list.push(Object::dict(autocmd_dict(event, ac, ap)));
            }
        });
    }
    Ok(autocmd_list)
}

/// The `group` key: a name or an id, and `0` for "every group".
fn group_filter(opts: &Opts) -> Result<::core::ffi::c_int, Error> {
    match opts.group.as_ref().unwrap_or(&Object::Nil) {
        Object::Nil => Ok(0),
        Object::String(group_name) => {
            let group = augroup_find(crate::cstr::until_nul(group_name.as_bytes()));
            if group < 0 {
                return Err(err_bad_value(c"group", group_name.as_cstr()));
            }
            Ok(group)
        }
        Object::Integer(group_id) => {
            let group = *group_id as ::core::ffi::c_int;
            // No name is the empty one, as a null `char *` was.
            let name = if group == 0 {
                None
            } else {
                augroup_name(group)
            };
            if !augroup_exists(name.as_deref().unwrap_or_default()) {
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
fn matches_pattern(ap: &AutoPat, filters: &[&::core::ffi::CStr]) -> bool {
    filters.iter().any(|filter| {
        let filter = filter.to_bytes();
        let normalized;
        let pat = if aupat_is_buflocal(filter) {
            normalized = aupat_normalize_buflocal_pat(filter, aupat_get_buflocal_nr(filter));
            &normalized[..]
        } else {
            filter
        };
        ap.pat[..] == *pat
    })
}

/// One command, as the Dict `nvim_get_autocmds` reports it.
fn autocmd_dict(event: AutoEvent, ac: &AutoCmd, ap: &AutoPat) -> ApiDict {
    let mut info = ApiDict::with_capacity(DICT_KEYS);
    if ap.group != AUGROUP_DEFAULT {
        info.insert(c"group", Object::integer(ap.group as Integer));
        let name =
            augroup_name(ap.group).map_or(String_0::NULL, |name| String_0::from_bytes(&name));
        info.insert(c"group_name", Object::string(name));
    }
    if ac.id > 0 {
        info.insert(c"id", Object::integer(ac.id));
    }
    if let Some(desc) = &ac.desc {
        info.insert(c"desc", Object::string(String_0::from_bytes(desc)));
    }
    match &ac.handler {
        AutoHandler::Fn(handler) => {
            info.insert(c"command", Object::string(String_0::NULL));
            match handler {
                Callback::Lua(luaref) => {
                    if nlua_ref_is_function(*luaref) {
                        info.insert(c"callback", Object::luaref(api_new_luaref(*luaref)));
                    }
                }
                Callback::Funcref(_) | Callback::Partial(_) => {
                    let name = String_0::from(callback_to_string(handler).as_cstr());
                    info.insert(c"callback", Object::string(name));
                }
                // A live row with neither a command nor a handler cannot
                // exist.
                Callback::None => std::process::abort(),
            }
        }
        AutoHandler::Cmd(command) => {
            info.insert(c"command", Object::string(String_0::from_bytes(command)));
        }
    }
    info.insert(c"pattern", Object::string(String_0::from_bytes(&ap.pat)));
    let event = String_0::from_cstr(event_nr2name(event));
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
