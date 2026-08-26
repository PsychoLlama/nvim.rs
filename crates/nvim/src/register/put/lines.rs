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

use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int, c_void};

use super::Put;
use crate::edit::BeginlineOpts;
use crate::normal::visual_active;
use crate::register::*;
use crate::types::{FAIL, NUL};

impl Put {
    /// Splice `count` copies of the register's single line into the buffer at
    /// `col`, once per line of a Visual selection.
    ///
    /// # Safety
    /// `lnum`/`col` must be a valid position, and undo already saved.
    pub(crate) unsafe fn charwise_one_line(&mut self, mut lnum: linenr_T, mut col: colnr_T) {
        // SAFETY: a charwise register holds at least one line, so `y_array`'s
        // first string is there.
        let yanklen = unsafe { (*self.y_array).len() } as c_int;
        let start_lnum = lnum;
        let mut end_lnum = 0;
        let mut first_byte_off = 0;
        let mut vcol: colnr_T = 0;
        let mut totlen: size_t = 0;

        if visual_active() {
            let visual = cur_buf().b_visual;
            end_lnum = visual.vi_end.lnum.max(visual.vi_start.lnum);
            if end_lnum > start_lnum {
                // `col` is only right for the first line; the others have
                // to be found by *screen* column, which matters as soon as
                // a multi-byte character is involved.
                let mut pos = pos_T {
                    lnum,
                    col,
                    coladd: 0,
                };
                let none = ::core::ptr::null_mut();
                // SAFETY: a live window and a writable local position; only
                // the cursor column of the three is asked for.
                unsafe { getvcol(curwin.get(), &raw mut pos, none, &raw mut vcol, none) };
            }
        }

        if self.count == 0 || yanklen == 0 {
            if visual_active() {
                lnum = end_lnum;
            }
        } else if self.count > c_int::MAX / yanklen {
            // SAFETY: a NUL-terminated message literal.
            unsafe {
                emsg(gettext(
                    &raw const e_resulting_text_too_long as *const c_char,
                ))
            };
        } else {
            totlen = (self.count as size_t).wrapping_mul(yanklen as size_t);
            loop {
                // SAFETY: `lnum` starts at the caller's valid line and the
                // walk stops at the end of the Visual selection, so it is a
                // line of the buffer; `ml_get` hands back its NUL-terminated
                // text and `ml_get_len` its length.
                let (oldp, oldlen) = unsafe { (ml_get(lnum), ml_get_len(lnum)) };
                if lnum > start_lnum {
                    let mut pos = pos_T {
                        lnum,
                        col: 0,
                        coladd: 0,
                    };
                    // SAFETY: a live window and a writable local position.
                    let found = unsafe { getvpos(curwin.get(), &raw mut pos, vcol) };
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
                    memmove(newp as *mut c_void, oldp as *const c_void, col as size_t);
                    let mut ptr = newp.offset(col as isize);
                    let put = (*self.y_array).data() as *const c_void;
                    for _ in 0..self.count {
                        memmove(ptr as *mut c_void, put, yanklen as size_t);
                        ptr = ptr.offset(yanklen as isize);
                    }
                    // +1 for the NUL.
                    let tail = oldp.offset(col as isize) as *const c_void;
                    memmove(ptr as *mut c_void, tail, (oldlen - col) as size_t + 1);
                    ptr
                };
                // SAFETY: `newp` is a NUL-terminated line the buffer takes
                // ownership of.
                unsafe { ml_replace(lnum, newp, false) };

                // Where the last character of the put text starts.
                //
                // SAFETY: `ptr` is one past the put text, so `ptr - 1` is its
                // last byte, and `newp` is the line it belongs to.
                first_byte_off = unsafe { utf_head_off(newp, ptr.offset(-1)) };

                if lnum == cur_win().w_cursor.lnum {
                    // Land the cursor on the last character put, keeping
                    // w_virtcol right.
                    //
                    // SAFETY (both): a live window, whose cursor line is the
                    // one that just changed.
                    unsafe { changed_cline_bef_curs(curwin.get()) };
                    unsafe { invalidate_botline_win(curwin.get()) };
                    cur_win().w_cursor.col += (totlen - 1) as colnr_T;
                }
                // SAFETY: `lnum`/`col` is where the line changed.
                unsafe { changed_bytes(lnum, col) };
                let inserted = totlen as c_int;
                // SAFETY: a live buffer; nothing was removed, so the splice
                // is `inserted` bytes going in at `col`.
                unsafe {
                    extmark_splice_cols(curbuf.get(), lnum - 1, col, 0, inserted, kExtmarkUndo)
                };
                if visual_active() {
                    lnum += 1;
                }
                if !(visual_active() && lnum <= end_lnum) {
                    break;
                }
            }
            if visual_active() {
                lnum -= 1; // back to the last Visual line
            }
        }

        // `']` goes on the *first byte* of the last character put.
        cur_buf().b_op_end = cur_win().w_cursor;
        cur_buf().b_op_end.col -= first_byte_off;

        // `CTRL-O p` in Insert mode leaves the cursor after the last
        // character rather than on it.
        if totlen != 0 && (restart_edit.get() != 0 || self.flags & PUT_CURSEND as c_int != 0) {
            cur_win().w_cursor.col += 1;
        } else {
            cur_win().w_cursor.col -= first_byte_off;
        }
    }

    /// Break the cursor line in two and hang the register's first and last
    /// lines off the halves.
    ///
    /// Answers the line the second half ended up on, and how many bytes of
    /// the register's first line went onto the first half.
    ///
    /// # Safety
    /// `lnum`/`col` must be a valid position.
    unsafe fn split_line_for_charwise(&self, lnum: linenr_T, col: colnr_T) {
        // The tail of the cursor line, with the register's *last* line in
        // front of it, becomes a new line below.
        //
        // SAFETY: the caller promises `lnum`/`col` is a valid position, so
        // `ml_get` hands back a NUL-terminated line with at least `col` bytes
        // in it; `y_array` holds `y_size` strings and `y_size` is at least
        // one, so the last is there; and `newp` is allocated for the two
        // strings and the NUL that `strcpy` writes.
        unsafe {
            let ptr = ml_get(lnum).offset(col as isize);
            let ptrlen = (ml_get_len(lnum) - col) as size_t;
            let last = *self.y_array.add(self.y_size.wrapping_sub(1));
            let newp = xmalloc(ptrlen.wrapping_add(last.len()).wrapping_add(1)) as *mut c_char;
            strcpy(newp, last.data());
            strcpy(newp.add(last.len()), ptr);
            ml_append(lnum, newp, 0, false);
            xfree(newp as *mut c_void);
        }

        // The head of the cursor line keeps the register's *first* line.
        //
        // SAFETY: the same position, re-read because `ml_append` moved the
        // line; `newp` is `col` bytes of it followed by the register's first
        // line and that line's NUL, which is the `col + yanklen + 1` asked
        // for, and `ml_replace` takes ownership of it.
        unsafe {
            let yanklen = (*self.y_array).len() as c_int;
            let oldp = ml_get(lnum);
            let newp = xmalloc((col + yanklen + 1) as size_t) as *mut c_char;
            memmove(newp as *mut c_void, oldp as *const c_void, col as size_t);
            // +1 to bring the NUL across.
            let put = (*self.y_array).data() as *const c_void;
            let at = newp.offset(col as isize) as *mut c_void;
            memmove(at, put, (yanklen + 1) as size_t);
            ml_replace(lnum, newp, false);
        }
    }

    /// Reindent line `lnum` the way `]p` wants: keep the *relative* indent of
    /// the register's lines, but move the block as a whole to `orig_indent`.
    ///
    /// # Safety
    /// `lnum` must be a valid line.
    unsafe fn fix_indent(&self, lnum: linenr_T, state: &mut FixIndent) {
        let old_pos = cur_win().w_cursor;
        cur_win().w_cursor.lnum = lnum;
        // SAFETY: the caller promises `lnum` is a line of the buffer, so
        // `ml_get` hands back its NUL-terminated text.
        let first = unsafe { c_int::from(*ml_get(lnum)) };
        // A `#` line stays at the start of the line, and an empty line
        // has no indent to keep.
        //
        // SAFETY (the three below): the cursor is on `lnum`, a line of the
        // current buffer, which is the line they measure and reindent.
        let indent = if (first == '#' as c_int && unsafe { preprocs_left() }) || first == NUL {
            0
        } else if state.first {
            state.diff = state.orig_indent - unsafe { get_indent() };
            state.first = false;
            state.orig_indent
        } else {
            (unsafe { get_indent() } + state.diff).max(0)
        };
        unsafe { set_indent(indent, SIN_NOMARK) };
        cur_win().w_cursor = old_pos;
    }

    /// The `'[` and `']` marks, and where the cursor ends up.
    ///
    /// # Safety
    /// `lnum` must be the last line the put touched and `new_lnum` the line
    /// the `']` mark belongs on.
    unsafe fn multiline_marks(
        &self,
        lnum: linenr_T,
        new_lnum: linenr_T,
        new_cursor: pos_T,
        col: colnr_T,
        lendiff: c_int,
    ) {
        if self.y_type == kMTLineWise {
            cur_buf().b_op_start.col = 0;
            if self.dir == FORWARD {
                cur_buf().b_op_start.lnum += 1;
            }
        }

        // Only a plain linewise put moves the marks itself; a split put
        // has already spliced them.
        let kind = if self.y_type == kMTLineWise && self.flags & PUT_LINE_SPLIT as c_int == 0 {
            kExtmarkUndo
        } else {
            kExtmarkNOOP
        };
        let from = cur_buf().b_op_start.lnum + linenr_T::from(self.y_type == kMTCharWise);
        // SAFETY: main thread, with a current buffer; the range runs from the
        // put's first line to the end of the buffer.
        unsafe { mark_adjust(from, MAXLNUM as linenr_T, self.nr_lines, 0, kind) };

        // SAFETY (both): a live buffer, and the range is the lines the put
        // just rewrote.
        if self.y_type == kMTCharWise {
            let at = cur_win().w_cursor.lnum;
            unsafe { changed_lines(curbuf.get(), at, col, at + 1, self.nr_lines, true) };
        } else {
            let at = cur_buf().b_op_start.lnum;
            unsafe { changed_lines(curbuf.get(), at, 0, at, self.nr_lines, true) };
        }

        // `']` goes on the first byte of the last character put, its
        // column corrected for whatever the reindent above removed.
        cur_buf().b_op_end.lnum = new_lnum;
        // SAFETY: `y_array` holds `y_size` strings and `y_size` is at least
        // one, so the last is there.
        let last = unsafe { *self.y_array.add(self.y_size.wrapping_sub(1)) };
        let col = (last.len() as colnr_T - lendiff).max(0);
        if col > 1 {
            cur_buf().b_op_end.col = col - 1;
            if !last.is_empty() {
                // SAFETY: `last` is NUL-terminated and `len()` bytes long, so
                // its final byte is one of them.
                let head = unsafe { utf_head_off(last.data(), last.data().add(last.len() - 1)) };
                cur_buf().b_op_end.col -= head;
            }
        } else {
            cur_buf().b_op_end.col = 0;
        }

        if self.flags & PUT_CURSLINE as c_int != 0 {
            // `:put`: the cursor goes on the last inserted line.
            cur_win().w_cursor.lnum = lnum;
            // SAFETY: the cursor is on a line of the current buffer.
            unsafe { beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX) };
        } else if self.flags & PUT_CURSEND as c_int != 0 {
            // The cursor goes after the inserted text.
            if self.y_type == kMTLineWise {
                cur_win().w_cursor.lnum = if lnum >= cur_buf().b_ml.ml_line_count {
                    cur_buf().b_ml.ml_line_count
                } else {
                    lnum + 1
                };
                cur_win().w_cursor.col = 0;
            } else {
                cur_win().w_cursor.lnum = new_lnum;
                cur_win().w_cursor.col = col;
                cur_buf().b_op_end = cur_win().w_cursor;
                if col > 1 {
                    cur_buf().b_op_end.col = col - 1;
                }
            }
        } else if self.y_type == kMTLineWise {
            // The cursor goes on the first non-blank of the first line.
            cur_win().w_cursor.col = 0;
            if self.dir == FORWARD {
                cur_win().w_cursor.lnum += 1;
            }
            // SAFETY: the cursor is on a line of the current buffer.
            unsafe { beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX) };
        } else {
            // The cursor goes on the first character put.
            cur_win().w_cursor = new_cursor;
        }
    }

    /// The linewise put, and the charwise put of more than one line.
    ///
    /// # Safety
    /// `lnum`/`col` must be a valid position, and undo already saved.
    pub(crate) unsafe fn multiline(&mut self, mut lnum: linenr_T, col: colnr_T, new_cursor: pos_T) {
        let mut new_lnum = new_cursor.lnum;
        let mut lendiff = 0;
        let mut indent_state = FixIndent {
            orig_indent: if self.flags & PUT_FIXINDENT as c_int != 0 {
                // SAFETY: the cursor is on a line of the current buffer.
                unsafe { get_indent() }
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
                    // SAFETY: `lnum`/`col` is the caller's valid position.
                    unsafe { self.split_line_for_charwise(lnum, col) };
                    new_lnum += 1;
                    cur_win().w_cursor.lnum = lnum;
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
                        if unsafe { ml_append(lnum, text, 0, false) } == FAIL {
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
                        // SAFETY (all three): `lnum` is the line just added.
                        if measured {
                            lendiff = unsafe { ml_get_len(lnum) };
                        }
                        unsafe { self.fix_indent(lnum, &mut indent_state) };
                        if measured {
                            lendiff -= unsafe { ml_get_len(lnum) };
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
                    let mut totsize: bcount_t = 0;
                    for i in 0..self.y_size.wrapping_sub(1) {
                        // SAFETY: `i` is below `y_size`.
                        totsize += unsafe { (*self.y_array.add(i)).len() } as bcount_t + 1;
                    }
                    let last = self.y_size.wrapping_sub(1);
                    // SAFETY: `y_size` is at least one, so `last` names a line.
                    let lastsize = unsafe { (*self.y_array.add(last)).len() } as c_int;
                    totsize += lastsize as bcount_t;

                    let buf = curbuf.get();
                    let at = new_cursor.lnum - 1;
                    let (start, rows, cols, bytes) = if self.y_type == kMTCharWise {
                        (col, self.y_size as c_int - 1, lastsize, totsize)
                    } else {
                        // Account for the last pasted newline and the
                        // newline the split itself added.
                        (self.split_pos, self.y_size as c_int + 1, 0, totsize + 2)
                    };
                    // SAFETY: a live buffer; nothing was replaced, so the
                    // "old" extent of the splice is zero throughout.
                    unsafe {
                        extmark_splice(buf, at, start, 0, 0, 0, rows, cols, bytes, kExtmarkUndo)
                    };
                }

                if cnt == 1 {
                    new_lnum = lnum;
                }
            }
        }

        // SAFETY: `lnum` is the last line the put touched and `new_lnum` the
        // one the `']` mark belongs on.
        unsafe { self.multiline_marks(lnum, new_lnum, new_cursor, col, lendiff) };
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
