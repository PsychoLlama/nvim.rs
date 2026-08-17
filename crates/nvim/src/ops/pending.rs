//! `do_pending_operator` -- the operator dispatcher.
//!
//! Normal mode reads an operator and then a motion; this runs when both have
//! arrived, and it is a pipeline whose *last* step is the switch that calls
//! one of this module's operators. Everything before it decides what the
//! region actually is, and that is where the complexity lives:
//!
//! | step | what it decides |
//! | --- | --- |
//! | [`apply_motion_force`] | `v`/`V`/CTRL-V typed between the operator and the motion |
//! | [`record_operator_redo`] | what `.` will replay, and which operators are not replayable at all |
//! | [`resume_redo_visual`] | when `.` is *already* replaying: the region's size comes from [`REDO_VISUAL`], not from a selection |
//! | [`start_visual_region`] | the operator was typed *after* a selection, so the region is the selection |
//! | [`order_region`] | which end is the start, and closed folds swallowed whole |
//! | [`prepare_visual_redo`] | the selection's size, for `gv` and for the next `.` |
//! | [`finish_visual_region`] | linewise/charwise fixups, and switching Visual off |
//! | [`adjust_region_end`] | an exclusive end in column one belongs to the line before |
//! | [`run_operator`] | the switch |
//!
//! Two things thread through all of it. 'linebreak' is turned off at the top
//! and put back at every exit, because it changes what `getvcol` answers and
//! every column here is measured without it -- which is why so many switch
//! arms call `restore_lbr` before handing control to Insert mode or to a
//! user callback. And [`REDO_VISUAL`] is process-wide state: `.` after a
//! Visual operator re-creates a region of the *same size* at the cursor,
//! which is why `OP_FUNCTION` saves and restores it around 'operatorfunc'
//! (the callback may run another operator and overwrite it).

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_int, c_void};

use super::*;
use crate::keycodes::{K_COMMAND, K_LUA};

/// The Visual area a `.` replays: its mode and size, not its position.
///
/// A `static` inside `do_pending_operator` in C. Process-wide on purpose --
/// `.` after `viwd` deletes the same *number of characters* at the cursor.
static REDO_VISUAL: GlobalCell<redo_VIsual_T> = GlobalCell::new(redo_VIsual_T {
    rv_mode: NUL,
    rv_line_count: 0,
    rv_vcol: 0,
    rv_count: 0,
    rv_arg: 0,
});

/// Zero an `oparg_T` between commands.
///
/// # Safety
/// `oap` must point to a live `oparg_T`.
pub unsafe fn clear_oparg(oap: *mut oparg_T) {
    unsafe { *oap = oparg_T::ZERO };
}

/// Was the operator reached through a command line rather than a key?
///
/// `:` and `<Cmd>` both arrive here as an operator over the Visual area.
///
/// # Safety
/// `cap` must point to a live `cmdarg_T`.
unsafe fn is_ex_cmdchar(cap: *mut cmdarg_T) -> bool {
    unsafe { (*cap).cmdchar == ':' as c_int || (*cap).cmdchar == K_COMMAND }
}

/// Run the operator that a motion (or a Visual selection) has just completed.
///
/// `old_col` is the column to return to when 'startofline' is off.
/// `gui_yank` marks the yank the clipboard does behind the user's back: it
/// must not clear the selection, redraw, or leave a `.` behind.
///
/// # Safety
/// `cap` must point to a live `cmdarg_T` whose `oap` describes a region of the
/// current buffer.
pub unsafe fn do_pending_operator(cap: *mut cmdarg_T, old_col: c_int, gui_yank: bool) {
    unsafe {
        let oap = (*cap).oap;
        let lbr_saved = (*curwin.get()).w_onebuf_opt.wo_lbr;
        let old_cursor = (*curwin.get()).w_cursor;

        if (!finish_op.get() && !VIsual_active.get()) || (*oap).op_type == OP_NOP {
            restore_lbr(lbr_saved != 0);
            return;
        }

        // A yank can be redone when 'cpoptions' has `y`, but never the one the
        // clipboard does for itself.
        let redo_yank = !vim_strchr(p_cpo.get(), CPO_YANK).is_null() && !gui_yank;

        // Unwanted line breaks would move every column measured below.
        reset_lbr();
        (*oap).is_VIsual = VIsual_active.get();
        apply_motion_force(oap);
        record_operator_redo(cap, oap, redo_yank);

        let mut include_line_break = false;
        if redo_VIsual_busy.get() {
            resume_redo_visual(cap, oap);
        } else if VIsual_active.get() {
            include_line_break = start_visual_region(cap, oap, gui_yank);
        }

        order_region(oap);

        // Just in case lines were deleted that make the position invalid.
        check_pos((*curwin.get()).w_buffer, &raw mut (*oap).end);
        (*oap).line_count = (*oap).end.lnum - (*oap).start.lnum + 1;
        // Set before `VIsual_active` is reset below.
        virtual_op.set(virtual_active(curwin.get()) as TriState);

        if VIsual_active.get() || redo_VIsual_busy.get() {
            get_op_vcol(oap, REDO_VISUAL.get().rv_vcol, true);
            prepare_visual_redo(cap, oap, gui_yank, redo_yank);
            finish_visual_region(oap, include_line_break, gui_yank, lbr_saved);
        }

        // Include the trailing byte of a multi-byte character.
        if (*oap).inclusive {
            let l = utfc_ptr2len(ml_get_pos(&raw mut (*oap).end));
            if l > 1 {
                (*oap).end.col += l - 1;
            }
        }
        (*curwin.get()).w_set_curswant = true_0;

        // `empty` is set when start and end are the same. `inclusive` affects
        // that too, unless yanking with the end on a NUL.
        (*oap).empty = (*oap).motion_type != kMTLineWise
            && (!(*oap).inclusive
                || ((*oap).op_type == OP_YANK && gchar_pos(&raw mut (*oap).end) == NUL))
            && equalpos((*oap).start, (*oap).end)
            && !(virtual_op.get() != 0 && (*oap).start.coladd != (*oap).end.coladd);
        // For delete, change and yank it is an error to operate on an empty
        // region when 'cpoptions' has `E` (Vi compatible).
        let empty_region_error =
            (*oap).empty && !vim_strchr(p_cpo.get(), CPO_EMPTYREGION).is_null();

        // Force a redraw for an empty Visual region, an unmodifiable buffer,
        // or a fold: none of those will redraw by themselves.
        if (*oap).is_VIsual
            && ((*oap).empty || (*curbuf.get()).b_p_ma == 0 || (*oap).op_type == OP_FOLD)
        {
            restore_lbr(lbr_saved != 0);
            redraw_curbuf_later(UPD_INVERTED);
        }

        adjust_region_end(cap, oap);
        run_operator(cap, oap, empty_region_error, gui_yank, lbr_saved);

        virtual_op.set(kNone);
        if gui_yank {
            (*curwin.get()).w_cursor = old_cursor;
        } else if p_sol.get() == 0
            && (*oap).motion_type == kMTLineWise
            && !(*oap).end_adjusted
            && ((*oap).op_type == OP_LSHIFT
                || (*oap).op_type == OP_RSHIFT
                || (*oap).op_type == OP_DELETE)
        {
            // 'startofline' is off: go back to the column the command started
            // in.
            reset_lbr();
            (*curwin.get()).w_curswant = old_col;
            coladvance(curwin.get(), (*curwin.get()).w_curswant);
        }
        clearop(oap);
        motion_force.set(NUL);

        restore_lbr(lbr_saved != 0);
    }
}

/// `v`, `V` or CTRL-V typed between the operator and its motion.
///
/// # Safety
/// `oap` must point to a live `oparg_T`.
unsafe fn apply_motion_force(oap: *mut oparg_T) {
    unsafe {
        if (*oap).motion_force == 'V' as c_int {
            (*oap).motion_type = kMTLineWise;
        } else if (*oap).motion_force == 'v' as c_int {
            if (*oap).motion_type == kMTLineWise {
                // A linewise motion never set `inclusive`; "exclusive" is the
                // consistent reading, and makes `dvj` behave.
                (*oap).inclusive = false;
            } else if (*oap).motion_type == kMTCharWise {
                (*oap).inclusive = !(*oap).inclusive;
            }
            (*oap).motion_type = kMTCharWise;
        } else if (*oap).motion_force == Ctrl_V {
            // Turn a line- or charwise motion into a Visual block.
            if !VIsual_active.get() {
                VIsual_active.set(true);
                VIsual.set((*oap).start);
            }
            VIsual_mode.set(Ctrl_V);
            VIsual_select.set(false);
            VIsual_reselect.set(false_0);
        }
    }
}

/// Put the command in the redo buffer, so that `.` repeats it.
///
/// Yank is only redoable under 'cpoptions' `y`, `zf` never is, and neither is
/// any of the fold operators; a search or a `:` command has to have its own
/// text appended so that the repeat really is the same command.
///
/// # Safety
/// `cap` must point to a live `cmdarg_T`.
unsafe fn record_operator_redo(cap: *mut cmdarg_T, oap: *mut oparg_T, redo_yank: bool) {
    unsafe {
        let is_fold_op = matches!(
            (*oap).op_type,
            OP_FOLD
                | OP_FOLDOPEN
                | OP_FOLDOPENREC
                | OP_FOLDCLOSE
                | OP_FOLDCLOSEREC
                | OP_FOLDDEL
                | OP_FOLDDELREC
        );
        let replayable = (redo_yank || (*oap).op_type != OP_YANK)
            && (!VIsual_active.get()
                || (*oap).motion_force != 0
                // Also redo Operator-pending Visual mode mappings.
                || ((is_ex_cmdchar(cap) || (*cap).cmdchar == K_LUA)
                    && (*oap).op_type != OP_COLON))
            && (*cap).cmdchar != 'D' as c_int
            && !is_fold_op;
        if !replayable {
            return;
        }

        prep_redo(
            (*oap).regname,
            (*cap).count0,
            get_op_char((*oap).op_type),
            get_extra_op_char((*oap).op_type),
            (*oap).motion_force,
            (*cap).cmdchar,
            (*cap).nchar,
        );

        if (*cap).cmdchar == '/' as c_int || (*cap).cmdchar == '?' as c_int {
            // A search: without 'cpoptions' `r` the pattern goes in too, so
            // that the repeat really is the same command.
            if vim_strchr(p_cpo.get(), CPO_REDO).is_null() {
                AppendToRedobuffLit((*cap).searchbuf, -1);
            }
            AppendToRedobuff(c"\n".as_ptr());
        } else if is_ex_cmdchar(cap) {
            // `do_cmdline` stored the first typed line in `repeat_cmdline`.
            // When several lines were typed, repeating is not possible.
            if repeat_cmdline.get().is_null() {
                ResetRedobuff();
            } else {
                if (*cap).cmdchar == ':' as c_int {
                    AppendToRedobuffLit(repeat_cmdline.get(), -1);
                } else {
                    AppendToRedobuffSpec(repeat_cmdline.get());
                }
                AppendToRedobuff(c"\n".as_ptr());
                xfree(repeat_cmdline.get() as *mut c_void);
                repeat_cmdline.set(::core::ptr::null_mut());
            }
        } else if (*cap).cmdchar == K_LUA {
            AppendNumberToRedobuff(repeat_luaref.get() as c_int);
            AppendToRedobuff(c"\n".as_ptr());
        }
    }
}

/// `.` replaying a Visual operator: rebuild a region of the recorded size at
/// the cursor.
///
/// # Safety
/// `cap` must point to a live `cmdarg_T`.
unsafe fn resume_redo_visual(cap: *mut cmdarg_T, oap: *mut oparg_T) {
    unsafe {
        let redo = REDO_VISUAL.get();
        (*oap).start = (*curwin.get()).w_cursor;
        (*curwin.get()).w_cursor.lnum += redo.rv_line_count - 1;
        (*curwin.get()).w_cursor.lnum = (*curwin.get())
            .w_cursor
            .lnum
            .min((*curbuf.get()).b_ml.ml_line_count);
        VIsual_mode.set(redo.rv_mode);

        if redo.rv_vcol == MAXCOL || VIsual_mode.get() == 'v' as c_int {
            if VIsual_mode.get() != 'v' as c_int {
                (*curwin.get()).w_curswant = MAXCOL;
            } else if redo.rv_line_count <= 1 {
                // A one-line charwise region is that many columns *from the
                // cursor*, not to a fixed column.
                validate_virtcol(curwin.get());
                (*curwin.get()).w_curswant = (*curwin.get()).w_virtcol + redo.rv_vcol - 1;
            } else {
                (*curwin.get()).w_curswant = redo.rv_vcol;
            }
            coladvance(curwin.get(), (*curwin.get()).w_curswant);
        }
        (*cap).count0 = redo.rv_count;
        (*cap).count1 = if (*cap).count0 == 0 { 1 } else { (*cap).count0 };
    }
}

/// The operator was typed after a selection: the region is the selection.
///
/// Answers `include_line_break`, which 'selection' `exclusive` sets when the
/// backed-off end lands on a line break.
///
/// # Safety
/// `cap` must point to a live `cmdarg_T`; a Visual selection must be active.
unsafe fn start_visual_region(cap: *mut cmdarg_T, oap: *mut oparg_T, gui_yank: bool) -> bool {
    unsafe {
        let mut include_line_break = false;

        if !gui_yank {
            // Keep the area for `'<`/`'>` and for `gv`.
            (*curbuf.get()).b_visual.vi_start = VIsual.get();
            (*curbuf.get()).b_visual.vi_end = (*curwin.get()).w_cursor;
            (*curbuf.get()).b_visual.vi_mode = VIsual_mode.get();
            restore_visual_mode();
            (*curbuf.get()).b_visual.vi_curswant = (*curwin.get()).w_curswant;
            (*curbuf.get()).b_visual_mode_eval = VIsual_mode.get();
        }

        // In Select mode a linewise selection is operated on like a charwise
        // one. `gH<Del>`, which deletes the last line, is the exception.
        if VIsual_select.get()
            && VIsual_mode.get() == 'V' as c_int
            && (*(*cap).oap).op_type != OP_DELETE
        {
            if lt(VIsual.get(), (*curwin.get()).w_cursor) {
                (*VIsual.ptr()).col = 0;
                (*curwin.get()).w_cursor.col = ml_get_len((*curwin.get()).w_cursor.lnum);
            } else {
                (*curwin.get()).w_cursor.col = 0;
                (*VIsual.ptr()).col = ml_get_len((*VIsual.ptr()).lnum);
            }
            VIsual_mode.set('v' as c_int);
        } else if VIsual_mode.get() == 'v' as c_int {
            // 'selection' "exclusive": back off one character.
            include_line_break = unadjust_for_sel();
        }

        (*oap).start = VIsual.get();
        if VIsual_mode.get() == 'V' as c_int {
            (*oap).start.col = 0;
            (*oap).start.coladd = 0;
        }
        include_line_break
    }
}

/// Put `oap.start` at the first position of the region and `oap.end` at the
/// last, with the cursor on the start.
///
/// Outside Visual mode a closed fold at either end is swallowed whole, which
/// is why this is more than a swap.
///
/// # Safety
/// `oap` must point to a live `oparg_T`.
unsafe fn order_region(oap: *mut oparg_T) {
    unsafe {
        if lt((*oap).start, (*curwin.get()).w_cursor) {
            if !VIsual_active.get() {
                if hasFolding(
                    curwin.get(),
                    (*oap).start.lnum,
                    &raw mut (*oap).start.lnum,
                    ::core::ptr::null_mut(),
                ) {
                    (*oap).start.col = 0;
                }
                if ((*curwin.get()).w_cursor.col > 0
                    || (*oap).inclusive
                    || (*oap).motion_type == kMTLineWise)
                    && hasFolding(
                        curwin.get(),
                        (*curwin.get()).w_cursor.lnum,
                        ::core::ptr::null_mut(),
                        &raw mut (*curwin.get()).w_cursor.lnum,
                    )
                {
                    (*curwin.get()).w_cursor.col = get_cursor_line_len();
                }
            }
            (*oap).end = (*curwin.get()).w_cursor;
            (*curwin.get()).w_cursor = (*oap).start;
            // `w_virtcol` was updated for the old position and is not
            // recomputed automatically when the cursor goes back.
            (*curwin.get()).w_valid &= !VALID_VIRTCOL;
        } else {
            if !VIsual_active.get() && (*oap).motion_type == kMTLineWise {
                if hasFolding(
                    curwin.get(),
                    (*curwin.get()).w_cursor.lnum,
                    &raw mut (*curwin.get()).w_cursor.lnum,
                    ::core::ptr::null_mut(),
                ) {
                    (*curwin.get()).w_cursor.col = 0;
                }
                if hasFolding(
                    curwin.get(),
                    (*oap).start.lnum,
                    ::core::ptr::null_mut(),
                    &raw mut (*oap).start.lnum,
                ) {
                    (*oap).start.col = ml_get_len((*oap).start.lnum);
                }
            }
            (*oap).end = (*oap).start;
            (*oap).start = (*curwin.get()).w_cursor;
        }
    }
}

/// Record the selection's *size* so that `gv` can reselect it and `.` can
/// build one like it.
///
/// # Safety
/// `cap` must point to a live `cmdarg_T`; a Visual selection must be active or
/// being replayed.
unsafe fn prepare_visual_redo(
    cap: *mut cmdarg_T,
    oap: *mut oparg_T,
    gui_yank: bool,
    redo_yank: bool,
) {
    unsafe {
        if !redo_VIsual_busy.get() && !gui_yank {
            resel_VIsual_mode.set(VIsual_mode.get());
            if (*curwin.get()).w_curswant == MAXCOL {
                resel_VIsual_vcol.set(MAXCOL);
            } else {
                if VIsual_mode.get() != Ctrl_V {
                    getvvcol(
                        curwin.get(),
                        &raw mut (*oap).end,
                        ::core::ptr::null_mut(),
                        ::core::ptr::null_mut(),
                        &raw mut (*oap).end_vcol,
                    );
                }
                if VIsual_mode.get() == Ctrl_V || (*oap).line_count <= 1 {
                    // A block, or a one-line region: the size is a width.
                    if VIsual_mode.get() != Ctrl_V {
                        getvvcol(
                            curwin.get(),
                            &raw mut (*oap).start,
                            &raw mut (*oap).start_vcol,
                            ::core::ptr::null_mut(),
                            ::core::ptr::null_mut(),
                        );
                    }
                    resel_VIsual_vcol.set((*oap).end_vcol - (*oap).start_vcol + 1);
                } else {
                    // Several lines: the size is the end column.
                    resel_VIsual_vcol.set((*oap).end_vcol);
                }
            }
            resel_VIsual_line_count.set((*oap).line_count);
        }

        let is_fold_op = matches!(
            (*oap).op_type,
            OP_FOLD
                | OP_FOLDOPEN
                | OP_FOLDOPENREC
                | OP_FOLDCLOSE
                | OP_FOLDCLOSEREC
                | OP_FOLDDEL
                | OP_FOLDDELREC
        );
        // A yank cannot be redone unless 'cpoptions' has `y`, and neither can
        // `:`.
        if !((redo_yank || (*oap).op_type != OP_YANK)
            && (*oap).op_type != OP_COLON
            && !is_fold_op
            && (*oap).motion_force == NUL)
        {
            return;
        }

        if (*cap).cmdchar == 'g' as c_int
            && ((*cap).nchar == 'n' as c_int || (*cap).nchar == 'N' as c_int)
        {
            // `gn`/`gN` carry their own region, so the whole command repeats.
            prep_redo(
                (*oap).regname,
                (*cap).count0,
                get_op_char((*oap).op_type),
                get_extra_op_char((*oap).op_type),
                (*oap).motion_force,
                (*cap).cmdchar,
                (*cap).nchar,
            );
        } else if !is_ex_cmdchar(cap) && (*cap).cmdchar != K_LUA {
            let opchar = get_op_char((*oap).op_type);
            let extra_opchar = get_extra_op_char((*oap).op_type);
            // Only `r` uses `nchar`; for anything else it would be the
            // operator's own second character.
            let mut nchar = if (*oap).op_type == OP_REPLACE {
                (*cap).nchar
            } else {
                NUL
            };
            // Undo what `nv_replace` did.
            if nchar == REPLACE_CR_NCHAR {
                nchar = CAR;
            } else if nchar == REPLACE_NL_NCHAR {
                nchar = NL;
            }

            if opchar == 'g' as c_int && extra_opchar == '@' as c_int {
                // `g@` also repeats the count, for 'operatorfunc'.
                prep_redo_num2(
                    (*oap).regname,
                    0,
                    NUL,
                    'v' as c_int,
                    (*cap).count0,
                    opchar,
                    extra_opchar,
                    nchar,
                );
            } else {
                prep_redo(
                    (*oap).regname,
                    0,
                    NUL,
                    'v' as c_int,
                    opchar,
                    extra_opchar,
                    nchar,
                );
            }
        }

        if !redo_VIsual_busy.get() {
            REDO_VISUAL.set(redo_VIsual_T {
                rv_mode: resel_VIsual_mode.get(),
                rv_vcol: resel_VIsual_vcol.get(),
                rv_line_count: resel_VIsual_line_count.get(),
                rv_count: (*cap).count0,
                rv_arg: (*cap).arg,
            });
        }
    }
}

/// Turn the Visual mode letter into a motion type, and switch Visual off.
///
/// Visual goes off *now* rather than after the operator so that the screen
/// update does not show inverted text. `OP_YANK`, `OP_COLON`, `OP_FUNCTION`
/// and `OP_FILTER` do not redraw by themselves, so they get one here.
///
/// # Safety
/// `cap` must point to a live `cmdarg_T`.
unsafe fn finish_visual_region(
    oap: *mut oparg_T,
    include_line_break: bool,
    gui_yank: bool,
    lbr_saved: c_int,
) {
    unsafe {
        // `inclusive` defaults to true; an end on a NUL (an empty line) makes
        // it false, which is what makes `d}P` and `v}dP` behave the same.
        if (*oap).motion_force == NUL || (*oap).motion_type == kMTLineWise {
            (*oap).inclusive = true;
        }
        if VIsual_mode.get() == 'V' as c_int {
            (*oap).motion_type = kMTLineWise;
        } else if VIsual_mode.get() == 'v' as c_int {
            (*oap).motion_type = kMTCharWise;
            if *ml_get_pos(&raw mut (*oap).end) as c_int == NUL
                && (include_line_break || virtual_op.get() == 0)
            {
                (*oap).inclusive = false;
                // Take the line break too, unless the operator only works on
                // whole lines anyway.
                if *p_sel.get() as c_int != 'o' as c_int
                    && !op_on_lines((*oap).op_type)
                    && (*oap).end.lnum < (*curbuf.get()).b_ml.ml_line_count
                {
                    (*oap).end.lnum += 1;
                    (*oap).end.col = 0;
                    (*oap).end.coladd = 0;
                    (*oap).line_count += 1;
                }
            }
        }

        redo_VIsual_busy.set(false);

        if !gui_yank {
            VIsual_active.set(false);
            setmouse();
            mouse_dragging.set(0);
            may_clear_cmdline();
            if ((*oap).op_type == OP_YANK
                || (*oap).op_type == OP_COLON
                || (*oap).op_type == OP_FUNCTION
                || (*oap).op_type == OP_FILTER)
                && (*oap).motion_force == NUL
            {
                restore_lbr(lbr_saved != 0);
                redraw_curbuf_later(UPD_INVERTED);
            }
        }
    }
}

/// An exclusive charwise end in column one belongs to the *previous* line.
///
/// And if the start is on or before that line's first non-blank, the operator
/// becomes linewise -- strange, but that is what vi does.
///
/// # Safety
/// `cap` must point to a live `cmdarg_T`.
unsafe fn adjust_region_end(cap: *mut cmdarg_T, oap: *mut oparg_T) {
    unsafe {
        if !((*oap).motion_type == kMTCharWise
            && !(*oap).inclusive
            && (*cap).retval & CA_NO_ADJ_OP_END as c_int == 0
            && (*oap).end.col == 0
            && (!(*oap).is_VIsual || *p_sel.get() as c_int == 'o' as c_int)
            && (*oap).line_count > 1)
        {
            (*oap).end_adjusted = false;
            return;
        }

        // Remembered, because the cursor column is restored differently after
        // an adjusted region.
        (*oap).end_adjusted = true;
        (*oap).line_count -= 1;
        (*oap).end.lnum -= 1;
        if inindent(0) {
            (*oap).motion_type = kMTLineWise;
        } else {
            (*oap).end.col = ml_get_len((*oap).end.lnum);
            if (*oap).end.col != 0 {
                (*oap).end.col -= 1;
                (*oap).inclusive = true;
            }
        }
    }
}

/// The switch: hand the region to the operator that was typed.
///
/// `empty_region_error` is 'cpoptions' `E` having refused an empty region;
/// every operator that would change text beeps instead. `lbr_saved` is
/// 'linebreak' as it was before the dispatcher turned it off -- the arms that
/// give control away (Insert mode, 'operatorfunc', an external filter) have to
/// put it back first, because the user is about to look at the screen.
///
/// # Safety
/// `cap` must point to a live `cmdarg_T` whose region is set up.
unsafe fn run_operator(
    cap: *mut cmdarg_T,
    oap: *mut oparg_T,
    empty_region_error: bool,
    gui_yank: bool,
    lbr_saved: c_int,
) {
    unsafe {
        /// Refuse an empty region: beep and drop the half-recorded `.`.
        unsafe fn refuse() {
            unsafe {
                vim_beep(kOptBoFlagOperator as ::core::ffi::c_uint);
                CancelRedo();
            }
        }

        match (*oap).op_type {
            OP_LSHIFT | OP_RSHIFT => {
                op_shift(oap, true, if (*oap).is_VIsual { (*cap).count1 } else { 1 });
                auto_format(false, true);
            }

            OP_JOIN_NS | OP_JOIN => {
                (*oap).line_count = (*oap).line_count.max(2);
                if (*curwin.get()).w_cursor.lnum + (*oap).line_count - 1
                    > (*curbuf.get()).b_ml.ml_line_count
                {
                    beep_flush();
                } else {
                    do_join(
                        (*oap).line_count as size_t,
                        (*oap).op_type == OP_JOIN,
                        true,
                        true,
                        true,
                    );
                    auto_format(false, true);
                }
            }

            OP_DELETE => {
                // Do not reselect now.
                VIsual_reselect.set(false_0);
                if empty_region_error {
                    refuse();
                } else {
                    op_delete(oap);
                    // Save the cursor line for undo if that has not happened.
                    if (*oap).motion_type == kMTLineWise
                        && has_format_option(FO_AUTO)
                        && u_save_cursor() == OK
                    {
                        auto_format(false, true);
                    }
                }
            }

            OP_YANK => {
                if empty_region_error {
                    if !gui_yank {
                        refuse();
                    }
                } else {
                    restore_lbr(lbr_saved != 0);
                    // `zy` yanks without the trailing white space.
                    (*oap).excl_tr_ws = (*cap).cmdchar == 'z' as c_int;
                    op_yank(oap, !gui_yank);
                }
                check_cursor_col(curwin.get());
            }

            OP_CHANGE => {
                VIsual_reselect.set(false_0);
                if empty_region_error {
                    refuse();
                } else {
                    run_change(cap, oap, lbr_saved);
                }
            }

            OP_FILTER => {
                if !vim_strchr(p_cpo.get(), CPO_FILTER).is_null() {
                    // Use whichever `!cmd` was last used.
                    AppendToRedobuff(c"!\r".as_ptr());
                } else {
                    // `do_bang` will put the command in the redo buffer.
                    bangredo.set(true);
                }
                // Falls through to the `:` handling below, as upstream does.
                indent_or_colon(oap);
            }
            OP_INDENT | OP_COLON => indent_or_colon(oap),

            OP_TILDE | OP_UPPER | OP_LOWER | OP_ROT13 => {
                if empty_region_error {
                    refuse();
                } else {
                    op_tilde(oap);
                }
                check_cursor_col(curwin.get());
            }

            OP_FORMAT => {
                if *(*curbuf.get()).b_p_fex as c_int != NUL {
                    op_formatexpr(oap);
                } else if *p_fp.get() as c_int != NUL || *(*curbuf.get()).b_p_fp as c_int != NUL {
                    // An external program.
                    op_colon(oap);
                } else {
                    op_format(oap, false);
                }
            }
            OP_FORMAT2 => op_format(oap, true),

            OP_FUNCTION => {
                // 'operatorfunc' may run another operator and overwrite the
                // recorded Visual area, so it is put back afterwards.
                let saved = REDO_VISUAL.get();
                restore_lbr(lbr_saved != 0);
                op_function(oap);
                REDO_VISUAL.set(saved);
            }

            OP_INSERT | OP_APPEND => {
                VIsual_reselect.set(false_0);
                if empty_region_error {
                    refuse();
                } else {
                    run_block_insert(cap, oap, lbr_saved);
                }
            }

            OP_REPLACE => {
                VIsual_reselect.set(false_0);
                if empty_region_error {
                    refuse();
                } else {
                    restore_lbr(lbr_saved != 0);
                    op_replace(oap, (*cap).nchar);
                }
            }

            OP_FOLD => {
                VIsual_reselect.set(false_0);
                foldCreate(curwin.get(), (*oap).start, (*oap).end);
            }
            OP_FOLDOPEN | OP_FOLDOPENREC | OP_FOLDCLOSE | OP_FOLDCLOSEREC => {
                VIsual_reselect.set(false_0);
                opFoldRange(
                    (*oap).start,
                    (*oap).end,
                    c_int::from((*oap).op_type == OP_FOLDOPEN || (*oap).op_type == OP_FOLDOPENREC),
                    c_int::from(
                        (*oap).op_type == OP_FOLDOPENREC || (*oap).op_type == OP_FOLDCLOSEREC,
                    ),
                    (*oap).is_VIsual,
                );
            }
            OP_FOLDDEL | OP_FOLDDELREC => {
                VIsual_reselect.set(false_0);
                deleteFold(
                    curwin.get(),
                    (*oap).start.lnum,
                    (*oap).end.lnum,
                    c_int::from((*oap).op_type == OP_FOLDDELREC),
                    (*oap).is_VIsual,
                );
            }

            OP_NR_ADD | OP_NR_SUB => {
                if empty_region_error {
                    refuse();
                } else {
                    // `op_addsub` reads `VIsual_active` to decide whether the
                    // region or the cursor line is meant, and this dispatcher
                    // has already switched it off.
                    VIsual_active.set(true);
                    restore_lbr(lbr_saved != 0);
                    op_addsub(
                        oap,
                        (*cap).count1 as linenr_T,
                        REDO_VISUAL.get().rv_arg != 0,
                    );
                    VIsual_active.set(false);
                }
                check_cursor_col(curwin.get());
            }

            _ => clearopbeep(oap),
        }
    }
}

/// `=` and `:` -- and `!`, which falls through to here.
///
/// With an empty 'equalprg' the indenting is done internally; otherwise the
/// region is handed to a `:` command line.
///
/// # Safety
/// `oap` must point to a live `oparg_T`.
unsafe fn indent_or_colon(oap: *mut oparg_T) {
    unsafe {
        if (*oap).op_type != OP_INDENT || *get_equalprg() as c_int != NUL {
            op_colon(oap);
            return;
        }
        if (*curbuf.get()).b_p_lisp != 0 {
            if use_indentexpr_for_lisp() {
                op_reindent(
                    oap,
                    Some(get_expr_indent as unsafe extern "C" fn() -> c_int),
                );
            } else {
                op_reindent(
                    oap,
                    Some(get_lisp_indent as unsafe extern "C" fn() -> c_int),
                );
            }
            return;
        }
        op_reindent(
            oap,
            if *(*curbuf.get()).b_p_inde as c_int != NUL {
                Some(get_expr_indent as unsafe extern "C" fn() -> c_int)
            } else {
                Some(get_c_indent as unsafe extern "C" fn() -> c_int)
            },
        );
    }
}

/// The `c` arm: run `op_change`, which enters Insert mode.
///
/// # Safety
/// `cap` must point to a live `cmdarg_T` whose region is set up.
unsafe fn run_change(cap: *mut cmdarg_T, oap: *mut oparg_T, lbr_saved: c_int) {
    unsafe {
        // A new edit command, not a restart. Remembering that is what makes
        // `i_CTRL-O` work with a mapping for Visual mode -- but only when the
        // key was not typed.
        let restart_edit_save = if KeyTyped.get() {
            0
        } else {
            restart_edit.get()
        };
        restart_edit.set(0);

        // The user is about to edit: 'linebreak' has to look as it did.
        restore_lbr(lbr_saved != 0);
        // Trigger TextChangedI.
        (*curbuf.get()).b_last_changedtick_i = buf_get_changedtick(curbuf.get());

        if op_change(oap) != 0 {
            // `edit()` returned because of a CTRL-O command.
            (*cap).retval |= CA_COMMAND_BUSY as c_int;
        }
        if restart_edit.get() == 0 {
            restart_edit.set(restart_edit_save);
        }
    }
}

/// The `I`/`A` arm: run `op_insert`, which enters Insert mode.
///
/// # Safety
/// `cap` must point to a live `cmdarg_T` whose region is set up.
unsafe fn run_block_insert(cap: *mut cmdarg_T, oap: *mut oparg_T, lbr_saved: c_int) {
    unsafe {
        let restart_edit_save = restart_edit.get();
        restart_edit.set(0);

        restore_lbr(lbr_saved != 0);
        (*curbuf.get()).b_last_changedtick_i = buf_get_changedtick(curbuf.get());

        op_insert(oap, (*cap).count1);

        // Back off again, so that formatting measures columns correctly.
        reset_lbr();
        auto_format(false, true);

        if restart_edit.get() == 0 {
            restart_edit.set(restart_edit_save);
        } else {
            (*cap).retval |= CA_COMMAND_BUSY as c_int;
        }
    }
}
