//! Where an option's value actually lives.
//!
//! Upstream declares these in `option_vars.h` as one `EXTERN` per option, and
//! the transpiler parked the lot in `startup/mod.rs` beside `main()`. An
//! option's value is one field of [`crate::options::vars::Options`] now --
//! one record behind one cell, generated from the same metadata as the table
//! row that names it -- and this module re-exports the accessors so that
//! `crate::option::vars::p_sh()` still means what `p_sh` meant.
//!
//! What is left of its own is the *decoded* half. `<abbrev>_flags` is the
//! parsed form of a string option whose value is a comma-separated set, kept
//! beside the string so a hot path tests a bit instead of parsing
//! ('backspace', 'clipboard', 'display', ...); `crate::optionstr` fills it in
//! from the `did_set_*` callback. Those are derived state, not an option's
//! value, so they are not rows of the record and they keep their own cells.
//!
//! The window- and buffer-local options do not live here either: a window or
//! buffer carries its own copy, and the record holds the global values.

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

pub use crate::options::vars::*;

use crate::global_cell::GlobalCell;
use crate::types::{BreakAt, OptInt, uint8_t};
use core::ffi::{c_char, c_uint};

pub(crate) static fenc_default: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());

pub(crate) static wim_flags: GlobalCell<[uint8_t; 4]> = GlobalCell::new([0; 4]);
pub(crate) static bkc_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static breakat_flags: GlobalCell<BreakAt> = GlobalCell::new(BreakAt::NONE);
pub(crate) static bo_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static cmp_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static cb_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static cia_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static cot_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static dy_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static fdo_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static jop_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static p_mousescroll_vert: GlobalCell<OptInt> = GlobalCell::new(3);
pub(crate) static p_mousescroll_hor: GlobalCell<OptInt> = GlobalCell::new(6);
pub(crate) static rdb_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static ssop_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static tpf_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static spo_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static swb_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static tcl_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static tc_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static vop_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static ve_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static wop_flags: GlobalCell<c_uint> = GlobalCell::new(0);
