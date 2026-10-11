//! `:print` and `:list`: one buffer line onto the message area.
//!
//! [`msg_prt_line`] is the only message path that knows about `'listchars'`,
//! `'tabstop'` and the lead/trail/multispace distinctions, which is why it
//! duplicates so much of the drawing code's character loop.

#![forbid(unsafe_code)]

use super::*;
use crate::charset::CharDisplay;
use crate::cstr;
use crate::grid::{MAX_SCHAR_SIZE, schar_text};
use crate::mbyte::char_at;
use crate::types::{MB_MAXBYTES, NUL};
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

/// Show one line of buffer text, as `:print` and `:list` do.
///
/// Runs a cut-down version of the drawing code's character loop: each source
/// byte becomes one or more *cells*, and a cell that stands for several
/// columns (a tab, a `<xx>` escape) leaves the rest of them queued in
/// `extra_*` for the following turns of the loop. Past the end of `line`
/// the walk reads the NUL the C string ended at.
pub fn msg_prt_line(line: &CStr, list: bool) {
    let text = line.to_bytes();
    let at = |i: usize| cstr::byte_at(text, i);
    // `'list'` on the window forces the listing form whatever the caller
    // asked for.
    let list = list || Win::current().w_onebuf_opt.wo_list != 0;
    // The window's 'listchars', copied: printing can redraw, and the walk
    // holds no borrow of the window across it.
    let lcs = Win::current().w_p_lcs_chars.clone();
    let (multispace, leadmultispace) = lcs.multispace_runs();

    // Where the trailing whitespace starts, and where the leading
    // whitespace ends; `None` when no 'listchars' item needs them.
    let mut trail = None;
    let mut lead = None;
    if list {
        if lcs.trail != 0 {
            let mut start = text.len();
            while start > 0 && ascii_iswhite(c_int::from(text[start - 1])) {
                start -= 1;
            }
            trail = Some(start);
        }
        if lcs.lead != 0 || leadmultispace.is_some() || lcs.leadtab1 != 0 {
            let mut end = 0;
            while ascii_iswhite(c_int::from(at(end))) {
                end += 1;
            }
            // In a line of nothing but spaces they all count as trailing.
            lead = (at(end) != 0).then_some(end);
        }
    }

    // Output a space for an empty line, or it would not overwrite what is
    // already on the row.
    if at(0) == 0 && !(list && lcs.eol != 0) {
        msg_putchar(b' ' as c_int);
    }

    let mut s = 0;
    let mut col = 0;
    let mut hl_id = 0;
    let mut in_multispace = false;
    let mut multispace_pos = 0;
    // Cells still owed for the character just consumed: `extra_fill` for
    // all but the last, `extra_last` for the last one, or, when neither is
    // set, one byte at a time out of `extra_text` from `extra_at` on.
    let mut extra_left = 0;
    let mut extra_fill: ScreenChar = 0;
    let mut extra_last: ScreenChar = 0;
    let mut extra_text: CharDisplay = [0; _];
    let mut extra_at = 0;

    while !got_int.get() {
        let sc: ScreenChar;
        if extra_left > 0 {
            extra_left -= 1;
            sc = if extra_left == 0 && extra_last != 0 {
                extra_last
            } else if extra_fill != 0 {
                extra_fill
            } else {
                let byte = extra_text.get(extra_at).map_or(0, |&b| b as u8);
                extra_at += 1;
                ScreenChar::from(byte)
            };
        } else {
            let rest = &text[s.min(text.len())..];
            let len = c_cluster_len(rest);
            if len > 1 {
                // A multi-byte character goes out whole, not as a cell.
                col += cells_at(rest);
                let mut glyph = [0; MAX_SCHAR_SIZE as usize];
                let c = char_at(rest);
                let shown: &[u8] = if len >= MB_MAXBYTES {
                    b"?"
                } else if lcs.nbsp != 0 && list && (c == 160 || c == 0x202f) {
                    schar_text(lcs.nbsp, &mut glyph)
                } else {
                    &rest[..len]
                };
                msg_bytes(shown, 0, false);
                s += len;
                continue;
            }

            hl_id = 0;
            let c = c_int::from(at(s));
            s += 1;
            if c >= 0x80 {
                // Illegal byte.
                col += utf_char2cells(c);
                msg_putchar(c);
                continue;
            }
            extra_fill = 0;
            extra_last = 0;
            if list {
                // `s` is already past `c`, so `s - 2` is the byte before it
                // -- the lookbehind that makes the *second* space of a run
                // count as multispace too.
                in_multispace = c == b' ' as c_int
                    && (at(s) == b' ' || (col > 0 && s >= 2 && at(s - 2) == b' '));
                if !in_multispace {
                    multispace_pos = 0;
                }
            }
            let in_lead = lead.is_some_and(|lead| s <= lead);

            if c == TAB && (!list || lcs.tab1 != 0) {
                // How wide the tab is depends on where it starts.
                extra_left = Buf::current().tab_padding(col) - 1;
                if list {
                    let (mut tab1, mut tab2, mut tab3) = (lcs.tab1, lcs.tab2, lcs.tab3);
                    if in_lead && lcs.leadtab1 != 0 {
                        tab1 = lcs.leadtab1;
                        tab2 = lcs.leadtab2;
                        tab3 = lcs.leadtab3;
                    }
                    sc = if extra_left == 0 && tab3 != 0 {
                        tab3
                    } else {
                        tab1
                    };
                    extra_fill = tab2;
                    extra_last = tab3;
                    hl_id = HLF_0;
                } else {
                    sc = b' ' as ScreenChar;
                    extra_fill = b' ' as ScreenChar;
                }
            } else if c == NUL && list && lcs.eol != 0 {
                // One more turn of the loop, which reads a NUL out of
                // `extra_text` and stops.
                extra_text = [0; _];
                extra_at = 0;
                extra_left = 1;
                sc = lcs.eol;
                hl_id = HLF_AT;
                s -= 1;
            } else if c != NUL && byte2cells(c) > 1 {
                // An unprintable byte, shown as `<xx>`.
                extra_left = byte2cells(c) - 1;
                extra_text = transchar_byte_buf(None, c);
                sc = ScreenChar::from(extra_text[0] as u8);
                extra_at = 1;
                // Its own highlight, so `<ff>` can be told apart from the
                // same four characters typed literally.
                hl_id = HLF_0;
            } else if c == b' ' as c_int {
                hl_id = HLF_0;
                sc = if in_lead && in_multispace && leadmultispace.is_some() {
                    cycle(
                        leadmultispace.as_deref().unwrap_or_default(),
                        &mut multispace_pos,
                    )
                } else if in_lead && lcs.lead != 0 {
                    lcs.lead
                } else if trail.is_some_and(|trail| s > trail) {
                    lcs.trail
                } else if in_multispace && multispace.is_some() {
                    cycle(
                        multispace.as_deref().unwrap_or_default(),
                        &mut multispace_pos,
                    )
                } else if list && lcs.space != 0 {
                    lcs.space
                } else {
                    hl_id = 0;
                    b' ' as ScreenChar
                };
            } else {
                sc = c as ScreenChar;
            }
        }

        if sc == 0 {
            break;
        }
        emit(sc, hl_id, &mut col);
    }
    msg_clr_eos();
}

/// Put one cell on the message area.
fn emit(sc: ScreenChar, hl_id: c_int, col: &mut c_int) {
    // TODO(bfredl): this is such baloney. need msg_put_schar
    let mut glyph = [0; MAX_SCHAR_SIZE as usize];
    msg_bytes(schar_text(sc, &mut glyph), hl_id, false);
    *col += 1;
}

/// One character of a cycling `'listchars'` sequence, wrapping at its end.
fn cycle(seq: &[ScreenChar], at: &mut usize) -> ScreenChar {
    let sc = seq.get(*at).copied().unwrap_or(0);
    *at += 1;
    if *at >= seq.len() {
        *at = 0;
    }
    sc
}
