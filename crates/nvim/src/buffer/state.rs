//! What to do about a buffer whose swap file already exists.
//!
//! `swap_exists_action` is how `attention_message`'s answer reaches the
//! caller that opened the buffer -- the dialog is several frames below the
//! code that has to abandon the edit, so the answer travels in a global
//! rather than a return value, and `swap_exists_did_quit` records that the
//! abandonment already happened.
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
use crate::types::BlnFlags;
use core::ffi::c_int;

pub(crate) const BLN_LISTED: BlnFlags = 2;
pub(crate) const SEA_NONE: c_int = 0 as c_int;
pub(crate) const SEA_DIALOG: c_int = 1 as c_int;
pub(crate) const SEA_QUIT: c_int = 2 as c_int;
pub(crate) static swap_exists_action: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static swap_exists_did_quit: GlobalCell<bool> = GlobalCell::new(false);
