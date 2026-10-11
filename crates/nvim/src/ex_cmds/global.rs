//! `:global` and `:vglobal` -- run a command on every line that matches.
//!
//! The two-pass shape is the whole point: [`ex_global`] marks every matching
//! line first, then [`global_exe`] runs the command on the marks, so that the
//! command may delete, add and move lines without the scan losing its place.
//! Each execution re-enters `do_cmdline`, which is why an error, an interrupt
//! or a `:global` nested inside another has to be handled here rather than by
//! the caller.
//!
//! Original: `src/nvim/ex_cmds.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

use super::say;
use super::{check_regexp_delim, do_sub_msg, global_need_beginline, global_need_msg_kind};
use crate::cursor::check_cursor;
use crate::edit::{BeginlineOpts, beginline};
use crate::ex_cmds::{sub_nlines, sub_nsubs};
use crate::ex_docmd::state::global_busy;
use crate::ex_docmd::{DoCmdOpts, do_cmdline};
use crate::getchar::state::got_int;
use crate::mark::setpcmark;
use crate::memline::{ml_clearmarked, ml_firstmarked, ml_setmarked};
use crate::message::state::{msg_col, msg_didout, msg_scrolled};
use crate::message::{e_backslash, e_interr, e_invcmd};
use crate::message::{emsg, msg};
use crate::message_fmt::msg_bytes;
use crate::r#move::changed_line_abv_curs;
use crate::option::magic_isset;
use crate::os::cshim::gettext;
use crate::os::input::{line_breakcheck, os_breakcheck};
use crate::regexp::{OwnedMMatch, RE_BOTH, RE_LAST, RE_SEARCH, RE_SUBST, skip_regexp_in_place};
use crate::search::{SEARCH_HIS, search_regcomp_owned};
use crate::smsg;
use crate::types::{ColNr, ExArg, LineNr};
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int};

/// Run `cmd` on line `lnum`, with the cursor at its start.
///
/// An empty command (or one that is only a line break) means `:print`, which
/// is what a bare `:g/pat/` does. This re-enters `do_cmdline`, so every
/// global may change.
fn global_exe_one(cmd: &[u8], lnum: LineNr) {
    Win::current().w_cursor.lnum = lnum;
    Win::current().w_cursor.col = 0 as ColNr;
    let cmd = match cmd.first() {
        None | Some(b'\n') => b"p".as_slice(),
        Some(_) => cmd,
    };
    let _ = do_cmdline(Some(cmd), None, DoCmdOpts::NOWAIT);
}

/// Does the command letter `kind` select a line that (did not) match?
///
/// `g` takes the matching lines, `v` the rest.  Upstream tests the letter
/// itself and selects nothing for any other one; only those two can reach
/// here, since `:global!` is rewritten to `v` before the test.
fn selects(kind: u8, matched: bool) -> bool {
    (kind == b'g' && matched) || (kind == b'v' && !matched)
}

/// Does the pattern match line `lnum`, searching from its first column?
fn matches_line(regmatch: &mut OwnedMMatch, lnum: LineNr) -> bool {
    regmatch.exec(Some(Win::current()), Buf::current(), lnum, 0 as ColNr) != 0
}

/// The head of a `:global` argument, parsed.
struct GlobalPat {
    /// The pattern text, copied out of the line.  Empty for the `\/`, `\?`
    /// and `\&` forms, which take the remembered pattern that `which_pat`
    /// names instead.
    pat: Vec<u8>,
    /// `RE_LAST`, `RE_SEARCH` or `RE_SUBST` -- which previous pattern to fall
    /// back on when `pat` is empty.
    which_pat: c_int,
    /// Where the Ex command to run on each selected line starts, in the
    /// command line.
    cmd_at: usize,
}

/// Split `:g/pat/cmd` into its pattern and its command, reporting the three
/// ways it can be malformed.
///
/// The closing delimiter is replaced by a NUL in place, as upstream does,
/// so the command that follows is a string of its own in `args`'s line.
fn global_pattern(args: &mut ExArg) -> Option<GlobalPat> {
    let arg = args.line.arg;
    let first = args.line.byte_at(arg);

    // Undocumented vi feature: "\/" and "\?" use the previous search pattern,
    // "\&" the previous substitute pattern.
    if first == b'\\' {
        let which_pat = match args.line.byte_at(arg + 1) {
            b'&' => RE_SUBST as c_int,
            b'/' | b'?' => RE_SEARCH as c_int,
            _ => {
                emsg(gettext(e_backslash));
                return None;
            }
        };
        return Some(GlobalPat {
            pat: Vec::new(),
            which_pat,
            cmd_at: arg + 2,
        });
    }

    if first == 0 {
        emsg(gettext(c"E148: Regular expression missing from global"));
        return None;
    }
    let delim = first;
    // The delimiter is handed on as a `char`, so a high byte arrives
    // sign-extended -- which is what indexes the ctype table upstream.
    if check_regexp_delim(c_int::from(delim as c_char)).is_err() {
        return None;
    }

    let pat_at = arg + 1;
    // A `?` delimiter's `\?` is unescaped in the line itself, as upstream's
    // `&eap->arg` slot does.
    let mut cmd_at = skip_regexp_in_place(
        args.line.buffer_mut(),
        pat_at,
        c_int::from(delim),
        c_int::from(magic_isset()),
    );
    if args.line.byte_at(cmd_at) == delim {
        // End delimiter found: replace it with a NUL.
        args.line.terminate_at(cmd_at);
        cmd_at += 1;
    }
    Some(GlobalPat {
        // Terminated by the argument's own NUL when there was no closing
        // delimiter, by the one just written when there was.
        pat: args.line.rest_of(pat_at).to_vec(),
        which_pat: RE_LAST as c_int,
        cmd_at,
    })
}

/// Pass 1: mark every line of the range that the pattern (does not) match,
/// and answer how many were marked.
fn global_mark(args: &ExArg, regmatch: &mut OwnedMMatch, kind: u8) -> c_int {
    let (mut lnum, line2) = (args.line1, args.line2);
    let mut ndone = 0 as c_int;
    while lnum <= line2 && !got_int.get() {
        let matched = matches_line(regmatch, lnum);
        // Re-compiling the program can have failed.
        if !regmatch.has_program() {
            break;
        }
        if selects(kind, matched) {
            ml_setmarked(lnum);
            ndone += 1;
        }
        line_breakcheck();
        lnum += 1;
    }
    ndone
}

/// Execute a global command of the form
///
/// * `g/pattern/X`: execute X on all lines where pattern matches
/// * `v/pattern/X`: execute X on all lines where pattern does not match
///
/// where `X` is an Ex command.  The command character (and the trailing
/// delimiter) may be left out, and is then `p`.
///
/// This runs in two passes: first scan the range for the pattern and set a
/// mark on each line that (does not) match, then execute the command for each
/// marked line.  The split is required because after deleting lines we would
/// not know where to search for the next match.
pub fn ex_global(excmd: &mut ExArg) {
    // When nesting, the command works on one line.  That allows for
    // ":g/found/v/notfound/command".
    if global_busy.get() != 0
        && (excmd.line1 != 1 || excmd.line2 != Buf::current().b_ml.ml_line_count)
    {
        // Will increment global_busy to break out of the loop.
        emsg(gettext(c"E147: Cannot do :global recursive with a range"));
        return;
    }

    // ":global!" is like ":vglobal".
    let kind = if excmd.forceit {
        b'v'
    } else {
        excmd.line.byte_at(excmd.line.cmd)
    };
    let Some(parsed) = global_pattern(excmd) else {
        return;
    };

    let compiled = search_regcomp_owned(
        &parsed.pat,
        RE_BOTH as c_int,
        parsed.which_pat,
        SEARCH_HIS as c_int,
    );
    let Some((mut regmatch, used_pat)) = compiled else {
        emsg(gettext(e_invcmd));
        return;
    };

    // The command is read in place: nothing the walk runs can reach this
    // command's own line, and `do_cmdline` copies what it is handed.
    if global_busy.get() != 0 {
        let lnum = Win::current().w_cursor.lnum;
        if selects(kind, matches_line(&mut regmatch, lnum)) {
            global_exe_one(excmd.line.rest_of(parsed.cmd_at), lnum);
        }
    } else {
        let ndone = global_mark(excmd, &mut regmatch, kind);
        // Pass 2: execute the command for each line that has been marked.
        if got_int.get() {
            msg(gettext(e_interr), 0 as c_int);
        } else if ndone == 0 as c_int {
            let pat = msg_bytes(&used_pat);
            if kind == b'v' {
                smsg!(0, "Pattern found in every line: {pat}");
            } else {
                smsg!(0, "Pattern not found: {pat}");
            }
        } else {
            global_exe(excmd.line.rest_of(parsed.cmd_at));
        }
        ml_clearmarked(); // clear rest of the marks
    }
}

/// Execute `cmd` on the lines marked with `ml_setmarked`.
///
/// Every iteration re-enters `do_cmdline`, so nothing may be cached across
/// the loop.
pub fn global_exe(cmd: &[u8]) {
    // Remember what buffer we started in.
    let old_buf = Buf::current_raw();

    // Set the current position only once for a global command.  If
    // global_busy is set, setpcmark() will not do anything.  If there is an
    // error, global_busy will be incremented.
    setpcmark();

    // When the command writes a message, don't overwrite the command.
    msg_didout.set(true);

    sub_nsubs.set(0 as c_int);
    sub_nlines.set(0 as LineNr);
    global_need_msg_kind.set(true);
    global_need_beginline.set(false);
    global_busy.set(1 as c_int);
    let old_lcount = Buf::current().b_ml.ml_line_count;

    while !got_int.get() {
        let lnum = ml_firstmarked();
        if lnum == 0 as LineNr || global_busy.get() != 1 as c_int {
            break;
        }
        global_exe_one(cmd, lnum);
        os_breakcheck();
    }

    global_busy.set(0 as c_int);
    if global_need_beginline.get() {
        beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
    } else {
        check_cursor(Win::current());
    }

    // The cursor may not have moved in the text but a change in a previous
    // line may move it on the screen.
    changed_line_abv_curs();

    // If it looks like no message was written, allow overwriting the command
    // with the report for number of changes.
    if msg_col.get() == 0 as c_int && msg_scrolled.get() == 0 as c_int {
        msg_didout.set(false);
    }

    // If substitutes were done, report the number of substitutes, otherwise
    // report the number of extra or deleted lines.  Don't report those in the
    // edge case where the buffer we are in after execution is different from
    // the one we started in.
    if !do_sub_msg(false) && Buf::current_raw() == old_buf {
        say::more(Buf::current().b_ml.ml_line_count as c_int - old_lcount as c_int);
    }
}
