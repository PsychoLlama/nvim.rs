//! Adding lines -- `:append`, `:insert`, `:change` and `:z`.
//!
//! [`ex_append`] reads lines from the command line's input stream until a lone
//! `.`, honouring 'autoindent' ([`append_indent`]) and the `:change` variant
//! ([`ex_change`]) that deletes the range first.  [`ex_z`] is the paging
//! command: print a window of lines around a position, with the `+`/`-`/`=`/
//! `.`/`^` forms picking which window and `:z#` numbering it.
//!
//! Original: `src/nvim/ex_cmds.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::lines::set_op_range;
use super::{
    B_IMODE_LMAP, BL_FIX, BL_SOL, CMD_append, CMD_change, EXFLAG_LIST, EXFLAG_NR, FAIL, ML_EMPTY,
    NL, NUL, print_line, true_0,
};
use crate::change::{appended_lines, appended_lines_mark, deleted_lines_mark};
use crate::cursor::check_cursor_lnum;
use crate::edit::beginline;
use crate::global_cell::GlobalCell;
use crate::indent::get_indent_lnum;
use crate::main::{
    Columns, Rows, State, curbuf, curwin, ex_no_reprint, firstwin, lastwin, lines_left, msg_scroll,
    need_wait_return, p_window,
};
use crate::memline::{ml_append, ml_delete};
use crate::memory::{xfree, xmemdupz, xstrdup};
use crate::message::{emsg, msg_putchar};
use crate::os::cshim::gettext;
use crate::state::{MODE_CMDLINE, MODE_INSERT, MODE_LANGMAP, MODE_NORMAL};
use crate::strings::vim_strchr;
use crate::types::{OptInt, exarg_T, int64_t, linenr_T, size_t};
use crate::ui::ui_cursor_shape;
use crate::undo::u_save;
use ::libc::{atol, strlen};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// The indent `:append`/`:insert`/`:change` gives the first line it reads,
/// taken from the line the command started on.  `-1` once it has been used.
static append_indent: GlobalCell<c_int> = GlobalCell::new(0);

/// `:insert` and `:append`, also used by [`ex_change`].
///
/// # Safety
/// `eap` must be a live Ex command whose range is inside the current buffer.
pub unsafe fn ex_append(eap: *mut exarg_T) {
    let mut did_undo = false;
    // SAFETY: caller's contract.
    let (cmdidx, forceit, line2) = unsafe { ((*eap).cmdidx, (*eap).forceit, (*eap).line2) };
    let mut lnum = line2;
    let mut indent = 0;
    // SAFETY: `curbuf` is the live current buffer.
    let mut empty = unsafe { (*curbuf.get()).b_ml.ml_flags } & ML_EMPTY != 0;

    // The ! flag toggles autoindent.
    if forceit != 0 {
        // SAFETY: as above.
        unsafe { toggle_autoindent() };
    }

    // First autoindent comes from the line we start on.
    // SAFETY: as above.
    if cmdidx != CMD_change && unsafe { (*curbuf.get()).b_p_ai } != 0 && lnum > 0 {
        // SAFETY: `lnum` is a line of the current buffer.
        append_indent.set(unsafe { get_indent_lnum(lnum) });
    }

    if cmdidx != CMD_append {
        lnum -= 1;
    }
    // When the buffer is empty we need to delete the dummy line.
    if empty && lnum == 1 {
        lnum = 0;
    }

    // Behave like in Insert mode.
    State.set(MODE_INSERT);
    // SAFETY: `curbuf` is live.
    if unsafe { (*curbuf.get()).b_p_iminsert } == B_IMODE_LMAP as OptInt {
        State.set(State.get() | MODE_LANGMAP);
    }

    loop {
        msg_scroll.set(true_0);
        need_wait_return.set(false);
        // SAFETY: `curbuf` is live; `lnum` is a line of it, or zero.
        unsafe {
            if (*curbuf.get()).b_p_ai != 0 {
                if append_indent.get() >= 0 {
                    indent = append_indent.replace(-1);
                } else if lnum > 0 {
                    indent = get_indent_lnum(lnum);
                }
            }
        }

        // SAFETY: caller's contract.
        let Some(theline) = (unsafe { next_append_line(eap, indent) }) else {
            break;
        };
        lines_left.set(Rows.get() - 1);
        if theline.is_null() {
            break;
        }

        // Look for the "." after the automatic indent.
        // SAFETY: every source above hands back a NUL-terminated string.
        let text = unsafe { CStr::from_ptr(theline) }.to_bytes();
        let mut vcol = 0;
        let mut typed = 0;
        while indent > vcol && typed < text.len() {
            match text[typed] {
                b' ' => vcol += 1,
                b'\t' => vcol += 8 - vcol % 8,
                _ => break,
            }
            typed += 1;
        }

        let ended = &text[typed..] == b".";
        // SAFETY: `lnum` is a line of the current buffer, or zero.
        let undo_failed = !ended
            && !did_undo
            && unsafe { u_save(lnum, lnum + 1 + linenr_T::from(empty)) } == FAIL;
        if ended || undo_failed {
            // SAFETY: the line is ours.
            unsafe { xfree(theline.cast()) };
            break;
        }

        // Don't use autoindent if nothing was typed.
        // SAFETY: as above; `theline` is at least one byte long.
        if text.len() == typed {
            unsafe { *theline = NUL as c_char };
        }

        did_undo = true;
        // SAFETY: `lnum` is a line of the current buffer, or zero.
        unsafe {
            ml_append(lnum, theline, 0, false);
            if empty {
                // There are no marks below the inserted lines.
                appended_lines(lnum, 1);
            } else {
                appended_lines_mark(lnum, 1);
            }
            xfree(theline.cast());
        }
        lnum += 1;

        if empty {
            // SAFETY: the dummy line the buffer started with.
            unsafe { ml_delete(2) };
            empty = false;
        }
    }

    State.set(MODE_NORMAL);
    // SAFETY: cursor state, main thread.
    unsafe { ui_cursor_shape() };

    if forceit != 0 {
        // SAFETY: `curbuf` is live.
        unsafe { toggle_autoindent() };
    }

    // "start" is set to eap->line2+1 unless that position is invalid (when
    // eap->line2 pointed to the end of the buffer and nothing was appended);
    // "end" is set to lnum when something has been appended, otherwise
    // it is the same as "start"  -- Acevedo
    // SAFETY: `curbuf` is live.
    let mut start = unsafe { (*curbuf.get()).b_ml.ml_line_count };
    if line2 < start {
        start = line2 + 1;
    }
    if cmdidx != CMD_append {
        start -= 1;
    }
    // SAFETY: `curbuf` is live.
    unsafe { set_op_range(start, if line2 < lnum { lnum } else { start }) };

    // SAFETY: `curwin` is the live current window.
    unsafe {
        (*curwin.get()).w_cursor.lnum = lnum;
        check_cursor_lnum(curwin.get());
        beginline((BL_SOL | BL_FIX) as c_int);
    }

    // Don't use wait_return() now.
    need_wait_return.set(false);
    ex_no_reprint.set(true);
}

/// Flip 'autoindent' for the duration of a `!` command.
///
/// # Safety
/// The current buffer must be live.
unsafe fn toggle_autoindent() {
    // SAFETY: caller's contract.
    unsafe { (*curbuf.get()).b_p_ai = c_int::from((*curbuf.get()).b_p_ai == 0) };
}

/// The next line for `:append` to insert, freshly allocated.
///
/// There are three sources, in upstream's order: the text after a trailing
/// bar, the lines that follow the command in the same script, and the command
/// line's own `getline` callback.  `None` means the second source ran out,
/// which upstream leaves the loop for *without* resetting `lines_left`;
/// `Some(NULL)` is the callback saying the input ended.
///
/// # Safety
/// `eap` must be a live Ex command.
unsafe fn next_append_line(eap: *mut exarg_T, indent: c_int) -> Option<*mut c_char> {
    // SAFETY: caller's contract.
    unsafe {
        let arg = (*eap).arg;
        if *arg == '|' as c_char {
            // Get the text after the trailing bar.
            let line = xstrdup(arg.add(1));
            *arg = NUL as c_char;
            return Some(line);
        }

        let Some(getline) = (*eap).ea_getline else {
            // No getline() function: use the lines that follow.  This ends
            // when there is no more.
            let next = (*eap).nextcmd;
            if next.is_null() {
                return None;
            }
            let mut end = vim_strchr(next, NL);
            if end.is_null() {
                end = next.add(strlen(next));
            }
            let line = xmemdupz(next.cast(), end.offset_from(next) as size_t).cast::<c_char>();
            (*eap).nextcmd = if *end != NUL as c_char {
                end.add(1)
            } else {
                ptr::null_mut()
            };
            return Some(line);
        };

        // Set State to avoid the cursor shape being set to MODE_INSERT state
        // when getline() returns.
        let save_state = State.replace(MODE_CMDLINE);
        let first = if (*(*eap).cstack).cs_looplevel > 0 {
            -1
        } else {
            NUL
        };
        let line = getline(first, (*eap).cookie, indent, true);
        State.set(save_state);
        Some(line)
    }
}

/// `:change` -- delete the range, then append in its place.
///
/// # Safety
/// `eap` must be a live Ex command whose range is inside the current buffer.
pub unsafe fn ex_change(eap: *mut exarg_T) {
    // SAFETY: caller's contract.
    let (forceit, line1, line2) = unsafe { ((*eap).forceit, (*eap).line1, (*eap).line2) };
    // SAFETY: the range is inside the current buffer.
    if line2 >= line1 && unsafe { u_save(line1 - 1, line2 + 1) } == FAIL {
        return;
    }

    // The ! flag toggles autoindent.
    // SAFETY: `curbuf` is live.
    let autoindent = unsafe { (*curbuf.get()).b_p_ai };
    if if forceit != 0 {
        autoindent == 0
    } else {
        autoindent != 0
    } {
        // SAFETY: `line1` is a line of the current buffer.
        append_indent.set(unsafe { get_indent_lnum(line1) });
    }

    let mut lnum = line2;
    while lnum >= line1 {
        // SAFETY: `curbuf` is live and `line1` is a line of it.
        unsafe {
            if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
                // Nothing left to delete.
                break;
            }
            ml_delete(line1);
        }
        lnum -= 1;
    }

    // Make sure the cursor is not beyond the end of the file now.
    // SAFETY: `curwin` is the live current window.
    unsafe {
        check_cursor_lnum(curwin.get());
        deleted_lines_mark(line1, line2 - lnum);
        // ":append" on the line above the deleted lines.
        (*eap).line2 = line1;
        ex_append(eap);
    }
}

/// `:z` -- print a window of lines around the range's last line.
///
/// # Safety
/// `eap` must be a live Ex command whose range is inside the current buffer.
pub unsafe fn ex_z(eap: *mut exarg_T) {
    // SAFETY: caller's contract.
    let (arg, forceit, addr_count, flags, lnum) = unsafe {
        (
            (*eap).arg,
            (*eap).forceit,
            (*eap).addr_count,
            (*eap).flags,
            (*eap).line2,
        )
    };
    // SAFETY: the window layout and 'scroll' are live.
    let mut bigness = unsafe { default_bigness(forceit) }.max(1);

    // SAFETY: the command argument is NUL-terminated.
    let text = unsafe { CStr::from_ptr(arg) }.to_bytes();
    let kind = text.first().copied().unwrap_or(0);
    let mut at = usize::from(matches!(kind, b'-' | b'+' | b'=' | b'^' | b'.'));
    while matches!(text.get(at), Some(b'-' | b'+')) {
        at += 1;
    }

    if at < text.len() {
        if !text[at].is_ascii_digit() {
            // SAFETY: a static message.
            unsafe { emsg(gettext(c"E144: Non-numeric argument to :z".as_ptr())) };
            return;
        }
        // SAFETY: `at` indexes the argument's own bytes.
        bigness = unsafe { atol(arg.add(at)) };
        // `bigness` could be < 0 if atol() overflowed.
        // SAFETY: `curbuf` is live.
        let cap = int64_t::from(unsafe { (*curbuf.get()).b_ml.ml_line_count }) * 2;
        if bigness > cap || bigness < 0 {
            bigness = cap;
        }
        p_window.set(bigness as c_int as OptInt);
        if kind == b'=' {
            bigness += 2;
        }
    }

    // The number of '-' or '+' multiplies the distance.
    let mut repeat = 1;
    if kind == b'-' || kind == b'+' {
        while text.get(repeat) == Some(&kind) {
            repeat += 1;
        }
    }
    let repeat = repeat as linenr_T;
    let bigness = bigness as linenr_T;
    let half = (bigness + 1) / 2;

    // `minus` asks for the ruled line `:z=` draws around the current line.
    let mut minus = false;
    let (mut start, mut end, mut curs) = match kind {
        b'-' => {
            let start = lnum - bigness * repeat + 1;
            (start, start + bigness - 1, start + bigness - 1)
        }
        b'=' => {
            minus = true;
            (lnum - half + 1, lnum + half - 1, lnum)
        }
        b'^' => (lnum - bigness * 2, lnum - bigness, lnum - bigness),
        b'.' => (lnum - half + 1, lnum + half - 1, lnum + half - 1),
        // '+', and anything else
        _ => {
            let start = if kind == b'+' {
                lnum + bigness * (repeat - 1) + 1
            } else if addr_count == 0 {
                lnum + 1
            } else {
                lnum
            };
            (start, start + bigness - 1, start + bigness - 1)
        }
    };

    // SAFETY: `curbuf` is live.
    let last = unsafe { (*curbuf.get()).b_ml.ml_line_count };
    start = start.max(1);
    end = end.min(last);
    curs = curs.max(1).min(last);

    for i in start..=end {
        if minus && i == lnum {
            // SAFETY: message state, main thread.
            unsafe { rule_off() };
        }
        // SAFETY: `i` is a line of the current buffer.
        unsafe {
            print_line(
                i,
                flags & EXFLAG_NR != 0,
                flags & EXFLAG_LIST != 0,
                i == start,
            )
        };
        if minus && i == lnum {
            // SAFETY: message state, main thread.
            unsafe { rule_off() };
        }
    }

    // SAFETY: `curwin` is the live current window.
    unsafe {
        if (*curwin.get()).w_cursor.lnum != curs {
            (*curwin.get()).w_cursor.lnum = curs;
            (*curwin.get()).w_cursor.col = 0;
        }
    }
    ex_no_reprint.set(true);
}

/// How many lines `:z` shows: the display height for `:z!`, twice 'scroll' in
/// the only window, and the window's height less three otherwise.
///
/// # Safety
/// The window layout must be live.
unsafe fn default_bigness(forceit: c_int) -> int64_t {
    // SAFETY: caller's contract.
    unsafe {
        if forceit != 0 {
            int64_t::from(Rows.get() - 1)
        } else if firstwin.get() == lastwin.get() {
            (*curwin.get()).w_onebuf_opt.wo_scr * 2
        } else {
            int64_t::from((*curwin.get()).w_view_height - 3)
        }
    }
}

/// The line of dashes `:z=` rules the current line off with.
///
/// # Safety
/// Message state must be usable.
unsafe fn rule_off() {
    // SAFETY: caller's contract.
    unsafe {
        msg_putchar(NL);
        for _ in 1..Columns.get() {
            msg_putchar('-' as c_int);
        }
    }
}
