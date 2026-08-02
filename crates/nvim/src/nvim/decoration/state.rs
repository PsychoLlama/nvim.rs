//! `DecorState`: the decorations in play while a window is drawn.
//!
//! The drawing code walks a window top to bottom and left to right, and this
//! is the machinery that keeps up with it. Marks are pulled out of the
//! marktree as their row is reached and split into `DecorRange`s — one per
//! highlight, virt text or virt-lines block.
//!
//! # The two lists
//!
//! Ranges live in `slots`, a slab that is never compacted: a freed slot goes
//! on a freelist threaded through the union's `next_free_i`, so an index
//! stays valid for as long as the drawing code holds it. `ranges_i` indexes
//! into that slab twice over:
//!
//! * `ranges_i[..current_end]` — the ranges that have *started* before the
//!   current position, sorted by `(priority_internal, ordering)` ascending.
//!   That is the order highlights combine in and the order virtual texts are
//!   drawn in.
//! * `ranges_i[future_begin..]` — the ranges that start *later*, sorted by
//!   starting position, so that the next one to promote is always the first.
//!
//! The gap between the two grows as ranges are promoted and end; it is
//! squeezed out once per line by [`decor_state_pack`].
//!
//! [`decor_redraw_col_impl`] is the one that moves both: it pulls in the
//! marks the column has reached, promotes the ranges that have started,
//! drops the ones that have ended, and answers the combined attribute. It
//! runs per column of every drawn line, so it works through raw pointers
//! into the two vectors rather than re-indexing them.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{
    DECOR_ID_INVALID, decor_item, decor_sh_from_inline, kSHConceal, kSHHlEol, kSHIsSign,
    kSHSpellOff, kSHSpellOn, kSHUIWatched, kSHUIWatchedOverlay, kVPosEndOfLine, kVPosInline,
    kVPosOverlay, kVTHide, kVTIsLines, ns_in_win,
};
use crate::src::nvim::decoration::clear_virttext;
use crate::src::nvim::highlight::{hl_add_url, hl_combine_attr};
use crate::src::nvim::highlight_group::syn_id2attr;
use crate::src::nvim::main::decor_state;
use crate::src::nvim::marktree::key::{mt_decor, mt_decor_any, mt_end, mt_invalid};
use crate::src::nvim::marktree::{
    marktree_get_altpos, marktree_itr_current, marktree_itr_get, marktree_itr_get_overlap,
    marktree_itr_next, marktree_itr_step_overlap,
};
use crate::src::nvim::memory::xfree;
use crate::src::nvim::types::{
    DecorInline, DecorPriority, DecorPriorityInternal, DecorRange, DecorRange_data,
    DecorRange_data_ui, DecorRangeKind, DecorRangeSlot, DecorSignHighlight, DecorState,
    DecorVirtText, MTPair, MTPos, MarkTree, MarkTreeIter, TriState, VirtTextPos, buf_T, uint32_t,
    win_T,
};
use ::core::ffi::c_int;
use ::core::{mem, ptr};

/// `DecorRangeKind`: what a `DecorRange`'s `data` union holds.
const kDecorKindHighlight: DecorRangeKind = 0;
const kDecorKindVirtText: DecorRangeKind = 2;
const kDecorKindVirtLines: DecorRangeKind = 3;
const kDecorKindUIWatched: DecorRangeKind = 4;

/// `TriState`, the spelling state a cell inherits from its decorations.
const kTrue: TriState = 1;
const kFalse: TriState = 0;
const kNone: TriState = -1;

const MAXCOL: c_int = c_int::MAX;

/// `draw_col` sentinels, in the order the drawing code goes through them.
/// A new virtual-position range starts at [`DRAW_COL_NEW`]; the column loop
/// resolves it either to a real column, to [`DRAW_COL_PENDING`] ("a position
/// yet to be assigned"), to [`DRAW_COL_LATER`] ("decide when the line's end
/// is known") or to [`DRAW_COL_NEVER`].
const DRAW_COL_NEW: c_int = -10;
const DRAW_COL_PENDING: c_int = -3;
const DRAW_COL_LATER: c_int = -1;
const DRAW_COL_NEVER: c_int = c_int::MIN;

// ---------------------------------------------------------------------------
// The lists
// ---------------------------------------------------------------------------

/// How many entries `ranges_i` has, active and future together.
///
/// # Safety
/// `state` must point to a live `DecorState`.
pub unsafe fn decor_range_count(state: *const DecorState) -> c_int {
    // SAFETY: the caller's state.
    unsafe { (*state).ranges_i.len() as c_int }
}

/// The range `ranges_i[i]` names.
///
/// A pointer, not a borrow: the drawing code holds several of these at once
/// and writes `draw_col` through them while reading the rest of the state.
///
/// # Safety
/// `state` must be live and `i` a valid index into `ranges_i`.
pub unsafe fn decor_range_at(state: *mut DecorState, i: c_int) -> *mut DecorRange {
    // SAFETY: the caller's state and index. Raw rather than `Vec` indexing:
    // the callers hold several of these at once, and an `&mut` through the
    // vector would invalidate the others.
    unsafe {
        assert!((i as usize) < (*state).ranges_i.len());
        let slot = *(*state).ranges_i.as_ptr().add(i as usize) as usize;
        assert!(slot < (*state).slots.len());
        &raw mut (*(*state).slots.as_mut_ptr().add(slot)).range
    }
}

/// Called whenever a public API function adds or deletes marks, in case that
/// happened in a callback the drawing code is inside: the marktree iterator
/// `state` is holding cannot be trusted across a structural change.
///
/// # Safety
/// `buf` must be live or null.
pub unsafe fn decor_state_invalidate(buf: *mut buf_T) {
    decor_state.with_mut(|state| {
        // SAFETY: `state.win` is a live window while a redraw is running.
        if !state.win.is_null() && unsafe { (*state.win).w_buffer } == buf {
            state.itr_valid = false;
        }
    });
}

/// Releases the two vectors. The ranges themselves are not owned here — see
/// [`decor_redraw_reset`], which is what frees the ephemeral ones.
///
/// # Safety
/// `state` must point to a live `DecorState`.
pub unsafe fn decor_state_free(state: *mut DecorState) {
    // SAFETY: the caller's state.
    unsafe {
        (*state).slots = Vec::new();
        (*state).ranges_i = Vec::new();
    }
}

/// Starts a fresh window: empties both lists, freeing the ephemeral virtual
/// texts a decoration provider left behind.
///
/// Answers whether the buffer has any marks at all, which is the caller's cue
/// to bother with the rest of the machinery.
///
/// # Safety
/// `wp` and `state` must be live.
pub unsafe fn decor_redraw_reset(wp: *mut win_T, state: *mut DecorState) -> bool {
    // SAFETY: the caller's window and state.
    unsafe {
        (*state).row = -1;
        (*state).win = wp;

        for i in list_spans(state) {
            let r = decor_range_at(state, i);
            if (*r).owned && (*r).kind == kDecorKindVirtText {
                clear_virttext(&raw mut (*(*r).data.vt).data.virt_text);
                xfree((*r).data.vt.cast());
            }
        }

        (*state).slots.clear();
        (*state).ranges_i.clear();
        (*state).free_slot_i = -1;
        (*state).current_end = 0;
        (*state).future_begin = 0;
        (*state).new_range_ordering = 0;

        (*(*wp).w_buffer).b_marktree[0].n_keys != 0
    }
}

/// The indices of every live entry of `ranges_i` — the active list and the
/// future list, skipping the gap between them.
///
/// # Safety
/// `state` must be live.
unsafe fn list_spans(state: *const DecorState) -> impl Iterator<Item = c_int> {
    // SAFETY: the caller's state.
    let (current_end, future_begin, count) = unsafe {
        (
            (*state).current_end,
            (*state).future_begin,
            decor_range_count(state),
        )
    };
    (0..current_end).chain(future_begin..count)
}

/// Whether `decor` occupies a position of its own rather than colouring the
/// text under it — virtual text, or a mark a UI wants told about.
///
/// # Safety
/// `decor` must be live.
pub unsafe fn decor_virt_pos(decor: *const DecorRange) -> bool {
    // SAFETY: the caller's range.
    unsafe { (*decor).kind == kDecorKindVirtText || (*decor).kind == kDecorKindUIWatched }
}

/// Where a virtual-position range wants to be drawn.
///
/// # Safety
/// `decor` must be live.
pub unsafe fn decor_virt_pos_kind(decor: *const DecorRange) -> VirtTextPos {
    // SAFETY: the caller's range.
    unsafe {
        match (*decor).kind {
            kDecorKindVirtText => (*(*decor).data.vt).pos,
            kDecorKindUIWatched => (*decor).data.ui.pos,
            // Not used; answer whatever.
            _ => kVPosEndOfLine,
        }
    }
}

/// Seeds the state at the top of a window with the marks that start *above*
/// `top_row` and reach into it.
///
/// # Safety
/// `wp` and `state` must be live.
pub unsafe fn decor_redraw_start(wp: *mut win_T, top_row: c_int, state: *mut DecorState) -> bool {
    // SAFETY: the caller's window and state, and the editor's marktree.
    unsafe {
        let tree: *mut MarkTree = (&raw mut (*(*wp).w_buffer).b_marktree).cast();
        (*state).top_row = top_row;
        (*state).itr_valid = true;

        let itr: *mut MarkTreeIter = (&raw mut (*state).itr).cast();
        if !marktree_itr_get_overlap(tree, top_row, 0, itr) {
            return false;
        }

        let mut pair: MTPair = mem::zeroed();
        while marktree_itr_step_overlap(tree, itr, &raw mut pair) {
            let m = pair.start;
            if mt_invalid(m) || !mt_decor_any(m) {
                continue;
            }
            decor_range_add_from_inline(
                state,
                m.pos.row,
                m.pos.col,
                pair.end_pos.row,
                pair.end_pos.col,
                mt_decor(m),
                false,
                m.ns,
                m.id,
            );
        }

        true // TODO(bfredl): check if available in the region
    }
}

/// Squeezes the gap between the active list and the future list out of
/// `ranges_i`, so that the future list does not walk off to infinity as the
/// window is drawn.
///
/// # Safety
/// `state` must be live.
pub(crate) unsafe fn decor_state_pack(state: *mut DecorState) {
    // SAFETY: the caller's state.
    unsafe {
        let count = (*state).ranges_i.len();
        let cur_end = (*state).current_end as usize;
        let fut_beg = (*state).future_begin as usize;

        if fut_beg == count {
            (*state).ranges_i.truncate(cur_end);
        } else if fut_beg != cur_end {
            (*state).ranges_i.copy_within(fut_beg..count, cur_end);
            (*state).ranges_i.truncate(cur_end + (count - fut_beg));
        }
        (*state).future_begin = (*state).current_end;
    }
}

/// Moves the state on to `row`.
///
/// # Safety
/// `wp` and `state` must be live.
pub unsafe fn decor_redraw_line(wp: *mut win_T, row: c_int, state: *mut DecorState) {
    // SAFETY: the caller's window and state.
    unsafe {
        decor_state_pack(state);

        if (*state).row == -1 {
            decor_redraw_start(wp, row, state);
        } else if !(*state).itr_valid {
            let tree: *mut MarkTree = (&raw mut (*(*wp).w_buffer).b_marktree).cast();
            marktree_itr_get(tree, row, 0, (&raw mut (*state).itr).cast());
            (*state).itr_valid = true;
        }

        (*state).row = row;
        (*state).col_last = -1;
        (*state).eol_col = -1;
    }
}

/// Whether there are (likely) more decorations on `row`.
///
/// # Safety
/// `state` must be live.
pub unsafe fn decor_has_more_decorations(state: *mut DecorState, row: c_int) -> bool {
    // SAFETY: the caller's state.
    unsafe {
        if (*state).current_end != 0 || (*state).future_begin != decor_range_count(state) {
            return true;
        }
        let k = marktree_itr_current((&raw mut (*state).itr).cast());
        k.pos.row >= 0 && k.pos.row <= row
    }
}

// ---------------------------------------------------------------------------
// Adding ranges
// ---------------------------------------------------------------------------

/// Splits one mark's decoration into ranges and adds them all: a mark can
/// carry a chain of virtual texts and a chain of sign/highlight items.
///
/// # Safety
/// `state` must be live and `decor` must belong to a live mark.
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn decor_range_add_from_inline(
    state: *mut DecorState,
    start_row: c_int,
    start_col: c_int,
    end_row: c_int,
    end_col: c_int,
    decor: DecorInline,
    owned: bool,
    ns: uint32_t,
    mark_id: uint32_t,
) {
    // SAFETY: the caller's state and decoration.
    unsafe {
        if !decor.ext {
            let mut sh = decor_sh_from_inline(decor.data.hl);
            decor_range_add_sh(
                state, start_row, start_col, end_row, end_col, &mut sh, owned, ns, mark_id, 0,
            );
            return;
        }

        let mut vt = decor.data.ext.vt;
        while !vt.is_null() {
            decor_range_add_virt(state, start_row, start_col, end_row, end_col, vt, owned);
            vt = (*vt).next;
        }

        let mut idx: uint32_t = decor.data.ext.sh_idx;
        while idx != DECOR_ID_INVALID {
            let sh = decor_item(idx);
            decor_range_add_sh(
                state, start_row, start_col, end_row, end_col, sh, owned, ns, mark_id, 0,
            );
            idx = (*sh).next;
        }
    }
}

/// Files `range` in a slot and puts its index in the future list, which stays
/// sorted by starting position.
///
/// # Safety
/// `state` must be live.
unsafe fn decor_range_insert(state: *mut DecorState, range: &mut DecorRange) {
    // SAFETY: the caller's state.
    unsafe {
        range.ordering = (*state).new_range_ordering;
        (*state).new_range_ordering += 1;

        // Reuse a freed slot if there is one; the freelist is threaded
        // through the slots themselves.
        let index = if (*state).free_slot_i >= 0 {
            let index = (*state).free_slot_i as usize;
            let slot = (*state).slots.as_mut_ptr().add(index);
            (*state).free_slot_i = (*slot).next_free_i;
            (*slot).range = *range;
            index
        } else {
            (*state).slots.push(DecorRangeSlot { range: *range });
            (*state).slots.len() - 1
        };

        // Binary search for the first entry that starts after this one — but
        // stopping early on an exact position match, which puts equal
        // positions in insertion order.
        let count = (*state).ranges_i.len();
        let mut begin = (*state).future_begin as usize;
        let mut end = count;
        while begin < end {
            let mid = begin + ((end - begin) >> 1);
            let mr = decor_range_at(state, mid as c_int);
            let (mrow, mcol) = ((*mr).start_row, (*mr).start_col);
            if mrow < range.start_row || (mrow == range.start_row && mcol <= range.start_col) {
                begin = mid + 1;
                if mrow == range.start_row && mcol == range.start_col {
                    break;
                }
            } else {
                end = mid;
            }
        }

        (*state).ranges_i.insert(begin, index as c_int);
    }
}

/// Adds the range a virtual text or virtual-lines block occupies.
///
/// # Safety
/// `state` and `vt` must be live; `vt` is borrowed for as long as the range
/// is, unless `owned` says the range took it over.
pub unsafe fn decor_range_add_virt(
    state: *mut DecorState,
    start_row: c_int,
    start_col: c_int,
    end_row: c_int,
    end_col: c_int,
    vt: *mut DecorVirtText,
    owned: bool,
) {
    // SAFETY: the caller's state and virtual text.
    unsafe {
        let is_lines = (*vt).flags as c_int & kVTIsLines as c_int != 0;
        let mut range = DecorRange {
            start_row,
            start_col,
            end_row,
            end_col,
            ordering: 0,
            // Virtual texts carry no subpriority, so the low 16 bits are
            // zero; `decor_range_add_sh` fills them in. Both sides must use
            // the same shift or the two kinds no longer interleave.
            priority_internal: DecorPriorityInternal::from((*vt).priority) << 16,
            owned,
            kind: if is_lines {
                kDecorKindVirtLines
            } else {
                kDecorKindVirtText
            },
            data: DecorRange_data { vt },
            attr_id: 0,
            draw_col: DRAW_COL_NEW,
        };
        decor_range_insert(state, &mut range);
    }
}

/// Adds the range a highlight (or a `ui_watched` mark) occupies. A sign is
/// not a range — it is drawn in the sign column, not over the text — so it is
/// dropped here.
///
/// One `sh` can produce two ranges: a highlight and a UI-watched position.
///
/// # Safety
/// `state` and `sh` must be live.
#[allow(clippy::too_many_arguments)]
pub unsafe fn decor_range_add_sh(
    state: *mut DecorState,
    start_row: c_int,
    start_col: c_int,
    end_row: c_int,
    end_col: c_int,
    sh: *mut DecorSignHighlight,
    owned: bool,
    ns: uint32_t,
    mark_id: uint32_t,
    subpriority: DecorPriority,
) {
    // SAFETY: the caller's state and sign/highlight item.
    unsafe {
        let flags = (*sh).flags as c_int;
        if flags & kSHIsSign as c_int != 0 {
            return;
        }

        let mut range = DecorRange {
            start_row,
            start_col,
            end_row,
            end_col,
            ordering: 0,
            priority_internal: (DecorPriorityInternal::from((*sh).priority) << 16)
                + DecorPriorityInternal::from(subpriority),
            owned,
            kind: kDecorKindHighlight,
            data: DecorRange_data { sh: *sh },
            attr_id: 0,
            draw_col: DRAW_COL_NEW,
        };

        if (*sh).hl_id != 0
            || !(*sh).url.is_null()
            || flags & (kSHConceal | kSHSpellOn | kSHSpellOff) as c_int != 0
        {
            if (*sh).hl_id != 0 {
                range.attr_id = syn_id2attr((*sh).hl_id);
            }
            decor_range_insert(state, &mut range);
        }

        if flags & kSHUIWatched as c_int != 0 {
            range.kind = kDecorKindUIWatched;
            range.data.ui = DecorRange_data_ui {
                ns_id: ns,
                mark_id,
                pos: if flags & kSHUIWatchedOverlay as c_int != 0 {
                    kVPosOverlay
                } else {
                    kVPosEndOfLine
                },
            };
            decor_range_insert(state, &mut range);
        }
    }
}

// ---------------------------------------------------------------------------
// Drawing positions
// ---------------------------------------------------------------------------

/// Decides where a newly-started virtual-position range goes.
///
/// # Safety
/// `item` must be live.
pub unsafe fn decor_init_draw_col(win_col: c_int, hidden: bool, item: *mut DecorRange) {
    // SAFETY: the caller's range.
    unsafe {
        let vt = if (*item).kind == kDecorKindVirtText {
            (*item).data.vt
        } else {
            ptr::null_mut()
        };
        let pos = decor_virt_pos_kind(item);
        (*item).draw_col = if win_col < 0 && pos != kVPosInline {
            // A negative `win_col` is itself a sentinel the caller passes in.
            win_col
        } else if pos == kVPosOverlay {
            let hide = !vt.is_null() && (*vt).flags as c_int & kVTHide as c_int != 0 && hidden;
            if hide { DRAW_COL_NEVER } else { win_col }
        } else {
            DRAW_COL_LATER
        };
    }
}

/// Assigns a column to every range still waiting for one, now that the
/// caller knows where it is.
///
/// # Safety
/// `state` must be live.
pub unsafe fn decor_recheck_draw_col(win_col: c_int, hidden: bool, state: *mut DecorState) {
    // SAFETY: the caller's state.
    unsafe {
        for i in 0..(*state).current_end {
            let r = decor_range_at(state, i);
            if (*r).draw_col == DRAW_COL_PENDING {
                decor_init_draw_col(win_col, hidden, r);
            }
        }
    }
}

/// Advances the state to `col` and answers the attribute the cell there is
/// drawn with.
///
/// Four things happen, in order: marks the column has reached are pulled out
/// of the marktree and turned into ranges; ranges whose start has been passed
/// are promoted into the active list, in priority order; the active list is
/// walked to combine attributes and to drop the ranges that have ended; and
/// `col_last` records how far the answer stays valid, which is what lets
/// `decor_redraw_col` skip this entirely for most columns.
///
/// # Safety
/// `wp` and `state` must be live.
pub unsafe fn decor_redraw_col_impl(
    wp: *mut win_T,
    col: c_int,
    win_col: c_int,
    hidden: bool,
    state: *mut DecorState,
    max_col_last: c_int,
) -> c_int {
    // SAFETY: the caller's window and state, and the editor's marktree.
    unsafe {
        let tree: *mut MarkTree = (&raw mut (*(*wp).w_buffer).b_marktree).cast();
        let row = (*state).row;
        let mut col_last = max_col_last;
        let itr: *mut MarkTreeIter = (&raw mut (*state).itr).cast();

        loop {
            // TODO(bfredl): check duplicate entry in "intersection" branch
            let mark = marktree_itr_current(itr);
            if mark.pos.row < 0 || mark.pos.row > row {
                break;
            } else if mark.pos.row == row && mark.pos.col > col {
                col_last = col_last.min(mark.pos.col - 1);
                break;
            }

            if !mt_invalid(mark) && !mt_end(mark) && mt_decor_any(mark) && ns_in_win(mark.ns, wp) {
                let endpos: MTPos = marktree_get_altpos(tree, mark, ptr::null_mut());
                decor_range_add_from_inline(
                    state,
                    mark.pos.row,
                    mark.pos.col,
                    endpos.row,
                    endpos.col,
                    mt_decor(mark),
                    false,
                    mark.ns,
                    mark.id,
                );
            }
            marktree_itr_next(tree, itr);
        }

        // Raw pointers into the two vectors for the rest of the function, as
        // upstream has: nothing below adds a range, and `hl_combine_attr`
        // does not reach back here.
        let indices = (*state).ranges_i.as_mut_ptr();
        let slots = (*state).slots.as_mut_ptr();
        let range_at = |i: c_int| &raw mut (*slots.add(*indices.add(i as usize) as usize)).range;

        let mut count = decor_range_count(state);
        let mut cur_end = (*state).current_end;
        let mut fut_beg = (*state).future_begin;

        // Promote the ranges the column has reached into the active list,
        // each inserted at its place in priority order.
        while fut_beg < count {
            let index = *indices.add(fut_beg as usize);
            let r = &raw mut (*slots.add(index as usize)).range;
            if (*r).start_row > row || ((*r).start_row == row && (*r).start_col > col) {
                break;
            }
            let ordering = (*r).ordering;
            let priority = (*r).priority_internal;

            let mut begin = 0;
            let mut end = cur_end;
            while begin < end {
                let mid = begin + ((end - begin) >> 1);
                let mr = range_at(mid);
                if (*mr).priority_internal < priority
                    || ((*mr).priority_internal == priority && (*mr).ordering < ordering)
                {
                    begin = mid + 1;
                } else {
                    end = mid;
                }
            }

            let at = indices.add(begin as usize);
            ptr::copy(at, at.add(1), (cur_end - begin) as usize);
            *at = index;
            cur_end += 1;
            fut_beg += 1;
        }

        // The next range to start bounds how far this answer holds.
        if fut_beg < count {
            let r = range_at(fut_beg);
            if (*r).start_row == row {
                col_last = col_last.min((*r).start_col - 1);
            }
        }

        let mut new_cur_end = 0;
        let mut attr = 0;
        let mut conceal = 0;
        let mut conceal_char = 0;
        let mut conceal_attr = 0;
        let mut spell = kNone;

        for i in 0..cur_end {
            let index = *indices.add(i as usize);
            let slot = slots.add(index as usize);
            let r = &raw mut (*slot).range;

            let ended = (*r).end_row < row || ((*r).end_row == row && (*r).end_col <= col);
            let keep = if ended {
                // A virtual position that starts on this row is kept even
                // past its end: it has not been drawn yet.
                (*r).start_row >= row && decor_virt_pos(r)
            } else {
                if (*r).end_row == row && (*r).end_col > col {
                    col_last = col_last.min((*r).end_col - 1);
                }
                if (*r).attr_id > 0 {
                    attr = hl_combine_attr(attr, (*r).attr_id);
                }
                if (*r).kind == kDecorKindHighlight {
                    let sh_flags = (*r).data.sh.flags as c_int;
                    if sh_flags & kSHConceal as c_int != 0 {
                        conceal = 1;
                        // The replacement character only shows at the very
                        // first cell of the concealed range.
                        if (*r).start_row == row && (*r).start_col == col {
                            conceal = 2;
                            conceal_char = (*r).data.sh.text[0];
                            col_last = col_last.min((*r).start_col);
                            conceal_attr = (*r).attr_id;
                        }
                    }
                    if sh_flags & kSHSpellOn as c_int != 0 {
                        spell = kTrue;
                    } else if sh_flags & kSHSpellOff as c_int != 0 {
                        spell = kFalse;
                    }
                    if !(*r).data.sh.url.is_null() {
                        attr = hl_add_url(attr, (*r).data.sh.url);
                    }
                }
                true
            };

            if (*r).start_row == row
                && (*r).start_col <= col
                && decor_virt_pos(r)
                && (*r).draw_col == DRAW_COL_NEW
            {
                decor_init_draw_col(win_col, hidden, r);
            }

            if keep {
                *indices.add(new_cur_end as usize) = index;
                new_cur_end += 1;
            } else {
                if (*r).owned {
                    if (*r).kind == kDecorKindVirtText {
                        clear_virttext(&raw mut (*(*r).data.vt).data.virt_text);
                        xfree((*r).data.vt.cast());
                    } else if (*r).kind == kDecorKindHighlight {
                        xfree((*r).data.sh.url as *mut _);
                    }
                }
                (*slot).next_free_i = (*state).free_slot_i;
                (*state).free_slot_i = index;
            }
        }
        cur_end = new_cur_end;

        if fut_beg == count {
            count = cur_end;
            fut_beg = cur_end;
        }

        (*state).ranges_i.truncate(count as usize);
        (*state).future_begin = fut_beg;
        (*state).current_end = cur_end;
        (*state).col_last = col_last;

        (*state).current = attr;
        (*state).conceal = conceal;
        (*state).conceal_char = conceal_char;
        (*state).conceal_attr = conceal_attr;
        (*state).spell = spell;
        attr
    }
}

/// The attribute of the cell at `col`, answered from the cached one when the
/// column has not passed the point the last answer holds to.
///
/// # Safety
/// `wp` and `state` must be live.
#[inline(always)]
pub unsafe fn decor_redraw_col(
    wp: *mut win_T,
    col: c_int,
    win_col: c_int,
    hidden: bool,
    state: *mut DecorState,
    max_col_last: c_int,
) -> c_int {
    // SAFETY: the caller's window and state.
    unsafe {
        if col <= (*state).col_last {
            return (*state).current;
        }
        decor_redraw_col_impl(wp, col, win_col, hidden, state, max_col_last)
    }
}

/// Finishes the line: folds in the `hl_eol` highlights that colour past the
/// end of the text, and says whether anything virtual still wants drawing.
///
/// # Safety
/// `wp` and `state` must be live; `eol_attr` must be writable.
pub unsafe fn decor_redraw_eol(
    wp: *mut win_T,
    state: *mut DecorState,
    eol_attr: *mut c_int,
    eol_col: c_int,
) -> bool {
    // SAFETY: the caller's window, state and out-parameter.
    unsafe {
        decor_redraw_col(wp, MAXCOL, MAXCOL, false, state, MAXCOL);
        (*state).eol_col = eol_col;

        let mut has_virt_pos = false;
        for i in 0..(*state).current_end {
            let r = decor_range_at(state, i);
            has_virt_pos |= (*r).start_row == (*state).row && decor_virt_pos(r);
            if (*r).kind == kDecorKindHighlight
                && (*r).data.sh.flags as c_int & kSHHlEol as c_int != 0
            {
                *eol_attr = hl_combine_attr(*eol_attr, (*r).attr_id);
            }
        }
        has_virt_pos
    }
}
