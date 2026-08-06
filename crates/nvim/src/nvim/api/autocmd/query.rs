//! `nvim_get_autocmds()`: the whole autocommand table as data.
//!
//! One function, and the longest in the family, because it is three nested
//! filters -- over events, over groups and over patterns -- each of which
//! may be given as a string, an array or not at all, and because every
//! matching command is rendered into a Dict of its own.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
#[allow(unused_imports)]
use crate::src::nvim::api::private::helpers::{array_add, dict_put, dict_put_str};
use crate::src::nvim::kvec::InitVec;
use crate::src::nvim::types::OptionalKeys;

pub unsafe extern "C" fn nvim_get_autocmds(
    mut opts: *mut KeyDict_get_autocmds,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    unsafe {
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
                (::core::mem::size_of::<[Object; 16]>()
                    .wrapping_rem(::core::mem::size_of::<Object>())
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
                            c"group".as_ptr(),
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
                            c"group".as_ptr(),
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
                            c"group".as_ptr(),
                            c"String or Integer".as_ptr(),
                            api_typename((*opts).group.type_0),
                        );
                        break '_cleanup;
                    }
                }
            }
            id = if has_key(
                (*opts).is_set__get_autocmds_,
                KEYSET_OPTIDX_get_autocmds__id,
            ) {
                (*opts).id as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            };
            's_299: {
                if has_key(
                    (*opts).is_set__get_autocmds_,
                    KEYSET_OPTIDX_get_autocmds__event,
                ) {
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
                                c"event".as_ptr(),
                                v.data.string.data,
                                0 as int64_t,
                                true_0 != 0,
                            );
                            break '_cleanup;
                        }
                        event_set[event_nr as usize] = true_0 != 0;
                    } else if v.type_0 as ::core::ffi::c_uint
                        == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        let mut event_v_index: size_t = 0 as size_t;
                        loop {
                            if event_v_index >= v.data.array.size {
                                break 's_299;
                            }
                            let mut event_v: Object = *v.data.array.items.add(event_v_index);
                            if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                                != event_v.type_0 as ::core::ffi::c_uint
                            {
                                api_err_exp(
                                    err,
                                    c"event item".as_ptr(),
                                    api_typename(kObjectTypeString),
                                    api_typename(event_v.type_0),
                                );
                                break '_cleanup;
                            }
                            let mut event_nr_0: event_T = event_name2nr_str(event_v.data.string);
                            if !((event_nr_0 as ::core::ffi::c_uint)
                                < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint)
                            {
                                api_err_invalid(
                                    err,
                                    c"event".as_ptr(),
                                    event_v.data.string.data,
                                    0 as int64_t,
                                    true,
                                );
                                break '_cleanup;
                            }
                            event_set[event_nr_0 as usize] = true;
                            event_v_index = event_v_index.wrapping_add(1);
                        }
                    } else if true {
                        api_err_exp(
                            err,
                            c"event".as_ptr(),
                            c"String or Array".as_ptr(),
                            ::core::ptr::null::<::core::ffi::c_char>(),
                        );
                        break '_cleanup;
                    }
                }
            }
            has_buf = has_key(
                (*opts).is_set__get_autocmds_,
                KEYSET_OPTIDX_get_autocmds__buf,
            ) || has_key(
                (*opts).is_set__get_autocmds_,
                KEYSET_OPTIDX_get_autocmds__buffer,
            );
            buf = if has_key(
                (*opts).is_set__get_autocmds_,
                KEYSET_OPTIDX_get_autocmds__buf,
            ) {
                (*opts).buf
            } else {
                (*opts).buffer
            };
            if !(!(has_key((*opts).is_set__get_autocmds_, 2 as ::core::ffi::c_int))
                || !(has_key((*opts).is_set__get_autocmds_, 5 as ::core::ffi::c_int)))
            {
                api_err_conflict(err, c"buf".as_ptr(), c"buffer".as_ptr());
            } else if !(!(has_key((*opts).is_set__get_autocmds_, 6 as ::core::ffi::c_int))
                || !has_buf)
            {
                api_err_conflict(err, c"pattern".as_ptr(), c"buf".as_ptr());
            } else {
                pattern_filter_count = 0 as ::core::ffi::c_int;
                's_506: {
                    if has_key(
                        (*opts).is_set__get_autocmds_,
                        KEYSET_OPTIDX_get_autocmds__pattern,
                    ) {
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
                                    c"Too many patterns (maximum of %d)".as_ptr(),
                                    256 as ::core::ffi::c_int,
                                );
                                break '_cleanup;
                            }
                            let mut item_index: size_t = 0 as size_t;
                            loop {
                                if item_index >= v_0.data.array.size {
                                    break 's_506;
                                }
                                let mut item: Object = *v_0.data.array.items.add(item_index);
                                if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                                    != item.type_0 as ::core::ffi::c_uint
                                {
                                    api_err_exp(
                                        err,
                                        c"pattern".as_ptr(),
                                        api_typename(kObjectTypeString),
                                        api_typename(item.type_0),
                                    );
                                    break '_cleanup;
                                }
                                pattern_filters[pattern_filter_count as usize] =
                                    item.data.string.data;
                                pattern_filter_count += 1 as ::core::ffi::c_int;
                                item_index = item_index.wrapping_add(1);
                            }
                        } else if true {
                            api_err_exp(
                                err,
                                c"pattern".as_ptr(),
                                c"String or Array".as_ptr(),
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
                        let mut b: *mut buf_T =
                            find_buffer_by_handle(buf.data.integer as Buffer, err);
                        if (*err).type_0 as ::core::ffi::c_int
                            != kErrorTypeNone as ::core::ffi::c_int
                        {
                            break '_cleanup;
                        }
                        let mut pat: String_0 =
                            arena_printf(arena, c"<buffer=%d>".as_ptr(), (*b).handle);
                        buffers = arena_array(arena, 1 as size_t);
                        array_add(&mut buffers, Object::string(pat));
                    } else if buf.type_0 as ::core::ffi::c_uint
                        == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        if !(buf.data.array.size <= 256 as size_t) {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                c"Too many buffers (maximum of %d)".as_ptr(),
                                256 as ::core::ffi::c_int,
                            );
                            break '_cleanup;
                        }
                        buffers = arena_array(arena, buf.data.array.size);
                        let mut bufnr_index: size_t = 0 as size_t;
                        loop {
                            if bufnr_index >= buf.data.array.size {
                                break 's_659;
                            }
                            let mut bufnr: Object = *buf.data.array.items.add(bufnr_index);
                            if !(bufnr.type_0 as ::core::ffi::c_uint
                                == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                                || bufnr.type_0 as ::core::ffi::c_uint
                                    == kObjectTypeBuffer as ::core::ffi::c_int
                                        as ::core::ffi::c_uint)
                            {
                                api_err_exp(
                                    err,
                                    c"buffer".as_ptr(),
                                    c"Integer".as_ptr(),
                                    api_typename(bufnr.type_0),
                                );
                                break '_cleanup;
                            }
                            let mut b_0: *mut buf_T =
                                find_buffer_by_handle(bufnr.data.integer as Buffer, err);
                            if (*err).type_0 as ::core::ffi::c_int
                                != kErrorTypeNone as ::core::ffi::c_int
                            {
                                break '_cleanup;
                            }
                            array_add(
                                &mut buffers,
                                Object::string(arena_printf(
                                    arena,
                                    c"<buffer=%d>".as_ptr(),
                                    (*b_0).handle,
                                )),
                            );
                            bufnr_index = bufnr_index.wrapping_add(1);
                        }
                    } else if has_buf {
                        if true {
                            api_err_exp(
                                err,
                                c"buffer".as_ptr(),
                                c"Integer or Array".as_ptr(),
                                api_typename(buf.type_0),
                            );
                            break '_cleanup;
                        }
                    }
                }
                let mut bufnr_index_0: size_t = 0 as size_t;
                while bufnr_index_0 < buffers.size {
                    let mut bufnr_0: Object = *buffers.items.add(bufnr_index_0);
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
                            let ac: *mut AutoCmd = (*acs).items.add(i);
                            let ap: *mut AutoPat = (*ac).pat;
                            's_712: {
                                if !ap.is_null() {
                                    if !(id != -1 as ::core::ffi::c_int
                                        && (*ac).id != id as int64_t)
                                    {
                                        if !(group != 0 as ::core::ffi::c_int
                                            && (*ap).group != group)
                                        {
                                            if pattern_filter_count > 0 as ::core::ffi::c_int {
                                                let mut passed: bool = false_0 != 0;
                                                let mut j: ::core::ffi::c_int =
                                                    0 as ::core::ffi::c_int;
                                                while j < pattern_filter_count {
                                                    '_c2rust_label: {
                                                        if j < 256 as ::core::ffi::c_int {
                                                        } else {
                                                            __assert_fail(
                                                            c"j < AUCMD_MAX_PATTERNS".as_ptr(),
                                                            c"src/nvim/api/autocmd.rs".as_ptr(),
                                                            256 as ::core::ffi::c_uint,
                                                            c"Array nvim_get_autocmds(KeyDict_get_autocmds *, Arena *, Error *)".as_ptr(),
                                                        );
                                                        }
                                                    };
                                                    '_c2rust_label_0: {
                                                        if !pattern_filters[j as usize].is_null() {
                                                        } else {
                                                            __assert_fail(
                                                            c"pattern_filters[j]".as_ptr(),
                                                            c"src/nvim/api/autocmd.rs".as_ptr(),
                                                            257 as ::core::ffi::c_uint,
                                                            c"Array nvim_get_autocmds(KeyDict_get_autocmds *, Arena *, Error *)".as_ptr(),
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
                                            if (*ap).group != AUGROUP_DEFAULT as ::core::ffi::c_int
                                            {
                                                dict_put(
                                                    &mut autocmd_info,
                                                    c"group",
                                                    Object::integer((*ap).group as Integer),
                                                );
                                                dict_put(
                                                    &mut autocmd_info,
                                                    c"group_name",
                                                    Object::string(cstr_as_string(augroup_name(
                                                        (*ap).group,
                                                    ))),
                                                );
                                            }
                                            if (*ac).id > 0 as int64_t {
                                                dict_put(
                                                    &mut autocmd_info,
                                                    c"id",
                                                    Object::integer((*ac).id),
                                                );
                                            }
                                            if !(*ac).desc.is_null() {
                                                dict_put(
                                                    &mut autocmd_info,
                                                    c"desc",
                                                    Object::string(cstr_as_string((*ac).desc)),
                                                );
                                            }
                                            if !(*ac).handler_cmd.is_null() {
                                                dict_put(
                                                    &mut autocmd_info,
                                                    c"command",
                                                    Object::string(cstr_as_string(
                                                        (*ac).handler_cmd,
                                                    )),
                                                );
                                            } else {
                                                dict_put(
                                                    &mut autocmd_info,
                                                    c"command",
                                                    Object::string(String_0 {
                                                        data: ::core::ptr::null_mut::<
                                                            ::core::ffi::c_char,
                                                        >(
                                                        ),
                                                        size: 0 as size_t,
                                                    }),
                                                );
                                                let mut cb: *mut Callback =
                                                    &raw mut (*ac).handler_fn;
                                                match (*cb).type_0 as ::core::ffi::c_uint {
                                                    3 => {
                                                        if nlua_ref_is_function((*cb).data.luaref) {
                                                            dict_put_str(
                                                                &mut autocmd_info,
                                                                cstr_as_string(
                                                                    c"callback".as_ptr(),
                                                                ),
                                                                Object::luaref(api_new_luaref(
                                                                    (*cb).data.luaref,
                                                                )),
                                                            );
                                                        }
                                                    }
                                                    1 | 2 => {
                                                        dict_put_str(
                                                            &mut autocmd_info,
                                                            cstr_as_string(c"callback".as_ptr()),
                                                            Object::string(cstr_as_string(
                                                                callback_to_string(cb, arena),
                                                            )),
                                                        );
                                                    }
                                                    0 => {
                                                        abort();
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            dict_put(
                                                &mut autocmd_info,
                                                c"pattern",
                                                Object::string(cstr_as_string((*ap).pat)),
                                            );
                                            dict_put(
                                                &mut autocmd_info,
                                                c"event",
                                                Object::string(cstr_as_string(event_nr2name(
                                                    event,
                                                ))),
                                            );
                                            dict_put(
                                                &mut autocmd_info,
                                                c"once",
                                                Object::boolean((*ac).once),
                                            );
                                            if (*ap).buflocal_nr != 0 {
                                                dict_put(
                                                    &mut autocmd_info,
                                                    c"buflocal",
                                                    Object::boolean(true),
                                                );
                                                dict_put(
                                                    &mut autocmd_info,
                                                    c"buf",
                                                    Object::integer((*ap).buflocal_nr as Integer),
                                                );
                                                dict_put(
                                                    &mut autocmd_info,
                                                    c"buffer",
                                                    Object::integer((*ap).buflocal_nr as Integer),
                                                );
                                            } else {
                                                dict_put(
                                                    &mut autocmd_info,
                                                    c"buflocal",
                                                    Object::boolean(false),
                                                );
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
}

/// `HAS_KEY(d, kind, key)`: is the keyset's "was this key given?" bit set?
///
/// The keysets carry one `is_set__<kind>_` mask whose bits are indexed by
/// the generated `KEYSET_OPTIDX_<kind>__<key>` constants. c2rust expanded
/// the macro at every use, which is three lines of shifting and casting per
/// question asked.
const fn has_key(set: OptionalKeys, idx: ::core::ffi::c_int) -> bool {
    set & (1 as OptionalKeys) << idx != 0
}
