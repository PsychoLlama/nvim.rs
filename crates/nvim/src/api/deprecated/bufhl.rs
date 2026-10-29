//! Buffer highlights and virtual text before extmarks.
//!
//! `nvim_buf_add_highlight`, `nvim_buf_clear_highlight` and
//! `nvim_buf_set_virtual_text` are the pre-namespace decoration API; each is
//! now a shim that allocates or reuses a namespace (`src2ns`) and calls
//! `extmark_set`.  `nvim_buf_get_number` is the handle-to-number accessor from
//! before handles *were* numbers.

#![forbid(unsafe_code)]

use super::*;
use crate::api::private::helpers::{Reported, find_buffer_by_handle};
use crate::api::private::validate::{Bad, err_invalid};
use crate::marktree::key::MtFlags;
use crate::winlayer::Buf;

pub fn nvim_buf_get_number(buffer: BufferHandle) -> Result<Integer, Error> {
    let Some(buf) = find_buffer_by_handle(buffer)? else {
        return Ok(0 as Integer);
    };
    Ok(buf.handle as Integer)
}

/// The namespace `src_id` names, allocating one for the 0 every pre-namespace
/// caller passed and answering the reserved "no namespace" id for a negative.
fn src2ns(src_id: &mut Integer) -> uint32_t {
    if *src_id == 0 {
        *src_id = nvim_create_namespace(String_0::NULL);
    }
    if *src_id < 0 {
        return (1u32 << 31).wrapping_sub(1);
    }
    *src_id as uint32_t
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
    hl_group: String_0,
    line: Integer,
    col_start: Integer,
    mut col_end: Integer,
) -> Result<Integer, Error> {
    let mut error = Error::none();
    let Some(buf) = find_buffer_by_handle(buffer)? else {
        return Ok(0 as Integer);
    };
    let out_of_range = c"out of range";
    if line < 0 as Integer || line >= MAXLNUM as Integer {
        error = err_invalid(c"line number", Bad::Bare(out_of_range));
        return (0 as Integer).reported(error);
    }
    if col_start < 0 as Integer || col_start > MAXCOL as ::core::ffi::c_int as Integer {
        error = err_invalid(c"column", Bad::Bare(out_of_range));
        return (0 as Integer).reported(error);
    }
    if col_end < 0 as Integer || col_end > MAXCOL as ::core::ffi::c_int as Integer {
        col_end = MAXCOL as ::core::ffi::c_int as Integer;
    }
    let ns = src2ns(&mut ns_id);
    if line >= buf.b_ml.ml_line_count as Integer {
        return ns_id.reported(error);
    }
    if hl_group.is_empty() {
        return ns_id.reported(error);
    }
    let hl_id = syn_check_group(hl_group.as_bytes());

    // A highlight that runs to the end of the line is one that ends at
    // column zero of the next.
    let mut end_line = line as ::core::ffi::c_int;
    if col_end == MAXCOL as ::core::ffi::c_int as Integer {
        col_end = 0 as Integer;
        end_line += 1;
    }
    set_decor(
        buf,
        ns,
        line as ::core::ffi::c_int,
        col_start as ColNr,
        end_line,
        col_end as ColNr,
        decor_inline_hl(hl_id),
        MtFlags::DECOR_HL,
    );
    ns_id.reported(error)
}

pub fn nvim_buf_set_virtual_text(
    buffer: BufferHandle,
    mut src_id: Integer,
    line: Integer,
    chunks: Array,
    _opts: &mut KeyDict_empty,
) -> Result<Integer, Error> {
    let mut error = Error::none();
    let Some(buf) = find_buffer_by_handle(buffer)? else {
        return Ok(0 as Integer);
    };
    if line < 0 as Integer || line >= MAXLNUM as Integer {
        error = Error::validation(c"Line number outside range");
        return (0 as Integer).reported(error);
    }
    let ns_id = src2ns(&mut src_id);
    let mut width: ::core::ffi::c_int = 0;
    let virt_text: VirtText = parse_virt_text(&chunks, Some(&mut width))?;

    let lnum = line as ::core::ffi::c_int;
    if let Some(existing) = decor_find_virttext(buf, lnum, ns_id as uint64_t) {
        // Replacing what this namespace already put on the line, rather than
        // stacking a second decoration on it.
        with_decor_virt_mut(existing, |vt| {
            *vt.data.text_mut() = virt_text;
            vt.width = width;
        });
        return src_id.reported(error);
    }

    let vt = DecorVirtText {
        flags: 0 as uint8_t,
        hl_mode: kHlModeUnknown as ::core::ffi::c_int as uint8_t,
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
        0 as ColNr,
        -1 as ::core::ffi::c_int,
        -1 as ColNr,
        decor,
        MtFlags::NONE,
    );
    src_id.reported(error)
}
