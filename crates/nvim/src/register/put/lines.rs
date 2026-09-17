//! The two non-blockwise puts.
//!
//! [`Put::charwise_one_line`] is the common case by far -- one charwise line
//! spliced into the cursor line, `count` times -- and is separate because it
//! never adds a line, so it can splice bytes and be done. Visual mode makes
//! it a loop: `p` over a Visual *block* selection repeats the insert on every
//! line of it, at the same screen column.
//!
//! [`Put::multiline`] is everything else: a linewise register, or a charwise
//! one holding more than one line, in which case the cursor line is broken in
//! two and the register's first and last lines are joined onto the halves.
//! That is also where 'autoindent'-style reindenting (`PUT_FIXINDENT`, which
//! is `]p`) and the `'[`/`']` marks are settled.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::memory::XString;
use crate::winlayer::{Buf, PosRef, Win};
use core::ffi::{c_char, c_int, c_void};
use core::slice;

use super::Put;
use crate::edit::BeginlineOpts;
use crate::normal::visual_active;
use crate::register::*;
use crate::types::NUL;

impl Put {
    /// Splice `count` copies of the register's single line into the buffer at
    /// `col`, once per line of a Visual selection.
    pub(crate) fn charwise_one_line(&mut self, mut lnum: LineNr, mut col: ColNr) {
        // SAFETY: a charwise register holds at least one line, so `y_array`'s
        // first string is there.
        let yanklen = unsafe { (*self.y_array).len() } as c_int;
        let start_lnum = lnum;
        let mut end_lnum = 0;
        let mut first_byte_off = 0;
        let mut vcol: ColNr = 0;
        let mut totlen: size_t = 0;

        if visual_active() {
            let visual = Buf::current().b_visual;
            end_lnum = visual.vi_end.lnum.max(visual.vi_start.lnum);
            if end_lnum > start_lnum {
                // `col` is only right for the first line; the others have
                // to be found by *screen* column, which matters as soon as
                // a multi-byte character is involved.
                let mut pos = Pos {
                    lnum,
                    col,
                    coladd: 0,
                };
                let none = ::core::ptr::null_mut();
                let win = Win::current();
                // SAFETY: a writable local position; only the cursor column
                // of the three is asked for.
                unsafe { getvcol(win, &raw mut pos, none, &raw mut vcol, none) };
            }
        }

        if self.count == 0 || yanklen == 0 {
            // Nothing to put.
        } else if self.count > c_int::MAX / yanklen {
            emsg(gettext(e_resulting_text_too_long));
        } else {
            totlen = (self.count as size_t).wrapping_mul(yanklen as size_t);
            loop {
                // SAFETY: `lnum` starts at the caller's valid line and the
                // walk stops at the end of the Visual selection, so it is a
                // line of the buffer; `ml_get` hands back its NUL-terminated
                // text and `ml_get_len` its length.
                let (oldp, oldlen) = (ml_get(lnum), ml_get_len(lnum));
                if lnum > start_lnum {
                    let mut pos = Pos {
                        lnum,
                        col: 0,
                        coladd: 0,
                    };
                    // SAFETY: a live window and a writable local position.
                    let found = unsafe { getvpos(Win::current(), PosRef::new(&raw mut pos), vcol) };
                    col = if found { pos.col } else { MAXCOL };
                }
                // A Visual line too short to reach the column is skipped
                // -- upstream's `continue`, which in its do-while jumps
                // straight to the condition at the bottom.
                if visual_active() && col > oldlen {
                    lnum += 1;
                    if !(visual_active() && lnum <= end_lnum) {
                        break;
                    }
                    continue;
                }

                // SAFETY: the new line is `col` bytes of the old one, then
                // `count` copies of the register's `yanklen`-byte line, then
                // the rest of the old line and its NUL -- which is exactly
                // the `totlen + oldlen + 1` bytes asked for here.
                let room = totlen.wrapping_add(oldlen as size_t).wrapping_add(1);
                let newp = unsafe { xmalloc(room) } as *mut c_char;
                // SAFETY: as above; `oldp` is `oldlen` bytes plus a NUL and
                // `col` is within it, and the register's line is `yanklen`.
                let ptr = unsafe {
                    newp.cast::<u8>().copy_from(oldp.cast(), col as size_t);
                    let mut ptr = newp.offset(col as isize);
                    let put = (*self.y_array).data() as *const c_void;
                    for _ in 0..self.count {
                        ptr.cast::<u8>().copy_from(put.cast(), yanklen as size_t);
                        ptr = ptr.offset(yanklen as isize);
                    }
                    // +1 for the NUL.
                    let tail = oldp.offset(col as isize) as *const c_void;
                    ptr.cast::<u8>()
                        .copy_from(tail.cast(), (oldlen - col) as size_t + 1);
                    ptr
                };
                // SAFETY: `newp` is a NUL-terminated line the buffer takes
                // ownership of.
                let _ = unsafe { ml_replace(lnum, newp, false) };

                // Where the last character of the put text starts.
                //
                // SAFETY: `ptr` is one past the put text, so `ptr - 1` is its
                // last byte, and `newp` is the line it belongs to.
                first_byte_off = unsafe { utf_head_off(newp, ptr.offset(-1)) };

                if lnum == Win::current().w_cursor.lnum {
                    // Land the cursor on the last character put, keeping
                    // w_virtcol right.
                    changed_cline_bef_curs(Win::current());
                    invalidate_botline_win(Win::current());
                    Win::current().w_cursor.col += (totlen - 1) as ColNr;
                }
                changed_bytes(lnum, col);
                let inserted = totlen as c_int;
                let buffer = Buf::current();
                extmark_splice_cols(buffer, lnum - 1, col, 0, inserted, kExtmarkUndo);
                if visual_active() {
                    lnum += 1;
                }
                if !(visual_active() && lnum <= end_lnum) {
                    break;
                }
            }
        }

        // `']` goes on the *first byte* of the last character put.
        Buf::current().b_op_end = Win::current().w_cursor;
        Buf::current().b_op_end.col -= first_byte_off;

        // `CTRL-O p` in Insert mode leaves the cursor after the last
        // character rather than on it.
        if totlen != 0 && (restart_edit.get() != 0 || self.flags & PUT_CURSEND as c_int != 0) {
            Win::current().w_cursor.col += 1;
        } else {
            Win::current().w_cursor.col -= first_byte_off;
        }
    }

    /// Break the cursor line in two and hang the register's first and last
    /// lines off the halves.
    ///
    /// Answers the line the second half ended up on, and how many bytes of
    /// the register's first line went onto the first half.
    fn split_line_for_charwise(&self, lnum: LineNr, col: ColNr) {
        // The tail of the cursor line, with the register's *last* line in
        // front of it, becomes a new line below.
        //
        // SAFETY: the caller promises `lnum`/`col` is a valid position, so
        // `ml_get` hands back a NUL-terminated line with at least `col` bytes
        // in it; `y_array` holds `y_size` strings and `y_size` is at least
        // one, so the last is there.
        let mut appended = unsafe {
            let tail = ml_get(lnum).offset(col as isize);
            let tail_len = (ml_get_len(lnum) - col) as size_t;
            let last = &*self.y_array.add(self.y_size.wrapping_sub(1));
            let mut joined = XString::from_bytes(last.as_bytes());
            joined.push_bytes(slice::from_raw_parts(tail.cast::<u8>(), tail_len));
            joined
        };
        // SAFETY: the line just built, which `ml_append` copies.
        let _ = unsafe { ml_append(lnum, appended.as_mut_ptr(), 0, false) };

        // The head of the cursor line keeps the register's *first* line.
        //
        // SAFETY: the same position, re-read because `ml_append` moved the
        // line; `newp` is `col` bytes of it followed by the register's first
        // line and that line's NUL, which is the `col + yanklen + 1` asked
        // for, and `ml_replace` takes ownership of it.
        let replacement = unsafe {
            let oldp = ml_get(lnum);
            let mut head =
                XString::from_bytes(slice::from_raw_parts(oldp.cast::<u8>(), col as size_t));
            head.push_bytes((*self.y_array).as_bytes());
            head
        };
        // SAFETY: the line just built, whose block `ml_replace` takes over.
        let _ = unsafe { ml_replace(lnum, replacement.into_raw(), false) };
    }

    /// Reindent line `lnum` the way `]p` wants: keep the *relative* indent of
    /// the register's lines, but move the block as a whole to `orig_indent`.
    fn fix_indent(&self, lnum: LineNr, state: &mut FixIndent) {
        let old_pos = Win::current().w_cursor;
        Win::current().w_cursor.lnum = lnum;
        // SAFETY: the caller promises `lnum` is a line of the buffer, so
        // `ml_get` hands back its NUL-terminated text.
        let first = unsafe { c_int::from(*ml_get(lnum)) };
        // A `#` line stays at the start of the line, and an empty line
        // has no indent to keep.
        //
        let indent = if (first == '#' as c_int && preprocs_left()) || first == NUL {
            0
        } else if state.first {
            state.diff = state.orig_indent - get_indent();
            state.first = false;
            state.orig_indent
        } else {
            (get_indent() + state.diff).max(0)
        };
        set_indent(indent, SIN_NOMARK);
        Win::current().w_cursor = old_pos;
    }

    /// The `'[` and `']` marks, and where the cursor ends up.
    fn multiline_marks(
        &self,
        lnum: LineNr,
        new_lnum: LineNr,
        new_cursor: Pos,
        col: ColNr,
        lendiff: c_int,
    ) {
        if self.y_type == kMTLineWise {
            Buf::current().b_op_start.col = 0;
            if self.dir == FORWARD {
                Buf::current().b_op_start.lnum += 1;
            }
        }

        // Only a plain linewise put moves the marks itself; a split put
        // has already spliced them.
        let kind = if self.y_type == kMTLineWise && self.flags & PUT_LINE_SPLIT as c_int == 0 {
            kExtmarkUndo
        } else {
            kExtmarkNOOP
        };
        let from = Buf::current().b_op_start.lnum + LineNr::from(self.y_type == kMTCharWise);
        mark_adjust(from, MAXLNUM, self.nr_lines, 0, kind);

        // SAFETY (both): a live buffer, and the range is the lines the put
        // just rewrote.
        if self.y_type == kMTCharWise {
            let at = Win::current().w_cursor.lnum;
            changed_lines(Buf::current(), at, col, at + 1, self.nr_lines, true);
        } else {
            let at = Buf::current().b_op_start.lnum;
            changed_lines(Buf::current(), at, 0, at, self.nr_lines, true);
        }

        // `']` goes on the first byte of the last character put, its
        // column corrected for whatever the reindent above removed.
        Buf::current().b_op_end.lnum = new_lnum;
        // SAFETY: `y_array` holds `y_size` strings and `y_size` is at least
        // one, so the last is there.
        let last = unsafe { &*self.y_array.add(self.y_size.wrapping_sub(1)) };
        let col = (last.len() as ColNr - lendiff).max(0);
        if col > 1 {
            Buf::current().b_op_end.col = col - 1;
            if !last.is_empty() {
                // SAFETY: `last` is NUL-terminated and `len()` bytes long, so
                // its final byte is one of them.
                let head = unsafe { utf_head_off(last.data(), last.data().add(last.len() - 1)) };
                Buf::current().b_op_end.col -= head;
            }
        } else {
            Buf::current().b_op_end.col = 0;
        }

        if self.flags & PUT_CURSLINE as c_int != 0 {
            // `:put`: the cursor goes on the last inserted line.
            Win::current().w_cursor.lnum = lnum;
            // SAFETY: the cursor is on a line of the current buffer.
            beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
        } else if self.flags & PUT_CURSEND as c_int != 0 {
            // The cursor goes after the inserted text.
            if self.y_type == kMTLineWise {
                Win::current().w_cursor.lnum = if lnum >= Buf::current().b_ml.ml_line_count {
                    Buf::current().b_ml.ml_line_count
                } else {
                    lnum + 1
                };
                Win::current().w_cursor.col = 0;
            } else {
                Win::current().w_cursor.lnum = new_lnum;
                Win::current().w_cursor.col = col;
                Buf::current().b_op_end = Win::current().w_cursor;
                if col > 1 {
                    Buf::current().b_op_end.col = col - 1;
                }
            }
        } else if self.y_type == kMTLineWise {
            // The cursor goes on the first non-blank of the first line.
            Win::current().w_cursor.col = 0;
            if self.dir == FORWARD {
                Win::current().w_cursor.lnum += 1;
            }
            // SAFETY: the cursor is on a line of the current buffer.
            beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
        } else {
            // The cursor goes on the first character put.
            Win::current().w_cursor = new_cursor;
        }
    }

    /// The linewise put, and the charwise put of more than one line.
    pub(crate) fn multiline(&mut self, mut lnum: LineNr, col: ColNr, new_cursor: Pos) {
        let mut new_lnum = new_cursor.lnum;
        let mut lendiff = 0;
        let mut indent_state = FixIndent {
            orig_indent: if self.flags & PUT_FIXINDENT as c_int != 0 {
                // SAFETY: the cursor is on a line of the current buffer.
                get_indent()
            } else {
                0
            },
            diff: 0,
            first: true,
        };

        // At least one line goes in. A charwise register breaks the first
        // one in two.
        'error: {
            for cnt in 1..=self.count {
                let mut i: size_t = 0;
                if self.y_type == kMTCharWise {
                    lnum = new_cursor.lnum;
                    self.split_line_for_charwise(lnum, col);
                    new_lnum += 1;
                    Win::current().w_cursor.lnum = lnum;
                    i = 1;
                }

                while i < self.y_size {
                    // A charwise register's last line is already on the
                    // second half of the split.
                    if self.y_type != kMTCharWise || i < self.y_size.wrapping_sub(1) {
                        // SAFETY: `i` is below `y_size`, so `y_array`'s `i`th
                        // string is there and NUL-terminated; `lnum` is a
                        // line of the buffer, and `ml_append` copies the text.
                        let text = unsafe { (*self.y_array.add(i)).data() };
                        if unsafe { ml_append(lnum, text, 0, false) }.is_err() {
                            break 'error;
                        }
                        new_lnum += 1;
                    }
                    lnum += 1;
                    self.nr_lines += 1;
                    if self.flags & PUT_FIXINDENT as c_int != 0 {
                        // Only the very last line's length is wanted, to see
                        // what the reindent took off it.
                        let measured = cnt == self.count && i == self.y_size.wrapping_sub(1);
                        if measured {
                            lendiff = ml_get_len(lnum);
                        }
                        self.fix_indent(lnum, &mut indent_state);
                        if measured {
                            lendiff -= ml_get_len(lnum);
                        }
                    }
                    i = i.wrapping_add(1);
                }

                // Splice the extmarks for what was inserted. A linewise
                // put that did *not* split a line has its marks moved by
                // `mark_adjust` below instead.
                let splits_a_line =
                    self.y_type == kMTLineWise && self.flags & PUT_LINE_SPLIT as c_int != 0;
                if self.y_type == kMTCharWise || splits_a_line {
                    let mut totsize: BCount = 0;
                    for i in 0..self.y_size.wrapping_sub(1) {
                        // SAFETY: `i` is below `y_size`.
                        totsize += unsafe { (*self.y_array.add(i)).len() } as BCount + 1;
                    }
                    let last = self.y_size.wrapping_sub(1);
                    // SAFETY: `y_size` is at least one, so `last` names a line.
                    let lastsize = unsafe { (*self.y_array.add(last)).len() } as c_int;
                    totsize += lastsize as BCount;

                    let buf = Buf::current();
                    let at = new_cursor.lnum - 1;
                    let (start, rows, cols, bytes) = if self.y_type == kMTCharWise {
                        (col, self.y_size as c_int - 1, lastsize, totsize)
                    } else {
                        // Account for the last pasted newline and the
                        // newline the split itself added.
                        (self.split_pos, self.y_size as c_int + 1, 0, totsize + 2)
                    };
                    extmark_splice(buf, at, start, 0, 0, 0, rows, cols, bytes, kExtmarkUndo);
                }

                if cnt == 1 {
                    new_lnum = lnum;
                }
            }
        }

        self.multiline_marks(lnum, new_lnum, new_cursor, col, lendiff);
    }
}

/// What `PUT_FIXINDENT` carries from one line to the next.
struct FixIndent {
    /// The indent of the line the put started on, which the whole block is
    /// moved to.
    orig_indent: c_int,
    /// `orig_indent` minus the first pasted line's own indent, added to every
    /// line after it so that the block keeps its shape.
    diff: c_int,
    /// Whether `diff` still has to be measured.
    first: bool,
}
