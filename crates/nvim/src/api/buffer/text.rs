//! `nvim_buf_set_text()`: replacing an arbitrary byte range.
//!
//! The one API call that can start and end mid-line, which is why it owns
//! three cursor fixups of its own: `fix_cursor` for a whole-line change,
//! `fix_pos_col` for a mark or cursor column inside the replaced span, and
//! `fix_cursor_cols` for the columns of every window showing the buffer.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::api::private::validate::err_out_of_range;
use crate::memline::Lines;
use crate::r#move::WinValid;
use crate::normal::{set_visual_anchor, visual_active, visual_anchor, visual_mode};
use crate::winlayer::{Buf, PosRef, Win, tab_windows};

/// Replace the text from (`start_row`, `start_col`) to (`end_row`,
/// `end_col`) of `buf` with `replacement`'s lines.
pub fn nvim_buf_set_text(
    channel_id: uint64_t,
    buf: BufferHandle,
    mut start_row: Integer,
    mut start_col: Integer,
    mut end_row: Integer,
    mut end_col: Integer,
    mut replacement: Array,
) -> Result<(), Error> {
    if replacement.is_empty() {
        // An empty replacement deletes the range, which is the same as
        // replacing it with one empty line.
        replacement.push(Object::string(String_0::from_cstr(c"")));
    }
    let Some(b) = api_buf_ensure_loaded(buf)? else {
        return Ok(());
    };
    let buffer = b;
    let mut oob: bool = false;
    start_row = unsafe { normalize_index(b, start_row as int64_t, false, &raw mut oob) } as Integer;
    if oob {
        return Err(err_out_of_range(c"start_row"));
    }
    end_row = unsafe { normalize_index(b, end_row as int64_t, false, &raw mut oob) } as Integer;
    if oob {
        return Err(err_out_of_range(c"end_row"));
    }
    let at_start: Vec<u8> = Lines::in_buffer(b).line(start_row as LineNr).to_vec();
    let len_at_start = at_start.len() as ColNr;
    start_col = if start_col < 0 as Integer {
        len_at_start as Integer + start_col + 1 as Integer
    } else {
        start_col
    };
    if !(start_col >= 0 as Integer && start_col <= len_at_start as Integer) {
        return Err(err_out_of_range(c"start_col"));
    }
    let at_end: Vec<u8> = Lines::in_buffer(b).line(end_row as LineNr).to_vec();
    let len_at_end = at_end.len() as ColNr;
    end_col = if end_col < 0 as Integer {
        len_at_end as Integer + end_col + 1 as Integer
    } else {
        end_col
    };
    if !(end_col >= 0 as Integer && end_col <= len_at_end as Integer) {
        return Err(err_out_of_range(c"end_col"));
    }
    if !(start_row <= end_row && !(end_row == start_row && start_col > end_col)) {
        return Err(Error::validation(c"'start' is higher than 'end'"));
    }
    let disallow_nl: bool = channel_id != VIML_INTERNAL_CALL;
    check_string_array(&replacement, c"replacement string", disallow_nl)?;
    let new_len: size_t = replacement.len();
    let mut new_byte: BCount = 0 as BCount;
    let mut old_byte: BCount = 0 as BCount;
    if start_row == end_row {
        old_byte = end_col as BCount - start_col as BCount;
    } else {
        old_byte = (old_byte as ::core::ffi::c_long
            + (len_at_start as Integer - start_col) as ::core::ffi::c_long)
            as BCount;
        let mut i: int64_t = 1 as int64_t;
        while i < end_row - start_row {
            let lnum: int64_t = start_row as int64_t + i;
            old_byte += (ml_get_buf_len(b, lnum as LineNr) + 1 as ::core::ffi::c_int) as BCount;
            i += 1;
        }
        old_byte += end_col as BCount + 1 as BCount;
    }
    let last_index = replacement.len().wrapping_sub(1 as size_t);
    // Every item is a String: `check_string_array` above turned anything else
    // into an error.
    let only_strings = "check_string_array accepted only Strings";
    let first_item: String_0 = replacement[0].as_string().expect(only_strings).clone();
    // SAFETY: as above.
    let last_item: String_0 = replacement[last_index]
        .as_string()
        .expect(only_strings)
        .clone();
    // The new lines, each NUL-terminated: the first keeps the old start
    // line's head and the last the old end line's tail, which are one line
    // when the replacement is.
    let head = &at_start[..start_col as usize];
    let tail = &at_end[end_col as usize..];
    let mut lines: Vec<Vec<u8>> = Vec::with_capacity(new_len);
    let mut first = head.to_vec();
    api_text_into(&mut first, first_item.as_bytes());
    if new_len > 1 {
        first.push(0);
    }
    new_byte += first_item.len() as BCount;
    lines.push(first);
    for i in 1..new_len.saturating_sub(1) {
        // SAFETY: `i` is below `replacement.size`.
        let l = replacement[i].as_string().expect(only_strings);
        lines.push(api_line(l.as_bytes()));
        new_byte += l.len() as BCount + 1 as BCount;
    }
    if new_len > 1 {
        let mut last = Vec::with_capacity(last_item.len() + tail.len() + 1);
        api_text_into(&mut last, last_item.as_bytes());
        lines.push(last);
        new_byte += last_item.len() as BCount + 1 as BCount;
    }
    let last = lines.last_mut().expect("the replacement has a first line");
    last.extend_from_slice(tail);
    last.push(0);
    let mut tstate: TryState = TryState::INIT;
    unsafe { try_enter(&raw mut tstate) };
    let edit = Replacement {
        buffer,
        start_row,
        start_col,
        end_row,
        end_col,
        lines,
        last_len: last_item.len() as ColNr,
        old_byte,
        new_byte,
    };
    let outcome = edit.apply();
    // The bracket outranks whatever the body answered, which is the order the
    // two had when both went through one slot.
    unsafe { try_leave(&raw mut tstate) }?;
    outcome
}

/// The edit `nvim_buf_set_text` has prepared, once its arguments are checked
/// and the replacement lines are built.
struct Replacement {
    buffer: Buf,
    start_row: Integer,
    start_col: Integer,
    end_row: Integer,
    end_col: Integer,
    /// The lines that replace the range, each NUL-terminated.
    lines: Vec<Vec<u8>>,
    /// How long the last one is: where the range's tail ends up.
    last_len: ColNr,
    /// How many bytes the range held, and how many replace them.
    old_byte: BCount,
    new_byte: BCount,
}

impl Replacement {
    /// Write the lines in, then tell the marks, the extmarks and every
    /// window's cursor where the text moved.
    fn apply(mut self) -> Result<(), Error> {
        let b = self.buffer;
        if b.b_p_ma == 0 {
            return Err(Error::exception(c"Buffer is not 'modifiable'"));
        }
        let (from, to) = (self.start_row as LineNr - 1, self.end_row as LineNr + 1);
        if u_save_buf(self.buffer, from, to).is_err() {
            return Err(Error::exception(c"Failed to save undo information"));
        }
        let extra = self.write_lines()?;
        let new_len = self.lines.len();

        let col_extent: ColNr = (self.end_col
            - if self.end_row == self.start_row {
                self.start_col
            } else {
                0
            }) as ColNr;
        let adjust: LineNr = if self.end_row >= self.start_row {
            MAXLNUM as LineNr
        } else {
            0
        };
        mark_adjust_buf(
            self.buffer,
            self.start_row as LineNr,
            self.end_row as LineNr - 1,
            adjust,
            extra as LineNr,
            true,
            kMarkAdjustApi,
            kExtmarkNOOP,
        );
        if visual_active() && b.raw() == Buf::current_raw() && !visual_mode().is_block() {
            let mut anchor = visual_anchor();
            // SAFETY: `anchor` is this frame's own position.
            unsafe {
                fix_pos_col(
                    b,
                    &raw mut anchor,
                    self.start_row as LineNr,
                    self.start_col as ColNr,
                    self.end_row as LineNr,
                    self.end_col as ColNr,
                    new_len as LineNr,
                    self.last_len,
                    1,
                )
            };
            set_visual_anchor(anchor);
            check_visual_pos();
        }
        extmark_splice(
            self.buffer,
            self.start_row as ::core::ffi::c_int - 1,
            self.start_col as ColNr,
            (self.end_row - self.start_row) as ::core::ffi::c_int,
            col_extent,
            self.old_byte,
            new_len as ::core::ffi::c_int - 1,
            self.last_len,
            self.new_byte,
            kExtmarkUndo,
        );
        changed_lines(
            self.buffer,
            self.start_row as LineNr,
            self.start_col as ColNr,
            self.end_row as LineNr + 1,
            extra as LineNr,
            true,
        );
        for win in tab_windows() {
            if win.w_buffer != b {
                continue;
            }
            let cursor = win.w_cursor.lnum as Integer;
            if cursor >= self.start_row && cursor <= self.end_row {
                fix_cursor_cols(
                    win,
                    self.start_row as LineNr,
                    self.start_col as ColNr,
                    self.end_row as LineNr,
                    self.end_col as ColNr,
                    new_len as LineNr,
                    self.last_len,
                );
            } else {
                let (lo, hi) = (self.start_row as LineNr, self.end_row as LineNr);
                fix_cursor(win, lo, hi, extra as LineNr);
            }
        }
        Ok(())
    }

    /// Delete, replace and append until the range holds the new lines,
    /// answering how many lines the buffer grew by.
    fn write_lines(&mut self) -> Result<ptrdiff_t, Error> {
        let b = self.buffer;
        let mut extra: ptrdiff_t = 0;
        let old_len: size_t = (self.end_row - self.start_row + 1) as size_t;
        let new_len = self.lines.len();
        let to_delete = old_len.saturating_sub(new_len);
        for _ in 0..to_delete {
            if ml_delete_buf(b, self.start_row as LineNr, false).is_err() {
                return Err(Error::exception(c"Failed to delete line"));
            }
        }
        extra -= to_delete as ptrdiff_t;
        let to_replace = old_len.min(new_len);
        for i in 0..to_replace {
            let lnum: int64_t = self.start_row as int64_t + i as int64_t;
            if lnum >= MAXLNUM as int64_t {
                return Err(Error::validation(c"Index out of bounds"));
            }
            let line = self.lines[i].as_mut_ptr().cast();
            // SAFETY: a loaded buffer and a line that is still in it; the
            // memline borrows `line` (`noalloc`) and flushes it straight out.
            if unsafe { ml_replace_buf(b, lnum as LineNr, line, false, true) }.is_err() {
                return Err(Error::exception(c"Failed to replace line"));
            }
        }
        for i in to_replace..new_len {
            let lnum: int64_t = self.start_row as int64_t + i as int64_t - 1;
            if lnum >= MAXLNUM as int64_t {
                return Err(Error::validation(c"Index out of bounds"));
            }
            let line = self.lines[i].as_mut_ptr().cast();
            // SAFETY: a loaded buffer; the append copies the NUL-terminated
            // `line`.
            if unsafe { ml_append_buf(b, lnum as LineNr, line, 0, false) }.is_err() {
                return Err(Error::exception(c"Failed to insert line"));
            }
            extra += 1;
        }
        Ok(extra)
    }
}

pub(crate) fn fix_cursor(mut win: Win, lo: LineNr, hi: LineNr, extra: LineNr) {
    if win.w_cursor.lnum >= lo {
        if win.w_cursor.lnum >= hi {
            win.w_cursor.lnum += extra;
        } else if extra < 0 as LineNr {
            check_cursor_lnum(win);
        }
        check_cursor_col(win);
        changed_cline_bef_curs(win);
        win.w_valid.clear(WinValid::BOTLINE_AP);
        update_topline(win);
    } else {
        invalidate_botline_win(win);
    };
}

/// # Safety
///
/// `pos` must point at an initialized position, unaliased for the call.
unsafe fn fix_pos_col(
    buffer: Buf,
    pos: *mut Pos,
    start_row: LineNr,
    start_col: ColNr,
    end_row: LineNr,
    end_col: ColNr,
    new_rows: LineNr,
    new_cols_at_end_row: ColNr,
    mode_col_adj: ColNr,
) {
    // SAFETY: the caller's promise -- `pos` is a live position, and nothing
    // below can move it.
    let mut pos = unsafe { PosRef::new(pos) };
    if pos.lnum < start_row {
        return;
    }
    let old_rows: LineNr = end_row - start_row + 1 as LineNr;
    let lnum_shift: LineNr = new_rows - old_rows;
    if pos.lnum > end_row {
        pos.lnum += lnum_shift;
        return;
    }
    let end_row_change_start: ColNr = if new_rows == 1 as LineNr {
        start_col
    } else {
        0 as ColNr
    };
    let end_row_change_end: ColNr = end_row_change_start + new_cols_at_end_row;
    if pos.lnum == end_row && pos.col + mode_col_adj > end_col {
        pos.lnum += lnum_shift;
        pos.col += end_row_change_end - end_col;
        return;
    }
    let old_coladd: ColNr = pos.coladd;
    let coladd = pos.coladd;
    pos.col += coladd;
    pos.coladd = 0;
    let new_end_row: LineNr = start_row + new_rows - 1 as LineNr;
    if pos.lnum > new_end_row {
        pos.lnum = new_end_row;
        let len: ColNr = ml_get_buf_len(buffer, new_end_row);
        if pos.col < len {
            pos.col = len;
        }
    }
    if pos.lnum == new_end_row && pos.col > end_row_change_end && old_coladd == 0 {
        pos.col = end_row_change_end;
        if pos.col - mode_col_adj >= end_row_change_start {
            pos.col -= mode_col_adj;
        }
    }
}

fn fix_cursor_cols(
    mut win: Win,
    start_row: LineNr,
    start_col: ColNr,
    end_row: LineNr,
    end_col: ColNr,
    new_rows: LineNr,
    new_cols_at_end_row: ColNr,
) {
    let mode_col_adj: ColNr = if win == Win::current() && State.get() & MODE_INSERT != 0 {
        0 as ColNr
    } else {
        1 as ColNr
    };
    unsafe {
        fix_pos_col(
            win.buffer(),
            &raw mut win.w_cursor,
            start_row,
            start_col,
            end_row,
            end_col,
            new_rows,
            new_cols_at_end_row,
            mode_col_adj,
        )
    };
    check_cursor_col(win);
    changed_cline_bef_curs(win);
    invalidate_botline_win(win);
}
