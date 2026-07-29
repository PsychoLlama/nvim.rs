//! Visual mode: entering and leaving it, and the area two corners describe.
//!
//! The off-by-one rules live here. 'selection' decides whether the character
//! under the far end is part of the selection; `adjust_for_sel` moves the
//! cursor one on so an exclusive selection covers what it looks like it
//! covers, and `unadjust_for_sel` puts that back before anything reads the
//! area again.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ptr;

use super::*;

/// Whether 'selection' is "exclusive": the character under the far end of the
/// selection is not part of it.
#[inline(always)]
fn selection_exclusive() -> bool {
    // SAFETY: 'selection' is a non-empty C string option.
    unsafe { *p_sel.get() as c_int == 'e' as c_int }
}

/// Leave Visual mode, remembering the selection for `gv` and `'<`/`'>`.
pub(crate) fn end_visual_mode() {
    VIsual_select_exclu_adj.set(false);
    VIsual_active.set(false);
    // SAFETY: all of this is the current buffer's and window's own state.
    unsafe {
        setmouse();
        mouse_dragging.set(0);
        (*curbuf.get()).b_visual.vi_mode = VIsual_mode.get();
        (*curbuf.get()).b_visual.vi_start = VIsual.get();
        (*curbuf.get()).b_visual.vi_end = (*curwin.get()).w_cursor;
        (*curbuf.get()).b_visual.vi_curswant = (*curwin.get()).w_curswant;
        (*curbuf.get()).b_visual_mode_eval = VIsual_mode.get();
        if !virtual_active(curwin.get()) {
            (*curwin.get()).w_cursor.coladd = 0;
        }
        may_clear_cmdline();
        adjust_cursor_eol();
        may_trigger_modechanged();
    }
}

/// Leave Visual mode and forget the selection, so `gv` will not bring it back.
pub(crate) fn reset_VIsual_and_resel() {
    if VIsual_active.get() {
        end_visual_mode();
        // SAFETY: schedules a redraw of the current buffer.
        unsafe { redraw_curbuf_later(UPD_INVERTED as c_int) };
    }
    VIsual_reselect.set(false_0);
}

/// As [`reset_VIsual_and_resel`], but only when there was a selection.
pub(crate) fn reset_VIsual() {
    if VIsual_active.get() {
        end_visual_mode();
        // SAFETY: schedules a redraw of the current buffer.
        unsafe { redraw_curbuf_later(UPD_INVERTED as c_int) };
        VIsual_reselect.set(false_0);
    }
}

/// Put back the Visual mode `v_visop` forced to linewise for an uppercase
/// operator.
pub(crate) fn restore_visual_mode() {
    if VIsual_mode_orig.get() != NUL {
        // SAFETY: `curbuf` is the current buffer.
        unsafe { (*curbuf.get()).b_visual.vi_mode = VIsual_mode_orig.get() };
        VIsual_mode_orig.set(NUL);
    }
}

/// The text the Visual selection covers, for a command that wants it as a
/// string rather than as an operator target.
///
/// Refuses -- and beeps, when it was given an operator to clear -- for a
/// selection spanning more than one line. Leaves Visual mode either way it
/// succeeds.
pub(crate) unsafe fn get_visual_text(
    cap: *mut cmdarg_T,
    pp: *mut *mut c_char,
    lenp: *mut size_t,
) -> bool {
    if VIsual_mode.get() != 'V' as c_int {
        // SAFETY: adjusts the current window's cursor or `VIsual`.
        unsafe { unadjust_for_sel() };
    }
    // SAFETY: `cap` is null or the caller's live command argument, and `pp`
    // and `lenp` are its out-parameters.
    unsafe {
        if (*VIsual.ptr()).lnum != (*curwin.get()).w_cursor.lnum {
            if !cap.is_null() {
                clearopbeep((*cap).oap);
            }
            return false;
        }
        if VIsual_mode.get() == 'V' as c_int {
            *pp = get_cursor_line_ptr();
            *lenp = get_cursor_line_len() as size_t;
        } else {
            // The earlier of the two ends is the start; the length is the
            // column difference, inclusive.
            if lt((*curwin.get()).w_cursor, VIsual.get()) {
                *pp = ml_get_pos(&raw mut (*curwin.get()).w_cursor);
                *lenp = ((*VIsual.ptr()).col - (*curwin.get()).w_cursor.col + 1) as size_t;
            } else {
                *pp = ml_get_pos(VIsual.ptr());
                *lenp = ((*curwin.get()).w_cursor.col - (*VIsual.ptr()).col + 1) as size_t;
            }
            if **pp as c_int == NUL {
                *lenp = 0;
            }
            // The last character may be multibyte; take the rest of it.
            //
            // `utfc_ptr2len` answers 0 for a NUL, and upstream adds `0 - 1`
            // as a `size_t` -- which wraps and so takes one *off* the length.
            // Reachable: a blockwise selection whose last line is short ends
            // on the terminator. Kept wrapping, deliberately.
            if *lenp > 0 {
                let tail = utfc_ptr2len((*pp).add(*lenp as usize - 1));
                *lenp = (*lenp).wrapping_add((tail - 1) as size_t);
            }
        }
    }
    reset_VIsual_and_resel();
    true
}

/// Swap the two ends of the selection.
///
/// `o` swaps them outright. `O` in blockwise mode swaps only the *columns*,
/// which means moving both ends -- and the second half of this only runs when
/// the first attempt left the cursor where it started, which happens when the
/// two columns are the same width.
pub(crate) unsafe fn v_swap_corners(cmdchar: c_int) {
    // SAFETY: `curwin` is the current window and `VIsual` a live position.
    unsafe {
        if cmdchar != 'O' as c_int || VIsual_mode.get() != Ctrl_V {
            let old_cursor = (*curwin.get()).w_cursor;
            (*curwin.get()).w_cursor = VIsual.get();
            VIsual.set(old_cursor);
            (*curwin.get()).w_set_curswant = true_0;
            return;
        }

        let (mut left, mut right): (colnr_T, colnr_T) = (0, 0);
        let mut old_cursor = (*curwin.get()).w_cursor;
        getvcols(
            curwin.get(),
            &raw mut old_cursor,
            VIsual.ptr(),
            &raw mut left,
            &raw mut right,
        );
        (*curwin.get()).w_cursor.lnum = (*VIsual.ptr()).lnum;
        coladvance(curwin.get(), left);
        VIsual.set((*curwin.get()).w_cursor);
        (*curwin.get()).w_cursor.lnum = old_cursor.lnum;
        (*curwin.get()).w_curswant = right;
        // An exclusive selection ends one past the last column it covers.
        if old_cursor.lnum >= (*VIsual.ptr()).lnum && selection_exclusive() {
            (*curwin.get()).w_curswant += 1;
        }
        coladvance(curwin.get(), (*curwin.get()).w_curswant);

        // Nothing moved: the block's two columns are the same width, so swap
        // them the other way round instead.
        if (*curwin.get()).w_cursor.col == old_cursor.col
            && (!virtual_active(curwin.get())
                || (*curwin.get()).w_cursor.coladd == old_cursor.coladd)
        {
            (*curwin.get()).w_cursor.lnum = (*VIsual.ptr()).lnum;
            if old_cursor.lnum <= (*VIsual.ptr()).lnum && selection_exclusive() {
                right += 1;
            }
            coladvance(curwin.get(), right);
            VIsual.set((*curwin.get()).w_cursor);
            (*curwin.get()).w_cursor.lnum = old_cursor.lnum;
            coladvance(curwin.get(), left);
            (*curwin.get()).w_curswant = left;
        }
    }
}

/// An operator typed in Visual mode, as the pairs of "what was typed" and
/// "what it means".
///
/// Upstream spells this as the string `"YyDdCcxdXdAAIIrr"` and finds the
/// character with `strchr`, taking the byte after it.
const VISUAL_OPS: [(u8, u8); 8] = [
    (b'Y', b'y'),
    (b'D', b'd'),
    (b'C', b'c'),
    (b'x', b'd'),
    (b'X', b'd'),
    (b'A', b'A'),
    (b'I', b'I'),
    (b'r', b'r'),
];

/// Run an operator typed in Visual mode.
///
/// An uppercase one forces the selection linewise -- except in blockwise
/// mode, where `C` and `D` instead extend every line to its end.
pub(crate) unsafe fn v_visop(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if (*cap).cmdchar >= 'A' as c_int && (*cap).cmdchar <= 'Z' as c_int {
            if VIsual_mode.get() != Ctrl_V {
                VIsual_mode_orig.set(VIsual_mode.get());
                VIsual_mode.set('V' as c_int);
            } else if (*cap).cmdchar == 'C' as c_int || (*cap).cmdchar == 'D' as c_int {
                (*curwin.get()).w_curswant = MAXCOL as colnr_T;
            }
        }
        let typed = (*cap).cmdchar as u8;
        (*cap).cmdchar = VISUAL_OPS
            .iter()
            .find(|(from, _)| *from == typed)
            .expect("v_visop is only reached for a character in VISUAL_OPS")
            .1 as c_int;
        nv_operator(cap);
    }
}

/// Reselect the previous selection, `count` times as large.
///
/// Only reached with a count: `3v` means "three times whatever was selected
/// last". The line count and the column count multiply separately, which is
/// why the charwise and blockwise cases are spelled out.
unsafe fn reselect_scaled(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        VIsual.set((*curwin.get()).w_cursor);
        VIsual_active.set(true);
        VIsual_reselect.set(true_0);
        if (*cap).arg == 0 {
            may_start_select('c' as c_int);
        }
        setmouse();
        if p_smd.get() != 0 && msg_silent.get() == 0 {
            redraw_cmdline.set(true);
        }
        // The count multiplies the size of the remembered selection, and it
        // is user input: `999999999v` after a three-column selection
        // overflows. Upstream does this arithmetic in C, where it wraps, and
        // `check_cursor`/`coladvance` clamp whatever comes out -- so wrapping
        // is both what the C produces and safe. The transpile used Rust's
        // checked operators here and aborted the debug build instead.
        if resel_VIsual_mode.get() != 'v' as c_int || resel_VIsual_line_count.get() > 1 {
            (*curwin.get()).w_cursor.lnum = (*curwin.get()).w_cursor.lnum.wrapping_add(
                resel_VIsual_line_count
                    .get()
                    .wrapping_mul((*cap).count0 as linenr_T)
                    .wrapping_sub(1),
            );
            check_cursor(curwin.get());
        }
        VIsual_mode.set(resel_VIsual_mode.get());

        if VIsual_mode.get() == 'v' as c_int {
            if resel_VIsual_line_count.get() <= 1 {
                update_curswant_force();
                (*curwin.get()).w_curswant = (*curwin.get())
                    .w_curswant
                    .wrapping_add(resel_VIsual_vcol.get().wrapping_mul((*cap).count0) as colnr_T);
                if !selection_exclusive() {
                    (*curwin.get()).w_curswant -= 1;
                }
            } else {
                (*curwin.get()).w_curswant = resel_VIsual_vcol.get();
            }
            coladvance(curwin.get(), (*curwin.get()).w_curswant);
        }

        if resel_VIsual_vcol.get() == MAXCOL as c_int {
            (*curwin.get()).w_curswant = MAXCOL as colnr_T;
            coladvance(curwin.get(), MAXCOL as c_int);
        } else if VIsual_mode.get() == Ctrl_V {
            // The width is measured from the *start* line, so the cursor goes
            // there while 'curswant' is recomputed and comes back after.
            let lnum = (*curwin.get()).w_cursor.lnum;
            (*curwin.get()).w_cursor.lnum = (*VIsual.ptr()).lnum;
            update_curswant_force();
            (*curwin.get()).w_curswant = (*curwin.get()).w_curswant.wrapping_add(
                resel_VIsual_vcol
                    .get()
                    .wrapping_mul((*cap).count0)
                    .wrapping_sub(1) as colnr_T,
            );
            (*curwin.get()).w_cursor.lnum = lnum;
            if selection_exclusive() {
                (*curwin.get()).w_curswant += 1;
            }
            coladvance(curwin.get(), (*curwin.get()).w_curswant);
        } else {
            (*curwin.get()).w_set_curswant = true_0;
        }
        redraw_curbuf_later(UPD_INVERTED as c_int);
    }
}

/// `v`, `V`, `CTRL-V` and their Select-mode twins.
///
/// Kept `extern "C"`: this is an `nv_cmds` row's handler, and `nv_func_T` is
/// still a C function pointer.
pub(crate) unsafe fn nv_visual(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if (*cap).cmdchar == Ctrl_Q {
            (*cap).cmdchar = Ctrl_V;
        }
        // After an operator these are not commands but a forced motion kind:
        // `dv`, `dV`, `d CTRL-V`.
        if (*(*cap).oap).op_type != OP_NOP as c_int {
            (*(*cap).oap).motion_force = (*cap).cmdchar;
            motion_force.set((*(*cap).oap).motion_force);
            finish_op.set(false);
            return;
        }

        VIsual_select.set((*cap).arg != 0);
        if VIsual_active.get() {
            // The same key again leaves Visual mode; a different one switches
            // to that kind of selection.
            if VIsual_mode.get() == (*cap).cmdchar {
                end_visual_mode();
            } else {
                VIsual_mode.set((*cap).cmdchar);
                showmode();
                may_trigger_modechanged();
            }
            redraw_curbuf_later(UPD_INVERTED as c_int);
        } else if (*cap).count0 > 0 && resel_VIsual_mode.get() != NUL {
            reselect_scaled(cap);
        } else {
            if (*cap).arg == 0 {
                may_start_select('c' as c_int);
            }
            n_start_visual_mode((*cap).cmdchar);
            // An exclusive selection needs one more character to cover the
            // same text, so the count is raised before it is spent.
            if VIsual_mode.get() != 'V' as c_int && selection_exclusive() {
                (*cap).count1 += 1;
            } else {
                VIsual_select_exclu_adj.set(false);
            }
            // A count means "select this many characters or lines".
            if (*cap).count0 > 0 && {
                (*cap).count1 -= 1;
                (*cap).count1 > 0
            } {
                if VIsual_mode.get() == 'v' as c_int || VIsual_mode.get() == Ctrl_V {
                    nv_right(cap);
                } else if VIsual_mode.get() == 'V' as c_int {
                    nv_down(cap);
                }
            }
        }
    }
}

/// Start a charwise selection because a shifted key was pressed.
pub(crate) fn start_selection() {
    may_start_select('k' as c_int);
    // SAFETY: enters Visual mode on the current window.
    unsafe { n_start_visual_mode('v' as c_int) };
}

/// Decide between Visual and Select mode for a selection about to start.
///
/// `c` says how it is starting -- 'k'ey, 'm'ouse or 'c'ommand -- and
/// 'selectmode' says which of those mean Select. A command-started selection
/// only counts as typed when nothing is being replayed.
pub(crate) fn may_start_select(c: c_int) {
    // SAFETY: 'selectmode' is a C string option.
    let by_selectmode = unsafe { !vim_strchr(p_slm.get(), c).is_null() };
    // SAFETY: reads the typeahead state.
    let typed = unsafe { c == 'o' as c_int || (stuff_empty() && typebuf_typed() != 0) };
    VIsual_select.set(typed && by_selectmode);
}

/// Enter Visual mode of kind `c` at the cursor.
pub(crate) unsafe fn n_start_visual_mode(c: c_int) {
    VIsual_mode.set(c);
    VIsual_active.set(true);
    VIsual_reselect.set(true_0);
    // SAFETY: `curwin` is the current window.
    unsafe {
        // A block selection starting inside a TAB starts at the column the
        // cursor is displayed at, not at the TAB's first column.
        if c == Ctrl_V
            && get_ve_flags(curwin.get()) & kOptVeFlagBlock as c_int as c_uint != 0
            && gchar_cursor() == TAB
        {
            validate_virtcol(curwin.get());
            coladvance(curwin.get(), (*curwin.get()).w_virtcol);
        }
        VIsual.set((*curwin.get()).w_cursor);
        foldAdjustVisual();
        may_trigger_modechanged();
        setmouse();
        conceal_check_cursor_line();
        if p_smd.get() != 0 && msg_silent.get() == 0 {
            redraw_cmdline.set(true);
        }
        // Seed the "what was highlighted last time" pair so the first redraw
        // has something to compare against.
        if (*curwin.get()).w_redr_type < UPD_INVERTED as c_int {
            (*curwin.get()).w_old_cursor_lnum = (*curwin.get()).w_cursor.lnum;
            (*curwin.get()).w_old_visual_lnum = (*curwin.get()).w_cursor.lnum;
        }
        redraw_curbuf_later(UPD_VALID as c_int);
    }
}

/// `gv`: select what was selected last.
///
/// Doing it while a selection is up *swaps* the two, so `gv` twice comes back
/// where it started.
pub(crate) unsafe fn nv_gv_cmd(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let vi = &raw mut (*curbuf.get()).b_visual;
        if (*vi).vi_start.lnum == 0
            || (*vi).vi_start.lnum > (*curbuf.get()).b_ml.ml_line_count
            || (*vi).vi_end.lnum == 0
        {
            beep_flush();
            return;
        }

        let tpos;
        if VIsual_active.get() {
            let mode = VIsual_mode.get();
            VIsual_mode.set((*vi).vi_mode);
            (*vi).vi_mode = mode;
            (*curbuf.get()).b_visual_mode_eval = mode;
            let curswant = (*curwin.get()).w_curswant;
            (*curwin.get()).w_curswant = (*vi).vi_curswant;
            (*vi).vi_curswant = curswant;
            tpos = (*vi).vi_end;
            (*vi).vi_end = (*curwin.get()).w_cursor;
            (*curwin.get()).w_cursor = (*vi).vi_start;
            (*vi).vi_start = VIsual.get();
        } else {
            VIsual_mode.set((*vi).vi_mode);
            (*curwin.get()).w_curswant = (*vi).vi_curswant;
            tpos = (*vi).vi_end;
            (*curwin.get()).w_cursor = (*vi).vi_start;
        }

        VIsual_active.set(true);
        VIsual_reselect.set(true_0);
        // Both ends are checked against the buffer: it may have shrunk since.
        check_cursor(curwin.get());
        VIsual.set((*curwin.get()).w_cursor);
        (*curwin.get()).w_cursor = tpos;
        check_cursor(curwin.get());
        update_topline(curwin.get());
        if (*cap).arg != 0 {
            VIsual_select.set(true);
            VIsual_select_reg.set(0);
        } else {
            may_start_select('c' as c_int);
        }
        setmouse();
        redraw_curbuf_later(UPD_INVERTED as c_int);
        showmode();
    }
}

/// Make an exclusive selection cover the character the cursor is on, so the
/// operator about to run sees what the highlight showed.
pub(crate) unsafe fn adjust_for_sel(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if VIsual_active.get()
            && (*(*cap).oap).inclusive
            && selection_exclusive()
            && gchar_cursor() != NUL
            && lt(VIsual.get(), (*curwin.get()).w_cursor)
        {
            inc_cursor();
            (*(*cap).oap).inclusive = false;
            VIsual_select_exclu_adj.set(true);
        }
    }
}

/// Undo [`adjust_for_sel`] on whichever end is the later one.
///
/// Answers whether the position moved to the previous line.
pub(crate) unsafe fn unadjust_for_sel() -> bool {
    // SAFETY: `curwin` is the current window and `VIsual` a live position.
    unsafe {
        if selection_exclusive() && !equalpos(VIsual.get(), (*curwin.get()).w_cursor) {
            let later = if lt(VIsual.get(), (*curwin.get()).w_cursor) {
                &raw mut (*curwin.get()).w_cursor
            } else {
                VIsual.ptr()
            };
            return unadjust_for_sel_inner(later);
        }
        false
    }
}

/// Move one position back, across a line break if there is nothing else left.
///
/// Answers whether it crossed one.
pub(crate) unsafe fn unadjust_for_sel_inner(pp: *mut pos_T) -> bool {
    VIsual_select_exclu_adj.set(false);
    // SAFETY: `pp` is the caller's live position in the current buffer.
    unsafe {
        if (*pp).coladd > 0 {
            (*pp).coladd -= 1;
        } else if (*pp).col > 0 {
            (*pp).col -= 1;
            mark_mb_adjustpos(curbuf.get(), pp);
            // Inside a TAB, stepping back a byte means stepping to the last
            // screen column the TAB covers.
            if virtual_active(curwin.get()) {
                let (mut cs, mut ce): (colnr_T, colnr_T) = (0, 0);
                getvcol(curwin.get(), pp, &raw mut cs, ptr::null_mut(), &raw mut ce);
                (*pp).coladd = ce - cs;
            }
        } else if (*pp).lnum > 1 {
            (*pp).lnum -= 1;
            (*pp).col = ml_get_len((*pp).lnum);
            return true;
        }
        false
    }
}

/// `gh`, `gH`, `g CTRL-H`: Select mode, either fresh or from a reselection.
pub(crate) unsafe fn nv_select(cap: *mut cmdarg_T) {
    if VIsual_active.get() {
        VIsual_select.set(true);
        VIsual_select_reg.set(0);
    } else if VIsual_reselect.get() != 0 {
        // Re-enter through `gv`, which is where the reselection lives.
        // SAFETY: `cap` is the caller's live command argument.
        unsafe {
            (*cap).nchar = 'v' as c_int;
            (*cap).arg = true_0;
            nv_g_cmd(cap);
        }
    }
}

/// A text object: `iw`, `a(`, `it` and the rest.
///
/// 'matchpairs' is forced to the four bracket pairs for the duration, because
/// a text object's idea of a block is fixed and must not follow the option.
pub(crate) unsafe fn nv_object(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let include = (*cap).cmdchar != 'i' as c_int;
        let mps_save = (*curbuf.get()).b_p_mps;
        (*curbuf.get()).b_p_mps = c"(:),{:},[:],<:>".as_ptr().cast_mut();

        let oap = (*cap).oap;
        let n = (*cap).count1;
        let found = match u8::try_from((*cap).nchar).unwrap_or(0) {
            b'w' => current_word(oap, n, include, false) != 0,
            b'W' => current_word(oap, n, include, true) != 0,
            b'b' | b'(' | b')' => current_block(oap, n, include, '(' as c_int, ')' as c_int) != 0,
            b'B' | b'{' | b'}' => current_block(oap, n, include, '{' as c_int, '}' as c_int) != 0,
            b'[' | b']' => current_block(oap, n, include, '[' as c_int, ']' as c_int) != 0,
            b'<' | b'>' => current_block(oap, n, include, '<' as c_int, '>' as c_int) != 0,
            b't' => {
                // A tag block's end is already where it should be; the
                // operator must not push it back over the closing tag.
                (*cap).retval |= CA_NO_ADJ_OP_END as c_int;
                current_tagblock(oap, n, include) != 0
            }
            b'p' => current_par(oap, n, include, 'p' as c_int) != 0,
            b's' => current_sent(oap, n, include) != 0,
            b'"' | b'\'' | b'`' => current_quote(oap, n, include, (*cap).nchar),
            _ => false,
        };

        (*curbuf.get()).b_p_mps = mps_save;
        if !found {
            clearopbeep(oap);
        }
        adjust_cursor_col();
        (*curwin.get()).w_set_curswant = true_0;
    }
}
