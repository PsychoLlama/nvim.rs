//! Resolving a mark's name to a store, and moving the cursor to it.
//!
//! Three name spaces meet here. `'A`-`'Z` and `'0`-`'9` live in the one global
//! table and may name any buffer; `'a`-`'z` live in the buffer; and the rest
//! of the punctuation is a mix of stores that are never adjusted (`'[`, `']`,
//! `'<`, `'>`) and *motions* (`'(`, `')`, `'{`, `'}`) computed from the cursor
//! on every call and handed back through a shared scratch record.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::ascii::{ascii_isdigit, ascii_islower, ascii_isupper};
use crate::buffer::{bt_prompt, buflist_getfile};
use crate::cursor::check_cursor;
use crate::edit::{BeginlineOpts, beginline};
use crate::global_cell::GlobalCell;
use crate::main::listcmd_busy;
use crate::message::emsg;
use crate::pos::{MAXCOL, lt};
use crate::textobject::{findpar, findsent};
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int, c_uint};
use core::ptr;

use super::jumplist::*;
use super::store::{Fmark, GlobalMarks, UNSET_FMARK, UNSET_POS};
use super::*;
use crate::search::{BACKWARD, FORWARD};

/// Resolve a mark of any name.
///
/// `fmp` is an out-parameter: when it is given *and* the mark resolved, the
/// record is copied into it and its address answered, so the caller holds a
/// snapshot rather than a pointer into a store that the next edit will move.
///
/// # Safety
/// `buf` must be a live buffer and `win` a live window; `fmp` must be null or
/// point at a live, writable `fmark_T`.
pub unsafe fn mark_get(
    buf: *mut buf_T,
    win: *mut win_T,
    fmp: *mut fmark_T,
    flag: MarkGet,
    name: c_int,
) -> *mut fmark_T {
    // SAFETY: the caller promised a live buffer.
    let handle = unsafe { Buf::new(buf) }.handle;
    let mut fm: *mut fmark_T = ptr::null_mut();
    if ascii_isupper(name) || ascii_isdigit(name) {
        // SAFETY: `name` is a digit or an upper-case letter, which is what
        // `mark_get_global` needs; the editor's globals are live.
        let xfm = unsafe { mark_get_global(flag as c_uint != kMarkAllNoResolve as c_uint, name) };
        // SAFETY: `mark_get_global` answers a slot of the global table.
        let xfm = unsafe { Xfmark::new(xfm) };
        // A global mark that names another buffer is *not* this buffer's
        // mark: `kMarkBufLocal` answers the unset scratch record rather than
        // a position the caller would then apply to the wrong file.
        if flag as c_uint == kMarkBufLocal as c_uint && xfm.fmark().fnum() != handle {
            // SAFETY: `buf` is live and a null `fmp` asks for the scratch.
            return unsafe { pos_to_mark(buf, ptr::null_mut(), UNSET_POS) };
        }
        fm = xfm.fmark().raw();
    } else if name > 0 && name < NMARK_LOCAL_MAX {
        // SAFETY: the caller promised a live buffer and window.
        fm = unsafe { mark_get_local(buf, win, name) };
    }
    if fmp.is_null() || fm.is_null() {
        return fm;
    }
    // SAFETY: both records are live; the caller asked for the copy.
    unsafe { *fmp = (*fm).clone() };
    fmp
}

/// Get a global mark {A-Z0-9}.
///
/// `name` — the name of the mark.
/// `resolve` — Whether to try resolving the mark fnum (i.e., load the buffer stored in
///                 the mark fname and update the xfmark_T (expensive)).
///
/// Returns mark
///
/// # Safety
/// `name` must be a digit or an upper-case letter; anything else panics rather
/// than reading out of the global table. The editor's globals must be live.
pub unsafe fn mark_get_global(resolve: bool, name: c_int) -> *mut xfmark_T {
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
    let mark = GlobalMarks::at(idx);
    // A mark read out of the shada file names its buffer by file name until
    // something asks for it; loading that file is what `resolve` pays for.
    if resolve && mark.fmark().fnum() == 0 {
        // SAFETY: the slot is live and its name, if any, is a C string.
        unsafe { fname2fnum(mark.raw()) };
    }
    mark.raw()
}

/// Get a local mark (lowercase and symbols).
///
/// Some marks are not actually marks, but positions that are never adjusted or motions presented as
/// marks. Search first for marks and fallback to finding motion type marks. If it's known
/// ahead of time that the mark is actually a motion use the mark_get_motion() directly.
///
/// @note  Lowercase, last_cursor '"', last insert '^', last change '.' are not statically
/// allocated, everything else is.
/// `name` — the name of the mark.
/// `win` — window to retrieve marks that belong to it (motions and context mark).
/// `buf` — buf to retrieve marks that belong to it.
///
/// Returns mark, NULL if not found.
///
/// # Safety
/// `buf` must be a live buffer and `win` a live window.
pub unsafe fn mark_get_local(buf: *mut buf_T, win: *mut win_T, name: c_int) -> *mut fmark_T {
    // SAFETY: the caller promised a live buffer and window.
    let (bufh, winh) = unsafe { (Buf::new(buf), Win::new(win)) };
    let mark: *mut fmark_T = if ascii_islower(name) {
        bufh.named_mark(name - 'a' as c_int).raw()
    } else if name == '[' as c_int {
        // SAFETY: `buf` is live; a null `fmp` asks for the scratch record.
        unsafe { pos_to_mark(buf, ptr::null_mut(), bufh.b_op_start) }
    } else if name == ']' as c_int {
        // SAFETY: as above.
        unsafe { pos_to_mark(buf, ptr::null_mut(), bufh.b_op_end) }
    } else if name == '<' as c_int || name == '>' as c_int {
        // SAFETY: as above.
        unsafe { mark_get_visual(buf, name) }
    } else if name == '\'' as c_int || name == '`' as c_int {
        // The context mark is the WINDOW's, but it is reported against the
        // current buffer rather than against `buf` — upstream reads `curbuf`
        // here and `nvim_buf_get_mark` relies on it.
        // SAFETY: `curbuf` is live from startup to exit.
        unsafe { pos_to_mark(Buf::current().raw(), ptr::null_mut(), winh.w_pcmark) }
    } else if name == '"' as c_int {
        bufh.last_cursor().raw()
    } else if name == '^' as c_int {
        bufh.last_insert().raw()
    } else if name == '.' as c_int {
        bufh.last_change().raw()
    // SAFETY: `buf` is live, which is all `bt_prompt` reads.
    } else if name == ':' as c_int && unsafe { bt_prompt(buf) } {
        bufh.prompt_start().raw()
    } else {
        // SAFETY: the caller promised a live buffer and window.
        unsafe { mark_get_motion(buf, win, name) }
    };
    if !mark.is_null() {
        // SAFETY: every arm above answers a live record or null.
        unsafe { Fmark::new(mark) }.set_fnum(bufh.handle as c_int);
    }
    mark
}

/// Get marks that are actually motions but return them as marks
///
/// Gets the following motions as marks: '{', '}', '(', ')'
/// `name` — name of the mark
/// `win` — window to retrieve the cursor to calculate the mark.
/// `buf` — buf to wrap motion marks with it's buffer number (fm->fnum).
///
/// @return[static] Mark.
///
/// # Safety
/// `buf` must be a live buffer and `win` a live window.
pub unsafe fn mark_get_motion(buf: *mut buf_T, win: *mut win_T, name: c_int) -> *mut fmark_T {
    // SAFETY: the caller promised a live window; `curwin` is live too.
    let (winh, mut cur) = unsafe { (Win::new(win), Win::current()) };
    // The motion is computed by *moving the cursor* and reading where it
    // landed, so the cursor is put back before answering. `listcmd_busy`
    // suppresses the jumplist entry the move would otherwise push.
    let saved = cur.w_cursor;
    let was_busy = listcmd_busy.get();
    listcmd_busy.set(true);
    let mut mark: *mut fmark_T = ptr::null_mut();
    if name == '{' as c_int || name == '}' as c_int {
        let mut oa = oparg_T {
            motion_type: kMTCharWise,
            ..OPARG_EMPTY
        };
        let dir = if name == '}' as c_int {
            FORWARD
        } else {
            BACKWARD
        };
        // SAFETY: the editor's globals are live and `oa` lives on the stack.
        if unsafe { findpar(&raw mut oa.inclusive, dir as c_int, 1, NUL, false) } {
            // SAFETY: `buf` is live; a null `fmp` asks for the scratch.
            mark = unsafe { pos_to_mark(buf, ptr::null_mut(), winh.w_cursor) };
        }
    } else if name == '(' as c_int || name == ')' as c_int {
        let dir = if name == ')' as c_int {
            FORWARD
        } else {
            BACKWARD
        };
        // SAFETY: the editor's globals are live.
        if unsafe { findsent(dir as Direction, 1) } != 0 {
            // SAFETY: as above.
            mark = unsafe { pos_to_mark(buf, ptr::null_mut(), winh.w_cursor) };
        }
    }
    cur.w_cursor = saved;
    listcmd_busy.set(was_busy);
    mark
}

/// An `oparg_T` with every field zeroed, which is what the motion lookups need
/// (only `inclusive` is read back).
const OPARG_EMPTY: oparg_T = oparg_T {
    op_type: 0,
    regname: 0,
    motion_type: 0,
    motion_force: 0,
    use_reg_one: false,
    inclusive: false,
    end_adjusted: false,
    start: UNSET_POS,
    end: UNSET_POS,
    cursor_start: UNSET_POS,
    line_count: 0,
    empty: false,
    is_VIsual: false,
    start_vcol: 0,
    end_vcol: 0,
    prev_opcount: 0,
    prev_count0: 0,
    excl_tr_ws: false,
};

/// Get visual marks '<', '>'
///
/// This marks are different to normal marks:
/// 1. Never adjusted.
/// 2. Different behavior depending on editor state (visual mode).
/// 3. Not saved in shada.
/// 4. Re-ordered when defined in reverse.
///
/// `buf` — Buffer to get the mark from.
/// `name` — Mark name '<' or '>'.
///
/// @return[static]  Mark
///
/// # Safety
/// `buf` must be a live buffer. The answer is the shared scratch record, so it
/// is only valid until the next motion or visual lookup.
pub unsafe fn mark_get_visual(buf: *mut buf_T, name: c_int) -> *mut fmark_T {
    if name != '<' as c_int && name != '>' as c_int {
        return ptr::null_mut();
    }
    // SAFETY: the caller promised a live buffer.
    let bufh = unsafe { Buf::new(buf) };
    let (start, end) = (bufh.b_visual.vi_start, bufh.b_visual.vi_end);
    // `'<` is whichever end comes FIRST, not whichever was set first: a
    // Visual selection made backwards still reports its marks in order.
    let wants_start = (name == '<' as c_int) == lt(start, end) || end.lnum == 0;
    let at = if wants_start && start.lnum != 0 {
        start
    } else {
        end
    };
    // SAFETY: `buf` is live; a null `fmp` asks for the scratch record.
    let mark = unsafe { pos_to_mark(buf, ptr::null_mut(), at) };
    if bufh.b_visual.vi_mode == 'V' as c_int {
        // A linewise selection has no columns of its own; `'<` is the start
        // of the line and `'>` the end of it.
        // SAFETY: `pos_to_mark` answered a live record.
        let mark = unsafe { Fmark::new(mark) };
        let mut pos = mark.pos();
        pos.col = if name == '<' as c_int {
            0
        } else {
            MAXCOL as colnr_T
        };
        pos.coladd = 0;
        mark.set_pos(pos);
    }
    mark
}

/// Search for the next named mark in the current file from a start position.
///
/// `startpos` — where to start.
/// `dir` — direction for search.
///
/// Returns next mark or NULL if no mark is found.
///
/// # Safety
/// `startpos` must point at a live position, and `curbuf` must be live.
pub unsafe fn getnextmark(startpos: *mut pos_T, dir: c_int, begin_line: c_int) -> *mut fmark_T {
    // SAFETY: the caller promised a live position.
    let mut pos = unsafe { *startpos };
    // `]'` and `['` are line motions: they land on a mark on another LINE, so
    // the column the search starts from is pushed to the far end of the
    // current one.
    if begin_line != 0 {
        pos.col = if dir == BACKWARD as c_int {
            0
        } else {
            MAXCOL as colnr_T
        };
    }
    // SAFETY: `curbuf` is live from startup to exit.
    let buf = unsafe { Buf::current() };
    let mut result: Option<Fmark> = None;
    for mark in buf.named_marks() {
        // `> 0` rather than `!= 0`: a negative line number is what a mangled
        // shada record leaves behind, and it is not a mark to jump to.
        if mark.lnum() <= 0 {
            continue;
        }
        let nearer = match result {
            None => true,
            Some(best) if dir == FORWARD as c_int => lt(mark.pos(), best.pos()),
            Some(best) => lt(best.pos(), mark.pos()),
        };
        let beyond = if dir == FORWARD as c_int {
            lt(pos, mark.pos())
        } else {
            lt(mark.pos(), pos)
        };
        if nearer && beyond {
            result = Some(mark);
        }
    }
    result.map_or(ptr::null_mut(), Fmark::raw)
}

/// Move to the given file mark, changing the buffer and cursor position.
///
/// Validate the mark, switch to the buffer, and move the cursor.
/// `fm` — Mark, can be NULL will raise E78: Unknown mark
/// `flags` — MarkMove flags to configure the movement to the mark.
///
/// Returns markMovekRes flags representing the outcome
///
/// # Safety
/// `fm` must be null or point at a live `fmark_T`, and the editor's globals
/// must be live.
pub unsafe fn mark_move_to(mut fm: *mut fmark_T, flags: MarkMove) -> MarkMoveRes {
    /// The mark being jumped to, copied out before the buffer switch: loading
    /// another file can free the store `fm` points into (a jumplist entry, a
    /// global slot whose buffer is wiped), and the position is still needed
    /// afterwards.
    static IN_FLIGHT: GlobalCell<fmark_T> = GlobalCell::new(UNSET_FMARK);

    let mut errormsg: *const c_char = ptr::null();
    // SAFETY: the caller promised a live record or null, and the message is
    // an out-parameter on this stack.
    if !unsafe { mark_check(fm, &raw mut errormsg) } {
        if !errormsg.is_null() {
            // SAFETY: `mark_check` wrote a `'static` message.
            unsafe { emsg(errormsg) };
        }
        return kMarkMoveFailed;
    }

    // SAFETY: `mark_check` said the record is live, and `curbuf`/`curwin` are
    // live from startup to exit.
    let (mark, buf) = unsafe { (Fmark::new(fm), Buf::current()) };
    let mut res: MarkMoveRes = kMarkMoveSuccess;
    if mark.fnum() != buf.handle {
        IN_FLIGHT.set(mark.read());
        fm = IN_FLIGHT.ptr();
        // SAFETY: `IN_FLIGHT` is a live record for the whole run.
        res |= unsafe { switch_to_mark_buf(fm, flags as c_uint & kMarkJumpList as c_uint == 0) };
        if res & kMarkMoveFailed != 0 {
            return res;
        }
        // The mark's line was checked against the OLD buffer above; now that
        // the file is loaded, ask again against the real one.
        // SAFETY: `curbuf` is live and the record is the static.
        if !unsafe { mark_check_line_bounds(Buf::current().raw(), fm, &raw mut errormsg) } {
            if !errormsg.is_null() {
                // SAFETY: as above.
                unsafe { emsg(errormsg) };
            }
            return res | kMarkMoveFailed;
        }
    } else if flags as c_uint & kMarkContext as c_uint != 0 {
        // SAFETY: the editor's globals are live.
        unsafe { setpcmark() };
    }

    // SAFETY: `curwin` is live from startup to exit.
    let mut win = unsafe { Win::current() };
    // SAFETY: `fm` is either the caller's live record or the static.
    let pos = unsafe { Fmark::new(fm) }.pos();
    let prev_pos = win.w_cursor;
    win.w_cursor = pos;
    if flags as c_uint & kMarkBeginLine as c_uint != 0 {
        // SAFETY: the editor's globals are live.
        unsafe { beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX) };
    }
    if prev_pos.lnum != pos.lnum {
        res |= kMarkChangedLine | kMarkChangedCursor;
    }
    if prev_pos.col != pos.col {
        res |= kMarkChangedCol | kMarkChangedCursor;
    }
    if flags as c_uint & kMarkSetView as c_uint != 0 {
        // SAFETY: as above.
        unsafe { mark_view_restore(fm) };
    }
    if res & (kMarkSwitchedBuf | kMarkChangedCursor) != 0 {
        // SAFETY: as above.
        unsafe { check_cursor(win.raw()) };
    }
    res
}

/// Attempt to switch to the buffer of the given global mark
///
/// `fm`
/// `pcmark_on_switch` — leave a context mark when switching buffer.
/// Returns whether the buffer was switched or not.
///
/// # Safety
/// `fm` must point at a live `fmark_T`, and the editor's globals must be live.
pub(super) unsafe fn switch_to_mark_buf(fm: *mut fmark_T, pcmark_on_switch: bool) -> MarkMoveRes {
    // SAFETY: the caller promised a live record.
    let fm = unsafe { Fmark::new(fm) };
    // SAFETY: `curbuf` is live from startup to exit.
    if fm.fnum() == unsafe { Buf::current() }.handle {
        return 0;
    }
    let getfile_flag = if pcmark_on_switch { GETF_SETMARK } else { 0 };
    // SAFETY: the editor's globals are live; `buflist_getfile` loads the file.
    let ok = unsafe { buflist_getfile(fm.fnum(), fm.lnum(), getfile_flag.cast_signed(), 0) } == OK;
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
/// `ex_delmarks`, and its inverse in `ex_marks` and `mark_global_iter`);
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
