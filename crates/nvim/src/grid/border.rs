#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! The border drawn around a floating window.
//!
//! Eight glyphs -- four corners and four edges, indices 0..8 clockwise from
//! the top left -- plus an optional title and footer laid into the top and
//! bottom edges. The `adj` array says which of the four sides the window
//! actually has, in the order top, right, bottom, left.

use super::*;
use crate::cstr;
use crate::highlight_group::{HLF_BFOOTER, HLF_BTITLE};
use crate::mbyte::{c_cells_at, c_cluster_len, c_string_cells};
use crate::statusline::hl_attr;

/// Draw one of the two border texts into the batch in progress.
///
/// `overflow` is how many cells the text is wider than the space available;
/// that many are dropped from the front and a `<` marks the truncation.
/// `default_attr` is what a chunk with no highlight of its own gets.
fn grid_draw_bordertext(
    vt: &VirtText,
    mut col: c_int,
    winbl: c_int,
    default_attr: c_int,
    mut overflow: c_int,
) {
    if overflow > 0 {
        grid_line_puts(1, b"<", hl_apply_winblend(winbl, default_attr));
        col += 1;
        overflow += 1;
    }

    let mut i: size_t = 0;
    while i < vt.size {
        let mut attr = -1;
        let Some(text) = next_virt_text_chunk(vt, &mut i, &mut attr) else {
            break;
        };
        let mut text = text.to_bytes();
        if attr == -1 {
            // No highlight specified.
            attr = default_attr;
        }

        // Skip characters from the beginning when the text overflows.
        if overflow > 0 {
            let cells = c_string_cells(text);
            let cells = c_int::try_from(cells).expect("a chunk is never wider than an int");
            if overflow >= cells {
                // The whole chunk is off the left edge.
                overflow -= cells;
                continue;
            }
            // Skip partial characters within the chunk.
            let mut at = 0;
            while at < text.len() && overflow > 0 {
                overflow -= c_cells_at(&text[at..]);
                at += c_cluster_len(&text[at..]);
            }
            text = &text[at..];
        }

        col += grid_line_puts(col, text, hl_apply_winblend(winbl, attr));
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
/// `adj` is the four sides the window has (top, right, bottom, left); `None`
/// means all four. The title and footer take their default highlight from
/// `hl_attr`, an `HLF_*` table -- a window's own namespace table -- or from
/// the active one when there is none. No line batch may be in progress.
pub fn grid_draw_border(
    grid: GridRef,
    config: &WinConfig,
    adj: Option<[c_int; 4]>,
    winbl: c_int,
    hl_attr_table: Option<&[c_int]>,
) {
    let adj = adj.unwrap_or([1, 1, 1, 1]);
    let attrs = config.border_attr;
    let side = |i: usize| adj[i] != 0;
    let text_attr = |hlf: c_int| {
        hl_attr_table.map_or_else(
            || hl_attr(hlf),
            |table| table[usize::try_from(hlf).expect("an HLF_* index")],
        )
    };

    let chars: [ScreenChar; 8] =
        core::array::from_fn(|i| schar_from_str(cstr::in_chars(&config.border_chars[i])));

    // Interior size, i.e. the window minus whichever sides it has.
    let irow = grid.rows - adj[0] - adj[2];
    let icol = grid.cols - adj[1] - adj[3];

    if side(0) {
        screengrid_line_start(grid, 0, 0);
        if side(3) {
            grid_line_put_schar(0, chars[0], attrs[0]);
        }
        let mut i = 0;
        while i < icol {
            grid_line_put_schar(i + adj[3], chars[1], attrs[1]);
            i += 1;
        }
        if config.title {
            let title_col = get_bordertext_col(icol, config.title_width, config.title_pos);
            grid_draw_bordertext(
                &config.title_chunks,
                title_col,
                winbl,
                text_attr(HLF_BTITLE),
                config.title_width - icol,
            );
        }
        if side(1) {
            grid_line_put_schar(icol + adj[3], chars[2], attrs[2]);
        }
        grid_line_flush();
    }

    let mut i = 0;
    while i < irow {
        if side(3) {
            screengrid_line_start(grid, i + adj[0], 0);
            grid_line_put_schar(0, chars[7], attrs[7]);
            grid_line_flush();
        }
        if side(1) {
            // With no top edge, the first row's right cell is the corner.
            let ic: usize = if i == 0 && !side(0) && chars[2] != 0 {
                2
            } else {
                3
            };
            screengrid_line_start(grid, i + adj[0], 0);
            grid_line_put_schar(icol + adj[3], chars[ic], attrs[ic]);
            grid_line_flush();
        }
        i += 1;
    }

    if side(2) {
        screengrid_line_start(grid, irow + adj[0], 0);
        if side(3) {
            grid_line_put_schar(0, chars[6], attrs[6]);
        }
        let mut i = 0;
        while i < icol {
            // With no left edge, the first column is the corner.
            let ic: usize = if i == 0 && !side(3) && chars[6] != 0 {
                6
            } else {
                5
            };
            grid_line_put_schar(i + adj[3], chars[ic], attrs[ic]);
            i += 1;
        }
        if config.footer {
            let footer_col = get_bordertext_col(icol, config.footer_width, config.footer_pos);
            grid_draw_bordertext(
                &config.footer_chunks,
                footer_col,
                winbl,
                text_attr(HLF_BFOOTER),
                config.footer_width - icol,
            );
        }
        if side(1) {
            grid_line_put_schar(icol + adj[3], chars[4], attrs[4]);
        }
        grid_line_flush();
    }
}
