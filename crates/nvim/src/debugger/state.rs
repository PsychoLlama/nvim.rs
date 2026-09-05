//! Whether the debugger has the editor stopped.
//!
//! `debug_break_level` is both the switch and the depth: above zero, every
//! sourced line offers a `>` prompt. `debug_backtrace_level` is where `:up`
//! and `:down` have moved within the stack, `debug_tick` invalidates the
//! cached breakpoint lookup when the set changes, and `debug_mode` says the
//! prompt itself is running so it does not recurse.
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

pub(crate) static debug_break_level: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub(crate) static debug_did_msg: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static debug_tick: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static debug_backtrace_level: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static debug_mode: GlobalCell<bool> = GlobalCell::new(false);
