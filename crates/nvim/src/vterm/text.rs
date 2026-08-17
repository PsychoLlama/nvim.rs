//! Printable text and the sequences that are not control sequences: the
//! C0/C1 controls and the two-byte escapes.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![forbid(unsafe_code)]

use core::ffi::c_int;

use crate::types::{VTermRect, VTermState, schar_T};
use crate::vterm::geometry::{DHL_BOTTOM, DHL_OFF, DHL_TOP, DWL_OFF, DWL_ON};
use crate::vterm::mode;
use crate::vterm::pen::save_pen;
use crate::vterm::state::{PenChange, is_composing};
use crate::vterm::vterm::{
    VTERM_PROP_CURSORBLINK, VTERM_PROP_CURSORSHAPE, VTERM_PROP_CURSORVISIBLE,
};

/// Prints a decoded run of codepoints, gathering each grapheme cluster and
/// stamping it as one glyph.
pub(super) fn print(state: &mut VTermState, codepoints: &[u32]) {
    let oldpos = state.pos;
    let mut grapheme = 0 as _;
    let mut len = 0usize;
    let mut recombine = false;

    // A combining character arriving right after the glyph it belongs to is
    // merged into it rather than printed on its own.
    if state.pos.row == state.combine_pos.row
        && state.pos.col >= state.combine_pos.col
        && state.pos.col <= state.combine_pos.col + state.combine_width
        && is_composing(
            state.grapheme_last,
            codepoints[0],
            &mut state.grapheme_state,
        )
    {
        len = state.grapheme_len;
        grapheme = state.grapheme_state;
        state.pos.col = state.combine_pos.col;
        state.at_phantom = 0;
        recombine = true;
    }

    let mut i = 0usize;
    while i < codepoints.len() {
        // Gather this codepoint and every combining one that follows it.
        loop {
            if len < state.grapheme_buf.len() - 4 {
                len += state.append_grapheme(len, codepoints[i]);
            }
            i += 1;
            if i >= codepoints.len()
                || !is_composing(codepoints[i - 1], codepoints[i], &mut grapheme)
            {
                break;
            }
        }

        let (width, schar) = state.grapheme_metrics(len);
        if state.at_phantom != 0 || state.pos.col + width > state.cursor_row_width() {
            state.linefeed();
            state.pos.col = 0;
            state.at_phantom = 0;
            let row = state.pos.row as usize;
            state.lineinfo_mut()[row].set_continuation(1);
        }

        if state.mode.insert() != 0 && !recombine {
            // TODO(vterm): one insert per glyph is wasteful for a long run;
            // the run could be scanned ahead and inserted for in one go.
            let rect = VTermRect {
                start_row: state.pos.row,
                end_row: state.pos.row + 1,
                start_col: state.pos.col,
                end_col: state.cursor_row_width(),
            };
            state.scroll(rect, 0, -1);
        }

        let pos = state.pos;
        state.put_glyph(schar, width, pos);

        if i == codepoints.len() {
            // End of the run: remember the glyph in case the next call starts
            // with something that combines with it.
            state.grapheme_len = len;
            state.grapheme_last = codepoints[i - 1];
            state.grapheme_state = grapheme;
            state.combine_width = width;
            state.combine_pos = state.pos;
        } else {
            len = 0;
            recombine = false;
        }

        if state.pos.col + width >= state.cursor_row_width() {
            // Sit on the phantom column past the right edge, so that the wrap
            // only happens once something else is printed.
            if state.mode.autowrap() != 0 {
                state.at_phantom = 1;
            }
        } else {
            state.pos.col += width;
        }
    }

    state.update_cursor(oldpos, false);
}

/// Handles one C0 or C1 control. Reports whether it was recognised, here or
/// by the consumer's fallback.
pub(super) fn control(state: &mut VTermState, control: u8) -> bool {
    let oldpos = state.pos;
    match control {
        0x07 => state.bell(), // BEL - ECMA-48 8.3.3
        // BS - ECMA-48 8.3.5
        0x08 => {
            if state.pos.col > 0 {
                state.pos.col -= 1;
            }
        }
        0x09 => state.tab(1, true), // HT - ECMA-48 8.3.60
        // LF, VT and FF all just feed a line; LNM makes them return as well.
        0x0a..=0x0c => {
            state.linefeed();
            if state.mode.newline() != 0 {
                state.pos.col = 0;
            }
        }
        0x0d => state.pos.col = 0, // CR - ECMA-48 8.3.15
        0x0e => state.gl_set = 1,  // LS1 - ECMA-48 8.3.76
        0x0f => state.gl_set = 0,  // LS0 - ECMA-48 8.3.75
        0x84 => state.linefeed(),  // IND - deprecated, implemented for completeness
        0x85 => {
            // NEL - ECMA-48 8.3.86
            state.linefeed();
            state.pos.col = 0;
        }
        0x88 => state.set_tabstop(state.pos.col), // HTS - ECMA-48 8.3.62
        0x8d => {
            // RI - ECMA-48 8.3.104
            if state.pos.row == state.scrollregion_top {
                let rect = state.scroll_region();
                state.scroll(rect, -1, 0);
            } else if state.pos.row > 0 {
                state.pos.row -= 1;
            }
        }
        0x8e => state.gsingle_set = 2, // SS2 - ECMA-48 8.3.141
        0x8f => state.gsingle_set = 3, // SS3 - ECMA-48 8.3.142
        _ => return false,
    }
    state.update_cursor(oldpos, true);
    true
}

/// Handles an escape sequence — the intermediates followed by the final byte.
/// Returns how many of those bytes it consumed, zero when it recognised
/// nothing.
pub(super) fn escape(state: &mut VTermState, seq: &[u8]) -> c_int {
    // The first byte decides the sequence even though the last one ends it.
    match seq.first().copied().unwrap_or(0) {
        b' ' => {
            if seq.len() != 2 {
                return 0;
            }
            match seq[1] {
                b'F' => state.set_ctrl8bit(false), // S7C1T
                b'G' => state.set_ctrl8bit(true),  // S8C1T
                _ => return 0,
            }
            2
        }
        b'#' => {
            if seq.len() != 2 {
                return 0;
            }
            // The line-size controls are meaningless while the left and right
            // margins are in play, so they are dropped there.
            let margins = state.mode.leftrightmargin() != 0;
            match seq[1] {
                b'3' if !margins => state.set_lineinfo(state.pos.row, false, DWL_ON, DHL_TOP),
                b'4' if !margins => state.set_lineinfo(state.pos.row, false, DWL_ON, DHL_BOTTOM),
                b'5' if !margins => state.set_lineinfo(state.pos.row, false, DWL_OFF, DHL_OFF),
                b'6' if !margins => state.set_lineinfo(state.pos.row, false, DWL_ON, DHL_OFF),
                b'3' | b'4' | b'5' | b'6' => {}
                b'8' => screen_alignment_test(state), // DECALN
                _ => return 0,
            }
            2
        }
        // SCS - designate a character set into one of the four slots.
        first @ (b'(' | b')' | b'*' | b'+') => {
            if seq.len() != 2 {
                return 0;
            }
            state.designate_charset(usize::from(first - 0x28), seq[1] as _);
            2
        }
        b'7' => {
            // DECSC
            save_cursor(state, true);
            1
        }
        b'8' => {
            // DECRC
            save_cursor(state, false);
            1
        }
        // Ignored by the VT100; in VT52 mode it switched up to VT100.
        b'<' => 1,
        b'=' => {
            // DECKPAM
            state.mode.set_keypad(1);
            1
        }
        b'>' => {
            // DECKPNM
            state.mode.set_keypad(0);
            1
        }
        b'c' => {
            // RIS - ECMA-48 8.3.105. The move is reported even when the
            // cursor did not actually move, so that the consumer redraws it.
            let oldpos = state.pos;
            mode::reset(state, true);
            state.force_cursor_report(oldpos);
            1
        }
        b'n' => {
            // LS2 - ECMA-48 8.3.78
            state.gl_set = 2;
            1
        }
        b'o' => {
            // LS3 - ECMA-48 8.3.80
            state.gl_set = 3;
            1
        }
        b'~' => {
            // LS1R - ECMA-48 8.3.77
            state.gr_set = 1;
            1
        }
        b'}' => {
            // LS2R - ECMA-48 8.3.79
            state.gr_set = 2;
            1
        }
        b'|' => {
            // LS3R - ECMA-48 8.3.81
            state.gr_set = 3;
            1
        }
        _ => 0,
    }
}

/// DECALN: fill the screen with `E`, the alignment pattern.
fn screen_alignment_test(state: &mut VTermState) {
    let e = schar_T::from(b'E');
    for row in 0..state.rows {
        for col in 0..state.row_width(row) {
            let pos = crate::types::VTermPos { row, col };
            state.put_glyph(e, 1, pos);
        }
    }
}

/// DECSC and DECRC: stash the cursor, the pen and the cursor's appearance, or
/// put them all back.
pub(super) fn save_cursor(state: &mut VTermState, save: bool) {
    if save {
        state.saved.pos = state.pos;
        let (visible, blink, shape) = (
            state.mode.cursor_visible(),
            state.mode.cursor_blink(),
            state.mode.cursor_shape(),
        );
        state.saved.mode.set_cursor_visible(visible);
        state.saved.mode.set_cursor_blink(blink);
        state.saved.mode.set_cursor_shape(shape);
        save_pen(state);
    } else {
        let oldpos = state.pos;
        state.pos = state.saved.pos;
        let (visible, blink, shape) = (
            state.saved.mode.cursor_visible() != 0,
            state.saved.mode.cursor_blink() != 0,
            state.saved.mode.cursor_shape() as c_int,
        );
        state.set_termprop_bool(VTERM_PROP_CURSORVISIBLE, visible);
        state.set_termprop_bool(VTERM_PROP_CURSORBLINK, blink);
        state.set_termprop_int(VTERM_PROP_CURSORSHAPE, shape);
        state.change_pen(PenChange::Restore);
        state.update_cursor(oldpos, true);
    }
}
