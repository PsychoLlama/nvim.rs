use crate::src::nvim::api::private::helpers::{
    api_set_error, api_set_sctx, api_typename, arena_array, arena_dict, arena_string,
    arena_take_arraybuilder, cstr_as_string, find_buffer_by_handle, string_to_cstr, try_enter,
    try_leave,
};
use crate::src::nvim::api::private::validate::{
    api_err_conflict, api_err_exp, api_err_invalid, api_err_required, check_string_array,
};
use crate::src::nvim::autocmd::{
    EVENT_BUFADD, apply_autocmds_group, au_get_autocmds_for_event, aucmd_del_for_event_and_group,
    aucmd_span_pattern, augroup_add, augroup_del, augroup_exists, augroup_find, augroup_name,
    aupat_get_buflocal_nr, aupat_is_buflocal, aupat_normalize_buflocal_pat, autocmd_delete_id,
    autocmd_register, do_autocmd_event, event_name2nr_str, event_nr2name,
};
use crate::src::nvim::buffer::do_modelines;
use crate::src::nvim::eval::typval::{
    callback_free, callback_to_string, kCallbackFuncref, kCallbackLua, kCallbackNone,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::kvec::_memcpy_free;
use crate::src::nvim::lua::executor::{api_new_luaref, nlua_ref_is_function};
use crate::src::nvim::main::{curbuf, current_sctx};
use crate::src::nvim::memory::{strequal, xfree, xmalloc, xrealloc};
use crate::src::nvim::os::libc::{__assert_fail, abort, memcpy, strlen};
use crate::src::nvim::strings::arena_printf;
use crate::src::nvim::types::{
    Arena, Array, ArrayBuilder, AutoCmd, AutoCmdVec, AutoPat, Buffer, Callback,
    Callback_data as C2Rust_Unnamed_5, Dict, Error, Integer, KeyDict_clear_autocmds,
    KeyDict_create_augroup, KeyDict_create_autocmd, KeyDict_exec_autocmds, KeyDict_get_autocmds,
    LuaRef, Object, String_0, TryState, auto_event, buf_T, event_T, exarg_T, except_T, int64_t,
    kErrorTypeException, kErrorTypeNone, kErrorTypeValidation, kObjectTypeArray,
    kObjectTypeBoolean, kObjectTypeBuffer, kObjectTypeDict, kObjectTypeInteger, kObjectTypeLuaRef,
    kObjectTypeNil, kObjectTypeString, key_value_pair, msglist_T, object,
    object_data as C2Rust_Unnamed, sctx_T, size_t, uint64_t,
};
pub const NUM_EVENTS: auto_event = 145;
pub const AUGROUP_DEFAULT: C2Rust_Unnamed_14 = -1;
pub const AUGROUP_ERROR: C2Rust_Unnamed_14 = -2;
pub const AUGROUP_ALL: C2Rust_Unnamed_14 = -3;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_clear_autocmds__buf: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_clear_autocmds__buffer: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_create_autocmd__buf: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_create_autocmd__desc: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_create_autocmd__buffer: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_create_autocmd__command: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_create_autocmd__callback: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_exec_autocmds__buf: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_exec_autocmds__data: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_exec_autocmds__buffer: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_exec_autocmds__modeline: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_autocmds__id: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_autocmds__buf: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_autocmds__event: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_autocmds__buffer: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_autocmds__pattern: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_create_augroup__clear: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static next_autocmd_id: GlobalCell<int64_t> = GlobalCell::new(1 as int64_t);
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
pub unsafe extern "C" fn nvim_create_autocmd(
    mut channel_id: uint64_t,
    mut event: Object,
    mut opts: *mut KeyDict_create_autocmd,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Integer {
    let mut au_group: ::core::ffi::c_int = 0;
    let mut has_buf: bool = false;
    let mut buf: Buffer = 0;
    let mut patterns: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut autocmd_id: int64_t = -1 as int64_t;
    let mut desc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut handler_cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut handler_fn: Callback = Callback {
        data: C2Rust_Unnamed_5 {
            funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        type_0: kCallbackNone,
    };
    let mut event_array: Array = unpack_string_or_array(
        event,
        b"event\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        true_0 != 0,
        arena,
        err,
    );
    '_cleanup: {
        if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            if !(!((*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 9 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_ulonglong)
                || !((*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << 7 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_ulonglong))
            {
                api_err_conflict(
                    err,
                    b"callback\0".as_ptr() as *const ::core::ffi::c_char,
                    b"command\0".as_ptr() as *const ::core::ffi::c_char,
                );
            } else {
                if (*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_create_autocmd__callback
                    != 0 as ::core::ffi::c_ulonglong
                {
                    let mut callback: *mut Object = &raw mut (*opts).callback;
                    match (*callback).type_0 as ::core::ffi::c_uint {
                        7 => {
                            if !((*callback).data.luaref != -2 as ::core::ffi::c_int) {
                                api_err_invalid(
                                    err,
                                    b"callback\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"<no value>\0".as_ptr() as *const ::core::ffi::c_char,
                                    0 as int64_t,
                                    true_0 != 0,
                                );
                                break '_cleanup;
                            } else if !nlua_ref_is_function((*callback).data.luaref) {
                                api_err_invalid(
                                    err,
                                    b"callback\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"<not a function>\0".as_ptr() as *const ::core::ffi::c_char,
                                    0 as int64_t,
                                    true_0 != 0,
                                );
                                break '_cleanup;
                            } else {
                                handler_fn.type_0 = kCallbackLua;
                                handler_fn.data.luaref = (*callback).data.luaref;
                                (*callback).data.luaref = LUA_NOREF as LuaRef;
                            }
                        }
                        4 => {
                            handler_fn.type_0 = kCallbackFuncref;
                            handler_fn.data.funcref = string_to_cstr((*callback).data.string);
                        }
                        _ => {
                            if true {
                                api_err_exp(
                                    err,
                                    b"callback\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"Lua function or Vim function name\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    api_typename((*callback).type_0),
                                );
                                break '_cleanup;
                            }
                        }
                    }
                } else if (*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_create_autocmd__command
                    != 0 as ::core::ffi::c_ulonglong
                {
                    handler_cmd = string_to_cstr((*opts).command);
                } else if true {
                    api_err_required(
                        err,
                        b"'command' or 'callback'\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break '_cleanup;
                }
                au_group = get_augroup_from_object((*opts).group, err);
                if au_group != AUGROUP_ERROR as ::core::ffi::c_int {
                    has_buf = (*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_create_autocmd__buf
                        != 0 as ::core::ffi::c_ulonglong
                        || (*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong)
                                << KEYSET_OPTIDX_create_autocmd__buffer
                            != 0 as ::core::ffi::c_ulonglong;
                    buf = if (*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_create_autocmd__buf
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        (*opts).buf
                    } else {
                        (*opts).buffer
                    };
                    if !(!((*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_ulonglong)
                        || !((*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                            != 0 as ::core::ffi::c_ulonglong))
                    {
                        api_err_conflict(
                            err,
                            b"buf\0".as_ptr() as *const ::core::ffi::c_char,
                            b"buffer\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                    } else if !(!((*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << 8 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_ulonglong)
                        || !has_buf)
                    {
                        api_err_conflict(
                            err,
                            b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                            b"buf\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                    } else {
                        patterns = get_patterns_from_pattern_or_buf(
                            (*opts).pattern,
                            has_buf,
                            buf,
                            b"*\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            arena,
                            err,
                        );
                        if (*err).type_0 as ::core::ffi::c_int
                            == kErrorTypeNone as ::core::ffi::c_int
                        {
                            if (*opts).is_set__create_autocmd_ as ::core::ffi::c_ulonglong
                                & (1 as ::core::ffi::c_ulonglong)
                                    << KEYSET_OPTIDX_create_autocmd__desc
                                != 0 as ::core::ffi::c_ulonglong
                            {
                                desc = (*opts).desc.data;
                            }
                            if !(event_array.size > 0 as size_t) {
                                api_err_required(
                                    err,
                                    b"event\0".as_ptr() as *const ::core::ffi::c_char,
                                );
                            } else {
                                let c2rust_fresh18 = next_autocmd_id.get();
                                next_autocmd_id.set(next_autocmd_id.get() + 1);
                                autocmd_id = c2rust_fresh18;
                                let mut event_str_index: size_t = 0 as size_t;
                                loop {
                                    if event_str_index >= event_array.size {
                                        break '_cleanup;
                                    }
                                    let mut event_str: Object =
                                        *event_array.items.offset(event_str_index as isize);
                                    let mut event_nr: event_T =
                                        event_name2nr_str(event_str.data.string);
                                    if !((event_nr as ::core::ffi::c_uint)
                                        < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint)
                                    {
                                        api_err_invalid(
                                            err,
                                            b"event\0".as_ptr() as *const ::core::ffi::c_char,
                                            event_str.data.string.data,
                                            0 as int64_t,
                                            true,
                                        );
                                        break '_cleanup;
                                    } else {
                                        let mut retval: ::core::ffi::c_int = 0;
                                        let mut pat_index: size_t = 0 as size_t;
                                        while pat_index < patterns.size {
                                            let mut pat: Object =
                                                *patterns.items.offset(pat_index as isize);
                                            let save_current_sctx: sctx_T =
                                                api_set_sctx(channel_id);
                                            retval = autocmd_register(
                                                autocmd_id,
                                                event_nr,
                                                pat.data.string.data,
                                                pat.data.string.size as ::core::ffi::c_int,
                                                au_group,
                                                (*opts).once as bool,
                                                (*opts).nested as bool,
                                                desc,
                                                handler_cmd,
                                                &raw mut handler_fn,
                                            );
                                            current_sctx.set(save_current_sctx);
                                            if retval == 0 as ::core::ffi::c_int {
                                                api_set_error(
                                                    err,
                                                    kErrorTypeException,
                                                    b"Failed to set autocmd\0".as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                );
                                                break '_cleanup;
                                            } else {
                                                pat_index = pat_index.wrapping_add(1);
                                            }
                                        }
                                        event_str_index = event_str_index.wrapping_add(1);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if !handler_cmd.is_null() {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut handler_cmd as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
    } else {
        callback_free(&raw mut handler_fn);
    }
    return autocmd_id as Integer;
}
pub unsafe extern "C" fn nvim_del_autocmd(mut id: Integer, mut err: *mut Error) {
    if !(id > 0 as Integer) {
        api_err_invalid(
            err,
            b"autocmd id\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
            id as int64_t,
            false_0 != 0,
        );
        return;
    }
    if !autocmd_delete_id(id as int64_t) {
        api_set_error(
            err,
            kErrorTypeException,
            b"Failed to delete autocmd\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
}
pub unsafe extern "C" fn nvim_clear_autocmds(
    mut opts: *mut KeyDict_clear_autocmds,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    let mut event_array: Array = unpack_string_or_array(
        (*opts).event,
        b"event\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        false_0 != 0,
        arena,
        err,
    );
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return;
    }
    let mut has_buf: bool = (*opts).is_set__clear_autocmds_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_clear_autocmds__buf
        != 0 as ::core::ffi::c_ulonglong
        || (*opts).is_set__clear_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_clear_autocmds__buffer
            != 0 as ::core::ffi::c_ulonglong;
    let mut buf: ::core::ffi::c_int = if (*opts).is_set__clear_autocmds_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_clear_autocmds__buf
        != 0 as ::core::ffi::c_ulonglong
    {
        (*opts).buf as ::core::ffi::c_int
    } else {
        (*opts).buffer as ::core::ffi::c_int
    };
    if !(!((*opts).is_set__clear_autocmds_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_ulonglong)
        || !((*opts).is_set__clear_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 4 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong))
    {
        api_err_conflict(
            err,
            b"buf\0".as_ptr() as *const ::core::ffi::c_char,
            b"buffer\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if !(!((*opts).is_set__clear_autocmds_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_ulonglong)
        || !has_buf)
    {
        api_err_conflict(
            err,
            b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
            b"buf\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut au_group: ::core::ffi::c_int = get_augroup_from_object((*opts).group, err);
    if au_group == AUGROUP_ERROR as ::core::ffi::c_int {
        return;
    }
    let mut patterns: Array = get_patterns_from_pattern_or_buf(
        (*opts).pattern,
        has_buf,
        buf as Buffer,
        b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        arena,
        err,
    );
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return;
    }
    if event_array.size == 0 as size_t {
        let mut event: event_T = EVENT_BUFADD;
        while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
            let mut pat_object_index: size_t = 0 as size_t;
            while pat_object_index < patterns.size {
                let mut pat_object: Object = *patterns.items.offset(pat_object_index as isize);
                let mut pat: *mut ::core::ffi::c_char = pat_object.data.string.data;
                if !clear_autocmd(event, pat, au_group, err) {
                    return;
                }
                pat_object_index = pat_object_index.wrapping_add(1);
            }
            event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
        }
    } else {
        let mut event_str_index: size_t = 0 as size_t;
        while event_str_index < event_array.size {
            let mut event_str: Object = *event_array.items.offset(event_str_index as isize);
            let mut event_nr: event_T = event_name2nr_str(event_str.data.string);
            if !((event_nr as ::core::ffi::c_uint)
                < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                api_err_invalid(
                    err,
                    b"event\0".as_ptr() as *const ::core::ffi::c_char,
                    event_str.data.string.data,
                    0 as int64_t,
                    true,
                );
                return;
            }
            let mut pat_object_index_0: size_t = 0 as size_t;
            while pat_object_index_0 < patterns.size {
                let mut pat_object_0: Object = *patterns.items.offset(pat_object_index_0 as isize);
                let mut pat_0: *mut ::core::ffi::c_char = pat_object_0.data.string.data;
                if !clear_autocmd(event_nr, pat_0, au_group, err) {
                    return;
                }
                pat_object_index_0 = pat_object_index_0.wrapping_add(1);
            }
            event_str_index = event_str_index.wrapping_add(1);
        }
    };
}
pub unsafe extern "C" fn nvim_create_augroup(
    mut channel_id: uint64_t,
    mut name: String_0,
    mut opts: *mut KeyDict_create_augroup,
    mut err: *mut Error,
) -> Integer {
    let mut augroup_name_0: *mut ::core::ffi::c_char = name.data;
    let mut clear_autocmds: bool = if (*opts).is_set__create_augroup_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_create_augroup__clear
        != 0 as ::core::ffi::c_ulonglong
    {
        (*opts).clear as ::core::ffi::c_int
    } else {
        true_0
    } != 0;
    let mut augroup: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let save_current_sctx: sctx_T = api_set_sctx(channel_id);
    augroup = augroup_add(augroup_name_0);
    if augroup == AUGROUP_ERROR as ::core::ffi::c_int {
        api_set_error(
            err,
            kErrorTypeException,
            b"Failed to set augroup\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return -1 as Integer;
    }
    if clear_autocmds {
        let mut event: event_T = EVENT_BUFADD;
        while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
            aucmd_del_for_event_and_group(event, augroup);
            event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
        }
    }
    current_sctx.set(save_current_sctx);
    return augroup as Integer;
}
pub unsafe extern "C" fn nvim_del_augroup_by_id(mut id: Integer, mut err: *mut Error) {
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    let mut name: *mut ::core::ffi::c_char = if id == 0 as Integer {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    } else {
        augroup_name(id as ::core::ffi::c_int)
    };
    augroup_del(name, false);
    try_leave(&raw mut tstate, err);
}
pub unsafe extern "C" fn nvim_del_augroup_by_name(mut name: String_0, mut err: *mut Error) {
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    augroup_del(name.data, false);
    try_leave(&raw mut tstate, err);
}
pub unsafe extern "C" fn nvim_exec_autocmds(
    mut event: Object,
    mut opts: *mut KeyDict_exec_autocmds,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    let mut au_group: ::core::ffi::c_int = AUGROUP_ALL as ::core::ffi::c_int;
    let mut modeline: bool = true_0 != 0;
    let mut b: *mut buf_T = curbuf.get();
    let mut data: *mut Object = ::core::ptr::null_mut::<Object>();
    let mut event_array: Array = unpack_string_or_array(
        event,
        b"event\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        true_0 != 0,
        arena,
        err,
    );
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return;
    }
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    match (*opts).group.type_0 as ::core::ffi::c_uint {
        0 => {}
        4 => {
            au_group = augroup_find((*opts).group.data.string.data);
            if !(au_group != AUGROUP_ERROR as ::core::ffi::c_int) {
                api_err_invalid(
                    err,
                    b"group\0".as_ptr() as *const ::core::ffi::c_char,
                    (*opts).group.data.string.data,
                    0 as int64_t,
                    true_0 != 0,
                );
                return;
            }
        }
        2 => {
            au_group = (*opts).group.data.integer as ::core::ffi::c_int;
            name = if au_group == 0 as ::core::ffi::c_int {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            } else {
                augroup_name(au_group)
            };
            if !augroup_exists(name) {
                api_err_invalid(
                    err,
                    b"group\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    au_group as int64_t,
                    false_0 != 0,
                );
                return;
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
                return;
            }
        }
    }
    let mut has_buf: bool = (*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_exec_autocmds__buf
        != 0 as ::core::ffi::c_ulonglong
        || (*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_exec_autocmds__buffer
            != 0 as ::core::ffi::c_ulonglong;
    let mut buf: Buffer = if (*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_exec_autocmds__buf
        != 0 as ::core::ffi::c_ulonglong
    {
        (*opts).buf
    } else {
        (*opts).buffer
    };
    if !(!((*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_ulonglong)
        || !((*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 4 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong))
    {
        api_err_conflict(
            err,
            b"buf\0".as_ptr() as *const ::core::ffi::c_char,
            b"buffer\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if has_buf {
        if (*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong
        {
            api_err_conflict(
                err,
                b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                b"buf\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
        b = find_buffer_by_handle(buf, err);
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return;
        }
    }
    let mut patterns: Array = get_patterns_from_pattern_or_buf(
        (*opts).pattern,
        has_buf,
        buf,
        b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        arena,
        err,
    );
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        return;
    }
    if (*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_exec_autocmds__data
        != 0 as ::core::ffi::c_ulonglong
    {
        data = &raw mut (*opts).data;
    }
    modeline = if (*opts).is_set__exec_autocmds_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_exec_autocmds__modeline
        != 0 as ::core::ffi::c_ulonglong
    {
        (*opts).modeline as ::core::ffi::c_int
    } else {
        true_0
    } != 0;
    let mut did_aucmd: bool = false_0 != 0;
    let mut event_str_index: size_t = 0 as size_t;
    while event_str_index < event_array.size {
        let mut event_str: Object = *event_array.items.offset(event_str_index as isize);
        let mut event_nr: event_T = event_name2nr_str(event_str.data.string);
        if !((event_nr as ::core::ffi::c_uint)
            < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint)
        {
            api_err_invalid(
                err,
                b"event\0".as_ptr() as *const ::core::ffi::c_char,
                event_str.data.string.data,
                0 as int64_t,
                true,
            );
            return;
        }
        let mut pat_index: size_t = 0 as size_t;
        while pat_index < patterns.size {
            let mut pat: Object = *patterns.items.offset(pat_index as isize);
            let mut fname: *mut ::core::ffi::c_char = if !has_buf {
                pat.data.string.data
            } else {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            };
            did_aucmd = did_aucmd as ::core::ffi::c_int
                | apply_autocmds_group(
                    event_nr,
                    fname,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    true,
                    au_group,
                    b,
                    ::core::ptr::null_mut::<exarg_T>(),
                    data,
                ) as ::core::ffi::c_int
                != 0;
            pat_index = pat_index.wrapping_add(1);
        }
        event_str_index = event_str_index.wrapping_add(1);
    }
    if did_aucmd as ::core::ffi::c_int != 0 && modeline as ::core::ffi::c_int != 0 {
        do_modelines(0 as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn unpack_string_or_array(
    mut v: Object,
    mut k: *mut ::core::ffi::c_char,
    mut required: bool,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    if v.type_0 as ::core::ffi::c_uint
        == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut arr: Array = arena_array(arena, 1 as size_t);
        let c2rust_fresh23 = arr.size;
        arr.size = arr.size.wrapping_add(1);
        *arr.items.offset(c2rust_fresh23 as isize) = v;
        return arr;
    } else if v.type_0 as ::core::ffi::c_uint
        == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !check_string_array(v.data.array, k, true_0 != 0, err) {
            return Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            };
        }
        return v.data.array;
    } else if !(!required
        && v.type_0 as ::core::ffi::c_uint
            == kObjectTypeNil as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        api_err_exp(
            err,
            k,
            b"Array or String\0".as_ptr() as *const ::core::ffi::c_char,
            api_typename(v.type_0),
        );
        return Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
    }
    return Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
}
unsafe extern "C" fn get_augroup_from_object(
    mut group: Object,
    mut err: *mut Error,
) -> ::core::ffi::c_int {
    let mut au_group: ::core::ffi::c_int = AUGROUP_ERROR as ::core::ffi::c_int;
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    match group.type_0 as ::core::ffi::c_uint {
        0 => return AUGROUP_DEFAULT as ::core::ffi::c_int,
        4 => {
            au_group = augroup_find(group.data.string.data);
            if !(au_group != AUGROUP_ERROR as ::core::ffi::c_int) {
                api_err_invalid(
                    err,
                    b"group\0".as_ptr() as *const ::core::ffi::c_char,
                    group.data.string.data,
                    0 as int64_t,
                    true_0 != 0,
                );
                return AUGROUP_ERROR as ::core::ffi::c_int;
            }
            return au_group;
        }
        2 => {
            au_group = group.data.integer as ::core::ffi::c_int;
            name = if au_group == 0 as ::core::ffi::c_int {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            } else {
                augroup_name(au_group)
            };
            if !augroup_exists(name) {
                api_err_invalid(
                    err,
                    b"group\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    au_group as int64_t,
                    false_0 != 0,
                );
                return AUGROUP_ERROR as ::core::ffi::c_int;
            }
            return au_group;
        }
        _ => {
            if true {
                api_err_exp(
                    err,
                    b"group\0".as_ptr() as *const ::core::ffi::c_char,
                    b"String or Integer\0".as_ptr() as *const ::core::ffi::c_char,
                    api_typename(group.type_0),
                );
                return AUGROUP_ERROR as ::core::ffi::c_int;
            }
        }
    }
    panic!("Reached end of non-void function without returning");
}
unsafe extern "C" fn get_patterns_from_pattern_or_buf(
    mut pattern: Object,
    mut has_buf: bool,
    mut buf: Buffer,
    mut fallback: *mut ::core::ffi::c_char,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut patterns: ArrayBuilder = ArrayBuilder {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
        init_array: [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 16],
    };
    patterns.capacity = ::core::mem::size_of::<[Object; 16]>()
        .wrapping_div(::core::mem::size_of::<Object>())
        .wrapping_div(
            (::core::mem::size_of::<[Object; 16]>().wrapping_rem(::core::mem::size_of::<Object>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    patterns.size = 0 as size_t;
    patterns.items = &raw mut patterns.init_array as *mut Object;
    if pattern.type_0 as ::core::ffi::c_uint
        != kObjectTypeNil as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if pattern.type_0 as ::core::ffi::c_uint
            == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut pat: *const ::core::ffi::c_char = pattern.data.string.data;
            let mut patlen: size_t = aucmd_span_pattern(pat, &raw mut pat);
            while patlen != 0 {
                if patterns.size == patterns.capacity {
                    patterns.capacity = if patterns.capacity << 1 as ::core::ffi::c_int
                        > ::core::mem::size_of::<[Object; 16]>()
                            .wrapping_div(::core::mem::size_of::<Object>())
                            .wrapping_div(
                                (::core::mem::size_of::<[Object; 16]>()
                                    .wrapping_rem(::core::mem::size_of::<Object>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        patterns.capacity << 1 as ::core::ffi::c_int
                    } else {
                        ::core::mem::size_of::<[Object; 16]>()
                            .wrapping_div(::core::mem::size_of::<Object>())
                            .wrapping_div(
                                (::core::mem::size_of::<[Object; 16]>()
                                    .wrapping_rem(::core::mem::size_of::<Object>())
                                    == 0) as ::core::ffi::c_int
                                    as size_t,
                            )
                    };
                    patterns.items = (if patterns.capacity
                        == ::core::mem::size_of::<[Object; 16]>()
                            .wrapping_div(::core::mem::size_of::<Object>())
                            .wrapping_div(
                                (::core::mem::size_of::<[Object; 16]>()
                                    .wrapping_rem(::core::mem::size_of::<Object>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        if patterns.items == &raw mut patterns.init_array as *mut Object {
                            patterns.items as *mut ::core::ffi::c_void
                        } else {
                            _memcpy_free(
                                &raw mut patterns.init_array as *mut Object
                                    as *mut ::core::ffi::c_void,
                                patterns.items as *mut ::core::ffi::c_void,
                                patterns.size.wrapping_mul(::core::mem::size_of::<Object>()),
                            )
                        }
                    } else {
                        if patterns.items == &raw mut patterns.init_array as *mut Object {
                            memcpy(
                                xmalloc(
                                    patterns
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                ),
                                patterns.items as *const ::core::ffi::c_void,
                                patterns.size.wrapping_mul(::core::mem::size_of::<Object>()),
                            )
                        } else {
                            xrealloc(
                                patterns.items as *mut ::core::ffi::c_void,
                                patterns
                                    .capacity
                                    .wrapping_mul(::core::mem::size_of::<Object>()),
                            )
                        }
                    }) as *mut Object;
                } else {
                };
                let c2rust_fresh19 = patterns.size;
                patterns.size = patterns.size.wrapping_add(1);
                *patterns.items.offset(c2rust_fresh19 as isize) = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: arena_string(
                            arena,
                            String_0 {
                                data: pat as *mut ::core::ffi::c_char,
                                size: patlen,
                            },
                        ),
                    },
                };
                patlen = aucmd_span_pattern(pat.offset(patlen as isize), &raw mut pat);
            }
        } else if pattern.type_0 as ::core::ffi::c_uint
            == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if !check_string_array(
                pattern.data.array,
                b"pattern\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                true_0 != 0,
                err,
            ) {
                return Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                };
            }
            let mut array: Array = pattern.data.array;
            let mut entry_index: size_t = 0 as size_t;
            while entry_index < array.size {
                let mut entry: Object = *array.items.offset(entry_index as isize);
                let mut pat_0: *const ::core::ffi::c_char = entry.data.string.data;
                let mut patlen_0: size_t = aucmd_span_pattern(pat_0, &raw mut pat_0);
                while patlen_0 != 0 {
                    if patterns.size == patterns.capacity {
                        patterns.capacity = if patterns.capacity << 1 as ::core::ffi::c_int
                            > ::core::mem::size_of::<[Object; 16]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 16]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            patterns.capacity << 1 as ::core::ffi::c_int
                        } else {
                            ::core::mem::size_of::<[Object; 16]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 16]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                )
                        };
                        patterns.items = (if patterns.capacity
                            == ::core::mem::size_of::<[Object; 16]>()
                                .wrapping_div(::core::mem::size_of::<Object>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[Object; 16]>()
                                        .wrapping_rem(::core::mem::size_of::<Object>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            if patterns.items == &raw mut patterns.init_array as *mut Object {
                                patterns.items as *mut ::core::ffi::c_void
                            } else {
                                _memcpy_free(
                                    &raw mut patterns.init_array as *mut Object
                                        as *mut ::core::ffi::c_void,
                                    patterns.items as *mut ::core::ffi::c_void,
                                    patterns.size.wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        } else {
                            if patterns.items == &raw mut patterns.init_array as *mut Object {
                                memcpy(
                                    xmalloc(
                                        patterns
                                            .capacity
                                            .wrapping_mul(::core::mem::size_of::<Object>()),
                                    ),
                                    patterns.items as *const ::core::ffi::c_void,
                                    patterns.size.wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            } else {
                                xrealloc(
                                    patterns.items as *mut ::core::ffi::c_void,
                                    patterns
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<Object>()),
                                )
                            }
                        }) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh20 = patterns.size;
                    patterns.size = patterns.size.wrapping_add(1);
                    *patterns.items.offset(c2rust_fresh20 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed {
                            string: arena_string(
                                arena,
                                String_0 {
                                    data: pat_0 as *mut ::core::ffi::c_char,
                                    size: patlen_0,
                                },
                            ),
                        },
                    };
                    patlen_0 = aucmd_span_pattern(pat_0.offset(patlen_0 as isize), &raw mut pat_0);
                }
                entry_index = entry_index.wrapping_add(1);
            }
        } else if true {
            api_err_exp(
                err,
                b"pattern\0".as_ptr() as *const ::core::ffi::c_char,
                b"String or Table\0".as_ptr() as *const ::core::ffi::c_char,
                api_typename(pattern.type_0),
            );
            return Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            };
        }
    } else if has_buf {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            };
        }
        if patterns.size == patterns.capacity {
            patterns.capacity = if patterns.capacity << 1 as ::core::ffi::c_int
                > ::core::mem::size_of::<[Object; 16]>()
                    .wrapping_div(::core::mem::size_of::<Object>())
                    .wrapping_div(
                        (::core::mem::size_of::<[Object; 16]>()
                            .wrapping_rem(::core::mem::size_of::<Object>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                patterns.capacity << 1 as ::core::ffi::c_int
            } else {
                ::core::mem::size_of::<[Object; 16]>()
                    .wrapping_div(::core::mem::size_of::<Object>())
                    .wrapping_div(
                        (::core::mem::size_of::<[Object; 16]>()
                            .wrapping_rem(::core::mem::size_of::<Object>())
                            == 0) as ::core::ffi::c_int as size_t,
                    )
            };
            patterns.items = (if patterns.capacity
                == ::core::mem::size_of::<[Object; 16]>()
                    .wrapping_div(::core::mem::size_of::<Object>())
                    .wrapping_div(
                        (::core::mem::size_of::<[Object; 16]>()
                            .wrapping_rem(::core::mem::size_of::<Object>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                if patterns.items == &raw mut patterns.init_array as *mut Object {
                    patterns.items as *mut ::core::ffi::c_void
                } else {
                    _memcpy_free(
                        &raw mut patterns.init_array as *mut Object as *mut ::core::ffi::c_void,
                        patterns.items as *mut ::core::ffi::c_void,
                        patterns.size.wrapping_mul(::core::mem::size_of::<Object>()),
                    )
                }
            } else {
                if patterns.items == &raw mut patterns.init_array as *mut Object {
                    memcpy(
                        xmalloc(
                            patterns
                                .capacity
                                .wrapping_mul(::core::mem::size_of::<Object>()),
                        ),
                        patterns.items as *const ::core::ffi::c_void,
                        patterns.size.wrapping_mul(::core::mem::size_of::<Object>()),
                    )
                } else {
                    xrealloc(
                        patterns.items as *mut ::core::ffi::c_void,
                        patterns
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<Object>()),
                    )
                }
            }) as *mut Object;
        } else {
        };
        let c2rust_fresh21 = patterns.size;
        patterns.size = patterns.size.wrapping_add(1);
        *patterns.items.offset(c2rust_fresh21 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: arena_printf(
                    arena,
                    b"<buffer=%d>\0".as_ptr() as *const ::core::ffi::c_char,
                    (*b).handle,
                ),
            },
        };
    }
    if patterns.size == 0 as size_t && !fallback.is_null() {
        if patterns.size == patterns.capacity {
            patterns.capacity = if patterns.capacity << 1 as ::core::ffi::c_int
                > ::core::mem::size_of::<[Object; 16]>()
                    .wrapping_div(::core::mem::size_of::<Object>())
                    .wrapping_div(
                        (::core::mem::size_of::<[Object; 16]>()
                            .wrapping_rem(::core::mem::size_of::<Object>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                patterns.capacity << 1 as ::core::ffi::c_int
            } else {
                ::core::mem::size_of::<[Object; 16]>()
                    .wrapping_div(::core::mem::size_of::<Object>())
                    .wrapping_div(
                        (::core::mem::size_of::<[Object; 16]>()
                            .wrapping_rem(::core::mem::size_of::<Object>())
                            == 0) as ::core::ffi::c_int as size_t,
                    )
            };
            patterns.items = (if patterns.capacity
                == ::core::mem::size_of::<[Object; 16]>()
                    .wrapping_div(::core::mem::size_of::<Object>())
                    .wrapping_div(
                        (::core::mem::size_of::<[Object; 16]>()
                            .wrapping_rem(::core::mem::size_of::<Object>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                if patterns.items == &raw mut patterns.init_array as *mut Object {
                    patterns.items as *mut ::core::ffi::c_void
                } else {
                    _memcpy_free(
                        &raw mut patterns.init_array as *mut Object as *mut ::core::ffi::c_void,
                        patterns.items as *mut ::core::ffi::c_void,
                        patterns.size.wrapping_mul(::core::mem::size_of::<Object>()),
                    )
                }
            } else {
                if patterns.items == &raw mut patterns.init_array as *mut Object {
                    memcpy(
                        xmalloc(
                            patterns
                                .capacity
                                .wrapping_mul(::core::mem::size_of::<Object>()),
                        ),
                        patterns.items as *const ::core::ffi::c_void,
                        patterns.size.wrapping_mul(::core::mem::size_of::<Object>()),
                    )
                } else {
                    xrealloc(
                        patterns.items as *mut ::core::ffi::c_void,
                        patterns
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<Object>()),
                    )
                }
            }) as *mut Object;
        } else {
        };
        let c2rust_fresh22 = patterns.size;
        patterns.size = patterns.size.wrapping_add(1);
        *patterns.items.offset(c2rust_fresh22 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(fallback),
            },
        };
    }
    return arena_take_arraybuilder(arena, &raw mut patterns);
}
unsafe extern "C" fn clear_autocmd(
    mut event: event_T,
    mut pat: *mut ::core::ffi::c_char,
    mut au_group: ::core::ffi::c_int,
    mut err: *mut Error,
) -> bool {
    if do_autocmd_event(
        event,
        pat,
        false_0 != 0,
        false_0,
        b"\0".as_ptr() as *const ::core::ffi::c_char,
        true_0 != 0,
        au_group,
    ) == FAIL
    {
        api_set_error(
            err,
            kErrorTypeException,
            b"Failed to clear autocmd\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return false_0 != 0;
    }
    return true_0 != 0;
}
