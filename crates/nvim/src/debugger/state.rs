//! Whether the debugger has the editor stopped.
//!
//! `DEBUG_BREAK_LEVEL` is both the switch and the depth: above zero, every
//! sourced line offers a `>` prompt. `DEBUG_BACKTRACE_LEVEL` is where `:up`
//! and `:down` have moved within the stack, `DEBUG_TICK` invalidates the
//! cached breakpoint lookup when the set changes, and `DEBUG_MODE` says the
//! prompt itself is running so it does not recurse.
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

pub(crate) static DEBUG_BREAK_LEVEL: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub(crate) static DEBUG_DID_MSG: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static DEBUG_TICK: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static DEBUG_BACKTRACE_LEVEL: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static DEBUG_MODE: GlobalCell<bool> = GlobalCell::new(false);
