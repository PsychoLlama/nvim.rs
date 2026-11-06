//! Buffer highlights and virtual text before extmarks.
//!
//! `nvim_buf_add_highlight`, `nvim_buf_clear_highlight` and
//! `nvim_buf_set_virtual_text` are the pre-namespace decoration API; each is
//! now a shim that allocates or reuses a namespace (`src2ns`) and calls
//! `extmark_set`.  `nvim_buf_get_number` is the handle-to-number accessor from
//! before handles *were* numbers.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api::private::helpers::find_buffer_by_handle;
use crate::api::private::validate::{Bad, err_invalid};
use crate::marktree::key::MtFlags;
use crate::narrow::{number_as_int, number_as_u32};
use crate::winlayer::Buf;

pub fn nvim_buf_get_number(buffer: BufferHandle) -> Result<Integer, Error> {
    let Some(buf) = find_buffer_by_handle(buffer)? else {
        return Ok(0 as Integer);
    };
    Ok(Integer::from(buf.handle))
}

/// The namespace `src_id` names, allocating one for the 0 every pre-namespace
/// caller passed and answering the reserved "no namespace" id for a negative.
fn src2ns(src_id: &mut Integer) -> uint32_t {
    if *src_id == 0 {
        *src_id = nvim_create_namespace(ApiString::NULL);
    }
    if *src_id < 0 {
        return (1u32 << 31).wrapping_sub(1);
    }
    number_as_u32(*src_id)
}

/// `extmark_set` as the three shims here use it: one decoration over one
/// range of `buffer`, created rather than moved, so it needs no id of its own
/// and has nowhere to report. The mark takes `decor` over.
#[expect(clippy::too_many_arguments, reason = "one per extmark_set parameter")]
fn set_decor(
    buffer: Buf,
    ns: uint32_t,
    line: ::core::ffi::c_int,
    col: ColNr,
    end_line: ::core::ffi::c_int,
    end_col: ColNr,
    decor: DecorInline,
    flags: MtFlags,
) {
    // The mark is new, so it is given no id slot.
    extmark_set(
        buffer, ns, None, line, col, end_line, end_col, decor, flags, true, false, false, false,
    );
}

pub fn nvim_buf_clear_highlight(
    buffer: BufferHandle,
    ns_id: Integer,
    line_start: Integer,
    line_end: Integer,
) -> Result<(), Error> {
    nvim_buf_clear_namespace(buffer, ns_id, line_start, line_end)
}

pub fn nvim_buf_add_highlight(
    buffer: BufferHandle,
    mut ns_id: Integer,
    hl_group: ApiString,
    line: Integer,
    col_start: Integer,
    mut col_end: Integer,
) -> Result<Integer, Error> {
    let Some(buf) = find_buffer_by_handle(buffer)? else {
        return Ok(0 as Integer);
    };
    let out_of_range = c"out of range";
    if line < 0 as Integer || line >= Integer::from(MAXLNUM) {
        return Err(err_invalid(c"line number", Bad::Bare(out_of_range)));
    }
    if col_start < 0 as Integer || col_start > Integer::from(MAXCOL) {
        return Err(err_invalid(c"column", Bad::Bare(out_of_range)));
    }
    if col_end < 0 as Integer || col_end > Integer::from(MAXCOL) {
        col_end = Integer::from(MAXCOL);
    }
    let ns = src2ns(&mut ns_id);
    if line >= Integer::from(buf.b_ml.ml_line_count) {
        return Ok(ns_id);
    }
    if hl_group.is_empty() {
        return Ok(ns_id);
    }
    let hl_id = syn_check_group(hl_group.as_bytes());

    // A highlight that runs to the end of the line is one that ends at
    // column zero of the next.
    let mut end_line = number_as_int(line);
    if col_end == Integer::from(MAXCOL) {
        col_end = 0 as Integer;
        end_line += 1;
    }
    set_decor(
        buf,
        ns,
        number_as_int(line),
        ColNr::new(number_as_int(col_start)),
        end_line,
        ColNr::new(number_as_int(col_end)),
        decor_inline_hl(hl_id),
        MtFlags::DECOR_HL,
    );
    Ok(ns_id)
}

pub fn nvim_buf_set_virtual_text(
    buffer: BufferHandle,
    mut src_id: Integer,
    line: Integer,
    chunks: Array,
    _opts: &mut KeyDict_empty,
) -> Result<Integer, Error> {
    let Some(buf) = find_buffer_by_handle(buffer)? else {
        return Ok(0 as Integer);
    };
    if line < 0 as Integer || line >= Integer::from(MAXLNUM) {
        return Err(Error::validation(c"Line number outside range"));
    }
    let ns_id = src2ns(&mut src_id);
    let mut width: ::core::ffi::c_int = 0;
    let virt_text: VirtText = parse_virt_text(&chunks, Some(&mut width))?;

    let lnum = number_as_int(line);
    if let Some(existing) = decor_find_virttext(buf, lnum, uint64_t::from(ns_id)) {
        // Replacing what this namespace already put on the line, rather than
        // stacking a second decoration on it.
        with_decor_virt_mut(existing, |vt| {
            *vt.data.text_mut() = virt_text;
            vt.width = width;
        });
        return Ok(src_id);
    }

    let vt = DecorVirtText {
        flags: 0 as uint8_t,
        hl_mode: kHlModeUnknown,
        priority: 0 as DecorPriority,
        width,
        col: 0 as ::core::ffi::c_int,
        pos: kVPosEndOfLine,
        data: DecorVirtText_data::Text(virt_text),
        next: DECOR_ID_INVALID,
    };
    let decor = DecorInline::Ext(DecorExt {
        sh_idx: DECOR_ID_INVALID,
        vt_idx: decor_put_vt(vt, DECOR_ID_INVALID),
    });
    set_decor(
        buf,
        ns_id,
        lnum,
        ColNr::ZERO,
        -1 as ::core::ffi::c_int,
        ColNr::NONE,
        decor,
        MtFlags::NONE,
    );
    Ok(src_id)
}
