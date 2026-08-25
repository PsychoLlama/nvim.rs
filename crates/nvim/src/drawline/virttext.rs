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

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::decoration::{
    kDecorKindUIWatched, kDecorKindVirtText, kHlModeBlend, kHlModeCombine, kVPosEndOfLine,
    kVPosEndOfLineRightAlign, kVPosInline, kVPosRightAlign, kVPosWinCol, kVTRepeatLinebreak,
};
use crate::grid::linebuf;
use crate::types::NUL;

/// Put one character of `*pp` into `dest`, and advance `*pp` past it.
///
/// Answers how many cells were used. A double-width character with only one
/// cell left is replaced by a space and `*pp` is *not* advanced, so the caller
/// gets another go at it on the next row. A Tab expands to its padding.
///
/// # Safety
/// `*pp` must be a live NUL-terminated string, `dest` must have `maxcells`
/// writable cells (with one more readable, which is how the right half of a
/// double-width character being overwritten is found), and `maxcells` must be
/// positive.
pub(crate) unsafe fn line_putchar(
    buf: *mut buf_T,
    pp: &mut *const ::core::ffi::c_char,
    dest: &mut [schar_T],
    maxcells: ::core::ffi::c_int,
    vcol: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    // The caller handles overwriting the right half of a double-width
    // character; a zero here means it did not.
    debug_assert!(dest[0] != 0);
    debug_assert!(maxcells > 0);

    // SAFETY: the caller's string and buffer.
    unsafe {
        let p = *pp;
        let mut cells = utf_ptr2cells(p);
        let c_len = utfc_ptr2len(p);
        if cells > maxcells {
            dest[0] = schar_from_ascii(b' ');
            return 1;
        }

        let is_tab = *p as ::core::ffi::c_int == TAB;
        if is_tab {
            cells = tabstop_padding(vcol, (*buf).b_p_ts, (*buf).b_p_vts_array).min(maxcells);
        }
        // Overwriting the left half of a double-width character: clear its
        // orphaned right half.
        if cells < maxcells && dest[cells as usize] == 0 {
            dest[cells as usize] = schar_from_ascii(b' ');
        }
        if is_tab {
            dest[..cells as usize].fill(schar_from_ascii(b' '));
        } else {
            let mut u8c: ::core::ffi::c_int = 0;
            dest[0] = utfc_ptr2schar(p, &raw mut u8c);
            if cells > 1 {
                // The right half of a double-width glyph is a zero cell.
                dest[1] = 0;
            }
        }

        *pp = p.add(c_len as usize);
        cells
    }
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
/// window row to report a `ui_watched` mark at and the decoration state the
/// redraw is walking.
///
/// # Safety
/// `wp`/`buf` must be live and [`WinLineVars::decor`] must hold the active
/// ranges for its `row`.
pub(crate) unsafe fn draw_virt_text(
    wp: *mut win_T,
    buf: *mut buf_T,
    col_off: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
    wlv: &WinLineVars,
) -> ::core::ffi::c_int {
    let (win_row, mut state) = (wlv.row, wlv.decor);
    // SAFETY: the caller's window and decoration state.
    unsafe {
        let max_col = (*wp).w_view_width;
        let end = state.current_end;
        let do_eol = state.eol_col > -1;

        // Walks leftwards as window-right-aligned texts are placed.
        let mut right_pos = max_col;
        // Total width of every "eol_right_align" text on this row. Worked out
        // by a look-ahead the first time one is placed, and zero until then —
        // which is also what makes the *second* such text take an offset of
        // zero and simply follow the first, whose drawing advanced `eol_col`.
        let mut total_eol_right_width = 0;

        for i in 0..end {
            let item = decor_range_at(state, i);
            if (*item).start_row != state.row || !decor_virt_pos(item) {
                continue;
            }

            let vt = if (*item).kind == kDecorKindVirtText {
                debug_assert!(!(*item).data.vt.is_null());
                (*item).data.vt
            } else {
                ::core::ptr::null_mut::<DecorVirtText>()
            };

            if (*item).draw_col == -1 {
                let placed = match decor_virt_pos_kind(item) {
                    kVPosEndOfLineRightAlign if do_eol => {
                        let mut eol_offset = 0;
                        if total_eol_right_width == 0 {
                            for j in i..end {
                                let ahead = decor_range_at(state, j);
                                if (*ahead).start_row != state.row
                                    || !decor_virt_pos(ahead)
                                    || (*ahead).draw_col != -1
                                {
                                    continue;
                                }
                                if decor_virt_pos_kind(ahead) == kVPosEndOfLineRightAlign {
                                    // Only virtual texts can report this
                                    // position: a `ui_watched` range's is
                                    // always eol or overlay
                                    // (`decor_range_add_sh`). Upstream
                                    // dereferences unconditionally here.
                                    debug_assert!((*ahead).kind == kDecorKindVirtText);
                                    // One space between neighbours.
                                    total_eol_right_width += (*(*ahead).data.vt).width + 1;
                                }
                            }
                            // ...but none after the last one.
                            total_eol_right_width -= 1;
                            if total_eol_right_width <= right_pos - state.eol_col {
                                eol_offset = right_pos - total_eol_right_width - state.eol_col;
                            }
                        }
                        Some(state.eol_col + eol_offset)
                    }
                    kVPosRightAlign => {
                        right_pos -= (*vt).width;
                        Some(right_pos)
                    }
                    kVPosEndOfLine if do_eol => Some(state.eol_col),
                    kVPosWinCol => Some((col_off + (*vt).col).max(0)),
                    // Inline, overlay, or an eol form with no end of line to
                    // hang off: placed elsewhere, or not at all.
                    _ => None,
                };
                if let Some(col) = placed {
                    // Out of the window: do not draw it at all.
                    (*item).draw_col = if col < 0 || col >= (*wp).w_view_width {
                        INT_MIN
                    } else {
                        col
                    };
                }
            }

            if (*item).draw_col < 0 {
                continue;
            }

            if (*item).kind == kDecorKindUIWatched {
                push_win_extmark(WinExtmark {
                    ns_id: (*item).data.ui.ns_id as NS,
                    mark_id: (*item).data.ui.mark_id as uint64_t,
                    win_row,
                    win_col: (*item).draw_col,
                });
            }

            if !vt.is_null() {
                let col = draw_virt_text_item(
                    buf,
                    (*item).draw_col,
                    (*vt).data.virt_text,
                    (*vt).hl_mode as HlMode,
                    max_col,
                    (*item).draw_col - col_off,
                    0,
                );
                if do_eol && ((*vt).pos == kVPosEndOfLine || (*vt).pos == kVPosEndOfLineRightAlign)
                {
                    // The next end-of-line text starts one cell further on.
                    state.eol_col = col + 1;
                }
                end_col = end_col.max(col);
            }

            // Deactivate, unless it is to be repeated on every screen row of
            // a wrapped line.
            if vt.is_null()
                || (*vt).flags as ::core::ffi::c_int & kVTRepeatLinebreak as ::core::ffi::c_int == 0
            {
                (*item).draw_col = INT_MIN;
            }
        }
        end_col
    }
}

/// Paint one virtual text into the line buffer starting at `col`, and answer
/// the column after it.
///
/// `skip_cells` drops that many cells off the front — how a virtual line
/// follows `'leftcol'`. It can go *negative*, which means the opposite: a
/// double-width character or Tab was cut in half by the skip, and that many
/// spaces are owed before the text resumes.
///
/// # Safety
/// `buf` must be live, `vt`'s chunks must be live NUL-terminated strings, and
/// the line buffers must be at least `max_col` wide.
pub(crate) unsafe fn draw_virt_text_item(
    buf: *mut buf_T,
    mut col: ::core::ffi::c_int,
    vt: VirtText,
    hl_mode: HlMode,
    max_col: ::core::ffi::c_int,
    mut vcol: ::core::ffi::c_int,
    mut skip_cells: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut line = linebuf();
    // SAFETY: the caller's buffer, chunks and line-buffer width.
    unsafe {
        let mut virt_str = c"".as_ptr();
        let mut virt_attr = 0;
        let mut virt_pos: size_t = 0;

        while col < max_col {
            if skip_cells >= 0 && *virt_str as ::core::ffi::c_int == NUL {
                if virt_pos >= vt.size {
                    break;
                }
                virt_attr = 0;
                virt_str = next_virt_text_chunk(vt, &raw mut virt_pos, &raw mut virt_attr);
                if virt_str.is_null() {
                    break;
                }
            }

            // Skip cells at the front of this chunk.
            while skip_cells > 0 && *virt_str as ::core::ffi::c_int != NUL {
                let c_len = utfc_ptr2len(virt_str);
                let cells = if *virt_str as ::core::ffi::c_int == TAB {
                    tabstop_padding(vcol, (*buf).b_p_ts, (*buf).b_p_vts_array)
                } else {
                    utf_ptr2cells(virt_str)
                };
                skip_cells -= cells;
                vcol += cells;
                virt_str = virt_str.add(c_len as usize);
            }

            // A wide character the skip cut in half is paid back as spaces.
            let mut draw_str = if skip_cells < 0 {
                c" ".as_ptr()
            } else {
                virt_str
            };
            if *draw_str as ::core::ffi::c_int == NUL {
                continue;
            }
            debug_assert!(skip_cells <= 0);

            let mut through = false;
            let under = line.attrs_mut()[col as usize];
            let attr = match hl_mode {
                kHlModeCombine => hl_combine_attr(under, virt_attr),
                kHlModeBlend => {
                    // A space blends the cell underneath through, rather than
                    // painting over it.
                    through = *draw_str as ::core::ffi::c_int == ' ' as ::core::ffi::c_int;
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

            let mut dummy: [schar_T; 2] = [schar_from_ascii(b' '); 2];
            let dest: &mut [schar_T] = if through {
                &mut dummy
            } else {
                &mut line.chars_mut()[col as usize..]
            };
            let cells = line_putchar(buf, &mut draw_str, dest, max_col - col, vcol);
            let attrs = line.attrs_mut();
            for _ in 0..cells {
                attrs[col as usize] = attr as sattr_T;
                col += 1;
            }

            if skip_cells < 0 {
                skip_cells += 1;
            } else {
                vcol += cells;
                virt_str = draw_str;
            }
        }
        col
    }
}

// ---------------------------------------------------------------------------
// Inline virtual text
// ---------------------------------------------------------------------------

/// Whether an active range describes inline virtual text at or after byte `v`
/// of the line.
///
/// # Safety
/// `item` must be a live decoration range.
#[inline]
unsafe fn inline_virt_at(item: *const DecorRange, row: ::core::ffi::c_int) -> bool {
    // SAFETY: the caller's range.
    unsafe {
        (*item).start_row == row
            && (*item).kind == kDecorKindVirtText
            && (*(*item).data.vt).pos == kVPosInline
            && (*(*item).data.vt).width != 0
            && (*item).draw_col >= -1
    }
}

impl WinLineVars {
    /// Whether there is inline virtual text still to draw at or after byte `v`
    /// of the line.
    ///
    /// The character loop asks this to decide whether it may take its fast
    /// path to the end of the line.
    ///
    /// # Safety
    /// [`WinLineVars::decor`] must hold this row's ranges.
    pub(crate) unsafe fn has_more_inline_virt(&self, v: ptrdiff_t) -> bool {
        // SAFETY: the redraw's decoration state, threaded in `self`.
        unsafe {
            if self.virt_inline_i < self.virt_inline.size {
                return true;
            }
            let state = self.decor;
            let row = state.row;
            // Both halves of `ranges_i`: the ranges active now, and the ones
            // that start later on this row.
            let spans = [
                (0, state.current_end),
                (state.future_begin, decor_range_count(state)),
            ];
            for (from, to) in spans {
                for i in from..to {
                    let item = decor_range_at(state, i);
                    if inline_virt_at(item, row) && (*item).start_col as ptrdiff_t >= v {
                        return true;
                    }
                }
            }
            false
        }
    }

    /// Load the inline virtual text that starts at byte `v` into
    /// [`WinLineVars::extra_text`], so the character loop draws it as if it were
    /// buffer text.
    ///
    /// Runs until something is loaded or there is nothing left: a chunk may be
    /// empty, and a chunk entirely left of the first visible column is dropped
    /// and the next one tried.
    ///
    /// # Safety
    /// [`WinLineVars::decor`] must hold this row's ranges.
    pub(crate) unsafe fn handle_inline_virtual_text(&mut self, v: ptrdiff_t, selected: bool) {
        // SAFETY: the redraw's decoration state, threaded in `self`;
        // `extra_text` borrows the chunk, which that state owns for the rest
        // of the redraw.
        unsafe {
            while self.extra_todo == 0 {
                if self.virt_inline_i >= self.virt_inline.size {
                    // Find the next inline text to start.
                    self.virt_inline = VIRTTEXT_EMPTY;
                    self.virt_inline_i = 0;
                    let state = self.decor;
                    let row = state.row;
                    for i in 0..state.current_end {
                        let item = decor_range_at(state, i);
                        if (*item).draw_col == -3 {
                            // Nothing inline can precede this non-inline text
                            // any more, so its column can be fixed now.
                            decor_init_draw_col(self.off, selected, item);
                        }
                        if inline_virt_at(item, row) && (*item).start_col as ptrdiff_t == v {
                            self.virt_inline = (*(*item).data.vt).data.virt_text;
                            self.virt_inline_hl_mode = (*(*item).data.vt).hl_mode as HlMode;
                            (*item).draw_col = INT_MIN;
                            break;
                        }
                    }
                    if self.virt_inline.size == 0 {
                        // No more inline virtual text here.
                        break;
                    }
                    continue;
                }

                // Already inside a text with several chunks.
                let mut attr = 0;
                let text = next_virt_text_chunk(
                    self.virt_inline,
                    &raw mut self.virt_inline_i,
                    &raw mut attr,
                );
                if text.is_null() {
                    continue;
                }
                self.extra_text = text;
                self.extra_todo = strlen(text) as ::core::ffi::c_int;
                if self.extra_todo == 0 {
                    continue;
                }
                self.extra_fill = NUL as schar_T;
                self.extra_last = NUL as schar_T;
                self.extra_attr = attr;
                self.n_attr = mb_charlen(text);

                if self.skip_cells > 0 {
                    // The text starts left of the first visible column.
                    let width = mb_string2cells(self.extra_text) as ::core::ffi::c_int;
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
                        let cells = utf_ptr2cells(self.extra_text);
                        if cells > remaining {
                            break;
                        }
                        let c_len = utfc_ptr2len(self.extra_text);
                        remaining -= cells;
                        self.extra_text = self.extra_text.add(c_len as usize);
                        self.extra_todo -= c_len;
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
}
