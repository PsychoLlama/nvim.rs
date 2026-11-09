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

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::extmark::kExtmarkUndo;
use crate::guard::Suppress;
use crate::memline::MlFlags;
use crate::narrow::len_as_int;
use crate::types::Key;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;
use crate::ascii::NUL;
use crate::cursor::getvpos;
use crate::ex_docmd::cmdmod_has;
use crate::mbyte::MB_MAXCHAR;
use crate::mbyte::{c_cluster_len, encode_char};
use crate::types::Failed;

/// Overwrite the single byte at `pos` with `c`.
///
/// Only for a one-byte character replacing a one-byte character; anything else
/// changes the line's length and has to go through [`replace_character`].
pub(crate) fn pbyte(mut pos: Pos, c: u8) {
    let mut lines = Lines::current();
    let line = lines.line_mut(pos.lnum);
    // The caller's column may be past the line: the last character then.
    // (Upstream measures with the NUL, so an empty line would have its NUL
    // written; no caller gets here on an empty line.)
    let len = ColNr::try_from(line.len()).unwrap_or(ColNr::MAX);
    if pos.col >= len {
        pos.col = (len - 1).max(ColNr::ZERO);
    }
    if let Some(byte) = line.get_mut(pos.col.as_usize()) {
        *byte = c;
    }
    if curbuf_splice_pending.get() == 0 {
        let row = pos.lnum.get() - 1;
        extmark_splice_cols(
            Buf::current(),
            row,
            pos.col,
            ColNr::new(1),
            ColNr::new(1),
            kExtmarkUndo,
        );
    }
}

/// Replace the character under the cursor with `c`, whatever the two widths.
///
/// Goes through Replace mode's own insert so that a multi-byte character on
/// either side is handled; leaves the cursor back on the replaced character.
fn replace_character(c: Key) {
    let saved = STATE.get();
    STATE.set(MODE_REPLACE);
    ins_char(c);
    STATE.set(saved);
    // Back up onto the character just replaced.
    dec_cursor();
}

/// `r` over the operator's region.
///
/// `c` is the replacement, or `REPLACE_CR_NCHAR`/`REPLACE_NL_NCHAR` for the
/// `CTRL-V <CR>`/`CTRL-V <NL>` spellings that mean the literal byte rather
/// than a line split. `op` describes a region of the current buffer.
pub(crate) fn op_replace(op: &mut OpArg, mut c: Key) -> Result<(), Failed> {
    if Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY) || op.empty {
        return Ok(());
    }

    // CTRL-V CR / CTRL-V NL: put the byte in, do not split the line.
    let mut had_ctrl_v_cr = false;
    if c == REPLACE_CR_NCHAR {
        had_ctrl_v_cr = true;
        c = Key::CAR;
    } else if c == REPLACE_NL_NCHAR {
        had_ctrl_v_cr = true;
        c = Key::NL;
    }

    mb_adjust_opend(op);

    let (above, below) = (op.start.lnum - 1, op.end.lnum + 1);
    u_save(above, below)?;

    if op.motion_type == MT_BLOCK_WISE {
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
fn replace_block(op: &mut OpArg, c: Key, had_ctrl_v_cr: bool) {
    let mut bd = BlockDef::ZERO;
    bd.is_max = Win::current().w_curswant == ColNr::MAX;
    while Win::current().w_cursor.lnum <= op.end.lnum {
        // Make sure the cursor position is valid for `block_prep`.
        Win::current().w_cursor.col = ColNr::ZERO;
        // The cursor walks the region, so its line is the buffer's.
        let lnum = Win::current().w_cursor.lnum;
        block_prep(&mut bd, op, lnum, true);
        if bd.textlen != 0 || (op_virtual() && !bd.is_max) {
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
fn replace_block_line(op: &mut OpArg, bd: &mut BlockDef, c: Key, had_ctrl_v_cr: bool) {
    // Replacing with `\r`/`\n` splits the line rather than overwriting.
    let splits_line = !had_ctrl_v_cr && (c == Key::CAR || c == Key::NL);
    let baselnum = Win::current().w_cursor.lnum;

    // When the block starts in virtual space, that offset counts as
    // pre-padding. (Upstream also keeps a running count `n` of the extra
    // characters a split TAB needs, here and just below; nothing reads it,
    // so only this side effect on `startspaces` is carried over.)
    if op_virtual()
        && bd.is_short
        && byte_at(Lines::current().line(baselnum), bd.textcol.as_usize()) == 0
    {
        let mut vpos = Pos {
            lnum: baselnum,
            col: ColNr::ZERO,
            coladd: 0,
        };
        getvpos(Win::current(), &mut vpos, op.start_vcol);
        bd.startspaces += vpos.coladd;
    }

    // How many characters to replace.
    let mut numc = op.end_vcol - op.start_vcol + 1;
    if bd.is_short && (!op_virtual() || bd.is_max) {
        numc -= (op.end_vcol - bd.end_vcol) + 1;
    }
    // A double-wide character only fits half as many times.
    if utf_char2cells(c.code()) > 1 {
        if numc & 1 != 0 && !bd.is_short {
            bd.endspaces += 1;
        }
        numc /= 2;
    }
    let mut char_bytes = [0u8; MB_MAXCHAR];
    let encoded_len = encode_char(c.code(), &mut char_bytes);
    let encoded = &char_bytes[..encoded_len];

    // Up to the replaced part, then the pre-spaces; then, unless the line
    // splits there, the replacement, the post-spaces and what is left of
    // the line after the block. Splitting, that tail becomes the next line,
    // NUL included.  The borrow ends before `ml_replace` writes it back.
    let (line, after, newcols) = {
        let mut lines = Lines::current();
        let oldp = lines.line(baselnum);
        let textcol = usize::try_from(bd.textcol).unwrap_or(0).min(oldp.len());
        let tail = &oldp[(textcol + width(bd.textlen)).min(oldp.len())..];
        let mut line = Vec::with_capacity(
            textcol
                + width(bd.startspaces)
                + width(numc) * encoded.len()
                + width(bd.endspaces)
                + tail.len()
                + 1,
        );
        line.extend_from_slice(&oldp[..textcol]);
        line.resize(line.len() + width(bd.startspaces), b' ');
        if splits_line {
            let mut after = Vec::with_capacity(tail.len() + 1);
            after.extend_from_slice(tail);
            after.push(0);
            (line, Some(after), 0)
        } else {
            for _ in 0..numc {
                line.extend_from_slice(encoded);
            }
            let mut newcols = line.len() - textcol;
            if !bd.is_short {
                line.resize(line.len() + width(bd.endspaces), b' ');
                newcols += width(bd.endspaces);
                line.extend_from_slice(tail);
            }
            (line, None, len_as_int(newcols))
        }
    };
    let newrows = c_int::from(after.is_some());

    let _ = ml_replace_owned(baselnum, XString::from_vec(line));
    let splice = Suppress::splice();
    if let Some(after) = after {
        let _ = ml_append_terminated(Win::current().w_cursor.lnum, &after, false);
        Win::current().w_cursor.lnum += 1;
        appended_lines_mark(Win::current().w_cursor.lnum, 1);
        op.end.lnum += 1;
    }
    drop(splice);
    let old_bytes = bd.textlen as BCount;
    let new_bytes = (newrows + newcols) as BCount;
    let row = baselnum.get() - 1;
    let (col, len) = (bd.textcol, bd.textlen);
    let op = kExtmarkUndo;
    extmark_splice(
        Buf::current(),
        row,
        col,
        0,
        ColNr::new(len),
        old_bytes,
        newrows,
        ColNr::new(newcols),
        new_bytes,
        op,
    );
}

/// The charwise and linewise arm: walk the region a character at a time.
///
/// `op` must be charwise or linewise.
fn replace_chars(op: &mut OpArg, c: Key) {
    // The cursor walks the region, so it names a position of the current
    // buffer at every step, which is what each of these asks for.
    if op.motion_type == MT_LINE_WISE {
        op.start.col = ColNr::ZERO;
        Win::current().w_cursor.col = ColNr::ZERO;
        op.end.col = Lines::current().line_len(op.end.lnum);
        if op.end.col != ColNr::ZERO {
            op.end.col -= 1;
        }
    } else if !op.inclusive {
        dec(&mut op.end);
    }

    while ltoreq(Win::current().w_cursor, op.end) {
        let mut done = false;

        let under_cursor = gchar_cursor();
        if under_cursor != NUL {
            let new_byte_len = len_as_int(c.utf8_len());
            let old_byte_len = {
                let cursor = Win::current().w_cursor;
                let mut lines = Lines::current();
                let line = lines.line(cursor.lnum);
                let col = usize::try_from(cursor.col).unwrap_or(0).min(line.len());
                len_as_int(c_cluster_len(&line[col..]))
            };

            if new_byte_len > 1 || old_byte_len > 1 {
                // Slow, but it handles a single-byte character replacing a
                // multi-byte one and the other way around.
                if Win::current().w_cursor.lnum == op.end.lnum {
                    op.end.col += new_byte_len - old_byte_len;
                }
                replace_character(c);
                done = true;
            } else {
                if under_cursor == TAB {
                    // Breaking the TAB moves the end, so remember where it
                    // was in columns first.
                    let mut end_vcol = 0;
                    if Win::current().w_cursor.lnum == op.end.lnum {
                        end_vcol = getviscol2(op.end.col, op.end.coladd).get();
                    }
                    coladvance_force(getviscol());
                    if Win::current().w_cursor.lnum == op.end.lnum {
                        getvpos(Win::current(), &mut op.end, ColNr::new(end_vcol));
                    }
                }
                // With `coladd` set the cursor may now be just past a TAB.
                if gchar_cursor() != NUL {
                    // Both sides are one byte, so the key is one.
                    pbyte(
                        Win::current().w_cursor,
                        c.as_byte().expect("a one-byte character"),
                    );
                    done = true;
                }
            }
        }

        if !done && op_virtual() && Win::current().w_cursor.lnum == op.end.lnum {
            replace_virtual_tail(op, c);
        }

        // On to the next character; stop at the end of the file.
        if inc_cursor().is_stuck() {
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
fn replace_virtual_tail(op: &OpArg, c: Key) {
    let mut virtcols = op.end.coladd;
    if Win::current().w_cursor.lnum == op.start.lnum
        && op.start.col == op.end.col
        && op.start.coladd != 0
    {
        virtcols -= op.start.coladd;
    }

    // `op.end` has been trimmed, so it is effectively inclusive: the extra
    // +1 is what keeps the NUL byte from being trampled.
    let endcol = getviscol2(op.end.col, op.end.coladd);
    coladvance_force(endcol + 1);
    Win::current().w_cursor.col -= virtcols + 1;
    while virtcols >= 0 {
        match c.as_byte().filter(|_| c.utf8_len() == 1) {
            Some(byte) => pbyte(Win::current().w_cursor, byte),
            None => replace_character(c),
        }
        if inc(&mut Win::current().w_cursor).is_stuck() {
            break;
        }
        virtcols -= 1;
    }
}
