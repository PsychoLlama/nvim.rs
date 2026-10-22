//! Resolving a mark's name to a store, and moving the cursor to it.
//!
//! Three name spaces meet here. `'A`-`'Z` and `'0`-`'9` live in the one global
//! table and may name any buffer; `'a`-`'z` live in the buffer; and the rest
//! of the punctuation is a mix of stores that are never adjusted (`'[`, `']`,
//! `'<`, `'>`) and *motions* (`'(`, `')`, `'{`, `'}`) computed from the cursor
//! on every call and handed back through a shared scratch record.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::ascii::{ascii_isdigit, ascii_isupper};
use crate::buffer::{buf_is_prompt, buflist_getfile};
use crate::cursor::check_cursor;
use crate::edit::{BeginlineOpts, beginline};
use crate::ex_docmd::state::listcmd_busy;
use crate::message::emsg;
use crate::pos::{MAXCOL, lt};
use crate::textobject::{findpar, findsent};
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int, c_uint};

use super::jumplist::*;
use super::store::{LocalMark, UNSET_POS, global_mark, slot};
use super::*;
use crate::search::{BACKWARD, FORWARD};

/// Resolve a mark of any name: a copy of the mark, or `None` for a name that
/// names no mark.
///
/// A mark that comes from a store is copied out, so the caller holds a
/// snapshot rather than a reference into a store the next edit moves -- and
/// can run user code (a jump loading a file) while it holds it. The marks
/// that are really *motions* or never-adjusted positions have no store of
/// their own and are computed; their `view` and `timestamp` are unset.
pub fn mark_get(buffer: Buf, win: Win, flag: MarkGet, name: c_int) -> Option<FileMark> {
    let handle = buffer.handle;
    if ascii_isupper(name) || ascii_isdigit(name) {
        let xfm = mark_get_global(flag as c_uint != kMarkAllNoResolve as c_uint, name);
        // A global mark that names another buffer is *not* this buffer's
        // mark: `kMarkBufLocal` answers an unset record rather than a
        // position the caller would then apply to the wrong file.
        if flag as c_uint == kMarkBufLocal as c_uint && xfm.fmark.fnum != handle {
            return Some(pos_to_mark(buffer, UNSET_POS));
        }
        return Some(xfm.fmark);
    }
    if name > 0 && name < NMARK_LOCAL_MAX {
        return mark_get_local(buffer, win, name);
    }
    None
}

/// Get a global mark {A-Z0-9}: a copy of the slot.
///
/// `name` — the name of the mark; a digit or an upper-case letter, and
///          anything else panics rather than reading out of the global
///          table.
/// `resolve` — Whether to try resolving the mark fnum (i.e., load the buffer stored in
///                 the mark fname and update the XFileMark (expensive)).
pub fn mark_get_global(resolve: bool, name: c_int) -> XFileMark {
    // Spelled out rather than handed to `mark_global_index` because that
    // takes a `c_char`, and narrowing `name` first would turn `'A' + 256`
    // into a valid index instead of the abort below. See p20-12's trap 9:
    // these two range tests are a NO-MUTATE zone, because perturbing either
    // turns an ordinary `'a` lookup into a panic.
    let idx = if ascii_isdigit(name) {
        name - '0' as c_int + NMARKS
    } else if ascii_isupper(name) {
        name - 'A' as c_int
    } else {
        // Deliberately a hard failure, not a `debug_assert!`: `idx` is the
        // index into `namedfm` on the next line, and neither branch above has
        // clamped it, so falling through reads out of bounds. Both callers
        // (`mark_get` and `nvim_get_mark`) reject anything that is not a
        // digit or an uppercase letter before they get here.
        unreachable!("mark name is neither a digit nor an uppercase letter");
    };
    let idx = slot(idx);
    let mark = global_mark(idx);
    // A mark read out of the shada file names its buffer by file name until
    // something asks for it; loading that file is what `resolve` pays for,
    // and it rewrites the slot -- so the answer is read again after.
    if resolve
        && mark.fmark.fnum == 0
        && let Some(fname) = &mark.fname
    {
        fname2fnum(fname.as_cstr());
        return global_mark(idx);
    }
    mark
}

/// Get a local mark (lowercase and symbols).
///
/// Some marks are not actually marks, but positions that are never adjusted or motions presented as
/// marks. Search first for marks and fallback to finding motion type marks. If it's known
/// ahead of time that the mark is actually a motion use the mark_get_motion() directly.
///
/// `name` — the name of the mark.
/// `win` — window to retrieve marks that belong to it (motions and context mark).
/// `buffer` — buf to retrieve marks that belong to it.
///
/// Returns a copy of the mark, `None` if not found. Whatever it is, it is
/// reported against `buffer`: a store's record has its `fnum` set to the
/// buffer's in place, as upstream writes it through the pointer it answers.
pub fn mark_get_local(buffer: Buf, win: Win, name: c_int) -> Option<FileMark> {
    let handle = buffer.handle;
    let store = LocalMark::named(name)
        .filter(|store| *store != LocalMark::PromptStart || buf_is_prompt(Some(buffer)));
    if let Some(store) = store {
        return Some(store.with(buffer, |mark| {
            mark.fnum = handle;
            mark.clone()
        }));
    }
    let mut mark = if name == '[' as c_int {
        pos_to_mark(buffer, buffer.b_op_start)
    } else if name == ']' as c_int {
        pos_to_mark(buffer, buffer.b_op_end)
    } else if name == '<' as c_int || name == '>' as c_int {
        mark_get_visual(buffer, name)?
    } else if name == '\'' as c_int || name == '`' as c_int {
        // The context mark is the WINDOW's; its number is overwritten with
        // `buffer`'s below, as upstream does after reading `curbuf`'s.
        pos_to_mark(Buf::current(), win.w_pcmark)
    } else {
        mark_get_motion(buffer, win, name)?
    };
    mark.fnum = handle;
    Some(mark)
}

/// Get marks that are actually motions but return them as marks
///
/// Gets the following motions as marks: '{', '}', '(', ')'
/// `name` — name of the mark
/// `win` — window to retrieve the cursor to calculate the mark.
/// `buffer` — buf to wrap motion marks with it's buffer number (fm->fnum).
///
/// Returns the mark, or `None` when `name` names no motion or the motion
/// found nothing.
pub fn mark_get_motion(buffer: Buf, win: Win, name: c_int) -> Option<FileMark> {
    let mut cur = Win::current();
    // The motion is computed by *moving the cursor* and reading where it
    // landed, so the cursor is put back before answering. `listcmd_busy`
    // suppresses the jumplist entry the move would otherwise push.
    let saved = cur.w_cursor;
    let was_busy = listcmd_busy.get();
    listcmd_busy.set(true);
    let mut mark = None;
    if name == '{' as c_int || name == '}' as c_int {
        let dir = if name == '}' as c_int {
            FORWARD
        } else {
            BACKWARD
        };
        if findpar(dir as c_int, 1, NUL, false).is_some() {
            mark = Some(pos_to_mark(buffer, win.w_cursor));
        }
    } else if name == '(' as c_int || name == ')' as c_int {
        let dir = if name == ')' as c_int {
            FORWARD
        } else {
            BACKWARD
        };
        if findsent(dir as Direction, 1).is_ok() {
            mark = Some(pos_to_mark(buffer, win.w_cursor));
        }
    }
    cur.w_cursor = saved;
    listcmd_busy.set(was_busy);
    mark
}

/// Get visual marks '<', '>'
///
/// This marks are different to normal marks:
/// 1. Never adjusted.
/// 2. Different behavior depending on editor state (visual mode).
/// 3. Not saved in shada.
/// 4. Re-ordered when defined in reverse.
///
/// `buffer` — Buffer to get the mark from.
/// `name` — Mark name '<' or '>'.
///
/// Returns the mark, or `None` when `name` is neither `'<` nor `'>`.
pub fn mark_get_visual(buffer: Buf, name: c_int) -> Option<FileMark> {
    if name != '<' as c_int && name != '>' as c_int {
        return None;
    }
    let (start, end) = (buffer.b_visual.vi_start, buffer.b_visual.vi_end);
    // `'<` is whichever end comes FIRST, not whichever was set first: a
    // Visual selection made backwards still reports its marks in order.
    let wants_start = (name == '<' as c_int) == lt(start, end) || end.lnum == 0;
    let at = if wants_start && start.lnum != 0 {
        start
    } else {
        end
    };
    let mut mark = pos_to_mark(buffer, at);
    if buffer.b_visual.vi_mode == 'V' as c_int {
        // A linewise selection has no columns of its own; `'<` is the start
        // of the line and `'>` the end of it.
        mark.mark.col = if name == '<' as c_int {
            0
        } else {
            MAXCOL as ColNr
        };
        mark.mark.coladd = 0;
    }
    Some(mark)
}

/// Search for the next named mark in the current file from a start position.
///
/// `startpos` — where to start.
/// `dir` — direction for search.
///
/// Returns a copy of the next mark, `None` if no mark is found.
pub fn getnextmark(startpos: Pos, dir: c_int, begin_line: bool) -> Option<FileMark> {
    let mut pos = startpos;
    // `]'` and `['` are line motions: they land on a mark on another LINE, so
    // the column the search starts from is pushed to the far end of the
    // current one.
    if begin_line {
        pos.col = if dir == BACKWARD as c_int {
            0
        } else {
            MAXCOL as ColNr
        };
    }
    let buf = Buf::current();
    let mut result: Option<&FileMark> = None;
    for mark in &buf.b_namedm {
        // `> 0` rather than `!= 0`: a negative line number is what a mangled
        // shada record leaves behind, and it is not a mark to jump to.
        if mark.mark.lnum <= 0 {
            continue;
        }
        let nearer = match result {
            None => true,
            Some(best) if dir == FORWARD as c_int => lt(mark.mark, best.mark),
            Some(best) => lt(best.mark, mark.mark),
        };
        let beyond = if dir == FORWARD as c_int {
            lt(pos, mark.mark)
        } else {
            lt(mark.mark, pos)
        };
        if nearer && beyond {
            result = Some(mark);
        }
    }
    result.cloned()
}

/// Move to the given file mark, changing the buffer and cursor position.
///
/// Validate the mark, switch to the buffer, and move the cursor.
/// `fm` — Mark, can be NULL will raise E78: Unknown mark
/// `flags` — MarkMove flags to configure the movement to the mark.
///
/// The mark is the caller's own copy, so loading another file -- which can
/// free the store it came from, and runs autocommands that may jump to
/// marks of their own -- leaves it as it was.
///
/// Returns markMovekRes flags representing the outcome
pub fn mark_move_to(fm: Option<&FileMark>, flags: MarkMove) -> MarkMoveRes {
    let mut errormsg = None;
    if !mark_check(fm, &mut errormsg) {
        if let Some(msg) = &errormsg {
            emsg(msg);
        }
        return kMarkMoveFailed;
    }
    let Some(fm) = fm else {
        return kMarkMoveFailed;
    };

    let mut res: MarkMoveRes = kMarkMoveSuccess;
    if fm.fnum != Buf::current().handle {
        res |= switch_to_mark_buf(fm, flags as c_uint & kMarkJumpList as c_uint == 0);
        if res & kMarkMoveFailed != 0 {
            return res;
        }
        // The mark's line was checked against the OLD buffer above; now that
        // the file is loaded, ask again against the real one.
        if !mark_check_line_bounds(Buf::current_or_none(), fm, &mut errormsg) {
            if let Some(msg) = &errormsg {
                emsg(msg);
            }
            return res | kMarkMoveFailed;
        }
    } else if flags as c_uint & kMarkContext as c_uint != 0 {
        setpcmark();
    }

    let mut win = Win::current();
    let pos = fm.mark;
    let prev_pos = win.w_cursor;
    win.w_cursor = pos;
    if flags as c_uint & kMarkBeginLine as c_uint != 0 {
        beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
    }
    if prev_pos.lnum != pos.lnum {
        res |= kMarkChangedLine | kMarkChangedCursor;
    }
    if prev_pos.col != pos.col {
        res |= kMarkChangedCol | kMarkChangedCursor;
    }
    if flags as c_uint & kMarkSetView as c_uint != 0 {
        mark_view_restore(Some(fm));
    }
    if res & (kMarkSwitchedBuf | kMarkChangedCursor) != 0 {
        check_cursor(win);
    }
    res
}

/// Attempt to switch to the buffer of the given global mark
///
/// `fm`
/// `pcmark_on_switch` — leave a context mark when switching buffer.
/// Returns whether the buffer was switched or not.
fn switch_to_mark_buf(fm: &FileMark, pcmark_on_switch: bool) -> MarkMoveRes {
    if fm.fnum == Buf::current().handle {
        return 0;
    }
    let getfile_flag = if pcmark_on_switch { GETF_SETMARK } else { 0 };
    // `buflist_getfile` loads the file.
    let ok = buflist_getfile(fm.fnum, fm.mark.lnum, getfile_flag.cast_signed(), 0).is_ok();
    if ok {
        kMarkSwitchedBuf
    } else {
        kMarkMoveFailed
    }
}

/// Where in the global table the mark called `name` lives: `'A`-`'Z` at
/// `0..NMARKS` and `'0`-`'9` above them, `-1` for any other name.
///
/// The same arithmetic is written out at four other sites (`setmark_pos`,
/// `ex_delmarks`, and its inverse in `ex_marks` and `shada_global_marks`);
/// `1787242636-jmarkmutate.py`'s `mark-digit-slot` is the anchor on all of
/// them at once. Consolidating them means replacing the fixed array with a
/// keyed map, which is a later phase's change.
#[inline]
pub(super) fn mark_global_index(name: c_char) -> c_int {
    let name = c_int::from(name);
    if ascii_isupper(name) {
        name - 'A' as c_int
    } else if ascii_isdigit(name) {
        NMARKS + (name - '0' as c_int)
    } else {
        -1
    }
}
