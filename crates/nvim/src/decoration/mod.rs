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

//! Extmark decorations: what a mark makes the screen do.
//!
//! A decoration is whatever an extmark carries beyond its position — a
//! highlight over the text, virtual text beside or instead of it, whole
//! virtual lines, a sign, a conceal, a spelling override, a URL. This file
//! owns the storage for all of that and the "something changed, redraw it"
//! side; the four children do the work:
//!
//! * [`state`] — `DecorState`, the per-window iteration the drawing code
//!   walks a window with.
//! * [`signs`] — signs from extmarks, and the sign-column width histogram.
//! * [`query`] — the row questions the layout code asks without drawing.
//! * [`dict`] — a decoration as an API dictionary.
//!
//! # How a decoration is stored
//!
//! Small ones live *inline* in the marktree key: a `DecorHighlightInline` is
//! a highlight id, a priority and a conceal character, and fits in the space
//! the mark already has. Anything bigger is `ext`: the mark then holds the
//! heads of two chains in the [`store`]s — one of [`DecorSignHighlight`]
//! items, one of [`DecorVirtText`] links — which own everything the
//! decoration holds.
//!
//! # Deleting while drawing
//!
//! A decoration can be deleted by a decoration provider's Lua callback in the
//! middle of the redraw that is reading it. [`decor_free`] therefore asks
//! [`decor_provider_running`], and puts anything freed at such a moment on a
//! to-free chain that [`decor_check_to_be_deleted`] frees once the redraw is
//! over.

use crate::api::extmark::ns_is_local;
use crate::buffer::buf_meta_total;
use crate::change::changed_lines_invalidate_buf;
use crate::decoration_provider::decor_provider_running;
use crate::drawscreen::{redraw_buf_line_later, redraw_buf_range_later};
use crate::extmark::extmark_set;
use crate::global_cell::GlobalCell;
use crate::grid::{schar_from_char, schar_get_first_codepoint, schar_high};
use crate::marktree::key::MtFlags;
use crate::r#move::changed_window_setting;
use crate::types::{
    ColNr, DecorHighlightInline, DecorInline, DecorPriority, DecorRangeKind, DecorSignHighlight,
    DecorVirtText, HlMode, LPos, LineNr, MTKey, MetaIndex, VirtTextPos, uint8_t, uint16_t,
    uint32_t,
};
use crate::winlayer::{self, Buf, Win};
use core::ffi::c_int;

mod dict;
mod query;
mod signs;
mod state;
mod store;

use self::store::Store;

pub use self::dict::*;
pub use self::query::*;
pub use self::signs::*;
pub use self::state::*;

// ---------------------------------------------------------------------------
// The shapes a decoration is made of
// ---------------------------------------------------------------------------

/// `kSH*`: what a `DecorSignHighlight` (or its inline form) actually does.
/// Several can be set at once, which is why one item can be both a highlight
/// and a spelling override.
pub(crate) const kSHIsSign: uint16_t = 1;
/// Colour the rest of the screen line past the end of the text too.
pub(crate) const kSHHlEol: uint16_t = 2;
/// The position is reported to the UI rather than drawn (`ui_watched`).
pub(crate) const kSHUIWatched: uint16_t = 4;
/// A `ui_watched` mark that reports an overlay position, not an eol one.
pub(crate) const kSHUIWatchedOverlay: uint16_t = 8;
/// Force spell checking on, and off, over the range.
pub(crate) const kSHSpellOn: uint16_t = 16;
pub(crate) const kSHSpellOff: uint16_t = 32;
/// Replace the range with `text[0]` at 'conceallevel' 2 or more.
pub(crate) const kSHConceal: uint16_t = 64;
/// Hide the whole line.
pub(crate) const kSHConcealLines: uint16_t = 128;

/// `kVT*`: flags of a `DecorVirtText`.
/// The item is a block of whole lines, not text alongside the line.
pub(crate) const kVTIsLines: uint8_t = 1;
/// Do not draw the virtual text while the line is selected.
pub(crate) const kVTHide: uint8_t = 2;
/// Draw the virtual lines above their own line rather than below it.
pub(crate) const kVTLinesAbove: uint8_t = 4;
/// Repeat the virtual text on every screen line a wrapped line takes.
pub(crate) const kVTRepeatLinebreak: uint8_t = 8;

/// `kVL*`: flags of one `virt_line` inside a virtual-lines block.
/// Start at the left window edge, ignoring the number column and friends.
pub(crate) const kVLLeftcol: c_int = 1;
/// May scroll horizontally with `nowrap`.
pub(crate) const kVLScroll: c_int = 2;

/// `VirtTextPos`: where virtual text goes relative to the line. Keep in sync
/// with `dict::VIRT_TEXT_POS_STR`.
pub(crate) const kVPosEndOfLine: VirtTextPos = 0;
pub(crate) const kVPosEndOfLineRightAlign: VirtTextPos = 1;
pub(crate) const kVPosInline: VirtTextPos = 2;
pub(crate) const kVPosOverlay: VirtTextPos = 3;
pub(crate) const kVPosRightAlign: VirtTextPos = 4;
pub(crate) const kVPosWinCol: VirtTextPos = 5;

/// `HlMode`: how a virtual text's highlight combines with what is under it.
pub(crate) const kHlModeUnknown: HlMode = 0;
/// Replace whatever is under the text.
pub(crate) const kHlModeReplace: HlMode = 1;
/// Combine with it, as `:highlight` links do.
pub(crate) const kHlModeCombine: HlMode = 2;
/// Blend the two colours, as `'winblend'` does.
pub(crate) const kHlModeBlend: HlMode = 3;

/// `DecorRangeKind`: which arm of a `DecorRange`'s union is live.
pub(crate) const kDecorKindHighlight: DecorRangeKind = 0;
pub(crate) const kDecorKindVirtText: DecorRangeKind = 2;
pub(crate) const kDecorKindVirtLines: DecorRangeKind = 3;
/// A position reported to the UI rather than anything drawn.
pub(crate) const kDecorKindUIWatched: DecorRangeKind = 4;

/// `kMTMeta*`: the marktree's per-node counts, which is what lets a walk skip
/// a whole subtree that has no mark of the kind it is looking for.
pub(crate) const kMTMetaInline: MetaIndex = 0;
pub(crate) const kMTMetaLines: MetaIndex = 1;
pub(crate) const kMTMetaSignHL: MetaIndex = 2;
pub(crate) const kMTMetaSignText: MetaIndex = 3;
pub(crate) const kMTMetaConcealLines: MetaIndex = 4;
/// How many kinds there are — the length of a node's count array.
pub(crate) const kMTMetaCount: MetaIndex = 5;

/// Cells a sign takes in the sign column.
pub(crate) const SIGN_WIDTH: c_int = 2;
/// `'signcolumn'` value meaning "put signs in the number column".
pub(crate) const SCL_NUM: c_int = -2;

/// The index that ends a decoration's chain of items.
pub(crate) const DECOR_ID_INVALID: uint32_t = store::NO_ENTRY;
/// The priority a decoration gets when its creator did not name one.
const DECOR_PRIORITY_BASE: DecorPriority = 0x1000;

/// An unset inline highlight — no flags, no group, base priority.
pub(crate) const DECOR_HIGHLIGHT_INLINE_INIT: DecorHighlightInline = DecorHighlightInline {
    flags: 0,
    priority: DECOR_PRIORITY_BASE,
    hl_id: 0,
    conceal_char: 0,
};

/// An unset sign/highlight item.
pub(crate) const DECOR_SIGN_HIGHLIGHT_INIT: DecorSignHighlight = DecorSignHighlight {
    flags: 0,
    priority: DECOR_PRIORITY_BASE,
    hl_id: 0,
    text: [0; 2],
    sign_name: None,
    sign_add_id: 0,
    number_hl_id: 0,
    line_hl_id: 0,
    cursorline_hl_id: 0,
    next: DECOR_ID_INVALID,
    url: None,
};

/// "No decoration": by convention that is always the inline branch with an
/// unset highlight, never an `ext` one with empty chains.
pub const DECOR_INLINE_INIT: DecorInline = DecorInline::Hl(DECOR_HIGHLIGHT_INLINE_INIT);

/// An inline decoration that is just highlight `hl_id`.
pub(crate) const fn decor_inline_hl(hl_id: c_int) -> DecorInline {
    DecorInline::Hl(DecorHighlightInline {
        hl_id,
        ..DECOR_HIGHLIGHT_INLINE_INIT
    })
}

/// The head of `mark`'s virtual-text chain, [`DECOR_ID_INVALID`] when it has
/// no `ext` decoration.
pub(crate) fn mt_decor_virt(mark: MTKey) -> uint32_t {
    match mark.decor_data {
        DecorInline::Ext(ext) if mark.flags.has(MtFlags::DECOR_EXT) => ext.vt_idx,
        _ => DECOR_ID_INVALID,
    }
}

impl Buf {
    /// How many marks of `kind` the whole buffer holds.
    ///
    /// The cheap test every per-row question starts with: a buffer with none
    /// of the kind in question never touches the marktree.
    #[inline(always)]
    pub fn meta_total(self, kind: MetaIndex) -> uint32_t {
        buf_meta_total(self, kind)
    }
}

/// Whether marks in namespace `ns_id` are visible in `window`: a namespace that is
/// not window-local is visible everywhere, a window-local one only where it
/// was opted into.
///
#[inline]
pub fn ns_in_win(ns_id: uint32_t, window: Win) -> bool {
    if !ns_is_local(ns_id) {
        return true;
    }
    window.w_ns_set.contains(&ns_id)
}

// ---------------------------------------------------------------------------
// The stores
// ---------------------------------------------------------------------------

/// Every `DecorSignHighlight` that any extmark anywhere names, chained by
/// index; see [`store`].
static DECOR_ITEMS: GlobalCell<Store<DecorSignHighlight>> = GlobalCell::new(Store::new());

/// Every link of every virtual-text chain, likewise.
static DECOR_VIRT: GlobalCell<Store<DecorVirtText>> = GlobalCell::new(Store::new());

/// Runs `f` on sign/highlight item `idx`. Panics on an index the store never
/// handed out. `f` must not reach the store again.
pub(crate) fn with_decor_item<R>(idx: uint32_t, f: impl FnOnce(&DecorSignHighlight) -> R) -> R {
    DECOR_ITEMS.with(|items| f(items.get(idx)))
}

/// [`with_decor_item`], to write to.
pub(crate) fn with_decor_item_mut<R>(
    idx: uint32_t,
    f: impl FnOnce(&mut DecorSignHighlight) -> R,
) -> R {
    DECOR_ITEMS.with_mut(|items| f(items.get_mut(idx)))
}

/// Runs `f` on every slot of the item store, free ones included, in index
/// order. `f` must not reach the store again.
pub(crate) fn for_each_decor_item(mut f: impl FnMut(&mut DecorSignHighlight)) {
    DECOR_ITEMS.with_mut(|items| items.entries_mut().for_each(&mut f));
}

/// Runs `f` on virtual-text link `idx`. Panics on an index the store never
/// handed out. `f` must not reach the store again.
pub(crate) fn with_decor_virt<R>(idx: uint32_t, f: impl FnOnce(&DecorVirtText) -> R) -> R {
    DECOR_VIRT.with(|links| f(links.get(idx)))
}

/// [`with_decor_virt`], to write to.
pub(crate) fn with_decor_virt_mut<R>(idx: uint32_t, f: impl FnOnce(&mut DecorVirtText) -> R) -> R {
    DECOR_VIRT.with_mut(|links| f(links.get_mut(idx)))
}

/// The indices of chain `head`, read one link at a time: nothing is
/// borrowed between steps, so the caller may do anything in between but
/// free the chain.
fn chain_indices(head: uint32_t, next: fn(uint32_t) -> uint32_t) -> impl Iterator<Item = uint32_t> {
    let first = (head != DECOR_ID_INVALID).then_some(head);
    core::iter::successors(first, move |&idx| {
        let after = next(idx);
        (after != DECOR_ID_INVALID).then_some(after)
    })
}

/// Every sign/highlight item of `decor`, in chain order. Empty for an
/// inline decoration, which has no store entry at all.
pub(crate) fn sh_chain(decor: DecorInline) -> impl Iterator<Item = uint32_t> {
    let head = match decor {
        DecorInline::Ext(ext) => ext.sh_idx,
        DecorInline::Hl(_) => DECOR_ID_INVALID,
    };
    chain_indices(head, |idx| with_decor_item(idx, |sh| sh.next))
}

/// Every virtual-text link of chain `head`, in chain order.
pub(crate) fn virt_chain(head: uint32_t) -> impl Iterator<Item = uint32_t> {
    chain_indices(head, |idx| with_decor_virt(idx, |vt| vt.next))
}

/// Every virtual text `mark` carries, in chain order.
pub(crate) fn mark_virt_chain(mark: MTKey) -> impl Iterator<Item = uint32_t> {
    virt_chain(mt_decor_virt(mark))
}

/// Stores `item` and answers the index other code refers to it by, reusing a
/// freed slot when there is one.
pub fn decor_put_sh(item: DecorSignHighlight) -> uint32_t {
    DECOR_ITEMS.with_mut(|items| items.put(item))
}

/// Stores `vt` with `next` as its tail, which is how a virtual-text chain is
/// built, and answers its index. Freed by [`decor_free`].
pub fn decor_put_vt(mut vt: DecorVirtText, next: uint32_t) -> uint32_t {
    vt.next = next;
    DECOR_VIRT.with_mut(|links| links.put(vt))
}

/// Frees the one item `idx` an owned range took over.
pub(crate) fn decor_release_sh(idx: uint32_t) {
    DECOR_ITEMS.with_mut(|items| items.release(idx));
}

/// Frees the one virtual-text link `idx` an owned range took over.
pub(crate) fn decor_release_vt(idx: uint32_t) {
    DECOR_VIRT.with_mut(|links| links.release(idx));
}

/// The stored form of an inline highlight — the same thing with the fields a
/// full item has and it does not.
///
/// A sign is never inline, so an inline item claiming to be one is a bug —
/// asserted in a debug build only, as upstream's `assert()` is
/// (`v0.12.4:src/nvim/decoration.c:166`).
/// TODO(bfredl): eventually simple signs will be inlinable as well.
pub fn decor_sh_from_inline(item: DecorHighlightInline) -> DecorSignHighlight {
    debug_assert!(item.flags & kSHIsSign == 0);
    DecorSignHighlight {
        flags: item.flags,
        priority: item.priority,
        hl_id: item.hl_id,
        text: [item.conceal_char, 0],
        next: DECOR_ID_INVALID,
        ..DECOR_SIGN_HIGHLIGHT_INIT
    }
}

// ---------------------------------------------------------------------------
// Freeing
// ---------------------------------------------------------------------------

/// Decorations a callback asked to delete in the middle of a redraw, which
/// may still be named by the `DecorState` the redraw is walking: their
/// sign/highlight chains and their virtual-text chains, each spliced into
/// one chain. Freed by [`decor_check_to_be_deleted`] once the redraw is over.
static TO_FREE_SH: GlobalCell<uint32_t> = GlobalCell::new(DECOR_ID_INVALID);
static TO_FREE_VT: GlobalCell<uint32_t> = GlobalCell::new(DECOR_ID_INVALID);

/// Frees `decor`, or defers it when a decoration provider is running.
///
/// Its entries must not be reachable from any mark afterwards; an inline
/// decoration owns nothing.
pub fn decor_free(decor: DecorInline) {
    let DecorInline::Ext(ext) = decor else {
        return;
    };
    if !decor_provider_running() {
        // Safe to delete right now.
        DECOR_VIRT.with_mut(|links| links.release(ext.vt_idx));
        DECOR_ITEMS.with_mut(|items| items.release(ext.sh_idx));
        return;
    }
    let vt = DECOR_VIRT.with_mut(|links| links.splice(ext.vt_idx, TO_FREE_VT.get()));
    TO_FREE_VT.set(vt);
    let sh = DECOR_ITEMS.with_mut(|items| items.splice(ext.sh_idx, TO_FREE_SH.get()));
    TO_FREE_SH.set(sh);
}

/// Frees what [`decor_free`] deferred, at the end of a redraw, and forgets
/// the window the `DecorState` was drawing.
///
/// Must not be called while a decoration provider is running.
pub fn decor_check_to_be_deleted() {
    // The caller's precondition, restated as upstream's `assert()` does.
    debug_assert!(!decor_provider_running());
    let vt = TO_FREE_VT.replace(DECOR_ID_INVALID);
    DECOR_VIRT.with_mut(|links| links.release(vt));
    let sh = TO_FREE_SH.replace(DECOR_ID_INVALID);
    DECOR_ITEMS.with_mut(|items| items.release(sh));
    decor_forget_window();
}

/// Replaces any sign or conceal character that the glyph cache no longer
/// holds — called when the cache is rebuilt, since a `ScreenChar` is an index
/// into it once the character is longer than four bytes.
pub fn decor_check_invalid_glyphs() {
    for_each_decor_item(|it| {
        let width = if it.flags & kSHIsSign != 0 {
            SIGN_WIDTH as usize
        } else if it.flags & kSHConceal != 0 {
            1
        } else {
            0
        };
        for j in 0..width {
            if schar_high(it.text[j]) {
                it.text[j] = schar_from_char(schar_get_first_codepoint(it.text[j]));
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Telling the screen something changed
// ---------------------------------------------------------------------------

/// Adds highlighting between two positions, one extmark per line.
///
/// `offset` shifts the whole thing right, which is what `:substitute`'s
/// preview needs: it highlights a replacement that has not been made yet, in
/// a line the command line has already indented.
///
/// TODO(bfredl): make decoration powerful enough that this can be done with a
/// single ephemeral decoration.
pub fn bufhl_add_hl_pos_offset(
    buffer: Buf,
    src_id: c_int,
    hl_id: c_int,
    pos_start: LPos,
    pos_end: LPos,
    offset: ColNr,
) {
    let decor = decor_inline_hl(hl_id);

    // TODO(bfredl): if decoration had blocky mode, we could avoid this loop
    for lnum in pos_start.lnum..=pos_end.lnum {
        let first = lnum == pos_start.lnum;
        let last = lnum == pos_end.lnum;
        // A middle or last line starts one column left of `offset`. This
        // is quite ad-hoc, but the space between the number column and
        // the highlighted text is what shows that the `\n` is part of the
        // substituted text.
        let hl_start = if first {
            pos_start.col + offset
        } else {
            (offset - 1).max(0)
        };
        // Anything but the last line runs to the end, which is spelled as
        // "column 0 of the next line".
        let (end_off, hl_end) = if last {
            (0, pos_end.col + offset)
        } else {
            (1, 0)
        };

        let ns = u32::try_from(src_id).expect("a namespace id fits its own handle type");
        let (row, end_row) = (lnum as c_int - 1, lnum as c_int - 1 + end_off);
        let flags = MtFlags::DECOR_HL;
        extmark_set(
            buffer, ns, None, row, hl_start, end_row, hl_end, decor, flags, true, false, true,
            false,
        );
    }
}

/// What [`decor_redraw_sh`] needs to know about one sign/highlight item.
#[derive(Copy, Clone)]
pub(crate) struct ShRedraw {
    flags: uint16_t,
    hl_id: c_int,
    has_url: bool,
}

impl ShRedraw {
    fn of(sh: &DecorSignHighlight) -> Self {
        Self {
            flags: sh.flags,
            hl_id: sh.hl_id,
            has_url: sh.url.is_some(),
        }
    }
}

/// Marks the screen lines `decor` affects as needing a redraw.
pub fn decor_redraw(buffer: Buf, row1: c_int, row2: c_int, col1: c_int, decor: DecorInline) {
    let ext = match decor {
        DecorInline::Hl(hl) => {
            let sh = ShRedraw::of(&decor_sh_from_inline(hl));
            decor_redraw_sh(buffer, row1, row2, sh);
            return;
        }
        DecorInline::Ext(ext) => ext,
    };

    for idx in virt_chain(ext.vt_idx) {
        let (flags, pos) = with_decor_virt(idx, |vt| (vt.flags, vt.pos));
        let is_lines = flags & kVTIsLines != 0;
        let below = is_lines && flags & kVTLinesAbove == 0;
        let vt_lnum = row1 as LineNr + 1 + LineNr::from(below);
        redraw_buf_line_later(buffer, vt_lnum, true);
        // Virtual lines and inline virtual text change how much room the
        // line takes, so the cached line sizes have to go as well.
        if is_lines || pos == kVPosInline {
            let vt_col: ColNr = if is_lines { 0 } else { col1 };
            changed_lines_invalidate_buf(buffer, vt_lnum, vt_col, vt_lnum + 1, 0);
        }
    }

    for idx in sh_chain(decor) {
        let sh = with_decor_item(idx, ShRedraw::of);
        decor_redraw_sh(buffer, row1, row2, sh);
    }
}

/// [`decor_redraw`] for one sign/highlight item.
pub(crate) fn decor_redraw_sh(buffer: Buf, row1: c_int, row2: c_int, sh: ShRedraw) {
    let paints = sh.flags & (kSHIsSign | kSHSpellOn | kSHSpellOff | kSHConceal) != 0;
    if (sh.hl_id != 0 || sh.has_url || paints) && row2 >= row1 {
        redraw_buf_range_later(buffer, row1 as LineNr + 1, row2 as LineNr + 1);
    }

    if sh.flags & kSHConcealLines != 0 {
        // TODO(luukvbaal): redraw only unconcealed lines, and scroll
        // lines below it up or down. Also when opening/closing a fold.
        for wp in winlayer::windows() {
            if wp.w_buffer == buffer {
                changed_window_setting(wp);
            }
        }
    }

    if sh.flags & kSHUIWatched != 0 {
        redraw_buf_line_later(buffer, row1 as LineNr + 1, false);
    }
}

/// Accounts for a decoration that has just been placed on rows `row..=row2`.
pub fn buf_put_decor(buffer: Buf, decor: DecorInline, row: c_int, mut row2: c_int) {
    if !decor.is_ext() || row as LineNr >= buffer.b_ml.ml_line_count {
        return;
    }
    row2 = (buffer.b_ml.ml_line_count - 1).min(row2 as LineNr) as c_int;
    for idx in sh_chain(decor) {
        buf_put_decor_sh(buffer, idx, row, row2);
    }
}

/// Undoes [`buf_put_decor`] and schedules the redraw, freeing the decoration
/// too when `free` says the mark is going away with it.
pub fn buf_decor_remove(
    buffer: Buf,
    row1: c_int,
    mut row2: c_int,
    col1: c_int,
    decor: DecorInline,
    free: bool,
) {
    decor_redraw(buffer, row1, row2, col1, decor);
    if decor.is_ext() && (row1 as LineNr) < buffer.b_ml.ml_line_count {
        row2 = (buffer.b_ml.ml_line_count - 1).min(row2 as LineNr) as c_int;
        for idx in sh_chain(decor) {
            buf_remove_decor_sh(buffer, row1, row2, idx);
        }
    }
    if free {
        decor_free(decor);
    }
}
