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

pub use crate::options::vars::*;

use crate::global_cell::GlobalCell;
use crate::memory::XString;
use crate::optionstr::BreakAt;
use crate::os::libc::uint8_t;
use crate::types::OptInt;
use core::ffi::c_uint;

/// The encoding a file with no 'fileencodings' match is read as; set once
/// at startup.
pub(crate) static FENC_DEFAULT: GlobalCell<Option<XString>> = GlobalCell::new(None);

pub(crate) static WIM_FLAGS: GlobalCell<[uint8_t; 4]> = GlobalCell::new([0; 4]);
pub(crate) static BKC_FLAGS: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static BREAKAT_FLAGS: GlobalCell<BreakAt> = GlobalCell::new(BreakAt::NONE);
pub(crate) static BO_FLAGS: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static CMP_FLAGS: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static CB_FLAGS: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static CIA_FLAGS: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static COT_FLAGS: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static DY_FLAGS: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static FDO_FLAGS: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static JOP_FLAGS: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static P_MOUSESCROLL_VERT: GlobalCell<OptInt> = GlobalCell::new(3);
pub(crate) static P_MOUSESCROLL_HOR: GlobalCell<OptInt> = GlobalCell::new(6);
pub(crate) static RDB_FLAGS: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static SSOP_FLAGS: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static TPF_FLAGS: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static SPO_FLAGS: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static SWB_FLAGS: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static TCL_FLAGS: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static TC_FLAGS: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static VOP_FLAGS: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static VE_FLAGS: GlobalCell<c_uint> = GlobalCell::new(0);
pub(crate) static WOP_FLAGS: GlobalCell<c_uint> = GlobalCell::new(0);

/// Whether `set_init_1` has installed the option defaults — the point from
/// which upstream's global string options are never `NULL` again.
///
/// Table-wide, and hand-kept rather than generated: it is a fact about
/// startup, not about any one row. [`crate::options::vars::StrOpt`] is what
/// reads it, to tell an option that owns nothing *yet* (upstream's `NULL`)
/// from one explicitly left owning nothing (its `empty_string_option`).
static OPTION_DEFAULTS_SET: GlobalCell<bool> = GlobalCell::new(false);

/// Whether the option defaults are in; see [`OPTION_DEFAULTS_SET`].
pub(crate) fn option_defaults_set() -> bool {
    OPTION_DEFAULTS_SET.get()
}

/// Record that the option defaults are in. `set_init_1` calls it once, right
/// after `set_options_default`.
pub(crate) fn mark_option_defaults_set() {
    OPTION_DEFAULTS_SET.set(true);
}
