//! The blockwise put: CTRL-V text going in as a rectangle.
//!
//! One line of the register per buffer line, each inserted at the same
//! *screen column* rather than the same byte -- which is what makes this
//! harder than the other two. Reaching that column may mean padding a short
//! line with spaces, or splitting a tab the block lands inside; a line of the
//! block shorter than the widest one is padded on the right so that whatever
//! follows still lines up; and the buffer may run out of lines, in which case
//! empty ones are appended.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use crate::extmark::EXTMARK_UNDO;
use crate::mbyte::Codepoint;
use crate::mbyte::{c_char_info, c_next_cluster, head_off};
use crate::narrow::len_as_int;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::Put;
use crate::register::*;

/// Where in one buffer line the block goes, measured by [`Put::blockwise`].
#[derive(Debug)]
struct Landing {
    /// Byte offset in the line where the insert starts.
    textcol: ColNr,
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

/// Walk `line`, the cursor line, to screen column `col` and work out what has
/// to happen there.
fn land_block(line: &[u8], col: ColNr) -> Landing {
    let (win, lnum) = (Win::current(), Win::current().w_cursor.lnum);
    let (mut csarg, cstype) = CharsizeArg::new(win, lnum, line);

    // Walk to the block's screen column, or to the end of the line.
    let mut at = 0;
    let mut chr = c_char_info(line, 0);
    let (mut vcol, mut incr) = (0, 0);
    while vcol < col.get() && !(chr.value == 0 && cstr::byte_at(line, at) == 0) {
        incr = win_charsize(cstype, vcol, at, chr.value, &mut csarg).width;
        vcol += incr;
        (at, chr) = c_next_cluster(line, at, chr);
    }

    let textcol = ColNr::try_from(at).expect("a column of the line");
    let ends_here = vcol == col.get() && cstr::byte_at(line, at) == 0;
    let mut land = Landing {
        textcol,
        startspaces: 0,
        endspaces: 0,
        delcount: 0,
        shortline: vcol < col.get() || ends_here,
    };

    if vcol < col.get() {
        // The line stops short of the block: pad it out.
        land.startspaces = col.get() - vcol;
    } else if vcol > col.get() {
        // The block lands inside a character. Only a tab can be split
        // into spaces; anything else has to be pushed past the block,
        // which misaligns it, so it is left alone.
        land.endspaces = vcol - col.get();
        land.startspaces = incr - land.endspaces;
        land.textcol -= 1;
        land.delcount = 1;
        let back = head_off(line, land.textcol.as_usize());
        land.textcol -= ColNr::try_from(back).expect("a column of the line").get();
        if c_int::from(cstr::byte_at(line, land.textcol.as_usize())) != c_int::from(TAB) {
            land.delcount = 0;
            land.endspaces = 0;
        }
    }
    land
}

/// How many spaces `line`, one line of the block, needs on its right to
/// reach the block's full width.
fn right_padding(line: &[u8], y_width: c_int) -> c_int {
    let (mut csarg, cstype) = CharsizeArg::new(Win::current(), LineNr::ZERO, line);
    let mut at = 0;
    let mut chr = c_char_info(line, 0);
    let mut spaces = y_width + 1;
    while !(chr.value == 0 && cstr::byte_at(line, at) == 0) {
        spaces -= win_charsize(cstype, 0, at, chr.value, &mut csarg).width;
        (at, chr) = c_next_cluster(line, at, chr);
    }
    spaces.max(0)
}

/// Why a blockwise put stopped part-way.
#[derive(Clone, Copy, Debug)]
enum Stopped {
    /// The empty line a put past the end needs could not be appended (see
    /// [`LineError`](crate::memline::LineError)).
    NoLine,
    /// The line would grow past `INT_MAX`: E1240, said.
    TooLong,
}

impl Put {
    /// Move the cursor to the byte the block should start at, in the column
    /// sense, honouring 'virtualedit'.
    ///
    /// Answers the screen column the block lands at.
    fn block_start_col(&self) -> ColNr {
        let c = gchar_cursor();
        let mut col: ColNr;
        let mut endcol2: ColNr = ColNr::ZERO;

        if self.dir == FORWARD && c != Codepoint::NUL {
            if self.ve_flags == OPT_VE_FLAG_ALL as ::core::ffi::c_uint {
                (col, endcol2) = Win::current().vcol_span(Win::current().cursor());
            } else {
                col = Win::current().vcol_span(Win::current().cursor()).1;
            }
            // Move to the start of the next character: `c` is not the NUL
            // that ends the line, so there is one to step over.
            Win::current().w_cursor.col += super::cursor_char_len().get();
            col += 1;
        } else {
            (col, endcol2) = Win::current().vcol_span(Win::current().cursor());
        }

        col += Win::current().w_cursor.coladd;
        if self.ve_flags == OPT_VE_FLAG_ALL as ::core::ffi::c_uint
            && (Win::current().w_cursor.coladd > 0 || endcol2 == Win::current().w_cursor.col)
        {
            if self.dir == FORWARD && c == Codepoint::NUL {
                col += 1;
            }
            if self.dir != FORWARD && c != Codepoint::NUL && Win::current().w_cursor.coladd > 0 {
                Win::current().w_cursor.col += 1;
            }
            if c == TAB {
                if self.dir == BACKWARD && Win::current().w_cursor.col != ColNr::ZERO {
                    Win::current().w_cursor.col -= 1;
                }
                if self.dir == FORWARD && col - 1 == endcol2 {
                    Win::current().w_cursor.col += 1;
                }
            }
        }
        Win::current().w_cursor.coladd = 0;
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
    /// Answers why when the put must stop.
    fn blockwise_line(
        &mut self,
        i: size_t,
        col: ColNr,
        textcol: &mut ColNr,
        totlen: &mut size_t,
    ) -> Result<(), Stopped> {
        // Pasting past the end of the buffer appends empty lines.
        let mut lines_appended = 0;
        if Win::current().w_cursor.lnum > Buf::current().b_ml.ml_line_count {
            let last = Buf::current().b_ml.ml_line_count;
            ml_append_cstr(last, c"").map_err(|_| Stopped::NoLine)?;
            self.nr_lines += 1;
            lines_appended = 1;
        }

        let lnum = Win::current().w_cursor.lnum;
        let put = &self.lines[i];
        let yanklen = c_int::try_from(put.len()).unwrap_or(c_int::MAX);
        let spaces = if self.flags & PUT_BLOCK_INNER == 0 {
            right_padding(put, self.width)
        } else {
            0
        };

        let mut lines = Buf::current().lines();
        let old = lines.line(lnum);
        let land = land_block(old, col);
        *textcol = land.textcol;

        // The whole insert is `count` copies of the line plus its padding.
        if yanklen + spaces != 0
            && self.count > (c_int::MAX - (land.startspaces + land.endspaces)) / (yanklen + spaces)
        {
            emsg(gettext(E_RESULTING_TEXT_TOO_LONG));
            return Err(Stopped::TooLong);
        }
        let pad = |n: c_int| usize::try_from(n).expect("a count of spaces");
        // The check above keeps the sum inside a `c_int`, and every term is
        // a count.
        *totlen =
            pad(self.count) * pad(yanklen + spaces) + pad(land.startspaces) + pad(land.endspaces);

        // Lay the new line out: the head of the old line, the block's left
        // padding, `count` copies of the block's line each with its own right
        // padding, the spaces replacing the back half of a split tab, and
        // finally the rest of the old line.
        let textcol = pad(land.textcol.get());
        let mut newline = Vec::with_capacity(*totlen + old.len() + 1);
        newline.extend_from_slice(&old[..textcol]);
        newline.resize(newline.len() + pad(land.startspaces), b' ');
        for j in 0..self.count {
            newline.extend_from_slice(put);
            // The block's right padding only goes in if there is text
            // behind it; otherwise it would be trailing white space.
            if (j < self.count - 1 || !land.shortline) && spaces > 0 {
                newline.resize(newline.len() + pad(spaces), b' ');
            } else {
                *totlen -= pad(spaces);
            }
        }
        newline.resize(newline.len() + pad(land.endspaces), b' ');
        // The rest of the old line.
        newline.extend_from_slice(&old[textcol + pad(land.delcount)..]);
        // Neither the buffer's line nor the register's holds a NUL.
        let _ = ml_replace_owned(lnum, XString::from_vec(newline));

        let buf = Buf::current();
        let at = Win::current().w_cursor.lnum - 1;
        let inserted = len_as_int(*totlen) + lines_appended;
        extmark_splice_cols(
            buf,
            at.get(),
            land.textcol,
            ColNr::new(land.delcount),
            ColNr::new(inserted),
            EXTMARK_UNDO,
        );

        Win::current().w_cursor.lnum += 1;
        if i == 0 {
            Win::current().w_cursor.col += land.startspaces;
        }
        Ok(())
    }

    /// The whole blockwise put, starting at the cursor.
    ///
    /// `lnum` is the line the put started on, kept for the `'[` mark and the
    /// redraw range.
    pub(crate) fn blockwise(&mut self, lnum: LineNr) {
        let col = self.block_start_col();

        let mut textcol: ColNr = ColNr::ZERO;
        let mut totlen: size_t = 0;
        for i in 0..self.lines.len() {
            if self
                .blockwise_line(i, col, &mut textcol, &mut totlen)
                .is_err()
            {
                break;
            }
        }

        let to = Buf::current().b_op_start.lnum + len_as_int(self.lines.len()) - self.nr_lines;
        // The range is the lines the put rewrote.
        changed_lines(Buf::current(), lnum, ColNr::ZERO, to, self.nr_lines, true);

        Buf::current().b_op_start = Win::current().w_cursor;
        Buf::current().b_op_start.lnum = lnum;

        Buf::current().b_op_end.lnum = Win::current().w_cursor.lnum - 1;
        Buf::current().b_op_end.col = (textcol + len_as_int(totlen) - 1).max(ColNr::ZERO);
        Buf::current().b_op_end.coladd = 0;

        if self.flags & PUT_CURSEND != 0 {
            Win::current().w_cursor = Buf::current().b_op_end;
            Win::current().w_cursor.col += 1;
            // In Insert mode the cursor may be past the NUL.
            let len = get_cursor_line_len();
            Win::current().w_cursor.col = Win::current().w_cursor.col.min(len);
        } else {
            Win::current().w_cursor.lnum = lnum;
        }
    }
}
