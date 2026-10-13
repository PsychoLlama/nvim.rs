//! Adding lines -- `:append`, `:insert`, `:change` and `:z`.
//!
//! [`ex_append`] reads lines from the command line's input stream until a lone
//! `.`, honouring 'autoindent' ([`APPEND_INDENT`]) and the `:change` variant
//! ([`ex_change`]) that deletes the range first.  [`ex_z`] is the paging
//! command: print a window of lines around a position, with the `+`/`-`/`=`/
//! `.`/`^` forms picking which window and `:z#` numbering it.
//!
//! Original: `src/nvim/ex_cmds.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

use super::lines::set_op_range;
use super::say;
use super::{B_IMODE_LMAP, EXFLAG_LIST, EXFLAG_NR, NL, print_line};
use crate::change::{appended_lines, appended_lines_mark, deleted_lines_mark};
use crate::cursor::check_cursor_lnum;
use crate::edit::{BeginlineOpts, beginline};
use crate::ex_docmd::state::ex_no_reprint;
use crate::global_cell::GlobalCell;
use crate::indent::get_indent_lnum;
use crate::memline::MlFlags;
use crate::memline::{ml_append_cstr, ml_delete};
use crate::message::emsg;
use crate::message::state::{lines_left, msg_scroll, need_wait_return};
use crate::option::vars::P_WINDOW;
use crate::os::cshim::gettext;
use crate::state::mode::State;
use crate::state::{MODE_CMDLINE, MODE_INSERT, MODE_LANGMAP, MODE_NORMAL};
use crate::types::CmdIdx;
use crate::types::{ExArg, LineNr, NUL, OptInt, int64_t};
use crate::ui::state::{Columns, Rows};
use crate::ui::ui_cursor_shape;
use crate::undo::u_save;
use crate::winlayer::Buf;
use crate::winlayer::Win;
use crate::winlayer::graph::{firstwin, lastwin};
use core::ffi::{CStr, c_int};

/// The indent `:append`/`:insert`/`:change` gives the first line it reads,
/// taken from the line the command started on.  `-1` once it has been used.
static APPEND_INDENT: GlobalCell<c_int> = GlobalCell::new(0);

/// `:insert` and `:append`, also used by [`ex_change`].
pub fn ex_append(excmd: &mut ExArg) {
    let mut did_undo = false;
    let (cmdidx, forceit, line2) = (excmd.cmdidx, excmd.forceit, excmd.line2);
    let mut lnum = line2;
    let mut indent = 0;
    let mut empty = Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY);

    // The ! flag toggles autoindent.
    if forceit {
        toggle_autoindent();
    }

    // First autoindent comes from the line we start on.
    if cmdidx != CmdIdx::change && Buf::current().b_p_ai != 0 && lnum > 0 {
        APPEND_INDENT.set(get_indent_lnum(lnum));
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
            if APPEND_INDENT.get() >= 0 {
                indent = APPEND_INDENT.replace(-1);
            } else if lnum > 0 {
                indent = get_indent_lnum(lnum);
            }
        }

        let Some(theline) = next_append_line(excmd, indent) else {
            break;
        };
        lines_left.set(Rows.get() - 1);
        // The line source saying the input ended.
        let Some(mut theline) = theline else {
            break;
        };

        // Look for the "." after the automatic indent.
        let text = theline.as_slice();
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
        if text.len() == typed {
            theline.clear();
        }

        did_undo = true;
        theline.push(NUL as u8);
        let theline = CStr::from_bytes_until_nul(&theline).expect("just terminated");
        let _ = ml_append_cstr(lnum, theline);
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

/// The next line for `:append` to insert.
///
/// There are three sources, in upstream's order: the text after a trailing
/// bar, the lines that follow the command in the same script, and the
/// command's own line source ([`ExArg::source`]).  `None` means the second
/// source ran out, which upstream leaves the loop for *without* resetting
/// `lines_left`; `Some(None)` is the line source saying the input ended.
fn next_append_line(args: &mut ExArg, indent: c_int) -> Option<Option<Vec<u8>>> {
    let arg = args.line.arg;
    if args.line.byte_at(arg) == b'|' {
        // Get the text after the trailing bar, and end the argument there.
        let line = args.line.rest_of(arg + 1).to_vec();
        args.line.terminate_at(arg);
        return Some(Some(line));
    }

    let Some(source) = args.source.as_mut() else {
        // No line source: use the lines that follow.  This ends
        // when there is no more.
        let next = args.line.next?;
        let rest = args.line.rest_of(next);
        let (line, rest) = match rest.iter().position(|&byte| byte == NL as u8) {
            Some(end) => (rest[..end].to_vec(), Some(next + end + 1)),
            None => (rest.to_vec(), None),
        };
        args.line.next = rest;
        return Some(Some(line));
    };

    // Set State to avoid the cursor shape being set to MODE_INSERT state
    // when the line source returns.
    let save_state = State.replace(MODE_CMDLINE);
    let in_loop = args
        .cond_stack
        .is_some_and(|cond| cond.with(|cs| cs.loop_level > 0));
    let first = if in_loop { -1 } else { NUL };
    // Up to its first NUL, which is where upstream's `char *` ended it.
    let line = source.next_line(first, indent, true).map(|line| {
        let mut line = line.into_vec();
        line.truncate(line.iter().position(|&byte| byte == 0).unwrap_or(0));
        line
    });
    State.set(save_state);
    Some(line)
}

/// `:change` -- delete the range, then append in its place.
pub fn ex_change(excmd: &mut ExArg) {
    let (forceit, line1, line2) = (excmd.forceit, excmd.line1, excmd.line2);
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
        APPEND_INDENT.set(get_indent_lnum(line1));
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
    check_cursor_lnum(Win::current());
    deleted_lines_mark(line1, line2 - lnum);
    // ":append" on the line above the deleted lines.
    excmd.line2 = line1;
    ex_append(excmd);
}

/// `:z` -- print a window of lines around the range's last line.
pub fn ex_z(excmd: &mut ExArg) {
    let (forceit, addr_count, flags, lnum) =
        (excmd.forceit, excmd.addr_count, excmd.flags, excmd.line2);
    let mut bigness = default_bigness(c_int::from(forceit)).max(1);

    let text = excmd.line.arg();
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
        // `atol`, which saturates.
        bigness = crate::charset::skip::decimal(&text[at..]).0;
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
