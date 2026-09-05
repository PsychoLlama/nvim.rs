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

use crate::global_cell::GlobalCell;
use core::ffi::c_int;

pub(crate) static mouse_grid: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static mouse_row: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static mouse_col: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static mouse_past_bottom: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static mouse_past_eol: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static mouse_dragging: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
