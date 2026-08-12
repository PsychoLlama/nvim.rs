//! Closing a `%(...%)` group.
//!
//! A group is remembered as one item holding the width the `%(` asked for and
//! the offset the text starts at; everything between it and the matching `%)`
//! has already been written. Closing it is three decisions, in this order:
//! is the group empty enough to erase, is it too wide, is it too narrow. All
//! three move text that later items point into, which is why every item index
//! above the group's own is fixed up as they go.
//!
//! Original: `src/nvim/statusline.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

use core::ffi::c_int;

use super::{Fill, Kind, StlScratch, cells_at, char_len_at, strsize_at};

/// Close the innermost open group, answering where the write cursor ends up.
///
/// `end` is the last byte of the output buffer the expander may write before.
pub(super) fn close(
    s: &mut StlScratch,
    out: &mut [u8],
    pos: usize,
    end: usize,
    groupdepth: &mut c_int,
    fill: &Fill,
) -> usize {
    *groupdepth -= 1;
    let gi = s.groupitems[*groupdepth as usize];
    let start = s.items[gi].start;

    // How long the group is. The write cursor is NUL-terminated first so
    // that measuring it works.
    out[pos] = 0;
    let mut pos = pos;
    let mut group_len = i64::from(strsize_at(out, start));

    if s.curitem > gi + 1 && s.items[gi].minwid == 0 {
        if let Some(erased) = erase_if_empty(s, gi, start) {
            pos = start;
            group_len = erased;
        }
    }

    let mut minwid = s.items[gi].minwid;
    let maxwid = s.items[gi].maxwid;
    // Too wide: cut bytes off the front. A `'statuscolumn'` fold item is
    // never cut, so that the mouse click regions stay right.
    if group_len > i64::from(maxwid) && s.items[gi].kind != Kind::HighlightFold {
        pos = truncate(s, out, pos, gi, start, group_len, minwid, maxwid, fill);
    } else if i64::from(minwid.abs()) > group_len {
        // Too narrow: pad it, to the right when the group is left-aligned
        // (which is what the negative width means) and to the left otherwise.
        if minwid < 0 {
            minwid = -minwid;
            let width = fill.len();
            while group_len < i64::from(minwid) && pos + width <= end {
                group_len += 1;
                pos = fill.put(out, pos);
            }
        } else {
            pos = pad_left(s, out, pos, gi, start, group_len, minwid, end, fill);
        }
    }
    pos
}

/// Erase the group when every item in it came out empty and the highlight in
/// force did not change across it, answering its new length.
///
/// This deletes any literal text in the group too, which is the whole point:
/// `%( [%f] %)` should leave nothing behind when there is no file name.
fn erase_if_empty(s: &mut StlScratch, gi: usize, start: usize) -> Option<i64> {
    // The highlight in force where the group starts is whichever one the
    // nearest preceding highlight item set -- including one an *outer*
    // expansion set, since the items are one shared stack.
    let mut group_start_userhl = 0;
    let mut group_end_userhl = 0;
    for n in (0..gi).rev() {
        let item = s.items[n];
        if matches!(item.kind, Kind::Highlight | Kind::HighlightCombining) {
            group_end_userhl = item.minwid;
            group_start_userhl = group_end_userhl;
            break;
        }
    }

    // Walk forward for a Normal item, which is text and stops the erasure.
    let mut n = gi + 1;
    while n < s.curitem {
        let item = s.items[n];
        if item.kind == Kind::Normal {
            break;
        }
        if matches!(item.kind, Kind::Highlight | Kind::HighlightCombining) {
            group_end_userhl = item.minwid;
        }
        n += 1;
    }
    if n != s.curitem || group_start_userhl != group_end_userhl {
        return None;
    }

    for n in gi + 1..s.curitem {
        // Do not keep the highlighting the erased group asked for.
        if matches!(s.items[n].kind, Kind::Highlight | Kind::HighlightCombining) {
            s.items[n].kind = Kind::Empty;
        }
        // A tab page label inside it collapses onto what follows.
        if s.items[n].kind == Kind::TabPage {
            s.items[n].start = start;
        }
    }
    Some(0)
}

/// Cut the group down to `maxwid` cells by dropping bytes from its front and
/// marking the cut with a `<`.
#[allow(clippy::too_many_arguments)]
fn truncate(
    s: &mut StlScratch,
    out: &mut [u8],
    pos: usize,
    gi: usize,
    start: usize,
    mut group_len: i64,
    minwid: c_int,
    maxwid: c_int,
    fill: &Fill,
) -> usize {
    // Find the first character that still fits.
    let mut dropped = 0usize;
    while group_len >= i64::from(maxwid) {
        group_len -= i64::from(cells_at(out, start + dropped));
        dropped += char_len_at(out, start + dropped);
    }

    // Prepend the `<` that says the text was cut, and close the gap.
    out[start] = b'<';
    out.copy_within(start + dropped..pos, start + 1);
    let mut pos = pos - dropped + 1;

    // Fill up the space left over by half a double-width character.
    let minwid = minwid.min(maxwid);
    loop {
        group_len += 1;
        if group_len >= i64::from(minwid) {
            break;
        }
        pos = fill.put(out, pos);
    }

    // Shift the items back by what was dropped, less the one byte the `<`
    // took; anything that was cut away starts at the `<` itself.
    let offset = dropped as isize - 1;
    for idx in gi + 1..s.curitem {
        let moved = s.items[idx].start as isize - offset;
        s.items[idx].start = moved.max(start as isize) as usize;
    }
    pos
}

/// Shift the group right and fill the space in front of it, so that a
/// right-aligned group reaches its minimum width.
#[allow(clippy::too_many_arguments)]
fn pad_left(
    s: &mut StlScratch,
    out: &mut [u8],
    pos: usize,
    gi: usize,
    start: usize,
    group_len: i64,
    minwid: c_int,
    end: usize,
    fill: &Fill,
) -> usize {
    let width = fill.len();
    let mut added_cells = i64::from(minwid) - group_len;
    let mut added_bytes = added_cells * width as i64;
    if pos + added_bytes as usize > end {
        added_cells = ((end - pos) / width) as i64;
        added_bytes = added_cells * width as i64;
    }
    let added_bytes = added_bytes as usize;

    out.copy_within(start..pos, start + added_bytes);
    let pos = pos + added_bytes;
    for n in gi + 1..s.curitem {
        s.items[n].start += added_bytes;
    }
    let mut at = start;
    for _ in 0..added_cells {
        at = fill.put(out, at);
    }
    pos
}
