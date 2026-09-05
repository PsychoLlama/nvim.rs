//! The global argument list, the one `:args` without a window prints.
//!
//! `global_alist` is the list itself; `max_alist_id` mints the id a
//! window-local copy is compared against, and `arg_had_last` records that
//! the last file in it has been reached.
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
use crate::types::{ArgList, Refcount};
use core::ffi::c_int;

pub(crate) static global_alist: GlobalCell<ArgList> = GlobalCell::new(ArgList {
    al_ga: Vec::new(),
    al_refcount: Refcount::ZERO,
    id: 0,
});
pub(crate) static max_alist_id: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static arg_had_last: GlobalCell<bool> = GlobalCell::new(false);
