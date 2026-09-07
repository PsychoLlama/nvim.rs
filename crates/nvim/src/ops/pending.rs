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
//! which is why `OpType::Function` saves and restores it around 'operatorfunc'
//! (the callback may run another operator and overwrite it).

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::keycodes::Key;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_int, c_void};
use core::ops::{Deref, DerefMut};

use super::*;
use crate::r#move::WinValid;
use crate::normal::{
    VisualMode, set_visual_active, set_visual_anchor, set_visual_mode, set_visual_select,
    visual_active, visual_anchor, visual_mode, visual_select,
};
use crate::option::cpo_has;
use crate::types::{CpoFlag, FoFlag, NUL};

/// The Visual area a `.` replays: its mode and size, not its position.
///
/// A `static` inside `do_pending_operator` in C. Process-wide on purpose --
/// `.` after `viwd` deletes the same *number of characters* at the cursor.
static REDO_VISUAL: GlobalCell<RedoVisual> = GlobalCell::new(RedoVisual {
    rv_mode: NUL,
    rv_line_count: 0,
    rv_vcol: 0,
    rv_count: 0,
    rv_arg: 0,
});

/// A `CmdArg` the caller has promised is live: the normal-mode command that
/// carried the operator here.
///
/// [`Op`]'s shape, for the other half of the pair `do_pending_operator` is
/// handed.
#[derive(Clone, Copy)]
struct Cmd(*mut CmdArg);

impl Cmd {
    /// # Safety
    /// `cmd_arg` must stay a live `CmdArg` for as long as the value is used.
    #[inline(always)]
    const unsafe fn new(cmd_arg: *mut CmdArg) -> Self {
        Self(cmd_arg)
    }
}

impl Deref for Cmd {
    type Target = CmdArg;

    #[inline(always)]
    fn deref(&self) -> &CmdArg {
        // SAFETY: the constructor's promise -- a live `CmdArg`.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Cmd {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut CmdArg {
        // SAFETY: the constructor's promise -- a live `CmdArg`. The borrow
        // lasts only as long as the field access that asked for it.
        unsafe { &mut *self.0 }
    }
}

/// Zero an `OpArg` between commands.
///
/// # Safety
/// `op` must point to a live `OpArg`.
pub unsafe fn clear_oparg(op: *mut OpArg) {
    unsafe { *op = OpArg::ZERO };
}

/// Was the operator reached through a command line rather than a key?
///
/// `:` and `<Cmd>` both arrive here as an operator over the Visual area.
fn is_ex_cmdchar(cmd_arg: Cmd) -> bool {
    cmd_arg.cmdchar == ':' as c_int || cmd_arg.cmdchar == Key::Command.code()
}

/// Run the operator that a motion (or a Visual selection) has just completed.
///
/// `old_col` is the column to return to when 'startofline' is off.
/// `gui_yank` marks the yank the clipboard does behind the user's back: it
/// must not clear the selection, redraw, or leave a `.` behind.
///
/// # Safety
/// `cmd_arg` must point to a live `CmdArg` whose `op` describes a region of the
/// current buffer.
pub unsafe fn do_pending_operator(cmd_arg: *mut CmdArg, old_col: c_int, gui_yank: bool) {
    // SAFETY: the caller's promise -- a live `CmdArg` whose `op` is a live
    // `OpArg`. The two wrappers carry that promise on from here, so every
    // field access below is the compiler's business rather than a note.
    let cmd_arg = unsafe { Cmd::new(cmd_arg) };
    let mut op = unsafe { Op::new(cmd_arg.oap) };
    let lbr_saved = Win::current().w_onebuf_opt.wo_lbr;
    let old_cursor = Win::current().w_cursor;

    if (!finish_op.get() && !visual_active()) || op.op_type == OpType::Nop {
        restore_lbr(lbr_saved != 0);
        return;
    }

    // A yank can be redone when 'cpoptions' has `y`, but never the one the
    // clipboard does for itself.
    let redo_yank = cpo_has(CpoFlag::YANK) && !gui_yank;

    // Unwanted line breaks would move every column measured below.
    reset_lbr();
    op.is_visual = visual_active();
    apply_motion_force(op);
    record_operator_redo(cmd_arg, op, redo_yank);

    let mut include_line_break = false;
    if redo_VIsual_busy.get() {
        resume_redo_visual(cmd_arg, op);
    } else if visual_active() {
        include_line_break = start_visual_region(op, gui_yank);
    }

    order_region(op);

    // Just in case lines were deleted that make the position invalid.
    check_pos(Win::current().buffer(), &mut op.end);
    op.line_count = op.end.lnum - op.start.lnum + 1;
    // Set before `VIsual_active` is reset below.
    // SAFETY: a live window.
    let virt = virtual_active(Win::current());
    virtual_op.set(Some(virt));

    if visual_active() || redo_VIsual_busy.get() {
        get_op_vcol(op, REDO_VISUAL.get().rv_vcol, true);
        prepare_visual_redo(cmd_arg, op, gui_yank, redo_yank);
        finish_visual_region(op, include_line_break, gui_yank, lbr_saved);
    }

    // Include the trailing byte of a multi-byte character.
    if op.inclusive {
        // SAFETY: `op.end` is a position of the current buffer.
        let l = unsafe { utfc_ptr2len(ml_get_pos(op.end().raw())) };
        if l > 1 {
            op.end.col += l - 1;
        }
    }
    Win::current().w_set_curswant = true;

    // `empty` is set when start and end are the same. `inclusive` affects
    // that too, unless yanking with the end on a NUL.
    // SAFETY: the `gchar_pos` reads a live position of the current buffer,
    // and only when the operator is a yank -- the chain is left as it is so
    // that it stays as conditional as upstream wrote it.
    op.empty = op.motion_type != kMTLineWise
        && (!op.inclusive
            || (op.op_type == OpType::Yank && unsafe { gchar_pos(op.end().raw()) } == NUL))
        && equalpos(op.start, op.end)
        && !(op_virtual() && op.start.coladd != op.end.coladd);
    // For delete, change and yank it is an error to operate on an empty
    // region when 'cpoptions' has `E` (Vi compatible).
    let empty_region_error = op.empty && cpo_has(CpoFlag::EMPTYREGION);

    // Force a redraw for an empty Visual region, an unmodifiable buffer,
    // or a fold: none of those will redraw by themselves.
    if op.is_visual && (op.empty || Buf::current().b_p_ma == 0 || op.op_type == OpType::Fold) {
        restore_lbr(lbr_saved != 0);
        // SAFETY: touches only the current buffer's windows.
        redraw_curbuf_later(UPD_INVERTED);
    }

    adjust_region_end(cmd_arg, op);
    run_operator(cmd_arg, op, empty_region_error, gui_yank, lbr_saved);

    virtual_op.set(None);
    if gui_yank {
        Win::current().w_cursor = old_cursor;
    } else if p_sol.get() == 0
        && op.motion_type == kMTLineWise
        && !op.end_adjusted
        && (op.op_type == OpType::Lshift
            || op.op_type == OpType::Rshift
            || op.op_type == OpType::Delete)
    {
        // 'startofline' is off: go back to the column the command started
        // in.
        reset_lbr();
        Win::current().w_curswant = old_col;
        Win::current().coladvance(Win::current().w_curswant);
    }
    // SAFETY: a live `OpArg`.
    unsafe { clearop(op.raw()) };
    motion_force.set(NUL);

    restore_lbr(lbr_saved != 0);
}

/// `v`, `V` or CTRL-V typed between the operator and its motion.
fn apply_motion_force(mut op: Op) {
    if op.motion_force == 'V' as c_int {
        op.motion_type = kMTLineWise;
    } else if op.motion_force == 'v' as c_int {
        if op.motion_type == kMTLineWise {
            // A linewise motion never set `inclusive`; "exclusive" is the
            // consistent reading, and makes `dvj` behave.
            op.inclusive = false;
        } else if op.motion_type == kMTCharWise {
            op.inclusive = !op.inclusive;
        }
        op.motion_type = kMTCharWise;
    } else if op.motion_force == Ctrl_V {
        // Turn a line- or charwise motion into a Visual block.
        if !visual_active() {
            set_visual_active(true);
            set_visual_anchor(op.start);
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
fn record_operator_redo(cmd_arg: Cmd, op: Op, redo_yank: bool) {
    let is_fold_op = matches!(
        op.op_type,
        OpType::Fold
            | OpType::Foldopen
            | OpType::Foldopenrec
            | OpType::Foldclose
            | OpType::Foldcloserec
            | OpType::Folddel
            | OpType::Folddelrec
    );
    let replayable = (redo_yank || op.op_type != OpType::Yank)
        && (!visual_active()
            || op.motion_force != 0
            // Also redo Operator-pending Visual mode mappings.
            || ((is_ex_cmdchar(cmd_arg) || cmd_arg.cmdchar == Key::Lua.code()) && op.op_type != OpType::Colon))
        && cmd_arg.cmdchar != 'D' as c_int
        && !is_fold_op;
    if !replayable {
        return;
    }

    prep_redo(
        op.regname,
        cmd_arg.count0,
        get_op_char(op.op_type),
        get_extra_op_char(op.op_type),
        op.motion_force,
        cmd_arg.cmdchar,
        cmd_arg.nchar,
    );

    // SAFETY: every call below only appends to the redo buffer, and the two
    // strings handed to it are `cmd_arg.searchbuf` and `repeat_cmdline`, both
    // NUL-terminated for as long as the editor owns them.
    if cmd_arg.cmdchar == '/' as c_int || cmd_arg.cmdchar == '?' as c_int {
        // A search: without 'cpoptions' `r` the pattern goes in too, so
        // that the repeat really is the same command.
        if !cpo_has(CpoFlag::REDO) {
            unsafe { append_to_redobuff_literally(cmd_arg.searchbuf, -1) };
        }
        unsafe { append_to_redobuff(c"\n".as_ptr()) };
    } else if is_ex_cmdchar(cmd_arg) {
        // `do_cmdline` stored the first typed line in `repeat_cmdline`.
        // When several lines were typed, repeating is not possible.
        let line = repeat_cmdline.get();
        if line.is_null() {
            unsafe { reset_redobuff() };
        } else {
            if cmd_arg.cmdchar == ':' as c_int {
                unsafe { append_to_redobuff_literally(line, -1) };
            } else {
                unsafe { append_to_redobuff_keys(line) };
            }
            unsafe { append_to_redobuff(c"\n".as_ptr()) };
            unsafe { xfree(line as *mut c_void) };
            repeat_cmdline.set(::core::ptr::null_mut());
        }
    } else if cmd_arg.cmdchar == Key::Lua.code() {
        append_to_redobuff_number(repeat_luaref.get() as c_int);
        unsafe { append_to_redobuff(c"\n".as_ptr()) };
    }
}

/// `.` replaying a Visual operator: rebuild a region of the recorded size at
/// the cursor.
fn resume_redo_visual(mut cmd_arg: Cmd, mut op: Op) {
    let redo = REDO_VISUAL.get();
    op.start = Win::current().w_cursor;
    Win::current().w_cursor.lnum += redo.rv_line_count - 1;
    Win::current().w_cursor.lnum = Win::current()
        .w_cursor
        .lnum
        .min(Buf::current().line_count());
    set_visual_mode(VisualMode::from_raw(redo.rv_mode));

    if redo.rv_vcol == MAXCOL || visual_mode().is_char() {
        if !visual_mode().is_char() {
            Win::current().w_curswant = MAXCOL;
        } else if redo.rv_line_count <= 1 {
            // A one-line charwise region is that many columns *from the
            // cursor*, not to a fixed column.
            validate_virtcol(Win::current());
            Win::current().w_curswant = Win::current().w_virtcol + redo.rv_vcol - 1;
        } else {
            Win::current().w_curswant = redo.rv_vcol;
        }
        Win::current().coladvance(Win::current().w_curswant);
    }
    cmd_arg.count0 = redo.rv_count;
    cmd_arg.count1 = if cmd_arg.count0 == 0 {
        1
    } else {
        cmd_arg.count0
    };
}

/// The operator was typed after a selection: the region is the selection.
///
/// Answers `include_line_break`, which 'selection' `exclusive` sets when the
/// backed-off end lands on a line break.
///
/// A Visual selection must be active.
fn start_visual_region(mut op: Op, gui_yank: bool) -> bool {
    let mut include_line_break = false;

    if !gui_yank {
        // Keep the area for `'<`/`'>` and for `gv`.
        Buf::current().b_visual.vi_start = visual_anchor();
        Buf::current().b_visual.vi_end = Win::current().w_cursor;
        Buf::current().b_visual.vi_mode = visual_mode().raw();
        restore_visual_mode();
        Buf::current().b_visual.vi_curswant = Win::current().w_curswant;
        Buf::current().b_visual_mode_eval = visual_mode().raw();
    }

    // In Select mode a linewise selection is operated on like a charwise
    // one. `gH<Del>`, which deletes the last line, is the exception.
    // SAFETY: both lines are the current buffer's -- one holds the cursor,
    // the other the Visual anchor. `unadjust_for_sel` only moves the cursor.
    if visual_select() && visual_mode().is_line() && op.op_type != OpType::Delete {
        if lt(visual_anchor(), Win::current().w_cursor) {
            set_visual_anchor(visual_anchor().with_col(0));
            Win::current().w_cursor.col = ml_get_len(Win::current().w_cursor.lnum);
        } else {
            Win::current().w_cursor.col = 0;
            let end = ml_get_len(visual_anchor().lnum);
            set_visual_anchor(visual_anchor().with_col(end));
        }
        set_visual_mode(VisualMode::CHAR);
    } else if visual_mode().is_char() {
        // 'selection' "exclusive": back off one character.
        include_line_break = unadjust_for_sel();
    }

    op.start = visual_anchor();
    if visual_mode().is_line() {
        op.start.col = 0;
        op.start.coladd = 0;
    }
    include_line_break
}

/// Put `op.start` at the first position of the region and `op.end` at the
/// last, with the cursor on the start.
///
/// Outside Visual mode a closed fold at either end is swallowed whole, which
/// is why this is more than a swap.
fn order_region(mut op: Op) {
    let win = Win::current();
    if lt(op.start, Win::current().w_cursor) {
        if !visual_active() {
            if let Some(first) = win.fold_first(op.start.lnum) {
                op.start.lnum = first;
                op.start.col = 0;
            }
            let past_start =
                Win::current().w_cursor.col > 0 || op.inclusive || op.motion_type == kMTLineWise;
            if past_start && let Some(last) = win.fold_end(Win::current().w_cursor.lnum) {
                Win::current().w_cursor.lnum = last;
                // SAFETY: the cursor line is a line of the buffer.
                Win::current().w_cursor.col = get_cursor_line_len();
            }
        }
        op.end = Win::current().w_cursor;
        Win::current().w_cursor = op.start;
        // `w_virtcol` was updated for the old position and is not
        // recomputed automatically when the cursor goes back.
        Win::current().w_valid.clear(WinValid::VIRTCOL);
    } else {
        if !visual_active() && op.motion_type == kMTLineWise {
            if let Some(first) = win.fold_first(Win::current().w_cursor.lnum) {
                Win::current().w_cursor.lnum = first;
                Win::current().w_cursor.col = 0;
            }
            if let Some(last) = win.fold_end(op.start.lnum) {
                op.start.lnum = last;
                // SAFETY: a line of the current buffer.
                op.start.col = ml_get_len(last);
            }
        }
        op.end = op.start;
        op.start = Win::current().w_cursor;
    }
}

/// Record the selection's *size* so that `gv` can reselect it and `.` can
/// build one like it.
///
/// A Visual selection must be active or being replayed.
fn prepare_visual_redo(cmd_arg: Cmd, mut op: Op, gui_yank: bool, redo_yank: bool) {
    if !redo_VIsual_busy.get() && !gui_yank {
        resel_VIsual_mode.set(visual_mode());
        if Win::current().w_curswant == MAXCOL {
            resel_VIsual_vcol.set(MAXCOL);
        } else {
            if !visual_mode().is_block() {
                op.end_vcol = Win::current().virtual_vcol_span(op.end()).1;
            }
            if visual_mode().is_block() || op.line_count <= 1 {
                // A block, or a one-line region: the size is a width.
                if !visual_mode().is_block() {
                    op.start_vcol = Win::current().virtual_vcol(op.start());
                }
                resel_VIsual_vcol.set(op.end_vcol - op.start_vcol + 1);
            } else {
                // Several lines: the size is the end column.
                resel_VIsual_vcol.set(op.end_vcol);
            }
        }
        resel_VIsual_line_count.set(op.line_count);
    }

    let is_fold_op = matches!(
        op.op_type,
        OpType::Fold
            | OpType::Foldopen
            | OpType::Foldopenrec
            | OpType::Foldclose
            | OpType::Foldcloserec
            | OpType::Folddel
            | OpType::Folddelrec
    );
    // A yank cannot be redone unless 'cpoptions' has `y`, and neither can
    // `:`.
    if !((redo_yank || op.op_type != OpType::Yank)
        && op.op_type != OpType::Colon
        && !is_fold_op
        && op.motion_force == NUL)
    {
        return;
    }

    if cmd_arg.cmdchar == 'g' as c_int
        && (cmd_arg.nchar == 'n' as c_int || cmd_arg.nchar == 'N' as c_int)
    {
        // `gn`/`gN` carry their own region, so the whole command repeats.
        prep_redo(
            op.regname,
            cmd_arg.count0,
            get_op_char(op.op_type),
            get_extra_op_char(op.op_type),
            op.motion_force,
            cmd_arg.cmdchar,
            cmd_arg.nchar,
        );
    } else if !is_ex_cmdchar(cmd_arg) && cmd_arg.cmdchar != Key::Lua.code() {
        let opchar = get_op_char(op.op_type);
        let extra_opchar = get_extra_op_char(op.op_type);
        // Only `r` uses `nchar`; for anything else it would be the
        // operator's own second character.
        let mut nchar = if op.op_type == OpType::Replace {
            cmd_arg.nchar
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
                op.regname,
                0,
                NUL,
                'v' as c_int,
                cmd_arg.count0,
                opchar,
                extra_opchar,
                nchar,
            );
        } else {
            prep_redo(
                op.regname,
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
        REDO_VISUAL.set(RedoVisual {
            rv_mode: resel_VIsual_mode.get().raw(),
            rv_vcol: resel_VIsual_vcol.get(),
            rv_line_count: resel_VIsual_line_count.get(),
            rv_count: cmd_arg.count0,
            rv_arg: cmd_arg.arg,
        });
    }
}

/// Turn the Visual mode letter into a motion type, and switch Visual off.
///
/// Visual goes off *now* rather than after the operator so that the screen
/// update does not show inverted text. `OpType::Yank`, `OpType::Colon`, `OpType::Function`
/// and `OpType::Filter` do not redraw by themselves, so they get one here.
fn finish_visual_region(mut op: Op, include_line_break: bool, gui_yank: bool, lbr_saved: c_int) {
    // `inclusive` defaults to true; an end on a NUL (an empty line) makes
    // it false, which is what makes `d}P` and `v}dP` behave the same.
    if op.motion_force == NUL || op.motion_type == kMTLineWise {
        op.inclusive = true;
    }
    if visual_mode().is_line() {
        op.motion_type = kMTLineWise;
    } else if visual_mode().is_char() {
        op.motion_type = kMTCharWise;
        // SAFETY: `op.end` is a position of the current buffer, and 'sel'
        // is a NUL-terminated option string.
        let ends_on_nul = unsafe { *ml_get_pos(op.end().raw()) } as c_int == NUL;
        if ends_on_nul && (include_line_break || !op_virtual()) {
            op.inclusive = false;
            // Take the line break too, unless the operator only works on
            // whole lines anyway.
            if unsafe { *p_sel.get() } as c_int != 'o' as c_int
                && !op_on_lines(op.op_type)
                && op.end.lnum < Buf::current().line_count()
            {
                op.end.lnum += 1;
                op.end.col = 0;
                op.end.coladd = 0;
                op.line_count += 1;
            }
        }
    }

    redo_VIsual_busy.set(false);

    if !gui_yank {
        set_visual_active(false);
        setmouse();
        mouse_dragging.set(0);
        may_clear_cmdline();
        if (op.op_type == OpType::Yank
            || op.op_type == OpType::Colon
            || op.op_type == OpType::Function
            || op.op_type == OpType::Filter)
            && op.motion_force == NUL
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
fn adjust_region_end(cmd_arg: Cmd, mut op: Op) {
    // SAFETY: 'sel' is a NUL-terminated option string.
    if !(op.motion_type == kMTCharWise
        && !op.inclusive
        && cmd_arg.retval & CA_NO_ADJ_OP_END as c_int == 0
        && op.end.col == 0
        && (!op.is_visual || unsafe { *p_sel.get() } as c_int == 'o' as c_int)
        && op.line_count > 1)
    {
        op.end_adjusted = false;
        return;
    }

    // Remembered, because the cursor column is restored differently after
    // an adjusted region.
    op.end_adjusted = true;
    op.line_count -= 1;
    op.end.lnum -= 1;
    if unsafe { inindent(0) } {
        op.motion_type = kMTLineWise;
    } else {
        // SAFETY: a line of the current buffer.
        op.end.col = ml_get_len(op.end.lnum);
        if op.end.col != 0 {
            op.end.col -= 1;
            op.inclusive = true;
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
    cmd_arg: Cmd,
    mut op: Op,
    empty_region_error: bool,
    gui_yank: bool,
    lbr_saved: c_int,
) {
    /// Refuse an empty region: beep and drop the half-recorded `.`.
    fn refuse() {
        // SAFETY: neither touches anything but editor-wide state.
        unsafe { vim_beep(kOptBoFlagOperator as ::core::ffi::c_uint) };
        unsafe { cancel_redo() };
    }

    // SAFETY: every operator below is handed the same live `OpArg` and the
    // current window, which is exactly what each of them asks for.
    match op.op_type {
        OpType::Lshift | OpType::Rshift => {
            let amount = if op.is_visual { cmd_arg.count1 } else { 1 };
            unsafe { op_shift(op.raw(), true, amount) };
            unsafe { auto_format(false, true) };
        }

        OpType::JoinNs | OpType::Join => {
            op.line_count = op.line_count.max(2);
            if Win::current().w_cursor.lnum + op.line_count - 1 > Buf::current().line_count() {
                beep_flush();
            } else {
                let count = op.line_count as size_t;
                let _ = unsafe { do_join(count, op.op_type == OpType::Join, true, true, true) };
                unsafe { auto_format(false, true) };
            }
        }

        OpType::Delete => {
            // Do not reselect now.
            VIsual_reselect.set(0);
            if empty_region_error {
                refuse();
            } else {
                // Nothing to do about a refusal: the message is out and
                // the buffer is untouched.
                let _ = unsafe { op_delete(op.raw()) };
                // Save the cursor line for undo if that has not happened.
                if op.motion_type == kMTLineWise
                    && has_format_option(FoFlag::AUTO)
                    && u_save_cursor().is_ok()
                {
                    unsafe { auto_format(false, true) };
                }
            }
        }

        OpType::Yank => {
            if empty_region_error {
                if !gui_yank {
                    refuse();
                }
            } else {
                restore_lbr(lbr_saved != 0);
                // `zy` yanks without the trailing white space.
                op.excl_tr_ws = cmd_arg.cmdchar == 'z' as c_int;
                unsafe { op_yank(op.raw(), !gui_yank) };
            }
            check_cursor_col(Win::current());
        }

        OpType::Change => {
            VIsual_reselect.set(0);
            if empty_region_error {
                refuse();
            } else {
                run_change(cmd_arg, op, lbr_saved);
            }
        }

        OpType::Filter => {
            if cpo_has(CpoFlag::FILTER) {
                // Use whichever `!cmd` was last used.
                unsafe { append_to_redobuff(c"!\r".as_ptr()) };
            } else {
                // `do_bang` will put the command in the redo buffer.
                bangredo.set(true);
            }
            // Falls through to the `:` handling below, as upstream does.
            indent_or_colon(op);
        }
        OpType::Indent | OpType::Colon => indent_or_colon(op),

        OpType::Tilde | OpType::Upper | OpType::Lower | OpType::Rot13 => {
            if empty_region_error {
                refuse();
            } else {
                unsafe { op_tilde(op.raw()) };
            }
            check_cursor_col(Win::current());
        }

        OpType::Format => {
            if unsafe { *Buf::current().b_p_fex } as c_int != NUL {
                unsafe { op_formatexpr(op.raw()) };
            } else if unsafe { *p_fp.get() } as c_int != NUL
                || unsafe { *Buf::current().b_p_fp } as c_int != NUL
            {
                // An external program.
                unsafe { op_colon(op.raw()) };
            } else {
                unsafe { op_format(op.raw(), false) };
            }
        }
        OpType::Format2 => unsafe { op_format(op.raw(), true) },

        OpType::Function => {
            // 'operatorfunc' may run another operator and overwrite the
            // recorded Visual area, so it is put back afterwards.
            let saved = REDO_VISUAL.get();
            restore_lbr(lbr_saved != 0);
            unsafe { op_function(op.raw()) };
            REDO_VISUAL.set(saved);
        }

        OpType::Insert | OpType::Append => {
            VIsual_reselect.set(0);
            if empty_region_error {
                refuse();
            } else {
                run_block_insert(cmd_arg, op, lbr_saved);
            }
        }

        OpType::Replace => {
            VIsual_reselect.set(0);
            if empty_region_error {
                refuse();
            } else {
                restore_lbr(lbr_saved != 0);
                let _ = unsafe { op_replace(op.raw(), cmd_arg.nchar) };
            }
        }

        OpType::Fold => {
            VIsual_reselect.set(0);
            // SAFETY: a live current window, and the operator's own range.
            unsafe { fold_create(Win::current(), op.start, op.end) };
        }
        OpType::Foldopen | OpType::Foldopenrec | OpType::Foldclose | OpType::Foldcloserec => {
            VIsual_reselect.set(0);
            let opening = op.op_type == OpType::Foldopen || op.op_type == OpType::Foldopenrec;
            let recursive = op.op_type == OpType::Foldopenrec || op.op_type == OpType::Foldcloserec;
            let (start, end, visual) = (op.start, op.end, op.is_visual);
            let (opening, recursive) = (c_int::from(opening), c_int::from(recursive));
            op_fold_range(start, end, opening, recursive, visual);
        }
        OpType::Folddel | OpType::Folddelrec => {
            VIsual_reselect.set(0);
            let recursive = c_int::from(op.op_type == OpType::Folddelrec);
            let (first, last, visual) = (op.start.lnum, op.end.lnum, op.is_visual);
            unsafe { delete_fold(Win::current(), first, last, recursive, visual) };
        }

        OpType::NrAdd | OpType::NrSub => {
            if empty_region_error {
                refuse();
            } else {
                // `op_addsub` reads `VIsual_active` to decide whether the
                // region or the cursor line is meant, and this dispatcher
                // has already switched it off.
                set_visual_active(true);
                restore_lbr(lbr_saved != 0);
                let (count, g) = (cmd_arg.count1 as LineNr, REDO_VISUAL.get().rv_arg != 0);
                unsafe { op_addsub(op.raw(), count, g) };
                set_visual_active(false);
            }
            check_cursor_col(Win::current());
        }

        _ => unsafe { clearopbeep(op.raw()) },
    }
}

/// `=` and `:` -- and `!`, which falls through to here.
///
/// With an empty 'equalprg' the indenting is done internally; otherwise the
/// region is handed to a `:` command line.
fn indent_or_colon(op: Op) {
    // SAFETY: a live `OpArg` describing a region of the current buffer, and
    // 'equalprg'/'indentexpr' are NUL-terminated option strings.
    if op.op_type != OpType::Indent || unsafe { *get_equalprg() } as c_int != NUL {
        unsafe { op_colon(op.raw()) };
        return;
    }
    if Buf::current().b_p_lisp != 0 {
        let indent = if unsafe { use_indentexpr_for_lisp() } {
            get_expr_indent as unsafe fn() -> c_int
        } else {
            get_lisp_indent as unsafe fn() -> c_int
        };
        unsafe { op_reindent(op.raw(), Some(indent)) };
        return;
    }
    let indent = if unsafe { *Buf::current().b_p_inde } as c_int != NUL {
        get_expr_indent as unsafe fn() -> c_int
    } else {
        get_c_indent as unsafe fn() -> c_int
    };
    unsafe { op_reindent(op.raw(), Some(indent)) };
}

/// The `c` arm: run `op_change`, which enters Insert mode.
fn run_change(mut cmd_arg: Cmd, op: Op, lbr_saved: c_int) {
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
    Buf::current().b_last_changedtick_i = buf_get_changedtick(Buf::current());

    if unsafe { op_change(op.raw()) } != 0 {
        // `edit()` returned because of a CTRL-O command.
        cmd_arg.retval |= CA_COMMAND_BUSY as c_int;
    }
    if restart_edit.get() == 0 {
        restart_edit.set(restart_edit_save);
    }
}

/// The `I`/`A` arm: run `op_insert`, which enters Insert mode.
fn run_block_insert(mut cmd_arg: Cmd, op: Op, lbr_saved: c_int) {
    let restart_edit_save = restart_edit.get();
    restart_edit.set(0);

    restore_lbr(lbr_saved != 0);
    Buf::current().b_last_changedtick_i = buf_get_changedtick(Buf::current());

    unsafe { op_insert(op.raw(), cmd_arg.count1) };

    // Back off again, so that formatting measures columns correctly.
    reset_lbr();
    unsafe { auto_format(false, true) };

    if restart_edit.get() == 0 {
        restart_edit.set(restart_edit_save);
    } else {
        cmd_arg.retval |= CA_COMMAND_BUSY as c_int;
    }
}
