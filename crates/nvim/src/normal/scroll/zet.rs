//! The `z` prefix tree: positioning the view, scrolling sideways, the fold
//! commands and the spellfile additions.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::winlayer::{Buf, Win, windows};
use core::ptr;

use crate::ascii::ascii_isdigit;
use crate::cursor::{check_cursor_col, set_leftcol};
use crate::drawscreen::{UPD_NOT_VALID, UPD_VALID, redraw_later};
use crate::edit::{BeginlineOpts, beginline};
use crate::fold::{
    clear_folding, close_fold, close_fold_recurse, deepest_fold_nesting, delete_fold,
    fold_manual_allowed, fold_move_to, fold_open_cursor, foldmethod_is_diff, foldmethod_is_manual,
    foldmethod_is_marker, has_folding, new_fold_level, open_fold, open_fold_recurse,
};
use crate::guard::Suppress;
use crate::main::{curwin, finish_op};
use crate::mark::setpcmark;
use crate::memline::ml_get_pos;
use crate::message::emsg;
use crate::normal::{
    CAR, CmdArg, FIND_IDENT, INT_MAX, SPELL_ADD_BAD, SPELL_ADD_GOOD, check_clear_op, clear_op_beep,
    find_ident_under_cursor, get_visual_text, nv_operator, nv_put, read_command_char,
    visual_active,
};
use crate::option::get_sidescrolloff_value;
use crate::os::cshim::gettext;
use crate::spell::{SMT_ALL, spell_move_to};
use crate::spellfile::spell_add_word;
use crate::spellsuggest::spell_suggest;
use crate::strings::vim_strchr;
use crate::types::{
    FAIL, OK, OP_FOLD, OP_NOP, OptInt, SpellAddType, cmdarg_T, colnr_T, int64_t, linenr_T, size_t,
};
use crate::window::{set_fraction, win_setheight};
use core::ffi::{c_char, c_int};

use crate::keycodes::{K_DEL, K_KDEL, K_KENTER, K_LEFT, K_RIGHT};
use crate::r#move::{
    changed_window_setting, scroll_cursor_bot, scroll_cursor_halfway, scroll_cursor_top,
    validate_botline_win, win_col_off,
};
use crate::search::{BACKWARD, FORWARD};

/// Where the positioning commands put the cursor's line.
#[derive(Copy, Clone)]
enum Place {
    /// `zt`, `z<CR>`, `z+`.
    Top,
    /// `zz`, `z.`.
    Middle,
    /// `zb`, `z-`, `z^`.
    Bottom,
}

/// `z<digits>`: read the rest of a count that belongs to `z` itself.
///
/// `z<n><CR>` sets the window height and is finished here. `z<n>l` and its
/// three friends multiply the command's own count by this one and hand the
/// key back to the caller through `nchar_arg`; everything else is an error.
pub(crate) unsafe fn nv_z_get_count(cap: *mut cmdarg_T, nchar_arg: *mut c_int) -> bool {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    // SAFETY: `cap` is the caller's live command argument and `nchar_arg`
    // points at the caller's own second character.
    if check_clear_op(ca.op()) {
        return false;
    }
    let mut n = unsafe { *nchar_arg } - '0' as c_int;
    loop {
        let nchar = unsafe { read_command_char() };
        if nchar == K_DEL || nchar == K_KDEL {
            // Rubbing out a digit.
            n /= 10;
        } else if ascii_isdigit(nchar) {
            if crate::math::vim_append_digit_int(&mut n, nchar - '0' as c_int) {
                continue;
            }
            clear_op_beep(ca.op());
            break;
        } else if nchar == CAR {
            win_setheight(n);
            break;
        } else if nchar == 'l' as c_int
            || nchar == 'h' as c_int
            || nchar == K_LEFT
            || nchar == K_RIGHT
        {
            // Both counts came from the user, so this can overflow -- the
            // C wraps, and `set_leftcol` clamps whatever comes out.
            if n != 0 {
                ca.count1 = n.wrapping_mul(ca.count1);
            }
            unsafe { *nchar_arg = nchar };
            return true;
        } else {
            clear_op_beep(ca.op());
            break;
        }
    }
    ca.op().op_type = OP_NOP;
    false
}

/// `zg`, `zG`, `zw`, `zW` and the `zug` family: add the word under the cursor
/// to the spellfile as good or as bad, or take it back out again.
///
/// Answers `FAIL` when there was no word to act on, which stops `nv_zet`
/// running its tail.
pub(crate) unsafe fn nv_zg_zw(cap: *mut cmdarg_T, mut nchar: c_int) -> c_int {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    // `zu` is the undo prefix: `zug` takes back what `zg` added.
    let mut undo = false;
    if nchar == 'u' as c_int {
        nchar = unsafe { read_command_char() };
        if unsafe { vim_strchr(c"gGwW".as_ptr(), nchar) }.is_null() {
            clear_op_beep(ca.op());
            return OK;
        }
        undo = true;
    }
    if check_clear_op(ca.op()) {
        return OK;
    }

    // Three ways to find the word, in order: the selection, the
    // misspelling the cursor is inside, and the identifier under it.
    let mut word: *mut c_char = ptr::null_mut();
    let mut len: size_t = 0;
    if visual_active() && !unsafe { get_visual_text(cap, &raw mut word, &raw mut len) } {
        return FAIL;
    }
    if word.is_null() {
        let pos = cur_win().w_cursor;
        // The search is only being used to find where the bad word
        // starts; its "no more misspellings" message is not wanted.
        let no_emsg = Suppress::emsg();
        let (fwd, none) = (FORWARD as c_int, ptr::null_mut());
        len = unsafe { spell_move_to(curwin.get(), fwd, SMT_ALL, true, none) };
        drop(no_emsg);
        // Only if it found one at or before the cursor, i.e. the one the
        // cursor is inside rather than the next one.
        if len != 0 && cur_win().w_cursor.col <= pos.col {
            word = unsafe { ml_get_pos(&raw mut (*curwin.get()).w_cursor) };
        }
        cur_win().w_cursor = pos;
    }
    if word.is_null() {
        len =
            unsafe { find_ident_under_cursor(&raw mut word, FIND_IDENT as c_int, ptr::null_mut()) };
        if len == 0 {
            return FAIL;
        }
    }
    debug_assert!(len <= c_int::MAX as size_t);

    // Lower case adds to the file 'spellfile' names, upper case to the
    // internal list that lasts for this session only -- which is why the
    // upper-case forms pass no index.
    let what = if nchar == 'w' as c_int || nchar == 'W' as c_int {
        SPELL_ADD_BAD as SpellAddType
    } else {
        SPELL_ADD_GOOD as SpellAddType
    };
    let index = if nchar == 'G' as c_int || nchar == 'W' as c_int {
        0
    } else {
        ca.count1
    };
    unsafe { spell_add_word(word, len as c_int, what, index, undo) };
    OK
}

/// Scroll sideways by `count1` columns, which 'wrap' makes meaningless.
unsafe fn scroll_sideways(cap: *mut cmdarg_T, right: bool) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    let mut win = cur_win();
    if win.w_onebuf_opt.wo_wrap != 0 {
        return;
    }
    let target = if right {
        win.w_leftcol + ca.count1
    } else if ca.count1 > win.w_leftcol {
        0
    } else {
        win.w_leftcol - ca.count1
    };
    unsafe { set_leftcol(target) };
}

/// `zs` and `ze`: scroll sideways until the cursor is at the left or right
/// edge, keeping 'sidescrolloff' columns of context.
unsafe fn scroll_cursor_to_edge(to_left: bool) {
    // SAFETY (throughout): reads and scrolls the current window.
    let mut win = cur_win();
    if win.w_onebuf_opt.wo_wrap != 0 {
        return;
    }
    let siso = get_sidescrolloff_value(win);
    let mut col: colnr_T = 0;
    // A closed fold shows one line of its own, which starts at column 0.
    if !folded(win.w_cursor.lnum) {
        let (start, end) = win.vcol_span(win.cursor());
        col = if to_left { start } else { end };
    }
    if to_left {
        col = if col as int64_t > siso {
            col - siso as c_int
        } else {
            0
        };
    } else {
        let width = win.w_view_width - unsafe { win_col_off(win.raw()) };
        col = if (col as int64_t + siso) < width as int64_t {
            0
        } else if (siso - width as int64_t) < (INT_MAX - col) as int64_t {
            (col as int64_t + siso - width as int64_t + 1) as c_int
        } else {
            INT_MAX
        };
    }
    if win.w_leftcol != col {
        win.w_leftcol = col;
        unsafe { redraw_later(win.raw(), UPD_NOT_VALID) };
    }
}

/// The fold half of the `z` tree. Answers whether the key was one of them.
unsafe fn nv_zet_fold(cap: *mut cmdarg_T, nchar: c_int, old_fdl: &mut c_int) -> bool {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    let mut win = cur_win();
    // Whether the cursor is inside a fold, which is what decides between
    // opening and closing for the toggles.
    let in_fold = || folded(win.w_cursor.lnum);
    match u8::try_from(nchar) {
        // `zf`/`zF`: create a fold. `zF` folds `count1` lines, which is
        // the operator applied to itself.
        Ok(b'F' | b'f') => {
            if unsafe { fold_manual_allowed(true) } != 0 {
                ca.nchar = 'f' as c_int;
                unsafe { nv_operator(cap) };
                win.w_onebuf_opt.wo_fen = 1;
                if nchar == 'F' as c_int && ca.op().op_type == OP_FOLD {
                    unsafe { nv_operator(cap) };
                    finish_op.set(true);
                }
            } else {
                clear_op_beep(ca.op());
            }
        }
        // `zd`/`zD`: delete a fold, recursively for `zD`.
        Ok(b'd' | b'D') => {
            if unsafe { fold_manual_allowed(false) } != 0 {
                if visual_active() {
                    unsafe { nv_operator(cap) };
                } else {
                    let lnum = win.w_cursor.lnum;
                    let deep = (nchar == 'D' as c_int) as c_int;
                    unsafe { delete_fold(win.raw(), lnum, lnum, deep, false) };
                }
            }
        }
        // `zE`: delete every fold.
        Ok(b'E') => {
            if foldmethod_is_manual(win) {
                clear_folding(win);
                changed_window_setting(win);
            } else if foldmethod_is_marker(win) {
                unsafe { delete_fold(win.raw(), 1, cur_buf().b_ml.ml_line_count, 1, false) };
            } else {
                let msg = c"E352: Cannot erase folds with current 'foldmethod'";
                unsafe { emsg(gettext(msg.as_ptr())) };
            }
        }
        // `zn`/`zN`/`zi`: 'foldenable' off, on, toggled.
        Ok(b'n') => win.w_onebuf_opt.wo_fen = 0,
        Ok(b'N') => win.w_onebuf_opt.wo_fen = 1,
        Ok(b'i') => win.w_onebuf_opt.wo_fen = (win.w_onebuf_opt.wo_fen == 0) as c_int,
        // `za`/`zA`: toggle this fold, recursively for `zA`.
        Ok(b'a') => {
            if in_fold() {
                unsafe { open_fold(win.w_cursor, ca.count1) };
            } else {
                unsafe { close_fold(win.w_cursor, ca.count1) };
                win.w_onebuf_opt.wo_fen = 1;
            }
        }
        Ok(b'A') => {
            if in_fold() {
                unsafe { open_fold_recurse(win.w_cursor) };
            } else {
                unsafe { close_fold_recurse(win.w_cursor) };
                win.w_onebuf_opt.wo_fen = 1;
            }
        }
        // `zo`/`zO`: open. With a selection they are the operator form.
        Ok(b'o') => {
            if visual_active() {
                unsafe { nv_operator(cap) };
            } else {
                unsafe { open_fold(win.w_cursor, ca.count1) };
            }
        }
        Ok(b'O') => {
            if visual_active() {
                unsafe { nv_operator(cap) };
            } else {
                unsafe { open_fold_recurse(win.w_cursor) };
            }
        }
        // `zc`/`zC`: close. Closing always turns 'foldenable' back on --
        // there would be nothing to see otherwise.
        Ok(b'c') => {
            if visual_active() {
                unsafe { nv_operator(cap) };
            } else {
                unsafe { close_fold(win.w_cursor, ca.count1) };
            }
            win.w_onebuf_opt.wo_fen = 1;
        }
        Ok(b'C') => {
            if visual_active() {
                unsafe { nv_operator(cap) };
            } else {
                unsafe { close_fold_recurse(win.w_cursor) };
            }
            win.w_onebuf_opt.wo_fen = 1;
        }
        // `zv`: open just enough to see the cursor line.
        Ok(b'v') => unsafe { fold_open_cursor() },
        // `zx`/`zX`: recompute the folds. `zx` also reopens to the cursor.
        Ok(b'x') => {
            win.w_onebuf_opt.wo_fen = 1;
            win.w_foldinvalid = true;
            unsafe { new_fold_level() };
            unsafe { fold_open_cursor() };
        }
        Ok(b'X') => {
            win.w_onebuf_opt.wo_fen = 1;
            win.w_foldinvalid = true;
            // Force the tail's `new_fold_level`.
            *old_fdl = -1;
        }
        // `zm`/`zM`: fold more, or all the way.
        Ok(b'm') => {
            if win.w_onebuf_opt.wo_fdl > 0 {
                win.w_onebuf_opt.wo_fdl -= ca.count1 as OptInt;
                win.w_onebuf_opt.wo_fdl = win.w_onebuf_opt.wo_fdl.max(0);
            }
            *old_fdl = -1;
            win.w_onebuf_opt.wo_fen = 1;
        }
        Ok(b'M') => {
            win.w_onebuf_opt.wo_fdl = 0;
            *old_fdl = -1;
            win.w_onebuf_opt.wo_fen = 1;
        }
        // `zr`/`zR`: reduce the folding, or open everything.
        Ok(b'r') => {
            win.w_onebuf_opt.wo_fdl += ca.count1 as OptInt;
            let deepest = deepest_fold_nesting(win);
            win.w_onebuf_opt.wo_fdl = win.w_onebuf_opt.wo_fdl.min(deepest as OptInt);
        }
        Ok(b'R') => {
            win.w_onebuf_opt.wo_fdl = deepest_fold_nesting(win) as OptInt;
            *old_fdl = -1;
        }
        // `zj`/`zk`: to the next or previous fold's edge.
        Ok(b'j' | b'k') => {
            let dir = if nchar == 'j' as c_int {
                FORWARD as c_int
            } else {
                BACKWARD as c_int
            };
            if unsafe { fold_move_to(true, dir, ca.count1) } == 0 {
                clear_op_beep(ca.op());
            }
        }
        _ => return false,
    }
    true
}

/// `z`, whose second character says what part of the view or of the folding
/// it is about.
pub(crate) unsafe fn nv_zet(cap: *mut cmdarg_T) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    let mut win = cur_win();
    let mut nchar = ca.nchar;
    let mut old_fdl = win.w_onebuf_opt.wo_fdl as c_int;
    let old_fen = win.w_onebuf_opt.wo_fen;

    // `z` may take a count of its own between the `z` and the command.
    if ascii_isdigit(nchar) && !unsafe { nv_z_get_count(cap, &raw mut nchar) } {
        return;
    }
    // The commands that are operators or motions of their own answer for
    // a pending operator themselves; the rest refuse one.
    if ca.nchar != 'f' as c_int
        && ca.nchar != 'F' as c_int
        && !(visual_active() && !unsafe { vim_strchr(c"dcCoO".as_ptr(), ca.nchar) }.is_null())
        && ca.nchar != 'j' as c_int
        && ca.nchar != 'k' as c_int
        && check_clear_op(ca.op())
    {
        return;
    }
    // For the positioning commands a count names the line to position,
    // not how many of anything.
    if !unsafe { vim_strchr(c"+\r\nt.z^-b".as_ptr(), nchar) }.is_null()
        && ca.count0 != 0
        && ca.count0 as linenr_T != win.w_cursor.lnum
    {
        setpcmark();
        let count0 = ca.count0 as linenr_T;
        win.w_cursor.lnum = count0.min(cur_buf().b_ml.ml_line_count);
        check_cursor_col(win);
    }

    // Where the cursor line ends up, and whether the cursor also moves to
    // the first non-blank of it. `None` means the key did its own work.
    let place: Option<(Place, bool)> = match nchar {
        // The three keys that are not bytes.
        K_KENTER => Some((Place::Top, true)),
        K_LEFT => {
            unsafe { scroll_sideways(cap, false) };
            None
        }
        K_RIGHT => {
            unsafe { scroll_sideways(cap, true) };
            None
        }
        _ => match u8::try_from(nchar) {
            Ok(b'\r' | b'\n') => Some((Place::Top, true)),
            Ok(b'+') => {
                // Without a count, `z+` starts from the line below the
                // window rather than from the cursor.
                if ca.count0 == 0 {
                    validate_botline_win(win);
                    win.w_cursor.lnum = win.w_botline.min(cur_buf().b_ml.ml_line_count);
                }
                Some((Place::Top, true))
            }
            Ok(b't') => Some((Place::Top, false)),
            Ok(b'.') => Some((Place::Middle, true)),
            Ok(b'z') => Some((Place::Middle, false)),
            Ok(b'^') => {
                // `z^` positions the line *above* the window, so with no
                // count it first scrolls the current top out of sight.
                if ca.count0 != 0 {
                    scroll_cursor_bot(win, 0, true);
                    win.w_cursor.lnum = win.w_topline;
                } else if win.w_topline == 1 {
                    win.w_cursor.lnum = 1;
                } else {
                    win.w_cursor.lnum = win.w_topline - 1;
                }
                Some((Place::Bottom, true))
            }
            Ok(b'-') => Some((Place::Bottom, true)),
            Ok(b'b') => Some((Place::Bottom, false)),
            // `zH`/`zL` scroll half a screen each, so the count and the
            // width multiply -- and the count is the user's. The C wraps;
            // `set_leftcol` clamps whatever comes out.
            Ok(b'H') => {
                ca.count1 = ca.count1.wrapping_mul(win.w_view_width / 2);
                unsafe { scroll_sideways(cap, false) };
                None
            }
            Ok(b'h') => {
                unsafe { scroll_sideways(cap, false) };
                None
            }
            Ok(b'L') => {
                ca.count1 = ca.count1.wrapping_mul(win.w_view_width / 2);
                unsafe { scroll_sideways(cap, true) };
                None
            }
            Ok(b'l') => {
                unsafe { scroll_sideways(cap, true) };
                None
            }
            Ok(b's') => {
                unsafe { scroll_cursor_to_edge(true) };
                None
            }
            Ok(b'e') => {
                unsafe { scroll_cursor_to_edge(false) };
                None
            }
            // `zp`/`zP`: put a blockwise register without widening the
            // lines it lands on.
            Ok(b'P' | b'p') => {
                unsafe { nv_put(cap) };
                None
            }
            // `zy`: yank without a trailing newline.
            Ok(b'y') => {
                unsafe { nv_operator(cap) };
                None
            }
            // `zg`/`zG`/`zw`/`zW`/`zu…`: the spellfile.
            Ok(b'u' | b'g' | b'w' | b'G' | b'W') => {
                if unsafe { nv_zg_zw(cap, nchar) } == FAIL {
                    return;
                }
                None
            }
            // `z=`: suggest corrections.
            Ok(b'=') => {
                if !check_clear_op(ca.op()) {
                    unsafe { spell_suggest(ca.count0) };
                }
                None
            }
            _ => {
                if !unsafe { nv_zet_fold(cap, nchar, &mut old_fdl) } {
                    clear_op_beep(ca.op());
                }
                None
            }
        },
    };

    if let Some((place, to_first_non_blank)) = place {
        if to_first_non_blank {
            beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
        }
        match place {
            Place::Top => scroll_cursor_top(win, 0, 1),
            Place::Middle => scroll_cursor_halfway(win, true, false),
            Place::Bottom => scroll_cursor_bot(win, 0, true),
        }
        unsafe { redraw_later(win.raw(), UPD_VALID) };
        unsafe { set_fraction(win.raw()) };
    }

    if old_fen != win.w_onebuf_opt.wo_fen {
        // Windows bound by 'scrollbind' in diff mode have to fold alike,
        // or the same line is at a different height in each.
        if foldmethod_is_diff(win) && win.w_onebuf_opt.wo_scb != 0 {
            for mut wp in windows() {
                if wp.raw() != win.raw() && foldmethod_is_diff(wp) && wp.w_onebuf_opt.wo_scb != 0 {
                    wp.w_onebuf_opt.wo_fen = win.w_onebuf_opt.wo_fen;
                    changed_window_setting(wp);
                }
            }
        }
        changed_window_setting(win);
    }
    if old_fdl as OptInt != win.w_onebuf_opt.wo_fdl {
        unsafe { new_fold_level() };
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

/// Whether `lnum` is inside a closed fold of the current window.
fn folded(lnum: linenr_T) -> bool {
    // SAFETY: `cur_win()` is the live window.
    has_folding(cur_win(), lnum, None, None)
}
