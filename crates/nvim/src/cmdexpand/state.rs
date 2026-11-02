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
use crate::types::OptInt;
use core::ffi::c_int;

/// `'laststatus'` and `'winminheight'` as they were before the wildmenu forced
/// a status line open, while it holds one open; `None` otherwise.
pub(crate) static saved_status_opts: GlobalCell<Option<(OptInt, OptInt)>> = GlobalCell::new(None);
pub(crate) static wild_menu_showing: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
