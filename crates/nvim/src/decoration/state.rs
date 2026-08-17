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
//! runs per column of every drawn line, so it works through slices of the
//! two vectors rather than re-indexing them.
//!
//! # The state's own walk
//!
//! Unlike everywhere else in `decoration/`, the marktree walk here is **not**
//! a [`Cursor`](crate::marktree::cursor::Cursor). The iterator is
//! a *field of the state*, kept between rows of a redraw, and a `Cursor` would
//! have to hold a pointer to it across the state's own field writes — which
//! `State`'s `DerefMut` invalidates, because that reborrows the whole struct.
//! So the four walk steps are [`State`] accessors that derive the iterator
//! afresh each call, out of the raw pointer and inside the one statement that
//! uses it. Same promise, taken at the same place; it just cannot be cached.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{
    Range, Sh, State, Virt, decor_sh_from_inline, kSHConceal, kSHHlEol, kSHIsSign, kSHSpellOff,
    kSHSpellOn, kSHUIWatched, kSHUIWatchedOverlay, ns_in_win, slot_range,
};
use crate::decoration::{
    clear_virttext, kDecorKindHighlight, kDecorKindUIWatched, kDecorKindVirtLines,
    kDecorKindVirtText, kVPosEndOfLine, kVPosInline, kVPosOverlay,
};
use crate::highlight::{hl_add_url, hl_combine_attr};
use crate::highlight_group::syn_id2attr;
use crate::main::decor_state;
use crate::marktree::cursor::tree_of;
use crate::marktree::key::{MT_INVALID_KEY, mt_decor, mt_decor_any, mt_end, mt_invalid};
use crate::marktree::{
    marktree_get_altpos, marktree_itr_current, marktree_itr_get, marktree_itr_get_overlap,
    marktree_itr_next, marktree_itr_step_overlap,
};
use crate::memory::xfree;
use crate::pos::MAXCOL;
use crate::types::{
    DecorInline, DecorPriority, DecorPriorityInternal, DecorRange, DecorRange_data,
    DecorRange_data_ui, DecorRangeSlot, DecorSignHighlight, DecorState, DecorVirtText, MTKey,
    MTPair, MTPos, VirtTextPos, buf_T, kFalse, kNone, kTrue, uint32_t, win_T,
};
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;
use core::slice;

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
// The state's walk, and its slab
// ---------------------------------------------------------------------------

impl State {
    /// The mark the state's walk is on — an invalid key once it has run off
    /// the end.
    fn mark(self) -> MTKey {
        // SAFETY: the state is live and its iterator is empty or positioned
        // in the tree of the buffer being drawn. Derived from the raw
        // pointer and spent in this statement, so a later field write
        // through the state cannot invalidate it.
        unsafe { marktree_itr_current(&mut (*self.raw()).itr[0]) }
    }

    /// Positions the walk at the first mark of `buf` at or after `row`.
    fn seek(self, buf: Buf, row: c_int) {
        // SAFETY: as [`State::mark`]; this is what positions the iterator.
        unsafe { marktree_itr_get(&mut *tree_of(buf), row, 0, &mut (*self.raw()).itr[0]) };
    }

    /// Steps the walk to the next mark of `buf`.
    fn step(self, buf: Buf) {
        // SAFETY: as [`State::mark`].
        unsafe { marktree_itr_next(&mut *tree_of(buf), &mut (*self.raw()).itr[0]) };
    }

    /// Positions the walk to enumerate the ranges *covering* (`row`, 0).
    fn seek_overlap(self, buf: Buf, row: c_int) -> bool {
        // SAFETY: as [`State::mark`].
        unsafe { marktree_itr_get_overlap(&mut *tree_of(buf), row, 0, &mut (*self.raw()).itr[0]) }
    }

    /// One more range covering the position [`State::seek_overlap`] was given.
    fn step_overlap(self, buf: Buf, pair: &mut MTPair) -> bool {
        // SAFETY: as [`State::mark`].
        unsafe { marktree_itr_step_overlap(&mut *tree_of(buf), &mut (*self.raw()).itr[0], pair) }
    }

    /// The range `ranges_i[i]` names.
    fn range_at(self, i: c_int) -> Range {
        let slot = self.ranges_i[i as usize] as usize;
        assert!(slot < self.slots.len());
        let state = self.raw();
        // SAFETY: the slab lives on the heap, so the pointer survives any
        // later write through the state itself; what invalidates it is a
        // range being *added*, which is why every caller re-derives after
        // one.
        unsafe { Range::new(&raw mut (*(*state).slots.as_mut_ptr().add(slot)).range) }
    }

    /// The indices of every live entry of `ranges_i` — the active list and
    /// the future list, skipping the gap between them.
    fn list_spans(self) -> impl Iterator<Item = c_int> {
        let count = self.ranges_i.len() as c_int;
        (0..self.current_end).chain(self.future_begin..count)
    }
}

impl Range {
    /// The highlight item this range draws, for a `kDecorKindHighlight`.
    fn sh(self) -> DecorSignHighlight {
        // SAFETY: the caller checked `kind`, which says the union holds the
        // `sh` branch.
        unsafe { self.data.sh }
    }

    /// The virtual text this range draws, if it is one at all — `data.vt` is
    /// the union's own branch and may itself be null.
    fn virt_opt(self) -> Option<Virt> {
        // SAFETY: `kind` says the union holds the `vt` branch, and a range's
        // virtual text is live for as long as the range is.
        (self.kind == kDecorKindVirtText)
            .then(|| unsafe { Virt::from_raw(self.data.vt) })
            .flatten()
    }

    /// The UI-watched position this range reports, for a
    /// `kDecorKindUIWatched`.
    fn ui(self) -> DecorRange_data_ui {
        // SAFETY: the caller checked `kind`.
        unsafe { self.data.ui }
    }

    /// Whether this range occupies a position of its own rather than
    /// colouring the text under it — virtual text, or a mark a UI wants told
    /// about.
    fn is_virt_pos(self) -> bool {
        self.kind == kDecorKindVirtText || self.kind == kDecorKindUIWatched
    }

    /// Where a virtual-position range wants to be drawn.
    fn virt_pos_kind(self) -> VirtTextPos {
        match self.kind {
            kDecorKindVirtText => self.virt().pos,
            kDecorKindUIWatched => self.ui().pos,
            // Not used; answer whatever.
            _ => kVPosEndOfLine,
        }
    }

    /// Frees whatever an *owned* range carries — the ephemeral virtual text
    /// a decoration provider left behind, or a URL.
    fn free_owned(self) {
        if !self.owned {
            return;
        }
        if self.kind == kDecorKindVirtText {
            let vt = self.virt();
            // SAFETY: an owned range owns its virtual text outright.
            unsafe { clear_virttext(vt.text_ptr()) };
            // SAFETY: as above.
            unsafe { xfree(vt.raw().cast()) };
        } else if self.kind == kDecorKindHighlight {
            // SAFETY: an owned highlight owns its URL string.
            unsafe { xfree(self.sh().url.cast_mut().cast()) };
        }
    }
}

// ---------------------------------------------------------------------------
// The lists
// ---------------------------------------------------------------------------

/// How many entries `ranges_i` has, active and future together.
///
/// # Safety
/// `state` must point to a live `DecorState`.
pub unsafe fn decor_range_count(state: *const DecorState) -> c_int {
    // SAFETY: the caller's state.
    let state = unsafe { State::new(state.cast_mut()) };
    state.ranges_i.len() as c_int
}

/// The range `ranges_i[i]` names.
///
/// A pointer, not a borrow: the drawing code holds several of these at once
/// and writes `draw_col` through them while reading the rest of the state.
///
/// # Safety
/// `state` must be live and `i` a valid index into `ranges_i`.
pub unsafe fn decor_range_at(state: *mut DecorState, i: c_int) -> *mut DecorRange {
    // SAFETY: the caller's state and index.
    let state = unsafe { State::new(state) };
    state.range_at(i).raw()
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
        if let Some(win) = unsafe { Win::from_raw(state.win) } {
            state.itr_valid &= win.w_buffer != buf;
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
    let mut state = unsafe { State::new(state) };
    state.slots = Vec::new();
    state.ranges_i = Vec::new();
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
    let (wp, mut state) = unsafe { (Win::new(wp), State::new(state)) };
    state.row = -1;
    state.win = wp.raw();

    for i in state.list_spans() {
        // Only the ephemeral virtual texts: an owned URL belongs to a range
        // the column loop is still holding, and is freed there.
        let r = state.range_at(i);
        if r.kind == kDecorKindVirtText {
            r.free_owned();
        }
    }

    state.slots.clear();
    state.ranges_i.clear();
    state.free_slot_i = -1;
    state.current_end = 0;
    state.future_begin = 0;
    state.new_range_ordering = 0;

    wp.buffer().b_marktree[0].n_keys != 0
}

/// Whether `decor` occupies a position of its own rather than colouring the
/// text under it — virtual text, or a mark a UI wants told about.
///
/// # Safety
/// `decor` must be live.
pub unsafe fn decor_virt_pos(decor: *const DecorRange) -> bool {
    // SAFETY: the caller's range.
    unsafe { Range::new(decor.cast_mut()) }.is_virt_pos()
}

/// Where a virtual-position range wants to be drawn.
///
/// # Safety
/// `decor` must be live.
pub unsafe fn decor_virt_pos_kind(decor: *const DecorRange) -> VirtTextPos {
    // SAFETY: the caller's range.
    unsafe { Range::new(decor.cast_mut()) }.virt_pos_kind()
}

/// Seeds the state at the top of a window with the marks that start *above*
/// `top_row` and reach into it.
///
/// # Safety
/// `wp` and `state` must be live.
pub unsafe fn decor_redraw_start(wp: *mut win_T, top_row: c_int, state: *mut DecorState) -> bool {
    // SAFETY: the caller's window and state.
    let (wp, mut state) = unsafe { (Win::new(wp), State::new(state)) };
    let buf = wp.buffer();
    state.top_row = top_row;
    state.itr_valid = true;

    if !state.seek_overlap(buf, top_row) {
        return false;
    }

    // Only read back when the step answers true, which is when it has
    // written every field.
    let mut pair = MTPair {
        start: MT_INVALID_KEY,
        end_pos: MTPos::default(),
        end_right_gravity: false,
    };
    while state.step_overlap(buf, &mut pair) {
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

/// Squeezes the gap between the active list and the future list out of
/// `ranges_i`, so that the future list does not walk off to infinity as the
/// window is drawn.
///
/// # Safety
/// `state` must be live.
pub(crate) unsafe fn decor_state_pack(state: *mut DecorState) {
    // SAFETY: the caller's state.
    let mut state = unsafe { State::new(state) };
    let count = state.ranges_i.len();
    let cur_end = state.current_end as usize;
    let fut_beg = state.future_begin as usize;

    if fut_beg == count {
        state.ranges_i.truncate(cur_end);
    } else if fut_beg != cur_end {
        state.ranges_i.copy_within(fut_beg..count, cur_end);
        state.ranges_i.truncate(cur_end + (count - fut_beg));
    }
    state.future_begin = state.current_end;
}

/// Moves the state on to `row`.
///
/// # Safety
/// `wp` and `state` must be live.
pub unsafe fn decor_redraw_line(wp: *mut win_T, row: c_int, state: *mut DecorState) {
    // SAFETY: the caller's window and state.
    let (wp, mut state) = unsafe { (Win::new(wp), State::new(state)) };
    // SAFETY: as above.
    unsafe { decor_state_pack(state.raw()) };

    if state.row == -1 {
        // SAFETY: as above.
        unsafe { decor_redraw_start(wp.raw(), row, state.raw()) };
    } else if !state.itr_valid {
        state.seek(wp.buffer(), row);
        state.itr_valid = true;
    }

    state.row = row;
    state.col_last = -1;
    state.eol_col = -1;
}

/// Whether there are (likely) more decorations on `row`.
///
/// # Safety
/// `state` must be live.
pub unsafe fn decor_has_more_decorations(state: *mut DecorState, row: c_int) -> bool {
    // SAFETY: the caller's state.
    let state = unsafe { State::new(state) };
    if state.current_end != 0 || state.future_begin != state.ranges_i.len() as c_int {
        return true;
    }
    let k = state.mark();
    k.pos.row >= 0 && k.pos.row <= row
}

// ---------------------------------------------------------------------------
// Adding ranges
// ---------------------------------------------------------------------------

/// Splits one mark's decoration into ranges and adds them all: a mark can
/// carry a chain of virtual texts and a chain of sign/highlight items.
#[allow(clippy::too_many_arguments)]
pub(crate) fn decor_range_add_from_inline(
    state: State,
    start_row: c_int,
    start_col: c_int,
    end_row: c_int,
    end_col: c_int,
    decor: DecorInline,
    owned: bool,
    ns: uint32_t,
    mark_id: uint32_t,
) {
    if !decor.ext {
        // SAFETY: an inline decoration's highlight is the union's own data.
        let mut sh = unsafe { decor_sh_from_inline(decor.data.hl) };
        add_sh(
            state, start_row, start_col, end_row, end_col, &mut sh, owned, ns, mark_id, 0,
        );
        return;
    }

    // SAFETY: `ext` says the union holds the chain head, which belongs to a
    // live mark.
    for vt in unsafe { Virt::chain(decor.data.ext.vt) } {
        decor_range_add_virt_h(state, start_row, start_col, end_row, end_col, vt, owned);
    }

    for mut sh in Sh::chain(decor) {
        add_sh(
            state, start_row, start_col, end_row, end_col, &mut sh, owned, ns, mark_id, 0,
        );
    }
}

/// Files `range` in a slot and puts its index in the future list, which stays
/// sorted by starting position.
fn decor_range_insert(mut state: State, range: &mut DecorRange) {
    range.ordering = state.new_range_ordering;
    state.new_range_ordering += 1;

    // Reuse a freed slot if there is one; the freelist is threaded through
    // the slots themselves.
    let index = if state.free_slot_i >= 0 {
        let index = state.free_slot_i as usize;
        // SAFETY: a freelist index is one this slab handed out; the union's
        // other branch is what the freelist wrote there.
        state.free_slot_i = unsafe { state.slots[index].next_free_i };
        state.slots[index].range = *range;
        index
    } else {
        state.slots.push(DecorRangeSlot { range: *range });
        state.slots.len() - 1
    };

    // Binary search for the first entry that starts after this one — but
    // stopping early on an exact position match, which puts equal positions
    // in insertion order.
    let count = state.ranges_i.len();
    let mut begin = state.future_begin as usize;
    let mut end = count;
    while begin < end {
        let mid = begin + ((end - begin) >> 1);
        let mr = state.range_at(mid as c_int);
        let (mrow, mcol) = (mr.start_row, mr.start_col);
        if mrow < range.start_row || (mrow == range.start_row && mcol <= range.start_col) {
            begin = mid + 1;
            if mrow == range.start_row && mcol == range.start_col {
                break;
            }
        } else {
            end = mid;
        }
    }

    state.ranges_i.insert(begin, index as c_int);
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
    let (state, vt) = unsafe { (State::new(state), Virt::new(vt)) };
    decor_range_add_virt_h(state, start_row, start_col, end_row, end_col, vt, owned);
}

/// [`decor_range_add_virt`] for handles the caller has already promised.
fn decor_range_add_virt_h(
    state: State,
    start_row: c_int,
    start_col: c_int,
    end_row: c_int,
    end_col: c_int,
    vt: Virt,
    owned: bool,
) {
    let mut range = DecorRange {
        start_row,
        start_col,
        end_row,
        end_col,
        ordering: 0,
        // Virtual texts carry no subpriority, so the low 16 bits are zero;
        // `decor_range_add_sh` fills them in. Both sides must use the same
        // shift or the two kinds no longer interleave.
        priority_internal: DecorPriorityInternal::from(vt.priority) << 16,
        owned,
        kind: if vt.is_lines() {
            kDecorKindVirtLines
        } else {
            kDecorKindVirtText
        },
        data: DecorRange_data { vt: vt.raw() },
        attr_id: 0,
        draw_col: DRAW_COL_NEW,
    };
    decor_range_insert(state, &mut range);
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
    let (state, mut sh) = unsafe { (State::new(state), Sh::new(sh)) };
    add_sh(
        state,
        start_row,
        start_col,
        end_row,
        end_col,
        &mut sh,
        owned,
        ns,
        mark_id,
        subpriority,
    );
}

/// [`decor_range_add_sh`] for a state the caller has already promised.
#[allow(clippy::too_many_arguments)]
fn add_sh(
    state: State,
    start_row: c_int,
    start_col: c_int,
    end_row: c_int,
    end_col: c_int,
    sh: &mut DecorSignHighlight,
    owned: bool,
    ns: uint32_t,
    mark_id: uint32_t,
    subpriority: DecorPriority,
) {
    let flags = sh.flags as c_int;
    if flags & kSHIsSign as c_int != 0 {
        return;
    }

    let mut range = DecorRange {
        start_row,
        start_col,
        end_row,
        end_col,
        ordering: 0,
        priority_internal: (DecorPriorityInternal::from(sh.priority) << 16)
            + DecorPriorityInternal::from(subpriority),
        owned,
        kind: kDecorKindHighlight,
        data: DecorRange_data { sh: *sh },
        attr_id: 0,
        draw_col: DRAW_COL_NEW,
    };

    if sh.hl_id != 0
        || !sh.url.is_null()
        || flags & (kSHConceal | kSHSpellOn | kSHSpellOff) as c_int != 0
    {
        if sh.hl_id != 0 {
            // SAFETY: the highlight tables are the editor's own.
            range.attr_id = unsafe { syn_id2attr(sh.hl_id) };
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

// ---------------------------------------------------------------------------
// Drawing positions
// ---------------------------------------------------------------------------

/// Decides where a newly-started virtual-position range goes.
///
/// # Safety
/// `item` must be live.
pub unsafe fn decor_init_draw_col(win_col: c_int, hidden: bool, item: *mut DecorRange) {
    // SAFETY: the caller's range.
    init_draw_col(win_col, hidden, unsafe { Range::new(item) });
}

/// [`decor_init_draw_col`] for a range already promised live.
fn init_draw_col(win_col: c_int, hidden: bool, mut item: Range) {
    let pos = item.virt_pos_kind();
    item.draw_col = if win_col < 0 && pos != kVPosInline {
        // A negative `win_col` is itself a sentinel the caller passes in.
        win_col
    } else if pos == kVPosOverlay {
        let hides = item.virt_opt().is_some_and(Virt::hides_over_concealed);
        if hides && hidden {
            DRAW_COL_NEVER
        } else {
            win_col
        }
    } else {
        DRAW_COL_LATER
    };
}

/// Assigns a column to every range still waiting for one, now that the
/// caller knows where it is.
///
/// # Safety
/// `state` must be live.
pub unsafe fn decor_recheck_draw_col(win_col: c_int, hidden: bool, state: *mut DecorState) {
    // SAFETY: the caller's state.
    let state = unsafe { State::new(state) };
    for i in 0..state.current_end {
        let r = state.range_at(i);
        if r.draw_col == DRAW_COL_PENDING {
            init_draw_col(win_col, hidden, r);
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
    // SAFETY: the caller's window and state.
    let (wp, mut state) = unsafe { (Win::new(wp), State::new(state)) };
    let buf = wp.buffer();
    let row = state.row;
    let mut col_last = max_col_last;

    loop {
        // TODO(bfredl): check duplicate entry in "intersection" branch
        let mark = state.mark();
        if mark.pos.row < 0 || mark.pos.row > row {
            break;
        } else if mark.pos.row == row && mark.pos.col > col {
            col_last = col_last.min(mark.pos.col - 1);
            break;
        }

        if !mt_invalid(mark) && !mt_end(mark) && mt_decor_any(mark) && ns_in_win(mark.ns, wp) {
            // SAFETY: `mark` was read out of this buffer's live tree.
            let endpos: MTPos = unsafe { marktree_get_altpos(&mut *tree_of(buf), mark, None) };
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
        state.step(buf);
    }

    // Slices of the two vectors for the rest of the function, as upstream
    // takes raw pointers: nothing below adds a range, and `hl_combine_attr`
    // does not reach back here.
    let (n_idx, n_slots) = (state.ranges_i.len(), state.slots.len());
    let s = state.raw();
    // SAFETY: the state's own two vectors, neither of which is grown below.
    // Both keep their entries on the heap, so the slices survive the field
    // writes at the bottom of this function.
    let indices = unsafe { slice::from_raw_parts_mut((*s).ranges_i.as_mut_ptr(), n_idx) };
    // SAFETY: as above.
    let slots = unsafe { slice::from_raw_parts_mut((*s).slots.as_mut_ptr(), n_slots) };

    let mut count = indices.len() as c_int;
    let mut cur_end = state.current_end;
    let mut fut_beg = state.future_begin;

    // Promote the ranges the column has reached into the active list, each
    // inserted at its place in priority order.
    while fut_beg < count {
        let index = indices[fut_beg as usize];
        let r = slot_range(&mut slots[index as usize]);
        if r.start_row > row || (r.start_row == row && r.start_col > col) {
            break;
        }
        let (ordering, priority) = (r.ordering, r.priority_internal);

        let mut begin = 0;
        let mut end = cur_end;
        while begin < end {
            let mid = begin + ((end - begin) >> 1);
            let mr = slot_range(&mut slots[indices[mid as usize] as usize]);
            if mr.priority_internal < priority
                || (mr.priority_internal == priority && mr.ordering < ordering)
            {
                begin = mid + 1;
            } else {
                end = mid;
            }
        }

        indices.copy_within(begin as usize..cur_end as usize, begin as usize + 1);
        indices[begin as usize] = index;
        cur_end += 1;
        fut_beg += 1;
    }

    // The next range to start bounds how far this answer holds.
    if fut_beg < count {
        let r = slot_range(&mut slots[indices[fut_beg as usize] as usize]);
        if r.start_row == row {
            col_last = col_last.min(r.start_col - 1);
        }
    }

    let mut new_cur_end = 0;
    let mut attr = 0;
    let mut conceal = 0;
    let mut conceal_char = 0;
    let mut conceal_attr = 0;
    let mut spell = kNone;

    for i in 0..cur_end {
        let index = indices[i as usize];
        // SAFETY: an index out of the active list names an occupied slot of
        // the slab, which lives on the heap and is not grown here.
        let r = unsafe { Range::new(&raw mut *slot_range(&mut slots[index as usize])) };

        let ended = r.end_row < row || (r.end_row == row && r.end_col <= col);
        let keep = if ended {
            // A virtual position that starts on this row is kept even past
            // its end: it has not been drawn yet.
            r.start_row >= row && r.is_virt_pos()
        } else {
            if r.end_row == row && r.end_col > col {
                col_last = col_last.min(r.end_col - 1);
            }
            if r.attr_id > 0 {
                // SAFETY: the highlight tables are the editor's own.
                attr = unsafe { hl_combine_attr(attr, r.attr_id) };
            }
            if r.kind == kDecorKindHighlight {
                let sh = r.sh();
                let sh_flags = sh.flags as c_int;
                if sh_flags & kSHConceal as c_int != 0 {
                    conceal = 1;
                    // The replacement character only shows at the very first
                    // cell of the concealed range.
                    if r.start_row == row && r.start_col == col {
                        conceal = 2;
                        conceal_char = sh.text[0];
                        col_last = col_last.min(r.start_col);
                        conceal_attr = r.attr_id;
                    }
                }
                if sh_flags & kSHSpellOn as c_int != 0 {
                    spell = kTrue;
                } else if sh_flags & kSHSpellOff as c_int != 0 {
                    spell = kFalse;
                }
                if !sh.url.is_null() {
                    // SAFETY: an item's URL is a live NUL-terminated string.
                    attr = unsafe { hl_add_url(attr, sh.url) };
                }
            }
            true
        };

        if r.start_row == row && r.start_col <= col && r.is_virt_pos() && r.draw_col == DRAW_COL_NEW
        {
            init_draw_col(win_col, hidden, r);
        }

        if keep {
            indices[new_cur_end as usize] = index;
            new_cur_end += 1;
        } else {
            r.free_owned();
            // Writing the union's *other* branch, which is what puts the
            // slot on the freelist.
            slots[index as usize].next_free_i = state.free_slot_i;
            state.free_slot_i = index;
        }
    }
    cur_end = new_cur_end;

    if fut_beg == count {
        count = cur_end;
        fut_beg = cur_end;
    }

    state.ranges_i.truncate(count as usize);
    state.future_begin = fut_beg;
    state.current_end = cur_end;
    state.col_last = col_last;

    state.current = attr;
    state.conceal = conceal;
    state.conceal_char = conceal_char;
    state.conceal_attr = conceal_attr;
    state.spell = spell;
    attr
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
    // SAFETY: the caller's state.
    let handle = unsafe { State::new(state) };
    if col <= handle.col_last {
        return handle.current;
    }
    // SAFETY: the caller's window and state.
    unsafe { decor_redraw_col_impl(wp, col, win_col, hidden, state, max_col_last) }
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
    // SAFETY: the caller's state and out-parameter.
    let (mut state, eol_attr) = unsafe { (State::new(state), &mut *eol_attr) };
    // SAFETY: the caller's window and state.
    unsafe { decor_redraw_col(wp, MAXCOL, MAXCOL, false, state.raw(), MAXCOL) };
    state.eol_col = eol_col;

    let mut has_virt_pos = false;
    for i in 0..state.current_end {
        let r = state.range_at(i);
        has_virt_pos |= r.start_row == state.row && r.is_virt_pos();
        if r.kind == kDecorKindHighlight && r.sh().flags as c_int & kSHHlEol as c_int != 0 {
            // SAFETY: the highlight tables are the editor's own.
            *eol_attr = unsafe { hl_combine_attr(*eol_attr, r.attr_id) };
        }
    }
    has_virt_pos
}
