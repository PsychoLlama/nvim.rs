//! Putting the command line on the screen.
//!
//! [`draw_cmdline`] writes the buffer out with the colour chunks
//! [`super::color`] computed; [`redrawcmd`] is the whole-line redraw and
//! [`cursorcmd`] the cursor placement.  [`put_on_cmdline`] is the other
//! direction — inserting text into the buffer and recomputing the screen
//! columns it now occupies.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::NUL;

/// The screen width of the command-line byte at `idx`.
pub(crate) unsafe fn cmdline_charsize(idx: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        if cmdline_star.get() > 0 {
            // Showing '*': always one position.
            return 1;
        }
        ptr2cells(Cc::current().at(idx))
    }
}

/// The screen column the command line's first byte occupies: the indent, plus
/// one for the `:` / `/` / `?` when there is one.
pub(crate) fn cmd_startcol() -> ::core::ffi::c_int {
    let cc = Cc::current();
    cc.cmdindent + i32::from(cc.cmdfirstc != NUL)
}

/// The screen column for a byte position on the command line.
pub unsafe fn cmd_screencol(bytepos: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        let mut cc = Cc::current();
        let mut col = cmd_startcol();

        // The maximum column. A weird 'columns'/'lines' can overflow the
        // product, which reads as negative.
        let m = if KeyTyped.get() {
            let cells = if !cmdline_win.get().is_null() {
                (*cmdline_win.get()).w_view_width * (*cmdline_win.get()).w_view_height
            } else {
                Columns.get() * Rows.get()
            };
            if cells < 0 { MAXCOL } else { cells }
        } else {
            MAXCOL
        };

        let mut i = 0;
        while i < cc.len() && i < bytepos {
            let c = cmdline_charsize(i);
            // Count ">" for a double-wide character that doesn't fit.
            correct_screencol(i, c, &raw mut col);

            // If the command line doesn't fit, show the cursor on the last
            // visible character. Don't move the cursor itself, so text can
            // still be appended.
            col += c;
            if col >= m {
                col -= c;
                break;
            }
            i += utfc_ptr2len(cc.at(i));
        }
        col
    }
}

/// If the character at `idx` is a `cells`-wide multi-byte character that does
/// not fit on the line, account for the ">" that will be displayed instead.
pub(crate) unsafe fn correct_screencol(
    idx: ::core::ffi::c_int,
    cells: ::core::ffi::c_int,
    col: *mut ::core::ffi::c_int,
) {
    unsafe {
        let at = Cc::current().at(idx);
        if utfc_ptr2len(at) > 1
            && utf_ptr2cells(at) > 1
            && *col % Columns.get() + cells > Columns.get()
        {
            *col += 1;
        }
    }
}

/// Draw `len` bytes of the command line from `start`, at the cursor position
/// — or stars, when `cmdline_star` is set.
pub(crate) unsafe fn draw_cmdline(start: ::core::ffi::c_int, len: ::core::ffi::c_int) {
    unsafe {
        let mut cc = Cc::current();
        if !cc.in_use() || !color_cmdline(cc) {
            return;
        }

        if ui_has(kUICmdline) {
            cc.special_char = NUL as ::core::ffi::c_char;
            cc.redraw_state = kCmdRedrawAll;
            return;
        }

        if cmdline_star.get() > 0 {
            // One star per character, not per byte.
            let mut i = 0;
            while i < len {
                msg_putchar('*' as ::core::ffi::c_int);
                i += utfc_ptr2len(cc.at(start + i));
            }
        } else if cc.last_colors.colors.size != 0 {
            let mut i: size_t = 0;
            while i < cc.last_colors.colors.size {
                let chunk: CmdlineColorChunk = *cc.last_colors.colors.items.add(i);
                if chunk.end > start {
                    let chunk_start = chunk.start.max(start);
                    msg_outtrans_len(
                        cc.at(chunk_start),
                        chunk.end - chunk_start,
                        chunk.hl_id,
                        false,
                    );
                }
                i += 1;
            }
        } else {
            msg_outtrans_len(cc.at(start), len, 0, false);
        }
    }
}

/// Put character `c` on the command line, shifting the text after the cursor
/// right when `shift` is set.  Used for CTRL-V, CTRL-K and the like; `c` must
/// be printable and fit in one display cell.
pub unsafe fn putcmdline(c: ::core::ffi::c_char, shift: bool) {
    unsafe {
        if cmd_silent.get() {
            return;
        }
        let mut cc = Cc::current();
        if !ui_has(kUICmdline) {
            msg_no_more.set(true);
            msg_putchar(c as ::core::ffi::c_int);
            if shift {
                draw_cmdline(cc.cmdpos, cc.len() - cc.cmdpos);
            }
            msg_no_more.set(false);
        } else if cc.redraw_state != kCmdRedrawAll {
            let mut charbuf: [::core::ffi::c_char; 2] = [c, 0];
            ui_call_cmdline_special_char(
                cstr_as_string(charbuf.as_mut_ptr()),
                shift as Boolean,
                cc.level as Integer,
            );
        }
        cursorcmd();
        cc.special_char = c;
        cc.special_shift = shift;
        ui_cursor_shape();
    }
}

/// Undo a `putcmdline(c, false)`.
pub unsafe fn unputcmdline() {
    unsafe {
        if cmd_silent.get() {
            return;
        }
        let mut cc = Cc::current();
        msg_no_more.set(true);
        if cc.len() == cc.cmdpos && !ui_has(kUICmdline) {
            msg_putchar(' ' as ::core::ffi::c_int);
        } else {
            draw_cmdline(
                cc.cmdpos,
                utfc_ptr2len(cc.text().offset(cc.cmdpos as isize)),
            );
        }
        msg_no_more.set(false);
        cursorcmd();
        cc.special_char = NUL as ::core::ffi::c_char;
        ui_cursor_shape();
    }
}

/// Insert `len` bytes of `str` into the command line at the cursor; `len` of
/// −1 means `strlen(str)`.
///
/// With `redraw`, the new part of the command line and the rest of it are
/// redrawn.  Two calls in a row should pass `false` and be followed by
/// [`redrawcmd`].
pub unsafe fn put_on_cmdline(
    str: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    redraw: bool,
) {
    unsafe {
        if len < 0 {
            len = strlen(str) as ::core::ffi::c_int;
        }

        let mut cc = Cc::current();
        realloc_cmdbuff(cc, cc.len() + len + 1);

        if cc.overstrike == 0 {
            memmove(
                cc.at(cc.cmdpos + len) as *mut ::core::ffi::c_void,
                cc.text().offset(cc.cmdpos as isize) as *const ::core::ffi::c_void,
                (cc.len() - cc.cmdpos) as size_t,
            );
            cc.set_len(cc.len() + (len));
        } else {
            // Count the characters in the new string.
            let mut m = 0;
            let mut i = 0;
            while i < len {
                m += 1;
                i += utfc_ptr2len(str.offset(i as isize));
            }
            // Count the bytes in the command line those characters
            // overwrite.
            i = cc.cmdpos;
            while i < cc.len() && m > 0 {
                m -= 1;
                i += utfc_ptr2len(cc.at(i));
            }
            if i < cc.len() {
                memmove(
                    cc.at(cc.cmdpos + len) as *mut ::core::ffi::c_void,
                    cc.at(i) as *const ::core::ffi::c_void,
                    (cc.len() - i) as size_t,
                );
                cc.set_len(cc.len() + (cc.cmdpos + len - i));
            } else {
                cc.set_len(cc.cmdpos + len);
            }
        }
        memmove(
            cc.text().offset(cc.cmdpos as isize) as *mut ::core::ffi::c_void,
            str as *const ::core::ffi::c_void,
            len as size_t,
        );
        *cc.text().offset(cc.len() as isize) = NUL as ::core::ffi::c_char;

        // When the inserted text starts with a composing character, back up
        // to the character before it.
        if cc.cmdpos > 0
            && *cc.text().offset(cc.cmdpos as isize) as uint8_t as ::core::ffi::c_int >= 0x80
        {
            let head_off = utf_head_off(cc.text(), cc.text().offset(cc.cmdpos as isize));
            if head_off != 0 {
                cc.cmdpos -= head_off;
                len += head_off;
                cc.cmdspos = cmd_screencol(cc.cmdpos);
            }
        }

        if redraw && !cmd_silent.get() {
            msg_no_more.set(true);
            let row_before = cmdline_row.get();
            cursorcmd();
            draw_cmdline(cc.cmdpos, cc.len() - cc.cmdpos);
            // Avoid clearing the rest of the line too often.
            if cmdline_row.get() != row_before || cc.overstrike != 0 {
                msg_clr_eos();
            }
            msg_no_more.set(false);
        }

        // The maximum column; an overflowed product reads as negative.
        let m = if KeyTyped.get() {
            let cells = Columns.get() * Rows.get();
            if cells < 0 { MAXCOL } else { cells }
        } else {
            MAXCOL
        };

        let mut i = 0;
        while i < len {
            let mut c = cmdline_charsize(cc.cmdpos);
            // Count ">" for a double-wide character that doesn't fit.
            correct_screencol(cc.cmdpos, c, &raw mut cc.cmdspos);
            // Stop the cursor at the end of the screen, but do advance the
            // insert position, so that entering a very long command works
            // even though it cannot all be seen.
            if cc.cmdspos + c < m {
                cc.cmdspos += c;
            }
            c = (utfc_ptr2len(cc.text().offset(cc.cmdpos as isize)) - 1).min(len - i - 1);
            cc.cmdpos += c + 1;
            i += c + 1;
        }

        if redraw {
            msg_check();
        }
    }
}

/// Redraw the command line after a screen size change, an incremental search
/// or anything else that may have overwritten it.
pub unsafe fn redrawcmdline() {
    unsafe {
        if cmd_silent.get() {
            return;
        }
        need_wait_return.set(false);
        compute_cmdrow();
        redrawcmd();
        cursorcmd();
        ui_cursor_shape();
    }
}

/// Draw the `:` / `/` / `?` and the prompt in front of the command line, and
/// record the indent they take up.
pub(crate) unsafe fn redrawcmdprompt() {
    unsafe {
        if cmd_silent.get() {
            return;
        }
        let mut cc = Cc::current();
        if ui_has(kUICmdline) {
            cc.redraw_state = kCmdRedrawAll;
            return;
        }
        if cc.cmdfirstc != NUL {
            msg_putchar(cc.cmdfirstc);
        }
        if !cc.cmdprompt.is_null() {
            msg_puts_hl(cc.cmdprompt, cc.hl_id, false);
            cc.cmdindent = msg_col.get() + (msg_row.get() - cmdline_row.get()) * Columns.get();
            // The reverse of cmd_startcol().
            if cc.cmdfirstc != NUL {
                cc.cmdindent -= 1;
            }
        } else {
            let mut i = cc.cmdindent;
            while i > 0 {
                msg_putchar(' ' as ::core::ffi::c_int);
                i -= 1;
            }
        }
    }
}

/// Redraw what is currently on the command line.
pub unsafe fn redrawcmd() {
    unsafe {
        if cmd_silent.get() {
            return;
        }

        let mut cc = Cc::current();
        if ui_has(kUICmdline) {
            draw_cmdline(0, cc.len());
            return;
        }

        // With 'incsearch' there may be no command line while redrawing.
        if !cc.in_use() {
            msg_cursor_goto(cmdline_row.get(), 0);
            msg_clr_eos();
            return;
        }

        redrawing_cmdline.set(true);

        sb_text_restart_cmdline();
        msg_start();
        redrawcmdprompt();

        // Don't use the more prompt; truncate the command line if it doesn't
        // fit.
        msg_no_more.set(true);
        draw_cmdline(0, cc.len());
        msg_clr_eos();
        msg_no_more.set(false);

        cc.cmdspos = cmd_screencol(cc.cmdpos);

        if cc.special_char as ::core::ffi::c_int != NUL {
            putcmdline(cc.special_char, cc.special_shift);
        }

        // An earlier emsg() may have set msg_scroll; in command-line mode it
        // can be reset now, so the next message overwrites the command line.
        msg_scroll.set(0);

        // Typing ':' at the more prompt may set skip_redraw, which is not
        // wanted in command-line mode.
        skip_redraw.set(false);
        cmdline_was_last_drawn.set(true);
        redrawing_cmdline.set(false);
    }
}

/// Recompute the screen row the command line lives on.
pub unsafe fn compute_cmdrow() {
    unsafe {
        if exmode_active.get() || msg_scrolled.get() != 0 {
            cmdline_row.set(Rows.get() - 1);
        } else {
            let wp = lastwin_nofloating(::core::ptr::null_mut::<tabpage_T>());
            cmdline_row.set(
                (*wp).w_winrow
                    + (*wp).w_height
                    + (*wp).w_hsep_height
                    + (*wp).w_status_height
                    + global_stl_height(),
            );
        }
        if cmdline_row.get() == Rows.get() && p_ch.get() > 0 {
            cmdline_row.set(cmdline_row.get() - 1);
        }
        lines_left.set(cmdline_row.get());
    }
}

/// Move the screen cursor to the command line's cursor position.
pub unsafe fn cursorcmd() {
    unsafe {
        if cmd_silent.get() || ui_has(kUICmdline) {
            return;
        }

        let cmdspos = Cc::current().cmdspos;
        msg_row.set(cmdline_row.get() + cmdspos / Columns.get());
        msg_col.set(cmdspos % Columns.get());
        msg_row.set(msg_row.get().min(Rows.get() - 1));

        msg_cursor_goto(msg_row.get(), msg_col.get());
    }
}

/// Move the screen cursor to the start of the command line, clearing the
/// bottom lines when `clr` is set.
pub unsafe fn gotocmdline(clr: bool) {
    unsafe {
        if ui_has(kUICmdline) {
            return;
        }
        msg_start();
        // Always start in column 0.
        msg_col.set(0);
        if clr {
            // Clear the bottom line(s); this resets clear_cmdline.
            msg_clr_eos();
        }
        msg_cursor_goto(cmdline_row.get(), 0);
    }
}
