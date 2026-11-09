//! `nvim_buf_set_text()`: replacing an arbitrary byte range.
//!
//! The one API call that can start and end mid-line, which is why it owns
//! three cursor fixups of its own: `fix_cursor` for a whole-line change,
//! `fix_pos_col` for a mark or cursor column inside the replaced span, and
//! `fix_cursor_cols` for the columns of every window showing the buffer.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api::private::validate::err_out_of_range;
use crate::extmark::kExtmarkNOOP;
use crate::extmark::kExtmarkUndo;
use crate::mark::kMarkAdjustApi;
use crate::memline::Lines;
use crate::r#move::WinValid;
use crate::narrow::{len_as_int, len_as_integer};

use super::lines::{byte_count, line_below, line_span};
use crate::normal::{set_visual_anchor, visual_active, visual_anchor, visual_mode};
use crate::winlayer::{Buf, Win, tab_windows};

/// `col` as a byte offset into `line`, if it lands inside it or just past
/// its end; a negative one counts back from the end, `-1` being the end.
fn column_in(line: &[u8], col: Integer) -> Option<usize> {
    let len = len_as_integer(line.len());
    let col = if col < 0 { len + col + 1 } else { col };
    usize::try_from(col).ok().filter(|&col| col <= line.len())
}

/// Replace the text from (`start_row`, `start_col`) to (`end_row`,
/// `end_col`) of `buf` with `replacement`'s lines.
pub fn nvim_buf_set_text(
    channel_id: uint64_t,
    buf: BufferHandle,
    start_row: Integer,
    start_col: Integer,
    end_row: Integer,
    end_col: Integer,
    mut replacement: Array,
) -> Result<(), Error> {
    if replacement.is_empty() {
        // An empty replacement deletes the range, which is the same as
        // replacing it with one empty line.
        replacement = Array::from([Object::string(ApiString::from_cstr(c""))]);
    }
    let Some(buffer) = api_buf_ensure_loaded(buf)? else {
        return Ok(());
    };
    let mut oob: bool = false;
    let start_row = normalize_index(buffer, start_row, false, &mut oob);
    if oob {
        return Err(err_out_of_range(c"start_row"));
    }
    let end_row = normalize_index(buffer, end_row, false, &mut oob);
    if oob {
        return Err(err_out_of_range(c"end_row"));
    }
    let at_start: Vec<u8> = Lines::in_buffer(buffer).line(start_row).to_vec();
    let start_col =
        column_in(&at_start, start_col).ok_or_else(|| err_out_of_range(c"start_col"))?;
    let at_end: Vec<u8> = Lines::in_buffer(buffer).line(end_row).to_vec();
    let end_col = column_in(&at_end, end_col).ok_or_else(|| err_out_of_range(c"end_col"))?;
    if !(start_row <= end_row && !(end_row == start_row && start_col > end_col)) {
        return Err(Error::validation(c"'start' is higher than 'end'"));
    }
    let disallow_nl: bool = channel_id != VIML_INTERNAL_CALL;
    check_string_array(&replacement, c"replacement string", disallow_nl)?;
    let new_len = replacement.len();
    let old_bytes = if start_row == end_row {
        end_col - start_col
    } else {
        let between: usize = (start_row + 1)
            .up_to(end_row)
            .map(|lnum| {
                let len = ml_get_buf_len(buffer, lnum);
                usize::try_from(len).expect("a line length is not negative") + 1
            })
            .sum();
        at_start.len() - start_col + between + end_col + 1
    };
    // Every item is a String: `check_string_array` above turned anything else
    // into an error.
    let only_strings = "check_string_array accepted only Strings";
    let first_item: ApiString = replacement[0].as_string().expect(only_strings).clone();
    let last_item: ApiString = replacement[new_len - 1]
        .as_string()
        .expect(only_strings)
        .clone();
    // The new lines, each NUL-terminated: the first keeps the old start
    // line's head and the last the old end line's tail, which are one line
    // when the replacement is.
    let head = &at_start[..start_col];
    let tail = &at_end[end_col..];
    let mut lines: Vec<Vec<u8>> = Vec::with_capacity(new_len);
    let mut first = head.to_vec();
    api_text_into(&mut first, first_item.as_bytes());
    if new_len > 1 {
        first.push(0);
    }
    let mut new_bytes = first_item.len();
    lines.push(first);
    for item in replacement.iter().take(new_len.saturating_sub(1)).skip(1) {
        let l = item.as_string().expect(only_strings);
        lines.push(api_line(l.as_bytes()));
        new_bytes += l.len() + 1;
    }
    if new_len > 1 {
        let mut last = Vec::with_capacity(last_item.len() + tail.len() + 1);
        api_text_into(&mut last, last_item.as_bytes());
        lines.push(last);
        new_bytes += last_item.len() + 1;
    }
    let last = lines.last_mut().expect("the replacement has a first line");
    last.extend_from_slice(tail);
    last.push(0);
    let mut tstate: TryState = TryState::INIT;
    try_enter(&mut tstate);
    let edit = Replacement {
        buffer,
        start_row,
        start_col: ColNr::new(len_as_int(start_col)),
        end_row,
        end_col: ColNr::new(len_as_int(end_col)),
        lines,
        last_len: ColNr::new(len_as_int(last_item.len())),
        old_byte: byte_count(old_bytes),
        new_byte: byte_count(new_bytes),
    };
    let outcome = edit.apply();
    // The bracket outranks whatever the body answered, which is the order the
    // two had when both went through one slot.
    try_leave(&tstate)?;
    outcome
}

/// The edit `nvim_buf_set_text` has prepared, once its arguments are checked
/// and the replacement lines are built.
#[derive(Debug)]
struct Replacement {
    buffer: Buf,
    start_row: LineNr,
    start_col: ColNr,
    end_row: LineNr,
    end_col: ColNr,
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
        let (start_row, end_row) = (self.start_row, self.end_row);
        if u_save_buf(b, start_row - 1, end_row + 1).is_err() {
            return Err(Error::exception(c"Failed to save undo information"));
        }
        let extra = self.write_lines()?;
        let new_rows = len_as_int(self.lines.len());

        let col_extent: i32 = self.end_col
            - if end_row == start_row {
                self.start_col
            } else {
                ColNr::ZERO
            };
        let adjust: i32 = if end_row >= start_row {
            MAXLNUM.get()
        } else {
            0
        };
        mark_adjust_buf(
            b,
            start_row,
            end_row - 1,
            adjust,
            extra,
            true,
            kMarkAdjustApi,
            kExtmarkNOOP,
        );
        if visual_active() && b.raw() == Buf::current_raw() && !visual_mode().is_block() {
            let mut anchor = visual_anchor();
            fix_pos_col(
                b,
                &mut anchor,
                start_row,
                self.start_col,
                end_row,
                self.end_col,
                new_rows,
                self.last_len.get(),
                1,
            );
            set_visual_anchor(anchor);
            check_visual_pos();
        }
        extmark_splice(
            b,
            start_row.get() - 1,
            self.start_col,
            end_row - start_row,
            ColNr::new(col_extent),
            self.old_byte,
            new_rows - 1,
            self.last_len,
            self.new_byte,
            kExtmarkUndo,
        );
        changed_lines(b, start_row, self.start_col, end_row + 1, extra, true);
        for win in tab_windows() {
            if win.w_buffer != b {
                continue;
            }
            if (start_row..=end_row).contains(&win.w_cursor.lnum) {
                fix_cursor_cols(
                    win,
                    start_row,
                    self.start_col,
                    end_row,
                    self.end_col,
                    new_rows,
                    self.last_len.get(),
                );
            } else {
                fix_cursor(win, start_row, end_row, extra);
            }
        }
        Ok(())
    }

    /// Delete, replace and append until the range holds the new lines,
    /// answering how many lines the buffer grew by.
    fn write_lines(&mut self) -> Result<i32, Error> {
        let b = self.buffer;
        let old_len = line_span(self.start_row, self.end_row) + 1;
        let new_len = self.lines.len();
        let to_delete = old_len.saturating_sub(new_len);
        for _ in 0..to_delete {
            if ml_delete_buf(b, self.start_row, false).is_err() {
                return Err(Error::exception(c"Failed to delete line"));
            }
        }
        let mut extra = -len_as_int(to_delete);
        let to_replace = old_len.min(new_len);
        for (i, line) in self.lines.iter().enumerate().take(to_replace) {
            let lnum = line_below(self.start_row, i)?;
            // The memline writes the line straight into its block.
            if ml_replace_buf_now(b, lnum, line).is_err() {
                return Err(Error::exception(c"Failed to replace line"));
            }
        }
        for (i, line) in self.lines.iter().enumerate().skip(to_replace) {
            let lnum = line_below(self.start_row - 1, i)?;
            if ml_append_buf_terminated(b, lnum, line, false).is_err() {
                return Err(Error::exception(c"Failed to insert line"));
            }
            extra += 1;
        }
        Ok(extra)
    }
}

pub(crate) fn fix_cursor(mut win: Win, lo: LineNr, hi: LineNr, extra: i32) {
    if win.w_cursor.lnum >= lo {
        if win.w_cursor.lnum >= hi {
            win.w_cursor.lnum += extra;
        } else if extra < 0 {
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

/// Moves `pos` to where the text it was in went, after rows
/// `start_row..=end_row` were replaced.
fn fix_pos_col(
    buffer: Buf,
    pos: &mut Pos,
    start_row: LineNr,
    start_col: ColNr,
    end_row: LineNr,
    end_col: ColNr,
    new_rows: i32,
    new_cols_at_end_row: i32,
    mode_col_adj: i32,
) {
    if pos.lnum < start_row {
        return;
    }
    let old_rows: i32 = end_row - start_row + 1;
    let lnum_shift = new_rows - old_rows;
    if pos.lnum > end_row {
        pos.lnum += lnum_shift;
        return;
    }
    let end_row_change_start: ColNr = if new_rows == 1 {
        start_col
    } else {
        ColNr::ZERO
    };
    let end_row_change_end: ColNr = end_row_change_start + new_cols_at_end_row;
    if pos.lnum == end_row && pos.col + mode_col_adj > end_col {
        pos.lnum += lnum_shift;
        pos.col += end_row_change_end - end_col;
        return;
    }
    let old_coladd: i32 = pos.coladd;
    let coladd = pos.coladd;
    pos.col += coladd;
    pos.coladd = 0;
    let new_end_row: LineNr = start_row + new_rows - 1;
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
    new_rows: i32,
    new_cols_at_end_row: i32,
) {
    let mode_col_adj: i32 = if win == Win::current() && STATE.get() & MODE_INSERT != 0 {
        0
    } else {
        1
    };
    let mut cursor = win.w_cursor;
    fix_pos_col(
        win.buffer(),
        &mut cursor,
        start_row,
        start_col,
        end_row,
        end_col,
        new_rows,
        new_cols_at_end_row,
        mode_col_adj,
    );
    win.w_cursor = cursor;
    check_cursor_col(win);
    changed_cline_bef_curs(win);
    invalidate_botline_win(win);
}
