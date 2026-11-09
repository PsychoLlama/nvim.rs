//! Where the last mouse event landed.
//!
//! The grid and cell the click or drag was reported on, whether it fell off
//! the bottom of the window or past the end of the line -- which is what
//! decides between a scroll and a selection -- and whether a drag is in
//! progress.
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::global_cell::GlobalCell;
use core::ffi::c_int;

pub(crate) static MOUSE_GRID: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static MOUSE_ROW: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static MOUSE_COL: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static MOUSE_PAST_BOTTOM: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static MOUSE_PAST_EOL: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static MOUSE_DRAGGING: GlobalCell<c_int> = GlobalCell::new(0);
