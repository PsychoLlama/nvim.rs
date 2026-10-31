//! The grid handle: [`GridRef`], a screen grid named by address.
//!
//! Every grid on screen is owned by something else -- a window, the message
//! area, the popup menu, or the `default_grid` static -- and drawing on any
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

use super::Win;
use crate::global_cell::GlobalCell;
use crate::types::{GridView, ScreenGrid};

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
/// A view always names a live grid -- a window's own or `default_grid` --
/// once `win_grid_alloc` has resolved it, which every caller runs after;
/// that is the one promise the draw path takes on trust, and the reason
/// [`GridRef`] stays a handle.
pub(crate) fn grid_of_view(view: GridView) -> GridRef {
    // SAFETY: as above.
    unsafe { GridRef::new(view.target) }
}

impl Win {
    /// The window's own grid, as a handle.
    pub(crate) fn grid_alloc(mut self) -> GridRef {
        // SAFETY: a live window owns its `w_grid_alloc` outright, for as
        // long as the window is.
        unsafe { GridRef::new(&raw mut self.w_grid_alloc) }
    }
}
