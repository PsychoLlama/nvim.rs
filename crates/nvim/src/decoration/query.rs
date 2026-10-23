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

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::{kVTIsLines, kVTLinesAbove, mark_virt_chain, ns_in_win, with_decor_virt};
use crate::decoration::{kMTMetaConcealLines, kMTMetaLines};
use crate::decoration_provider::decor_providers_invoke_conceal_line;
use crate::drawscreen::conceal_cursor_line;
use crate::highlight::hl_combine_attr;
use crate::highlight_group::syn_id2attr;
use crate::marktree::cursor::Cursor;
use crate::marktree::key::{kMTFilterSelect, mt_conceal_lines, mt_invalid};
use crate::marktree::meta::MetaCount;
use crate::memory::XString;
use crate::narrow::len_as_int;
use crate::types::{LineNr, MarkTreeIter, OptInt, VirtTextChunk, size_t, uint32_t, uint64_t};
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_int};

/// Marktree filters, indexed by `MetaIndex`: `kMTMetaLines` is 1 and
/// `kMTMetaConcealLines` 4.
static LINES_FILTER: MetaCount = [0, kMTFilterSelect, 0, 0, 0];
static CONCEAL_FILTER: MetaCount = [0, 0, 0, 0, kMTFilterSelect];

impl Win {
    /// Whether `'concealcursor'` says the cursor line conceals too.
    fn conceals_cursor_line(self) -> bool {
        conceal_cursor_line(self)
    }

    /// Asks every decoration provider to place `row`'s conceal marks, and
    /// answers whether one of them says the row is concealed.
    ///
    /// Runs Lua, which can place and delete marks.
    fn providers_conceal_line(self, row: c_int) -> bool {
        decor_providers_invoke_conceal_line(self, row)
    }
}

/// The texts of every chunk of `vt`, joined: how a reader that wants flat
/// text takes a chunked answer (a list 'foldtext' asked of by
/// `foldtextresult()`).
pub(crate) fn virt_text_into_string(vt: &[VirtTextChunk]) -> XString {
    let mut text = XString::new();
    let (mut at, mut attr) = (0, 0);
    while let Some(chunk) = next_virt_text_chunk(vt, &mut at, &mut attr) {
        text.push_bytes(chunk.to_bytes());
    }
    text
}

/// The text of the next chunk of `vt`, or `None` when there are no more.
///
/// `pos` is the walk's position and is advanced past the chunk answered;
/// `attr` accumulates the highlights of every chunk passed, including the
/// empty ones that are skipped over. A chunk with `hl_id` of 0 still lifts
/// `attr` from -1 ("no highlight given") to 0, which is how a caller tells
/// "explicitly unhighlighted" from "nothing said".
///
pub fn next_virt_text_chunk<'a>(
    chunks: &'a [VirtTextChunk],
    pos: &mut size_t,
    attr: &mut c_int,
) -> Option<&'a CStr> {
    while let Some(chunk) = chunks.get(*pos) {
        if chunk.hl_id >= 0 {
            *attr = (*attr).max(0);
            if chunk.hl_id > 0 {
                *attr = hl_combine_attr(*attr, syn_id2attr(chunk.hl_id));
            }
        }
        *pos += 1;
        if let Some(text) = &chunk.text {
            return Some(text);
        }
    }
    None
}

/// The first virtual *text* on `row` — skipping virtual *lines*, which are a
/// different thing on the same chain — from namespace `ns_id`, or from any
/// namespace when that is 0: its index in the virtual-text store, `None` if
/// the row has none.
pub fn decor_find_virttext(buffer: Buf, row: c_int, ns_id: uint64_t) -> Option<uint32_t> {
    let mut itr = MarkTreeIter::default();
    let mut walk = Cursor::in_buffer(buffer, &mut itr);
    walk.seek(row, 0);
    loop {
        let mark = walk.current();
        if mark.pos.row < 0 || mark.pos.row > row {
            return None;
        }
        if !mt_invalid(mark)
            && (ns_id == 0 || ns_id == uint64_t::from(mark.ns))
            && let Some(vt) =
                mark_virt_chain(mark).find(|&vt| with_decor_virt(vt, |l| l.flags & kVTIsLines == 0))
        {
            return Some(vt);
        }
        walk.step();
    }
}

/// Whether `row` of `window` is hidden entirely by a `conceal_lines` decoration.
///
/// `check_cursor` asks for the answer the row would get if it were not the
/// cursor line: the cursor line is normally exempt (unless `'concealcursor'`
/// says otherwise), but some callers still need to know.
///
/// Providers are asked last, and only when no mark already answered — their
/// `_on_conceal_line` callback may place marks, which is what the answer of
/// [`decor_providers_invoke_conceal_line`] reports.
pub fn decor_conceal_line(window: Win, row: c_int, check_cursor: bool) -> bool {
    if row < 0
        || window.w_onebuf_opt.wo_cole < 2 as OptInt
        || (!check_cursor
            && window.is_current()
            && row as LineNr + 1 == window.w_cursor.lnum
            && !window.conceals_cursor_line())
    {
        return false;
    }

    // No need to scan the marktree if there are no conceal_line marks.
    let buf = window.buffer();
    if buf.meta_total(kMTMetaConcealLines) == 0 {
        return window.providers_conceal_line(row);
    }

    let mut itr = MarkTreeIter::default();
    let mut walk = Cursor::in_buffer(buf, &mut itr);

    walk.seek_overlap(row, 0);
    while let Some(pair) = walk.step_overlap() {
        if mt_conceal_lines(pair.start) && ns_in_win(pair.start.ns, window) {
            return true;
        }
    }

    walk.step_out_filter(&CONCEAL_FILTER);
    while !walk.is_empty() {
        let mark = walk.current();
        if mark.pos.row > row {
            break;
        }
        if mt_conceal_lines(mark) && ns_in_win(mark.ns, window) {
            return true;
        }
        walk.step_filter(row + 1, 0, &CONCEAL_FILTER);
    }

    window.providers_conceal_line(row)
}

/// Whether `window` may have folded or concealed lines at all — the cheap test
/// that lets the layout code skip the per-row questions above.
pub fn win_lines_concealed(window: Win) -> bool {
    window.has_any_folding() || window.w_onebuf_opt.wo_cole >= 2 as OptInt
}

/// How many virtual lines fall in the window rows `start_row..end_row`.
///
/// A virtual-lines block is drawn above or below the line its mark sits on,
/// so a mark one row before `start_row` can still contribute. `lines`, when
/// given, receives where each virtual line is ([`VirtLineRef`]); `num_below` counts
/// only the ones drawn *below* their mark; `apply_folds` drops the blocks
/// whose own line is folded away or concealed.
pub(crate) fn decor_virt_lines(
    window: Win,
    start_row: c_int,
    end_row: c_int,
    mut num_below: Option<&mut c_int>,
    mut lines: Option<&mut Vec<VirtLineRef>>,
    apply_folds: bool,
) -> c_int {
    let wp = window;
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
            for vt in mark_virt_chain(mark) {
                let (flags, count) = with_decor_virt(vt, |link| {
                    let count = if link.flags & kVTIsLines != 0 {
                        link.data.lines().len()
                    } else {
                        0
                    };
                    (link.flags, count)
                });
                if flags & kVTIsLines == 0 {
                    continue;
                }
                let above = flags & kVTLinesAbove != 0;
                let mrow = mark.pos.row;
                let draw_row = mrow + c_int::from(!above);
                // The fold and conceal tests stay inside the `&&` chain:
                // `decor_conceal_line` runs the providers' Lua, so asking it
                // for a row that is out of range would be a new side effect,
                // not just extra work.
                if draw_row >= start_row
                    && draw_row < end_row
                    && (!apply_folds
                        || !(wp.fold_span(mrow as LineNr + 1).0 || wp.conceal_line(mrow, false)))
                {
                    virt_lines += len_as_int(count);
                    if let Some(lines) = lines.as_deref_mut() {
                        let link = vt;
                        lines.extend((0..count).map(|line| VirtLineRef {
                            link,
                            line: uint32_t::try_from(line).expect("a block's lines fit a u32"),
                        }));
                    }
                    if let (false, Some(num_below)) = (above, num_below.as_deref_mut()) {
                        *num_below += len_as_int(count);
                    }
                }
            }
        }

        if !walk.step_filter(end_row, 0, &LINES_FILTER) {
            return virt_lines;
        }
    }
}

impl Win {
    /// [`decor_conceal_line`] for a window already promised live.
    fn conceal_line(self, row: c_int, check_cursor: bool) -> bool {
        decor_conceal_line(self, row, check_cursor)
    }
}

/// Where one virtual line [`decor_virt_lines`] collected lives: line `line`
/// of virtual-text link `link`, which the mark it came from owns for the rest
/// of the redraw (a provider deleting it defers the free).
#[derive(Copy, Clone)]
pub(crate) struct VirtLineRef {
    pub(crate) link: uint32_t,
    pub(crate) line: uint32_t,
}

/// The chunks a virtual-text drawer walks: the caller's own, an inline
/// text in the virtual-text store, or one line of a stored virtual-lines
/// block.
///
/// The stored forms are walked a chunk at a time, borrowing the store for
/// the read of one chunk only: resolving a chunk's highlight can run a
/// provider's `hl_def` Lua, which may reach the store again.
#[derive(Copy, Clone)]
pub(crate) enum VirtChunks<'a> {
    Owned(&'a [VirtTextChunk]),
    Text(uint32_t),
    Line(VirtLineRef),
}

impl VirtChunks<'_> {
    /// Runs `f` on the chunks, borrowing the store for the call.
    fn with<R>(self, f: impl FnOnce(&[VirtTextChunk]) -> R) -> R {
        match self {
            VirtChunks::Owned(chunks) => f(chunks),
            VirtChunks::Text(link) => with_decor_virt(link, |vt| f(vt.data.text())),
            VirtChunks::Line(at) => {
                with_decor_virt(at.link, |vt| f(&vt.data.lines()[at.line as usize].line))
            }
        }
    }

    /// How many chunks there are.
    pub(crate) fn len(self) -> size_t {
        self.with(<[VirtTextChunk]>::len)
    }

    /// [`next_virt_text_chunk`], handing the chunk's text to `take` rather
    /// than answering it: a stored chunk is borrowed for that call only.
    /// `take` must not reach the store.
    pub(crate) fn next<R: Copy>(
        self,
        pos: &mut size_t,
        attr: &mut c_int,
        take: impl Fn(&CStr) -> R,
    ) -> Option<R> {
        loop {
            let (hl_id, text) = self.with(|chunks| {
                let chunk = chunks.get(*pos)?;
                Some((chunk.hl_id, chunk.text.as_deref().map(&take)))
            })?;
            if hl_id >= 0 {
                *attr = (*attr).max(0);
                if hl_id > 0 {
                    *attr = hl_combine_attr(*attr, syn_id2attr(hl_id));
                }
            }
            *pos += 1;
            if text.is_some() {
                return text;
            }
        }
    }
}
