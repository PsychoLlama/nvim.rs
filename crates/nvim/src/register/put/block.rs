//! The blockwise put: CTRL-V text going in as a rectangle.
//!
//! One line of the register per buffer line, each inserted at the same
//! *screen column* rather than the same byte -- which is what makes this
//! harder than the other two. Reaching that column may mean padding a short
//! line with spaces, or splitting a tab the block lands inside; a line of the
//! block shorter than the widest one is padded on the right so that whatever
//! follows still lines up; and the buffer may run out of lines, in which case
//! empty ones are appended.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};

use super::Put;
use crate::register::*;
use crate::types::FAIL;

/// Where in one buffer line the block goes, measured by [`Put::blockwise`].
struct Landing {
    /// Byte offset in the line where the insert starts.
    textcol: colnr_T,
    /// Spaces to add in front of the block, padding a short line out to the
    /// block's column, or replacing the front half of a split tab.
    startspaces: c_int,
    /// Spaces replacing the back half of a split tab.
    endspaces: c_int,
    /// 1 when a tab is being split and so replaced, 0 otherwise.
    delcount: c_int,
    /// Whether the line ends at or before the block's column, so that the
    /// block's own right padding would be trailing white space.
    shortline: bool,
}

/// Walk `oldp` to screen column `col` and work out what has to happen there.
///
/// # Safety
/// `oldp` must be the cursor line, NUL-terminated.
unsafe fn land_block(oldp: *mut c_char, col: colnr_T) -> Landing {
    unsafe {
        let mut csarg = CharsizeArg::default();
        let cstype = init_charsize_arg(
            &mut csarg,
            curwin.get(),
            (*curwin.get()).w_cursor.lnum,
            oldp,
        );
        let mut ci = utf_ptr2StrCharInfo(oldp);
        let mut vcol: colnr_T = 0;
        let mut incr = 0;
        while vcol < col && c_int::from(*ci.ptr) != NUL {
            incr = win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &mut csarg).width;
            vcol += incr;
            ci = utfc_next(ci);
        }
        let ptr = ci.ptr;

        let mut land = Landing {
            textcol: ptr.offset_from(oldp) as colnr_T,
            startspaces: 0,
            endspaces: 0,
            delcount: 0,
            shortline: vcol < col || (vcol == col && c_int::from(*ptr) == NUL),
        };

        if vcol < col {
            // The line stops short of the block: pad it out.
            land.startspaces = col - vcol;
        } else if vcol > col {
            // The block lands inside a character. Only a tab can be split
            // into spaces; anything else has to be pushed past the block,
            // which misaligns it, so it is left alone.
            land.endspaces = vcol - col;
            land.startspaces = incr - land.endspaces;
            land.textcol -= 1;
            land.delcount = 1;
            land.textcol -= utf_head_off(oldp, oldp.offset(land.textcol as isize));
            if c_int::from(*oldp.offset(land.textcol as isize)) != TAB {
                land.delcount = 0;
                land.endspaces = 0;
            }
        }
        land
    }
}

/// How many spaces one line of the block needs on its right to reach the
/// block's full width.
///
/// # Safety
/// `line` must be NUL-terminated.
unsafe fn right_padding(line: *mut c_char, y_width: c_int) -> c_int {
    unsafe {
        let mut csarg = CharsizeArg::default();
        let cstype = init_charsize_arg(&mut csarg, curwin.get(), 0, line);
        let mut ci = utf_ptr2StrCharInfo(line);
        let mut spaces = y_width + 1;
        while c_int::from(*ci.ptr) != NUL {
            spaces -= win_charsize(cstype, 0, ci.ptr, ci.chr.value, &mut csarg).width;
            ci = utfc_next(ci);
        }
        spaces.max(0)
    }
}

impl Put {
    /// Move the cursor to the byte the block should start at, in the column
    /// sense, honouring 'virtualedit'.
    ///
    /// Answers the screen column the block lands at.
    ///
    /// # Safety
    /// The cursor must be on a valid line.
    unsafe fn block_start_col(&self) -> colnr_T {
        unsafe {
            let c = gchar_cursor();
            let mut col: colnr_T = 0;
            let mut endcol2: colnr_T = 0;

            if self.dir == FORWARD && c != NUL {
                if self.ve_flags == kOptVeFlagAll as ::core::ffi::c_uint {
                    getvcol(
                        curwin.get(),
                        &raw mut (*curwin.get()).w_cursor,
                        &raw mut col,
                        ::core::ptr::null_mut(),
                        &raw mut endcol2,
                    );
                } else {
                    getvcol(
                        curwin.get(),
                        &raw mut (*curwin.get()).w_cursor,
                        ::core::ptr::null_mut(),
                        ::core::ptr::null_mut(),
                        &raw mut col,
                    );
                }
                // Move to the start of the next character.
                (*curwin.get()).w_cursor.col += utfc_ptr2len(get_cursor_pos_ptr());
                col += 1;
            } else {
                getvcol(
                    curwin.get(),
                    &raw mut (*curwin.get()).w_cursor,
                    &raw mut col,
                    ::core::ptr::null_mut(),
                    &raw mut endcol2,
                );
            }

            col += (*curwin.get()).w_cursor.coladd;
            if self.ve_flags == kOptVeFlagAll as ::core::ffi::c_uint
                && ((*curwin.get()).w_cursor.coladd > 0 || endcol2 == (*curwin.get()).w_cursor.col)
            {
                if self.dir == FORWARD && c == NUL {
                    col += 1;
                }
                if self.dir != FORWARD && c != NUL && (*curwin.get()).w_cursor.coladd > 0 {
                    (*curwin.get()).w_cursor.col += 1;
                }
                if c == TAB {
                    if self.dir == BACKWARD && (*curwin.get()).w_cursor.col != 0 {
                        (*curwin.get()).w_cursor.col -= 1;
                    }
                    if self.dir == FORWARD && col - 1 == endcol2 {
                        (*curwin.get()).w_cursor.col += 1;
                    }
                }
            }
            (*curwin.get()).w_cursor.coladd = 0;
            col
        }
    }

    /// Insert line `i` of the block into the cursor line at screen column
    /// `col`, and step the cursor to the next line.
    ///
    /// `textcol` and `totlen` are where the line grew and by how much; the
    /// caller keeps the last pair for the `']` mark, so they are written as
    /// soon as they are known rather than returned -- an early stop leaves
    /// `textcol` updated and `totlen` not, which is what upstream's shared
    /// `block_def` does.
    ///
    /// Answers false when the put must stop.
    ///
    /// # Safety
    /// The cursor must be on a valid line or one past the end of the buffer.
    unsafe fn blockwise_line(
        &mut self,
        i: size_t,
        col: colnr_T,
        textcol: &mut colnr_T,
        totlen: &mut size_t,
    ) -> bool {
        unsafe {
            // Pasting past the end of the buffer appends empty lines.
            let mut lines_appended = 0;
            if (*curwin.get()).w_cursor.lnum > (*curbuf.get()).b_ml.ml_line_count {
                if ml_append(
                    (*curbuf.get()).b_ml.ml_line_count,
                    c"".as_ptr().cast_mut(),
                    1,
                    false,
                ) == FAIL
                {
                    return false;
                }
                self.nr_lines += 1;
                lines_appended = 1;
            }

            let oldp = get_cursor_line_ptr();
            let oldlen = get_cursor_line_len();
            let land = land_block(oldp, col);
            *textcol = land.textcol;

            let yanklen = (*self.y_array.add(i)).size as c_int;
            let spaces = if self.flags & PUT_BLOCK_INNER as c_int == 0 {
                right_padding((*self.y_array.add(i)).data, self.y_width)
            } else {
                0
            };

            // The whole insert is `count` copies of the line plus its padding.
            if yanklen + spaces != 0
                && self.count
                    > (c_int::MAX - (land.startspaces + land.endspaces)) / (yanklen + spaces)
            {
                emsg(gettext(
                    &raw const e_resulting_text_too_long as *const c_char,
                ));
                return false;
            }
            *totlen = (self.count as size_t)
                .wrapping_mul((yanklen + spaces) as size_t)
                .wrapping_add(land.startspaces as size_t)
                .wrapping_add(land.endspaces as size_t);
            let newp =
                xmalloc((*totlen).wrapping_add(oldlen as size_t).wrapping_add(1)) as *mut c_char;

            let mut ptr = newp;
            memmove(
                ptr as *mut c_void,
                oldp as *const c_void,
                land.textcol as size_t,
            );
            ptr = ptr.offset(land.textcol as isize);
            memset(ptr as *mut c_void, ' ' as c_int, land.startspaces as size_t);
            ptr = ptr.offset(land.startspaces as isize);

            for j in 0..self.count {
                memmove(
                    ptr as *mut c_void,
                    (*self.y_array.add(i)).data as *const c_void,
                    yanklen as size_t,
                );
                ptr = ptr.offset(yanklen as isize);
                // The block's right padding only goes in if there is text
                // behind it; otherwise it would be trailing white space.
                if (j < self.count - 1 || !land.shortline) && spaces > 0 {
                    memset(ptr as *mut c_void, ' ' as c_int, spaces as size_t);
                    ptr = ptr.offset(spaces as isize);
                } else {
                    *totlen -= spaces as size_t;
                }
            }

            memset(ptr as *mut c_void, ' ' as c_int, land.endspaces as size_t);
            ptr = ptr.offset(land.endspaces as isize);

            // The rest of the old line, including its NUL.
            let columns = oldlen - land.textcol - land.delcount + 1;
            debug_assert!(columns >= 0);
            memmove(
                ptr as *mut c_void,
                oldp.offset((land.textcol + land.delcount) as isize) as *const c_void,
                columns as size_t,
            );
            ml_replace((*curwin.get()).w_cursor.lnum, newp, false);
            extmark_splice_cols(
                curbuf.get(),
                (*curwin.get()).w_cursor.lnum - 1,
                land.textcol,
                land.delcount,
                *totlen as c_int + lines_appended,
                kExtmarkUndo,
            );

            (*curwin.get()).w_cursor.lnum += 1;
            if i == 0 {
                (*curwin.get()).w_cursor.col += land.startspaces;
            }
            true
        }
    }

    /// The whole blockwise put, starting at the cursor.
    ///
    /// `lnum` is the line the put started on, kept for the `'[` mark and the
    /// redraw range.
    ///
    /// # Safety
    /// The cursor must be on a valid line, and undo already saved.
    pub(crate) unsafe fn blockwise(&mut self, lnum: linenr_T) {
        unsafe {
            let col = self.block_start_col();

            let mut textcol: colnr_T = 0;
            let mut totlen: size_t = 0;
            for i in 0..self.y_size {
                if !self.blockwise_line(i, col, &mut textcol, &mut totlen) {
                    break;
                }
            }

            changed_lines(
                curbuf.get(),
                lnum,
                0,
                (*curbuf.get()).b_op_start.lnum + self.y_size as linenr_T - self.nr_lines,
                self.nr_lines,
                true,
            );

            (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor;
            (*curbuf.get()).b_op_start.lnum = lnum;

            (*curbuf.get()).b_op_end.lnum = (*curwin.get()).w_cursor.lnum - 1;
            (*curbuf.get()).b_op_end.col = (textcol + totlen as colnr_T - 1).max(0);
            (*curbuf.get()).b_op_end.coladd = 0;

            if self.flags & PUT_CURSEND as c_int != 0 {
                (*curwin.get()).w_cursor = (*curbuf.get()).b_op_end;
                (*curwin.get()).w_cursor.col += 1;
                // In Insert mode the cursor may be past the NUL.
                (*curwin.get()).w_cursor.col =
                    (*curwin.get()).w_cursor.col.min(get_cursor_line_len());
            } else {
                (*curwin.get()).w_cursor.lnum = lnum;
            }
        }
    }
}
