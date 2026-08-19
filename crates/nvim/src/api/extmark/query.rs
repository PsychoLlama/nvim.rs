//! Reading extmarks back out.
//!
//! `nvim_buf_get_extmark_by_id` answers for one mark and `nvim_buf_get_extmarks`
//! for a range, with the same `details`/`hl_name` options; both render through
//! `extmark_to_array`, which is the one place a mark's decoration -- highlight,
//! sign, virtual text, virtual lines, conceal, url -- is turned back into a
//! Dict.  `extmark_get_index_from_obj` decodes the `0`/`-1`/`[row, col]`
//! spellings a range endpoint may take.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported, array_add, dict_put};

pub unsafe fn virt_text_to_array(
    mut vt: VirtText,
    mut hl_name: bool,
    mut arena: *mut Arena,
) -> Array {
    unsafe {
        let mut chunks: Array = arena_array(arena, vt.size);
        let mut i: size_t = 0 as size_t;
        while i < vt.size {
            let mut j: size_t = i;
            while j < vt.size {
                if !(*vt.items.add(j)).text.is_null() {
                    break;
                }
                j = j.wrapping_add(1);
            }
            let mut hl_array: Array = arena_array(
                arena,
                if i < j {
                    j.wrapping_sub(i).wrapping_add(1 as size_t)
                } else {
                    0 as size_t
                },
            );
            while i < j {
                let mut hl_id: ::core::ffi::c_int = (*vt.items.add(i)).hl_id;
                if hl_id >= 0 as ::core::ffi::c_int {
                    array_add(&mut hl_array, hl_group_name(hl_id, hl_name));
                }
                i = i.wrapping_add(1);
            }
            let mut text: *mut ::core::ffi::c_char = (*vt.items.add(i)).text;
            let mut hl_id_0: ::core::ffi::c_int = (*vt.items.add(i)).hl_id;
            let mut chunk: Array = arena_array(arena, 2 as size_t);
            array_add(&mut chunk, Object::string(cstr_as_string(text)));
            if hl_array.size > 0 as size_t {
                if hl_id_0 >= 0 as ::core::ffi::c_int {
                    array_add(&mut hl_array, hl_group_name(hl_id_0, hl_name));
                }
                array_add(&mut chunk, Object::array(hl_array));
            } else if hl_id_0 >= 0 as ::core::ffi::c_int {
                array_add(&mut chunk, hl_group_name(hl_id_0, hl_name));
            }
            array_add(&mut chunks, Object::array(chunk));
            i = i.wrapping_add(1);
        }
        return chunks;
    }
}

unsafe fn extmark_to_array(
    mut extmark: MTPair,
    mut id: bool,
    mut add_dict: bool,
    mut hl_name: bool,
    mut arena: *mut Arena,
) -> Array {
    unsafe {
        let mut start: MTKey = extmark.start;
        let mut rv: Array = arena_array(arena, 4 as size_t);
        if id {
            array_add(&mut rv, Object::integer(start.id as Integer));
        }
        array_add(&mut rv, Object::integer(start.pos.row as Integer));
        array_add(&mut rv, Object::integer(start.pos.col as Integer));
        if add_dict {
            let mut dict: Dict = arena_dict(
                arena,
                ::core::mem::size_of::<[KeySetLink; 36]>()
                    .wrapping_div(::core::mem::size_of::<KeySetLink>())
                    .wrapping_div(
                        (::core::mem::size_of::<[KeySetLink; 36]>()
                            .wrapping_rem(::core::mem::size_of::<KeySetLink>())
                            == 0) as ::core::ffi::c_int as size_t,
                    ),
            );
            dict_put(&mut dict, c"ns_id", Object::integer(start.ns as Integer));
            dict_put(
                &mut dict,
                c"right_gravity",
                Object::boolean(mt_right(start)),
            );
            if mt_paired(start) {
                dict_put(
                    &mut dict,
                    c"end_row",
                    Object::integer(extmark.end_pos.row as Integer),
                );
                dict_put(
                    &mut dict,
                    c"end_col",
                    Object::integer(extmark.end_pos.col as Integer),
                );
                dict_put(
                    &mut dict,
                    c"end_right_gravity",
                    Object::boolean(extmark.end_right_gravity),
                );
            }
            if mt_no_undo(start) {
                dict_put(&mut dict, c"undo_restore", Object::boolean(false));
            }
            if mt_invalidate(start) {
                dict_put(&mut dict, c"invalidate", Object::boolean(true));
            }
            if mt_invalid(start) {
                dict_put(&mut dict, c"invalid", Object::boolean(true));
            }
            decor_to_dict_legacy(&mut dict, mt_decor(start), hl_name, arena);
            array_add(&mut rv, Object::dict(dict));
        }
        return rv;
    }
}

pub unsafe fn nvim_buf_get_extmark_by_id(
    buf: Buffer,
    ns_id: Integer,
    id: Integer,
    opts: *mut KeyDict_get_extmark,
    arena: *mut Arena,
) -> Result<Array, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut rv: Array = ARRAY_DICT_INIT;
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return rv.reported(error);
        }
        if !ns_initialized(ns_id as uint32_t) {
            api_err_invalid(
                err,
                c"ns_id".as_ptr(),
                ::core::ptr::null::<::core::ffi::c_char>(),
                ns_id as int64_t,
                false,
            );
            return rv.reported(error);
        }
        let mut details: bool = (*opts).details;
        let mut hl_name: bool = if has_key(
            (*opts).is_set__get_extmark_,
            KEYSET_OPTIDX_get_extmark__hl_name,
        ) {
            (*opts).hl_name as ::core::ffi::c_int
        } else {
            true_0
        } != 0;
        let mut extmark: MTPair = extmark_from_id(b, ns_id as uint32_t, id as uint32_t);
        if extmark.start.pos.row < 0 as int32_t {
            return rv.reported(error);
        }
        return extmark_to_array(extmark, false, details, hl_name, arena).reported(error);
    }
}

pub unsafe fn nvim_buf_get_extmarks(
    buf: Buffer,
    ns_id: Integer,
    start: Object,
    end: Object,
    opts: *mut KeyDict_get_extmarks,
    arena: *mut Arena,
) -> Result<Array, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut rv: Array = ARRAY_DICT_INIT;
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return rv.reported(error);
        }
        if !(ns_id == -1 as Integer || ns_initialized(ns_id as uint32_t) as ::core::ffi::c_int != 0)
        {
            api_err_invalid(
                err,
                c"ns_id".as_ptr(),
                ::core::ptr::null::<::core::ffi::c_char>(),
                ns_id as int64_t,
                false,
            );
            return rv.reported(error);
        }
        let mut details: bool = (*opts).details;
        let mut hl_name: bool = if has_key(
            (*opts).is_set__get_extmarks_,
            KEYSET_OPTIDX_get_extmarks__hl_name,
        ) {
            (*opts).hl_name as ::core::ffi::c_int
        } else {
            true_0
        } != 0;
        let mut type_0: ExtmarkType = kExtmarkNone;
        if has_key(
            (*opts).is_set__get_extmarks_,
            KEYSET_OPTIDX_get_extmarks__type,
        ) {
            if strequal((*opts).type_0.data, c"sign".as_ptr()) {
                type_0 = kExtmarkSign;
            } else if strequal((*opts).type_0.data, c"virt_text".as_ptr()) {
                type_0 = kExtmarkVirtText;
            } else if strequal((*opts).type_0.data, c"virt_lines".as_ptr()) {
                type_0 = kExtmarkVirtLines;
            } else if strequal((*opts).type_0.data, c"highlight".as_ptr()) {
                type_0 = kExtmarkHighlight;
            } else if true {
                api_err_exp(
                    err,
                    c"type".as_ptr(),
                    c"sign, virt_text, virt_lines or highlight".as_ptr(),
                    (*opts).type_0.data,
                );
                return rv.reported(error);
            }
        }
        let mut limit: Integer = if has_key(
            (*opts).is_set__get_extmarks_,
            KEYSET_OPTIDX_get_extmarks__limit,
        ) {
            (*opts).limit
        } else {
            -1 as Integer
        };
        if limit == 0 as Integer {
            return rv.reported(error);
        } else if limit < 0 as Integer {
            limit = INT64_MAX as Integer;
        }
        let mut l_row: ::core::ffi::c_int = 0;
        let mut l_col: colnr_T = 0;
        if !extmark_get_index_from_obj(b, ns_id, start, &raw mut l_row, &raw mut l_col, err) {
            return rv.reported(error);
        }
        let mut u_row: ::core::ffi::c_int = 0;
        let mut u_col: colnr_T = 0;
        if !extmark_get_index_from_obj(b, ns_id, end, &raw mut u_row, &raw mut u_col, err) {
            return rv.reported(error);
        }
        let mut rv_limit: size_t = limit as size_t;
        let mut reverse: bool = l_row > u_row || l_row == u_row && l_col > u_col;
        if reverse {
            limit = INT64_MAX as Integer;
            ::core::mem::swap(&mut l_row, &mut u_row);
            ::core::mem::swap(&mut l_col, &mut u_col);
        }
        let mut marks: ExtmarkInfoArray = extmark_get(
            b,
            ns_id as uint32_t,
            l_row,
            l_col,
            u_row,
            u_col,
            limit,
            type_0,
            (*opts).overlap,
        );
        rv = arena_array(
            arena,
            if marks.size < rv_limit {
                marks.size
            } else {
                rv_limit
            },
        );
        if reverse {
            let mut i: ::core::ffi::c_int =
                marks.size as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
            while i >= 0 as ::core::ffi::c_int && rv.size < rv_limit {
                array_add(
                    &mut rv,
                    Object::array(extmark_to_array(
                        *marks.items.offset(i as isize),
                        true,
                        details,
                        hl_name,
                        arena,
                    )),
                );
                i -= 1;
            }
        } else {
            let mut i_0: size_t = 0 as size_t;
            while i_0 < marks.size {
                array_add(
                    &mut rv,
                    Object::array(extmark_to_array(
                        *marks.items.add(i_0),
                        true,
                        details,
                        hl_name,
                        arena,
                    )),
                );
                i_0 = i_0.wrapping_add(1);
            }
        }
        xfree(marks.items as *mut ::core::ffi::c_void);
        marks.capacity = 0 as size_t;
        marks.size = marks.capacity;
        marks.items = ::core::ptr::null_mut::<MTPair>();
        return rv.reported(error);
    }
}

unsafe fn extmark_get_index_from_obj(
    mut buf: *mut buf_T,
    mut ns_id: Integer,
    mut obj: Object,
    mut row: *mut ::core::ffi::c_int,
    mut col: *mut colnr_T,
    mut err: *mut Error,
) -> bool {
    unsafe {
        if obj.type_0 as ::core::ffi::c_uint
            == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut id: Integer = obj.data.integer;
            if id == 0 as Integer {
                *row = 0 as ::core::ffi::c_int;
                *col = 0 as ::core::ffi::c_int as colnr_T;
                return true;
            } else if id == -1 as Integer {
                *row = MAXLNUM as ::core::ffi::c_int;
                *col = MAXCOL as ::core::ffi::c_int as colnr_T;
                return true;
            } else if id < 0 as Integer {
                if true {
                    api_err_invalid(
                        err,
                        c"mark id".as_ptr(),
                        ::core::ptr::null::<::core::ffi::c_char>(),
                        id as int64_t,
                        false,
                    );
                    return false;
                }
            }
            let mut extmark: MTPair = extmark_from_id(buf, ns_id as uint32_t, id as uint32_t);
            if !(extmark.start.pos.row >= 0 as int32_t) {
                api_err_invalid(
                    err,
                    c"mark id (not found)".as_ptr(),
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    id as int64_t,
                    false,
                );
                return false;
            }
            *row = extmark.start.pos.row as ::core::ffi::c_int;
            *col = extmark.start.pos.col as colnr_T;
            return true;
        } else if obj.type_0 as ::core::ffi::c_uint
            == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut pos: Array = obj.data.array;
            if !(pos.size == 2 as size_t
                && (*pos.items.offset(0 as ::core::ffi::c_int as isize)).type_0
                    as ::core::ffi::c_uint
                    == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*pos.items.offset(1 as ::core::ffi::c_int as isize)).type_0
                    as ::core::ffi::c_uint
                    == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                api_err_exp(
                    err,
                    c"mark position".as_ptr(),
                    c"2 Integer items".as_ptr(),
                    ::core::ptr::null::<::core::ffi::c_char>(),
                );
                return false;
            }
            let mut pos_row: Integer = (*pos.items.offset(0 as ::core::ffi::c_int as isize))
                .data
                .integer;
            let mut pos_col: Integer = (*pos.items.offset(1 as ::core::ffi::c_int as isize))
                .data
                .integer;
            *row = (if pos_row >= 0 as Integer {
                pos_row
            } else {
                MAXLNUM as ::core::ffi::c_int as Integer
            }) as ::core::ffi::c_int;
            *col = (if pos_col >= 0 as Integer {
                pos_col
            } else {
                MAXCOL as ::core::ffi::c_int as Integer
            }) as colnr_T;
            return true;
        } else if true {
            api_err_exp(
                err,
                c"mark position".as_ptr(),
                c"mark id Integer or 2-item Array".as_ptr(),
                ::core::ptr::null::<::core::ffi::c_char>(),
            );
            return false;
        }
        panic!("Reached end of non-void function without returning");
    }
}

pub unsafe fn nvim__buf_debug_extmarks(
    buf: Buffer,
    keys: Boolean,
    dot: Boolean,
) -> Result<String_0, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if b.is_null() {
            return NULL_STRING.reported(error);
        }
        return mt_inspect(&mut (*b).b_marktree[0], keys, dot).reported(error);
    }
}
