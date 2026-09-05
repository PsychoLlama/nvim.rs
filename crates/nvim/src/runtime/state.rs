//! Which script the running code belongs to.
//!
//! `current_sctx` is what `<sfile>`, `:verbose` and every `sourcing` message
//! read, and what `guard::Script` saves and restores around anything that
//! runs code from elsewhere. The `SID_*` triple names the three contexts
//! that are not a file (a `-c` command, a `--cmd` command, an environment
//! variable), the `ETYPE_*` triple is the same distinction on the exception
//! stack, and `DOSO_*` says which of the startup scripts is being sourced.
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::global_cell::GlobalCell;
use crate::types::{EStackType, ScriptCtx};
use core::ffi::{c_int, c_uint};

pub(crate) const ETYPE_ENV: EStackType = 7;
pub(crate) const ETYPE_ARGS: EStackType = 6;
pub(crate) const ETYPE_TOP: EStackType = 0;
pub(crate) const DOSO_VIMRC: c_uint = 1;
pub(crate) const DOSO_NONE: c_uint = 0;
pub(crate) const SID_CMDARG: c_int = -2 as c_int;
pub(crate) const SID_CARG: c_int = -3 as c_int;
pub(crate) const SID_ENV: c_int = -4 as c_int;
pub(crate) static current_sctx: GlobalCell<ScriptCtx> = GlobalCell::new(ScriptCtx::NONE);
pub(crate) static did_source_packages: GlobalCell<bool> = GlobalCell::new(false);
