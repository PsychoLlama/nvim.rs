//! Reading extmarks back out.
//!
//! `nvim_buf_get_extmark_by_id` answers for one mark and `nvim_buf_get_extmarks`
//! for a range, with the same `details`/`hl_name` options; both render through
//! `extmark_to_array`, which is the one place a mark's decoration -- highlight,
//! sign, virtual text, virtual lines, conceal, url -- is turned back into a
//! Dict.  `extmark_get_index_from_obj` decodes the `0`/`-1`/`[row, col]`
//! spellings a range endpoint may take.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api::private::validate::{err_bad_number, err_expected};
use crate::extmark::kExtmarkNone;
use crate::narrow::{number_as_int, number_as_u32};
use crate::types::DictBuilder;
use crate::winlayer::Buf;
use core::ffi::CStr;

/// How many keys a mark's `details` dictionary can grow to.
const DETAILS_KEYS: size_t = 36;

/// The `[[text, hl], ..]` form of a decoration's virtual text: the inverse of
/// [`parse_virt_text`].
///
/// A run of text-less chunks is the stack of highlights the chunk that ends
/// it carries, so those become one array rather than chunks of their own.
pub fn virt_text_to_array(items: &[VirtTextChunk], hl_name: bool) -> Array {
    let mut chunks = Vec::with_capacity(items.len());
    let mut i = 0;
    while i < items.len() {
        let Some(run) = items[i..].iter().position(|chunk| chunk.text.is_some()) else {
            break;
        };
        let stack_end = i + run;
        let mut groups = Vec::with_capacity(if i < stack_end { stack_end - i + 1 } else { 0 });
        for chunk in &items[i..stack_end] {
            if chunk.hl_id >= 0 {
                groups.push(hl_group_name(chunk.hl_id, hl_name));
            }
        }
        let last = &items[stack_end];
        let mut chunk = Vec::with_capacity(2);
        let text = last.text.as_deref().map_or(&b""[..], CStr::to_bytes);
        chunk.push(Object::string(ApiString::from_bytes(text)));
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
    chunks.into()
}

/// One mark as `[id, row, col]`, with its decoration as a fourth item when
/// there is a `details` scratch.
///
/// `details` is the scratch the details dictionary is built in, reserved
/// for [`DETAILS_KEYS`] and reused from mark to mark: the frozen dictionary
/// is an exact copy of however many keys the mark has, where freezing the
/// reservation itself would shrink it with a `realloc` per mark.
fn extmark_to_array(
    extmark: MTPair,
    id: bool,
    details: Option<&mut DictBuilder>,
    hl_name: bool,
) -> Array {
    let start: MTKey = extmark.start;
    let mut rv = Vec::with_capacity(2 + usize::from(id) + usize::from(details.is_some()));
    if id {
        rv.push(Object::integer(Integer::from(start.id)));
    }
    rv.push(Object::integer(Integer::from(start.pos.row)));
    rv.push(Object::integer(Integer::from(start.pos.col)));
    if let Some(dict) = details {
        debug_assert!(dict.is_empty() && dict.capacity() >= DETAILS_KEYS);
        dict.insert(c"ns_id", Object::integer(Integer::from(start.ns)));
        let d_right_gravity = Object::boolean(mt_right(start));
        dict.insert(c"right_gravity", d_right_gravity);
        if mt_paired(start) {
            let d_end_row = Object::integer(Integer::from(extmark.end_pos.row));
            dict.insert(c"end_row", d_end_row);
            let d_end_col = Object::integer(Integer::from(extmark.end_pos.col));
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
        decor_to_dict_legacy(dict, mt_decor(start), hl_name);
        rv.push(Object::dict(dict.take_exact()));
    }
    rv.into()
}

pub fn nvim_buf_get_extmark_by_id(
    buf: BufferHandle,
    ns_id: Integer,
    id: Integer,
    opts: &mut KeyDict_get_extmark,
) -> Result<Array, Error> {
    let rv: Array = Array::new();
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(rv);
    };
    if !ns_initialized(number_as_u32(ns_id)) {
        return Err(err_bad_number(c"ns_id", ns_id));
    }
    let details: bool = opts.details.unwrap_or(false);
    let hl_name: bool = opts.hl_name.unwrap_or(true);
    let extmark: MTPair = extmark_from_id(b, number_as_u32(ns_id), number_as_u32(id));
    if extmark.start.pos.row < 0 as int32_t {
        return Ok(rv);
    }
    let mut scratch = details.then(|| DictBuilder::with_capacity(DETAILS_KEYS));
    Ok(extmark_to_array(extmark, false, scratch.as_mut(), hl_name))
}

pub fn nvim_buf_get_extmarks(
    buf: BufferHandle,
    ns_id: Integer,
    start: Object,
    end: Object,
    opts: &mut KeyDict_get_extmarks,
) -> Result<Array, Error> {
    let mut rv = Vec::new();
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(rv.into());
    };
    if !(ns_id == -1 || ns_initialized(number_as_u32(ns_id))) {
        return Err(err_bad_number(c"ns_id", ns_id));
    }
    let details: bool = opts.details.unwrap_or(false);
    let hl_name: bool = opts.hl_name.unwrap_or(true);
    let mut type_0: ExtmarkType = kExtmarkNone;
    if let Some(named) = opts.type_0.as_ref() {
        let name = (!named.is_null()).then(|| named.as_cstr());
        type_0 = match name.map(CStr::to_bytes) {
            Some(b"sign") => EXTMARK_SIGN,
            Some(b"virt_text") => EXTMARK_VIRT_TEXT,
            Some(b"virt_lines") => EXTMARK_VIRT_LINES,
            Some(b"highlight") => EXTMARK_HIGHLIGHT,
            _ => {
                let want = c"sign, virt_text, virt_lines or highlight";
                return Err(err_expected(c"type", want, name));
            }
        };
    }
    let mut limit: Integer = opts.limit.unwrap_or(-1);
    if limit == 0 {
        return Ok(rv.into());
    } else if limit < 0 {
        limit = Integer::MAX;
    }
    let (mut l_row, mut l_col) = extmark_index_from_obj(b, ns_id, start)?;
    let (mut u_row, mut u_col) = extmark_index_from_obj(b, ns_id, end)?;
    let rv_limit: size_t = usize::try_from(limit).unwrap_or(usize::MAX);
    let reverse: bool = l_row > u_row || l_row == u_row && l_col > u_col;
    if reverse {
        limit = Integer::MAX;
        ::core::mem::swap(&mut l_row, &mut u_row);
        ::core::mem::swap(&mut l_col, &mut u_col);
    }
    let found: Vec<MTPair> = extmark_get(
        b,
        number_as_u32(ns_id),
        l_row,
        l_col,
        u_row,
        u_col,
        limit,
        type_0,
        opts.overlap.unwrap_or(false),
    );
    rv = Vec::with_capacity(found.len().min(rv_limit));
    let mut scratch = details.then(|| DictBuilder::with_capacity(DETAILS_KEYS));
    if reverse {
        // A backwards range takes the *last* `limit` marks, so the walk goes
        // the other way and the limit is applied here rather than by the
        // search.
        for mark in found.iter().rev().take(rv_limit) {
            rv.push(Object::array(extmark_to_array(
                *mark,
                true,
                scratch.as_mut(),
                hl_name,
            )));
        }
    } else {
        for mark in &found {
            rv.push(Object::array(extmark_to_array(
                *mark,
                true,
                scratch.as_mut(),
                hl_name,
            )));
        }
    }
    Ok(rv.into())
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
            return Ok((0, ColNr::ZERO));
        }
        if id == -1 {
            return Ok((MAXLNUM.get(), ColNr::MAX));
        }
        if id < 0 {
            return Err(err_bad_number(c"mark id", id));
        }
        let extmark: MTPair = extmark_from_id(buffer, number_as_u32(ns_id), number_as_u32(id));
        if extmark.start.pos.row < 0 as int32_t {
            return Err(err_bad_number(c"mark id (not found)", id));
        }
        return Ok((extmark.start.pos.row, ColNr::new(extmark.start.pos.col)));
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
            number_as_int(row)
        } else {
            MAXLNUM.get()
        },
        if col >= 0 {
            ColNr::new(number_as_int(col))
        } else {
            ColNr::MAX
        },
    ))
}

// `nvim__buf_debug_extmarks` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub fn nvim__buf_debug_extmarks(
    buf: BufferHandle,
    keys: Boolean,
    dot: Boolean,
) -> Result<ApiString, Error> {
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(ApiString::NULL);
    };
    Ok(mt_inspect(&b.b_marktree, keys, dot))
}
