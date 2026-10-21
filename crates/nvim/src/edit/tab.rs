//! TAB, CR and the two shift keys -- the characters that change an indent.
//!
//! [`ins_tab`] is TAB.  With 'expandtab' off and neither 'softtabstop' nor
//! 'smarttab' in play it answers `true` and the caller inserts a literal TAB;
//! everything else in the file is the case where it does not.  Then TAB means
//! "advance to the next stop", the stop comes from one of three options, and
//! the padding goes in as *spaces* -- which are turned back into TABs
//! afterwards by [`tab_spaces_to_tabs`], because 'expandtab' is off.
//!
//! That last phase is the awkward one, and it is why the file is not just
//! three short functions: it rewrites the whole white-space run in front of
//! the cursor rather than the characters just inserted, since a TAB can only
//! start at a tab stop and the run may already have begun before this key was
//! pressed.  In `MODE_VREPLACE` it does all of that on a *copy* of the line
//! and replays the result through `ins_bytes`, so the replace stack stays
//! consistent.
//!
//! [`ins_eol`] is CR/NL: `open_line` does the work, but the replace stack and
//! 'formatoptions' have to be told first.  [`ins_shift`] is `i_CTRL-T` and
//! `i_CTRL-D`, which add or remove one 'shiftwidth'.

#![forbid(unsafe_code)]

use crate::charset::skip;
use crate::cstr::byte_at;
use crate::indent::StopList;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;
use crate::option::cpo_has;
use crate::types::{CpoFlag, FoFlag};

/// `i_CTRL-T` and `i_CTRL-D`: add or remove one 'shiftwidth' of indent.
///
/// `lastc` is the character typed before this one: `0 CTRL-D` deletes all
/// indent and `^ CTRL-D` deletes it for this line only, restoring it on the
/// next -- which is what saving `old_indent` is for.
pub(crate) fn ins_shift(c: c_int, lastc: c_int) {
    if stop_arrow_failed() {
        return;
    }
    append_to_redobuff_char(c);

    // `0 CTRL-D` and `^ CTRL-D`: the `0`/`^` was inserted as an ordinary
    // character and has to come off again.
    if c == Ctrl_D
        && (lastc == '0' as c_int || lastc == '^' as c_int)
        && Win::current().w_cursor.col > 0
    {
        Win::current().w_cursor.col -= 1;
        let _ = del_char(false);
        if State.get() & REPLACE_FLAG != 0 {
            replace_pop_ins();
        }
        if lastc == '^' as c_int {
            old_indent.set(get_indent()); // remember the indent
        }
        change_indent(INDENT_SET, 0, 1, true);
    } else {
        let dir = if c == Ctrl_D { INDENT_DEC } else { INDENT_INC };
        change_indent(dir, 0, 1, true);
    }

    if did_ai.get() {
        let mut lines = Lines::current();
        let line = lines.line(Win::current().w_cursor.lnum);
        if skip::white(line) < line.len() {
            did_ai.set(false);
        }
    }
    did_si.set(false);
    can_si.set(false);
    can_si_back.set(false);
    can_cindent.set(false);
}

/// Handle TAB in Insert or Replace mode.
///
/// Answers `true` when the TAB is to be inserted like an ordinary character,
/// which is the common case: 'expandtab' off, 'softtabstop' unset, and either
/// 'smarttab' off or 'tabstop' equal to 'shiftwidth' anyway.
pub(crate) fn ins_tab() -> bool {
    if Insstart_blank_vcol.get() == MAXCOL as ColNr
        && Win::current().w_cursor.lnum == Insstart.get().lnum
    {
        Insstart_blank_vcol.set(nolist_virtcol());
    }
    if echeck_abbr(TAB + ABBR_OFF) {
        return false;
    }

    let ind = inindent(0);
    if ind {
        can_cindent.set(false);
    }

    // 'smarttab' only does something in the indent, and only when
    // 'tabstop' differs from 'shiftwidth' -- which is what these three
    // 'vartabstop' cases are asking.
    let smart_tab = p_sta()
        && ind
        && (tabstops(StopList::Tab) > 1
            || (tabstops(StopList::Tab) == 1 && Buf::current().first_tab_stop() != sw_value())
            || (tabstops(StopList::Tab) == 0 && Buf::current().b_p_ts != sw_value() as OptInt));
    let soft_tab = tabstops(StopList::SoftTab) != 0 || sts_value() != 0;
    if Buf::current().b_p_et == 0 && !smart_tab && !soft_tab {
        // Nothing special: insert TAB like a normal character.
        return true;
    }

    if stop_arrow_failed() {
        return true;
    }

    did_ai.set(false);
    did_si.set(false);
    can_si.set(false);
    can_si_back.set(false);
    append_to_redobuff(b"\t");

    // How many columns to the next stop, from whichever option owns it.
    let mut temp = if p_sta() && ind {
        // A tab in the indent uses 'shiftwidth'.
        let sw = sw_value();
        sw - nolist_virtcol() % sw
    } else if tabstops(StopList::SoftTab) > 0 || Buf::current().b_p_sts != 0 {
        let sts = sts_value() as OptInt;
        Buf::current().stop_padding(StopList::SoftTab, nolist_virtcol(), sts)
    } else {
        let ts = Buf::current().b_p_ts;
        Buf::current().stop_padding(StopList::Tab, nolist_virtcol(), ts)
    };

    // The first space goes in with `ins_char`, which in Replace mode
    // deletes one character; the rest with `ins_str`, which deletes none.
    // In `MODE_VREPLACE` every one goes through `ins_char`.
    insert_space();
    temp -= 1;
    while temp > 0 {
        if State.get() & VREPLACE_FLAG != 0 {
            insert_space();
        } else {
            ins_str(b" ");
            if State.get() & REPLACE_FLAG != 0 {
                replace_push_nul(); // no character was replaced
            }
        }
        temp -= 1;
    }

    // With 'expandtab' off, put TABs back where the spaces will do.
    if Buf::current().b_p_et == 0
        && (tabstops(StopList::SoftTab) > 0 || sts_value() > 0 || (p_sta() && ind))
    {
        tab_spaces_to_tabs();
    }
    false
}

/// Replace the white space in front of the cursor by TABs wherever a TAB
/// reaches the same screen column.
///
/// Three phases.  Walk back to the start of the run; walk forward writing a
/// TAB wherever one fits, remembering the first column changed; and then
/// delete the spaces the TABs made redundant, which rebuilds the line.
///
/// The walk is over a copy of the line, because it measures columns through
/// the memline as it goes. In `MODE_VREPLACE` the copy is all that changes,
/// and the result is replayed with `backspace_until_column` + `ins_bytes` so
/// the replace stack sees an ordinary edit; otherwise the copy goes back into
/// the cached line -- in place, as upstream writes its TABs through the
/// cache, when no space was dropped.
fn tab_spaces_to_tabs() {
    let mut pos = Pos::default();
    // In `MODE_VREPLACE` the cursor that moves is the copy in `pos`; the
    // real one must not move until the change is replayed.  Nothing holds a
    // long-lived reference to it: the editor writes `curwin`'s own cursor
    // from under this function, so it is read and written a field at a time.
    let vreplace = State.get() & VREPLACE_FLAG != 0;
    let mut change_col = -1;
    let save_list = Win::current().w_onebuf_opt.wo_list;
    let lnum = Win::current().w_cursor.lnum;

    let mut line = Lines::current().line(lnum).to_vec();
    if vreplace {
        pos = Win::current().w_cursor;
    }
    // The byte being looked at, which is the cursor's column to begin with.
    let mut at = Win::current().w_cursor.col as usize;

    // 'list' changes what a TAB is worth; unless 'cpoptions' has `L`, it
    // must not be allowed to.
    if !cpo_has(CpoFlag::LISTWM) {
        Win::current().w_onebuf_opt.wo_list = 0;
    }

    // Find the first white character of the run.
    let mut fpos = Win::current().w_cursor;
    while fpos.col > 0 && ascii_iswhite(c_int::from(line[at - 1])) {
        fpos.col -= 1;
        at -= 1;
    }
    // In Replace mode the run must not reach back before the insert.
    if State.get() & REPLACE_FLAG != 0
        && fpos.lnum == Insstart.get().lnum
        && fpos.col < Insstart.get().col
    {
        at += (Insstart.get().col - fpos.col) as usize;
        fpos.col = Insstart.get().col;
    }

    let mut vcol: ColNr = getvcol(Win::current(), &mut fpos).0;
    let want_vcol: ColNr = if vreplace {
        getvcol(Win::current(), &mut pos).0
    } else {
        let win = Win::current();
        win.vcol_triple(win.cursor()).0
    };

    // Use as many TABs as possible, measuring each one's width where it
    // lands.
    let (mut csarg, cstype) = CharsizeArg::new(Win::current(), 0, b"\t");
    loop {
        let byte = c_int::from(byte_at(&line, at));
        if !ascii_iswhite(byte) {
            break;
        }
        let i = win_charsize(cstype, vcol, 0, TAB, &mut csarg).width;
        if vcol + i > want_vcol {
            break;
        }
        if byte != TAB {
            line[at] = TAB as u8;
            if change_col < 0 {
                change_col = fpos.col; // remember the first changed column
                if fpos.lnum == Insstart.get().lnum && fpos.col < Insstart.get().col {
                    Insstart.set(Insstart.get().with_col(fpos.col));
                }
            }
        }
        fpos.col += 1;
        at += 1;
        vcol += i;
    }

    if change_col >= 0 {
        // Skip over the spaces the TABs have made redundant.
        let mut repl_off = 0;
        let base = at;
        let (mut csarg, cstype) = CharsizeArg::new(Win::current(), 0, &line[base..]);
        while vcol < want_vcol && byte_at(&line, at) == b' ' {
            vcol += win_charsize(cstype, vcol, at - base, b' ' as int32_t, &mut csarg).width;
            at += 1;
            repl_off += 1;
        }
        if vcol > want_vcol {
            at -= 1;
            repl_off -= 1;
        }
        fpos.col += repl_off;

        // Delete the spaces between `fpos` and the cursor.
        let i = walk_col(&pos, vreplace) - fpos.col;
        if i > 0 && !vreplace {
            // Rebuild the line without them, its NUL included.
            let mut newp = Vec::with_capacity(line.len() + 1 - i as usize);
            newp.extend_from_slice(&line[..at]);
            newp.extend_from_slice(&line[at + i as usize..]);
            newp.push(0);
            let _ = Lines::current().line(lnum);
            Buf::current().b_ml.swap_cached_text(newp);
            let old_len = walk_col(&pos, vreplace) - change_col;
            let new_len = fpos.col - change_col;
            inserted_bytes(fpos.lnum, change_col, old_len, new_len);
        } else if i > 0 {
            line.drain(at..at + i as usize);
        } else if !vreplace {
            // Only TABs went in: they are written over the spaces in the
            // cached line, which keeps its length.
            let _ = Lines::current().line(lnum);
            let mut buffer = Buf::current();
            let cached = buffer.b_ml.cached_bytes_mut();
            if cached.len() == line.len() {
                cached.copy_from_slice(&line);
            }
        }
        if i > 0 {
            // Each deleted space had an entry on the replace stack.
            if State.get() & REPLACE_FLAG != 0 && !vreplace {
                for _ in 0..i {
                    replace_join(repl_off);
                }
            }
        }
        if vreplace {
            pos.col -= i;
        } else {
            Win::current().w_cursor.col -= i;
        }

        // In `MODE_VREPLACE` the change was made to the copy; replay it
        // onto the real line.
        if vreplace {
            backspace_until_column(change_col);
            let from = change_col as usize;
            let len = (walk_col(&pos, vreplace) - change_col) as usize;
            ins_bytes(&line[from..from + len]);
        }
    }

    Win::current().w_onebuf_opt.wo_list = save_list;
}

/// Handle CR or NL in Insert mode.
///
/// Answers false when undo could not be saved -- but *true* when an
/// abbreviation swallowed the key, which is not the same thing.
pub(crate) fn ins_eol(c: c_int) -> bool {
    if echeck_abbr(c + ABBR_OFF) {
        return true;
    }
    if stop_arrow_failed() {
        return false;
    }
    undisplay_dollar();

    // Strange, but this is what the NL replaces in Replace mode.
    if State.get() & REPLACE_FLAG != 0 && State.get() & VREPLACE_FLAG == 0 {
        replace_push_nul();
    }

    // In 'virtualedit' past the end of the line, make the position real
    // first.
    if virtual_active(Win::current()) && Win::current().w_cursor.coladd > 0 {
        let vcol = getviscol();
        coladvance(Win::current(), vcol);
    }

    // In 'revins' the cursor is at the start of what was typed, and the
    // line is broken at its end.
    if revins_on.get() {
        Win::current().w_cursor.col += get_cursor_pos_len();
    }

    append_to_redobuff(NL_STR.to_bytes());
    let comments = if has_format_option(FoFlag::RET_COMS) {
        OPENLINE_DO_COM
    } else {
        0
    };
    let indent = old_indent.get();
    let ok = open_line(FORWARD, comments, indent, None);
    old_indent.set(0);
    can_cindent.set(true);
    // The new line may be in a closed fold.
    fold_open_cursor();
    ok
}

/// The column [`tab_spaces_to_tabs`] measures against: the real cursor's, or
/// -- in `MODE_VREPLACE`, where nothing may move yet -- the copy's.
#[inline(always)]
fn walk_col(pos: &Pos, vreplace: bool) -> ColNr {
    if vreplace {
        pos.col
    } else {
        Win::current().w_cursor.col
    }
}

/// Could the insert not be ended here?  `stop_arrow` says the change cannot
/// be saved for undo, in which case nothing may be edited.
#[inline(always)]
fn stop_arrow_failed() -> bool {
    stop_arrow().is_err()
}

/// The cursor's virtual column, as it would be with 'list' off.
#[inline(always)]
fn nolist_virtcol() -> ColNr {
    get_nolist_virtcol()
}

/// The effective 'shiftwidth' of the current buffer.
#[inline(always)]
fn sw_value() -> c_int {
    get_sw_value(Buf::current())
}

/// The effective 'softtabstop' of the current buffer.
#[inline(always)]
fn sts_value() -> c_int {
    get_sts_value()
}

/// How many stops the current buffer's `which` list holds.
#[inline(always)]
fn tabstops(which: StopList) -> c_int {
    Buf::current().stop_count(which)
}

/// Insert one space at the cursor, replacing a character in Replace mode.
#[inline(always)]
fn insert_space() {
    ins_char(' ' as c_int)
}
