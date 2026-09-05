//! What a regexp match passes to the next one.
//!
//! `\z(...)` submatches cross two regexp runs -- syntax highlighting
//! captures them in the start pattern and the end pattern reads them back --
//! so they travel in `re_extmatch_in`/`re_extmatch_out` rather than in the
//! match result, with `reg_do_extmatch` saying which direction is wanted.
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

use crate::global_cell::GlobalCell;
use crate::types::{OptMagic, RegExtMatch};
use core::ffi::c_int;

pub(crate) const OPTION_MAGIC_OFF: OptMagic = 2;
pub(crate) const OPTION_MAGIC_ON: OptMagic = 1;
pub(crate) const OPTION_MAGIC_NOT_SET: OptMagic = 0;
pub(crate) static rc_did_emsg: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static reg_do_extmatch: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static re_extmatch_in: GlobalCell<*mut RegExtMatch> =
    GlobalCell::new(::core::ptr::null_mut::<RegExtMatch>());
pub(crate) static re_extmatch_out: GlobalCell<*mut RegExtMatch> =
    GlobalCell::new(::core::ptr::null_mut::<RegExtMatch>());
