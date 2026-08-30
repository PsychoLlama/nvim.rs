//! Buffer highlights and virtual text before extmarks.
//!
//! `nvim_buf_add_highlight`, `nvim_buf_clear_highlight` and
//! `nvim_buf_set_virtual_text` are the pre-namespace decoration API; each is
//! now a shim that allocates or reuses a namespace (`src2ns`) and calls
//! `extmark_set`.  `nvim_buf_get_number` is the handle-to-number accessor from
//! before handles *were* numbers.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported, buffer_by_handle};
use crate::api::private::validate::err_invalid_ptr;

pub unsafe fn nvim_buf_get_number(buffer: Buffer) -> Result<Integer, Error> {
    let mut error = ERROR_INIT;
    let Some(buf) = buffer_by_handle(buffer, &mut error) else {
        return (0 as Integer).reported(error);
    };
    (buf.handle as Integer).reported(error)
}

/// The namespace `src_id` names, allocating one for the 0 every pre-namespace
/// caller passed and answering the reserved "no namespace" id for a negative.
fn src2ns(src_id: &mut Integer) -> uint32_t {
    if *src_id == 0 {
        // SAFETY: the null string names no namespace, so nothing is read.
        *src_id = unsafe { nvim_create_namespace(String_0::NULL) };
    }
    if *src_id < 0 {
        return (1u32 << 31).wrapping_sub(1);
    }
    *src_id as uint32_t
}

/// `extmark_set` as the three shims here use it: one decoration over one
/// range of `buf`, created rather than moved, so it needs no id of its own
/// and has nowhere to report.
///
/// # Safety
/// `buf` must be a live buffer and `decor` must own whatever it points at.
#[expect(clippy::too_many_arguments, reason = "one per extmark_set parameter")]
unsafe fn set_decor(
    buf: *mut buf_T,
    ns: uint32_t,
    line: ::core::ffi::c_int,
    col: colnr_T,
    end_line: ::core::ffi::c_int,
    end_col: colnr_T,
    decor: DecorInline,
    flags: uint16_t,
) {
    // SAFETY: the caller's promise. The mark is new, so it is given no id
    // slot; a null error slot is what upstream passes here too.
    unsafe {
        extmark_set(
            buf,
            ns,
            ::core::ptr::null_mut::<uint32_t>(),
            line,
            col,
            end_line,
            end_col,
            decor,
            flags,
            true,
            false,
            false,
            false,
        );
    }
}

pub unsafe fn nvim_buf_clear_highlight(
    buffer: Buffer,
    ns_id: Integer,
    line_start: Integer,
    line_end: Integer,
) -> Result<(), Error> {
    // SAFETY: every argument is a plain integer off the wire.
    unsafe { nvim_buf_clear_namespace(buffer, ns_id, line_start, line_end) }
}

pub unsafe fn nvim_buf_add_highlight(
    buffer: Buffer,
    mut ns_id: Integer,
    hl_group: String_0,
    line: Integer,
    col_start: Integer,
    mut col_end: Integer,
) -> Result<Integer, Error> {
    let mut error = ERROR_INIT;
    let Some(buf) = buffer_by_handle(buffer, &mut error) else {
        return (0 as Integer).reported(error);
    };
    let out_of_range = c"out of range".as_ptr();
    if line < 0 as Integer || line >= MAXLNUM as ::core::ffi::c_int as Integer {
        // SAFETY: `err` is this frame's slot and both strings are static.
        error = unsafe { err_invalid_ptr(c"line number".as_ptr(), out_of_range, 0, false) };
        return (0 as Integer).reported(error);
    }
    if col_start < 0 as Integer || col_start > MAXCOL as ::core::ffi::c_int as Integer {
        // SAFETY: as above.
        error = unsafe { err_invalid_ptr(c"column".as_ptr(), out_of_range, 0, false) };
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
    // SAFETY: `hl_group` names its own bytes.
    let hl_id = unsafe { syn_check_group(hl_group.data(), hl_group.len()) };

    // A highlight that runs to the end of the line is one that ends at
    // column zero of the next.
    let mut end_line = line as ::core::ffi::c_int;
    if col_end == MAXCOL as ::core::ffi::c_int as Integer {
        col_end = 0 as Integer;
        end_line += 1;
    }
    let mut decor: DecorInline = DECOR_INLINE_INIT;
    decor.data.hl.hl_id = hl_id;
    // SAFETY: `buf` is live, and an inline highlight owns nothing.
    unsafe {
        set_decor(
            buf.raw(),
            ns,
            line as ::core::ffi::c_int,
            col_start as colnr_T,
            end_line,
            col_end as colnr_T,
            decor,
            MT_FLAG_DECOR_HL as uint16_t,
        );
    }
    ns_id.reported(error)
}

pub unsafe fn nvim_buf_set_virtual_text(
    buffer: Buffer,
    mut src_id: Integer,
    line: Integer,
    chunks: Array,
    _opts: *mut KeyDict_empty,
) -> Result<Integer, Error> {
    let mut error = ERROR_INIT;
    let Some(buf) = buffer_by_handle(buffer, &mut error) else {
        return (0 as Integer).reported(error);
    };
    if line < 0 as Integer || line >= MAXLNUM as ::core::ffi::c_int as Integer {
        error = Error::validation(c"Line number outside range");
        return (0 as Integer).reported(error);
    }
    let ns_id = src2ns(&mut src_id);
    let mut width: ::core::ffi::c_int = 0;
    // SAFETY: `chunks` is the caller's array, and `error`/`width` are this
    // frame's.
    let virt_text: VirtText = unsafe { parse_virt_text(chunks, &mut error, &raw mut width) };
    if error.is_set() {
        return (0 as Integer).reported(error);
    }

    let lnum = line as ::core::ffi::c_int;
    // SAFETY: `buf` is live.
    let existing = unsafe { decor_find_virttext(buf.raw(), lnum, ns_id as uint64_t) };
    if !existing.is_null() {
        // Replacing what this namespace already put on the line, rather than
        // stacking a second decoration on it.
        // SAFETY: `existing` is that decoration, which owns its text.
        unsafe {
            clear_virttext(&raw mut (*existing).data.virt_text);
            (*existing).data.virt_text = virt_text;
            (*existing).width = width;
        }
        return src_id.reported(error);
    }

    // SAFETY: `xmalloc` answers a block of exactly this size, which nothing
    // else names yet.
    let vt: *mut DecorVirtText = unsafe { xmalloc(size_of::<DecorVirtText>()).cast() };
    // SAFETY: as above -- the whole struct is written before it is read.
    unsafe {
        *vt = DecorVirtText {
            flags: 0 as uint8_t,
            hl_mode: kHlModeUnknown as ::core::ffi::c_int as uint8_t,
            priority: 0 as DecorPriority,
            width,
            col: 0 as ::core::ffi::c_int,
            pos: kVPosEndOfLine,
            data: DecorVirtText_data { virt_text },
            next: ::core::ptr::null_mut::<DecorVirtText>(),
        };
    }
    let decor: DecorInline = DecorInline {
        ext: true,
        data: DecorInlineData {
            ext: DecorExt {
                sh_idx: DECOR_ID_INVALID as uint32_t,
                vt,
            },
        },
    };
    // SAFETY: `buf` is live and `decor` owns `vt`, which it hands over.
    unsafe {
        set_decor(
            buf.raw(),
            ns_id,
            lnum,
            0 as colnr_T,
            -1 as ::core::ffi::c_int,
            -1 as colnr_T,
            decor,
            0 as uint16_t,
        );
    }
    src_id.reported(error)
}
