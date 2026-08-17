#![deny(unsafe_op_in_unsafe_fn)]

//! Building one screen line and pushing it to the grid.
//!
//! Drawing a line is a batch: `grid_line_start` claims the shared scratch
//! buffers (`linebuf_char`/`_attr`/`_vcol`), any number of
//! `grid_line_puts`/`_fill`/`_put_schar` calls fill columns in, and
//! `grid_line_flush` hands the result to [`grid_put_linebuf`], which
//! compares it against what is already on the grid and sends the UI only the
//! cells that actually changed. One batch at a time, process-wide -- that is
//! what [`BATCH`] is.
//!
//! `grid_put_linebuf` runs per cell of every redraw. Plain loops here.

use super::*;
use crate::grid::{SLF_INC_VCOL, SLF_RIGHTLEFT, SLF_WRAP};
use crate::log::logmsg_c;

/// The line batch in progress. Only one exists at a time; `grid` being null
/// means there is none.
#[derive(Clone, Copy)]
struct LineBatch {
    grid: *mut ScreenGrid,
    row: c_int,
    /// Column on the grid that batch column 0 maps to.
    coloff: c_int,
    /// One past the last batch column that fits on the grid.
    maxcol: c_int,
    /// Lowest column written so far, and one past the highest.
    first: c_int,
    last: c_int,
    /// Clear from `last` up to here on flush.
    clear_to: c_int,
    bg_attr: c_int,
    clear_attr: c_int,
    flags: c_int,
}

impl LineBatch {
    const fn new() -> Self {
        LineBatch {
            grid: ::core::ptr::null_mut(),
            row: -1,
            coloff: 0,
            maxcol: 0,
            first: c_int::MAX,
            last: 0,
            clear_to: 0,
            bg_attr: 0,
            clear_attr: 0,
            flags: 0,
        }
    }
}

static BATCH: GlobalCell<LineBatch> = GlobalCell::new(LineBatch::new());

/// Which columns of a line the UI has to be told about.
///
/// `start` of -1 means nothing was touched, matching the sentinel the
/// original carries through this whole function.
#[derive(Clone, Copy)]
struct Dirty {
    start: c_int,
    end: c_int,
}

/// The columns of a line that hold content, and how far the rest is cleared.
#[derive(Clone, Copy)]
pub struct LineSpan {
    /// First column with content.
    pub col: c_int,
    /// One past the last column with content.
    pub endcol: c_int,
    /// Clear up to here. See `SLF_RIGHTLEFT` for which side is cleared.
    pub clear_width: c_int,
}

/// The two attributes [`grid_put_linebuf`] applies.
#[derive(Clone, Copy)]
pub struct LineAttrs {
    /// Combined into every cell of the line and of the cleared columns.
    pub bg: c_int,
    /// Combined into the cleared columns only.
    pub clear: c_int,
}

/// Begin a batch on `row` of `view`.
///
/// Must be matched with a [`grid_line_flush`] before moving to another line.
///
/// # Safety
/// `view` must be live and no other batch may be in progress.
pub unsafe fn grid_line_start(view: *mut GridView, mut row: c_int) {
    unsafe {
        let mut col = 0;
        let grid = grid_adjust(view, &raw mut row, &raw mut col);
        screengrid_line_start(grid, row, col);
    }
}

/// [`grid_line_start`] against a `ScreenGrid` directly, for the callers that
/// have no `GridView` (float borders, the popup menu, the statusline).
///
/// # Safety
/// `grid` must be live and no other batch may be in progress.
pub unsafe fn screengrid_line_start(grid: *mut ScreenGrid, row: c_int, col: c_int) {
    unsafe {
        let b = BATCH.ptr();
        debug_assert!((*b).grid.is_null(), "grid_line_grid == NULL");
        *b = LineBatch {
            grid,
            row,
            coloff: col,
            maxcol: (*grid).cols.min((*grid).cols - col),
            first: LINEBUF_SIZE.get() as c_int,
            last: 0,
            clear_to: 0,
            bg_attr: 0,
            clear_attr: 0,
            flags: 0,
        };
        debug_assert!(
            (*b).maxcol as size_t <= LINEBUF_SIZE.get(),
            "(size_t)grid_line_maxcol <= linebuf_size"
        );

        if full_screen.get() && rdb_flags.get() & kOptRdbFlagInvalid != 0 {
            debug_assert!(!linebuf_char.get().is_null(), "linebuf_char");
            // This batch must not depend on the previous contents of
            // linebuf_char. Poison it so that any such dependency trips an
            // assertion further down.
            memset(
                linebuf_char.get().cast::<c_void>(),
                0xff,
                size_of::<schar_T>() * LINEBUF_SIZE.get(),
            );
            memset(
                linebuf_attr.get().cast::<c_void>(),
                0xff,
                size_of::<sattr_T>() * LINEBUF_SIZE.get(),
            );
        }
    }
}

/// The glyph currently *on screen* at `col` -- not what the pending batch has
/// put there. A space when `col` is off the end of the line.
///
/// # Safety
/// A batch must be in progress.
pub unsafe fn grid_line_getchar(mut col: c_int, attr: *mut c_int) -> schar_T {
    unsafe {
        let b = *BATCH.ptr();
        if col >= b.maxcol {
            // NUL is a very special value (right half of a double-width
            // cell); a space is True Neutral.
            return schar_from_ascii(b' ');
        }
        col += b.coloff;
        let off = *(*b.grid).line_offset.offset(b.row as isize) + col as size_t;
        if !attr.is_null() {
            *attr = *(*b.grid).attrs.add(off);
        }
        *(*b.grid).chars.add(off)
    }
}

/// Put one glyph at `col`.
///
/// # Safety
/// A batch must be in progress.
pub unsafe fn grid_line_put_schar(col: c_int, schar: schar_T, attr: c_int) {
    unsafe {
        let b = BATCH.ptr();
        debug_assert!(!(*b).grid.is_null(), "grid_line_grid");
        if col >= (*b).maxcol {
            return;
        }

        *linebuf_char.get().offset(col as isize) = schar;
        *linebuf_attr.get().offset(col as isize) = attr;
        *linebuf_vcol.get().offset(col as isize) = -1;

        (*b).first = (*b).first.min(col);
        // TODO(bfredl): Y U NO DOUBLEWIDTH?
        (*b).last = (*b).last.max(col + 1);
    }
}

/// Put `text` at `col`, answering the number of cells used. Only ever writes
/// within the one row.
///
/// `textlen` of -1 means "to the NUL".
///
/// # Safety
/// A batch must be in progress and `text` must hold `textlen` readable bytes
/// (or be NUL-terminated).
pub unsafe fn grid_line_puts(
    col: c_int,
    text: *const c_char,
    textlen: c_int,
    attr: c_int,
) -> c_int {
    unsafe {
        let b = BATCH.ptr();
        debug_assert!(!(*b).grid.is_null(), "grid_line_grid");

        let chars = linebuf_char.get();
        let attrs = linebuf_attr.get();
        let vcols = linebuf_vcol.get();

        let max_col = (*b).maxcol;
        let start_col = col;
        let mut col = col;
        let mut ptr = text;

        while col < max_col
            && (textlen < 0 || (ptr.offset_from(text) as c_int) < textlen)
            && *ptr != NUL
        {
            // How many bytes is this character, composing marks included?
            let mbyte_blen = if textlen >= 0 {
                let maxlen = text.offset(textlen as isize).offset_from(ptr) as c_int;
                let blen = utfc_ptr2len_len(ptr, maxlen);
                if blen > maxlen { 1 } else { blen }
            } else {
                utfc_ptr2len(ptr)
            };

            let mut firstc = 0;
            let mut schar = utfc_ptrlen2schar(ptr, mbyte_blen, &raw mut firstc);
            let mut mbyte_cells = utf_ptr2cells_len(ptr, mbyte_blen);
            if mbyte_cells > 2 || schar == 0 {
                mbyte_cells = 1;
                schar = schar_from_char(0xfffd);
            }

            if col + mbyte_cells > max_col {
                // Only one cell left but the character needs two: show a '>'
                // in the last column rather than wrap.
                schar = schar_from_ascii(b'>');
                mbyte_cells = 1;
            }

            // At the start of the text, overwriting the right half of a
            // two-cell character already in this batch truncates it to '>'.
            if ptr == text
                && col > (*b).first
                && col < (*b).last
                && *chars.offset(col as isize) == 0
            {
                *chars.offset((col - 1) as isize) = schar_from_ascii(b'>');
            }

            *chars.offset(col as isize) = schar;
            *attrs.offset(col as isize) = attr;
            *vcols.offset(col as isize) = -1;
            if mbyte_cells == 2 {
                *chars.offset((col + 1) as isize) = 0;
                *attrs.offset((col + 1) as isize) = attr;
                *vcols.offset((col + 1) as isize) = -1;
            }

            col += mbyte_cells;
            ptr = ptr.offset(mbyte_blen as isize);
        }

        if col > start_col {
            (*b).first = (*b).first.min(start_col);
            (*b).last = (*b).last.max(col);
        }

        col - start_col
    }
}

/// Fill `start_col..end_col` with one glyph, answering where it stopped.
///
/// # Safety
/// A batch must be in progress.
pub unsafe fn grid_line_fill(
    start_col: c_int,
    mut end_col: c_int,
    sc: schar_T,
    attr: c_int,
) -> c_int {
    unsafe {
        let b = BATCH.ptr();
        end_col = end_col.min((*b).maxcol);
        if start_col >= end_col {
            return end_col;
        }

        let chars = linebuf_char.get();
        let attrs = linebuf_attr.get();
        let vcols = linebuf_vcol.get();
        let mut col = start_col;
        while col < end_col {
            *chars.offset(col as isize) = sc;
            *attrs.offset(col as isize) = attr;
            *vcols.offset(col as isize) = -1;
            col += 1;
        }

        (*b).first = (*b).first.min(start_col);
        (*b).last = (*b).last.max(end_col);
        end_col
    }
}

/// Declare that the batch clears `start_col..end_col` on flush.
///
/// `bg_attr` applies to both the buffered line and the cleared columns;
/// `clear_attr` only to the cleared columns.
///
/// # Safety
/// A batch must be in progress.
pub unsafe fn grid_line_clear_end(
    start_col: c_int,
    end_col: c_int,
    bg_attr: c_int,
    clear_attr: c_int,
) {
    unsafe {
        let b = BATCH.ptr();
        if (*b).first > start_col {
            (*b).first = start_col;
            (*b).last = start_col;
        }
        (*b).clear_to = end_col;
        (*b).bg_attr = bg_attr;
        (*b).clear_attr = clear_attr;
    }
}

/// Move the cursor to a column of the line being rendered.
///
/// # Safety
/// A batch must be in progress.
pub unsafe fn grid_line_cursor_goto(col: c_int) {
    unsafe {
        let b = *BATCH.ptr();
        ui_grid_cursor_goto((*b.grid).handle, b.row, col);
    }
}

/// Reverse the batch for a 'rightleft' window.
///
/// # Safety
/// A batch must be in progress.
pub unsafe fn grid_line_mirror(width: c_int) {
    unsafe {
        let b = BATCH.ptr();
        (*b).clear_to = (*b).last.max((*b).clear_to);
        if (*b).first >= (*b).clear_to {
            return;
        }
        let (mut first, mut last, mut clear_to) = ((*b).first, (*b).last, (*b).clear_to);
        linebuf_mirror(&mut first, &mut last, &mut clear_to, width);
        (*b).first = first;
        (*b).last = last;
        (*b).clear_to = clear_to;
        (*b).flags |= SLF_RIGHTLEFT;
    }
}

/// Reverse `*firstp..*lastp` of the scratch buffers about a line of `width`
/// columns, and rewrite the three bounds to describe the mirrored line.
///
/// # Safety
/// The scratch buffers must be allocated and at least `width` wide.
pub unsafe fn linebuf_mirror(
    firstp: &mut c_int,
    lastp: &mut c_int,
    clearp: &mut c_int,
    width: c_int,
) {
    unsafe {
        let first = *firstp;
        let last = *lastp;
        let n = (last - first) as size_t;
        let mirror = width - 1; // Mirrors are more fun than television.

        let chars = linebuf_char.get();
        let scratch_char = linebuf_scratch.get().cast::<schar_T>();
        memcpy(
            scratch_char.offset(first as isize).cast::<c_void>(),
            chars.offset(first as isize).cast::<c_void>(),
            n * size_of::<schar_T>(),
        );
        let mut col = first;
        while col < last {
            let rev = mirror - col;
            if col + 1 < last && *scratch_char.offset((col + 1) as isize) == 0 {
                *chars.offset((rev - 1) as isize) = *scratch_char.offset(col as isize);
                *chars.offset(rev as isize) = 0;
                col += 1;
            } else {
                *chars.offset(rev as isize) = *scratch_char.offset(col as isize);
            }
            col += 1;
        }

        // For attrs and vcols: assumes double-width chars are self-consistent.
        let attrs = linebuf_attr.get();
        let scratch_attr = linebuf_scratch.get().cast::<sattr_T>();
        memcpy(
            scratch_attr.offset(first as isize).cast::<c_void>(),
            attrs.offset(first as isize).cast::<c_void>(),
            n * size_of::<sattr_T>(),
        );
        let mut col = first;
        while col < last {
            *attrs.offset((mirror - col) as isize) = *scratch_attr.offset(col as isize);
            col += 1;
        }

        let vcols = linebuf_vcol.get();
        let scratch_vcol = linebuf_scratch.get().cast::<colnr_T>();
        memcpy(
            scratch_vcol.offset(first as isize).cast::<c_void>(),
            vcols.offset(first as isize).cast::<c_void>(),
            n * size_of::<colnr_T>(),
        );
        let mut col = first;
        while col < last {
            *vcols.offset((mirror - col) as isize) = *scratch_vcol.offset(col as isize);
            col += 1;
        }

        *firstp = width - *clearp;
        *clearp = width - first;
        *lastp = width - last;
    }
}

/// End the batch and send the line to the UI.
///
/// # Safety
/// A batch must be in progress.
pub unsafe fn grid_line_flush() {
    unsafe {
        let b = BATCH.ptr();
        let grid = (*b).grid;
        (*b).grid = ::core::ptr::null_mut();
        (*b).clear_to = (*b).last.max((*b).clear_to);
        debug_assert!(
            (*b).clear_to <= (*b).maxcol,
            "grid_line_clear_to <= grid_line_maxcol"
        );
        if (*b).first >= (*b).clear_to {
            return;
        }

        grid_put_linebuf(
            grid,
            (*b).row,
            (*b).coloff,
            LineSpan {
                col: (*b).first,
                endcol: (*b).last,
                clear_width: (*b).clear_to,
            },
            LineAttrs {
                bg: (*b).bg_attr,
                clear: (*b).clear_attr,
            },
            -1,
            (*b).flags,
        );
    }
}

/// Flush the batch, but only if it is on a row the grid really has.
///
/// A stopgap until message.c has been refactored to behave.
///
/// # Safety
/// A batch must be in progress.
pub unsafe fn grid_line_flush_if_valid_row() {
    unsafe {
        let b = BATCH.ptr();
        if (*b).row < 0 || (*b).row >= (*(*b).grid).rows {
            if rdb_flags.get() & kOptRdbFlagInvalid != 0 {
                abort();
            }
            (*b).grid = ::core::ptr::null_mut();
            return;
        }
        grid_line_flush();
    }
}

/// Clear a rectangle of `grid` to `attr`.
///
/// # Safety
/// `grid` must be live and no batch may be in progress.
pub unsafe fn grid_clear(
    grid: *mut GridView,
    start_row: c_int,
    end_row: c_int,
    start_col: c_int,
    mut end_col: c_int,
    attr: c_int,
) {
    unsafe {
        let mut row = start_row;
        while row < end_row {
            grid_line_start(grid, row);
            let b = BATCH.ptr();
            end_col = end_col.min((*b).maxcol);
            if (*b).row >= (*(*b).grid).rows || start_col >= end_col {
                // TODO(bfredl): make callers behave instead.
                (*b).grid = ::core::ptr::null_mut();
                return;
            }
            grid_line_clear_end(start_col, end_col, attr, 0);
            grid_line_flush();
            row += 1;
        }
    }
}

/// Whether the character at batch column `col` differs from what the grid
/// already holds at `off_to`:
///
/// - a different glyph,
/// - different attributes, or
/// - a double-width character whose second cell differs.
///
/// # Safety
/// `grid` must be live and `off_to` in range.
#[inline]
unsafe fn grid_char_needs_redraw(
    grid: *mut ScreenGrid,
    col: c_int,
    off_to: size_t,
    cols: c_int,
) -> bool {
    unsafe {
        cols > 0
            && ((*linebuf_char.get().offset(col as isize) != *(*grid).chars.add(off_to)
                || *linebuf_attr.get().offset(col as isize) != *(*grid).attrs.add(off_to)
                || (cols > 1
                    && *linebuf_char.get().offset((col + 1) as isize) == 0
                    && *linebuf_char.get().offset((col + 1) as isize)
                        != *(*grid).chars.add(off_to + 1)))
                || exmode_active.get() // TODO(bfredl): what in the actual fuck
                || rdb_flags.get() & kOptRdbFlagNodelta != 0)
    }
}

/// What [`copy_changed_cells`] wrote.
struct Copied {
    dirty: Dirty,
    /// Whether the cell just past `endcol` is the orphaned right half of a
    /// double-width character and has to be blanked.
    clear_next: bool,
    /// Where the loop stopped, which is `endcol` rounded up to a whole
    /// character.
    col: c_int,
}

/// Copy `col..endcol` of the scratch buffers onto the grid, writing only the
/// cells that changed and reporting the range that did.
///
/// The vcols are written unconditionally: they are bookkeeping the UI never
/// sees, so there is nothing to compare against.
///
/// # Safety
/// `grid` must be live, `off_to`/`max_off_to` its offsets for this row, and
/// `col..endcol` within the scratch buffers.
unsafe fn copy_changed_cells(
    grid: *mut ScreenGrid,
    off_to: size_t,
    max_off_to: size_t,
    mut col: c_int,
    endcol: c_int,
) -> Copied {
    unsafe {
        let chars = linebuf_char.get();
        let attrs = linebuf_attr.get();
        let vcols = linebuf_vcol.get();

        let mut redraw_next =
            grid_char_needs_redraw(grid, col, off_to + col as size_t, endcol - col);
        let mut start_dirty = -1;
        let mut end_dirty = 0;
        let mut clear_next = false;

        while col < endcol {
            // 1 for a normal char, 2 when it occupies two display cells.
            let char_cells = if col + 1 < endcol && *chars.offset((col + 1) as isize) == 0 {
                2
            } else {
                1
            };
            let redraw_this = redraw_next;
            let off = off_to + col as size_t;
            redraw_next = grid_char_needs_redraw(
                grid,
                col + char_cells,
                off + char_cells as size_t,
                endcol - col - char_cells,
            );

            if redraw_this {
                if start_dirty == -1 {
                    start_dirty = col;
                }
                end_dirty = col + char_cells;
                // Writing a single-width char over a double-width one at the
                // end of the redrawn text leaves the old right half behind.
                // Same when writing the right half of a double-width char
                // over the left half of an existing one.
                if col + char_cells == endcol
                    && off + (char_cells as size_t) < max_off_to
                    && *(*grid).chars.add(off + char_cells as size_t) == 0
                {
                    clear_next = true;
                }

                *(*grid).chars.add(off) = *chars.offset(col as isize);
                *(*grid).attrs.add(off) = *attrs.offset(col as isize);
                if char_cells == 2 {
                    *(*grid).chars.add(off + 1) = *chars.offset((col + 1) as isize);
                    // For simplicity the second half of a double-width
                    // character gets the first half's attributes.
                    *(*grid).attrs.add(off + 1) = *attrs.offset(col as isize);
                }
            }

            *(*grid).vcols.add(off) = *vcols.offset(col as isize);
            if char_cells == 2 {
                *(*grid).vcols.add(off + 1) = *vcols.offset((col + 1) as isize);
            }

            col += char_cells;
        }

        Copied {
            dirty: Dirty {
                start: start_dirty,
                end: end_dirty,
            },
            clear_next,
            col,
        }
    }
}

/// Blank `clear_start..clear_width` of the grid row and fill in its vcols,
/// reporting the range that actually changed.
///
/// # Safety
/// `grid` must be live and `off_to` its offset for this row.
unsafe fn clear_rest_of_line(
    grid: *mut ScreenGrid,
    off_to: size_t,
    clear_start: c_int,
    clear_width: c_int,
    clear_attr: c_int,
    flags: c_int,
    mut last_vcol: colnr_T,
) -> Dirty {
    unsafe {
        let inc_vcol = flags & SLF_INC_VCOL != 0;
        let rightleft = flags & SLF_RIGHTLEFT != 0;

        // Rightleft fills the vcols back to front, before the blanking pass.
        if rightleft {
            let mut col = clear_width - 1;
            while col >= clear_start {
                *(*grid).vcols.add(off_to + col as size_t) = if inc_vcol {
                    last_vcol += 1;
                    last_vcol
                } else {
                    last_vcol
                };
                col -= 1;
            }
        }

        let mut start = -1;
        let mut end = -1;
        // TODO(bfredl): we could cache winline widths.
        let mut col = clear_start;
        while col < clear_width {
            let off = off_to + col as size_t;
            if *(*grid).chars.add(off) != schar_from_ascii(b' ')
                || *(*grid).attrs.add(off) != clear_attr
                || rdb_flags.get() & kOptRdbFlagNodelta != 0
            {
                *(*grid).chars.add(off) = schar_from_ascii(b' ');
                *(*grid).attrs.add(off) = clear_attr;
                if start == -1 {
                    start = col;
                }
                end = col + 1;
            }
            if !rightleft {
                *(*grid).vcols.add(off) = if inc_vcol {
                    last_vcol += 1;
                    last_vcol
                } else {
                    last_vcol
                };
            }
            col += 1;
        }

        Dirty { start, end }
    }
}

/// Move one buffered line to the window grid, writing only the cells that
/// actually changed, and tell the UI about them.
///
/// `flags`:
///
/// - `SLF_RIGHTLEFT` -- 'rightleft' text. When clear, columns `endcol` to
///   `clear_width` are cleared; when set, columns `col` to `endcol` are.
/// - `SLF_WRAP` -- hint to the UI that `row` holds a line wrapped into the
///   next row.
/// - `SLF_INC_VCOL` -- number the cleared columns' vcols upwards from
///   `last_vcol + 1` rather than giving them all `last_vcol`.
///
/// # Safety
/// `grid` must be live, `row` within it, and the scratch buffers must hold
/// the line.
pub unsafe fn grid_put_linebuf(
    grid: *mut ScreenGrid,
    row: c_int,
    coloff: c_int,
    span: LineSpan,
    attrs: LineAttrs,
    last_vcol: colnr_T,
    flags: c_int,
) {
    unsafe {
        let LineSpan {
            mut col,
            mut endcol,
            mut clear_width,
        } = span;
        debug_assert!(
            0 <= row && row < (*grid).rows,
            "0 <= row && row < grid->rows"
        );
        // TODO(bfredl): check all callsites and eliminate.
        endcol = endcol.min((*grid).cols);

        // Safety check; avoids clang warnings down the call stack.
        if (*grid).chars.is_null() || row >= (*grid).rows || coloff >= (*grid).cols {
            logmsg_c!(
                LOGLVL_DBG,
                ::core::ptr::null(),
                c"grid_put_linebuf".as_ptr(),
                line!() as c_int,
                true,
                c"invalid state, skipped".as_ptr(),
            );
            return;
        }

        let invalid_row = grid != default_grid.ptr() && grid_invalid_row(grid, row) && col == 0;
        let off_to = *(*grid).line_offset.offset(row as isize) + coloff as size_t;
        let max_off_to = *(*grid).line_offset.offset(row as isize) + (*grid).cols as size_t;

        // At the start of the text, overwriting the right half of a two-cell
        // character already on the grid truncates it into a '>'.
        if col > 0 && *(*grid).chars.add(off_to + col as size_t) == 0 {
            *linebuf_char.get().offset((col - 1) as isize) = schar_from_ascii(b'>');
            *linebuf_attr.get().offset((col - 1) as isize) =
                *(*grid).attrs.add(off_to + col as size_t - 1);
            col -= 1;
        }

        let mut clear_start = endcol;
        if flags & SLF_RIGHTLEFT != 0 {
            clear_start = col;
            col = endcol;
            endcol = clear_width;
            clear_width = col;
        }

        if p_arshape.get() != 0 && p_tbidi.get() == 0 && endcol > col {
            line_do_arabic_shape(linebuf_char.get().offset(col as isize), endcol - col);
        }

        if attrs.bg != 0 {
            let buf = linebuf_attr.get();
            let mut c = col;
            while c < endcol {
                *buf.offset(c as isize) = hl_combine_attr(attrs.bg, *buf.offset(c as isize));
                c += 1;
            }
        }

        let copied = copy_changed_cells(grid, off_to, max_off_to, col, endcol);
        let mut start_dirty = copied.dirty.start;
        let mut end_dirty = copied.dirty.end;
        col = copied.col;

        if copied.clear_next {
            // Clear the second half of a double-width character whose left
            // half was overwritten with a single-width one.
            *(*grid).chars.add(off_to + col as size_t) = schar_from_ascii(b' ');
            end_dirty += 1;
        }

        // Clearing the left half of a double-width char clears the right too.
        if off_to + (clear_width as size_t) < max_off_to
            && *(*grid).chars.add(off_to + clear_width as size_t) == 0
        {
            clear_width += 1;
        }

        let clear_attr = hl_combine_attr(attrs.bg, attrs.clear);
        let cleared = clear_rest_of_line(
            grid,
            off_to,
            clear_start,
            clear_width,
            clear_attr,
            flags,
            last_vcol,
        );
        let mut clear_end = cleared.end;

        if flags & SLF_RIGHTLEFT != 0 && start_dirty != -1 && cleared.start != -1 {
            if (*grid).throttled || cleared.start >= start_dirty - 5 {
                // Cannot draw now, or too small to be worth a separate
                // "clear" event.
                start_dirty = cleared.start;
            } else {
                ui_line(
                    grid,
                    row,
                    invalid_row,
                    coloff + cleared.start,
                    coloff + cleared.start,
                    coloff + clear_end,
                    clear_attr,
                    flags & SLF_WRAP != 0,
                );
            }
            clear_end = end_dirty;
        } else if start_dirty == -1 {
            // Clear only.
            start_dirty = cleared.start;
            end_dirty = cleared.start;
        } else if clear_end < end_dirty {
            // Put only.
            clear_end = end_dirty;
        } else {
            end_dirty = endcol;
        }

        if clear_end > start_dirty {
            if !(*grid).throttled {
                ui_line(
                    grid,
                    row,
                    invalid_row,
                    coloff + start_dirty,
                    coloff + end_dirty,
                    coloff + clear_end,
                    clear_attr,
                    flags & SLF_WRAP != 0,
                );
            } else if !(*grid).dirty_col.is_null() {
                // TODO(bfredl): really get rid of the extra pseudo terminal
                // in message.c by using a linebuf_char copy for the
                // "throttled message line".
                if clear_end > *(*grid).dirty_col.offset(row as isize) {
                    *(*grid).dirty_col.offset(row as isize) = clear_end;
                }
            }
        }
    }
}
