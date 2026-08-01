#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

// `bhdr_T` and `memfile_T` own heap state now, so they live with the code
// that maintains it.
pub use crate::src::nvim::memfile::{bhdr_T, memfile_T};

/// A block number. Non-negative ones are page numbers in the swap file;
/// negative ones name a block that has never been written.
pub type blocknr_T = int64_t;
pub use crate::src::nvim::memfile::MfDirty;
/// Retained for the modules that still re-export the whole type namespace.
pub type mfdirty_T = ::core::ffi::c_uint;
