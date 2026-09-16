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
use crate::api::private::helpers::Reported;
use crate::api::private::validate::{err_bad_number, err_expected};
use crate::winlayer::Buf;
use crate::winlayer::Live;

/// How many keys a mark's `details` dictionary can grow to.
const DETAILS_KEYS: size_t = 36;

/// The `[[text, hl], ..]` form of a decoration's virtual text: the inverse of
/// [`parse_virt_text`].
///
/// A run of text-less chunks is the stack of highlights the chunk that ends
/// it carries, so those become one array rather than chunks of their own.
pub fn virt_text_to_array(vt: VirtText, hl_name: bool) -> Array {
    // SAFETY: the caller's promise: `size` initialized chunks. An empty
    // vector has never allocated, so its `items` is null and Rust will not
    // make a slice over one.
    let items: &[VirtTextChunk] = if vt.items.is_null() {
        &[]
    } else {
        unsafe { ::core::slice::from_raw_parts(vt.items, vt.size) }
    };
    let mut chunks = Array::with_capacity(vt.size);
    let mut i = 0;
    while i < items.len() {
        let Some(run) = items[i..].iter().position(|chunk| !chunk.text.is_null()) else {
            break;
        };
        let stack_end = i + run;
        let mut groups = Array::with_capacity(if i < stack_end { stack_end - i + 1 } else { 0 });
        for chunk in &items[i..stack_end] {
            if chunk.hl_id >= 0 {
                groups.push(hl_group_name(chunk.hl_id, hl_name));
            }
        }
        let last = items[stack_end];
        let mut chunk = Array::with_capacity(2);
        // SAFETY: the chunk's text is NUL-terminated.
        chunk.push(Object::string(unsafe { cstr_to_string(last.text) }));
        if groups.is_empty() {
            if last.hl_id >= 0 {
                chunk.push(hl_group_name(last.hl_id, hl_name));
            }
        } else {
            if last.hl_id >= 0 {
                groups.push(hl_group_name(last.hl_id, hl_name));
            }
            chunk.push(Object::array(groups));
        }
        chunks.push(Object::array(chunk));
        i = stack_end + 1;
    }
    chunks
}

/// One mark as `[id, row, col]`, with its decoration as a fourth item when
/// `add_dict`.
fn extmark_to_array(extmark: MTPair, id: bool, add_dict: bool, hl_name: bool) -> Array {
    let start: MTKey = extmark.start;
    let mut rv = Array::with_capacity(4);
    if id {
        rv.push(Object::integer(start.id as Integer));
    }
    rv.push(Object::integer(start.pos.row as Integer));
    rv.push(Object::integer(start.pos.col as Integer));
    if add_dict {
        // As many keys as `decor_to_dict_legacy` and the block above can add.
        let mut dict = ApiDict::with_capacity(DETAILS_KEYS);
        dict.insert(c"ns_id", Object::integer(start.ns as Integer));
        let d_right_gravity = Object::boolean(mt_right(start));
        dict.insert(c"right_gravity", d_right_gravity);
        if mt_paired(start) {
            let d_end_row = Object::integer(extmark.end_pos.row as Integer);
            dict.insert(c"end_row", d_end_row);
            let d_end_col = Object::integer(extmark.end_pos.col as Integer);
            dict.insert(c"end_col", d_end_col);
            let gravity = Object::boolean(extmark.end_right_gravity);
            dict.insert(c"end_right_gravity", gravity);
        }
        if mt_no_undo(start) {
            dict.insert(c"undo_restore", Object::boolean(false));
        }
        if mt_invalidate(start) {
            dict.insert(c"invalidate", Object::boolean(true));
        }
        if mt_invalid(start) {
            dict.insert(c"invalid", Object::boolean(true));
        }
        // SAFETY: the mark's own decoration.
        unsafe { decor_to_dict_legacy(&mut dict, mt_decor(start), hl_name) };
        rv.push(Object::dict(dict));
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
) -> Result<Array, Error> {
    // SAFETY: the dispatcher's keyset outlives this call.
    let opts = unsafe { Live::<KeyDict_get_extmark>::new(opts) };
    let mut error = Error::none();
    let rv: Array = Array::EMPTY;
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(rv);
    };
    if !ns_initialized(ns_id as uint32_t) {
        error = err_bad_number(c"ns_id", ns_id);
        return rv.reported(error);
    }
    let details: bool = opts.details.unwrap_or(false);
    let hl_name: bool = opts.hl_name.unwrap_or(true);
    let extmark: MTPair = extmark_from_id(b, ns_id as uint32_t, id as uint32_t);
    if extmark.start.pos.row < 0 as int32_t {
        return rv.reported(error);
    }
    extmark_to_array(extmark, false, details, hl_name).reported(error)
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
) -> Result<Array, Error> {
    // SAFETY: the dispatcher's keyset outlives this call.
    let opts = unsafe { Live::<KeyDict_get_extmarks>::new(opts) };
    let mut error = Error::none();
    let mut rv: Array = Array::EMPTY;
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(rv);
    };
    if !(ns_id == -1 || ns_initialized(ns_id as uint32_t)) {
        error = err_bad_number(c"ns_id", ns_id);
        return rv.reported(error);
    }
    let details: bool = opts.details.unwrap_or(false);
    let hl_name: bool = opts.hl_name.unwrap_or(true);
    let mut type_0: ExtmarkType = kExtmarkNone;
    if let Some(named) = opts.type_0.as_ref() {
        // SAFETY: the keyset's string names its own NUL-terminated bytes.
        let name = unsafe { crate::cstr::at_opt(named.data()) };
        type_0 = match name.map(core::ffi::CStr::to_bytes) {
            Some(b"sign") => kExtmarkSign,
            Some(b"virt_text") => kExtmarkVirtText,
            Some(b"virt_lines") => kExtmarkVirtLines,
            Some(b"highlight") => kExtmarkHighlight,
            _ => {
                let want = c"sign, virt_text, virt_lines or highlight";
                error = err_expected(c"type", want, name);
                return rv.reported(error);
            }
        };
    }
    let mut limit: Integer = opts.limit.unwrap_or(-1);
    if limit == 0 {
        return rv.reported(error);
    } else if limit < 0 {
        limit = Integer::MAX;
    }
    let (mut l_row, mut l_col) = extmark_index_from_obj(b, ns_id, start)?;
    let (mut u_row, mut u_col) = extmark_index_from_obj(b, ns_id, end)?;
    let rv_limit: size_t = limit as size_t;
    let reverse: bool = l_row > u_row || l_row == u_row && l_col > u_col;
    if reverse {
        limit = Integer::MAX;
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
        opts.overlap.unwrap_or(false),
    );
    rv = Array::with_capacity(if marks.size < rv_limit {
        marks.size
    } else {
        rv_limit
    });
    // SAFETY: `extmark_get` answers `size` initialized marks -- or, when it
    // found none, a vector that never allocated and whose `items` is null.
    let found: &[MTPair] = if marks.items.is_null() {
        &[]
    } else {
        unsafe { ::core::slice::from_raw_parts(marks.items, marks.size) }
    };
    if reverse {
        // A backwards range takes the *last* `limit` marks, so the walk goes
        // the other way and the limit is applied here rather than by the
        // search.
        for mark in found.iter().rev().take(rv_limit) {
            rv.push(Object::array(extmark_to_array(
                *mark, true, details, hl_name,
            )));
        }
    } else {
        for mark in found {
            rv.push(Object::array(extmark_to_array(
                *mark, true, details, hl_name,
            )));
        }
    }
    // SAFETY: the block `extmark_get` handed this call.
    unsafe { xfree(marks.items.cast::<::core::ffi::c_void>()) };
    rv.reported(error)
}

/// A range endpoint, in the spellings one may take: a mark id, `0` for the
/// start of the buffer, `-1` for the end of it, or an explicit `[row, col]`.
fn extmark_index_from_obj(
    buffer: Buf,
    ns_id: Integer,
    obj: Object,
) -> Result<(::core::ffi::c_int, ColNr), Error> {
    if let Object::Integer(id) = obj {
        if id == 0 {
            return Ok((0, 0));
        }
        if id == -1 {
            return Ok((MAXLNUM, MAXCOL));
        }
        if id < 0 {
            return Err(err_bad_number(c"mark id", id));
        }
        let extmark: MTPair = extmark_from_id(buffer, ns_id as uint32_t, id as uint32_t);
        if extmark.start.pos.row < 0 as int32_t {
            return Err(err_bad_number(c"mark id (not found)", id));
        }
        return Ok((extmark.start.pos.row, extmark.start.pos.col as ColNr));
    }
    let Some(pos) = obj.as_array() else {
        let want = c"mark id Integer or 2-item Array";
        return Err(err_expected(c"mark position", want, None));
    };
    let two = match pos.len() {
        2 => pos[0].as_integer().zip(pos[1].as_integer()),
        _ => None,
    };
    let Some((row, col)) = two else {
        let want = c"2 Integer items";
        return Err(err_expected(c"mark position", want, None));
    };
    // A negative half means "as far as the buffer goes" in that dimension.
    Ok((
        if row >= 0 {
            row as ::core::ffi::c_int
        } else {
            MAXLNUM
        },
        if col >= 0 { col as ColNr } else { MAXCOL },
    ))
}

// `nvim__buf_debug_extmarks` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__buf_debug_extmarks(
    buf: BufferHandle,
    keys: Boolean,
    dot: Boolean,
) -> Result<String_0, Error> {
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(String_0::NULL);
    };
    Ok(unsafe { mt_inspect(&mut (*b.raw()).b_marktree, keys, dot) })
}
