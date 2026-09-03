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
//! and replays the result through `ins_bytes_len`, so the replace stack stays
//! consistent.
//!
//! [`ins_eol`] is CR/NL: `open_line` does the work, but the replace stack and
//! 'formatoptions' have to be told first.  [`ins_shift`] is `i_CTRL-T` and
//! `i_CTRL-D`, which add or remove one 'shiftwidth'.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int};

use super::*;
use crate::option::cpo_has;
use crate::types::{CpoFlag, FoFlag, NUL};

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
    if c == Ctrl_D && (lastc == '0' as c_int || lastc == '^' as c_int) && cur_win().w_cursor.col > 0
    {
        cur_win().w_cursor.col -= 1;
        // SAFETY: every `unsafe` call in this function is an editor-wide
        // routine whose only precondition is the live `curwin`/`curbuf`
        // Insert mode runs with.
        let _ = unsafe { del_char(false) };
        if State.get() & REPLACE_FLAG != 0 {
            replace_pop_ins();
        }
        if lastc == '^' as c_int {
            old_indent.set(get_indent()); // remember the indent
        }
        unsafe { change_indent(INDENT_SET, 0, 1, true) };
    } else {
        let dir = if c == Ctrl_D { INDENT_DEC } else { INDENT_INC };
        unsafe { change_indent(dir, 0, 1, true) };
    }

    // SAFETY: the cursor's line is NUL-terminated, so `skipwhite` stops at
    // its NUL and the byte it answers is still in the line.
    if did_ai.get() && unsafe { *skipwhite(get_cursor_line_ptr()) } as c_int != NUL {
        did_ai.set(false);
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
    if Insstart_blank_vcol.get() == MAXCOL as colnr_T
        && cur_win().w_cursor.lnum == Insstart.get().lnum
    {
        Insstart_blank_vcol.set(nolist_virtcol());
    }
    if echeck_abbr(TAB + ABBR_OFF) {
        return false;
    }

    let ind = unsafe { inindent(0) };
    if ind {
        can_cindent.set(false);
    }

    // 'smarttab' only does something in the indent, and only when
    // 'tabstop' differs from 'shiftwidth' -- which is what these three
    // 'vartabstop' cases are asking.
    let smart_tab = p_sta.get() != 0
        && ind
        && (tabstops(cur_buf().b_p_vts_array) > 1
            || (tabstops(cur_buf().b_p_vts_array) == 1
                && unsafe { tabstop_first(cur_buf().b_p_vts_array) } != sw_value())
            || (tabstops(cur_buf().b_p_vts_array) == 0
                && cur_buf().b_p_ts != sw_value() as OptInt));
    let soft_tab = tabstops(cur_buf().b_p_vsts_array) != 0 || sts_value() != 0;
    if cur_buf().b_p_et == 0 && !smart_tab && !soft_tab {
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
    // SAFETY: a static one-byte string.
    unsafe { append_to_redobuff(c"\t".as_ptr()) };

    // How many columns to the next stop, from whichever option owns it.
    let mut temp = if p_sta.get() != 0 && ind {
        // A tab in the indent uses 'shiftwidth'.
        let sw = sw_value();
        sw - nolist_virtcol() % sw
    } else if tabstops(cur_buf().b_p_vsts_array) > 0 || cur_buf().b_p_sts != 0 {
        let sts = sts_value() as OptInt;
        // SAFETY: a live buffer's own 'vartabstop' array.
        unsafe { tabstop_padding(nolist_virtcol(), sts, cur_buf().b_p_vsts_array) }
    } else {
        let ts = cur_buf().b_p_ts;
        // SAFETY: a live buffer's own 'vartabstop' array.
        unsafe { tabstop_padding(nolist_virtcol(), ts, cur_buf().b_p_vts_array) }
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
            unsafe { ins_str(c" ".as_ptr().cast_mut(), 1) };
            if State.get() & REPLACE_FLAG != 0 {
                unsafe { replace_push_nul() }; // no character was replaced
            }
        }
        temp -= 1;
    }

    // With 'expandtab' off, put TABs back where the spaces will do.
    if cur_buf().b_p_et == 0
        && (tabstops(cur_buf().b_p_vsts_array) > 0 || sts_value() > 0 || (p_sta.get() != 0 && ind))
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
/// delete the spaces the TABs made redundant, which is the `memmove` that
/// rebuilds the line.
///
/// In `MODE_VREPLACE` all of it happens on a copy (`saved_line`), and the
/// result is replayed with `backspace_until_column` + `ins_bytes_len` so the
/// replace stack sees an ordinary edit.
fn tab_spaces_to_tabs() {
    let mut ptr: *mut c_char;
    let mut saved_line: *mut c_char = ::core::ptr::null_mut();
    let mut pos = pos_T::default();
    // In `MODE_VREPLACE` the cursor that moves is the copy in `pos`; the
    // real one must not move until the change is replayed.  Nothing holds a
    // long-lived reference to it: the editor writes `curwin`'s own cursor
    // from under this function, so it is read and written a field at a time.
    let vreplace = State.get() & VREPLACE_FLAG != 0;
    let mut change_col = -1;
    let save_list = cur_win().w_onebuf_opt.wo_list;

    // Get the current line.  In `MODE_VREPLACE` no real change may
    // happen yet, so work on a copy.
    // SAFETY: `ptr` addresses the cursor's own column of a NUL-terminated
    // line -- the buffer's, or the copy of it `saved_line` owns -- and every
    // walk below stays between that line's start and its NUL.
    if vreplace {
        pos = cur_win().w_cursor;
        let col = pos.col as isize;
        let len = get_cursor_line_len() as size_t;
        saved_line = unsafe { xstrnsave(get_cursor_line_ptr(), len) };
        ptr = unsafe { saved_line.offset(col) };
    } else {
        ptr = get_cursor_pos_ptr();
    }

    // 'list' changes what a TAB is worth; unless 'cpoptions' has `L`, it
    // must not be allowed to.
    if !cpo_has(CpoFlag::LISTWM) {
        cur_win().w_onebuf_opt.wo_list = 0;
    }

    // Find the first white character of the run.
    let mut fpos = cur_win().w_cursor;
    while fpos.col > 0 && ascii_iswhite(unsafe { *ptr.offset(-1) } as c_int) {
        fpos.col -= 1;
        ptr = unsafe { ptr.offset(-1) };
    }
    // In Replace mode the run must not reach back before the insert.
    if State.get() & REPLACE_FLAG != 0
        && fpos.lnum == Insstart.get().lnum
        && fpos.col < Insstart.get().col
    {
        ptr = unsafe { ptr.offset((Insstart.get().col - fpos.col) as isize) };
        fpos.col = Insstart.get().col;
    }

    let mut vcol: colnr_T = 0;
    let mut want_vcol: colnr_T = 0;
    let none = ::core::ptr::null_mut();
    // SAFETY: `fpos` and `cursor` are live positions in the current buffer.
    unsafe { getvcol(cur_win(), &raw mut fpos, &raw mut vcol, none, none) };
    let cursor: *mut pos_T = if vreplace {
        &raw mut pos
    } else {
        &raw mut cur_win().w_cursor
    };
    unsafe { getvcol(cur_win(), cursor, &raw mut want_vcol, none, none) };

    // Use as many TABs as possible, measuring each one's width where it
    // lands.
    let tab = c"\t".as_ptr().cast_mut();
    // SAFETY: `tab` is a static one-byte string, and the character widths
    // are asked of a live window.
    let tab_v = unsafe { *tab } as uint8_t as int32_t;
    let mut csarg = CharsizeArg::default();
    let mut cstype = unsafe { init_charsize_arg(&mut csarg, cur_win(), 0, tab) };
    loop {
        let byte = unsafe { *ptr } as c_int;
        if !ascii_iswhite(byte) {
            break;
        }
        let i = unsafe { win_charsize(cstype, vcol, tab, tab_v, &mut csarg) }.width;
        if vcol + i > want_vcol {
            break;
        }
        if byte != TAB {
            unsafe { *ptr = TAB as c_char };
            if change_col < 0 {
                change_col = fpos.col; // remember the first changed column
                if fpos.lnum == Insstart.get().lnum && fpos.col < Insstart.get().col {
                    Insstart.set(Insstart.get().with_col(fpos.col));
                }
            }
        }
        fpos.col += 1;
        ptr = unsafe { ptr.offset(1) };
        vcol += i;
    }

    if change_col >= 0 {
        // Skip over the spaces the TABs have made redundant.
        let mut repl_off = 0;
        cstype = unsafe { init_charsize_arg(&mut csarg, cur_win(), 0, ptr) };
        while vcol < want_vcol && unsafe { *ptr } as c_int == ' ' as c_int {
            vcol += unsafe { win_charsize(cstype, vcol, ptr, b' ' as int32_t, &mut csarg) }.width;
            ptr = unsafe { ptr.offset(1) };
            repl_off += 1;
        }
        if vcol > want_vcol {
            ptr = unsafe { ptr.offset(-1) };
            repl_off -= 1;
        }
        fpos.col += repl_off;

        // Delete the spaces between `fpos` and the cursor.
        let i = walk_col(&pos, vreplace) - fpos.col;
        if i > 0 {
            if State.get() & VREPLACE_FLAG == 0 {
                // Rebuild the line without them.
                let newp_len = cur_buf().b_ml.cached_len() - i;
                // SAFETY: `newp` is `newp_len` bytes, `col` is how far
                // `ptr` is into the line, and `i` is the run of spaces being
                // dropped -- so the head is `col` bytes and the tail the rest
                // of the line, and the two together fit.
                let newp = unsafe { xmalloc(newp_len as size_t) } as *mut c_char;
                let col = unsafe { ptr.offset_from(cur_buf().b_ml.cached_text()) };
                if col > 0 {
                    let head = unsafe { ptr.offset(-col) };
                    unsafe { newp.cast::<u8>().copy_from(head.cast(), col as size_t) };
                }
                let tail = unsafe { ptr.offset(i as isize) };
                let tail_len = (newp_len as ptrdiff_t - col) as size_t;
                let into = unsafe { newp.offset(col) }.cast::<u8>();
                unsafe { into.copy_from(tail.cast(), tail_len) };
                if let Some(old) = cur_buf().b_ml.swap_cached_text(newp, newp_len) {
                    unsafe { xfree(old.cast()) };
                }
                cur_buf().b_ml.line_was_replaced();
                let old_len = walk_col(&pos, vreplace) - change_col;
                let new_len = fpos.col - change_col;
                unsafe { inserted_bytes(fpos.lnum, change_col, old_len, new_len) };
            } else {
                // SAFETY: the tail is NUL-terminated and moves down over the
                // `i` spaces in front of it, terminator included.
                let tail = unsafe { ptr.offset(i as isize) };
                let tail_len = unsafe { cstr::bytes_at(tail) }.len() + 1;
                unsafe { ptr.cast::<u8>().copy_from(tail.cast(), tail_len) };
            }

            // Each deleted space had an entry on the replace stack.
            if State.get() & REPLACE_FLAG != 0 && State.get() & VREPLACE_FLAG == 0 {
                for _ in 0..i {
                    replace_join(repl_off);
                }
            }
        }
        if vreplace {
            pos.col -= i;
        } else {
            cur_win().w_cursor.col -= i;
        }

        // In `MODE_VREPLACE` the change was made to the copy; replay it
        // onto the real line.
        if State.get() & VREPLACE_FLAG != 0 {
            unsafe { backspace_until_column(change_col) };
            // SAFETY: `saved_line` holds the line as it now stands, and the
            // run from `change_col` to the cursor is inside it.
            let from = unsafe { saved_line.offset(change_col as isize) };
            let len = (walk_col(&pos, vreplace) - change_col) as size_t;
            unsafe { ins_bytes_len(from, len) };
        }
    }

    if vreplace {
        unsafe { xfree(saved_line.cast()) };
    }
    cur_win().w_onebuf_opt.wo_list = save_list;
}

/// Handle CR or NL in Insert mode.
///
/// Answers false when undo could not be saved -- but *true* when an
/// abbreviation swallowed the key, which is not the same thing.
pub(crate) fn ins_eol(c: c_int) -> bool {
    // SAFETY: every `unsafe` call in this function is an editor-wide routine
    // whose only precondition is the live `curwin`/`curbuf` Insert mode runs
    // with.
    if echeck_abbr(c + ABBR_OFF) {
        return true;
    }
    if stop_arrow_failed() {
        return false;
    }
    unsafe { undisplay_dollar() };

    // Strange, but this is what the NL replaces in Replace mode.
    if State.get() & REPLACE_FLAG != 0 && State.get() & VREPLACE_FLAG == 0 {
        unsafe { replace_push_nul() };
    }

    // In 'virtualedit' past the end of the line, make the position real
    // first.
    if virtual_active(cur_win()) && cur_win().w_cursor.coladd > 0 {
        let vcol = unsafe { getviscol() };
        coladvance(cur_win(), vcol);
    }

    // In 'revins' the cursor is at the start of what was typed, and the
    // line is broken at its end.
    if revins_on.get() {
        cur_win().w_cursor.col += get_cursor_pos_len();
    }

    unsafe { append_to_redobuff(NL_STR.as_ptr()) };
    let comments = if has_format_option(FoFlag::RET_COMS) {
        OPENLINE_DO_COM
    } else {
        0
    };
    let indent = old_indent.get();
    let ok = unsafe { open_line(FORWARD, comments, indent, ::core::ptr::null_mut()) };
    old_indent.set(0);
    can_cindent.set(true);
    // The new line may be in a closed fold.
    unsafe { fold_open_cursor() };
    ok
}

/// The column [`tab_spaces_to_tabs`] measures against: the real cursor's, or
/// -- in `MODE_VREPLACE`, where nothing may move yet -- the copy's.
#[inline(always)]
fn walk_col(pos: &pos_T, vreplace: bool) -> colnr_T {
    if vreplace {
        pos.col
    } else {
        cur_win().w_cursor.col
    }
}

/// Could the insert not be ended here?  `stop_arrow` says the change cannot
/// be saved for undo, in which case nothing may be edited.
#[inline(always)]
fn stop_arrow_failed() -> bool {
    // SAFETY: `curbuf` is live for the whole session.
    unsafe { stop_arrow().is_err() }
}

/// The cursor's virtual column, as it would be with 'list' off.
#[inline(always)]
fn nolist_virtcol() -> colnr_T {
    // SAFETY: `curwin` is live for the whole session.
    unsafe { get_nolist_virtcol() }
}

/// The effective 'shiftwidth' of the current buffer.
#[inline(always)]
fn sw_value() -> c_int {
    // SAFETY: `curbuf` is live for the whole session.
    unsafe { get_sw_value(curbuf.get()) }
}

/// The effective 'softtabstop' of the current buffer.
#[inline(always)]
fn sts_value() -> c_int {
    // SAFETY: `curbuf` is live for the whole session.
    unsafe { get_sts_value() }
}

/// How many stops the 'vartabstop'-style array `ts` holds.
#[inline(always)]
fn tabstops(ts: *mut colnr_T) -> c_int {
    // SAFETY: a live buffer's own tab-stop array, or null for none.
    unsafe { tabstop_count(ts) }
}

/// Insert one space at the cursor, replacing a character in Replace mode.
#[inline(always)]
fn insert_space() {
    // SAFETY: `curwin`/`curbuf` are live for the whole session.
    unsafe { ins_char(' ' as c_int) }
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
