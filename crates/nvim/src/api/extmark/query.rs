//! Reading extmarks back out.
//!
//! `nvim_buf_get_extmark_by_id` answers for one mark and `nvim_buf_get_extmarks`
//! for a range, with the same `details`/`hl_name` options; both render through
//! `extmark_to_array`, which is the one place a mark's decoration -- highlight,
//! sign, virtual text, virtual lines, conceal, url -- is turned back into a
//! Dict.  `extmark_get_index_from_obj` decodes the `0`/`-1`/`[row, col]`
//! spellings a range endpoint may take.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::api::private::helpers::{Reported, array_add, dict_put};
use crate::api::private::validate::{err_bad_number, err_expected};
use crate::winlayer::Buf;
use crate::winlayer::Live;

/// # Safety
///
/// `arena` must point at a live arena, which the memory this answers with is
/// taken from and must outlive.
pub unsafe fn virt_text_to_array(vt: VirtText, hl_name: bool, arena: *mut Arena) -> Array {
    let mut chunks: Array = arena_array(arena, vt.size);
    let mut i: size_t = 0 as size_t;
    while i < vt.size {
        let mut j: size_t = i;
        while j < vt.size {
            if !unsafe { (*vt.items.add(j)).text }.is_null() {
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
            let hl_id: ::core::ffi::c_int = unsafe { (*vt.items.add(i)).hl_id };
            if hl_id >= 0 as ::core::ffi::c_int {
                unsafe { array_add(&mut hl_array, hl_group_name(hl_id, hl_name)) };
            }
            i = i.wrapping_add(1);
        }
        let text: *mut ::core::ffi::c_char = unsafe { (*vt.items.add(i)).text };
        let hl_id_0: ::core::ffi::c_int = unsafe { (*vt.items.add(i)).hl_id };
        let mut chunk: Array = arena_array(arena, 2 as size_t);
        unsafe { array_add(&mut chunk, Object::string(cstr_as_string(text))) };
        if hl_array.size > 0 as size_t {
            if hl_id_0 >= 0 as ::core::ffi::c_int {
                unsafe { array_add(&mut hl_array, hl_group_name(hl_id_0, hl_name)) };
            }
            unsafe { array_add(&mut chunk, Object::array(hl_array)) };
        } else if hl_id_0 >= 0 as ::core::ffi::c_int {
            unsafe { array_add(&mut chunk, hl_group_name(hl_id_0, hl_name)) };
        }
        unsafe { array_add(&mut chunks, Object::array(chunk)) };
        i = i.wrapping_add(1);
    }
    chunks
}

/// # Safety
///
/// `arena` must point at a live arena, which the memory this answers with is
/// taken from and must outlive.
unsafe fn extmark_to_array(
    extmark: MTPair,
    id: bool,
    add_dict: bool,
    hl_name: bool,
    arena: *mut Arena,
) -> Array {
    let start: MTKey = extmark.start;
    let mut rv: Array = arena_array(arena, 4 as size_t);
    if id {
        unsafe { array_add(&mut rv, Object::integer(start.id as Integer)) };
    }
    unsafe { array_add(&mut rv, Object::integer(start.pos.row as Integer)) };
    unsafe { array_add(&mut rv, Object::integer(start.pos.col as Integer)) };
    if add_dict {
        let mut dict: ApiDict = arena_dict(
            arena,
            ::core::mem::size_of::<[KeySetLink; 36]>()
                .wrapping_div(::core::mem::size_of::<KeySetLink>())
                .wrapping_div(
                    (::core::mem::size_of::<[KeySetLink; 36]>()
                        .wrapping_rem(::core::mem::size_of::<KeySetLink>())
                        == 0) as ::core::ffi::c_int as size_t,
                ),
        );
        unsafe { dict_put(&mut dict, c"ns_id", Object::integer(start.ns as Integer)) };
        let d_right_gravity = Object::boolean(mt_right(start));
        // SAFETY: the collection is this call's own.
        unsafe { dict_put(&mut dict, c"right_gravity", d_right_gravity) };
        if mt_paired(start) {
            let d_end_row = Object::integer(extmark.end_pos.row as Integer);
            // SAFETY: the collection is this call's own.
            unsafe { dict_put(&mut dict, c"end_row", d_end_row) };
            let d_end_col = Object::integer(extmark.end_pos.col as Integer);
            // SAFETY: the collection is this call's own.
            unsafe { dict_put(&mut dict, c"end_col", d_end_col) };
            let gravity = Object::boolean(extmark.end_right_gravity);
            // SAFETY: `dict` is this call's own.
            unsafe { dict_put(&mut dict, c"end_right_gravity", gravity) };
        }
        if mt_no_undo(start) {
            unsafe { dict_put(&mut dict, c"undo_restore", Object::boolean(false)) };
        }
        if mt_invalidate(start) {
            unsafe { dict_put(&mut dict, c"invalidate", Object::boolean(true)) };
        }
        if mt_invalid(start) {
            unsafe { dict_put(&mut dict, c"invalid", Object::boolean(true)) };
        }
        unsafe { decor_to_dict_legacy(&mut dict, mt_decor(start), hl_name, arena) };
        unsafe { array_add(&mut rv, Object::dict(dict)) };
    }
    rv
}

/// # Safety
///
/// `opts` must point at the `KeyDict_get_extmark` the dispatcher filled in,
/// live for the call. `arena` must point at a live arena, which the memory
/// this answers with is taken from and must outlive.
pub unsafe fn nvim_buf_get_extmark_by_id(
    buf: BufferHandle,
    ns_id: Integer,
    id: Integer,
    opts: *mut KeyDict_get_extmark,
    arena: *mut Arena,
) -> Result<Array, Error> {
    // SAFETY: the dispatcher's keyset outlives this call.
    let opts = unsafe { Live::<KeyDict_get_extmark>::new(opts) };
    let mut error = Error::none();
    let rv: Array = ARRAY_DICT_INIT;
    let Some(b) = find_buffer_by_handle(buf, &mut error) else {
        return rv.reported(error);
    };
    if !ns_initialized(ns_id as uint32_t) {
        error = err_bad_number(c"ns_id", ns_id);
        return rv.reported(error);
    }
    let details: bool = opts.details;
    let hl_name: bool = if has_key(
        opts.is_set__get_extmark_,
        KEYSET_OPTIDX_get_extmark__hl_name,
    ) {
        opts.hl_name as ::core::ffi::c_int
    } else {
        1
    } != 0;
    let extmark: MTPair = extmark_from_id(b, ns_id as uint32_t, id as uint32_t);
    if extmark.start.pos.row < 0 as int32_t {
        return rv.reported(error);
    }
    unsafe { extmark_to_array(extmark, false, details, hl_name, arena) }.reported(error)
}

/// # Safety
///
/// `start` must be a well-formed API object the caller owns for the call.
/// `end` must be a well-formed API object the caller owns for the call.
/// `opts` must point at the `KeyDict_get_extmarks` the dispatcher filled in,
/// live for the call. `arena` must point at a live arena, which the memory
/// this answers with is taken from and must outlive.
pub unsafe fn nvim_buf_get_extmarks(
    buf: BufferHandle,
    ns_id: Integer,
    start: Object,
    end: Object,
    opts: *mut KeyDict_get_extmarks,
    arena: *mut Arena,
) -> Result<Array, Error> {
    // SAFETY: the dispatcher's keyset outlives this call.
    let opts = unsafe { Live::<KeyDict_get_extmarks>::new(opts) };
    let mut error = Error::none();
    let mut rv: Array = ARRAY_DICT_INIT;
    let Some(b) = find_buffer_by_handle(buf, &mut error) else {
        return rv.reported(error);
    };
    if !(ns_id == -1 as Integer || ns_initialized(ns_id as uint32_t) as ::core::ffi::c_int != 0) {
        error = err_bad_number(c"ns_id", ns_id);
        return rv.reported(error);
    }
    let details: bool = opts.details;
    let hl_name: bool = if has_key(
        opts.is_set__get_extmarks_,
        KEYSET_OPTIDX_get_extmarks__hl_name,
    ) {
        opts.hl_name as ::core::ffi::c_int
    } else {
        1
    } != 0;
    let mut type_0: ExtmarkType = kExtmarkNone;
    if has_key(opts.is_set__get_extmarks_, KEYSET_OPTIDX_get_extmarks__type) {
        if unsafe { strequal(opts.type_0.data(), c"sign".as_ptr()) } {
            type_0 = kExtmarkSign;
        } else if unsafe { strequal(opts.type_0.data(), c"virt_text".as_ptr()) } {
            type_0 = kExtmarkVirtText;
        } else if unsafe { strequal(opts.type_0.data(), c"virt_lines".as_ptr()) } {
            type_0 = kExtmarkVirtLines;
        } else if unsafe { strequal(opts.type_0.data(), c"highlight".as_ptr()) } {
            type_0 = kExtmarkHighlight;
        } else if true {
            let want = c"sign, virt_text, virt_lines or highlight";
            let got = opts.type_0.data();
            // SAFETY: the keyset's string names its own NUL-terminated bytes.
            let got = unsafe { crate::cstr::at_opt(got) };
            error = err_expected(c"type", want, got);
            return rv.reported(error);
        }
    }
    let mut limit: Integer = if has_key(
        opts.is_set__get_extmarks_,
        KEYSET_OPTIDX_get_extmarks__limit,
    ) {
        opts.limit
    } else {
        -1 as Integer
    };
    if limit == 0 as Integer {
        return rv.reported(error);
    } else if limit < 0 as Integer {
        limit = INT64_MAX as Integer;
    }
    let mut l_row: ::core::ffi::c_int = 0;
    let mut l_col: ColNr = 0;
    if !unsafe {
        extmark_get_index_from_obj(b, ns_id, start, &raw mut l_row, &raw mut l_col, &mut error)
    } {
        return rv.reported(error);
    }
    let mut u_row: ::core::ffi::c_int = 0;
    let mut u_col: ColNr = 0;
    if !unsafe {
        extmark_get_index_from_obj(b, ns_id, end, &raw mut u_row, &raw mut u_col, &mut error)
    } {
        return rv.reported(error);
    }
    let rv_limit: size_t = limit as size_t;
    let reverse: bool = l_row > u_row || l_row == u_row && l_col > u_col;
    if reverse {
        limit = INT64_MAX as Integer;
        ::core::mem::swap(&mut l_row, &mut u_row);
        ::core::mem::swap(&mut l_col, &mut u_col);
    }
    let marks: ExtmarkInfoArray = extmark_get(
        b,
        ns_id as uint32_t,
        l_row,
        l_col,
        u_row,
        u_col,
        limit,
        type_0,
        opts.overlap,
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
        let mut i: ::core::ffi::c_int = marks.size as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int && rv.size < rv_limit {
            // SAFETY: `i` indexes the array `extmark_get` filled.
            let mark = unsafe { *marks.items.offset(i as isize) };
            // SAFETY: `arena` is the caller's.
            let put_value =
                unsafe { Object::array(extmark_to_array(mark, true, details, hl_name, arena)) };
            // SAFETY: the collection is this call's own.
            unsafe { array_add(&mut rv, put_value) };
            i -= 1;
        }
    } else {
        let mut i_0: size_t = 0 as size_t;
        while i_0 < marks.size {
            // SAFETY: `i_0` indexes the array `extmark_get` filled.
            let mark = unsafe { *marks.items.add(i_0) };
            // SAFETY: `arena` is the caller's.
            let put_value =
                unsafe { Object::array(extmark_to_array(mark, true, details, hl_name, arena)) };
            // SAFETY: the collection is this call's own.
            unsafe { array_add(&mut rv, put_value) };
            i_0 = i_0.wrapping_add(1);
        }
    }
    unsafe { xfree(marks.items as *mut ::core::ffi::c_void) };
    rv.reported(error)
}

/// # Safety
///
/// `obj` must be a well-formed API object the caller owns for the call. `row`
/// must point at a writable `int` the caller owns. `col` must point at a
/// writable column the caller owns.
unsafe fn extmark_get_index_from_obj(
    buffer: Buf,
    ns_id: Integer,
    obj: Object,
    row: *mut ::core::ffi::c_int,
    col: *mut ColNr,
    err: &mut Error,
) -> bool {
    if let Object::Integer(id) = obj {
        if id == 0 as Integer {
            unsafe { *row = 0 as ::core::ffi::c_int };
            unsafe { *col = 0 as ::core::ffi::c_int as ColNr };
            return true;
        } else if id == -1 as Integer {
            unsafe { *row = MAXLNUM };
            unsafe { *col = MAXCOL as ::core::ffi::c_int as ColNr };
            return true;
        } else if id < 0 as Integer && true {
            *err = err_bad_number(c"mark id", id);
            return false;
        }
        let extmark: MTPair = extmark_from_id(buffer, ns_id as uint32_t, id as uint32_t);
        if !(extmark.start.pos.row >= 0 as int32_t) {
            *err = err_bad_number(c"mark id (not found)", id);
            return false;
        }
        unsafe { *row = extmark.start.pos.row as ::core::ffi::c_int };
        unsafe { *col = extmark.start.pos.col as ColNr };
        return true;
    } else if let Object::Array(pos) = obj {
        let two = match pos.size {
            // SAFETY: a two-item array names the two items read here.
            2 => unsafe {
                (*pos.items)
                    .as_integer()
                    .zip((*pos.items.add(1)).as_integer())
            },
            _ => None,
        };
        let Some((pos_row, pos_col)) = two else {
            let want = c"2 Integer items";
            *err = err_expected(c"mark position", want, None);
            return false;
        };
        let r = (if pos_row >= 0 as Integer {
            pos_row
        } else {
            MAXLNUM as Integer
        }) as ::core::ffi::c_int;
        let c = (if pos_col >= 0 as Integer {
            pos_col
        } else {
            MAXCOL as ::core::ffi::c_int as Integer
        }) as ColNr;
        // SAFETY: the caller's own out-parameters.
        unsafe { *row = r };
        // SAFETY: as above.
        unsafe { *col = c };
        return true;
    } else if true {
        let want = c"mark id Integer or 2-item Array";
        *err = err_expected(c"mark position", want, None);
        return false;
    }
    panic!("Reached end of non-void function without returning");
}

// `nvim__buf_debug_extmarks` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__buf_debug_extmarks(
    buf: BufferHandle,
    keys: Boolean,
    dot: Boolean,
) -> Result<String_0, Error> {
    let mut error = Error::none();
    let Some(b) = find_buffer_by_handle(buf, &mut error) else {
        return String_0::NULL.reported(error);
    };
    unsafe { mt_inspect(&mut (*b.raw()).b_marktree, keys, dot) }.reported(error)
}
