//! `r` -- overwriting every character in the region with one character.
//!
//! Three things make this more than a memset:
//!
//! * the replacement can be a *different width* from what it replaces, in
//!   bytes and in screen cells, so a line often has to be rebuilt rather than
//!   patched -- [`pbyte`] is the fast path that only applies when both sides
//!   are one byte, and [`replace_character`] the general one;
//! * a blockwise replace has to pad short lines out to the block's edge and
//!   re-lay any TAB the block splits, which is why [`replace_block_line`]
//!   builds a whole new line;
//! * `r<CR>` does not replace with a character at all, it *splits* the line,
//!   and `CTRL-V <CR>` (`REPLACE_CR_NCHAR`) asks for the literal carriage
//!   return instead -- which is the whole job of the `had_ctrl_v_cr` flag.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::guard::Suppress;
use crate::memline::MlFlags;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int, c_void};

use super::*;
use crate::ex_docmd::cmdmod_has;
use crate::types::{Failed, NUL};

/// Overwrite the single byte at `pos` with `c`.
///
/// Only for a one-byte character replacing a one-byte character; anything else
/// changes the line's length and has to go through [`replace_character`].
///
/// # Safety
/// `pos` must name a line of the current buffer.
pub(crate) unsafe fn pbyte(mut pos: Pos, c: c_int) {
    debug_assert!(c <= c_int::from(u8::MAX));
    // SAFETY: the caller's promise -- `pos` names a line of the current
    // buffer, and the column is clamped to that line below before the write.
    let p = unsafe { ml_get_buf_mut(Buf::current(), pos.lnum) };
    let len = Buf::current().b_ml.cached_len();

    // Safety check: the caller's column may be past the line.
    if pos.col >= len {
        pos.col = if len > 1 { len - 2 } else { 0 };
    }
    unsafe { *p.offset(pos.col as isize) = c as c_char };
    if curbuf_splice_pending.get() == 0 {
        let row = pos.lnum as c_int - 1;
        extmark_splice_cols(Buf::current(), row, pos.col, 1, 1, kExtmarkUndo);
    }
}

/// Replace the character under the cursor with `c`, whatever the two widths.
///
/// Goes through Replace mode's own insert so that a multi-byte character on
/// either side is handled; leaves the cursor back on the replaced character.
///
/// # Safety
/// The cursor must name a valid position in the current buffer.
unsafe fn replace_character(c: c_int) {
    let saved = State.get();
    State.set(MODE_REPLACE);
    // SAFETY: the caller's promise -- the cursor names a valid position.
    unsafe { ins_char(c) };
    State.set(saved);
    // Back up onto the character just replaced.
    dec_cursor();
}

/// `r` over the operator's region.
///
/// `c` is the replacement, or `REPLACE_CR_NCHAR`/`REPLACE_NL_NCHAR` for the
/// `CTRL-V <CR>`/`CTRL-V <NL>` spellings that mean the literal byte rather
/// than a line split.
///
/// # Safety
/// `op` must point to a live `OpArg` describing a region of the current
/// buffer.
pub(crate) unsafe fn op_replace(op: *mut OpArg, mut c: c_int) -> Result<(), Failed> {
    // SAFETY: the caller's promise -- a live `OpArg` of the current buffer.
    let op = unsafe { Op::new(op) };
    if Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY) || op.empty {
        return Ok(());
    }

    // CTRL-V CR / CTRL-V NL: put the byte in, do not split the line.
    let mut had_ctrl_v_cr = false;
    if c == REPLACE_CR_NCHAR {
        had_ctrl_v_cr = true;
        c = CAR;
    } else if c == REPLACE_NL_NCHAR {
        had_ctrl_v_cr = true;
        c = NL;
    }

    unsafe { mb_adjust_opend(op.raw()) };

    let (above, below) = (op.start.lnum - 1, op.end.lnum + 1);
    u_save(above, below)?;

    if op.motion_type == kMTBlockWise {
        replace_block(op, c, had_ctrl_v_cr);
    } else {
        replace_chars(op, c);
    }

    Win::current().w_cursor = op.start;
    let (lnum, col, last) = (op.start.lnum, op.start.col, op.end.lnum + 1);
    check_cursor(Win::current());
    changed_lines(Buf::current(), lnum, col, last, 0, true);

    if !cmdmod_has(CmdModFlags::LOCKMARKS) {
        Buf::current().b_op_start = op.start;
        Buf::current().b_op_end = op.end;
    }
    Ok(())
}

/// The blockwise arm: one rebuilt line per line the block reaches.
///
/// `op` must be blockwise.
fn replace_block(op: Op, c: c_int, had_ctrl_v_cr: bool) {
    let mut bd = BlockDef::ZERO;
    bd.is_max = c_int::from(Win::current().w_curswant == MAXCOL);
    while Win::current().w_cursor.lnum <= op.end.lnum {
        // Make sure the cursor position is valid for `block_prep`.
        Win::current().w_cursor.col = 0;
        // SAFETY: the cursor walks the region, so its line is the buffer's.
        let lnum = Win::current().w_cursor.lnum;
        unsafe { block_prep(op.raw(), &raw mut bd, lnum, true) };
        if bd.textlen != 0 || (op_virtual() && bd.is_max == 0) {
            replace_block_line(op, &mut bd, c, had_ctrl_v_cr);
        }
        Win::current().w_cursor.lnum += 1;
    }
}

/// One line of the blockwise arm.
///
/// Splitting a TAB the block only partly covers can make the line *longer*, so
/// the line is rebuilt: the text before the block, `startspaces` pad, the
/// replacement repeated, `endspaces` pad, and the text after. With `\r` or
/// `\n` and no CTRL-V there is no "after": the tail becomes a new line.
///
/// `op` and `bd` must describe the cursor line, as [`block_prep`] left them.
fn replace_block_line(mut op: Op, bd: &mut BlockDef, c: c_int, had_ctrl_v_cr: bool) {
    // Replacing with `\r`/`\n` splits the line rather than overwriting.
    let splits_line = !had_ctrl_v_cr && (c == '\r' as c_int || c == '\n' as c_int);

    // When the block starts in virtual space, that offset counts as
    // pre-padding. (Upstream also keeps a running count `n` of the extra
    // characters a split TAB needs, here and just below; nothing reads it,
    // so only this side effect on `startspaces` is carried over.)
    // SAFETY: `bd` describes the cursor line, so `bd.textstart` is inside it.
    if op_virtual() && bd.is_short != 0 && unsafe { *bd.textstart } as c_int == NUL {
        let mut vpos = Pos {
            lnum: Win::current().w_cursor.lnum,
            col: 0,
            coladd: 0,
        };
        // SAFETY: a live current window, and a local position in the cursor's
        // own line.
        unsafe { getvpos(Win::current(), PosRef::new(&raw mut vpos), op.start_vcol) };
        bd.startspaces += vpos.coladd;
    }

    // How many characters to replace.
    let mut numc = op.end_vcol - op.start_vcol + 1;
    if bd.is_short != 0 && (!op_virtual() || bd.is_max != 0) {
        numc -= (op.end_vcol - bd.end_vcol) + 1;
    }
    // A double-wide character only fits half as many times.
    if unsafe { utf_char2cells(c) } > 1 {
        if numc & 1 != 0 && bd.is_short == 0 {
            bd.endspaces += 1;
        }
        numc /= 2;
    }

    let mut num_chars = numc;
    numc *= utf_char2len(c);

    let mut oldp = get_cursor_line_ptr();
    let oldlen = get_cursor_line_len();

    let mut newp_size = bd.textcol as size_t + bd.startspaces as size_t;
    if !splits_line {
        newp_size += numc as size_t;
        if bd.is_short == 0 {
            newp_size += (bd.endspaces + oldlen - bd.textcol - bd.textlen) as size_t;
        }
    }
    // SAFETY: `newp_size` counts the prefix, the pre-spaces, the repeated
    // replacement, the post-spaces and the tail -- exactly what is written.
    let newp = unsafe { xmallocz(newp_size) } as *mut c_char;

    // Up to the replaced part, then the pre-spaces.
    let into = newp.cast::<u8>();
    unsafe { into.copy_from(oldp.cast(), bd.textcol as size_t) };
    oldp = unsafe { oldp.offset((bd.textcol + bd.textlen) as isize) };
    let at = unsafe { newp.offset(bd.textcol as isize) } as *mut c_void;
    unsafe { at.cast::<u8>().write_bytes(b' ', bd.startspaces as size_t) };

    // What is left of the line after the block, NUL included.
    let col = oldlen - bd.textcol - bd.textlen + 1;
    debug_assert!(col >= 0);

    let mut after_p: *mut c_char = ::core::ptr::null_mut();
    let mut after_p_len: size_t = 0;
    let mut newrows = 0;
    let mut newcols = 0;
    if !splits_line {
        let mut newp_len = bd.textcol + bd.startspaces;
        while num_chars > 0 {
            num_chars -= 1;
            newp_len += unsafe { utf_char2bytes(c, newp.offset(newp_len as isize)) };
        }
        if bd.is_short == 0 {
            let at = unsafe { newp.offset(newp_len as isize) } as *mut c_void;
            unsafe { at.cast::<u8>().write_bytes(b' ', bd.endspaces as size_t) };
            newp_len += bd.endspaces;
            let tail = unsafe { newp.offset(newp_len as isize) } as *mut c_void;
            unsafe { tail.cast::<u8>().copy_from(oldp.cast(), col as size_t) };
        }
        newcols = newp_len - bd.textcol;
    } else {
        // The tail becomes the next line.
        after_p_len = col as size_t;
        after_p = unsafe { xmalloc(after_p_len) } as *mut c_char;
        unsafe { after_p.cast::<u8>().copy_from(oldp.cast(), after_p_len) };
        newrows = 1;
    }

    let baselnum = Win::current().w_cursor.lnum;
    let _ = unsafe { ml_replace(baselnum, newp, false) };
    let splice = Suppress::splice();
    if !after_p.is_null() {
        let len = after_p_len as ColNr;
        let _ = unsafe { ml_append(Win::current().w_cursor.lnum, after_p, len, false) };
        Win::current().w_cursor.lnum += 1;
        unsafe { appended_lines_mark(Win::current().w_cursor.lnum, 1) };
        op.end.lnum += 1;
        unsafe { xfree(after_p as *mut c_void) };
    }
    drop(splice);
    let old_bytes = bd.textlen as BCount;
    let new_bytes = (newrows + newcols) as BCount;
    let row = baselnum as c_int - 1;
    let (col, len) = (bd.textcol, bd.textlen);
    let op = kExtmarkUndo;
    extmark_splice(
        Buf::current(),
        row,
        col,
        0,
        len,
        old_bytes,
        newrows,
        newcols,
        new_bytes,
        op,
    );
}

/// The charwise and linewise arm: walk the region a character at a time.
///
/// `op` must be charwise or linewise.
fn replace_chars(mut op: Op, c: c_int) {
    // SAFETY: the cursor walks the region, so it names a position of the
    // current buffer at every step, which is what each of these asks for.
    if op.motion_type == kMTLineWise {
        op.start.col = 0;
        Win::current().w_cursor.col = 0;
        op.end.col = ml_get_len(op.end.lnum);
        if op.end.col != 0 {
            op.end.col -= 1;
        }
    } else if !op.inclusive {
        unsafe { dec(&mut op.end) };
    }

    while ltoreq(Win::current().w_cursor, op.end) {
        let mut done = false;

        let under_cursor = gchar_cursor();
        if under_cursor != NUL {
            let new_byte_len = utf_char2len(c);
            let old_byte_len = unsafe { utfc_ptr2len(get_cursor_pos_ptr()) };

            if new_byte_len > 1 || old_byte_len > 1 {
                // Slow, but it handles a single-byte character replacing a
                // multi-byte one and the other way around.
                if Win::current().w_cursor.lnum == op.end.lnum {
                    op.end.col += new_byte_len - old_byte_len;
                }
                unsafe { replace_character(c) };
                done = true;
            } else {
                if under_cursor == TAB {
                    // Breaking the TAB moves the end, so remember where it
                    // was in columns first.
                    let mut end_vcol = 0;
                    if Win::current().w_cursor.lnum == op.end.lnum {
                        end_vcol = unsafe { getviscol2(op.end.col, op.end.coladd) };
                    }
                    unsafe { coladvance_force(getviscol()) };
                    if Win::current().w_cursor.lnum == op.end.lnum {
                        // SAFETY: a live current window, and the operator's
                        // end position in the cursor's own line.
                        unsafe { getvpos(Win::current(), op.end(), end_vcol) };
                    }
                }
                // With `coladd` set the cursor may now be just past a TAB.
                if gchar_cursor() != NUL {
                    unsafe { pbyte(Win::current().w_cursor, c) };
                    done = true;
                }
            }
        }

        if !done && op_virtual() && Win::current().w_cursor.lnum == op.end.lnum {
            replace_virtual_tail(op, c);
        }

        // On to the next character; stop at the end of the file.
        if inc_cursor() == -1 {
            break;
        }
    }
}

/// 'virtualedit' only: replace the columns past the end of the last line.
///
/// Reached when the region extends into virtual space, where there is no
/// character to overwrite; `coladvance_force` fills the line out with spaces
/// first and those are then replaced.
///
/// The cursor must be on `op.end.lnum`.
fn replace_virtual_tail(op: Op, c: c_int) {
    let mut virtcols = op.end.coladd;
    if Win::current().w_cursor.lnum == op.start.lnum
        && op.start.col == op.end.col
        && op.start.coladd != 0
    {
        virtcols -= op.start.coladd;
    }

    // `op.end` has been trimmed, so it is effectively inclusive: the extra
    // +1 is what keeps the NUL byte from being trampled.
    // SAFETY: the cursor is on `op.end.lnum`, a line of the current buffer,
    // and `coladvance_force` fills it out to the column being replaced.
    let endcol = unsafe { getviscol2(op.end.col, op.end.coladd) };
    unsafe { coladvance_force(endcol + 1) };
    Win::current().w_cursor.col -= virtcols + 1;
    while virtcols >= 0 {
        if utf_char2len(c) > 1 {
            unsafe { replace_character(c) };
        } else {
            unsafe { pbyte(Win::current().w_cursor, c) };
        }
        if unsafe { inc(&mut Win::current().w_cursor) } == -1 {
            break;
        }
        virtcols -= 1;
    }
}
