//! The compositor's scratch line.
//!
//! `compose_line` flattens one screen row into these two parallel buffers
//! before handing them to the UI. The passes over it that need no grid
//! pointer live here, where they can be plain safe code: the
//! `'winblend'`/`'pumblend'` blend against the backdrop, and the sweep that
//! turns a never-drawn attribute into the default.
//!
//! Derived from Neovim's `src/nvim/ui_compositor.c`. Copyright Neovim
//! contributors; licensed under the Apache License, Version 2.0, as recorded
//! in `LICENSE.txt`.

#![forbid(unsafe_code)]

use crate::grid::{schar_from_ascii, schar_from_char};
use crate::types::{sattr_T, schar_T};

/// An empty cell: never drawn into, or the right half of a double-width
/// character whose left half carries the whole thing.
pub(super) const NUL: schar_T = 0;

/// The default grid's own text and attributes for the row being composed.
pub(super) type Backdrop<'a> = (&'a [schar_T], &'a [sattr_T]);

/// A half-open run of columns within the scratch line.
pub(super) type Cells = core::ops::Range<usize>;

/// The scratch line, kept at the width of the default grid by
/// `ui_comp_grid_resize`.
pub(super) struct Bufs {
    pub(super) chars: Vec<schar_T>,
    pub(super) attrs: Vec<sattr_T>,
}

impl Bufs {
    /// What the buffers hold before the first resize, and again once the
    /// last composed UI detaches.
    pub(super) const EMPTY: Bufs = Bufs {
        chars: Vec::new(),
        attrs: Vec::new(),
    };
}

/// Blends `cells` of the scratch line against the backdrop `bg`, for
/// `'winblend'` and `'pumblend'`.
///
/// A blank cell of the layer is *negative space*: the backdrop shows through
/// it whole, character included. `end` bounds the double-width lookahead,
/// which reaches one cell past the range. `blend_attrs` is C's
/// `hl_blend_attrs`, which this module may not call for itself.
pub(super) fn blend(
    line: &mut [schar_T],
    attrbuf: &mut [sattr_T],
    bg: Backdrop<'_>,
    cells: Cells,
    end: usize,
    mut blend_attrs: impl FnMut(sattr_T, sattr_T, &mut bool) -> sattr_T,
) {
    let (bg_line, bg_attrs) = bg;
    let blank = schar_from_ascii(b' ');
    let braille = schar_from_char(0x2800);
    let mut i = cells.start;
    while i < cells.end {
        let mut width = 1;
        let mut thru = (line[i] == blank || line[i] == braille) && bg_line[i] != NUL;
        if i + 1 < end && bg_line[i + 1] == NUL {
            width = 2;
            thru &= line[i + 1] == blank || line[i + 1] == braille;
        }
        attrbuf[i] = blend_attrs(bg_attrs[i], attrbuf[i], &mut thru);
        if width == 2 {
            attrbuf[i + 1] = blend_attrs(bg_attrs[i + 1], attrbuf[i + 1], &mut thru);
        }
        if thru {
            line[i..i + width].copy_from_slice(&bg_line[i..i + width]);
        }
        i += width;
    }
}

/// Replaces the never-drawn attributes in `cells` with the default: they are
/// negative, which the downstream UIs cannot express. `fatal` is
/// `'redrawdebug'`'s `invalid` flag, which makes one an abort instead.
pub(super) fn clear_invalid_attrs(attrbuf: &mut [sattr_T], cells: Cells, fatal: bool) {
    for attr in &mut attrbuf[cells] {
        if *attr < 0 {
            if fatal {
                std::process::abort();
            }
            *attr = 0;
        }
    }
}
