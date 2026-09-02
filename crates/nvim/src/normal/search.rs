//! Pattern searches driven from normal mode, and the marks and jumps that
//! share their "remember where we were" bookkeeping.
//!
//! What a search and a mark jump have in common is [`nv_mark_move_to`] and
//! the fold-opening tail: whether 'foldopen' lets the destination's fold
//! spring open depends on the command having been *typed*, not replayed.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::keycodes::ModMask;
use crate::winlayer::{Buf, Win};
use core::ptr;

use crate::cursor::check_cursor;
use crate::drawscreen::{UPD_SOME_VALID, redraw_later};
use crate::ex_getln::getcmdline;
use crate::fold::fold_open_cursor;
use crate::highlight::win_hl_attr;
use crate::highlight_group::{HLF_L, HLF_LC};
use crate::main::{KeyTyped, curbuf, curwin, fdo_flags, jop_flags, mod_mask, no_hlsearch, p_hls};
use crate::mark::{get_changelist, get_jumplist, mark_get, mark_move_to, setmark};
use crate::message::emsg;
use crate::normal::{
    CmdArg, KMarkNoContext, TAB, check_clear_op, check_clear_op_quit, clear_op, clear_op_beep,
    e_changelist_is_empty, kMTCharWise, kMTLineWise, kMarkAll, kMarkBeginLine, kMarkChangedCursor,
    kMarkChangedLine, kMarkContext, kMarkJumpList, kMarkMoveFailed, kMarkMoveSuccess, kMarkSetView,
    kMarkSwitchedBuf, nv_operator,
};
use crate::options::{kOptFdoFlagMark, kOptFdoFlagSearch, kOptJopFlagView};
use crate::os::cshim::gettext;
use crate::pos::equalpos;
use crate::search::{SEARCH_ECHO, SEARCH_MARK, SEARCH_MSG, SEARCH_OPT, do_search};
use crate::state::virtual_active;
use crate::types::{MarkMove, MarkMoveRes, OpType, cmdarg_T, fmark_T, searchit_arg_T, size_t};
use crate::window::goto_tabpage_lastused;
use core::ffi::{c_char, c_int, c_uint};

/// Whether the highlight of the previous match has to be redrawn.
///
/// Only when 'hlsearch' is on, `:nohlsearch` has not turned it off for now,
/// and the "current match" highlight actually differs from the others --
/// otherwise nothing on screen would change.
fn current_match_is_distinct() -> bool {
    // SAFETY (throughout): `curwin` is the current window.
    p_hls.get() != 0
        && !no_hlsearch.get()
        && unsafe { win_hl_attr(curwin.get(), HLF_LC) }
            != unsafe { win_hl_attr(curwin.get(), HLF_L) }
}

/// `/` and `?`: read a pattern from the command line and search for it.
pub(crate) unsafe fn nv_search(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    let mut op = ca.op();
    let save_cursor = cur_win().w_cursor;
    // `g?` is rot13; `?` after it is the operator, not a search.
    if ca.cmdchar == '?' as c_int && op.op_type == OpType::Rot13 {
        ca.cmdchar = 'g' as c_int;
        ca.nchar = '?' as c_int;
        unsafe { nv_operator(cap) };
        return;
    }
    ca.searchbuf = unsafe { getcmdline(ca.cmdchar, ca.count1, 0, true) };
    if ca.searchbuf.is_null() {
        clear_op(op);
        return;
    }
    // Reading the pattern may itself have moved the cursor ('incsearch'),
    // in which case the previous position is already on the jump list.
    let moved_while_typing = ca.arg != 0 || !equalpos(save_cursor, cur_win().w_cursor);
    let mark = if moved_while_typing {
        0
    } else {
        SEARCH_MARK as c_int
    };
    let (pat, none) = (ca.searchbuf, ptr::null_mut());
    // SAFETY: `pat` is the NUL-terminated pattern just read.
    let len = unsafe { cstr::bytes_at(pat) }.len();
    unsafe { normal_search(cap, ca.cmdchar, pat, len, mark, none) };
}

/// `n` and `N`: search again for the last pattern.
pub(crate) unsafe fn nv_next(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    let old = cur_win().w_cursor;
    let mut wrapped: c_int = 0;
    let (none, opt) = (ptr::null_mut(), SEARCH_MARK as c_int | ca.arg);
    let i = unsafe { normal_search(cap, 0, none, 0, opt, &raw mut wrapped) };
    // A match that lands where the cursor already is, without having
    // wrapped, is the one we are standing on: search once more so `n`
    // always moves.
    if i == 1 && wrapped == 0 && equalpos(old, cur_win().w_cursor) {
        ca.count1 += 1;
        let again = SEARCH_MARK as c_int | ca.arg;
        unsafe { normal_search(cap, 0, none, 0, again, ptr::null_mut()) };
        ca.count1 -= 1;
    }
    if i > 0 && current_match_is_distinct() {
        unsafe { redraw_later(curwin.get(), UPD_SOME_VALID) };
    }
}

/// The search every normal-mode search command goes through.
///
/// Answers `do_search`'s result: 0 for no match, 1 for a match, 2 for a match
/// the offset made linewise.
pub(crate) unsafe fn normal_search(
    cap: *mut cmdarg_T,
    dir: c_int,
    pat: *mut c_char,
    patlen: size_t,
    opt: c_int,
    wrapped: *mut c_int,
) -> c_int {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    // SAFETY: `cap` is the caller's live command argument, `pat` is null or a
    // pattern `patlen` bytes long, and `wrapped` is null or an out-parameter.
    let mut sia: searchit_arg_T = unsafe { core::mem::zeroed() };
    let prev_cursor = cur_win().w_cursor;
    let mut op = ca.op();
    op.motion_type = kMTCharWise;
    op.inclusive = false;
    // A search is one of the motions that fills the "1 last change"
    // register rather than the small-delete one.
    op.use_reg_one = true;
    cur_win().w_set_curswant = true;

    let flags = opt | SEARCH_OPT as c_int | SEARCH_ECHO as c_int | SEARCH_MSG as c_int;
    let (oap, n, arg) = (op.raw(), ca.count1, &raw mut sia);
    let i = unsafe { do_search(oap, dir, dir, pat, patlen, n, flags, arg) };
    if !wrapped.is_null() {
        unsafe { *wrapped = sia.sa_wrapped };
    }

    if i == 0 {
        clear_op(op);
    } else {
        // A `/pat/+1`-style offset makes the motion linewise.
        if i == 2 {
            op.motion_type = kMTLineWise;
        }
        cur_win().w_cursor.coladd = 0;
        if op.op_type == OpType::Nop
            && fdo_flags.get() & kOptFdoFlagSearch as c_int as c_uint != 0
            && KeyTyped.get()
        {
            unsafe { fold_open_cursor() };
        }
    }
    if !equalpos(cur_win().w_cursor, prev_cursor) && current_match_is_distinct() {
        unsafe { redraw_later(curwin.get(), UPD_SOME_VALID) };
    }
    check_cursor(unsafe { Win::current() });
    i
}

/// `m`: set a mark.
pub(crate) unsafe fn nv_mark(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if check_clear_op(ca.op()) {
        return;
    }
    if unsafe { setmark(ca.nchar) }.is_err() {
        clear_op_beep(ca.op());
    }
}

/// Jump to a mark, and describe the jump to the operator that may be pending.
pub(crate) unsafe fn nv_mark_move_to(
    cap: *mut cmdarg_T,
    flags: MarkMove,
    fm: *mut fmark_T,
) -> MarkMoveRes {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    // SAFETY: `cap` is the caller's live command argument and `fm` is null or
    // a mark `mark_move_to` may read.
    let res = unsafe { mark_move_to(fm, flags) };
    if res & kMarkMoveFailed as MarkMoveRes != 0 {
        clear_op(ca.op());
    }
    // `'a` is linewise, `` `a `` is charwise -- and only the charwise form
    // fills the "1 last change" register.
    ca.op().motion_type = if flags & kMarkBeginLine as MarkMove != 0 {
        kMTLineWise
    } else {
        kMTCharWise
    };
    if ca.cmdchar == '`' as c_int {
        ca.op().use_reg_one = true;
    }
    ca.op().inclusive = false;
    cur_win().w_set_curswant = true;
    res
}

/// The 'jumpoptions' half of a mark jump's flags: whether the view is
/// restored along with the position.
fn view_flag() -> MarkMove {
    if jop_flags.get() & kOptJopFlagView as c_int as c_uint != 0 {
        kMarkSetView as MarkMove
    } else {
        0
    }
}

/// Whether the destination's fold should be opened.
///
/// `old_KeyTyped` rather than the current value: the jump itself may have
/// consumed the "typed" flag.
unsafe fn may_open_fold(cap: *mut cmdarg_T, moved: bool, old_key_typed: bool) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    if ca.op().op_type == OpType::Nop
        && moved
        && fdo_flags.get() & kOptFdoFlagMark as c_int as c_uint != 0
        && old_key_typed
    {
        unsafe { fold_open_cursor() };
    }
}

/// `'` and `` ` ``, and their `g` forms.
pub(crate) unsafe fn nv_gomark(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    // A mark used as an operator's motion must not restore the view.
    let mut flags = if ca.op().op_type != OpType::Nop {
        0
    } else {
        view_flag()
    };
    let old_key_typed = KeyTyped.get();

    // `g'` and ``g` `` jump without touching the previous-context mark.
    let name = if ca.cmdchar == 'g' as c_int {
        flags |= KMarkNoContext as MarkMove;
        ca.extra_char
    } else {
        flags |= kMarkContext as MarkMove;
        ca.nchar
    };
    if ca.arg != 0 {
        flags |= kMarkBeginLine as MarkMove;
    }
    // An explicit count means "restore the view too".
    if ca.count0 != 0 {
        flags |= kMarkSetView as MarkMove;
    }

    // The record the lookup answers into; it outlives the jump below.
    let mut slot = fmark_T::UNSET;
    let fm = unsafe { mark_get(curbuf.get(), curwin.get(), &raw mut slot, kMarkAll, name) };
    let move_res = unsafe { nv_mark_move_to(cap, flags, fm) };
    if !virtual_active(cur_win()) {
        cur_win().w_cursor.coladd = 0;
    }
    let moved = move_res & kMarkMoveSuccess as MarkMoveRes != 0
        && (move_res & kMarkSwitchedBuf as MarkMoveRes != 0
            || move_res & kMarkChangedCursor as MarkMoveRes != 0);
    unsafe { may_open_fold(cap, moved, old_key_typed) };
}

/// `CTRL-O`, `CTRL-I` and `g;`/`g,`: step along the jump list or the change
/// list.
pub(crate) unsafe fn nv_pcmark(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    let mut flags = view_flag();
    let mut move_res: MarkMoveRes = 0;
    let old_key_typed = KeyTyped.get();
    if check_clear_op_quit(ca.op()) {
        return;
    }
    // CTRL-TAB is the last-used tab page, not a jump.
    if ca.cmdchar == TAB && mod_mask.get() == ModMask::CTRL {
        if !goto_tabpage_lastused() {
            clear_op_beep(ca.op());
        }
        return;
    }

    let fm = if ca.cmdchar == 'g' as c_int {
        unsafe { get_changelist(curbuf.get(), curwin.get(), ca.count1) }
    } else {
        flags |= (KMarkNoContext as c_int | kMarkJumpList as c_int) as MarkMove;
        unsafe { get_jumplist(curwin.get(), ca.count1) }
    };

    if !fm.is_null() {
        move_res = unsafe { nv_mark_move_to(cap, flags, fm) };
    } else if ca.cmdchar == 'g' as c_int {
        // Three different reasons the change list had nothing.
        if cur_buf().b_changelistlen == 0 {
            emsg(gettext(e_changelist_is_empty));
        } else if ca.count1 < 0 {
            emsg(gettext(c"E662: At start of changelist"));
        } else {
            emsg(gettext(c"E663: At end of changelist"));
        }
    } else {
        clear_op_beep(ca.op());
    }

    let moved = move_res & kMarkSwitchedBuf as MarkMoveRes != 0
        || move_res & kMarkChangedLine as MarkMoveRes != 0;
    unsafe { may_open_fold(cap, moved, old_key_typed) };
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
