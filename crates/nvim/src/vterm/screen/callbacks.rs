//! What the state machine reports to the screen it paints on.
//!
//! Split out of [`super`] for the file-size cap. Each of these is one slot of
//! libvterm's `VTermStateCallbacks` as the screen fills it: the state calls
//! them with the line marks they need, and the ones the host also wants to
//! see are passed on to it.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use core::ffi::c_int;

use crate::types::{
    VTermAttr, VTermGlyphInfo, VTermLineInfo, VTermPos, VTermProp, VTermRect, VTermScreen,
};
use crate::vterm::cell::SCHAR_CONTINUATION;
use crate::vterm::host::Value;
use crate::vterm::vterm::{
    VTERM_ATTR_BACKGROUND, VTERM_ATTR_BASELINE, VTERM_ATTR_BLINK, VTERM_ATTR_BOLD,
    VTERM_ATTR_CONCEAL, VTERM_ATTR_DIM, VTERM_ATTR_FONT, VTERM_ATTR_FOREGROUND, VTERM_ATTR_ITALIC,
    VTERM_ATTR_OVERLINE, VTERM_ATTR_REVERSE, VTERM_ATTR_SMALL, VTERM_ATTR_STRIKE,
    VTERM_ATTR_UNDERLINE, VTERM_ATTR_URI, VTERM_PROP_ALTSCREEN, VTERM_PROP_REVERSE,
};

impl VTermScreen {
    /// Paints one glyph, marking the cells a wide one covers.
    pub(crate) fn put_glyph(&mut self, info: &VTermGlyphInfo, pos: VTermPos) -> c_int {
        let pen = self.pen;
        let Some(cell) = self.cell_mut(pos.row, pos.col) else {
            return 0;
        };
        cell.schar = info.schar;
        // An erasing glyph keeps the cell's existing pen.
        if info.schar != 0 {
            cell.pen = pen;
        }
        cell.pen.set_protected_cell(info.protected_cell());
        cell.pen.set_dwl(info.dwl());
        cell.pen.set_dhl(info.dhl());
        // A wide glyph can reach the last column with no cell to continue
        // into: on a one-column grid there is no next column for the state to
        // wrap it away into first, so it is placed and its continuation runs
        // off the end. Truncate the glyph to the cells that exist, which is
        // what the caller's clipped damage rect already claims.
        for col in 1..info.width {
            let Some(continuation) = self.cell_mut(pos.row, pos.col + col) else {
                break;
            };
            continuation.schar = SCHAR_CONTINUATION;
        }
        self.damage(VTermRect {
            start_row: pos.row,
            end_row: pos.row + 1,
            start_col: pos.col,
            end_col: pos.col + info.width,
        });
        1
    }

    /// The cursor moved; the host is told, the screen keeps nothing of it.
    pub(crate) fn movecursor(&mut self, pos: VTermPos, oldpos: VTermPos, visible: c_int) -> c_int {
        self.host()
            .and_then(|host| host.movecursor(pos, oldpos, visible))
            .unwrap_or(0)
    }

    /// Applies one SGR attribute to the pen the screen paints with.
    pub(crate) fn set_pen_attr(&mut self, attr: VTermAttr, val: &Value<'_>) -> c_int {
        let number = || val.int().cast_unsigned();
        let pen = &mut self.pen;
        match attr {
            VTERM_ATTR_BOLD => pen.set_bold(number()),
            VTERM_ATTR_UNDERLINE => pen.set_underline(number()),
            VTERM_ATTR_ITALIC => pen.set_italic(number()),
            VTERM_ATTR_BLINK => pen.set_blink(number()),
            VTERM_ATTR_REVERSE => pen.set_reverse(number()),
            VTERM_ATTR_CONCEAL => pen.set_conceal(number()),
            VTERM_ATTR_STRIKE => pen.set_strike(number()),
            VTERM_ATTR_FONT => pen.set_font(number()),
            VTERM_ATTR_FOREGROUND => pen.fg = val.color(),
            VTERM_ATTR_BACKGROUND => pen.bg = val.color(),
            VTERM_ATTR_SMALL => pen.set_small(number()),
            VTERM_ATTR_BASELINE => pen.set_baseline(number()),
            VTERM_ATTR_URI => pen.uri = val.int(),
            VTERM_ATTR_DIM => pen.set_dim(number()),
            VTERM_ATTR_OVERLINE => pen.set_overline(number()),
            _ => return 0,
        }
        1
    }

    /// Acts on the two terminal properties the screen itself keeps, then
    /// passes every property on to the host.
    pub(crate) fn set_termprop(&mut self, prop: VTermProp, val: &Value<'_>) -> c_int {
        let boolean = val.int();
        match prop {
            VTERM_PROP_ALTSCREEN => {
                let want_altscreen = boolean != 0;
                if want_altscreen && !self.owned.has_altscreen {
                    return 0;
                }
                self.owned.active = if want_altscreen {
                    super::BUFIDX_ALTSCREEN
                } else {
                    super::BUFIDX_PRIMARY
                };
                // Only on disable: enabling is followed by an erase, which
                // reports the damage anyway.
                if !want_altscreen {
                    self.damage_screen();
                }
            }
            VTERM_PROP_REVERSE => {
                self.set_global_reverse(boolean.cast_unsigned());
                self.damage_screen();
            }
            _ => {}
        }
        self.host()
            .and_then(|host| host.settermprop(prop, val))
            .unwrap_or(1)
    }

    /// A line's double-width or double-height mark changed: restamp the row's
    /// cells and report the damage. Going double-width halves the usable row,
    /// so the right half is erased outright. `lines` are the state's marks,
    /// still the old ones.
    pub(crate) fn set_lineinfo(
        &mut self,
        row: c_int,
        new: &VTermLineInfo,
        old: &VTermLineInfo,
        state_rows: c_int,
        lines: &[VTermLineInfo],
    ) -> c_int {
        if new.doublewidth() == old.doublewidth() && new.doubleheight() == old.doubleheight() {
            return 1;
        }
        let (dwl, dhl) = (new.doublewidth(), new.doubleheight());
        for col in 0..self.cols {
            let Some(cell) = self.cell_mut(row, col) else {
                continue;
            };
            cell.pen.set_dwl(dwl);
            cell.pen.set_dhl(dhl);
        }
        let doublewidth = dwl != 0;
        let mut rect = VTermRect {
            start_row: row,
            end_row: row + 1,
            start_col: 0,
            end_col: if doublewidth {
                self.cols / 2
            } else {
                self.cols
            },
        };
        self.damage(rect);
        if doublewidth {
            rect.start_col = self.cols / 2;
            rect.end_col = self.cols;
            self.erase_cells(rect, false, state_rows, lines);
        }
        1
    }

    /// Blanks a rectangle and reports it.
    pub(crate) fn erase(
        &mut self,
        rect: VTermRect,
        selective: bool,
        state_rows: c_int,
        lines: &[VTermLineInfo],
    ) -> c_int {
        self.erase_cells(rect, selective, state_rows, lines);
        self.damage(rect);
        1
    }

    pub(crate) fn bell(&mut self) -> c_int {
        self.host().and_then(|host| host.bell()).unwrap_or(0)
    }

    /// Whether the host shows a dark theme, into `dark`; 1 when it does not
    /// say, leaving `dark` as it was.
    pub(crate) fn theme(&mut self, dark: &mut bool) -> c_int {
        self.host().and_then(|host| host.theme(dark)).unwrap_or(1)
    }

    /// Asks the host to drop its scrollback; answers whether it did.
    pub(crate) fn sb_clear(&mut self) -> c_int {
        let cleared = self.host().and_then(|host| host.sb_clear()).unwrap_or(0);
        c_int::from(cleared != 0)
    }

    /// Which buffer a resize treats as the one holding the cursor.
    pub(super) fn altscreen_active(&self) -> bool {
        self.owned.has_altscreen && self.owned.active == super::BUFIDX_ALTSCREEN
    }
}
