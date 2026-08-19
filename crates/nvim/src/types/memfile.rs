#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

// `bhdr_T` and `memfile_T` own heap state now, so they live with the code
// that maintains it.
pub use crate::memfile::{bhdr_T, memfile_T};

/// A block number. Non-negative ones are page numbers in the swap file;
/// negative ones name a block that has never been written.
pub type blocknr_T = int64_t;
pub use crate::memfile::MfDirty;
