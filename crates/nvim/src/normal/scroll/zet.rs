//! The `z` prefix tree: positioning the view, scrolling sideways, the fold
//! commands and the spellfile additions.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::keycodes::SpecialKey;
use crate::narrow::number_as_int;
use crate::spell::{SMT_ALL, SPELL_ADD_BAD, SPELL_ADD_GOOD, SpellAddType};
use crate::strings::has_char;
use crate::types::Key;
use crate::winlayer::{Buf, Win, windows};

use crate::ascii::ascii_isdigit;
use crate::cursor::{check_cursor_col, set_leftcol};
use crate::drawscreen::{UPD_NOT_VALID, UPD_VALID, redraw_later};
use crate::edit::{BeginlineOpts, beginline};
use crate::fold::{
    clear_folding, close_fold, close_fold_recurse, deepest_fold_nesting, delete_fold,
    fold_manual_allowed, fold_move_to, fold_open_cursor, foldmethod_is_diff, foldmethod_is_manual,
    foldmethod_is_marker, new_fold_level, open_fold, open_fold_recurse,
};
use crate::guard::Suppress;
use crate::mark::setpcmark;
use crate::memory::XString;
use crate::message::emsg;
use crate::normal::defs::CmdArg;
use crate::normal::{
    FIND_IDENT, INT_MAX, check_clear_op, clear_op_beep, get_visual_text, ident_under_cursor,
    nv_operator, nv_put, read_command_char, visual_active,
};
use crate::ops::OpType;
use crate::option::get_sidescrolloff_value;
use crate::os::cshim::gettext;
use crate::os::libc::int64_t;
use crate::spellfile::spell_add_word;
use crate::spellsuggest::spell_suggest;
use crate::state::mode::FINISH_OP;
use crate::types::{ColNr, Failed, LineNr, OptInt};
use crate::window::{set_fraction, win_setheight};
use core::ffi::c_int;

use crate::r#move::{
    changed_window_setting, scroll_cursor_bot, scroll_cursor_top, validate_botline_win,
};
use crate::search::{BACKWARD, FORWARD};

/// Where the positioning commands put the cursor's line.
#[derive(Copy, Clone, Debug)]
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
pub(crate) fn nv_z_get_count(cmd_arg: &mut CmdArg, nchar_arg: &mut Key) -> bool {
    if check_clear_op(cmd_arg.op()) {
        return false;
    }
    let mut n = c_int::from(nchar_arg.digit().unwrap_or(0));
    loop {
        let nchar = read_command_char();
        if nchar == SpecialKey::Del || nchar == SpecialKey::Kdel {
            // Rubbing out a digit.
            n /= 10;
        } else if let Some(digit) = nchar.digit() {
            if crate::math::vim_append_digit_int(&mut n, c_int::from(digit)) {
                continue;
            }
            clear_op_beep(cmd_arg.op());
            break;
        } else if nchar == Key::CAR {
            win_setheight(n);
            break;
        } else if nchar.is('l')
            || nchar.is('h')
            || nchar == SpecialKey::Left
            || nchar == SpecialKey::Right
        {
            // Both counts came from the user, so this can overflow -- the
            // C wraps, and `set_leftcol` clamps whatever comes out.
            if n != 0 {
                cmd_arg.count1 = n.wrapping_mul(cmd_arg.count1);
            }
            *nchar_arg = nchar;
            return true;
        } else {
            clear_op_beep(cmd_arg.op());
            break;
        }
    }
    cmd_arg.op().op_type = OpType::Nop;
    false
}

/// `zg`, `zG`, `zw`, `zW` and the `zug` family: add the word under the cursor
/// to the spellfile as good or as bad, or take it back out again.
///
/// Answers `Err` when there was no word to act on, which stops `nv_zet`
/// running its tail.
pub(crate) fn nv_zg_zw(cmd_arg: &mut CmdArg, mut nchar: Key) -> Result<(), Failed> {
    // `zu` is the undo prefix: `zug` takes back what `zg` added.
    let mut undo = false;
    if nchar.is('u') {
        nchar = read_command_char();
        if !has_char(c"gGwW", nchar.code()) {
            clear_op_beep(cmd_arg.op());
            return Ok(());
        }
        undo = true;
    }
    if check_clear_op(cmd_arg.op()) {
        return Ok(());
    }

    // Three ways to find the word, in order: the selection, the
    // misspelling the cursor is inside, and the identifier under it.
    // The word to add, copied out of the buffer: `spell_add_word` rewrites
    // a word list and reloads it, which is not something to hold a line
    // pointer across.
    let mut word: Option<XString> = None;
    if visual_active() {
        let Some(text) = get_visual_text(Some(cmd_arg)) else {
            return Err(Failed);
        };
        word = Some(XString::from_bytes(text.selected()));
    }
    if word.is_none() {
        let pos = Win::current().w_cursor;
        // The search is only being used to find where the bad word
        // starts; its "no more misspellings" message is not wanted.
        let no_emsg = Suppress::emsg();
        let bad_len = Win::current().spell_move_to(FORWARD as c_int, SMT_ALL, true, None);
        drop(no_emsg);
        // Only if it found one at or before the cursor, i.e. the one the
        // cursor is inside rather than the next one.
        if bad_len != 0 && Win::current().w_cursor.col <= pos.col {
            let win = Win::current();
            let at = usize::try_from(win.w_cursor.col).unwrap_or(0);
            let mut lines = win.buffer().lines();
            let line = lines.line(win.w_cursor.lnum);
            let end = at.saturating_add(bad_len).min(line.len());
            word = Some(XString::from_bytes(&line[at.min(end)..end]));
        }
        Win::current().w_cursor = pos;
    }
    let word = match word {
        Some(word) => word,
        None => {
            let found = ident_under_cursor(FIND_IDENT);
            if found.is_empty() {
                return Err(Failed);
            }
            XString::from_vec(found)
        }
    };

    // Lower case adds to the file 'spellfile' names, upper case to the
    // internal list that lasts for this session only -- which is why the
    // upper-case forms pass no index.
    let what = if nchar.is('w') || nchar.is('W') {
        SPELL_ADD_BAD as SpellAddType
    } else {
        SPELL_ADD_GOOD as SpellAddType
    };
    let index = if nchar.is('G') || nchar.is('W') {
        0
    } else {
        cmd_arg.count1
    };
    spell_add_word(&word, what, index, undo);
    Ok(())
}

/// Scroll sideways by `count1` columns, which 'wrap' makes meaningless.
fn scroll_sideways(cmd_arg: &mut CmdArg, right: bool) {
    let win = Win::current();
    if win.w_onebuf_opt.wo_wrap != 0 {
        return;
    }
    let target = if right {
        win.w_leftcol + cmd_arg.count1
    } else {
        (win.w_leftcol - cmd_arg.count1).max(ColNr::ZERO)
    };
    set_leftcol(target);
}

/// `zs` and `ze`: scroll sideways until the cursor is at the left or right
/// edge, keeping 'sidescrolloff' columns of context.
fn scroll_cursor_to_edge(to_left: bool) {
    let mut win = Win::current();
    if win.w_onebuf_opt.wo_wrap != 0 {
        return;
    }
    let siso = get_sidescrolloff_value(win);
    let mut col: ColNr = ColNr::ZERO;
    // A closed fold shows one line of its own, which starts at column 0.
    if !folded(win.w_cursor.lnum) {
        let (start, end) = win.vcol_span(win.cursor());
        col = if to_left { start } else { end };
    }
    if to_left {
        col = if i64::from(col) > siso {
            // Below `col`, so it fits.
            col - number_as_int(siso)
        } else {
            ColNr::ZERO
        };
    } else {
        let width = win.w_view_width - win.col_off();
        let width = int64_t::from(width);
        col = if (i64::from(col) + siso) < width {
            ColNr::ZERO
        } else if (siso - width) < i64::from(INT_MAX - col.get()) {
            // Bounded by `INT_MAX` just above.
            ColNr::new(number_as_int(i64::from(col) + siso - width + 1))
        } else {
            ColNr::new(INT_MAX)
        };
    }
    if win.w_leftcol != col {
        win.w_leftcol = col;
        redraw_later(win, UPD_NOT_VALID);
    }
}

/// The fold half of the `z` tree. Answers whether the key was one of them.
fn nv_zet_fold(cmd_arg: &mut CmdArg, nchar: Key, old_fdl: &mut Option<OptInt>) -> bool {
    let mut win = Win::current();
    // Whether the cursor is inside a fold, which is what decides between
    // opening and closing for the toggles.
    let in_fold = || folded(win.w_cursor.lnum);
    match nchar.as_byte() {
        // `zf`/`zF`: create a fold. `zF` folds `count1` lines, which is
        // the operator applied to itself.
        Some(b'F' | b'f') => {
            if fold_manual_allowed(true).is_ok() {
                cmd_arg.nchar = Key::ascii(b'f');
                nv_operator(cmd_arg);
                win.w_onebuf_opt.wo_fen = 1;
                if nchar.is('F') && cmd_arg.op().op_type == OpType::Fold {
                    nv_operator(cmd_arg);
                    FINISH_OP.set(true);
                }
            } else {
                clear_op_beep(cmd_arg.op());
            }
        }
        // `zd`/`zD`: delete a fold, recursively for `zD`.
        Some(b'd' | b'D') => {
            if fold_manual_allowed(false).is_ok() {
                if visual_active() {
                    nv_operator(cmd_arg);
                } else {
                    let lnum = win.w_cursor.lnum;
                    let deep = c_int::from(nchar.is('D'));
                    delete_fold(win, lnum, lnum, deep, false);
                }
            }
        }
        // `zE`: delete every fold.
        Some(b'E') => {
            if foldmethod_is_manual(win) {
                clear_folding(win);
                changed_window_setting(win);
            } else if foldmethod_is_marker(win) {
                delete_fold(
                    win,
                    LineNr::ONE,
                    Buf::current().b_ml.ml_line_count,
                    1,
                    false,
                );
            } else {
                let msg = c"E352: Cannot erase folds with current 'foldmethod'";
                emsg(gettext(msg));
            }
        }
        // `zn`/`zN`/`zi`: 'foldenable' off, on, toggled.
        Some(b'n') => win.w_onebuf_opt.wo_fen = 0,
        Some(b'N') => win.w_onebuf_opt.wo_fen = 1,
        Some(b'i') => win.w_onebuf_opt.wo_fen = c_int::from(win.w_onebuf_opt.wo_fen == 0),
        // `za`/`zA`: toggle this fold, recursively for `zA`.
        Some(b'a') => {
            if in_fold() {
                open_fold(win.w_cursor, cmd_arg.count1);
            } else {
                close_fold(win.w_cursor, cmd_arg.count1);
                win.w_onebuf_opt.wo_fen = 1;
            }
        }
        Some(b'A') => {
            if in_fold() {
                open_fold_recurse(win.w_cursor);
            } else {
                close_fold_recurse(win.w_cursor);
                win.w_onebuf_opt.wo_fen = 1;
            }
        }
        // `zo`/`zO`: open. With a selection they are the operator form.
        Some(b'o') => {
            if visual_active() {
                nv_operator(cmd_arg);
            } else {
                open_fold(win.w_cursor, cmd_arg.count1);
            }
        }
        Some(b'O') => {
            if visual_active() {
                nv_operator(cmd_arg);
            } else {
                open_fold_recurse(win.w_cursor);
            }
        }
        // `zc`/`zC`: close. Closing always turns 'foldenable' back on --
        // there would be nothing to see otherwise.
        Some(b'c') => {
            if visual_active() {
                nv_operator(cmd_arg);
            } else {
                close_fold(win.w_cursor, cmd_arg.count1);
            }
            win.w_onebuf_opt.wo_fen = 1;
        }
        Some(b'C') => {
            if visual_active() {
                nv_operator(cmd_arg);
            } else {
                close_fold_recurse(win.w_cursor);
            }
            win.w_onebuf_opt.wo_fen = 1;
        }
        // `zv`: open just enough to see the cursor line.
        Some(b'v') => fold_open_cursor(),
        // `zx`/`zX`: recompute the folds. `zx` also reopens to the cursor.
        Some(b'x') => {
            win.w_onebuf_opt.wo_fen = 1;
            win.w_foldinvalid = true;
            new_fold_level();
            fold_open_cursor();
        }
        Some(b'X') => {
            win.w_onebuf_opt.wo_fen = 1;
            win.w_foldinvalid = true;
            // Force the tail's `new_fold_level`.
            *old_fdl = None;
        }
        // `zm`/`zM`: fold more, or all the way.
        Some(b'm') => {
            if win.w_onebuf_opt.wo_fdl > 0 {
                win.w_onebuf_opt.wo_fdl -= OptInt::from(cmd_arg.count1);
                win.w_onebuf_opt.wo_fdl = win.w_onebuf_opt.wo_fdl.max(0);
            }
            *old_fdl = None;
            win.w_onebuf_opt.wo_fen = 1;
        }
        Some(b'M') => {
            win.w_onebuf_opt.wo_fdl = 0;
            *old_fdl = None;
            win.w_onebuf_opt.wo_fen = 1;
        }
        // `zr`/`zR`: reduce the folding, or open everything.
        Some(b'r') => {
            win.w_onebuf_opt.wo_fdl += OptInt::from(cmd_arg.count1);
            let deepest = deepest_fold_nesting(win);
            win.w_onebuf_opt.wo_fdl = win.w_onebuf_opt.wo_fdl.min(OptInt::from(deepest));
        }
        Some(b'R') => {
            win.w_onebuf_opt.wo_fdl = OptInt::from(deepest_fold_nesting(win));
            *old_fdl = None;
        }
        // `zj`/`zk`: to the next or previous fold's edge.
        Some(b'j' | b'k') => {
            let dir = if nchar.is('j') {
                FORWARD as c_int
            } else {
                BACKWARD as c_int
            };
            if !fold_move_to(true, dir, cmd_arg.count1) {
                clear_op_beep(cmd_arg.op());
            }
        }
        _ => return false,
    }
    true
}

/// `z`, whose second character says what part of the view or of the folding
/// it is about.
pub(crate) fn nv_zet(cmd_arg: &mut CmdArg) {
    let mut win = Win::current();
    let mut nchar = cmd_arg.nchar;
    // The 'foldlevel' to compare against at the end; `None` when a command
    // set it and the folds must be updated regardless.
    let mut old_fdl = Some(win.w_onebuf_opt.wo_fdl);
    let old_fen = win.w_onebuf_opt.wo_fen;

    // `z` may take a count of its own between the `z` and the command.
    if ascii_isdigit(nchar.code()) && !nv_z_get_count(cmd_arg, &mut nchar) {
        return;
    }
    // The commands that are operators or motions of their own answer for
    // a pending operator themselves; the rest refuse one.
    if !cmd_arg.nchar.is('f')
        && !cmd_arg.nchar.is('F')
        && !(visual_active() && has_char(c"dcCoO", cmd_arg.nchar.code()))
        && !cmd_arg.nchar.is('j')
        && !cmd_arg.nchar.is('k')
        && check_clear_op(cmd_arg.op())
    {
        return;
    }
    // For the positioning commands a count names the line to position,
    // not how many of anything.
    if has_char(c"+\r\nt.z^-b", nchar.code())
        && cmd_arg.count0 != 0
        && LineNr::new(cmd_arg.count0) != win.w_cursor.lnum
    {
        setpcmark();
        let count0 = LineNr::new(cmd_arg.count0);
        win.w_cursor.lnum = count0.min(Buf::current().b_ml.ml_line_count);
        check_cursor_col(win);
    }

    // Where the cursor line ends up, and whether the cursor also moves to
    // the first non-blank of it. `None` means the key did its own work.
    let place: Option<(Place, bool)> = match nchar.special() {
        // The three keys that are not bytes.
        Some(SpecialKey::Kenter) => Some((Place::Top, true)),
        Some(SpecialKey::Left) => {
            scroll_sideways(cmd_arg, false);
            None
        }
        Some(SpecialKey::Right) => {
            scroll_sideways(cmd_arg, true);
            None
        }
        _ => match nchar.as_byte() {
            Some(b'\r' | b'\n') => Some((Place::Top, true)),
            Some(b'+') => {
                // Without a count, `z+` starts from the line below the
                // window rather than from the cursor.
                if cmd_arg.count0 == 0 {
                    validate_botline_win(win);
                    win.w_cursor.lnum = win.w_botline.min(Buf::current().b_ml.ml_line_count);
                }
                Some((Place::Top, true))
            }
            Some(b't') => Some((Place::Top, false)),
            Some(b'.') => Some((Place::Middle, true)),
            Some(b'z') => Some((Place::Middle, false)),
            Some(b'^') => {
                // `z^` positions the line *above* the window, so with no
                // count it first scrolls the current top out of sight.
                if cmd_arg.count0 != 0 {
                    scroll_cursor_bot(win, 0, true);
                    win.w_cursor.lnum = win.w_topline;
                } else if win.w_topline == LineNr::ONE {
                    win.w_cursor.lnum = LineNr::ONE;
                } else {
                    win.w_cursor.lnum = win.w_topline - 1;
                }
                Some((Place::Bottom, true))
            }
            Some(b'-') => Some((Place::Bottom, true)),
            Some(b'b') => Some((Place::Bottom, false)),
            // `zH`/`zL` scroll half a screen each, so the count and the
            // width multiply -- and the count is the user's. The C wraps;
            // `set_leftcol` clamps whatever comes out.
            Some(b'H') => {
                cmd_arg.count1 = cmd_arg.count1.wrapping_mul(win.w_view_width / 2);
                scroll_sideways(cmd_arg, false);
                None
            }
            Some(b'h') => {
                scroll_sideways(cmd_arg, false);
                None
            }
            Some(b'L') => {
                cmd_arg.count1 = cmd_arg.count1.wrapping_mul(win.w_view_width / 2);
                scroll_sideways(cmd_arg, true);
                None
            }
            Some(b'l') => {
                scroll_sideways(cmd_arg, true);
                None
            }
            Some(b's') => {
                scroll_cursor_to_edge(true);
                None
            }
            Some(b'e') => {
                scroll_cursor_to_edge(false);
                None
            }
            // `zp`/`zP`: put a blockwise register without widening the
            // lines it lands on.
            Some(b'P' | b'p') => {
                nv_put(cmd_arg);
                None
            }
            // `zy`: yank without a trailing newline.
            Some(b'y') => {
                nv_operator(cmd_arg);
                None
            }
            // `zg`/`zG`/`zw`/`zW`/`zu…`: the spellfile.
            Some(b'u' | b'g' | b'w' | b'G' | b'W') => {
                if nv_zg_zw(cmd_arg, nchar).is_err() {
                    return;
                }
                None
            }
            // `z=`: suggest corrections.
            Some(b'=') => {
                if !check_clear_op(cmd_arg.op()) {
                    spell_suggest(cmd_arg.count0);
                }
                None
            }
            _ => {
                if !nv_zet_fold(cmd_arg, nchar, &mut old_fdl) {
                    clear_op_beep(cmd_arg.op());
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
            Place::Middle => win.scroll_cursor_halfway(true, false),
            Place::Bottom => scroll_cursor_bot(win, 0, true),
        }
        redraw_later(win, UPD_VALID);
        set_fraction(win);
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
    if old_fdl != Some(win.w_onebuf_opt.wo_fdl) {
        new_fold_level();
    }
}

/// Whether `lnum` is inside a closed fold of the current window.
fn folded(lnum: LineNr) -> bool {
    Win::current().has_folding(lnum, None, None)
}
