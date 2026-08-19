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

use core::ffi::{c_char, c_int, c_void};

use super::Put;
use crate::edit::BeginlineOpts;
use crate::register::*;
use crate::types::{FAIL, NUL};

impl Put {
    /// Splice `count` copies of the register's single line into the buffer at
    /// `col`, once per line of a Visual selection.
    ///
    /// # Safety
    /// `lnum`/`col` must be a valid position, and undo already saved.
    pub(crate) unsafe fn charwise_one_line(&mut self, mut lnum: linenr_T, mut col: colnr_T) {
        unsafe {
            let yanklen = (*self.y_array).size as c_int;
            let start_lnum = lnum;
            let mut end_lnum = 0;
            let mut first_byte_off = 0;
            let mut vcol: colnr_T = 0;
            let mut totlen: size_t = 0;

            if VIsual_active.get() {
                end_lnum = (*curbuf.get())
                    .b_visual
                    .vi_end
                    .lnum
                    .max((*curbuf.get()).b_visual.vi_start.lnum);
                if end_lnum > start_lnum {
                    // `col` is only right for the first line; the others have
                    // to be found by *screen* column, which matters as soon as
                    // a multi-byte character is involved.
                    let mut pos = pos_T {
                        lnum,
                        col,
                        coladd: 0,
                    };
                    getvcol(
                        curwin.get(),
                        &raw mut pos,
                        ::core::ptr::null_mut(),
                        &raw mut vcol,
                        ::core::ptr::null_mut(),
                    );
                }
            }

            if self.count == 0 || yanklen == 0 {
                if VIsual_active.get() {
                    lnum = end_lnum;
                }
            } else if self.count > c_int::MAX / yanklen {
                emsg(gettext(
                    &raw const e_resulting_text_too_long as *const c_char,
                ));
            } else {
                totlen = (self.count as size_t).wrapping_mul(yanklen as size_t);
                loop {
                    let oldp = ml_get(lnum);
                    let oldlen = ml_get_len(lnum);
                    if lnum > start_lnum {
                        let mut pos = pos_T {
                            lnum,
                            col: 0,
                            coladd: 0,
                        };
                        col = if getvpos(curwin.get(), &raw mut pos, vcol) {
                            pos.col
                        } else {
                            MAXCOL
                        };
                    }
                    // A Visual line too short to reach the column is skipped
                    // -- upstream's `continue`, which in its do-while jumps
                    // straight to the condition at the bottom.
                    if VIsual_active.get() && col > oldlen {
                        lnum += 1;
                        if !(VIsual_active.get() && lnum <= end_lnum) {
                            break;
                        }
                        continue;
                    }

                    let newp = xmalloc(totlen.wrapping_add(oldlen as size_t).wrapping_add(1))
                        as *mut c_char;
                    memmove(newp as *mut c_void, oldp as *const c_void, col as size_t);
                    let mut ptr = newp.offset(col as isize);
                    for _ in 0..self.count {
                        memmove(
                            ptr as *mut c_void,
                            (*self.y_array).data as *const c_void,
                            yanklen as size_t,
                        );
                        ptr = ptr.offset(yanklen as isize);
                    }
                    // +1 for the NUL.
                    memmove(
                        ptr as *mut c_void,
                        oldp.offset(col as isize) as *const c_void,
                        (oldlen - col) as size_t + 1,
                    );
                    ml_replace(lnum, newp, false);

                    // Where the last character of the put text starts.
                    first_byte_off = utf_head_off(newp, ptr.offset(-1));

                    if lnum == (*curwin.get()).w_cursor.lnum {
                        // Land the cursor on the last character put, keeping
                        // w_virtcol right.
                        changed_cline_bef_curs(curwin.get());
                        invalidate_botline_win(curwin.get());
                        (*curwin.get()).w_cursor.col += (totlen - 1) as colnr_T;
                    }
                    changed_bytes(lnum, col);
                    extmark_splice_cols(
                        curbuf.get(),
                        lnum - 1,
                        col,
                        0,
                        totlen as c_int,
                        kExtmarkUndo,
                    );
                    if VIsual_active.get() {
                        lnum += 1;
                    }
                    if !(VIsual_active.get() && lnum <= end_lnum) {
                        break;
                    }
                }
                if VIsual_active.get() {
                    lnum -= 1; // back to the last Visual line
                }
            }

            // `']` goes on the *first byte* of the last character put.
            (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
            (*curbuf.get()).b_op_end.col -= first_byte_off;

            // `CTRL-O p` in Insert mode leaves the cursor after the last
            // character rather than on it.
            if totlen != 0 && (restart_edit.get() != 0 || self.flags & PUT_CURSEND as c_int != 0) {
                (*curwin.get()).w_cursor.col += 1;
            } else {
                (*curwin.get()).w_cursor.col -= first_byte_off;
            }
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
        unsafe {
            // The tail of the cursor line, with the register's *last* line in
            // front of it, becomes a new line below.
            let ptr = ml_get(lnum).offset(col as isize);
            let ptrlen = (ml_get_len(lnum) - col) as size_t;
            let last = *self.y_array.add(self.y_size.wrapping_sub(1));
            let newp = xmalloc(ptrlen.wrapping_add(last.size).wrapping_add(1)) as *mut c_char;
            strcpy(newp, last.data);
            strcpy(newp.add(last.size), ptr);
            ml_append(lnum, newp, 0, false);
            xfree(newp as *mut c_void);

            // The head of the cursor line keeps the register's *first* line.
            let yanklen = (*self.y_array).size as c_int;
            let oldp = ml_get(lnum);
            let newp = xmalloc((col + yanklen + 1) as size_t) as *mut c_char;
            memmove(newp as *mut c_void, oldp as *const c_void, col as size_t);
            // +1 to bring the NUL across.
            memmove(
                newp.offset(col as isize) as *mut c_void,
                (*self.y_array).data as *const c_void,
                (yanklen + 1) as size_t,
            );
            ml_replace(lnum, newp, false);
        }
    }

    /// Reindent line `lnum` the way `]p` wants: keep the *relative* indent of
    /// the register's lines, but move the block as a whole to `orig_indent`.
    ///
    /// # Safety
    /// `lnum` must be a valid line.
    unsafe fn fix_indent(&self, lnum: linenr_T, state: &mut FixIndent) {
        unsafe {
            let old_pos = (*curwin.get()).w_cursor;
            (*curwin.get()).w_cursor.lnum = lnum;
            let ptr = ml_get(lnum);
            let indent = if c_int::from(*ptr) == '#' as c_int && preprocs_left() {
                0 // leave `#` lines at the start of the line
            } else if c_int::from(*ptr) == NUL {
                0 // ignore empty lines
            } else if state.first {
                state.diff = state.orig_indent - get_indent();
                state.first = false;
                state.orig_indent
            } else {
                (get_indent() + state.diff).max(0)
            };
            set_indent(indent, SIN_NOMARK);
            (*curwin.get()).w_cursor = old_pos;
        }
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
        unsafe {
            if self.y_type == kMTLineWise {
                (*curbuf.get()).b_op_start.col = 0;
                if self.dir == FORWARD {
                    (*curbuf.get()).b_op_start.lnum += 1;
                }
            }

            // Only a plain linewise put moves the marks itself; a split put
            // has already spliced them.
            let kind = if self.y_type == kMTLineWise && self.flags & PUT_LINE_SPLIT as c_int == 0 {
                kExtmarkUndo
            } else {
                kExtmarkNOOP
            };
            mark_adjust(
                (*curbuf.get()).b_op_start.lnum + linenr_T::from(self.y_type == kMTCharWise),
                MAXLNUM as linenr_T,
                self.nr_lines,
                0,
                kind,
            );

            if self.y_type == kMTCharWise {
                changed_lines(
                    curbuf.get(),
                    (*curwin.get()).w_cursor.lnum,
                    col,
                    (*curwin.get()).w_cursor.lnum + 1,
                    self.nr_lines,
                    true,
                );
            } else {
                changed_lines(
                    curbuf.get(),
                    (*curbuf.get()).b_op_start.lnum,
                    0,
                    (*curbuf.get()).b_op_start.lnum,
                    self.nr_lines,
                    true,
                );
            }

            // `']` goes on the first byte of the last character put, its
            // column corrected for whatever the reindent above removed.
            (*curbuf.get()).b_op_end.lnum = new_lnum;
            let last = *self.y_array.add(self.y_size.wrapping_sub(1));
            let col = (last.size as colnr_T - lendiff).max(0);
            if col > 1 {
                (*curbuf.get()).b_op_end.col = col - 1;
                if last.size > 0 {
                    (*curbuf.get()).b_op_end.col -=
                        utf_head_off(last.data, last.data.add(last.size - 1));
                }
            } else {
                (*curbuf.get()).b_op_end.col = 0;
            }

            if self.flags & PUT_CURSLINE as c_int != 0 {
                // `:put`: the cursor goes on the last inserted line.
                (*curwin.get()).w_cursor.lnum = lnum;
                beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
            } else if self.flags & PUT_CURSEND as c_int != 0 {
                // The cursor goes after the inserted text.
                if self.y_type == kMTLineWise {
                    (*curwin.get()).w_cursor.lnum = if lnum >= (*curbuf.get()).b_ml.ml_line_count {
                        (*curbuf.get()).b_ml.ml_line_count
                    } else {
                        lnum + 1
                    };
                    (*curwin.get()).w_cursor.col = 0;
                } else {
                    (*curwin.get()).w_cursor.lnum = new_lnum;
                    (*curwin.get()).w_cursor.col = col;
                    (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
                    if col > 1 {
                        (*curbuf.get()).b_op_end.col = col - 1;
                    }
                }
            } else if self.y_type == kMTLineWise {
                // The cursor goes on the first non-blank of the first line.
                (*curwin.get()).w_cursor.col = 0;
                if self.dir == FORWARD {
                    (*curwin.get()).w_cursor.lnum += 1;
                }
                beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
            } else {
                // The cursor goes on the first character put.
                (*curwin.get()).w_cursor = new_cursor;
            }
        }
    }

    /// The linewise put, and the charwise put of more than one line.
    ///
    /// # Safety
    /// `lnum`/`col` must be a valid position, and undo already saved.
    pub(crate) unsafe fn multiline(&mut self, mut lnum: linenr_T, col: colnr_T, new_cursor: pos_T) {
        unsafe {
            let mut new_lnum = new_cursor.lnum;
            let mut lendiff = 0;
            let mut indent_state = FixIndent {
                orig_indent: if self.flags & PUT_FIXINDENT as c_int != 0 {
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
                        (*curwin.get()).w_cursor.lnum = lnum;
                        i = 1;
                    }

                    while i < self.y_size {
                        // A charwise register's last line is already on the
                        // second half of the split.
                        if self.y_type != kMTCharWise || i < self.y_size.wrapping_sub(1) {
                            if ml_append(lnum, (*self.y_array.add(i)).data, 0, false) == FAIL {
                                break 'error;
                            }
                            new_lnum += 1;
                        }
                        lnum += 1;
                        self.nr_lines += 1;
                        if self.flags & PUT_FIXINDENT as c_int != 0 {
                            if cnt == self.count && i == self.y_size.wrapping_sub(1) {
                                lendiff = ml_get_len(lnum);
                            }
                            self.fix_indent(lnum, &mut indent_state);
                            // How many bytes the reindent removed.
                            if cnt == self.count && i == self.y_size.wrapping_sub(1) {
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
                        let mut totsize: bcount_t = 0;
                        for i in 0..self.y_size.wrapping_sub(1) {
                            totsize += (*self.y_array.add(i)).size as bcount_t + 1;
                        }
                        let lastsize =
                            (*self.y_array.add(self.y_size.wrapping_sub(1))).size as c_int;
                        totsize += lastsize as bcount_t;
                        if self.y_type == kMTCharWise {
                            extmark_splice(
                                curbuf.get(),
                                new_cursor.lnum - 1,
                                col,
                                0,
                                0,
                                0,
                                self.y_size as c_int - 1,
                                lastsize,
                                totsize,
                                kExtmarkUndo,
                            );
                        } else {
                            // Account for the last pasted newline and the
                            // newline the split itself added.
                            extmark_splice(
                                curbuf.get(),
                                new_cursor.lnum - 1,
                                self.split_pos,
                                0,
                                0,
                                0,
                                self.y_size as c_int + 1,
                                0,
                                totsize + 2,
                                kExtmarkUndo,
                            );
                        }
                    }

                    if cnt == 1 {
                        new_lnum = lnum;
                    }
                }
            }

            self.multiline_marks(lnum, new_lnum, new_cursor, col, lendiff);
        }
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
