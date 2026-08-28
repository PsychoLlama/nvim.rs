#![deny(unsafe_op_in_unsafe_fn)]

//! Building one screen line and pushing it to the grid.
//!
//! Drawing a line is a batch: `grid_line_start` claims the shared line
//! buffer ([`LineBuf`], reached through [`linebuf`]), any number of
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
use crate::types::{LineBuf, NUL};

/// The one line under construction. See [`LineBuf`].
static LINEBUF: GlobalCell<LineBuf> = GlobalCell::new(LineBuf::empty());

/// The line buffers, as a handle.
///
/// One acquisition per function, never one per cell: `grid_put_linebuf` and
/// the `drawline` writers run per cell of every redraw. A `&mut LineBuf` will
/// not thread down that path for the same reason a `&mut ScreenGrid` will
/// not -- drawing a line calls out to the decoration providers and to the UI
/// -- so this is [`GridRef`]'s shape again, and every borrow of the columns
/// lasts one accessor call.
#[derive(Clone, Copy)]
pub(crate) struct LineBufRef(*mut LineBuf);

impl ::core::ops::Deref for LineBufRef {
    type Target = LineBuf;

    fn deref(&self) -> &LineBuf {
        // SAFETY: the only constructor names a `static`.
        unsafe { &*self.0 }
    }
}

impl ::core::ops::DerefMut for LineBufRef {
    fn deref_mut(&mut self) -> &mut LineBuf {
        // SAFETY: the only constructor names a `static`. The borrow lasts one
        // call: see the type's own docs for why it may not last longer.
        unsafe { &mut *self.0 }
    }
}

/// The shared scratch line buffers.
pub(crate) fn linebuf() -> LineBufRef {
    LineBufRef(LINEBUF.ptr())
}

/// The line batch in progress. Only one exists at a time; `grid` being
/// `None` means there is none.
#[derive(Clone, Copy)]
struct LineBatch {
    grid: Option<GridRef>,
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
            grid: None,
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

/// The batch in progress, as a handle.
///
/// One acquisition per entry point, for the same reason [`LineBufRef`] is
/// one: `grid_line_flush` hands the batch to `grid_put_linebuf`, which draws
/// -- and drawing re-enters this module through the compositor. Every borrow
/// of a field lasts one access.
#[derive(Clone, Copy)]
struct BatchRef(*mut LineBatch);

impl ::core::ops::Deref for BatchRef {
    type Target = LineBatch;

    fn deref(&self) -> &LineBatch {
        // SAFETY: the only constructor names a `static`.
        unsafe { &*self.0 }
    }
}

impl ::core::ops::DerefMut for BatchRef {
    fn deref_mut(&mut self) -> &mut LineBatch {
        // SAFETY: the only constructor names a `static`. The borrow lasts one
        // access: see the type's own docs for why it may not last longer.
        unsafe { &mut *self.0 }
    }
}

/// The one line batch.
fn batch() -> BatchRef {
    BatchRef(BATCH.ptr())
}

/// A column as an index. Every column reaching this module is non-negative.
#[inline(always)]
fn at(col: c_int) -> size_t {
    debug_assert!(col >= 0, "col >= 0");
    col as size_t
}

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
pub unsafe fn grid_line_start(view: GridView, mut row: c_int) {
    let mut col = 0;
    // SAFETY: the caller's promise, for both calls.
    let grid = unsafe { grid_adjust(view, &mut row, &mut col) };
    unsafe { screengrid_line_start(grid, row, col) };
}

/// [`grid_line_start`] against a `ScreenGrid` directly, for the callers that
/// have no `GridView` (float borders, the popup menu, the statusline).
///
/// # Safety
/// No other batch may be in progress.
pub unsafe fn screengrid_line_start(grid: GridRef, row: c_int, col: c_int) {
    let mut buf = linebuf();
    let mut b = batch();
    debug_assert!(b.grid.is_none(), "grid_line_grid == NULL");
    *b = LineBatch {
        grid: Some(grid),
        row,
        coloff: col,
        maxcol: grid.cols.min(grid.cols - col),
        first: buf.width() as c_int,
        last: 0,
        clear_to: 0,
        bg_attr: 0,
        clear_attr: 0,
        flags: 0,
    };
    debug_assert!(
        b.maxcol as size_t <= buf.width(),
        "(size_t)grid_line_maxcol <= linebuf_size"
    );

    if full_screen.get() && rdb_flags.get() & kOptRdbFlagInvalid != 0 {
        // This batch must not depend on the previous line's contents. Poison
        // the buffers so that any such dependency trips an assertion further
        // down.
        buf.poison();
    }
}

/// The glyph currently *on screen* at `col` -- not what the pending batch has
/// put there. A space when `col` is off the end of the line.
///
/// # Safety
/// A batch must be in progress.
pub unsafe fn grid_line_getchar(mut col: c_int, attr: *mut c_int) -> schar_T {
    let b = *batch();
    if col >= b.maxcol {
        // NUL is a very special value (right half of a double-width
        // cell); a space is True Neutral.
        return schar_from_ascii(b' ');
    }
    col += b.coloff;
    let grid = b.grid.expect("a batch is in progress");
    let off = grid.cell_offset(b.row, col);
    if !attr.is_null() {
        unsafe { *attr = grid.attr_at(off) };
    }
    grid.char_at(off)
}

/// Put one glyph at `col`. A no-op when no batch is open.
pub fn grid_line_put_schar(col: c_int, schar: schar_T, attr: c_int) {
    let mut b = batch();
    debug_assert!(b.grid.is_some(), "grid_line_grid");
    if col >= b.maxcol {
        return;
    }

    linebuf().put(col as size_t, schar, attr, -1);

    b.first = b.first.min(col);
    // TODO(bfredl): Y U NO DOUBLEWIDTH?
    b.last = b.last.max(col + 1);
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
    let mut buf = linebuf();
    let (chars, attrs, vcols) = buf.parts_mut();
    let mut b = batch();
    // SAFETY: the caller's promise, for the batch and for `text`.
    debug_assert!(b.grid.is_some(), "grid_line_grid");

    let max_col = b.maxcol;
    let start_col = col;
    let mut col = col;
    let mut ptr = text;

    while col < max_col
        && (textlen < 0 || (unsafe { ptr.offset_from(text) } as c_int) < textlen)
        && unsafe { *ptr } != NUL as c_char
    {
        // How many bytes is this character, composing marks included?
        let mbyte_blen = if textlen >= 0 {
            let maxlen = unsafe { text.offset(textlen as isize).offset_from(ptr) } as c_int;
            let blen = unsafe { utfc_ptr2len_len(ptr, maxlen) };
            if blen > maxlen { 1 } else { blen }
        } else {
            unsafe { utfc_ptr2len(ptr) }
        };

        let mut firstc = 0;
        let mut schar = unsafe { utfc_ptrlen2schar(ptr, mbyte_blen, &raw mut firstc) };
        let mut mbyte_cells = unsafe { utf_ptr2cells_len(ptr, mbyte_blen) };
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
        if ptr == text && col > b.first && col < b.last && chars[col as size_t] == 0 {
            chars[(col - 1) as size_t] = schar_from_ascii(b'>');
        }

        chars[col as size_t] = schar;
        attrs[col as size_t] = attr;
        vcols[col as size_t] = -1;
        if mbyte_cells == 2 {
            chars[(col + 1) as size_t] = 0;
            attrs[(col + 1) as size_t] = attr;
            vcols[(col + 1) as size_t] = -1;
        }

        col += mbyte_cells;
        ptr = unsafe { ptr.offset(mbyte_blen as isize) };
    }

    if col > start_col {
        b.first = b.first.min(start_col);
        b.last = b.last.max(col);
    }

    col - start_col
}

/// Fill `start_col..end_col` with one glyph, answering where it stopped.
///
/// # Safety
/// A batch must be in progress.
pub fn grid_line_fill(start_col: c_int, mut end_col: c_int, sc: schar_T, attr: c_int) -> c_int {
    let mut b = batch();
    end_col = end_col.min(b.maxcol);
    if start_col >= end_col {
        return end_col;
    }

    let mut buf = linebuf();
    let (chars, attrs, vcols) = buf.parts_mut();
    let span = at(start_col)..at(end_col);
    chars[span.clone()].fill(sc);
    attrs[span.clone()].fill(attr);
    vcols[span].fill(-1);

    b.first = b.first.min(start_col);
    b.last = b.last.max(end_col);
    end_col
}

/// Declare that the batch clears `start_col..end_col` on flush.
///
/// `bg_attr` applies to both the buffered line and the cleared columns;
/// `clear_attr` only to the cleared columns.
pub fn grid_line_clear_end(start_col: c_int, end_col: c_int, bg_attr: c_int, clear_attr: c_int) {
    let mut b = batch();
    if b.first > start_col {
        b.first = start_col;
        b.last = start_col;
    }
    b.clear_to = end_col;
    b.bg_attr = bg_attr;
    b.clear_attr = clear_attr;
}

/// Move the cursor to a column of the line being rendered.
///
/// # Safety
/// A batch must be in progress.
pub unsafe fn grid_line_cursor_goto(col: c_int) {
    let b = *batch();
    let grid = b.grid.expect("a batch is in progress");
    ui_grid_cursor_goto(grid.handle, b.row, col);
}

/// Reverse the batch for a 'rightleft' window.
pub fn grid_line_mirror(width: c_int) {
    let mut b = batch();
    b.clear_to = b.last.max(b.clear_to);
    if b.first >= b.clear_to {
        return;
    }
    let (mut first, mut last, mut clear_to) = (b.first, b.last, b.clear_to);
    linebuf_mirror(&mut first, &mut last, &mut clear_to, width);
    b.first = first;
    b.last = last;
    b.clear_to = clear_to;
    b.flags |= SLF_RIGHTLEFT;
}

/// Reverse `*firstp..*lastp` of the line buffer about a line of `width`
/// columns, and rewrite the three bounds to describe the mirrored line.
pub fn linebuf_mirror(firstp: &mut c_int, lastp: &mut c_int, clearp: &mut c_int, width: c_int) {
    let (first, last) = (*firstp, *lastp);
    linebuf().mirror(first, last, width);

    *firstp = width - *clearp;
    *clearp = width - first;
    *lastp = width - last;
}

/// End the batch and send the line to the UI.
///
/// # Safety
/// A batch must be in progress.
pub unsafe fn grid_line_flush() {
    let mut b = batch();
    // Ended here, whether or not there turns out to be anything to send.
    let grid = b.grid.take();
    b.clear_to = b.last.max(b.clear_to);
    debug_assert!(
        b.clear_to <= b.maxcol,
        "grid_line_clear_to <= grid_line_maxcol"
    );
    if b.first >= b.clear_to {
        return;
    }

    unsafe {
        grid_put_linebuf(
            grid.expect("a batch is in progress"),
            b.row,
            b.coloff,
            LineSpan {
                col: b.first,
                endcol: b.last,
                clear_width: b.clear_to,
            },
            LineAttrs {
                bg: b.bg_attr,
                clear: b.clear_attr,
            },
            -1,
            b.flags,
        )
    };
}

/// Flush the batch, but only if it is on a row the grid really has.
///
/// A stopgap until message.c has been refactored to behave.
///
/// # Safety
/// A batch must be in progress.
pub unsafe fn grid_line_flush_if_valid_row() {
    let mut b = batch();
    if b.row < 0 || b.row >= b.grid.expect("a batch is in progress").rows {
        if rdb_flags.get() & kOptRdbFlagInvalid != 0 {
            unsafe { abort() };
        }
        b.grid = None;
        return;
    }
    unsafe { grid_line_flush() };
}

/// Clear a rectangle of `grid` to `attr`.
///
/// # Safety
/// `grid` must be live and no batch may be in progress.
pub unsafe fn grid_clear(
    view: GridView,
    start_row: c_int,
    end_row: c_int,
    start_col: c_int,
    mut end_col: c_int,
    attr: c_int,
) {
    let mut row = start_row;
    while row < end_row {
        unsafe { grid_line_start(view, row) };
        let mut b = batch();
        end_col = end_col.min(b.maxcol);
        if b.row >= b.grid.expect("a batch is in progress").rows || start_col >= end_col {
            // TODO(bfredl): make callers behave instead.
            b.grid = None;
            return;
        }
        grid_line_clear_end(start_col, end_col, attr, 0);
        unsafe { grid_line_flush() };
        row += 1;
    }
}

/// Whether the character at batch column `col` differs from what the grid
/// already holds at `off_to`:
///
/// - a different glyph,
/// - different attributes, or
/// - a double-width character whose second cell differs.
///
#[inline]
fn grid_char_needs_redraw(
    line: &LineBuf,
    on_grid: &GridCells<'_>,
    col: c_int,
    cols: c_int,
) -> bool {
    let at = col as size_t;
    let (chars, attrs) = (line.chars(), line.attrs());
    cols > 0
        && ((chars[at] != on_grid.chars[at]
            || attrs[at] != on_grid.attrs[at]
            || (cols > 1 && chars[at + 1] == 0 && chars[at + 1] != on_grid.chars[at + 1]))
            || exmode_active.get() // TODO(bfredl): what in the actual fuck
            || rdb_flags.get() & kOptRdbFlagNodelta != 0)
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
/// `on_grid` is the row from the batch's first column on; every column here
/// indexes into it.
///
fn copy_changed_cells(
    line: &LineBuf,
    on_grid: &mut GridCells<'_>,
    mut col: c_int,
    endcol: c_int,
) -> Copied {
    let (chars, attrs, vcols) = (line.chars(), line.attrs(), line.vcols());

    let mut redraw_next = grid_char_needs_redraw(line, on_grid, col, endcol - col);
    let mut start_dirty = -1;
    let mut end_dirty = 0;
    let mut clear_next = false;

    while col < endcol {
        // 1 for a normal char, 2 when it occupies two display cells.
        let char_cells = if col + 1 < endcol && chars[(col + 1) as size_t] == 0 {
            2
        } else {
            1
        };
        let redraw_this = redraw_next;
        let off = col as size_t;
        redraw_next =
            grid_char_needs_redraw(line, on_grid, col + char_cells, endcol - col - char_cells);

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
                && off + (char_cells as size_t) < on_grid.chars.len()
                && on_grid.chars[off + char_cells as size_t] == 0
            {
                clear_next = true;
            }

            on_grid.chars[off] = chars[off];
            on_grid.attrs[off] = attrs[off];
            if char_cells == 2 {
                on_grid.chars[off + 1] = chars[off + 1];
                // For simplicity the second half of a double-width
                // character gets the first half's attributes.
                on_grid.attrs[off + 1] = attrs[off];
            }
        }

        on_grid.vcols[off] = vcols[off];
        if char_cells == 2 {
            on_grid.vcols[off + 1] = vcols[off + 1];
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

/// Blank `clear_start..clear_width` of the grid row and fill in its vcols,
/// reporting the range that actually changed.
///
/// `on_grid` is the row from the batch's first column on, as in
/// [`copy_changed_cells`].
fn clear_rest_of_line(
    on_grid: &mut GridCells<'_>,
    clear_start: c_int,
    clear_width: c_int,
    clear_attr: c_int,
    flags: c_int,
    mut last_vcol: colnr_T,
) -> Dirty {
    let inc_vcol = flags & SLF_INC_VCOL != 0;
    let rightleft = flags & SLF_RIGHTLEFT != 0;

    // Rightleft fills the vcols back to front, before the blanking pass.
    if rightleft {
        let mut col = clear_width - 1;
        while col >= clear_start {
            on_grid.vcols[col as size_t] = if inc_vcol {
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
        let off = col as size_t;
        if on_grid.chars[off] != schar_from_ascii(b' ')
            || on_grid.attrs[off] != clear_attr
            || rdb_flags.get() & kOptRdbFlagNodelta != 0
        {
            on_grid.chars[off] = schar_from_ascii(b' ');
            on_grid.attrs[off] = clear_attr;
            if start == -1 {
                start = col;
            }
            end = col + 1;
        }
        if !rightleft {
            on_grid.vcols[off] = if inc_vcol {
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
/// `row` must be within `grid` and the scratch buffers must hold the line.
pub unsafe fn grid_put_linebuf(
    mut grid: GridRef,
    row: c_int,
    coloff: c_int,
    span: LineSpan,
    attrs: LineAttrs,
    last_vcol: colnr_T,
    flags: c_int,
) {
    let mut line = linebuf();
    // SAFETY: the caller's promise about `row` and the buffers.
    let LineSpan {
        mut col,
        mut endcol,
        mut clear_width,
    } = span;
    debug_assert!(0 <= row && row < grid.rows, "0 <= row && row < grid->rows");
    // TODO(bfredl): check all callsites and eliminate.
    endcol = endcol.min(grid.cols);

    // Safety check; avoids clang warnings down the call stack.
    if !grid.is_allocated() || row >= grid.rows || coloff >= grid.cols {
        unsafe {
            logmsg_c!(
                LOGLVL_DBG,
                ::core::ptr::null(),
                c"grid_put_linebuf".as_ptr(),
                line!() as c_int,
                true,
                c"invalid state, skipped".as_ptr(),
            )
        };
        return;
    }

    let invalid_row = !grid.same(default_grid_ref()) && grid.invalid_row(row) && col == 0;
    // The row from `coloff` on: every column below indexes into it.
    let off_to = grid.cell_offset(row, coloff);
    let span_width = (grid.cols - coloff) as size_t;

    // At the start of the text, overwriting the right half of a two-cell
    // character already on the grid truncates it into a '>'.
    if col > 0 && grid.char_at(off_to + col as size_t) == 0 {
        let at = (col - 1) as size_t;
        line.chars_mut()[at] = schar_from_ascii(b'>');
        line.attrs_mut()[at] = grid.attr_at(off_to + col as size_t - 1);
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
        unsafe { line_do_arabic_shape(&mut line.chars_mut()[at(col)..at(endcol)]) };
    }

    if attrs.bg != 0 {
        for cell in &mut line.attrs_mut()[at(col)..at(endcol)] {
            *cell = unsafe { hl_combine_attr(attrs.bg, *cell) };
        }
    }

    let clear_attr = unsafe { hl_combine_attr(attrs.bg, attrs.clear) };
    let mut on_grid = grid.cells_mut(off_to, span_width);
    let copied = copy_changed_cells(&line, &mut on_grid, col, endcol);
    let mut start_dirty = copied.dirty.start;
    let mut end_dirty = copied.dirty.end;
    col = copied.col;

    if copied.clear_next {
        // Clear the second half of a double-width character whose left
        // half was overwritten with a single-width one.
        on_grid.chars[col as size_t] = schar_from_ascii(b' ');
        end_dirty += 1;
    }

    // Clearing the left half of a double-width char clears the right too.
    if (clear_width as size_t) < span_width && on_grid.chars[clear_width as size_t] == 0 {
        clear_width += 1;
    }

    let cleared = clear_rest_of_line(
        &mut on_grid,
        clear_start,
        clear_width,
        clear_attr,
        flags,
        last_vcol,
    );
    let mut clear_end = cleared.end;

    if flags & SLF_RIGHTLEFT != 0 && start_dirty != -1 && cleared.start != -1 {
        if grid.throttled || cleared.start >= start_dirty - 5 {
            // Cannot draw now, or too small to be worth a separate
            // "clear" event.
            start_dirty = cleared.start;
        } else {
            unsafe {
                ui_line(
                    grid,
                    row,
                    invalid_row,
                    coloff + cleared.start,
                    coloff + cleared.start,
                    coloff + clear_end,
                    clear_attr,
                    flags & SLF_WRAP != 0,
                )
            };
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
        if !grid.throttled {
            unsafe {
                ui_line(
                    grid,
                    row,
                    invalid_row,
                    coloff + start_dirty,
                    coloff + end_dirty,
                    coloff + clear_end,
                    clear_attr,
                    flags & SLF_WRAP != 0,
                )
            };
        } else if grid.tracks_dirty_cols() {
            // TODO(bfredl): really get rid of the extra pseudo terminal
            // in message.c by using a line-buffer copy for the
            // "throttled message line".
            grid.raise_dirty_col(row, clear_end);
        }
    }
}
