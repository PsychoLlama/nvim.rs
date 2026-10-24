//! Virtual text — `nvim_buf_set_extmark`'s `virt_text`.
//!
//! Two shapes, reaching the line buffer by different routes.
//! [`draw_virt_text`] runs once after the text of a screen line has been laid
//! out and paints the texts positioned relative to the *line*: at the end of
//! it, right-aligned in the window, or at a fixed window column. Inline
//! virtual text is instead fed to `win_line`'s character loop as if it were
//! buffer text, which is what [`WinLineVars::handle_inline_virtual_text`] does
//! — including dropping, or partially skipping, a chunk that starts left of
//! the first visible column.
//!
//! [`draw_virt_text_item`] paints one text through the `hl_mode` blend, and
//! [`line_putchar`] is the single-character primitive under both of them (and
//! under the column drawing in `columns.rs`).

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::decoration::{
    DECOR_ID_INVALID, VirtChunks, decor_inline_virt_from, decor_lists, decor_range,
    kDecorKindUIWatched, kDecorKindVirtText, kHlModeBlend, kHlModeCombine, kVPosEndOfLine,
    kVPosEndOfLineRightAlign, kVPosInline, kVPosRightAlign, kVPosWinCol, kVTRepeatLinebreak,
    set_decor_eol_col, set_decor_range_draw_col, with_decor_virt,
};
use crate::grid::with_linebuf;
use crate::types::NUL;
use crate::winlayer::Buf;

/// Put the character at the start of `text` into `dest`, and answer how many
/// cells it used and how many bytes of `text` it took.
///
/// A double-width character with only one cell left is replaced by a space
/// and *not* taken, so the caller gets another go at it on the next row. A
/// Tab expands to its padding. `text` ends at its end or a NUL in it.
///
/// `dest` must have `maxcells` writable cells, and one more readable when
/// there are more: that is how the right half of a double-width character
/// being overwritten is found. `maxcells` must be positive.
pub(crate) fn line_putchar(
    buffer: Buf,
    text: &[u8],
    dest: &mut [ScreenChar],
    maxcells: ::core::ffi::c_int,
    vcol: ::core::ffi::c_int,
) -> (::core::ffi::c_int, usize) {
    // The caller handles overwriting the right half of a double-width
    // character; a zero here means it did not.
    debug_assert!(dest[0] != 0);
    debug_assert!(maxcells > 0);

    let mut cells = c_cells_at(text);
    if cells > maxcells {
        dest[0] = schar_from_ascii(b' ');
        return (1, 0);
    }
    let (c_len, cell, _) = c_cluster_cell(text);

    let is_tab = text.first() == Some(&b'\t');
    if is_tab {
        cells = buffer.tab_padding(vcol).min(maxcells);
    }
    // Overwriting the left half of a double-width character: clear its
    // orphaned right half.
    if cells < maxcells && dest[cells as usize] == 0 {
        dest[cells as usize] = schar_from_ascii(b' ');
    }
    if is_tab {
        dest[..cells as usize].fill(schar_from_ascii(b' '));
    } else {
        dest[0] = cell;
        if cells > 1 {
            // The right half of a double-width glyph is a zero cell.
            dest[1] = 0;
        }
    }
    (cells, c_len)
}

// ---------------------------------------------------------------------------
// Line-positioned virtual text
// ---------------------------------------------------------------------------

/// Append to the per-redraw `win_extmark` list `win_update` drains.
fn push_win_extmark(m: WinExtmark) {
    win_extmark_arr.with_mut(|marks| marks.push(m));
}

/// Paint every line-positioned virtual text of the row just laid out, and
/// report any `ui_watched` mark's final column to the UI.
///
/// `col_off` is where the buffer text starts (past the left columns), and the
/// answer is `end_col` raised to the rightmost column anything was drawn at,
/// so the caller knows how much of the line buffer to flush. `wlv` carries the
/// window row to report a `ui_watched` mark at.
pub(crate) fn draw_virt_text(
    window: Win,
    buffer: Buf,
    col_off: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
    wlv: &WinLineVars,
) -> ::core::ffi::c_int {
    let win_row = wlv.row;
    let max_col = window.w_view_width;
    let lists = decor_lists();
    let (row, end) = (lists.row, lists.current_end);
    // Where the next end-of-line text goes; drawing one moves it.
    let mut eol_col = lists.eol_col;
    let do_eol = eol_col > -1;

    // Walks leftwards as window-right-aligned texts are placed.
    let mut right_pos = max_col;
    // Total width of every "eol_right_align" text on this row. Worked out
    // by a look-ahead the first time one is placed, and zero until then —
    // which is also what makes the *second* such text take an offset of
    // zero and simply follow the first, whose drawing advanced `eol_col`.
    let mut total_eol_right_width = 0;

    for i in 0..end {
        // A copy: drawing reads it, and `draw_col` is written back.
        let mut item = decor_range(i);
        if item.start_row != row || !decor_virt_pos(&item) {
            continue;
        }

        // The link, and a copy of the fields read off it below.
        let vt = (item.kind == kDecorKindVirtText).then(|| {
            let link = item.data.virt();
            with_decor_virt(link, |vt| VirtShape {
                link,
                width: vt.width,
                col: vt.col,
                hl_mode: vt.hl_mode,
                pos: vt.pos,
                flags: vt.flags,
            })
        });

        if item.draw_col == -1 {
            let placed = match decor_virt_pos_kind(&item) {
                kVPosEndOfLineRightAlign if do_eol => {
                    let mut eol_offset = 0;
                    if total_eol_right_width == 0 {
                        for j in i..end {
                            let ahead = decor_range(j);
                            if ahead.start_row != row
                                || !decor_virt_pos(&ahead)
                                || ahead.draw_col != -1
                            {
                                continue;
                            }
                            if decor_virt_pos_kind(&ahead) == kVPosEndOfLineRightAlign {
                                // Only virtual texts can report this
                                // position: a `ui_watched` range's is
                                // always eol or overlay
                                // (`decor_range_add_sh`). Upstream
                                // dereferences unconditionally here.
                                debug_assert!(ahead.kind == kDecorKindVirtText);
                                // One space between neighbours.
                                let link = ahead.data.virt();
                                total_eol_right_width += with_decor_virt(link, |vt| vt.width) + 1;
                            }
                        }
                        // ...but none after the last one.
                        total_eol_right_width -= 1;
                        if total_eol_right_width <= right_pos - eol_col {
                            eol_offset = right_pos - total_eol_right_width - eol_col;
                        }
                    }
                    Some(eol_col + eol_offset)
                }
                kVPosRightAlign => {
                    right_pos -= vt.map_or(0, |vt| vt.width);
                    Some(right_pos)
                }
                kVPosEndOfLine if do_eol => Some(eol_col),
                kVPosWinCol => Some((col_off + vt.map_or(0, |vt| vt.col)).max(0)),
                // Inline, overlay, or an eol form with no end of line to
                // hang off: placed elsewhere, or not at all.
                _ => None,
            };
            if let Some(col) = placed {
                // Out of the window: do not draw it at all.
                item.draw_col = if col < 0 || col >= window.w_view_width {
                    INT_MIN
                } else {
                    col
                };
                set_decor_range_draw_col(i, item.draw_col);
            }
        }

        if item.draw_col < 0 {
            continue;
        }

        if item.kind == kDecorKindUIWatched {
            let ui = item.data.ui_watched();
            push_win_extmark(WinExtmark {
                ns_id: ui.ns_id as NS,
                mark_id: ui.mark_id as uint64_t,
                win_row,
                win_col: item.draw_col,
            });
        }

        if let Some(vt) = vt {
            let col = draw_virt_text_item(
                buffer,
                item.draw_col,
                VirtChunks::Text(vt.link),
                vt.hl_mode as HlMode,
                max_col,
                item.draw_col - col_off,
                0,
            );
            if do_eol && (vt.pos == kVPosEndOfLine || vt.pos == kVPosEndOfLineRightAlign) {
                // The next end-of-line text starts one cell further on.
                eol_col = col + 1;
                set_decor_eol_col(eol_col);
            }
            end_col = end_col.max(col);
        }

        // Deactivate, unless it is to be repeated on every screen row of
        // a wrapped line.
        if vt.is_none_or(|vt| vt.flags & kVTRepeatLinebreak == 0) {
            set_decor_range_draw_col(i, INT_MIN);
        }
    }
    end_col
}

/// The fields of a virtual-text link [`draw_virt_text`] reads, copied out of
/// the store.
#[derive(Copy, Clone)]
struct VirtShape {
    link: uint32_t,
    width: ::core::ffi::c_int,
    col: ::core::ffi::c_int,
    hl_mode: uint8_t,
    pos: VirtTextPos,
    flags: uint8_t,
}

/// Paint one virtual text into the line buffer starting at `col`, and answer
/// the column after it.
///
/// `skip_cells` drops that many cells off the front — how a virtual line
/// follows `'leftcol'`. It can go *negative*, which means the opposite: a
/// double-width character or Tab was cut in half by the skip, and that many
/// spaces are owed before the text resumes.
///
/// Each chunk's text is copied out of its store before it is drawn: reading
/// the next chunk resolves its highlight, which can run a provider's Lua.
/// Out of line, for the copy's allocation: the callers are `win_line`.
#[inline(never)]
pub(crate) fn draw_virt_text_item(
    buffer: Buf,
    mut col: ::core::ffi::c_int,
    vt: VirtChunks<'_>,
    hl_mode: HlMode,
    max_col: ::core::ffi::c_int,
    mut vcol: ::core::ffi::c_int,
    mut skip_cells: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    // The chunk being drawn, and how far into it the drawing has got.
    let mut chunk: Vec<u8> = Vec::new();
    let mut at = 0;
    let mut virt_attr = 0;
    let mut virt_pos: size_t = 0;

    // The line buffer for the whole walk: nothing below draws.
    with_linebuf(|line| {
        while col < max_col {
            if skip_cells >= 0 && at >= chunk.len() {
                if virt_pos >= vt.len() {
                    break;
                }
                virt_attr = 0;
                let copied = vt.next(&mut virt_pos, &mut virt_attr, |text| {
                    chunk.clear();
                    chunk.extend_from_slice(text.to_bytes());
                });
                if copied.is_none() {
                    break;
                }
                at = 0;
            }

            // Skip cells at the front of this chunk.
            while skip_cells > 0 && at < chunk.len() {
                let rest = &chunk[at..];
                let cells = if rest[0] == b'\t' {
                    buffer.tab_padding(vcol)
                } else {
                    c_cells_at(rest)
                };
                skip_cells -= cells;
                vcol += cells;
                at += c_cluster_len(rest);
            }

            // A wide character the skip cut in half is paid back as spaces.
            let draw_text: &[u8] = if skip_cells < 0 { b" " } else { &chunk[at..] };
            let Some(&first) = draw_text.first() else {
                continue;
            };
            debug_assert!(skip_cells <= 0);

            let mut through = false;
            let under = line.attrs_mut()[col as usize];
            let attr = match hl_mode {
                kHlModeCombine => hl_combine_attr(under, virt_attr),
                kHlModeBlend => {
                    // A space blends the cell underneath through, rather than
                    // painting over it.
                    through = first == b' ';
                    hl_blend_attrs(under, virt_attr, &mut through)
                }
                _ => virt_attr,
            };

            // Landing on the right half of a double-width character: clear
            // its left half too, and the right half itself so `line_putchar`
            // has a non-zero cell to start from.
            if !through && line.chars()[col as usize] == 0 {
                debug_assert!(col > 0);
                line.chars_mut()[col as usize - 1] = schar_from_ascii(b' ');
                line.chars_mut()[col as usize] = schar_from_ascii(b' ');
            }

            let mut dummy: [ScreenChar; 2] = [schar_from_ascii(b' '); 2];
            let dest: &mut [ScreenChar] = if through {
                &mut dummy
            } else {
                &mut line.chars_mut()[col as usize..]
            };
            let (cells, taken) = line_putchar(buffer, draw_text, dest, max_col - col, vcol);
            let attrs = line.attrs_mut();
            for _ in 0..cells {
                attrs[col as usize] = attr as ScreenAttr;
                col += 1;
            }

            if skip_cells < 0 {
                skip_cells += 1;
            } else {
                vcol += cells;
                at += taken;
            }
        }
    });
    col
}

// ---------------------------------------------------------------------------
// Inline virtual text
// ---------------------------------------------------------------------------

/// Whether an active range describes inline virtual text on `row`.
#[inline]
fn inline_virt_at(item: &DecorRange, row: ::core::ffi::c_int) -> bool {
    item.start_row == row
        && item.kind == kDecorKindVirtText
        && with_decor_virt(item.data.virt(), |vt| {
            vt.pos == kVPosInline && vt.width != 0
        })
        && item.draw_col >= -1
}

impl WinLineVars {
    /// Whether there is inline virtual text still to draw at or after byte `v`
    /// of the line.
    ///
    /// The character loop asks this to decide whether it may take its fast
    /// path to the end of the line.
    pub(crate) fn has_more_inline_virt(&self, v: ::core::ffi::c_int) -> bool {
        if self.virt_inline_i < self.virt_inline_len {
            return true;
        }
        decor_inline_virt_from(v as ptrdiff_t)
    }

    /// Load the inline virtual text that starts at byte `v` into
    /// [`WinLineVars::extra_text`], so the character loop draws it as if it were
    /// buffer text.
    ///
    /// Runs until something is loaded or there is nothing left: a chunk may be
    /// empty, and a chunk entirely left of the first visible column is dropped
    /// and the next one tried.
    pub(crate) fn handle_inline_virtual_text(&mut self, v: ::core::ffi::c_int, selected: bool) {
        while self.extra_todo == 0 {
            if self.virt_inline_i >= self.virt_inline_len {
                // Find the next inline text to start.
                self.virt_inline = DECOR_ID_INVALID;
                self.virt_inline_len = 0;
                self.virt_inline_i = 0;
                let lists = decor_lists();
                for i in 0..lists.current_end {
                    if decor_range(i).draw_col == -3 {
                        // Nothing inline can precede this non-inline text
                        // any more, so its column can be fixed now.
                        decor_init_draw_col(self.off, selected, i);
                    }
                    let item = decor_range(i);
                    if inline_virt_at(&item, lists.row) && item.start_col == v {
                        let link = item.data.virt();
                        let (len, hl_mode) =
                            with_decor_virt(link, |vt| (vt.data.text().len(), vt.hl_mode));
                        self.virt_inline = link;
                        self.virt_inline_len = len;
                        self.virt_inline_hl_mode = hl_mode as HlMode;
                        set_decor_range_draw_col(i, INT_MIN);
                        break;
                    }
                }
                if self.virt_inline_len == 0 {
                    // No more inline virtual text here.
                    break;
                }
                continue;
            }

            // Already inside a text with several chunks: copy the next one
            // in as the run.
            let mut attr = 0;
            let vt = VirtChunks::Text(self.virt_inline);
            let extra = &mut self.extra;
            let copied = vt.next(&mut self.virt_inline_i, &mut attr, |text| {
                extra.set(text.to_bytes());
                text.to_bytes().len()
            });
            let Some(len) = copied else {
                continue;
            };
            self.extra_todo = len as ::core::ffi::c_int;
            if self.extra_todo == 0 {
                continue;
            }
            self.extra_fill = NUL as ScreenChar;
            self.extra_last = NUL as ScreenChar;
            self.extra_attr = attr;
            let text = self.extra.rest();
            let text = &text[..text.len() - 1];
            self.n_attr = char_count(text) as ::core::ffi::c_int;

            if self.skip_cells > 0 {
                // The text starts left of the first visible column.
                let width = c_string_cells(text) as ::core::ffi::c_int;
                if width <= self.skip_cells {
                    // Entirely off-screen: drop it and take the next
                    // chunk. The cells it would have taken still count
                    // towards `vcol`.
                    self.skip_cells -= width;
                    self.skipped_cells += width;
                    self.n_attr = 0;
                    self.extra_todo = 0;
                    continue;
                }
                // Partly visible: step over whole characters until the
                // next one would straddle the edge, and leave the rest of
                // the skip to the character loop.
                let mut remaining = self.skip_cells;
                while remaining > 0 {
                    let rest = self.extra.rest();
                    let cells = c_cells_at(rest);
                    if cells > remaining {
                        break;
                    }
                    let c_len = c_cluster_len(rest);
                    remaining -= cells;
                    self.extra.advance(c_len);
                    self.extra_todo -= c_len as ::core::ffi::c_int;
                    self.n_attr -= 1;
                }
                self.skipped_cells += self.skip_cells - remaining;
                self.skip_cells = remaining;
            }

            debug_assert!(self.extra_todo > 0);
            self.extra_is_virt_text = true;
        }
    }
}
