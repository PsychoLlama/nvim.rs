//! Control sequences: everything introduced by `CSI`.
//!
//! A sequence is identified by up to three bytes — an optional private-use
//! leader, an optional intermediate, and the final byte that names the
//! command — so the dispatch matches on that triple.

#![forbid(unsafe_code)]

use core::ffi::{c_int, c_long};
use core::fmt::Write;

use crate::src::nvim::types::{VTermRect, VTermState};
use crate::src::nvim::vterm::geometry::{DHL_OFF, DWL_OFF};
use crate::src::nvim::vterm::mode;
use crate::src::nvim::vterm::mode::{
    VTERM_PROP_CURSORSHAPE_BAR_LEFT, VTERM_PROP_CURSORSHAPE_BLOCK, VTERM_PROP_CURSORSHAPE_UNDERLINE,
};
use crate::src::nvim::vterm::output::EscapeSeq;
use crate::src::nvim::vterm::pen::{CSI_ARG_MASK, CSI_ARG_MISSING};
use crate::src::nvim::vterm::state::{PenChange, vterm_primary_device_attr};
use crate::src::nvim::vterm::vterm::{VTERM_PROP_CURSORBLINK, VTERM_PROP_CURSORSHAPE};

/// What became of a control sequence.
pub(super) enum Outcome {
    /// Acted upon.
    Handled,
    /// Not recognised; the consumer's fallback gets a turn.
    Unrecognised,
    /// Recognised but malformed, which upstream drops without telling anyone.
    Ignored,
}

/// The `n`th parameter, or `None` when the sequence omitted it. The
/// sub-parameter flag is not part of the value.
fn param(args: &[c_long], n: usize) -> Option<c_int> {
    let raw = args.get(n).copied().unwrap_or(CSI_ARG_MISSING) & CSI_ARG_MASK as c_long;
    (raw != CSI_ARG_MISSING).then_some(raw as c_int)
}

/// The `n`th parameter, or `default` when it was omitted.
fn param_or(args: &[c_long], n: usize, default: c_int) -> c_int {
    param(args, n).unwrap_or(default)
}

/// The `n`th parameter read as a repeat count, where both an omitted
/// parameter and an explicit zero mean once.
fn repeat(args: &[c_long], n: usize) -> c_int {
    match param(args, n) {
        Some(0) | None => 1,
        Some(count) => count,
    }
}

/// Acts on one control sequence.
pub(super) fn dispatch(
    state: &mut VTermState,
    leader: [u8; 2],
    args: &[c_long],
    intermed: [u8; 2],
    command: u8,
) -> Outcome {
    // Only single-byte leaders and intermediates from the sets below name a
    // sequence this terminal knows; anything else is dropped whole.
    let leader = match leader {
        [0, _] => 0,
        [b'?' | b'>' | b'<' | b'=', 0] => leader[0],
        _ => return Outcome::Ignored,
    };
    let intermed = match intermed {
        [0, _] => 0,
        [b' ' | b'!' | b'"' | b'$' | b'\'', 0] => intermed[0],
        _ => return Outcome::Ignored,
    };

    let oldpos = state.pos;
    // A sequence normally cancels a pending wrap; REP is the exception,
    // because it may have just set one up.
    let mut cancel_phantom = true;

    match (intermed, leader, command) {
        (0, 0, 0x40) => {
            // ICH - ECMA-48 8.3.64
            let count = repeat(args, 0);
            if !state.cursor_in_scroll_region() {
                return finish(state, oldpos, cancel_phantom);
            }
            let rect = VTermRect {
                start_row: state.pos.row,
                end_row: state.pos.row + 1,
                start_col: state.pos.col,
                end_col: if state.mode.leftrightmargin() != 0 {
                    state.scroll_right()
                } else {
                    state.cursor_row_width()
                },
            };
            state.scroll(rect, 0, -count);
        }
        (0, 0, 0x41) => cursor_by(state, -repeat(args, 0), 0), // CUU - ECMA-48 8.3.22
        (0, 0, 0x42) => cursor_by(state, repeat(args, 0), 0),  // CUD - ECMA-48 8.3.19
        (0, 0, 0x43) => cursor_by(state, 0, repeat(args, 0)),  // CUF - ECMA-48 8.3.20
        (0, 0, 0x44) => cursor_by(state, 0, -repeat(args, 0)), // CUB - ECMA-48 8.3.18
        (0, 0, 0x45) => {
            // CNL - ECMA-48 8.3.12
            state.pos.col = 0;
            cursor_by(state, repeat(args, 0), 0);
        }
        (0, 0, 0x46) => {
            // CPL - ECMA-48 8.3.13
            state.pos.col = 0;
            cursor_by(state, -repeat(args, 0), 0);
        }
        (0, 0, 0x47) => {
            // CHA - ECMA-48 8.3.9
            state.pos.col = param_or(args, 0, 1) - 1;
            state.at_phantom = 0;
        }
        // CUP and HVP are the same command under two names.
        (0, 0, 0x48) | (0, 0, 0x66) => {
            state.pos.row = param_or(args, 0, 1) - 1;
            state.pos.col = param_or(args, 1, 1) - 1;
            if state.mode.origin() != 0 {
                state.pos.row += state.scrollregion_top;
                state.pos.col += state.scroll_left();
            }
            state.at_phantom = 0;
        }
        (0, 0, 0x49) => state.tab(repeat(args, 0), true), // CHT - ECMA-48 8.3.10
        // ED - ECMA-48 8.3.39, and its selective form DECSED.
        (0, 0, 0x4a) | (0, b'?', 0x4a) => {
            if erase_display(state, args, leader == b'?') {
                return Outcome::Handled;
            }
        }
        // EL - ECMA-48 8.3.41, and its selective form DECSEL.
        (0, 0, 0x4b) | (0, b'?', 0x4b) => {
            let (start_col, end_col) = match param(args, 0) {
                None | Some(0) => (state.pos.col, state.cursor_row_width()),
                Some(1) => (0, state.pos.col + 1),
                Some(2) => (0, state.cursor_row_width()),
                Some(_) => return Outcome::Ignored,
            };
            if end_col > start_col {
                let rect = VTermRect {
                    start_row: state.pos.row,
                    end_row: state.pos.row + 1,
                    start_col,
                    end_col,
                };
                state.erase(rect, leader == b'?');
            }
        }
        (0, 0, 0x4c) => scroll_region_by(state, -repeat(args, 0)), // IL - ECMA-48 8.3.67
        (0, 0, 0x4d) => scroll_region_by(state, repeat(args, 0)),  // DL - ECMA-48 8.3.32
        (0, 0, 0x50) => {
            // DCH - ECMA-48 8.3.26
            let count = repeat(args, 0);
            if !state.cursor_in_scroll_region() {
                return finish(state, oldpos, cancel_phantom);
            }
            let rect = VTermRect {
                start_row: state.pos.row,
                end_row: state.pos.row + 1,
                start_col: state.pos.col,
                end_col: if state.mode.leftrightmargin() != 0 {
                    state.scroll_right()
                } else {
                    state.cursor_row_width()
                },
            };
            state.scroll(rect, 0, count);
        }
        (0, 0, 0x53) => {
            // SU - ECMA-48 8.3.147
            let rect = state.scroll_region();
            state.scroll(rect, repeat(args, 0), 0);
        }
        (0, 0, 0x54) => {
            // SD - ECMA-48 8.3.113
            let rect = state.scroll_region();
            state.scroll(rect, -repeat(args, 0), 0);
        }
        (0, 0, 0x58) => {
            // ECH - ECMA-48 8.3.38
            let rect = VTermRect {
                start_row: state.pos.row,
                end_row: state.pos.row + 1,
                start_col: state.pos.col,
                end_col: (state.pos.col + repeat(args, 0)).min(state.cursor_row_width()),
            };
            state.erase(rect, false);
        }
        (0, 0, 0x5a) => state.tab(repeat(args, 0), false), // CBT - ECMA-48 8.3.7
        (0, 0, 0x60) => {
            // HPA - ECMA-48 8.3.57
            state.pos.col = param_or(args, 0, 1) - 1;
            state.at_phantom = 0;
        }
        (0, 0, 0x61) => cursor_by(state, 0, repeat(args, 0)), // HPR - ECMA-48 8.3.59
        (0, 0, 0x62) => repeat_glyph(state, repeat(args, 0), &mut cancel_phantom), // REP
        (0, 0, 0x63) => {
            // DA - ECMA-48 8.3.24
            if param_or(args, 0, 0) == 0 {
                let attr = vterm_primary_device_attr.with(|attr| *attr);
                let text: Vec<u8> = attr
                    .iter()
                    .take_while(|&&byte| byte != 0)
                    .map(|&byte| byte as u8)
                    .collect();
                let mut seq = EscapeSeq::csi(state.ctrl8bit());
                seq.push(b'?');
                seq.extend(&text);
                seq.push(b'c');
                state.reply(&seq);
            }
        }
        (0, b'>', 0x63) => {
            // DEC secondary Device Attributes
            let mut seq = EscapeSeq::csi(state.ctrl8bit());
            let _ = write!(seq, ">0;100;0c");
            state.reply(&seq);
        }
        (0, 0, 0x64) => {
            // VPA - ECMA-48 8.3.158
            state.pos.row = param_or(args, 0, 1) - 1;
            if state.mode.origin() != 0 {
                state.pos.row += state.scrollregion_top;
            }
            state.at_phantom = 0;
        }
        (0, 0, 0x65) => cursor_by(state, repeat(args, 0), 0), // VPR - ECMA-48 8.3.160
        (0, 0, 0x67) => {
            // TBC - ECMA-48 8.3.154. Line tab stops are not modelled, so the
            // three parameters that clear them do nothing.
            match param_or(args, 0, 0) {
                0 => state.clear_tabstop(state.pos.col),
                3 | 5 => {
                    for col in 0..state.cols {
                        state.clear_tabstop(col);
                    }
                }
                1 | 2 | 4 => {}
                _ => return Outcome::Ignored,
            }
        }
        (0, 0, 0x68) => {
            // SM - ECMA-48 8.3.125
            if let Some(num) = param(args, 0) {
                mode::set_ansi_mode(state, num, true);
            }
        }
        (0, 0, 0x6c) => {
            // RM - ECMA-48 8.3.106
            if let Some(num) = param(args, 0) {
                mode::set_ansi_mode(state, num, false);
            }
        }
        // DEC private mode set and reset, which take a list of modes.
        (0, b'?', 0x68) | (0, b'?', 0x6c) => {
            for i in 0..args.len() {
                if let Some(num) = param(args, i) {
                    mode::set_dec_mode(state, num, command == 0x68);
                }
            }
        }
        (0, 0, 0x6a) => cursor_by(state, 0, -repeat(args, 0)), // HPB - ECMA-48 8.3.58
        (0, 0, 0x6b) => cursor_by(state, -repeat(args, 0), 0), // VPB - ECMA-48 8.3.159
        (0, 0, 0x6d) => state.change_pen(PenChange::Sgr(args)), // SGR - ECMA-48 8.3.117
        (0, b'?', 0x6d) => {
            // DECSGR. No DEC terminal recognised these, but some printers
            // did; they are another way to ask for super- and subscript.
            for i in 0..args.len() {
                let replacement = match param(args, i) {
                    Some(4) => 73,  // superscript on
                    Some(5) => 74,  // subscript on
                    Some(24) => 75, // both off
                    _ => continue,
                };
                state.change_pen(PenChange::Sgr(&[replacement]));
            }
        }
        // DSR - ECMA-48 8.3.35, and DECDSR.
        (0, 0, 0x6e) | (0, b'?', 0x6e) => device_status(state, param_or(args, 0, 0), leader),
        (b'!', 0, 0x70) => mode::reset(state, false), // DECSTR - soft terminal reset
        (b'$', b'?', 0x70) => {
            mode::request_dec_mode(state, param_or(args, 0, CSI_ARG_MISSING as _))
        }
        (0, b'>', 0x71) => mode::request_version_string(state), // XTVERSION
        (b' ', 0, 0x71) => {
            // DECSCUSR - the cursor's shape, and whether it blinks.
            let (blink, shape) = match param_or(args, 0, 1) {
                0 | 1 => (true, VTERM_PROP_CURSORSHAPE_BLOCK),
                2 => (false, VTERM_PROP_CURSORSHAPE_BLOCK),
                3 => (true, VTERM_PROP_CURSORSHAPE_UNDERLINE),
                4 => (false, VTERM_PROP_CURSORSHAPE_UNDERLINE),
                5 => (true, VTERM_PROP_CURSORSHAPE_BAR_LEFT),
                6 => (false, VTERM_PROP_CURSORSHAPE_BAR_LEFT),
                _ => return finish(state, oldpos, cancel_phantom),
            };
            state.set_termprop_bool(VTERM_PROP_CURSORBLINK, blink);
            state.set_termprop_int(VTERM_PROP_CURSORSHAPE, shape);
        }
        (b'"', 0, 0x71) => {
            // DECSCA - whether following cells resist a selective erase.
            match param_or(args, 0, 0) {
                0 | 2 => state.set_protected_cell(0),
                1 => state.set_protected_cell(1),
                _ => {}
            }
        }
        (0, 0, 0x72) => set_vertical_margins(state, args), // DECSTBM
        (0, 0, 0x73) => set_horizontal_margins(state, args), // DECSLRM
        (0, b'?', 0x75) => mode::request_key_encoding_flags(state), // kitty query
        (0, b'>', 0x75) => mode::push_key_encoding_flags(state, param_or(args, 0, 0)),
        (0, b'<', 0x75) => mode::pop_key_encoding_flags(state, param_or(args, 0, 1)),
        (0, b'=', 0x75) => {
            let how = param_or(args, 1, 1);
            mode::set_key_encoding_flags(state, param_or(args, 0, 0), how);
        }
        // DECIC and DECDC, which open or close a column at the cursor.
        (b'\'', 0, command @ (0x7d | 0x7e)) => {
            let count = repeat(args, 0);
            if !state.cursor_in_scroll_region() {
                return finish(state, oldpos, cancel_phantom);
            }
            let rect = VTermRect {
                start_row: state.scrollregion_top,
                end_row: state.scroll_bottom(),
                start_col: state.pos.col,
                end_col: state.scroll_right(),
            };
            state.scroll(rect, 0, if command == 0x7d { -count } else { count });
        }
        _ => return Outcome::Unrecognised,
    }

    finish(state, oldpos, cancel_phantom)
}

/// Pulls the cursor back inside its bounds and reports the move.
fn finish(
    state: &mut VTermState,
    oldpos: crate::src::nvim::types::VTermPos,
    cancel: bool,
) -> Outcome {
    state.clamp_cursor();
    state.update_cursor(oldpos, cancel);
    Outcome::Handled
}

/// Moves the cursor relatively. A host may name a distance far larger than
/// the screen, so the arithmetic wraps rather than trapping; the caller pulls
/// the cursor back inside afterwards.
fn cursor_by(state: &mut VTermState, rows: c_int, cols: c_int) {
    state.pos.row = state.pos.row.wrapping_add(rows);
    state.pos.col = state.pos.col.wrapping_add(cols);
    state.at_phantom = 0;
}

/// `IL` and `DL`: open or close whole lines at the cursor.
fn scroll_region_by(state: &mut VTermState, downward: c_int) {
    if !state.cursor_in_scroll_region() {
        return;
    }
    let rect = VTermRect {
        start_row: state.pos.row,
        end_row: state.scroll_bottom(),
        start_col: state.scroll_left(),
        end_col: state.scroll_right(),
    };
    state.scroll(rect, downward, 0);
}

/// `ED` / `DECSED`. Reports whether the sequence is already complete, which
/// only happens when the consumer swallowed a scrollback clear.
fn erase_display(state: &mut VTermState, args: &[c_long], selective: bool) -> bool {
    let (rows, cols) = (state.rows, state.cols);
    match param(args, 0) {
        // Below the cursor: the rest of its line, then everything under it.
        None | Some(0) => {
            if cols > state.pos.col {
                let rect = VTermRect {
                    start_row: state.pos.row,
                    end_row: state.pos.row + 1,
                    start_col: state.pos.col,
                    end_col: cols,
                };
                state.erase(rect, selective);
            }
            let rect = VTermRect {
                start_row: state.pos.row + 1,
                end_row: rows,
                start_col: 0,
                end_col: cols,
            };
            for row in rect.start_row..rect.end_row {
                state.set_lineinfo(row, true, DWL_OFF, DHL_OFF);
            }
            if rect.end_row > rect.start_row {
                state.erase(rect, selective);
            }
        }
        // Above the cursor: everything over it, then the start of its line.
        Some(1) => {
            let rect = VTermRect {
                start_row: 0,
                end_row: state.pos.row,
                start_col: 0,
                end_col: cols,
            };
            for row in rect.start_row..rect.end_row {
                state.set_lineinfo(row, true, DWL_OFF, DHL_OFF);
            }
            if rect.end_col > rect.start_col {
                state.erase(rect, selective);
            }
            let rect = VTermRect {
                start_row: state.pos.row,
                end_row: state.pos.row + 1,
                start_col: 0,
                end_col: state.pos.col + 1,
            };
            if rect.end_row > rect.start_row {
                state.erase(rect, selective);
            }
        }
        Some(2) => {
            let rect = VTermRect {
                start_row: 0,
                end_row: rows,
                start_col: 0,
                end_col: cols,
            };
            for row in 0..rows {
                state.set_lineinfo(row, true, DWL_OFF, DHL_OFF);
            }
            state.erase(rect, selective);
        }
        Some(3) => return state.clear_scrollback(),
        Some(_) => {}
    }
    false
}

/// `REP`: print the last glyph again, `count` more times.
fn repeat_glyph(state: &mut VTermState, count: c_int, cancel_phantom: &mut bool) {
    let row_width = state.cursor_row_width();
    let last_col = (state.pos.col + count).min(row_width);
    let (_, schar) = state.grapheme_metrics(state.grapheme_len);
    // A glyph no columns wide would never reach `last_col`; upstream span in
    // place forever, which a program can provoke by asking for a repeat
    // before it has printed anything at all.
    let width = state.combine_width;
    if width > 0 {
        while state.pos.col < last_col {
            let pos = state.pos;
            state.put_glyph(schar, width, pos);
            state.pos.col += width;
        }
    }
    if state.pos.col + width >= row_width && state.mode.autowrap() != 0 {
        state.at_phantom = 1;
        *cancel_phantom = false;
    }
}

/// `DSR` / `DECDSR`: the status and cursor-position reports.
fn device_status(state: &mut VTermState, request: c_int, leader: u8) {
    let private = leader == b'?';
    let mut seq = EscapeSeq::csi(state.ctrl8bit());
    match request {
        // 0 to 4 are replies, not requests.
        0..=4 => return,
        5 => {
            if private {
                seq.push(b'?');
            }
            let _ = write!(seq, "0n");
        }
        6 => {
            // CPR - cursor position report
            if private {
                seq.push(b'?');
            }
            let _ = write!(seq, "{};{}R", state.pos.row + 1, state.pos.col + 1);
        }
        996 => match state.theme_is_dark() {
            Some(dark) => {
                let _ = write!(seq, "?997;{}n", if dark { '1' } else { '2' });
            }
            None => return,
        },
        _ => return,
    }
    state.reply(&seq);
}

/// `DECSTBM`: the top and bottom margins, which also homes the cursor.
fn set_vertical_margins(state: &mut VTermState, args: &[c_long]) {
    state.scrollregion_top = (param_or(args, 0, 1) - 1).max(0).min(state.rows);
    state.scrollregion_bottom = match param(args, 1) {
        Some(bottom) if args.len() >= 2 => bottom.max(-1),
        _ => -1,
    };
    if state.scrollregion_top == 0 && state.scrollregion_bottom == state.rows {
        state.scrollregion_bottom = -1;
    } else {
        state.scrollregion_bottom = state.scrollregion_bottom.min(state.rows);
    }
    // A region with no room in it is no region at all.
    if state.scroll_bottom() <= state.scrollregion_top {
        state.scrollregion_top = 0;
        state.scrollregion_bottom = -1;
    }
    home_cursor(state);
}

/// `DECSLRM`: the left and right margins. They are always accepted, but only
/// take effect once DECVSSM turns margin mode on.
fn set_horizontal_margins(state: &mut VTermState, args: &[c_long]) {
    state.scrollregion_left = (param_or(args, 0, 1) - 1).max(0).min(state.cols);
    state.scrollregion_right = match param(args, 1) {
        Some(right) if args.len() >= 2 => right.max(-1),
        _ => -1,
    };
    if state.scrollregion_left == 0 && state.scrollregion_right == state.cols {
        state.scrollregion_right = -1;
    } else {
        state.scrollregion_right = state.scrollregion_right.min(state.cols);
    }
    if state.scrollregion_right > -1 && state.scrollregion_right <= state.scrollregion_left {
        state.scrollregion_left = 0;
        state.scrollregion_right = -1;
    }
    home_cursor(state);
}

/// Setting either pair of margins puts the cursor back at the origin.
fn home_cursor(state: &mut VTermState) {
    state.pos.row = 0;
    state.pos.col = 0;
    if state.mode.origin() != 0 {
        state.pos.row += state.scrollregion_top;
        state.pos.col += state.scroll_left();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MORE: c_long = 1 << 31;

    #[test]
    fn an_omitted_parameter_falls_back_to_its_default() {
        let args = [CSI_ARG_MISSING, 7];
        assert_eq!(param(&args, 0), None);
        assert_eq!(param_or(&args, 0, 4), 4);
        assert_eq!(param(&args, 1), Some(7));
        assert_eq!(param_or(&args, 1, 4), 7);
    }

    #[test]
    fn a_parameter_past_the_end_reads_as_omitted() {
        let args = [5];
        assert_eq!(param(&args, 1), None);
        assert_eq!(param_or(&args, 1, 1), 1);
        assert_eq!(param(&[], 0), None);
    }

    #[test]
    fn the_sub_parameter_flag_is_not_part_of_the_value() {
        assert_eq!(param(&[38 | MORE], 0), Some(38));
    }

    #[test]
    fn a_repeat_count_of_none_or_zero_means_once() {
        assert_eq!(repeat(&[CSI_ARG_MISSING], 0), 1);
        assert_eq!(repeat(&[0], 0), 1);
        assert_eq!(repeat(&[3], 0), 3);
        assert_eq!(repeat(&[], 0), 1);
    }
}
