#![deny(unsafe_op_in_unsafe_fn)]

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
//! the mark already has. Anything bigger is `ext`: the mark then holds a
//! pointer to a chain of [`DecorVirtText`] and an index into
//! [`DECOR_ITEMS`], the one global store of `DecorSignHighlight`s that every
//! mark's chain runs through.
//!
//! # Deleting while drawing
//!
//! A decoration can be deleted by a decoration provider's Lua callback in the
//! middle of the redraw that is reading it. [`decor_free`] therefore checks
//! `running_decor_provider`, and puts anything freed at such a moment on a
//! to-free list that [`decor_check_to_be_deleted`] drains once the redraw is
//! over.

use crate::change::changed_lines_invalidate_buf;
use crate::drawscreen::{redraw_buf_line_later, redraw_buf_range_later};
use crate::extmark::extmark_set;
use crate::global_cell::GlobalCell;
use crate::grid::{schar_from_char, schar_get_first_codepoint, schar_high};
use crate::main::{decor_state, firstwin, namespace_localscope};
use crate::map::set_has_uint32_t;
use crate::marktree::key::{MT_FLAG_DECOR_EXT, MT_FLAG_DECOR_HL};
use crate::memory::{xfree, xmalloc};
use crate::r#move::changed_window_setting;
use crate::types::{
    DecorHighlightInline, DecorInline, DecorInlineData, DecorPriority, DecorRangeKind,
    DecorSignHighlight, DecorVirtText, HlMode, MTKey, MetaIndex, VirtLines, VirtText,
    VirtTextChunk, VirtTextPos, buf_T, colnr_T, linenr_T, lpos_T, uint8_t, uint16_t, uint32_t,
    virt_line,
};
use crate::winlayer::Win;
use ::core::ffi::c_int;
use ::core::{mem, ptr};

mod dict;
mod handles;
mod query;
mod signs;
mod state;

pub use self::dict::*;
pub use self::handles::*;
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
pub(crate) const DECOR_ID_INVALID: uint32_t = uint32_t::MAX;
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
    sign_name: ptr::null_mut(),
    sign_add_id: 0,
    number_hl_id: 0,
    line_hl_id: 0,
    cursorline_hl_id: 0,
    next: DECOR_ID_INVALID,
    url: ptr::null(),
};

/// "No decoration": by convention that is always the inline branch with an
/// unset highlight, never an `ext` one with empty chains.
pub const DECOR_INLINE_INIT: DecorInline = DecorInline {
    ext: false,
    data: DecorInlineData {
        hl: DECOR_HIGHLIGHT_INLINE_INIT,
    },
};

/// The virtual-text chain of `mark`, or null if it has no `ext` decoration.
///
/// Safe: the flag decides which branch of the union is read, and the answer
/// is a pointer, which dereferencing is what needs a promise.
pub(crate) fn mt_decor_virt(mark: MTKey) -> *mut DecorVirtText {
    if mark.flags as c_int & MT_FLAG_DECOR_EXT != 0 {
        // SAFETY: the flag says the union holds the `ext` branch.
        unsafe { mark.decor_data.ext.vt }
    } else {
        ptr::null_mut()
    }
}

/// Whether marks in namespace `ns_id` are visible in `wp`: a namespace that is
/// not window-local is visible everywhere, a window-local one only where it
/// was opted into.
///
#[inline]
pub fn ns_in_win(ns_id: uint32_t, mut wp: Win) -> bool {
    // SAFETY: the editor's own namespace table, live from startup to exit.
    if !unsafe { set_has_uint32_t(namespace_localscope.ptr(), ns_id) } {
        return true;
    }
    // SAFETY: a live window's `w_ns_set` is one of its own fields.
    unsafe { set_has_uint32_t(&raw mut wp.w_ns_set, ns_id) }
}

// ---------------------------------------------------------------------------
// The item store
// ---------------------------------------------------------------------------

/// Every `DecorSignHighlight` that any extmark anywhere points at, in one
/// array. A mark stores the *index* of the first item of its decoration and
/// each item the index of the next, so one decoration is a chain through this
/// store and [`DECOR_ID_INVALID`] ends it.
///
/// Entries are never removed. An index handed to a marktree entry has to stay
/// valid until that entry is deleted, so a freed item goes on a freelist
/// threaded through the same `next` field ([`DECOR_FREELIST`]) and is reused
/// in place.
static DECOR_ITEMS: GlobalCell<Vec<DecorSignHighlight>> = GlobalCell::new(Vec::new());

/// Index of the first free slot of [`DECOR_ITEMS`], or [`DECOR_ID_INVALID`].
static DECOR_FREELIST: GlobalCell<uint32_t> = GlobalCell::new(DECOR_ID_INVALID);

/// A pointer to decoration item `idx`.
///
/// A pointer and not a borrow, because callers walk a `next` chain writing
/// through it, across calls that can add another item — see [`decor_put_sh`],
/// which is what invalidates them. Panics on an index no [`decor_put_sh`]
/// handed out, where upstream would read past the array.
pub(crate) fn decor_item(idx: uint32_t) -> *mut DecorSignHighlight {
    DECOR_ITEMS.with(|items| {
        let items = &items[..];
        assert!((idx as usize) < items.len(), "decoration item out of range");
        items.as_ptr().cast_mut().wrapping_add(idx as usize)
    })
}

/// How many slots [`DECOR_ITEMS`] has handed out, free ones included.
pub(crate) fn decor_item_count() -> usize {
    DECOR_ITEMS.with(Vec::len)
}

/// Stores `item` and answers the index other code refers to it by, reusing a
/// freed slot when there is one.
///
/// Invalidates every pointer [`decor_item`] has handed out.
pub fn decor_put_sh(item: DecorSignHighlight) -> uint32_t {
    let free = DECOR_FREELIST.get();
    if free != DECOR_ID_INVALID {
        // SAFETY: a freelist index is one this store handed out.
        DECOR_FREELIST.set(unsafe { (*decor_item(free)).next });
        DECOR_ITEMS.with_mut(|items| items[free as usize] = item);
        return free;
    }
    DECOR_ITEMS.with_mut(|items| {
        items.push(item);
        (items.len() - 1) as uint32_t
    })
}

/// Heap-allocates a copy of `vt` with `next` as its tail, which is how a
/// virtual-text chain is built. Freed by [`decor_free`].
pub fn decor_put_vt(vt: DecorVirtText, next: *mut DecorVirtText) -> *mut DecorVirtText {
    // SAFETY: `xmalloc` never answers null and the size is this type's.
    unsafe {
        let alloc: *mut DecorVirtText = xmalloc(mem::size_of::<DecorVirtText>()).cast();
        *alloc = vt;
        (*alloc).next = next;
        alloc
    }
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
/// may still be referenced by the `DecorState` the redraw is walking. Drained
/// by [`decor_check_to_be_deleted`] once the redraw is over.
static TO_FREE_VIRT: GlobalCell<*mut DecorVirtText> = GlobalCell::new(ptr::null_mut());
static TO_FREE_SH: GlobalCell<uint32_t> = GlobalCell::new(DECOR_ID_INVALID);

/// Frees `decor`, or defers it when a decoration provider is running.
///
/// Deferring works by splicing the decoration's own chains onto the to-free
/// lists, which is why it costs no allocation: the last link of each chain
/// takes the old list head.
///
/// # Safety
/// `decor` must be live and must not be reachable from any mark afterwards.
pub unsafe fn decor_free(decor: DecorInline) {
    if !decor.ext {
        return;
    }
    // SAFETY: the caller's decoration.
    unsafe {
        let mut vt = decor.data.ext.vt;
        let mut idx: uint32_t = decor.data.ext.sh_idx;

        if !(*decor_state.ptr()).running_decor_provider {
            // Safe to delete right now.
            decor_free_inner(vt, idx);
            return;
        }

        while !vt.is_null() {
            if (*vt).next.is_null() {
                (*vt).next = TO_FREE_VIRT.get();
                TO_FREE_VIRT.set(decor.data.ext.vt);
                break;
            }
            vt = (*vt).next;
        }
        while idx != DECOR_ID_INVALID {
            let sh = decor_item(idx);
            if (*sh).next == DECOR_ID_INVALID {
                (*sh).next = TO_FREE_SH.get();
                TO_FREE_SH.set(decor.data.ext.sh_idx);
                break;
            }
            idx = (*sh).next;
        }
    }
}

/// Frees a virtual-text chain and returns a chain of items to the freelist.
///
/// # Safety
/// Both chains must be live and unreachable.
unsafe fn decor_free_inner(mut vt: *mut DecorVirtText, first_idx: uint32_t) {
    // SAFETY: the caller's chains.
    unsafe {
        while !vt.is_null() {
            if (*vt).flags as c_int & kVTIsLines as c_int != 0 {
                clear_virtlines(&raw mut (*vt).data.virt_lines);
            } else {
                clear_virttext(&raw mut (*vt).data.virt_text);
            }
            let tofree = vt;
            vt = (*vt).next;
            xfree(tofree.cast());
        }

        let mut idx = first_idx;
        while idx != DECOR_ID_INVALID {
            let sh = decor_item(idx);
            if (*sh).flags & kSHIsSign != 0 {
                xfree((*sh).sign_name.cast());
                (*sh).sign_name = ptr::null_mut();
            }
            (*sh).flags = 0;
            if !(*sh).url.is_null() {
                xfree((*sh).url as *mut _);
                (*sh).url = ptr::null();
            }
            // The whole chain goes on the freelist at once, head first.
            if (*sh).next == DECOR_ID_INVALID {
                (*sh).next = DECOR_FREELIST.get();
                DECOR_FREELIST.set(first_idx);
                break;
            }
            idx = (*sh).next;
        }
    }
}

/// Drains the to-free lists at the end of a redraw, and forgets the window
/// the `DecorState` was drawing.
///
/// # Safety
/// Must not be called while a decoration provider is running.
pub unsafe fn decor_check_to_be_deleted() {
    // SAFETY: the caller's precondition — restated as a debug assertion, as
    // upstream's `assert()` is — and the lists are this file's.
    unsafe {
        debug_assert!(!(*decor_state.ptr()).running_decor_provider);
        decor_free_inner(TO_FREE_VIRT.get(), TO_FREE_SH.get());
        TO_FREE_VIRT.set(ptr::null_mut());
        TO_FREE_SH.set(DECOR_ID_INVALID);
        (*decor_state.ptr()).win = ptr::null_mut();
    }
}

/// Frees the chunks of a virtual text and empties it.
///
/// # Safety
/// `text` must point to a live `VirtText` that owns its chunks.
pub unsafe fn clear_virttext(text: *mut VirtText) {
    // SAFETY: the caller's virtual text.
    unsafe {
        for i in 0..(*text).size {
            xfree((*(*text).items.add(i)).text.cast());
        }
        xfree((*text).items.cast());
        *text = VirtText {
            size: 0,
            capacity: 0,
            items: ptr::null_mut::<VirtTextChunk>(),
        };
    }
}

/// [`clear_virttext`] for a block of virtual lines.
///
/// # Safety
/// `lines` must point to a live `VirtLines` that owns its lines.
pub unsafe fn clear_virtlines(lines: *mut VirtLines) {
    // SAFETY: the caller's virtual lines.
    unsafe {
        for i in 0..(*lines).size {
            clear_virttext(&raw mut (*(*lines).items.add(i)).line);
        }
        xfree((*lines).items.cast());
        *lines = VirtLines {
            size: 0,
            capacity: 0,
            items: ptr::null_mut::<virt_line>(),
        };
    }
}

/// Replaces any sign or conceal character that the glyph cache no longer
/// holds — called when the cache is rebuilt, since a `schar_T` is an index
/// into it once the character is longer than four bytes.
///
/// # Safety
/// Reaches the glyph cache; main thread only.
pub unsafe fn decor_check_invalid_glyphs() {
    for i in 0..decor_item_count() {
        let it = decor_item(i as uint32_t);
        // SAFETY: an index below `decor_item_count()` is a live item.
        unsafe {
            let width = if (*it).flags & kSHIsSign != 0 {
                SIGN_WIDTH
            } else if (*it).flags & kSHConceal != 0 {
                1
            } else {
                0
            };
            for j in 0..width as usize {
                if schar_high((*it).text[j]) {
                    (*it).text[j] = schar_from_char(schar_get_first_codepoint((*it).text[j]));
                }
            }
        }
    }
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
///
/// # Safety
/// `buf` must be live and the positions must be inside it.
pub unsafe fn bufhl_add_hl_pos_offset(
    buf: *mut buf_T,
    src_id: c_int,
    hl_id: c_int,
    pos_start: lpos_T,
    pos_end: lpos_T,
    offset: colnr_T,
) {
    let mut decor = DECOR_INLINE_INIT;
    decor.data.hl.hl_id = hl_id;

    // SAFETY: the caller's buffer.
    unsafe {
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

            extmark_set(
                buf,
                src_id as uint32_t,
                ptr::null_mut(),
                lnum as c_int - 1,
                hl_start,
                lnum as c_int - 1 + end_off,
                hl_end,
                decor,
                MT_FLAG_DECOR_HL as uint16_t,
                true,
                false,
                true,
                false,
                ptr::null_mut(),
            );
        }
    }
}

/// Marks the screen lines `decor` affects as needing a redraw.
///
/// # Safety
/// `buf` must be live and `decor` must be its mark's decoration.
pub unsafe fn decor_redraw(
    buf: *mut buf_T,
    row1: c_int,
    row2: c_int,
    col1: c_int,
    decor: DecorInline,
) {
    // SAFETY: the caller's buffer and decoration.
    unsafe {
        if !decor.ext {
            decor_redraw_sh(buf, row1, row2, decor_sh_from_inline(decor.data.hl));
            return;
        }

        let mut vt = decor.data.ext.vt;
        while !vt.is_null() {
            let is_lines = (*vt).flags & kVTIsLines != 0;
            let below = is_lines && (*vt).flags & kVTLinesAbove == 0;
            let vt_lnum = row1 as linenr_T + 1 + linenr_T::from(below);
            redraw_buf_line_later(buf, vt_lnum, true);
            // Virtual lines and inline virtual text change how much room the
            // line takes, so the cached line sizes have to go as well.
            if is_lines || (*vt).pos == kVPosInline {
                let vt_col: colnr_T = if is_lines { 0 } else { col1 };
                changed_lines_invalidate_buf(buf, vt_lnum, vt_col, vt_lnum + 1, 0);
            }
            vt = (*vt).next;
        }

        let mut idx: uint32_t = decor.data.ext.sh_idx;
        while idx != DECOR_ID_INVALID {
            let sh = decor_item(idx);
            decor_redraw_sh(buf, row1, row2, *sh);
            idx = (*sh).next;
        }
    }
}

/// [`decor_redraw`] for one sign/highlight item.
///
/// # Safety
/// `buf` must be live.
pub unsafe fn decor_redraw_sh(buf: *mut buf_T, row1: c_int, row2: c_int, sh: DecorSignHighlight) {
    // SAFETY: the caller's buffer and the editor's window list.
    unsafe {
        let paints = sh.flags & (kSHIsSign | kSHSpellOn | kSHSpellOff | kSHConceal) != 0;
        if (sh.hl_id != 0 || !sh.url.is_null() || paints) && row2 >= row1 {
            redraw_buf_range_later(buf, row1 as linenr_T + 1, row2 as linenr_T + 1);
        }

        if sh.flags & kSHConcealLines != 0 {
            // The current tabpage's window list is in the globals, which is
            // what `FOR_ALL_WINDOWS_IN_TAB(wp, curtab)` expands to.
            // TODO(luukvbaal): redraw only unconcealed lines, and scroll
            // lines below it up or down. Also when opening/closing a fold.
            let mut wp = firstwin.get();
            while !wp.is_null() {
                if (*wp).w_buffer == buf {
                    changed_window_setting(wp);
                }
                wp = (*wp).w_next;
            }
        }

        if sh.flags & kSHUIWatched != 0 {
            redraw_buf_line_later(buf, row1 as linenr_T + 1, false);
        }
    }
}

/// Accounts for a decoration that has just been placed on rows `row..=row2`.
///
/// # Safety
/// `buf` must be live and `decor` must be its mark's decoration.
pub unsafe fn buf_put_decor(buf: *mut buf_T, decor: DecorInline, row: c_int, mut row2: c_int) {
    // SAFETY: the caller's buffer and decoration.
    unsafe {
        if !decor.ext || row as linenr_T >= (*buf).b_ml.ml_line_count {
            return;
        }
        row2 = ((*buf).b_ml.ml_line_count - 1).min(row2 as linenr_T) as c_int;
        let mut idx: uint32_t = decor.data.ext.sh_idx;
        while idx != DECOR_ID_INVALID {
            let sh = decor_item(idx);
            buf_put_decor_sh(buf, sh, row, row2);
            idx = (*sh).next;
        }
    }
}

/// Undoes [`buf_put_decor`] and schedules the redraw, freeing the decoration
/// too when `free` says the mark is going away with it.
///
/// # Safety
/// `buf` must be live and `decor` must be its mark's decoration.
pub unsafe fn buf_decor_remove(
    buf: *mut buf_T,
    row1: c_int,
    mut row2: c_int,
    col1: c_int,
    decor: DecorInline,
    free: bool,
) {
    // SAFETY: the caller's buffer and decoration.
    unsafe {
        decor_redraw(buf, row1, row2, col1, decor);
        if decor.ext && (row1 as linenr_T) < (*buf).b_ml.ml_line_count {
            row2 = ((*buf).b_ml.ml_line_count - 1).min(row2 as linenr_T) as c_int;
            let mut idx: uint32_t = decor.data.ext.sh_idx;
            while idx != DECOR_ID_INVALID {
                let sh = decor_item(idx);
                buf_remove_decor_sh(buf, row1, row2, sh);
                idx = (*sh).next;
            }
        }
        if free {
            decor_free(decor);
        }
    }
}
