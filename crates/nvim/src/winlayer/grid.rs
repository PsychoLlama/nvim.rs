//! The grid handle: [`GridRef`], a screen grid named by address.
//!
//! Every grid on screen is owned by something else -- a window, the message
//! area, the popup menu, or the `DEFAULT_GRID` static -- and drawing on any
//! of them emits UI events, which the compositor answers by reading *the
//! very grid being drawn on* out of its own layer list. A `&mut ScreenGrid`
//! held across such a call would therefore be aliased, and no `&mut` will
//! thread down the draw path at all (`crate::grid`'s module docs).
//!
//! So the draw path carries a `Copy` handle that names a grid by address and
//! by owner -- a window's own grid carries the window's slot generation and
//! is checked against it at every access, as a [`Win`] is -- and lends no
//! borrow that leaves this module: a read is a by-value accessor
//! (`grid_reads!`), a write one of its in-place writers (`grid_writes!`),
//! and the one borrow held across a call, the cells the UI layer and the
//! compositor read, is counted ([`GridRef::with_cells`]): a write through
//! any copy panics while it is out. The safe constructors are here too: a
//! grid in a static ([`GridCell::grid`]) and a window's own
//! ([`Win::grid_alloc`]); a view stores one of those ([`grid_of_view`]).
//!
//! [`Buf`]: super::Buf

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `winlayer/` row in docs/perimeter.md.
#![allow(unsafe_code)]

use core::cell::UnsafeCell;
use core::ffi::c_int;
use core::mem::offset_of;
use core::ptr::NonNull;

use super::Win;
use crate::global_cell::GlobalCell;
use crate::grid::defs::{GridView, LineBuf, ScreenGrid};
use crate::grid::line::{LinePut, LinePutResult};
use crate::os::libc::size_t;
use crate::slot;
use crate::types::{ColNr, Handle, ScreenAttr, ScreenChar, Window};

/// A live grid, named by address and by its owner rather than borrowed. See
/// the module docs for why nothing in the draw path may hold a
/// `&mut ScreenGrid`.
///
/// A window's own grid ([`Win::grid_alloc`]) carries the window's slot
/// generation, and every access compares it with the slot's word, as a
/// [`Win`] does: a handle kept past its window, or used while an `edit()` of
/// the window is out, panics instead of reaching the grid. A grid in a
/// static ([`GridCell`]) has no owner to outlive (`owner` 0, which no
/// generation is).
///
/// Never null: an absent grid is `Option<GridRef>` (a window's view before
/// `win_grid_alloc` resolves it is `None`).
#[derive(Clone, Copy, Debug)]
pub struct GridRef {
    grid: NonNull<ScreenGrid>,
    /// The owning window's slot generation, or 0 for a static grid.
    owner: u32,
}

/// The `owner` of a grid in a static: even, so no slot generation.
const STATIC_OWNER: u32 = 0;

/// A window's own grid, inside its [`Window`]: reachable only as a
/// [`GridRef`] ([`Win::grid_alloc`]), never through a borrow of the window.
///
/// The `UnsafeCell` is what lets a `read()` guard of the window (a
/// `&Window`) and a write through the grid's handle coexist; an `edit()` of
/// the window lends the whole of it, grid included, and the grid's handle
/// refuses while one is out.
#[repr(transparent)]
pub struct OwnedGrid(UnsafeCell<ScreenGrid>);

impl OwnedGrid {
    /// A grid with no cells yet.
    pub(crate) const fn empty() -> OwnedGrid {
        OwnedGrid(UnsafeCell::new(ScreenGrid::empty()))
    }
}

impl ::core::fmt::Debug for OwnedGrid {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.write_str("OwnedGrid { .. }")
    }
}

/// Why a grid handle refused, out of line.
#[cold]
#[inline(never)]
#[track_caller]
fn refused_grid(lent: bool) -> ! {
    if lent {
        panic!(
            "a grid was written while its cells were lent out (`GridRef::with_cells`) through \
             another copy of its handle"
        )
    }
    panic!(
        "a window's grid was reached after its window was freed, or while an edit() of the \
         window was held: re-derive the grid from the window (the re-entry rule)"
    )
}

/// The field is private, so a handle is made only in this module, by the
/// three constructors below ([`GridCell::grid`], [`Win::grid_alloc`] and the
/// views that store one); each names a grid in a static or in a window's
/// slot, whose memory is never given back, and the owner check stands for
/// the rest.
impl GridRef {
    /// Whether both name the same grid of the same owner.
    pub fn same(self, other: GridRef) -> bool {
        self.grid == other.grid && self.owner == other.owner
    }

    /// The owning window's address: the grid's, less the field's offset,
    /// with the window's provenance (`grid_alloc` derived it so).
    #[inline(always)]
    fn owner_window(self) -> *mut Window {
        self.grid
            .as_ptr()
            .wrapping_byte_sub(offset_of!(Window, w_grid_alloc))
            .cast::<Window>()
    }

    /// Panic unless the grid's owner is still the window this handle was
    /// made for and no `edit()` of it is out: one load and one compare, as a
    /// [`Win`]'s dereference.
    #[inline(always)]
    #[track_caller]
    fn check_owner(self) {
        if self.owner != STATIC_OWNER {
            // SAFETY: a non-static handle's grid is a window's
            // `w_grid_alloc`, so `owner_window` is a slot's value address
            // with its provenance (`Win::grid_alloc`).
            let word = unsafe { slot::word_at(self.owner_window()) };
            if word != self.owner {
                refused_grid(false);
            }
        }
    }

    /// A shared borrow of the grid for one forwarded read, which never
    /// leaves this module.
    #[inline(always)]
    #[track_caller]
    fn read<R>(self, f: impl FnOnce(&ScreenGrid) -> R) -> R {
        self.check_owner();
        // SAFETY: the owner check says the grid is still its window's (or a
        // static's); the borrow lasts one forwarded call that writes nothing
        // (`grid_reads!`), and a write through another copy cannot start
        // inside it.
        f(unsafe { self.grid.as_ref() })
    }

    /// The one `&mut` this handle makes, for the writers below. Private: a
    /// closure from outside this module could reach another copy of the
    /// handle while the borrow is live.
    #[inline(always)]
    #[track_caller]
    fn write<R>(self, f: impl FnOnce(&mut ScreenGrid) -> R) -> R {
        self.check_owner();
        // SAFETY: as `read`, for the place of the count alone (a `Cell`).
        if unsafe { self.grid.as_ref() }.lent.get() != 0 {
            refused_grid(true);
        }
        // SAFETY: as `read`; no cells are lent (above), and the borrow lasts
        // one forwarded call that reaches no other handle (`grid_writes!`).
        f(unsafe { &mut *self.grid.as_ptr() })
    }

    /// `n` cells from `off`, lent to `f`: what the compositor and the UI
    /// layer read across a call. Counted: a write through any copy of the
    /// handle panics while `f` runs, and so does an `edit()` of the owning
    /// window.
    #[track_caller]
    pub(crate) fn with_cells<R>(
        self,
        off: size_t,
        n: size_t,
        f: impl FnOnce(&[ScreenChar], &[ScreenAttr]) -> R,
    ) -> R {
        // A counted read of the owner keeps an edit of the window out.
        let _owner = (self.owner != STATIC_OWNER).then(|| {
            // SAFETY: as `check_owner`.
            unsafe { slot::read_at(self.owner_window(), self.owner) }
        });
        // SAFETY: as `read`: the owner is live (the read above, or a
        // static's grid), and every write refuses while the count is up.
        let grid = unsafe { self.grid.as_ref() };
        grid.lent.set(grid.lent.get() + 1);
        let result = grid.with_cells(off, n, f);
        grid.lent.set(grid.lent.get() - 1);
        result
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
        GridRef {
            grid: NonNull::new(self.0.ptr()).expect("a static's address is never null"),
            owner: STATIC_OWNER,
        }
    }
}

/// The by-value reads: each runs one `&self` [`ScreenGrid`] method through a
/// shared borrow that lasts that call alone. A method listed here answers a
/// value, never a borrow.
macro_rules! grid_reads {
    ($($name:ident($($arg:ident: $ty:ty),*) -> $ret:ty;)*) => {
        impl GridRef {
            $(
                #[doc = concat!("[`ScreenGrid::", stringify!($name), "`].")]
                #[inline]
                #[track_caller]
                pub(crate) fn $name(self, $($arg: $ty),*) -> $ret {
                    self.read(|grid| grid.$name($($arg),*))
                }
            )*
        }
    };
}

/// A getter per public field of [`ScreenGrid`], by value.
macro_rules! grid_field_reads {
    ($($field:ident: $ty:ty;)*) => {
        impl GridRef {
            $(
                #[doc = concat!("The grid's `", stringify!($field), "`.")]
                #[inline]
                #[track_caller]
                pub(crate) fn $field(self) -> $ty {
                    self.read(|grid| grid.$field)
                }
            )*
        }
    };
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
                #[track_caller]
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
                #[track_caller]
                pub(crate) fn $setter(self, value: $ty) {
                    self.write(|grid| grid.$field = value);
                }
            )*
        }
    };
}

grid_reads! {
    is_allocated() -> bool;
    row_start(row: c_int) -> size_t;
    cell_offset(row: c_int, col: c_int) -> size_t;
    char_at(off: size_t) -> ScreenChar;
    attr_at(off: size_t) -> ScreenAttr;
    vcol_at(off: size_t) -> ColNr;
    covers(row: c_int, col: c_int) -> bool;
    invalid_row(row: c_int) -> bool;
    tracks_dirty_cols() -> bool;
}

grid_field_reads! {
    handle: Handle;
    rows: c_int;
    cols: c_int;
    valid: bool;
    throttled: bool;
    blending: bool;
    mouse_enabled: bool;
    zindex: c_int;
    comp_row: c_int;
    comp_col: c_int;
    comp_width: c_int;
    comp_height: c_int;
    comp_index: size_t;
    comp_disabled: bool;
    pending_comp_index_update: bool;
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
    reset();
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
/// A view always names a grid -- a window's own or `DEFAULT_GRID` -- once
/// `win_grid_alloc` has resolved it, which every caller runs after. Panics
/// on a view not resolved yet: the callers that may see one read
/// `view.target` (an `Option`).
#[track_caller]
pub(crate) fn grid_of_view(view: GridView) -> GridRef {
    view.target
        .expect("a view is resolved to its grid before it is drawn on")
}

impl Win {
    /// The window's own grid, as a handle.
    ///
    /// The address is computed from the window's, as [`Win::cursor`] is:
    /// `&raw mut self.edit().w_grid_alloc` would carry the provenance of
    /// that one `&mut Window` -- which the next write to any field of the
    /// window ends, leaving the handle dangling under the aliasing model
    /// (Miri caught it).
    pub(crate) fn grid_alloc(self) -> GridRef {
        let grid = self
            .raw()
            .wrapping_byte_add(offset_of!(Window, w_grid_alloc))
            .cast::<ScreenGrid>();
        // A live window owns its `w_grid_alloc` outright, for as long as the
        // window is (the owner check), and the address keeps the window's
        // provenance.
        GridRef {
            grid: NonNull::new(grid).expect("a window's address is never null"),
            owner: self.generation(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use super::*;
    use crate::global_cell::editor_state_lock;
    use crate::window::{bare_window, free_bare_window};

    /// The draw path's shape: two handles on one grid, each borrow of the
    /// cells lasting one call, writes through either seen by the other --
    /// including after one of them has replaced the cell buffers outright.
    #[test]
    fn two_handles_on_one_grid_take_turns() {
        static GRID: GridCell = GridCell::new();
        let _held = editor_state_lock();
        let view = GridView {
            target: Some(GRID.grid()),
            row_offset: 0,
            col_offset: 0,
        };
        let drawer = grid_of_view(view);
        let compositor = grid_of_view(view);
        assert!(drawer.same(compositor));

        drawer.alloc(2, 3, false, false);
        assert_eq!((compositor.rows(), compositor.cols()), (2, 3));
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

        // Cells lent across a call: reads through the other copy welcome,
        // a write refused until the loan ends.
        drawer.with_cells(moved, 1, |_, attrs| {
            assert_eq!(attrs, [7]);
            assert_eq!(compositor.attr_at(moved), 7);
            let write = catch_unwind(AssertUnwindSafe(|| compositor.set_attr(moved, 8)));
            assert!(write.is_err(), "a write under a loan of the cells");
        });
        compositor.set_attr(moved, 8);
        drawer.free();
        assert!(!compositor.is_allocated());
        assert_eq!(GRID.grid().rows(), 3);
        GRID.grid().reset();
    }

    /// A view whose target is not resolved yet gives no handle at all.
    #[test]
    fn an_unresolved_view_is_refused() {
        let unresolved = GridView {
            target: None,
            row_offset: 0,
            col_offset: 0,
        };
        assert!(catch_unwind(AssertUnwindSafe(|| grid_of_view(unresolved))).is_err());
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
        assert_eq!(win.grid_alloc().rows(), 2);
        // A write to the window around the handle leaves the handle good.
        win.grid_alloc().set_zindex(3);
        win.set_w_wrow(1);
        drop(win.read());
        assert_eq!(own.zindex(), 3);
        assert!(win.grid_alloc().same(own));
        own.free();
        free_bare_window(win);
    }

    /// A window's grid is checked against its window: refused while an
    /// edit of the window is out, an edit refused while its cells are lent,
    /// and the handle refused once the window is gone.
    #[test]
    fn a_window_grid_answers_to_its_window() {
        let _held = editor_state_lock();
        let win = bare_window();
        let own = win.grid_alloc();
        own.alloc(1, 2, false, true);
        {
            let _edit = win.edit();
            assert!(catch_unwind(AssertUnwindSafe(|| own.rows())).is_err());
            assert!(catch_unwind(AssertUnwindSafe(|| own.set_zindex(1))).is_err());
        }
        // A read guard of the window does not reach the grid: writes go on.
        {
            let _read = win.read();
            own.set_zindex(2);
        }
        own.with_cells(0, 2, |chars, _| {
            assert_eq!(chars.len(), 2);
            assert!(catch_unwind(AssertUnwindSafe(|| drop(win.edit()))).is_err());
        });
        drop(win.edit());
        own.free();
        free_bare_window(win);
        assert!(catch_unwind(AssertUnwindSafe(|| own.rows())).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| own.alloc(1, 1, false, true))).is_err());
    }
}
