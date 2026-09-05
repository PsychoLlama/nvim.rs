//! The re-entrancy flags the window code sets around itself.
//!
//! Each one says "the caller is mid-operation on the window tree, so the
//! usual repair pass must not run": `skip_win_fix_cursor` and
//! `skip_win_fix_scroll` over a resize that will place the cursor itself,
//! `skip_update_topline` over a move that is not a scroll, and
//! `tabpage_move_disallowed` over a walk an autocommand must not reorder.
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
use core::ffi::{c_char, c_int};

pub(crate) static skip_win_fix_cursor: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static skip_win_fix_scroll: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static skip_update_topline: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static tabpage_move_disallowed: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static float_anchor_str: GlobalCell<[*const c_char; 4]> = GlobalCell::new([
    c"NW".as_ptr(),
    c"NE".as_ptr(),
    c"SW".as_ptr(),
    c"SE".as_ptr(),
]);
