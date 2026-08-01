#![deny(unsafe_op_in_unsafe_fn)]

//! The border drawn around a floating window.
//!
//! Eight glyphs -- four corners and four edges, indices 0..8 clockwise from
//! the top left -- plus an optional title and footer laid into the top and
//! bottom edges. The `adj` array says which of the four sides the window
//! actually has, in the order top, right, bottom, left.

use super::*;

/// Draw one of the two border texts into the batch in progress.
///
/// `overflow` is how many cells the text is wider than the space available;
/// that many are dropped from the front and a `<` marks the truncation.
///
/// # Safety
/// A line batch must be in progress and `hl_attr` must point to the
/// highlight-attribute table.
unsafe fn grid_draw_bordertext(
    vt: VirtText,
    mut col: c_int,
    winbl: c_int,
    hl_attr: *const c_int,
    bt: BorderTextType,
    mut overflow: c_int,
) {
    unsafe {
        let default_attr = *hl_attr.offset(if bt == kBorderTextTitle {
            HLF_BTITLE as isize
        } else {
            HLF_BFOOTER as isize
        });

        if overflow > 0 {
            grid_line_puts(1, c"<".as_ptr(), -1, hl_apply_winblend(winbl, default_attr));
            col += 1;
            overflow += 1;
        }

        let mut i: size_t = 0;
        while i < vt.size {
            let mut attr = -1;
            let mut text = next_virt_text_chunk(vt, &raw mut i, &raw mut attr);
            if text.is_null() {
                break;
            }
            if attr == -1 {
                // No highlight specified.
                attr = default_attr;
            }

            // Skip characters from the beginning when the text overflows.
            if overflow > 0 {
                let cells = mb_string2cells(text) as c_int;
                if overflow >= cells {
                    // The whole chunk is off the left edge.
                    overflow -= cells;
                    continue;
                }
                // Skip partial characters within the chunk.
                let mut p = text;
                while *p != NUL && overflow > 0 {
                    overflow -= utf_ptr2cells(p);
                    p = p.offset(utfc_ptr2len(p) as isize);
                }
                text = p;
            }

            col += grid_line_puts(col, text, -1, hl_apply_winblend(winbl, attr));
        }
    }
}

/// Where a border text of `text_width` cells starts, within `total_col`
/// cells of edge.
fn get_bordertext_col(total_col: c_int, text_width: c_int, align: AlignTextPos) -> c_int {
    match align {
        kAlignLeft => 1,
        kAlignCenter => ((total_col - text_width) / 2 + 1).max(1),
        kAlignRight => (total_col - text_width + 1).max(1),
        _ => unreachable!(),
    }
}

/// Draw the border on a floating window's grid.
///
/// `adj` is the four sides the window has (top, right, bottom, left); null
/// means all four. Null `hl_attr` means the active highlight table.
///
/// # Safety
/// `grid` and `config` must be live and no line batch may be in progress.
pub unsafe fn grid_draw_border(
    grid: *mut ScreenGrid,
    config: *mut WinConfig,
    adj: *mut c_int,
    winbl: c_int,
    hl_attr: *mut c_int,
) {
    unsafe {
        let attrs = (&raw mut (*config).border_attr).cast::<c_int>();
        let mut default_adj: [c_int; 4] = [1, 1, 1, 1];
        let adj = if adj.is_null() {
            default_adj.as_mut_ptr()
        } else {
            adj
        };
        let hl_attr = if hl_attr.is_null() {
            hl_attr_active.get()
        } else {
            hl_attr
        };
        let side = |i: isize| *adj.offset(i) != 0;

        let mut chars: [schar_T; 8] = [0; 8];
        for (i, ch) in chars.iter_mut().enumerate() {
            *ch = schar_from_str(
                (&raw mut (*config).border_chars)
                    .cast::<[c_char; 32]>()
                    .add(i)
                    .cast::<c_char>(),
            );
        }

        // Interior size, i.e. the window minus whichever sides it has.
        let irow = (*grid).rows - *adj.offset(0) - *adj.offset(2);
        let icol = (*grid).cols - *adj.offset(1) - *adj.offset(3);

        if side(0) {
            screengrid_line_start(grid, 0, 0);
            if side(3) {
                grid_line_put_schar(0, chars[0], *attrs.offset(0));
            }
            let mut i = 0;
            while i < icol {
                grid_line_put_schar(i + *adj.offset(3), chars[1], *attrs.offset(1));
                i += 1;
            }
            if (*config).title {
                let title_col =
                    get_bordertext_col(icol, (*config).title_width, (*config).title_pos);
                grid_draw_bordertext(
                    (*config).title_chunks,
                    title_col,
                    winbl,
                    hl_attr,
                    kBorderTextTitle,
                    (*config).title_width - icol,
                );
            }
            if side(1) {
                grid_line_put_schar(icol + *adj.offset(3), chars[2], *attrs.offset(2));
            }
            grid_line_flush();
        }

        let mut i = 0;
        while i < irow {
            if side(3) {
                screengrid_line_start(grid, i + *adj.offset(0), 0);
                grid_line_put_schar(0, chars[7], *attrs.offset(7));
                grid_line_flush();
            }
            if side(1) {
                // With no top edge, the first row's right cell is the corner.
                let ic: isize = if i == 0 && !side(0) && chars[2] != 0 {
                    2
                } else {
                    3
                };
                screengrid_line_start(grid, i + *adj.offset(0), 0);
                grid_line_put_schar(icol + *adj.offset(3), chars[ic as usize], *attrs.offset(ic));
                grid_line_flush();
            }
            i += 1;
        }

        if side(2) {
            screengrid_line_start(grid, irow + *adj.offset(0), 0);
            if side(3) {
                grid_line_put_schar(0, chars[6], *attrs.offset(6));
            }
            let mut i = 0;
            while i < icol {
                // With no left edge, the first column is the corner.
                let ic: isize = if i == 0 && !side(3) && chars[6] != 0 {
                    6
                } else {
                    5
                };
                grid_line_put_schar(i + *adj.offset(3), chars[ic as usize], *attrs.offset(ic));
                i += 1;
            }
            if (*config).footer {
                let footer_col =
                    get_bordertext_col(icol, (*config).footer_width, (*config).footer_pos);
                grid_draw_bordertext(
                    (*config).footer_chunks,
                    footer_col,
                    winbl,
                    hl_attr,
                    kBorderTextFooter,
                    (*config).footer_width - icol,
                );
            }
            if side(1) {
                grid_line_put_schar(icol + *adj.offset(3), chars[4], *attrs.offset(4));
            }
            grid_line_flush();
        }
    }
}
