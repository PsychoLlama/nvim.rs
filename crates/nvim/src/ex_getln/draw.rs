//! Putting the command line on the screen.
//!
//! [`draw_cmdline`] writes the buffer out with the colour chunks
//! [`super::color`] computed; [`redrawcmd`] is the whole-line redraw and
//! [`cursorcmd`] the cursor placement.  [`put_on_cmdline`] is the other
//! direction — inserting text into the buffer and recomputing the screen
//! columns it now occupies.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use super::*;
use crate::ascii::NUL;
use crate::cstr;
use crate::winlayer::cmdline_window;

/// The screen width of the command-line byte at `idx`.
pub(crate) fn cmdline_charsize(idx: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if CMDLINE_STAR.get() > 0 {
        // Showing '*': always one position.
        return 1;
    }
    let cc = Cc::current();
    let line = cc.bytes();
    ptr2cells_at(&line[usize::try_from(idx).unwrap_or(0).min(line.len())..])
}

/// The screen column the command line's first byte occupies: the indent, plus
/// one for the `:` / `/` / `?` when there is one.
pub(crate) fn cmd_startcol() -> ::core::ffi::c_int {
    let cc = Cc::current();
    cc.cmdindent + i32::from(cc.cmdfirstc != 0)
}

/// The screen column for a byte position on the command line.
pub fn cmd_screencol(bytepos: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let cc = Cc::current();
    let mut col = cmd_startcol();

    // The maximum column. A weird 'columns'/'lines' can overflow the
    // product, which reads as negative.
    let m = if KEY_TYPED.get() {
        let cells = match cmdline_window() {
            Some(w) => w.w_view_width * w.w_view_height,
            None => COLUMNS.get() * ROWS.get(),
        };
        if cells < 0 { MAXCOL } else { cells }
    } else {
        MAXCOL
    };

    let mut i = 0;
    while i < cc.len() && i < bytepos {
        let c = cmdline_charsize(i);
        // Count ">" for a double-wide character that doesn't fit.
        col = correct_screencol(i, c, col);

        // If the command line doesn't fit, show the cursor on the last
        // visible character. Don't move the cursor itself, so text can
        // still be appended.
        col += c;
        if col >= m {
            col -= c;
            break;
        }
        i += cluster_len_at(cc, i);
    }
    col
}

/// If the character at `idx` is a `cells`-wide multi-byte character that does
/// not fit on the line, account for the ">" that will be displayed instead:
/// `col`, moved on by one when it does.
pub(crate) fn correct_screencol(
    idx: ::core::ffi::c_int,
    cells: ::core::ffi::c_int,
    col: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let cc = Cc::current();
    let line = cc.bytes();
    let at = &line[usize::try_from(idx).unwrap_or(0).min(line.len())..];
    // An ASCII character takes one cell, composing characters or not.
    if at.first().is_none_or(|&b| b < 0x80) {
        return col;
    }
    if c_cluster_len(at) > 1 && c_cells_at(at) > 1 && col % COLUMNS.get() + cells > COLUMNS.get() {
        col + 1
    } else {
        col
    }
}

/// Draw `len` bytes of the command line from `start`, at the cursor position
/// — or stars, when `CMDLINE_STAR` is set.
pub(crate) fn draw_cmdline(start: ::core::ffi::c_int, len: ::core::ffi::c_int) {
    let mut cc = Cc::current();
    if !cc.in_use() || !color_cmdline(cc) {
        return;
    }

    if ui_has(UI_CMDLINE) {
        cc.special_char = 0;
        cc.redraw_state = CMD_REDRAW_ALL;
        return;
    }

    if CMDLINE_STAR.get() > 0 {
        // One star per character, not per byte.
        let mut i = 0;
        while i < len {
            msg_putchar(::core::ffi::c_int::from(b'*'));
            i += cluster_len_at(cc, start + i).max(1);
        }
    } else if !cc.last_colors.chunks().is_empty() {
        // Indexed rather than iterated: `msg_display_bytes` re-enters the
        // editor, which can replace the whole command line, so no borrow
        // of it may outlive one access.
        let mut i: size_t = 0;
        while i < cc.last_colors.chunks().len() {
            let chunk: CmdlineColorChunk = cc.last_colors.chunks()[i];
            if chunk.end > start {
                let chunk_start = chunk.start.max(start);
                display_span(cc, chunk_start, chunk.end - chunk_start, chunk.hl_id);
            }
            i += 1;
        }
    } else {
        display_span(cc, start, len, 0);
    }
}

/// Show `len` bytes of the command line from `start`, cut at the end of the
/// text, in `hl_id`: what C handed the message code as a pointer and a
/// length.
///
/// The bytes are copied first, because the message code re-enters the
/// editor (a UI's flush can run Lua), which may change the line under it.
/// A typed line fits the buffer on the stack; this runs for every key.
fn display_span(
    cc: Cc,
    start: ::core::ffi::c_int,
    len: ::core::ffi::c_int,
    hl_id: ::core::ffi::c_int,
) {
    let mut room = [0u8; 64];
    let line = cc.bytes();
    let from = usize::try_from(start).unwrap_or(0).min(line.len());
    let to = from
        .saturating_add(usize::try_from(len).unwrap_or(0))
        .min(line.len());
    let text = &line[from..to];
    if let Some(copy) = room.get_mut(..text.len()) {
        copy.copy_from_slice(text);
        msg_display_bytes(copy, hl_id, false);
    } else {
        let copy = text.to_vec();
        msg_display_bytes(&copy, hl_id, false);
    }
}

/// Put character `c` on the command line, shifting the text after the cursor
/// right when `shift` is set.  Used for CTRL-V, CTRL-K and the like; `c` must
/// be printable and fit in one display cell.
pub fn putcmdline(c: ::core::ffi::c_char, shift: bool) {
    if CMD_SILENT.get() {
        return;
    }
    let mut cc = Cc::current();
    if !ui_has(UI_CMDLINE) {
        MSG_NO_MORE.set(true);
        msg_putchar(::core::ffi::c_int::from(c));
        if shift {
            draw_cmdline(cc.cmdpos, cc.len() - cc.cmdpos);
        }
        MSG_NO_MORE.set(false);
    } else if cc.redraw_state != CMD_REDRAW_ALL {
        ui_call_cmdline_special_char(
            ApiString::from_bytes(cstr::until_nul(&[c.cast_unsigned()])),
            shift as Boolean,
            Integer::from(cc.level),
        );
    }
    cursorcmd();
    cc.special_char = c;
    cc.special_shift = shift;
    ui_cursor_shape();
}

/// Undo a `putcmdline(c, false)`.
pub fn unputcmdline() {
    if CMD_SILENT.get() {
        return;
    }
    let mut cc = Cc::current();
    MSG_NO_MORE.set(true);
    if cc.len() == cc.cmdpos && !ui_has(UI_CMDLINE) {
        msg_putchar(::core::ffi::c_int::from(b' '));
    } else {
        draw_cmdline(cc.cmdpos, cluster_len_at(cc, cc.cmdpos));
    }
    MSG_NO_MORE.set(false);
    cursorcmd();
    cc.special_char = 0;
    ui_cursor_shape();
}

/// Insert `text` into the command line at the cursor.
///
/// With `redraw`, the new part of the command line and the rest of it are
/// redrawn.  Two calls in a row should pass `false` and be followed by
/// [`redrawcmd`].
pub fn put_on_cmdline(text: &[u8], redraw: bool) {
    let mut len = ::core::ffi::c_int::try_from(text.len()).expect("a command line fits a c_int");

    let mut cc = Cc::current();
    realloc_cmdbuff(cc, cc.len() + len + 1);

    let pos = cc.cmdpos;
    if !cc.overstrike {
        cc.cmdbuff.splice(pos, pos, text);
    } else {
        // Count the characters in the new string.
        let mut m = 0;
        let mut i = 0;
        while i < text.len() {
            m += 1;
            i += c_cluster_len(&text[i..]).max(1);
        }
        // Count the bytes in the command line those characters
        // overwrite.
        let mut i = pos;
        while i < cc.len() && m > 0 {
            m -= 1;
            i += cluster_len_at(cc, i).max(1);
        }
        let to = i.min(cc.len());
        cc.cmdbuff.splice(pos, to, text);
    }

    // When the inserted text starts with a composing character, back up
    // to the character before it.
    if cc.cmdpos > 0 && cc.byte_at(cc.cmdpos) >= 0x80 {
        let back = head_off(cc.bytes(), usize::try_from(cc.cmdpos).unwrap_or(0));
        let back = ::core::ffi::c_int::try_from(back).unwrap_or(0);
        if back != 0 {
            cc.cmdpos -= back;
            len += back;
            cc.cmdspos = cmd_screencol(cc.cmdpos);
        }
    }

    if redraw && !CMD_SILENT.get() {
        MSG_NO_MORE.set(true);
        let row_before = CMDLINE_ROW.get();
        cursorcmd();
        draw_cmdline(cc.cmdpos, cc.len() - cc.cmdpos);
        // Avoid clearing the rest of the line too often.
        if CMDLINE_ROW.get() != row_before || cc.overstrike {
            msg_clr_eos();
        }
        MSG_NO_MORE.set(false);
    }

    // The maximum column; an overflowed product reads as negative.
    let m = if KEY_TYPED.get() {
        let cells = COLUMNS.get() * ROWS.get();
        if cells < 0 { MAXCOL } else { cells }
    } else {
        MAXCOL
    };

    let mut i = 0;
    while i < len {
        let mut c = cmdline_charsize(cc.cmdpos);
        // Count ">" for a double-wide character that doesn't fit.
        cc.cmdspos = correct_screencol(cc.cmdpos, c, cc.cmdspos);
        // Stop the cursor at the end of the screen, but do advance the
        // insert position, so that entering a very long command works
        // even though it cannot all be seen.
        if cc.cmdspos + c < m {
            cc.cmdspos += c;
        }
        c = (cluster_len_at(cc, cc.cmdpos) - 1).min(len - i - 1);
        cc.cmdpos += c + 1;
        i += c + 1;
    }

    if redraw {
        msg_check();
    }
}

/// Redraw the command line after a screen size change, an incremental search
/// or anything else that may have overwritten it.
pub fn redrawcmdline() {
    if CMD_SILENT.get() {
        return;
    }
    NEED_WAIT_RETURN.set(false);
    compute_cmdrow();
    redrawcmd();
    cursorcmd();
    ui_cursor_shape();
}

/// Draw the `:` / `/` / `?` and the prompt in front of the command line, and
/// record the indent they take up.
pub(crate) fn redrawcmdprompt() {
    if CMD_SILENT.get() {
        return;
    }
    let mut cc = Cc::current();
    if ui_has(UI_CMDLINE) {
        cc.redraw_state = CMD_REDRAW_ALL;
        return;
    }
    if cc.cmdfirstc != 0 {
        msg_putchar(::core::ffi::c_int::from(cc.cmdfirstc));
    }
    // A copy: the message code re-enters the editor.
    if let Some(prompt) = cc.cmdprompt.clone() {
        msg_str_hl(prompt.as_cstr(), cc.hl_id, false);
        cc.cmdindent = MSG_COL.get() + (MSG_ROW.get() - CMDLINE_ROW.get()) * COLUMNS.get();
        // The reverse of cmd_startcol().
        if cc.cmdfirstc != 0 {
            cc.cmdindent -= 1;
        }
    } else {
        let mut i = cc.cmdindent;
        while i > 0 {
            msg_putchar(::core::ffi::c_int::from(b' '));
            i -= 1;
        }
    }
}

/// Redraw what is currently on the command line.
pub fn redrawcmd() {
    if CMD_SILENT.get() {
        return;
    }

    let mut cc = Cc::current();
    if ui_has(UI_CMDLINE) {
        draw_cmdline(0, cc.len());
        return;
    }

    // With 'incsearch' there may be no command line while redrawing.
    if !cc.in_use() {
        msg_cursor_goto(CMDLINE_ROW.get(), 0);
        msg_clr_eos();
        return;
    }

    REDRAWING_CMDLINE.set(true);

    sb_text_restart_cmdline();
    msg_start();
    redrawcmdprompt();

    // Don't use the more prompt; truncate the command line if it doesn't
    // fit.
    MSG_NO_MORE.set(true);
    draw_cmdline(0, cc.len());
    msg_clr_eos();
    MSG_NO_MORE.set(false);

    cc.cmdspos = cmd_screencol(cc.cmdpos);

    if ::core::ffi::c_int::from(cc.special_char) != NUL {
        putcmdline(cc.special_char, cc.special_shift);
    }

    // An earlier emsg() may have set msg_scroll; in command-line mode it
    // can be reset now, so the next message overwrites the command line.
    MSG_SCROLL.set(0);

    // Typing ':' at the more prompt may set skip_redraw, which is not
    // wanted in command-line mode.
    SKIP_REDRAW.set(false);
    CMDLINE_WAS_LAST_DRAWN.set(true);
    REDRAWING_CMDLINE.set(false);
}

/// Recompute the screen row the command line lives on.
pub fn compute_cmdrow() {
    if EXMODE_ACTIVE.get() || MSG_SCROLLED.get() != 0 {
        CMDLINE_ROW.set(ROWS.get() - 1);
    } else {
        let wp = lastwin_nofloating(None);
        CMDLINE_ROW.set(
            wp.w_winrow + wp.w_height + wp.w_hsep_height + wp.w_status_height + global_stl_height(),
        );
    }
    if CMDLINE_ROW.get() == ROWS.get() && p_ch() > 0 {
        CMDLINE_ROW.set(CMDLINE_ROW.get() - 1);
    }
    LINES_LEFT.set(CMDLINE_ROW.get());
}

/// Move the screen cursor to the command line's cursor position.
pub fn cursorcmd() {
    if CMD_SILENT.get() || ui_has(UI_CMDLINE) {
        return;
    }

    let cmdspos = Cc::current().cmdspos;
    MSG_ROW.set(CMDLINE_ROW.get() + cmdspos / COLUMNS.get());
    MSG_COL.set(cmdspos % COLUMNS.get());
    MSG_ROW.set(MSG_ROW.get().min(ROWS.get() - 1));

    msg_cursor_goto(MSG_ROW.get(), MSG_COL.get());
}

/// Move the screen cursor to the start of the command line, clearing the
/// bottom lines when `clr` is set.
pub fn gotocmdline(clr: bool) {
    if ui_has(UI_CMDLINE) {
        return;
    }
    msg_start();
    // Always start in column 0.
    MSG_COL.set(0);
    if clr {
        // Clear the bottom line(s); this resets clear_cmdline.
        msg_clr_eos();
    }
    msg_cursor_goto(CMDLINE_ROW.get(), 0);
}
