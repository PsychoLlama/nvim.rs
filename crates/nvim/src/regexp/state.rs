//! What a regexp compile leaves for its caller: the `OPTION_MAGIC_*` triple
//! a `\v`/`\V` prefix decided.
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::types::OptMagic;

pub(crate) const OPTION_MAGIC_OFF: OptMagic = 2;
pub(crate) const OPTION_MAGIC_ON: OptMagic = 1;
pub(crate) const OPTION_MAGIC_NOT_SET: OptMagic = 0;
