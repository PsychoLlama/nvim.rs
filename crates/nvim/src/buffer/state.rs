//! What to do about a buffer whose swap file already exists.
//!
//! `SWAP_EXISTS_ACTION` is how `attention_message`'s answer reaches the
//! caller that opened the buffer -- the dialog is several frames below the
//! code that has to abandon the edit, so the answer travels in a global
//! rather than a return value, and `SWAP_EXISTS_DID_QUIT` records that the
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

use crate::global_cell::GlobalCell;
use core::ffi::c_int;

pub(crate) const SEA_NONE: c_int = 0 as c_int;
pub(crate) const SEA_DIALOG: c_int = 1 as c_int;
pub(crate) const SEA_QUIT: c_int = 2 as c_int;
pub(crate) static SWAP_EXISTS_ACTION: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static SWAP_EXISTS_DID_QUIT: GlobalCell<bool> = GlobalCell::new(false);
