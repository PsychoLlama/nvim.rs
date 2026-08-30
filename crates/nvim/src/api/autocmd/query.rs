//! `nvim_get_autocmds()`: the whole autocommand table as data.
//!
//! One function, and the longest in the family, because it is three nested
//! filters -- over events, over groups and over patterns -- each of which
//! may be given as a string, an array or not at all, and because every
//! matching command is rendered into a Dict of its own.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{
    ERROR_INIT, NIL, Reported, array_add, dict_put, dict_put_str, has_key,
};
use crate::eval::typval::{kCallbackFuncref, kCallbackLua, kCallbackNone, kCallbackPartial};
use crate::kvec::InitVec;
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

pub unsafe fn nvim_get_autocmds(
    opts: *mut KeyDict_get_autocmds,
    arena: *mut Arena,
) -> Result<Array, Error> {
    // SAFETY: the dispatcher's keyset outlives this call.
    let opts = unsafe { Live::<KeyDict_get_autocmds>::new(opts) };
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut id: ::core::ffi::c_int = 0;
    let mut has_buf: bool = false;
    let mut buf: Object = NIL;
    let mut pattern_filter_count: ::core::ffi::c_int = 0;
    let mut autocmd_list: ArrayBuilder = ArrayBuilder {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
        init_array: [NIL; 16],
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
        match opts.group.type_0 as ::core::ffi::c_uint {
            kObjectTypeNil => {}
            kObjectTypeString => {
                group = unsafe { augroup_find(opts.group.data.string.data()) };
                if !(group >= 0 as ::core::ffi::c_int) {
                    // SAFETY: `err` is this call's own error slot.
                    unsafe { err_bad_value(err, c"group", opts.group.data.string.data()) };
                    break '_cleanup;
                }
            }
            kObjectTypeInteger => {
                // SAFETY: the type tag says this arm's union field is the live one.
                group = unsafe { opts.group.data.integer } as ::core::ffi::c_int;
                name = if group == 0 as ::core::ffi::c_int {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                } else {
                    augroup_name(group)
                };
                if !unsafe { augroup_exists(name) } {
                    // SAFETY: `err` is this call's own error slot.
                    unsafe { err_bad_number(err, c"group", opts.group.data.integer) };
                    break '_cleanup;
                }
            }
            _ => {
                if true {
                    let want = c"String or Integer".as_ptr();
                    let got = api_typename(opts.group.type_0);
                    // SAFETY: `err` is this call's own error slot.
                    unsafe { err_expected(err, c"group", want, got) };
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
                let mut v: Object = opts.event;
                if v.type_0 as ::core::ffi::c_uint
                    == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut event_nr: event_T = unsafe { event_name2nr_str(v.data.string) };
                    if !((event_nr as ::core::ffi::c_uint)
                        < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint)
                    {
                        // SAFETY: `err` is this call's own error slot.
                        unsafe { err_bad_value(err, c"event", v.data.string.data()) };
                        break '_cleanup;
                    }
                    event_set[event_nr as usize] = true;
                } else if v.type_0 as ::core::ffi::c_uint
                    == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut event_v_index: size_t = 0 as size_t;
                    loop {
                        if event_v_index >= unsafe { v.data.array }.size {
                            break 's_299;
                        }
                        let mut event_v: Object = unsafe { *v.data.array.items.add(event_v_index) };
                        if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                            != event_v.type_0 as ::core::ffi::c_uint
                        {
                            let want = api_typename(kObjectTypeString);
                            let got = api_typename(event_v.type_0);
                            // SAFETY: `err` is this call's own error slot.
                            unsafe { err_expected(err, c"event item", want, got) };
                            break '_cleanup;
                        }
                        let mut event_nr_0: event_T =
                            unsafe { event_name2nr_str(event_v.data.string) };
                        if !((event_nr_0 as ::core::ffi::c_uint)
                            < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint)
                        {
                            // SAFETY: `err` is this call's own error slot.
                            unsafe { err_bad_value(err, c"event", event_v.data.string.data()) };
                            break '_cleanup;
                        }
                        event_set[event_nr_0 as usize] = true;
                        event_v_index = event_v_index.wrapping_add(1);
                    }
                } else if true {
                    let want = c"String or Array".as_ptr();
                    let got = ::core::ptr::null::<::core::ffi::c_char>();
                    // SAFETY: `err` is this call's own error slot.
                    unsafe { err_expected(err, c"event", want, got) };
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
            unsafe { api_err_conflict(err, c"buf".as_ptr(), c"buffer".as_ptr()) };
        } else if !(!(has_key(opts.is_set__get_autocmds_, 6 as ::core::ffi::c_int)) || !has_buf) {
            unsafe { api_err_conflict(err, c"pattern".as_ptr(), c"buf".as_ptr()) };
        } else {
            pattern_filter_count = 0 as ::core::ffi::c_int;
            's_506: {
                if has_key(
                    opts.is_set__get_autocmds_,
                    KEYSET_OPTIDX_get_autocmds__pattern,
                ) {
                    let mut v_0: Object = opts.pattern;
                    if v_0.type_0 as ::core::ffi::c_uint
                        == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        pattern_filters[pattern_filter_count as usize] =
                            unsafe { v_0.data.string }.data();
                        pattern_filter_count += 1 as ::core::ffi::c_int;
                    } else if v_0.type_0 as ::core::ffi::c_uint
                        == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        if !(unsafe { v_0.data.array }.size <= 256 as size_t) {
                            let fmt = c"Too many patterns (maximum of %d)".as_ptr();
                            let max = 256 as ::core::ffi::c_int;
                            // SAFETY: `err` is this call's own error slot, and
                            // the `%d` takes the one `c_int` it is given.
                            unsafe { api_set_error(err, kErrorTypeValidation, fmt, max) };
                            break '_cleanup;
                        }
                        let mut item_index: size_t = 0 as size_t;
                        loop {
                            if item_index >= unsafe { v_0.data.array }.size {
                                break 's_506;
                            }
                            let mut item: Object = unsafe { *v_0.data.array.items.add(item_index) };
                            if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                                != item.type_0 as ::core::ffi::c_uint
                            {
                                let want = api_typename(kObjectTypeString);
                                let got = api_typename(item.type_0);
                                // SAFETY: `err` is this call's own error slot.
                                unsafe { err_expected(err, c"pattern", want, got) };
                                break '_cleanup;
                            }
                            pattern_filters[pattern_filter_count as usize] =
                                unsafe { item.data.string }.data();
                            pattern_filter_count += 1 as ::core::ffi::c_int;
                            item_index = item_index.wrapping_add(1);
                        }
                    } else if true {
                        let want = c"String or Array".as_ptr();
                        let got = api_typename(v_0.type_0);
                        // SAFETY: `err` is this call's own error slot.
                        unsafe { err_expected(err, c"pattern", want, got) };
                        break '_cleanup;
                    }
                }
            }
            's_659: {
                if buf.type_0 as ::core::ffi::c_uint
                    == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                    || buf.type_0 as ::core::ffi::c_uint
                        == kObjectTypeBuffer as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut b: *mut buf_T =
                        unsafe { find_buffer_by_handle(buf.data.integer as Buffer, err) };
                    if unsafe { (*err).kind() } as ::core::ffi::c_int
                        != kErrorTypeNone as ::core::ffi::c_int
                    {
                        break '_cleanup;
                    }
                    let mut pat: String_0 =
                        unsafe { arena_printf(arena, c"<buffer=%d>".as_ptr(), (*b).handle) };
                    buffers = arena_array(arena, 1 as size_t);
                    unsafe { array_add(&mut buffers, Object::string(pat)) };
                } else if buf.type_0 as ::core::ffi::c_uint
                    == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if !(unsafe { buf.data.array }.size <= 256 as size_t) {
                        let fmt = c"Too many buffers (maximum of %d)".as_ptr();
                        let max = 256 as ::core::ffi::c_int;
                        // SAFETY: `err` is this call's own error slot, and the
                        // `%d` takes the one `c_int` it is given.
                        unsafe { api_set_error(err, kErrorTypeValidation, fmt, max) };
                        break '_cleanup;
                    }
                    buffers = arena_array(arena, unsafe { buf.data.array }.size);
                    let mut bufnr_index: size_t = 0 as size_t;
                    loop {
                        if bufnr_index >= unsafe { buf.data.array }.size {
                            break 's_659;
                        }
                        let mut bufnr: Object = unsafe { *buf.data.array.items.add(bufnr_index) };
                        if !(bufnr.type_0 as ::core::ffi::c_uint
                            == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                            || bufnr.type_0 as ::core::ffi::c_uint
                                == kObjectTypeBuffer as ::core::ffi::c_int as ::core::ffi::c_uint)
                        {
                            let want = c"Integer".as_ptr();
                            let got = api_typename(bufnr.type_0);
                            // SAFETY: `err` is this call's own error slot.
                            unsafe { err_expected(err, c"buffer", want, got) };
                            break '_cleanup;
                        }
                        let mut b_0: *mut buf_T =
                            unsafe { find_buffer_by_handle(bufnr.data.integer as Buffer, err) };
                        if unsafe { (*err).kind() } as ::core::ffi::c_int
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
                    let want = c"Integer or Array".as_ptr();
                    // SAFETY: `err` is this call's own error slot.
                    unsafe { err_expected(err, c"buffer", want, api_typename(buf.type_0)) };
                    break '_cleanup;
                }
            }
            let mut bufnr_index_0: size_t = 0 as size_t;
            while bufnr_index_0 < buffers.size {
                let mut bufnr_0: Object = unsafe { *buffers.items.add(bufnr_index_0) };
                pattern_filters[pattern_filter_count as usize] =
                    unsafe { bufnr_0.data.string }.data();
                pattern_filter_count += 1 as ::core::ffi::c_int;
                bufnr_index_0 = bufnr_index_0.wrapping_add(1);
            }
            let mut event: event_T = EVENT_BUFADD;
            while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
                if !(check_event as ::core::ffi::c_int != 0 && !event_set[event as usize]) {
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
                                            unsafe { strlen(pat_0) } as ::core::ffi::c_int;
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
                                    match unsafe { (*cb).type_0 } as ::core::ffi::c_uint {
                                        kCallbackLua => {
                                            if unsafe { nlua_ref_is_function((*cb).data.luaref) } {
                                                // SAFETY: a static C string.
                                                let key =
                                                    unsafe { cstr_as_string(c"callback".as_ptr()) };
                                                // SAFETY: the callback holds a
                                                // Lua reference in this arm.
                                                let value = unsafe {
                                                    Object::luaref(api_new_luaref(
                                                        (*cb).data.luaref,
                                                    ))
                                                };
                                                // SAFETY: the dict is this call's own.
                                                unsafe {
                                                    dict_put_str(&mut autocmd_info, key, value)
                                                };
                                            }
                                        }
                                        kCallbackFuncref | kCallbackPartial => {
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
                                        kCallbackNone => {
                                            unsafe { abort() };
                                        }
                                        _ => {}
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
                event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
            }
        }
    }
    unsafe { arena_take_arraybuilder(arena, &raw mut autocmd_list) }.reported(error)
}
