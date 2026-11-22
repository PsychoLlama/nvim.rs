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
//! checked once where it is made, after which every borrow of the
//! cells lasts exactly one accessor call -- the shape [`Win`] and [`Buf`]
//! have, which is why it lives beside them. Being `Copy`, it lends no
//! `&mut`: a write is one of its in-place writers (`grid_writes!`), whose
//! borrow never leaves this module. The safe constructors are here too: a
//! grid in a static ([`GridCell::grid`]), a window's own
//! ([`Win::grid_alloc`]), and the one a window's view resolves to
//! ([`grid_of_view`], [`try_grid_of_view`]).
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

use core::ffi::c_int;
use core::mem::offset_of;
use core::ptr::NonNull;

use super::Win;
use crate::global_cell::GlobalCell;
use crate::grid::defs::{GridView, LineBuf, ScreenGrid};
use crate::grid::line::{LinePut, LinePutResult};
use crate::os::libc::size_t;
use crate::types::{Handle, ScreenAttr, Window};

/// A live grid, named by address rather than borrowed. See the module docs
/// for why nothing in the draw path may hold a `&mut ScreenGrid`.
///
/// Never null: an absent grid is `Option<GridRef>` (a window's view before
/// `win_grid_alloc` resolves it is [`try_grid_of_view`]'s `None`), so no
/// handle's `Deref` claims a promise it cannot keep.
#[derive(Clone, Copy, Debug)]
pub struct GridRef(NonNull<ScreenGrid>);

/// The field is private, so a handle is made only in this module, by the
/// three constructors below ([`GridCell::grid`], [`Win::grid_alloc`],
/// [`try_grid_of_view`]); each names a live grid that outlives every use of
/// the handle, and that is "the constructor's promise" the accessors cite.
impl GridRef {
    /// The address, for the calls that still spell a raw pointer.
    pub fn raw(self) -> *mut ScreenGrid {
        self.0.as_ptr()
    }

    /// Whether both name the same grid.
    pub fn same(self, other: GridRef) -> bool {
        self.0 == other.0
    }
}

/// Reads lend a `&ScreenGrid` for as long as the caller keeps it. Not
/// checked: a `&` held across a write through another copy of the handle is
/// still the construction contract's to forbid (the draw path reads a field
/// out and lets go).
impl ::core::ops::Deref for GridRef {
    type Target = ScreenGrid;

    #[inline(always)]
    fn deref(&self) -> &ScreenGrid {
        // SAFETY: the constructor's promise.
        unsafe { self.0.as_ref() }
    }
}

/// A screen grid that lives in a static: the screen's own, the message
/// area's and the popup menu's.
///
/// Reachable only as a [`GridRef`]: there is no `with`/`with_mut` here, so
/// the cell's borrow tracker has no loan for a handle to go around -- every
/// access to the grid is through a handle, under the handle's rules.
#[derive(Debug)]
pub(crate) struct GridCell(GlobalCell<ScreenGrid>);

impl GridCell {
    /// A cell holding a grid with no cells yet.
    pub(crate) const fn new() -> GridCell {
        GridCell(GlobalCell::new(ScreenGrid::empty()))
    }

    /// The grid, as a handle. Safe: a `'static` grid outlives every handle.
    #[inline]
    pub(crate) fn grid(&'static self) -> GridRef {
        GridRef(NonNull::new(self.0.ptr()).expect("a static's address is never null"))
    }
}

/// The in-place writers: each runs one [`ScreenGrid`] method, or stores one
/// field, through a `&mut` that lives for that call alone and never leaves
/// this module. The handle is `Copy`, so it lends no `&mut` -- two copies
/// would lend two -- and every write to a grid goes through one of these.
///
/// Sound because a forwarded method touches only its own grid: none reaches
/// a handle, a global or user code, so no other borrow of the grid can start
/// while its `&mut` is live. A method that ever does may not be listed.
macro_rules! grid_writes {
    ($($name:ident($($arg:ident: $ty:ty),*) $(-> $ret:ty)?;)*) => {
        impl GridRef {
            $(
                #[doc = concat!("[`ScreenGrid::", stringify!($name), "`], in place.")]
                #[inline]
                pub(crate) fn $name(self, $($arg: $ty),*) $(-> $ret)? {
                    self.write(|grid| grid.$name($($arg),*))
                }
            )*
        }
    };
}

/// A setter per public field of [`ScreenGrid`] the draw path stores, as
/// [`grid_writes!`] does a method.
macro_rules! grid_field_writes {
    ($($setter:ident => $field:ident: $ty:ty;)*) => {
        impl GridRef {
            $(
                #[doc = concat!("Store `", stringify!($field), "`.")]
                #[inline]
                pub(crate) fn $setter(self, value: $ty) {
                    self.write(|grid| grid.$field = value);
                }
            )*
        }
    };
}

impl GridRef {
    /// The one `&mut` this handle makes, for the writers above. Private: a
    /// closure from outside this module could reach another copy of the
    /// handle while the borrow is live.
    #[inline(always)]
    fn write<R>(self, f: impl FnOnce(&mut ScreenGrid) -> R) -> R {
        // SAFETY: the constructor's promise for liveness; for aliasing, the
        // borrow lasts one forwarded call that reaches no other handle
        // (`grid_writes!`).
        f(unsafe { &mut *self.0.as_ptr() })
    }
}

grid_writes! {
    set_row_start(row: c_int, off: size_t);
    set_comp_index(index: size_t);
    set_attr(off: size_t, attr: ScreenAttr);
    clear_line(off: size_t, width: c_int, valid: bool);
    copy_cells(to: size_t, from: size_t, width: c_int);
    invalidate();
    revalidate();
    alloc(rows: c_int, cols: c_int, copy: bool, valid: bool);
    free();
    track_dirty_cols(rows: c_int);
    forget_dirty_cols();
    raise_dirty_col(row: c_int, col: c_int);
    take_dirty_col(row: c_int) -> c_int;
    scroll_dirty_cols();
    put_line_cells(line: &LineBuf, put: &LinePut) -> LinePutResult;
}

grid_field_writes! {
    set_handle => handle: Handle;
    set_valid => valid: bool;
    set_throttled => throttled: bool;
    set_blending => blending: bool;
    set_mouse_enabled => mouse_enabled: bool;
    set_zindex => zindex: c_int;
    set_comp_row => comp_row: c_int;
    set_comp_col => comp_col: c_int;
    set_comp_width => comp_width: c_int;
    set_comp_height => comp_height: c_int;
    set_comp_disabled => comp_disabled: bool;
    set_pending_comp_index_update => pending_comp_index_update: bool;
}

/// The grid a view draws on.
///
/// A view always names a live grid -- a window's own or `DEFAULT_GRID` --
/// once `win_grid_alloc` has resolved it, which every caller runs after;
/// that is the one promise the draw path takes on trust, and the reason
/// [`GridRef`] stays a handle. Panics on a view not resolved yet: the
/// callers that may see one ask [`try_grid_of_view`].
pub(crate) fn grid_of_view(view: GridView) -> GridRef {
    try_grid_of_view(view).expect("a view is resolved to its grid before it is drawn on")
}

/// The grid a view draws on, or `None` before `win_grid_alloc` has given the
/// view one (a window's `w_grid` is a transpiled struct whose target starts
/// null).
pub(crate) fn try_grid_of_view(view: GridView) -> Option<GridRef> {
    // The constructor's promise: a non-null target is a resolved view, as
    // above.
    NonNull::new(view.target).map(GridRef)
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
        // The constructor's promise: a live window owns its `w_grid_alloc`
        // outright, for as long as the window is, and the address keeps the
        // window's provenance.
        GridRef(NonNull::new(grid).expect("a window's address is never null"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_cell::editor_state_lock;
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
        let drawer = grid_of_view(view);
        let compositor = grid_of_view(view);
        assert!(drawer.same(compositor));

        drawer.alloc(2, 3, false, false);
        assert_eq!((compositor.rows, compositor.cols), (2, 3));
        let off = compositor.cell_offset(1, 2);
        assert_eq!(compositor.attr_at(off), -1);
        drawer.clear_line(off, 1, true);
        assert_eq!(compositor.attr_at(off), 0);
        compositor.set_attr(off, 7);
        assert_eq!(drawer.attr_at(off), 7);

        // A resize that keeps the contents moves them to new buffers, which
        // the other handle reads at once.
        compositor.alloc(3, 4, true, true);
        let moved = drawer.cell_offset(1, 2);
        assert_eq!(drawer.char_at(moved), crate::grid::defs::BLANK);
        assert_eq!(compositor.attr_at(moved), 7);
        drawer.free();
        assert!(!compositor.is_allocated());
        assert_eq!(grid.rows, 3);
    }

    /// A view whose target is not resolved yet gives no handle at all.
    #[test]
    fn an_unresolved_view_is_no_handle() {
        let unresolved = GridView {
            target: ::core::ptr::null_mut(),
            row_offset: 0,
            col_offset: 0,
        };
        assert!(try_grid_of_view(unresolved).is_none());
    }

    /// The safe constructors: a grid in a static, and a window's own.
    #[test]
    fn a_cell_grid_and_a_window_grid_are_named_safely() {
        use crate::popupmenu::state::PUM_GRID;
        let _held = editor_state_lock();
        let cell = PUM_GRID.grid();
        assert!(!cell.is_allocated(), "no popup menu in a lib test");
        cell.alloc(1, 2, false, false);
        assert!(PUM_GRID.grid().same(cell));
        assert_eq!(PUM_GRID.grid().attr_at(1), -1);
        cell.free();

        let win = bare_window();
        let own = win.grid_alloc();
        own.alloc(2, 2, false, true);
        assert_eq!(win.grid_alloc().rows, 2);
        // A write to the window around the handle leaves the handle good.
        win.edit().w_grid_alloc.zindex = 3;
        win.set_w_wrow(1);
        assert_eq!(own.zindex, 3);
        assert!(win.grid_alloc().same(own));
        // The window's block is freed without dropping its fields.
        own.free();
        free_bare_window(win);
    }
}
