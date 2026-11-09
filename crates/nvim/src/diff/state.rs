//! What the diff view is showing.
//!
//! The `'diffopt'` values the fold and scroll code reads back (`DIFF_CONTEXT`,
//! `DIFF_FOLDCOLUMN`) and the two "the diff is stale" flags that a change to
//! a diffed buffer raises.
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

pub(crate) static DIFF_CONTEXT: GlobalCell<c_int> = GlobalCell::new(6 as c_int);
pub(crate) static DIFF_FOLDCOLUMN: GlobalCell<c_int> = GlobalCell::new(2 as c_int);
pub(crate) static DIFF_NEED_SCROLLBIND: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static NEED_DIFF_REDRAW: GlobalCell<bool> = GlobalCell::new(false);
