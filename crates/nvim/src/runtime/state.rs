//! Which script the running code belongs to.
//!
//! `CURRENT_SCTX` is what `<sfile>`, `:verbose` and every `sourcing` message
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
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::global_cell::GlobalCell;
use crate::types::ScriptCtx;

pub(crate) static CURRENT_SCTX: GlobalCell<ScriptCtx> = GlobalCell::new(ScriptCtx::NONE);
pub(crate) static DID_SOURCE_PACKAGES: GlobalCell<bool> = GlobalCell::new(false);
