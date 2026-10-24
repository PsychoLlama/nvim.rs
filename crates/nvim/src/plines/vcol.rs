#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

//! Virtual columns of a position.
//!
//! The `getvcol` family: where a `Pos` lands on screen, once tabs, inline
//! virtual text, double-width characters and 'virtualedit' have had their
//! say. All of it walks the line with the parent module's charsize
//! functions.

use super::*;
use crate::charset::ptr2cells_at;
use crate::option::vars::P_SEL;

use crate::normal::{visual_active, visual_anchor};
use crate::winlayer::PosRef;

/// Whether byte offset `at` lies past column `end_col`; every offset is past
/// a negative one.
#[inline(always)]
fn past_end(at: usize, end_col: ColNr) -> bool {
    usize::try_from(end_col).map_or(true, |end| at > end)
}

/// Virtual columns of `pos`: the first column the character occupies,
/// where the cursor sits on it (the last column of a tab, in Normal mode
/// without 'list'), and the last column it occupies.
///
/// Called very often; keep it fast. A `pos.col` past the end of the line is
/// clamped to the line length on the way out.
pub(crate) fn getvcol(window: Win, pos: &mut Pos) -> (ColNr, ColNr, ColNr) {
    let mut lines = window.buffer().lines();
    let line = lines.line(pos.lnum);
    let end_col = pos.col;

    let (mut csarg, cstype) = CharsizeArg::new(window, pos.lnum, line);
    csarg.max_head_vcol = -1;

    let mut on_nul = false;
    let mut vcol: ColNr = 0;
    let mut char_size;
    let mut at = 0;
    let mut chr = c_char_info(line, 0);

    if cstype == CharsizeKind::Fast {
        let use_tabstop = csarg.use_tabstop;
        loop {
            if ends_at(line, at, chr) {
                // The cursor on a NUL is treated like a one-cell char.
                char_size = CharSize { width: 1, head: 0 };
                break;
            }
            char_size = charsize_fast_impl(window, line, at, use_tabstop, vcol, chr.value);
            let (next, next_chr) = c_next_cluster(line, at, chr);
            if past_end(next, end_col) {
                break;
            }
            (at, chr) = (next, next_chr);
            vcol += char_size.width;
        }
    } else {
        loop {
            char_size = charsize_regular(&mut csarg, at, vcol, chr.value);
            // Don't go past the end of the line.
            if ends_at(line, at, chr) {
                // A NUL at the end of the line takes one column, unless
                // there is virtual text.
                char_size.width = 1 + csarg.cur_text_width_left + csarg.cur_text_width_right;
                on_nul = true;
                break;
            }
            let (next, next_chr) = c_next_cluster(line, at, chr);
            if past_end(next, end_col) {
                break;
            }
            (at, chr) = (next, next_chr);
            vcol += char_size.width;
        }
    }

    if byte_at(line, at) == b'\0'
        && end_col < MAXCOL
        && !past_end(at, end_col)
        && usize::try_from(end_col) != Ok(at)
    {
        pos.col = ColNr::try_from(at).unwrap_or(ColNr::MAX);
    }

    let head = char_size.head;
    let incr = char_size.width;

    let start = vcol + head;
    let end = vcol + incr - 1;
    let cursor_at_tab_end = chr.value == TAB
        && State.get() & MODE_NORMAL != 0
        && window.w_onebuf_opt.wo_list == 0
        && !virtual_active(window)
        && !(visual_active() && (P_SEL.first_byte() == b'e' || ltoreq(*pos, visual_anchor())));
    let cursor = if cursor_at_tab_end {
        vcol + incr - 1
    } else {
        vcol + virt_text_cursor_off(&csarg, on_nul) + head
    };
    (start, cursor, end)
}

/// Virtual cursor column in the current window, pretending 'list' is off.
pub(crate) fn getvcol_nolist(pos: &mut Pos) -> ColNr {
    let mut win = Win::current();
    let list_save = win.w_onebuf_opt.wo_list;

    win.w_onebuf_opt.wo_list = 0;
    let vcol = if pos.coladd != 0 {
        getvvcol(win, pos).1
    } else {
        getvcol(win, pos).1
    };
    win.w_onebuf_opt.wo_list = list_save;
    vcol
}

/// [`getvcol`] in virtual-edit mode, where the cursor can sit past the end of
/// a line or inside a tab.
pub(crate) fn getvvcol(window: Win, pos: &mut Pos) -> (ColNr, ColNr, ColNr) {
    if !virtual_active(window) {
        return getvcol(window, pos);
    }

    // In virtual mode only one value is wanted.
    let mut col = getvcol(window, pos).0;

    let mut coladd = pos.coladd;
    let mut endadd: ColNr = 0;

    // The cursor cannot sit on part of a wide character.
    let mut lines = window.buffer().lines();
    let line = lines.line(pos.lnum);
    if let Some(text) = usize::try_from(pos.col)
        .ok()
        .and_then(|at| line.get(at..))
        .filter(|t| !t.is_empty())
    {
        let c = char_at(text);
        if c != TAB && vim_isprintc(c) {
            endadd = ptr2cells_at(text) - 1;
            if coladd > endadd {
                // Past the end of the line.
                endadd = 0;
            } else {
                coladd = 0;
            }
        }
    }
    col += coladd;
    (col, col, col + endadd)
}

/// Leftmost and rightmost virtual column of `pos1` and `pos2`, for Visual
/// block mode.
pub(crate) fn getvcols(window: Win, pos1: &mut Pos, pos2: &mut Pos) -> (ColNr, ColNr) {
    let (first, second) = if lt(*pos1, *pos2) {
        (pos1, pos2)
    } else {
        (pos2, pos1)
    };

    let (from1, _, to1) = getvvcol(window, first);
    let (from2, _, to2) = getvvcol(window, second);

    let left = from1.min(from2);
    // With 'selection' exclusive the block stops one column short of the
    // second position -- but only when that still leaves the first one's
    // last column inside the block.
    let before_second = from2 - 1;
    let exclusive = P_SEL.first_byte() == b'e';
    let right = if to2 > to1 {
        if exclusive && before_second >= to1 {
            before_second
        } else {
            to2
        }
    } else {
        to1
    };
    (left, right)
}

/// [`getvcol`]/[`getvvcol`] of the position `pos` names.
///
/// The position is copied out and the copy written back: `pos` is most
/// often the window's own cursor, and the walk reads the window while it
/// runs, so the window may not be lent out mutably for its duration. The
/// write-back keeps the clamp of a column past the end of the line.
#[inline(always)]
fn columns(win: Win, mut pos: PosRef, virtual_edit: bool) -> (ColNr, ColNr, ColNr) {
    let mut at = *pos;
    // Two arms rather than one call through a function pointer: an indirect
    // call here is not inlined, and `getvcol` is on the cursor and statusline
    // paths -- it cost 1.3% of `scrbench` when this was `let get = if ...`.
    let cols = if virtual_edit {
        getvvcol(win, &mut at)
    } else {
        getvcol(win, &mut at)
    };
    *pos = at;
    cols
}

/// The virtual-column questions a caller asks *of a window*, as [`Win`]
/// methods.
///
/// They live beside [`getvcol`] rather than in `winlayer` for the reason that
/// module's docs give: a family hangs its own projections off `impl Win`, so
/// the shared module stays the minimum.
impl Win {
    /// First and last virtual column of the character at `pos`.
    #[inline(always)]
    pub fn vcol_span(self, pos: PosRef) -> (ColNr, ColNr) {
        let (start, _, end) = columns(self, pos, false);
        (start, end)
    }

    /// Start, cursor and end virtual column of the character at `pos`.
    #[inline(always)]
    pub fn vcol_triple(self, pos: PosRef) -> (ColNr, ColNr, ColNr) {
        columns(self, pos, false)
    }

    /// The first virtual column of the character at `pos`.
    #[inline(always)]
    pub fn vcol(self, pos: PosRef) -> ColNr {
        self.vcol_span(pos).0
    }

    /// [`Win::vcol_span`] with 'virtualedit' taken into account.
    #[inline(always)]
    pub fn virtual_vcol_span(self, pos: PosRef) -> (ColNr, ColNr) {
        let (start, _, end) = columns(self, pos, true);
        (start, end)
    }

    /// [`Win::virtual_vcol_span`] of a position held by value.
    pub(crate) fn virtual_vcol_span_at(self, mut pos: Pos) -> (ColNr, ColNr) {
        let (start, _, end) = getvvcol(self, &mut pos);
        (start, end)
    }

    /// [`Win::vcol_triple`] with 'virtualedit' taken into account.
    #[inline(always)]
    pub fn virtual_vcol_triple(self, pos: PosRef) -> (ColNr, ColNr, ColNr) {
        columns(self, pos, true)
    }

    /// The first virtual column of the character at `pos`, 'virtualedit'
    /// included.
    #[inline(always)]
    pub fn virtual_vcol(self, pos: PosRef) -> ColNr {
        self.virtual_vcol_span(pos).0
    }

    /// The virtual column the *cursor* shows at within the character at
    /// `pos`, which is not its first column when the character is a tab.
    #[inline(always)]
    pub fn virtual_cursor_vcol(self, pos: PosRef) -> ColNr {
        columns(self, pos, true).1
    }
}
