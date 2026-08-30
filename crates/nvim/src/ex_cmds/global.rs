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

#![deny(unsafe_op_in_unsafe_fn)]

use super::{check_regexp_delim, do_sub_msg, global_need_beginline, global_need_msg_kind};
use crate::cstr;
use crate::cursor::check_cursor;
use crate::edit::{BeginlineOpts, beginline};
use crate::ex_docmd::{DoCmdOpts, do_cmdline};
use crate::main::{
    curbuf, curwin, e_backslash, e_interr, e_invcmd, global_busy, got_int, msg_col, msg_didout,
    msg_scrolled, sub_nlines, sub_nsubs,
};
use crate::mark::setpcmark;
use crate::memline::{ml_clearmarked, ml_firstmarked, ml_setmarked};
use crate::message::{emsg, msg, msgmore};
use crate::message_fmt::c_str;
use crate::r#move::changed_line_abv_curs;
use crate::option::magic_isset;
use crate::os::cshim::gettext;
use crate::os::input::{line_breakcheck, os_breakcheck};
use crate::regexp::{
    RE_BOTH, RE_LAST, RE_SEARCH, RE_SUBST, skip_regexp_ex, vim_regexec_multi, vim_regfree,
};
use crate::search::{SEARCH_HIS, search_regcomp};
use crate::smsg;
use crate::types::{NUL, colnr_T, exarg_T, linenr_T, regmmatch_T, size_t};
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// Run `cmd` on line `lnum`, with the cursor at its start.
///
/// An empty command (or one that is only a line break) means `:print`, which
/// is what a bare `:g/pat/` does.
///
/// # Safety
/// Main thread; `lnum` must be a line of the current buffer and `cmd` a live
/// C string.  This re-enters `do_cmdline`, so every global may change.
unsafe fn global_exe_one(cmd: *mut c_char, lnum: linenr_T) {
    // SAFETY: caller's contract -- the current window is live.
    cur_win().w_cursor.lnum = lnum;
    cur_win().w_cursor.col = 0 as colnr_T;
    // SAFETY: caller's contract -- `cmd` is NUL-terminated.
    let first = unsafe { *cmd } as c_int;
    let cmd = if first == NUL || first == '\n' as c_int {
        c"p".as_ptr() as *mut c_char
    } else {
        cmd
    };
    // SAFETY: a live command string; re-enters the Ex layer.
    let _ = unsafe { do_cmdline(cmd, None, ptr::null_mut(), DoCmdOpts::NOWAIT) };
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
///
/// # Safety
/// Main thread; `regmatch` must hold a compiled program.
unsafe fn matches_line(regmatch: *mut regmmatch_T, lnum: linenr_T) -> bool {
    // SAFETY: caller's contract; `curwin`/`curbuf` are the live pair.
    unsafe {
        vim_regexec_multi(
            regmatch,
            curwin.get(),
            curbuf.get(),
            lnum,
            0 as colnr_T,
            ptr::null_mut(),
            ptr::null_mut(),
        ) != 0
    }
}

/// The head of a `:global` argument, parsed.
struct GlobalPat {
    /// The pattern text.  Empty for the `\/`, `\?` and `\&` forms, which take
    /// the remembered pattern that `which_pat` names instead.
    pat: *mut c_char,
    patlen: size_t,
    /// `RE_LAST`, `RE_SEARCH` or `RE_SUBST` -- which previous pattern to fall
    /// back on when `pat` is empty.
    which_pat: c_int,
    /// The Ex command to run on each selected line.
    cmd: *mut c_char,
}

/// Split `:g/pat/cmd` into its pattern and its command, reporting the three
/// ways it can be malformed.
///
/// The closing delimiter is replaced by a NUL in place, so the pattern that
/// comes back borrows `eap`'s argument.
///
/// # Safety
/// `eap` must be the live Ex-command argument; its `arg` must be writable.
unsafe fn global_pattern(eap: *mut exarg_T) -> Option<GlobalPat> {
    // SAFETY: caller's contract.
    let arg = unsafe { (*eap).arg };
    // SAFETY: an Ex-command argument is NUL-terminated, and nothing below
    // rewrites it before the last read of this borrow.
    let bytes = unsafe { CStr::from_ptr(arg) }.to_bytes();

    // Undocumented vi feature: "\/" and "\?" use the previous search pattern,
    // "\&" the previous substitute pattern.
    if bytes.first() == Some(&b'\\') {
        let which_pat = match bytes.get(1) {
            Some(b'&') => RE_SUBST as c_int,
            Some(b'/' | b'?') => RE_SEARCH as c_int,
            _ => {
                emsg(gettext(e_backslash));
                return None;
            }
        };
        return Some(GlobalPat {
            pat: c"".as_ptr() as *mut c_char,
            patlen: 0 as size_t,
            which_pat,
            cmd: arg.wrapping_add(2),
        });
    }

    let Some(&delim) = bytes.first() else {
        emsg(gettext(c"E148: Regular expression missing from global"));
        return None;
    };
    // The delimiter is handed on as a `char`, so a high byte arrives
    // sign-extended -- which is what indexes the ctype table upstream.
    // SAFETY: message state.
    if unsafe { check_regexp_delim(delim as c_char as c_int) }.is_err() {
        return None;
    }

    let pat = arg.wrapping_add(1);
    // SAFETY: `pat` is the pattern's first byte, inside the NUL-terminated
    // argument.  `newp` lets the skip hand back a rewritten copy for the `?`
    // delimiter, which is why it is `eap->arg` that receives it.
    let mut cmd = unsafe {
        skip_regexp_ex(
            pat,
            delim as c_int,
            magic_isset() as c_int,
            &raw mut (*eap).arg,
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    // SAFETY: the skip stopped at the delimiter, at the NUL, or in between.
    if unsafe { *cmd } as u8 == delim {
        // End delimiter found: replace it with a NUL.
        unsafe { *cmd = NUL as c_char };
        cmd = unsafe { cmd.add(1) };
    }
    Some(GlobalPat {
        pat,
        // SAFETY: `pat` is NUL-terminated -- by the argument's own terminator
        // when there was no closing delimiter, by the one just written when
        // there was.
        patlen: unsafe { cstr::bytes_at(pat) }.len(),
        which_pat: RE_LAST as c_int,
        cmd,
    })
}

/// Pass 1: mark every line of the range that the pattern (does not) match,
/// and answer how many were marked.
///
/// # Safety
/// Main thread; `regmatch` must hold a compiled program and `eap` be the live
/// Ex-command argument.
unsafe fn global_mark(eap: *mut exarg_T, regmatch: *mut regmmatch_T, kind: u8) -> c_int {
    // SAFETY: caller's contract.  Nothing in the loop touches `eap`.
    let (mut lnum, line2) = unsafe { ((*eap).line1, (*eap).line2) };
    let mut ndone = 0 as c_int;
    while lnum <= line2 && !got_int.get() {
        // SAFETY: caller's contract.
        let matched = unsafe { matches_line(regmatch, lnum) };
        // SAFETY: as above -- re-compiling the program can have failed.
        if unsafe { (*regmatch).regprog.is_null() } {
            break;
        }
        if selects(kind, matched) {
            // SAFETY: `lnum` is inside the range, so inside the buffer.
            unsafe { ml_setmarked(lnum) };
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
///
/// # Safety
/// Main thread; `eap` must be the live Ex-command argument.
pub unsafe fn ex_global(eap: *mut exarg_T) {
    // When nesting, the command works on one line.  That allows for
    // ":g/found/v/notfound/command".
    if global_busy.get() != 0 {
        // SAFETY: caller's contract; `curbuf` is the live buffer.
        let whole = unsafe { (*eap).line1 == 1 && (*eap).line2 == cur_buf().b_ml.ml_line_count };
        if !whole {
            // Will increment global_busy to break out of the loop.
            emsg(gettext(c"E147: Cannot do :global recursive with a range"));
            return;
        }
    }

    // ":global!" is like ":vglobal".
    // SAFETY: caller's contract -- `eap->cmd` is the command word.
    let kind = unsafe {
        if (*eap).forceit != 0 {
            b'v'
        } else {
            *(*eap).cmd as u8
        }
    };
    // SAFETY: caller's contract.
    let Some(parsed) = (unsafe { global_pattern(eap) }) else {
        return;
    };

    let mut regmatch = regmmatch_T::default();
    let mut used_pat: *mut c_char = ptr::null_mut();
    // SAFETY: a pattern and its length, and out-parameters we own.
    let compiled = unsafe {
        search_regcomp(
            parsed.pat,
            parsed.patlen,
            &raw mut used_pat,
            RE_BOTH as c_int,
            parsed.which_pat,
            SEARCH_HIS as c_int,
            &raw mut regmatch,
        )
    };
    if compiled.is_err() {
        emsg(gettext(e_invcmd));
        return;
    }

    if global_busy.get() != 0 {
        // SAFETY: the program is compiled and the cursor line is in the
        // buffer.
        let lnum = cur_win().w_cursor.lnum;
        if selects(kind, unsafe { matches_line(&raw mut regmatch, lnum) }) {
            unsafe { global_exe_one(parsed.cmd, lnum) };
        }
    } else {
        // SAFETY: as above.
        let ndone = unsafe { global_mark(eap, &raw mut regmatch, kind) };
        // Pass 2: execute the command for each line that has been marked.
        if got_int.get() {
            msg(gettext(e_interr), 0 as c_int);
        } else if ndone == 0 as c_int {
            // SAFETY: `used_pat` is the pattern `search_regcomp` reported.
            let pat = unsafe { c_str(used_pat) };
            if kind == b'v' {
                smsg!(0, "Pattern found in every line: {pat}");
            } else {
                smsg!(0, "Pattern not found: {pat}");
            }
        } else {
            // SAFETY: `parsed.cmd` is a live C string.
            unsafe { global_exe(parsed.cmd) };
        }
        // SAFETY: main thread, live buffer.
        unsafe { ml_clearmarked() }; // clear rest of the marks
    }
    // SAFETY: the program `search_regcomp` produced, used for the last time.
    unsafe { vim_regfree(regmatch.regprog) };
}

/// Execute `cmd` on the lines marked with `ml_setmarked`.
///
/// # Safety
/// Main thread; `cmd` must be a live C string.  Every iteration re-enters
/// `do_cmdline`, so nothing may be cached across the loop.
pub unsafe fn global_exe(cmd: *mut c_char) {
    // Remember what buffer we started in.
    let old_buf = curbuf.get();

    // Set the current position only once for a global command.  If
    // global_busy is set, setpcmark() will not do anything.  If there is an
    // error, global_busy will be incremented.
    // SAFETY: main thread, live window.
    setpcmark();

    // When the command writes a message, don't overwrite the command.
    msg_didout.set(true);

    sub_nsubs.set(0 as c_int);
    sub_nlines.set(0 as linenr_T);
    global_need_msg_kind.set(true);
    global_need_beginline.set(false);
    global_busy.set(1 as c_int);
    // SAFETY: `curbuf` is the live buffer.
    let old_lcount = cur_buf().b_ml.ml_line_count;

    while !got_int.get() {
        // SAFETY: main thread, live buffer.
        let lnum = unsafe { ml_firstmarked() };
        if lnum == 0 as linenr_T || global_busy.get() != 1 as c_int {
            break;
        }
        // SAFETY: `lnum` is a marked line of the buffer; `cmd` is live.
        unsafe { global_exe_one(cmd, lnum) };
        os_breakcheck();
    }

    global_busy.set(0 as c_int);
    if global_need_beginline.get() {
        // SAFETY: main thread, live window.
        beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
    } else {
        // SAFETY: as above -- the cursor may be beyond the end of the line.
        check_cursor(unsafe { Win::current() });
    }

    // The cursor may not have moved in the text but a change in a previous
    // line may move it on the screen.
    // SAFETY: main thread, live window.
    unsafe { changed_line_abv_curs() };

    // If it looks like no message was written, allow overwriting the command
    // with the report for number of changes.
    if msg_col.get() == 0 as c_int && msg_scrolled.get() == 0 as c_int {
        msg_didout.set(false);
    }

    // If substitutes were done, report the number of substitutes, otherwise
    // report the number of extra or deleted lines.  Don't report those in the
    // edge case where the buffer we are in after execution is different from
    // the one we started in.
    // SAFETY: message state; `curbuf` is live.
    if !unsafe { do_sub_msg(false) } && curbuf.get() == old_buf {
        unsafe { msgmore(cur_buf().b_ml.ml_line_count as c_int - old_lcount as c_int) };
    }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
