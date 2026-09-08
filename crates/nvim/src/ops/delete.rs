//! `d` -- deleting the operator's region.
//!
//! [`op_delete`] is a prologue that all three region shapes share followed by
//! one of three arms. The prologue is where the surprises are:
//!
//! * the region is *yanked* first, into as many as three registers -- the
//!   named one, the shift of `"1`..`"9` when the delete crosses a line, and
//!   the small-delete `"-` when it does not -- and only then deleted, which is
//!   why [`save_deleted_text`] runs before any of the arms;
//! * a charwise delete of more than one line that would leave a blank line
//!   becomes a *linewise* one, which is upstream's "strange Vi behaviour";
//! * deleting an empty region is an error under 'cpoptions' `E`, except in
//!   'virtualedit', where nothing is deleted but the marks are set anyway.
//!
//! Of the three arms [`delete_chars`] is the delicate one: a charwise region
//! that spans lines is deleted as a truncate, a line delete, a byte delete and
//! a join, with `curbuf_splice_pending` held over the lot so that the four
//! edits reach extmarks and the buffer-update RPC as the single splice
//! [`get_region_bytecount`] measured up front.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::guard::Suppress;
use crate::memline::MlFlags;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int};

use super::*;
use crate::edit::BeginlineOpts;
use crate::ex_docmd::cmdmod_has;
use crate::normal::visual_select;
use crate::option::cpo_has;
use crate::register::is_append_register;
use crate::types::{CpoFlag, Failed, NUL};
use crate::undo::UndoFailed;

/// The region was not deleted, and the buffer is as it was.
///
/// Only two things stop a delete, and neither leaves half a change behind.
/// The distinction is not one any caller acts on — `op_change` and the
/// `:normal` path both simply give up — but it is one the `c_int` erased.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NotDeleted {
    /// 'modifiable' is off. E21 has already been reported.
    NotModifiable,
    /// Undo could not record the lines, so they must not be touched.
    NoUndo,
}

impl From<UndoFailed> for NotDeleted {
    fn from(_: UndoFailed) -> Self {
        NotDeleted::NoUndo
    }
}

// `?` applies `From` once, so the `u_save*` family's own error needs its own
// arm here rather than travelling through `UndoFailed`.
impl From<Failed> for NotDeleted {
    fn from(_: Failed) -> Self {
        NotDeleted::NoUndo
    }
}

/// `d` (and the delete half of `c`) over the operator's region.
///
/// An empty or refused region is *success*: nothing to delete is not a
/// failure, and neither is a read-only register (which beeps instead).
///
/// # Safety
/// `op` must point to a live `OpArg` describing a region of the current
/// buffer.
pub unsafe fn op_delete(op: *mut OpArg) -> Result<(), NotDeleted> {
    // SAFETY: the caller's promise -- a live `OpArg` of the current buffer.
    // Every line and column touched below is one of that region's.
    let mut op = unsafe { Op::new(op) };
    let old_lcount = Buf::current().line_count();

    if Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY) {
        return Ok(());
    }
    // Nothing to delete -- but still prepare undo, for `op_change`.
    if op.empty {
        u_save_cursor()?;
        return Ok(());
    }
    if Buf::current().b_p_ma == 0 {
        emsg(gettext(e_modifiable));
        return Err(NotDeleted::NotModifiable);
    }
    if visual_select() && op.is_visual {
        // The register given with CTRL-R, zero by default.
        op.regname = VIsual_select_reg.get();
    }

    unsafe { mb_adjust_opend(op.raw()) };

    // Imitate the strange Vi behaviour: a charwise delete spanning more
    // than one line whose result would be a blank line becomes linewise.
    // Not for `c`, and not in Visual mode.
    if op.motion_type == kMTCharWise
        && !op.is_visual
        && op.line_count > 1
        && op.motion_force == NUL
        && op.op_type == OpType::Delete
    {
        let blank = {
            let mut lines = Lines::current();
            let line = lines.line(op.end.lnum);
            let mut at = usize::try_from(op.end.col).unwrap_or(0);
            if byte_at(line, at) != 0 {
                at += usize::from(op.inclusive);
            }
            let at = at.min(line.len());
            skip::white(&line[at..]) == line.len() - at
        };
        if blank && unsafe { inindent(0) } {
            op.motion_type = kMTLineWise;
        }
    }

    // Trying to delete (e.g. `D`) in an empty line. For `c` that is fine.
    let empty_region = op.motion_type != kMTLineWise
        && op.line_count == 1
        && op.op_type == OpType::Delete
        && Lines::current().line(op.start.lnum).is_empty();

    if !empty_region {
        // Yank whatever is about to be deleted. `"_` takes nothing.
        if op.regname != '_' as c_int && !save_deleted_text(op) {
            return Ok(());
        }

        // `?` converts the undo layer's refusal into this one's.
        if op.motion_type == kMTBlockWise {
            delete_block(op)?;
        } else if op.motion_type == kMTLineWise {
            delete_whole_lines(op)?;
        } else {
            delete_chars(op)?;
        }

        let n = Buf::current().line_count() as c_int - old_lcount as c_int;
        unsafe { msgmore(n) };
    } else if !op_virtual() {
        // Operating on an empty region is an error when 'cpoptions'
        // contains 'E' (Vi compatible).
        if cpo_has(CpoFlag::EMPTYREGION) {
            beep_flush();
        }
        return Ok(());
    }
    // In 'virtualedit' an empty region deletes nothing, but the marks are
    // set as if it had.

    if !cmdmod_has(CmdModFlags::LOCKMARKS) {
        if op.motion_type == kMTBlockWise {
            Buf::current().b_op_end.lnum = op.end.lnum;
            Buf::current().b_op_end.col = op.start.col;
        } else {
            Buf::current().b_op_end = op.start;
        }
        Buf::current().b_op_start = op.start;
    }

    Ok(())
}

/// Yank the region into every register a delete is supposed to fill, and fire
/// TextYankPost.
///
/// Answers false when the named register is read-only, which is the one case
/// where `op_delete` gives up without deleting anything.
///
/// Up to three registers are written: the named one, `"1` (with `"1`..`"9`
/// shifted up) when the delete crosses a line or the caller asked for it, and
/// the small-delete `"-` when no register was named and the delete stays
/// inside one line. Only the *last* one written reaches the clipboard and the
/// autocommand, which is upstream's behaviour and the reason `reg` is carried
/// rather than each branch handling its own.
fn save_deleted_text(op: Op) -> bool {
    // SAFETY: a live `OpArg` of the current buffer, and every register
    // written is one `get_yank_register`/`get_y_register` just handed back.
    let mut reg: *mut YankReg = ::core::ptr::null_mut();
    let mut did_yank = false;

    if op.regname != 0 {
        if !unsafe { valid_yank_reg(op.regname, true) } {
            beep_flush();
            return false;
        }
        reg = unsafe { get_yank_register(op.regname, YREG_YANK as c_int) };
        // Yank without a message.
        let append = is_append_register(op.regname);
        unsafe { op_yank_reg(op.raw(), false, reg, append) };
        did_yank = true;
    }

    // Into `"1`, shifting the number registers, when the delete contains a
    // line break or a specific operator was used (Vi compatible).
    if op.motion_type == kMTLineWise || op.line_count > 1 || op.use_reg_one {
        unsafe { shift_delete_registers(is_append_register(op.regname)) };
        reg = unsafe { get_y_register(1) };
        unsafe { op_yank_reg(op.raw(), false, reg, false) };
        did_yank = true;
    }

    // Into the small-delete register when no register was named and the
    // delete is within one line.
    if op.regname == 0 && op.motion_type != kMTLineWise && op.line_count == 1 {
        reg = unsafe { get_yank_register('-' as c_int, YREG_YANK as c_int) };
        unsafe { op_yank_reg(op.raw(), false, reg, false) };
        did_yank = true;
    }

    if did_yank || op.regname == 0 {
        if reg.is_null() {
            unsafe { abort() };
        }
        unsafe { crate::clipboard::set_clipboard(op.regname, reg as *mut _) };
        unsafe { do_autocmd_textyankpost(op.raw(), reg) };
    }
    true
}

/// The blockwise arm: cut the rectangle out of every line it reaches.
///
/// Deleting a TAB that straddles an edge can make the line *longer*, because
/// the part of it outside the block comes back as spaces -- which is what
/// `startspaces`/`endspaces` are, and why the new line is built rather than
/// patched.
///
/// `op` must be blockwise.
fn delete_block(mut op: Op) -> Result<(), UndoFailed> {
    // SAFETY: every line the walk reaches is one of the region's, so it is a
    // line of the current buffer.
    let (above, below) = (op.start.lnum - 1, op.end.lnum + 1);
    u_save(above, below)?;

    let mut bd = BlockDef::ZERO;
    let mut lnum = Win::current().w_cursor.lnum;
    while lnum <= op.end.lnum {
        unsafe { block_prep(op.raw(), &raw mut bd, lnum, true) };
        if bd.textlen != 0 {
            // Adjust the cursor for a TAB replaced by spaces, and 'lbr'.
            if lnum == Win::current().w_cursor.lnum {
                Win::current().w_cursor.col = bd.textcol + bd.startspaces;
                Win::current().w_cursor.coladd = 0;
            }

            // The line loses the block's text and gains the padding that
            // replaces the characters it only partly covers -- and a
            // deleted TAB can be replaced by more spaces than it took, so
            // the line may *grow*.  Upstream sizes the allocation as
            // `oldlen - n` in `size_t` and relies on the wraparound when it
            // does; here the two parts are counted separately, which needs
            // no wraparound and is the same allocation.
            let pad = bd.startspaces + bd.endspaces;
            // The line minus the block plus the padding.  The borrow ends
            // before `ml_replace` writes it back.
            let line = {
                let mut lines = Lines::current();
                let oldp = lines.line(lnum);
                let at = usize::try_from(bd.textcol).unwrap_or(0).min(oldp.len());
                let after = at + usize::try_from(bd.textlen).unwrap_or(0);
                let mut line = Vec::with_capacity(oldp.len() + pad.max(0) as usize + 1);
                line.extend_from_slice(&oldp[..at]);
                line.resize(line.len() + pad as usize, b' ');
                line.extend_from_slice(&oldp[after.min(oldp.len())..]);
                line
            };
            // SAFETY: `line` holds exactly the bytes named, and `copy` is
            // what hands the memline its own allocation of them.
            let _ = unsafe {
                ml_replace_len(
                    lnum,
                    line.as_ptr().cast_mut().cast::<c_char>(),
                    line.len(),
                    true,
                )
            };
            let row = lnum as c_int - 1;
            let buffer = Buf::current();
            extmark_splice_cols(buffer, row, bd.textcol, bd.textlen, pad, kExtmarkUndo);
        }
        lnum += 1;
    }

    let (lnum, col) = (Win::current().w_cursor.lnum, Win::current().w_cursor.col);
    check_cursor_col(Win::current());
    changed_lines(Buf::current(), lnum, col, op.end.lnum + 1, 0, true);
    // No whole lines were deleted, so `msgmore` must not report any.
    op.line_count = 0;
    Ok(())
}

/// The linewise arm.
///
/// `c` is the odd one: it deletes every line *but the first* and then empties
/// the first, so that the insert starts on a line that already exists and
/// 'autoindent' has an indent to keep.
///
/// `op` must be linewise.
fn delete_whole_lines(op: Op) -> Result<(), UndoFailed> {
    // SAFETY: the region is the current buffer's, and the cursor stays on a
    // line of it throughout.
    if op.op_type != OpType::Change {
        unsafe { del_lines(op.line_count, true) };
        beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
        // `U` is not possible after `dd`.
        u_clearline(Buf::current());
        return Ok(());
    }

    // Delete every line but the first, with the cursor moved off it: the
    // line number is remembered because deleting the last line moves it.
    if op.line_count > 1 {
        let lnum = Win::current().w_cursor.lnum;
        Win::current().w_cursor.lnum += 1;
        unsafe { del_lines(op.line_count - 1, true) };
        Win::current().w_cursor.lnum = lnum;
    }
    u_save_cursor()?;
    if Buf::current().b_p_ai != 0 {
        // Keep the indent, on the first non-white character; `did_ai` is
        // what deletes it again if the insert is left with ESC.
        beginline(BeginlineOpts::WHITE);
        did_ai.set(true);
        ai_col.set(Win::current().w_cursor.col);
    } else {
        beginline(BeginlineOpts::NONE);
    }
    // The rest of the line, leaving the cursor past its last character.
    unsafe { truncate_line(0) };
    if op.line_count > 1 {
        // `U` is not possible after `2cc`.
        u_clearline(Buf::current());
    }
    Ok(())
}

/// The charwise arm.
///
/// `op` must be charwise.
fn delete_chars(op: Op) -> Result<(), UndoFailed> {
    if op_virtual() {
        break_tabs_at_edges(op)?;
    }

    if op.line_count == 1 {
        delete_chars_one_line(op)?;
    } else {
        delete_chars_across_lines(op)?;
    }

    if op.op_type == OpType::Delete {
        // SAFETY: formats the current buffer around the cursor.
        unsafe { auto_format(false, true) };
    }
    Ok(())
}

/// 'virtualedit' only: replace a TAB the region starts or ends inside with the
/// spaces it covers, so that the delete has real byte positions to work with.
///
/// Moves `op.start`/`op.end` onto those positions.
///
/// `op` must be charwise.
fn break_tabs_at_edges(mut op: Op) -> Result<(), UndoFailed> {
    // SAFETY: both ends name positions of the current buffer, and the cursor
    // is put on one of them before each column is measured.
    if unsafe { gchar_pos(op.start().raw()) } == '\t' as c_int {
        // Save the first line for undo.
        u_save_cursor()?;
        // Breaking the start TAB moves the end too, so remember where the
        // end was in *columns* first.
        let mut endcol = 0;
        if op.line_count == 1 {
            endcol = unsafe { getviscol2(op.end.col, op.end.coladd) };
        }
        let startcol = unsafe { getviscol2(op.start.col, op.start.coladd) };
        unsafe { coladvance_force(startcol) };
        op.start = Win::current().w_cursor;
        if op.line_count == 1 {
            Win::current().coladvance(endcol);
            op.end.col = Win::current().w_cursor.col;
            op.end.coladd = Win::current().w_cursor.coladd;
            Win::current().w_cursor = op.start;
        }
    }

    // Break the end TAB only when it is inside the region.
    if unsafe { gchar_pos(op.end().raw()) } == '\t' as c_int && op.end.coladd == 0 && op.inclusive {
        // Save the last line for undo.
        u_save(op.end.lnum - 1, op.end.lnum + 1)?;
        Win::current().w_cursor = op.end;
        let endcol = unsafe { getviscol2(op.end.col, op.end.coladd) };
        unsafe { coladvance_force(endcol) };
        op.end = Win::current().w_cursor;
        Win::current().w_cursor = op.start;
    }

    unsafe { mb_adjust_opend(op.raw()) };
    Ok(())
}

/// Delete characters within one line.
///
/// `op` must be charwise, and its region one line.
fn delete_chars_one_line(op: Op) -> Result<(), UndoFailed> {
    // SAFETY: the region is one line of the current buffer, and the cursor
    // is on it.
    // Save the line for undo.
    u_save_cursor()?;

    // 'cpoptions' `$`: show a `$` at the end of the change rather than
    // removing the text now.
    if cpo_has(CpoFlag::DOLLAR)
        && op.op_type == OpType::Change
        && op.end.lnum == Win::current().w_cursor.lnum
        && !op.is_visual
    {
        unsafe { display_dollar(op.end.col - c_int::from(!op.inclusive)) };
    }

    let mut n = op.end.col - op.start.col + 1 - c_int::from(!op.inclusive);

    if op_virtual() {
        let len = get_cursor_line_len();
        if op.end.coladd != 0
            && op.end.col >= len - 1
            && !(op.start.coladd != 0 && op.end.col >= len - 1)
        {
            n += 1;
        }
        // Delete at least one character, e.g. when on a control character.
        if n == 0 && op.start.coladd != op.end.coladd {
            n = 1;
        }
        // Having deleted a character in the line, `coladd` is stale.
        if gchar_cursor() != NUL {
            Win::current().w_cursor.coladd = 0;
        }
    }

    let fixpos = op.op_type == OpType::Delete && !op.is_visual;
    let _ = unsafe { del_bytes(n, !op_virtual(), fixpos) };
    Ok(())
}

/// Delete a charwise region that spans lines.
///
/// Four edits -- truncate the first line, delete the whole lines between,
/// delete the head of the last, join what is left -- bracketed by
/// `curbuf_splice_pending` so that extmarks and the buffer-update RPC see the
/// one splice measured up front rather than four.
///
/// `op` must be charwise and span at least two lines.
fn delete_chars_across_lines(op: Op) -> Result<(), UndoFailed> {
    // SAFETY: the region is the current buffer's and spans at least two of
    // its lines; the cursor stays inside it through all four edits.
    let above = Win::current().w_cursor.lnum - 1;
    let past = Win::current().w_cursor.lnum + op.line_count;
    // Save the deleted and changed lines for undo.
    u_save(above, past)?;

    let splice = Suppress::splice();
    let startpos = Win::current().w_cursor;
    let (lnum, col) = (startpos.lnum, startpos.col);
    let buf = Buf::current();
    let spanned = get_region_bytecount(buf, lnum, op.end.lnum, col, op.end.col);
    let deleted_bytes = spanned + BCount::from(op.inclusive);

    // From the cursor to the end of the line.
    unsafe { truncate_line(1) };

    let curpos = Win::current().w_cursor;
    Win::current().w_cursor.lnum += 1;
    unsafe { del_lines(op.line_count - 2, false) };

    // From the start of the last line up to the region's end.
    let n = op.end.col + 1 - c_int::from(!op.inclusive);
    Win::current().w_cursor.col = 0;
    let fixpos = op.op_type == OpType::Delete && !op.is_visual;
    let _ = unsafe { del_bytes(n, !op_virtual(), fixpos) };

    Win::current().w_cursor = curpos;
    let _ = unsafe { do_join(2, false, false, false, false) };
    drop(splice);

    let rows = op.line_count as c_int - 1;
    let row = startpos.lnum as c_int - 1;
    let buf = Buf::current();
    extmark_splice(buf, row, col, rows, n, deleted_bytes, 0, 0, 0, kExtmarkUndo);
    Ok(())
}

/// Pull `op.end` back onto the *last byte* of the character it lands in, so
/// that an inclusive delete takes the whole character.
///
/// # Safety
/// `op` must point to a live `OpArg` whose end names a position in the
/// current buffer.
pub(crate) unsafe fn mb_adjust_opend(op: *mut OpArg) {
    // SAFETY: the caller's promise -- `op.end` names a position of the
    // current buffer, so its line is live and `end.col` a column of it.
    let mut op = unsafe { Op::new(op) };
    if !op.inclusive {
        return;
    }
    let mut lines = Lines::current();
    let line = lines.line(op.end.lnum);
    let at = usize::try_from(op.end.col).unwrap_or(0);
    if byte_at(line, at) != 0 {
        let at = at - head_off(line, at);
        let at = at + cluster_len(&line[at..]) - 1;
        op.end.col = ColNr::try_from(at).unwrap_or(ColNr::MAX);
    }
}
