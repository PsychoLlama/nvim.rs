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
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::grid::{schar_from_ascii, schar_from_char};
use crate::mbyte::Codepoint;
use crate::types::{ScreenAttr, ScreenChar};

/// An empty cell: never drawn into, or the right half of a double-width
/// character whose left half carries the whole thing.
pub(super) const EMPTY_CELL: ScreenChar = 0;

/// The default grid's own text and attributes for the row being composed.
pub(super) type Backdrop<'a> = (&'a [ScreenChar], &'a [ScreenAttr]);

/// A half-open run of columns within the scratch line.
pub(super) type Cells = core::ops::Range<usize>;

/// The scratch line, kept at the width of the default grid by
/// `ui_comp_grid_resize`.
#[derive(Debug)]
pub(super) struct Bufs {
    pub(super) chars: Vec<ScreenChar>,
    pub(super) attrs: Vec<ScreenAttr>,
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
    line: &mut [ScreenChar],
    attrbuf: &mut [ScreenAttr],
    bg: Backdrop<'_>,
    cells: Cells,
    end: usize,
    mut blend_attrs: impl FnMut(ScreenAttr, ScreenAttr, &mut bool) -> ScreenAttr,
) {
    let (bg_line, bg_attrs) = bg;
    let blank = schar_from_ascii(b' ');
    let braille = schar_from_char(Codepoint::new(0x2800));
    let mut i = cells.start;
    while i < cells.end {
        let mut width = 1;
        let mut thru = (line[i] == blank || line[i] == braille) && bg_line[i] != EMPTY_CELL;
        if i + 1 < end && bg_line[i + 1] == EMPTY_CELL {
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
pub(super) fn clear_invalid_attrs(attrbuf: &mut [ScreenAttr], cells: Cells, fatal: bool) {
    for attr in &mut attrbuf[cells] {
        if *attr < 0 {
            if fatal {
                std::process::abort();
            }
            *attr = 0;
        }
    }
}

/// The two passes over the scratch line, on hand-built rows: which cells of
/// a blended layer let the backdrop through, how a double-width backdrop
/// character is taken as a pair, where the lookahead stops, and which
/// attributes the never-drawn sweep rewrites. The blend of two attributes
/// is a stand-in that records its arguments, so the cases pin what
/// [`blend`] asks for, not what the highlight table answers.
#[cfg(test)]
mod tests {
    use super::*;

    const BLANK: ScreenChar = schar_from_ascii(b' ');

    fn text(s: &str) -> Vec<ScreenChar> {
        s.bytes().map(schar_from_ascii).collect()
    }

    /// `back * 100 + front`, logging each call; `veto` names backdrop
    /// attributes whose blend refuses to let the character through.
    fn blend_log<'a>(
        log: &'a mut Vec<(ScreenAttr, ScreenAttr)>,
        veto: &'a [ScreenAttr],
    ) -> impl FnMut(ScreenAttr, ScreenAttr, &mut bool) -> ScreenAttr + 'a {
        move |back, front, thru| {
            log.push((back, front));
            if veto.contains(&back) {
                *thru = false;
            }
            back * 100 + front
        }
    }

    #[test]
    fn a_blank_layer_cell_shows_the_backdrop_character() {
        let mut line = text("a b");
        let mut attrs = vec![1, 2, 3];
        let (bg, bg_attrs) = (text("xyz"), vec![7, 8, 9]);
        let mut log = Vec::new();
        blend(
            &mut line,
            &mut attrs,
            (&bg, &bg_attrs),
            0..3,
            3,
            blend_log(&mut log, &[]),
        );
        assert_eq!(line, text("ayb"), "only the blank cell lets 'y' through");
        assert_eq!(attrs, vec![701, 802, 903], "every attribute is blended");
        assert_eq!(log, vec![(7, 1), (8, 2), (9, 3)]);
    }

    #[test]
    fn braille_blank_is_negative_space_too() {
        let braille = schar_from_char(Codepoint::new(0x2800));
        let mut line = vec![braille, schar_from_ascii(b'q')];
        let mut attrs = vec![0, 0];
        let (bg, bg_attrs) = (text("xy"), vec![0, 0]);
        blend(
            &mut line,
            &mut attrs,
            (&bg, &bg_attrs),
            0..2,
            2,
            |_, front, _| front,
        );
        assert_eq!(line, text("xq"));
    }

    #[test]
    fn a_never_drawn_backdrop_cell_is_not_shown_through() {
        let mut line = vec![BLANK, BLANK];
        let mut attrs = vec![0, 0];
        let (bg, bg_attrs) = (vec![EMPTY_CELL, schar_from_ascii(b'y')], vec![5, 6]);
        let mut log = Vec::new();
        // `end` is 1: no lookahead, so cell 0 is a single cell whose backdrop
        // was never drawn.
        blend(
            &mut line,
            &mut attrs,
            (&bg, &bg_attrs),
            0..1,
            1,
            blend_log(&mut log, &[]),
        );
        assert_eq!(line, vec![BLANK, BLANK]);
        assert_eq!(log, vec![(5, 0)]);
    }

    #[test]
    fn a_double_width_backdrop_character_is_taken_as_a_pair() {
        // The backdrop holds a wide character at 1 (its right half, 2, is the
        // empty cell). The layer is blank over both halves, so both are
        // copied, and both attributes blended, in one step of two.
        let wide = schar_from_char(Codepoint::new(0x4e00));
        let mut line = vec![schar_from_ascii(b'a'), BLANK, BLANK, BLANK];
        let mut attrs = vec![1, 2, 3, 4];
        let bg = vec![
            schar_from_ascii(b'x'),
            wide,
            EMPTY_CELL,
            schar_from_ascii(b'z'),
        ];
        let bg_attrs = vec![10, 20, 30, 40];
        let mut log = Vec::new();
        blend(
            &mut line,
            &mut attrs,
            (&bg, &bg_attrs),
            0..4,
            4,
            blend_log(&mut log, &[]),
        );
        assert_eq!(
            line,
            vec![
                schar_from_ascii(b'a'),
                wide,
                EMPTY_CELL,
                schar_from_ascii(b'z')
            ]
        );
        assert_eq!(attrs, vec![1001, 2002, 3003, 4004]);
        assert_eq!(log, vec![(10, 1), (20, 2), (30, 3), (40, 4)]);
    }

    #[test]
    fn a_wide_backdrop_shows_through_only_if_both_halves_are_blank() {
        let wide = schar_from_char(Codepoint::new(0x4e00));
        let mut line = vec![BLANK, schar_from_ascii(b'q')];
        let mut attrs = vec![0, 0];
        let (bg, bg_attrs) = (vec![wide, EMPTY_CELL], vec![0, 0]);
        blend(
            &mut line,
            &mut attrs,
            (&bg, &bg_attrs),
            0..2,
            2,
            |_, front, _| front,
        );
        assert_eq!(line, vec![BLANK, schar_from_ascii(b'q')]);
    }

    #[test]
    fn the_pair_lookahead_stops_at_end_but_may_pass_the_range() {
        let wide = schar_from_char(Codepoint::new(0x4e00));
        let bg = vec![wide, EMPTY_CELL];
        let bg_attrs = vec![10, 20];
        // `end` 1: the right half is out of reach, so cell 0 is single.
        let (mut line, mut attrs, mut log) = (vec![BLANK, BLANK], vec![1, 2], Vec::new());
        blend(
            &mut line,
            &mut attrs,
            (&bg, &bg_attrs),
            0..1,
            1,
            blend_log(&mut log, &[]),
        );
        assert_eq!(line, vec![wide, BLANK]);
        assert_eq!(attrs, vec![1001, 2]);
        // `end` 2 with the range still 0..1: the pair reaches one past it.
        let (mut line, mut attrs, mut log) = (vec![BLANK, BLANK], vec![1, 2], Vec::new());
        blend(
            &mut line,
            &mut attrs,
            (&bg, &bg_attrs),
            0..1,
            2,
            blend_log(&mut log, &[]),
        );
        assert_eq!(line, vec![wide, EMPTY_CELL]);
        assert_eq!(attrs, vec![1001, 2002]);
    }

    #[test]
    fn the_attribute_blend_can_refuse_the_backdrop() {
        let mut line = vec![BLANK, BLANK, BLANK];
        let mut attrs = vec![1, 2, 3];
        let (bg, bg_attrs) = (text("xyz"), vec![7, 8, 9]);
        let mut log = Vec::new();
        blend(
            &mut line,
            &mut attrs,
            (&bg, &bg_attrs),
            1..3,
            3,
            blend_log(&mut log, &[8]),
        );
        assert_eq!(line, vec![BLANK, BLANK, schar_from_ascii(b'z')]);
        assert_eq!(attrs, vec![1, 802, 903], "cell 0 is outside the range");
    }

    #[test]
    fn never_drawn_attributes_become_the_default_inside_the_range_only() {
        let mut attrs = vec![-1, -5, 3, -2, 0];
        clear_invalid_attrs(&mut attrs, 1..4, false);
        assert_eq!(attrs, vec![-1, 0, 3, 0, 0]);
    }
}
