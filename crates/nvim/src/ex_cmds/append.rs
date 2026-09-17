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
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::lines::set_op_range;
use super::say;
use super::{B_IMODE_LMAP, EXFLAG_LIST, EXFLAG_NR, NL, print_line};
use crate::change::{appended_lines, appended_lines_mark, deleted_lines_mark};
use crate::cstr;
use crate::cursor::check_cursor_lnum;
use crate::edit::{BeginlineOpts, beginline};
use crate::ex_docmd::state::ex_no_reprint;
use crate::global_cell::GlobalCell;
use crate::indent::get_indent_lnum;
use crate::memline::MlFlags;
use crate::memline::{ml_append, ml_delete};
use crate::memory::{xfree, xmemdupz, xstrdup};
use crate::message::emsg;
use crate::message::state::{lines_left, msg_scroll, need_wait_return};
use crate::option::vars::P_WINDOW;
use crate::os::cshim::gettext;
use crate::state::mode::State;
use crate::state::{MODE_CMDLINE, MODE_INSERT, MODE_LANGMAP, MODE_NORMAL};
use crate::strings::vim_strchr;
use crate::types::CmdIdx;
use crate::types::{ExArg, LineNr, NUL, OptInt, int64_t, size_t};
use crate::ui::state::{Columns, Rows};
use crate::ui::ui_cursor_shape;
use crate::undo::u_save;
use crate::winlayer::Buf;
use crate::winlayer::Win;
use crate::winlayer::graph::{firstwin, lastwin};
use ::libc::atol;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// One line `:append` read, freed when it goes out of scope.
///
/// The seam: all three sources -- the text after a trailing bar, the script's
/// next line, and the command line's own `getline` callback -- answer an
/// `xmalloc`ed string, and the last of the three is a C callee's.  A NULL is
/// allowed and frees nothing; it is `getline` saying the input ended.
struct Line(*mut c_char);

impl Drop for Line {
    fn drop(&mut self) {
        // SAFETY: our own allocation, or NULL.
        unsafe { xfree(self.0.cast()) };
    }
}

/// The indent `:append`/`:insert`/`:change` gives the first line it reads,
/// taken from the line the command started on.  `-1` once it has been used.
static append_indent: GlobalCell<c_int> = GlobalCell::new(0);

/// `:insert` and `:append`, also used by [`ex_change`].
pub fn ex_append(excmd: &mut ExArg) {
    let mut did_undo = false;
    let (cmdidx, forceit, line2) = (excmd.cmdidx, excmd.forceit, excmd.line2);
    let mut lnum = line2;
    let mut indent = 0;
    // SAFETY: `curbuf` is the live current buffer.
    let mut empty = Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY);

    // The ! flag toggles autoindent.
    if forceit {
        toggle_autoindent();
    }

    // First autoindent comes from the line we start on.
    // SAFETY: as above.
    if cmdidx != CmdIdx::change && Buf::current().b_p_ai != 0 && lnum > 0 {
        // SAFETY: `lnum` is a line of the current buffer.
        append_indent.set(get_indent_lnum(lnum));
    }

    if cmdidx != CmdIdx::append {
        lnum -= 1;
    }
    // When the buffer is empty we need to delete the dummy line.
    if empty && lnum == 1 {
        lnum = 0;
    }

    // Behave like in Insert mode.
    State.set(MODE_INSERT);
    if Buf::current().b_p_iminsert == B_IMODE_LMAP as OptInt {
        State.set(State.get() | MODE_LANGMAP);
    }

    loop {
        msg_scroll.set(1);
        need_wait_return.set(false);
        if Buf::current().b_p_ai != 0 {
            if append_indent.get() >= 0 {
                indent = append_indent.replace(-1);
            } else if lnum > 0 {
                indent = get_indent_lnum(lnum);
            }
        }

        let Some(theline) = next_append_line(excmd, indent) else {
            break;
        };
        lines_left.set(Rows.get() - 1);
        if theline.0.is_null() {
            break;
        }

        // Look for the "." after the automatic indent.
        // SAFETY: every source above hands back a NUL-terminated string.
        let text = unsafe { CStr::from_ptr(theline.0) }.to_bytes();
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
        let undo_failed =
            !ended && !did_undo && u_save(lnum, lnum + 1 + LineNr::from(empty)).is_err();
        if ended || undo_failed {
            break;
        }

        // Don't use autoindent if nothing was typed.
        // SAFETY: as above; `theline` is at least one byte long.
        if text.len() == typed {
            unsafe { *theline.0 = NUL as c_char };
        }

        did_undo = true;
        // SAFETY: `lnum` is a line of the current buffer, or zero.
        let _ = unsafe { ml_append(lnum, theline.0, 0, false) };
        if empty {
            // There are no marks below the inserted lines.
            appended_lines(lnum, 1);
        } else {
            appended_lines_mark(lnum, 1);
        }
        lnum += 1;

        if empty {
            let _ = ml_delete(2);
            empty = false;
        }
    }

    State.set(MODE_NORMAL);
    ui_cursor_shape();

    if forceit {
        toggle_autoindent();
    }

    // "start" is set to eap->line2+1 unless that position is invalid (when
    // eap->line2 pointed to the end of the buffer and nothing was appended);
    // "end" is set to lnum when something has been appended, otherwise
    // it is the same as "start"  -- Acevedo
    let mut start = Buf::current().b_ml.ml_line_count;
    if line2 < start {
        start = line2 + 1;
    }
    if cmdidx != CmdIdx::append {
        start -= 1;
    }
    set_op_range(start, if line2 < lnum { lnum } else { start });

    // SAFETY: `curwin` is the live current window.
    Win::current().w_cursor.lnum = lnum;
    check_cursor_lnum(Win::current());
    beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);

    // Don't use wait_return() now.
    need_wait_return.set(false);
    ex_no_reprint.set(true);
}

/// Flip 'autoindent' for the duration of a `!` command.
fn toggle_autoindent() {
    Buf::current().b_p_ai = c_int::from(Buf::current().b_p_ai == 0);
}

/// The next line for `:append` to insert, freshly allocated.
///
/// There are three sources, in upstream's order: the text after a trailing
/// bar, the lines that follow the command in the same script, and the command
/// line's own `getline` callback.  `None` means the second source ran out,
/// which upstream leaves the loop for *without* resetting `lines_left`;
/// `Some(NULL)` is the callback saying the input ended.
fn next_append_line(args: &mut ExArg, indent: c_int) -> Option<Line> {
    let arg = args.arg;
    // SAFETY: caller's contract.
    if unsafe { *arg } == '|' as c_char {
        // Get the text after the trailing bar.
        let line = unsafe { xstrdup(arg.add(1)) };
        unsafe { *arg = NUL as c_char };
        return Some(Line(line));
    }

    let Some(getline) = args.ea_getline else {
        // No getline() function: use the lines that follow.  This ends
        // when there is no more.
        let next = args.nextcmd;
        if next.is_null() {
            return None;
        }
        // SAFETY: caller's contract -- the script's remaining text.
        let (line, rest) = unsafe {
            let mut end = vim_strchr(next, NL);
            if end.is_null() {
                end = next.add(cstr::bytes_at(next).len());
            }
            let line = xmemdupz(next.cast(), end.offset_from(next) as size_t).cast::<c_char>();
            let rest = if *end != NUL as c_char {
                end.add(1)
            } else {
                ptr::null_mut()
            };
            (line, rest)
        };
        args.nextcmd = rest;
        return Some(Line(line));
    };

    // Set State to avoid the cursor shape being set to MODE_INSERT state
    // when getline() returns.
    let save_state = State.replace(MODE_CMDLINE);
    // SAFETY: caller's contract -- the condition stack is the command's.
    let first = if unsafe { (*args.cstack).cs_looplevel } > 0 {
        -1
    } else {
        NUL
    };
    // SAFETY: the cookie is the one the getter was handed with.
    let line = unsafe { getline(first, args.cookie, indent, true) };
    State.set(save_state);
    Some(Line(line))
}

/// `:change` -- delete the range, then append in its place.
pub fn ex_change(excmd: &mut ExArg) {
    let (forceit, line1, line2) = (excmd.forceit, excmd.line1, excmd.line2);
    // SAFETY: the range is inside the current buffer.
    if line2 >= line1 && u_save(line1 - 1, line2 + 1).is_err() {
        return;
    }

    // The ! flag toggles autoindent.
    let autoindent = Buf::current().b_p_ai;
    if if forceit {
        autoindent == 0
    } else {
        autoindent != 0
    } {
        // SAFETY: `line1` is a line of the current buffer.
        append_indent.set(get_indent_lnum(line1));
    }

    let mut lnum = line2;
    while lnum >= line1 {
        if Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY) {
            // Nothing left to delete.
            break;
        }
        let _ = ml_delete(line1);
        lnum -= 1;
    }

    // Make sure the cursor is not beyond the end of the file now.
    // SAFETY: `curwin` is the live current window.
    check_cursor_lnum(Win::current());
    deleted_lines_mark(line1, line2 - lnum);
    // ":append" on the line above the deleted lines.
    excmd.line2 = line1;
    // SAFETY: the command block is the one borrowed here.
    ex_append(excmd);
}

/// `:z` -- print a window of lines around the range's last line.
pub fn ex_z(excmd: &mut ExArg) {
    // SAFETY: caller's contract.
    let excmd = &*excmd;
    let (arg, forceit, addr_count, flags, lnum) = (
        excmd.arg,
        excmd.forceit,
        excmd.addr_count,
        excmd.flags,
        excmd.line2,
    );
    let mut bigness = default_bigness(c_int::from(forceit)).max(1);

    // SAFETY: the command argument is NUL-terminated.
    let text = unsafe { CStr::from_ptr(arg) }.to_bytes();
    let kind = text.first().copied().unwrap_or(0);
    let mut at = usize::from(matches!(kind, b'-' | b'+' | b'=' | b'^' | b'.'));
    while matches!(text.get(at), Some(b'-' | b'+')) {
        at += 1;
    }

    if at < text.len() {
        if !text[at].is_ascii_digit() {
            emsg(gettext(c"E144: Non-numeric argument to :z"));
            return;
        }
        // SAFETY: `at` indexes the argument's own bytes.
        bigness = unsafe { atol(arg.add(at)) };
        // `bigness` could be < 0 if atol() overflowed.
        let cap = int64_t::from(Buf::current().b_ml.ml_line_count) * 2;
        if bigness > cap || bigness < 0 {
            bigness = cap;
        }
        P_WINDOW.set(bigness as c_int as OptInt);
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
    let repeat = repeat as LineNr;
    let bigness = bigness as LineNr;
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

    let last = Buf::current().b_ml.ml_line_count;
    start = start.max(1);
    end = end.min(last);
    curs = curs.max(1).min(last);

    for i in start..=end {
        if minus && i == lnum {
            rule_off();
        }
        print_line(
            i,
            flags & EXFLAG_NR != 0,
            flags & EXFLAG_LIST != 0,
            i == start,
        );
        if minus && i == lnum {
            rule_off();
        }
    }

    // SAFETY: `curwin` is the live current window.
    if Win::current().w_cursor.lnum != curs {
        Win::current().w_cursor.lnum = curs;
        Win::current().w_cursor.col = 0;
    }
    ex_no_reprint.set(true);
}

/// How many lines `:z` shows: the display height for `:z!`, twice 'scroll' in
/// the only window, and the window's height less three otherwise.
fn default_bigness(forceit: c_int) -> int64_t {
    if forceit != 0 {
        int64_t::from(Rows.get() - 1)
    } else if firstwin.get() == lastwin.get() {
        Win::current().w_onebuf_opt.wo_scr * 2
    } else {
        int64_t::from(Win::current().w_view_height - 3)
    }
}

/// The line of dashes `:z=` rules the current line off with.
fn rule_off() {
    say::putchar(NL);
    for _ in 1..Columns.get() {
        say::putchar('-' as c_int);
    }
}
