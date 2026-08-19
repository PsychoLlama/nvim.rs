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

/// One terminal cell: a grapheme handle and the highlight attributes it was
/// last drawn with.
#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub struct UCell {
    pub data: schar_T,
    pub attr: sattr_T,
}

/// A [`UGrid`]'s cells, row-major. Carries its own width so a row slice stays
/// correct no matter what the grid's public dimensions say.
pub struct UGridCells {
    pub(crate) width: usize,
    pub(crate) cells: Vec<UCell>,
}

/// The TUI's shadow copy of what the terminal is showing, plus where it
/// believes the cursor sits (`row` is -1 when that is unknown).
///
/// `cells` is `None` until the first [`UGrid::resize`]. That is also what the
/// zeroed `TUIData` the TUI allocates itself out of leaves behind, because
/// `Option<Box<_>>` spells `None` as a null pointer — so there is nothing to
/// initialise before the first resize.
pub struct UGrid {
    pub row: ::core::ffi::c_int,
    pub col: ::core::ffi::c_int,
    pub width: ::core::ffi::c_int,
    pub height: ::core::ffi::c_int,
    pub cells: Option<Box<UGridCells>>,
}
