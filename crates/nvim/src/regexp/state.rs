//! What a regexp compile leaves for its caller.
//!
//! `rc_did_emsg` is the compiler's own "an error was already reported" flag,
//! and the `OPTION_MAGIC_*` triple is what a `\v`/`\V` prefix decided.
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
use crate::types::OptMagic;

pub(crate) const OPTION_MAGIC_OFF: OptMagic = 2;
pub(crate) const OPTION_MAGIC_ON: OptMagic = 1;
pub(crate) const OPTION_MAGIC_NOT_SET: OptMagic = 0;
pub(crate) static rc_did_emsg: GlobalCell<bool> = GlobalCell::new(false);
