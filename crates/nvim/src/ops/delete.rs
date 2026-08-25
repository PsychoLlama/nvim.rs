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

use crate::memline::MlFlags;
use core::ffi::{c_char, c_int, c_void};

use super::*;
use crate::edit::BeginlineOpts;
use crate::ex_docmd::cmdmod_has;
use crate::normal::visual_select;
use crate::option::cpo_has;
use crate::register::is_append_register;
use crate::types::{CpoFlag, NUL};
use crate::undo::{UndoFailed, saved};

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

/// `d` (and the delete half of `c`) over the operator's region.
///
/// An empty or refused region is *success*: nothing to delete is not a
/// failure, and neither is a read-only register (which beeps instead).
///
/// # Safety
/// `oap` must point to a live `oparg_T` describing a region of the current
/// buffer.
pub unsafe fn op_delete(oap: *mut oparg_T) -> Result<(), NotDeleted> {
    unsafe {
        let old_lcount = (*curbuf.get()).b_ml.ml_line_count;

        if (*curbuf.get()).b_ml.ml_flags.has(MlFlags::EMPTY) {
            return Ok(());
        }
        // Nothing to delete -- but still prepare undo, for `op_change`.
        if (*oap).empty {
            saved(u_save_cursor())?;
            return Ok(());
        }
        if (*curbuf.get()).b_p_ma == 0 {
            emsg(gettext(&raw const e_modifiable as *const c_char));
            return Err(NotDeleted::NotModifiable);
        }
        if visual_select() && (*oap).is_VIsual {
            // The register given with CTRL-R, zero by default.
            (*oap).regname = VIsual_select_reg.get();
        }

        mb_adjust_opend(oap);

        // Imitate the strange Vi behaviour: a charwise delete spanning more
        // than one line whose result would be a blank line becomes linewise.
        // Not for `c`, and not in Visual mode.
        if (*oap).motion_type == kMTCharWise
            && !(*oap).is_VIsual
            && (*oap).line_count > 1
            && (*oap).motion_force == NUL
            && (*oap).op_type == OP_DELETE
        {
            let mut ptr = ml_get((*oap).end.lnum).offset((*oap).end.col as isize);
            if *ptr as c_int != NUL {
                ptr = ptr.offset((*oap).inclusive as isize);
            }
            ptr = skipwhite(ptr);
            if *ptr as c_int == NUL && inindent(0) {
                (*oap).motion_type = kMTLineWise;
            }
        }

        // Trying to delete (e.g. `D`) in an empty line. For `c` that is fine.
        let empty_region = (*oap).motion_type != kMTLineWise
            && (*oap).line_count == 1
            && (*oap).op_type == OP_DELETE
            && *ml_get((*oap).start.lnum) as c_int == NUL;

        if !empty_region {
            // Yank whatever is about to be deleted. `"_` takes nothing.
            if (*oap).regname != '_' as c_int && !save_deleted_text(oap) {
                return Ok(());
            }

            // `?` converts the undo layer's refusal into this one's.
            if (*oap).motion_type == kMTBlockWise {
                delete_block(oap)?;
            } else if (*oap).motion_type == kMTLineWise {
                delete_whole_lines(oap)?;
            } else {
                delete_chars(oap)?;
            }

            msgmore((*curbuf.get()).b_ml.ml_line_count as c_int - old_lcount as c_int);
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
            if (*oap).motion_type == kMTBlockWise {
                (*curbuf.get()).b_op_end.lnum = (*oap).end.lnum;
                (*curbuf.get()).b_op_end.col = (*oap).start.col;
            } else {
                (*curbuf.get()).b_op_end = (*oap).start;
            }
            (*curbuf.get()).b_op_start = (*oap).start;
        }

        Ok(())
    }
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
///
/// # Safety
/// `oap` must point to a live `oparg_T` describing a region of the current
/// buffer.
unsafe fn save_deleted_text(oap: *mut oparg_T) -> bool {
    unsafe {
        let mut reg: *mut yankreg_T = ::core::ptr::null_mut();
        let mut did_yank = false;

        if (*oap).regname != 0 {
            if !valid_yank_reg((*oap).regname, true) {
                beep_flush();
                return false;
            }
            reg = get_yank_register((*oap).regname, YREG_YANK as c_int);
            // Yank without a message.
            op_yank_reg(oap, false, reg, is_append_register((*oap).regname));
            did_yank = true;
        }

        // Into `"1`, shifting the number registers, when the delete contains a
        // line break or a specific operator was used (Vi compatible).
        if (*oap).motion_type == kMTLineWise || (*oap).line_count > 1 || (*oap).use_reg_one {
            shift_delete_registers(is_append_register((*oap).regname));
            reg = get_y_register(1);
            op_yank_reg(oap, false, reg, false);
            did_yank = true;
        }

        // Into the small-delete register when no register was named and the
        // delete is within one line.
        if (*oap).regname == 0 && (*oap).motion_type != kMTLineWise && (*oap).line_count == 1 {
            reg = get_yank_register('-' as c_int, YREG_YANK as c_int);
            op_yank_reg(oap, false, reg, false);
            did_yank = true;
        }

        if did_yank || (*oap).regname == 0 {
            if reg.is_null() {
                abort();
            }
            crate::clipboard::set_clipboard((*oap).regname, reg as *mut _);
            do_autocmd_textyankpost(oap, reg);
        }
        true
    }
}

/// The blockwise arm: cut the rectangle out of every line it reaches.
///
/// Deleting a TAB that straddles an edge can make the line *longer*, because
/// the part of it outside the block comes back as spaces -- which is what
/// `startspaces`/`endspaces` are, and why the new line is built rather than
/// patched.
///
/// # Safety
/// `oap` must point to a live blockwise `oparg_T`.
unsafe fn delete_block(oap: *mut oparg_T) -> Result<(), UndoFailed> {
    unsafe {
        saved(u_save((*oap).start.lnum - 1, (*oap).end.lnum + 1))?;

        let mut bd = block_def::ZERO;
        let mut lnum = (*curwin.get()).w_cursor.lnum;
        while lnum <= (*oap).end.lnum {
            block_prep(oap, &raw mut bd, lnum, true);
            if bd.textlen != 0 {
                // Adjust the cursor for a TAB replaced by spaces, and 'lbr'.
                if lnum == (*curwin.get()).w_cursor.lnum {
                    (*curwin.get()).w_cursor.col = bd.textcol + bd.startspaces;
                    (*curwin.get()).w_cursor.coladd = 0;
                }

                // The line shrinks by the block's text minus the padding that
                // replaces the characters it only partly covers -- and a
                // deleted TAB can be replaced by more spaces than it took, so
                // `n` may be *negative* and the line grow. The arithmetic
                // stays in `c_int` for that reason; upstream does it in
                // `size_t` and relies on the wraparound.
                let n = bd.textlen - bd.startspaces - bd.endspaces;
                let oldp = ml_get(lnum);
                let newp = xmalloc((ml_get_len(lnum) - n + 1) as size_t) as *mut c_char;
                memmove(
                    newp as *mut c_void,
                    oldp as *const c_void,
                    bd.textcol as size_t,
                );
                memset(
                    newp.offset(bd.textcol as isize) as *mut c_void,
                    ' ' as c_int,
                    bd.startspaces as size_t + bd.endspaces as size_t,
                );
                strcpy(
                    newp.offset((bd.textcol + bd.startspaces + bd.endspaces) as isize),
                    oldp.offset((bd.textcol + bd.textlen) as isize),
                );
                ml_replace(lnum, newp, false);

                extmark_splice_cols(
                    curbuf.get(),
                    lnum as c_int - 1,
                    bd.textcol,
                    bd.textlen,
                    bd.startspaces + bd.endspaces,
                    kExtmarkUndo,
                );
            }
            lnum += 1;
        }

        check_cursor_col(curwin.get());
        changed_lines(
            curbuf.get(),
            (*curwin.get()).w_cursor.lnum,
            (*curwin.get()).w_cursor.col,
            (*oap).end.lnum + 1,
            0,
            true,
        );
        // No whole lines were deleted, so `msgmore` must not report any.
        (*oap).line_count = 0;
        Ok(())
    }
}

/// The linewise arm.
///
/// `c` is the odd one: it deletes every line *but the first* and then empties
/// the first, so that the insert starts on a line that already exists and
/// 'autoindent' has an indent to keep.
///
/// # Safety
/// `oap` must point to a live linewise `oparg_T`.
unsafe fn delete_whole_lines(oap: *mut oparg_T) -> Result<(), UndoFailed> {
    unsafe {
        if (*oap).op_type != OP_CHANGE {
            del_lines((*oap).line_count, true);
            beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
            // `U` is not possible after `dd`.
            u_clearline(curbuf.get());
            return Ok(());
        }

        // Delete every line but the first, with the cursor moved off it: the
        // line number is remembered because deleting the last line moves it.
        if (*oap).line_count > 1 {
            let lnum = (*curwin.get()).w_cursor.lnum;
            (*curwin.get()).w_cursor.lnum += 1;
            del_lines((*oap).line_count - 1, true);
            (*curwin.get()).w_cursor.lnum = lnum;
        }
        saved(u_save_cursor())?;
        if (*curbuf.get()).b_p_ai != 0 {
            // Keep the indent, on the first non-white character; `did_ai` is
            // what deletes it again if the insert is left with ESC.
            beginline(BeginlineOpts::WHITE);
            did_ai.set(true);
            ai_col.set((*curwin.get()).w_cursor.col);
        } else {
            beginline(BeginlineOpts::NONE);
        }
        // The rest of the line, leaving the cursor past its last character.
        truncate_line(0);
        if (*oap).line_count > 1 {
            // `U` is not possible after `2cc`.
            u_clearline(curbuf.get());
        }
        Ok(())
    }
}

/// The charwise arm.
///
/// # Safety
/// `oap` must point to a live charwise `oparg_T`.
unsafe fn delete_chars(oap: *mut oparg_T) -> Result<(), UndoFailed> {
    unsafe {
        if op_virtual() {
            break_tabs_at_edges(oap)?;
        }

        if (*oap).line_count == 1 {
            delete_chars_one_line(oap)?;
        } else {
            delete_chars_across_lines(oap)?;
        }

        if (*oap).op_type == OP_DELETE {
            auto_format(false, true);
        }
        Ok(())
    }
}

/// 'virtualedit' only: replace a TAB the region starts or ends inside with the
/// spaces it covers, so that the delete has real byte positions to work with.
///
/// Moves `oap.start`/`oap.end` onto those positions.
///
/// # Safety
/// `oap` must point to a live charwise `oparg_T`.
unsafe fn break_tabs_at_edges(oap: *mut oparg_T) -> Result<(), UndoFailed> {
    unsafe {
        if gchar_pos(&raw mut (*oap).start) == '\t' as c_int {
            // Save the first line for undo.
            saved(u_save_cursor())?;
            // Breaking the start TAB moves the end too, so remember where the
            // end was in *columns* first.
            let mut endcol = 0;
            if (*oap).line_count == 1 {
                endcol = getviscol2((*oap).end.col, (*oap).end.coladd);
            }
            coladvance_force(getviscol2((*oap).start.col, (*oap).start.coladd));
            (*oap).start = (*curwin.get()).w_cursor;
            if (*oap).line_count == 1 {
                coladvance(curwin.get(), endcol);
                (*oap).end.col = (*curwin.get()).w_cursor.col;
                (*oap).end.coladd = (*curwin.get()).w_cursor.coladd;
                (*curwin.get()).w_cursor = (*oap).start;
            }
        }

        // Break the end TAB only when it is inside the region.
        if gchar_pos(&raw mut (*oap).end) == '\t' as c_int
            && (*oap).end.coladd == 0
            && (*oap).inclusive
        {
            // Save the last line for undo.
            saved(u_save((*oap).end.lnum - 1, (*oap).end.lnum + 1))?;
            (*curwin.get()).w_cursor = (*oap).end;
            coladvance_force(getviscol2((*oap).end.col, (*oap).end.coladd));
            (*oap).end = (*curwin.get()).w_cursor;
            (*curwin.get()).w_cursor = (*oap).start;
        }

        mb_adjust_opend(oap);
        Ok(())
    }
}

/// Delete characters within one line.
///
/// # Safety
/// `oap` must point to a live charwise `oparg_T` whose region is one line.
unsafe fn delete_chars_one_line(oap: *mut oparg_T) -> Result<(), UndoFailed> {
    unsafe {
        // Save the line for undo.
        saved(u_save_cursor())?;

        // 'cpoptions' `$`: show a `$` at the end of the change rather than
        // removing the text now.
        if cpo_has(CpoFlag::DOLLAR)
            && (*oap).op_type == OP_CHANGE
            && (*oap).end.lnum == (*curwin.get()).w_cursor.lnum
            && !(*oap).is_VIsual
        {
            display_dollar((*oap).end.col - c_int::from(!(*oap).inclusive));
        }

        let mut n = (*oap).end.col - (*oap).start.col + 1 - c_int::from(!(*oap).inclusive);

        if op_virtual() {
            let len = get_cursor_line_len();
            if (*oap).end.coladd != 0
                && (*oap).end.col >= len - 1
                && !((*oap).start.coladd != 0 && (*oap).end.col >= len - 1)
            {
                n += 1;
            }
            // Delete at least one character, e.g. when on a control character.
            if n == 0 && (*oap).start.coladd != (*oap).end.coladd {
                n = 1;
            }
            // Having deleted a character in the line, `coladd` is stale.
            if gchar_cursor() != NUL {
                (*curwin.get()).w_cursor.coladd = 0;
            }
        }

        del_bytes(
            n,
            !op_virtual(),
            (*oap).op_type == OP_DELETE && !(*oap).is_VIsual,
        );
        Ok(())
    }
}

/// Delete a charwise region that spans lines.
///
/// Four edits -- truncate the first line, delete the whole lines between,
/// delete the head of the last, join what is left -- bracketed by
/// `curbuf_splice_pending` so that extmarks and the buffer-update RPC see the
/// one splice measured up front rather than four.
///
/// # Safety
/// `oap` must point to a live charwise `oparg_T` spanning at least two lines.
unsafe fn delete_chars_across_lines(oap: *mut oparg_T) -> Result<(), UndoFailed> {
    unsafe {
        // Save the deleted and changed lines for undo.
        saved(u_save(
            (*curwin.get()).w_cursor.lnum - 1,
            (*curwin.get()).w_cursor.lnum + (*oap).line_count,
        ))?;

        curbuf_splice_pending.set(curbuf_splice_pending.get() + 1);
        let startpos = (*curwin.get()).w_cursor;
        let deleted_bytes = get_region_bytecount(
            curbuf.get(),
            startpos.lnum,
            (*oap).end.lnum,
            startpos.col,
            (*oap).end.col,
        ) + bcount_t::from((*oap).inclusive);

        // From the cursor to the end of the line.
        truncate_line(1);

        let curpos = (*curwin.get()).w_cursor;
        (*curwin.get()).w_cursor.lnum += 1;
        del_lines((*oap).line_count - 2, false);

        // From the start of the last line up to the region's end.
        let n = (*oap).end.col + 1 - c_int::from(!(*oap).inclusive);
        (*curwin.get()).w_cursor.col = 0;
        del_bytes(
            n,
            !op_virtual(),
            (*oap).op_type == OP_DELETE && !(*oap).is_VIsual,
        );

        (*curwin.get()).w_cursor = curpos;
        do_join(2, false, false, false, false);
        curbuf_splice_pending.set(curbuf_splice_pending.get() - 1);

        extmark_splice(
            curbuf.get(),
            startpos.lnum as c_int - 1,
            startpos.col,
            (*oap).line_count as c_int - 1,
            n,
            deleted_bytes,
            0,
            0,
            0,
            kExtmarkUndo,
        );
        Ok(())
    }
}

/// Pull `oap.end` back onto the *last byte* of the character it lands in, so
/// that an inclusive delete takes the whole character.
///
/// # Safety
/// `oap` must point to a live `oparg_T` whose end names a position in the
/// current buffer.
pub(crate) unsafe fn mb_adjust_opend(oap: *mut oparg_T) {
    unsafe {
        if !(*oap).inclusive {
            return;
        }
        let line: *const c_char = ml_get((*oap).end.lnum);
        let mut ptr = line.offset((*oap).end.col as isize);
        if *ptr as c_int != NUL {
            ptr = ptr.offset(-(utf_head_off(line, ptr) as isize));
            ptr = ptr.offset((utfc_ptr2len(ptr) - 1) as isize);
            (*oap).end.col = ptr.offset_from(line) as colnr_T;
        }
    }
}
