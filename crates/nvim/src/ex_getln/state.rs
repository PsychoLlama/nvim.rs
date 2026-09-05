//! What the command line is doing between keys.
//!
//! `cmdline_star` hides the text behind `*` for `inputsecret()`, the two
//! `*_drawn` flags say whether the last thing painted was the command line
//! (so a message knows whether it has to repaint it), and `cmdpreview` says
//! the buffer is showing an `'inccommand'` preview rather than its own text.
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

pub(crate) static cmdline_star: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static redrawing_cmdline: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static cmdline_was_last_drawn: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static cmdpreview: GlobalCell<bool> = GlobalCell::new(false);
