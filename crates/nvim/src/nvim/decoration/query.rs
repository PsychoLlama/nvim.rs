//! Asking the marktree about a row without drawing it.
//!
//! The questions the layout code needs answered before it knows how tall a
//! line is: does it have virtual text ([`decor_find_virttext`]), is it
//! concealed ([`decor_conceal_line`]), and how many virtual lines does it
//! carry ([`decor_virt_lines`]) — plus [`next_virt_text_chunk`], which every
//! drawer of virtual text walks its chunks with.
//!
//! `plines.rs` and `move.rs` call into here on paths that run per line of a
//! redraw, so each entry point starts with a `buf_meta_total` test: a buffer
//! with no marks of the kind in question does not touch the marktree at all.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{mt_decor_virt, ns_in_win};
use crate::src::nvim::buffer::buf_meta_total;
use crate::src::nvim::decoration::{kMTMetaConcealLines, kMTMetaLines, kVTIsLines, kVTLinesAbove};
use crate::src::nvim::decoration_provider::decor_providers_invoke_conceal_line;
use crate::src::nvim::drawscreen::conceal_cursor_line;
use crate::src::nvim::fold::{hasAnyFolding, hasFolding};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight::hl_combine_attr;
use crate::src::nvim::highlight_group::syn_id2attr;
use crate::src::nvim::main::curwin;
use crate::src::nvim::marktree::key::{kMTFilterSelect, mt_conceal_lines, mt_invalid};
use crate::src::nvim::marktree::{
    marktree_itr_current, marktree_itr_get, marktree_itr_get_filter, marktree_itr_get_overlap,
    marktree_itr_next, marktree_itr_next_filter, marktree_itr_step_out_filter,
    marktree_itr_step_overlap,
};
use crate::src::nvim::memory::xrealloc;
use crate::src::nvim::os::libc::memcpy;
use crate::src::nvim::types::{
    DecorVirtText, MTPair, MarkTree, MarkTreeIter, MetaFilter, OptInt, VirtLines, VirtText, buf_T,
    linenr_T, size_t, uint32_t, uint64_t, virt_line, win_T,
};
use ::core::ffi::{c_char, c_int};
use ::core::{mem, ptr};

/// Marktree filters, indexed by `MetaIndex`: `kMTMetaLines` is 1 and
/// `kMTMetaConcealLines` 4.
static LINES_FILTER: GlobalCell<[uint32_t; 5]> = GlobalCell::new([0, kMTFilterSelect, 0, 0, 0]);
static CONCEAL_FILTER: GlobalCell<[uint32_t; 5]> = GlobalCell::new([0, 0, 0, 0, kMTFilterSelect]);

fn lines_filter() -> MetaFilter {
    LINES_FILTER.ptr().cast::<uint32_t>()
}

fn conceal_filter() -> MetaFilter {
    CONCEAL_FILTER.ptr().cast::<uint32_t>()
}

/// A zeroed marktree iterator, which is the state a walk starts from.
fn new_iter() -> MarkTreeIter {
    // SAFETY: `MarkTreeIter` is plain data and all-zero is its initial state.
    unsafe { mem::zeroed() }
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
    // SAFETY: the caller's virtual text and out-parameters.
    unsafe {
        let mut text: *mut c_char = ptr::null_mut();
        while text.is_null() && *pos < vt.size {
            let chunk = *vt.items.add(*pos);
            text = chunk.text;
            if chunk.hl_id >= 0 {
                *attr = (*attr).max(0);
                if chunk.hl_id > 0 {
                    *attr = hl_combine_attr(*attr, syn_id2attr(chunk.hl_id));
                }
            }
            *pos += 1;
        }
        text
    }
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
    // SAFETY: the caller's buffer and the editor's own marktree.
    unsafe {
        let tree: *mut MarkTree = (&raw mut (*buf).b_marktree).cast();
        let mut itr = new_iter();
        marktree_itr_get(tree, row, 0, &raw mut itr);
        loop {
            let mark = marktree_itr_current(&raw mut itr);
            if mark.pos.row < 0 || mark.pos.row > row {
                return ptr::null_mut();
            }
            if !mt_invalid(mark) {
                let mut decor = mt_decor_virt(mark);
                while !decor.is_null() && (*decor).flags as c_int & kVTIsLines as c_int != 0 {
                    decor = (*decor).next;
                }
                if (ns_id == 0 || ns_id == uint64_t::from(mark.ns)) && !decor.is_null() {
                    return decor;
                }
            }
            marktree_itr_next(tree, &raw mut itr);
        }
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
    // SAFETY: the caller's window and the editor's own marktree.
    unsafe {
        if row < 0
            || (*wp).w_onebuf_opt.wo_cole < 2 as OptInt
            || (!check_cursor
                && wp == curwin.get()
                && row as linenr_T + 1 == (*wp).w_cursor.lnum
                && !conceal_cursor_line(wp))
        {
            return false;
        }

        // No need to scan the marktree if there are no conceal_line marks.
        if buf_meta_total((*wp).w_buffer, kMTMetaConcealLines) == 0 {
            return decor_providers_invoke_conceal_line(wp, row);
        }

        let tree: *mut MarkTree = (&raw mut (*(*wp).w_buffer).b_marktree).cast();
        let mut itr = new_iter();
        let mut pair: MTPair = mem::zeroed();

        marktree_itr_get_overlap(tree, row, 0, &raw mut itr);
        while marktree_itr_step_overlap(tree, &raw mut itr, &raw mut pair) {
            if mt_conceal_lines(pair.start) && ns_in_win(pair.start.ns, wp) {
                return true;
            }
        }

        marktree_itr_step_out_filter(tree, &raw mut itr, conceal_filter());
        while !itr.x.is_null() {
            let mark = marktree_itr_current(&raw mut itr);
            if mark.pos.row > row {
                break;
            }
            if mt_conceal_lines(mark) && ns_in_win(mark.ns, wp) {
                return true;
            }
            marktree_itr_next_filter(tree, &raw mut itr, row + 1, 0, conceal_filter());
        }

        decor_providers_invoke_conceal_line(wp, row)
    }
}

/// Whether `wp` may have folded or concealed lines at all — the cheap test
/// that lets the layout code skip the per-row questions above.
///
/// # Safety
/// `wp` must point to a live window.
pub unsafe fn win_lines_concealed(wp: *mut win_T) -> bool {
    // SAFETY: the caller's window.
    unsafe { hasAnyFolding(wp) != 0 || (*wp).w_onebuf_opt.wo_cole >= 2 as OptInt }
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
    // SAFETY: the caller's window and out-parameters.
    unsafe {
        let buf = (*wp).w_buffer;
        // Only pay for what you use: in a buffer with no virt_lines the
        // layout code does not reach the marktree at all.
        if buf_meta_total(buf, kMTMetaLines) == 0 {
            return 0;
        }

        let tree: *mut MarkTree = (&raw mut (*buf).b_marktree).cast();
        let mut itr = new_iter();
        if !marktree_itr_get_filter(
            tree,
            (start_row - 1).max(0),
            0,
            end_row,
            0,
            lines_filter(),
            &raw mut itr,
        ) {
            return 0;
        }
        debug_assert!(start_row >= 0);

        let mut virt_lines = 0;
        loop {
            let mark = marktree_itr_current(&raw mut itr);
            if !mt_invalid(mark) && ns_in_win(mark.ns, wp) {
                let mut vt = mt_decor_virt(mark);
                while !vt.is_null() {
                    if (*vt).flags as c_int & kVTIsLines as c_int != 0 {
                        let above = (*vt).flags as c_int & kVTLinesAbove as c_int != 0;
                        let mrow = mark.pos.row;
                        let draw_row = mrow + c_int::from(!above);
                        // The fold and conceal tests stay inside the `&&`
                        // chain: `decor_conceal_line` runs the providers'
                        // Lua, so asking it for a row that is out of range
                        // would be a new side effect, not just extra work.
                        if draw_row >= start_row
                            && draw_row < end_row
                            && (!apply_folds
                                || !(hasFolding(
                                    wp,
                                    mrow as linenr_T + 1,
                                    ptr::null_mut(),
                                    ptr::null_mut(),
                                ) || decor_conceal_line(wp, mrow, false)))
                        {
                            let block = (*vt).data.virt_lines;
                            virt_lines += block.size as c_int;
                            if !lines.is_null() {
                                append_virt_lines(&mut *lines, block);
                            }
                            if !num_below.is_null() && !above {
                                *num_below += block.size as c_int;
                            }
                        }
                    }
                    vt = (*vt).next;
                }
            }

            if !marktree_itr_next_filter(tree, &raw mut itr, end_row, 0, lines_filter()) {
                return virt_lines;
            }
        }
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
    // SAFETY: `dst` is the caller's growable vector and `src` its source.
    unsafe {
        let wanted = dst.size + src.size;
        if dst.capacity < wanted {
            dst.capacity = wanted.next_power_of_two();
            dst.items =
                xrealloc(dst.items.cast(), mem::size_of::<virt_line>() * dst.capacity).cast();
        }
        assert!(!dst.items.is_null());
        memcpy(
            dst.items.add(dst.size).cast(),
            src.items.cast(),
            mem::size_of::<virt_line>() * src.size,
        );
        dst.size = wanted;
    }
}
