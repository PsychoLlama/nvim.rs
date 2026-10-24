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
//! on a freelist threaded through the slots themselves, so an index
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
//! drops the ones that have ended, and answers the combined attribute.
//!
//! # One state, borrowed per step
//!
//! There is one state, [`DECOR_STATE`], and nothing holds a borrow of it
//! across a call that can run Lua. That rule is what makes it a cell rather
//! than a lent `&mut`: the redraw calls decoration providers between and
//! inside lines, and their callbacks come back — an ephemeral extmark adds a
//! range, any extmark change invalidates the walk
//! ([`Buf::invalidate_decor_state`]), and resolving a highlight id can run a
//! provider's `hl_def`. So every function here borrows the state for the
//! pure part of its work and resolves highlights outside it, and the drawing
//! code reads the state through the small accessors at the bottom rather
//! than through a handle.
//!
//! The marktree walk is the state's own iterator, a field kept between rows
//! of a redraw; each step is a [`Cursor`] built over it for that one call.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::{
    DECOR_ID_INVALID, decor_release_sh, decor_release_vt, kSHConceal, kSHHlEol, kSHIsSign,
    kSHSpellOff, kSHSpellOn, kSHUIWatched, kSHUIWatchedOverlay, kVTHide, kVTIsLines, ns_in_win,
    sh_chain, virt_chain, with_decor_item, with_decor_virt,
};
use crate::decoration::{
    kDecorKindHighlight, kDecorKindUIWatched, kDecorKindVirtLines, kDecorKindVirtText,
    kVPosEndOfLine, kVPosInline, kVPosOverlay,
};
use crate::global_cell::GlobalCell;
use crate::highlight::{hl_add_url, hl_combine_attr};
use crate::highlight_group::syn_id2attr;
use crate::marktree::cursor::Cursor;
use crate::marktree::key::{mt_decor, mt_decor_any, mt_end, mt_invalid};
use crate::marktree::marktree_itr_current;
use crate::pos::MAXCOL;
use crate::types::{
    DecorHighlightInline, DecorInline, DecorPriority, DecorPriorityInternal, DecorRange,
    DecorRange_data, DecorRange_data_ui, DecorRangeSlot, DecorState, MTKey, MTNode, MTPos,
    MarkTreeIter, MarkTreeIterLevel, RangeHighlight, ScreenChar, VirtTextPos, uint16_t, uint32_t,
};
use crate::winlayer::WinId;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

/// The decoration state the redraw walks. See the module docs for why it is
/// borrowed per step.
static DECOR_STATE: GlobalCell<DecorState> = GlobalCell::new(DecorState {
    itr: [MarkTreeIter {
        pos: MTPos { row: 0, col: 0 },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [MarkTreeIterLevel { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }],
    slots: Vec::new(),
    ranges_i: Vec::new(),
    current_end: 0,
    future_begin: 0,
    free_slot_i: None,
    new_range_ordering: 0,
    win: None,
    top_row: 0,
    row: 0,
    col_last: 0,
    current: 0,
    eol_col: 0,
    conceal: 0,
    conceal_char: 0,
    conceal_attr: 0,
    spell: Some(false),
    itr_valid: false,
});

/// Runs `f` on the state. `f` must not run Lua or reach the state again.
#[inline(always)]
fn with_state<R>(f: impl FnOnce(&mut DecorState) -> R) -> R {
    DECOR_STATE.with_mut(f)
}

/// Reads one thing off the state.
#[inline(always)]
fn read_state<R>(f: impl FnOnce(&DecorState) -> R) -> R {
    DECOR_STATE.with(f)
}

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

impl DecorState {
    /// The state's walk over `buffer`'s marks, for one step.
    fn walk(&mut self, buffer: Buf) -> Cursor<'_> {
        Cursor::in_buffer(buffer, &mut self.itr[0])
    }

    /// The mark the state's walk is on — an invalid key once it has run off
    /// the end. Reads only the iterator.
    fn mark(&mut self) -> MTKey {
        marktree_itr_current(&mut self.itr[0])
    }

    /// The range `ranges_i[i]` names.
    fn range_at(&mut self, i: usize) -> &mut DecorRange {
        let slot = self.ranges_i[i];
        slot_range(&mut self.slots[slot])
    }

    /// The indices of every live entry of `ranges_i` — the active list and
    /// the future list, skipping the gap between them.
    fn list_spans(&self) -> impl Iterator<Item = usize> + use<> {
        let count = self.ranges_i.len();
        (0..self.current_end).chain(self.future_begin..count)
    }

    /// Frees what every owned range still in the lists holds.
    fn free_all_owned(&mut self) {
        for i in self.list_spans() {
            free_owned(self.range_at(i));
        }
    }
}

/// The range in a slab slot.
///
/// Every caller has an index out of one of the two sorted lists, and those
/// name occupied slots; a freed slot holds a freelist link instead.
#[inline(always)]
fn slot_range(slot: &mut DecorRangeSlot) -> &mut DecorRange {
    match slot {
        DecorRangeSlot::Range(range) => range,
        DecorRangeSlot::Free(_) => unreachable!("decor: a sorted index names a freed slot"),
    }
}

/// The virtual-text link a range draws, if it is a virtual *text* at all.
fn virt_opt(range: &DecorRange) -> Option<uint32_t> {
    (range.kind == kDecorKindVirtText).then(|| range.data.virt())
}

/// Whether `range` occupies a position of its own rather than colouring the
/// text under it — virtual text, or a mark a UI wants told about.
#[inline]
pub(crate) fn decor_virt_pos(range: &DecorRange) -> bool {
    range.kind == kDecorKindVirtText || range.kind == kDecorKindUIWatched
}

/// Where a virtual-position range wants to be drawn.
pub(crate) fn decor_virt_pos_kind(range: &DecorRange) -> VirtTextPos {
    match range.kind {
        kDecorKindVirtText => with_decor_virt(range.data.virt(), |vt| vt.pos),
        kDecorKindUIWatched => range.data.ui_watched().pos,
        // Not used; answer whatever.
        _ => kVPosEndOfLine,
    }
}

/// Frees whatever an *owned* range carries — the ephemeral virtual text or
/// virtual lines a decoration provider left behind, or the store entry
/// holding an ephemeral highlight's URL.
fn free_owned(range: &DecorRange) {
    if !range.owned {
        return;
    }
    match range.kind {
        kDecorKindVirtText | kDecorKindVirtLines => decor_release_vt(range.data.virt()),
        kDecorKindHighlight if range.data.highlight().url != DECOR_ID_INVALID => {
            decor_release_sh(range.data.highlight().url);
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// The lists
// ---------------------------------------------------------------------------

/// A copy of the range `ranges_i[i]` names. Write its `draw_col` back with
/// [`set_decor_range_draw_col`].
///
/// # Panics
/// If `i` is not an index into `ranges_i`.
pub(crate) fn decor_range(i: usize) -> DecorRange {
    with_state(|state| *state.range_at(i))
}

/// Sets the `draw_col` of the range `ranges_i[i]` names.
pub(crate) fn set_decor_range_draw_col(i: usize, draw_col: c_int) {
    with_state(|state| state.range_at(i).draw_col = draw_col);
}

impl Buf {
    /// Called whenever a public API function adds or deletes marks, in case that
    /// happened in a callback the drawing code is inside: the marktree iterator
    /// the state is holding cannot be trusted across a structural change.
    pub fn invalidate_decor_state(self) {
        with_state(|state| {
            // The provider that called in may have closed the window.
            if let Some(win) = state.win.and_then(WinId::get) {
                state.itr_valid &= win.buffer() != self;
            }
        });
    }
}

/// Puts the state aside, answering what it held and leaving an empty one:
/// the spelling-navigation scan rebuilds it per line and puts the old one
/// back with [`decor_state_restore`] when it is done.
pub(crate) fn decor_state_set_aside() -> DecorState {
    DECOR_STATE.take()
}

/// Puts back what [`decor_state_set_aside`] answered, releasing what the
/// ephemeral ranges of the state it replaces own.
pub(crate) fn decor_state_restore(saved: DecorState) {
    let replaced = with_state(|state| {
        state.free_all_owned();
        core::mem::replace(state, saved)
    });
    drop(replaced);
}

impl Win {
    /// Starts a fresh window: empties both lists, freeing what the ephemeral
    /// ranges a decoration provider left behind own.
    ///
    /// Answers whether the buffer has any marks at all, which is the caller's cue
    /// to bother with the rest of the machinery.
    pub fn decor_redraw_reset(self) -> bool {
        with_state(|state| {
            state.row = -1;
            state.win = Some(self.id());

            state.free_all_owned();

            state.slots.clear();
            state.ranges_i.clear();
            state.free_slot_i = None;
            state.current_end = 0;
            state.future_begin = 0;
            state.new_range_ordering = 0;
        });

        self.buffer().b_marktree.n_keys != 0
    }
}

/// The window the redraw is drawing, between [`Win::decor_redraw_reset`] and
/// the end of the redraw — `None` once that window is gone, or outside a
/// redraw.
pub(crate) fn decor_drawing_window() -> Option<Win> {
    read_state(|state| state.win).and_then(WinId::get)
}

/// Forgets the window the redraw was drawing: the redraw is over.
pub(crate) fn decor_forget_window() {
    with_state(|state| state.win = None);
}

/// Seeds the state at the top of a window with the marks that start *above*
/// `top_row` and reach into it.
pub fn decor_redraw_start(window: Win, top_row: c_int) -> bool {
    let buf = window.buffer();
    let found = with_state(|state| {
        state.top_row = top_row;
        state.itr_valid = true;
        state.walk(buf).seek_overlap(top_row, 0)
    });
    if !found {
        return false;
    }

    // Each step borrows the state on its own: adding a range resolves its
    // highlight, which can run Lua.
    while let Some(pair) = with_state(|state| state.walk(buf).step_overlap()) {
        let m = pair.start;
        if mt_invalid(m) || !mt_decor_any(m) {
            continue;
        }
        decor_range_add_from_inline(
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
fn decor_state_pack(state: &mut DecorState) {
    let count = state.ranges_i.len();
    let cur_end = state.current_end;
    let fut_beg = state.future_begin;

    if fut_beg == count {
        state.ranges_i.truncate(cur_end);
    } else if fut_beg != cur_end {
        state.ranges_i.copy_within(fut_beg..count, cur_end);
        state.ranges_i.truncate(cur_end + (count - fut_beg));
    }
    state.future_begin = state.current_end;
}

/// Moves the state on to `row`.
pub fn decor_redraw_line(window: Win, row: c_int) {
    let fresh = with_state(|state| {
        decor_state_pack(state);
        state.row == -1
    });

    if fresh {
        decor_redraw_start(window, row);
    }
    with_state(|state| {
        if !fresh && !state.itr_valid {
            state.walk(window.buffer()).seek(row, 0);
            state.itr_valid = true;
        }
        state.row = row;
        state.col_last = -1;
        state.eol_col = -1;
    });
}

/// Whether there are (likely) more decorations on `row`.
pub fn decor_has_more_decorations(row: c_int) -> bool {
    with_state(|state| {
        if state.current_end != 0 || state.future_begin != state.ranges_i.len() {
            return true;
        }
        let k = state.mark();
        k.pos.row >= 0 && k.pos.row <= row
    })
}

// ---------------------------------------------------------------------------
// Adding ranges
// ---------------------------------------------------------------------------

/// Splits one mark's decoration into ranges and adds them all: a mark can
/// carry a chain of virtual texts and a chain of sign/highlight items.
#[allow(clippy::too_many_arguments)]
pub(crate) fn decor_range_add_from_inline(
    start_row: c_int,
    start_col: c_int,
    end_row: c_int,
    end_col: c_int,
    decor: DecorInline,
    owned: bool,
    ns: uint32_t,
    mark_id: uint32_t,
) {
    let ext = match decor {
        DecorInline::Hl(hl) => {
            let sh = HlSource::inline(hl);
            add_sh(
                start_row, start_col, end_row, end_col, sh, owned, ns, mark_id, 0,
            );
            return;
        }
        DecorInline::Ext(ext) => ext,
    };

    for vt in virt_chain(ext.vt_idx) {
        decor_range_add_virt(start_row, start_col, end_row, end_col, vt, owned);
    }

    for idx in sh_chain(decor) {
        let sh = with_decor_item(idx, |item| HlSource {
            flags: item.flags,
            priority: item.priority,
            hl_id: item.hl_id,
            conceal_char: item.text[0],
            url: if item.url.is_some() {
                idx
            } else {
                DECOR_ID_INVALID
            },
        });
        add_sh(
            start_row, start_col, end_row, end_col, sh, owned, ns, mark_id, 0,
        );
    }
}

/// What a highlight range is made from: one sign/highlight item's fields,
/// and the store entry holding its URL (`DECOR_ID_INVALID` for none).
#[derive(Copy, Clone)]
pub(crate) struct HlSource {
    pub(crate) flags: uint16_t,
    pub(crate) priority: DecorPriority,
    pub(crate) hl_id: c_int,
    pub(crate) conceal_char: ScreenChar,
    pub(crate) url: uint32_t,
}

impl HlSource {
    /// An inline highlight, which has no URL.
    pub(crate) fn inline(hl: DecorHighlightInline) -> Self {
        // A sign is never inline; see `decor_sh_from_inline`.
        debug_assert!(hl.flags & kSHIsSign == 0);
        Self {
            flags: hl.flags,
            priority: hl.priority,
            hl_id: hl.hl_id,
            conceal_char: hl.conceal_char,
            url: DECOR_ID_INVALID,
        }
    }
}

/// Files `range` in a slot and puts its index in the future list, which stays
/// sorted by starting position.
fn decor_range_insert(state: &mut DecorState, mut range: DecorRange) {
    range.ordering = state.new_range_ordering;
    state.new_range_ordering += 1;

    // Reuse a freed slot if there is one; the freelist is threaded through
    // the slots themselves.
    let index = if let Some(index) = state.free_slot_i {
        let DecorRangeSlot::Free(next) = state.slots[index] else {
            unreachable!("decor: the freelist names an occupied slot")
        };
        state.free_slot_i = next;
        state.slots[index] = DecorRangeSlot::Range(range);
        index
    } else {
        state.slots.push(DecorRangeSlot::Range(range));
        state.slots.len() - 1
    };

    // Binary search for the first entry that starts after this one — but
    // stopping early on an exact position match, which puts equal positions
    // in insertion order.
    let count = state.ranges_i.len();
    let mut begin = state.future_begin;
    let mut end = count;
    while begin < end {
        let mid = begin + ((end - begin) >> 1);
        let mr = state.range_at(mid);
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

    state.ranges_i.insert(begin, index);
}

/// Adds the range a virtual text or virtual-lines block occupies: link
/// `vt` of the virtual-text store, which the range takes over when `owned`
/// and otherwise borrows from its mark.
pub fn decor_range_add_virt(
    start_row: c_int,
    start_col: c_int,
    end_row: c_int,
    end_col: c_int,
    vt: uint32_t,
    owned: bool,
) {
    let (priority, flags) = with_decor_virt(vt, |link| (link.priority, link.flags));
    let range = DecorRange {
        start_row,
        start_col,
        end_row,
        end_col,
        ordering: 0,
        // Virtual texts carry no subpriority, so the low 16 bits are zero;
        // `decor_range_add_sh` fills them in. Both sides must use the same
        // shift or the two kinds no longer interleave.
        priority_internal: DecorPriorityInternal::from(priority) << 16,
        owned,
        kind: if flags & kVTIsLines != 0 {
            kDecorKindVirtLines
        } else {
            kDecorKindVirtText
        },
        data: DecorRange_data::Virt(vt),
        attr_id: 0,
        draw_col: DRAW_COL_NEW,
    };
    with_state(|state| decor_range_insert(state, range));
}

/// Adds the range a highlight (or a `ui_watched` mark) occupies. A sign is
/// not a range — it is drawn in the sign column, not over the text — so it is
/// dropped here.
///
/// One `sh` can produce two ranges: a highlight and a UI-watched position.
/// An `owned` range takes over the store entry holding `sh`'s URL.
#[allow(clippy::too_many_arguments)]
pub(crate) fn decor_range_add_sh(
    start_row: c_int,
    start_col: c_int,
    end_row: c_int,
    end_col: c_int,
    sh: HlSource,
    owned: bool,
    ns: uint32_t,
    mark_id: uint32_t,
    subpriority: DecorPriority,
) {
    add_sh(
        start_row,
        start_col,
        end_row,
        end_col,
        sh,
        owned,
        ns,
        mark_id,
        subpriority,
    );
}

/// [`decor_range_add_sh`].
#[allow(clippy::too_many_arguments)]
fn add_sh(
    start_row: c_int,
    start_col: c_int,
    end_row: c_int,
    end_col: c_int,
    sh: HlSource,
    owned: bool,
    ns: uint32_t,
    mark_id: uint32_t,
    subpriority: DecorPriority,
) {
    let flags = sh.flags;
    if flags & kSHIsSign != 0 {
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
        data: DecorRange_data::Highlight(RangeHighlight {
            flags,
            conceal_char: sh.conceal_char,
            url: sh.url,
        }),
        attr_id: 0,
        draw_col: DRAW_COL_NEW,
    };

    if sh.hl_id != 0
        || sh.url != DECOR_ID_INVALID
        || flags & (kSHConceal | kSHSpellOn | kSHSpellOff) != 0
    {
        if sh.hl_id != 0 {
            // Outside the state's borrow: this can run a provider's `hl_def`.
            range.attr_id = syn_id2attr(sh.hl_id);
        }
        with_state(|state| decor_range_insert(state, range));
    }

    if flags & kSHUIWatched != 0 {
        range.kind = kDecorKindUIWatched;
        range.data = DecorRange_data::UIWatched(DecorRange_data_ui {
            ns_id: ns,
            mark_id,
            pos: if flags & kSHUIWatchedOverlay != 0 {
                kVPosOverlay
            } else {
                kVPosEndOfLine
            },
        });
        with_state(|state| decor_range_insert(state, range));
    }
}

// ---------------------------------------------------------------------------
// Drawing positions
// ---------------------------------------------------------------------------

/// Decides where the newly-started virtual-position range `ranges_i[i]`
/// goes.
pub(crate) fn decor_init_draw_col(win_col: c_int, hidden: bool, i: usize) {
    with_state(|state| init_draw_col(win_col, hidden, state.range_at(i)));
}

/// [`decor_init_draw_col`] for a range in hand.
fn init_draw_col(win_col: c_int, hidden: bool, item: &mut DecorRange) {
    let pos = decor_virt_pos_kind(item);
    item.draw_col = if win_col < 0 && pos != kVPosInline {
        // A negative `win_col` is itself a sentinel the caller passes in.
        win_col
    } else if pos == kVPosOverlay {
        let hides =
            virt_opt(item).is_some_and(|vt| with_decor_virt(vt, |link| link.flags & kVTHide != 0));
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
pub fn decor_recheck_draw_col(win_col: c_int, hidden: bool) {
    with_state(|state| {
        for i in 0..state.current_end {
            let r = state.range_at(i);
            if r.draw_col == DRAW_COL_PENDING {
                init_draw_col(win_col, hidden, r);
            }
        }
    });
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
pub fn decor_redraw_col_impl(
    window: Win,
    col: c_int,
    win_col: c_int,
    hidden: bool,
    max_col_last: c_int,
) -> c_int {
    let buf = window.buffer();
    let row = read_state(|state| state.row);
    let mut col_last = max_col_last;

    loop {
        // TODO(bfredl): check duplicate entry in "intersection" branch
        let mark = with_state(DecorState::mark);
        if mark.pos.row < 0 || mark.pos.row > row {
            break;
        } else if mark.pos.row == row && mark.pos.col > col {
            col_last = col_last.min(mark.pos.col - 1);
            break;
        }

        if !mt_invalid(mark) && !mt_end(mark) && mt_decor_any(mark) && ns_in_win(mark.ns, window) {
            let endpos: MTPos = with_state(|state| state.walk(buf).altpos(mark));
            // Outside the state's borrow: adding a range resolves its
            // highlight, which can run Lua.
            decor_range_add_from_inline(
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
        with_state(|state| state.walk(buf).step());
    }

    // The rest adds no range and runs no Lua (the attribute table it combines
    // into reaches no decoration), so it is one borrow.
    with_state(|state| columns_reached(state, row, col, win_col, hidden, col_last))
}

/// [`decor_redraw_col_impl`] once the marks up to `col` are ranges: promote,
/// combine, drop, and record the answer.
fn columns_reached(
    state: &mut DecorState,
    row: c_int,
    col: c_int,
    win_col: c_int,
    hidden: bool,
    mut col_last: c_int,
) -> c_int {
    let mut count = state.ranges_i.len();
    let mut cur_end = state.current_end;
    let mut fut_beg = state.future_begin;
    let mut free_slot_i = state.free_slot_i;
    let indices = &mut state.ranges_i[..];
    let slots = &mut state.slots[..];

    // Promote the ranges the column has reached into the active list, each
    // inserted at its place in priority order.
    while fut_beg < count {
        let index = indices[fut_beg];
        let r = slot_range(&mut slots[index]);
        if r.start_row > row || (r.start_row == row && r.start_col > col) {
            break;
        }
        let (ordering, priority) = (r.ordering, r.priority_internal);

        let mut begin = 0;
        let mut end = cur_end;
        while begin < end {
            let mid = begin + ((end - begin) >> 1);
            let mr = slot_range(&mut slots[indices[mid]]);
            if mr.priority_internal < priority
                || (mr.priority_internal == priority && mr.ordering < ordering)
            {
                begin = mid + 1;
            } else {
                end = mid;
            }
        }

        indices.copy_within(begin..cur_end, begin + 1);
        indices[begin] = index;
        cur_end += 1;
        fut_beg += 1;
    }

    // The next range to start bounds how far this answer holds.
    if fut_beg < count {
        let r = slot_range(&mut slots[indices[fut_beg]]);
        if r.start_row == row {
            col_last = col_last.min(r.start_col - 1);
        }
    }

    let mut new_cur_end = 0;
    let mut attr = 0;
    let mut conceal = 0;
    let mut conceal_char = 0;
    let mut conceal_attr = 0;
    let mut spell = None;

    for i in 0..cur_end {
        let index = indices[i];
        let r = slot_range(&mut slots[index]);

        let ended = r.end_row < row || (r.end_row == row && r.end_col <= col);
        let keep = if ended {
            // A virtual position that starts on this row is kept even past
            // its end: it has not been drawn yet.
            r.start_row >= row && decor_virt_pos(r)
        } else {
            if r.end_row == row && r.end_col > col {
                col_last = col_last.min(r.end_col - 1);
            }
            if r.attr_id > 0 {
                attr = hl_combine_attr(attr, r.attr_id);
            }
            if r.kind == kDecorKindHighlight {
                let sh = r.data.highlight();
                if sh.flags & kSHConceal != 0 {
                    conceal = 1;
                    // The replacement character only shows at the very first
                    // cell of the concealed range.
                    if r.start_row == row && r.start_col == col {
                        conceal = 2;
                        conceal_char = sh.conceal_char;
                        col_last = col_last.min(r.start_col);
                        conceal_attr = r.attr_id;
                    }
                }
                if sh.flags & kSHSpellOn != 0 {
                    spell = Some(true);
                } else if sh.flags & kSHSpellOff != 0 {
                    spell = Some(false);
                }
                if sh.url != DECOR_ID_INVALID {
                    attr = with_decor_item(sh.url, |item| {
                        item.url
                            .as_deref()
                            .map_or(attr, |url| hl_add_url(attr, url))
                    });
                }
            }
            true
        };

        if r.start_row == row
            && r.start_col <= col
            && decor_virt_pos(r)
            && r.draw_col == DRAW_COL_NEW
        {
            init_draw_col(win_col, hidden, r);
        }

        if keep {
            indices[new_cur_end] = index;
            new_cur_end += 1;
        } else {
            free_owned(r);
            // The slot becomes the head of the freelist.
            slots[index] = DecorRangeSlot::Free(free_slot_i);
            free_slot_i = Some(index);
        }
    }
    cur_end = new_cur_end;

    if fut_beg == count {
        count = cur_end;
        fut_beg = cur_end;
    }

    state.ranges_i.truncate(count);
    state.free_slot_i = free_slot_i;
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
#[inline(always)]
pub(crate) fn decor_redraw_col(
    window: Win,
    col: c_int,
    win_col: c_int,
    hidden: bool,
    max_col_last: c_int,
) -> c_int {
    let (col_last, current) = read_state(|state| (state.col_last, state.current));
    if col <= col_last {
        return current;
    }
    decor_redraw_col_impl(window, col, win_col, hidden, max_col_last)
}

/// Finishes the line: folds into `eol_attr` the `hl_eol` highlights that
/// colour past the end of the text, and says whether anything virtual still
/// wants drawing.
pub fn decor_redraw_eol(window: Win, eol_attr: &mut c_int, eol_col: c_int) -> bool {
    decor_redraw_col(window, MAXCOL, MAXCOL, false, MAXCOL);
    with_state(|state| {
        state.eol_col = eol_col;
        let row = state.row;
        let mut has_virt_pos = false;
        for i in 0..state.current_end {
            let r = state.range_at(i);
            has_virt_pos |= r.start_row == row && decor_virt_pos(r);
            if r.kind == kDecorKindHighlight && r.data.highlight().flags & kSHHlEol != 0 {
                *eol_attr = hl_combine_attr(*eol_attr, r.attr_id);
            }
        }
        has_virt_pos
    })
}

// ---------------------------------------------------------------------------
// What the drawing code reads
// ---------------------------------------------------------------------------

/// Where the state stands, for the drawing code: the row it is on, the
/// active list `ranges_i[..current_end]`, the later-starting list
/// `ranges_i[future_begin..count]`, and the column the end-of-line virtual
/// texts start at (-1 for none yet).
#[derive(Copy, Clone)]
pub(crate) struct DecorLists {
    pub(crate) row: c_int,
    pub(crate) current_end: usize,
    pub(crate) future_begin: usize,
    pub(crate) count: usize,
    pub(crate) eol_col: c_int,
}

/// A copy of where the state stands; see [`DecorLists`].
#[inline]
pub(crate) fn decor_lists() -> DecorLists {
    read_state(|state| DecorLists {
        row: state.row,
        current_end: state.current_end,
        future_begin: state.future_begin,
        count: state.ranges_i.len(),
        eol_col: state.eol_col,
    })
}

/// Whether a range on the state's row draws inline virtual text at or
/// after byte `v` and is not drawn yet — in the active list or among the
/// ranges that start later.
///
/// The character loop asks this per character, so it is one borrow, and it
/// reads the store only for a range that could answer yes.
pub(crate) fn decor_inline_virt_from(v: isize) -> bool {
    read_state(|state| {
        let row = state.row;
        let count = state.ranges_i.len();
        let spans = [(0, state.current_end), (state.future_begin, count)];
        spans.into_iter().any(|(from, to)| {
            state.ranges_i[from..to].iter().any(|&slot| {
                let DecorRangeSlot::Range(item) = &state.slots[slot] else {
                    unreachable!("decor: a sorted index names a freed slot")
                };
                item.start_row == row
                    && item.kind == kDecorKindVirtText
                    && item.draw_col >= -1
                    && item.start_col as isize >= v
                    && with_decor_virt(item.data.virt(), |vt| {
                        vt.pos == kVPosInline && vt.width != 0
                    })
            })
        })
    })
}

/// Moves where the next end-of-line virtual text goes.
#[inline]
pub(crate) fn set_decor_eol_col(eol_col: c_int) {
    with_state(|state| state.eol_col = eol_col);
}

/// What the last [`decor_redraw_col`] said about concealing: 0 not
/// concealed, 1 concealed, 2 the first cell of a concealed range; with the
/// replacement character and its attribute, for the first cell.
#[inline]
pub(crate) fn decor_conceal() -> (c_int, ScreenChar, c_int) {
    read_state(|state| (state.conceal, state.conceal_char, state.conceal_attr))
}

/// What the last [`decor_redraw_col`] said about spell checking: forced on,
/// forced off, or `None` for "as the window says".
#[inline]
pub(crate) fn decor_spell() -> Option<bool> {
    read_state(|state| state.spell)
}
