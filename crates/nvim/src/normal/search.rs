//! Pattern searches driven from normal mode, and the marks and jumps that
//! share their "remember where we were" bookkeeping.
//!
//! What a search and a mark jump have in common is [`nv_mark_move_to`] and
//! the fold-opening tail: whether 'foldopen' lets the destination's fold
//! spring open depends on the command having been *typed*, not replayed.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::keycodes::ModMask;
use crate::mark::MARK_BEGIN_LINE;
use crate::mark::MARK_CHANGED_CURSOR;
use crate::mark::MARK_CHANGED_LINE;
use crate::mark::MARK_CONTEXT;
use crate::mark::MARK_JUMP_LIST;
use crate::mark::MARK_MOVE_FAILED;
use crate::mark::MARK_MOVE_SUCCESS;
use crate::mark::MARK_SET_VIEW;
use crate::mark::MARK_SWITCHED_BUF;
use crate::types::Failed;
use crate::types::Key;
use crate::winlayer::{Buf, Win};

use crate::cursor::check_cursor;
use crate::drawscreen::{UPD_SOME_VALID, redraw_later};
use crate::ex_getln::search_cmdline;
use crate::fold::fold_open_cursor;
use crate::getchar::state::{KEY_TYPED, MOD_MASK};
use crate::highlight::win_hl_attr;
use crate::highlight_group::{HLF_L, HLF_LC};
use crate::mark::defs::{FileMark, MarkMove, MarkMoveRes};
use crate::mark::{get_changelist, get_jumplist, mark_get, mark_move_to, setmark};
use crate::message::emsg;
use crate::normal::defs::CmdArg;
use crate::normal::{
    E_CHANGELIST_IS_EMPTY, K_MARK_NO_CONTEXT, MARK_ALL, MT_CHAR_WISE, MT_LINE_WISE, check_clear_op,
    check_clear_op_quit, clear_op, clear_op_beep, nv_operator,
};
use crate::ops::OpType;
use crate::option::vars::{FDO_FLAGS, JOP_FLAGS, p_hls};
use crate::options::{OPT_FDO_FLAG_MARK, OPT_FDO_FLAG_SEARCH, OPT_JOP_FLAG_VIEW};
use crate::os::cshim::gettext;
use crate::pos::equalpos;
use crate::search::state::NO_HLSEARCH;
use crate::search::state::SearchItArg;
use crate::search::{Hit, SEARCH_ECHO, SEARCH_MARK, SEARCH_MSG, SEARCH_OPT, do_search};
use crate::state::virtual_active;
use crate::window::goto_tabpage_lastused;
use core::ffi::c_int;

/// Whether the highlight of the previous match has to be redrawn.
///
/// Only when 'hlsearch' is on, `:nohlsearch` has not turned it off for now,
/// and the "current match" highlight actually differs from the others --
/// otherwise nothing on screen would change.
fn current_match_is_distinct() -> bool {
    p_hls()
        && !NO_HLSEARCH.get()
        && win_hl_attr(Win::current(), HLF_LC) != win_hl_attr(Win::current(), HLF_L)
}

/// `/` and `?`: read a pattern from the command line and search for it.
pub(crate) fn nv_search(cmd_arg: &mut CmdArg) {
    let op = cmd_arg.op();
    let save_cursor = Win::current().w_cursor;
    // `g?` is rot13; `?` after it is the operator, not a search.
    if cmd_arg.cmdchar.is('?') && op.op_type == OpType::Rot13 {
        cmd_arg.cmdchar = Key::ascii(b'g');
        cmd_arg.nchar = Key::ascii(b'?');
        nv_operator(cmd_arg);
        return;
    }
    cmd_arg.searchbuf = search_cmdline(cmd_arg.cmdchar.as_byte().unwrap_or(0), cmd_arg.count1);
    if cmd_arg.searchbuf.is_none() {
        clear_op(op);
        return;
    }
    // Reading the pattern may itself have moved the cursor ('incsearch'),
    // in which case the previous position is already on the jump list.
    let moved_while_typing = cmd_arg.arg != 0 || !equalpos(save_cursor, Win::current().w_cursor);
    let mark = if moved_while_typing {
        0
    } else {
        SEARCH_MARK as c_int
    };
    // A copy: the command keeps its buffer for the redo.
    let pat = cmd_arg.search_text().to_vec();
    normal_search(cmd_arg, cmd_arg.cmdchar.code(), &pat, mark, None);
}

impl CmdArg {
    /// The pattern `/` or `?` read, which the command keeps for its redo;
    /// empty before one is read.
    pub(crate) fn search_text(&self) -> &[u8] {
        self.searchbuf.as_deref().unwrap_or_default()
    }
}

/// `n` and `N`: search again for the last pattern.
pub(crate) fn nv_next(cmd_arg: &mut CmdArg) {
    let old = Win::current().w_cursor;
    let mut wrapped = false;
    let opt = SEARCH_MARK as c_int | cmd_arg.arg;
    let i = normal_search(cmd_arg, 0, b"", opt, Some(&mut wrapped));
    // A match that lands where the cursor already is, without having
    // wrapped, is the one we are standing on: search once more so `n`
    // always moves.
    if i == Some(Hit::Match) && !wrapped && equalpos(old, Win::current().w_cursor) {
        cmd_arg.count1 += 1;
        let again = SEARCH_MARK as c_int | cmd_arg.arg;
        normal_search(cmd_arg, 0, b"", again, None);
        cmd_arg.count1 -= 1;
    }
    if i.is_some() && current_match_is_distinct() {
        redraw_later(Win::current(), UPD_SOME_VALID);
    }
}

/// The search every normal-mode search command goes through.
///
/// Answers `do_search`'s result. `wrapped` is told whether the search
/// wrapped.
pub(crate) fn normal_search(
    cmd_arg: &mut CmdArg,
    dir: c_int,
    pat: &[u8],
    opt: c_int,
    wrapped: Option<&mut bool>,
) -> Option<Hit> {
    let mut sia = SearchItArg::default();
    let prev_cursor = Win::current().w_cursor;
    let mut op = cmd_arg.op();
    op.motion_type = MT_CHAR_WISE;
    op.inclusive = false;
    // A search is one of the motions that fills the "1 last change"
    // register rather than the small-delete one.
    op.use_reg_one = true;
    Win::current().w_set_curswant = true;

    let flags = opt | SEARCH_OPT as c_int | SEARCH_ECHO as c_int | SEARCH_MSG as c_int;
    let n = cmd_arg.count1;
    let i = do_search(Some(&mut *op), dir, dir, pat, n, flags, Some(&mut sia));
    if let Some(wrapped) = wrapped {
        *wrapped = sia.sa_wrapped;
    }

    if let Some(hit) = i {
        // A `/pat/+1`-style offset makes the motion linewise.
        if hit == Hit::Linewise {
            op.motion_type = MT_LINE_WISE;
        }
        Win::current().w_cursor.coladd = 0;
        if op.op_type == OpType::Nop
            && FDO_FLAGS.get() & OPT_FDO_FLAG_SEARCH != 0
            && KEY_TYPED.get()
        {
            fold_open_cursor();
        }
    } else {
        clear_op(op);
    }
    if !equalpos(Win::current().w_cursor, prev_cursor) && current_match_is_distinct() {
        redraw_later(Win::current(), UPD_SOME_VALID);
    }
    check_cursor(Win::current());
    i
}

/// `m`: set a mark.
pub(crate) fn nv_mark(cmd_arg: &mut CmdArg) {
    if check_clear_op(cmd_arg.op()) {
        return;
    }
    if cmd_arg
        .nchar
        .as_byte()
        .ok_or(Failed)
        .and_then(setmark)
        .is_err()
    {
        clear_op_beep(cmd_arg.op());
    }
}

/// Jump to a mark, and describe the jump to the operator that may be pending.
///
/// `None` is a name that named no mark, which the jump reports.
pub(crate) fn nv_mark_move_to(
    cmd_arg: &mut CmdArg,
    flags: MarkMove,
    fm: Option<&FileMark>,
) -> MarkMoveRes {
    let res = mark_move_to(fm, flags);
    if res & MARK_MOVE_FAILED as MarkMoveRes != 0 {
        clear_op(cmd_arg.op());
    }
    // `'a` is linewise, `` `a `` is charwise -- and only the charwise form
    // fills the "1 last change" register.
    cmd_arg.op().motion_type = if flags & MARK_BEGIN_LINE as MarkMove != 0 {
        MT_LINE_WISE
    } else {
        MT_CHAR_WISE
    };
    if cmd_arg.cmdchar.is('`') {
        cmd_arg.op().use_reg_one = true;
    }
    cmd_arg.op().inclusive = false;
    Win::current().w_set_curswant = true;
    res
}

/// The 'jumpoptions' half of a mark jump's flags: whether the view is
/// restored along with the position.
fn view_flag() -> MarkMove {
    if JOP_FLAGS.get() & OPT_JOP_FLAG_VIEW != 0 {
        MARK_SET_VIEW as MarkMove
    } else {
        0
    }
}

/// Whether the destination's fold should be opened.
///
/// `old_KeyTyped` rather than the current value: the jump itself may have
/// consumed the "typed" flag.
fn may_open_fold(cmd_arg: &mut CmdArg, moved: bool, old_key_typed: bool) {
    if cmd_arg.op().op_type == OpType::Nop
        && moved
        && FDO_FLAGS.get() & OPT_FDO_FLAG_MARK != 0
        && old_key_typed
    {
        fold_open_cursor();
    }
}

/// `'` and `` ` ``, and their `g` forms.
pub(crate) fn nv_gomark(cmd_arg: &mut CmdArg) {
    // A mark used as an operator's motion must not restore the view.
    let mut flags = if cmd_arg.op().op_type != OpType::Nop {
        0
    } else {
        view_flag()
    };
    let old_key_typed = KEY_TYPED.get();

    // `g'` and ``g` `` jump without touching the previous-context mark.
    let name = if cmd_arg.cmdchar.is('g') {
        flags |= K_MARK_NO_CONTEXT as MarkMove;
        cmd_arg.extra_char
    } else {
        flags |= MARK_CONTEXT as MarkMove;
        cmd_arg.nchar
    };
    if cmd_arg.arg != 0 {
        flags |= MARK_BEGIN_LINE as MarkMove;
    }
    // An explicit count means "restore the view too".
    if cmd_arg.count0 != 0 {
        flags |= MARK_SET_VIEW as MarkMove;
    }

    // A copy of the mark, which outlives the jump below.
    let fm = name
        .as_byte()
        .and_then(|name| mark_get(Buf::current(), Win::current(), MARK_ALL, name));
    let move_res = nv_mark_move_to(cmd_arg, flags, fm.as_ref());
    if !virtual_active(Win::current()) {
        Win::current().w_cursor.coladd = 0;
    }
    let moved = move_res & MARK_MOVE_SUCCESS as MarkMoveRes != 0
        && (move_res & MARK_SWITCHED_BUF as MarkMoveRes != 0
            || move_res & MARK_CHANGED_CURSOR as MarkMoveRes != 0);
    may_open_fold(cmd_arg, moved, old_key_typed);
}

/// `CTRL-O`, `CTRL-I` and `g;`/`g,`: step along the jump list or the change
/// list.
pub(crate) fn nv_pcmark(cmd_arg: &mut CmdArg) {
    let mut flags = view_flag();
    let mut move_res: MarkMoveRes = 0;
    let old_key_typed = KEY_TYPED.get();
    if check_clear_op_quit(cmd_arg.op()) {
        return;
    }
    // CTRL-TAB is the last-used tab page, not a jump.
    if cmd_arg.cmdchar == Key::TAB && MOD_MASK.get() == ModMask::CTRL {
        if !goto_tabpage_lastused() {
            clear_op_beep(cmd_arg.op());
        }
        return;
    }

    let fm = if cmd_arg.cmdchar.is('g') {
        get_changelist(Buf::current(), Win::current(), cmd_arg.count1)
    } else {
        flags |= K_MARK_NO_CONTEXT | MARK_JUMP_LIST;
        get_jumplist(Win::current(), cmd_arg.count1)
    };

    if let Some(fm) = &fm {
        move_res = nv_mark_move_to(cmd_arg, flags, Some(fm));
    } else if cmd_arg.cmdchar.is('g') {
        // Three different reasons the change list had nothing.
        if Buf::current().b_changelistlen == 0 {
            emsg(gettext(E_CHANGELIST_IS_EMPTY));
        } else if cmd_arg.count1 < 0 {
            emsg(gettext(c"E662: At start of changelist"));
        } else {
            emsg(gettext(c"E663: At end of changelist"));
        }
    } else {
        clear_op_beep(cmd_arg.op());
    }

    let moved = move_res & MARK_SWITCHED_BUF as MarkMoveRes != 0
        || move_res & MARK_CHANGED_LINE as MarkMoveRes != 0;
    may_open_fold(cmd_arg, moved, old_key_typed);
}
