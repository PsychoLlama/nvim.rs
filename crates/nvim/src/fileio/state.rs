//! Whether the files on disk still match the buffers.
//!
//! A `FocusGained` or a shell command raises `need_check_timestamps`; the
//! next safe moment runs the check and sets `did_check_timestamps`, and
//! `no_check_timestamps` suppresses both for a scope that is itself
//! rewriting files. The `READ_*` flags travel with them: they are what
//! `readfile` is being asked to do.
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
use core::ffi::{c_int, c_uint};

pub(crate) const READ_STDIN: c_uint = 4;
pub(crate) const READ_NEW: c_uint = 1;
pub(crate) static need_check_timestamps: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static did_check_timestamps: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static no_check_timestamps: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
