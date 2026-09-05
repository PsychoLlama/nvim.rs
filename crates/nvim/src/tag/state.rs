//! The window a tag jump has not opened yet.
//!
//! `:stag` and friends decide to split before they know whether the tag
//! exists, so the split is *postponed*: the size and flags wait here and
//! `jumpto_tag` performs it once the match is in hand. `g_do_tagpreview`
//! is the same for the preview window, `g_tag_at_cursor` says the tag came
//! from CTRL-] rather than a typed name, and `keep_help_flag` keeps the new
//! window a help window.
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

pub(crate) static postponed_split: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static postponed_split_flags: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static postponed_split_tab: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static g_do_tagpreview: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static g_tag_at_cursor: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static keep_help_flag: GlobalCell<bool> = GlobalCell::new(false);
