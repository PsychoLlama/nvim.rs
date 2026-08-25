#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

/// The `grid_line` message being decoded, mid-flight.
///
/// `Copy`: counters and offsets.
#[derive(Copy, Clone)]
pub struct GridLineEvent {
    pub args: [::core::ffi::c_int; 3],
    pub icell: ::core::ffi::c_int,
    pub ncells: ::core::ffi::c_int,
    pub coloff: ::core::ffi::c_int,
    pub cur_attr: ::core::ffi::c_int,
    pub clear_width: ::core::ffi::c_int,
    pub wrap: bool,
}
#[derive(Copy, Clone)]
pub struct GridView {
    pub target: *mut ScreenGrid,
    pub row_offset: ::core::ffi::c_int,
    pub col_offset: ::core::ffi::c_int,
}
/// Not `Copy` and not `Clone`: a grid owns its cells outright. Resizing one
/// is [`ScreenGrid::alloc`], which replaces the buffers in place.
pub struct ScreenGrid {
    pub handle: handle_T,
    /// One glyph per cell, `rows * cols` of them. Empty until
    /// [`alloc`](ScreenGrid::alloc).
    chars: Vec<schar_T>,
    /// The highlight-attribute id of each cell, parallel to `chars`. A
    /// negative one means the UI has not been told what is there.
    attrs: Vec<sattr_T>,
    /// Which virtual column of the buffer line each cell came from, for the
    /// mouse. Parallel to `chars`.
    vcols: Vec<colnr_T>,
    /// Where each row starts in the three buffers above -- indirection on
    /// purpose, so that scrolling permutes offsets instead of moving cells.
    line_offset: Vec<size_t>,
    /// Per row, one past the last column changed since the last flush. Only
    /// a throttled grid keeps these; empty means "not tracked".
    dirty_col: Vec<::core::ffi::c_int>,
    pub rows: ::core::ffi::c_int,
    pub cols: ::core::ffi::c_int,
    pub valid: bool,
    pub throttled: bool,
    pub blending: bool,
    pub mouse_enabled: bool,
    pub zindex: ::core::ffi::c_int,
    pub comp_row: ::core::ffi::c_int,
    pub comp_col: ::core::ffi::c_int,
    pub comp_width: ::core::ffi::c_int,
    pub comp_height: ::core::ffi::c_int,
    pub comp_index: size_t,
    pub comp_disabled: bool,
    pub pending_comp_index_update: bool,
}

/// A run of cells, as the three parallel slices that make one up. Handed out
/// by [`ScreenGrid::cells_mut`] so that a per-cell loop indexes slices
/// instead of walking three raw pointers.
pub(crate) struct GridCells<'a> {
    pub chars: &'a mut [schar_T],
    pub attrs: &'a mut [sattr_T],
    pub vcols: &'a mut [colnr_T],
}

/// The blank a cleared cell holds.
const BLANK: schar_T = b' ' as schar_T;

/// A row number as an index. Panics rather than wrapping: every caller has
/// already bounded the row against `rows`.
fn at(row: ::core::ffi::c_int) -> size_t {
    size_t::try_from(row).expect("a grid row is never negative")
}

impl ScreenGrid {
    /// A grid with no cells: what one is before its first
    /// [`alloc`](ScreenGrid::alloc), and what it falls back to when freed.
    pub(crate) const fn empty() -> ScreenGrid {
        ScreenGrid {
            handle: 0,
            chars: Vec::new(),
            attrs: Vec::new(),
            vcols: Vec::new(),
            line_offset: Vec::new(),
            dirty_col: Vec::new(),
            rows: 0,
            cols: 0,
            valid: false,
            throttled: false,
            blending: false,
            mouse_enabled: true,
            zindex: 0,
            comp_row: 0,
            comp_col: 0,
            comp_width: 0,
            comp_height: 0,
            comp_index: 0,
            comp_disabled: false,
            pending_comp_index_update: true,
        }
    }

    /// Whether the grid has cells. The C spelling was `!grid->chars`, and it
    /// means "nothing may be drawn here yet".
    pub(crate) fn is_allocated(&self) -> bool {
        !self.chars.is_empty()
    }

    /// Where `row` starts in the cell buffers.
    pub(crate) fn row_start(&self, row: ::core::ffi::c_int) -> size_t {
        self.line_offset[at(row)]
    }

    /// Point `row` at `off`. Only scrolling moves a row.
    pub(crate) fn set_row_start(&mut self, row: ::core::ffi::c_int, off: size_t) {
        self.line_offset[at(row)] = off;
    }

    /// Where the cell at `row`/`col` sits in the cell buffers.
    pub(crate) fn cell_offset(&self, row: ::core::ffi::c_int, col: ::core::ffi::c_int) -> size_t {
        self.row_start(row) + at(col)
    }

    /// The glyph at `off`, which is a [`row_start`](ScreenGrid::row_start)
    /// plus a column.
    pub(crate) fn char_at(&self, off: size_t) -> schar_T {
        self.chars[off]
    }

    /// The highlight attribute at `off`.
    pub(crate) fn attr_at(&self, off: size_t) -> sattr_T {
        self.attrs[off]
    }

    /// The buffer column the cell at `off` came from.
    pub(crate) fn vcol_at(&self, off: size_t) -> colnr_T {
        self.vcols[off]
    }

    /// Overwrite the highlight attribute at `off`.
    pub(crate) fn set_attr(&mut self, off: size_t, attr: sattr_T) {
        self.attrs[off] = attr;
    }

    /// `n` cells from `off`: what the compositor and the UI layer read.
    pub(crate) fn cells(&self, off: size_t, n: size_t) -> (&[schar_T], &[sattr_T]) {
        (&self.chars[off..off + n], &self.attrs[off..off + n])
    }

    /// `n` cells from `off`, writable. See [`GridCells`].
    pub(crate) fn cells_mut(&mut self, off: size_t, n: size_t) -> GridCells<'_> {
        GridCells {
            chars: &mut self.chars[off..off + n],
            attrs: &mut self.attrs[off..off + n],
            vcols: &mut self.vcols[off..off + n],
        }
    }

    /// Blank `width` cells from `off`.
    ///
    /// `valid` false marks the attributes as invalid (-1), which is how a
    /// resized grid says "nothing here matches what the UI has".
    pub(crate) fn clear_line(&mut self, off: size_t, width: ::core::ffi::c_int, valid: bool) {
        let end = off + at(width);
        self.chars[off..end].fill(BLANK);
        self.attrs[off..end].fill(if valid { 0 } else { -1 });
        self.vcols[off..end].fill(-1);
    }

    /// Copy `width` cells from `from` to `to`. The runs may overlap.
    pub(crate) fn copy_cells(&mut self, to: size_t, from: size_t, width: ::core::ffi::c_int) {
        let n = at(width);
        self.chars.copy_within(from..from + n, to);
        self.attrs.copy_within(from..from + n, to);
        self.vcols.copy_within(from..from + n, to);
    }

    /// Mark every cell as not matching what the UI has.
    pub(crate) fn invalidate(&mut self) {
        self.attrs.fill(-1);
    }

    /// Whether `row` was invalidated and never redrawn.
    pub(crate) fn invalid_row(&self, row: ::core::ffi::c_int) -> bool {
        self.attrs[self.row_start(row)] < 0
    }

    /// (Re)size the grid to `rows` x `cols`, replacing its cells.
    ///
    /// With `copy`, as much of the old contents as still fits is carried
    /// over and the rest cleared -- what a resize at the "--more--" prompt or
    /// around an external command wants. `valid` is passed through to
    /// [`clear_line`](ScreenGrid::clear_line).
    pub(crate) fn alloc(
        &mut self,
        rows: ::core::ffi::c_int,
        cols: ::core::ffi::c_int,
        copy: bool,
        valid: bool,
    ) {
        let (nrows, ncols) = (at(rows), at(cols));
        let cells = nrows * ncols;
        let mut chars = vec![BLANK; cells];
        let mut attrs = vec![if valid { 0 } else { -1 }; cells];
        let mut vcols = vec![-1; cells];

        if copy && self.is_allocated() {
            let width = ncols.min(at(self.cols));
            for row in 0..nrows.min(at(self.rows)) {
                let (noff, ooff) = (row * ncols, self.line_offset[row]);
                chars[noff..noff + width].copy_from_slice(&self.chars[ooff..ooff + width]);
                attrs[noff..noff + width].copy_from_slice(&self.attrs[ooff..ooff + width]);
                vcols[noff..noff + width].copy_from_slice(&self.vcols[ooff..ooff + width]);
            }
        }

        self.chars = chars;
        self.attrs = attrs;
        self.vcols = vcols;
        self.line_offset = (0..nrows).map(|row| row * ncols).collect();
        self.rows = rows;
        self.cols = cols;
    }

    /// Release the grid's cells. Everything else about it is kept.
    pub(crate) fn free(&mut self) {
        self.chars = Vec::new();
        self.attrs = Vec::new();
        self.vcols = Vec::new();
        self.line_offset = Vec::new();
    }

    /// Whether this grid accumulates dirty columns. Only the message grid
    /// does; see [`track_dirty_cols`](ScreenGrid::track_dirty_cols).
    pub(crate) fn tracks_dirty_cols(&self) -> bool {
        !self.dirty_col.is_empty()
    }

    /// Start accumulating dirty columns, one per row, all clear.
    pub(crate) fn track_dirty_cols(&mut self, rows: ::core::ffi::c_int) {
        self.dirty_col = vec![0; at(rows)];
    }

    /// Stop accumulating dirty columns.
    pub(crate) fn forget_dirty_cols(&mut self) {
        self.dirty_col = Vec::new();
    }

    /// Widen `row`'s dirty run to reach `col`.
    pub(crate) fn raise_dirty_col(&mut self, row: ::core::ffi::c_int, col: ::core::ffi::c_int) {
        let slot = &mut self.dirty_col[at(row)];
        *slot = (*slot).max(col);
    }

    /// Read `row`'s dirty run and clear it: what flushing one row does.
    pub(crate) fn take_dirty_col(&mut self, row: ::core::ffi::c_int) -> ::core::ffi::c_int {
        ::core::mem::take(&mut self.dirty_col[at(row)])
    }

    /// Move the dirty columns up one row, with the lines they describe.
    pub(crate) fn scroll_dirty_cols(&mut self) {
        self.dirty_col.rotate_left(1);
        if let Some(last) = self.dirty_col.last_mut() {
            *last = 0;
        }
    }
}
