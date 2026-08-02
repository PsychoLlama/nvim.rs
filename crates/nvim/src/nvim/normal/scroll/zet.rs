//! The `z` prefix tree: positioning the view, scrolling sideways, the fold
//! commands and the spellfile additions.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ptr;

use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::cursor::{check_cursor_col, set_leftcol};
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, UPD_VALID, redraw_later};
use crate::src::nvim::edit::beginline;
use crate::src::nvim::fold::{
    clearFolding, closeFold, closeFoldRecurse, deleteFold, foldManualAllowed, foldMoveTo,
    foldOpenCursor, foldmethodIsDiff, foldmethodIsManual, foldmethodIsMarker, getDeepestNesting,
    hasFolding, newFoldLevel, openFold, openFoldRecurse,
};
use crate::src::nvim::main::{VIsual_active, curbuf, curwin, emsg_off, finish_op, firstwin};
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::memline::ml_get_pos;
use crate::src::nvim::message::emsg;
use crate::src::nvim::normal::{
    BL_FIX, BL_WHITE, CAR, FAIL, FIND_IDENT, INT_MAX, OK, OP_FOLD, OP_NOP, SPELL_ADD_BAD,
    SPELL_ADD_GOOD, checkclearop, clearopbeep, false_0, find_ident_under_cursor, get_visual_text,
    nv_operator, nv_put, read_command_char, true_0,
};
use crate::src::nvim::option::get_sidescrolloff_value;
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::plines::getvcol;
use crate::src::nvim::spell::{SMT_ALL, spell_move_to};
use crate::src::nvim::spellfile::spell_add_word;
use crate::src::nvim::spellsuggest::spell_suggest;
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{OptInt, SpellAddType, cmdarg_T, colnr_T, int64_t, linenr_T, size_t};
use crate::src::nvim::window::{set_fraction, win_setheight};
use core::ffi::{c_char, c_int};

use crate::src::nvim::keycodes::{K_DEL, K_KDEL, K_KENTER, K_LEFT, K_RIGHT};
use crate::src::nvim::r#move::{
    changed_window_setting, scroll_cursor_bot, scroll_cursor_halfway, scroll_cursor_top,
    validate_botline_win, win_col_off,
};
use crate::src::nvim::search::{BACKWARD, FORWARD};

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
    // SAFETY: `cap` is the caller's live command argument and `nchar_arg`
    // points at the caller's own second character.
    unsafe {
        if checkclearop((*cap).oap) {
            return false;
        }
        let mut n = *nchar_arg - '0' as c_int;
        loop {
            let nchar = read_command_char();
            if nchar == K_DEL || nchar == K_KDEL {
                // Rubbing out a digit.
                n /= 10;
            } else if ascii_isdigit(nchar) {
                if crate::src::nvim::math::vim_append_digit_int(&mut n, nchar - '0' as c_int) {
                    continue;
                }
                clearopbeep((*cap).oap);
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
                    (*cap).count1 = n.wrapping_mul((*cap).count1);
                }
                *nchar_arg = nchar;
                return true;
            } else {
                clearopbeep((*cap).oap);
                break;
            }
        }
        (*(*cap).oap).op_type = OP_NOP as c_int;
        false
    }
}

/// `zg`, `zG`, `zw`, `zW` and the `zug` family: add the word under the cursor
/// to the spellfile as good or as bad, or take it back out again.
///
/// Answers `FAIL` when there was no word to act on, which stops `nv_zet`
/// running its tail.
pub(crate) unsafe fn nv_zg_zw(cap: *mut cmdarg_T, mut nchar: c_int) -> c_int {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        // `zu` is the undo prefix: `zug` takes back what `zg` added.
        let mut undo = false;
        if nchar == 'u' as c_int {
            nchar = read_command_char();
            if vim_strchr(c"gGwW".as_ptr(), nchar).is_null() {
                clearopbeep((*cap).oap);
                return OK;
            }
            undo = true;
        }
        if checkclearop((*cap).oap) {
            return OK;
        }

        // Three ways to find the word, in order: the selection, the
        // misspelling the cursor is inside, and the identifier under it.
        let mut word: *mut c_char = ptr::null_mut();
        let mut len: size_t = 0;
        if VIsual_active.get() && !get_visual_text(cap, &raw mut word, &raw mut len) {
            return FAIL;
        }
        if word.is_null() {
            let pos = (*curwin.get()).w_cursor;
            // The search is only being used to find where the bad word
            // starts; its "no more misspellings" message is not wanted.
            (*emsg_off.ptr()) += 1;
            len = spell_move_to(
                curwin.get(),
                FORWARD as c_int,
                SMT_ALL,
                true,
                ptr::null_mut(),
            );
            (*emsg_off.ptr()) -= 1;
            // Only if it found one at or before the cursor, i.e. the one the
            // cursor is inside rather than the next one.
            if len != 0 && (*curwin.get()).w_cursor.col <= pos.col {
                word = ml_get_pos(&raw mut (*curwin.get()).w_cursor);
            }
            (*curwin.get()).w_cursor = pos;
        }
        if word.is_null() {
            len = find_ident_under_cursor(&raw mut word, FIND_IDENT as c_int, ptr::null_mut());
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
            (*cap).count1
        };
        spell_add_word(word, len as c_int, what, index, undo);
        OK
    }
}

/// Scroll sideways by `count1` columns, which 'wrap' makes meaningless.
unsafe fn scroll_sideways(cap: *mut cmdarg_T, right: bool) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let win = curwin.get();
        if (*win).w_onebuf_opt.wo_wrap != 0 {
            return;
        }
        if right {
            set_leftcol((*win).w_leftcol + (*cap).count1);
        } else {
            set_leftcol(if (*cap).count1 > (*win).w_leftcol {
                0
            } else {
                (*win).w_leftcol - (*cap).count1
            });
        }
    }
}

/// `zs` and `ze`: scroll sideways until the cursor is at the left or right
/// edge, keeping 'sidescrolloff' columns of context.
unsafe fn scroll_cursor_to_edge(to_left: bool) {
    // SAFETY: reads and scrolls the current window.
    unsafe {
        let win = curwin.get();
        if (*win).w_onebuf_opt.wo_wrap != 0 {
            return;
        }
        let siso = get_sidescrolloff_value(win);
        let mut col: colnr_T = 0;
        // A closed fold shows one line of its own, which starts at column 0.
        if !hasFolding(win, (*win).w_cursor.lnum, ptr::null_mut(), ptr::null_mut()) {
            if to_left {
                getvcol(
                    win,
                    &raw mut (*win).w_cursor,
                    &raw mut col,
                    ptr::null_mut(),
                    ptr::null_mut(),
                );
            } else {
                getvcol(
                    win,
                    &raw mut (*win).w_cursor,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    &raw mut col,
                );
            }
        }
        if to_left {
            col = if col as int64_t > siso {
                col - siso as c_int
            } else {
                0
            };
        } else {
            let width = (*win).w_view_width - win_col_off(win);
            col = if (col as int64_t + siso) < width as int64_t {
                0
            } else if (siso - width as int64_t) < (INT_MAX - col) as int64_t {
                (col as int64_t + siso - width as int64_t + 1) as c_int
            } else {
                INT_MAX
            };
        }
        if (*win).w_leftcol != col {
            (*win).w_leftcol = col;
            redraw_later(win, UPD_NOT_VALID);
        }
    }
}

/// The fold half of the `z` tree. Answers whether the key was one of them.
unsafe fn nv_zet_fold(cap: *mut cmdarg_T, nchar: c_int, old_fdl: &mut c_int) -> bool {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let win = curwin.get();
        // Whether the cursor is inside a fold, which is what decides between
        // opening and closing for the toggles.
        let in_fold = || hasFolding(win, (*win).w_cursor.lnum, ptr::null_mut(), ptr::null_mut());
        match u8::try_from(nchar) {
            // `zf`/`zF`: create a fold. `zF` folds `count1` lines, which is
            // the operator applied to itself.
            Ok(b'F' | b'f') => {
                if foldManualAllowed(true) != 0 {
                    (*cap).nchar = 'f' as c_int;
                    nv_operator(cap);
                    (*win).w_onebuf_opt.wo_fen = true_0;
                    if nchar == 'F' as c_int && (*(*cap).oap).op_type == OP_FOLD as c_int {
                        nv_operator(cap);
                        finish_op.set(true);
                    }
                } else {
                    clearopbeep((*cap).oap);
                }
            }
            // `zd`/`zD`: delete a fold, recursively for `zD`.
            Ok(b'd' | b'D') => {
                if foldManualAllowed(false) != 0 {
                    if VIsual_active.get() {
                        nv_operator(cap);
                    } else {
                        deleteFold(
                            win,
                            (*win).w_cursor.lnum,
                            (*win).w_cursor.lnum,
                            (nchar == 'D' as c_int) as c_int,
                            false,
                        );
                    }
                }
            }
            // `zE`: delete every fold.
            Ok(b'E') => {
                if foldmethodIsManual(win) {
                    clearFolding(win);
                    changed_window_setting(win);
                } else if foldmethodIsMarker(win) {
                    deleteFold(win, 1, (*curbuf.get()).b_ml.ml_line_count, true_0, false);
                } else {
                    emsg(gettext(
                        c"E352: Cannot erase folds with current 'foldmethod'".as_ptr(),
                    ));
                }
            }
            // `zn`/`zN`/`zi`: 'foldenable' off, on, toggled.
            Ok(b'n') => (*win).w_onebuf_opt.wo_fen = false_0,
            Ok(b'N') => (*win).w_onebuf_opt.wo_fen = true_0,
            Ok(b'i') => (*win).w_onebuf_opt.wo_fen = ((*win).w_onebuf_opt.wo_fen == 0) as c_int,
            // `za`/`zA`: toggle this fold, recursively for `zA`.
            Ok(b'a') => {
                if in_fold() {
                    openFold((*win).w_cursor, (*cap).count1);
                } else {
                    closeFold((*win).w_cursor, (*cap).count1);
                    (*win).w_onebuf_opt.wo_fen = true_0;
                }
            }
            Ok(b'A') => {
                if in_fold() {
                    openFoldRecurse((*win).w_cursor);
                } else {
                    closeFoldRecurse((*win).w_cursor);
                    (*win).w_onebuf_opt.wo_fen = true_0;
                }
            }
            // `zo`/`zO`: open. With a selection they are the operator form.
            Ok(b'o') => {
                if VIsual_active.get() {
                    nv_operator(cap);
                } else {
                    openFold((*win).w_cursor, (*cap).count1);
                }
            }
            Ok(b'O') => {
                if VIsual_active.get() {
                    nv_operator(cap);
                } else {
                    openFoldRecurse((*win).w_cursor);
                }
            }
            // `zc`/`zC`: close. Closing always turns 'foldenable' back on --
            // there would be nothing to see otherwise.
            Ok(b'c') => {
                if VIsual_active.get() {
                    nv_operator(cap);
                } else {
                    closeFold((*win).w_cursor, (*cap).count1);
                }
                (*win).w_onebuf_opt.wo_fen = true_0;
            }
            Ok(b'C') => {
                if VIsual_active.get() {
                    nv_operator(cap);
                } else {
                    closeFoldRecurse((*win).w_cursor);
                }
                (*win).w_onebuf_opt.wo_fen = true_0;
            }
            // `zv`: open just enough to see the cursor line.
            Ok(b'v') => foldOpenCursor(),
            // `zx`/`zX`: recompute the folds. `zx` also reopens to the cursor.
            Ok(b'x') => {
                (*win).w_onebuf_opt.wo_fen = true_0;
                (*win).w_foldinvalid = true;
                newFoldLevel();
                foldOpenCursor();
            }
            Ok(b'X') => {
                (*win).w_onebuf_opt.wo_fen = true_0;
                (*win).w_foldinvalid = true;
                // Force the tail's `newFoldLevel`.
                *old_fdl = -1;
            }
            // `zm`/`zM`: fold more, or all the way.
            Ok(b'm') => {
                if (*win).w_onebuf_opt.wo_fdl > 0 {
                    (*win).w_onebuf_opt.wo_fdl -= (*cap).count1 as OptInt;
                    (*win).w_onebuf_opt.wo_fdl = (*win).w_onebuf_opt.wo_fdl.max(0);
                }
                *old_fdl = -1;
                (*win).w_onebuf_opt.wo_fen = true_0;
            }
            Ok(b'M') => {
                (*win).w_onebuf_opt.wo_fdl = 0;
                *old_fdl = -1;
                (*win).w_onebuf_opt.wo_fen = true_0;
            }
            // `zr`/`zR`: reduce the folding, or open everything.
            Ok(b'r') => {
                (*win).w_onebuf_opt.wo_fdl += (*cap).count1 as OptInt;
                let deepest = getDeepestNesting(win);
                (*win).w_onebuf_opt.wo_fdl = (*win).w_onebuf_opt.wo_fdl.min(deepest as OptInt);
            }
            Ok(b'R') => {
                (*win).w_onebuf_opt.wo_fdl = getDeepestNesting(win) as OptInt;
                *old_fdl = -1;
            }
            // `zj`/`zk`: to the next or previous fold's edge.
            Ok(b'j' | b'k') => {
                let dir = if nchar == 'j' as c_int {
                    FORWARD as c_int
                } else {
                    BACKWARD as c_int
                };
                if foldMoveTo(true, dir, (*cap).count1) == false_0 {
                    clearopbeep((*cap).oap);
                }
            }
            _ => return false,
        }
        true
    }
}

/// `z`, whose second character says what part of the view or of the folding
/// it is about.
pub(crate) unsafe fn nv_zet(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let win = curwin.get();
        let mut nchar = (*cap).nchar;
        let mut old_fdl = (*win).w_onebuf_opt.wo_fdl as c_int;
        let old_fen = (*win).w_onebuf_opt.wo_fen;

        // `z` may take a count of its own between the `z` and the command.
        if ascii_isdigit(nchar) && !nv_z_get_count(cap, &raw mut nchar) {
            return;
        }
        // The commands that are operators or motions of their own answer for
        // a pending operator themselves; the rest refuse one.
        if (*cap).nchar != 'f' as c_int
            && (*cap).nchar != 'F' as c_int
            && !(VIsual_active.get() && !vim_strchr(c"dcCoO".as_ptr(), (*cap).nchar).is_null())
            && (*cap).nchar != 'j' as c_int
            && (*cap).nchar != 'k' as c_int
            && checkclearop((*cap).oap)
        {
            return;
        }
        // For the positioning commands a count names the line to position,
        // not how many of anything.
        if !vim_strchr(c"+\r\nt.z^-b".as_ptr(), nchar).is_null()
            && (*cap).count0 != 0
            && (*cap).count0 as linenr_T != (*win).w_cursor.lnum
        {
            setpcmark();
            (*win).w_cursor.lnum =
                ((*cap).count0 as linenr_T).min((*curbuf.get()).b_ml.ml_line_count);
            check_cursor_col(win);
        }

        // Where the cursor line ends up, and whether the cursor also moves to
        // the first non-blank of it. `None` means the key did its own work.
        let place: Option<(Place, bool)> = match nchar {
            // The three keys that are not bytes.
            K_KENTER => Some((Place::Top, true)),
            K_LEFT => {
                scroll_sideways(cap, false);
                None
            }
            K_RIGHT => {
                scroll_sideways(cap, true);
                None
            }
            _ => match u8::try_from(nchar) {
                Ok(b'\r' | b'\n') => Some((Place::Top, true)),
                Ok(b'+') => {
                    // Without a count, `z+` starts from the line below the
                    // window rather than from the cursor.
                    if (*cap).count0 == 0 {
                        validate_botline_win(win);
                        (*win).w_cursor.lnum =
                            (*win).w_botline.min((*curbuf.get()).b_ml.ml_line_count);
                    }
                    Some((Place::Top, true))
                }
                Ok(b't') => Some((Place::Top, false)),
                Ok(b'.') => Some((Place::Middle, true)),
                Ok(b'z') => Some((Place::Middle, false)),
                Ok(b'^') => {
                    // `z^` positions the line *above* the window, so with no
                    // count it first scrolls the current top out of sight.
                    if (*cap).count0 != 0 {
                        scroll_cursor_bot(win, 0, true);
                        (*win).w_cursor.lnum = (*win).w_topline;
                    } else if (*win).w_topline == 1 {
                        (*win).w_cursor.lnum = 1;
                    } else {
                        (*win).w_cursor.lnum = (*win).w_topline - 1;
                    }
                    Some((Place::Bottom, true))
                }
                Ok(b'-') => Some((Place::Bottom, true)),
                Ok(b'b') => Some((Place::Bottom, false)),
                // `zH`/`zL` scroll half a screen each, so the count and the
                // width multiply -- and the count is the user's. The C wraps;
                // `set_leftcol` clamps whatever comes out.
                Ok(b'H') => {
                    (*cap).count1 = (*cap).count1.wrapping_mul((*win).w_view_width / 2);
                    scroll_sideways(cap, false);
                    None
                }
                Ok(b'h') => {
                    scroll_sideways(cap, false);
                    None
                }
                Ok(b'L') => {
                    (*cap).count1 = (*cap).count1.wrapping_mul((*win).w_view_width / 2);
                    scroll_sideways(cap, true);
                    None
                }
                Ok(b'l') => {
                    scroll_sideways(cap, true);
                    None
                }
                Ok(b's') => {
                    scroll_cursor_to_edge(true);
                    None
                }
                Ok(b'e') => {
                    scroll_cursor_to_edge(false);
                    None
                }
                // `zp`/`zP`: put a blockwise register without widening the
                // lines it lands on.
                Ok(b'P' | b'p') => {
                    nv_put(cap);
                    None
                }
                // `zy`: yank without a trailing newline.
                Ok(b'y') => {
                    nv_operator(cap);
                    None
                }
                // `zg`/`zG`/`zw`/`zW`/`zu…`: the spellfile.
                Ok(b'u' | b'g' | b'w' | b'G' | b'W') => {
                    if nv_zg_zw(cap, nchar) == FAIL {
                        return;
                    }
                    None
                }
                // `z=`: suggest corrections.
                Ok(b'=') => {
                    if !checkclearop((*cap).oap) {
                        spell_suggest((*cap).count0);
                    }
                    None
                }
                _ => {
                    if !nv_zet_fold(cap, nchar, &mut old_fdl) {
                        clearopbeep((*cap).oap);
                    }
                    None
                }
            },
        };

        if let Some((place, to_first_non_blank)) = place {
            if to_first_non_blank {
                beginline(BL_WHITE as c_int | BL_FIX as c_int);
            }
            match place {
                Place::Top => scroll_cursor_top(win, 0, true_0),
                Place::Middle => scroll_cursor_halfway(win, true, false),
                Place::Bottom => scroll_cursor_bot(win, 0, true),
            }
            redraw_later(win, UPD_VALID);
            set_fraction(win);
        }

        if old_fen != (*win).w_onebuf_opt.wo_fen {
            // Windows bound by 'scrollbind' in diff mode have to fold alike,
            // or the same line is at a different height in each.
            if foldmethodIsDiff(win) && (*win).w_onebuf_opt.wo_scb != 0 {
                let mut wp = firstwin.get();
                while !wp.is_null() {
                    if wp != win && foldmethodIsDiff(wp) && (*wp).w_onebuf_opt.wo_scb != 0 {
                        (*wp).w_onebuf_opt.wo_fen = (*win).w_onebuf_opt.wo_fen;
                        changed_window_setting(wp);
                    }
                    wp = (*wp).w_next;
                }
            }
            changed_window_setting(win);
        }
        if old_fdl as OptInt != (*win).w_onebuf_opt.wo_fdl {
            newFoldLevel();
        }
    }
}
