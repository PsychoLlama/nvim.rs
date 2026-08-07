//! Pattern searches driven from normal mode, and the marks and jumps that
//! share their "remember where we were" bookkeeping.
//!
//! What a search and a mark jump have in common is [`nv_mark_move_to`] and
//! the fold-opening tail: whether 'foldopen' lets the destination's fold
//! spring open depends on the command having been *typed*, not replayed.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ptr;

use crate::src::nvim::cursor::check_cursor;
use crate::src::nvim::drawscreen::{UPD_SOME_VALID, redraw_later};
use crate::src::nvim::ex_getln::getcmdline;
use crate::src::nvim::fold::foldOpenCursor;
use crate::src::nvim::highlight::win_hl_attr;
use crate::src::nvim::highlight_group::{HLF_L, HLF_LC};
use crate::src::nvim::main::{
    KeyTyped, curbuf, curwin, fdo_flags, jop_flags, mod_mask, no_hlsearch, p_hls,
};
use crate::src::nvim::mark::{get_changelist, get_jumplist, mark_get, mark_move_to, setmark};
use crate::src::nvim::message::emsg;
use crate::src::nvim::normal::{
    KMarkNoContext, MOD_MASK_CTRL, TAB, checkclearop, checkclearopq, clearop, clearopbeep,
    e_changelist_is_empty, false_0, kMTCharWise, kMTLineWise, kMarkAll, kMarkBeginLine,
    kMarkChangedCursor, kMarkChangedLine, kMarkContext, kMarkJumpList, kMarkMoveFailed,
    kMarkMoveSuccess, kMarkSetView, kMarkSwitchedBuf, nv_operator, true_0,
};
use crate::src::nvim::options::{kOptFdoFlagMark, kOptFdoFlagSearch, kOptJopFlagView};
use crate::src::nvim::os::libc::{gettext, strlen};
use crate::src::nvim::pos::equalpos;
use crate::src::nvim::search::{SEARCH_ECHO, SEARCH_MARK, SEARCH_MSG, SEARCH_OPT, do_search};
use crate::src::nvim::state::virtual_active;
use crate::src::nvim::types::{
    MarkMove, MarkMoveRes, OP_NOP, OP_ROT13, cmdarg_T, fmark_T, searchit_arg_T, size_t,
};
use crate::src::nvim::window::goto_tabpage_lastused;
use core::ffi::{c_char, c_int, c_uint};

/// Whether the highlight of the previous match has to be redrawn.
///
/// Only when 'hlsearch' is on, `:nohlsearch` has not turned it off for now,
/// and the "current match" highlight actually differs from the others --
/// otherwise nothing on screen would change.
fn current_match_is_distinct() -> bool {
    // SAFETY: `curwin` is the current window.
    unsafe {
        p_hls.get() != 0
            && !no_hlsearch.get()
            && win_hl_attr(curwin.get(), HLF_LC) != win_hl_attr(curwin.get(), HLF_L)
    }
}

/// `/` and `?`: read a pattern from the command line and search for it.
pub(crate) unsafe fn nv_search(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let oap = (*cap).oap;
        let save_cursor = (*curwin.get()).w_cursor;
        // `g?` is rot13; `?` after it is the operator, not a search.
        if (*cap).cmdchar == '?' as c_int && (*oap).op_type == OP_ROT13 {
            (*cap).cmdchar = 'g' as c_int;
            (*cap).nchar = '?' as c_int;
            nv_operator(cap);
            return;
        }
        (*cap).searchbuf = getcmdline((*cap).cmdchar, (*cap).count1, 0, true);
        if (*cap).searchbuf.is_null() {
            clearop(oap);
            return;
        }
        // Reading the pattern may itself have moved the cursor ('incsearch'),
        // in which case the previous position is already on the jump list.
        let moved_while_typing =
            (*cap).arg != 0 || !equalpos(save_cursor, (*curwin.get()).w_cursor);
        normal_search(
            cap,
            (*cap).cmdchar,
            (*cap).searchbuf,
            strlen((*cap).searchbuf),
            if moved_while_typing {
                0
            } else {
                SEARCH_MARK as c_int
            },
            ptr::null_mut(),
        );
    }
}

/// `n` and `N`: search again for the last pattern.
pub(crate) unsafe fn nv_next(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let old = (*curwin.get()).w_cursor;
        let mut wrapped: c_int = false_0;
        let i = normal_search(
            cap,
            0,
            ptr::null_mut(),
            0,
            SEARCH_MARK as c_int | (*cap).arg,
            &raw mut wrapped,
        );
        // A match that lands where the cursor already is, without having
        // wrapped, is the one we are standing on: search once more so `n`
        // always moves.
        if i == 1 && wrapped == 0 && equalpos(old, (*curwin.get()).w_cursor) {
            (*cap).count1 += 1;
            normal_search(
                cap,
                0,
                ptr::null_mut(),
                0,
                SEARCH_MARK as c_int | (*cap).arg,
                ptr::null_mut(),
            );
            (*cap).count1 -= 1;
        }
        if i > 0 && current_match_is_distinct() {
            redraw_later(curwin.get(), UPD_SOME_VALID);
        }
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
    // SAFETY: `cap` is the caller's live command argument, `pat` is null or a
    // pattern `patlen` bytes long, and `wrapped` is null or an out-parameter.
    unsafe {
        let mut sia: searchit_arg_T = core::mem::zeroed();
        let prev_cursor = (*curwin.get()).w_cursor;
        let oap = (*cap).oap;
        (*oap).motion_type = kMTCharWise;
        (*oap).inclusive = false;
        // A search is one of the motions that fills the "1 last change"
        // register rather than the small-delete one.
        (*oap).use_reg_one = true;
        (*curwin.get()).w_set_curswant = true_0;

        let i = do_search(
            oap,
            dir,
            dir,
            pat,
            patlen,
            (*cap).count1,
            opt | SEARCH_OPT as c_int | SEARCH_ECHO as c_int | SEARCH_MSG as c_int,
            &raw mut sia,
        );
        if !wrapped.is_null() {
            *wrapped = sia.sa_wrapped;
        }

        if i == 0 {
            clearop(oap);
        } else {
            // A `/pat/+1`-style offset makes the motion linewise.
            if i == 2 {
                (*oap).motion_type = kMTLineWise;
            }
            (*curwin.get()).w_cursor.coladd = 0;
            if (*oap).op_type == OP_NOP
                && fdo_flags.get() & kOptFdoFlagSearch as c_int as c_uint != 0
                && KeyTyped.get()
            {
                foldOpenCursor();
            }
        }
        if !equalpos((*curwin.get()).w_cursor, prev_cursor) && current_match_is_distinct() {
            redraw_later(curwin.get(), UPD_SOME_VALID);
        }
        check_cursor(curwin.get());
        i
    }
}

/// `m`: set a mark.
pub(crate) unsafe fn nv_mark(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if checkclearop((*cap).oap) {
            return;
        }
        if setmark((*cap).nchar) == false_0 {
            clearopbeep((*cap).oap);
        }
    }
}

/// Jump to a mark, and describe the jump to the operator that may be pending.
pub(crate) unsafe fn nv_mark_move_to(
    cap: *mut cmdarg_T,
    flags: MarkMove,
    fm: *mut fmark_T,
) -> MarkMoveRes {
    // SAFETY: `cap` is the caller's live command argument and `fm` is null or
    // a mark `mark_move_to` may read.
    unsafe {
        let res = mark_move_to(fm, flags);
        if res & kMarkMoveFailed as MarkMoveRes != 0 {
            clearop((*cap).oap);
        }
        // `'a` is linewise, `` `a `` is charwise -- and only the charwise form
        // fills the "1 last change" register.
        (*(*cap).oap).motion_type = if flags & kMarkBeginLine as MarkMove != 0 {
            kMTLineWise
        } else {
            kMTCharWise
        };
        if (*cap).cmdchar == '`' as c_int {
            (*(*cap).oap).use_reg_one = true;
        }
        (*(*cap).oap).inclusive = false;
        (*curwin.get()).w_set_curswant = true_0;
        res
    }
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
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if (*(*cap).oap).op_type == OP_NOP
            && moved
            && fdo_flags.get() & kOptFdoFlagMark as c_int as c_uint != 0
            && old_key_typed
        {
            foldOpenCursor();
        }
    }
}

/// `'` and `` ` ``, and their `g` forms.
pub(crate) unsafe fn nv_gomark(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        // A mark used as an operator's motion must not restore the view.
        let mut flags = if (*(*cap).oap).op_type != OP_NOP {
            0
        } else {
            view_flag()
        };
        let old_key_typed = KeyTyped.get();

        // `g'` and ``g` `` jump without touching the previous-context mark.
        let name = if (*cap).cmdchar == 'g' as c_int {
            flags |= KMarkNoContext as MarkMove;
            (*cap).extra_char
        } else {
            flags |= kMarkContext as MarkMove;
            (*cap).nchar
        };
        if (*cap).arg != 0 {
            flags |= kMarkBeginLine as MarkMove;
        }
        // An explicit count means "restore the view too".
        if (*cap).count0 != 0 {
            flags |= kMarkSetView as MarkMove;
        }

        let fm = mark_get(curbuf.get(), curwin.get(), ptr::null_mut(), kMarkAll, name);
        let move_res = nv_mark_move_to(cap, flags, fm);
        if !virtual_active(curwin.get()) {
            (*curwin.get()).w_cursor.coladd = 0;
        }
        let moved = move_res & kMarkMoveSuccess as MarkMoveRes != 0
            && (move_res & kMarkSwitchedBuf as MarkMoveRes != 0
                || move_res & kMarkChangedCursor as MarkMoveRes != 0);
        may_open_fold(cap, moved, old_key_typed);
    }
}

/// `CTRL-O`, `CTRL-I` and `g;`/`g,`: step along the jump list or the change
/// list.
pub(crate) unsafe fn nv_pcmark(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let mut flags = view_flag();
        let mut move_res: MarkMoveRes = 0;
        let old_key_typed = KeyTyped.get();
        if checkclearopq((*cap).oap) {
            return;
        }
        // CTRL-TAB is the last-used tab page, not a jump.
        if (*cap).cmdchar == TAB && mod_mask.get() == MOD_MASK_CTRL {
            if !goto_tabpage_lastused() {
                clearopbeep((*cap).oap);
            }
            return;
        }

        let fm = if (*cap).cmdchar == 'g' as c_int {
            get_changelist(curbuf.get(), curwin.get(), (*cap).count1)
        } else {
            flags |= (KMarkNoContext as c_int | kMarkJumpList as c_int) as MarkMove;
            get_jumplist(curwin.get(), (*cap).count1)
        };

        if !fm.is_null() {
            move_res = nv_mark_move_to(cap, flags, fm);
        } else if (*cap).cmdchar == 'g' as c_int {
            // Three different reasons the change list had nothing.
            if (*curbuf.get()).b_changelistlen == 0 {
                emsg(gettext(e_changelist_is_empty.as_ptr()));
            } else if (*cap).count1 < 0 {
                emsg(gettext(c"E662: At start of changelist".as_ptr()));
            } else {
                emsg(gettext(c"E663: At end of changelist".as_ptr()));
            }
        } else {
            clearopbeep((*cap).oap);
        }

        let moved = move_res & kMarkSwitchedBuf as MarkMoveRes != 0
            || move_res & kMarkChangedLine as MarkMoveRes != 0;
        may_open_fold(cap, moved, old_key_typed);
    }
}
