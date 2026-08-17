//! Asking the marktree about a row without drawing it.
//!
//! The questions the layout code needs answered before it knows how tall a
//! line is: does it have virtual text ([`decor_find_virttext`]), is it
//! concealed ([`decor_conceal_line`]), and how many virtual lines does it
//! carry ([`decor_virt_lines`]) — plus [`next_virt_text_chunk`], which every
//! drawer of virtual text walks its chunks with.
//!
//! `plines.rs` and `move.rs` call into here on paths that run per line of a
//! redraw, so each entry point starts with a [`Buf::meta_total`] test: a
//! buffer with no marks of the kind in question does not touch the marktree
//! at all.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{mark_virt_chain, ns_in_win};
use crate::decoration::{kMTMetaConcealLines, kMTMetaLines};
use crate::decoration_provider::decor_providers_invoke_conceal_line;
use crate::drawscreen::conceal_cursor_line;
use crate::highlight::hl_combine_attr;
use crate::highlight_group::syn_id2attr;
use crate::marktree::cursor::Cursor;
use crate::marktree::key::{kMTFilterSelect, mt_conceal_lines, mt_invalid};
use crate::marktree::meta::MetaCount;
use crate::memory::xrealloc;
use crate::os::libc::memcpy;
use crate::types::{
    DecorVirtText, MarkTreeIter, OptInt, VirtLines, VirtText, buf_T, linenr_T, size_t, uint64_t,
    virt_line, win_T,
};
use crate::winlayer::{Buf, Win};
use ::core::ffi::{c_char, c_int};
use ::core::{mem, ptr, slice};

/// Marktree filters, indexed by `MetaIndex`: `kMTMetaLines` is 1 and
/// `kMTMetaConcealLines` 4.
static LINES_FILTER: MetaCount = [0, kMTFilterSelect, 0, 0, 0];
static CONCEAL_FILTER: MetaCount = [0, 0, 0, 0, kMTFilterSelect];

impl Win {
    /// Whether `'concealcursor'` says the cursor line conceals too.
    fn conceals_cursor_line(self) -> bool {
        // SAFETY: a live window.
        unsafe { conceal_cursor_line(self.raw()) }
    }

    /// Asks every decoration provider to place `row`'s conceal marks, and
    /// answers whether one of them says the row is concealed.
    ///
    /// Runs Lua, which can place and delete marks.
    fn providers_conceal_line(self, row: c_int) -> bool {
        // SAFETY: a live window.
        unsafe { decor_providers_invoke_conceal_line(self.raw(), row) }
    }
}

/// The text of the next chunk of `vt`, or null when there are no more.
///
/// `pos` is the walk's position and is advanced past the chunk answered;
/// `attr` accumulates the highlights of every chunk passed, including the
/// empty ones that are skipped over. A chunk with `hl_id` of 0 still lifts
/// `attr` from -1 ("no highlight given") to 0, which is how a caller tells
/// "explicitly unhighlighted" from "nothing said".
///
/// # Safety
/// `vt` must be live; `pos` and `attr` must be writable. They are raw
/// pointers rather than references because two of the callers pass fields of
/// a struct they keep reading through a raw pointer across the call.
pub unsafe fn next_virt_text_chunk(
    vt: VirtText,
    pos: *mut size_t,
    attr: *mut c_int,
) -> *mut c_char {
    if vt.size == 0 {
        return ptr::null_mut();
    }
    // SAFETY: a non-empty kvec's `items` is its own allocation, `size` long.
    let chunks = unsafe { slice::from_raw_parts(vt.items, vt.size) };
    // SAFETY: the caller's out-parameters.
    let (pos, attr) = unsafe { (&mut *pos, &mut *attr) };

    let mut text: *mut c_char = ptr::null_mut();
    while text.is_null() && *pos < chunks.len() {
        let chunk = chunks[*pos];
        text = chunk.text;
        if chunk.hl_id >= 0 {
            *attr = (*attr).max(0);
            if chunk.hl_id > 0 {
                // SAFETY: the highlight tables are the editor's own.
                *attr = unsafe { hl_combine_attr(*attr, syn_id2attr(chunk.hl_id)) };
            }
        }
        *pos += 1;
    }
    text
}

/// The first virtual *text* on `row` — skipping virtual *lines*, which are a
/// different thing on the same chain — from namespace `ns_id`, or from any
/// namespace when that is 0. Null if the row has none.
///
/// # Safety
/// `buf` must point to a live buffer.
pub unsafe fn decor_find_virttext(
    buf: *mut buf_T,
    row: c_int,
    ns_id: uint64_t,
) -> *mut DecorVirtText {
    // SAFETY: the caller's buffer.
    let buf = unsafe { Buf::new(buf) };
    let mut itr = MarkTreeIter::default();
    let mut walk = Cursor::in_buffer(buf, &mut itr);
    walk.seek(row, 0);
    loop {
        let mark = walk.current();
        if mark.pos.row < 0 || mark.pos.row > row {
            return ptr::null_mut();
        }
        if !mt_invalid(mark) && (ns_id == 0 || ns_id == uint64_t::from(mark.ns)) {
            if let Some(vt) = mark_virt_chain(mark).find(|vt| !vt.is_lines()) {
                return vt.raw();
            }
        }
        walk.next();
    }
}

/// Whether `row` of `wp` is hidden entirely by a `conceal_lines` decoration.
///
/// `check_cursor` asks for the answer the row would get if it were not the
/// cursor line: the cursor line is normally exempt (unless `'concealcursor'`
/// says otherwise), but some callers still need to know.
///
/// Providers are asked last, and only when no mark already answered — their
/// `_on_conceal_line` callback may place marks, which is what the answer of
/// [`decor_providers_invoke_conceal_line`] reports.
///
/// # Safety
/// `wp` must point to a live window; runs Lua through the providers.
pub unsafe fn decor_conceal_line(wp: *mut win_T, row: c_int, check_cursor: bool) -> bool {
    // SAFETY: the caller's window.
    let wp = unsafe { Win::new(wp) };
    if row < 0
        || wp.w_onebuf_opt.wo_cole < 2 as OptInt
        || (!check_cursor
            && wp.is_current()
            && row as linenr_T + 1 == wp.w_cursor.lnum
            && !wp.conceals_cursor_line())
    {
        return false;
    }

    // No need to scan the marktree if there are no conceal_line marks.
    let buf = wp.buffer();
    if buf.meta_total(kMTMetaConcealLines) == 0 {
        return wp.providers_conceal_line(row);
    }

    let mut itr = MarkTreeIter::default();
    let mut walk = Cursor::in_buffer(buf, &mut itr);

    walk.seek_overlap(row, 0);
    while let Some(pair) = walk.step_overlap() {
        if mt_conceal_lines(pair.start) && ns_in_win(pair.start.ns, wp) {
            return true;
        }
    }

    walk.step_out_filter(&CONCEAL_FILTER);
    while !walk.is_empty() {
        let mark = walk.current();
        if mark.pos.row > row {
            break;
        }
        if mt_conceal_lines(mark) && ns_in_win(mark.ns, wp) {
            return true;
        }
        walk.next_filter(row + 1, 0, &CONCEAL_FILTER);
    }

    wp.providers_conceal_line(row)
}

/// Whether `wp` may have folded or concealed lines at all — the cheap test
/// that lets the layout code skip the per-row questions above.
///
/// # Safety
/// `wp` must point to a live window.
pub unsafe fn win_lines_concealed(wp: *mut win_T) -> bool {
    // SAFETY: the caller's window.
    let wp = unsafe { Win::new(wp) };
    wp.has_any_folding() || wp.w_onebuf_opt.wo_cole >= 2 as OptInt
}

/// How many virtual lines fall in the window rows `start_row..end_row`.
///
/// A virtual-lines block is drawn above or below the line its mark sits on,
/// so a mark one row before `start_row` can still contribute. `lines`, when
/// given, receives copies of the `virt_line`s themselves; `num_below` counts
/// only the ones drawn *below* their mark; `apply_folds` drops the blocks
/// whose own line is folded away or concealed.
///
/// # Safety
/// `wp` must be live; `lines` must be null or a live `VirtLines` this may
/// grow; `num_below` null or writable.
pub unsafe fn decor_virt_lines(
    wp: *mut win_T,
    start_row: c_int,
    end_row: c_int,
    num_below: *mut c_int,
    lines: *mut VirtLines,
    apply_folds: bool,
) -> c_int {
    // SAFETY: the caller's window and out-parameters, null or writable.
    let (wp, mut lines, mut num_below) =
        unsafe { (Win::new(wp), lines.as_mut(), num_below.as_mut()) };
    let buf = wp.buffer();
    // Only pay for what you use: in a buffer with no virt_lines the layout
    // code does not reach the marktree at all.
    if buf.meta_total(kMTMetaLines) == 0 {
        return 0;
    }

    let mut itr = MarkTreeIter::default();
    let mut walk = Cursor::in_buffer(buf, &mut itr);
    if !walk.seek_filter((start_row - 1).max(0), 0, end_row, 0, &LINES_FILTER) {
        return 0;
    }
    debug_assert!(start_row >= 0);

    let mut virt_lines = 0;
    loop {
        let mark = walk.current();
        if !mt_invalid(mark) && ns_in_win(mark.ns, wp) {
            for vt in mark_virt_chain(mark).filter(|vt| vt.is_lines()) {
                let above = vt.lines_above();
                let mrow = mark.pos.row;
                let draw_row = mrow + c_int::from(!above);
                // The fold and conceal tests stay inside the `&&` chain:
                // `decor_conceal_line` runs the providers' Lua, so asking it
                // for a row that is out of range would be a new side effect,
                // not just extra work.
                if draw_row >= start_row
                    && draw_row < end_row
                    && (!apply_folds
                        || !(wp.fold_span(mrow as linenr_T + 1).0 || wp.conceal_line(mrow, false)))
                {
                    let block = vt.lines();
                    virt_lines += block.size as c_int;
                    if let Some(lines) = lines.as_deref_mut() {
                        // SAFETY: a live growable vector, and `block` is the
                        // decoration's own live one.
                        unsafe { append_virt_lines(lines, block) };
                    }
                    if let (false, Some(num_below)) = (above, num_below.as_deref_mut()) {
                        *num_below += block.size as c_int;
                    }
                }
            }
        }

        if !walk.next_filter(end_row, 0, &LINES_FILTER) {
            return virt_lines;
        }
    }
}

impl Win {
    /// [`decor_conceal_line`] for a window already promised live.
    fn conceal_line(self, row: c_int, check_cursor: bool) -> bool {
        // SAFETY: a live window. Runs Lua through the providers.
        unsafe { decor_conceal_line(self.raw(), row, check_cursor) }
    }
}

/// Appends `src`'s entries to `dst`, growing it — `kv_splice`.
///
/// The entries are copied, not cloned: the `VirtText` inside each one is
/// still owned by the decoration it came from, which is why the caller of
/// [`decor_virt_lines`] must not free what it collects.
///
/// # Safety
/// `dst` must be a live, growable `VirtLines`; `src`'s entries must be live.
unsafe fn append_virt_lines(dst: &mut VirtLines, src: VirtLines) {
    if src.size == 0 {
        return;
    }
    let wanted = dst.size + src.size;
    if dst.capacity < wanted {
        dst.capacity = wanted.next_power_of_two();
        // SAFETY: `dst` owns `items`, which `xrealloc` grows in place.
        dst.items =
            unsafe { xrealloc(dst.items.cast(), mem::size_of::<virt_line>() * dst.capacity) }
                .cast();
    }
    assert!(!dst.items.is_null());
    let bytes = mem::size_of::<virt_line>() * src.size;
    let end = dst.items.wrapping_add(dst.size).cast();
    // SAFETY: `dst` now has room for `src`'s entries, and the two vectors
    // never overlap — `src` belongs to a decoration, `dst` to the caller.
    unsafe { memcpy(end, src.items.cast(), bytes) };
    dst.size = wanted;
}
