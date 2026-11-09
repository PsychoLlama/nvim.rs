//! The window a tag jump has not opened yet.
//!
//! `:stag` and friends decide to split before they know whether the tag
//! exists, so the split is *postponed*: the size and flags wait here and
//! `jumpto_tag` performs it once the match is in hand. `G_DO_TAGPREVIEW`
//! is the same for the preview window, `G_TAG_AT_CURSOR` says the tag came
//! from CTRL-] rather than a typed name, and `KEEP_HELP_FLAG` keeps the new
//! window a help window.
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

pub(crate) static POSTPONED_SPLIT: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static POSTPONED_SPLIT_FLAGS: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static POSTPONED_SPLIT_TAB: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static G_DO_TAGPREVIEW: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static G_TAG_AT_CURSOR: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static KEEP_HELP_FLAG: GlobalCell<bool> = GlobalCell::new(false);
