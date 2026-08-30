#![deny(unsafe_op_in_unsafe_fn)]

//! The screen grid: allocating it, scrolling it, reading cells out of it.
//!
//! A `ScreenGrid` is three parallel flat arrays -- `chars` (one [`schar_T`]
//! per cell), `attrs` (a highlight-attribute id) and `vcols` (which virtual
//! column of the buffer line the cell came from, for the mouse) -- plus a
//! `line_offset` table giving each row's start. The offsets are indirection
//! on purpose: scrolling permutes them instead of moving cells.
//!
//! Everything is performed on the internal image first; the UI is told about
//! the difference. That is what lets the editor anticipate the effect of an
//! edit without a full redraw.
//!
//! Split for size:
//!
//! * [`schar`] -- the glyph encoding and its intern cache.
//! * [`line`] -- building one line and diffing it onto the grid.
//! * [`border`] -- the frame around a floating window.
//!
//! # Grids are named, not borrowed
//!
//! Every grid on screen is owned by something else -- a window, the message
//! area, the popup menu, or the `default_grid` static -- and drawing on any
//! of them emits UI events, which the compositor answers by reading *the
//! very grid being drawn on* out of its own layer list. A `&mut ScreenGrid`
//! held across such a call would therefore be aliased, and no `&mut` will
//! thread down the draw path at all.
//!
//! So the whole draw path carries [`GridRef`]: a `Copy` handle that names a
//! grid by address, checked once at its `unsafe` constructor, after which
//! every borrow of the cells lasts exactly one accessor call. It is the one
//! handle -- the compositor's layer, the statusline's canvas and the line
//! batch's target are all `GridRef`, not wrappers around it. `DecorStateRef`
//! and `Rex` are the same shape for the same reason.

use crate::arabic::arabic_shape;
use crate::decoration::{decor_check_invalid_glyphs, next_virt_text_chunk};
use crate::global_cell::GlobalCell;
use crate::highlight::{hl_apply_winblend, hl_combine_attr};
use crate::log::LOGLVL_DBG;
use crate::main::{
    default_grid, exmode_active, full_screen, hl_attr_active, p_arshape, p_tbidi, rdb_flags,
    resizing_screen,
};
use crate::map::mh_clear;
use crate::map_glyph_cache::mh_put_glyph;
use crate::mbyte::{
    mb_string2cells, utf_char2bytes, utf_char2len, utf_cp_bounds, utf_ptr2cells, utf_ptr2cells_len,
    utf_ptr2char, utf_ptr2len, utfc_ptr2len, utfc_ptr2len_len, utfc_ptrlen2schar,
};
use crate::memory::{xcalloc, xfree};
use crate::options::{kOptRdbFlagInvalid, kOptRdbFlagNodelta};
use crate::optionstr::check_chars_options;
use crate::types::ui::kUIMultigrid;
use crate::types::{
    AlignTextPos, BorderTextType, GridCells, GridView, Integer, MHPutStatus, MapHash, ScreenGrid,
    Set_glyph, String_0, VirtText, WinConfig, colnr_T, handle_T, schar_T, size_t, uint32_t, win_T,
    wline_T,
};
use crate::ui::{
    ui_call_grid_resize, ui_call_grid_scroll, ui_check_cursor_grid, ui_grid_cursor_goto, ui_has,
    ui_line,
};
use crate::winlayer::{Win, windows};
use ::libc::{abort, strnlen};

use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of;

// Split out for size; the rest of the tree calls all of it as `grid::*`.
pub mod border;
pub mod line;
pub mod schar;

pub use border::grid_draw_border;
pub(crate) use line::linebuf;
pub use line::{
    LineAttrs, LineSpan, grid_clear, grid_line_clear_end, grid_line_cursor_goto, grid_line_fill,
    grid_line_flush, grid_line_flush_if_valid_row, grid_line_getchar, grid_line_mirror,
    grid_line_put_schar, grid_line_puts, grid_line_start, grid_put_linebuf, linebuf_mirror,
    screengrid_line_start,
};
pub use schar::{
    MAX_SCHAR_SIZE, line_do_arabic_shape, schar_cache_clear, schar_cache_clear_if_full,
    schar_cells, schar_from_ascii, schar_from_buf, schar_from_char, schar_from_str, schar_get,
    schar_get_adv, schar_get_ascii, schar_get_first_codepoint, schar_high, schar_len,
};

const kMHExisting: MHPutStatus = 0;
const kBorderTextTitle: BorderTextType = 0;
const kBorderTextFooter: BorderTextType = 1;
const kAlignLeft: AlignTextPos = 0;
const kAlignCenter: AlignTextPos = 1;
const kAlignRight: AlignTextPos = 2;
/// `grid_put_linebuf` flag: 'rightleft' text.
pub const SLF_RIGHTLEFT: c_int = 1;
/// `grid_put_linebuf` flag: this row is a line wrapped into the next.
pub const SLF_WRAP: c_int = 2;
/// `grid_put_linebuf` flag: number the cleared columns' vcols upwards.
pub const SLF_INC_VCOL: c_int = 4;
/// Handle of `default_grid`; window grids are numbered above it.
const DEFAULT_GRID_HANDLE: c_int = 1;

/// A live grid, named by address rather than borrowed. See the module docs
/// for why nothing in the draw path may hold a `&mut ScreenGrid`.
#[derive(Clone, Copy)]
pub struct GridRef(*mut ScreenGrid);

impl GridRef {
    /// # Safety
    /// `grid` must name a live `ScreenGrid` that outlives every use of the
    /// handle -- a window's own grid, or a local the caller keeps alive.
    /// For a grid that lives in a global cell use [`GridRef::of_cell`],
    /// which discharges this obligation from the cell's own lifetime.
    pub const unsafe fn new(grid: *mut ScreenGrid) -> GridRef {
        GridRef(grid)
    }

    /// The grid a global cell holds.
    ///
    /// Safe, and the only constructor that is: a `&'static GlobalCell` is
    /// exactly the promise [`GridRef::new`] has to be told, so the three
    /// screen grids need no `unsafe` at all to be named.
    pub(crate) fn of_cell(cell: &'static GlobalCell<ScreenGrid>) -> GridRef {
        GridRef(cell.ptr())
    }

    /// The address, for the calls that still spell a raw pointer.
    pub fn raw(self) -> *mut ScreenGrid {
        self.0
    }

    /// Whether the handle names no grid at all.
    ///
    /// The one caller that has to ask is [`grid_adjust`]'s: a window's
    /// `w_grid` view is a transpiled struct whose target is null until the
    /// window has one. Nothing else may produce such a handle -- the batch
    /// in progress and the compositor's selected layer are `Option<GridRef>`
    /// rather than a null one, so that no absent grid is ever a value whose
    /// `Deref` claims a promise it cannot keep.
    pub fn is_unresolved(self) -> bool {
        self.0.is_null()
    }

    /// Whether both name the same grid.
    pub fn same(self, other: GridRef) -> bool {
        ::core::ptr::eq(self.0, other.0)
    }
}

impl ::core::ops::Deref for GridRef {
    type Target = ScreenGrid;

    fn deref(&self) -> &ScreenGrid {
        // SAFETY: the constructor's promise.
        unsafe { &*self.0 }
    }
}

impl ::core::ops::DerefMut for GridRef {
    fn deref_mut(&mut self) -> &mut ScreenGrid {
        // SAFETY: the constructor's promise. The borrow lasts one call: see
        // the type's own docs for why it may not last longer.
        unsafe { &mut *self.0 }
    }
}

/// The screen grid, as a handle.
///
/// One acquisition per path: the grid is the bottom layer of the
/// compositor's stack and is reached from there while it is being drawn on,
/// so it is named rather than borrowed. See the module docs.
pub(crate) fn default_grid_ref() -> GridRef {
    GridRef::of_cell(&default_grid)
}

/// A view of the whole screen: a screen row already is a row of the screen
/// grid, so both offsets are zero.
///
/// Computed, never stored. As a cell (`default_gridview`) this was a second
/// name for `default_grid`'s address, initialised in a `const` initialiser
/// through `as_raw` -- the one spelling that skips the main-thread check --
/// and thereafter trusted to stay in step by hand. See [`msg_grid_view`],
/// whose stored form was a live bug.
///
/// [`msg_grid_view`]: crate::message::msg_grid_view
pub(crate) fn default_gridview() -> GridView {
    GridView {
        target: default_grid_ref().raw(),
        row_offset: 0,
        col_offset: 0,
    }
}

/// Resolve a window-relative view to the grid it really draws on, folding the
/// view's offsets into `row_off`/`col_off`.
///
/// Without `ext_multigrid` every window draws on `default_grid`, and the
/// offsets turn window-relative positions into screen-relative ones.
///
/// # Safety
/// `win_grid_alloc` must already have run for this view.
pub unsafe fn grid_adjust(view: GridView, row_off: &mut c_int, col_off: &mut c_int) -> GridRef {
    *row_off += view.row_offset;
    *col_off += view.col_offset;
    // SAFETY: a view always names a live grid; the caller's promise is that
    // it has been resolved.
    unsafe { GridRef::new(view.target) }
}

/// Read one cell straight out of the grid, optionally with its attribute.
/// Answers NUL when the position is out of bounds.
pub fn grid_getchar(grid: GridRef, row: c_int, col: c_int, attrp: Option<&mut c_int>) -> schar_T {
    // Safety check.
    if !grid.is_allocated() || row >= grid.rows || col >= grid.cols {
        return 0;
    }

    let off = grid.cell_offset(row, col);
    if let Some(attrp) = attrp {
        *attrp = grid.attr_at(off);
    }
    grid.char_at(off)
}

/// (Re)allocate `grid` at `rows` x `columns`, and keep the shared scratch
/// line buffers as wide as the widest grid.
///
/// See [`ScreenGrid::alloc`] for `copy` and `valid`.
pub fn grid_alloc(grid: &mut ScreenGrid, rows: c_int, columns: c_int, copy: bool, valid: bool) {
    debug_assert!(rows >= 0 && columns >= 0, "rows >= 0 && columns >= 0");
    grid.alloc(rows, columns, copy, valid);

    // One line buffer is shared by every grid, so keep it as wide as the
    // widest of them.
    linebuf().widen(columns as size_t);
}

/// (Re)allocate a window's own grid if its size changed while in
/// `ext_multigrid` mode, and update its size, offsets and handle regardless.
///
/// # Safety
/// `wp` must be live.
pub unsafe fn win_grid_alloc(wp: *mut win_T) {
    let grid: *mut GridView = unsafe { &raw mut (*wp).w_grid };
    let grid_allocated: *mut ScreenGrid = unsafe { &raw mut (*wp).w_grid_alloc };

    let total_rows = unsafe { (*wp).w_height_outer };
    let total_cols = unsafe { (*wp).w_width_outer };

    // A window only gets a grid of its own when the UI asked for
    // multigrid, or when it is a float (which needs one to be composed).
    let want_allocation = ui_has(kUIMultigrid) || unsafe { (*wp).w_floating };
    let has_allocation = unsafe { (*grid_allocated).is_allocated() };

    if unsafe { (*wp).w_view_height } > unsafe { (*wp).w_lines_size } {
        unsafe { (*wp).w_lines_valid = 0 };
        unsafe { xfree((*wp).w_lines.cast::<c_void>()) };
        unsafe {
            (*wp).w_lines =
                xcalloc((*wp).w_view_height as size_t + 1, size_of::<wline_T>()).cast::<wline_T>()
        };
        unsafe { (*wp).w_lines_size = (*wp).w_view_height };
    }

    let mut was_resized = false;
    if want_allocation
        && (!has_allocation
            || unsafe { (*grid_allocated).rows } != total_rows
            || unsafe { (*grid_allocated).cols } != total_cols)
    {
        grid_alloc(
            unsafe { &mut *grid_allocated },
            total_rows,
            total_cols,
            unsafe { (*wp).w_grid_alloc.valid },
            false,
        );
        unsafe { (*grid_allocated).valid = true };
        if unsafe { (*wp).w_floating } && unsafe { (*wp).w_config.border } {
            unsafe { (*wp).w_redr_border = true };
        }
        was_resized = true;
    } else if !want_allocation && has_allocation {
        // Single-grid mode: all rendering is redirected to default_grid
        // and only the window's size and offset are tracked.
        unsafe { (*grid_allocated).free() };
        unsafe { (*grid_allocated).valid = false };
        was_resized = true;
    } else if want_allocation && has_allocation && !unsafe { (*wp).w_grid_alloc.valid } {
        unsafe { (*grid_allocated).revalidate() };
    }

    if want_allocation {
        unsafe { (*grid).target = grid_allocated };
        unsafe { (*grid).row_offset = (*wp).w_winrow_off };
        unsafe { (*grid).col_offset = (*wp).w_wincol_off };
    } else {
        unsafe { (*grid).target = default_grid_ref().raw() };
        unsafe { (*grid).row_offset = (*wp).w_winrow + (*wp).w_winrow_off };
        unsafe { (*grid).col_offset = (*wp).w_wincol + (*wp).w_wincol_off };
    }

    // Send a grid resize event when a grid was just resized, or when
    // screen_resize asked for every size to be re-sent.
    if (resizing_screen.get() || was_resized) && want_allocation {
        ui_call_grid_resize(
            unsafe { (*grid_allocated).handle } as Integer,
            unsafe { (*grid_allocated).cols } as Integer,
            unsafe { (*grid_allocated).rows } as Integer,
        );
        ui_check_cursor_grid(unsafe { (*grid_allocated).handle });
    }
}

/// Give `grid` a handle if it has none. The grid need not be allocated.
pub fn grid_assign_handle(grid: &mut ScreenGrid) {
    static LAST_GRID_HANDLE: GlobalCell<c_int> = GlobalCell::new(DEFAULT_GRID_HANDLE);
    if grid.handle == 0 {
        LAST_GRID_HANDLE.set(LAST_GRID_HANDLE.get() + 1);
        grid.handle = LAST_GRID_HANDLE.get() as handle_T;
    }
}

/// Insert `line_count` blank lines at `row`, pushing the existing ones down.
///
/// `end` is the line after the scrolled region; `col` and `width` bound it
/// horizontally. All of them are relative to the start of the region.
///
/// A full-width region only permutes the row offsets, which is why the grid
/// keeps that indirection; a partial-width one has to copy cells.
///
pub fn grid_ins_lines(
    mut grid: GridRef,
    row: c_int,
    line_count: c_int,
    end: c_int,
    col: c_int,
    width: c_int,
) {
    if line_count <= 0 {
        return;
    }

    // Shift the row offsets down by line_count and clear the new lines.
    for i in 0..line_count {
        let mut j = end - 1 - i;
        if width != grid.cols {
            // Only part of each line moves.
            loop {
                j -= line_count;
                if j < row {
                    break;
                }
                let (to, from) = (grid.row_start(j + line_count), grid.row_start(j));
                grid.copy_cells(to + col as size_t, from + col as size_t, width);
            }
            j += line_count;
            let off = grid.row_start(j) + col as size_t;
            grid.clear_line(off, width, false);
        } else {
            let temp = grid.row_start(j);
            loop {
                j -= line_count;
                if j < row {
                    break;
                }
                let off = grid.row_start(j);
                grid.set_row_start(j + line_count, off);
            }
            grid.set_row_start(j + line_count, temp);
            let cols = grid.cols;
            grid.clear_line(temp, cols, false);
        }
    }

    if !grid.throttled {
        ui_call_grid_scroll(
            grid.handle as Integer,
            row as Integer,
            end as Integer,
            col as Integer,
            (col + width) as Integer,
            -line_count as Integer,
            0,
        );
    }
}

/// Delete `line_count` lines at `row`, pulling the ones below it up. The
/// mirror of [`grid_ins_lines`].
///
pub fn grid_del_lines(
    mut grid: GridRef,
    row: c_int,
    line_count: c_int,
    end: c_int,
    col: c_int,
    width: c_int,
) {
    if line_count <= 0 {
        return;
    }

    // Shift the row offsets up by line_count and clear the vacated lines.
    for i in 0..line_count {
        let mut j = row + i;
        if width != grid.cols {
            // Only part of each line moves.
            loop {
                j += line_count;
                if j > end - 1 {
                    break;
                }
                let (to, from) = (grid.row_start(j - line_count), grid.row_start(j));
                grid.copy_cells(to + col as size_t, from + col as size_t, width);
            }
            j -= line_count;
            let off = grid.row_start(j) + col as size_t;
            grid.clear_line(off, width, false);
        } else {
            let temp = grid.row_start(j);
            loop {
                j += line_count;
                if j > end - 1 {
                    break;
                }
                let off = grid.row_start(j);
                grid.set_row_start(j - line_count, off);
            }
            grid.set_row_start(j - line_count, temp);
            let cols = grid.cols;
            grid.clear_line(temp, cols, false);
        }
    }

    if !grid.throttled {
        ui_call_grid_scroll(
            grid.handle as Integer,
            row as Integer,
            end as Integer,
            col as Integer,
            (col + width) as Integer,
            line_count as Integer,
            0,
        );
    }
}

/// The window in the current tab whose own grid has `handle`, if any.
///
/// # Safety
/// The window list must be consistent.
///
/// The walk keeps that promise itself now; the signature stays `unsafe`
/// because every caller still spells the call out that way.
pub unsafe fn get_win_by_grid_handle(handle: handle_T) -> *mut win_T {
    windows()
        .find(|wp| wp.w_grid_alloc.handle == handle)
        .map_or(::core::ptr::null_mut(), Win::raw)
}
