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

use crate::winlayer::{Buf, Win};
use core::ffi::{c_int, c_void};
use core::ops::{Deref, DerefMut};

use super::*;
use crate::keycodes::{K_COMMAND, K_LUA};
use crate::r#move::WinValid;
use crate::normal::{
    VisualMode, set_visual_active, set_visual_anchor, set_visual_mode, set_visual_select,
    visual_active, visual_anchor, visual_mode, visual_select,
};
use crate::option::cpo_has;
use crate::types::{CpoFlag, FoFlag, NUL, OK};

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

/// A `cmdarg_T` the caller has promised is live: the normal-mode command that
/// carried the operator here.
///
/// [`Op`]'s shape, for the other half of the pair `do_pending_operator` is
/// handed.
#[derive(Clone, Copy)]
struct Cmd(*mut cmdarg_T);

impl Cmd {
    /// # Safety
    /// `cap` must stay a live `cmdarg_T` for as long as the value is used.
    #[inline(always)]
    const unsafe fn new(cap: *mut cmdarg_T) -> Self {
        Self(cap)
    }
}

impl Deref for Cmd {
    type Target = cmdarg_T;

    #[inline(always)]
    fn deref(&self) -> &cmdarg_T {
        // SAFETY: the constructor's promise -- a live `cmdarg_T`.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Cmd {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut cmdarg_T {
        // SAFETY: the constructor's promise -- a live `cmdarg_T`. The borrow
        // lasts only as long as the field access that asked for it.
        unsafe { &mut *self.0 }
    }
}

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
fn is_ex_cmdchar(cap: Cmd) -> bool {
    cap.cmdchar == ':' as c_int || cap.cmdchar == K_COMMAND
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
    // SAFETY: the caller's promise -- a live `cmdarg_T` whose `oap` is a live
    // `oparg_T`. The two wrappers carry that promise on from here, so every
    // field access below is the compiler's business rather than a note.
    let mut cap = unsafe { Cmd::new(cap) };
    let mut oap = unsafe { Op::new(cap.oap) };
    let lbr_saved = cur_win().w_onebuf_opt.wo_lbr;
    let old_cursor = cur_win().w_cursor;

    if (!finish_op.get() && !visual_active()) || oap.op_type == OP_NOP {
        restore_lbr(lbr_saved != 0);
        return;
    }

    // A yank can be redone when 'cpoptions' has `y`, but never the one the
    // clipboard does for itself.
    let redo_yank = cpo_has(CpoFlag::YANK) && !gui_yank;

    // Unwanted line breaks would move every column measured below.
    reset_lbr();
    oap.is_VIsual = visual_active();
    apply_motion_force(oap);
    record_operator_redo(cap, oap, redo_yank);

    let mut include_line_break = false;
    if redo_VIsual_busy.get() {
        resume_redo_visual(cap, oap);
    } else if visual_active() {
        include_line_break = start_visual_region(oap, gui_yank);
    }

    order_region(oap);

    // Just in case lines were deleted that make the position invalid.
    // SAFETY: a live buffer and a live position.
    unsafe { check_pos(cur_win().w_buffer, &mut oap.end) };
    oap.line_count = oap.end.lnum - oap.start.lnum + 1;
    // Set before `VIsual_active` is reset below.
    // SAFETY: a live window.
    let virt = unsafe { virtual_active(curwin.get()) };
    virtual_op.set(Some(virt));

    if visual_active() || redo_VIsual_busy.get() {
        get_op_vcol(oap, REDO_VISUAL.get().rv_vcol, true);
        prepare_visual_redo(cap, oap, gui_yank, redo_yank);
        finish_visual_region(oap, include_line_break, gui_yank, lbr_saved);
    }

    // Include the trailing byte of a multi-byte character.
    if oap.inclusive {
        // SAFETY: `oap.end` is a position of the current buffer.
        let l = unsafe { utfc_ptr2len(ml_get_pos(oap.end().raw())) };
        if l > 1 {
            oap.end.col += l - 1;
        }
    }
    cur_win().w_set_curswant = true;

    // `empty` is set when start and end are the same. `inclusive` affects
    // that too, unless yanking with the end on a NUL.
    // SAFETY: the `gchar_pos` reads a live position of the current buffer,
    // and only when the operator is a yank -- the chain is left as it is so
    // that it stays as conditional as upstream wrote it.
    oap.empty = oap.motion_type != kMTLineWise
        && (!oap.inclusive
            || (oap.op_type == OP_YANK && unsafe { gchar_pos(oap.end().raw()) } == NUL))
        && equalpos(oap.start, oap.end)
        && !(op_virtual() && oap.start.coladd != oap.end.coladd);
    // For delete, change and yank it is an error to operate on an empty
    // region when 'cpoptions' has `E` (Vi compatible).
    let empty_region_error = oap.empty && cpo_has(CpoFlag::EMPTYREGION);

    // Force a redraw for an empty Visual region, an unmodifiable buffer,
    // or a fold: none of those will redraw by themselves.
    if oap.is_VIsual && (oap.empty || cur_buf().b_p_ma == 0 || oap.op_type == OP_FOLD) {
        restore_lbr(lbr_saved != 0);
        // SAFETY: touches only the current buffer's windows.
        redraw_curbuf_later(UPD_INVERTED);
    }

    adjust_region_end(cap, oap);
    run_operator(cap, oap, empty_region_error, gui_yank, lbr_saved);

    virtual_op.set(None);
    if gui_yank {
        cur_win().w_cursor = old_cursor;
    } else if p_sol.get() == 0
        && oap.motion_type == kMTLineWise
        && !oap.end_adjusted
        && (oap.op_type == OP_LSHIFT || oap.op_type == OP_RSHIFT || oap.op_type == OP_DELETE)
    {
        // 'startofline' is off: go back to the column the command started
        // in.
        reset_lbr();
        cur_win().w_curswant = old_col;
        cur_win().coladvance(cur_win().w_curswant);
    }
    // SAFETY: a live `oparg_T`.
    unsafe { clearop(oap.raw()) };
    motion_force.set(NUL);

    restore_lbr(lbr_saved != 0);
}

/// `v`, `V` or CTRL-V typed between the operator and its motion.
fn apply_motion_force(mut oap: Op) {
    if oap.motion_force == 'V' as c_int {
        oap.motion_type = kMTLineWise;
    } else if oap.motion_force == 'v' as c_int {
        if oap.motion_type == kMTLineWise {
            // A linewise motion never set `inclusive`; "exclusive" is the
            // consistent reading, and makes `dvj` behave.
            oap.inclusive = false;
        } else if oap.motion_type == kMTCharWise {
            oap.inclusive = !oap.inclusive;
        }
        oap.motion_type = kMTCharWise;
    } else if oap.motion_force == Ctrl_V {
        // Turn a line- or charwise motion into a Visual block.
        if !visual_active() {
            set_visual_active(true);
            set_visual_anchor(oap.start);
        }
        set_visual_mode(VisualMode::BLOCK);
        set_visual_select(false);
        VIsual_reselect.set(0);
    }
}

/// Put the command in the redo buffer, so that `.` repeats it.
///
/// Yank is only redoable under 'cpoptions' `y`, `zf` never is, and neither is
/// any of the fold operators; a search or a `:` command has to have its own
/// text appended so that the repeat really is the same command.
fn record_operator_redo(cap: Cmd, oap: Op, redo_yank: bool) {
    let is_fold_op = matches!(
        oap.op_type,
        OP_FOLD
            | OP_FOLDOPEN
            | OP_FOLDOPENREC
            | OP_FOLDCLOSE
            | OP_FOLDCLOSEREC
            | OP_FOLDDEL
            | OP_FOLDDELREC
    );
    let replayable = (redo_yank || oap.op_type != OP_YANK)
        && (!visual_active()
            || oap.motion_force != 0
            // Also redo Operator-pending Visual mode mappings.
            || ((is_ex_cmdchar(cap) || cap.cmdchar == K_LUA) && oap.op_type != OP_COLON))
        && cap.cmdchar != 'D' as c_int
        && !is_fold_op;
    if !replayable {
        return;
    }

    prep_redo(
        oap.regname,
        cap.count0,
        get_op_char(oap.op_type),
        get_extra_op_char(oap.op_type),
        oap.motion_force,
        cap.cmdchar,
        cap.nchar,
    );

    // SAFETY: every call below only appends to the redo buffer, and the two
    // strings handed to it are `cap.searchbuf` and `repeat_cmdline`, both
    // NUL-terminated for as long as the editor owns them.
    if cap.cmdchar == '/' as c_int || cap.cmdchar == '?' as c_int {
        // A search: without 'cpoptions' `r` the pattern goes in too, so
        // that the repeat really is the same command.
        if !cpo_has(CpoFlag::REDO) {
            unsafe { append_to_redobuff_literally(cap.searchbuf, -1) };
        }
        unsafe { append_to_redobuff(c"\n".as_ptr()) };
    } else if is_ex_cmdchar(cap) {
        // `do_cmdline` stored the first typed line in `repeat_cmdline`.
        // When several lines were typed, repeating is not possible.
        let line = repeat_cmdline.get();
        if line.is_null() {
            unsafe { reset_redobuff() };
        } else {
            if cap.cmdchar == ':' as c_int {
                unsafe { append_to_redobuff_literally(line, -1) };
            } else {
                unsafe { append_to_redobuff_keys(line) };
            }
            unsafe { append_to_redobuff(c"\n".as_ptr()) };
            unsafe { xfree(line as *mut c_void) };
            repeat_cmdline.set(::core::ptr::null_mut());
        }
    } else if cap.cmdchar == K_LUA {
        append_to_redobuff_number(repeat_luaref.get() as c_int);
        unsafe { append_to_redobuff(c"\n".as_ptr()) };
    }
}

/// `.` replaying a Visual operator: rebuild a region of the recorded size at
/// the cursor.
fn resume_redo_visual(mut cap: Cmd, mut oap: Op) {
    let redo = REDO_VISUAL.get();
    oap.start = cur_win().w_cursor;
    cur_win().w_cursor.lnum += redo.rv_line_count - 1;
    cur_win().w_cursor.lnum = cur_win().w_cursor.lnum.min(cur_buf().line_count());
    set_visual_mode(VisualMode::from_raw(redo.rv_mode));

    if redo.rv_vcol == MAXCOL || visual_mode().is_char() {
        if !visual_mode().is_char() {
            cur_win().w_curswant = MAXCOL;
        } else if redo.rv_line_count <= 1 {
            // A one-line charwise region is that many columns *from the
            // cursor*, not to a fixed column.
            // SAFETY: a live window.
            unsafe { validate_virtcol(curwin.get()) };
            cur_win().w_curswant = cur_win().w_virtcol + redo.rv_vcol - 1;
        } else {
            cur_win().w_curswant = redo.rv_vcol;
        }
        cur_win().coladvance(cur_win().w_curswant);
    }
    cap.count0 = redo.rv_count;
    cap.count1 = if cap.count0 == 0 { 1 } else { cap.count0 };
}

/// The operator was typed after a selection: the region is the selection.
///
/// Answers `include_line_break`, which 'selection' `exclusive` sets when the
/// backed-off end lands on a line break.
///
/// A Visual selection must be active.
fn start_visual_region(mut oap: Op, gui_yank: bool) -> bool {
    let mut include_line_break = false;

    if !gui_yank {
        // Keep the area for `'<`/`'>` and for `gv`.
        cur_buf().b_visual.vi_start = visual_anchor();
        cur_buf().b_visual.vi_end = cur_win().w_cursor;
        cur_buf().b_visual.vi_mode = visual_mode().raw();
        restore_visual_mode();
        cur_buf().b_visual.vi_curswant = cur_win().w_curswant;
        cur_buf().b_visual_mode_eval = visual_mode().raw();
    }

    // In Select mode a linewise selection is operated on like a charwise
    // one. `gH<Del>`, which deletes the last line, is the exception.
    // SAFETY: both lines are the current buffer's -- one holds the cursor,
    // the other the Visual anchor. `unadjust_for_sel` only moves the cursor.
    if visual_select() && visual_mode().is_line() && oap.op_type != OP_DELETE {
        if lt(visual_anchor(), cur_win().w_cursor) {
            set_visual_anchor(visual_anchor().with_col(0));
            cur_win().w_cursor.col = ml_get_len(cur_win().w_cursor.lnum);
        } else {
            cur_win().w_cursor.col = 0;
            let end = ml_get_len(visual_anchor().lnum);
            set_visual_anchor(visual_anchor().with_col(end));
        }
        set_visual_mode(VisualMode::CHAR);
    } else if visual_mode().is_char() {
        // 'selection' "exclusive": back off one character.
        include_line_break = unsafe { unadjust_for_sel() };
    }

    oap.start = visual_anchor();
    if visual_mode().is_line() {
        oap.start.col = 0;
        oap.start.coladd = 0;
    }
    include_line_break
}

/// Put `oap.start` at the first position of the region and `oap.end` at the
/// last, with the cursor on the start.
///
/// Outside Visual mode a closed fold at either end is swallowed whole, which
/// is why this is more than a swap.
fn order_region(mut oap: Op) {
    let win = cur_win();
    if lt(oap.start, cur_win().w_cursor) {
        if !visual_active() {
            if let Some(first) = win.fold_first(oap.start.lnum) {
                oap.start.lnum = first;
                oap.start.col = 0;
            }
            let past_start =
                cur_win().w_cursor.col > 0 || oap.inclusive || oap.motion_type == kMTLineWise;
            if past_start && let Some(last) = win.fold_end(cur_win().w_cursor.lnum) {
                cur_win().w_cursor.lnum = last;
                // SAFETY: the cursor line is a line of the buffer.
                cur_win().w_cursor.col = get_cursor_line_len();
            }
        }
        oap.end = cur_win().w_cursor;
        cur_win().w_cursor = oap.start;
        // `w_virtcol` was updated for the old position and is not
        // recomputed automatically when the cursor goes back.
        cur_win().w_valid.clear(WinValid::VIRTCOL);
    } else {
        if !visual_active() && oap.motion_type == kMTLineWise {
            if let Some(first) = win.fold_first(cur_win().w_cursor.lnum) {
                cur_win().w_cursor.lnum = first;
                cur_win().w_cursor.col = 0;
            }
            if let Some(last) = win.fold_end(oap.start.lnum) {
                oap.start.lnum = last;
                // SAFETY: a line of the current buffer.
                oap.start.col = ml_get_len(last);
            }
        }
        oap.end = oap.start;
        oap.start = cur_win().w_cursor;
    }
}

/// Record the selection's *size* so that `gv` can reselect it and `.` can
/// build one like it.
///
/// A Visual selection must be active or being replayed.
fn prepare_visual_redo(cap: Cmd, mut oap: Op, gui_yank: bool, redo_yank: bool) {
    if !redo_VIsual_busy.get() && !gui_yank {
        resel_VIsual_mode.set(visual_mode());
        if cur_win().w_curswant == MAXCOL {
            resel_VIsual_vcol.set(MAXCOL);
        } else {
            if !visual_mode().is_block() {
                oap.end_vcol = cur_win().virtual_vcol_span(oap.end()).1;
            }
            if visual_mode().is_block() || oap.line_count <= 1 {
                // A block, or a one-line region: the size is a width.
                if !visual_mode().is_block() {
                    oap.start_vcol = cur_win().virtual_vcol(oap.start());
                }
                resel_VIsual_vcol.set(oap.end_vcol - oap.start_vcol + 1);
            } else {
                // Several lines: the size is the end column.
                resel_VIsual_vcol.set(oap.end_vcol);
            }
        }
        resel_VIsual_line_count.set(oap.line_count);
    }

    let is_fold_op = matches!(
        oap.op_type,
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
    if !((redo_yank || oap.op_type != OP_YANK)
        && oap.op_type != OP_COLON
        && !is_fold_op
        && oap.motion_force == NUL)
    {
        return;
    }

    if cap.cmdchar == 'g' as c_int && (cap.nchar == 'n' as c_int || cap.nchar == 'N' as c_int) {
        // `gn`/`gN` carry their own region, so the whole command repeats.
        prep_redo(
            oap.regname,
            cap.count0,
            get_op_char(oap.op_type),
            get_extra_op_char(oap.op_type),
            oap.motion_force,
            cap.cmdchar,
            cap.nchar,
        );
    } else if !is_ex_cmdchar(cap) && cap.cmdchar != K_LUA {
        let opchar = get_op_char(oap.op_type);
        let extra_opchar = get_extra_op_char(oap.op_type);
        // Only `r` uses `nchar`; for anything else it would be the
        // operator's own second character.
        let mut nchar = if oap.op_type == OP_REPLACE {
            cap.nchar
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
                oap.regname,
                0,
                NUL,
                'v' as c_int,
                cap.count0,
                opchar,
                extra_opchar,
                nchar,
            );
        } else {
            prep_redo(
                oap.regname,
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
            rv_mode: resel_VIsual_mode.get().raw(),
            rv_vcol: resel_VIsual_vcol.get(),
            rv_line_count: resel_VIsual_line_count.get(),
            rv_count: cap.count0,
            rv_arg: cap.arg,
        });
    }
}

/// Turn the Visual mode letter into a motion type, and switch Visual off.
///
/// Visual goes off *now* rather than after the operator so that the screen
/// update does not show inverted text. `OP_YANK`, `OP_COLON`, `OP_FUNCTION`
/// and `OP_FILTER` do not redraw by themselves, so they get one here.
fn finish_visual_region(mut oap: Op, include_line_break: bool, gui_yank: bool, lbr_saved: c_int) {
    // `inclusive` defaults to true; an end on a NUL (an empty line) makes
    // it false, which is what makes `d}P` and `v}dP` behave the same.
    if oap.motion_force == NUL || oap.motion_type == kMTLineWise {
        oap.inclusive = true;
    }
    if visual_mode().is_line() {
        oap.motion_type = kMTLineWise;
    } else if visual_mode().is_char() {
        oap.motion_type = kMTCharWise;
        // SAFETY: `oap.end` is a position of the current buffer, and 'sel'
        // is a NUL-terminated option string.
        let ends_on_nul = unsafe { *ml_get_pos(oap.end().raw()) } as c_int == NUL;
        if ends_on_nul && (include_line_break || !op_virtual()) {
            oap.inclusive = false;
            // Take the line break too, unless the operator only works on
            // whole lines anyway.
            if unsafe { *p_sel.get() } as c_int != 'o' as c_int
                && !op_on_lines(oap.op_type)
                && oap.end.lnum < cur_buf().line_count()
            {
                oap.end.lnum += 1;
                oap.end.col = 0;
                oap.end.coladd = 0;
                oap.line_count += 1;
            }
        }
    }

    redo_VIsual_busy.set(false);

    if !gui_yank {
        set_visual_active(false);
        setmouse();
        mouse_dragging.set(0);
        may_clear_cmdline();
        if (oap.op_type == OP_YANK
            || oap.op_type == OP_COLON
            || oap.op_type == OP_FUNCTION
            || oap.op_type == OP_FILTER)
            && oap.motion_force == NUL
        {
            restore_lbr(lbr_saved != 0);
            // SAFETY: touches only the current buffer's windows.
            redraw_curbuf_later(UPD_INVERTED);
        }
    }
}

/// An exclusive charwise end in column one belongs to the *previous* line.
///
/// And if the start is on or before that line's first non-blank, the operator
/// becomes linewise -- strange, but that is what vi does.
fn adjust_region_end(cap: Cmd, mut oap: Op) {
    // SAFETY: 'sel' is a NUL-terminated option string.
    if !(oap.motion_type == kMTCharWise
        && !oap.inclusive
        && cap.retval & CA_NO_ADJ_OP_END as c_int == 0
        && oap.end.col == 0
        && (!oap.is_VIsual || unsafe { *p_sel.get() } as c_int == 'o' as c_int)
        && oap.line_count > 1)
    {
        oap.end_adjusted = false;
        return;
    }

    // Remembered, because the cursor column is restored differently after
    // an adjusted region.
    oap.end_adjusted = true;
    oap.line_count -= 1;
    oap.end.lnum -= 1;
    if unsafe { inindent(0) } {
        oap.motion_type = kMTLineWise;
    } else {
        // SAFETY: a line of the current buffer.
        oap.end.col = ml_get_len(oap.end.lnum);
        if oap.end.col != 0 {
            oap.end.col -= 1;
            oap.inclusive = true;
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
fn run_operator(
    mut cap: Cmd,
    mut oap: Op,
    empty_region_error: bool,
    gui_yank: bool,
    lbr_saved: c_int,
) {
    /// Refuse an empty region: beep and drop the half-recorded `.`.
    fn refuse() {
        // SAFETY: neither touches anything but editor-wide state.
        unsafe {
            vim_beep(kOptBoFlagOperator as ::core::ffi::c_uint);
            cancel_redo();
        }
    }

    // SAFETY: every operator below is handed the same live `oparg_T` and the
    // current window, which is exactly what each of them asks for.
    match oap.op_type {
        OP_LSHIFT | OP_RSHIFT => {
            let amount = if oap.is_VIsual { cap.count1 } else { 1 };
            unsafe { op_shift(oap.raw(), true, amount) };
            unsafe { auto_format(false, true) };
        }

        OP_JOIN_NS | OP_JOIN => {
            oap.line_count = oap.line_count.max(2);
            if cur_win().w_cursor.lnum + oap.line_count - 1 > cur_buf().line_count() {
                beep_flush();
            } else {
                let count = oap.line_count as size_t;
                unsafe { do_join(count, oap.op_type == OP_JOIN, true, true, true) };
                unsafe { auto_format(false, true) };
            }
        }

        OP_DELETE => {
            // Do not reselect now.
            VIsual_reselect.set(0);
            if empty_region_error {
                refuse();
            } else {
                // Nothing to do about a refusal: the message is out and
                // the buffer is untouched.
                let _ = unsafe { op_delete(oap.raw()) };
                // Save the cursor line for undo if that has not happened.
                if oap.motion_type == kMTLineWise
                    && has_format_option(FoFlag::AUTO)
                    && u_save_cursor() == OK
                {
                    unsafe { auto_format(false, true) };
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
                oap.excl_tr_ws = cap.cmdchar == 'z' as c_int;
                unsafe { op_yank(oap.raw(), !gui_yank) };
            }
            unsafe { check_cursor_col(curwin.get()) };
        }

        OP_CHANGE => {
            VIsual_reselect.set(0);
            if empty_region_error {
                refuse();
            } else {
                run_change(cap, oap, lbr_saved);
            }
        }

        OP_FILTER => {
            if cpo_has(CpoFlag::FILTER) {
                // Use whichever `!cmd` was last used.
                unsafe { append_to_redobuff(c"!\r".as_ptr()) };
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
                unsafe { op_tilde(oap.raw()) };
            }
            unsafe { check_cursor_col(curwin.get()) };
        }

        OP_FORMAT => {
            if unsafe { *cur_buf().b_p_fex } as c_int != NUL {
                unsafe { op_formatexpr(oap.raw()) };
            } else if unsafe { *p_fp.get() } as c_int != NUL
                || unsafe { *cur_buf().b_p_fp } as c_int != NUL
            {
                // An external program.
                unsafe { op_colon(oap.raw()) };
            } else {
                unsafe { op_format(oap.raw(), false) };
            }
        }
        OP_FORMAT2 => unsafe { op_format(oap.raw(), true) },

        OP_FUNCTION => {
            // 'operatorfunc' may run another operator and overwrite the
            // recorded Visual area, so it is put back afterwards.
            let saved = REDO_VISUAL.get();
            restore_lbr(lbr_saved != 0);
            unsafe { op_function(oap.raw()) };
            REDO_VISUAL.set(saved);
        }

        OP_INSERT | OP_APPEND => {
            VIsual_reselect.set(0);
            if empty_region_error {
                refuse();
            } else {
                run_block_insert(cap, oap, lbr_saved);
            }
        }

        OP_REPLACE => {
            VIsual_reselect.set(0);
            if empty_region_error {
                refuse();
            } else {
                restore_lbr(lbr_saved != 0);
                unsafe { op_replace(oap.raw(), cap.nchar) };
            }
        }

        OP_FOLD => {
            VIsual_reselect.set(0);
            unsafe { fold_create(curwin.get(), oap.start, oap.end) };
        }
        OP_FOLDOPEN | OP_FOLDOPENREC | OP_FOLDCLOSE | OP_FOLDCLOSEREC => {
            VIsual_reselect.set(0);
            let opening = oap.op_type == OP_FOLDOPEN || oap.op_type == OP_FOLDOPENREC;
            let recursive = oap.op_type == OP_FOLDOPENREC || oap.op_type == OP_FOLDCLOSEREC;
            let (start, end, visual) = (oap.start, oap.end, oap.is_VIsual);
            let (opening, recursive) = (c_int::from(opening), c_int::from(recursive));
            unsafe { op_fold_range(start, end, opening, recursive, visual) };
        }
        OP_FOLDDEL | OP_FOLDDELREC => {
            VIsual_reselect.set(0);
            let recursive = c_int::from(oap.op_type == OP_FOLDDELREC);
            let (first, last, visual) = (oap.start.lnum, oap.end.lnum, oap.is_VIsual);
            unsafe { delete_fold(curwin.get(), first, last, recursive, visual) };
        }

        OP_NR_ADD | OP_NR_SUB => {
            if empty_region_error {
                refuse();
            } else {
                // `op_addsub` reads `VIsual_active` to decide whether the
                // region or the cursor line is meant, and this dispatcher
                // has already switched it off.
                set_visual_active(true);
                restore_lbr(lbr_saved != 0);
                let (count, g) = (cap.count1 as linenr_T, REDO_VISUAL.get().rv_arg != 0);
                unsafe { op_addsub(oap.raw(), count, g) };
                set_visual_active(false);
            }
            unsafe { check_cursor_col(curwin.get()) };
        }

        _ => unsafe { clearopbeep(oap.raw()) },
    }
}

/// `=` and `:` -- and `!`, which falls through to here.
///
/// With an empty 'equalprg' the indenting is done internally; otherwise the
/// region is handed to a `:` command line.
fn indent_or_colon(oap: Op) {
    // SAFETY: a live `oparg_T` describing a region of the current buffer, and
    // 'equalprg'/'indentexpr' are NUL-terminated option strings.
    if oap.op_type != OP_INDENT || unsafe { *get_equalprg() } as c_int != NUL {
        unsafe { op_colon(oap.raw()) };
        return;
    }
    if cur_buf().b_p_lisp != 0 {
        let indent = if unsafe { use_indentexpr_for_lisp() } {
            get_expr_indent as unsafe fn() -> c_int
        } else {
            get_lisp_indent as unsafe fn() -> c_int
        };
        unsafe { op_reindent(oap.raw(), Some(indent)) };
        return;
    }
    let indent = if unsafe { *cur_buf().b_p_inde } as c_int != NUL {
        get_expr_indent as unsafe fn() -> c_int
    } else {
        get_c_indent as unsafe fn() -> c_int
    };
    unsafe { op_reindent(oap.raw(), Some(indent)) };
}

/// The `c` arm: run `op_change`, which enters Insert mode.
fn run_change(mut cap: Cmd, oap: Op, lbr_saved: c_int) {
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
    // SAFETY: a live buffer, and a live `oparg_T` whose region is set up.
    cur_buf().b_last_changedtick_i = unsafe { buf_get_changedtick(curbuf.get()) };

    if unsafe { op_change(oap.raw()) } != 0 {
        // `edit()` returned because of a CTRL-O command.
        cap.retval |= CA_COMMAND_BUSY as c_int;
    }
    if restart_edit.get() == 0 {
        restart_edit.set(restart_edit_save);
    }
}

/// The `I`/`A` arm: run `op_insert`, which enters Insert mode.
fn run_block_insert(mut cap: Cmd, oap: Op, lbr_saved: c_int) {
    let restart_edit_save = restart_edit.get();
    restart_edit.set(0);

    restore_lbr(lbr_saved != 0);
    // SAFETY: a live buffer, and a live `oparg_T` whose region is set up.
    cur_buf().b_last_changedtick_i = unsafe { buf_get_changedtick(curbuf.get()) };

    unsafe { op_insert(oap.raw(), cap.count1) };

    // Back off again, so that formatting measures columns correctly.
    reset_lbr();
    unsafe { auto_format(false, true) };

    if restart_edit.get() == 0 {
        restart_edit.set(restart_edit_save);
    } else {
        cap.retval |= CA_COMMAND_BUSY as c_int;
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
