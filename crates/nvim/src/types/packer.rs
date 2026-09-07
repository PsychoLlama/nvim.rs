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
//
// The packer's buffer is the exception: building one is an unsafe step
// (three raw pointers that have to describe one live allocation), so the
// type lives next to the codec that establishes and trusts that invariant,
// and this module only names it.
pub use crate::msgpack_rpc::packer::{PackerBuffer, PackerBufferFlush};
