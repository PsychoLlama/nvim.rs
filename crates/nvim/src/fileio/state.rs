//! Whether the files on disk still match the buffers.
//!
//! A `FocusGained` or a shell command raises `NEED_CHECK_TIMESTAMPS`; the
//! next safe moment runs the check and sets `DID_CHECK_TIMESTAMPS`, and
//! `NO_CHECK_TIMESTAMPS` suppresses both for a scope that is itself
//! rewriting files.
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

pub(crate) static NEED_CHECK_TIMESTAMPS: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static DID_CHECK_TIMESTAMPS: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static NO_CHECK_TIMESTAMPS: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
