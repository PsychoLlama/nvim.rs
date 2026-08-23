//! Translating between a screen cell's internal pen and its reported form.
//!
//! The grid stores a [`ScreenPen`] per cell, which carries the layout bits
//! (double-width/height, protection) the emulator needs. What a consumer
//! reads back is a [`VTermScreenCell`], whose attributes are the same
//! rendition minus the protection bit — and with the screen-wide reverse
//! folded in, so that reversing the whole screen costs nothing per cell.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_uint;

use crate::types::{ScreenCell, ScreenPen, VTermColor, VTermScreenCell, schar_T};

/// The `schar` of the cell hidden behind a double-width glyph.
pub const SCHAR_CONTINUATION: schar_T = schar_T::MAX;

/// The pen an erased cell is left with: the current colours, everything else
/// back to its reset state. The caller stamps the line's double-width and
/// double-height bits on afterwards.
pub fn erased_pen(fg: VTermColor, bg: VTermColor) -> ScreenPen {
    ScreenPen {
        fg,
        bg,
        uri: 0,
        bold_underline_italic_blink_reverse_conceal_strike_font_small_baseline_dim_overline_protected_cell_dwl_dhl:
            [0; 3],
        _pad: [0; 1],
    }
}

/// Copies a cell's rendition, colours and URI into its reported form.
///
/// `global_reverse` is exclusive-or'd into the reverse bit rather than
/// stored, so that the screen-wide reverse property applies to every cell at
/// once. The caller still owns `schar` and `width`, which depend on the
/// cells around this one.
pub fn export_pen(pen: &ScreenPen, global_reverse: bool, cell: &mut VTermScreenCell) {
    cell.attrs.set_bold(pen.bold());
    cell.attrs.set_underline(pen.underline());
    cell.attrs.set_italic(pen.italic());
    cell.attrs.set_blink(pen.blink());
    cell.attrs
        .set_reverse(pen.reverse() ^ c_uint::from(global_reverse));
    cell.attrs.set_conceal(pen.conceal());
    cell.attrs.set_strike(pen.strike());
    cell.attrs.set_font(pen.font());
    cell.attrs.set_small(pen.small());
    cell.attrs.set_baseline(pen.baseline());
    cell.attrs.set_dim(pen.dim());
    cell.attrs.set_overline(pen.overline());
    cell.attrs.set_dwl(pen.dwl());
    cell.attrs.set_dhl(pen.dhl());
    cell.fg = pen.fg;
    cell.bg = pen.bg;
    cell.uri = pen.uri;
}

/// The reverse of [`export_pen`], for a cell coming back out of scrollback.
///
/// The screen-wide reverse is exclusive-or'd back out. The layout bits the
/// reported form does not carry — protection, and the double-width and
/// double-height marks — are left as the destination pen had them, which for
/// a freshly allocated buffer means clear.
pub fn import_pen(cell: &VTermScreenCell, global_reverse: bool, pen: &mut ScreenPen) {
    pen.set_bold(cell.attrs.bold());
    pen.set_underline(cell.attrs.underline());
    pen.set_italic(cell.attrs.italic());
    pen.set_blink(cell.attrs.blink());
    pen.set_reverse(cell.attrs.reverse() ^ c_uint::from(global_reverse));
    pen.set_conceal(cell.attrs.conceal());
    pen.set_strike(cell.attrs.strike());
    pen.set_font(cell.attrs.font());
    pen.set_small(cell.attrs.small());
    pen.set_baseline(cell.attrs.baseline());
    pen.set_dim(cell.attrs.dim());
    pen.set_overline(cell.attrs.overline());
    pen.fg = cell.fg;
    pen.bg = cell.bg;
    pen.uri = cell.uri;
}

/// Blanks cells, leaving each of them in `pen`.
pub fn blank_cells(cells: &mut [ScreenCell], pen: &ScreenPen) {
    for cell in cells {
        cell.schar = 0;
        cell.pen = *pen;
    }
}

/// Lays a row popped from scrollback into a screen row: every glyph with its
/// rendition, a gap cell behind each double-width glyph, and blanks from
/// wherever the source runs out.
///
/// The two rows need not be the same width — the screen may have been resized
/// since the line was pushed — so the shorter one wins. A cell claiming zero
/// width would spin forever, so it counts as one.
pub fn import_row(
    src: &[VTermScreenCell],
    dst: &mut [ScreenCell],
    global_reverse: bool,
    blank: &ScreenPen,
) {
    let mut col = 0;
    while col < src.len() && col < dst.len() {
        let cell = &src[col];
        dst[col].schar = cell.schar;
        import_pen(cell, global_reverse, &mut dst[col].pen);
        if cell.width == 2 && col + 1 < dst.len() {
            dst[col + 1].schar = SCHAR_CONTINUATION;
        }
        col += usize::try_from(cell.width).unwrap_or(1).max(1);
    }
    let tail = col.min(dst.len());
    blank_cells(&mut dst[tail..], blank);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{VTermColor, VTermScreenCellAttrs};

    fn black() -> VTermColor {
        VTermColor { type_0: 0 }
    }

    fn blank_cell() -> VTermScreenCell {
        VTermScreenCell {
            schar: 0,
            width: 1,
            attrs: VTermScreenCellAttrs {
                bold_underline_italic_blink_reverse_conceal_strike_font_dwl_dhl_small_baseline_dim_overline:
                    [0; 3],
                _pad: [0; 1],
            },
            fg: black(),
            bg: black(),
            uri: 0,
        }
    }

    fn decorated_pen() -> ScreenPen {
        let mut pen = erased_pen(black(), black());
        pen.set_bold(1);
        pen.set_underline(3);
        pen.set_italic(1);
        pen.set_blink(1);
        pen.set_conceal(1);
        pen.set_strike(1);
        pen.set_font(5);
        pen.set_small(1);
        pen.set_baseline(2);
        pen.set_dim(1);
        pen.set_overline(1);
        pen.uri = 7;
        pen
    }

    #[test]
    fn an_erased_pen_keeps_only_the_colours() {
        let pen = erased_pen(black(), black());
        assert_eq!(pen.bold(), 0);
        assert_eq!(pen.underline(), 0);
        assert_eq!(pen.uri, 0);
        assert_eq!(pen.protected_cell(), 0);
        assert_eq!(pen.dwl(), 0);
        assert_eq!(pen.dhl(), 0);
    }

    #[test]
    fn a_pen_survives_a_round_trip_through_the_reported_form() {
        let pen = decorated_pen();
        let mut cell = blank_cell();
        export_pen(&pen, false, &mut cell);
        let mut back = erased_pen(black(), black());
        import_pen(&cell, false, &mut back);

        assert_eq!(back.bold(), 1);
        assert_eq!(back.underline(), 3);
        assert_eq!(back.font(), 5);
        assert_eq!(back.baseline(), 2);
        assert_eq!(back.overline(), 1);
        assert_eq!(back.uri, 7);
    }

    #[test]
    fn the_screen_wide_reverse_is_folded_in_and_back_out() {
        let mut pen = erased_pen(black(), black());
        pen.set_reverse(1);

        let mut cell = blank_cell();
        export_pen(&pen, true, &mut cell);
        assert_eq!(cell.attrs.reverse(), 0);

        let mut back = erased_pen(black(), black());
        import_pen(&cell, true, &mut back);
        assert_eq!(back.reverse(), 1);
    }

    fn blank_row(cols: usize) -> Vec<ScreenCell> {
        vec![
            ScreenCell {
                schar: 0,
                pen: erased_pen(black(), black()),
            };
            cols
        ]
    }

    #[test]
    fn an_imported_row_keeps_a_gap_behind_a_wide_glyph() {
        let mut src = vec![blank_cell(); 4];
        src[0].schar = 'a' as schar_T;
        src[1].schar = 0x4e00;
        src[1].width = 2;
        src[3].schar = 'b' as schar_T;

        let mut dst = blank_row(4);
        import_row(&src, &mut dst, false, &erased_pen(black(), black()));
        assert_eq!(dst[0].schar, 'a' as schar_T);
        assert_eq!(dst[1].schar, 0x4e00);
        assert_eq!(dst[2].schar, SCHAR_CONTINUATION);
        assert_eq!(dst[3].schar, 'b' as schar_T);
    }

    #[test]
    fn an_imported_row_is_blanked_past_the_shorter_of_the_two() {
        let mut src = vec![blank_cell(); 2];
        src[0].schar = 'a' as schar_T;
        src[1].schar = 'b' as schar_T;

        let mut dst = blank_row(4);
        for cell in &mut dst {
            cell.schar = 'z' as schar_T;
        }
        import_row(&src, &mut dst, false, &erased_pen(black(), black()));
        assert_eq!(dst[1].schar, 'b' as schar_T);
        assert_eq!(dst[2].schar, 0);
        assert_eq!(dst[3].schar, 0);

        // Narrower destination: the extra source cells are dropped.
        let mut dst = blank_row(1);
        import_row(&src, &mut dst, false, &erased_pen(black(), black()));
        assert_eq!(dst[0].schar, 'a' as schar_T);
    }

    #[test]
    fn a_zero_width_cell_does_not_stall_the_import() {
        let mut src = vec![blank_cell(); 2];
        src[0].width = 0;
        src[1].schar = 'b' as schar_T;
        let mut dst = blank_row(2);
        import_row(&src, &mut dst, false, &erased_pen(black(), black()));
        assert_eq!(dst[1].schar, 'b' as schar_T);
    }

    #[test]
    fn the_protection_bit_does_not_survive_scrollback() {
        let mut pen = decorated_pen();
        pen.set_protected_cell(1);
        pen.set_dwl(1);

        let mut cell = blank_cell();
        export_pen(&pen, false, &mut cell);
        let mut back = erased_pen(black(), black());
        import_pen(&cell, false, &mut back);

        assert_eq!(back.protected_cell(), 0);
        assert_eq!(back.dwl(), 0);
    }
}
