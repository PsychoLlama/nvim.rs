//! `nvim_get_autocmds()`: the whole autocommand table as data.
//!
//! One function, and the longest in the family, because it is three nested
//! filters -- over events, over groups and over patterns -- each of which
//! may be given as a string, an array or not at all, and because every
//! matching command is rendered into a Dict of its own.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{Reported, array_add, dict_put, dict_put_str, has_key};
use crate::api::private::validate::{err_bad_number, err_bad_value, err_conflict, err_expected};
use crate::api_error;
use crate::cstr;
use crate::kvec::InitVec;
use crate::winlayer::Live;

pub unsafe fn nvim_get_autocmds(
    opts: *mut KeyDict_get_autocmds,
    arena: *mut Arena,
) -> Result<Array, Error> {
    // SAFETY: the dispatcher's keyset outlives this call.
    let opts = unsafe { Live::<KeyDict_get_autocmds>::new(opts) };
    let mut error = Error::none();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut id: ::core::ffi::c_int = 0;
    let mut has_buf: bool = false;
    let mut buf: Object = Object::Nil;
    let mut pattern_filter_count: ::core::ffi::c_int = 0;
    let mut autocmd_list: ArrayBuilder = ArrayBuilder {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
        init_array: [Object::Nil; 16],
    };
    autocmd_list.capacity = ::core::mem::size_of::<[Object; 16]>()
        .wrapping_div(::core::mem::size_of::<Object>())
        .wrapping_div(
            (::core::mem::size_of::<[Object; 16]>().wrapping_rem(::core::mem::size_of::<Object>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    autocmd_list.size = 0 as size_t;
    autocmd_list.items = &raw mut autocmd_list.init_array as *mut Object;
    let mut pattern_filters: [*mut ::core::ffi::c_char; 256] =
        [::core::ptr::null_mut::<::core::ffi::c_char>(); 256];
    let mut buffers: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut event_set: [bool; 145] = [false; 145];
    let mut check_event: bool = false;
    let mut group: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    '_cleanup: {
        match opts.group {
            Object::Nil => {}
            Object::String(group_name) => {
                group = unsafe { augroup_find(group_name.data()) };
                if !(group >= 0 as ::core::ffi::c_int) {
                    // SAFETY: the value the keyset carried, live for this call.
                    let name = unsafe { group_name.as_cstr() };
                    error = err_bad_value(c"group", name);
                    break '_cleanup;
                }
            }
            Object::Integer(group_id) => {
                group = group_id as ::core::ffi::c_int;
                name = if group == 0 as ::core::ffi::c_int {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                } else {
                    augroup_name(group)
                };
                if !unsafe { augroup_exists(name) } {
                    error = err_bad_number(c"group", group_id);
                    break '_cleanup;
                }
            }
            _ => {
                if true {
                    let want = c"String or Integer";
                    let got = api_typename(opts.group.kind());
                    error = err_expected(c"group", want, Some(got));
                    break '_cleanup;
                }
            }
        }
        id = if has_key(opts.is_set__get_autocmds_, KEYSET_OPTIDX_get_autocmds__id) {
            opts.id as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
        's_299: {
            if has_key(
                opts.is_set__get_autocmds_,
                KEYSET_OPTIDX_get_autocmds__event,
            ) {
                check_event = true;
                let v: Object = opts.event;
                if let Object::String(event_name) = v {
                    let Some(event_nr) = (unsafe { event_name2nr_str(event_name) }) else {
                        // SAFETY: the value the keyset carried, live for this call.
                        error = err_bad_value(c"event", unsafe { event_name.as_cstr() });
                        break '_cleanup;
                    };
                    event_set[event_nr.index()] = true;
                } else if let Object::Array(events) = v {
                    let mut event_v_index: size_t = 0 as size_t;
                    loop {
                        if event_v_index >= events.size {
                            break 's_299;
                        }
                        let event_v: Object = unsafe { *events.items.add(event_v_index) };
                        let Some(event_v) = event_v.as_string() else {
                            let want = api_typename(kObjectTypeString);
                            let got = api_typename(event_v.kind());
                            error = err_expected(c"event item", want, Some(got));
                            break '_cleanup;
                        };
                        let Some(event_nr_0) = (unsafe { event_name2nr_str(event_v) }) else {
                            // SAFETY: the value the keyset carried, live for this call.
                            let name = unsafe { event_v.as_cstr() };
                            error = err_bad_value(c"event", name);
                            break '_cleanup;
                        };
                        event_set[event_nr_0.index()] = true;
                        event_v_index = event_v_index.wrapping_add(1);
                    }
                } else if true {
                    let want = c"String or Array";
                    error = err_expected(c"event", want, None);
                    break '_cleanup;
                }
            }
        }
        has_buf = has_key(opts.is_set__get_autocmds_, KEYSET_OPTIDX_get_autocmds__buf)
            || has_key(
                opts.is_set__get_autocmds_,
                KEYSET_OPTIDX_get_autocmds__buffer,
            );
        buf = if has_key(opts.is_set__get_autocmds_, KEYSET_OPTIDX_get_autocmds__buf) {
            opts.buf
        } else {
            opts.buffer
        };
        if !(!(has_key(opts.is_set__get_autocmds_, 2 as ::core::ffi::c_int))
            || !(has_key(opts.is_set__get_autocmds_, 5 as ::core::ffi::c_int)))
        {
            error = err_conflict(c"buf", c"buffer");
        } else if !(!(has_key(opts.is_set__get_autocmds_, 6 as ::core::ffi::c_int)) || !has_buf) {
            error = err_conflict(c"pattern", c"buf");
        } else {
            pattern_filter_count = 0 as ::core::ffi::c_int;
            's_506: {
                if has_key(
                    opts.is_set__get_autocmds_,
                    KEYSET_OPTIDX_get_autocmds__pattern,
                ) {
                    let v_0: Object = opts.pattern;
                    if let Object::String(pattern) = v_0 {
                        pattern_filters[pattern_filter_count as usize] = pattern.data();
                        pattern_filter_count += 1 as ::core::ffi::c_int;
                    } else if let Object::Array(pattern_list) = v_0 {
                        if !(pattern_list.size <= 256 as size_t) {
                            let max = 256 as ::core::ffi::c_int;
                            error = api_error!(
                                kErrorTypeValidation,
                                "Too many patterns (maximum of {max})"
                            );
                            break '_cleanup;
                        }
                        let mut item_index: size_t = 0 as size_t;
                        loop {
                            if item_index >= pattern_list.size {
                                break 's_506;
                            }
                            let item: Object = unsafe { *pattern_list.items.add(item_index) };
                            let Some(item) = item.as_string() else {
                                let want = api_typename(kObjectTypeString);
                                let got = api_typename(item.kind());
                                error = err_expected(c"pattern", want, Some(got));
                                break '_cleanup;
                            };
                            pattern_filters[pattern_filter_count as usize] = item.data();
                            pattern_filter_count += 1 as ::core::ffi::c_int;
                            item_index = item_index.wrapping_add(1);
                        }
                    } else if true {
                        let want = c"String or Array";
                        let got = api_typename(v_0.kind());
                        error = err_expected(c"pattern", want, Some(got));
                        break '_cleanup;
                    }
                }
            }
            's_659: {
                if let Object::Integer(handle) | Object::Buffer(handle) = buf {
                    let mut b: *mut buf_T =
                        unsafe { find_buffer_by_handle(handle as Buffer, &mut error) };
                    if error.kind() as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        break '_cleanup;
                    }
                    let mut pat: String_0 =
                        unsafe { arena_printf(arena, c"<buffer=%d>".as_ptr(), (*b).handle) };
                    buffers = arena_array(arena, 1 as size_t);
                    unsafe { array_add(&mut buffers, Object::string(pat)) };
                } else if let Object::Array(bufnrs) = buf {
                    if !(bufnrs.size <= 256 as size_t) {
                        let max = 256 as ::core::ffi::c_int;
                        error =
                            api_error!(kErrorTypeValidation, "Too many buffers (maximum of {max})");
                        break '_cleanup;
                    }
                    buffers = arena_array(arena, bufnrs.size);
                    let mut bufnr_index: size_t = 0 as size_t;
                    loop {
                        if bufnr_index >= bufnrs.size {
                            break 's_659;
                        }
                        let bufnr: Object = unsafe { *bufnrs.items.add(bufnr_index) };
                        let (Object::Integer(handle) | Object::Buffer(handle)) = bufnr else {
                            let want = c"Integer";
                            let got = api_typename(bufnr.kind());
                            error = err_expected(c"buffer", want, Some(got));
                            break '_cleanup;
                        };
                        let mut b_0: *mut buf_T =
                            unsafe { find_buffer_by_handle(handle as Buffer, &mut error) };
                        if error.kind() as ::core::ffi::c_int
                            != kErrorTypeNone as ::core::ffi::c_int
                        {
                            break '_cleanup;
                        }
                        // SAFETY: a live pointer the code around it already holds.
                        let put_value = unsafe {
                            Object::string(arena_printf(
                                arena,
                                c"<buffer=%d>".as_ptr(),
                                (*b_0).handle,
                            ))
                        };
                        // SAFETY: the collection is this call's own.
                        unsafe { array_add(&mut buffers, put_value) };
                        bufnr_index = bufnr_index.wrapping_add(1);
                    }
                } else if has_buf && true {
                    let want = c"Integer or Array";
                    error = err_expected(c"buffer", want, Some(api_typename(buf.kind())));
                    break '_cleanup;
                }
            }
            let mut bufnr_index_0: size_t = 0 as size_t;
            while bufnr_index_0 < buffers.size {
                let bufnr_0: Object = unsafe { *buffers.items.add(bufnr_index_0) };
                pattern_filters[pattern_filter_count as usize] = bufnr_0
                    .as_string()
                    .expect("`buffers` was filled with `<buffer=N>` Strings just above")
                    .data();
                pattern_filter_count += 1 as ::core::ffi::c_int;
                bufnr_index_0 = bufnr_index_0.wrapping_add(1);
            }
            for event in AutoEvent::all() {
                if !(check_event as ::core::ffi::c_int != 0 && !event_set[event.index()]) {
                    let mut acs: *mut AutoCmdVec = au_get_autocmds_for_event(event);
                    let mut i: size_t = 0 as size_t;
                    while i < unsafe { (*acs).size } {
                        // SAFETY: `i` is below `(*acs).size`.
                        let ac: *mut AutoCmd = unsafe { (*acs).items.add(i) };
                        let ap: *mut AutoPat = unsafe { (*ac).pat };
                        's_712: {
                            if !ap.is_null()
                                && !(id != -1 as ::core::ffi::c_int
                                    && unsafe { (*ac).id } != id as int64_t)
                                && !(group != 0 as ::core::ffi::c_int
                                    && unsafe { (*ap).group } != group)
                            {
                                if pattern_filter_count > 0 as ::core::ffi::c_int {
                                    let mut passed: bool = false;
                                    let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    while j < pattern_filter_count {
                                        debug_assert!(
                                            j < 256 as ::core::ffi::c_int,
                                            "j < AUCMD_MAX_PATTERNS"
                                        );
                                        debug_assert!(
                                            !pattern_filters[j as usize].is_null(),
                                            "pattern_filters[j]"
                                        );
                                        let mut pat_0: *mut ::core::ffi::c_char =
                                            pattern_filters[j as usize];
                                        let mut patlen: ::core::ffi::c_int =
                                            unsafe { cstr::bytes_at(pat_0) }.len()
                                                as ::core::ffi::c_int;
                                        let mut pattern_buflocal: [::core::ffi::c_char; 25] =
                                            [0; 25];
                                        if unsafe { aupat_is_buflocal(pat_0, patlen) } {
                                            let dest = &raw mut pattern_buflocal
                                                as *mut ::core::ffi::c_char;
                                            // SAFETY: `pat_0` is a C string of
                                            // `patlen` bytes.
                                            let nr =
                                                unsafe { aupat_get_buflocal_nr(pat_0, patlen) };
                                            // SAFETY: `dest` is this frame's
                                            // 25-byte buffer, which is what the
                                            // normalised form fits in.
                                            unsafe {
                                                aupat_normalize_buflocal_pat(
                                                    dest, pat_0, patlen, nr,
                                                )
                                            };
                                            pat_0 = &raw mut pattern_buflocal
                                                as *mut ::core::ffi::c_char;
                                        }
                                        if unsafe { strequal((*ap).pat, pat_0) } {
                                            passed = true;
                                            break;
                                        } else {
                                            j += 1;
                                        }
                                    }
                                    if !passed {
                                        break 's_712;
                                    }
                                }
                                let mut autocmd_info: Dict = arena_dict(arena, 12 as size_t);
                                if unsafe { (*ap).group } != AUGROUP_DEFAULT as ::core::ffi::c_int {
                                    // SAFETY: a live pointer the code around it already holds.
                                    let d_group =
                                        unsafe { Object::integer((*ap).group as Integer) };
                                    // SAFETY: the collection is this call's own.
                                    unsafe { dict_put(&mut autocmd_info, c"group", d_group) };
                                    // SAFETY: `augroup_name` answers a C string
                                    // for a group this pattern belongs to.
                                    let name = unsafe { cstr_as_string(augroup_name((*ap).group)) };
                                    let d_group_name = Object::string(name);
                                    // SAFETY: the collection is this call's own.
                                    unsafe {
                                        dict_put(&mut autocmd_info, c"group_name", d_group_name)
                                    };
                                }
                                if unsafe { (*ac).id } > 0 as int64_t {
                                    // SAFETY: a live pointer the code around it already holds.
                                    let d_id = unsafe { Object::integer((*ac).id) };
                                    // SAFETY: the collection is this call's own.
                                    unsafe { dict_put(&mut autocmd_info, c"id", d_id) };
                                }
                                if !unsafe { (*ac).desc }.is_null() {
                                    // SAFETY: a live pointer the code around it already holds.
                                    let d_desc =
                                        unsafe { Object::string(cstr_as_string((*ac).desc)) };
                                    // SAFETY: the collection is this call's own.
                                    unsafe { dict_put(&mut autocmd_info, c"desc", d_desc) };
                                }
                                if !unsafe { (*ac).handler_cmd }.is_null() {
                                    // SAFETY: a live pointer the code around it already holds.
                                    let d_command = unsafe {
                                        Object::string(cstr_as_string((*ac).handler_cmd))
                                    };
                                    // SAFETY: the collection is this call's own.
                                    unsafe { dict_put(&mut autocmd_info, c"command", d_command) };
                                } else {
                                    let d_command = Object::string(String_0::NULL);
                                    // SAFETY: the collection is this call's own.
                                    unsafe { dict_put(&mut autocmd_info, c"command", d_command) };
                                    let mut cb: *mut Callback =
                                        unsafe { &raw mut (*ac).handler_fn };
                                    // SAFETY: `cb` is this command's callback.
                                    match unsafe { &*cb } {
                                        Callback::Lua(luaref) => {
                                            let luaref = *luaref;
                                            // SAFETY: the reference the row owns.
                                            if unsafe { nlua_ref_is_function(luaref) } {
                                                // SAFETY: a static C string.
                                                let key =
                                                    unsafe { cstr_as_string(c"callback".as_ptr()) };
                                                // SAFETY: as above.
                                                let value = unsafe {
                                                    Object::luaref(api_new_luaref(luaref))
                                                };
                                                // SAFETY: the dict is this call's own.
                                                unsafe {
                                                    dict_put_str(&mut autocmd_info, key, value)
                                                };
                                            }
                                        }
                                        Callback::Funcref(_) | Callback::Partial(_) => {
                                            // SAFETY: a static C string.
                                            let key =
                                                unsafe { cstr_as_string(c"callback".as_ptr()) };
                                            // SAFETY: `cb` is this command's
                                            // callback and `arena` the caller's.
                                            let name = unsafe {
                                                cstr_as_string(callback_to_string(cb, arena))
                                            };
                                            let value = Object::string(name);
                                            // SAFETY: the dict is this call's own.
                                            unsafe { dict_put_str(&mut autocmd_info, key, value) };
                                        }
                                        // A row with neither a command nor a
                                        // handler cannot exist.
                                        Callback::None => unsafe { abort() },
                                    }
                                }
                                // SAFETY: a live pointer the code around it already holds.
                                let d_pattern =
                                    unsafe { Object::string(cstr_as_string((*ap).pat)) };
                                // SAFETY: the collection is this call's own.
                                unsafe { dict_put(&mut autocmd_info, c"pattern", d_pattern) };
                                // SAFETY: `event_nr2name` answers a static C string.
                                let name = unsafe { cstr_as_string(event_nr2name(event)) };
                                let d_event = Object::string(name);
                                // SAFETY: the collection is this call's own.
                                unsafe { dict_put(&mut autocmd_info, c"event", d_event) };
                                // SAFETY: a live pointer the code around it already holds.
                                let d_once = unsafe { Object::boolean((*ac).once) };
                                // SAFETY: the collection is this call's own.
                                unsafe { dict_put(&mut autocmd_info, c"once", d_once) };
                                if unsafe { (*ap).buflocal_nr } != 0 {
                                    let d_buflocal = Object::boolean(true);
                                    // SAFETY: the collection is this call's own.
                                    unsafe { dict_put(&mut autocmd_info, c"buflocal", d_buflocal) };
                                    // SAFETY: a live pointer the code around it already holds.
                                    let d_buf =
                                        unsafe { Object::integer((*ap).buflocal_nr as Integer) };
                                    // SAFETY: the collection is this call's own.
                                    unsafe { dict_put(&mut autocmd_info, c"buf", d_buf) };
                                    // SAFETY: a live pointer the code around it already holds.
                                    let d_buffer =
                                        unsafe { Object::integer((*ap).buflocal_nr as Integer) };
                                    // SAFETY: the collection is this call's own.
                                    unsafe { dict_put(&mut autocmd_info, c"buffer", d_buffer) };
                                } else {
                                    let d_buflocal = Object::boolean(false);
                                    // SAFETY: the collection is this call's own.
                                    unsafe { dict_put(&mut autocmd_info, c"buflocal", d_buflocal) };
                                }
                                // `kv_push`, whose growth step c2rust expanded inline.
                                InitVec::new(
                                    &mut autocmd_list.size,
                                    &mut autocmd_list.capacity,
                                    &mut autocmd_list.items,
                                    &mut autocmd_list.init_array,
                                )
                                .push(Object::dict(autocmd_info));
                            }
                        }
                        i = i.wrapping_add(1);
                    }
                }
            }
        }
    }
    unsafe { arena_take_arraybuilder(arena, &raw mut autocmd_list) }.reported(error)
}
