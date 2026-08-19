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

use core::ffi::{c_char, c_int};

use super::*;
use crate::option::cpo_has;
use crate::types::{CpoFlag, FAIL, FoFlag, NUL};

/// `i_CTRL-T` and `i_CTRL-D`: add or remove one 'shiftwidth' of indent.
///
/// `lastc` is the character typed before this one: `0 CTRL-D` deletes all
/// indent and `^ CTRL-D` deletes it for this line only, restoring it on the
/// next -- which is what saving `old_indent` is for.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn ins_shift(c: c_int, lastc: c_int) {
    unsafe {
        if stop_arrow() == FAIL {
            return;
        }
        AppendCharToRedobuff(c);

        // `0 CTRL-D` and `^ CTRL-D`: the `0`/`^` was inserted as an ordinary
        // character and has to come off again.
        if c == Ctrl_D
            && (lastc == '0' as c_int || lastc == '^' as c_int)
            && (*curwin.get()).w_cursor.col > 0
        {
            (*curwin.get()).w_cursor.col -= 1;
            del_char(false);
            if State.get() & REPLACE_FLAG != 0 {
                replace_pop_ins();
            }
            if lastc == '^' as c_int {
                old_indent.set(get_indent()); // remember the indent
            }
            change_indent(INDENT_SET, 0, 1, true);
        } else {
            change_indent(
                if c == Ctrl_D { INDENT_DEC } else { INDENT_INC },
                0,
                1,
                true,
            );
        }

        if did_ai.get() && *skipwhite(get_cursor_line_ptr()) as c_int != NUL {
            did_ai.set(false);
        }
        did_si.set(false);
        can_si.set(false);
        can_si_back.set(false);
        can_cindent.set(false);
    }
}

/// Handle TAB in Insert or Replace mode.
///
/// Answers `true` when the TAB is to be inserted like an ordinary character,
/// which is the common case: 'expandtab' off, 'softtabstop' unset, and either
/// 'smarttab' off or 'tabstop' equal to 'shiftwidth' anyway.
///
/// # Safety
/// Must run with a live `curwin`/`curbuf`.
pub(crate) unsafe fn ins_tab() -> bool {
    unsafe {
        if Insstart_blank_vcol.get() == MAXCOL as colnr_T
            && (*curwin.get()).w_cursor.lnum == (*Insstart.ptr()).lnum
        {
            Insstart_blank_vcol.set(get_nolist_virtcol());
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
        let smart_tab = p_sta.get() != 0
            && ind
            && (tabstop_count((*curbuf.get()).b_p_vts_array) > 1
                || (tabstop_count((*curbuf.get()).b_p_vts_array) == 1
                    && tabstop_first((*curbuf.get()).b_p_vts_array) != get_sw_value(curbuf.get()))
                || (tabstop_count((*curbuf.get()).b_p_vts_array) == 0
                    && (*curbuf.get()).b_p_ts != get_sw_value(curbuf.get()) as OptInt));
        let soft_tab = tabstop_count((*curbuf.get()).b_p_vsts_array) != 0 || get_sts_value() != 0;
        if (*curbuf.get()).b_p_et == 0 && !smart_tab && !soft_tab {
            // Nothing special: insert TAB like a normal character.
            return true;
        }

        if stop_arrow() == FAIL {
            return true;
        }

        did_ai.set(false);
        did_si.set(false);
        can_si.set(false);
        can_si_back.set(false);
        AppendToRedobuff(c"\t".as_ptr());

        // How many columns to the next stop, from whichever option owns it.
        let mut temp = if p_sta.get() != 0 && ind {
            // A tab in the indent uses 'shiftwidth'.
            let sw = get_sw_value(curbuf.get());
            sw - get_nolist_virtcol() % sw
        } else if tabstop_count((*curbuf.get()).b_p_vsts_array) > 0 || (*curbuf.get()).b_p_sts != 0
        {
            tabstop_padding(
                get_nolist_virtcol(),
                get_sts_value() as OptInt,
                (*curbuf.get()).b_p_vsts_array,
            )
        } else {
            tabstop_padding(
                get_nolist_virtcol(),
                (*curbuf.get()).b_p_ts,
                (*curbuf.get()).b_p_vts_array,
            )
        };

        // The first space goes in with `ins_char`, which in Replace mode
        // deletes one character; the rest with `ins_str`, which deletes none.
        // In `MODE_VREPLACE` every one goes through `ins_char`.
        ins_char(' ' as c_int);
        temp -= 1;
        while temp > 0 {
            if State.get() & VREPLACE_FLAG != 0 {
                ins_char(' ' as c_int);
            } else {
                ins_str(c" ".as_ptr().cast_mut(), 1);
                if State.get() & REPLACE_FLAG != 0 {
                    replace_push_nul(); // no character was replaced
                }
            }
            temp -= 1;
        }

        // With 'expandtab' off, put TABs back where the spaces will do.
        if (*curbuf.get()).b_p_et == 0
            && (tabstop_count((*curbuf.get()).b_p_vsts_array) > 0
                || get_sts_value() > 0
                || (p_sta.get() != 0 && ind))
        {
            tab_spaces_to_tabs();
        }
        false
    }
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
///
/// # Safety
/// Must run with a live `curwin`/`curbuf`, 'expandtab' off.
unsafe fn tab_spaces_to_tabs() {
    unsafe {
        let mut ptr: *mut c_char;
        let mut saved_line: *mut c_char = ::core::ptr::null_mut();
        let mut pos = pos_T::default();
        let cursor: *mut pos_T;
        let mut change_col = -1;
        let save_list = (*curwin.get()).w_onebuf_opt.wo_list;

        // Get the current line.  In `MODE_VREPLACE` no real change may
        // happen yet, so work on a copy.
        if State.get() & VREPLACE_FLAG != 0 {
            pos = (*curwin.get()).w_cursor;
            cursor = &raw mut pos;
            saved_line = xstrnsave(get_cursor_line_ptr(), get_cursor_line_len() as size_t);
            ptr = saved_line.offset(pos.col as isize);
        } else {
            ptr = get_cursor_pos_ptr();
            cursor = &raw mut (*curwin.get()).w_cursor;
        }

        // 'list' changes what a TAB is worth; unless 'cpoptions' has `L`, it
        // must not be allowed to.
        if !cpo_has(CpoFlag::LISTWM) {
            (*curwin.get()).w_onebuf_opt.wo_list = 0;
        }

        // Find the first white character of the run.
        let mut fpos = (*curwin.get()).w_cursor;
        while fpos.col > 0 && ascii_iswhite(*ptr.offset(-1) as c_int) {
            fpos.col -= 1;
            ptr = ptr.offset(-1);
        }
        // In Replace mode the run must not reach back before the insert.
        if State.get() & REPLACE_FLAG != 0
            && fpos.lnum == (*Insstart.ptr()).lnum
            && fpos.col < (*Insstart.ptr()).col
        {
            ptr = ptr.offset(((*Insstart.ptr()).col - fpos.col) as isize);
            fpos.col = (*Insstart.ptr()).col;
        }

        let mut vcol: colnr_T = 0;
        let mut want_vcol: colnr_T = 0;
        getvcol(
            curwin.get(),
            &raw mut fpos,
            &raw mut vcol,
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
        );
        getvcol(
            curwin.get(),
            cursor,
            &raw mut want_vcol,
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
        );

        // Use as many TABs as possible, measuring each one's width where it
        // lands.
        let tab = c"\t".as_ptr().cast_mut();
        let tab_v = *tab as uint8_t as int32_t;
        let mut csarg = CharsizeArg::default();
        let mut cstype = init_charsize_arg(&mut csarg, curwin.get(), 0, tab);
        while ascii_iswhite(*ptr as c_int) {
            let i = win_charsize(cstype, vcol, tab, tab_v, &mut csarg).width;
            if vcol + i > want_vcol {
                break;
            }
            if *ptr as c_int != TAB {
                *ptr = TAB as c_char;
                if change_col < 0 {
                    change_col = fpos.col; // remember the first changed column
                    if fpos.lnum == (*Insstart.ptr()).lnum && fpos.col < (*Insstart.ptr()).col {
                        (*Insstart.ptr()).col = fpos.col;
                    }
                }
            }
            fpos.col += 1;
            ptr = ptr.offset(1);
            vcol += i;
        }

        if change_col >= 0 {
            // Skip over the spaces the TABs have made redundant.
            let mut repl_off = 0;
            cstype = init_charsize_arg(&mut csarg, curwin.get(), 0, ptr);
            while vcol < want_vcol && *ptr as c_int == ' ' as c_int {
                vcol += win_charsize(cstype, vcol, ptr, b' ' as int32_t, &mut csarg).width;
                ptr = ptr.offset(1);
                repl_off += 1;
            }
            if vcol > want_vcol {
                ptr = ptr.offset(-1);
                repl_off -= 1;
            }
            fpos.col += repl_off;

            // Delete the spaces between `fpos` and the cursor.
            let i = (*cursor).col - fpos.col;
            if i > 0 {
                if State.get() & VREPLACE_FLAG == 0 {
                    // Rebuild the line without them.
                    let newp_len = (*curbuf.get()).b_ml.ml_line_textlen - i;
                    let newp = xmalloc(newp_len as size_t) as *mut c_char;
                    let col = ptr.offset_from((*curbuf.get()).b_ml.ml_line_ptr);
                    if col > 0 {
                        memmove(
                            newp as *mut ::core::ffi::c_void,
                            ptr.offset(-col) as *const ::core::ffi::c_void,
                            col as size_t,
                        );
                    }
                    memmove(
                        newp.offset(col) as *mut ::core::ffi::c_void,
                        ptr.offset(i as isize) as *const ::core::ffi::c_void,
                        (newp_len as ptrdiff_t - col) as size_t,
                    );
                    if (*curbuf.get()).b_ml.ml_flags & (ML_LINE_DIRTY | ML_ALLOCATED) != 0 {
                        xfree((*curbuf.get()).b_ml.ml_line_ptr as *mut ::core::ffi::c_void);
                    }
                    (*curbuf.get()).b_ml.ml_line_ptr = newp;
                    (*curbuf.get()).b_ml.ml_line_textlen = newp_len;
                    (*curbuf.get()).b_ml.ml_flags =
                        ((*curbuf.get()).b_ml.ml_flags | ML_LINE_DIRTY) & !ML_EMPTY;
                    inserted_bytes(
                        fpos.lnum,
                        change_col,
                        (*cursor).col - change_col,
                        fpos.col - change_col,
                    );
                } else {
                    memmove(
                        ptr as *mut ::core::ffi::c_void,
                        ptr.offset(i as isize) as *const ::core::ffi::c_void,
                        strlen(ptr.offset(i as isize)) + 1,
                    );
                }

                // Each deleted space had an entry on the replace stack.
                if State.get() & REPLACE_FLAG != 0 && State.get() & VREPLACE_FLAG == 0 {
                    for _ in 0..i {
                        replace_join(repl_off);
                    }
                }
            }
            (*cursor).col -= i;

            // In `MODE_VREPLACE` the change was made to the copy; replay it
            // onto the real line.
            if State.get() & VREPLACE_FLAG != 0 {
                backspace_until_column(change_col);
                ins_bytes_len(
                    saved_line.offset(change_col as isize),
                    ((*cursor).col - change_col) as size_t,
                );
            }
        }

        if State.get() & VREPLACE_FLAG != 0 {
            xfree(saved_line as *mut ::core::ffi::c_void);
        }
        (*curwin.get()).w_onebuf_opt.wo_list = save_list;
    }
}

/// Handle CR or NL in Insert mode.
///
/// Answers false when undo could not be saved -- but *true* when an
/// abbreviation swallowed the key, which is not the same thing.
///
/// # Safety
/// Must run with a live `curwin`.
pub(crate) unsafe fn ins_eol(c: c_int) -> bool {
    unsafe {
        if echeck_abbr(c + ABBR_OFF) {
            return true;
        }
        if stop_arrow() == FAIL {
            return false;
        }
        undisplay_dollar();

        // Strange, but this is what the NL replaces in Replace mode.
        if State.get() & REPLACE_FLAG != 0 && State.get() & VREPLACE_FLAG == 0 {
            replace_push_nul();
        }

        // In 'virtualedit' past the end of the line, make the position real
        // first.
        if virtual_active(curwin.get()) && (*curwin.get()).w_cursor.coladd > 0 {
            coladvance(curwin.get(), getviscol());
        }

        // In 'revins' the cursor is at the start of what was typed, and the
        // line is broken at its end.
        if revins_on.get() {
            (*curwin.get()).w_cursor.col += get_cursor_pos_len();
        }

        AppendToRedobuff(NL_STR.as_ptr());
        let ok = open_line(
            FORWARD,
            if has_format_option(FoFlag::RET_COMS) {
                OPENLINE_DO_COM
            } else {
                0
            },
            old_indent.get(),
            ::core::ptr::null_mut(),
        );
        old_indent.set(0);
        can_cindent.set(true);
        // The new line may be in a closed fold.
        foldOpenCursor();
        ok
    }
}
