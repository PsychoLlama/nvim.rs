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

use crate::arabic::arabic_shape;
use crate::decoration::{decor_check_invalid_glyphs, next_virt_text_chunk};
use crate::global_cell::GlobalCell;
use crate::highlight::{hl_apply_winblend, hl_combine_attr};
use crate::log::LOGLVL_DBG;
use crate::main::{
    default_grid, exmode_active, firstwin, full_screen, hl_attr_active, linebuf_attr, linebuf_char,
    linebuf_scratch, linebuf_vcol, p_arshape, p_tbidi, rdb_flags, resizing_screen,
};
use crate::map::mh_clear;
use crate::map_glyph_cache::mh_put_glyph;
use crate::mbyte::{
    mb_string2cells, utf_char2bytes, utf_char2len, utf_cp_bounds, utf_ptr2cells, utf_ptr2cells_len,
    utf_ptr2char, utf_ptr2len, utfc_ptr2len, utfc_ptr2len_len, utfc_ptrlen2schar,
};
use crate::memory::{xcalloc, xfree, xmalloc};
use crate::options::{kOptRdbFlagInvalid, kOptRdbFlagNodelta};
use crate::optionstr::check_chars_options;
use crate::os::cshim::memmove;
use crate::types::ui::kUIMultigrid;
use crate::types::{
    AlignTextPos, BorderTextType, GridView, Integer, MHPutStatus, MapHash, ScreenGrid, Set_glyph,
    String_0, VirtText, WinConfig, colnr_T, handle_T, sattr_T, schar_T, size_t, uint32_t, win_T,
    wline_T,
};
use crate::ui::{
    ui_call_grid_resize, ui_call_grid_scroll, ui_check_cursor_grid, ui_grid_cursor_goto, ui_has,
    ui_line,
};
use ::libc::{abort, memcpy, memset, strlen, strnlen};

use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of;

// Split out for size; the rest of the tree calls all of it as `grid::*`.
pub mod border;
pub mod line;
pub mod schar;

pub use border::grid_draw_border;
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

/// The element type of `linebuf_scratch`, which is reinterpreted as
/// `schar_T`, `sattr_T` or `colnr_T` in turn by [`linebuf_mirror`]. All three
/// are four bytes wide.
type sscratch_T = c_int;

/// Width of the shared scratch line buffers, which are kept as wide as the
/// widest grid.
static LINEBUF_SIZE: GlobalCell<size_t> = GlobalCell::new(0);

/// Resolve a window-relative view to the grid it really draws on, folding the
/// view's offsets into `row_off`/`col_off`.
///
/// Without `ext_multigrid` every window draws on `default_grid`, and the
/// offsets turn window-relative positions into screen-relative ones.
///
/// # Safety
/// `win_grid_alloc` must already have run for this view.
pub unsafe fn grid_adjust(
    grid: *mut GridView,
    row_off: *mut c_int,
    col_off: *mut c_int,
) -> *mut ScreenGrid {
    unsafe {
        *row_off += (*grid).row_offset;
        *col_off += (*grid).col_offset;
        (*grid).target
    }
}

/// Blank `width` cells of `grid` from `off`.
///
/// `valid` false marks the attributes as invalid (-1), which is how a resized
/// grid says "nothing here matches what the UI has".
///
/// # Safety
/// `grid` must be live and `off..off + width` within it.
pub unsafe fn grid_clear_line(grid: *mut ScreenGrid, off: size_t, width: c_int, valid: bool) {
    unsafe {
        let mut col = 0;
        while col < width {
            *(*grid).chars.add(off + col as size_t) = schar_from_ascii(b' ');
            col += 1;
        }
        let fill = if valid { 0 } else { -1 };
        memset(
            (*grid).attrs.add(off).cast::<c_void>(),
            fill,
            width as size_t * size_of::<sattr_T>(),
        );
        memset(
            (*grid).vcols.add(off).cast::<c_void>(),
            -1,
            width as size_t * size_of::<colnr_T>(),
        );
    }
}

/// Mark every cell of `grid` as not matching what the UI has.
///
/// # Safety
/// `grid` must be live and allocated.
pub unsafe fn grid_invalidate(grid: *mut ScreenGrid) {
    unsafe {
        memset(
            (*grid).attrs.cast::<c_void>(),
            -1,
            size_of::<sattr_T>() * (*grid).rows as size_t * (*grid).cols as size_t,
        );
    }
}

/// Whether `row` of `grid` was invalidated and never redrawn.
///
/// # Safety
/// `grid` must be live and allocated, and `row` within it.
unsafe fn grid_invalid_row(grid: *mut ScreenGrid, row: c_int) -> bool {
    unsafe { *(*grid).attrs.add(*(*grid).line_offset.offset(row as isize)) < 0 }
}

/// Read one cell straight out of `grid.chars`, optionally with its attribute.
/// Answers NUL when the position is out of bounds.
///
/// # Safety
/// `grid` must be live.
pub unsafe fn grid_getchar(
    grid: *mut ScreenGrid,
    row: c_int,
    col: c_int,
    attrp: *mut c_int,
) -> schar_T {
    unsafe {
        // Safety check.
        if (*grid).chars.is_null() || row >= (*grid).rows || col >= (*grid).cols {
            return 0;
        }

        let off = *(*grid).line_offset.offset(row as isize) + col as size_t;
        if !attrp.is_null() {
            *attrp = *(*grid).attrs.add(off);
        }
        *(*grid).chars.add(off)
    }
}

/// (Re)allocate `grid` at `rows` x `columns`.
///
/// With `copy`, as much of the old contents as still fits is carried over
/// and the rest cleared -- what a resize at the "--more--" prompt or around
/// an external command wants. `valid` is passed through to
/// [`grid_clear_line`].
///
/// # Safety
/// `grid` must be live; its old buffers are freed.
pub unsafe fn grid_alloc(
    grid: *mut ScreenGrid,
    rows: c_int,
    columns: c_int,
    copy: bool,
    valid: bool,
) {
    unsafe {
        debug_assert!(rows >= 0 && columns >= 0, "rows >= 0 && columns >= 0");
        // The new grid starts as a shallow copy of the old one: everything
        // but the five buffers, which are replaced below. The old ones stay
        // the old grid's until `grid_free` takes them.
        let mut ngrid: ScreenGrid = (*grid).clone();
        let ncells = rows as size_t * columns as size_t;
        ngrid.chars = xmalloc(ncells * size_of::<schar_T>()).cast::<schar_T>();
        ngrid.attrs = xmalloc(ncells * size_of::<sattr_T>()).cast::<sattr_T>();
        ngrid.vcols = xmalloc(ncells * size_of::<colnr_T>()).cast::<colnr_T>();
        memset(
            ngrid.vcols.cast::<c_void>(),
            -1,
            ncells * size_of::<colnr_T>(),
        );
        ngrid.line_offset = xmalloc(rows as size_t * size_of::<size_t>()).cast::<size_t>();
        ngrid.rows = rows;
        ngrid.cols = columns;

        let mut new_row = 0;
        while new_row < ngrid.rows {
            let noff = new_row as size_t * ngrid.cols as size_t;
            *ngrid.line_offset.offset(new_row as isize) = noff;
            grid_clear_line(&raw mut ngrid, noff, columns, valid);

            if copy && new_row < (*grid).rows && !(*grid).chars.is_null() {
                let ooff = *(*grid).line_offset.offset(new_row as isize);
                let len = (*grid).cols.min(ngrid.cols) as size_t;
                memmove(
                    ngrid.chars.add(noff).cast::<c_void>(),
                    (*grid).chars.add(ooff).cast::<c_void>(),
                    len * size_of::<schar_T>(),
                );
                memmove(
                    ngrid.attrs.add(noff).cast::<c_void>(),
                    (*grid).attrs.add(ooff).cast::<c_void>(),
                    len * size_of::<sattr_T>(),
                );
                memmove(
                    ngrid.vcols.add(noff).cast::<c_void>(),
                    (*grid).vcols.add(ooff).cast::<c_void>(),
                    len * size_of::<colnr_T>(),
                );
            }
            new_row += 1;
        }

        grid_free(grid);
        *grid = ngrid;

        // One scratch buffer is shared by every grid, so keep it as wide as
        // the widest of them.
        if LINEBUF_SIZE.get() < columns as size_t {
            xfree(linebuf_char.get().cast::<c_void>());
            xfree(linebuf_attr.get().cast::<c_void>());
            xfree(linebuf_vcol.get().cast::<c_void>());
            xfree(linebuf_scratch.get().cast::<c_void>());
            let n = columns as size_t;
            linebuf_char.set(xmalloc(n * size_of::<schar_T>()).cast::<schar_T>());
            linebuf_attr.set(xmalloc(n * size_of::<sattr_T>()).cast::<sattr_T>());
            linebuf_vcol.set(xmalloc(n * size_of::<colnr_T>()).cast::<colnr_T>());
            linebuf_scratch.set(xmalloc(n * size_of::<sscratch_T>()).cast::<c_char>());
            LINEBUF_SIZE.set(n);
        }
    }
}

/// Release `grid`'s buffers and null them out.
///
/// # Safety
/// `grid` must be live.
pub unsafe fn grid_free(grid: *mut ScreenGrid) {
    unsafe {
        xfree((*grid).chars.cast::<c_void>());
        xfree((*grid).attrs.cast::<c_void>());
        xfree((*grid).vcols.cast::<c_void>());
        xfree((*grid).line_offset.cast::<c_void>());

        (*grid).chars = ::core::ptr::null_mut();
        (*grid).attrs = ::core::ptr::null_mut();
        (*grid).vcols = ::core::ptr::null_mut();
        (*grid).line_offset = ::core::ptr::null_mut();
    }
}

/// (Re)allocate a window's own grid if its size changed while in
/// `ext_multigrid` mode, and update its size, offsets and handle regardless.
///
/// # Safety
/// `wp` must be live.
pub unsafe fn win_grid_alloc(wp: *mut win_T) {
    unsafe {
        let grid: *mut GridView = &raw mut (*wp).w_grid;
        let grid_allocated: *mut ScreenGrid = &raw mut (*wp).w_grid_alloc;

        let total_rows = (*wp).w_height_outer;
        let total_cols = (*wp).w_width_outer;

        // A window only gets a grid of its own when the UI asked for
        // multigrid, or when it is a float (which needs one to be composed).
        let want_allocation = ui_has(kUIMultigrid) || (*wp).w_floating;
        let has_allocation = !(*grid_allocated).chars.is_null();

        if (*wp).w_view_height > (*wp).w_lines_size {
            (*wp).w_lines_valid = 0;
            xfree((*wp).w_lines.cast::<c_void>());
            (*wp).w_lines =
                xcalloc((*wp).w_view_height as size_t + 1, size_of::<wline_T>()).cast::<wline_T>();
            (*wp).w_lines_size = (*wp).w_view_height;
        }

        let mut was_resized = false;
        if want_allocation
            && (!has_allocation
                || (*grid_allocated).rows != total_rows
                || (*grid_allocated).cols != total_cols)
        {
            grid_alloc(
                grid_allocated,
                total_rows,
                total_cols,
                (*wp).w_grid_alloc.valid,
                false,
            );
            (*grid_allocated).valid = true;
            if (*wp).w_floating && (*wp).w_config.border {
                (*wp).w_redr_border = true;
            }
            was_resized = true;
        } else if !want_allocation && has_allocation {
            // Single-grid mode: all rendering is redirected to default_grid
            // and only the window's size and offset are tracked.
            grid_free(grid_allocated);
            (*grid_allocated).valid = false;
            was_resized = true;
        } else if want_allocation && has_allocation && !(*wp).w_grid_alloc.valid {
            grid_invalidate(grid_allocated);
            (*grid_allocated).valid = true;
        }

        if want_allocation {
            (*grid).target = grid_allocated;
            (*grid).row_offset = (*wp).w_winrow_off;
            (*grid).col_offset = (*wp).w_wincol_off;
        } else {
            (*grid).target = default_grid.ptr();
            (*grid).row_offset = (*wp).w_winrow + (*wp).w_winrow_off;
            (*grid).col_offset = (*wp).w_wincol + (*wp).w_wincol_off;
        }

        // Send a grid resize event when a grid was just resized, or when
        // screen_resize asked for every size to be re-sent.
        if (resizing_screen.get() || was_resized) && want_allocation {
            ui_call_grid_resize(
                (*grid_allocated).handle as Integer,
                (*grid_allocated).cols as Integer,
                (*grid_allocated).rows as Integer,
            );
            ui_check_cursor_grid((*grid_allocated).handle);
        }
    }
}

/// Give `grid` a handle if it has none. The grid need not be allocated.
///
/// # Safety
/// `grid` must be live.
pub unsafe fn grid_assign_handle(grid: *mut ScreenGrid) {
    static LAST_GRID_HANDLE: GlobalCell<c_int> = GlobalCell::new(DEFAULT_GRID_HANDLE);
    unsafe {
        if (*grid).handle == 0 {
            LAST_GRID_HANDLE.set(LAST_GRID_HANDLE.get() + 1);
            (*grid).handle = LAST_GRID_HANDLE.get() as handle_T;
        }
    }
}

/// Copy `width` cells of row `from` to row `to`, starting at `col`.
///
/// # Safety
/// `grid` must be live and both rows within it.
unsafe fn linecopy(grid: *mut ScreenGrid, to: c_int, from: c_int, col: c_int, width: c_int) {
    unsafe {
        let off_to = *(*grid).line_offset.offset(to as isize) + col as size_t;
        let off_from = *(*grid).line_offset.offset(from as isize) + col as size_t;

        memmove(
            (*grid).chars.add(off_to).cast::<c_void>(),
            (*grid).chars.add(off_from).cast::<c_void>(),
            width as size_t * size_of::<schar_T>(),
        );
        memmove(
            (*grid).attrs.add(off_to).cast::<c_void>(),
            (*grid).attrs.add(off_from).cast::<c_void>(),
            width as size_t * size_of::<sattr_T>(),
        );
        memmove(
            (*grid).vcols.add(off_to).cast::<c_void>(),
            (*grid).vcols.add(off_from).cast::<c_void>(),
            width as size_t * size_of::<colnr_T>(),
        );
    }
}

/// Insert `line_count` blank lines at `row`, pushing the existing ones down.
///
/// `end` is the line after the scrolled region; `col` and `width` bound it
/// horizontally. All of them are relative to the start of the region.
///
/// A full-width region only permutes `line_offset`, which is why the grid
/// keeps that indirection; a partial-width one has to copy cells.
///
/// # Safety
/// `grid` must be live and the region within it.
pub unsafe fn grid_ins_lines(
    grid: *mut ScreenGrid,
    row: c_int,
    line_count: c_int,
    end: c_int,
    col: c_int,
    width: c_int,
) {
    unsafe {
        if line_count <= 0 {
            return;
        }

        // Shift line_offset[] down by line_count and clear the new lines.
        let mut i = 0;
        while i < line_count {
            let mut j = end - 1 - i;
            if width != (*grid).cols {
                // Only part of each line moves.
                loop {
                    j -= line_count;
                    if j < row {
                        break;
                    }
                    linecopy(grid, j + line_count, j, col, width);
                }
                j += line_count;
                grid_clear_line(
                    grid,
                    *(*grid).line_offset.offset(j as isize) + col as size_t,
                    width,
                    false,
                );
            } else {
                let temp = *(*grid).line_offset.offset(j as isize);
                loop {
                    j -= line_count;
                    if j < row {
                        break;
                    }
                    *(*grid).line_offset.offset((j + line_count) as isize) =
                        *(*grid).line_offset.offset(j as isize);
                }
                *(*grid).line_offset.offset((j + line_count) as isize) = temp;
                grid_clear_line(grid, temp, (*grid).cols, false);
            }
            i += 1;
        }

        if !(*grid).throttled {
            ui_call_grid_scroll(
                (*grid).handle as Integer,
                row as Integer,
                end as Integer,
                col as Integer,
                (col + width) as Integer,
                -line_count as Integer,
                0,
            );
        }
    }
}

/// Delete `line_count` lines at `row`, pulling the ones below it up. The
/// mirror of [`grid_ins_lines`].
///
/// # Safety
/// `grid` must be live and the region within it.
pub unsafe fn grid_del_lines(
    grid: *mut ScreenGrid,
    row: c_int,
    line_count: c_int,
    end: c_int,
    col: c_int,
    width: c_int,
) {
    unsafe {
        if line_count <= 0 {
            return;
        }

        // Shift line_offset[] up by line_count and clear the vacated lines.
        let mut i = 0;
        while i < line_count {
            let mut j = row + i;
            if width != (*grid).cols {
                // Only part of each line moves.
                loop {
                    j += line_count;
                    if j > end - 1 {
                        break;
                    }
                    linecopy(grid, j - line_count, j, col, width);
                }
                j -= line_count;
                grid_clear_line(
                    grid,
                    *(*grid).line_offset.offset(j as isize) + col as size_t,
                    width,
                    false,
                );
            } else {
                let temp = *(*grid).line_offset.offset(j as isize);
                loop {
                    j += line_count;
                    if j > end - 1 {
                        break;
                    }
                    *(*grid).line_offset.offset((j - line_count) as isize) =
                        *(*grid).line_offset.offset(j as isize);
                }
                *(*grid).line_offset.offset((j - line_count) as isize) = temp;
                grid_clear_line(grid, temp, (*grid).cols, false);
            }
            i += 1;
        }

        if !(*grid).throttled {
            ui_call_grid_scroll(
                (*grid).handle as Integer,
                row as Integer,
                end as Integer,
                col as Integer,
                (col + width) as Integer,
                line_count as Integer,
                0,
            );
        }
    }
}

/// The window in the current tab whose own grid has `handle`, if any.
///
/// # Safety
/// The window list must be consistent.
pub unsafe fn get_win_by_grid_handle(handle: handle_T) -> *mut win_T {
    unsafe {
        // FOR_ALL_WINDOWS_IN_TAB over curtab, which always starts at firstwin.
        let mut wp = firstwin.get();
        while !wp.is_null() {
            if (*wp).w_grid_alloc.handle == handle {
                return wp;
            }
            wp = (*wp).w_next;
        }
        ::core::ptr::null_mut()
    }
}
