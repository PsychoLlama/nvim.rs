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

use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int, c_void};

use super::Put;
use crate::register::*;
use crate::types::{FAIL, NUL};

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
    let mut csarg = CharsizeArg::default();
    // SAFETY: a live window whose cursor is on `oldp`'s line, and `oldp` is
    // that line's NUL-terminated text.
    let cstype =
        unsafe { init_charsize_arg(&mut csarg, curwin.get(), cur_win().w_cursor.lnum, oldp) };

    // Walk to the block's screen column, or to the end of the line.
    //
    // SAFETY (all four): `ci` starts at `oldp` and `utfc_next` steps over one
    // whole character at a time, so it stays inside the NUL-terminated line;
    // the `!= NUL` test in front of the walk is what stops it there.
    let mut ci = unsafe { utf_ptr2str_char_info(oldp) };
    let (mut vcol, mut incr) = (0, 0);
    while vcol < col && unsafe { c_int::from(*ci.ptr) } != NUL {
        incr = unsafe { win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &mut csarg) }.width;
        vcol += incr;
        ci = unsafe { utfc_next(ci) };
    }
    let ptr = ci.ptr;

    // SAFETY: `ptr` is a position in `oldp`'s NUL-terminated line, so the
    // distance back to `oldp` is a column of it and the byte there readable.
    let textcol = unsafe { ptr.offset_from(oldp) } as colnr_T;
    let ends_here = vcol == col && unsafe { c_int::from(*ptr) } == NUL;
    let mut land = Landing {
        textcol,
        startspaces: 0,
        endspaces: 0,
        delcount: 0,
        shortline: vcol < col || ends_here,
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
        // SAFETY (both): `textcol` is a byte offset into `oldp`'s line, so
        // the head of the character there is one of its bytes too.
        land.textcol -= unsafe { utf_head_off(oldp, oldp.offset(land.textcol as isize)) };
        if unsafe { c_int::from(*oldp.offset(land.textcol as isize)) } != TAB {
            land.delcount = 0;
            land.endspaces = 0;
        }
    }
    land
}

/// How many spaces one line of the block needs on its right to reach the
/// block's full width.
///
/// # Safety
/// `line` must be NUL-terminated.
unsafe fn right_padding(line: *mut c_char, y_width: c_int) -> c_int {
    let mut csarg = CharsizeArg::default();
    // SAFETY: a live window, and `line` is NUL-terminated.
    let cstype = unsafe { init_charsize_arg(&mut csarg, curwin.get(), 0, line) };
    // SAFETY (all four): `ci` starts at `line` and `utfc_next` steps over one
    // whole character at a time, so it stays inside it; the `!= NUL` test in
    // front of the walk is what stops it at the end.
    let mut ci = unsafe { utf_ptr2str_char_info(line) };
    let mut spaces = y_width + 1;
    while unsafe { c_int::from(*ci.ptr) } != NUL {
        spaces -= unsafe { win_charsize(cstype, 0, ci.ptr, ci.chr.value, &mut csarg) }.width;
        ci = unsafe { utfc_next(ci) };
    }
    spaces.max(0)
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
        // SAFETY: the cursor is on a valid line.
        let c = gchar_cursor();
        let mut col: colnr_T = 0;
        let mut endcol2: colnr_T = 0;

        if self.dir == FORWARD && c != NUL {
            if self.ve_flags == kOptVeFlagAll as ::core::ffi::c_uint {
                (col, endcol2) = cur_win().vcol_span(cur_win().cursor());
            } else {
                col = cur_win().vcol_span(cur_win().cursor()).1;
            }
            // Move to the start of the next character.
            //
            // SAFETY: the cursor is on a valid line and `c` is not the NUL
            // that ends it, so there is a character there to step over.
            cur_win().w_cursor.col += unsafe { utfc_ptr2len(get_cursor_pos_ptr()) };
            col += 1;
        } else {
            (col, endcol2) = cur_win().vcol_span(cur_win().cursor());
        }

        col += cur_win().w_cursor.coladd;
        if self.ve_flags == kOptVeFlagAll as ::core::ffi::c_uint
            && (cur_win().w_cursor.coladd > 0 || endcol2 == cur_win().w_cursor.col)
        {
            if self.dir == FORWARD && c == NUL {
                col += 1;
            }
            if self.dir != FORWARD && c != NUL && cur_win().w_cursor.coladd > 0 {
                cur_win().w_cursor.col += 1;
            }
            if c == TAB {
                if self.dir == BACKWARD && cur_win().w_cursor.col != 0 {
                    cur_win().w_cursor.col -= 1;
                }
                if self.dir == FORWARD && col - 1 == endcol2 {
                    cur_win().w_cursor.col += 1;
                }
            }
        }
        cur_win().w_cursor.coladd = 0;
        col
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
        // Pasting past the end of the buffer appends empty lines.
        let mut lines_appended = 0;
        if cur_win().w_cursor.lnum > cur_buf().b_ml.ml_line_count {
            let last = cur_buf().b_ml.ml_line_count;
            let empty = c"".as_ptr().cast_mut();
            // SAFETY: `last` is the buffer's own last line, and the text is a
            // NUL-terminated literal that `ml_append` copies.
            if unsafe { ml_append(last, empty, 1, false) } == FAIL {
                return false;
            }
            self.nr_lines += 1;
            lines_appended = 1;
        }

        // SAFETY (both): the cursor is now on a line of the buffer, so this
        // is its NUL-terminated text and that text's length.
        let oldp = get_cursor_line_ptr();
        let oldlen = get_cursor_line_len();
        // SAFETY: `oldp` is the cursor line, NUL-terminated.
        let land = unsafe { land_block(oldp, col) };
        *textcol = land.textcol;

        // SAFETY (both): `i` is below `y_size`, so `y_array`'s `i`th string is
        // there, NUL-terminated and carrying its own length.
        let yanklen = unsafe { (*self.y_array.add(i)).len() } as c_int;
        let put = unsafe { (*self.y_array.add(i)).data() };
        let spaces = if self.flags & PUT_BLOCK_INNER as c_int == 0 {
            // SAFETY: `put` is NUL-terminated.
            unsafe { right_padding(put, self.y_width) }
        } else {
            0
        };

        // The whole insert is `count` copies of the line plus its padding.
        if yanklen + spaces != 0
            && self.count > (c_int::MAX - (land.startspaces + land.endspaces)) / (yanklen + spaces)
        {
            // SAFETY: a NUL-terminated message literal.
            unsafe {
                emsg(gettext(
                    &raw const e_resulting_text_too_long as *const c_char,
                ))
            };
            return false;
        }
        *totlen = (self.count as size_t)
            .wrapping_mul((yanklen + spaces) as size_t)
            .wrapping_add(land.startspaces as size_t)
            .wrapping_add(land.endspaces as size_t);
        let room = (*totlen).wrapping_add(oldlen as size_t).wrapping_add(1);
        // SAFETY: `room` is the whole new line and its NUL.
        let newp = unsafe { xmalloc(room) } as *mut c_char;

        // Lay the new line out: the head of the old line, the block's left
        // padding, `count` copies of the block's line each with its own right
        // padding, the spaces replacing the back half of a split tab, and
        // finally the rest of the old line.
        //
        // SAFETY: `newp` owns exactly the `totlen + oldlen + 1` bytes the runs
        // below write; `oldp` is `oldlen` bytes plus a NUL and `textcol +
        // delcount` is within it; and `put` is `yanklen` bytes.
        let mut ptr = newp;
        unsafe {
            memmove(
                ptr as *mut c_void,
                oldp as *const c_void,
                land.textcol as size_t,
            );
            ptr = ptr.offset(land.textcol as isize);
            memset(ptr as *mut c_void, ' ' as c_int, land.startspaces as size_t);
            ptr = ptr.offset(land.startspaces as isize);

            for j in 0..self.count {
                memmove(ptr as *mut c_void, put as *const c_void, yanklen as size_t);
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
            let rest = oldp.offset((land.textcol + land.delcount) as isize) as *const c_void;
            memmove(ptr as *mut c_void, rest, columns as size_t);
        }
        // SAFETY: `newp` is a NUL-terminated line the buffer takes over.
        unsafe { ml_replace(cur_win().w_cursor.lnum, newp, false) };

        let buf = curbuf.get();
        let at = cur_win().w_cursor.lnum - 1;
        let inserted = *totlen as c_int + lines_appended;
        // SAFETY: a live buffer; `delcount` bytes came out at `textcol` and
        // `inserted` went in there.
        unsafe {
            extmark_splice_cols(buf, at, land.textcol, land.delcount, inserted, kExtmarkUndo)
        };

        cur_win().w_cursor.lnum += 1;
        if i == 0 {
            cur_win().w_cursor.col += land.startspaces;
        }
        true
    }

    /// The whole blockwise put, starting at the cursor.
    ///
    /// `lnum` is the line the put started on, kept for the `'[` mark and the
    /// redraw range.
    ///
    /// # Safety
    /// The cursor must be on a valid line, and undo already saved.
    pub(crate) unsafe fn blockwise(&mut self, lnum: linenr_T) {
        // SAFETY: the cursor is on a valid line.
        let col = unsafe { self.block_start_col() };

        let mut textcol: colnr_T = 0;
        let mut totlen: size_t = 0;
        for i in 0..self.y_size {
            // SAFETY: the cursor is on a line of the buffer, or one past its
            // end, which is where the previous round left it.
            if !unsafe { self.blockwise_line(i, col, &mut textcol, &mut totlen) } {
                break;
            }
        }

        let to = cur_buf().b_op_start.lnum + self.y_size as linenr_T - self.nr_lines;
        // SAFETY: a live buffer; the range is the lines the put rewrote.
        unsafe { changed_lines(curbuf.get(), lnum, 0, to, self.nr_lines, true) };

        cur_buf().b_op_start = cur_win().w_cursor;
        cur_buf().b_op_start.lnum = lnum;

        cur_buf().b_op_end.lnum = cur_win().w_cursor.lnum - 1;
        cur_buf().b_op_end.col = (textcol + totlen as colnr_T - 1).max(0);
        cur_buf().b_op_end.coladd = 0;

        if self.flags & PUT_CURSEND as c_int != 0 {
            cur_win().w_cursor = cur_buf().b_op_end;
            cur_win().w_cursor.col += 1;
            // In Insert mode the cursor may be past the NUL.
            //
            // SAFETY: the cursor is on a line of the buffer.
            let len = get_cursor_line_len();
            cur_win().w_cursor.col = cur_win().w_cursor.col.min(len);
        } else {
            cur_win().w_cursor.lnum = lnum;
        }
    }
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
