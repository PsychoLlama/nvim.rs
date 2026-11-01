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

/// The link a spent block on the block pool's reuse list carries in its own
/// first bytes: the block below it on the list.
pub struct ConsumedBlk {
    pub prev: *mut ConsumedBlk,
}
