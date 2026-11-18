//! The grid handle: [`GridRef`], a screen grid named by address.
//!
//! Every grid on screen is owned by something else -- a window, the message
//! area, the popup menu, or the `DEFAULT_GRID` static -- and drawing on any
//! of them emits UI events, which the compositor answers by reading *the
//! very grid being drawn on* out of its own layer list. A `&mut ScreenGrid`
//! held across such a call would therefore be aliased, and no `&mut` will
//! thread down the draw path at all (`crate::grid`'s module docs).
//!
//! So the draw path carries a `Copy` handle that names a grid by address,
//! checked once at its `unsafe` constructor, after which every borrow of the
//! cells lasts exactly one accessor call -- the shape [`Win`] and [`Buf`]
//! have, which is why it lives beside them. The three safe constructors are
//! here too: a grid a global cell holds ([`GridRef::of_cell`]), a window's
//! own ([`Win::grid_alloc`]), and the one a window's view resolves to
//! ([`grid_of_view`]).
//!
//! [`Buf`]: super::Buf

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `winlayer/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::mem::offset_of;

use super::Win;
use crate::global_cell::GlobalCell;
use crate::grid::defs::{GridView, ScreenGrid};
use crate::types::Window;

/// A live grid, named by address rather than borrowed. See the module docs
/// for why nothing in the draw path may hold a `&mut ScreenGrid`.
#[derive(Clone, Copy, Debug)]
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
    /// Safe: a `&'static GlobalCell` is exactly the promise
    /// [`GridRef::new`] has to be told, so the three screen grids need no
    /// `unsafe` at all to be named.
    pub(crate) fn of_cell(cell: &'static GlobalCell<ScreenGrid>) -> GridRef {
        GridRef(cell.ptr())
    }

    /// The address, for the calls that still spell a raw pointer.
    pub fn raw(self) -> *mut ScreenGrid {
        self.0
    }

    /// Whether the handle names no grid at all.
    ///
    /// The one caller that has to ask is `grid::grid_adjust`'s: a window's
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
        // the module docs for why it may not last longer.
        unsafe { &mut *self.0 }
    }
}

/// The grid a view draws on.
///
/// A view always names a live grid -- a window's own or `DEFAULT_GRID` --
/// once `win_grid_alloc` has resolved it, which every caller runs after;
/// that is the one promise the draw path takes on trust, and the reason
/// [`GridRef`] stays a handle.
pub(crate) fn grid_of_view(view: GridView) -> GridRef {
    // SAFETY: as above.
    unsafe { GridRef::new(view.target) }
}

impl Win {
    /// The window's own grid, as a handle.
    ///
    /// The address is computed from the window's, as [`Win::cursor`] is:
    /// `&raw mut self.w_grid_alloc` would go through [`DerefMut`], and the
    /// handle would carry the provenance of that one `&mut Window` -- which
    /// the next write to any field of the window ends, leaving the handle
    /// dangling under the aliasing model (Miri caught it).
    ///
    /// [`DerefMut`]: core::ops::DerefMut
    pub(crate) fn grid_alloc(self) -> GridRef {
        let grid = self
            .raw()
            .wrapping_byte_add(offset_of!(Window, w_grid_alloc))
            .cast::<ScreenGrid>();
        // SAFETY: a live window owns its `w_grid_alloc` outright, for as
        // long as the window is, and the address keeps the window's
        // provenance.
        unsafe { GridRef::new(grid) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_cell::editor_state_lock;
    use crate::types::ScreenChar;
    use crate::window::{bare_window, free_bare_window};

    /// A view of the grid at `target`, unscrolled.
    fn view_of(target: &mut ScreenGrid) -> GridView {
        GridView {
            target,
            row_offset: 0,
            col_offset: 0,
        }
    }

    /// The draw path's shape: two handles on one grid, each borrow of the
    /// cells lasting one call, writes through either seen by the other --
    /// including after one of them has replaced the cell buffers outright.
    #[test]
    fn two_handles_on_one_grid_take_turns() {
        let mut grid = ScreenGrid::empty();
        let view = view_of(&mut grid);
        let mut drawer = grid_of_view(view);
        let mut compositor = grid_of_view(view);
        assert!(drawer.same(compositor));
        assert!(!drawer.is_unresolved());

        drawer.alloc(2, 3, false, true);
        assert_eq!((compositor.rows, compositor.cols), (2, 3));
        let off = compositor.cell_offset(1, 2);
        drawer.cells_mut(off, 1).chars[0] = ScreenChar::from(b'x');
        compositor.set_attr(off, 7);
        assert_eq!(compositor.char_at(off), ScreenChar::from(b'x'));
        assert_eq!(drawer.attr_at(off), 7);

        // A resize that keeps the contents moves them to new buffers, which
        // the other handle reads at once.
        compositor.alloc(3, 4, true, true);
        let moved = drawer.cell_offset(1, 2);
        assert_eq!(drawer.char_at(moved), ScreenChar::from(b'x'));
        assert_eq!(compositor.attr_at(moved), 7);
        drawer.free();
        assert!(!compositor.is_allocated());
        assert_eq!(grid.rows, 3);
    }

    /// A view whose target is not resolved yet gives a handle that says so
    /// and is never dereferenced.
    #[test]
    fn an_unresolved_view_is_a_handle_that_says_so() {
        let unresolved = grid_of_view(GridView {
            target: ::core::ptr::null_mut(),
            row_offset: 0,
            col_offset: 0,
        });
        assert!(unresolved.is_unresolved());
        assert!(unresolved.raw().is_null());
    }

    /// The safe constructors: a grid in a global cell, and a window's own.
    #[test]
    fn a_cell_grid_and_a_window_grid_are_named_safely() {
        use crate::popupmenu::state::PUM_GRID;
        let _held = editor_state_lock();
        let mut cell = GridRef::of_cell(&PUM_GRID);
        assert!(!cell.is_allocated(), "no popup menu in a lib test");
        cell.alloc(1, 2, false, false);
        assert!(GridRef::of_cell(&PUM_GRID).same(cell));
        assert_eq!(GridRef::of_cell(&PUM_GRID).attr_at(1), -1);
        cell.free();

        let mut win = bare_window();
        let mut own = win.grid_alloc();
        own.alloc(2, 2, false, true);
        assert_eq!(win.w_grid_alloc.rows, 2);
        // A write to the window around the handle leaves the handle good.
        win.w_grid_alloc.zindex = 3;
        win.w_wrow = 1;
        assert_eq!(own.zindex, 3);
        assert!(win.grid_alloc().same(own));
        // The window's block is freed without dropping its fields.
        own.free();
        free_bare_window(win);
    }
}
