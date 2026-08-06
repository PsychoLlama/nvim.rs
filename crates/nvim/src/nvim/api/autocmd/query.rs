//! `nvim_get_autocmds()`: the whole autocommand table as data.
//!
//! One function, and the longest in the family, because it is three nested
//! filters -- over events, over groups and over patterns -- each of which
//! may be given as a string, an array or not at all, and because every
//! matching command is rendered into a Dict of its own.

// One transpiled body of 900-odd lines: the four-space shift a wrapping
// block costs would put this file back over the 1,000-line cap.  Opt
// out until the rewrite shortens it.
#![allow(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn nvim_get_autocmds(
    mut opts: *mut KeyDict_get_autocmds,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut id: ::core::ffi::c_int = 0;
    let mut has_buf: bool = false;
    let mut buf: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut pattern_filter_count: ::core::ffi::c_int = 0;
    let mut autocmd_list: ArrayBuilder = ArrayBuilder {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
        init_array: [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 16],
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
    let mut event_set: [bool; 145] = [
        false_0 != 0,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
    ];
    let mut check_event: bool = false_0 != 0;
    let mut group: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    '_cleanup: {
        match (*opts).group.type_0 as ::core::ffi::c_uint {
            0 => {}
            4 => {
                group = augroup_find((*opts).group.data.string.data);
                if !(group >= 0 as ::core::ffi::c_int) {
                    api_err_invalid(
                        err,
                        b"group\0".as_ptr() as *const ::core::ffi::c_char,
                        (*opts).group.data.string.data,
                        0 as int64_t,
                        true_0 != 0,
                    );
                    break '_cleanup;
                }
            }
            2 => {
                group = (*opts).group.data.integer as ::core::ffi::c_int;
                name = if group == 0 as ::core::ffi::c_int {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                } else {
                    augroup_name(group)
                };
                if !augroup_exists(name) {
                    api_err_invalid(
                        err,
                        b"group\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                        (*opts).group.data.integer as int64_t,
                        false_0 != 0,
                    );
                    break '_cleanup;
                }
            }
            _ => {
                if true {
                    api_err_exp(
                        err,
                        b"group\0".as_ptr() as *const ::core::ffi::c_char,
                        b"String or Integer\0".as_ptr() as *const ::core::ffi::c_char,
                        api_typename((*opts).group.type_0),
                    );
                    break '_cleanup;
                }
            }
        }
        id = if (*opts).is_set__get_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_autocmds__id
            != 0 as ::core::ffi::c_ulonglong
        {
            (*opts).id as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
        's_299: {
            if (*opts).is_set__get_autocmds_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_autocmds__event
                != 0 as ::core::ffi::c_ulonglong
            {
                check_event = true_0 != 0;
                let mut v: Object = (*opts).event;
                if v.type_0 as ::core::ffi::c_uint
                    == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut event_nr: event_T = event_name2nr_str(v.data.string);
                    if !((event_nr as ::core::ffi::c_uint)
                        < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint)
                    {
                        api_err_invalid(
                            err,
                            b"event\0".as_ptr() as *const ::core::ffi::c_char,
                            v.data.string.data,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_cleanup;
                    } else {
                        event_set[event_nr as usize] = true_0 != 0;
                    }
                } else if v.type_0 as ::core::ffi::c_uint
                    == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut event_v_index: size_t = 0 as size_t;
                    loop {
                        if event_v_index >= v.data.array.size {
                            break 's_299;
                        }
                        let mut event_v: Object =
                            *v.data.array.items.offset(event_v_index as isize);
                        if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                            != event_v.type_0 as ::core::ffi::c_uint
                        {
                            api_err_exp(
                                err,
                                b"event item\0".as_ptr() as *const ::core::ffi::c_char,
                                api_typename(kObjectTypeString),
                                api_typename(event_v.type_0),
                            );
                            break '_cleanup;
                        } else {
                            let mut event_nr_0: event_T = event_name2nr_str(event_v.data.string);
                            if !((event_nr_0 as ::core::ffi::c_uint)
                                < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint)
                            {
                                api_err_invalid(
                                    err,
                                    b"event\0".as_ptr() as *const ::core::ffi::c_char,
                                    event_v.data.string.data,
                                    0 as int64_t,
                                    true,
                                );
                                break '_cleanup;
                            } else {
                                event_set[event_nr_0 as usize] = true;
                                event_v_index = event_v_index.wrapping_add(1);
                            }
                        }
                    }
                } else if true {
                    api_err_exp(
                        err,
                        b"event\0".as_ptr() as *const ::core::ffi::c_char,
                        b"String or Array\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                    break '_cleanup;
                }
            }
        }
        has_buf = (*opts).is_set__get_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_autocmds__buf
            != 0 as ::core::ffi::c_ulonglong
            || (*opts).is_set__get_autocmds_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_autocmds__buffer
                != 0 as ::core::ffi::c_ulonglong;
        buf = if (*opts).is_set__get_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_autocmds__buf
            != 0 as ::core::ffi::c_ulonglong
        {
            (*opts).buf
        } else {
            (*opts).buffer
        };
        if !(!((*opts).is_set__get_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 2 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong)
            || !((*opts).is_set__get_autocmds_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_ulonglong))
        {
            api_err_conflict(
                err,
                b"buf\0".as_ptr() as *const ::core::ffi::c_char,
                b"buffer\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else if !(!((*opts).is_set__get_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 6 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong)
            || !has_buf)
        {
            api_err_conflict(
                err,
                b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                b"buf\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else {
            pattern_filter_count = 0 as ::core::ffi::c_int;
            's_506: {
                if (*opts).is_set__get_autocmds_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_get_autocmds__pattern
                    != 0 as ::core::ffi::c_ulonglong
                {
                    let mut v_0: Object = (*opts).pattern;
                    if v_0.type_0 as ::core::ffi::c_uint
                        == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        pattern_filters[pattern_filter_count as usize] = v_0.data.string.data;
                        pattern_filter_count += 1 as ::core::ffi::c_int;
                    } else if v_0.type_0 as ::core::ffi::c_uint
                        == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        if !(v_0.data.array.size <= 256 as size_t) {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"Too many patterns (maximum of %d)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                256 as ::core::ffi::c_int,
                            );
                            break '_cleanup;
                        } else {
                            let mut item_index: size_t = 0 as size_t;
                            loop {
                                if item_index >= v_0.data.array.size {
                                    break 's_506;
                                }
                                let mut item: Object =
                                    *v_0.data.array.items.offset(item_index as isize);
                                if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                                    != item.type_0 as ::core::ffi::c_uint
                                {
                                    api_err_exp(
                                        err,
                                        b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                                        api_typename(kObjectTypeString),
                                        api_typename(item.type_0),
                                    );
                                    break '_cleanup;
                                } else {
                                    pattern_filters[pattern_filter_count as usize] =
                                        item.data.string.data;
                                    pattern_filter_count += 1 as ::core::ffi::c_int;
                                    item_index = item_index.wrapping_add(1);
                                }
                            }
                        }
                    } else if true {
                        api_err_exp(
                            err,
                            b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                            b"String or Array\0".as_ptr() as *const ::core::ffi::c_char,
                            api_typename(v_0.type_0),
                        );
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
                    let mut b: *mut buf_T = find_buffer_by_handle(buf.data.integer as Buffer, err);
                    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        break '_cleanup;
                    } else {
                        let mut pat: String_0 = arena_printf(
                            arena,
                            b"<buffer=%d>\0".as_ptr() as *const ::core::ffi::c_char,
                            (*b).handle,
                        );
                        buffers = arena_array(arena, 1 as size_t);
                        let c2rust_fresh0 = buffers.size;
                        buffers.size = buffers.size.wrapping_add(1);
                        *buffers.items.offset(c2rust_fresh0 as isize) = object {
                            type_0: kObjectTypeString,
                            data: C2Rust_Unnamed { string: pat },
                        };
                    }
                } else if buf.type_0 as ::core::ffi::c_uint
                    == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if !(buf.data.array.size <= 256 as size_t) {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            b"Too many buffers (maximum of %d)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            256 as ::core::ffi::c_int,
                        );
                        break '_cleanup;
                    } else {
                        buffers = arena_array(arena, buf.data.array.size);
                        let mut bufnr_index: size_t = 0 as size_t;
                        loop {
                            if bufnr_index >= buf.data.array.size {
                                break 's_659;
                            }
                            let mut bufnr: Object =
                                *buf.data.array.items.offset(bufnr_index as isize);
                            if !(bufnr.type_0 as ::core::ffi::c_uint
                                == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                                || bufnr.type_0 as ::core::ffi::c_uint
                                    == kObjectTypeBuffer as ::core::ffi::c_int
                                        as ::core::ffi::c_uint)
                            {
                                api_err_exp(
                                    err,
                                    b"buffer\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"Integer\0".as_ptr() as *const ::core::ffi::c_char,
                                    api_typename(bufnr.type_0),
                                );
                                break '_cleanup;
                            } else {
                                let mut b_0: *mut buf_T =
                                    find_buffer_by_handle(bufnr.data.integer as Buffer, err);
                                if (*err).type_0 as ::core::ffi::c_int
                                    != kErrorTypeNone as ::core::ffi::c_int
                                {
                                    break '_cleanup;
                                }
                                let c2rust_fresh1 = buffers.size;
                                buffers.size = buffers.size.wrapping_add(1);
                                *buffers.items.offset(c2rust_fresh1 as isize) = object {
                                    type_0: kObjectTypeString,
                                    data: C2Rust_Unnamed {
                                        string: arena_printf(
                                            arena,
                                            b"<buffer=%d>\0".as_ptr() as *const ::core::ffi::c_char,
                                            (*b_0).handle,
                                        ),
                                    },
                                };
                                bufnr_index = bufnr_index.wrapping_add(1);
                            }
                        }
                    }
                } else if has_buf {
                    if true {
                        api_err_exp(
                            err,
                            b"buffer\0".as_ptr() as *const ::core::ffi::c_char,
                            b"Integer or Array\0".as_ptr() as *const ::core::ffi::c_char,
                            api_typename(buf.type_0),
                        );
                        break '_cleanup;
                    }
                }
            }
            let mut bufnr_index_0: size_t = 0 as size_t;
            while bufnr_index_0 < buffers.size {
                let mut bufnr_0: Object = *buffers.items.offset(bufnr_index_0 as isize);
                pattern_filters[pattern_filter_count as usize] = bufnr_0.data.string.data;
                pattern_filter_count += 1 as ::core::ffi::c_int;
                bufnr_index_0 = bufnr_index_0.wrapping_add(1);
            }
            let mut event: event_T = EVENT_BUFADD;
            while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
                if !(check_event as ::core::ffi::c_int != 0 && !event_set[event as usize]) {
                    let mut acs: *mut AutoCmdVec = au_get_autocmds_for_event(event);
                    let mut i: size_t = 0 as size_t;
                    while i < (*acs).size {
                        let ac: *mut AutoCmd = (*acs).items.offset(i as isize);
                        let ap: *mut AutoPat = (*ac).pat;
                        's_712: {
                            if !ap.is_null() {
                                if !(id != -1 as ::core::ffi::c_int && (*ac).id != id as int64_t) {
                                    if !(group != 0 as ::core::ffi::c_int && (*ap).group != group) {
                                        if pattern_filter_count > 0 as ::core::ffi::c_int {
                                            let mut passed: bool = false_0 != 0;
                                            let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                            while j < pattern_filter_count {
                                                '_c2rust_label: {
                                                    if j < 256 as ::core::ffi::c_int {
                                                    } else {
                                                        __assert_fail(
                                                            b"j < AUCMD_MAX_PATTERNS\0".as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                            b"src/nvim/api/autocmd.rs\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            256 as ::core::ffi::c_uint,
                                                            b"Array nvim_get_autocmds(KeyDict_get_autocmds *, Arena *, Error *)\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                        );
                                                    }
                                                };
                                                '_c2rust_label_0: {
                                                    if !pattern_filters[j as usize].is_null() {
                                                    } else {
                                                        __assert_fail(
                                                            b"pattern_filters[j]\0".as_ptr()
                                                                as *const ::core::ffi::c_char,
                                                            b"src/nvim/api/autocmd.rs\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                            257 as ::core::ffi::c_uint,
                                                            b"Array nvim_get_autocmds(KeyDict_get_autocmds *, Arena *, Error *)\0"
                                                                .as_ptr() as *const ::core::ffi::c_char,
                                                        );
                                                    }
                                                };
                                                let mut pat_0: *mut ::core::ffi::c_char =
                                                    pattern_filters[j as usize];
                                                let mut patlen: ::core::ffi::c_int =
                                                    strlen(pat_0) as ::core::ffi::c_int;
                                                let mut pattern_buflocal: [::core::ffi::c_char;
                                                    25] = [0; 25];
                                                if aupat_is_buflocal(pat_0, patlen) {
                                                    aupat_normalize_buflocal_pat(
                                                        &raw mut pattern_buflocal
                                                            as *mut ::core::ffi::c_char,
                                                        pat_0,
                                                        patlen,
                                                        aupat_get_buflocal_nr(pat_0, patlen),
                                                    );
                                                    pat_0 = &raw mut pattern_buflocal
                                                        as *mut ::core::ffi::c_char;
                                                }
                                                if strequal((*ap).pat, pat_0) {
                                                    passed = true_0 != 0;
                                                    break;
                                                } else {
                                                    j += 1;
                                                }
                                            }
                                            if !passed {
                                                break 's_712;
                                            }
                                        }
                                        let mut autocmd_info: Dict =
                                            arena_dict(arena, 12 as size_t);
                                        if (*ap).group != AUGROUP_DEFAULT as ::core::ffi::c_int {
                                            let c2rust_fresh2 = autocmd_info.size;
                                            autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                            *autocmd_info.items.offset(c2rust_fresh2 as isize) =
                                                key_value_pair {
                                                    key: cstr_as_string(b"group\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    value: object {
                                                        type_0: kObjectTypeInteger,
                                                        data: C2Rust_Unnamed {
                                                            integer: (*ap).group as Integer,
                                                        },
                                                    },
                                                };
                                            let c2rust_fresh3 = autocmd_info.size;
                                            autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                            *autocmd_info.items.offset(c2rust_fresh3 as isize) =
                                                key_value_pair {
                                                    key: cstr_as_string(b"group_name\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    value: object {
                                                        type_0: kObjectTypeString,
                                                        data: C2Rust_Unnamed {
                                                            string: cstr_as_string(augroup_name(
                                                                (*ap).group,
                                                            )),
                                                        },
                                                    },
                                                };
                                        }
                                        if (*ac).id > 0 as int64_t {
                                            let c2rust_fresh4 = autocmd_info.size;
                                            autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                            *autocmd_info.items.offset(c2rust_fresh4 as isize) =
                                                key_value_pair {
                                                    key: cstr_as_string(b"id\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    value: object {
                                                        type_0: kObjectTypeInteger,
                                                        data: C2Rust_Unnamed { integer: (*ac).id },
                                                    },
                                                };
                                        }
                                        if !(*ac).desc.is_null() {
                                            let c2rust_fresh5 = autocmd_info.size;
                                            autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                            *autocmd_info.items.offset(c2rust_fresh5 as isize) =
                                                key_value_pair {
                                                    key: cstr_as_string(b"desc\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    value: object {
                                                        type_0: kObjectTypeString,
                                                        data: C2Rust_Unnamed {
                                                            string: cstr_as_string((*ac).desc),
                                                        },
                                                    },
                                                };
                                        }
                                        if !(*ac).handler_cmd.is_null() {
                                            let c2rust_fresh6 = autocmd_info.size;
                                            autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                            *autocmd_info.items.offset(c2rust_fresh6 as isize) =
                                                key_value_pair {
                                                    key: cstr_as_string(b"command\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    value: object {
                                                        type_0: kObjectTypeString,
                                                        data: C2Rust_Unnamed {
                                                            string: cstr_as_string(
                                                                (*ac).handler_cmd,
                                                            ),
                                                        },
                                                    },
                                                };
                                        } else {
                                            let c2rust_fresh7 = autocmd_info.size;
                                            autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                            *autocmd_info.items.offset(c2rust_fresh7 as isize) =
                                                key_value_pair {
                                                    key: cstr_as_string(b"command\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    value: object {
                                                        type_0: kObjectTypeString,
                                                        data: C2Rust_Unnamed {
                                                            string: String_0 {
                                                                data: ::core::ptr::null_mut::<
                                                                    ::core::ffi::c_char,
                                                                >(
                                                                ),
                                                                size: 0 as size_t,
                                                            },
                                                        },
                                                    },
                                                };
                                            let mut cb: *mut Callback = &raw mut (*ac).handler_fn;
                                            match (*cb).type_0 as ::core::ffi::c_uint {
                                                3 => {
                                                    if nlua_ref_is_function((*cb).data.luaref) {
                                                        let c2rust_fresh8 = autocmd_info.size;
                                                        autocmd_info.size =
                                                            autocmd_info.size.wrapping_add(1);
                                                        *autocmd_info.items.offset(c2rust_fresh8 as isize) = key_value_pair {
                                                            key: cstr_as_string(
                                                                b"callback\0".as_ptr() as *const ::core::ffi::c_char,
                                                            ),
                                                            value: object {
                                                                type_0: kObjectTypeLuaRef,
                                                                data: C2Rust_Unnamed {
                                                                    luaref: api_new_luaref((*cb).data.luaref),
                                                                },
                                                            },
                                                        };
                                                    }
                                                }
                                                1 | 2 => {
                                                    let c2rust_fresh9 = autocmd_info.size;
                                                    autocmd_info.size =
                                                        autocmd_info.size.wrapping_add(1);
                                                    *autocmd_info
                                                        .items
                                                        .offset(c2rust_fresh9 as isize) =
                                                        key_value_pair {
                                                            key: cstr_as_string(
                                                                b"callback\0".as_ptr()
                                                                    as *const ::core::ffi::c_char,
                                                            ),
                                                            value: object {
                                                                type_0: kObjectTypeString,
                                                                data: C2Rust_Unnamed {
                                                                    string: cstr_as_string(
                                                                        callback_to_string(
                                                                            cb, arena,
                                                                        ),
                                                                    ),
                                                                },
                                                            },
                                                        };
                                                }
                                                0 => {
                                                    abort();
                                                }
                                                _ => {}
                                            }
                                        }
                                        let c2rust_fresh10 = autocmd_info.size;
                                        autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                        *autocmd_info.items.offset(c2rust_fresh10 as isize) =
                                            key_value_pair {
                                                key: cstr_as_string(b"pattern\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                value: object {
                                                    type_0: kObjectTypeString,
                                                    data: C2Rust_Unnamed {
                                                        string: cstr_as_string((*ap).pat),
                                                    },
                                                },
                                            };
                                        let c2rust_fresh11 = autocmd_info.size;
                                        autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                        *autocmd_info.items.offset(c2rust_fresh11 as isize) =
                                            key_value_pair {
                                                key: cstr_as_string(b"event\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                value: object {
                                                    type_0: kObjectTypeString,
                                                    data: C2Rust_Unnamed {
                                                        string: cstr_as_string(event_nr2name(
                                                            event,
                                                        )),
                                                    },
                                                },
                                            };
                                        let c2rust_fresh12 = autocmd_info.size;
                                        autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                        *autocmd_info.items.offset(c2rust_fresh12 as isize) =
                                            key_value_pair {
                                                key: cstr_as_string(b"once\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                value: object {
                                                    type_0: kObjectTypeBoolean,
                                                    data: C2Rust_Unnamed {
                                                        boolean: (*ac).once,
                                                    },
                                                },
                                            };
                                        if (*ap).buflocal_nr != 0 {
                                            let c2rust_fresh13 = autocmd_info.size;
                                            autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                            *autocmd_info.items.offset(c2rust_fresh13 as isize) =
                                                key_value_pair {
                                                    key: cstr_as_string(b"buflocal\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    value: object {
                                                        type_0: kObjectTypeBoolean,
                                                        data: C2Rust_Unnamed { boolean: true },
                                                    },
                                                };
                                            let c2rust_fresh14 = autocmd_info.size;
                                            autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                            *autocmd_info.items.offset(c2rust_fresh14 as isize) =
                                                key_value_pair {
                                                    key: cstr_as_string(b"buf\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    value: object {
                                                        type_0: kObjectTypeInteger,
                                                        data: C2Rust_Unnamed {
                                                            integer: (*ap).buflocal_nr as Integer,
                                                        },
                                                    },
                                                };
                                            let c2rust_fresh15 = autocmd_info.size;
                                            autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                            *autocmd_info.items.offset(c2rust_fresh15 as isize) =
                                                key_value_pair {
                                                    key: cstr_as_string(b"buffer\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    value: object {
                                                        type_0: kObjectTypeInteger,
                                                        data: C2Rust_Unnamed {
                                                            integer: (*ap).buflocal_nr as Integer,
                                                        },
                                                    },
                                                };
                                        } else {
                                            let c2rust_fresh16 = autocmd_info.size;
                                            autocmd_info.size = autocmd_info.size.wrapping_add(1);
                                            *autocmd_info.items.offset(c2rust_fresh16 as isize) =
                                                key_value_pair {
                                                    key: cstr_as_string(b"buflocal\0".as_ptr()
                                                        as *const ::core::ffi::c_char),
                                                    value: object {
                                                        type_0: kObjectTypeBoolean,
                                                        data: C2Rust_Unnamed { boolean: false },
                                                    },
                                                };
                                        }
                                        if autocmd_list.size == autocmd_list.capacity {
                                            autocmd_list.capacity = if autocmd_list.capacity
                                                << 1 as ::core::ffi::c_int
                                                > ::core::mem::size_of::<[Object; 16]>()
                                                    .wrapping_div(::core::mem::size_of::<Object>())
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<[Object; 16]>()
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                Object,
                                                            >(
                                                            ))
                                                            == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    ) {
                                                autocmd_list.capacity << 1 as ::core::ffi::c_int
                                            } else {
                                                ::core::mem::size_of::<[Object; 16]>()
                                                    .wrapping_div(::core::mem::size_of::<Object>())
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<[Object; 16]>()
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                Object,
                                                            >(
                                                            ))
                                                            == 0)
                                                            as ::core::ffi::c_int
                                                            as size_t,
                                                    )
                                            };
                                            autocmd_list.items = (if autocmd_list.capacity
                                                == ::core::mem::size_of::<[Object; 16]>()
                                                    .wrapping_div(::core::mem::size_of::<Object>())
                                                    .wrapping_div(
                                                        (::core::mem::size_of::<[Object; 16]>()
                                                            .wrapping_rem(::core::mem::size_of::<
                                                                Object,
                                                            >(
                                                            ))
                                                            == 0)
                                                            as ::core::ffi::c_int
                                                            as usize,
                                                    ) {
                                                if autocmd_list.items
                                                    == &raw mut autocmd_list.init_array
                                                        as *mut Object
                                                {
                                                    autocmd_list.items as *mut ::core::ffi::c_void
                                                } else {
                                                    _memcpy_free(
                                                        &raw mut autocmd_list.init_array
                                                            as *mut Object
                                                            as *mut ::core::ffi::c_void,
                                                        autocmd_list.items
                                                            as *mut ::core::ffi::c_void,
                                                        autocmd_list.size.wrapping_mul(
                                                            ::core::mem::size_of::<Object>(),
                                                        ),
                                                    )
                                                }
                                            } else {
                                                if autocmd_list.items
                                                    == &raw mut autocmd_list.init_array
                                                        as *mut Object
                                                {
                                                    memcpy(
                                                        xmalloc(
                                                            autocmd_list.capacity.wrapping_mul(
                                                                ::core::mem::size_of::<Object>(),
                                                            ),
                                                        ),
                                                        autocmd_list.items
                                                            as *const ::core::ffi::c_void,
                                                        autocmd_list.size.wrapping_mul(
                                                            ::core::mem::size_of::<Object>(),
                                                        ),
                                                    )
                                                } else {
                                                    xrealloc(
                                                        autocmd_list.items
                                                            as *mut ::core::ffi::c_void,
                                                        autocmd_list.capacity.wrapping_mul(
                                                            ::core::mem::size_of::<Object>(),
                                                        ),
                                                    )
                                                }
                                            })
                                                as *mut Object;
                                        } else {
                                        };
                                        let c2rust_fresh17 = autocmd_list.size;
                                        autocmd_list.size = autocmd_list.size.wrapping_add(1);
                                        *autocmd_list.items.offset(c2rust_fresh17 as isize) =
                                            object {
                                                type_0: kObjectTypeDict,
                                                data: C2Rust_Unnamed { dict: autocmd_info },
                                            };
                                    }
                                }
                            }
                        }
                        i = i.wrapping_add(1);
                    }
                }
                event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
            }
        }
    }
    return arena_take_arraybuilder(arena, &raw mut autocmd_list);
}
