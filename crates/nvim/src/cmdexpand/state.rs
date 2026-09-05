//! The `'wildmenu'` bar, and the two options it borrows.
//!
//! Showing the bar forces `'laststatus'` and `'winminheight'`; the originals
//! wait in `save_p_ls`/`save_p_wmh` until the bar comes down.
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::global_cell::GlobalCell;
use core::ffi::c_int;

pub(crate) static save_p_ls: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub(crate) static save_p_wmh: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub(crate) static wild_menu_showing: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
