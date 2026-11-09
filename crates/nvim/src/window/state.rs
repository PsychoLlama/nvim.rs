//! The re-entrancy flags the window code sets around itself.
//!
//! Each one says "the caller is mid-operation on the window tree, so the
//! usual repair pass must not run": `SKIP_WIN_FIX_CURSOR` and
//! `SKIP_WIN_FIX_SCROLL` over a resize that will place the cursor itself,
//! `SKIP_UPDATE_TOPLINE` over a move that is not a scroll, and
//! `TABPAGE_MOVE_DISALLOWED` over a walk an autocommand must not reorder.
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
use core::ffi::{CStr, c_int};

pub(crate) static SKIP_WIN_FIX_CURSOR: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static SKIP_WIN_FIX_SCROLL: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static SKIP_UPDATE_TOPLINE: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static TABPAGE_MOVE_DISALLOWED: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
/// A float's anchor as the UI is told it, indexed by `FloatAnchor`.
pub(crate) static FLOAT_ANCHOR_STR: [&CStr; 4] = [c"NW", c"NE", c"SW", c"SE"];
